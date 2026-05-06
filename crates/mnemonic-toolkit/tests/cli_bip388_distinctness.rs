//! v0.4 SPEC §4.11 / §6.6 row 1 + row 13 BIP-388 + removed-subcommand —
//! CLI integration tests for byte-exact stderr at bundle-time.
//!
//! Verify-bundle-time symmetric enforcement (SPEC §4.11.c → exit 4 +
//! `error: bundle violates BIP-388 distinct-key rule; regenerate with distinct keys`)
//! is unit-tested via `error::tests::bip388_variants_exit_code_kind_message`.
//! End-to-end CLI integration for verify-bundle lands in Phase G.4 alongside
//! the verify-bundle forensic-diagnostics refactor where colliding-key fixtures
//! materialize naturally.

use assert_cmd::Command;

const TREZOR_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

// v0.4.2 Phase K-driven BIP-388 distinctness test using --slot directly:
// supply two slots that resolve to the same (xpub, path) → row 13 fires.
#[test]
fn bip388_row13_fires_for_duplicate_slot_phrases() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--template",
            "wsh-sortedmulti",
            "--threshold",
            "2",
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.phrase={}", TREZOR_24),
            "--slot",
            &format!("@1.phrase={}", TREZOR_24),
            "--no-engraving-card",
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert_eq!(
        stderr,
        "error: BIP-388 distinct-key violation: slot @0 and slot @1 resolve to identical (xpub, path)\n",
        "stderr must match SPEC §6.6 row 13 byte-exactly"
    );
}

// v0.5 trap deletion: the v0.4.2 detect_removed_subcommand pre-clap trap was
// removed in v0.5; clap's unknown-arg fallback (exit 64 — toolkit's
// format-violation override of clap's default 2) is the surviving rejection
// path.

#[test]
fn bundle_multisig_full_subtoken_rejected_by_clap_exit_64() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["bundle", "multisig-full"])
        .assert()
        .failure()
        .code(64);
}

#[test]
fn bundle_multisig_watch_only_subtoken_rejected_by_clap_exit_64() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["bundle", "multisig-watch-only"])
        .assert()
        .failure()
        .code(64);
}
