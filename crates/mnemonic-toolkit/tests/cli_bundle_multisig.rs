//! Phase C smoke tests for multisig bundle invocations.
//!
//! Self-multisig full + watch-only multisig with distinct cosigner xpubs.

use assert_cmd::Command;

const TREZOR_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

// deprecated v0.2 pattern; remove after v0.4 release. Per SPEC §4.11.b the
// `--cosigner-count > 1` self-multisig path now hard-rejects with exit 2 +
// `BIP-388 distinct-key violation` stderr.
#[ignore = "deprecated v0.2 pattern; remove after v0.4 release"]
#[test]
fn self_multisig_full_emits_warning_and_n_card_sets() {
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
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();

    // SELF-MULTISIG WARNING emitted (SPEC §4.1 byte-exact).
    assert!(
        stderr.contains("warning: full-mode multisig (--cosigner-count > 1)"),
        "missing self-multisig warning line 1; got: {}",
        stderr
    );
    assert!(
        stderr.contains("byte-identical interchangeable copies"),
        "missing self-multisig warning line 2"
    );

    // Three mk1[i] sections.
    assert!(stdout.contains("# mk1[0] (cosigner 0 xpub + origin)"));
    assert!(stdout.contains("# mk1[1] (cosigner 1 xpub + origin)"));
    assert!(stdout.contains("# mk1[2] (cosigner 2 xpub + origin)"));
    assert!(stdout.contains("# md1 (multisig wallet policy)"));
}

#[test]
fn watch_only_multisig_distinct_cosigners_emits_distinct_cards() {
    use bip39::Mnemonic;
    use bitcoin::bip32::{DerivationPath, Xpriv, Xpub};
    use bitcoin::secp256k1::Secp256k1;
    use std::str::FromStr;

    let phrases = [
        "legal winner thank year wave sausage worth useful legal winner thank yellow",
        "letter advice cage absurd amount doctor acoustic avoid letter advice cage above",
    ];
    let secp = Secp256k1::new();
    let path_str = "m/87'/0'/0'";
    let mut cosigner_args: Vec<String> = Vec::new();
    for p in &phrases {
        let m = Mnemonic::parse_in(bip39::Language::English, *p).unwrap();
        let seed = m.to_seed("");
        let master = Xpriv::new_master(bitcoin::NetworkKind::Main, &seed).unwrap();
        let fp = master.fingerprint(&secp);
        let path = DerivationPath::from_str(path_str).unwrap();
        let xpriv = master.derive_priv(&secp, &path).unwrap();
        let xpub = Xpub::from_priv(&secp, &xpriv);
        cosigner_args.push(format!(
            "{}:{}:{}",
            xpub,
            fp.to_string().to_lowercase(),
            path_str
        ));
    }

    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--network",
            "mainnet",
            "--template",
            "wsh-sortedmulti",
            "--threshold",
            "2",
            "--cosigner",
            &cosigner_args[0],
            "--cosigner",
            &cosigner_args[1],
            "--no-engraving-card",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("# mk1[0]"));
    assert!(stdout.contains("# mk1[1]"));
    assert!(stdout.contains("# md1 (multisig wallet policy)"));
    // ms1 omitted in watch-only.
    assert!(stdout.contains("# ms1 (omitted"));
}
