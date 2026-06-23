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

// ============================================================================
// cycle-11b L24 — verify-bundle descriptor-mode OOB-panic → typed DescriptorParse
// ============================================================================

/// cycle-11b L24 (E-panic-dos): `verify-bundle --descriptor` in non-canonical
/// mode applied `--slot @N.path` overrides via an unguarded `new_paths[idx]`
/// write (verify_bundle.rs override loop) — `validate_slot_set` only enforces
/// contiguity (`0..=max_idx`), NOT range-vs-`n`. A contiguous slot set whose max
/// index exceeds the descriptor's placeholder count `n` therefore passed
/// validation and panicked (index out of bounds) on the OOB write. `bundle.rs`
/// has the `max(idx+1) != n` exact-coverage gate; verify_bundle.rs omitted it
/// (hand-copied descriptor-mode binding guard-drift). Fix: mirror the gate →
/// clean `DescriptorParse` (exit 2).
///
/// M2 fixture preconditions (so the override loop genuinely reaches the OOB
/// write — without these the RED passes for the wrong reason):
///  1. The descriptor MUST be genuinely NON-CANONICAL so control enters the
///     `is_non_canonical` override block. `wsh(andor(pkh(@0),after(12000000),
///     pk(@1)))` is a general-policy wrapper with no canonical_origin mapping
///     (asserted via the `info: non-canonical descriptor` notice the bundle/
///     verify path emits for it). n = 2 (placeholders @0, @1).
///  2. `@2` MUST carry the LEGAL phrase-bearing set `[Phrase, Path]` — TWO
///     `--slot @2.*` flags. `@2.path=…` ALONE yields `{Path}`, which (a) is
///     rejected by `validate_slot_set` FIRST (no bare-`[Path]` legal-set arm) so
///     the override loop is never reached, and (b) even past validation would
///     hit the `subkeys.contains(Phrase|Seedqr|Ms1)`-else-`continue` filter. The
///     co-located `@2.phrase` makes `@2 = {Phrase, Path}` (a legal set), so it
///     clears both gates and reaches the unguarded `new_paths[2]` write.
#[test]
fn verify_bundle_descriptor_slot_over_n_rejects_not_panics() {
    // n = 2 placeholders (@0, @1); @2 over-runs new_paths (len 2).
    let descriptor = "wsh(andor(pkh(@0),after(12000000),pk(@1)))";
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "verify-bundle",
            "--descriptor",
            descriptor,
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.phrase={TREZOR_12_ZERO}"),
            "--slot",
            &format!("@1.phrase={BIP39_TEST_2}"),
            // @2 = {Phrase, Path}: legal phrase-bearing set, contiguous index 2.
            "--slot",
            &format!("@2.phrase={BIP39_TEST_3}"),
            "--slot",
            "@2.path=m/84'/0'/0'",
            // verify-bundle requires --mk1/--md1 (clap). Empty sentinels pass the
            // per-flag HRP gate (SPEC §5.8 exemption) and fail md1 reassembly, so
            // control falls through to the descriptor-mode binding (and the gate)
            // BEFORE any expected-wire comparison.
            "--mk1",
            "",
            "--md1",
            "",
        ])
        .assert()
        // Post-fix: clean typed DescriptorParse (exit 2), NOT a panic.
        .code(2)
        .stderr(
            predicate::str::contains(
                "descriptor has n=2 placeholders but --slot vec covers 3 slots",
            )
            .or(predicate::str::contains("n=2").and(predicate::str::contains("3 slots"))),
        );
}

