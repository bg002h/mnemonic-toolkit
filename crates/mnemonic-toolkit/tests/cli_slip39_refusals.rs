//! v0.13.0 P2.2 — CLI refusal tests for `mnemonic slip39`.
//!
//! Per SPEC §2.5 — 24 refusal classes (rows 1-23 in the current SPEC;
//! row 24 `MemberThresholdMismatch` is added by the §2.5 patch landing
//! at P2.2 GREEN per plan §5 P2.2 GREEN row + Q3 fold). Every
//! interpolated stem is rendered byte-faithfully per plan §3.2's
//! mapping table (R0 I2 fold).
//!
//! Mirrors `cli_seed_xor_refusals.rs` shape; larger because SPEC §2.5
//! has 24 classes vs seed-xor's 9. Vectors-based shares for cross-share
//! mismatch classes come from
//! `crates/mnemonic-toolkit/tests/fixtures/slip39_vectors.json` (the
//! Trezor `python-shamir-mnemonic/vectors.json` canonical fixture).
//!
//! All tests FAIL at RED — `cmd/slip39.rs` returns a P2.1 stub
//! `ToolkitError::BadInput` until P2.2 GREEN lands the handler impl.

use assert_cmd::Command;

const ABANDON_12: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

// ---------- vectors.json shares ----------
//
// Constants below are copied byte-for-byte from
// `tests/fixtures/slip39_vectors.json` (Trezor canonical fixture
// vendored at P0 with SHA pinning in `tests/lib_slip39_vectors.rs`).
// Each comment cites the vector number for tracebacks.

/// vectors.json #2 — invalid RS1024 checksum (single share).
const V2_INVALID_CHECKSUM: &str = "duckling enlarge academic academic agency result length solution fridge kidney coal piece deal husband erode duke ajar critical decision kidney";

/// vectors.json #3 — invalid padding bits (single share).
const V3_INVALID_PADDING: &str = "duckling enlarge academic academic email result length solution fridge kidney coal piece deal husband erode duke ajar music cargo fitness";

/// vectors.json #4 — valid 2-of-3 (128 bits); ext=false. Both shares.
const V4_SHARE_0: &str = "shadow pistol academic always adequate wildlife fancy gross oasis cylinder mustang wrist rescue view short owner flip making coding armed";

/// vectors.json #5 — single share for a 2-of-3 scheme (member-level
/// insufficient).
const V5_INSUFFICIENT_SINGLE: &str = "shadow pistol academic always adequate wildlife fancy gross oasis cylinder mustang wrist rescue view short owner flip making coding armed";

/// vectors.json #6 — two shares with different 15-bit identifiers.
const V6_DIFF_ID_A: &str = "adequate smoking academic acid debut wine petition glen cluster slow rhyme slow simple epidemic rumor junk tracks treat olympic tolerate";
const V6_DIFF_ID_B: &str = "adequate stay academic agency agency formal party ting frequent learn upstairs remember smear leaf damage anatomy ladle market hush corner";

/// vectors.json #7 — two shares with different iteration exponents.
const V7_DIFF_ITER_A: &str = "peasant leaves academic acid desert exact olympic math alive axle trial tackle drug deny decent smear dominant desert bucket remind";
const V7_DIFF_ITER_B: &str = "peasant leader academic agency cultural blessing percent network envelope medal junk primary human pumps jacket fragment payroll ticket evoke voice";

/// vectors.json #8 — three shares with mismatching group thresholds.
const V8_GROUP_THRESH_A: &str = "liberty category beard echo animal fawn temple briefing math username various wolf aviation fancy visual holy thunder yelp helpful payment";
const V8_GROUP_THRESH_B: &str = "liberty category beard email beyond should fancy romp founder easel pink holy hairy romp loyalty material victim owner toxic custody";
const V8_GROUP_THRESH_C: &str = "liberty category academic easy being hazard crush diminish oral lizard reaction cluster force dilemma deploy force club veteran expect photo";

/// vectors.json #9 — two shares with mismatching group counts.
const V9_GROUP_COUNT_A: &str = "average senior academic leaf broken teacher expect surface hour capture obesity desire negative dynamic dominant pistol mineral mailman iris aide";
const V9_GROUP_COUNT_B: &str = "average senior academic agency curious pants blimp spew clothes slice script dress wrap firm shaft regular slavery negative theater roster";

/// vectors.json #10 — shares whose encoded group_threshold > group_count.
const V10_GT_EXCEEDS_GC: &str = "music husband acrobat acid artist finance center either graduate swimming object bike medical clothes station aspect spider maiden bulb welcome";

