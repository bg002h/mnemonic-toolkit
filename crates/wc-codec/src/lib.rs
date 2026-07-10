//! `wc-codec` — the **Word-Card value engine** for the m-format constellation.
//!
//! This crate implements the codec-agnostic RS / RAID / sync / word engine that
//! turns a `(SourceKind, payload, payload_bits)` triple into an engravable BIP-39
//! word sequence and back (see `design/IMPLEMENTATION_PLAN_word_card_encoding.md`).
//!
//! **Phases present:**
//! - [`field`]: `GF(2^11)` arithmetic with the frozen primitive polynomial
//!   `x^11 + x^2 + 1` and primitive element `α = x` (plan §3);
//! - [`wordmap`]: the BIP-39 English symbol ↔ word map (symbol value == 11-bit
//!   index), sourced from the `bip39` crate as the single source of truth;
//! - [`regroup`]: bit-precise MSB-first 8 ↔ 11 regrouping (plan §4.1);
//! - [`pad`]: the frozen stripe zero-padding rule (plan §4.1 / M4);
//! - [`rs`] (**P2**): the systematic evaluation-form Reed–Solomon value layer
//!   — encode (interpolate + evaluate), decode (Gao partial-GCD with erasure
//!   puncturing), append-only prefix-extensible parity (plan §3 / §4.1).
//! - [`sync`] (**P3**): the structural sync / checkpoint layer — checkpoint word
//!   codec (marker + block-index mod 8 + CRC-5), `interleave` (insert
//!   checkpoints), and `sync_classify` (trichotomy + realignment + bounded
//!   single-deletion candidates / whole-block erasures), plan §4.3.
//! - **P4 (this layer):** the integrity tag + GEOM header + fixed-`U` ledger +
//!   stop-sign + the FULL end-to-end [`encode`] / [`decode`] pipeline — the
//!   integration phase. See [`pipeline`] for the engraved-stream layout and the
//!   ledger-OUTSIDE-the-RS-codeword decision (the #1 architectural point).
//!
//! RAID (P5) and the toolkit adapter (P6) are intentionally NOT present yet. The
//! toolkit crate does not depend on `wc-codec` until P6.

pub mod field;
pub mod pad;
pub mod pipeline;
mod poly;
pub mod raid;
pub mod regroup;
pub mod rs;
pub mod sync;
pub mod wordmap;

pub use pipeline::{
    decode, encode, DEFAULT_INTEGRITY_BITS, DEFAULT_U_SLOTS, MAX_INTEGRITY_BITS, MIN_INTEGRITY_BITS,
};
pub use raid::{raid_encode, raid_reconstruct, PlateRole, RaidPlate, RaidRecovery};

// ===========================================================================
// Public API surface (plan §6.1) — the consolidated codec-agnostic types.
// ===========================================================================

/// The source codec a Word-Card payload came from (plan §6.1). Encoded into the
/// H0 header word's 2-bit source-kind field (`00 = mk1`, `01 = md1`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceKind {
    /// An `mk1` compact-xpub payload (byte-aligned: `payload_bits = 8 * len`).
    Mk1Xpub,
    /// An `md1` descriptor payload (bit-precise `payload_bits`, generally NOT a
    /// multiple of 8).
    Md1Descriptor,
}

/// Encode-time options (plan §6.1). [`Default`] = `parity_words: 0`,
/// `integrity_bits: 44` ([`DEFAULT_INTEGRITY_BITS`]), `u_slots: 3`
/// ([`DEFAULT_U_SLOTS`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EncodeOpts {
    /// The number of appended RS parity words `m` (the repair budget; `2t+s ≤ m`).
    pub parity_words: usize,
    /// The integrity-tag bit width `t` (default 44; min 33). NON-LINEAR SHA-256
    /// truncation (plan §4.5).
    pub integrity_bits: u8,
    /// The number of reserved ledger slots `U` (default 3 = creation + 2
    /// upgrades; use 1 for tiny / never-upgrade cards). Each slot is `2` words.
    pub u_slots: u8,
}

