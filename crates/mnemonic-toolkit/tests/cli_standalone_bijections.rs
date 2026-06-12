//! Property B — standalone bijections.
//!
//! Design: `design/SPEC_cross_start_convergence_and_bijection_tests.md` +
//! FOLLOWUP `cross-start-convergence-remaining-cells`.
//!
//! Two edges the existing suite never closed as isolated loops:
//!   - `xpub → mk1 → xpub` (B1/B2 single-sig + reverse fp/path edges; B3 multisig)
//!   - `descriptor → md1 → descriptor` (B4 canonical / B5 non-canonical / B6 multisig)
//!
//! mk1 is decoded via `mnemonic convert --from mk1=… --to xpub[,fingerprint,path]`
//! (the `(Mk1,*)` edges); md1 is decoded via the `md_codec::chunk::reassemble`
//! library call (already a dependency). Self-contained — only the `mnemonic`
//! binary + md_codec lib; no sibling binary, no `#[ignore]`.

use assert_cmd::Command;
use bip39::Mnemonic;
use bitcoin::bip32::{DerivationPath, Xpriv, Xpub};
use bitcoin::secp256k1::Secp256k1;
use serde_json::Value;
use std::str::FromStr;

const TREZOR_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

const MS_PHRASES: [&str; 3] = [
    TREZOR_24,
    "legal winner thank year wave sausage worth useful legal winner thank yellow",
    "letter advice cage absurd amount doctor acoustic avoid letter advice cage above",
];

/// Derive `(account_xpub, master_fingerprint_hex)` from a phrase at `path_str`
/// (mainnet, empty passphrase). In-test derivation keeps seed↔xpub consistency
/// provable by construction (pattern: `cli_bundle_multisig.rs:25`).
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

fn bundle_json(args: &[&str]) -> Value {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(args)
        .assert()
        .success();
    serde_json::from_slice(&out.get_output().stdout).expect("--json output must be valid JSON")
}

/// Decode an mk1 chunk-list back to the requested field(s) via `convert`.
/// Returns the raw stdout (`"<field>: <value>\n…"`).
fn mk1_decode(chunks: &[String], to: &str) -> String {
    let joined = chunks.join(" ");
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["convert", "--from", &format!("mk1={joined}"), "--to", to])
        .assert()
        .success();
    String::from_utf8(out.get_output().stdout.clone()).unwrap()
}

fn json_str_array(v: &Value) -> Vec<String> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap().to_string())
        .collect()
}

// ===========================================================================
// B1 — xpub → mk1 → xpub (single-sig). Closes the loop the existing
// `mk1_to_xpub_decode` test left open (it decoded a hardcoded fixture; here
// we EMIT mk1 from an xpub via bundle, then decode back).
// ===========================================================================
#[test]
fn b1_xpub_to_mk1_to_xpub_singlesig() {
    let (xpub, fp) = derive_account_xpub(TREZOR_24, "m/84'/0'/0'");
    let v = bundle_json(&[
        "bundle",
        "--network",
        "mainnet",
        "--template",
        "bip84",
        "--slot",
        &format!("@0.xpub={xpub}"),
        "--slot",
        &format!("@0.fingerprint={fp}"),
        "--json",
        "--no-engraving-card",
    ]);
    let chunks = json_str_array(&v["mk1"]);
    let decoded = mk1_decode(&chunks, "xpub");
    let got = decoded
        .trim()
        .strip_prefix("xpub: ")
        .expect("convert emits 'xpub: <v>'");
    assert_eq!(got, xpub, "B1: xpub → mk1 → xpub must be byte-identical");
}

