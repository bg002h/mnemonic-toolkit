//! v0.36.0 — `mnemonic verify-message` integration tests. Crypto oracle is the
//! unit suite in `src/verify_message.rs` (BIP-322 mediawiki vectors + a
//! self-generated legacy P2PKH vector). These exercise the CLI surface: the
//! exit-code convention, --format override, --json, and --message-* sources.

use assert_cmd::Command;
use predicates::prelude::*;

// BIP-322 simple vectors (the bip322 crate's SEGWIT_ADDRESS test = BIP-322 mediawiki).
const SEGWIT_ADDR: &str = "bc1q9vza2e8x573nczrlzms0wvx3gsqjx7vavgkx0l";
const SIG_HELLO: &str = "AkcwRAIgZRfIY3p7/DoVTty6YZbWS71bc5Vct9p9Fia83eRmw2QCICK/ENGfwLtptFluMGs2KsqoNSk89pO7F29zJLUx9a/sASECx/EgAxlkQpQ9hYjgGu6EBCPMVPwVIVJqO4XCsMvViHI=";

#[test]
fn bip322_valid_auto_exit_zero() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "verify-message",
            "--address",
            SEGWIT_ADDR,
            "--message",
            "Hello World",
            "--signature",
            SIG_HELLO,
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("VALID").and(predicate::str::contains("bip322")));
}

#[test]
fn bip322_explicit_format() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "verify-message",
            "--address",
            SEGWIT_ADDR,
            "--message",
            "Hello World",
            "--signature",
            SIG_HELLO,
            "--format",
            "bip322",
        ])
        .assert()
        .success();
}

#[test]
fn wrong_message_invalid_exit_one_clean_stdout() {
    // Cleanly-decoded-but-does-not-verify → exit 1, structured result on stdout (no stderr error).
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "verify-message",
            "--address",
            SEGWIT_ADDR,
            "--message",
            "Goodbye World",
            "--signature",
            SIG_HELLO,
        ])
        .assert()
        .code(1)
        .stdout(predicate::str::contains("INVALID"));
}

#[test]
fn legacy_format_on_segwit_errors() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "verify-message",
            "--address",
            SEGWIT_ADDR,
            "--message",
            "Hello World",
            "--signature",
            SIG_HELLO,
            "--format",
            "legacy",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("P2PKH-only"));
}

#[test]
fn malformed_address_errors() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "verify-message",
            "--address",
            "not-an-address",
            "--message",
            "x",
            "--signature",
            SIG_HELLO,
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("verify-message"));
}

#[test]
fn json_shape() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "verify-message",
            "--address",
            SEGWIT_ADDR,
            "--message",
            "Hello World",
            "--signature",
            SIG_HELLO,
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["valid"], true);
    assert_eq!(v["format_matched"], "bip322");
    assert_eq!(v["format_requested"], "auto");
}

#[test]
fn message_via_stdin() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "verify-message",
            "--address",
            SEGWIT_ADDR,
            "--message-stdin",
            "--signature",
            SIG_HELLO,
        ])
        .write_stdin("Hello World\n") // single trailing newline stripped
        .assert()
        .success()
        .stdout(predicate::str::contains("VALID"));
}

#[test]
fn message_source_mutually_exclusive() {
    // ArgGroup: --message + --message-stdin together is a clap error.
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "verify-message",
            "--address",
            SEGWIT_ADDR,
            "--message",
            "x",
            "--message-stdin",
            "--signature",
            SIG_HELLO,
        ])
        .assert()
        .failure();
}

// ── Key-binding forgery, end-to-end (responsible disclosure, 2026-08-02) ─────
// The pinned `bip322 0.0.10` does not bind the witness public key to the
// challenged address for P2WPKH / P2SH-P2WPKH, so an unrelated key could
// produce a proof the toolkit reported as VALID. These assert the CLI
// contract scripted consumers actually rely on: INVALID on stdout, exit 1,
// and `"valid": false` in the --json envelope.

/// Sign `address` with a key that does NOT control it — the forgery primitive.
/// `sign_simple_encoded` performs no key↔address check either, so this is all
/// an attacker ever had to do.
fn forge_for(address: &str) -> String {
    use bitcoin::secp256k1::SecretKey;
    use bitcoin::{Network, PrivateKey};
    let sk = SecretKey::from_slice(&[0x42u8; 32]).unwrap();
    let wif = PrivateKey::new(sk, Network::Bitcoin).to_wif();
    bip322::sign_simple_encoded(address, "Hello World", &wif).unwrap()
}

#[test]
fn p2wpkh_foreign_key_forgery_rejected_by_cli() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "verify-message",
            "--address",
            SEGWIT_ADDR,
            "--message",
            "Hello World",
            "--signature",
            &forge_for(SEGWIT_ADDR),
        ])
        .assert()
        .code(1)
        .stdout(predicate::str::contains("INVALID"));
}

#[test]
fn p2sh_foreign_key_forgery_rejected_by_cli_json() {
    let addr = "3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy";
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "verify-message",
            "--address",
            addr,
            "--message",
            "Hello World",
            "--signature",
            &forge_for(addr),
            "--json",
        ])
        .assert()
        .code(1)
        .stdout(predicate::str::contains("\"valid\": false"));
}
