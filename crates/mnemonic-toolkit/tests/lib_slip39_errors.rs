//! v0.13.0 P1c — library tests for SLIP-39 `Slip39Error` enum.
//!
//! Per SPEC §2.5 (23 refusal classes; 21 library-mappable + 2 CLI-only;
//! the v0.13.0 P1c-E.1 expansion added rows 19–23 atop the original 18).
//! Each variant carries the diagnostic info the CLI handler needs to
//! synthesize the SPEC §2.5 stderr stem at P2.
//!
//! Coverage matrix:
//!   - all 21 library variants constructible
//!   - Display non-empty for each (CLI maps to `ToolkitError::BadInput`)
//!   - `std::error::Error` implemented
//!   - PartialEq + Eq + Clone + Debug derived (test ergonomics +
//!     vector-harness assertions at P1c-E.2 G1)
//!
//! No Display-stem byte-pinning: the SPEC §2.5 stems are CLI-layer
//! surfaces, pinned at P2's `cli_slip39_refusals.rs`. Here we pin only
//! that each variant carries enough diagnostic info to drive the CLI
//! mapping (e.g., InsufficientShares carries group_idx + needed + got).

use mnemonic_toolkit::slip39::Slip39Error;

// ============================================================================
// Variant constructibility + Display non-emptiness
// ============================================================================

#[test]
fn variant_bad_phrase_word_count() {
    let e = Slip39Error::BadPhraseWordCount(13);
    assert!(!format!("{e}").is_empty());
    assert!(format!("{e}").contains("13"));
}

#[test]
fn variant_bad_entropy_byte_length() {
    let e = Slip39Error::BadEntropyByteLength(17);
    assert!(!format!("{e}").is_empty());
    assert!(format!("{e}").contains("17"));
}

#[test]
fn variant_bad_group_threshold() {
    let e = Slip39Error::BadGroupThreshold {
        got: 4,
        group_count: 3,
    };
    let msg = format!("{e}");
    assert!(msg.contains('4'));
    assert!(msg.contains('3'));
}

#[test]
fn variant_bad_group_spec() {
    let e = Slip39Error::BadGroupSpec {
        group_idx: 1,
        n: 2,
        t: 3,
    };
    let msg = format!("{e}");
    assert!(msg.contains('1'));
    assert!(msg.contains('2'));
    assert!(msg.contains('3'));
}

#[test]
fn variant_bad_iteration_exponent() {
    let e = Slip39Error::BadIterationExponent(16);
    assert!(format!("{e}").contains("16"));
}

#[test]
fn variant_identifier_mismatch() {
    let e = Slip39Error::IdentifierMismatch;
    assert!(!format!("{e}").is_empty());
}

#[test]
fn variant_iteration_exponent_mismatch() {
    let e = Slip39Error::IterationExponentMismatch;
    assert!(!format!("{e}").is_empty());
}

#[test]
fn variant_group_threshold_mismatch() {
    let e = Slip39Error::GroupThresholdMismatch;
    assert!(!format!("{e}").is_empty());
}

#[test]
fn variant_group_count_mismatch() {
    let e = Slip39Error::GroupCountMismatch;
    assert!(!format!("{e}").is_empty());
}

#[test]
fn variant_member_threshold_mismatch() {
    let e = Slip39Error::MemberThresholdMismatch;
    assert!(!format!("{e}").is_empty());
}

#[test]
fn variant_invalid_checksum() {
    let e = Slip39Error::InvalidChecksum { share_idx: 2 };
    assert!(format!("{e}").contains('2'));
}

#[test]
fn variant_unknown_word() {
    let e = Slip39Error::UnknownWord {
        share_idx: 0,
        word_idx: 7,
    };
    let msg = format!("{e}");
    assert!(msg.contains('7'));
}

#[test]
fn variant_digest_verification_failed() {
    let e = Slip39Error::DigestVerificationFailed;
    assert!(!format!("{e}").is_empty());
}