// ===========================================================================
// B2 — xpub → mk1 → (xpub, fingerprint, path): all three (Mk1,*) reverse edges.
// ===========================================================================
#[test]
fn b2_xpub_to_mk1_reverse_edges_xpub_fingerprint_path() {
    let (xpub, fp) = derive_account_xpub(TREZOR_24, "m/84'/0'/0'");
    let v = bundle_json(&[
        "bundle",
        "--network",
        "mainnet",
        "--template",
        "bip84",
        "--slot",
        &format!("@0.xpub={xpub}"),
        "--slot",
        &format!("@0.fingerprint={fp}"),
        "--json",
        "--no-engraving-card",
    ]);
    let chunks = json_str_array(&v["mk1"]);
    let decoded = mk1_decode(&chunks, "xpub,fingerprint,path");
    let lines: Vec<&str> = decoded.lines().collect();
    assert_eq!(lines[0].strip_prefix("xpub: ").unwrap(), xpub, "B2 xpub");
    assert_eq!(
        lines[1].strip_prefix("fingerprint: ").unwrap(),
        fp,
        "B2 fingerprint"
    );
    assert!(
        lines[2] == "path: 84'/0'/0'" || lines[2] == "path: m/84'/0'/0'",
        "B2 path: got {:?}",
        lines[2]
    );
}

