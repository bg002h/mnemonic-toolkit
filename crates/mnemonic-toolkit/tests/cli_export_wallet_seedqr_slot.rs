//! v0.31.3 — `mnemonic export-wallet --slot @N.seedqr=` boundary test.
//!
//! Per `wallet_export/mod.rs::validate_watch_only` invariant
//! (SPEC §3), `export-wallet` is watch-only-by-definition: ALL
//! secret-bearing slot subkeys (phrase, seedqr, entropy, xprv, wif)
//! are REFUSED at the pre-resolve fast path. This is the
//! intentional boundary; the new Seedqr subkey participates in the
//! refusal correctly.

use assert_cmd::Command;

const DIGITS_12: &str = "000000000000000000000000000000000000000000000003";

#[test]
fn export_wallet_seedqr_slot_refused_watch_only_invariant() {
    let assertion = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--format",
            "bitcoin-core",
            "--template",
            "bip84",
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.seedqr={DIGITS_12}"),
        ])
        .assert()
        .failure();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("watch-only by definition")
            && stderr.contains("xpub/fingerprint/path"),
        "expected SPEC §3 watch-only-invariant refusal text; got: {stderr}"
    );
}
