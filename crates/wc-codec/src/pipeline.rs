//! Integration layer (P4): integrity tag + GEOM header + fixed-`U` ledger +
//! stop-sign + the full end-to-end encode / decode pipeline (plan §3, §4.1–4.5,
//! §5, §7 P4).
//!
//! This module ties together P1 ([`crate::field`] / [`crate::regroup`] /
//! [`crate::wordmap`] / [`crate::pad`]), P2 ([`crate::rs`]) and P3
//! ([`crate::sync`]) into the codec-agnostic `(SourceKind, payload, payload_bits)`
//! ↔ `Vec<&'static str>` round-trip.
//!
//! # The engraved-stream layout (the load-bearing architectural decision)
//!
//! ```text
//! [H0] [GEOM 4] [ledger 2U] [ interleave(K data + checkpoints) ] [parity m] [stop-sign 2]
//! \__________________________ engraved word stream ________________________________________/
//!  \__ K′ message __/         \________ K′ message (cont.) ______/
//!  (the ledger is SPLICED INTO the stream between GEOM and the interleave region,
//!   but is NOT part of the RS message K′)
//! ```
//!
//! **RESOLVED DESIGN DECISION — the ledger lives OUTSIDE the RS codeword.** The
//! plan §4.2 closed-form `K′` *appears* to fold the `2U` ledger into the RS
//! message. That is INCOMPATIBLE with append-only upgrades: filling a previously-
//! blank ledger slot mutates the RS message `K′`, which would invalidate every
//! parity word already engraved. We therefore freeze:
//!
//! - **RS message `K′` = `[H0] (1) ‖ [GEOM] (4) ‖ interleave(payload+tag, checkpoints)`**
//!   — the FIXED parts only. `K′ = 1 + 4 + (K + checkpoints)`. (RAID's `H1` /
//!   array-id are P5; for P4 `has-raid = 0`, solo.)
//! - **OUTSIDE the RS codeword:** the **ledger** (`2U` words, spliced into the
//!   engraved stream right after GEOM) and the **stop-sign** (2 words, after the
//!   parity). Each carries its OWN checksum.
//! - **RS codeword = `K′` ‖ parity(`m`).** The engraved word stream is
//!   `[H0][GEOM 4][ledger 2U][interleave(K data+checkpoints)][parity m][stop-sign 2]`.
//!   The decoder extracts `K′` by SKIPPING the `2U` ledger region (whose size is
//!   known the instant GEOM is read).
//!
//! This preserves append-only (an upgrade fills the next blank ledger slot +
//! appends parity + writes a new stop-sign, never touching `K′`) AND keeps the
//! header RS-correctable. Only the mutable ledger / stop-sign lose RS protection
//! — mitigated by their own checksums plus the stop-sign cross-check.

use crate::field;
use crate::regroup;
use crate::rs;
use crate::sync;
use crate::{Decoded, EncodeOpts, RepairSummary, SourceKind, WcError};
use sha2::{Digest, Sha256};

// ===========================================================================
// Frozen bit-layout constants (plan §4) — pinned, KAT-locked.
// ===========================================================================

/// 11-bit symbol mask (low 11 bits).
const SYM_MASK: u16 = 0x07FF;

/// H0 field widths (plan §4): `version(4) | source-kind(2) | has-raid(1) |
/// reserved(4)`, packed MSB-first across 11 bits.
const H0_VERSION: u16 = 0; // version 0
/// source-kind code for `mk1` xpubs (plan §4: `00`).
const SRC_MK1: u16 = 0b00;
/// source-kind code for `md1` descriptors (plan §4: `01`).
const SRC_MD1: u16 = 0b01;

/// Ledger slot marker `0b1110` (plan §4) — distinct from the checkpoint marker
/// (`0b101`) and the stop-sign marker (`0b1111`).
const LEDGER_MARKER: u16 = 0b1110;
/// Stop-sign marker `0b1111` (plan §4).
const STOP_MARKER: u16 = 0b1111;

/// Default integrity-tag bit-width `t` (plan §3 / §4.5): 44 bits = 4 words,
/// residual `≤ 2⁻⁴⁴`.
pub const DEFAULT_INTEGRITY_BITS: u8 = 44;
/// Minimum integrity-tag bit-width (plan §3 / §4.5): 33 bits (3 words).
pub const MIN_INTEGRITY_BITS: u8 = 33;
/// Maximum integrity-tag bit-width. Bounded by the **6-bit GEOM `t` field**
/// (`build_geom` / `parse_header` pack `t` in 6 bits → max value 63), NOT by the
/// SHA-256 digest. A larger `t` would overflow the field on encode (the low 6
/// bits are stored, e.g. `64 → 0`) and then fail `parse_header`'s range check on
/// decode — an `encode`-accepted-but-NEVER-decodable card (silent
/// unrecoverability). 63 bits is the true field ceiling and still gives a
/// residual `≤ 2⁻⁶³`. (P6 fuzz finding: `wc_roundtrip` over `t=64` surfaced the
/// encode/decode asymmetry.)
pub const MAX_INTEGRITY_BITS: u8 = 63;

/// Default reserved ledger slots `U` (plan §4.2): creation + 2 upgrades.
pub const DEFAULT_U_SLOTS: u8 = 3;

/// CRC-11 generator for the header-CRC. We use the same primitive polynomial as
/// the field, `x¹¹ + x² + 1` (`0x805` as a 12-bit value) — primitive ⇒ all
/// single-bit errors detected, uniform single-substitution miss `≤ 2⁻¹¹`.
const CRC11_POLY: u16 = 0x805;

// ===========================================================================
// Bit-stream helpers (MSB-first packing of fixed-width fields into 11-bit words).
// ===========================================================================

/// A small MSB-first bit writer that emits 11-bit symbols.
struct BitWriter {
    acc: u64,
    nbits: u32,
    out: Vec<u16>,
}

impl BitWriter {
    fn new() -> Self {
        BitWriter {
            acc: 0,
            nbits: 0,
            out: Vec::new(),
        }
    }

    /// Push the low `width` bits of `value` (MSB-first).
    fn push(&mut self, value: u64, width: u32) {
        debug_assert!(width <= 32);
        let masked = if width == 64 {
            value
        } else {
            value & ((1u64 << width) - 1)
        };
        self.acc = (self.acc << width) | masked;
        self.nbits += width;
        while self.nbits >= 11 {
            let shift = self.nbits - 11;
            let sym = ((self.acc >> shift) & SYM_MASK as u64) as u16;
            self.out.push(sym);
            self.nbits -= 11;
            self.acc &= (1u64 << self.nbits) - 1;
        }
    }

