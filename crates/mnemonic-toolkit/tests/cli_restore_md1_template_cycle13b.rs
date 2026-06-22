//! cycle-13b — `restore`/`verify-bundle` keyless MULTISIG-template completion:
//! L8 (non-mainnet all-own coin-type) + L9 (hardened-use-site / taproot-override
//! refusal parity with the non-template `run_multisig` path).
//!
//! Both are funds-safety / availability class:
//!   - L8: a testnet/signet/regtest ALL-OWN multisig-template bundle restored
//!     with no `--cosigner`/`--origin` must derive every own key at the bundle's
//!     coin-type (`network.coin_type()`, =1 non-mainnet) — NOT the hardcoded
//!     mainnet `0'` the `canonical_origin(tree)` fallback returns. The mainnet
//!     all-own case (coin `0'`) is the positive control.
//!   - L9: a keyless multisig TEMPLATE carrying a hardened use-site (`/*h`) must
//!     get the SAME early, precise refusal `run_multisig` applies (watch-only
//!     cannot do hardened public derivation) — not an opaque downstream
//!     NO-MATCH. A legit non-hardened named template stays GREEN (positive
//!     control).

use assert_cmd::Command;
use bip39::Mnemonic;
use bitcoin::bip32::{DerivationPath, Xpriv, Xpub};
use bitcoin::secp256k1::Secp256k1;
use miniscript::{Descriptor, DescriptorPublicKey};
use std::str::FromStr;

const SEED_A: &str = "legal winner thank year wave sausage worth useful legal winner thank yellow";

fn mnemonic() -> Command {
    Command::cargo_bin("mnemonic").expect("mnemonic binary builds")
}

/// Extract md1 string(s) from `bundle` text stdout (lines under `# md1`).
fn md1_lines(stdout: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut in_sec = false;
    for line in stdout.lines() {
        if line.starts_with("# md1") {
            in_sec = true;
            continue;
        }
        if in_sec {
            if line.trim().is_empty() {
                in_sec = false;
                continue;
            }
            out.push(line.trim().to_string());
        }
    }
    out
}

/// Derive an account xpub + master fingerprint at `path_str` from a BIP-39
/// phrase, for `network_kind` (Main vs Test). The xpub version bytes differ
/// between mainnet/testnet but the BIP-32 scalars are identical; the toolkit's
/// own emit/restore use the network's coin-type in the ORIGIN, which is what L8
/// exercises.
fn xpub_at(phrase: &str, path_str: &str, network_kind: bitcoin::NetworkKind) -> (Xpub, String) {
    let secp = Secp256k1::new();
    let m = Mnemonic::parse_in(bip39::Language::English, phrase).unwrap();
    let seed = m.to_seed("");
    let master = Xpriv::new_master(network_kind, &seed).unwrap();
    let fp = master.fingerprint(&secp);
    let path = DerivationPath::from_str(path_str).unwrap();
    let xpriv = master.derive_priv(&secp, &path).unwrap();
    let xpub = Xpub::from_priv(&secp, &xpriv);
    (xpub, fp.to_string().to_lowercase())
}

/// Emit a keyless `wsh(sortedmulti(...))` multisig template md1 for the given
/// cosigners on `network`, each at the BIP-48 canonical origin
/// `m/48'/<coin>'/account'/2'` where coin = network.coin_type().
fn emit_template_md1(network: &str, coin: u32, cosigners: &[(&str, u32)]) -> Vec<String> {
    let nk = if network == "mainnet" {
        bitcoin::NetworkKind::Main
    } else {
        bitcoin::NetworkKind::Test
    };
    let mut args: Vec<String> = vec![
        "bundle".into(),
        "--network".into(),
        network.into(),
        "--template".into(),
        "wsh-sortedmulti".into(),
        "--threshold".into(),
        "2".into(),
        "--md1-form".into(),
        "template".into(),
        "--group-size".into(),
        "0".into(),
        "--no-engraving-card".into(),
    ];
    for (idx, (phrase, account)) in cosigners.iter().enumerate() {
        let path = format!("48'/{coin}'/{account}'/2'");
        let (xpub, fp) = xpub_at(phrase, &path, nk);
        args.push("--slot".into());
        args.push(format!("@{idx}.xpub={xpub}"));
        args.push("--slot".into());
        args.push(format!("@{idx}.fingerprint={fp}"));
        args.push("--slot".into());
        args.push(format!("@{idx}.path={path}"));
    }
    let out = mnemonic().args(&args).assert().success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    md1_lines(&stdout)
}

