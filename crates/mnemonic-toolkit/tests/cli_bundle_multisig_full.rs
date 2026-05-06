//! 24-cell parametric multisig integration test (Phase E.2).
//!
//! Compares stdout byte-exactly against pinned fixtures in
//! `tests/vectors/v0_2/<template>-<network>-0-false-false.txt` for
//! 6 multisig templates × 4 networks (2-of-3, account=0, privacy off).

use assert_cmd::Command;

const TREZOR_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

// deprecated v0.2 pattern; remove after v0.4 release. v0.2 multisig-full
// fixtures violate BIP-388 distinctness (all N cosigner xpubs derived from
// one seed at one path). Per SPEC §10 v0.4 fixture exclusions.
#[ignore = "deprecated v0.2 pattern; remove after v0.4 release"]
#[test]
fn bundle_multisig_full_24_cells_byte_exact() {
    for &t in &[
        "wsh-multi",
        "wsh-sortedmulti",
        "sh-wsh-multi",
        "sh-wsh-sortedmulti",
        "tr-multi-a",
        "tr-sortedmulti-a",
    ] {
        for &n in &["mainnet", "testnet", "signet", "regtest"] {
            let expected = std::fs::read_to_string(format!(
                "tests/vectors/v0_2/{}-{}-0-false-false.txt",
                t, n
            ))
            .expect("fixture exists");
            let out = Command::cargo_bin("mnemonic")
                .unwrap()
                .args([
                    "bundle",
                    "--phrase",
                    TREZOR_24,
                    "--network",
                    n,
                    "--template",
                    t,
                    "--threshold",
                    "2",
                    "--cosigner-count",
                    "3",
                    "--no-engraving-card",
                ])
                .assert()
                .success();
            let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
            assert_eq!(stdout, expected, "byte-exact mismatch for {}-{}", t, n);
        }
    }
}
