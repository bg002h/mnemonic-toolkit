//! Cross-implementation BCH parity — md-codec's correcting decoder vs an
//! in-tree, algorithmically **independent Peterson–Gorenstein–Zierler (PGZ)**
//! reference decoder.
//!
//! ## Why this file exists / what changed
//!
//! The previous `parity_smoke` cross-checked md-codec against a *toolkit binary*
//! and **silently self-skipped** (returned green) whenever that binary was
//! absent — so on CI, where the binary is never present, the only
//! cross-implementation BCH check never actually ran. This replacement drops the
//! external-binary dependency entirely and runs an ALWAYS-ON differential
//! against a reference decoder that lives right here in the test.
//!
//! A brute-force nearest-codeword search is infeasible (≈ 2.7e12 residue checks
//! per word), so the reference is a genuine syndrome decoder — but built by a
//! deliberately DIFFERENT route from md-codec at every stage, so a shared bug is
//! implausible:
//!
//! | stage           | md-codec (`bch_decode.rs`)      | this reference (PGZ)               |
//! |-----------------|---------------------------------|------------------------------------|
//! | syndromes       | unpack polymod residue, Horner  | **direct** `Σ cᵢ·(βʲ)ⁱ` over the word |
//! | error locator   | Berlekamp–Massey                | **Gaussian elimination**           |
//! | root finding    | Chien search                    | **direct trial** over the 93-orbit |
//! | magnitudes      | Forney's algorithm              | **linear system** (Gaussian)       |
//! | field mul/inv   | direct `(lo,hi)` `ζ`-formula    | **own exp/log tables**             |
//!
//! ### Independence attestation
//!
//! * **Zero imports from `bch_decode.rs`.** The PGZ module below imports nothing
//!   from md-codec's decoder; it uses only the *encoder*/*polymod* wire
//!   primitives (`bch::…`) to build test inputs and to observe md-codec's own
//!   decode result (`decode_regular_errors`, `decode_with_correction`) as the
//!   thing under comparison.
//! * The PGZ **syndromes are computed directly from the received word** — a
//!   `GF(1024)` Horner evaluation of the codeword polynomial `P(x) =
//!   INIT·xᴺ + Σ uᵢ·x^{N-1-i} + T(x)` at `β^{77+m}` — NOT by unpacking
//!   md-codec's iterative `GF(32)` polymod residue.
//! * Field arithmetic is over `GF(1024) = GF(32²)` with `ζ² = ζ + 1` (the field
//!   β and the syndromes live in), driven by **independently generated exp/log
//!   tables** (a primitive element is found by search at test start).
//! * The only values shared with md-codec are code-**definition** wire
//!   constants, re-declared locally (never imported from the decoder): the
//!   `GF(32)` reduction poly (`α⁵=α³+1`), the `GF(32²)` extension `ζ²=ζ+1`,
//!   `β`, the root window start `j = 77`, `n ≤ 93`, `MD_REGULAR_CONST`, and the
//!   codex32 wire constants `POLYMOD_INIT` + `hrp_expand("md")` (these last two
//!   are part of the code's public definition, exactly like `MD_REGULAR_CONST`).
//!
//! ## What is pinned
//!
//! `≤ t` correction is already exhaustively pinned by `bch_exhaustive_sweep.rs`;
//! the reference's marginal value is on **beyond-t** words, where md-codec's
//! post-correction re-verify (`chunk.rs`) makes its accept-set exactly "within
//! Hamming distance ≤ 4 of a codeword". A correct PGZ decoder (which also
//! re-verifies) has the *same* accept-set, so the two agree deterministically on
//! master — both on accept/reject AND on the miscorrected codeword.
//!
//! RED-under-mutation (verified during authoring): an off-by-one in md-codec's
//! syndrome window (`compute_syndromes_regular`, e.g. `β^{j_start}` →
//! `β^{j_start+1}` while Forney's shift is unchanged) flips every mined
//! miscorrection from accept to reject on md's side while the independent PGZ
//! still accepts → deterministic disagreement → RED.

