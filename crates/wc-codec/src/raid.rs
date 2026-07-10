//! **P5 — cross-plate RAID layer** (`mk1` xpub arrays only): plan §3 (RAID
//! generator), §4.2 (H1 / array-id), §4.6, §7 P5; spec §7.
//!
//! Layer C ([`crate::rs`]) repairs a plate you still HAVE; this layer
//! reconstructs a plate you've LOST entirely. The `n` xpub payloads are striped
//! column-wise as `GF(2¹¹)` symbols and `r ∈ {1,2}` MDS parity stripes are added,
//! forming an `[n+r, n]` code that recovers **any `r` of the `n+r` plates**
//! (data OR parity).
//!
//! # Frozen RAID math (plan §3 — KAT-locked)
//!
//! - `P₁[c] = Σᵢ stripeᵢ[c]` (weights `α⁰ = 1` ⇒ field add = XOR; RAID-5).
//! - `P₂[c] = Σᵢ αⁱ · stripeᵢ[c]` where `i` = the data plate's `index-in-array`
//!   (`0..n−1`, the H1 field) and `α = field::ALPHA` (RAID-6). The distinct
//!   exponents over `0..n−1` make the parity columns a Vandermonde system, so
//!   **any `r` of `n+r`** erasures are recoverable.
//! - **`P₁` is byte-identical whether r=1 or r=2** (append-only across the RAID
//!   dimension — `P₂` is just an additional stripe).
//! - Surfaced `r ∈ {1,2}` (the construction admits r≥3; NOT surfaced).
//!
//! # Stripe format (self-describing length + array-wide padding)
//!
//! A reconstructed (entirely-missing) data plate has NO geometry of its own, so
//! the stripe must self-describe its true payload length (mk-codec rejects
//! trailing bytes). Each data plate's stripe, in the `GF(2¹¹)` symbol domain, is:
//!
//! ```text
//! [ len-prefix: 2 symbols = payload_bits(16) MSB-first ] [ payload symbols ] [ zero-pad to W ]
//! ```
//!
//! - `payload symbols` = the 8→11 regroup of the payload (`ceil(payload_bits/11)`
//!   symbols).
//! - `W` = the array-wide MAX over the length-prefixed stripe lengths
//!   (`2 + ceil(payload_bitsᵢ/11)`). Every stripe is zero-padded on the right to
//!   width `W`, so all `n` stripes (and the `r` parity stripes) are width `W`.
//! - RAID parity is computed over the width-`W` stripes (column-wise, in the
//!   symbol domain). **Each plate's Word-Card payload = its width-`W` stripe**
//!   (packed to bytes, `payload_bits = 11·W`), so the per-plate card is a full
//!   standalone Word-Card carrying the stripe.
//!
//! On reconstruct: RAID-solve the missing width-`W` stripe → read its length-
//! prefix → trim the payload symbols → 11→8 regroup → recover the EXACT
//! `(payload_bytes, payload_bits)`. The guarantee: `raid_reconstruct` recovers
//! each original `(payload_bytes, payload_bits)` exactly, including for plates
//! that were entirely missing.

use crate::field;
use crate::pipeline::{
    array_id_from_seed, encode_inner, RaidHeaderFields, RAID_ROLE_DATA, RAID_ROLE_PARITY_A,
    RAID_ROLE_PARITY_B,
};
use crate::regroup;
use crate::{decode, EncodeOpts, SourceKind, WcError};
use sha2::{Digest, Sha256};

/// A RAID plate's role in the array (plan §4.2 H1 role field). `Data` plates
/// carry the `n` xpub stripes; `ParityA` = `P₁` (RAID-5, r≥1); `ParityB` = `P₂`
/// (RAID-6, r=2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlateRole {
    /// A data plate carrying one xpub stripe.
    Data,
    /// The first recovery stripe `P₁` (XOR; RAID-5).
    ParityA,
    /// The second recovery stripe `P₂` (α-weighted; RAID-6).
    ParityB,
}

/// The maximum number of data plates in a RAID array (plan §3 / §4.2 — fits the
/// 5-bit `n−1` H1 field, `n−1 ≤ 31`).
const MAX_N: usize = 32;
/// The maximum surfaced recovery tier (plan §3 / §4.6 — r≥3 is admitted by the
/// construction but NOT surfaced).
const MAX_R: u8 = 2;