    /// Finish, asserting an exact 11-bit boundary (no residual partial symbol).
    fn finish(self) -> Vec<u16> {
        debug_assert_eq!(self.nbits, 0, "BitWriter: residual {} bits", self.nbits);
        self.out
    }
}

/// A small MSB-first bit reader over a slice of 11-bit symbols.
struct BitReader<'a> {
    syms: &'a [u16],
    pos: usize, // absolute bit index
}

impl<'a> BitReader<'a> {
    fn new(syms: &'a [u16]) -> Self {
        BitReader { syms, pos: 0 }
    }

    fn remaining(&self) -> usize {
        self.syms.len() * 11 - self.pos
    }

    /// Read `width` bits (MSB-first) as a u64. Returns `None` if not enough bits.
    fn read(&mut self, width: u32) -> Option<u64> {
        if (width as usize) > self.remaining() {
            return None;
        }
        let mut v: u64 = 0;
        for _ in 0..width {
            let sym_idx = self.pos / 11;
            let bit_in_sym = self.pos % 11; // 0 == MSB
            let bit = (self.syms[sym_idx] >> (10 - bit_in_sym)) & 1;
            v = (v << 1) | bit as u64;
            self.pos += 1;
        }
        Some(v)
    }
}

// ===========================================================================
// CRC-11 over a sequence of 11-bit symbols (for the header-CRC).
// ===========================================================================

/// CRC-11 over the 11-bit values of `words` (MSB-first, generator `x¹¹+x²+1`).
fn crc11(words: &[u16]) -> u16 {
    let mut reg: u32 = 0;
    for &w in words {
        for bit in (0..11).rev() {
            let in_bit = ((w >> bit) & 1) as u32;
            reg = (reg << 1) | in_bit;
            if reg & 0x800 != 0 {
                // bit 11 set ⇒ subtract the generator (x¹¹ + x² + 1)
                reg ^= CRC11_POLY as u32;
            }
            reg &= 0x7FF;
        }
    }
    (reg & 0x7FF) as u16
}

// ===========================================================================
// H0 / GEOM header (plan §4.2).
// ===========================================================================

fn source_kind_code(kind: SourceKind) -> u16 {
    match kind {
        SourceKind::Mk1Xpub => SRC_MK1,
        SourceKind::Md1Descriptor => SRC_MD1,
    }
}

fn source_kind_from_code(code: u16) -> Option<SourceKind> {
    match code {
        SRC_MK1 => Some(SourceKind::Mk1Xpub),
        SRC_MD1 => Some(SourceKind::Md1Descriptor),
        _ => None,
    }
}

/// Build the H0 word: `version(4) | source-kind(2) | has-raid(1) | reserved(4)`.
/// `has_raid` is `true` for a RAID plate (P5) — it gates the H1 + array-id words.
fn build_h0(kind: SourceKind, has_raid: bool) -> u16 {
    let mut w = BitWriter::new();
    w.push(H0_VERSION as u64, 4);
    w.push(source_kind_code(kind) as u64, 2);
    w.push(has_raid as u64, 1); // has-raid bit (plan §4.2)
    w.push(0, 4); // reserved
    w.finish()[0]
}

/// The fixed RAID header fields carried inside `K′` (plan §4.2 H1 + array-id),
/// present iff the H0 `has-raid` bit is set. RS-protected + header-CRC-covered.
///
/// **H1** (2 words = 22 bits): `n−1(5: 1..32) | role(2) | index-in-array(5: 0..31)
/// | reserved(10)`. The full **5-bit** `index-in-array` is the `P₂` α-exponent
/// (plan §3 / NEW-I2), so r=2 MDS holds for all `n ≤ 32`. **array-id** (2 words):
/// `top22(SHA-256(array_id_seed ‖ SHA-256(payload-canonical)))` — the payload
/// digest is folded in so two DIFFERENT wallets sharing a cosigner set get
/// DIFFERENT ids (constellation-eval **F2**; derivation in [`crate::raid`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RaidHeaderFields {
    /// Number of data plates `n` (`2..=32`).
    pub n: usize,
    /// The plate's role code (0 = Data, 1 = ParityA, 2 = ParityB) — plan §4.2.
    pub role: u16,
    /// The plate's `index-in-array` (0..31), the `P₂` α-exponent — plan §3.
    pub index: usize,
    /// The 22-bit array-id — `top22(SHA-256(array_id_seed ‖ SHA-256(payload-
    /// canonical)))` (the payload digest is folded in — F2; see [`crate::raid`]).
    pub array_id: u32,
}

/// Role codes (plan §4.2 H1): `0 = Data`, `1 = ParityA`, `2 = ParityB`.
pub(crate) const RAID_ROLE_DATA: u16 = 0;
pub(crate) const RAID_ROLE_PARITY_A: u16 = 1;
pub(crate) const RAID_ROLE_PARITY_B: u16 = 2;

/// The extra positional header words a RAID plate carries (H1 = 2 + array-id = 2).
const RAID_HEADER_WORDS: usize = 4;

/// The 22-bit array-id primitive: the **top 22 bits** of `SHA-256(bytes)`
/// (plan §3 / §4.2). The RAID encoder ([`crate::raid::raid_encode`]) calls this
/// with `array_id_seed ‖ SHA-256(payload-canonical)` — the payload digest is
/// folded in so two same-cosigner different-payload arrays get DIFFERENT ids
/// (constellation-eval **F2**), NOT the bare seed alone.
pub(crate) fn array_id_from_seed(bytes: &[u8]) -> u32 {
    let d = Sha256::digest(bytes);
    // Top 22 bits = the first 22 MSB-first bits of the digest.
    let top = ((d[0] as u32) << 16) | ((d[1] as u32) << 8) | (d[2] as u32); // 24 bits
    top >> 2 // drop the low 2 bits → top 22
}

/// Build the H1 two-word block: `n−1(5) | role(2) | index(5) | reserved(10)`.
fn build_h1(n: usize, role: u16, index: usize) -> [u16; 2] {
    let mut w = BitWriter::new();
    w.push((n as u64) - 1, 5);
    w.push(role as u64, 2);
    w.push(index as u64, 5);
    w.push(0, 10); // reserved
    let v = w.finish();
    [v[0], v[1]]
}

/// Build the array-id two-word block: the 22-bit array-id, MSB-first.
fn build_array_id(array_id: u32) -> [u16; 2] {
    let mut w = BitWriter::new();
    w.push(array_id as u64, 22);
    let v = w.finish();
    [v[0], v[1]]
}