use md_codec::bch::{MD_REGULAR_CONST, bch_create_checksum_regular, hrp_expand, polymod_run};
use md_codec::bch_decode::decode_regular_errors;
use md_codec::decode_with_correction;
use md_codec::error::Error;

// ===========================================================================
// Independent PGZ reference decoder over GF(1024) = GF(32²), ζ² = ζ + 1.
// FIREWALL: nothing in this module derives from `bch_decode.rs`.
// ===========================================================================
mod pgz {
    /// `GF(32) = GF(2)[α]/(α⁵+α³+1)` reduction mask (`0b0_1001`).
    const GF32_REDUCE: u8 = 0x09;
    /// `β = 8·ζ`, packed `lo | hi<<5` → `hi = 8, lo = 0`. Order 93.
    pub const BETA: u16 = 8 << 5;
    /// First exponent of the regular code's 8-consecutive-root window.
    pub const J_START: u32 = 77;
    /// MD-domain target residue (13 `GF(32)` symbols, LSB = x⁰).
    const MD_CONST: u128 = 0x0815c07747a3392e7;
    /// codex32 `ms32_polymod` initial residue (BIP-93).
    const POLYMOD_INIT: u128 = 0x23181b3;

    /// Carry-less `GF(32)` multiply with `α⁵ ≡ α³+1` reduction (own impl).
    fn gf32_mul(a: u8, b: u8) -> u8 {
        let mut result = 0u8;
        let mut a = a;
        for i in 0..5 {
            if (b >> i) & 1 != 0 {
                result ^= a;
            }
            let carry = (a >> 4) & 1;
            a = (a << 1) & 0x1F;
            if carry != 0 {
                a ^= GF32_REDUCE;
            }
        }
        result
    }

    /// `GF(1024)` multiply via `ζ² = ζ + 1` (bootstrap for the exp/log tables).
    fn mul_raw(x: u16, y: u16) -> u16 {
        let (alo, ahi) = ((x & 0x1F) as u8, ((x >> 5) & 0x1F) as u8);
        let (blo, bhi) = ((y & 0x1F) as u8, ((y >> 5) & 0x1F) as u8);
        let ll = gf32_mul(alo, blo);
        let lh = gf32_mul(alo, bhi);
        let hl = gf32_mul(ahi, blo);
        let hh = gf32_mul(ahi, bhi);
        let lo = ll ^ hh;
        let hi = lh ^ hl ^ hh;
        (lo as u16) | ((hi as u16) << 5)
    }

    /// Independently generated `GF(1024)*` exp/log tables (order 1023).
    pub struct Field {
        exp: Vec<u16>, // exp[i] = g^i
        log: Vec<u16>, // log[e] = i (log[0] unused)
    }

    impl Field {
        /// Build tables from a primitive element found by exhaustive order test.
        pub fn build() -> Field {
            let mut g = 0u16;
            for c in 2u16..1024 {
                let mut x = c;
                let mut ord = 1u32;
                while x != 1 && ord <= 1023 {
                    x = mul_raw(x, c);
                    ord += 1;
                }
                if ord == 1023 {
                    g = c;
                    break;
                }
            }
            assert_ne!(g, 0, "no primitive element found in GF(1024)");
            let mut exp = vec![0u16; 1023];
            let mut log = vec![0u16; 1024];
            let mut cur = 1u16;
            for (i, slot) in exp.iter_mut().enumerate() {
                *slot = cur;
                log[cur as usize] = i as u16;
                cur = mul_raw(cur, g);
            }
            assert_eq!(cur, 1, "g^1023 must return to 1");
            Field { exp, log }
        }

        pub fn mul(&self, a: u16, b: u16) -> u16 {
            if a == 0 || b == 0 {
                return 0;
            }
            let e = (self.log[a as usize] as u32 + self.log[b as usize] as u32) % 1023;
            self.exp[e as usize]
        }

        pub fn inv(&self, a: u16) -> u16 {
            assert_ne!(a, 0, "inverse of zero");
            self.exp[((1023 - self.log[a as usize] as u32) % 1023) as usize]
        }

