//! #28 phase 1 — `mnemonic verify-bundle` on a keyless single-sig TEMPLATE
//! bundle: bind-via-template-id-stub + complete + recompose the watch-only
//! wallet, with `--expect-wallet-id`.

use assert_cmd::Command;

const PHRASE_A: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

fn mnemonic() -> Command {
    Command::cargo_bin("mnemonic").expect("mnemonic binary builds")
}

/// Emit a template bundle and return its (ms1, mk1, md1) unbroken card vectors.
fn template_cards(template: &str, phrase: &str, account: &str) -> (Vec<String>, Vec<String>, Vec<String>) {
    let out = mnemonic()
        .args([
            "bundle",
            "--template",
            template,
            "--network",
            "mainnet",
            "--md1-form",
            "template",
            "--account",
            account,
            "--group-size",
            "0",
            "--slot",
            &format!("@0.phrase={phrase}"),
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let mut ms1 = Vec::new();
    let mut mk1 = Vec::new();
    let mut md1 = Vec::new();
    let mut section = "";
    for line in stdout.lines() {
        if line.starts_with("# ms1") {
            section = "ms1";
            continue;
        }
        if line.starts_with("# mk1") {
            section = "mk1";
            continue;
        }
        if line.starts_with("# md1") {
            section = "md1";
            continue;
        }
        let t = line.trim();
        if t.is_empty() {
            section = "";
            continue;
        }
        match section {
            "ms1" => ms1.push(t.to_string()),
            "mk1" => mk1.push(t.to_string()),
            "md1" => md1.push(t.to_string()),
            _ => {}
        }
    }
    (ms1, mk1, md1)
}

fn verify_args(template: &str, phrase: &str, account: &str, expect: Option<&str>) -> Vec<String> {
    let (ms1, mk1, md1) = template_cards(template, phrase, account);
    let mut args = vec![
        "verify-bundle".to_string(),
        "--network".to_string(),
        "mainnet".to_string(),
        "--account".to_string(),
        account.to_string(),
        "--slot".to_string(),
        format!("@0.phrase={phrase}"),
    ];
    for m in &ms1 {
        if !m.is_empty() {
            args.push("--ms1".to_string());
            args.push(m.clone());
        }
    }
    for m in &mk1 {
        args.push("--mk1".to_string());
        args.push(m.clone());
    }
    for m in &md1 {
        args.push("--md1".to_string());
        args.push(m.clone());
    }
    if let Some(e) = expect {
        args.push("--expect-wallet-id".to_string());
        args.push(e.to_string());
    }
    args
}

#[test]
fn verify_template_bundle_recomposes_and_passes() {
    for template in ["bip44", "bip84", "bip86"] {
        let out = mnemonic()
            .args(verify_args(template, PHRASE_A, "0", None))
            .assert()
            .success();
        let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
        assert!(
            stdout.contains("OK") && stdout.contains("descriptor:"),
            "{template}: verify must recompose + report OK: {stdout}"
        );
    }
}

#[test]
fn verify_template_bundle_without_seed_is_refused() {
    let (_ms1, mk1, md1) = template_cards("bip84", PHRASE_A, "0");
    let mut args = vec![
        "verify-bundle".to_string(),
        "--network".to_string(),
        "mainnet".to_string(),
    ];
    for m in &mk1 {
        args.push("--mk1".to_string());
        args.push(m.clone());
    }
    for m in &md1 {
        args.push("--md1".to_string());
        args.push(m.clone());
    }
    // No --slot seed → refused (the template is keyless; nothing to recompose).
    let assert = mnemonic().args(&args).assert().failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("seed") || stderr.contains("--slot"),
        "missing-seed refusal must name the seed requirement: {stderr}"
    );
}

#[test]
fn verify_template_bundle_expect_wallet_id_wrong_mismatch() {
    mnemonic()
        .args(verify_args("bip84", PHRASE_A, "0", Some("deadbeefdeadbeef")))
        .assert()
        .failure()
        .code(4);
}

#[test]
fn verify_template_bundle_recompose_matches_restore() {
    // The recomposed descriptor from verify-bundle must equal the restore
    // template-completion descriptor (cross-tool consistency).
    let out_v = mnemonic()
        .args(verify_args("bip84", PHRASE_A, "0", None))
        .assert()
        .success();
    let v_stdout = String::from_utf8(out_v.get_output().stdout.clone()).unwrap();
    let v_desc = v_stdout
        .lines()
        .find_map(|l| l.trim().strip_prefix("descriptor:"))
        .map(|s| s.trim().to_string())
        .expect("verify descriptor");

    let (_ms1, _mk1, md1) = template_cards("bip84", PHRASE_A, "0");
    let mut rargs = vec![
        "restore".to_string(),
        "--from".to_string(),
        format!("phrase={PHRASE_A}"),
        "--network".to_string(),
        "mainnet".to_string(),
    ];
    for m in &md1 {
        rargs.push("--md1".to_string());
        rargs.push(m.clone());
    }
    let out_r = mnemonic().args(&rargs).assert().success();
    let r_stdout = String::from_utf8(out_r.get_output().stdout.clone()).unwrap();
    let r_desc = r_stdout
        .lines()
        .find_map(|l| l.trim().strip_prefix("descriptor:"))
        .map(|s| s.trim().to_string())
        .expect("restore descriptor");

    assert_eq!(v_desc, r_desc, "verify recompose must equal restore completion");
}
