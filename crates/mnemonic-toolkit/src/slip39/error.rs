//! Library-local error type for the SLIP-39 module.
//!
//! Per the v0.11.0 `FinalWordError` / v0.12.0 `SeedXorError` precedent
//! (tracked under FOLLOWUP `library-error-and-language-surface-promotion`):
//! the library surfaces a dedicated `Slip39Error`, and the CLI handler at
//! `src/cmd/slip39.rs` (P2) wraps each variant into
//! `ToolkitError::BadInput(...)` formatted per the SPEC §B.2.5 stderr
//! stems.
//!
//! Coverage: 21 library variants spanning 21 of the 23 SPEC §2.5
//! refusal classes (post v0.13.0 P1c-E.1 expansion). The 2 CLI-only
//! rows (17, 18) — `--from` variant syntactically invalid; multi-stdin
//! contention across `--share` / `--from` / `--passphrase-stdin` — are
//! rejected at the CLI boundary before reaching the library. The fold
//! that keeps the variant count at 21 (instead of 22) is rows 4 and 5
//! (both group-spec policy refusals) collapsing into `BadGroupSpec`;
//! the CLI handler distinguishes them at the `ToolkitError` mapping
//! layer based on the carried (n, t) values (row 5 = `n == 1 && t == 1`;
//! row 4 = all other group-spec violations).
//!
//! Display messages here are diagnostic, not user-facing — the CLI
//! handler re-renders each variant into the SPEC §B.2.5 stem byte by
//! byte. Carried data fields are sized to provide the CLI handler
//! everything it needs without re-parsing.

/// Errors returned by the SLIP-39 library surface (`slip39_split`,
/// `slip39_combine`, `parse_slip39_share`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Slip39Error {
    /// Master secret presented as a BIP-39 phrase whose word count is
    /// not in `{12, 15, 18, 21, 24}`. Carries the actual word count.
    BadPhraseWordCount(usize),

    /// Master secret presented as hex whose decoded byte length is not
    /// in `{16, 20, 24, 28, 32}` (also covers odd-length hex / non-hex
    /// decode failures, mapped to the byte count the CLI would report).
    BadEntropyByteLength(usize),

    /// `--group-threshold` G outside `1..=group_count`.
    BadGroupThreshold { got: u8, group_count: u8 },

    /// `--group N,T` violates `1 <= T <= N <= 16` OR is the
    /// `1,1` toolkit-policy refusal. The library does not distinguish
    /// those two sub-cases; the CLI handler renders the appropriate
    /// stem per the SPEC §B.2.5 row 4 vs row 5.
    BadGroupSpec { group_idx: usize, n: u8, t: u8 },

    /// `--iteration-exponent` E outside `0..=15` (4-bit field).
    BadIterationExponent(u8),

    /// At combine time: shares disagree on the 15-bit identifier.
    IdentifierMismatch,

    /// At combine time: shares disagree on iteration_exponent.
    IterationExponentMismatch,

    /// At combine time: shares disagree on group_threshold.
    GroupThresholdMismatch,

    /// At combine time: shares disagree on group_count.
    GroupCountMismatch,

    /// At combine time: shares within a single group disagree on
    /// member_threshold (each group's member_threshold is fixed at
    /// split time and must match across the group's shares).
    MemberThresholdMismatch,

    /// RS1024 BCH checksum failure on the share at `share_idx`
    /// (0-based position within the combine input).
    InvalidChecksum { share_idx: usize },

    /// A word in the share at `share_idx` at offset `word_idx` is not
    /// in the SLIP-39 1024-word wordlist.
    UnknownWord { share_idx: usize, word_idx: usize },

    /// 4-byte `HMAC-SHA256(key=R, msg=decrypted-S)` mismatch — the
    /// reconstructed master secret failed digest verification. Most
    /// commonly: wrong passphrase, or a substituted share whose
    /// metadata matches but value bytes diverge.
    DigestVerificationFailed,

    /// Insufficient shares for the group at `group_idx`: need
    /// `needed` (= member_threshold), got `got`.
    InsufficientShares { group_idx: u8, needed: u8, got: u8 },

    /// Two shares in the same group share the same member_index
    /// (which would either be a duplicate or a Shamir-incompatible
    /// collision on the x-coordinate).
    DuplicateMemberIndex { group_idx: u8, member_idx: u8 },

    /// The encoded share at `share_idx` has non-zero padding bits in
    /// the final partial 10-bit word (encoding violation per
    /// SLIP-0039 §3.1).
    InvalidPadding { share_idx: usize },

    /// `slip39_combine` called with an empty share list. Distinct from
    /// `InsufficientShares` (which fires when a non-empty list is short
    /// of the required threshold for some group).
    EmptyShares,

    /// At combine time: the share at `share_idx` has a value-byte length
    /// not in `{16, 20, 24, 28, 32}`. The parse layer (correctly) does
    /// not enforce master-secret length per-share; combine checks each
    /// parsed share at entry. Pins vectors.json #40.
    InvalidShareValueLength { share_idx: usize, got: usize },

    /// At combine time: shares disagree on value-byte length. The
    /// SLIP-0039 spec requires all shares of one master secret to share
    /// the same length (`len(EMS) == len(master_secret)`).
    ShareValueLengthMismatch,

    /// At combine time: shares disagree on the `extendable` bit. The
    /// two ext-axes are orthogonal and use different salt_prefixes in
    /// the Feistel layer, so mixed-axis combines are structurally
    /// unrecoverable.
    ExtendableMismatch,

    /// Parse-time refusal: the share at `share_idx` encodes
    /// `group_count < group_threshold`, which is structurally
    /// inconsistent (the spec requires the threshold not to exceed the
    /// number of groups). Mirrors `python-shamir-mnemonic`
    /// `share.py:216-219` @ commit `17fcce14`. Pins vectors.json #10, #29.
    GroupThresholdExceedsCount { share_idx: usize, threshold: u8, count: u8 },
}

