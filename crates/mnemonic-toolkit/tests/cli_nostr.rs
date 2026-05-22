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