/// cycle-11b L24 REGRESSION — the gate is exact-coverage (`!= n`), so the
/// in-range path-override flow (here `@0`/`@1` path overrides, max idx+1 == n)
/// MUST NOT over-fire. The descriptor is the same non-canonical 2-key wrapper;
/// both slots carry `[Phrase, Path]`. The gate passes (covers exactly 2 of n=2),
/// the override loop runs, and verify-bundle proceeds past it (it then fails
/// downstream for lack of expected `--md1`/`--mk1` wire, NOT with the n/slot
/// mismatch). Assert the n/slot DescriptorParse message does NOT appear.
#[test]
fn verify_bundle_descriptor_exact_coverage_path_override_does_not_over_fire() {
    let descriptor = "wsh(andor(pkh(@0),after(12000000),pk(@1)))";
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "verify-bundle",
            "--descriptor",
            descriptor,
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.phrase={TREZOR_12_ZERO}"),
            "--slot",
            "@0.path=m/84'/0'/0'",
            "--slot",
            &format!("@1.phrase={BIP39_TEST_2}"),
            "--slot",
            "@1.path=m/84'/0'/1'",
            "--mk1",
            "",
            "--md1",
            "",
        ])
        .assert();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        !stderr.contains("placeholders but --slot vec covers"),
        "exact-coverage path-override over-fired the n/slot gate; got:\n{stderr}"
    );
}

// ============================================================================
// Wave-4 L1 — bundle ↔ verify-bundle descriptor-mode path-derivation PARITY
// matrix (SPEC_wave4_L1_verify_bundle_dedup.md §4). This is the regression
// oracle for the `bind_descriptor_mode_paths` dedup: a desync between the emit
// (bundle) and round-trip (verify-bundle) path derivation goes RED here — the
// exact cycle-11b L24 drift class, now caught at the unit-of-derivation.
//
// Known-fixture DERIVED master fingerprints (mainnet, verified live):
//   @0 = TREZOR_12_ZERO → 73c5da0a
//   @1 = BIP39_TEST_2    → b8688df1
// Inline-origin cells MUST embed each phrase slot's TRUE derived fp, or the
// `bundle.rs` fp-mismatch refusal (`--slot @N.phrase derives master fingerprint
// … but descriptor @N annotation specifies …`) false-REDs the cell before it
// ever reaches the `is_non_canonical` binding the matrix means to exercise.
// ============================================================================

const FP0: &str = "73c5da0a"; // TREZOR_12_ZERO derived master fp
const FP1: &str = "b8688df1"; // BIP39_TEST_2 derived master fp

/// Emit a `bundle --json` for (descriptor, slots), then `verify-bundle` the
/// SAME cards + descriptor + slots and return (result, all_checks_passed,
/// n_checks). Card flattening follows the n≥2-NESTED-mk1 / every-`--ms1` shape
/// (SPEC §4 Finding-2): for n=1 the bundle's `mk1` is a flat `[str,…]` array;
/// for n≥2 it is `[[str,…],[str,…]]` (one inner per cosigner). This helper
/// handles BOTH (per-element type check), so a single matrix can mix arities.
/// A single-sig flat-mk1 harness would truncate per-cosigner continuation
/// chunks for n≥2 and spuriously RED — see SPEC §4 Finding-2.
fn parity_round_trips(descriptor: &str, slots: &[String]) -> (String, bool, usize) {
    use serde_json::Value;

    let mut emit_args: Vec<String> = vec![
        "bundle".into(),
        "--descriptor".into(),
        descriptor.into(),
        "--network".into(),
        "mainnet".into(),
        "--json".into(),
    ];
    for s in slots {
        emit_args.push("--slot".into());
        emit_args.push(s.clone());
    }
    let emit = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&emit_args)
        .assert()
        .success();
    let bundle: Value =
        serde_json::from_slice(&emit.get_output().stdout).expect("valid bundle JSON");

    let mut vargs: Vec<String> = vec![
        "verify-bundle".into(),
        "--descriptor".into(),
        descriptor.into(),
        "--network".into(),
        "mainnet".into(),
        "--json".into(),
    ];
    for s in slots {
        vargs.push("--slot".into());
        vargs.push(s.clone());
    }
    // Every ms1 entry (the ms1 array is length-N for phrase-slot bundles).
    for e in bundle["ms1"].as_array().expect("ms1 array") {
        vargs.push("--ms1".into());
        vargs.push(e.as_str().expect("ms1 entry str").to_string());
    }
    // mk1: nested (n≥2) OR flat (n=1) — handle per-element.
    for entry in bundle["mk1"].as_array().expect("mk1 array") {
        match entry {
            Value::Array(inner) => {
                for chunk in inner {
                    vargs.push("--mk1".into());
                    vargs.push(chunk.as_str().expect("mk1 chunk str").to_string());
                }
            }
            Value::String(s) => {
                vargs.push("--mk1".into());
                vargs.push(s.clone());
            }
            other => panic!("unexpected mk1 element shape: {other:?}"),
        }
    }
    // md1 is always flat.
    for chunk in bundle["md1"].as_array().expect("md1 array") {
        vargs.push("--md1".into());
        vargs.push(chunk.as_str().expect("md1 chunk str").to_string());
    }

    let verify = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&vargs)
        .assert()
        .success();
    let out: Value =
        serde_json::from_slice(&verify.get_output().stdout).expect("valid verify JSON");
    let checks = out["checks"].as_array().expect("checks array");
    let all_passed = checks.iter().all(|c| c["passed"] == true);
    (
        out["result"].as_str().unwrap_or("<none>").to_string(),
        all_passed,
        checks.len(),
    )
}

