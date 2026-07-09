//! Cycle F (`ms1-repair-demote-to-candidate`) — Phase P0 (toolkit) SPEC-driven
//! integration tests.
//!
//! See `design/SPEC_ms1_repair_demote_to_candidate.md` §5 and
//! `design/IMPLEMENTATION_PLAN_ms1_repair_demote_to_candidate.md` Phase P0.
//!
//! This file carries the funds-critical §5.5 wrong-bundle ground-truth-
//! mismatch anchor (single-sig + multisig) — the C1 mechanism's core
//! guarantee: a `verify-bundle` auto-repair candidate is blessed ONLY when it
//! byte-matches the user's TYPED (expected) seed, never merely because it
//! decodes. The remaining SPEC §5 items are covered elsewhere (pre-existing
//! cells flipped in place, per the TEST-FLIP INVENTORY):
//!   - §5.1 (standalone `mnemonic repair --ms1` exit 4 + advisory) —
//!     `cli_repair.rs::cell_9_text_form_ms1_happy_path_exit_4_candidate_with_report`
//!     + `cell_9b_clean_ms1_stays_exit_0`.
//!   - §5.3 (convert/inspect/xpub-search auto-repair fall-through + advisory,
//!     exit 1) — `cli_auto_repair.rs::cell_19_*` / `cell_18b_*`,
//!     `cli_xpub_search_path_of_xpub.rs::path_of_xpub_ms1_decode_failure_auto_fires`.
//!   - §5.4 (verify-bundle MATCH) — `cli_auto_repair.rs::cell_27_*` /
//!     `cell_30_*`, plus the unit-level
//!     `verify_bundle::helper_tests::cycle_f_*_match` cells below in
//!     `src/cmd/verify_bundle.rs`.
//!   - §5.6 (indel keep-5 / ambiguous-4) — unaffected pre-existing coverage in
//!     `cli_indel.rs` (pure-indel path never routes through the substitution
//!     arm this cycle touches) + `repair.rs::indel_exit_code_precedence` /
//!     `indel.rs::recover_indel_reports_ambiguous_on_multiple_distinct_recovered`;
//!     re-pinned here (§5.6 CLI-level) for direct Cycle-F traceability.
//!   - §5.7 (mixed-kind OR-fold to exit 4) —
//!     `cli_positional_hrp_autodetect.rs::repair_mixed_positional_and_flag_combined_routing`
//!     (already an ms1+mk1 mixed invocation); re-pinned here too.
//!   - §5.8 (`--no-auto-repair` suppresses both) — `cli_auto_repair.rs::cell_28_*`
//!     (pre-existing, unaffected) + a direct assertion below.
//!   - §5.9 (`--json` `verdict`) — `cli_repair.rs::cell_10_*` / `cell_10b_*`.
//!   - §8.6 (secret-hygiene redaction) — unit-level in
//!     `verify_bundle::helper_tests::cycle_f_mismatch_redacts_seed_bytes`
//!     below in `src/cmd/verify_bundle.rs`, plus a CLI-level scan here.

use assert_cmd::Command;
use predicates::prelude::*;

/// Wallet A's seed (bip84 mainnet, canonical zero-entropy phrase — same
/// fixture reused across the repair test suite).
const PHRASE_A: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
/// A DIFFERENT seed (E) — the typed ground truth for the wrong-bundle
/// attack. Any valid BIP-39 phrase distinct from PHRASE_A works.
const PHRASE_E: &str =
    "legal winner thank year wave sausage worth useful legal winner thank yellow";
/// A third seed, used as cosigner 1 in the multisig analogue (kept fixed and
/// correct across both the "expected" and "supplied" sides).
const PHRASE_COSIGNER1: &str =
    "letter advice cage absurd amount doctor acoustic avoid letter advice cage above";

fn flip_at(chunk: &str, pos: usize) -> String {
    const ALPHABET: &str = "qpzry9x8gf2tvdw0s3jn54khce6mua7l";
    let sep = chunk.rfind('1').unwrap();
    let (prefix, rest) = chunk.split_at(sep + 1);
    let mut chars: Vec<char> = rest.chars().collect();
    let was = chars[pos];
    let was_idx = ALPHABET.find(was).unwrap();
    let new_idx = (was_idx + 1) % 32;
    chars[pos] = ALPHABET.chars().nth(new_idx).unwrap();
    let mut out = String::from(prefix);
    for c in chars {
        out.push(c);
    }
    out
}

/// Generate a clean bip84 mainnet single-sig bundle for `phrase`; returns
/// (ms1, mk1_chunks, md1_chunks).
fn gen_bundle_single(phrase: &str) -> (String, Vec<String>, Vec<String>) {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--network",
            "mainnet",
            "--template",
            "bip84",
            "--slot",
            &format!("@0.phrase={phrase}"),
            "--json",
            "--no-engraving-card",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid bundle JSON");
    let ms1 = v["ms1"][0].as_str().expect("ms1[0]").to_string();
    let mk1: Vec<String> = v["mk1"]
        .as_array()
        .expect("mk1 array")
        .iter()
        .map(|s| s.as_str().unwrap().to_string())
        .collect();
    let md1: Vec<String> = v["md1"]
        .as_array()
        .expect("md1 array")
        .iter()
        .map(|s| s.as_str().unwrap().to_string())
        .collect();
    (ms1, mk1, md1)
}