/// The printed WalletPolicyId (full hex) recorded by the template emit advisory.
fn emit_template_wallet_id(network: &str, coin: u32, cosigners: &[(&str, u32)]) -> String {
    let nk = if network == "mainnet" {
        bitcoin::NetworkKind::Main
    } else {
        bitcoin::NetworkKind::Test
    };
    let mut args: Vec<String> = vec![
        "bundle".into(),
        "--network".into(),
        network.into(),
        "--template".into(),
        "wsh-sortedmulti".into(),
        "--threshold".into(),
        "2".into(),
        "--md1-form".into(),
        "template".into(),
        "--group-size".into(),
        "0".into(),
        "--no-engraving-card".into(),
    ];
    for (idx, (phrase, account)) in cosigners.iter().enumerate() {
        let path = format!("48'/{coin}'/{account}'/2'");
        let (xpub, fp) = xpub_at(phrase, &path, nk);
        args.push("--slot".into());
        args.push(format!("@{idx}.xpub={xpub}"));
        args.push("--slot".into());
        args.push(format!("@{idx}.fingerprint={fp}"));
        args.push("--slot".into());
        args.push(format!("@{idx}.path={path}"));
    }
    let out = mnemonic().args(&args).assert().success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    let line = stderr
        .lines()
        .find(|l| l.contains("wallet-id (hex)"))
        .unwrap_or_else(|| panic!("no wallet-id (hex) line in: {stderr}"));
    line.split(':').next_back().unwrap().trim().to_string()
}

/// INDEPENDENT golden: build the watch-only `wsh(sortedmulti(...))` directly from
/// the cosigner account xpubs at the canonical BIP-48 origin (rust-miniscript),
/// and derive the first `count` receive addresses on `addr_network`.
fn golden_addresses(
    coin: u32,
    cosigners: &[(&str, u32)],
    addr_network: bitcoin::Network,
    network_kind: bitcoin::NetworkKind,
    count: u32,
) -> Vec<String> {
    let mut key_strs: Vec<String> = Vec::new();
    for (phrase, account) in cosigners {
        let path = format!("48'/{coin}'/{account}'/2'");
        let (xpub, fp) = xpub_at(phrase, &path, network_kind);
        let origin = path.replace('\'', "h");
        key_strs.push(format!("[{fp}/{origin}]{xpub}/<0;1>/*"));
    }
    let desc_str = format!("wsh(sortedmulti(2,{}))", key_strs.join(","));
    let desc = Descriptor::<DescriptorPublicKey>::from_str(&desc_str)
        .unwrap_or_else(|e| panic!("golden descriptor parse {desc_str}: {e}"));
    let receive = desc.clone().into_single_descriptors().unwrap().remove(0);
    (0..count)
        .map(|i| {
            receive
                .derive_at_index(i)
                .unwrap()
                .address(addr_network)
                .unwrap()
                .to_string()
        })
        .collect()
}

fn push_md1(args: &mut Vec<String>, md1: &[String]) {
    for c in md1 {
        args.push("--md1".into());
        args.push(c.clone());
    }
}

/// Run `restore` and return the completed receive addresses from the `--json`
/// envelope (asserting success).
fn restore_addresses(args: &[String]) -> Vec<String> {
    let out = mnemonic().args(args).assert().success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let j: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("restore --json parse: {e}; stdout: {stdout}"));
    j["wallets"][0]["first_addresses"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a.as_str().unwrap().to_string())
        .collect()
}

// ===========================================================================
// L8 — non-mainnet ALL-OWN multisig-template completion derives the own keys at
// the bundle's coin-type, not a hardcoded mainnet 0'.
//
// All-own = the SAME seed fills BOTH cosigner slots (accounts 0 and 1), restored
// with `--account 0,1` and NO `--cosigner` → the own-origin fallback closure
// (`canonical_origin(tree)`, hardcoded coin 0') is the ONLY origin source. On a
// testnet bundle the cosigner origins are at `m/48'/1'/…`; if the own keys are
// derived at `m/48'/0'/…` the wallet-id NEVER matches → silent NO-MATCH (RED).
// After the coin-type substitution the own origin is `m/48'/1'/…` → match (GREEN).
// ===========================================================================

