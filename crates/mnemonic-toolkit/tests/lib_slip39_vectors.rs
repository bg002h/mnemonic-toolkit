//! v0.13.0 P1c-E.2 G1 — SLIP-0039 spec test-vectors harness.
//!
//! Loads `tests/fixtures/slip39_vectors.json` (45 canonical vectors
//! from `python-shamir-mnemonic` @ commit `17fcce14`) via `include_str!`
//! and `serde_json::from_str`. One `#[test]` per vector via a small
//! macro for per-vector failure granularity (a single broken vector
//! does NOT mask the other 44).
//!
//! - **15 positive vectors** (1, 4, 17, 18, 19, 20, 23, 36, 37, 38,
//!   41, 42, 43, 44, 45): `slip39_combine(parse, b"TREZOR")` must
//!   succeed; recovered bytes must equal the hex-encoded master
//!   secret; the BIP-32 master xprv derived from the recovered bytes
//!   must equal the expected xprv string (SPEC §4 G1 — algorithm-
//!   correctness gate plus encoding-pathway gate).
//! - **30 negative vectors** (the remaining 30): the combine path
//!   (parse-time refusal OR combine-time refusal) must produce the
//!   specific `Slip39Error` variant per plan §4.1 (R0 N2 pre-pinned
//!   for the formerly-ambiguous rows #5 / #10 / #24 / #29 / #40).
//!
//! Passphrase: `b"TREZOR"`. This is Trezor's standard test passphrase
//! (the SLIP-0039 spec gives no default; the python ref test-suite +
//! the vendored `vectors.json` were all generated against this
//! string). NOT `b""`.
//!
//! Vector shape (each entry in vectors.json):
//!   `[description, mnemonics_list, hex_secret, expected_xprv]`
//! Positive vectors have non-empty `hex_secret` + `expected_xprv`;
//! negative vectors have both empty.
//!
//! Negative-variant mapping (plan §4.1 + R0 N2 fold):
//!
//! | # (1-based)      | Description fragment                                | Expected variant                                           |
//! |------------------|-----------------------------------------------------|------------------------------------------------------------|
//! | 2, 21            | invalid checksum                                    | InvalidChecksum { share_idx: 0 }                           |
//! | 3, 22            | invalid padding                                     | InvalidPadding { share_idx: 0 }                            |
//! | 5, 24            | basic sharing 2-of-3 (single share)                 | InsufficientShares { group_idx: 0, needed: 2, got: 1 }     |
//! | 6, 25            | different identifiers                               | IdentifierMismatch                                          |
//! | 7, 26            | different iteration exponents                       | IterationExponentMismatch                                   |
//! | 8, 27            | mismatching group thresholds                        | GroupThresholdMismatch                                      |
//! | 9, 28            | mismatching group counts                            | GroupCountMismatch                                          |
//! | 10, 29           | greater group threshold than group counts           | GroupThresholdExceedsCount { share_idx: 0, threshold: 2, count: 1 } (parse-time) |
//! | 11, 30           | duplicate member indices                            | DuplicateMemberIndex { group_idx: 0, member_idx: 2 } (pre-GREEN N1: both shares' share_params "academic always" → 0x21 → member_index=2, group_index=0) |
//! | 12, 31           | mismatching member thresholds                       | MemberThresholdMismatch                                     |
//! | 13, 32           | invalid digest                                      | DigestVerificationFailed                                    |
//! | 14, 15, 33, 34   | Insufficient number of groups                       | InsufficientShares { .. } (group-level; field values content-dependent) |
//! | 16, 35           | Threshold groups, insufficient members in one group | InsufficientShares { .. } (member-level; field values content-dependent) |
//! | 39               | insufficient length                                 | InvalidPadding { share_idx: 0 } (P1c-D fold)                |
//! | 40               | invalid master secret length                        | InvalidPadding { share_idx: 0 } (pre-GREEN C1 re-pin: 21 words → padding=140%16=12>8 → parser refuses at step 3 before combine's InvalidShareValueLength check can run; the variant is retained as defense-in-depth and exercised by a synthetic forged-share test in `mod tests`) |
//!
//! For the content-dependent fields (DuplicateMemberIndex's
//! member_idx, InsufficientShares' needed/got, InvalidShareValueLength's
//! got), this RED uses `matches!` shape pinning. Per R0 N2 the
//! pre-GREEN test-design review will pin exact values once the driver
//! impl exposes the concrete carried bytes.