impl Default for EncodeOpts {
    fn default() -> Self {
        EncodeOpts {
            parity_words: 0,
            integrity_bits: DEFAULT_INTEGRITY_BITS,
            u_slots: DEFAULT_U_SLOTS,
        }
    }
}

/// A small repair summary returned alongside a successful [`decode`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RepairSummary {
    /// The number of grid erasures the RS pass had to fill (located damage). `0`
    /// for a clean read.
    pub erasures_filled: usize,
}

/// The RAID metadata a Word-Card plate carries (plan §4.2 H1 + array-id), exposed
/// on [`Decoded::raid`] iff the plate's H0 `has-raid` bit was set (a RAID plate).
/// A solo card decodes with `raid == None`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RaidMeta {
    /// The number of data plates `n` in the array (`2..=32`).
    pub n: usize,
    /// The plate's role in the array.
    pub role: PlateRole,
    /// The plate's `index-in-array` (`0..n−1`) — the `P₂` α-exponent (plan §3).
    pub index: usize,
    /// The 22-bit array-id — `top22(SHA-256(array_id_seed ‖ SHA-256(payload-
    /// canonical)))`. The plate-matching aid that fixes stripe order (plan §3 /
    /// §4.2); the payload digest is folded in so two DIFFERENT wallets sharing a
    /// cosigner set get DIFFERENT ids, refusing a same-quorum plate mix instead
    /// of silently reconstructing a wrong xpub (constellation-eval **F2**).
    pub array_id: u32,
}

/// The result of a successful [`decode`] (plan §6.1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decoded {
    /// The recovered source-kind (from the H0 header).
    pub kind: SourceKind,
    /// The recovered canonical payload bytes (length `ceil(payload_bits / 8)`).
    pub payload: Vec<u8>,
    /// The exact payload BIT length (so md1's bit-precise payload round-trips).
    pub payload_bits: usize,
    /// `true` iff the card's recorded length (ledger / stop-sign) exceeds the
    /// number of words physically present — a chipped / lost tail (plan §4.4).
    pub truncated: bool,
    /// A small repair summary (erasures filled).
    pub repair: RepairSummary,
    /// The RAID plate metadata, present iff this card carried a RAID header
    /// (H0 `has-raid = 1`); `None` for a solo card (plan §4.2).
    pub raid: Option<RaidMeta>,
}