#[test]
fn testnet_all_own_multisig_template_completes_to_golden() {
    // 2-of-2 where BOTH keys come from SEED_A (accounts 0 and 1) on TESTNET.
    let cos = &[(SEED_A, 0u32), (SEED_A, 1u32)];
    let coin = 1u32; // testnet coin-type
    let md1 = emit_template_md1("testnet", coin, cos);
    let id = emit_template_wallet_id("testnet", coin, cos);
    let golden = golden_addresses(
        coin,
        cos,
        bitcoin::Network::Testnet,
        bitcoin::NetworkKind::Test,
        2,
    );

    let mut args = vec!["restore".into(), "--network".into(), "testnet".into()];
    push_md1(&mut args, &md1);
    args.extend([
        "--from".into(),
        format!("phrase={SEED_A}"),
        "--account".into(),
        "0,1".into(),
        "--expect-wallet-id".into(),
        id,
        "--count".into(),
        "2".into(),
        "--json".into(),
    ]);
    // No --cosigner: both slots are OWN → the own-origin fallback is the only
    // origin source, and on testnet it must use coin 1' (L8).
    let got = restore_addresses(&args);
    assert_eq!(
        got, golden,
        "testnet all-own multisig template must derive own keys at m/48'/1'/… \
         (coin-type) and match the testnet golden"
    );
}

#[test]
fn signet_all_own_multisig_template_completes_to_golden() {
    // Same as above but signet (also coin-type 1) — confirms the fix is keyed on
    // network.coin_type(), not literally "testnet".
    let cos = &[(SEED_A, 0u32), (SEED_A, 1u32)];
    let coin = 1u32;
    let md1 = emit_template_md1("signet", coin, cos);
    let id = emit_template_wallet_id("signet", coin, cos);
    // signet shares the testnet tb1 HRP / Test network kind.
    let golden = golden_addresses(
        coin,
        cos,
        bitcoin::Network::Signet,
        bitcoin::NetworkKind::Test,
        2,
    );

    let mut args = vec!["restore".into(), "--network".into(), "signet".into()];
    push_md1(&mut args, &md1);
    args.extend([
        "--from".into(),
        format!("phrase={SEED_A}"),
        "--account".into(),
        "0,1".into(),
        "--expect-wallet-id".into(),
        id,
        "--count".into(),
        "2".into(),
        "--json".into(),
    ]);
    let got = restore_addresses(&args);
    assert_eq!(
        got, golden,
        "signet all-own multisig template must derive own keys at m/48'/1'/…"
    );
}

#[test]
fn mainnet_all_own_multisig_template_completes_to_golden_regression() {
    // POSITIVE CONTROL: the mainnet all-own case (coin 0') must STAY green — the
    // coin-type substitution must reduce to the identity on mainnet.
    let cos = &[(SEED_A, 0u32), (SEED_A, 1u32)];
    let coin = 0u32;
    let md1 = emit_template_md1("mainnet", coin, cos);
    let id = emit_template_wallet_id("mainnet", coin, cos);
    let golden = golden_addresses(
        coin,
        cos,
        bitcoin::Network::Bitcoin,
        bitcoin::NetworkKind::Main,
        2,
    );

    let mut args = vec!["restore".into(), "--network".into(), "mainnet".into()];
    push_md1(&mut args, &md1);
    args.extend([
        "--from".into(),
        format!("phrase={SEED_A}"),
        "--account".into(),
        "0,1".into(),
        "--expect-wallet-id".into(),
        id,
        "--count".into(),
        "2".into(),
        "--json".into(),
    ]);
    let got = restore_addresses(&args);
    assert_eq!(
        got, golden,
        "mainnet all-own multisig template (coin 0') must remain restorable"
    );
}

// ===========================================================================
// L9 — the keyless MULTISIG-template completion path must apply the SAME
// has_hardened_use_site / unrestorable-taproot-override refusals the non-template
// `run_multisig` path applies, BEFORE reconstruction. A hardened use-site means
// watch-only public derivation is impossible (BIP-32), so the completion must
// REFUSE early with a precise, actionable message — not proceed into the search
// and emit an opaque downstream NO-MATCH (or, worse, silently render an
// unhardened path).
//
// The `bundle` CLI cannot EMIT a hardened-use-site keyless template (it refuses
// at emit, routing the operator to `--md1-form=policy`), so this guard is a
// latent / defense-in-depth gap: the RED input is constructed directly via the
// md-codec builder (mirroring `cli_restore_md1_template.rs`'s
// `keyless_multisig_md1_refused_at_restore`).
// ===========================================================================

