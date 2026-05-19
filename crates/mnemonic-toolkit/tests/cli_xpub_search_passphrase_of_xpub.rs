//! `mnemonic xpub-search passphrase-of-xpub` integration tests (v0.26.0 P4).
//!
//! Per `design/PLAN_v0_26_0_xpub_search.md` §10 C4 (commit-numbered phase P4).
//! Realizes plan §6 (P4 SPEC) — P4 is P1 + a fixed mandatory passphrase:
//! re-derive master via `derive_master_seed(mnemonic, passphrase)`, then
//! invoke the same `match_xpub_against_paths` primitive over the standard
//! BIP-44/49/84/86 single-sig + BIP-48 multisig templates × account range +
//! `--add-path` extensions.
//!
//! Semantic difference from P1: P1 asks "what path produced this xpub?";
//! P4 asks "does this specific passphrase produce this xpub (at some
//! standard path)?". Clap enforces the passphrase group required.
//!
//! Test design (TDD per plan §10 C4 — ~10 cells; load-bearing 1–7).

use assert_cmd::Command;
use bip39::Mnemonic;
use bitcoin::bip32::{DerivationPath, Xpriv, Xpub};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::NetworkKind;
use predicates::prelude::*;
use std::str::FromStr;

/// The canonical BIP-39 12-word test vector. Entropy is 16 zero bytes.
const PHRASE: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

/// Compute the xpub at `path` for the test phrase + passphrase.
fn xpub_at(path: &str, passphrase: &str) -> Xpub {
    let mnemonic = Mnemonic::parse_in(bip39::Language::English, PHRASE).unwrap();
    let seed = mnemonic.to_seed(passphrase);
    let secp = Secp256k1::new();
    let master = Xpriv::new_master(NetworkKind::Main, &seed).unwrap();
    let dp = DerivationPath::from_str(path).unwrap();
    let xpriv = master.derive_priv(&secp, &dp).unwrap();
    Xpub::from_priv(&secp, &xpriv)
}

/// The stderr advisory P4 emits on every invocation per plan §6.4.
const ADVISORY_FRAGMENT: &str = "passphrase verification searches the standard";

// ---------------------------------------------------------------------------
// Cell 1 — happy path: correct passphrase ("satoshi") at BIP-84 m/84'/0'/0'.
// ---------------------------------------------------------------------------
#[test]
fn passphrase_of_xpub_correct_passphrase_matches_bip84() {
    let target = xpub_at("m/84'/0'/0'", "satoshi").to_string();
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "passphrase-of-xpub",
            "--phrase-stdin",
            "--passphrase",
            "satoshi",
            "--target-xpub",
            &target,
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(0);
}

// ---------------------------------------------------------------------------
// Cell 2 — wrong passphrase → no-match → exit 4.
// ---------------------------------------------------------------------------
#[test]
fn passphrase_of_xpub_wrong_passphrase_no_match_exit_4() {
    // Target derived with "satoshi" passphrase; query with "nakamoto" → miss.
    let target = xpub_at("m/84'/0'/0'", "satoshi").to_string();
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "passphrase-of-xpub",
            "--phrase-stdin",
            "--passphrase",
            "nakamoto",
            "--target-xpub",
            &target,
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(4);
}

// ---------------------------------------------------------------------------
// Cell 3 — mandatory passphrase: omitting both --passphrase and
//   --passphrase-stdin → exit 64 (clap arg-parse error).
// ---------------------------------------------------------------------------
#[test]
fn passphrase_of_xpub_missing_passphrase_clap_error_exit_64() {
    let target = xpub_at("m/84'/0'/0'", "satoshi").to_string();
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "passphrase-of-xpub",
            "--phrase-stdin",
            "--target-xpub",
            &target,
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(64);
}

// ---------------------------------------------------------------------------
// Cell 4 — mutually exclusive: --passphrase AND --passphrase-stdin → clap
//   mutex error exit 64.
// ---------------------------------------------------------------------------
#[test]
fn passphrase_of_xpub_mutex_passphrase_and_passphrase_stdin_clap_error() {
    let target = xpub_at("m/84'/0'/0'", "satoshi").to_string();
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "passphrase-of-xpub",
            "--phrase",
            PHRASE,
            "--passphrase",
            "satoshi",
            "--passphrase-stdin",
            "--target-xpub",
            &target,
        ])
        .assert()
        .code(64);
}

// ---------------------------------------------------------------------------
// Cell 5 — stderr advisory always emitted, both on match AND no-match.
// ---------------------------------------------------------------------------
#[test]
fn passphrase_of_xpub_advisory_emitted_on_match_and_no_match() {
    let target = xpub_at("m/84'/0'/0'", "satoshi").to_string();

    // Match case.
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "passphrase-of-xpub",
            "--phrase-stdin",
            "--passphrase",
            "satoshi",
            "--target-xpub",
            &target,
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(0)
        .stderr(predicate::str::contains(ADVISORY_FRAGMENT));

    // No-match case.
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "passphrase-of-xpub",
            "--phrase-stdin",
            "--passphrase",
            "wrong",
            "--target-xpub",
            &target,
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(4)
        .stderr(predicate::str::contains(ADVISORY_FRAGMENT));
}

