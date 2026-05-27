//! Property A — cross-start convergence.
//!
//! Design: `design/SPEC_cross_start_convergence_and_bijection_tests.md`.
//!
//! The m-format premise is "one secret, expressed four ways." These tests pin
//! that the same key/policy, entered as a **seed**, an **xpub**, a **wallet
//! descriptor**, or a **wallet file**, produces **byte-identical `mk1` + `md1`**
//! cards (and the xpub + master-fingerprint they embed).
//!
//! `ms1` is EXCLUDED from convergence: only the seed/entropy start carries
//! entropy; watch-only starts carry the `[""]` sentinel (asserted as such, never
//! compared across starts).
//!
//! Mechanism: drive only the `mnemonic` binary; compare the `--json` `mk1`/`md1`
//! card arrays (deterministic, order-stable). Fully self-contained — no sibling
//! binary, no `#[ignore]`.

use assert_cmd::Command;
use bip39::Mnemonic;
use bitcoin::bip32::{DerivationPath, Xpriv, Xpub};
use bitcoin::secp256k1::Secp256k1;
use miniscript::descriptor::checksum::Engine as ChecksumEngine;
use serde_json::Value;
use std::str::FromStr;

const TREZOR_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
const TREZOR_24_FP: &str = "5436d724";

/// Derive `(account_xpub, master_fingerprint_hex)` from a BIP-39 phrase at the
/// given path on mainnet (empty passphrase). Deriving in-test (rather than
/// hard-coding a constant) keeps seed↔xpub consistency provable by construction
/// — the whole point of a convergence test. Pattern: `cli_bundle_multisig.rs:25`.
fn derive_account_xpub(phrase: &str, path_str: &str) -> (String, String) {
    let secp = Secp256k1::new();
    let m = Mnemonic::parse_in(bip39::Language::English, phrase).unwrap();
    let seed = m.to_seed("");
    let master = Xpriv::new_master(bitcoin::NetworkKind::Main, &seed).unwrap();
    let fp = master.fingerprint(&secp).to_string().to_lowercase();
    let path = DerivationPath::from_str(path_str).unwrap();
    let xpriv = master.derive_priv(&secp, &path).unwrap();
    let xpub = Xpub::from_priv(&secp, &xpriv).to_string();
    (xpub, fp)
}

/// Run `mnemonic <args>` expecting exit 0; parse stdout as JSON.
fn bundle_json(args: &[&str]) -> Value {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(args)
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    serde_json::from_str(&stdout).expect("--json output must be valid JSON")
}

/// Assert two bundle JSON values converge on the engraved cards (`mk1` + `md1`),
/// byte-identical. This is the cross-start convergence property.
fn assert_cards_converge(a: &Value, b: &Value, ctx: &str) {
    assert_eq!(
        a["mk1"], b["mk1"],
        "{ctx}: mk1 cards must be byte-identical across starts"
    );
    assert_eq!(
        a["md1"], b["md1"],
        "{ctx}: md1 cards must be byte-identical across starts"
    );
}

/// Build a BSMS 2-line blob from a descriptor body with a fresh BIP-380 checksum
/// (pattern: `cli_import_wallet_seed_overlay.rs:38`). Used by A7.
fn bsms_2line(body: &str) -> String {
    let mut e = ChecksumEngine::new();
    e.input(body).expect("checksum input must be ASCII");
    let csum = e.checksum();
    format!("BSMS 1.0\n{body}#{csum}\n")
}

// ===========================================================================
// A1 — seed ≡ xpub (single-sig BIP-84)
// ===========================================================================
#[test]
fn a1_seed_vs_xpub_singlesig_bip84_converge() {
    let seed = bundle_json(&[
        "bundle",
        "--network",
        "mainnet",
        "--template",
        "bip84",
        "--slot",
        &format!("@0.phrase={TREZOR_24}"),
        "--json",
        "--no-engraving-card",
    ]);
    let (acct_xpub, fp) = derive_account_xpub(TREZOR_24, "m/84'/0'/0'");
    assert_eq!(fp, TREZOR_24_FP, "sanity: derived master fingerprint");
    let xpub = bundle_json(&[
        "bundle",
        "--network",
        "mainnet",
        "--template",
        "bip84",
        "--slot",
        &format!("@0.xpub={acct_xpub}"),
        "--slot",
        &format!("@0.fingerprint={fp}"),
        "--json",
        "--no-engraving-card",
    ]);
    assert_cards_converge(&seed, &xpub, "A1 seed vs xpub");
    // ms1 excluded from convergence: seed carries entropy, xpub-start is sentinel.
    assert!(seed["ms1"][0].as_str().unwrap().starts_with("ms1"));
    assert_eq!(xpub["ms1"], serde_json::json!([""]));
}

