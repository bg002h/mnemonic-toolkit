//! ms-codec error taxonomy. Variants mirror SPEC §4 decoder validity rules
//! plus the encoder-side validation surface from SPEC §3.5 / §3.5.1.

use std::fmt;

/// ms-codec error type.
///
/// `Debug` is hand-implemented (NOT derived) so that neither `Display` nor
/// `Debug` of this type can echo ≥8 contiguous chars of secret input
/// (`ms-codec-error-display-echoes-input`, 0.4.4). A derived `Debug` would
/// print every field — including the raw input carried by the inner
/// `crate::codex32::Error` (`InvalidChecksum`/`MismatchedHrp`/`MismatchedId`) and the
/// `WrongHrp.got` HRP — so it is replaced by a delegation to the sanitized
/// `Display`. This is load-bearing for downstream `#[derive(Debug)]` wrappers
/// (toolkit `ToolkitError`/`CliError`) whose `{:?}` transitively renders this
/// type via panics / `expect` / logging. Replacing the derive is NOT a SemVer
/// break (the `Debug` IMPL is preserved; its exact output is not contractual).
#[non_exhaustive]
pub enum Error {
    /// codex32 parse / checksum failure, delegated from the vendored codex32
    /// module (`crate::codex32::Error`; Cycle-B inlined the formerly-external
    /// `codex32 = "=0.1.0"` crate). The variant NAME and field SHAPE are
    /// unchanged from the external-dep era; only the inner type's crate-path
    /// moved (`codex32::Error` → `crate::codex32::Error`) — a pre-1.0 breaking
    /// change for any downstream matcher that named the old extern type (the
    /// toolkit's friendly.rs is exactly such a matcher → its paired bump).
    Codex32(crate::codex32::Error),
    /// Mnem wordlist-language byte was not in the valid range 0..=9 (SPEC v0.2 §3).
    MnemUnknownLanguage(u8),
    /// HRP was not "ms" (SPEC §4 rule 2).
    WrongHrp {
        /// The HRP that was observed.
        got: String,
    },
    /// Threshold was not 0 (SPEC §4 rule 3).
    ThresholdNotZero {
        /// The threshold-position byte (ASCII digit) that was observed.
        got: u8,
    },
    /// Share-index was not 's' — BIP-93 requires 's' for threshold=0 (SPEC §4 rule 4).
    ShareIndexNotSecret {
        /// The share-index character that was observed.
        got: char,
    },
    /// Tag bytes were not in the codex32 alphabet (SPEC §4 rule 5).
    TagInvalidAlphabet {
        /// The 4-byte id-field bytes that failed alphabet validation.
        got: [u8; 4],
    },
    /// Tag was structurally valid but not in RESERVED_TAG_TABLE (SPEC §4 rule 6).
    UnknownTag {
        /// The 4-byte tag that was not recognized.
        got: [u8; 4],
    },
    /// Tag was in RESERVED_TAG_TABLE but reserved-not-emitted in v0.1 (SPEC §4 rule 7,
    /// SPEC §3.5.1 encoder symmetry).
    ReservedTagNotEmittedInV01 {
        /// The 4-byte reserved tag (one of seed/xprv/mnem/prvk in v0.1).
        got: [u8; 4],
    },
    /// Reserved-prefix byte was not 0x00 (SPEC §4 rule 8).
    ReservedPrefixViolation {
        /// The non-zero prefix byte that was observed.
        got: u8,
    },
    /// Total string length was outside the v0.1 emittable set (SPEC §4 rule 9).
    UnexpectedStringLength {
        /// The total string length that was observed.
        got: usize,
        /// The set of v0.1-emittable lengths.
        allowed: &'static [usize],
    },
    /// Payload byte length did not match the tag's spec (SPEC §3.5, §4 rule 10).
    PayloadLengthMismatch {
        /// The 4-byte tag whose length set was checked against.
        tag: [u8; 4],
        /// The set of valid byte lengths for this tag.
        expected: &'static [usize],
        /// The observed payload byte length (after stripping the prefix byte).
        got: usize,
    },
    /// BCH error-correction (`bch_decode`) reported the input is uncorrectable
    /// — the number of symbol errors exceeds the regular code's `t = 4`
    /// correction capacity (singleton bound `d = 8`). Surfaced by
    /// [`crate::decode_with_correction`] when `bch_decode::decode_regular_errors`
    /// returns `None`, or when a post-correction re-verification step fails
    /// (catches pathological 5+-error patterns that fool the decoder into
    /// producing a "consistent" but invalid locator). Added v0.2.0 per plan
    /// §1 D29 + §2.B.2.
    ///
    /// `bound = 8` is the BCH(93,80,8) singleton bound. ms1 is single-chunk
    /// only — no `chunk_index` field (cf. md-codec's `TooManyErrors` which
    /// carries chunk-set context).
    TooManyErrors {
        /// Singleton bound for the BCH regular code (always 8).
        bound: u8,
    },