        pub fn pow(&self, a: u16, e: u32) -> u16 {
            if e == 0 {
                return 1;
            }
            if a == 0 {
                return 0;
            }
            let idx = (self.log[a as usize] as u64 * e as u64 % 1023) as usize;
            self.exp[idx]
        }
    }

    /// `hrp_expand("md")` — independent re-implementation (BIP-173).
    fn hrp_expand_md() -> Vec<u8> {
        let mut out = Vec::with_capacity(5);
        for &c in b"md" {
            out.push(c >> 5);
        }
        out.push(0);
        for &c in b"md" {
            out.push(c & 31);
        }
        out
    }

    /// Evaluate `Σ_m coeffs[m]·y^m` where `coeffs[m]` is the 5-bit symbol at
    /// bits `5m..5m+5` of `packed` (LSB = x⁰). Horner high→low.
    fn eval_symbol_poly(f: &Field, packed: u128, nsym: usize, y: u16) -> u16 {
        let mut acc = 0u16;
        for m in (0..nsym).rev() {
            let c = ((packed >> (5 * m)) & 0x1F) as u16;
            acc = f.mul(acc, y) ^ c;
        }
        acc
    }

    /// Syndromes `S_m = E(β^{J_START+m})`, m = 0..8, computed DIRECTLY from the
    /// received `data_with_checksum` word as `P(β^j)` — no md polymod residue.
    pub fn syndromes(f: &Field, dwc: &[u8]) -> [u16; 8] {
        let mut u = hrp_expand_md();
        u.extend_from_slice(dwc);
        let n = u.len() as u32;
        let mut s = [0u16; 8];
        for (m, slot) in s.iter_mut().enumerate() {
            let y = f.pow(BETA, J_START + m as u32);
            // Σ uᵢ·y^{N-1-i}
            let mut su = 0u16;
            for &ui in &u {
                su = f.mul(su, y) ^ (ui as u16);
            }
            let init_term = f.mul(eval_symbol_poly(f, POLYMOD_INIT, 13, y), f.pow(y, n));
            let t_val = eval_symbol_poly(f, MD_CONST, 13, y);
            *slot = su ^ init_term ^ t_val;
        }
        s
    }

    /// Solve `A·x = b` over `GF(1024)` by Gaussian elimination; `None` if `A` is
    /// singular. `a` is `n × n`, `b` is length `n`.
    fn gauss(f: &Field, a: &mut [Vec<u16>], b: &mut [u16]) -> Option<Vec<u16>> {
        let n = b.len();
        for col in 0..n {
            let piv = (col..n).find(|&r| a[r][col] != 0)?;
            a.swap(col, piv);
            b.swap(col, piv);
            let inv = f.inv(a[col][col]);
            for c in col..n {
                a[col][c] = f.mul(a[col][c], inv);
            }
            b[col] = f.mul(b[col], inv);
            for r in 0..n {
                if r != col && a[r][col] != 0 {
                    let factor = a[r][col];
                    for c in col..n {
                        let t = f.mul(factor, a[col][c]);
                        a[r][c] ^= t;
                    }
                    b[r] ^= f.mul(factor, b[col]);
                }
            }
        }
        Some(b.to_vec())
    }