// ===========================================================================
// A2 — seed (template mode) ≡ descriptor mode (single-sig BIP-84, canonical)
// ===========================================================================
#[test]
fn a2_seed_template_vs_descriptor_singlesig_converge() {
    let templ = bundle_json(&[
        "bundle",
        "--network",
        "mainnet",
        "--template",
        "bip84",
        "--slot",
        &format!("@0.phrase={TREZOR_24}"),
        "--json",
        "--no-engraving-card",
    ]);
    let descriptor = format!("wpkh(@0[{TREZOR_24_FP}/84'/0'/0']/<0;1>/*)");
    let desc = bundle_json(&[
        "bundle",
        "--network",
        "mainnet",
        "--descriptor",
        &descriptor,
        "--slot",
        &format!("@0.phrase={TREZOR_24}"),
        "--json",
        "--no-engraving-card",
    ]);
    assert_cards_converge(&templ, &desc, "A2 template vs descriptor");
}

// ===========================================================================
// A1-neg — xpub-start WITHOUT @0.fingerprint diverges (fp is load-bearing).
// Guards A1 against a vacuous pass: if the fingerprint defaulted identically on
// both paths, A1 would be meaningless. Both invocations succeed (exit 0); the
// assertion is the inequality.
// ===========================================================================
#[test]
fn a1_neg_xpub_without_fingerprint_diverges_on_mk1() {
    let seed = bundle_json(&[
        "bundle",
        "--network",
        "mainnet",
        "--template",
        "bip84",
        "--slot",
        &format!("@0.phrase={TREZOR_24}"),
        "--json",
        "--no-engraving-card",
    ]);
    let (acct_xpub, _fp) = derive_account_xpub(TREZOR_24, "m/84'/0'/0'");
    let xpub_no_fp = bundle_json(&[
        "bundle",
        "--network",
        "mainnet",
        "--template",
        "bip84",
        "--slot",
        &format!("@0.xpub={acct_xpub}"),
        "--json",
        "--no-engraving-card",
    ]);
    assert_ne!(
        seed["mk1"], xpub_no_fp["mk1"],
        "xpub-start WITHOUT a fingerprint must diverge from seed-start mk1 (the master \
         fingerprint is load-bearing in mk1's origin field)"
    );
}

/// Export a single-sig bip84 bitcoin-core blob for `acct_xpub`+`fp`, and return
/// the FIRST (receive, `/0/*`) descriptor entry wrapped as a 1-entry
/// `listdescriptors` object. bitcoin-core splits `<0;1>` into `/0/*` + `/1/*`
/// (finding F1): we take the `/0/*` entry so the wallet-file carries a single
/// canonical descriptor to converge against. One entry ⇒ `bundle --import-json`
/// needs no `--import-json-index`.
fn core_blob_receive_only(acct_xpub: &str, fp: &str) -> String {
    let out = Command::cargo_bin("mnemonic")
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
            &format!("@0.xpub={acct_xpub}"),
            "--slot",
            &format!("@0.fingerprint={fp}"),
        ])
        .assert()
        .success();
    let arr: Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    let receive = arr.as_array().expect("core export is a bare array")[0].clone();
    serde_json::json!({ "wallet_name": "convergence", "descriptors": [receive] }).to_string()
}

/// Drive the wallet-file start: `import-wallet --json` (envelope) → `bundle
/// --import-json -` (cards). Returns the bundle JSON.
fn walletfile_to_bundle(format: &str, blob: &str) -> Value {
    let imp = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["import-wallet", "--format", format, "--blob", "-", "--json"])
        .write_stdin(blob.to_string())
        .assert()
        .success();
    let envelope = String::from_utf8(imp.get_output().stdout.clone()).unwrap();
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--network",
            "mainnet",
            "--import-json",
            "-",
            "--json",
            "--no-engraving-card",
        ])
        .write_stdin(envelope)
        .assert()
        .success();
    serde_json::from_slice(&out.get_output().stdout).unwrap()
}