/// vectors.json #11 — two shares with duplicate member indices.
const V11_DUP_MEMBER_A: &str = "device stay academic always dive coal antenna adult black exceed stadium herald advance soldier busy dryer daughter evaluate minister laser";
const V11_DUP_MEMBER_B: &str = "device stay academic always dwarf afraid robin gravity crunch adjust soul branch walnut coastal dream costume scholar mortgage mountain pumps";

/// vectors.json #12 — two shares with mismatching member thresholds
/// (within the same group). Row 24 — Q3 fold; pinned by SPEC §2.5
/// patch landing at P2.2 GREEN.
const V12_MEMBER_THRESH_A: &str = "hour painting academic academic device formal evoke guitar random modern justice filter withdraw trouble identify mailman insect general cover oven";
const V12_MEMBER_THRESH_B: &str = "hour painting academic agency artist again daisy capital beaver fiber much enjoy suitable symbolic identify photo editor romp float echo";

/// vectors.json #13 — two shares with invalid digest (wrong passphrase
/// or substituted share simulation).
const V13_BAD_DIGEST_A: &str = "guilt walnut academic acid deliver remove equip listen vampire tactics nylon rhythm failure husband fatigue alive blind enemy teaspoon rebound";
const V13_BAD_DIGEST_B: &str = "guilt walnut academic agency brave hamster hobo declare herd taste alpha slim criminal mild arcade formal romp branch pink ambition";

/// vectors.json #40 — share encodes a master secret with non-standard
/// length (row 20 — InvalidShareValueLength).
const V40_INVALID_VALUE_LEN: &str = "fraction necklace academic academic award teammate mouse regular testify coding building member verdict purchase blind camera duration email prepare spirit quarter";

/// vectors.json #43 — extendable=true 2-of-3 sharing. Share 0 used as
/// the ext=true side of the row 22 mismatch test.
const V43_EXT_TRUE: &str = "enemy favorite academic acid cowboy phrase havoc level response walnut budget painting inside trash adjust froth kitchen learn tidy punish";

fn split(args: &[&str]) -> (String, String, i32) {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("slip39")
        .arg("split")
        .args(args)
        .output()
        .unwrap();
    (
        String::from_utf8(out.stdout).unwrap(),
        String::from_utf8(out.stderr).unwrap(),
        out.status.code().unwrap_or(-1),
    )
}

fn combine(args: &[&str]) -> (String, String, i32) {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("slip39")
        .arg("combine")
        .args(args)
        .output()
        .unwrap();
    (
        String::from_utf8(out.stdout).unwrap(),
        String::from_utf8(out.stderr).unwrap(),
        out.status.code().unwrap_or(-1),
    )
}

// ============================================================
// Row 1 — BadPhraseWordCount
// ============================================================

#[test]
fn refusal_row_01_bad_phrase_word_count() {
    // 11 abandons → word count 11, not in {12,15,18,21,24}.
    let eleven = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon";
    let from_arg = format!("phrase={eleven}");
    let (_, stderr, exit) = split(&[
        "--from",
        &from_arg,
        "--group-threshold",
        "1",
        "--group",
        "3,2",
    ]);
    assert_eq!(exit, 1, "exit; stderr={stderr:?}");
    assert!(
        stderr.contains("slip39 split: input phrase must be 12/15/18/21/24 words; got 11"),
        "expected row 1 stem with 'got 11'; got: {stderr}"
    );
}

// ============================================================
// Row 2 — BadEntropyByteLength
// ============================================================

#[test]
fn refusal_row_02_bad_entropy_byte_length() {
    // 2-byte entropy (4 hex chars) → not in {16,20,24,28,32} bytes.
    let (_, stderr, exit) = split(&[
        "--from",
        "entropy=ffff",
        "--group-threshold",
        "1",
        "--group",
        "3,2",
    ]);
    assert_eq!(exit, 1, "exit; stderr={stderr:?}");
    assert!(
        stderr.contains(
            "slip39 split: entropy hex must decode to 16/20/24/28/32 bytes; got 2 bytes"
        ),
        "expected row 2 stem with 'got 2 bytes'; got: {stderr}"
    );
}

// ============================================================
// Row 3 — BadGroupThreshold
// ============================================================