    /// Decode `dwc` with PGZ + a post-correction re-verify. Returns
    /// `Some((positions, magnitudes))` for an accepted correction (positions
    /// ascending), or `None` for reject (> t = 4 from every codeword, or an
    /// inconsistent locator/magnitude).
    pub fn decode(f: &Field, dwc: &[u8]) -> Option<(Vec<usize>, Vec<u8>)> {
        let l = dwc.len();
        let s = syndromes(f, dwc);
        if s.iter().all(|&x| x == 0) {
            return Some((Vec::new(), Vec::new()));
        }
        // Locate: largest ν in 1..=4 whose ν×ν syndrome matrix is non-singular.
        let mut sigma: Option<Vec<u16>> = None; // σ_1..σ_ν
        let mut nu = 0usize;
        for cand in (1..=4usize).rev() {
            let mut a: Vec<Vec<u16>> = (0..cand)
                .map(|i| (0..cand).map(|j| s[i + j]).collect())
                .collect();
            let mut b: Vec<u16> = (0..cand).map(|i| s[cand + i]).collect();
            if let Some(sol) = gauss(f, &mut a, &mut b) {
                // sol[j] is the coefficient of σ_{ν-j}; re-index to σ_1..σ_ν.
                let mut sig = vec![0u16; cand];
                for (j, &xj) in sol.iter().enumerate() {
                    sig[cand - 1 - j] = xj;
                }
                sigma = Some(sig);
                nu = cand;
                break;
            }
        }
        let sig = sigma?;
        // Roots by direct trial over the 93-orbit: Λ(β^{-d}) = 0 ⇒ error at d.
        let beta_inv = f.inv(BETA);
        let mut xpow = 1u16; // (β^{-1})^d
        let mut degs = Vec::new();
        for d in 0..l {
            let mut val = 1u16;
            let mut xp = xpow;
            for &st in &sig {
                val ^= f.mul(st, xp);
                xp = f.mul(xp, xpow);
            }
            if val == 0 {
                degs.push(d);
            }
            xpow = f.mul(xpow, beta_inv);
        }
        if degs.len() != nu {
            return None;
        }
        // Magnitudes: Σ_k e_k · X_k^{J_START+m} = S_m, X_k = β^{d_k}.
        let xk: Vec<u16> = degs.iter().map(|&d| f.pow(BETA, d as u32)).collect();
        let mut a: Vec<Vec<u16>> = (0..nu)
            .map(|m| xk.iter().map(|&x| f.pow(x, J_START + m as u32)).collect())
            .collect();
        let mut b: Vec<u16> = (0..nu).map(|m| s[m]).collect();
        let mags = gauss(f, &mut a, &mut b)?;
        let mut out = Vec::with_capacity(nu);
        for &mg in &mags {
            // A real symbol error lies in GF(32): high half zero, low half nonzero.
            if (mg >> 5) != 0 || (mg & 0x1F) == 0 {
                return None;
            }
            out.push((mg & 0x1F) as u8);
        }
        // degree d ⇒ data_with_checksum index k = L-1-d; sort ascending.
        let mut pairs: Vec<(usize, u8)> = degs
            .iter()
            .zip(&out)
            .map(|(&d, &m)| (l - 1 - d, m))
            .collect();
        pairs.sort_by_key(|p| p.0);
        // Re-verify: apply and require zero syndromes (accept-set = "≤4 of a codeword").
        let mut corrected = dwc.to_vec();
        for &(p, m) in &pairs {
            corrected[p] ^= m;
        }
        if syndromes(f, &corrected).iter().any(|&x| x != 0) {
            return None;
        }
        Some((
            pairs.iter().map(|p| p.0).collect(),
            pairs.iter().map(|p| p.1).collect(),
        ))
    }

    /// Apply a PGZ correction `(positions, magnitudes)` to `dwc`.
    pub fn apply(dwc: &[u8], correction: &(Vec<usize>, Vec<u8>)) -> Vec<u8> {
        let mut out = dwc.to_vec();
        for (&p, &m) in correction.0.iter().zip(&correction.1) {
            out[p] ^= m;
        }
        out
    }
}

// ===========================================================================
// md-codec side + shared test scaffolding.
// ===========================================================================

const CODEX32_ALPHABET: &[u8; 32] = b"qpzry9x8gf2tvdw0s3jn54khce6mua7l";
const DATA_SYMBOLS: usize = 80;

/// Fixed 80-symbol data pattern the mined miscorrection cells were derived
/// against (identical to `bch_exhaustive_sweep::base_data`).
fn base_data() -> Vec<u8> {
    (0..DATA_SYMBOLS as u8)
        .map(|i| i.wrapping_mul(7).wrapping_add(3) & 0x1F)
        .collect()
}

fn valid_codeword(data: &[u8]) -> Vec<u8> {
    let checksum = bch_create_checksum_regular("md", data);
    let mut cw = data.to_vec();
    cw.extend_from_slice(&checksum);
    cw
}