// ===========================================================================
// A4 — seed ≡ wallet-file (bitcoin-core single-sig). Per F1, bitcoin-core
// carries the split `/0/*` descriptor, so the seed start uses that same
// descriptor (honest scoped convergence).
// ===========================================================================
#[test]
fn a4_seed_vs_walletfile_bitcoin_core_singlesig_converge() {
    let (acct_xpub, fp) = derive_account_xpub(TREZOR_24, "m/84'/0'/0'");
    let blob = core_blob_receive_only(&acct_xpub, &fp);
    let wf = walletfile_to_bundle("bitcoin-core", &blob);
    // Seed start with the MATCHING split descriptor (F1: bitcoin-core = /0/*).
    let seed = bundle_json(&[
        "bundle",
        "--network",
        "mainnet",
        "--descriptor",
        &format!("wpkh(@0[{fp}/84'/0'/0']/0/*)"),
        "--slot",
        &format!("@0.phrase={TREZOR_24}"),
        "--json",
        "--no-engraving-card",
    ]);
    assert_cards_converge(
        &seed,
        &wf,
        "A4 seed vs bitcoin-core wallet-file (split /0/* descriptor per F1)",
    );
}

// ===========================================================================
// A5 — all-four transitive convergence on the bitcoin-core `/0/*` descriptor:
// seed ≡ xpub ≡ descriptor(concrete) ≡ wallet-file.
// ===========================================================================
#[test]
fn a5_all_four_starts_converge_singlesig() {
    let (acct_xpub, fp) = derive_account_xpub(TREZOR_24, "m/84'/0'/0'");
    let placeholder_desc = format!("wpkh(@0[{fp}/84'/0'/0']/0/*)");

    // (1) seed start
    let seed = bundle_json(&[
        "bundle",
        "--network",
        "mainnet",
        "--descriptor",
        &placeholder_desc,
        "--slot",
        &format!("@0.phrase={TREZOR_24}"),
        "--json",
        "--no-engraving-card",
    ]);
    // (2) xpub start (watch-only, same descriptor)
    let xpub = bundle_json(&[
        "bundle",
        "--network",
        "mainnet",
        "--descriptor",
        &placeholder_desc,
        "--slot",
        &format!("@0.xpub={acct_xpub}"),
        "--json",
        "--no-engraving-card",
    ]);
    // (3) wallet-file start (bitcoin-core /0/*)
    let blob = core_blob_receive_only(&acct_xpub, &fp);
    let wf = walletfile_to_bundle("bitcoin-core", &blob);

    assert_cards_converge(&seed, &xpub, "A5 seed vs xpub");
    assert_cards_converge(&seed, &wf, "A5 seed vs wallet-file");
    // Transitive: xpub ≡ wallet-file follows, asserted explicitly for clarity.
    assert_cards_converge(&xpub, &wf, "A5 xpub vs wallet-file");
}

/// Three distinct BIP-39 seeds for the multisig cells. Mixed word-counts are
/// fine (each derives its own cosigner key).
const MS_PHRASES: [&str; 3] = [
    TREZOR_24,
    "legal winner thank year wave sausage worth useful legal winner thank yellow",
    "letter advice cage absurd amount doctor acoustic avoid letter advice cage above",
];

fn bundle_json_owned(args: &[String]) -> Value {
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    bundle_json(&refs)
}

// ===========================================================================
// A6 — multisig BIP-87 wsh-sortedmulti 2-of-3: seed ≡ xpub. Exercises the
// `MkField::Multi` per-cosigner card path. Cosigner declaration order is held
// identical across both starts, so mk1[i] and the sorted md1 policy converge.
// ===========================================================================
#[test]
fn a6_multisig_bip87_seed_vs_xpub_converge() {
    let common = |extra: &[String]| -> Vec<String> {
        let mut a = vec![
            "bundle".into(),
            "--network".into(),
            "mainnet".into(),
            "--template".into(),
            "wsh-sortedmulti".into(),
            "--threshold".into(),
            "2".into(),
            "--multisig-path-family".into(),
            "bip87".into(),
            "--json".into(),
            "--no-engraving-card".into(),
        ];
        a.extend_from_slice(extra);
        a
    };

    let mut seed_slots = Vec::new();
    for (i, p) in MS_PHRASES.iter().enumerate() {
        seed_slots.push("--slot".into());
        seed_slots.push(format!("@{i}.phrase={p}"));
    }
    let seed = bundle_json_owned(&common(&seed_slots));

    let mut xpub_slots = Vec::new();
    for (i, p) in MS_PHRASES.iter().enumerate() {
        let (xpub, fp) = derive_account_xpub(p, "m/87'/0'/0'");
        xpub_slots.push("--slot".into());
        xpub_slots.push(format!("@{i}.xpub={xpub}"));
        xpub_slots.push("--slot".into());
        xpub_slots.push(format!("@{i}.fingerprint={fp}"));
        xpub_slots.push("--slot".into());
        xpub_slots.push(format!("@{i}.path=m/87'/0'/0'"));
    }
    let xpub = bundle_json_owned(&common(&xpub_slots));

    assert_cards_converge(&seed, &xpub, "A6 multisig bip87 seed vs xpub");
    // Sanity: this is the multi-cosigner path (3 mk1 chunk-arrays).
    assert_eq!(seed["mk1"].as_array().unwrap().len(), 3, "A6 expects 3 cosigner mk1 sets");
}

