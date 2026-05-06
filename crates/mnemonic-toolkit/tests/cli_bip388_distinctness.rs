//! v0.4 SPEC §4.11 / §6.6 row 13 BIP-388 distinct-key conformance — CLI
//! integration test for byte-exact stderr at bundle-time.
//!
//! Verify-bundle-time symmetric enforcement (SPEC §4.11.c → exit 4 +
//! `error: bundle violates BIP-388 distinct-key rule; regenerate with distinct keys`)
//! is unit-tested via `error::tests::bip388_variants_exit_code_kind_message`.
//! End-to-end CLI integration for verify-bundle lands in Phase G.4 alongside
//! the verify-bundle forensic-diagnostics refactor where colliding-key fixtures
//! materialize naturally.

use assert_cmd::Command;

const TREZOR_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

#[test]
fn bundle_multisig_full_legacy_emits_row13_byte_exact_stderr() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--phrase",
            TREZOR_24,
            "--network",
            "mainnet",
            "--template",
            "wsh-sortedmulti",
            "--threshold",
            "2",
            "--cosigner-count",
            "3",
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
