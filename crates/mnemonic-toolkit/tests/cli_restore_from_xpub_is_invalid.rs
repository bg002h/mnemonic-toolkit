//! Final review M3: `--from xpub=…` (any node that is not a seed) handed to
//! `restore` is an invalid input (2), not a usage error (64), at each of the
//! `--from` sites: the plain restore, the keyless single-sig template, the
//! multisig wallet policy, and the keyless multisig template that
//! `verify-bundle` shares.

use assert_cmd::Command;
use serde_json::Value;

const PHRASE: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const FOREIGN: &str = "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong";
const XPUB: &str = "xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V";
const MESSAGE: &str = "--from xpub is not a seed source for restore";

fn mnemonic() -> Command {
    let mut c = Command::cargo_bin("mnemonic").expect("binary built");
    c.arg("--allow-argv-secret");
    c
}

fn bundle_json(extra: &[&str]) -> Value {
    let out = mnemonic()
        .args([
            "bundle",
            "--network",
            "mainnet",
            "--json",
            "--no-engraving-card",
        ])
        .args(extra)
        .output()
        .expect("spawn");
    serde_json::from_slice(&out.stdout).expect("bundle JSON")
}

fn strings(v: &Value, key: &str, flag: &str) -> Vec<String> {
    let mut a = Vec::new();
    for el in v[key].as_array().expect("array") {
        let chunks: Vec<&Value> = match el {
            Value::Array(inner) => inner.iter().collect(),
            other => vec![other],
        };
        for c in chunks {
            let c = c.as_str().expect("str");
            if !c.is_empty() {
                a.push(flag.to_owned());
                a.push(c.to_owned());
            }
        }
    }
    a
}

fn assert_invalid(name: &str, args: &[String]) {
    let out = mnemonic().args(args).output().expect("spawn");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(2), "{name}: {stderr}");
    assert!(stderr.contains(MESSAGE), "{name}: {stderr}");
    assert!(out.stdout.is_empty(), "{name}: nothing on stdout");
}

fn args(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| (*s).to_owned()).collect()
}

#[test]
fn final_m3_restore_from_xpub_is_2_on_every_path() {
    let from = format!("xpub={XPUB}");

    let mut plain = args(&["restore", "--template", "bip84", "--from"]);
    plain.push(from.clone());
    assert_invalid("plain restore", &plain);

    let single = bundle_json(&[
        "--template",
        "bip84",
        "--md1-form",
        "template",
        "--slot",
        &format!("@0.phrase={PHRASE}"),
    ]);
    let mut a = args(&["restore", "--network", "mainnet"]);
    a.extend(strings(&single, "md1", "--md1"));
    a.extend(["--from".to_owned(), from.clone()]);
    assert_invalid("restore --md1 <single-sig template>", &a);

    let policy = bundle_json(&[
        "--template",
        "wsh-sortedmulti",
        "--threshold",
        "2",
        "--slot",
        &format!("@0.phrase={PHRASE}"),
        "--slot",
        &format!("@1.phrase={FOREIGN}"),
    ]);
    let mut a = args(&["restore", "--network", "mainnet"]);
    a.extend(strings(&policy, "md1", "--md1"));
    a.extend(["--from".to_owned(), from.clone()]);
    assert_invalid("restore --md1 <multisig policy>", &a);

    let template = bundle_json(&[
        "--template",
        "wsh-sortedmulti",
        "--threshold",
        "2",
        "--md1-form",
        "template",
        "--slot",
        &format!("@0.phrase={PHRASE}"),
        "--slot",
        &format!("@1.phrase={FOREIGN}"),
    ]);
    let mut a = args(&["restore", "--network", "mainnet"]);
    a.extend(strings(&template, "md1", "--md1"));
    a.extend(["--from".to_owned(), from.clone()]);
    assert_invalid("restore --md1 <multisig template>", &a);

    let mut a = args(&["verify-bundle", "--network", "mainnet"]);
    a.extend(strings(&template, "md1", "--md1"));
    a.extend(strings(&template, "mk1", "--mk1"));
    a.extend(["--from".to_owned(), from]);
    assert_invalid("verify-bundle --md1 <multisig template>", &a);
}