/// Generate a clean 2-of-2 wsh-sortedmulti mainnet multisig bundle from two
/// phrases; returns (ms1_per_cosigner, mk1_chunks_flattened, md1_chunks).
fn gen_bundle_multisig(phrase0: &str, phrase1: &str) -> (Vec<String>, Vec<String>, Vec<String>) {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--network",
            "mainnet",
            "--template",
            "wsh-sortedmulti",
            "--threshold",
            "2",
            "--slot",
            &format!("@0.phrase={phrase0}"),
            "--slot",
            &format!("@1.phrase={phrase1}"),
            "--json",
            "--no-engraving-card",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid bundle JSON");
    let ms1: Vec<String> = v["ms1"]
        .as_array()
        .expect("ms1 array")
        .iter()
        .map(|s| s.as_str().unwrap().to_string())
        .collect();
    let mut mk1: Vec<String> = Vec::new();
    for inner in v["mk1"].as_array().expect("mk1 array") {
        for chunk in inner.as_array().expect("inner mk1 array") {
            mk1.push(chunk.as_str().unwrap().to_string());
        }
    }
    let md1: Vec<String> = v["md1"]
        .as_array()
        .expect("md1 array")
        .iter()
        .map(|s| s.as_str().unwrap().to_string())
        .collect();
    (ms1, mk1, md1)
}

// ============================================================================
// §5.5 (FUNDS ANCHOR) — single-sig wrong-bundle attack.
// ============================================================================

/// `--slot @0.phrase=<seed E> --ms1 <corrupted, corrects to wallet A's clean
/// ms1> --mk1 <clean mk1 A> --md1 <md1 A>` → the corrected candidate (A) ≠
/// `expected.ms1[0]` (E) → `ms1_entropy_match` FAILS, full check table,
/// `result: mismatch`, exit 4. NOT "recovered", NO exit 5, NO exit-2 abort,
/// NO short-circuit (G2/G3).
#[test]
fn verify_bundle_ms1_ground_truth_mismatch_wrong_bundle_exit_4() {
    let (ms1_a, mk1_a, md1_a) = gen_bundle_single(PHRASE_A);
    let bad_ms1_a = flip_at(&ms1_a, 17);
    assert_ne!(bad_ms1_a, ms1_a);

    let mut args: Vec<String> = vec![
        "verify-bundle".into(),
        "--network".into(),
        "mainnet".into(),
        "--template".into(),
        "bip84".into(),
        "--slot".into(),
        format!("@0.phrase={PHRASE_E}"),
        "--ms1".into(),
        bad_ms1_a.clone(),
    ];
    for c in &mk1_a {
        args.push("--mk1".into());
        args.push(c.clone());
    }
    for c in &md1_a {
        args.push("--md1".into());
        args.push(c.clone());
    }

    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .env("MNEMONIC_FORCE_TTY", "1")
        .args(&args)
        .assert()
        .code(4)
        .get_output()
        .clone();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains("ms1_entropy_match: fail"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("this card is not a card for this seed"),
        "stdout: {stdout}"
    );
    assert!(
        !stdout.contains("recovered via auto-repair, confirmed"),
        "must NOT be marked recovered/confirmed on a ground-truth mismatch; stdout: {stdout}"
    );
    assert!(stdout.contains("result: mismatch"), "stdout: {stdout}");

    // §8.6 secret hygiene (G5): neither the corrected (A) nor the supplied
    // corrupted candidate seed bytes appear anywhere in stdout/stderr.
    let stderr = String::from_utf8(out.stderr).unwrap();
    for secret in [ms1_a.as_str(), bad_ms1_a.as_str()] {
        assert!(!stdout.contains(secret), "stdout leaked a seed string");
        assert!(!stderr.contains(secret), "stderr leaked a seed string");
    }
}