/// The geometry derived from `(payload_bits, t, has_raid)` — closed-form, no RS
/// dependency.
#[derive(Debug, Clone, Copy)]
struct Geometry {
    k: usize,            // data symbols carrying payload+tag
    checkpoints: usize,  // checkpoint words interspersed
    header_words: usize, // positional header word count (5 solo, 9 raid)
    kprime: usize,       // RS message length = header_words + K + checkpoints
}

/// The positional header word count: `H0(1) + [H1(2)+array-id(2) if raid] +
/// GEOM(4)` = 5 solo, 9 raid (plan §4.2). These are the words that precede the
/// interleave region and are part of the RS message `K′`.
fn header_word_count(has_raid: bool) -> usize {
    1 + if has_raid { RAID_HEADER_WORDS } else { 0 } + 4
}

/// Closed-form geometry recovery (plan §4.2). `K = ceil((payload_bits + t)/11)`;
/// `checkpoint_count` via the P3 layout (which already encodes `b = floor(√K +
/// 0.5)` and the small-K degenerate-single-checkpoint rule). `K′ = header_words +
/// K + checkpoints` (header words inside the RS message; ledger is NOT). With
/// RAID the header carries 4 extra words (H1 + array-id) inside `K′`.
fn derive_geometry(payload_bits: usize, t: u8, has_raid: bool) -> Geometry {
    let total_bits = payload_bits + t as usize;
    let k = total_bits.div_ceil(11);
    let layout = sync::checkpoint_layout(k);
    let checkpoints = layout.checkpoint_count;
    let header_words = header_word_count(has_raid);
    let kprime = header_words + k + checkpoints;
    Geometry {
        k,
        checkpoints,
        header_words,
        kprime,
    }
}

/// The 4 GEOM words for `(payload_bits, t, u)`. The header-CRC (word D) covers
/// ALL positional header words preceding GEOM-D — `prefix` = `[H0]` (solo) or
/// `[H0, H1-a, H1-b, aid-a, aid-b]` (raid) — followed by GEOM-A..C (plan §4.2).
fn build_geom(prefix: &[u16], payload_bits: usize, t: u8, u: u8) -> [u16; 4] {
    // words A+B = payload_bits(16) | t(6)  (22 bits → exactly 2 words)
    let mut ab = BitWriter::new();
    ab.push(payload_bits as u64, 16);
    ab.push(t as u64, 6);
    let ab = ab.finish();
    let geom_a = ab[0];
    let geom_b = ab[1];

    // word C = U(3) | reserved(8)
    let mut c = BitWriter::new();
    c.push(u as u64, 3);
    c.push(0, 8); // reserved
    let geom_c = c.finish()[0];

    // word D = header-CRC(11) over (prefix ‖ GEOM-A ‖ GEOM-B ‖ GEOM-C).
    let mut crc_input: Vec<u16> = Vec::with_capacity(prefix.len() + 3);
    crc_input.extend_from_slice(prefix);
    crc_input.extend_from_slice(&[geom_a, geom_b, geom_c]);
    let geom_d = crc11(&crc_input);

    [geom_a, geom_b, geom_c, geom_d]
}

/// Build the full positional header (`[H0] [H1 2 + array-id 2]? [GEOM 4]`) for a
/// solo or RAID plate, returning the word vector. The header-CRC covers all of
/// `H0 ‖ (H1 ‖ array-id)? ‖ GEOM-A..C`.
fn build_header(
    kind: SourceKind,
    payload_bits: usize,
    t: u8,
    u: u8,
    raid: Option<RaidHeaderFields>,
) -> Vec<u16> {
    let h0 = build_h0(kind, raid.is_some());
    let mut prefix: Vec<u16> = vec![h0];
    if let Some(rf) = raid {
        let h1 = build_h1(rf.n, rf.role, rf.index);
        let aid = build_array_id(rf.array_id);
        prefix.extend_from_slice(&h1);
        prefix.extend_from_slice(&aid);
    }
    let geom = build_geom(&prefix, payload_bits, t, u);
    let mut out = prefix;
    out.extend_from_slice(&geom);
    out
}

/// The parsed, CRC-verified header geometry.
#[derive(Debug, Clone, Copy)]
struct ParsedHeader {
    kind: SourceKind,
    payload_bits: usize,
    t: u8,
    u: u8,
    /// The RAID header fields, present iff the H0 `has-raid` bit was set.
    raid: Option<RaidHeaderFields>,
    geom: Geometry,
}

/// Read H0 (+ H1 + array-id if RAID) + GEOM positionally from the engraved word
/// stream and verify the header-CRC. Returns the parsed geometry, or a `WcError`.
fn parse_header(words: &[u16]) -> Result<ParsedHeader, WcError> {
    // Need at least H0(1) to read the has-raid bit.
    if words.is_empty() {
        return Err(WcError::Truncated);
    }

    // H0 fields.
    let mut hr = BitReader::new(&words[0..1]);
    let _version = hr.read(4).unwrap();
    let src_code = hr.read(2).unwrap() as u16;
    let has_raid = hr.read(1).unwrap() != 0;
    let _reserved = hr.read(4).unwrap();
    let kind = source_kind_from_code(src_code).ok_or(WcError::HeaderCrcMismatch)?;

    let header_words = header_word_count(has_raid);
    if words.len() < header_words {
        return Err(WcError::Truncated);
    }

    // Positional prefix preceding GEOM: [H0] (+ [H1 2][array-id 2] if raid).
    let geom_start = 1 + if has_raid { RAID_HEADER_WORDS } else { 0 };
    let prefix = &words[0..geom_start];

    let geom_a = words[geom_start];
    let geom_b = words[geom_start + 1];
    let geom_c = words[geom_start + 2];
    let geom_d = words[geom_start + 3];

    // Verify the header-CRC FIRST (over prefix ‖ GEOM-A ‖ GEOM-B ‖ GEOM-C).
    let mut crc_input: Vec<u16> = Vec::with_capacity(prefix.len() + 3);
    crc_input.extend_from_slice(prefix);
    crc_input.extend_from_slice(&[geom_a, geom_b, geom_c]);
    let want = crc11(&crc_input) & SYM_MASK;
    if (geom_d & SYM_MASK) != want {
        return Err(WcError::HeaderCrcMismatch);
    }

    // GEOM A+B = payload_bits(16) | t(6).
    let ab = [geom_a, geom_b];
    let mut gr = BitReader::new(&ab);
    let payload_bits = gr.read(16).unwrap() as usize;
    let t = gr.read(6).unwrap() as u8;

    // GEOM C = U(3) | reserved(8).
    let c = [geom_c];
    let mut cr = BitReader::new(&c);
    let u = cr.read(3).unwrap() as u8;

    // Sanity: t in range. A corrupt-but-CRC-passing t is astronomically
    // unlikely, but a malformed/hostile word list must not panic downstream.
    if !(MIN_INTEGRITY_BITS..=MAX_INTEGRITY_BITS).contains(&t) {
        return Err(WcError::HeaderCrcMismatch);
    }

    // Parse the RAID header fields (H1 + array-id), if present. The CRC has
    // already verified these words, so a successful parse here is trustworthy.
    let raid = if has_raid {
        let h1a = words[1];
        let h1b = words[2];
        let aida = words[3];
        let aidb = words[4];
        let h1 = [h1a, h1b];
        let mut h1r = BitReader::new(&h1);
        let n = h1r.read(5).unwrap() as usize + 1; // n−1 stored
        let role = h1r.read(2).unwrap() as u16;
        let index = h1r.read(5).unwrap() as usize;
        let aid = [aida, aidb];
        let mut ar = BitReader::new(&aid);
        let array_id = ar.read(22).unwrap() as u32;
        // Range sanity (CRC-verified, but a hostile list must never panic). For a
        // DATA plate the `index` is the `P₂` α-exponent and MUST be `< n`; for a
        // PARITY plate the index is a wire placeholder (role identifies it), so it
        // is not exponent-constrained.
        if !(2..=32).contains(&n) || role > RAID_ROLE_PARITY_B {
            return Err(WcError::HeaderCrcMismatch);
        }
        if role == RAID_ROLE_DATA && index >= n {
            return Err(WcError::HeaderCrcMismatch);
        }
        Some(RaidHeaderFields {
            n,
            role,
            index,
            array_id,
        })
    } else {
        None
    };

    let geom = derive_geometry(payload_bits, t, has_raid);
    Ok(ParsedHeader {
        kind,
        payload_bits,
        t,
        u,
        raid,
        geom,
    })
}