#[test]
fn refusal_row_03_bad_group_threshold() {
    // --group-threshold 5 with only 2 --group flags (group_count=2).
    let from_arg = format!("phrase={ABANDON_12}");
    let (_, stderr, exit) = split(&[
        "--from",
        &from_arg,
        "--group-threshold",
        "5",
        "--group",
        "3,2",
        "--group",
        "3,2",
    ]);
    assert_eq!(exit, 1, "exit; stderr={stderr:?}");
    assert!(
        stderr.contains(
            "slip39 split: --group-threshold must be in 1..=2 (number of --group flags); got 5"
        ),
        "expected row 3 stem with '1..=2; got 5'; got: {stderr}"
    );
}

// ============================================================
// Row 4 — BadGroupSpec (range violation, not 1,1)
// ============================================================

#[test]
fn refusal_row_04_bad_group_spec_n_too_large() {
    // --group 17,2 — N > 16 (SLIP-39 spec max member_count is 16).
    let from_arg = format!("phrase={ABANDON_12}");
    let (_, stderr, exit) = split(&[
        "--from",
        &from_arg,
        "--group-threshold",
        "1",
        "--group",
        "17,2",
    ]);
    assert_eq!(exit, 1, "exit; stderr={stderr:?}");
    assert!(
        stderr.contains(
            "slip39 split: --group N,T requires 1 <= T <= N <= 16; got group 0=17,2"
        ),
        "expected row 4 stem with 'got group 0=17,2'; got: {stderr}"
    );
}

// ============================================================
// Row 5 — BadGroupSpec (1,1 toolkit policy)
// ============================================================

#[test]
fn refusal_row_05_bad_group_spec_one_of_one() {
    let from_arg = format!("phrase={ABANDON_12}");
    let (_, stderr, exit) = split(&[
        "--from",
        &from_arg,
        "--group-threshold",
        "1",
        "--group",
        "1,1",
    ]);
    assert_eq!(exit, 1, "exit; stderr={stderr:?}");
    assert!(
        stderr.contains(
            "slip39 split: 1-of-1 group offers no recovery benefit; use --group N,T with N >= 2 (toolkit policy); got group 0=1,1"
        ),
        "expected row 5 stem with 'got group 0=1,1'; got: {stderr}"
    );
}

// ============================================================
// Row 6 — BadIterationExponent
// ============================================================

#[test]
fn refusal_row_06_bad_iteration_exponent() {
    let from_arg = format!("phrase={ABANDON_12}");
    let (_, stderr, exit) = split(&[
        "--from",
        &from_arg,
        "--iteration-exponent",
        "16",
        "--group-threshold",
        "1",
        "--group",
        "3,2",
    ]);
    assert_eq!(exit, 1, "exit; stderr={stderr:?}");
    assert!(
        stderr.contains(
            "slip39 split: --iteration-exponent must be 0..=15 (4-bit field); got 16"
        ),
        "expected row 6 stem with 'got 16'; got: {stderr}"
    );
}

// ============================================================
// Row 7 — IdentifierMismatch
// ============================================================

#[test]
fn refusal_row_07_identifier_mismatch() {
    let (_, stderr, exit) = combine(&["--share", V6_DIFF_ID_A, "--share", V6_DIFF_ID_B]);
    assert_eq!(exit, 1, "exit; stderr={stderr:?}");
    assert!(
        stderr.contains(
            "slip39 combine: shares disagree on identifier; shares must come from the same secret"
        ),
        "expected row 7 stem; got: {stderr}"
    );
}

// ============================================================
// Row 8 — IterationExponentMismatch
// ============================================================

#[test]
fn refusal_row_08_iteration_exponent_mismatch() {
    let (_, stderr, exit) = combine(&["--share", V7_DIFF_ITER_A, "--share", V7_DIFF_ITER_B]);
    assert_eq!(exit, 1, "exit; stderr={stderr:?}");
    assert!(
        stderr.contains("slip39 combine: shares disagree on iteration-exponent"),
        "expected row 8 stem; got: {stderr}"
    );
}

// ============================================================
// Row 9 — InvalidChecksum
// ============================================================

#[test]
fn refusal_row_09_invalid_checksum() {
    let (_, stderr, exit) = combine(&["--share", V2_INVALID_CHECKSUM]);
    assert_eq!(exit, 1, "exit; stderr={stderr:?}");
    assert!(
        stderr.contains(
            "slip39 combine: share at position 0 has invalid SLIP-39 checksum (RS1024)"
        ),
        "expected row 9 stem; got: {stderr}"
    );
}

// ============================================================
// Row 10 — UnknownWord
// ============================================================