/// A single RAID plate produced by [`raid_encode`] — a full standalone Word-Card.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RaidPlate {
    /// The plate's role (Data / ParityA / ParityB).
    pub role: PlateRole,
    /// The plate's logical position in the `n+r` sequence: data plate `i` ⇒ `i`;
    /// ParityA ⇒ `n`; ParityB ⇒ `n+1`.
    pub index: usize,
    /// The engraved BIP-39 word sequence (a complete Word-Card for this stripe).
    pub words: Vec<&'static str>,
}

/// The result of a successful [`raid_reconstruct`] (plan §7 P5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RaidRecovery {
    /// The `n` recovered data payloads in index order: `(payload_bytes,
    /// payload_bits)` — EXACT, including any plates that were entirely missing.
    pub payloads: Vec<(Vec<u8>, usize)>,
    /// The data-plate indices that were reconstructed via the RAID solve (the
    /// plates that were missing from the input set), ascending.
    pub reconstructed: Vec<usize>,
}

// ===========================================================================
// Stripe construction (symbol domain).
// ===========================================================================

/// The number of length-prefix symbols (2 = 22 bits, carrying `payload_bits(16)`
/// MSB-first with a 6-bit zero tail).
const LEN_PREFIX_SYMBOLS: usize = 2;

/// Pack `payload_bits` (16 bits) into the 2 length-prefix symbols (MSB-first;
/// the low 6 bits of the 2nd symbol are zero).
fn len_prefix_symbols(payload_bits: usize) -> [u16; LEN_PREFIX_SYMBOLS] {
    // 16 bits MSB-first across 22 symbol bits: sym0 = bits[0..11], sym1 =
    // bits[11..16] << 6 (low 6 bits zero).
    let v = (payload_bits as u32) & 0xFFFF;
    let sym0 = ((v >> 5) & 0x07FF) as u16; // top 11 of the 16 bits
    let sym1 = (((v & 0x1F) as u16) << 6) & 0x07FF; // low 5 bits, left-justified
    [sym0, sym1]
}

/// Read the `payload_bits` back from the 2 length-prefix symbols.
fn read_len_prefix(sym0: u16, sym1: u16) -> usize {
    let hi = (sym0 & 0x07FF) as u32; // 11 bits
    let lo = ((sym1 >> 6) & 0x1F) as u32; // 5 bits
    ((hi << 5) | lo) as usize
}

/// Build the unpadded stripe for one data payload: `[len-prefix 2][payload
/// symbols]`. The caller zero-pads to the array-wide width `W`.
fn data_stripe_unpadded(payload: &[u8], payload_bits: usize) -> Vec<u16> {
    let mut s = Vec::with_capacity(LEN_PREFIX_SYMBOLS + payload_bits.div_ceil(11));
    s.extend_from_slice(&len_prefix_symbols(payload_bits));
    s.extend_from_slice(&regroup::bits_to_symbols(payload, payload_bits));
    s
}

/// Pack a width-`W` stripe (W symbols) into the per-plate Word-Card payload bytes
/// (`ceil(11·W / 8)` bytes; `payload_bits = 11·W`). Round-trips exactly via
/// [`unpack_stripe`].
fn pack_stripe(stripe: &[u16]) -> (Vec<u8>, usize) {
    let bits = stripe.len() * 11;
    // symbols_to_bits cannot fail here: all stripe symbols are 11-bit field
    // elements and total_bits == 11·len (no trailing pad to assert).
    let bytes = regroup::symbols_to_bits(stripe, bits).expect("stripe symbols are 11-bit");
    (bytes, bits)
}

/// Inverse of [`pack_stripe`]: recover the width-`W` stripe symbols from a plate's
/// recovered payload (`payload_bits = 11·W`).
fn unpack_stripe(payload: &[u8], payload_bits: usize) -> Vec<u16> {
    debug_assert_eq!(
        payload_bits % 11,
        0,
        "RAID stripe payload_bits is a multiple of 11"
    );
    regroup::bits_to_symbols(payload, payload_bits)
}