// ===========================================================================
// Ledger slots + stop-sign (plan §4.2 / §4.4) — OUTSIDE the RS codeword.
// ===========================================================================

/// 7-bit checksum over a ledger slot's `marker(4) | count(11)` (plan §4.2). A
/// SHA-256-derived non-linear check over the 15 covered bits, low 7 bits.
fn ledger_checksum(marker: u16, count: u16) -> u16 {
    let mut h = Sha256::new();
    h.update([(marker & 0xF) as u8]);
    h.update((count & SYM_MASK).to_be_bytes());
    let d = h.finalize();
    (d[d.len() - 1] & 0x7F) as u16
}

/// Build a 2-word ledger slot: `marker(4)=0b1110 | count(11) | checksum(7)`.
fn build_ledger_slot(count: u16) -> [u16; 2] {
    let chk = ledger_checksum(LEDGER_MARKER, count);
    let mut w = BitWriter::new();
    w.push(LEDGER_MARKER as u64, 4);
    w.push((count & SYM_MASK) as u64, 11);
    w.push(chk as u64, 7);
    let v = w.finish();
    [v[0], v[1]]
}

/// Parse a 2-word ledger slot. Returns `Some(count)` for a marker+checksum-valid
/// filled slot, `None` for an unfilled (all-zero) slot OR an invalid/corrupt
/// slot (treated as not-recorded — authoritative length is the max over the
/// VALID filled slots and the stop-sign).
fn parse_ledger_slot(w0: u16, w1: u16) -> Option<u16> {
    if w0 == 0 && w1 == 0 {
        return None; // unfilled
    }
    let slot = [w0, w1];
    let mut r = BitReader::new(&slot);
    let marker = r.read(4)? as u16;
    let count = r.read(11)? as u16;
    let chk = r.read(7)? as u16;
    if marker != LEDGER_MARKER {
        return None;
    }
    if chk != ledger_checksum(marker, count) {
        return None;
    }
    Some(count)
}

/// 7-bit stop-sign checksum = SHA-256 over the 11-bit values of all PRECEDING
/// engraved words (serialized big-endian, 2 bytes each), low 7 bits (plan §4.4).
fn stop_sign_checksum(preceding: &[u16]) -> u16 {
    let mut h = Sha256::new();
    for &w in preceding {
        h.update((w & SYM_MASK).to_be_bytes());
    }
    let d = h.finalize();
    (d[d.len() - 1] & 0x7F) as u16
}

/// Build a 2-word stop-sign: `marker(4)=0b1111 | count(11) | checksum(7)`, the
/// checksum over `preceding` (all words before this stop-sign).
fn build_stop_sign(count: u16, preceding: &[u16]) -> [u16; 2] {
    let chk = stop_sign_checksum(preceding);
    let mut w = BitWriter::new();
    w.push(STOP_MARKER as u64, 4);
    w.push((count & SYM_MASK) as u64, 11);
    w.push(chk as u64, 7);
    let v = w.finish();
    [v[0], v[1]]
}

/// Parse a 2-word stop-sign. Returns `Some(count)` if the marker matches (the
/// checksum is validated separately by the caller, which has the preceding
/// words). `None` if the marker does not match.
fn parse_stop_sign_marker(w0: u16, w1: u16) -> Option<u16> {
    let stop = [w0, w1];
    let mut r = BitReader::new(&stop);
    let marker = r.read(4)? as u16;
    let count = r.read(11)? as u16;
    let _chk = r.read(7)?;
    if marker != STOP_MARKER {
        return None;
    }
    Some(count)
}

// ===========================================================================
// Integrity tag (plan §4.5) — t-bit SHA-256(canonical_payload)[0..t], MSB-first.
// ===========================================================================

/// The **canonical** payload bytes: exactly `ceil(payload_bits / 8)` bytes, with
/// any bits of the final byte BEYOND `payload_bits` forced to zero.
///
/// The integrity tag MUST be computed over THIS form (not the raw input slice):
/// when `payload_bits` is not a multiple of 8 (the md1 case) the trailing
/// sub-byte bits are NOT part of the payload, and the 8↔11 regroup zero-pads them
/// — so the decoder can only ever recover the canonical form. Hashing the raw
/// input (with arbitrary trailing bits) would make encode/decode tags disagree
/// (an IntegrityMismatch on a perfectly clean round-trip).
fn canonical_payload_bytes(payload: &[u8], payload_bits: usize) -> Vec<u8> {
    extract_payload_bytes_from_slice(payload, payload_bits)
}

