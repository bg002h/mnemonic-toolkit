//! 16-cell parametric integration test (Task 5.2).
//!
//! Compares stdout byte-exactly against pinned fixtures in
//! `tests/vectors/v0_1/{template}-{network}.txt`. Byte-determinism is
//! guaranteed by `synthesize::derive_mk1_chunk_set_id` deriving the mk1
//! `chunk_set_id` from the policy_id_stub (mirrors md-codec's deterministic
//! CSI derivation).

use assert_cmd::Command;

const TREZOR_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

#[test]
fn bundle_full_16_cells_byte_exact_against_pinned_vectors() {
    for &t in &["bip44", "bip49", "bip84", "bip86"] {
        for &n in &["mainnet", "testnet", "signet", "regtest"] {
            let expected = std::fs::read_to_string(format!("tests/vectors/v0_1/{}-{}.txt", t, n))
                .expect("fixture exists");
            let out = Command::cargo_bin("mnemonic")
                .unwrap()
                .args([
                    "bundle",
                    "--slot",
                    &format!("@0.phrase={TREZOR_24}"),
                    "--network",
                    n,
                    "--template",
                    t,
                    "--no-engraving-card",
                ])
                .assert()
                .success();
            let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
            assert_eq!(stdout, expected, "byte-exact mismatch for {}-{}", t, n);
        }
    }
}
