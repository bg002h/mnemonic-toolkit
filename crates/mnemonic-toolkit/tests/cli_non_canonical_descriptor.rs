//! v0.19.0 non-canonical miniscript descriptor integration tests.
//!
//! Exercises Phase 4's default-path inference + `tr(NUMS, ...)` sentinel +
//! canonical-mode `--account != 0` refusal + bare-tr row-16 refusal.
//! See design/PLAN_v0_19_0_non_canonical_descriptors.md §6 test corpus.

use assert_cmd::Command;
use predicates::prelude::*;

const TREZOR_12_ZERO: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const BIP39_TEST_2: &str =
    "legal winner thank year wave sausage worth useful legal winner thank yellow";
const BIP39_TEST_3: &str =
    "letter advice cage absurd amount doctor acoustic avoid letter advice cage above";

/// SPEC §4.12.b — bare-`@N` non-canonical descriptor + phrase slots →
/// default path `m/48'/<coin>'/<account>'/2'` per `@N`. User's flagship
/// invocation per the v0.19.0 cycle target.
#[test]
fn non_canonical_wsh_andor_default_path_inference_emits_bundle() {
    let descriptor = "wsh(andor(pkh(@0),after(12000000),or_i(and_v(v:pkh(@1),older(4032)),and_v(v:pkh(@2),older(32768)))))";
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            descriptor,
            "--network",
            "mainnet",
            "--language",
            "english",
            "--account",
            "0",
            "--slot",
            &format!("@0.phrase={TREZOR_12_ZERO}"),
            "--slot",
            &format!("@1.phrase={BIP39_TEST_2}"),
            "--slot",
            &format!("@2.phrase={BIP39_TEST_3}"),
        ])
        .assert()
        .success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    // SPEC §4.12.d byte-exact info notice on default-path emission.
    assert!(
        stderr.contains(
            "info: non-canonical descriptor; defaulting origin path for @0,@1,@2 to m/48'/0'/0'/2' (BIP-48 cosigner path). Override per-placeholder with [fp/path]@N or --slot @N.path=m/..."
        ),
        "stderr did not contain default-path info notice; got:\n{stderr}"
    );
    // Bundle emitted; engraving card on stderr pins per-`@N` derivation
    // at the default path.
    assert!(
        stderr.contains("48'/0'/0'/2'"),
        "stderr engraving card missing default path; got:\n{stderr}"
    );
}

/// SPEC §4.12.b — `--account 5` parameterizes the default path to
/// `m/48'/0'/5'/2'` in non-canonical mode.
#[test]
fn non_canonical_default_path_consumes_account_arg() {
    let descriptor = "wsh(andor(pkh(@0),after(12000000),pk(@1)))";
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            descriptor,
            "--network",
            "mainnet",
            "--account",
            "5",
            "--slot",
            &format!("@0.phrase={TREZOR_12_ZERO}"),
            "--slot",
            &format!("@1.phrase={BIP39_TEST_2}"),
        ])
        .assert()
        .success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("m/48'/0'/5'/2'"),
        "stderr notice did not reflect --account 5; got:\n{stderr}"
    );
}

/// SPEC §4.12.b — `--network testnet` uses BIP-44 coin-type 1.
#[test]
fn non_canonical_default_path_uses_testnet_coin_type() {
    let descriptor = "wsh(andor(pkh(@0),after(12000000),pk(@1)))";
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            descriptor,
            "--network",
            "testnet",
            "--slot",
            &format!("@0.phrase={TREZOR_12_ZERO}"),
            "--slot",
            &format!("@1.phrase={BIP39_TEST_2}"),
        ])
        .assert()
        .success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("m/48'/1'/0'/2'"),
        "stderr notice did not reflect testnet coin-type=1; got:\n{stderr}"
    );
}

/// SPEC §4.12.g — `--account != 0` with a CANONICAL descriptor (wsh-sortedmulti
/// has a canonical_origin mapping) still refuses per the existing
/// `DESCRIPTOR_WITH_NONZERO_ACCOUNT` guard, restructured to post-parse +
/// canonicity-gated. Byte-exact stderr match.
#[test]
fn canonical_wsh_sortedmulti_with_nonzero_account_refuses() {
    let descriptor = "wsh(sortedmulti(2,@0,@1))";
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            descriptor,
            "--network",
            "mainnet",
            "--account",
            "5",
            "--slot",
            &format!("@0.phrase={TREZOR_12_ZERO}"),
            "--slot",
            &format!("@1.phrase={BIP39_TEST_2}"),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "error: --account != 0 is meaningful only with --template; descriptor mode encodes account index in the @i origin path.",
        ));
}