/// The integrity tag bits: the top `t` bits of `SHA-256(canonical_payload)`,
/// returned as a `Vec<bool>` (MSB-first), to be appended after the payload bits
/// before the 8→11 regroup so the tag is RS-protected. `canonical` MUST already
/// be the canonical-payload form (see [`canonical_payload_bytes`]).
fn integrity_tag_bits(canonical: &[u8], t: u8) -> Vec<bool> {
    let digest = Sha256::digest(canonical);
    let mut bits = Vec::with_capacity(t as usize);
    for i in 0..t as usize {
        let byte = digest[i / 8];
        let bit = (byte >> (7 - (i % 8))) & 1;
        bits.push(bit != 0);
    }
    bits
}

// ===========================================================================
// Encode (plan §5).
// ===========================================================================

/// Pack `payload_bytes` (taking the first `payload_bits` bits, MSB-first) then
/// the `t`-bit integrity tag into `K` GF(2¹¹) symbols (8→11 regroup, plan §4.1).
/// The tag is computed over the CANONICAL payload bytes (trailing sub-byte bits
/// zeroed) so it agrees with the decoder's recovered form.
fn build_data_symbols(payload: &[u8], payload_bits: usize, t: u8) -> Vec<u16> {
    let canonical = canonical_payload_bytes(payload, payload_bits);
    let tag_bits = integrity_tag_bits(&canonical, t);

    // Assemble the full bit string: payload_bits payload bits ‖ t tag bits, then
    // pack 11 at a time MSB-first, low-bit zero-padding the final symbol.
    let total_bits = payload_bits + t as usize;
    let mut symbols = Vec::with_capacity(total_bits.div_ceil(11));
    let mut acc: u32 = 0;
    let mut nbits: u32 = 0;

    let push_bit = |bit: u32, acc: &mut u32, nbits: &mut u32, out: &mut Vec<u16>| {
        *acc = (*acc << 1) | (bit & 1);
        *nbits += 1;
        if *nbits == 11 {
            out.push((*acc & SYM_MASK as u32) as u16);
            *acc = 0;
            *nbits = 0;
        }
    };

    for i in 0..payload_bits {
        let byte = payload[i / 8];
        let bit = ((byte >> (7 - (i % 8))) & 1) as u32;
        push_bit(bit, &mut acc, &mut nbits, &mut symbols);
    }
    for &b in &tag_bits {
        push_bit(b as u32, &mut acc, &mut nbits, &mut symbols);
    }
    if nbits > 0 {
        // Final partial symbol: low-bit zero-pad to 11 bits.
        let pad = 11 - nbits;
        symbols.push(((acc << pad) & SYM_MASK as u32) as u16);
    }
    symbols
}

/// Encode `(kind, payload, payload_bits)` into an engravable BIP-39 word stream
/// (plan §5). `mk1` callers pass `payload_bits = 8 * payload.len()`; `md1`
/// callers pass the exact bit-precise length. Produces a **solo** Word-Card
/// (`has-raid = 0`); the RAID layer ([`crate::raid`]) drives [`encode_inner`]
/// with the per-plate H1 / array-id fields.
pub fn encode(
    kind: SourceKind,
    payload: &[u8],
    payload_bits: usize,
    opts: &EncodeOpts,
) -> Result<Vec<&'static str>, WcError> {
    encode_inner(kind, payload, payload_bits, opts, None)
}

/// The shared encode core (plan §5), parameterized on the optional RAID header
/// (`raid = None` ⇒ solo card with `has-raid = 0`; `Some(..)` ⇒ a RAID plate
/// with H1 + array-id inside `K′`). Each plate is a full, standalone Word-Card.
pub(crate) fn encode_inner(
    kind: SourceKind,
    payload: &[u8],
    payload_bits: usize,
    opts: &EncodeOpts,
    raid: Option<RaidHeaderFields>,
) -> Result<Vec<&'static str>, WcError> {
    // --- Validate options / sizes. --------------------------------------
    let t = opts.integrity_bits;
    if !(MIN_INTEGRITY_BITS..=MAX_INTEGRITY_BITS).contains(&t) {
        return Err(WcError::InvalidParams);
    }
    if opts.u_slots == 0 {
        return Err(WcError::InvalidParams);
    }
    if payload_bits > payload.len() * 8 {
        return Err(WcError::InvalidParams);
    }
    // payload_bits must fit the 16-bit GEOM field.
    if payload_bits > 0xFFFF {
        return Err(WcError::InvalidParams);
    }
    if let Some(rf) = raid {
        // Defensive: the RAID header fields must fit their bit-field ranges. The
        // DATA-plate index is the `P₂` α-exponent (`< n`); a PARITY plate carries
        // a wire-placeholder index (`< 32`, fits 5 bits) identified by its role.
        if !(2..=32).contains(&rf.n)
            || rf.role > RAID_ROLE_PARITY_B
            || rf.index >= 32
            || rf.array_id > 0x3F_FFFF
        {
            return Err(WcError::InvalidParams);
        }
        if rf.role == RAID_ROLE_DATA && rf.index >= rf.n {
            return Err(WcError::InvalidParams);
        }
    }

    let has_raid = raid.is_some();
    let geom = derive_geometry(payload_bits, t, has_raid);

    // --- Layer A: payload+tag → K data symbols (8→11 regroup). ----------
    let data_symbols = build_data_symbols(payload, payload_bits, t);
    debug_assert_eq!(data_symbols.len(), geom.k);

    // --- Layer B: interleave checkpoints. -------------------------------
    let interleaved = sync::interleave(&data_symbols);
    debug_assert_eq!(interleaved.len(), geom.k + geom.checkpoints);

    // --- Build the RS message K′ = header ‖ interleave. -----------------
    let header_words = build_header(kind, payload_bits, t, opts.u_slots, raid);
    debug_assert_eq!(header_words.len(), geom.header_words);
    let mut kprime_msg: Vec<u16> = Vec::with_capacity(geom.kprime);
    kprime_msg.extend_from_slice(&header_words);
    kprime_msg.extend_from_slice(&interleaved);
    debug_assert_eq!(kprime_msg.len(), geom.kprime);

    // --- Layer C: RS parity over K′. ------------------------------------
    let parity = rs::rs_parity(&kprime_msg, opts.parity_words).map_err(WcError::from)?;

    // --- Assemble the engraved stream. ----------------------------------
    // Final word count = header + ledger(2U) + interleave + parity + stop-sign(2).
    // Equivalently kprime + 2U + parity + 2.
    let total_words = geom.kprime + 2 * opts.u_slots as usize + parity.len() + 2;
    let count = total_words as u16; // ≤ 2047 enforced below.
    if total_words > sync_field_cap() {
        return Err(WcError::InvalidParams);
    }

    // Ledger: slot 0 filled with the creation cumulative count; the rest blank.
    let mut ledger: Vec<u16> = Vec::with_capacity(2 * opts.u_slots as usize);
    let slot0 = build_ledger_slot(count);
    ledger.extend_from_slice(&slot0);
    for _ in 1..opts.u_slots {
        ledger.push(0);
        ledger.push(0);
    }

    // Stream so far (everything before the stop-sign), in engraved order:
    //   [header][ledger 2U][interleave][parity]
    let mut stream: Vec<u16> = Vec::with_capacity(total_words);
    stream.extend_from_slice(&header_words);
    stream.extend_from_slice(&ledger);
    stream.extend_from_slice(&interleaved);
    stream.extend_from_slice(&parity);

    // Stop-sign: checksum over ALL preceding engraved words.
    let stop = build_stop_sign(count, &stream);
    stream.extend_from_slice(&stop);
    debug_assert_eq!(stream.len(), total_words);

    // --- Map symbols → BIP-39 words. ------------------------------------
    symbols_to_words(&stream)
}