    // --- v0.2 K-of-N share variants (SPEC_ms_v0_2_kofn §2) ---
    //
    // Inserted alphabetically AMONG THEMSELVES (the pre-existing v0.1 variants
    // above are NOT retro-sorted — mirrors the toolkit's
    // `error-rs-retroactive-alphabetical-sort` deferral). These carry `Display`
    // arms only: `ms_codec::Error` has no `exit_code`/`kind` methods — the
    // exit-code/message mapping is ms-cli's `CliError` job.
    /// Share count `n` was outside the valid range for threshold `k` (need
    /// `k <= n <= 31`; there are exactly 31 valid non-`s` share indices).
    InvalidShareCount {
        /// The threshold `k` that was requested.
        k: u8,
        /// The share count `n` that was requested (out of range).
        n: usize,
    },
    /// Threshold `k` was not in the valid share range `2..=9`
    /// (`Threshold::ZERO` is the unshared single-string sentinel, a const).
    InvalidThreshold(u8),
    /// A single-string `decode` was handed one share of a K-of-N share-set
    /// (threshold char `2..9`). Use `ms combine` to recombine K shares.
    IsShareNotSingleString {
        /// The threshold char observed on the wire (`'2'..'9'`).
        threshold: char,
        /// The share-index char observed on the wire.
        index: char,
    },
    /// `combine_shares` was handed the secret-at-S (index `s`) as an input.
    /// The secret-at-S is the recovery target, never a combine input; codex32's
    /// `interpolate_at` would short-circuit on it and bypass validation (C1).
    SecretShareSuppliedToCombine,
    /// `combine_shares` was handed a same-id (same hrp/id/threshold/length) but
    /// cross-polynomial share set: the first `k` shares define one polynomial,
    /// but at least one EXTRA supplied share does not lie on it. Beyond-BIP-93
    /// defense-in-depth (codex32 K-of-N carries no digest share) — without this
    /// check the combine would silently return a WRONG secret. The supplied
    /// shares are not all from the same split.
    InconsistentShareSet,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // SECRET-LEAK BOUND (ms-codec-error-display-echoes-input, 0.4.4):
            // codex32-0.1.0's `Error` is `derive(Debug)`-only (NO `Display`), so
            // a manual variant match is mandatory. Exactly 3 of its 16 variants
            // carry the raw input string and MUST be intercepted EXPLICITLY (no
            // generic `{:?}` fallback for them, so a future codex32 bump can't
            // silently route a new leaky variant through):
            //   * `InvalidChecksum { checksum, string }` — `string` is the FULL
            //     input; `checksum` is a `&'static "short"/"long"` (safe).
            //   * `MismatchedHrp(String, String)` — both dropped.
            //   * `MismatchedId(String, String)` — both dropped.
            // (MismatchedHrp/Id are provenance-bounded SAFE for ms1 — from
            // `interpolate_at` on valid Codex32String, hrp="ms"/id=4 chars — but
            // dropped for robustness.) The other 13 carry only
            // `&'static str`/`usize`/`char`/`Case`/`Fe`/`field::Error` (all
            // ≤1 echoed char < the 8-char window) and are rendered structurally
            // via `{:?}` on the inner error AFTER the 3 leaky arms are peeled off.
            Error::Codex32(e) => match e {
                crate::codex32::Error::InvalidChecksum { checksum, .. } => {
                    write!(f, "invalid {checksum} checksum (input withheld)")
                }
                crate::codex32::Error::MismatchedHrp(..) => {
                    write!(f, "mismatched HRP across shares")
                }
                crate::codex32::Error::MismatchedId(..) => {
                    write!(f, "mismatched ID across shares")
                }
                // Safe variants only reach here (the 3 leaky ones are peeled off
                // above), so `{:?}` of the inner error echoes no secret window.
                safe => write!(f, "codex32 parse error: {safe:?}"),
            },
            Error::MnemUnknownLanguage(code) => {
                write!(f, "unknown mnem wordlist-language code: {0}", code)
            }
            Error::WrongHrp { got } => write!(f, "wrong HRP: got {:?}, expected \"ms\"", got),
            Error::ThresholdNotZero { got } => {
                write!(
                    f,
                    "threshold not 0 (got '{}'); v0.1 is single-string only",
                    *got as char
                )
            }
            Error::ShareIndexNotSecret { got } => {
                write!(
                    f,
                    "share-index not 's' (got '{}'); BIP-93 requires 's' for threshold=0",
                    got
                )
            }
            Error::TagInvalidAlphabet { got } => {
                write!(f, "tag bytes not in codex32 alphabet: {:?}", got)
            }
            Error::UnknownTag { got } => write!(
                f,
                "unknown tag {:?}; not a member of RESERVED_TAG_TABLE",
                std::str::from_utf8(got).unwrap_or("<non-utf8>")
            ),
            Error::ReservedTagNotEmittedInV01 { got } => write!(
                f,
                "tag {:?} reserved-not-emitted in v0.1; deferred to v0.2+",
                std::str::from_utf8(got).unwrap_or("<non-utf8>")
            ),
            Error::ReservedPrefixViolation { got } => {
                write!(f, "reserved-prefix byte was 0x{:02x}, expected 0x00", got)
            }
            Error::UnexpectedStringLength { got, allowed } => {
                write!(f, "string length {} outside v0.1 set {:?}", got, allowed)
            }
            Error::PayloadLengthMismatch { tag, expected, got } => write!(
                f,
                "tag {:?} payload length {} not in expected set {:?}",
                std::str::from_utf8(tag).unwrap_or("<non-utf8>"),
                got,
                expected
            ),
            Error::TooManyErrors { bound } => {
                write!(f, "more than {} errors; uncorrectable", bound)
            }
            Error::InvalidShareCount { k, n } => write!(
                f,
                "invalid share count n={} for threshold k={}; require k <= n <= 31",
                n, k
            ),
            Error::InvalidThreshold(k) => write!(
                f,
                "invalid threshold {}; K-of-N shares require k in 2..=9",
                k
            ),
            Error::IsShareNotSingleString { threshold, index } => write!(
                f,
                "this is one share of a K-of-N set (threshold '{}', index '{}'); \
                 use `ms combine` to recombine K shares",
                threshold, index
            ),
            Error::SecretShareSuppliedToCombine => write!(
                f,
                "the secret share (index 's') cannot be supplied to combine; \
                 supply only distributed shares (the secret is the recovery target)"
            ),
            Error::InconsistentShareSet => write!(
                f,
                "one or more shares are not from the same split; the supplied \
                 shares do not all lie on a single Shamir polynomial"
            ),
        }
    }
}

