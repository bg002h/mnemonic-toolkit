//! Integration tests for `mnemonic addresses` (batch watch-only derivation).
//!
//! Expected addresses are independently computed by `mnemonic convert --to
//! address` (different code path: leaf-from-master vs account-xpub→derive_pub),
//! so a shared bug would not mask. Xpubs/phrase are the all-zeros corpus.

use assert_cmd::Command;
use std::process::Output;

const V2_84_MAIN: &str = "xpub6BmeGmRo4LosAcU21HDaGcvtaQ7GrqQcY48nBkE22qM6KVwQUjRJ1BGzk84SFVHgLcd61Vcnhr8petHexjjn5WbQ9PriVrRhphw4oCp2z6a";
const V15_84_TEST: &str = "tpubDC9Go1KDateW3gS8VXZ6DD1Xu7PgoTdPcf1MX9Z6qVLiHbaeDJ78swPyuQ8YQY19QjtrzkfkZSXwqCcb7XArtid1iLq8Vy55Ydfm4giZh6X";
const ABANDON: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const FRENCH_12: &str = "abaisser abaisser abaisser abaisser abaisser abaisser abaisser abaisser abaisser abaisser abaisser abeille";

// Independent oracles (via `mnemonic convert --to address`):
const V2_M0_0_P2WPKH: &str = "bc1qfjxgzvdwrxh9ejp6jmdlr9tc6lfl6adcsx2z4f";
const ABANDON_ACCT0_M0_0: &str = "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu"; // m/84'/0'/0'/0/0
const ABANDON_ACCT1_M0_0: &str = "bc1qku0qh0mc00y8tk0n65x2tqw4trlspak0fnjmfz"; // m/84'/0'/1'/0/0

fn mn(args: &[&str]) -> Output {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(args)
        .output()
        .unwrap()
}
fn stdout(o: &Output) -> String {
    String::from_utf8(o.stdout.clone()).unwrap()
}
fn stderr(o: &Output) -> String {
    String::from_utf8(o.stderr.clone()).unwrap()
}
fn code(o: &Output) -> i32 {
    o.status.code().unwrap()
}

#[test]
fn xpub_default_p2wpkh_count10() {
    let o = mn(&[
        "addresses",
        "--from",
        &format!("xpub={V2_84_MAIN}"),
        "--address-type",
        "p2wpkh",
    ]);
    assert_eq!(code(&o), 0, "{}", stderr(&o));
    assert_eq!(
        stdout(&o).lines().filter(|l| l.contains("bc1q")).count(),
        10
    );
}

#[test]
fn xpub_first_matches_convert_oracle() {
    let o = mn(&[
        "addresses",
        "--from",
        &format!("xpub={V2_84_MAIN}"),
        "--address-type",
        "p2wpkh",
        "--count",
        "1",
    ]);
    assert!(stdout(&o).contains(V2_M0_0_P2WPKH), "{}", stdout(&o));
}

#[test]
fn all_four_address_types_prefixes() {
    for (ty, prefix) in [
        ("p2wpkh", "bc1q"),
        ("p2tr", "bc1p"),
        ("p2pkh", "1"),
        ("p2sh-p2wpkh", "3"),
    ] {
        let o = mn(&[
            "addresses",
            "--from",
            &format!("xpub={V2_84_MAIN}"),
            "--address-type",
            ty,
            "--count",
            "1",
        ]);
        assert_eq!(code(&o), 0, "{ty}: {}", stderr(&o));
        let last = stdout(&o)
            .lines()
            .next()
            .unwrap()
            .split_whitespace()
            .last()
            .unwrap()
            .to_string();
        assert!(last.starts_with(prefix), "{ty} → {last}");
    }
}

#[test]
fn phrase_source_matches_account_oracle() {
    let o = mn(&[
        "addresses",
        "--from",
        &format!("phrase={ABANDON}"),
        "--address-type",
        "p2wpkh",
        "--count",
        "1",
    ]);
    assert_eq!(code(&o), 0, "{}", stderr(&o));
    assert!(stdout(&o).contains(ABANDON_ACCT0_M0_0), "{}", stdout(&o));
}