fn residue_of(dwc: &[u8]) -> u128 {
    let mut input = hrp_expand("md");
    input.extend_from_slice(dwc);
    polymod_run(&input) ^ MD_REGULAR_CONST
}

/// md-codec's BCH-layer verdict, replicated from `chunk::decode_with_correction`
/// (`decode_regular_errors` + the defensive post-correction re-verify): the
/// repaired word for an accepted correction, or `None` for reject.
fn md_bch(dwc: &[u8]) -> Option<Vec<u8>> {
    let residue = residue_of(dwc);
    if residue == 0 {
        return Some(dwc.to_vec());
    }
    let (pos, mag) = decode_regular_errors(residue, dwc.len())?;
    let mut fixed = dwc.to_vec();
    for (&p, &m) in pos.iter().zip(&mag) {
        if p >= fixed.len() {
            return None;
        }
        fixed[p] ^= m;
    }
    if residue_of(&fixed) != 0 {
        return None;
    }
    Some(fixed)
}

fn encode_md1(dwc: &[u8]) -> String {
    let mut s = String::from("md1");
    for &v in dwc {
        s.push(CODEX32_ALPHABET[(v & 0x1F) as usize] as char);
    }
    s
}

/// Whether md-codec's public entry point rejects `dwc` at the BCH layer
/// (`Err(TooManyErrors)`). For a BCH-accepted word the public decode returns
/// `Ok` or a *non*-`TooManyErrors` `Err` (a downstream reassembly error for a
/// synthetic non-descriptor codeword) — never `TooManyErrors`.
fn public_decode_rejects(dwc: &[u8]) -> bool {
    let s = encode_md1(dwc);
    matches!(
        decode_with_correction(&[s.as_str()]),
        Err(Error::TooManyErrors { .. })
    )
}

struct Rng(u64);
impl Rng {
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }
    fn symbol(&mut self) -> u8 {
        (self.next_u64() as u8) & 0x1F
    }
    fn magnitude(&mut self) -> u8 {
        ((self.next_u64() as u8) & 0x1F).max(1)
    }
}

// ---------------------------------------------------------------------------
// Cell 1 — the reference decoder is itself correct on all single errors.
//
// Validates the PGZ oracle against the injected pattern (ground truth) so an
// agreement failure below implicates md-codec, not a broken reference.
// ---------------------------------------------------------------------------