/// SPEC §4 mandatory self-test (FIRST-landed cell): the parity harness MUST
/// reproduce `result=ok` / all-checks-passed for the wsh n=2 all-elided cell
/// ON CURRENT SOURCE. A green here proves the harness card-shape handling is
/// correct; only then does a RED elsewhere implicate the dedup rather than a
/// harness bug.
#[test]
fn parity_harness_self_test_wsh_n2_all_elided() {
    let (result, all_passed, n) = parity_round_trips(
        "wsh(andor(pkh(@0),after(12000000),pk(@1)))",
        &[
            format!("@0.phrase={TREZOR_12_ZERO}"),
            format!("@1.phrase={BIP39_TEST_2}"),
        ],
    );
    assert_eq!(result, "ok", "self-test harness desynced (result != ok)");
    assert!(all_passed, "self-test harness: not all checks passed ({n})");
}

/// PARITY (wsh, leaf 2') — all-elided → BIP-48 default; shared-explicit (same
/// inline path, correct fps) → Shared no-defaulting; divergent (different inline
/// paths, correct fps) → Divergent no-defaulting; slot-override → override
/// replaces default; mixed (@0 inline-correct-fp + @1 elided). Every cell MUST
/// round-trip `ok` / all-passed — proving emit and verify agree byte-for-byte.
#[test]
fn parity_wsh_all_shapes_round_trip() {
    let cases: Vec<(&str, Vec<String>)> = vec![
        // all-elided
        (
            "wsh(andor(pkh(@0),after(12000000),pk(@1)))",
            vec![
                format!("@0.phrase={TREZOR_12_ZERO}"),
                format!("@1.phrase={BIP39_TEST_2}"),
            ],
        ),
        // shared-explicit (same path on both, correct per-slot fps)
        (
            "wsh(andor(pkh([73c5da0a/48'/0'/0'/2']@0),after(12000000),pk([b8688df1/48'/0'/0'/2']@1)))",
            vec![
                format!("@0.phrase={TREZOR_12_ZERO}"),
                format!("@1.phrase={BIP39_TEST_2}"),
            ],
        ),
        // divergent (different paths, correct per-slot fps)
        (
            "wsh(andor(pkh([73c5da0a/48'/0'/0'/2']@0),after(12000000),pk([b8688df1/48'/0'/7'/2']@1)))",
            vec![
                format!("@0.phrase={TREZOR_12_ZERO}"),
                format!("@1.phrase={BIP39_TEST_2}"),
            ],
        ),
        // slot-override (bare @0 + --slot @0.path= overrides the default)
        (
            "wsh(andor(pkh(@0),after(12000000),pk(@1)))",
            vec![
                format!("@0.phrase={TREZOR_12_ZERO}"),
                "@0.path=m/48'/0'/9'/2'".into(),
                format!("@1.phrase={BIP39_TEST_2}"),
            ],
        ),
        // mixed (@0 inline-correct-fp, @1 elided-defaulted)
        (
            "wsh(andor(pkh([73c5da0a/48'/0'/0'/2']@0),after(12000000),pk(@1)))",
            vec![
                format!("@0.phrase={TREZOR_12_ZERO}"),
                format!("@1.phrase={BIP39_TEST_2}"),
            ],
        ),
    ];
    for (desc, slots) in &cases {
        let (result, all_passed, n) = parity_round_trips(desc, slots);
        assert_eq!(result, "ok", "wsh parity cell desynced ({desc})");
        assert!(all_passed, "wsh parity cell not all-passed ({n}): {desc}");
    }
    // Suppress the unused-const lint for the fp documentation anchors when this
    // file's other cells don't reference them through the format! literals.
    let _ = (FP0, FP1);
}

