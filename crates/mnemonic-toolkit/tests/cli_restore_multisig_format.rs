//! v0.45.0 — `mnemonic restore --md1 --format <export-format>` (FOLLOWUP
//! `restore-multisig-format-payloads`). Multisig restore emits an importable
//! wallet-software payload (the same payload class as `export-wallet
//! --template <multisig> --format X`) by building a multisig `EmitInputs`
//! from the reconstructed (template, slots, k, descriptor) and running the
//! shared emitter dispatch. 9 emit / 2 refuse (specter/green). Watch-only-out.
//! See design/SPEC_restore_multisig_format_payloads.md.

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;

const C0: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const C1: &str = "legal winner thank year wave sausage worth useful legal winner thank yellow";
const C2: &str = "letter advice cage absurd amount doctor acoustic avoid letter advice cage above";
/// A seed that is NOT one of the three cosigners (mismatch-precedence cell).
const FOREIGN: &str = "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong";

/// The three cosigner MASTER fingerprints (C0/C1/C2), embedded by restore in
/// the `[fp/…]` key-origins of the descriptor-bearing formats.
const FP: [&str; 3] = ["73c5da0a", "b8688df1", "28645006"];

fn bundle_md1(template: &str, network: &str) -> Vec<String> {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle", "--template", template, "--threshold", "2", "--network", network,
            "--slot", &format!("@0.phrase={C0}"),
            "--slot", &format!("@1.phrase={C1}"),
            "--slot", &format!("@2.phrase={C2}"),
            "--json", "--no-engraving-card",
        ])
        .assert()
        .success();
    let v: Value = serde_json::from_slice(&out.get_output().stdout).expect("bundle JSON");
    v["md1"]
        .as_array()
        .expect("md1 array")
        .iter()
        .map(|x| x.as_str().unwrap().to_string())
        .collect()
}

fn restore_args(md1: &[String]) -> Vec<String> {
    let mut a = vec!["restore".to_string(), "--network".into(), "mainnet".into()];
    for c in md1 {
        a.push("--md1".into());
        a.push(c.clone());
    }
    a
}

/// `restore --md1 … --format X` stdout (asserts exit 0).
fn restore_format_stdout(md1: &[String], format: &str) -> String {
    let mut a = restore_args(md1);
    a.push("--format".into());
    a.push(format.into());
    let out = Command::cargo_bin("mnemonic").unwrap().args(&a).assert().code(0);
    String::from_utf8(out.get_output().stdout.clone()).unwrap()
}

/// The exact per-format threshold token (R0-r1 I1 — non-vacuous; a K=1
/// single-sig-ify lacks the `2`-threshold token).
fn threshold_token(format: &str) -> &'static str {
    match format {
        "descriptor" | "bitcoin-core" | "bip388" | "sparrow" | "bsms" => "sortedmulti(2,",
        "coldcard" | "coldcard-multisig" | "jade" => "Policy: 2 of",
        "electrum" => "2of3",
        other => panic!("no threshold token pinned for {other}"),
    }
}

// ─── EMIT × multisig-fidelity (the primary single-sig-ify check) ────────────

#[test]
fn emit_all_formats_carry_threshold_token() {
    let md1 = bundle_md1("wsh-sortedmulti", "mainnet");
    for fmt in [
        "descriptor", "bitcoin-core", "bip388", "sparrow", "bsms",
        "coldcard", "coldcard-multisig", "jade", "electrum",
    ] {
        let payload = restore_format_stdout(&md1, fmt);
        let tok = threshold_token(fmt);
        assert!(
            payload.contains(tok),
            "format {fmt}: payload missing threshold token {tok:?}\n{payload}"
        );
    }
}

#[test]
fn emit_fp_embedding_formats_carry_all_three_fingerprints() {
    let md1 = bundle_md1("wsh-sortedmulti", "mainnet");
    // descriptor / bitcoin-core / bsms embed `[fp/…]` hex key-origins; restore
    // emits the md1's REAL master fps (proves the right 3 cosigners + drop-a-
    // cosigner). bip388/sparrow carry fps in a non-descriptor field — excluded.
    for fmt in ["descriptor", "bitcoin-core", "bsms"] {
        let payload = restore_format_stdout(&md1, fmt).to_lowercase();
        for fp in FP {
            assert!(
                payload.contains(fp),
                "format {fmt}: payload missing cosigner fingerprint {fp}"
            );
        }
    }
}

/// v0.47.3 (SPEC_timestamp_default_zero): the MULTISIG `restore --md1 --format
/// bitcoin-core` path (the v0.45.0 `build_multisig_import_payload`, the 2nd of
/// restore's two hardcoded `TimestampArg::Now` sites) must also emit `timestamp:
/// 0` (genesis rescan), not `"now"`. RED against the pre-v0.47.3 hardcode
/// (`as_u64()` returns None for the `"now"` string).
#[test]
fn restore_md1_format_bitcoin_core_default_timestamp_is_zero() {
    let md1 = bundle_md1("wsh-sortedmulti", "mainnet");
    let stdout = restore_format_stdout(&md1, "bitcoin-core");
    let v: Value = serde_json::from_str(&stdout).expect("importdescriptors array");
    let arr = v.as_array().expect("array");
    for (i, entry) in arr.iter().enumerate() {
        assert_eq!(
            entry["timestamp"].as_u64().unwrap_or_else(|| panic!(
                "entry {i} timestamp must be the number 0, not {:?}",
                entry["timestamp"]
            )),
            0,
            "restore --md1 --format bitcoin-core must emit timestamp 0:\n{stdout}"
        );
    }
}