#[test]
fn pgz_reference_decodes_all_single_errors() {
    let f = pgz::Field::build();
    let original = valid_codeword(&base_data());
    assert_eq!(
        pgz::syndromes(&f, &original),
        [0u16; 8],
        "a valid codeword must have zero syndromes"
    );
    for pos in 0..original.len() {
        for mag in 1u8..32 {
            let mut w = original.clone();
            w[pos] ^= mag;
            let c = pgz::decode(&f, &w)
                .unwrap_or_else(|| panic!("PGZ must decode 1-error at pos {pos} mag {mag:05b}"));
            assert_eq!(c.0, vec![pos], "PGZ position");
            assert_eq!(c.1, vec![mag], "PGZ magnitude");
            assert_eq!(
                pgz::apply(&w, &c),
                original,
                "PGZ recovers the injected original"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Cell 2 — beyond-t agreement (seeded random e≥5 samples).
//
// The overwhelming majority of e≥5 words are > 4 from every codeword: both
// decoders reject. This pins that md-codec and the independent PGZ decoder
// agree on the accept/reject boundary (and, on the rare accept, the repaired
// codeword), AND that md-codec's PUBLIC entry point (`decode_with_correction`,
// chunk.rs) rejects exactly when the BCH layer does.
// ---------------------------------------------------------------------------

#[test]
fn pgz_agrees_with_md_beyond_t() {
    let f = pgz::Field::build();
    let mut rng = Rng(0x00DD_BA11_FACE_C0DE);
    let trials = 4000u32;
    let mut rejects = 0u32;
    let mut accepts = 0u32;
    for _ in 0..trials {
        let data: Vec<u8> = (0..DATA_SYMBOLS).map(|_| rng.symbol()).collect();
        let cw = valid_codeword(&data);
        let l = cw.len();
        let e = 5 + rng.below(4); // e ∈ 5..=8
        let mut positions = std::collections::BTreeSet::new();
        while positions.len() < e {
            positions.insert(rng.below(l));
        }
        let mut w = cw.clone();
        for &p in &positions {
            w[p] ^= rng.magnitude();
        }

        let md = md_bch(&w);
        let pg = pgz::decode(&f, &w);
        assert_eq!(
            md.is_some(),
            pg.is_some(),
            "md-codec and PGZ disagree on accept/reject for {w:?}"
        );
        if let (Some(mc), Some(pc)) = (&md, &pg) {
            assert_eq!(
                mc,
                &pgz::apply(&w, pc),
                "md-codec and PGZ accepted but produced different corrected codewords"
            );
            accepts += 1;
        } else {
            rejects += 1;
        }
        // Public entry point (chunk.rs:decode_with_correction): TooManyErrors
        // iff the BCH layer rejected.
        assert_eq!(
            public_decode_rejects(&w),
            md.is_none(),
            "decode_with_correction's TooManyErrors must track the BCH-layer verdict"
        );
    }
    // The random loop is dominated by mutual rejects; that's expected. The
    // accept branch is exercised deterministically by the mined cells below.
    assert!(rejects > 0, "expected some beyond-t rejects");
    let _ = accepts;
}

// ---------------------------------------------------------------------------
// Cell 3 — mined beyond-t MISCORRECTIONS: md-codec accepts, and PGZ must accept
// the SAME miscorrected codeword.
//
// These e=5 patterns (mined against `base_data()`'s codeword) land within
// Hamming distance ≤ 4 of a DIFFERENT codeword, so md-codec's decoder repairs
// to it (a genuine, funds-relevant miscorrection). The independent PGZ decoder
// must agree exactly. This cell is the RED carrier: an off-by-one in md-codec's
// syndrome window flips md to reject while PGZ (independent) still accepts.
// ---------------------------------------------------------------------------

const MINED_MISCORRECTIONS: &[(&[usize], &[u8])] = &[
    (&[17, 20, 44, 47, 86], &[7, 21, 17, 11, 12]),
    (&[25, 55, 57, 70, 79], &[24, 5, 20, 13, 9]),
    (&[28, 31, 40, 49, 76], &[22, 18, 13, 22, 6]),
    (&[7, 52, 70, 73, 82], &[29, 19, 22, 22, 24]),
];

#[test]
fn pgz_agrees_with_md_on_mined_miscorrections() {
    let f = pgz::Field::build();
    let cw = valid_codeword(&base_data());
    for (i, (pos, mag)) in MINED_MISCORRECTIONS.iter().enumerate() {
        assert_eq!(
            pos.len(),
            5,
            "mined cell {i} must be a 5-error (beyond-t) pattern"
        );
        let mut w = cw.clone();
        for (&p, &m) in pos.iter().zip(*mag) {
            w[p] ^= m;
        }
        // Sanity: the injected word really is > t from the true original.
        assert_ne!(w, cw);

        let md = md_bch(&w)
            .unwrap_or_else(|| panic!("mined cell {i}: md-codec must BCH-accept (miscorrect)"));
        let pg = pgz::decode(&f, &w)
            .unwrap_or_else(|| panic!("mined cell {i}: PGZ must accept the same miscorrection"));
        let pg_corrected = pgz::apply(&w, &pg);
        assert_eq!(
            md, pg_corrected,
            "mined cell {i}: md-codec and PGZ produced different miscorrected codewords"
        );
        // The miscorrection is to a DIFFERENT codeword than the true original.
        assert_ne!(
            md, cw,
            "mined cell {i}: expected a miscorrection, not the true original"
        );
        // Public entry point does not report TooManyErrors for a BCH-accepted word.
        assert!(
            !public_decode_rejects(&w),
            "mined cell {i}: decode_with_correction must not report TooManyErrors"
        );
    }
}