/// The codeword-length cap = the field's distinct evaluation points (2047). A
/// stream longer than this cannot be a valid single RS codeword.
fn sync_field_cap() -> usize {
    field::MULTIPLICATIVE_ORDER as usize
}

/// Map a symbol stream to BIP-39 English words (each symbol < 2048).
fn symbols_to_words(syms: &[u16]) -> Result<Vec<&'static str>, WcError> {
    let mut out = Vec::with_capacity(syms.len());
    for &s in syms {
        let w = crate::wordmap::symbol_to_word(s).ok_or(WcError::InvalidParams)?;
        out.push(w);
    }
    Ok(out)
}

// ===========================================================================
// Decode (plan §5 / §8) — two-pass.
// ===========================================================================

/// Decode an engraved word list back to its payload (plan §5/§8). Never panics
/// on any input.
pub fn decode(words: &[&str]) -> Result<Decoded, WcError> {
    // --- words → symbols (case-insensitive). ----------------------------
    let mut syms: Vec<u16> = Vec::with_capacity(words.len());
    for w in words {
        let lc = w.to_ascii_lowercase();
        let s = crate::wordmap::word_to_symbol(&lc).ok_or(WcError::UnknownWord)?;
        syms.push(s);
    }

    // --- Pass 1: read H0 + GEOM positionally; verify header-CRC. ---------
    let header = parse_header(&syms)?;
    let geom = header.geom;
    let u = header.u as usize;

    // Engraved layout: [header][ledger(2U)][interleave][parity][stop(2)], where
    // header = H0(1) [+ H1(2) + array-id(2) if raid] + GEOM(4) = geom.header_words.
    // K′ = header + interleave; the ledger is spliced between the header and the
    // interleave region and is NOT part of K′.
    let ledger_start = geom.header_words;
    let ledger_len = 2 * u;
    let interleave_start = ledger_start + ledger_len;

    // The interleave region length is K + checkpoints; K′ words present in the
    // engraved stream before parity = 5 + interleave_len, BUT the K′ MESSAGE we
    // feed to RS is H0 ‖ GEOM ‖ interleave (i.e. skipping the ledger).
    let interleave_len = geom.k + geom.checkpoints;

    // --- Read the ledger (FIXED positions) + the tail stop-sign. --------
    // Authoritative recorded length = max over the VALID filled ledger slots AND
    // a VALID tail stop-sign (plan §4.2 / §4.4). The ledger is front-anchored at
    // KNOWN positions `[header_words .. header_words+2U)` — we read it ONLY there
    // (never scan), so a stray data word carrying the `0b1110` marker cannot inflate
    // the recorded length. The stop-sign is validated ONLY at the EXPECTED tail
    // position (the last 2 words) — scanning every position would give a 2⁻¹¹
    // false-positive PER position (≈ several % over a full card) and falsely flag
    // a clean card as truncated. The front ledger is what survives a lost tail
    // (spec §6.3 ledger-durability), so it carries the truncation signal alone.
    let mut recorded_max: u16 = 0;
    let mut any_record = false;
    {
        let mut p = ledger_start;
        for _ in 0..u {
            if p + 1 < syms.len() {
                if let Some(c) = parse_ledger_slot(syms[p], syms[p + 1]) {
                    recorded_max = recorded_max.max(c);
                    any_record = true;
                }
            }
            p += 2;
        }
    }

    // Defensive bounds: the stream must at least reach the interleave region.
    if syms.len() < interleave_start {
        return Err(WcError::Truncated);
    }

    // --- Region bounding (indel-aware). ---------------------------------
    // The CREATION total word count (ledger slot 0 / stop-sign) tells us the
    // parity width `m = creation_total − interleave_start − interleave_len − 2`.
    // The TAIL stop-sign re-anchors the parity end regardless of WHERE a single
    // indel fell upstream (its checksum covers all preceding words), so we set
    //   parity_end   = stop-sign position (or end if no valid stop-sign),
    //   parity_start = parity_end − m,
    //   interleave   = [interleave_start .. parity_start).
    // This makes the interleave region the right length for delta ∈ {0,−1,+1}:
    // a deletion upstream shifts the tail left by one, so the interleave region
    // is `interleave_len − 1` (a single-deletion the sync layer then localizes);
    // an insertion makes it `interleave_len + 1`. The sync layer handles ±1.
    let words_present = syms.len() as u16;

    // Re-anchor `parity_end` on the TAIL stop-sign's MARKER (position-anchored,
    // last 2 words). The marker is reliable even when an upstream indel/error has
    // changed the words the stop-sign's checksum covers — so we use the marker to
    // bound parity, and the checksum only as an additional clean-card recorded-
    // length cross-check (it validates exactly when the body is intact).
    let mut parity_end = syms.len();
    let mut stop_count: Option<u16> = None;
    if syms.len() >= interleave_start + 2 {
        let tail = syms.len() - 2;
        if let Some(c) = parse_stop_sign_marker(syms[tail], syms[tail + 1]) {
            // Position-anchored: trust the marker to bound parity.
            parity_end = tail;
            stop_count = Some(c);
            // Checksum cross-check (clean-body only) for the recorded length.
            let chk = stop_sign_checksum(&syms[..tail]);
            if (syms[tail + 1] & 0x7F) == chk {
                recorded_max = recorded_max.max(c);
                any_record = true;
            }
        }
    }

    let truncated = any_record && words_present < recorded_max;

    // Derive the creation total: prefer the stop-sign count (position-anchored
    // marker carries an exact 11-bit count even under upstream corruption), else
    // the ledger. If neither is available we fall back to the intact assumption.
    let creation_total = stop_count.map(|c| c as usize).or(if any_record {
        Some(recorded_max as usize)
    } else {
        None
    });

    // Compute `m` (parity width at creation) from the creation total, if known.
    let m = creation_total.and_then(|tot| tot.checked_sub(interleave_start + interleave_len + 2));

    // `parity_start`:
    //  - With a TAIL stop-sign present, re-anchor from the tail (`parity_end − m`).
    //    This correctly absorbs a single UPSTREAM indel (which shifts the whole
    //    tail by ±1) — the interleave region then comes out `interleave_len ∓ 1`
    //    and the sync layer localizes it.
    //  - WITHOUT a tail stop-sign the tail was LOST (truncation): the interleave
    //    region is intact at its NOMINAL offset and whatever parity survived runs
    //    to the end. Using `parity_end − m` here would wrongly eat into the intact
    //    interleave. So we anchor parity at the nominal interleave end.
    let parity_start = if stop_count.is_some() {
        match m {
            Some(m_words) if parity_end >= m_words => parity_end - m_words,
            _ => (interleave_start + interleave_len).min(parity_end),
        }
    } else {
        (interleave_start + interleave_len).min(parity_end)
    };

    // Guard the slice bounds.
    if parity_start < interleave_start || parity_start > syms.len() || parity_end < parity_start {
        // The interleave region itself is truncated below recoverable. Attempt the
        // short path (a one-short interleave) or refuse.
        return decode_short_interleave(&header, &syms, interleave_start, truncated);
    }

    let parity_region = &syms[parity_start..parity_end];
    let interleave_region = &syms[interleave_start..parity_start];

    // --- Pass 2: sync over the interleave region. -----------------------
    let outcome = sync::sync_classify(interleave_region, geom.k);
    finish_decode(
        &header,
        &syms,
        interleave_start,
        interleave_region,
        parity_region,
        outcome,
        truncated,
    )
}