// ===========================================================================
// B3 — xpub → mk1 → xpub, per-cosigner (multisig MkField::Multi chunking).
// ===========================================================================
#[test]
fn b3_xpub_to_mk1_to_xpub_multisig_per_cosigner() {
    let mut slots: Vec<String> = Vec::new();
    let mut xpubs: Vec<String> = Vec::new();
    for (i, p) in MS_PHRASES.iter().enumerate() {
        let (xpub, fp) = derive_account_xpub(p, "m/87'/0'/0'");
        slots.push("--slot".into());
        slots.push(format!("@{i}.xpub={xpub}"));
        slots.push("--slot".into());
        slots.push(format!("@{i}.fingerprint={fp}"));
        slots.push("--slot".into());
        slots.push(format!("@{i}.path=m/87'/0'/0'"));
        xpubs.push(xpub);
    }
    let mut args: Vec<String> = [
        "bundle",
        "--network",
        "mainnet",
        "--template",
        "wsh-sortedmulti",
        "--threshold",
        "2",
        "--multisig-path-family",
        "bip87",
        "--json",
        "--no-engraving-card",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    args.extend(slots);
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let v = bundle_json(&refs);

    let mk1_outer = v["mk1"]
        .as_array()
        .expect("multisig mk1 is an array of cosigner chunk-arrays");
    assert_eq!(mk1_outer.len(), 3, "B3 expects 3 cosigner mk1 sets");
    for (i, cosigner) in mk1_outer.iter().enumerate() {
        let chunks = json_str_array(cosigner);
        let decoded = mk1_decode(&chunks, "xpub");
        let got = decoded.trim().strip_prefix("xpub: ").unwrap();
        assert_eq!(got, xpubs[i], "B3: cosigner {i} xpub → mk1 → xpub mismatch");
    }
}

/// Assert the md1 ↔ descriptor bijection is byte-stable on the supplied md1
/// (which the bundle produced from a descriptor): `reassemble(&[&str]) ->
/// Descriptor` then `split(&Descriptor) -> Vec<String>` must reproduce the
/// byte-identical md1. md-codec exposes no string render for `Descriptor` and
/// no string→`Descriptor` parser, so the bijection is asserted in the direction
/// the API supports (md1 ⇄ Descriptor). Since the bundle already performed
/// descriptor → md1, this closes descriptor → md1 → descriptor → md1 and pins
/// it byte-stable. Returns the reassembled `Descriptor`.
fn assert_md1_bijection(md1: &[String], ctx: &str) -> md_codec::Descriptor {
    let strs: Vec<&str> = md1.iter().map(String::as_str).collect();
    let d = md_codec::chunk::reassemble(&strs)
        .unwrap_or_else(|e| panic!("{ctx}: md1 → descriptor reassemble failed: {e:?}"));
    let md1_again = md_codec::chunk::split(&d)
        .unwrap_or_else(|e| panic!("{ctx}: descriptor → md1 split failed: {e:?}"));
    assert_eq!(
        md1_again, md1,
        "{ctx}: descriptor → md1 → descriptor must be byte-identical"
    );
    d
}

// ===========================================================================
// B4 — descriptor → md1 → descriptor (canonical single-sig).
// ===========================================================================
#[test]
fn b4_descriptor_to_md1_to_descriptor_canonical_singlesig() {
    let (xpub, fp) = derive_account_xpub(TREZOR_24, "m/84'/0'/0'");
    let placeholder = format!("wpkh(@0[{fp}/84'/0'/0']/<0;1>/*)");
    let v = bundle_json(&[
        "bundle",
        "--network",
        "mainnet",
        "--descriptor",
        &placeholder,
        "--slot",
        &format!("@0.xpub={xpub}"),
        "--json",
        "--no-engraving-card",
    ]);
    let md1_bundle = json_str_array(&v["md1"]);
    let d = assert_md1_bijection(&md1_bundle, "B4 canonical wpkh");
    assert!(
        d.is_wallet_policy(),
        "B4: canonical single-sig is a wallet policy"
    );
}

// ===========================================================================
// B5 — descriptor → md1 → descriptor (non-canonical wsh(andor) miniscript).
// (No is_wallet_policy assertion — non-canonical miniscript need not be a
// BIP-388 wallet policy; the byte-stable round-trip is the bijection.)
// ===========================================================================
#[test]
fn b5_descriptor_to_md1_to_descriptor_non_canonical_andor() {
    let descriptor =
        "wsh(andor(pkh(@0),after(12000000),or_i(and_v(v:pkh(@1),older(4032)),and_v(v:pkh(@2),older(32768)))))";
    let mut slots: Vec<String> = Vec::new();
    for (i, p) in MS_PHRASES.iter().enumerate() {
        let (xpub, fp) = derive_account_xpub(p, "m/48'/0'/0'/2'");
        slots.push("--slot".into());
        slots.push(format!("@{i}.xpub={xpub}"));
        slots.push("--slot".into());
        slots.push(format!("@{i}.fingerprint={fp}"));
    }
    let mut args: Vec<String> = [
        "bundle",
        "--network",
        "mainnet",
        "--descriptor",
        descriptor,
        "--json",
        "--no-engraving-card",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    args.extend(slots);
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let v = bundle_json(&refs);
    let md1_bundle = json_str_array(&v["md1"]);
    assert_md1_bijection(&md1_bundle, "B5 non-canonical wsh(andor)");
}

// ===========================================================================
// B6 — descriptor → md1 → descriptor (multisig wsh-sortedmulti 2-of-3).
// ===========================================================================
#[test]
fn b6_descriptor_to_md1_to_descriptor_multisig() {
    let mut slots: Vec<String> = Vec::new();
    for (i, p) in MS_PHRASES.iter().enumerate() {
        let (xpub, fp) = derive_account_xpub(p, "m/87'/0'/0'");
        slots.push("--slot".into());
        slots.push(format!("@{i}.xpub={xpub}"));
        slots.push("--slot".into());
        slots.push(format!("@{i}.fingerprint={fp}"));
        slots.push("--slot".into());
        slots.push(format!("@{i}.path=m/87'/0'/0'"));
    }
    let mut args: Vec<String> = [
        "bundle",
        "--network",
        "mainnet",
        "--template",
        "wsh-sortedmulti",
        "--threshold",
        "2",
        "--multisig-path-family",
        "bip87",
        "--json",
        "--no-engraving-card",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    args.extend(slots);
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let v = bundle_json(&refs);
    let md1_bundle = json_str_array(&v["md1"]);
    let d = assert_md1_bijection(&md1_bundle, "B6 multisig wsh-sortedmulti");
    assert!(
        d.is_wallet_policy(),
        "B6: multisig sortedmulti is a wallet policy"
    );
}