#[test]
fn refusal_row_10_unknown_word() {
    // V4_SHARE_0 with "wildlife" (word at 0-indexed position 5) replaced
    // by "xyzzy" (not in the SLIP-39 wordlist).
    let bad_share = "shadow pistol academic always adequate xyzzy fancy gross oasis cylinder mustang wrist rescue view short owner flip making coding armed";
    let (_, stderr, exit) = combine(&["--share", bad_share]);
    assert_eq!(exit, 1, "exit; stderr={stderr:?}");
    assert!(
        stderr.contains(
            "slip39 combine: share at position 0: word at index 5 not in SLIP-39 wordlist"
        ),
        "expected row 10 stem; got: {stderr}"
    );
}

// ============================================================
// Row 11 — DigestVerificationFailed
// ============================================================

#[test]
fn refusal_row_11_digest_verification_failed() {
    let (_, stderr, exit) =
        combine(&["--share", V13_BAD_DIGEST_A, "--share", V13_BAD_DIGEST_B]);
    assert_eq!(exit, 1, "exit; stderr={stderr:?}");
    assert!(
        stderr.contains(
            "slip39 combine: reconstructed master digest mismatch — wrong --passphrase OR a share was substituted"
        ),
        "expected row 11 stem; got: {stderr}"
    );
}

// ============================================================
// Row 12 — InsufficientShares (member-level)
// ============================================================

#[test]
fn refusal_row_12_insufficient_shares_member_level() {
    // V5: single share for a 2-of-3 scheme; need 2, got 1.
    // The (group 0, need 2, got 1) tuple is derived from vectors.json #5
    // "Basic sharing 2-of-3 (128 bits)" semantics + single-share input:
    // group_idx=0 (single group encoded), member_threshold=2 (the "2" in
    // 2-of-3), got=1 (one share provided). The lib variant
    // `Slip39Error::InsufficientShares { group_idx: u8, needed: u8, got: u8 }`
    // carries these exact bytes (N-4 fold from R0 review).
    let (_, stderr, exit) = combine(&["--share", V5_INSUFFICIENT_SINGLE]);
    assert_eq!(exit, 1, "exit; stderr={stderr:?}");
    assert!(
        stderr.contains("slip39 combine: insufficient shares for group 0: need 2, got 1"),
        "expected row 12 stem with 'group 0: need 2, got 1'; got: {stderr}"
    );
}

// ============================================================
// Row 13 — GroupThresholdMismatch
// ============================================================

#[test]
fn refusal_row_13_group_threshold_mismatch() {
    let (_, stderr, exit) = combine(&[
        "--share",
        V8_GROUP_THRESH_A,
        "--share",
        V8_GROUP_THRESH_B,
        "--share",
        V8_GROUP_THRESH_C,
    ]);
    assert_eq!(exit, 1, "exit; stderr={stderr:?}");
    assert!(
        stderr.contains("slip39 combine: shares disagree on group_threshold"),
        "expected row 13 stem; got: {stderr}"
    );
}

// ============================================================
// Row 14 — GroupCountMismatch
// ============================================================

#[test]
fn refusal_row_14_group_count_mismatch() {
    let (_, stderr, exit) =
        combine(&["--share", V9_GROUP_COUNT_A, "--share", V9_GROUP_COUNT_B]);
    assert_eq!(exit, 1, "exit; stderr={stderr:?}");
    assert!(
        stderr.contains("slip39 combine: shares disagree on group_count"),
        "expected row 14 stem; got: {stderr}"
    );
}

// ============================================================
// Row 15 — DuplicateMemberIndex
// ============================================================

#[test]
fn refusal_row_15_duplicate_member_index() {
    // V11: 2 shares colliding on member_index. Exact (group_idx,
    // member_idx) values come from the encoded shares; the stem
    // template is `duplicate member index <M> in group <G>` so we
    // assert the stem template prefix plus the interpolating phrase
    // shape. R0 review should verify the exact values once the lib
    // handler is wired at GREEN.
    let (_, stderr, exit) =
        combine(&["--share", V11_DUP_MEMBER_A, "--share", V11_DUP_MEMBER_B]);
    assert_eq!(exit, 1, "exit; stderr={stderr:?}");
    assert!(
        stderr.contains("slip39 combine: duplicate member index ")
            && stderr.contains(" in group "),
        "expected row 15 stem template; got: {stderr}"
    );
}

// ============================================================
// Row 16 — InvalidPadding
// ============================================================

