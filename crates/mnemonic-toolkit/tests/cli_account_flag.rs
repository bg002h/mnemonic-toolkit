//! `--account` axis test (Phase E.2).
//!
//! Verifies `--account 5` produces matching pinned fixtures across all 4 networks
//! against `tests/vectors/v0_2/wsh-sortedmulti-<network>-5-false-false.txt`.

use assert_cmd::Command;

const TREZOR_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

// deprecated v0.2 pattern; remove after v0.4 release. Uses --cosigner-count > 1
// (BIP-388 violating self-multisig). Replacement v0.4 fixtures with multi-source
// secrets land in Phase G.7.
#[ignore = "deprecated v0.2 pattern; remove after v0.4 release"]
#[test]
fn bundle_account_5_4_cells_byte_exact() {
    for &n in &["mainnet", "testnet", "signet", "regtest"] {
        let expected = std::fs::read_to_string(format!(
            "tests/vectors/v0_2/wsh-sortedmulti-{}-5-false-false.txt",
            n
        ))
        .expect("fixture");
        let out = Command::cargo_bin("mnemonic")
            .unwrap()
            .args([
                "bundle",
                "--phrase",
                TREZOR_24,
                "--network",
                n,
                "--template",
                "wsh-sortedmulti",
                "--threshold",
                "2",
                "--cosigner-count",
                "3",
                "--account",
                "5",
                "--no-engraving-card",
            ])
            .assert()
            .success();
        let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
        assert_eq!(stdout, expected, "account-5 mismatch for {}", n);
    }
}