/// Multisig analogue of the above: cosigner[0]'s supplied ms1 is a corrupted
/// string that corrects to cosigner[0]'s OWN clean card, but the `--slot @0`
/// ground truth is a DIFFERENT seed (E) — mismatch on `ms1_entropy_match[0]`,
/// exit 4. Cosigner[1] (unaffected) stays consistent throughout.
#[test]
fn verify_bundle_ms1_ground_truth_mismatch_wrong_bundle_multisig_exit_4() {
    let (ms1s, mk1_chunks, md1_chunks) = gen_bundle_multisig(PHRASE_A, PHRASE_COSIGNER1);
    let ms1_cosigner0 = &ms1s[0];
    let ms1_cosigner1 = &ms1s[1];
    let bad_ms1_cosigner0 = flip_at(ms1_cosigner0, 17);
    assert_ne!(&bad_ms1_cosigner0, ms1_cosigner0);

    let mut args: Vec<String> = vec![
        "verify-bundle".into(),
        "--network".into(),
        "mainnet".into(),
        "--template".into(),
        "wsh-sortedmulti".into(),
        "--threshold".into(),
        "2".into(),
        "--slot".into(),
        format!("@0.phrase={PHRASE_E}"),
        "--slot".into(),
        format!("@1.phrase={PHRASE_COSIGNER1}"),
        "--ms1".into(),
        bad_ms1_cosigner0.clone(),
        "--ms1".into(),
        ms1_cosigner1.clone(),
    ];
    for c in &mk1_chunks {
        args.push("--mk1".into());
        args.push(c.clone());
    }
    for c in &md1_chunks {
        args.push("--md1".into());
        args.push(c.clone());
    }

    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .env("MNEMONIC_FORCE_TTY", "1")
        .args(&args)
        .assert()
        .code(4)
        .get_output()
        .clone();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains("ms1_entropy_match[0]: fail"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("this card is not a card for this seed"),
        "stdout: {stdout}"
    );
    assert!(
        !stdout.contains("cosigner[0] recovered via auto-repair, confirmed"),
        "must NOT be marked recovered/confirmed on a ground-truth mismatch; stdout: {stdout}"
    );
}

// ============================================================================
// §5.6 (indel keep-5 / ambiguous-4) — Cycle-F direct re-pin. The substitution
// demotion this cycle adds does NOT touch the indel recovery path (a
// separate oracle — `recover_indel_card` / `Ms1IndelOracle` — that
// RE-VALIDATES the full BCH checksum rather than spending it); confirm the
// carve-out (SPEC §3) survives unmodified.
// ============================================================================

/// A unique full-checksum indel candidate stays exit 5 (kept, justified per
/// SPEC §3 — cryptographically stronger than the 32-bit cross-chunk hash mk1
/// is ALREADY blessed on).
#[test]
fn ms1_unique_indel_recovery_stays_exit_5_post_cycle_f() {
    const VALID_MS1: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
    // Drop one payload char (data-index 1, 'e') → too-short-by-1, uniquely
    // recoverable (mirrors `cli_indel.rs::ms1_too_short_recovers_exit_5`).
    let mut out = String::from(VALID_MS1);
    out.remove(3 + 1);
    Command::cargo_bin("mnemonic")
        .unwrap()
        .env("MNEMONIC_FORCE_TTY", "1")
        .args(["repair", "--ms1", &out, "--max-indel", "1"])
        .assert()
        .code(5)
        .stdout(predicate::str::contains(VALID_MS1));
}

// ============================================================================
// §5.7 (mixed-kind OR-fold) — Cycle-F direct re-pin (the primary coverage is
// `cli_positional_hrp_autodetect.rs::repair_mixed_positional_and_flag_combined_routing`,
// already flipped to exit 4).
// ============================================================================

/// `mnemonic repair --ms1 <corrupted> --mk1 <clean>` → the ms1 candidate
/// dominates the clean mk1 in the OR-fold → exit 4.
#[test]
fn repair_mixed_ms1_candidate_and_clean_mk1_exit_4_dominates() {
    const VALID_MS1: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
    const VALID_MK1_CHUNK0: &str = "mk1qprsqhpqqsq3cqtsleeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8qp3yxg3dpe854wq4";
    let bad_ms1 = flip_at(VALID_MS1, 17);
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["repair", "--ms1", &bad_ms1, "--mk1", VALID_MK1_CHUNK0])
        .assert()
        .code(4)
        .stdout(predicate::str::contains(VALID_MS1))
        .stdout(predicate::str::contains(VALID_MK1_CHUNK0));
}

// ============================================================================
// §5.8 — `--no-auto-repair` suppresses BOTH the advisory (standalone-inline)
// AND the verify-bundle ground-truth compare.
// ============================================================================

/// verify-bundle with `--no-auto-repair` under TTY: the corrected-vs-expected
/// compare never runs — a decode-failing ms1 surfaces the legacy
/// decode-error check row (`ms1_decode: fail`), NOT a "recovered" note, even
/// when the correction WOULD have matched the expected seed.
#[test]
fn verify_bundle_no_auto_repair_suppresses_ground_truth_compare() {
    let (ms1_a, mk1_a, md1_a) = gen_bundle_single(PHRASE_A);
    let bad_ms1_a = flip_at(&ms1_a, 17);

    let mut args: Vec<String> = vec![
        "--no-auto-repair".into(),
        "verify-bundle".into(),
        "--network".into(),
        "mainnet".into(),
        "--template".into(),
        "bip84".into(),
        "--slot".into(),
        format!("@0.phrase={PHRASE_A}"),
        "--ms1".into(),
        bad_ms1_a,
    ];
    for c in &mk1_a {
        args.push("--mk1".into());
        args.push(c.clone());
    }
    for c in &md1_a {
        args.push("--md1".into());
        args.push(c.clone());
    }

    Command::cargo_bin("mnemonic")
        .unwrap()
        .env("MNEMONIC_FORCE_TTY", "1")
        .args(&args)
        .assert()
        .code(4)
        .stdout(predicate::str::contains("ms1_decode: fail"))
        .stdout(predicate::str::contains("recovered via auto-repair").not());
}