#[test]
fn entropy_and_seedqr_parity_with_phrase() {
    // all-zeros 16-byte entropy == the abandon phrase.
    let ent = mn(&[
        "addresses",
        "--from",
        "entropy=00000000000000000000000000000000",
        "--address-type",
        "p2wpkh",
        "--count",
        "1",
    ]);
    assert!(
        stdout(&ent).contains(ABANDON_ACCT0_M0_0),
        "entropy: {}",
        stdout(&ent)
    );
    // SeedQR digit-string for the abandon phrase (12 words × 4 digits, all word-index 0 except last=0003).
    let seedqr = "000000000000000000000000000000000000000000000003";
    let sq = mn(&[
        "addresses",
        "--from",
        &format!("seedqr={seedqr}"),
        "--address-type",
        "p2wpkh",
        "--count",
        "1",
    ]);
    assert!(
        stdout(&sq).contains(ABANDON_ACCT0_M0_0),
        "seedqr: {}",
        stdout(&sq)
    );
}

#[test]
fn account_index_changes_addresses() {
    let o = mn(&[
        "addresses",
        "--from",
        &format!("phrase={ABANDON}"),
        "--address-type",
        "p2wpkh",
        "--account",
        "1",
        "--count",
        "1",
    ]);
    assert!(stdout(&o).contains(ABANDON_ACCT1_M0_0), "{}", stdout(&o));
}

#[test]
fn range_and_ceiling_reject() {
    let ok = mn(&[
        "addresses",
        "--from",
        &format!("xpub={V2_84_MAIN}"),
        "--address-type",
        "p2wpkh",
        "--range",
        "2,4",
    ]);
    assert_eq!(
        stdout(&ok).lines().filter(|l| l.contains("bc1q")).count(),
        3
    );
    assert_eq!(
        code(&mn(&[
            "addresses",
            "--from",
            &format!("xpub={V2_84_MAIN}"),
            "--address-type",
            "p2wpkh",
            "--range",
            "5,2"
        ])),
        1
    );
    assert_eq!(
        code(&mn(&[
            "addresses",
            "--from",
            &format!("xpub={V2_84_MAIN}"),
            "--address-type",
            "p2wpkh",
            "--count",
            "2147483649"
        ])),
        1
    );
    assert_eq!(
        code(&mn(&[
            "addresses",
            "--from",
            &format!("xpub={V2_84_MAIN}"),
            "--address-type",
            "p2wpkh",
            "--range",
            "0,2147483648"
        ])),
        1
    );
}

#[test]
fn chain_both_grouped_receive_then_change() {
    let o = mn(&[
        "addresses",
        "--from",
        &format!("xpub={V2_84_MAIN}"),
        "--address-type",
        "p2wpkh",
        "--count",
        "1",
        "--chain",
        "both",
    ]);
    let out = stdout(&o);
    assert!(
        out.find("receive").unwrap() < out.find("change").unwrap(),
        "{out}"
    );
}

#[test]
fn network_infer_override_and_mismatch() {
    assert!(stdout(&mn(&[
        "addresses",
        "--from",
        &format!("xpub={V15_84_TEST}"),
        "--address-type",
        "p2wpkh",
        "--count",
        "1"
    ]))
    .contains("tb1q"));
    assert!(stdout(&mn(&[
        "addresses",
        "--from",
        &format!("xpub={V15_84_TEST}"),
        "--address-type",
        "p2wpkh",
        "--count",
        "1",
        "--network",
        "regtest"
    ]))
    .contains("bcrt1q"));
    assert_eq!(
        code(&mn(&[
            "addresses",
            "--from",
            &format!("xpub={V15_84_TEST}"),
            "--address-type",
            "p2wpkh",
            "--network",
            "mainnet"
        ])),
        1
    );
}

