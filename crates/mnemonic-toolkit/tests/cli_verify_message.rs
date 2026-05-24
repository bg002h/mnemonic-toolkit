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
        .args(["verify-message", "--address", SEGWIT_ADDR, "--message", "Hello World", "--signature", SIG_HELLO])
        .assert()
        .success()
        .stdout(predicate::str::contains("VALID").and(predicate::str::contains("bip322")));
}

#[test]
fn bip322_explicit_format() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["verify-message", "--address", SEGWIT_ADDR, "--message", "Hello World", "--signature", SIG_HELLO, "--format", "bip322"])
        .assert()
        .success();
}

#[test]
fn wrong_message_invalid_exit_one_clean_stdout() {
    // Cleanly-decoded-but-does-not-verify → exit 1, structured result on stdout (no stderr error).
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["verify-message", "--address", SEGWIT_ADDR, "--message", "Goodbye World", "--signature", SIG_HELLO])
        .assert()
        .code(1)
        .stdout(predicate::str::contains("INVALID"));
}

#[test]
fn legacy_format_on_segwit_errors() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["verify-message", "--address", SEGWIT_ADDR, "--message", "Hello World", "--signature", SIG_HELLO, "--format", "legacy"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("P2PKH-only"));
}

#[test]
fn malformed_address_errors() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["verify-message", "--address", "not-an-address", "--message", "x", "--signature", SIG_HELLO])
        .assert()
        .failure()
        .stderr(predicate::str::contains("verify-message"));
}

#[test]
fn json_shape() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["verify-message", "--address", SEGWIT_ADDR, "--message", "Hello World", "--signature", SIG_HELLO, "--json"])
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
        .args(["verify-message", "--address", SEGWIT_ADDR, "--message-stdin", "--signature", SIG_HELLO])
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
        .args(["verify-message", "--address", SEGWIT_ADDR, "--message", "x", "--message-stdin", "--signature", SIG_HELLO])
        .assert()
        .failure();
}
