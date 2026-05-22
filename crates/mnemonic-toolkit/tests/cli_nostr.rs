//! v0.34.0 — `mnemonic nostr` integration tests.
use assert_cmd::Command;
use predicates::prelude::*;

const NPUB: &str = "npub10elfcs4fr0l0r8af98jlmgdh9c8tcxjvz9qkw038js35mp4dma8qzvjptg";

#[test]
fn pubkey_default_p2tr_emits_descriptor_and_address() {
    Command::cargo_bin("mnemonic").unwrap()
        .args(["nostr", "--pubkey", NPUB])
        .assert()
        .success()
        .stdout(predicate::str::contains("script-type: p2tr"))
        .stdout(predicate::str::contains("descriptor:  tr("))
        .stdout(predicate::str::contains("address:     bc1p"));
}

const NSEC: &str = "nsec1vl029mgpspedva04g90vltkh6fvh240zqtv9k0t9af8935ke9laqsnlfe5";

#[test]
fn secret_emits_wif_and_electrum_hint() {
    Command::cargo_bin("mnemonic").unwrap()
        .args(["nostr", "--secret", NSEC, "--script-type", "p2wpkh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("wif:"))
        .stdout(predicate::str::contains("electrum:    p2wpkh:"));
}

#[test]
fn secret_via_stdin_works() {
    Command::cargo_bin("mnemonic").unwrap()
        .args(["nostr", "--secret-stdin", "--script-type", "p2tr"])
        .write_stdin(format!("{NSEC}\n"))
        .assert()
        .success()
        .stdout(predicate::str::contains("wif:"));
}

#[test]
fn all_script_types_emits_four() {
    Command::cargo_bin("mnemonic").unwrap()
        .args(["nostr", "--pubkey", NPUB, "--all-script-types"])
        .assert().success()
        .stdout(predicate::str::contains("tr("))
        .stdout(predicate::str::contains("wpkh("))
        .stdout(predicate::str::contains("sh(wpkh("));
}

#[test]
fn json_output_is_valid_and_has_fields() {
    let out = Command::cargo_bin("mnemonic").unwrap()
        .args(["nostr", "--pubkey", NPUB, "--json"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["kind"], "public");
    assert!(v["outputs"][0]["descriptor"].is_string());
    assert!(v["outputs"][0]["address"].is_string());
}

#[test]
fn json_secret_has_wif_and_electrum() {
    let out = Command::cargo_bin("mnemonic").unwrap()
        .args(["nostr", "--secret", NSEC, "--script-type", "p2wpkh", "--json"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["kind"], "secret");
    assert!(v["wif"].is_string());
    assert!(v["outputs"][0]["electrum"].is_string());
}

#[test]
fn no_key_input_is_refused() {
    Command::cargo_bin("mnemonic").unwrap().args(["nostr"]).assert().failure();
}

#[test]
fn pubkey_and_secret_together_is_refused() {
    Command::cargo_bin("mnemonic").unwrap()
        .args(["nostr", "--pubkey", NPUB, "--secret", NSEC])
        .assert().failure();
}

#[test]
fn nsec_to_pubkey_flag_is_refused() {
    Command::cargo_bin("mnemonic").unwrap()
        .args(["nostr", "--pubkey", NSEC])
        .assert().failure().stderr(predicate::str::contains("HRP"));
}

#[test]
fn secret_inline_warns_on_argv_and_stdout() {
    Command::cargo_bin("mnemonic").unwrap()
        .args(["nostr", "--secret", NSEC, "--script-type", "p2wpkh"])
        .assert().success()
        .stderr(predicate::str::contains("secret material on argv (--secret"))
        .stderr(predicate::str::contains("secret material on stdout"));
}

#[test]
fn secret_stdin_warns_on_stdout_but_not_argv() {
    Command::cargo_bin("mnemonic").unwrap()
        .args(["nostr", "--secret-stdin", "--script-type", "p2tr"])
        .write_stdin(format!("{NSEC}\n"))
        .assert().success()
        .stderr(predicate::str::contains("secret material on stdout"))
        .stderr(predicate::str::contains("secret material on argv").not());
}

#[test]
fn pubkey_path_has_no_wif_or_electrum() {
    Command::cargo_bin("mnemonic").unwrap()
        .args(["nostr", "--pubkey", NPUB])
        .assert().success()
        .stdout(predicate::str::contains("wif:").not())
        .stdout(predicate::str::contains("electrum:").not());
}

#[test]
fn all_script_types_emits_p2pkh_row() {
    Command::cargo_bin("mnemonic").unwrap()
        .args(["nostr", "--pubkey", NPUB, "--all-script-types"])
        .assert().success()
        .stdout(predicate::str::contains("script-type: p2pkh"))
        .stdout(predicate::str::contains("script-type: p2sh-p2wpkh"));
}

#[test]
fn p2tr_secret_has_no_electrum_line_but_p2wpkh_does() {
    // taproot: no Electrum WIF import → no electrum hint
    Command::cargo_bin("mnemonic").unwrap()
        .args(["nostr", "--secret", NSEC, "--script-type", "p2tr"])
        .assert().success()
        .stdout(predicate::str::contains("electrum:").not())
        .stdout(predicate::str::contains("wif:"));
    // p2wpkh: Electrum import supported → electrum hint present
    Command::cargo_bin("mnemonic").unwrap()
        .args(["nostr", "--secret", NSEC, "--script-type", "p2wpkh"])
        .assert().success()
        .stdout(predicate::str::contains("electrum:    p2wpkh:"));
}

fn import_json_from_stdout(s: &str) -> serde_json::Value {
    let inner = s
        .split("importdescriptors '")
        .nth(1)
        .expect("import line present")
        .split("'\n")
        .next()
        .expect("closing quote");
    serde_json::from_str(inner).expect("valid importdescriptors JSON")
}

#[test]
fn import_readonly_emits_watchonly_recipe() {
    let out = Command::cargo_bin("mnemonic").unwrap()
        .args(["nostr", "--pubkey", NPUB, "--script-type", "p2wpkh", "--import", "readonly"])
        .assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("import:      importdescriptors '["), "got: {s}");
    let v = import_json_from_stdout(&s);
    assert_eq!(v[0]["active"], false);
    assert_eq!(v[0]["internal"], false);
    assert_eq!(v[0]["timestamp"], 0); // default
    assert!(v[0]["desc"].as_str().unwrap().starts_with("wpkh("));
    assert!(v[0].get("range").is_none());
}

#[test]
fn import_all_script_types_one_array_four_entries() {
    let out = Command::cargo_bin("mnemonic").unwrap()
        .args(["nostr", "--pubkey", NPUB, "--all-script-types", "--import", "readonly"])
        .assert().success().get_output().stdout.clone();
    let v = import_json_from_stdout(&String::from_utf8(out).unwrap());
    assert_eq!(v.as_array().unwrap().len(), 4);
}

#[test]
fn import_spending_and_both_are_refused() {
    for mode in ["spending", "both"] {
        Command::cargo_bin("mnemonic").unwrap()
            .args(["nostr", "--pubkey", NPUB, "--import", mode])
            .assert().failure().stderr(predicate::str::contains("deferred to a future cycle"));
    }
}

#[test]
fn import_timestamp_flag_overrides_default() {
    let out = Command::cargo_bin("mnemonic").unwrap()
        .args(["nostr", "--pubkey", NPUB, "--script-type", "p2tr", "--import", "readonly", "--timestamp", "now"])
        .assert().success().get_output().stdout.clone();
    let v = import_json_from_stdout(&String::from_utf8(out).unwrap());
    assert_eq!(v[0]["timestamp"], "now");
}

#[test]
fn no_import_flag_emits_no_recipe() {
    Command::cargo_bin("mnemonic").unwrap()
        .args(["nostr", "--pubkey", NPUB])
        .assert().success().stdout(predicate::str::contains("import:").not());
}

#[test]
fn import_in_json_envelope() {
    let out = Command::cargo_bin("mnemonic").unwrap()
        .args(["nostr", "--pubkey", NPUB, "--import", "readonly", "--json"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert!(v["import"].is_array());
    assert_eq!(v["import"][0]["active"], false);
}