#[test]
fn xpub_rejects_account_and_passphrase() {
    assert_eq!(
        code(&mn(&[
            "addresses",
            "--from",
            &format!("xpub={V2_84_MAIN}"),
            "--address-type",
            "p2wpkh",
            "--account",
            "5"
        ])),
        1
    );
    assert_eq!(
        code(&mn(&[
            "addresses",
            "--from",
            &format!("xpub={V2_84_MAIN}"),
            "--address-type",
            "p2wpkh",
            "--passphrase",
            "x"
        ])),
        1
    );
}

#[test]
fn json_shape() {
    let o = mn(&[
        "addresses",
        "--from",
        &format!("xpub={V2_84_MAIN}"),
        "--address-type",
        "p2wpkh",
        "--count",
        "2",
        "--json",
    ]);
    let v: serde_json::Value = serde_json::from_str(&stdout(&o)).unwrap();
    assert_eq!(v["schema_version"], "1");
    assert_eq!(v["source"], "xpub");
    assert_eq!(v["address_type"], "p2wpkh");
    assert_eq!(v["network"], "mainnet");
    assert!(v.get("account").is_none(), "xpub source omits account");
    let addrs = v["addresses"].as_array().unwrap();
    assert_eq!(addrs.len(), 2);
    assert_eq!(addrs[0]["chain"], 0);
    assert_eq!(addrs[0]["index"], 0);
    assert_eq!(addrs[0]["address"], V2_M0_0_P2WPKH);
}

#[test]
fn json_seed_source_has_account() {
    let o = mn(&[
        "addresses",
        "--from",
        &format!("phrase={ABANDON}"),
        "--address-type",
        "p2wpkh",
        "--count",
        "1",
        "--account",
        "2",
        "--json",
    ]);
    let v: serde_json::Value = serde_json::from_str(&stdout(&o)).unwrap();
    assert_eq!(v["source"], "phrase");
    assert_eq!(v["account"], 2);
}

#[test]
fn french_phrase_no_non_english_advisory() {
    // Addresses are derived (language baked in) → the v0.37.11 advisory must NOT fire.
    let o = mn(&[
        "addresses",
        "--from",
        &format!("phrase={FRENCH_12}"),
        "--language",
        "french",
        "--address-type",
        "p2wpkh",
        "--count",
        "1",
    ]);
    assert_eq!(code(&o), 0, "{}", stderr(&o));
    assert!(
        !stderr(&o).contains("BIP-39 seed"),
        "no non-English advisory: {}",
        stderr(&o)
    );
}

#[test]
fn unsupported_from_rejected() {
    assert_ne!(
        code(&mn(&[
            "addresses",
            "--from",
            "mk1=mk1qxyz",
            "--address-type",
            "p2wpkh"
        ])),
        0
    );
}

#[test]
fn env_and_stdin_channels() {
    // @env: resolution.
    let env_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "addresses",
            "--from",
            "phrase=@env:TEST_PHRASE",
            "--address-type",
            "p2wpkh",
            "--count",
            "1",
        ])
        .env("TEST_PHRASE", ABANDON)
        .output()
        .unwrap();
    assert!(
        stdout(&env_out).contains(ABANDON_ACCT0_M0_0),
        "{}",
        stdout(&env_out)
    );
    // stdin `-`.
    let stdin_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "addresses",
            "--from",
            "phrase=-",
            "--address-type",
            "p2wpkh",
            "--count",
            "1",
        ])
        .write_stdin(ABANDON)
        .output()
        .unwrap();
    assert!(
        stdout(&stdin_out).contains(ABANDON_ACCT0_M0_0),
        "{}",
        stdout(&stdin_out)
    );
    // single-stdin guard.
    let dual = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "addresses",
            "--from",
            "phrase=-",
            "--passphrase-stdin",
            "--address-type",
            "p2wpkh",
        ])
        .write_stdin(ABANDON)
        .output()
        .unwrap();
    assert_eq!(dual.status.code().unwrap(), 1);
}