impl std::fmt::Display for Slip39Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadPhraseWordCount(got) => write!(
                f,
                "slip39: master-secret phrase must be 12/15/18/21/24 words; got {got}"
            ),
            Self::BadEntropyByteLength(got) => write!(
                f,
                "slip39: master-secret entropy must decode to 16/20/24/28/32 bytes; got {got}"
            ),
            Self::BadGroupThreshold { got, group_count } => write!(
                f,
                "slip39: --group-threshold must be in 1..={group_count}; got {got}"
            ),
            Self::BadGroupSpec { group_idx, n, t } => write!(
                f,
                "slip39: group {group_idx} (N={n}, T={t}) violates 1 <= T <= N <= 16 \
                 (or is the policy-refused 1-of-1 shape)"
            ),
            Self::BadIterationExponent(e) => write!(
                f,
                "slip39: --iteration-exponent must be 0..=15 (4-bit field); got {e}"
            ),
            Self::IdentifierMismatch => write!(
                f,
                "slip39: shares disagree on identifier"
            ),
            Self::IterationExponentMismatch => write!(
                f,
                "slip39: shares disagree on iteration_exponent"
            ),
            Self::GroupThresholdMismatch => write!(
                f,
                "slip39: shares disagree on group_threshold"
            ),
            Self::GroupCountMismatch => write!(
                f,
                "slip39: shares disagree on group_count"
            ),
            Self::MemberThresholdMismatch => write!(
                f,
                "slip39: shares within a group disagree on member_threshold"
            ),
            Self::InvalidChecksum { share_idx } => write!(
                f,
                "slip39: share at position {share_idx} has invalid RS1024 checksum"
            ),
            Self::UnknownWord { share_idx, word_idx } => write!(
                f,
                "slip39: share at position {share_idx}: word at index {word_idx} \
                 not in SLIP-39 wordlist"
            ),
            Self::DigestVerificationFailed => write!(
                f,
                "slip39: reconstructed master digest mismatch (wrong passphrase or \
                 substituted share)"
            ),
            Self::InsufficientShares { group_idx, needed, got } => write!(
                f,
                "slip39: insufficient shares for group {group_idx}: need {needed}, got {got}"
            ),
            Self::DuplicateMemberIndex { group_idx, member_idx } => write!(
                f,
                "slip39: duplicate member index {member_idx} in group {group_idx}"
            ),
            Self::InvalidPadding { share_idx } => write!(
                f,
                "slip39: share at position {share_idx} has non-zero padding bits \
                 (encoding violation)"
            ),
            Self::EmptyShares => write!(
                f,
                "slip39: combine called with empty share list"
            ),
            Self::InvalidShareValueLength { share_idx, got } => write!(
                f,
                "slip39: share at position {share_idx} has value length {got} \
                 (must be 16/20/24/28/32 bytes)"
            ),
            Self::ShareValueLengthMismatch => write!(
                f,
                "slip39: shares disagree on value length"
            ),
            Self::ExtendableMismatch => write!(
                f,
                "slip39: shares disagree on the extendable (ext) bit"
            ),
            Self::GroupThresholdExceedsCount { share_idx, threshold, count } => write!(
                f,
                "slip39: share at position {share_idx}: group_threshold {threshold} \
                 exceeds group_count {count}"
            ),
        }
    }
}

impl std::error::Error for Slip39Error {}