/// Build a keyless `wsh(sortedmulti(2, @0,@1))` template md1 with a `*h`
/// HARDENED wildcard use-site. Canonical-origin-elided (template form). Returns
/// the md1 chunk(s).
fn hardened_use_site_multisig_template_md1() -> Vec<String> {
    use md_codec::origin_path::{OriginPath, PathDecl, PathDeclPaths};
    use md_codec::tag::Tag;
    use md_codec::tree::{Body, Node};
    use md_codec::use_site_path::{Alternative, UseSitePath};
    use md_codec::{Descriptor, TlvSection};

    let tree = Node {
        tag: Tag::Wsh,
        body: Body::Children(vec![Node {
            tag: Tag::SortedMulti,
            body: Body::MultiKeys {
                k: 2,
                indices: vec![0, 1],
            },
        }]),
    };
    // `<0;1>/*h` — a standard multipath with a HARDENED trailing wildcard.
    let use_site_path = UseSitePath {
        multipath: Some(vec![
            Alternative {
                hardened: false,
                value: 0,
            },
            Alternative {
                hardened: false,
                value: 1,
            },
        ]),
        wildcard_hardened: true,
    };
    let desc = Descriptor {
        n: 2,
        path_decl: PathDecl {
            n: 2,
            paths: PathDeclPaths::Shared(OriginPath { components: vec![] }),
        },
        use_site_path,
        tree,
        tlv: TlvSection {
            use_site_path_overrides: None,
            fingerprints: None,
            pubkeys: None,
            origin_path_overrides: None,
            unknown: Vec::new(),
        },
    };
    md_codec::chunk::split(&desc).expect("hardened-use-site keyless multisig md1 encodes")
}

#[test]
fn hardened_use_site_multisig_template_refused_early() {
    // A keyless wsh-sortedmulti template carrying a `/*h` hardened wildcard,
    // restored with a valid `--from` (so it routes through completion, past the
    // floor-1(i) --from gate) → the completion path must REFUSE early with a
    // precise "hardened use-site" message (exit != 0), MIRRORING run_multisig.
    // BEFORE the guard hoist this proceeds into the search and yields an opaque
    // NO-MATCH (or a hardened-derivation failure) — not the precise refusal.
    let md1 = hardened_use_site_multisig_template_md1();
    let mut args = vec!["restore".into(), "--network".into(), "mainnet".into()];
    push_md1(&mut args, &md1);
    args.extend([
        "--from".into(),
        format!("phrase={SEED_A}"),
        "--account".into(),
        "0,1".into(),
    ]);
    let assert = mnemonic().args(&args).assert().failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let low = stderr.to_lowercase();
    assert!(
        low.contains("hardened use-site") || low.contains("hardened use site"),
        "the completion path must refuse a hardened use-site template with the SAME \
         precise message run_multisig gives — not an opaque NO-MATCH. stderr: {stderr}"
    );
    assert!(
        !low.contains("no match"),
        "the refusal must be the EARLY hardened-use-site refusal, not a downstream \
         search NO-MATCH: {stderr}"
    );
}

#[test]
fn non_hardened_named_template_still_completes_positive_control() {
    // POSITIVE CONTROL: a legit NON-hardened named template (the L8 mainnet
    // all-own vector) must STILL complete — the L9 guard must not over-reject
    // legitimate non-hardened templates.
    let cos = &[(SEED_A, 0u32), (SEED_A, 1u32)];
    let coin = 0u32;
    let md1 = emit_template_md1("mainnet", coin, cos);
    let id = emit_template_wallet_id("mainnet", coin, cos);
    let golden = golden_addresses(
        coin,
        cos,
        bitcoin::Network::Bitcoin,
        bitcoin::NetworkKind::Main,
        2,
    );

    let mut args = vec!["restore".into(), "--network".into(), "mainnet".into()];
    push_md1(&mut args, &md1);
    args.extend([
        "--from".into(),
        format!("phrase={SEED_A}"),
        "--account".into(),
        "0,1".into(),
        "--expect-wallet-id".into(),
        id,
        "--count".into(),
        "2".into(),
        "--json".into(),
    ]);
    let got = restore_addresses(&args);
    assert_eq!(
        got, golden,
        "a non-hardened named template must complete (the L9 guard must not \
         over-reject legitimate templates)"
    );
}