/// SPEC §4.12.e — `tr(NUMS, <ms>)` sentinel substitution + default-path
/// inference (tr() with TapTree is non-canonical per md-codec's table).
#[test]
fn tr_nums_sentinel_substitution_emits_bundle() {
    let descriptor = "tr(NUMS,and_v(v:pk(@0),after(12000000)))";
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            descriptor,
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.phrase={TREZOR_12_ZERO}"),
        ])
        .assert()
        .success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    // H12 (cycle-1): a taproot (`tr(...)`) root tag defaults the BIP-48
    // script-type leaf to 3' (P2TR), not 2' (P2WSH).
    assert!(
        stderr.contains(
            "info: non-canonical descriptor; defaulting origin path for @0 to m/48'/0'/0'/3'"
        ),
        "stderr did not contain default-path info notice for tr(NUMS); got:\n{stderr}"
    );
}

/// SPEC §6.6 row 16 — bare `tr(<miniscript>)` (no internal key) refuses with
/// the byte-exact friendly hint pointing to NUMS sentinel + `@N` form.
#[test]
fn bare_tr_no_internal_key_refuses_with_row_16_text() {
    let descriptor = "tr(andor(pkh(@0),after(12000000),pk(@1)))";
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            descriptor,
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.phrase={TREZOR_12_ZERO}"),
            "--slot",
            &format!("@1.phrase={BIP39_TEST_2}"),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "tr() requires an internal key. For script-path-only spending use tr(NUMS, <ms>); for full taproot use tr(@<index>, <ms>) with a slot binding for the internal key.",
        ));
}

/// SPEC §4.12.b — canonical descriptors do NOT receive default-path
/// inference (canonical_origin supplies the per-shape default). No stderr
/// info notice is emitted in canonical mode.
#[test]
fn canonical_descriptor_does_not_emit_default_path_notice() {
    let descriptor = "wsh(sortedmulti(2,@0,@1))";
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            descriptor,
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.phrase={TREZOR_12_ZERO}"),
            "--slot",
            &format!("@1.phrase={BIP39_TEST_2}"),
        ])
        .assert()
        .success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        !stderr.contains("info: non-canonical descriptor"),
        "stderr unexpectedly contained default-path notice for canonical descriptor; got:\n{stderr}"
    );
}

/// SPEC §4.11.c symmetric verify-bundle — a non-canonical default-inferred
/// bundle must round-trip through `bundle --self-check` (which re-parses
/// the emitted md1/mk1/ms1 cards and re-derives the descriptor + cosigner
/// signature, asserting byte-equal). This pins the C1 fix from end-of-cycle
/// opus review: verify-bundle (and self-check) mirrors bundle's
/// canonicity-aware default-path inference in `descriptor_mode_verify_run`.
#[test]
fn non_canonical_default_path_self_check_round_trips() {
    let descriptor = "wsh(andor(pkh(@0),after(12000000),or_i(and_v(v:pkh(@1),older(4032)),and_v(v:pkh(@2),older(32768)))))";
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            descriptor,
            "--network",
            "mainnet",
            "--language",
            "english",
            "--account",
            "0",
            "--slot",
            &format!("@0.phrase={TREZOR_12_ZERO}"),
            "--slot",
            &format!("@1.phrase={BIP39_TEST_2}"),
            "--slot",
            &format!("@2.phrase={BIP39_TEST_3}"),
            "--self-check",
        ])
        .assert()
        .success();
}

/// Same round-trip test for `tr(NUMS, <ms>)` + bare-`@N` (default-path
/// inference + NUMS substitution + tap-leaf walker round-trip).
#[test]
fn tr_nums_default_path_self_check_round_trips() {
    let descriptor = "tr(NUMS,and_v(v:pk(@0),after(12000000)))";
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            descriptor,
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.phrase={TREZOR_12_ZERO}"),
            "--self-check",
        ])
        .assert()
        .success();
}

/// SPEC §6.6 row 4 (canonical-mode rejection of v0.19.0 phrase-with-origin
/// pairs) — canonical descriptors refuse `[Phrase, Path]` and
/// `[Phrase, Fingerprint, Path]` subkey sets. The Phase-2 grammar relaxation
/// accepts these structurally; the post-parse canonicity gate refuses.
#[test]
fn canonical_descriptor_refuses_phrase_plus_path_subkey_pair() {
    let descriptor = "wsh(sortedmulti(2,@0,@1))";
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            descriptor,
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.phrase={TREZOR_12_ZERO}"),
            "--slot",
            "@0.path=m/48'/0'/0'/2'",
            "--slot",
            &format!("@1.phrase={BIP39_TEST_2}"),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "slot @0 has both secret-bearing input and watch-only input; pick one per slot.",
        ));
}