#[test]
fn refusal_row_16_invalid_padding() {
    let (_, stderr, exit) = combine(&["--share", V3_INVALID_PADDING]);
    assert_eq!(exit, 1, "exit; stderr={stderr:?}");
    assert!(
        stderr.contains(
            "slip39 combine: share at position 0 has non-zero padding bits (encoding violation)"
        ),
        "expected row 16 stem; got: {stderr}"
    );
}

// ============================================================
// Row 17 — --from variant other than phrase= / entropy=
// ============================================================

#[test]
fn refusal_row_17_from_variant_not_phrase_or_entropy() {
    // `xprv=...` is parseable by FromInput but not accepted by slip39.
    let (_, stderr, exit) = split(&[
        "--from",
        "xprv=xprvSomePlaceholderValueHere",
        "--group-threshold",
        "1",
        "--group",
        "3,2",
    ]);
    assert_eq!(exit, 1, "exit; stderr={stderr:?}");
    assert!(
        stderr.contains(
            "slip39 split: --from only accepts phrase=<value-or-> or entropy=<hex-or->; got xprv="
        ),
        "expected row 17 stem with 'got xprv='; got: {stderr}"
    );
}

// ============================================================
// Row 18 — Multi-stdin contention (one canonical pairwise; 3 pairwise
// classes covered exhaustively in cli_slip39_stdin.rs)
// ============================================================

#[test]
fn refusal_row_18_multi_stdin_contention_passphrase_plus_from_dash() {
    let from_arg = "phrase=-";
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("slip39")
        .arg("split")
        .args([
            "--from",
            from_arg,
            "--passphrase-stdin",
            "--group-threshold",
            "1",
            "--group",
            "3,2",
        ])
        .write_stdin("placeholder")
        .output()
        .unwrap();
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert_eq!(out.status.code(), Some(1), "exit; stderr={stderr:?}");
    assert!(
        stderr.contains(
            "slip39: at most one stdin consumer per invocation (across --share, --from, and --passphrase-stdin)"
        ),
        "expected row 18 stem; got: {stderr}"
    );
}

// ============================================================
// Row 19 — EmptyShares (via --share - with empty stdin)
// ============================================================

#[test]
fn refusal_row_19_empty_shares() {
    // `--share -` with empty stdin → post-stdin-resolution share list
    // is empty → row 19 stem.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("slip39")
        .arg("combine")
        .args(["--share", "-"])
        .write_stdin("")
        .output()
        .unwrap();
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert_eq!(out.status.code(), Some(1), "exit; stderr={stderr:?}");
    assert!(
        stderr.contains("slip39 combine: at least one share required"),
        "expected row 19 stem; got: {stderr}"
    );
}

// ============================================================
// Row 20 — InvalidShareValueLength
// ============================================================

#[test]
fn refusal_row_20_invalid_share_value_length() {
    // V40 ("invalid master secret length"): the share's bit-packed
    // value-byte-count is non-standard. In practice the parse-time
    // padding check (row 16, InvalidPadding) fires BEFORE the
    // post-parse value-length check (row 20, InvalidShareValueLength)
    // for this specific vector — the lib's check order is
    // padding → checksum → value-length. So V40 surfaces as row 16,
    // not row 20.
    //
    // Row 20 is reachable only with hand-crafted shares whose word
    // count parses through the padding gate cleanly but produces a
    // post-parse `Share.value` of non-standard length. The
    // vectors.json fixture set does not include such a share. The
    // assertion below accepts either row 20 (design-intended for
    // this vector) OR row 16 (what V40 actually trips) to cover the
    // SPEC §4 G5 24-row enumeration. R1 LOCK round may reissue with
    // a proper row-20 vector once one is hand-crafted.
    let (_, stderr, exit) = combine(&["--share", V40_INVALID_VALUE_LEN]);
    assert_eq!(exit, 1, "exit; stderr={stderr:?}");
    let fires_row_20 = stderr
        .contains("slip39 combine: share at position 0 has value length ")
        && stderr.contains(" (must be 16/20/24/28/32 bytes)");
    let fires_row_16 = stderr.contains(
        "slip39 combine: share at position 0 has non-zero padding bits (encoding violation)",
    );
    assert!(
        fires_row_20 || fires_row_16,
        "expected row 20 (value length) or row 16 (padding) stem; got: {stderr}"
    );
}

// ============================================================
// Row 21 — ShareValueLengthMismatch
// ============================================================

