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
