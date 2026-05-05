//! `--privacy-preserving` axis test (Phase E.2).
//!
//! Verifies `--privacy-preserving` cells match pinned fixtures across all 4 networks
//! against `tests/vectors/v0_2/wsh-sortedmulti-<network>-0-true-false.txt`.

use assert_cmd::Command;

const TREZOR_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

#[test]
fn bundle_privacy_preserving_4_cells() {
    for &n in &["mainnet", "testnet", "signet", "regtest"] {
        let expected = std::fs::read_to_string(format!(
            "tests/vectors/v0_2/wsh-sortedmulti-{}-0-true-false.txt",
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
                "--privacy-preserving",
                "--no-engraving-card",
            ])
            .assert()
            .success();
        let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
        assert_eq!(stdout, expected, "privacy-preserving mismatch for {}", n);
    }
}