// ---------------------------------------------------------------------------
// Cell 6 — passphrase via --passphrase-stdin: phrase via --phrase, passphrase
//   via stdin.
// ---------------------------------------------------------------------------
#[test]
fn passphrase_of_xpub_passphrase_stdin_happy_path() {
    let target = xpub_at("m/84'/0'/0'", "satoshi").to_string();
    // --phrase inline (argv-leaks but the test isn't checking that), pass
    // the passphrase via stdin. The handler reads the entire stdin for the
    // passphrase (matches `path_of_xpub.rs:181` pattern). Newline-stripped.
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "passphrase-of-xpub",
            "--phrase",
            PHRASE,
            "--passphrase-stdin",
            "--target-xpub",
            &target,
        ])
        .write_stdin("satoshi\n")
        .assert()
        .code(0);
}

// ---------------------------------------------------------------------------
// Cell 7 — JSON envelope shape (match): byte-pin `{"schema_version":"1",
//   "mode":"passphrase-of-xpub","result":"match",...}`.
// ---------------------------------------------------------------------------
#[test]
fn passphrase_of_xpub_json_envelope_shape() {
    let target = xpub_at("m/84'/0'/0'", "satoshi").to_string();
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "passphrase-of-xpub",
            "--phrase-stdin",
            "--passphrase",
            "satoshi",
            "--target-xpub",
            &target,
            "--json",
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();
    let body = String::from_utf8(out).unwrap();
    let v: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(v["schema_version"], "1");
    assert_eq!(v["mode"], "passphrase-of-xpub");
    assert_eq!(v["result"], "match");
    assert_eq!(v["path"], "m/84'/0'/0'");
    assert_eq!(v["template"], "bip84");
    assert_eq!(v["account"], 0);
    assert!(v["target_xpub_canonical"].as_str().unwrap().starts_with("xpub"));
    // Canonical xpub input → variant null.
    assert!(v["target_xpub_variant"].is_null());
    assert!(v["searched_count"].is_number());
}

// ---------------------------------------------------------------------------
// Cell 8 — --add-path extends the candidate set (matches BIP-87 path).
// ---------------------------------------------------------------------------
#[test]
fn passphrase_of_xpub_add_path_extends_search() {
    let target = xpub_at("m/87'/0'/2'", "satoshi").to_string();
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "passphrase-of-xpub",
            "--phrase-stdin",
            "--passphrase",
            "satoshi",
            "--target-xpub",
            &target,
            "--add-path",
            "m/87'/0'/account'",
            "--json",
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_str(&String::from_utf8(out).unwrap()).unwrap();
    assert_eq!(v["path"], "m/87'/0'/2'");
    assert_eq!(v["account"], 2);
    assert_eq!(v["template"], "m/87'/0'/account'");
}

// ---------------------------------------------------------------------------
// Cell 9 — mk1-card target intake: encode the xpub-at-path as an mk1 card
//   and supply it via --target-xpub. Mirrors the construction pattern in
//   `tests/cli_verify_bundle_watch_only.rs:264-270` (mk_codec::KeyCard::new
//   + mk_codec::encode).
// ---------------------------------------------------------------------------
#[test]
fn passphrase_of_xpub_mk1_target_intake() {
    let xpub = xpub_at("m/84'/0'/0'", "satoshi");
    // BIP-84 mainnet account-0 origin path.
    let origin_path = DerivationPath::from_str("m/84'/0'/0'").unwrap();
    // Origin fingerprint: None is fine — toolkit's mk1 intake walks the
    // decoded xpub bytes directly, not the fingerprint. mk-codec encoder
    // requires non-empty `policy_id_stubs`; supply a single dummy stub.
    let card = mk_codec::KeyCard::new(vec![[0u8; 4]], None, origin_path, xpub);
    let chunks = mk_codec::encode(&card).expect("mk_codec::encode");
    let mk1 = chunks.join(" ");

    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "passphrase-of-xpub",
            "--phrase-stdin",
            "--passphrase",
            "satoshi",
            "--target-xpub",
            &mk1,
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(0);
}

// ---------------------------------------------------------------------------
// Cell 10 — argv-leakage advisory for inline `--passphrase`.
// ---------------------------------------------------------------------------
#[test]
fn passphrase_of_xpub_argv_leak_advisory_on_inline_passphrase() {
    let target = xpub_at("m/84'/0'/0'", "satoshi").to_string();
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "passphrase-of-xpub",
            "--phrase-stdin",
            "--passphrase",
            "satoshi",
            "--target-xpub",
            &target,
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(0)
        .stderr(predicate::str::contains(
            "secret material on argv (--passphrase)",
        ));
}
