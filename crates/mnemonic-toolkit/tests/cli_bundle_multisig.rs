//! Phase C smoke tests for multisig bundle invocations.
//!
//! Self-multisig full + watch-only multisig with distinct cosigner xpubs.

use assert_cmd::Command;

// Deleted v0.4.2 cleanup: self_multisig_full_emits_warning_and_n_card_sets
// exercised the v0.2 self-multisig pattern which was hard-rejected by BIP-388
// in v0.4.0 and has no migration path.

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
    let mut slot_args: Vec<String> = Vec::new();
    for (idx, p) in phrases.iter().enumerate() {
        let m = Mnemonic::parse_in(bip39::Language::English, *p).unwrap();
        let seed = m.to_seed("");
        let master = Xpriv::new_master(bitcoin::NetworkKind::Main, &seed).unwrap();
        let fp = master.fingerprint(&secp);
        let path = DerivationPath::from_str(path_str).unwrap();
        let xpriv = master.derive_priv(&secp, &path).unwrap();
        let xpub = Xpub::from_priv(&secp, &xpriv);
        slot_args.push(format!("@{idx}.xpub={xpub}"));
        slot_args.push(format!("@{idx}.fingerprint={}", fp.to_string().to_lowercase()));
        slot_args.push(format!("@{idx}.path={path_str}"));
    }

    let mut cmd_args: Vec<String> = vec![
        "bundle".into(),
        "--network".into(),
        "mainnet".into(),
        "--template".into(),
        "wsh-sortedmulti".into(),
        "--threshold".into(),
        "2".into(),
        "--no-engraving-card".into(),
    ];
    for s in &slot_args {
        cmd_args.push("--slot".into());
        cmd_args.push(s.clone());
    }

    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&cmd_args)
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("# mk1[0]"));
    assert!(stdout.contains("# mk1[1]"));
    assert!(stdout.contains("# md1 (multisig wallet policy)"));
    // ms1 omitted in watch-only.
    assert!(stdout.contains("# ms1 (omitted"));
}