// ===========================================================================
// A7 — multisig BIP-48 wsh-sortedmulti 2-of-3: seed ≡ BSMS wallet-file. BSMS
// preserves the full `/<0;1>/*` multipath descriptor (unlike bitcoin-core, F1),
// so convergence is on the canonical multipath form. Cosigner declaration order
// in the BSMS body matches the seed-start slot order.
// ===========================================================================
#[test]
fn a7_multisig_bip48_seed_vs_bsms_walletfile_converge() {
    let mut keys = Vec::new();
    for p in MS_PHRASES.iter() {
        let (xpub, fp) = derive_account_xpub(p, "m/48'/0'/0'/2'");
        keys.push(format!("[{fp}/48'/0'/0'/2']{xpub}/<0;1>/*"));
    }
    let body = format!("wsh(sortedmulti(2,{}))", keys.join(","));
    let wf = walletfile_to_bundle("bsms", &bsms_2line(&body));

    let mut seed_args: Vec<String> = [
        "bundle",
        "--network",
        "mainnet",
        "--template",
        "wsh-sortedmulti",
        "--threshold",
        "2",
        "--multisig-path-family",
        "bip48",
        "--json",
        "--no-engraving-card",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    for (i, p) in MS_PHRASES.iter().enumerate() {
        seed_args.push("--slot".into());
        seed_args.push(format!("@{i}.phrase={p}"));
    }
    let seed = bundle_json_owned(&seed_args);

    assert_cards_converge(&seed, &wf, "A7 multisig bip48 seed vs BSMS wallet-file");
}

// ===========================================================================
// A8 — non-canonical `wsh(andor)` descriptor ≡ BSMS wallet-file (watch-only).
// Each placeholder key carries an explicit `/<0;1>/*` use-site so BOTH starts
// describe the SAME ranged wallet (a bare `@N` yields a no-wildcard use-site —
// a different wallet). Convergence proves the descriptor-mode synthesis and the
// BSMS import→bundle path agree for a non-canonical miniscript policy.
//
// This cell drove the F4 fix: the elided-origin default-path inference
// (bundle.rs) emitted `PathDecl::Divergent([p,p,p])` for identical inferred
// paths while the explicit-origin wallet-import path emitted `PathDecl::Shared`
// → byte-different md1 for the same wallet. F4 now collapses identical inferred
// paths to `Shared`, matching parse_descriptor + synthesize_unified.
// ===========================================================================
#[test]
fn a8_non_canonical_descriptor_vs_bsms_walletfile_converge() {
    let placeholder = "wsh(andor(pkh(@0/<0;1>/*),after(12000000),or_i(and_v(v:pkh(@1/<0;1>/*),older(4032)),and_v(v:pkh(@2/<0;1>/*),older(32768)))))";
    let mut slots: Vec<String> = Vec::new();
    let mut keys: Vec<String> = Vec::new();
    for (i, p) in MS_PHRASES.iter().enumerate() {
        let (xpub, fp) = derive_account_xpub(p, "m/48'/0'/0'/2'");
        slots.push("--slot".into());
        slots.push(format!("@{i}.xpub={xpub}"));
        slots.push("--slot".into());
        slots.push(format!("@{i}.fingerprint={fp}"));
        keys.push(format!("[{fp}/48'/0'/0'/2']{xpub}/<0;1>/*"));
    }
    let mut dargs: Vec<String> = [
        "bundle",
        "--network",
        "mainnet",
        "--descriptor",
        placeholder,
        "--json",
        "--no-engraving-card",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    dargs.extend(slots);
    let desc_start = bundle_json_owned(&dargs);

    let concrete = format!(
        "wsh(andor(pkh({}),after(12000000),or_i(and_v(v:pkh({}),older(4032)),and_v(v:pkh({}),older(32768)))))",
        keys[0], keys[1], keys[2]
    );
    let wf = walletfile_to_bundle("bsms", &bsms_2line(&concrete));

    assert_cards_converge(
        &desc_start,
        &wf,
        "A8 non-canonical wsh(andor) descriptor vs BSMS wallet-file",
    );
}