/// The public consolidated Word-Card error (plan §6.1). Variants are
/// **alphabetical** (plan / `CLAUDE.md` convention). It maps / wraps the
/// field-layer ([`regroup::RegroupError`]), value-layer ([`rs::RsError`]) and
/// sync-layer ([`sync::SyncError`]) errors plus the P4-new ones. No [`decode`]
/// input ever panics; malformed inputs return one of these.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WcError {
    /// The positional GEOM header-CRC did not verify — the geometry words are
    /// corrupt and cannot be trusted; refuse rather than decode garbage
    /// geometry (plan §4.2).
    HeaderCrcMismatch,
    /// The post-correction SHA-256 integrity tag did not match the recovered
    /// payload — an RS miscorrection onto a valid-but-WRONG codeword (caught at
    /// `≤ 2⁻ᵗ`), or an ambiguous single-deletion candidate set (plan §4.5 / §8
    /// step 5). The funds-safety net: refuse, NEVER return wrong payload.
    IntegrityMismatch,
    /// An encode/decode parameter was out of range (e.g. `integrity_bits` below
    /// the 33-bit floor, `u_slots == 0`, `payload_bits` exceeding the payload or
    /// the 16-bit GEOM capacity).
    InvalidParams,
    /// During RAID reconstruct the supplied plates did not form a single coherent
    /// array — mismatched array-ids (plates from two different wallets), or
    /// inconsistent `n` / duplicate index / inconsistent stripe width, OR a
    /// reconstructed set whose SPARE parity equation is inconsistent (a
    /// same-quorum chimera of plates from two DIFFERENT wallets sharing a cosigner
    /// set — the legacy F2 collision, caught by the spare-parity oracle). Refuse
    /// rather than silently mix unrelated plates (plan §4.2 / §7 P5 KAT 6; F2).
    RaidArrayMismatch,
    /// A RAID reconstruct had MORE than `r` plates missing — the MDS solve is
    /// underdetermined, so refuse rather than emit a wrong xpub (plan §7 P5
    /// KAT 8; the funds-safety net for the cross-plate layer).
    RaidUnrecoverable,
    /// A field-layer (8↔11 regroup) error surfaced while packing/unpacking.
    Regroup(regroup::RegroupError),
    /// A value-layer (Reed–Solomon) error surfaced while encoding/decoding.
    Rs(rs::RsError),
    /// A structural sync-layer refusal (ambiguous realignment, unbounded
    /// candidate set, multi-indel block, un-bridgeable checkpoint gap).
    Sync(sync::SyncError),
    /// The card's recorded length (ledger / stop-sign) exceeds the words
    /// present AND the missing tail cannot be structurally recovered — a chipped
    /// / lost tail beyond repair (plan §4.4). (Lesser truncation that is still
    /// recoverable is surfaced as the [`Decoded::truncated`] flag, not an error.)
    Truncated,
    /// The error/erasure weight exceeded the RS budget OR no single-deletion
    /// candidate produced a tag-valid payload — the card cannot be recovered
    /// (plan §8 step 3). Refuse rather than emit a wrong payload.
    Uncorrectable,
    /// A word was not in the BIP-39 English wordlist (after case-folding).
    UnknownWord,
}

impl core::fmt::Display for WcError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            WcError::HeaderCrcMismatch => {
                write!(
                    f,
                    "word-card: header-CRC mismatch (corrupt geometry) — refuse"
                )
            }
            WcError::IntegrityMismatch => write!(
                f,
                "word-card: integrity-tag mismatch (RS miscorrection / ambiguous) — refuse"
            ),
            WcError::InvalidParams => write!(f, "word-card: invalid encode/decode parameter"),
            WcError::RaidArrayMismatch => write!(
                f,
                "word-card: RAID plates do not form one coherent array (mismatched array-id / n / index, or inconsistent parity) — refuse"
            ),
            WcError::RaidUnrecoverable => write!(
                f,
                "word-card: RAID array has more than r plates missing — underdetermined, refuse"
            ),
            WcError::Regroup(e) => write!(f, "word-card: {e}"),
            WcError::Rs(e) => write!(f, "word-card: {e}"),
            WcError::Sync(e) => write!(f, "word-card: {e}"),
            WcError::Truncated => write!(f, "word-card: truncated card — tail lost beyond repair"),
            WcError::Uncorrectable => {
                write!(f, "word-card: uncorrectable (beyond RS budget) — refuse")
            }
            WcError::UnknownWord => write!(f, "word-card: word not in the BIP-39 English wordlist"),
        }
    }
}

impl std::error::Error for WcError {}

impl From<regroup::RegroupError> for WcError {
    fn from(e: regroup::RegroupError) -> Self {
        WcError::Regroup(e)
    }
}

impl From<rs::RsError> for WcError {
    fn from(e: rs::RsError) -> Self {
        // A pure budget overflow maps to the dedicated Uncorrectable variant so
        // callers can distinguish "card was too damaged" from a structural error.
        match e {
            rs::RsError::Uncorrectable => WcError::Uncorrectable,
            other => WcError::Rs(other),
        }
    }
}

impl From<sync::SyncError> for WcError {
    fn from(e: sync::SyncError) -> Self {
        WcError::Sync(e)
    }
}
