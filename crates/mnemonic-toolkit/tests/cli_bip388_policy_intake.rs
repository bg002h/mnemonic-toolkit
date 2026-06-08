//! Cycle D — `export-wallet` / `bundle` `--descriptor` accept a BIP-388
//! wallet-policy JSON `{name, description_template, keys_info}` and expand it
//! to a concrete descriptor before the existing pipeline. The inverse of
//! `export-wallet --format bip388`. See
//! `design/SPEC_bip388_policy_descriptor_expansion.md`.

use assert_cmd::Command;

/// Two valid mainnet account xpubs (m/48'/0'/0'/2') reused from the descriptor
/// test fixtures. Origin-annotated keys_info → bundle-compatible.
const A: &str = "xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX";
const B: &str = "xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6";

/// 2-of-2 sortedmulti BIP-388 wallet policy, origin-annotated keys_info.
fn policy_2of2() -> String {
    format!(
        r#"{{"name":"test-vault","description_template":"wsh(sortedmulti(2,@0/**,@1/**))","keys_info":["[704c7836/48'/0'/0'/2']{A}","[97139860/48'/0'/0'/2']{B}"]}}"#
    )
}

/// Single-key wpkh policy (n=1 → SingleSigWatchOnly in bundle).
fn policy_singlesig() -> String {
    format!(
        r#"{{"name":"single","description_template":"wpkh(@0/**)","keys_info":["[704c7836/84'/0'/0']{A}"]}}"#
    )
}

fn run_ok(args: &[&str]) -> String {
    let out = Command::cargo_bin("mnemonic").unwrap().args(args).assert().success();
    String::from_utf8(out.get_output().stdout.clone()).unwrap()
}

/// Load-bearing round-trip: policy → `--format descriptor` → concrete, then
/// concrete → `--format bip388` reproduces the original `description_template`
/// + `keys_info` byte-for-byte (modulo the dropped `name`, per SPEC §3).
#[test]
fn export_wallet_descriptor_bip388_policy_roundtrips() {
    let policy = policy_2of2();
    let concrete = run_ok(&[
        "export-wallet", "--descriptor", &policy, "--format", "descriptor",
    ]);
    let concrete_line = concrete.lines().next().unwrap();
    assert!(concrete_line.starts_with("wsh(sortedmulti(2,"), "{concrete_line}");
    assert!(concrete_line.contains("[704c7836/48'/0'/0'/2']"), "{concrete_line}");
    assert!(concrete_line.contains("[97139860/48'/0'/0'/2']"), "{concrete_line}");
    assert!(concrete_line.contains('#'), "must carry checksum: {concrete_line}");

    // Forward again → reproduces the original policy template + keys.
    let reemit = run_ok(&[
        "export-wallet", "--descriptor", concrete_line, "--format", "bip388",
    ]);
    let v: serde_json::Value = serde_json::from_str(&reemit).unwrap();
    assert_eq!(
        v["description_template"].as_str().unwrap(),
        "wsh(sortedmulti(2,@0/**,@1/**))",
    );
    let keys: Vec<&str> = v["keys_info"].as_array().unwrap().iter().map(|k| k.as_str().unwrap()).collect();
    assert_eq!(keys, vec![
        format!("[704c7836/48'/0'/0'/2']{A}"),
        format!("[97139860/48'/0'/0'/2']{B}"),
    ]);
}

/// Policy → `--format bitcoin-core` emits a 2-entry (receive+change) watch-only
/// importdescriptors array.
#[test]
fn export_wallet_descriptor_bip388_policy_to_bitcoin_core() {
    let out = run_ok(&[
        "export-wallet", "--descriptor", &policy_2of2(), "--format", "bitcoin-core",
    ]);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    let arr = v.as_array().expect("bitcoin-core emits a JSON array");
    assert_eq!(arr.len(), 2, "receive + change");
    for entry in arr {
        assert!(entry["desc"].as_str().unwrap().contains("sortedmulti(2,"));
    }
}