#[test]
fn variant_insufficient_shares() {
    let e = Slip39Error::InsufficientShares {
        group_idx: 1,
        needed: 3,
        got: 2,
    };
    let msg = format!("{e}");
    assert!(msg.contains('1'));
    assert!(msg.contains('3'));
    assert!(msg.contains('2'));
}

#[test]
fn variant_duplicate_member_index() {
    let e = Slip39Error::DuplicateMemberIndex {
        group_idx: 0,
        member_idx: 4,
    };
    let msg = format!("{e}");
    assert!(msg.contains('4'));
}

#[test]
fn variant_invalid_padding() {
    let e = Slip39Error::InvalidPadding { share_idx: 3 };
    assert!(format!("{e}").contains('3'));
}

// ============================================================================
// New variants — P1c-E.1 driver-scope expansion (plan §8).
//
//   - EmptyShares: `slip39_combine` called with `&[]` (R0 §3.4 step 1).
//   - InvalidShareValueLength: per-share value-byte-length sanity at
//     combine entry (R0 I2; pins vector #40).
//   - ShareValueLengthMismatch: cross-share value-byte-length divergence
//     at combine (R0 I1; 6th invariant beyond the 5 metadata fields).
//   - ExtendableMismatch: cross-share extendable-bit divergence at
//     combine (R0 I1; orthogonal to IdentifierMismatch).
//   - GroupThresholdExceedsCount: parse-time refusal when `group_count <
//     group_threshold` on a single share (R0 I3; pins vectors #10 / #29).
// ============================================================================

#[test]
fn variant_empty_shares() {
    let e = Slip39Error::EmptyShares;
    assert!(!format!("{e}").is_empty());
}

#[test]
fn variant_invalid_share_value_length() {
    let e = Slip39Error::InvalidShareValueLength {
        share_idx: 2,
        got: 19,
    };
    let msg = format!("{e}");
    assert!(msg.contains('2'));
    assert!(msg.contains("19"));
}

#[test]
fn variant_share_value_length_mismatch() {
    let e = Slip39Error::ShareValueLengthMismatch;
    assert!(!format!("{e}").is_empty());
}

#[test]
fn variant_extendable_mismatch() {
    let e = Slip39Error::ExtendableMismatch;
    assert!(!format!("{e}").is_empty());
}

#[test]
fn variant_group_threshold_exceeds_count() {
    let e = Slip39Error::GroupThresholdExceedsCount {
        share_idx: 0,
        threshold: 3,
        count: 2,
    };
    let msg = format!("{e}");
    assert!(msg.contains('3'));
    assert!(msg.contains('2'));
}

// ============================================================================
// Trait impls
// ============================================================================

#[test]
fn implements_std_error_error() {
    fn assert_error<E: std::error::Error>(_e: &E) {}
    let e = Slip39Error::IdentifierMismatch;
    assert_error(&e);
}

#[test]
fn implements_debug_clone_partialeq_eq() {
    let a = Slip39Error::BadPhraseWordCount(13);
    let b = a.clone();
    assert_eq!(a, b);
    let _dbg = format!("{a:?}");
}

#[test]
fn variants_with_carried_data_compare_by_value() {
    let a = Slip39Error::BadPhraseWordCount(13);
    let b = Slip39Error::BadPhraseWordCount(14);
    assert_ne!(a, b);
    let c = Slip39Error::InsufficientShares {
        group_idx: 1,
        needed: 3,
        got: 2,
    };
    let d = Slip39Error::InsufficientShares {
        group_idx: 1,
        needed: 3,
        got: 2,
    };
    let e = Slip39Error::InsufficientShares {
        group_idx: 1,
        needed: 3,
        got: 1,
    };
    assert_eq!(c, d);
    assert_ne!(c, e);
}

#[test]
fn unit_variants_compare_by_discriminant() {
    assert_eq!(
        Slip39Error::IdentifierMismatch,
        Slip39Error::IdentifierMismatch
    );
    assert_ne!(
        Slip39Error::IdentifierMismatch,
        Slip39Error::IterationExponentMismatch
    );
}