impl fmt::Debug for Error {
    /// Hand-rolled to match `Display`'s sanitization — see the type doc.
    /// Delegates to the (non-echoing) `Display` so the leaky inner
    /// `crate::codex32::Error` String fields and the (already construction-bounded)
    /// `WrongHrp.got` can never reach a derived field dump. Wrapped as
    /// `Error("…")` so the output still reads as a debug value.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error(\"{self}\")")
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        // The vendored crate::codex32::Error does not impl std::error::Error
        // (it didn't upstream either); the chain stops here.
        None
    }
}

impl From<crate::codex32::Error> for Error {
    fn from(e: crate::codex32::Error) -> Self {
        Error::Codex32(e)
    }
}

/// Result alias for ms-codec.
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod no_echo_tests {
    //! Red-first leak tests for the `ms-codec-error-display-echoes-input` fix
    //! (0.4.4). Neither `Display` NOR `Debug` of `ms_codec::Error` may contain
    //! any ≥8-char contiguous window of secret input, for ALL reachable inputs.
    //! These tests construct/trigger the three leaky surfaces (codex32
    //! `InvalidChecksum`/`MismatchedHrp`/`MismatchedId` + `WrongHrp`) and assert
    //! the rendered strings carry no 8-char window of the secret.
    use super::*;
    use crate::{decode, decode_with_correction};

    /// The contiguous-window length the fuzz oracle scans (8 chars = 40 bits
    /// over the 32-symbol codex32 alphabet). Mirror it here.
    const WINDOW: usize = 8;

    /// Does `haystack` contain any ≥WINDOW-char contiguous window of `needle`?
    fn contains_window(haystack: &str, needle: &str) -> Option<String> {
        let n: Vec<char> = needle.chars().collect();
        if n.len() < WINDOW {
            return None;
        }
        for w in n.windows(WINDOW) {
            let win: String = w.iter().collect();
            if haystack.contains(&win) {
                return Some(win);
            }
        }
        None
    }

    /// Assert neither Display nor Debug of `e` carries an 8-char window of
    /// `secret`.
    fn assert_no_leak(e: &Error, secret: &str, label: &str) {
        let display = format!("{e}");
        let debug = format!("{e:?}");
        if let Some(hit) = contains_window(&display, secret) {
            panic!(
                "{label}: Display leaked an {WINDOW}-char window of the secret: \
                 hit={hit:?}\n  rendered: {display:?}"
            );
        }
        if let Some(hit) = contains_window(&debug, secret) {
            panic!(
                "{label}: Debug leaked an {WINDOW}-char window of the secret: \
                 hit={hit:?}\n  rendered: {debug:?}"
            );
        }
    }

    /// A 50-char codex32-alphabet "secret" data-part for the constructed cases.
    const SECRET_50: &str = "qpzry9x8gf2tvdw0s3jn54khce6mua7lqpzry9x8gf2tvdw0s3";

    /// (1) `Codex32(InvalidChecksum)` reached via a real `decode` — take a
    /// valid 50-char ms1 string and flip one data char so the checksum fails.
    /// codex32-0.1.0's `InvalidChecksum.string` carries the FULL input, so
    /// pre-fix this leaks the whole secret data-part.
    #[test]
    fn codex32_invalid_checksum_from_decode_does_not_leak() {
        // Verified-valid 50-char ms1 vector (decodes OK at HEAD).
        let valid = "ms10entrsqgqqc83yukgh23xkvmp59xf2eldpk4cdrq2y4h82yz";
        assert!(
            decode(valid).is_ok(),
            "fixture must decode: {:?}",
            decode(valid)
        );
        let mut chars: Vec<char> = valid.chars().collect();
        // Flip a data char (well past the `ms10entrs` prefix) → checksum fails.
        let i = 14;
        chars[i] = if chars[i] == 'q' { 'p' } else { 'q' };
        let flipped: String = chars.iter().collect();
        let e = decode(&flipped).unwrap_err();
        // Must be the leaky Codex32(InvalidChecksum) arm.
        assert!(
            matches!(
                e,
                Error::Codex32(crate::codex32::Error::InvalidChecksum { .. })
            ),
            "expected Codex32(InvalidChecksum), got {e:?}"
        );
        // The secret is the data-part of the flipped string (after `ms1`).
        let secret = flipped.strip_prefix("ms1").unwrap();
        assert_no_leak(&e, secret, "codex32_invalid_checksum_from_decode");
    }

    /// (1b) `Codex32(InvalidChecksum)` constructed directly with a 50-char
    /// secret string — the construction-side red-first cell.
    #[test]
    fn codex32_invalid_checksum_constructed_does_not_leak() {
        let e = Error::Codex32(crate::codex32::Error::InvalidChecksum {
            checksum: "short",
            string: format!("ms1{SECRET_50}"),
        });
        assert_no_leak(&e, SECRET_50, "codex32_invalid_checksum_constructed");
    }

    /// (2) `WrongHrp` reached via a real `decode_with_correction` of a
    /// no-separator 50-char secret-shaped input — pre-fix the whole input
    /// rides in `got` (this is the path `parse_ms1_symbols` reaches directly;
    /// `decode`/`inspect` length/codex32-validate first and route a
    /// codex32-alphabet 50-char string to the checksum path instead).
    #[test]
    fn wrong_hrp_no_separator_does_not_leak() {
        // 50 codex32-alphabet chars, NO `'1'` separator → the whole string is
        // the observed HRP at the construction site (capped to 4 by the fix).
        let secret = "qpzry9x8gf2tvdw0s3jn54khce6mua7lqpzry9x8gf2tvdw0s3";
        assert!(!secret.contains('1'), "fixture must have no '1' separator");
        let e = decode_with_correction(secret).unwrap_err();
        assert!(
            matches!(e, Error::WrongHrp { .. }),
            "expected WrongHrp, got {e:?}"
        );
        assert_no_leak(&e, secret, "wrong_hrp_no_separator");
    }

    /// (3) `Codex32(MismatchedHrp)` constructed directly with secret strings.
    #[test]
    fn codex32_mismatched_hrp_does_not_leak() {
        let e = Error::Codex32(crate::codex32::Error::MismatchedHrp(
            SECRET_50.to_string(),
            SECRET_50.to_string(),
        ));
        assert_no_leak(&e, SECRET_50, "codex32_mismatched_hrp");
    }

    /// (4) `Codex32(MismatchedId)` constructed directly with secret strings.
    #[test]
    fn codex32_mismatched_id_does_not_leak() {
        let e = Error::Codex32(crate::codex32::Error::MismatchedId(
            SECRET_50.to_string(),
            SECRET_50.to_string(),
        ));
        assert_no_leak(&e, SECRET_50, "codex32_mismatched_id");
    }
}