/// PARITY (sh-wsh, leaf 1') — H12 selects the sh-wsh BIP-48 leaf `1'`. The
/// all-elided and slot-override shapes must round-trip, proving the shared fn's
/// `root_tag`-derived `default_script_type` agrees emit↔verify for the sh root.
#[test]
fn parity_sh_wsh_round_trips() {
    for (desc, slots) in [
        (
            "sh(wsh(andor(pkh(@0),after(12000000),pk(@1))))",
            vec![
                format!("@0.phrase={TREZOR_12_ZERO}"),
                format!("@1.phrase={BIP39_TEST_2}"),
            ],
        ),
        (
            "sh(wsh(andor(pkh(@0),after(12000000),pk(@1))))",
            vec![
                format!("@0.phrase={TREZOR_12_ZERO}"),
                "@0.path=m/48'/0'/4'/1'".into(),
                format!("@1.phrase={BIP39_TEST_2}"),
            ],
        ),
    ] {
        let (result, all_passed, n) = parity_round_trips(desc, &slots);
        assert_eq!(result, "ok", "sh-wsh parity cell desynced ({desc})");
        assert!(
            all_passed,
            "sh-wsh parity cell not all-passed ({n}): {desc}"
        );
    }
}

/// PARITY (tr(NUMS), leaf 3', n=1) — H12 selects the taproot BIP-48 leaf `3'`.
/// all-elided + slot-override. The n=1 bundle mk1 is FLAT (the harness handles
/// it), so this also exercises the flat-mk1 arm of `parity_round_trips`.
#[test]
fn parity_tr_nums_round_trips() {
    for (desc, slots) in [
        (
            "tr(NUMS,and_v(v:pk(@0),after(12000000)))",
            vec![format!("@0.phrase={TREZOR_12_ZERO}")],
        ),
        (
            "tr(NUMS,and_v(v:pk(@0),after(12000000)))",
            vec![
                format!("@0.phrase={TREZOR_12_ZERO}"),
                "@0.path=m/48'/0'/3'/3'".into(),
            ],
        ),
    ] {
        let (result, all_passed, n) = parity_round_trips(desc, &slots);
        assert_eq!(result, "ok", "tr parity cell desynced ({desc})");
        assert!(all_passed, "tr parity cell not all-passed ({n}): {desc}");
    }
}

// ============================================================================
// Wave-4 L1 — characterization cells pinning the three divergences (D1/D2/D3).
// These prove the dedup keeps the emit-only refusals/notice OUT of verify and
// IN bundle. They are the tests that go RED if a future edit accidentally
// imports the row-19 refusal into verify or drops it from bundle.
// ============================================================================

