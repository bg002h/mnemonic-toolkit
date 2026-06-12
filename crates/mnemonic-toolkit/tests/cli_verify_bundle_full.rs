//! Verify-bundle full-mode round-trip integration test.

use assert_cmd::Command;
use predicates::prelude::*;

const TREZOR_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
const BIP39_TEST_2: &str =
    "legal winner thank year wave sausage worth useful legal winner thank yellow";

#[test]
fn verify_bundle_full_bip84_mainnet_round_trip() {
    let fixture =
        std::fs::read_to_string("tests/vectors/v0_1/bip84-mainnet.txt").expect("fixture exists");
    let ms1 = fixture
        .lines()
        .find(|l| l.starts_with("ms1") && !l.contains(' '))
        .expect("compact ms1 line")
        .to_string();
    let mk1: Vec<String> = fixture
        .lines()
        .filter(|l| l.starts_with("mk1") && !l.contains(' ') && !l.contains('-'))
        .map(String::from)
        .collect();
    let md1: Vec<String> = fixture
        .lines()
        .filter(|l| l.starts_with("md1") && !l.contains(' ') && !l.contains('-'))
        .map(String::from)
        .collect();
    assert!(!mk1.is_empty() && !md1.is_empty());

    let mut args: Vec<String> = vec![
        "verify-bundle".into(),
        "--slot".into(),
        format!("@0.phrase={TREZOR_24}"),
        "--network".into(),
        "mainnet".into(),
        "--template".into(),
        "bip84".into(),
        "--ms1".into(),
        ms1,
    ];
    for s in &mk1 {
        args.push("--mk1".into());
        args.push(s.clone());
    }
    for s in &md1 {
        args.push("--md1".into());
        args.push(s.clone());
    }

    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&args)
        .assert()
        .success()
        .stdout(predicate::str::contains("result: ok"));
}

// ============================================================================
// v0.25.0 §2.D — ms1-driven parent_fingerprint check at depth ≥ 2 (full path).
// ============================================================================

/// Helper: extract mk1 chunks (flattened) + ms1 + md1 vecs from a multisig
/// `mnemonic bundle --json` invocation.
fn gen_bundle_full_multisig(args: &[&str]) -> (Vec<String>, Vec<String>, Vec<String>) {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(args)
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let bundle: serde_json::Value = serde_json::from_str(&stdout).expect("valid bundle JSON");
    let ms1: Vec<String> = bundle["ms1"]
        .as_array()
        .expect("ms1 array")
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    let mut mk1: Vec<String> = Vec::new();
    for inner in bundle["mk1"].as_array().expect("mk1 array") {
        for chunk in inner.as_array().expect("inner mk1 array") {
            mk1.push(chunk.as_str().unwrap().to_string());
        }
    }
    let md1: Vec<String> = bundle["md1"]
        .as_array()
        .expect("md1 array")
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    (ms1, mk1, md1)
}

/// v0.25.0 §2.D cell — full-path multisig at depth ≥ 2 (canonical multisig
/// templates produce depth-3 paths via the BIP-87 default purpose): both ms1
/// and mk1 are derived from the same phrase → parent_fingerprint check fires
/// silently (no `warning:` / `notice:` from the new helper).
#[test]
fn full_path_parent_fp_matches_silent_at_depth_3() {
    let (ms1, mk1, md1) = gen_bundle_full_multisig(&[
        "bundle",
        "--network",
        "mainnet",
        "--template",
        "wsh-sortedmulti",
        "--threshold",
        "2",
        "--slot",
        &format!("@0.phrase={TREZOR_24}"),
        "--slot",
        &format!("@1.phrase={BIP39_TEST_2}"),
        "--json",
    ]);

    let mut args: Vec<String> = vec![
        "verify-bundle".into(),
        "--network".into(),
        "mainnet".into(),
        "--template".into(),
        "wsh-sortedmulti".into(),
        "--threshold".into(),
        "2".into(),
        "--slot".into(),
        format!("@0.phrase={TREZOR_24}"),
        "--slot".into(),
        format!("@1.phrase={BIP39_TEST_2}"),
    ];
    for s in &ms1 {
        args.push("--ms1".into());
        args.push(s.clone());
    }
    for s in &mk1 {
        args.push("--mk1".into());
        args.push(s.clone());
    }
    for s in &md1 {
        args.push("--md1".into());
        args.push(s.clone());
    }

    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&args)
        .assert()
        .success()
        .stdout(predicate::str::contains("result: ok"))
        // Silent on parent_fingerprint check — no v0.25.0 NOTICE / WARNING.
        .stderr(predicate::str::contains("notice: cosigner[").not())
        .stderr(predicate::str::contains("does not match derived parent fingerprint").not());
}