/// Handle a stream whose interleave region is itself short (a tail deletion that
/// chipped a checkpoint / data word). We try a single-deletion structural recover
/// with whatever parity remains; else refuse as truncated.
fn decode_short_interleave(
    header: &ParsedHeader,
    syms: &[u16],
    interleave_start: usize,
    truncated: bool,
) -> Result<Decoded, WcError> {
    let geom = header.geom;
    let interleave_len = geom.k + geom.checkpoints;
    // The interleave region is everything from interleave_start to the end (no
    // parity / stop-sign survived). If it is exactly one short, try the sync
    // single-deletion path with zero parity (it can only succeed via erasure if
    // there is parity — so this typically refuses, which is correct).
    if syms.len() <= interleave_start {
        return Err(WcError::Truncated);
    }
    let interleave_region = &syms[interleave_start..];
    if interleave_region.len() + 1 == interleave_len || interleave_region.len() == interleave_len {
        let outcome = sync::sync_classify(interleave_region, geom.k);
        return finish_decode(
            header,
            syms,
            interleave_start,
            interleave_region,
            &[],
            outcome,
            true,
        );
    }
    let _ = truncated;
    Err(WcError::Truncated)
}

/// Common decode tail: drive the sync outcome through RS + the integrity tag.
#[allow(clippy::too_many_arguments)]
fn finish_decode(
    header: &ParsedHeader,
    _syms: &[u16],
    _interleave_start: usize,
    interleave_region: &[u16],
    parity_region: &[u16],
    outcome: sync::SyncOutcome,
    truncated: bool,
) -> Result<Decoded, WcError> {
    let geom = header.geom;

    match outcome {
        sync::SyncOutcome::Aligned { grid, erasures } => {
            // grid is the K'-length interleave region; reassemble the RS codeword:
            //   [H0][GEOM][grid][parity]   (K′ message ‖ parity)
            let recovered = rs_decode_and_check(header, &grid, &erasures, parity_region)?;
            Ok(make_decoded(header, recovered, truncated, &erasures, 0))
        }
        sync::SyncOutcome::SingleDeletionCandidates { gap_positions } => {
            // P3 located the lone deletion to a bounded candidate set; P4 is the
            // global tag oracle. For each candidate, reinsert a placeholder
            // erasure, RS-decode, recompute the SHA-256 tag, accept the unique
            // matching candidate; >1 match ⇒ refuse-ambiguous; none ⇒ refuse.
            let mut accepted: Option<Vec<u8>> = None;
            for &p in &gap_positions {
                // Reinsert a placeholder at p to rebuild the K'-length grid.
                let mut grid = interleave_region.to_vec();
                if p > grid.len() {
                    continue;
                }
                grid.insert(p, 0);
                if grid.len() != geom.k + geom.checkpoints {
                    continue;
                }
                let erasures = vec![p];
                if let Ok(payload) = rs_decode_and_check(header, &grid, &erasures, parity_region) {
                    // A tag-passing candidate. NOTE: several reinsertion slots in
                    // the same block legitimately yield the SAME correct payload
                    // (RS erasure-fills the gap regardless of the exact slot). That
                    // is NOT an ambiguity. We only refuse if two candidates yield
                    // DIFFERENT tag-passing payloads — two distinct valid
                    // pre-images at `≤ 2⁻ᵗ`, the genuine ambiguity (plan §4.3).
                    match &accepted {
                        None => accepted = Some(payload),
                        Some(prev) if *prev == payload => { /* same truth — fine */ }
                        Some(_) => return Err(WcError::IntegrityMismatch),
                    }
                }
            }
            match accepted {
                Some(payload) => Ok(make_decoded(
                    header,
                    payload,
                    truncated,
                    &[0; 1][..0], // erasure indices not surfaced for this path
                    1,            // one erasure filled (the reinserted gap)
                )),
                None => Err(WcError::Uncorrectable),
            }
        }
        sync::SyncOutcome::Refuse(e) => Err(WcError::from(e)),
    }
}