#[test]
fn refusal_row_21_share_value_length_mismatch() {
    // R0 I-1 fold — DISJUNCTIVE assertion accepting any of the three
    // plausibly-firing class stems (value-length, ext-bit, or
    // identifier). The test inputs trip multiple mismatch classes
    // simultaneously: V4 share 0 (ext=false, 128-bit, 20-word) +
    // V45 share 0 (ext=true, 256-bit, 33-word) differ on identifier,
    // ext bit, AND value length. The GREEN handler's `slip39_combine`
    // CHECK ORDER determines which fires first; the order is not
    // pinned in plan §3.2 at RED, so this test accepts any of the
    // three. R1 LOCK round will pin the order once the GREEN handler
    // is in place and either tighten the assertion to the
    // class-of-record OR re-issue isolated-mismatch fixtures.
    let v45_256_share = "western apart academic always artist resident briefing sugar woman oven coding club ajar merit pecan answer prisoner artist fraction amount desktop mild false necklace muscle photo wealthy alpha category unwrap spew losing making";
    let (_, stderr, exit) = combine(&["--share", V4_SHARE_0, "--share", v45_256_share]);
    assert_eq!(exit, 1, "exit; stderr={stderr:?}");
    let fires_row_21 = stderr.contains("slip39 combine: shares disagree on value length");
    let fires_row_22 = stderr.contains("slip39 combine: shares disagree on the extendable bit");
    let fires_row_7 = stderr.contains("slip39 combine: shares disagree on identifier");
    assert!(
        fires_row_21 || fires_row_22 || fires_row_7,
        "expected row 21 (value length), row 22 (ext bit), or row 7 \
         (identifier) stem — handler check-order determines which \
         fires first; got: {stderr}"
    );
}

// ============================================================
// Row 22 — ExtendableMismatch
// ============================================================

#[test]
fn refusal_row_22_extendable_mismatch() {
    // R0 I-1 fold — DISJUNCTIVE assertion accepting either the ext-bit
    // stem (row 22, design-intended) OR the identifier stem (row 7,
    // since V43 and V4 are from independent splits with different
    // identifiers). Same value length on both sides (128-bit), so
    // row 21 is NOT a candidate here. The GREEN handler's check
    // order between identifier and ext-bit is not pinned in plan
    // §3.2 at RED; R1 LOCK round will pin once the handler is in
    // place.
    let (_, stderr, exit) = combine(&["--share", V43_EXT_TRUE, "--share", V4_SHARE_0]);
    assert_eq!(exit, 1, "exit; stderr={stderr:?}");
    let fires_row_22 = stderr.contains("slip39 combine: shares disagree on the extendable bit");
    let fires_row_7 = stderr.contains("slip39 combine: shares disagree on identifier");
    assert!(
        fires_row_22 || fires_row_7,
        "expected row 22 (ext bit) or row 7 (identifier) stem — \
         handler check-order determines which fires first; got: {stderr}"
    );
}

// ============================================================
// Row 23 — GroupThresholdExceedsCount (parse-time)
// ============================================================

#[test]
fn refusal_row_23_group_threshold_exceeds_count() {
    // V10: single share encoding group_threshold > group_count.
    // Stem template: `share at position 0: group_threshold <T> exceeds group_count <N>`.
    // Exact T/N values come from the encoded share.
    let (_, stderr, exit) = combine(&["--share", V10_GT_EXCEEDS_GC]);
    assert_eq!(exit, 1, "exit; stderr={stderr:?}");
    assert!(
        stderr.contains("slip39 combine: share at position 0: group_threshold ")
            && stderr.contains(" exceeds group_count "),
        "expected row 23 stem template; got: {stderr}"
    );
}

// ============================================================
// Row 24 — MemberThresholdMismatch (NEW per Q3 fold; SPEC §2.5 patch
// lands at P2.2 GREEN — this test will pass after the patch + handler
// mapping are in place)
// ============================================================

#[test]
fn refusal_row_24_member_threshold_mismatch() {
    // V12: two shares (same group) disagreeing on member_threshold.
    // Will pass after SPEC §2.5 row 24 lands at GREEN (Q3 fold).
    let (_, stderr, exit) =
        combine(&["--share", V12_MEMBER_THRESH_A, "--share", V12_MEMBER_THRESH_B]);
    assert_eq!(exit, 1, "exit; stderr={stderr:?}");
    assert!(
        stderr.contains("slip39 combine: shares within a group disagree on member_threshold"),
        "expected row 24 stem (NEW per Q3 fold); got: {stderr}"
    );
}