/// v0.25.0 §2.D cell — full-path multi-cosigner with cosigner[0]'s ms1
/// spliced from a DIFFERENT phrase than the cosigner[0] mk1. The
/// parent_fingerprint derived from the spliced ms1 will not match the mk1
/// claim → stderr WARNING. Exit 0 (warning, not error; permissive-input /
/// expressive-output).
#[test]
fn full_path_parent_fp_mismatch_warns_at_depth_3() {
    // Bundle from canonical (TREZOR_24 + BIP39_TEST_2). Take mk1 + md1 from
    // this. The expected ms1 for cosigner[0] is TREZOR_24-derived.
    let (_ms1_canonical, mk1, md1) = gen_bundle_full_multisig(&[
        "bundle",
        "--network",
        "mainnet",
        "--template",
        "wsh-sortedmulti",
        "--threshold",
        "2",
        "--slot",
        &format!("@0.phrase={TREZOR_24}"),
        "--slot",
        &format!("@1.phrase={BIP39_TEST_2}"),
        "--json",
    ]);
    // Generate a 3rd phrase's ms1; splice it into cosigner[0]'s slot. The
    // parent_fingerprint derived from this ms1 will differ from the mk1's
    // claim.
    const BIP39_TEST_3: &str =
        "letter advice cage absurd amount doctor acoustic avoid letter advice cage above";
    // Use a single-sig bip84 bundle to source an alternate ms1 (single string).
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--network",
            "mainnet",
            "--template",
            "bip84",
            "--slot",
            &format!("@0.phrase={BIP39_TEST_3}"),
            "--json",
        ])
        .assert()
        .success();
    let alt_stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let alt_bundle: serde_json::Value =
        serde_json::from_str(&alt_stdout).expect("valid bundle JSON");
    let ms1_alt: Vec<String> = alt_bundle["ms1"]
        .as_array()
        .expect("ms1 array")
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();

    let mut args: Vec<String> = vec![
        "verify-bundle".into(),
        "--network".into(),
        "mainnet".into(),
        "--template".into(),
        "wsh-sortedmulti".into(),
        "--threshold".into(),
        "2".into(),
        "--slot".into(),
        format!("@0.phrase={TREZOR_24}"),
        "--slot".into(),
        format!("@1.phrase={BIP39_TEST_2}"),
        // Spliced ms1 for cosigner[0] (from a different phrase / template);
        // cosigner[1] ms1 omitted → NOTICE for that index (depth-≥-2 watch-only).
        "--ms1".into(),
        ms1_alt[0].clone(),
    ];
    for s in &mk1 {
        args.push("--mk1".into());
        args.push(s.clone());
    }
    for s in &md1 {
        args.push("--md1".into());
        args.push(s.clone());
    }

    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&args)
        .assert()
        // verify-bundle exit code may be 4 (ms1_entropy_match[0] fail is
        // expected since the spliced ms1 differs from the expected one),
        // but the new helper's WARNING is what we assert.
        .stderr(predicate::str::contains(
            "warning: cosigner[0] mk1 xpub parent_fingerprint",
        ))
        .stderr(predicate::str::contains(
            "does not match derived parent fingerprint",
        ))
        .stderr(predicate::str::contains(
            "cards are internally inconsistent",
        ));
}

/// v0.25.0 §2.D cell — passphrase threading: when the user supplies
/// `--passphrase <pp>`, the helper's BIP-39 → seed step uses it. Bundle
/// the cards with a passphrase, then verify-bundle with the SAME passphrase →
/// silent (derived parent_fingerprint matches claimed). The cell verifies the
/// passphrase arg is threaded all the way through to the parent-fp
/// derivation; without the passphrase the derived parent would differ and a
/// warning would fire (negative control omitted to keep the cell focused —
/// the mismatch case is exercised by `full_path_parent_fp_mismatch_warns_at_depth_3`).
#[test]
fn full_path_passphrase_supplied_check_fires_with_passphrase() {
    const PASSPHRASE: &str = "mypass";
    let (ms1, mk1, md1) = gen_bundle_full_multisig(&[
        "bundle",
        "--network",
        "mainnet",
        "--template",
        "wsh-sortedmulti",
        "--threshold",
        "2",
        "--passphrase",
        PASSPHRASE,
        "--slot",
        &format!("@0.phrase={TREZOR_24}"),
        "--slot",
        &format!("@1.phrase={BIP39_TEST_2}"),
        "--json",
    ]);

    let mut args: Vec<String> = vec![
        "verify-bundle".into(),
        "--network".into(),
        "mainnet".into(),
        "--template".into(),
        "wsh-sortedmulti".into(),
        "--threshold".into(),
        "2".into(),
        "--passphrase".into(),
        PASSPHRASE.into(),
        "--slot".into(),
        format!("@0.phrase={TREZOR_24}"),
        "--slot".into(),
        format!("@1.phrase={BIP39_TEST_2}"),
    ];
    for s in &ms1 {
        args.push("--ms1".into());
        args.push(s.clone());
    }
    for s in &mk1 {
        args.push("--mk1".into());
        args.push(s.clone());
    }
    for s in &md1 {
        args.push("--md1".into());
        args.push(s.clone());
    }

    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&args)
        .assert()
        .success()
        .stdout(predicate::str::contains("result: ok"))
        // Passphrase threaded all the way through → derived parent_fp matches → silent.
        .stderr(predicate::str::contains("does not match derived parent fingerprint").not());
}