/// Recover the exact `(payload_bytes, payload_bits)` from a width-`W` stripe by
/// reading its length-prefix and trimming. Returns `RaidArrayMismatch` if the
/// length-prefix is structurally impossible for this stripe width (a corrupt /
/// foreign stripe).
fn stripe_to_payload(stripe: &[u16]) -> Result<(Vec<u8>, usize), WcError> {
    if stripe.len() < LEN_PREFIX_SYMBOLS {
        return Err(WcError::RaidArrayMismatch);
    }
    let payload_bits = read_len_prefix(stripe[0], stripe[1]);
    let payload_syms = payload_bits.div_ceil(11);
    let available = stripe.len() - LEN_PREFIX_SYMBOLS;
    if payload_syms > available {
        // The declared length does not fit the stripe — corrupt/foreign.
        return Err(WcError::RaidArrayMismatch);
    }
    let body = &stripe[LEN_PREFIX_SYMBOLS..LEN_PREFIX_SYMBOLS + payload_syms];
    let bytes = regroup::symbols_to_bits(body, payload_bits).map_err(WcError::from)?;
    Ok((bytes, payload_bits))
}

// ===========================================================================
// RAID parity (column-wise, over the width-W stripes).
// ===========================================================================

/// `P₁[c] = Σᵢ stripeᵢ[c]` (GF(2¹¹) add = XOR).
fn parity_a(stripes: &[Vec<u16>], width: usize) -> Vec<u16> {
    let mut p = vec![0u16; width];
    for s in stripes {
        for c in 0..width {
            p[c] = field::add(p[c], s[c]);
        }
    }
    p
}

/// `P₂[c] = Σᵢ αⁱ · stripeᵢ[c]` where `i` = the data plate's `index-in-array`
/// (0..n−1). `α = field::ALPHA`.
fn parity_b(stripes: &[Vec<u16>], width: usize) -> Vec<u16> {
    let mut p = vec![0u16; width];
    for (i, s) in stripes.iter().enumerate() {
        let ai = field::pow(field::ALPHA, i as u32); // αⁱ
        for c in 0..width {
            p[c] = field::add(p[c], field::mul(ai, s[c]));
        }
    }
    p
}

// ===========================================================================
// Array-id derivation (plan §3 / §4.2; F2 payload-digest fold).
// ===========================================================================

/// The **FROZEN, INJECTIVE** canonical serialization of an array's payloads for
/// the array-id digest (constellation-eval **F2** §1a — this layout is engraved
/// into steel via `array_id`, so it MUST NEVER change):
///
/// ```text
/// u32-BE n ‖ for each payload in index order:
///     u16-BE payload_bits ‖ exactly ceil(payload_bits/8) payload bytes
/// ```
///
/// The exact minimal byte count is load-bearing for INJECTIVITY (a non-minimal
/// `payload.len()` is rejected upstream in [`raid_encode`]); with a variable byte
/// count a naive `payload_bits ‖ bytes` serialization would be ambiguous. `bits`
/// fits `u16` (validated `≤ 0xFFFF` upstream).
fn array_id_canonical(payloads: &[(Vec<u8>, usize)]) -> Vec<u8> {
    let mut c = Vec::new();
    c.extend_from_slice(&(payloads.len() as u32).to_be_bytes());
    for (bytes, bits) in payloads {
        c.extend_from_slice(&(*bits as u16).to_be_bytes());
        let n_bytes = bits.div_ceil(8);
        c.extend_from_slice(&bytes[..n_bytes]);
    }
    c
}

/// Derive the 22-bit array-id (F2): `top22(SHA-256(seed ‖ SHA-256(canonical)))`
/// where `seed` is the ordered cosigner fingerprints and `canonical` is
/// [`array_id_canonical`]. Folding the payload digest in gives two DIFFERENT
/// wallets sharing a cosigner set DIFFERENT ids, so a cross-array plate mix is
/// caught by the reconstruct equality gate instead of silently emitting a wrong
/// xpub. **Excludes `r`** (so the append-only `P₁` stays byte-identical for r=1
/// vs r=2). `top22` reuses [`array_id_from_seed`] over `seed ‖ digest`.
fn derive_array_id(seed: &[u8], payloads: &[(Vec<u8>, usize)]) -> u32 {
    let digest = Sha256::digest(array_id_canonical(payloads));
    let mut input = Vec::with_capacity(seed.len() + digest.len());
    input.extend_from_slice(seed);
    input.extend_from_slice(&digest);
    array_id_from_seed(&input)
}