use bitcoin::bip32::Xpriv;
use bitcoin::Network;
use mnemonic_toolkit::slip39::{
    parse_slip39_share, slip39_combine, GroupSpec, Slip39Error,
};
use serde::Deserialize;

const VECTORS_JSON: &str = include_str!("fixtures/slip39_vectors.json");
const PASSPHRASE: &[u8] = b"TREZOR";

#[derive(Deserialize)]
struct Vector(
    String,      // description
    Vec<String>, // mnemonics
    String,      // hex_secret (empty = negative)
    String,      // expected_xprv (empty = negative)
);

fn load() -> Vec<Vector> {
    serde_json::from_str(VECTORS_JSON)
        .expect("tests/fixtures/slip39_vectors.json must parse as Vec<Vector>")
}

// ============================================================================
// Negative-vector expectations
// ============================================================================

enum ExpectedNegative {
    /// Exact `Slip39Error` variant match — all fields fixed.
    Exact(Slip39Error),
    /// Variant-shape match only; carried fields are content-dependent
    /// and pre-GREEN review pins concrete values.
    Shape(fn(&Slip39Error) -> bool),
}

fn negative_expected(idx_1based: usize) -> ExpectedNegative {
    use ExpectedNegative::*;
    use Slip39Error::*;
    match idx_1based {
        2 | 21 => Exact(InvalidChecksum { share_idx: 0 }),
        3 | 22 => Exact(InvalidPadding { share_idx: 0 }),
        5 | 24 => Exact(InsufficientShares {
            group_idx: 0,
            needed: 2,
            got: 1,
        }),
        6 | 25 => Exact(IdentifierMismatch),
        7 | 26 => Exact(IterationExponentMismatch),
        8 | 27 => Exact(GroupThresholdMismatch),
        9 | 28 => Exact(GroupCountMismatch),
        10 | 29 => Exact(GroupThresholdExceedsCount {
            share_idx: 0,
            threshold: 2,
            count: 1,
        }),
        11 | 30 => Exact(DuplicateMemberIndex { group_idx: 0, member_idx: 2 }),
        12 | 31 => Exact(MemberThresholdMismatch),
        13 | 32 => Exact(DigestVerificationFailed),
        14 | 15 | 33 | 34 => Shape(|e| matches!(e, InsufficientShares { .. })),
        16 | 35 => Shape(|e| matches!(e, InsufficientShares { .. })),
        39 => Exact(InvalidPadding { share_idx: 0 }),
        40 => Exact(InvalidPadding { share_idx: 0 }),
        _ => panic!("vector #{idx_1based} is not a negative vector"),
    }
}

// ============================================================================
// Per-vector dispatch
// ============================================================================

fn run_vector(idx_1based: usize) {
    let vectors = load();
    let v = &vectors[idx_1based - 1];
    let Vector(desc, mnemonics, hex_secret, expected_xprv) = v;

    // Parse all shares. Some negative vectors (#2/#3/#10/#21/#22/#29/#39)
    // fail at parse time; the rest of the negative vectors and ALL
    // positive vectors must parse cleanly and reach the combine layer.
    let parsed: Result<Vec<_>, Slip39Error> =
        mnemonics.iter().map(|m| parse_slip39_share(m)).collect();

    if hex_secret.is_empty() {
        // NEGATIVE vector.
        let expected = negative_expected(idx_1based);
        let actual = match parsed {
            Err(e) => e,
            Ok(shares) => slip39_combine(&shares, PASSPHRASE).expect_err(&format!(
                "vector #{idx_1based} ({desc}): negative vector must refuse \
                 but combine succeeded"
            )),
        };
        match expected {
            ExpectedNegative::Exact(want) => {
                assert_eq!(
                    actual, want,
                    "vector #{idx_1based} ({desc}): variant mismatch (got {actual:?})",
                );
            }
            ExpectedNegative::Shape(check) => {
                assert!(
                    check(&actual),
                    "vector #{idx_1based} ({desc}): variant shape mismatch (got {actual:?})",
                );
            }
        }
    } else {
        // POSITIVE vector.
        let shares = parsed.unwrap_or_else(|e| {
            panic!(
                "vector #{idx_1based} ({desc}): positive vector must parse \
                 but got {e:?}"
            )
        });
        let recovered = slip39_combine(&shares, PASSPHRASE).unwrap_or_else(|e| {
            panic!(
                "vector #{idx_1based} ({desc}): positive vector must combine \
                 but got {e:?}"
            )
        });
        assert_eq!(
            hex::encode(recovered.as_slice()),
            *hex_secret,
            "vector #{idx_1based} ({desc}): hex_secret mismatch",
        );
        let xprv = Xpriv::new_master(Network::Bitcoin, &recovered)
            .expect("BIP-32 master derivation must succeed for valid SLIP-39 master secret");
        assert_eq!(
            xprv.to_string(),
            *expected_xprv,
            "vector #{idx_1based} ({desc}): xprv mismatch",
        );
    }
}