/// Reassemble the RS codeword from the aligned interleave `grid` + the `parity`
/// tail, RS-decode (with `erasures`), strip checkpoints/header, regroup 11→8,
/// and verify the post-correction integrity tag (plan §8 steps 3–5).
fn rs_decode_and_check(
    header: &ParsedHeader,
    grid: &[u16],
    interleave_erasures: &[usize],
    parity: &[u16],
) -> Result<Vec<u8>, WcError> {
    let geom = header.geom;
    if grid.len() != geom.k + geom.checkpoints {
        return Err(WcError::Uncorrectable);
    }

    // Rebuild the positional header (H0 ‖ [H1 ‖ array-id]? ‖ GEOM) from the parsed
    // header (canonical — the cold reader already trusts these via the header-CRC;
    // they are also inside the RS message).
    let header_words = build_header(
        header.kind,
        header.payload_bits,
        header.t,
        header.u,
        header.raid,
    );
    debug_assert_eq!(header_words.len(), geom.header_words);

    // RS message = [header][grid]; codeword = message ‖ parity.
    let mut codeword: Vec<u16> = Vec::with_capacity(geom.kprime + parity.len());
    codeword.extend_from_slice(&header_words);
    codeword.extend_from_slice(grid);
    codeword.extend_from_slice(parity);

    // Shift the interleave-region erasure indices into codeword coordinates: the
    // interleave region begins at offset `header_words` in the K′ message.
    let header_offset = geom.header_words;
    let mut erasures: Vec<usize> = interleave_erasures
        .iter()
        .map(|&e| e + header_offset)
        .collect();
    erasures.sort_unstable();
    erasures.dedup();

    let recovered_msg = rs::rs_decode(&codeword, geom.kprime, &erasures).map_err(WcError::from)?;

    // Strip the header → the interleave grid; then strip checkpoints.
    if recovered_msg.len() != geom.kprime {
        return Err(WcError::Uncorrectable);
    }
    let recovered_interleave = &recovered_msg[header_offset..];
    let data_symbols = strip_checkpoints(recovered_interleave, geom.k)?;

    // Regroup 11→8 → payload bytes ‖ tag bits. Total bits = payload_bits + t.
    let total_bits = header.payload_bits + header.t as usize;
    let all_bytes = regroup::symbols_to_bits(&data_symbols, total_bits).map_err(WcError::from)?;

    // Split: the first payload_bits bits are the payload; the next t are the tag.
    // `payload` is already the CANONICAL form (trailing sub-byte bits zeroed).
    let payload = extract_payload_bytes_from_slice(&all_bytes, header.payload_bits);

    // Post-correction integrity check (plan §8 step 5): recompute SHA-256 over
    // the recovered (canonical) payload, compare to the recovered tag bits. This
    // catches an RS miscorrection onto a valid-but-WRONG codeword at `≤ 2⁻ᵗ`.
    let want_tag = integrity_tag_bits(&payload, header.t);
    let got_tag = extract_tag_bits(&data_symbols, header.payload_bits, header.t);
    if want_tag != got_tag {
        return Err(WcError::IntegrityMismatch);
    }
    Ok(payload)
}

/// Strip the interspersed checkpoints from a `K'`-length interleave grid back to
/// the `K` data symbols (inverse of [`sync::interleave`]).
fn strip_checkpoints(grid: &[u16], k: usize) -> Result<Vec<u16>, WcError> {
    let layout = sync::checkpoint_layout(k);
    let mut out = Vec::with_capacity(k);
    let mut offset = 0usize;
    for &sz in &layout.block_sizes {
        let end = offset + sz;
        if end + 1 > grid.len() {
            return Err(WcError::Uncorrectable);
        }
        out.extend_from_slice(&grid[offset..end]);
        // skip the checkpoint word at `end`.
        offset = end + 1;
    }
    if offset != grid.len() {
        return Err(WcError::Uncorrectable);
    }
    if out.len() != k {
        return Err(WcError::Uncorrectable);
    }
    Ok(out)
}

/// Extract the first `payload_bits` bits (MSB-first) from `src` into a byte
/// vector of length `ceil(payload_bits/8)` (final byte low-bit-zero). This is the
/// canonical-payload projection, shared by encode (tag over the canonical form)
/// and decode (the recovered payload). `src` must carry `≥ payload_bits` bits.
fn extract_payload_bytes_from_slice(src: &[u8], payload_bits: usize) -> Vec<u8> {
    let n_bytes = payload_bits.div_ceil(8);
    let mut out = vec![0u8; n_bytes];
    for i in 0..payload_bits {
        let src_byte = src[i / 8];
        let bit = (src_byte >> (7 - (i % 8))) & 1;
        if bit != 0 {
            out[i / 8] |= bit << (7 - (i % 8));
        }
    }
    out
}

/// Extract the `t` tag bits that follow the `payload_bits` payload bits, from the
/// `K`-symbol data stream, as a `Vec<bool>` (MSB-first) — to compare against the
/// freshly-recomputed tag.
fn extract_tag_bits(data_symbols: &[u16], payload_bits: usize, t: u8) -> Vec<bool> {
    let mut bits = Vec::with_capacity(t as usize);
    for i in 0..t as usize {
        let abs = payload_bits + i;
        let sym_idx = abs / 11;
        let bit_in_sym = abs % 11; // 0 == MSB
        if sym_idx >= data_symbols.len() {
            bits.push(false);
            continue;
        }
        let bit = (data_symbols[sym_idx] >> (10 - bit_in_sym)) & 1;
        bits.push(bit != 0);
    }
    bits
}

/// Map the internal [`RaidHeaderFields`] (role as a u16 code) to the public
/// [`crate::RaidMeta`] (role as a [`crate::PlateRole`] enum). Role codes are
/// CRC-validated upstream in `parse_header`, so the match is total.
///
/// The public `index` is the **logical position in the `n+r` plate sequence**:
/// data plate `i` ⇒ `i`; ParityA ⇒ `n`; ParityB ⇒ `n+1`. (The H1 wire `index`
/// field carries the `P₂` α-exponent for a data plate — equal to `i` — and a `0`
/// placeholder for a parity plate, whose identity is its role.)
fn raid_meta(rf: &RaidHeaderFields) -> crate::RaidMeta {
    let (role, index) = match rf.role {
        RAID_ROLE_PARITY_A => (crate::PlateRole::ParityA, rf.n),
        RAID_ROLE_PARITY_B => (crate::PlateRole::ParityB, rf.n + 1),
        _ => (crate::PlateRole::Data, rf.index),
    };
    crate::RaidMeta {
        n: rf.n,
        role,
        index,
        array_id: rf.array_id,
    }
}

/// Build the public [`Decoded`] from a recovered payload.
fn make_decoded(
    header: &ParsedHeader,
    payload: Vec<u8>,
    truncated: bool,
    erasures: &[usize],
    extra_erasures: usize,
) -> Decoded {
    Decoded {
        kind: header.kind,
        payload,
        payload_bits: header.payload_bits,
        truncated,
        repair: RepairSummary {
            erasures_filled: erasures.len() + extra_erasures,
        },
        raid: header.raid.as_ref().map(raid_meta),
    }
}