// ===========================================================================
// Public API — encode (plan §6.1 / §7 P5).
// ===========================================================================

/// Encode `n` xpub payloads into `n` data plates + `r` recovery plates (plan §3 /
/// §4.6 / §7 P5). `payloads[i] = (payload_bytes, payload_bits)`; `array_id_seed`
/// = the concatenated ordered cosigner fingerprints. The 22-bit array-id is
/// `top22(SHA-256(array_id_seed ‖ SHA-256(canonical)))` where `canonical` is the
/// injective payload serialization (see [`array_id_canonical`]) — folding the
/// payload digest into the id so two DIFFERENT wallets sharing a cosigner set get
/// DIFFERENT array-ids (constellation-eval **F2**: prevents a same-quorum plate
/// mix from silently reconstructing a valid-but-wrong xpub). `r ∈ {1,2}`,
/// `2 ≤ n ≤ 32`, `r < n`.
///
/// Each payload's byte count MUST be exactly minimal (`ceil(payload_bits/8)`) —
/// the frozen array-id digest layout depends on this for injectivity, so a
/// non-minimal `payload.len()` is rejected ([`WcError::InvalidParams`]).
///
/// Returns the plates in order: `n` data plates (index `0..n−1`), then ParityA
/// (if `r ≥ 1`), then ParityB (if `r = 2`). Each plate is a full standalone
/// Word-Card (`decode`-able) carrying its width-`W` stripe + RAID header.
pub fn raid_encode(
    payloads: &[(Vec<u8>, usize)],
    array_id_seed: &[u8],
    r: u8,
    opts: &EncodeOpts,
) -> Result<Vec<RaidPlate>, WcError> {
    let n = payloads.len();
    // --- Validate (plan §7 P5 KAT 11). ----------------------------------
    if !(2..=MAX_N).contains(&n) {
        return Err(WcError::InvalidParams);
    }
    if r == 0 || r > MAX_R {
        return Err(WcError::InvalidParams);
    }
    if (r as usize) >= n {
        return Err(WcError::InvalidParams);
    }
    for (bytes, bits) in payloads {
        if *bits > 0xFFFF {
            return Err(WcError::InvalidParams);
        }
        // M-C (F2): the payload byte count MUST be exactly minimal —
        // `ceil(payload_bits/8)`. The frozen array-id digest layout (§1a)
        // serializes exactly `ceil(bits/8)` bytes per payload; a non-minimal
        // `bytes.len()` (too few OR too many) would break the digest's
        // injectivity. Unreachable from the real mk1 caller (byte-aligned
        // minimal), but pinned here since the derivation freezes into engraved
        // steel. (Subsumes the old `bits > bytes.len()*8` lower-bound check.)
        if bytes.len() != bits.div_ceil(8) {
            return Err(WcError::InvalidParams);
        }
    }

    // (a, F2) Fold a deterministic digest of the payloads into the array-id so a
    // same-quorum different-payload array gets a DIFFERENT id (validated above:
    // each payload is minimal, so `array_id_canonical` is injective).
    let array_id = derive_array_id(array_id_seed, payloads);

    // --- Build the data stripes; derive the array-wide width W. ----------
    let unpadded: Vec<Vec<u16>> = payloads
        .iter()
        .map(|(bytes, bits)| data_stripe_unpadded(bytes, *bits))
        .collect();
    let width = unpadded.iter().map(|s| s.len()).max().unwrap_or(0);

    let stripes: Vec<Vec<u16>> = unpadded
        .into_iter()
        .map(|mut s| {
            s.resize(width, 0); // zero-pad on the right to W
            s
        })
        .collect();

    // --- Compute the parity stripes (P₁ always; P₂ iff r == 2). ---------
    let p1 = parity_a(&stripes, width);
    let p2 = if r == 2 {
        Some(parity_b(&stripes, width))
    } else {
        None
    };

    // --- Emit each plate as a standalone Word-Card. ----------------------
    let mut plates: Vec<RaidPlate> = Vec::with_capacity(n + r as usize);
    for (i, stripe) in stripes.iter().enumerate() {
        let words = encode_plate(stripe, n, RAID_ROLE_DATA, i, array_id, opts)?;
        plates.push(RaidPlate {
            role: PlateRole::Data,
            index: i,
            words,
        });
    }
    // ParityA (index n; wire index 0 placeholder — identity is the role).
    let wa = encode_plate(&p1, n, RAID_ROLE_PARITY_A, 0, array_id, opts)?;
    plates.push(RaidPlate {
        role: PlateRole::ParityA,
        index: n,
        words: wa,
    });
    if let Some(p2) = p2 {
        let wb = encode_plate(&p2, n, RAID_ROLE_PARITY_B, 0, array_id, opts)?;
        plates.push(RaidPlate {
            role: PlateRole::ParityB,
            index: n + 1,
            words: wb,
        });
    }
    Ok(plates)
}