/// D2 — `bundle` RETAINS the §6.6 row-19 inline-vs-slot path-mismatch refusal
/// post-dedup. `@0` carries an inline `[fp/path]@0` origin AND a `--slot
/// @0.path=` that DISAGREES → refuse (exit ≠ 0), stderr names the conflict.
/// (`@0` is NOT defaulted — it has an inline origin — and the slot path differs,
/// so the row-19 `mode == Emit` arm fires.)
#[test]
fn bundle_retains_row19_inline_vs_slot_path_mismatch_refusal() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            // @0 inline path is .../0'/2'; --slot below says .../9'/2' → mismatch.
            "wsh(andor(pkh([73c5da0a/48'/0'/0'/2']@0),after(12000000),pk(@1)))",
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.phrase={TREZOR_12_ZERO}"),
            "--slot",
            "@0.path=m/48'/0'/9'/2'",
            "--slot",
            &format!("@1.phrase={BIP39_TEST_2}"),
        ])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("path mismatch").and(predicate::str::contains("disagrees")),
        );
}

/// D2 — `verify-bundle` does NOT acquire the row-19 refusal. The SAME
/// inline-vs-slot-mismatch inputs fed to `verify-bundle` MUST NOT surface the
/// `path mismatch`/`disagrees` refusal from the binding stage (verify is
/// read-only; a genuine conflict surfaces downstream as a md1 mismatch, never as
/// `SlotInputViolation{kind:"path-mismatch"}`). This is the cell that RED-flags
/// accidentally importing the row-19 refusal into the verify path.
#[test]
fn verify_bundle_does_not_acquire_row19_refusal() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "verify-bundle",
            "--descriptor",
            "wsh(andor(pkh([73c5da0a/48'/0'/0'/2']@0),after(12000000),pk(@1)))",
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.phrase={TREZOR_12_ZERO}"),
            "--slot",
            "@0.path=m/48'/0'/9'/2'",
            "--slot",
            &format!("@1.phrase={BIP39_TEST_2}"),
            // Empty sentinels reach the binding (and the would-be row-19 site)
            // BEFORE any expected-wire comparison; they fail md1 reassembly later.
            "--mk1",
            "",
            "--md1",
            "",
        ])
        .assert();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        !stderr.contains("path mismatch") && !stderr.contains("disagrees"),
        "verify-bundle acquired the emit-only row-19 path-mismatch refusal; got:\n{stderr}"
    );
}

/// D3 — `verify-bundle` still emits NO default-path-inference notice post-dedup
/// (the notice is emit-only and stays at the bundle call site). The notice
/// string is `emit_default_path_notice`'s `info: non-canonical descriptor;
/// defaulting origin path …`. verify-bundle is read-only.
#[test]
fn verify_bundle_emits_no_default_path_notice() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "verify-bundle",
            "--descriptor",
            "wsh(andor(pkh(@0),after(12000000),pk(@1)))",
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.phrase={TREZOR_12_ZERO}"),
            "--slot",
            &format!("@1.phrase={BIP39_TEST_2}"),
            "--mk1",
            "",
            "--md1",
            "",
        ])
        .assert();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        !stderr.contains("info: non-canonical descriptor; defaulting origin path"),
        "verify-bundle emitted the emit-only default-path notice; got:\n{stderr}"
    );
}

/// Gate-re-order safety (SPEC §3.2 / §5 item 2): an over-`n` slot vec on a
/// PARSE-FAILING descriptor still exits ≠ 0 after the dedup folds the gate AFTER
/// the canonicity probe. Either refusal message is acceptable (the gate-vs-probe
/// re-order flips WHICH surfaces); assert only the non-zero exit. (No existing
/// test pins this precedence.)
#[test]
fn over_n_slot_vec_on_parse_failing_descriptor_still_refuses() {
    // `wsh(@0)` is not a valid miniscript leaf (bare key under wsh w/o wrapper)
    // → parse_descriptor probe errors; @0,@1 over-run n=1.
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            "wsh(@0)",
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.phrase={TREZOR_12_ZERO}"),
            "--slot",
            &format!("@1.phrase={BIP39_TEST_2}"),
        ])
        .assert()
        .failure();
}