// ============================================================================
// Per-vector #[test]s — one per row in vectors.json for per-vector
// failure granularity (a macro-generated parameterized test would
// short-circuit on the first failure and mask later ones).
// ============================================================================

macro_rules! vector_test {
    ($idx:expr, $name:ident) => {
        #[test]
        fn $name() {
            run_vector($idx);
        }
    };
}

vector_test!(1, vector_01_valid_no_sharing_128);
vector_test!(2, vector_02_invalid_checksum_128);
vector_test!(3, vector_03_invalid_padding_128);
vector_test!(4, vector_04_basic_sharing_2of3_128);
vector_test!(5, vector_05_basic_sharing_2of3_single_share_128);
vector_test!(6, vector_06_different_identifiers_128);
vector_test!(7, vector_07_different_iteration_exponents_128);
vector_test!(8, vector_08_mismatching_group_thresholds_128);
vector_test!(9, vector_09_mismatching_group_counts_128);
vector_test!(10, vector_10_greater_group_threshold_than_count_128);
vector_test!(11, vector_11_duplicate_member_indices_128);
vector_test!(12, vector_12_mismatching_member_thresholds_128);
vector_test!(13, vector_13_invalid_digest_128);
vector_test!(14, vector_14_insufficient_groups_case1_128);
vector_test!(15, vector_15_insufficient_groups_case2_128);
vector_test!(16, vector_16_threshold_groups_insufficient_members_128);
vector_test!(17, vector_17_threshold_case1_128);
vector_test!(18, vector_18_threshold_case2_128);
vector_test!(19, vector_19_threshold_case3_128);
vector_test!(20, vector_20_valid_no_sharing_256);
vector_test!(21, vector_21_invalid_checksum_256);
vector_test!(22, vector_22_invalid_padding_256);
vector_test!(23, vector_23_basic_sharing_2of3_256);
vector_test!(24, vector_24_basic_sharing_2of3_single_share_256);
vector_test!(25, vector_25_different_identifiers_256);
vector_test!(26, vector_26_different_iteration_exponents_256);
vector_test!(27, vector_27_mismatching_group_thresholds_256);
vector_test!(28, vector_28_mismatching_group_counts_256);
vector_test!(29, vector_29_greater_group_threshold_than_count_256);
vector_test!(30, vector_30_duplicate_member_indices_256);
vector_test!(31, vector_31_mismatching_member_thresholds_256);
vector_test!(32, vector_32_invalid_digest_256);
vector_test!(33, vector_33_insufficient_groups_case1_256);
vector_test!(34, vector_34_insufficient_groups_case2_256);
vector_test!(35, vector_35_threshold_groups_insufficient_members_256);
vector_test!(36, vector_36_threshold_case1_256);
vector_test!(37, vector_37_threshold_case2_256);
vector_test!(38, vector_38_threshold_case3_256);
vector_test!(39, vector_39_insufficient_length);
vector_test!(40, vector_40_invalid_master_secret_length_folds_to_invalid_padding);
vector_test!(41, vector_41_modular_arithmetic_error_detection);
vector_test!(42, vector_42_extendable_no_sharing_128);
vector_test!(43, vector_43_extendable_basic_sharing_2of3_128);
vector_test!(44, vector_44_extendable_no_sharing_256);
vector_test!(45, vector_45_extendable_basic_sharing_2of3_256);

// ============================================================================
// Surface anchor — pin that the driver's GroupSpec re-export is in
// scope at the slip39 module surface. The use line above already does
// this implicitly; this anchor catches the case where GroupSpec is
// only imported via a sibling test file (i.e., the public surface
// must still expose it).
// ============================================================================

#[test]
fn group_spec_is_public_at_slip39_module_root() {
    // Constructibility anchor: GroupSpec must be a public type with at
    // least the (member_count, member_threshold) fields used by the
    // driver. The roundtrip test (G2) exercises GroupSpec end-to-end;
    // this test pins the public-surface contract on its own.
    let g = GroupSpec {
        member_count: 3,
        member_threshold: 2,
    };
    assert_eq!(g.member_count, 3);
    assert_eq!(g.member_threshold, 2);
}