/// Encode one width-`W` stripe into a standalone Word-Card with the given RAID
/// header (role / wire-index / array-id).
fn encode_plate(
    stripe: &[u16],
    n: usize,
    role: u16,
    wire_index: usize,
    array_id: u32,
    opts: &EncodeOpts,
) -> Result<Vec<&'static str>, WcError> {
    let (bytes, bits) = pack_stripe(stripe);
    let raid = RaidHeaderFields {
        n,
        role,
        index: wire_index,
        array_id,
    };
    encode_inner(SourceKind::Mk1Xpub, &bytes, bits, opts, Some(raid))
}

// ===========================================================================
// Public API — reconstruct (plan §6.1 / §7 P5).
// ===========================================================================

/// One decoded plate's RAID-relevant state.
struct DecodedPlate {
    role: u16,
    wire_index: usize, // the data plate's α-exponent (0..n−1); 0 for parity
    n: usize,
    array_id: u32,
    stripe: Vec<u16>,
}

/// Reconstruct an array from a set of present plates (plan §3 / §7 P5). Decodes
/// each plate (each self-heals typos via its own RS+tag), groups by array-id
/// (mismatched array-ids ⇒ [`WcError::RaidArrayMismatch`]), reconstructs `≤ r`
/// missing DATA plates via the MDS solve, and returns the `n` recovered
/// `(payload_bytes, payload_bits)` in index order plus which were reconstructed.
/// `> r` missing data plates ⇒ [`WcError::RaidUnrecoverable`] (refuse — never a
/// silent wrong reconstruction).
pub fn raid_reconstruct(plates: &[Vec<&str>]) -> Result<RaidRecovery, WcError> {
    if plates.is_empty() {
        return Err(WcError::RaidArrayMismatch);
    }

    // --- Decode every plate (self-healing per-plate via RS+tag). ---------
    let mut decoded: Vec<DecodedPlate> = Vec::with_capacity(plates.len());
    for words in plates {
        let d = decode(words)?;
        let meta = d.raid.ok_or(WcError::RaidArrayMismatch)?; // a solo card is not an array plate
        let role = match meta.role {
            PlateRole::Data => RAID_ROLE_DATA,
            PlateRole::ParityA => RAID_ROLE_PARITY_A,
            PlateRole::ParityB => RAID_ROLE_PARITY_B,
        };
        // The wire α-exponent for a data plate is its public index; parity plates
        // are exponent-irrelevant (their identity is the role).
        let wire_index = if role == RAID_ROLE_DATA {
            meta.index
        } else {
            0
        };
        let stripe = unpack_stripe(&d.payload, d.payload_bits);
        decoded.push(DecodedPlate {
            role,
            wire_index,
            n: meta.n,
            array_id: meta.array_id,
            stripe,
        });
    }

    // --- Group by array-id; require a single coherent array. -------------
    let array_id = decoded[0].array_id;
    let n = decoded[0].n;
    let width = decoded[0].stripe.len();
    for d in &decoded {
        if d.array_id != array_id || d.n != n || d.stripe.len() != width {
            // Plates from two different arrays / inconsistent geometry — refuse
            // rather than silently mix (plan §4.2 / §7 P5 KAT 6).
            return Err(WcError::RaidArrayMismatch);
        }
    }

    // --- Index the present plates by role / exponent. --------------------
    // present_data[i] = Some(stripe) for each present data plate i (0..n−1).
    let mut present_data: Vec<Option<Vec<u16>>> = vec![None; n];
    let mut p1: Option<Vec<u16>> = None;
    let mut p2: Option<Vec<u16>> = None;
    for d in &decoded {
        match d.role {
            RAID_ROLE_DATA => {
                if d.wire_index >= n {
                    return Err(WcError::RaidArrayMismatch);
                }
                if present_data[d.wire_index].is_some() {
                    // Duplicate data plate — incoherent set.
                    return Err(WcError::RaidArrayMismatch);
                }
                present_data[d.wire_index] = Some(d.stripe.clone());
            }
            RAID_ROLE_PARITY_A => {
                if p1.is_some() {
                    return Err(WcError::RaidArrayMismatch);
                }
                p1 = Some(d.stripe.clone());
            }
            RAID_ROLE_PARITY_B => {
                if p2.is_some() {
                    return Err(WcError::RaidArrayMismatch);
                }
                p2 = Some(d.stripe.clone());
            }
            _ => return Err(WcError::RaidArrayMismatch),
        }
    }

    // The surfaced recovery tier r = how many parity stripes are present in the
    // array's design. We infer it from the parity plates we actually hold AND
    // bound the solve by them: r_available = (p1? 1 : 0) + (p2? 1 : 0).
    let missing: Vec<usize> = (0..n).filter(|&i| present_data[i].is_none()).collect();
    let r_available = p1.is_some() as usize + p2.is_some() as usize;

    if missing.len() > r_available {
        // More missing data plates than parity stripes we can use ⇒ the MDS solve
        // is underdetermined. Refuse (plan §7 P5 KAT 8).
        return Err(WcError::RaidUnrecoverable);
    }

    // --- Solve the ≤ r missing data stripes. -----------------------------
    let recovered_stripes =
        solve_missing(&present_data, &missing, p1.as_deref(), p2.as_deref(), width)?;

    // --- Assemble the full set of n data stripes (present ∪ solved). ------
    let mut full: Vec<Vec<u16>> = Vec::with_capacity(n);
    for (i, slot) in present_data.iter().enumerate() {
        let stripe = match slot {
            Some(s) => s.clone(),
            None => recovered_stripes
                .get(&i)
                .cloned()
                .ok_or(WcError::RaidUnrecoverable)?,
        };
        full.push(stripe);
    }

    // --- (b, F2) Spare-parity consistency oracle. ------------------------
    // When we hold MORE parity plates than we had missing data plates, at least
    // one parity equation was NOT consumed by the MDS solve. Re-derive each
    // present parity stripe over the FULL reconstructed set and require it to
    // match the engraved parity. The equations the solve consumed hold by
    // construction (exact GF arithmetic), so this re-check can ONLY fail on a
    // genuine inconsistency: a same-quorum chimera whose plates carry equal
    // array-id / n / width (so they pass the coarse equality gate) but come from
    // DIFFERENT wallets sharing a cosigner set (the legacy F2 collision, for
    // plates engraved before the (a) payload-digest fold). A genuine array ALWAYS
    // passes — never over-rejected (G3). Accepted residual (SPEC §1b): a 0-missing
    // pure-data chimera with NO parity plate presented has no equation to check
    // and is info-theoretically undetectable in-band.
    if r_available > missing.len() {
        if let Some(p1) = p1.as_deref() {
            if parity_a(&full, width).as_slice() != p1 {
                return Err(WcError::RaidArrayMismatch);
            }
        }
        if let Some(p2) = p2.as_deref() {
            if parity_b(&full, width).as_slice() != p2 {
                return Err(WcError::RaidArrayMismatch);
            }
        }
    }

    // --- Read each data stripe's length-prefix → exact payload. ----------
    let mut payloads: Vec<(Vec<u8>, usize)> = Vec::with_capacity(n);
    for stripe in &full {
        payloads.push(stripe_to_payload(stripe)?);
    }

    Ok(RaidRecovery {
        payloads,
        reconstructed: missing,
    })
}