/// Ordering invariant (load-bearing): a raw policy JSON must NOT trip the
/// `is_at_n_form` refusal — the policy pre-check runs first. (If the order were
/// reversed, `@0/**` inside `description_template` matches the @N probe and the
/// `"accepts only concrete descriptors"` refusal fires.)
#[test]
fn export_wallet_raw_policy_not_refused_by_at_n_guard() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["export-wallet", "--descriptor", &policy_2of2(), "--format", "descriptor"])
        .assert()
        .success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(!stderr.contains("accepts only concrete descriptors"), "stderr: {stderr}");
}

/// Malformed policy: `description_template` references `@N` beyond `keys_info` →
/// the explicit, early `@N beyond keys_info` refusal (not a downstream parse).
#[test]
fn export_wallet_bip388_policy_at_n_beyond_keys_info_refused() {
    let bad = format!(
        r#"{{"name":"x","description_template":"wsh(multi(2,@0/**,@1/**))","keys_info":["[704c7836/48'/0'/0'/2']{A}"]}}"#
    );
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["export-wallet", "--descriptor", &bad, "--format", "descriptor"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("@N beyond keys_info"));
}

/// `bundle --descriptor <policy>` → watch-only m-format bundle: ms1 omitted,
/// one mk1 per cosigner, one md1. (2-of-2 → MultisigWatchOnly.)
#[test]
fn bundle_descriptor_bip388_policy_watch_only() {
    let out = run_ok(&["bundle", "--descriptor", &policy_2of2(), "--network", "mainnet"]);
    assert!(out.contains("# ms1 (omitted"), "ms1 omitted: {out}");
    assert!(out.contains("# mk1[0]"), "mk1[0]: {out}");
    assert!(out.contains("# mk1[1]"), "mk1[1]: {out}");
    assert!(!out.contains("# mk1[2]"), "exactly 2 cosigners");
    assert!(out.contains("# md1"), "md1 card: {out}");
}

/// n=1 policy → bundle SingleSigWatchOnly path (still emits md1 + one mk1; no
/// secret). Pins the M-1 single-key shape.
#[test]
fn bundle_descriptor_bip388_singlesig_policy_watch_only() {
    let out = run_ok(&["bundle", "--descriptor", &policy_singlesig(), "--network", "mainnet"]);
    // Single-sig labels the card `# mk1` (no `[N]` index), unlike multisig.
    assert!(out.contains("# mk1 "), "single-sig mk1 card: {out}");
    assert!(!out.contains("# mk1["), "no indexed cards in single-sig: {out}");
    assert!(out.contains("# md1"), "md1 card: {out}");
}

/// Bundle requires origin-annotated keys_info (M-2): a bare-xpub policy expands
/// fine but the bundle slot-resolver refuses it (classify (false,false) — no
/// `[fp/path]` origin). export-wallet accepts the same input (covered above).
#[test]
fn bundle_descriptor_bip388_bare_key_policy_refused() {
    let bare = format!(
        r#"{{"name":"x","description_template":"wpkh(@0/**)","keys_info":["{A}"]}}"#
    );
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["bundle", "--descriptor", &bare, "--network", "mainnet"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("must carry a key origin"));
}

/// I-2(c): xpub-search delegates to the shared expander, so the improved
/// malformed-`@N` error surfaces there too (was a downstream miniscript parse
/// error before the dedup; pin it so the improvement doesn't silently regress).
#[test]
fn xpub_search_bip388_policy_at_n_beyond_keys_info_refused() {
    let bad = format!(
        r#"{{"name":"x","description_template":"wsh(multi(2,@0/**,@1/**))","keys_info":["[704c7836/48'/0'/0'/2']{A}"]}}"#
    );
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search", "account-of-descriptor", "--descriptor", &bad,
            "--phrase",
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains("@N beyond keys_info"));
}