// ─── `--format descriptor` exact-equality (genuine byte-parity) ─────────────

#[test]
fn format_descriptor_equals_json_descriptor() {
    let md1 = bundle_md1("wsh-sortedmulti", "mainnet");
    // bare-descriptor payload == the same run's reconstructed descriptor.
    let payload = restore_format_stdout(&md1, "descriptor");
    let mut a = restore_args(&md1);
    a.push("--json".into());
    let out = Command::cargo_bin("mnemonic").unwrap().args(&a).assert().code(0);
    let v: Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    let json_desc = v["wallets"][0]["descriptor"].as_str().unwrap();
    assert_eq!(payload.trim_end(), json_desc, "--format descriptor != --json descriptor");
}

// ─── Refusals (match export-wallet) ─────────────────────────────────────────

#[test]
fn format_specter_refuses_missing_wallet_name() {
    let md1 = bundle_md1("wsh-sortedmulti", "mainnet");
    let mut a = restore_args(&md1);
    a.push("--format".into());
    a.push("specter".into());
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&a)
        .assert()
        .code(2)
        .stderr(predicate::str::contains("missing"));
}

#[test]
fn format_green_refuses_no_multisig() {
    let md1 = bundle_md1("wsh-sortedmulti", "mainnet");
    let mut a = restore_args(&md1);
    a.push("--format".into());
    a.push("green".into());
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&a)
        .assert()
        .code(1)
        .stderr(predicate::str::contains("multisig"));
}

// ─── Watch-only-out (no private material in any channel) ────────────────────

#[test]
fn format_payloads_are_watch_only() {
    let md1 = bundle_md1("wsh-sortedmulti", "mainnet");
    for fmt in ["descriptor", "bitcoin-core", "bsms"] {
        let mut a = restore_args(&md1);
        a.push("--format".into());
        a.push(fmt.into());
        let out = Command::cargo_bin("mnemonic").unwrap().args(&a).assert().code(0);
        let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
        let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
        for chan in [&stdout, &stderr] {
            assert!(!chan.contains("xprv"), "format {fmt}: xprv leaked");
            assert!(!chan.contains("tprv"), "format {fmt}: tprv leaked");
        }
    }
}

// ─── `--json` envelope carries import_payload + stays multisig ──────────────

#[test]
fn json_envelope_carries_import_payload() {
    let md1 = bundle_md1("wsh-sortedmulti", "mainnet");
    let mut a = restore_args(&md1);
    a.push("--format".into());
    a.push("bitcoin-core".into());
    a.push("--json".into());
    let out = Command::cargo_bin("mnemonic").unwrap().args(&a).assert().code(0);
    let v: Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    assert_eq!(v["mode"], "multisig");
    assert_eq!(v["threshold"], 2);
    assert_eq!(v["cosigners"], 3);
    let payload = v["import_payload"].as_str().expect("import_payload field");
    assert!(payload.contains("sortedmulti(2,"), "import_payload not multisig");
}

// ─── Mismatch precedence (exit 4 BEFORE any payload) ────────────────────────

#[test]
fn mismatch_blocks_payload_exit4() {
    let md1 = bundle_md1("wsh-sortedmulti", "mainnet");
    let mut a = restore_args(&md1);
    a.push("--from".into());
    a.push(format!("phrase={FOREIGN}"));
    a.push("--format".into());
    a.push("descriptor".into());
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&a)
        .assert()
        .code(4)
        // No descriptor payload on stdout (the refusal precedes emit).
        .stdout(predicate::str::contains("sortedmulti").not());
}

// ─── sh(wsh(sortedmulti)) emit ──────────────────────────────────────────────

#[test]
fn sh_wsh_sortedmulti_format_descriptor() {
    let md1 = bundle_md1("sh-wsh-sortedmulti", "mainnet");
    let payload = restore_format_stdout(&md1, "descriptor");
    assert!(
        payload.contains("sh(wsh(sortedmulti(2,"),
        "sh-wsh payload not nested-multisig:\n{payload}"
    );
}

// ─── `--output FILE` routes payload to file, verification to stderr ─────────

#[test]
fn format_output_file_routes_payload() {
    let md1 = bundle_md1("wsh-sortedmulti", "mainnet");
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("wallet.txt");
    let mut a = restore_args(&md1);
    a.push("--format".into());
    a.push("descriptor".into());
    a.push("--output".into());
    a.push(path.to_str().unwrap().into());
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&a)
        .assert()
        .code(0)
        // verification doc goes to stderr when --format + --output.
        .stderr(predicate::str::contains("cosigner @0"));
    let written = std::fs::read_to_string(&path).unwrap();
    assert!(written.contains("sortedmulti(2,"), "file missing descriptor payload");
}

/// (v2) taproot NUMS multisig `--format` thread-through: a `tr-sortedmulti-a`
/// md1 → `--format descriptor` emits the NUMS `tr(...,sortedmulti_a(2,...))`
/// (proves `build_multisig_import_payload` threads `Some(Nums)` — without it the
/// payload descriptor would carry a wrong/absent internal key).
#[test]
fn taproot_format_descriptor_carries_nums_sortedmulti_a() {
    const NUMS_HEX: &str = "50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0";
    let md1 = bundle_md1("tr-sortedmulti-a", "mainnet");
    let out = restore_format_stdout(&md1, "descriptor");
    assert!(
        out.contains(&format!("tr({NUMS_HEX},sortedmulti_a(2,")),
        "taproot --format descriptor must carry the NUMS internal key: {out}"
    );
}