/// Solve the missing data stripes from the present data + parity stripes via the
/// `[n+r, n]` MDS (Vandermonde) system, column-by-column (plan §3 reconstruction
/// math). `0`, `1` or `2` unknowns.
///
/// For each missing data plate `j` (exponent `αʲ`):
/// - **1 unknown:** prefer `P₁` (`xⱼ = P₁ − Σ_present xᵢ`); else use `P₂`
///   (`xⱼ = α⁻ʲ (P₂ − Σ_present αⁱ xᵢ)`).
/// - **2 unknowns `j,k`:** solve `xⱼ + xₖ = s₁` (from `P₁`) and
///   `αʲ xⱼ + αᵏ xₖ = s₂` (from `P₂`); determinant `αʲ − αᵏ ≠ 0` (distinct
///   exponents) ⇒ unique solution.
fn solve_missing(
    present: &[Option<Vec<u16>>],
    missing: &[usize],
    p1: Option<&[u16]>,
    p2: Option<&[u16]>,
    width: usize,
) -> Result<std::collections::BTreeMap<usize, Vec<u16>>, WcError> {
    let mut out: std::collections::BTreeMap<usize, Vec<u16>> = std::collections::BTreeMap::new();
    if missing.is_empty() {
        return Ok(out);
    }

    match missing.len() {
        1 => {
            let j = missing[0];
            let mut xj = vec![0u16; width];
            if let Some(p1) = p1 {
                // xⱼ = P₁ − Σ_present xᵢ  (subtract = add in GF(2ᵐ)).
                for c in 0..width {
                    let mut acc = p1[c];
                    for (i, s) in present.iter().enumerate() {
                        if i == j {
                            continue;
                        }
                        if let Some(s) = s {
                            acc = field::add(acc, s[c]);
                        }
                    }
                    xj[c] = acc;
                }
            } else if let Some(p2) = p2 {
                // αʲ xⱼ = P₂ − Σ_present αⁱ xᵢ  ⇒  xⱼ = α⁻ʲ (…).
                let aj = field::pow(field::ALPHA, j as u32);
                let aj_inv = field::inv(aj).ok_or(WcError::RaidUnrecoverable)?;
                for c in 0..width {
                    let mut acc = p2[c];
                    for (i, s) in present.iter().enumerate() {
                        if i == j {
                            continue;
                        }
                        if let Some(s) = s {
                            let ai = field::pow(field::ALPHA, i as u32);
                            acc = field::add(acc, field::mul(ai, s[c]));
                        }
                    }
                    xj[c] = field::mul(aj_inv, acc);
                }
            } else {
                return Err(WcError::RaidUnrecoverable);
            }
            out.insert(j, xj);
        }
        2 => {
            let (p1, p2) = match (p1, p2) {
                (Some(a), Some(b)) => (a, b),
                _ => return Err(WcError::RaidUnrecoverable), // need both stripes for 2 unknowns
            };
            let j = missing[0];
            let k = missing[1];
            let aj = field::pow(field::ALPHA, j as u32);
            let ak = field::pow(field::ALPHA, k as u32);
            // det = αʲ − αᵏ (= αʲ + αᵏ in char-2); nonzero for distinct exponents.
            let det = field::add(aj, ak);
            let det_inv = field::inv(det).ok_or(WcError::RaidUnrecoverable)?;

            let mut xj = vec![0u16; width];
            let mut xk = vec![0u16; width];
            for c in 0..width {
                // s₁ = P₁ − Σ_present xᵢ ; s₂ = P₂ − Σ_present αⁱ xᵢ.
                let mut s1 = p1[c];
                let mut s2 = p2[c];
                for (i, s) in present.iter().enumerate() {
                    if i == j || i == k {
                        continue;
                    }
                    if let Some(s) = s {
                        s1 = field::add(s1, s[c]);
                        let ai = field::pow(field::ALPHA, i as u32);
                        s2 = field::add(s2, field::mul(ai, s[c]));
                    }
                }
                // xⱼ + xₖ = s₁ ; αʲxⱼ + αᵏxₖ = s₂.
                // xⱼ = (αᵏ s₁ + s₂) / (αʲ + αᵏ) ; xₖ = s₁ + xⱼ.
                let num_j = field::add(field::mul(ak, s1), s2);
                let xjc = field::mul(num_j, det_inv);
                let xkc = field::add(s1, xjc);
                xj[c] = xjc;
                xk[c] = xkc;
            }
            out.insert(j, xj);
            out.insert(k, xk);
        }
        _ => return Err(WcError::RaidUnrecoverable),
    }
    Ok(out)
}
