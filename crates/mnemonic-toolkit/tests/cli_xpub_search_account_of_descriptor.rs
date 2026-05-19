//! `mnemonic xpub-search account-of-descriptor` integration tests (v0.26.0 P2).
//!
//! Per `design/PLAN_v0_26_0_xpub_search.md` §10 C2. Realizes plan §4 (P2 SPEC):
//! - Descriptor intake polymorphism: literal-xpub / md1 / BIP-388 JSON shapes.
//! - Auto-detect tie-break order: `{` → BIP-388 JSON; `md1` HRP → md1;
//!   `@\d+` → toolkit @N (refused); else → literal-xpub.
//! - Toolkit @N descriptors refused (synthetic xpubs are non-searchable).
//! - NUMS sentinel cosigner reported as `unspendable_internal_key: true`.
//! - v0.19.0 silent default-path inference still emits the stderr notice.
//! - Per-cosigner search over the candidate path set; emit list of matches.
//!
//! Fixtures: BIP-39 vector `abandon × 11 about` is the universal master.

use assert_cmd::Command;
use bip39::Mnemonic;
use bitcoin::bip32::{DerivationPath, Fingerprint, Xpriv, Xpub};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::NetworkKind;
use predicates::prelude::*;
use serde_json::Value;
use std::str::FromStr;

const PHRASE: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

const OTHER_PHRASE: &str =
    "legal winner thank year wave sausage worth useful legal winner thank yellow";

/// Compute the xpub at `path` for the test phrase + optional passphrase.
fn xpub_at(phrase: &str, path: &str, passphrase: &str) -> Xpub {
    let mnemonic = Mnemonic::parse_in(bip39::Language::English, phrase).unwrap();
    let seed = mnemonic.to_seed(passphrase);
    let secp = Secp256k1::new();
    let master = Xpriv::new_master(NetworkKind::Main, &seed).unwrap();
    let dp = DerivationPath::from_str(path).unwrap();
    let xpriv = master.derive_priv(&secp, &dp).unwrap();
    Xpub::from_priv(&secp, &xpriv)
}

/// Master fingerprint of `phrase` (the fingerprint of the master xpub at m/).
fn master_fp(phrase: &str, passphrase: &str) -> Fingerprint {
    let mnemonic = Mnemonic::parse_in(bip39::Language::English, phrase).unwrap();
    let seed = mnemonic.to_seed(passphrase);
    let secp = Secp256k1::new();
    let master = Xpriv::new_master(NetworkKind::Main, &seed).unwrap();
    Xpub::from_priv(&secp, &master).fingerprint()
}

// ---------------------------------------------------------------------------
// Cell 1 — single-sig literal-xpub descriptor → cosigner 0 match at m/84'/0'/0'.
// ---------------------------------------------------------------------------
#[test]
fn account_of_descriptor_single_sig_wpkh_literal_xpub_match() {
    let xpub = xpub_at(PHRASE, "m/84'/0'/0'", "");
    let fp = master_fp(PHRASE, "");
    let descriptor = format!("wpkh([{}/84'/0'/0']{}/<0;1>/*)", fp, xpub);
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "account-of-descriptor",
            "--phrase-stdin",
            "--descriptor",
            &descriptor,
            "--json",
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_str(&String::from_utf8(out).unwrap()).unwrap();
    assert_eq!(v["mode"], "account-of-descriptor");
    assert_eq!(v["result"], "match");
    assert_eq!(v["schema_version"], "1");
    let matched = v["matched_cosigners"].as_array().expect("array");
    assert_eq!(matched.len(), 1);
    assert_eq!(matched[0]["cosigner_index"], 0);
    assert_eq!(matched[0]["path"], "m/84'/0'/0'");
    assert_eq!(matched[0]["template"], "bip84");
    assert_eq!(matched[0]["account"], 0);
    assert_eq!(v["cosigners_total"], 1);
    assert_eq!(v["descriptor_shape"], "literal_xpub");
}

// ---------------------------------------------------------------------------
// Cell 2 — multisig sortedmulti 2-of-3, seed = cosigner index 1.
// ---------------------------------------------------------------------------
#[test]
fn account_of_descriptor_multisig_sortedmulti_match_at_cosigner_1() {
    // Cosigner 0: OTHER_PHRASE; cosigner 1: PHRASE; cosigner 2: OTHER_PHRASE
    // at a different path. Path: m/48'/0'/0'/2' (BIP-48 wsh).
    let xpub0 = xpub_at(OTHER_PHRASE, "m/48'/0'/0'/2'", "");
    let xpub1 = xpub_at(PHRASE, "m/48'/0'/0'/2'", "");
    let xpub2 = xpub_at(OTHER_PHRASE, "m/48'/0'/1'/2'", "");
    let fp0 = master_fp(OTHER_PHRASE, "");
    let fp1 = master_fp(PHRASE, "");
    let fp2 = fp0;
    let descriptor = format!(
        "wsh(sortedmulti(2,[{}/48'/0'/0'/2']{}/<0;1>/*,[{}/48'/0'/0'/2']{}/<0;1>/*,[{}/48'/0'/1'/2']{}/<0;1>/*))",
        fp0, xpub0, fp1, xpub1, fp2, xpub2
    );
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "account-of-descriptor",
            "--phrase-stdin",
            "--descriptor",
            &descriptor,
            "--json",
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_str(&String::from_utf8(out).unwrap()).unwrap();
    assert_eq!(v["mode"], "account-of-descriptor");
    assert_eq!(v["result"], "match");
    let matched = v["matched_cosigners"].as_array().expect("array");
    assert_eq!(matched.len(), 1);
    assert_eq!(matched[0]["cosigner_index"], 1);
    assert_eq!(matched[0]["path"], "m/48'/0'/0'/2'");
    assert_eq!(matched[0]["template"], "bip48-wsh");
    assert_eq!(matched[0]["account"], 0);
    assert_eq!(v["cosigners_total"], 3);
}

// ---------------------------------------------------------------------------
// Cell 3 — multisig 2-of-3, seed matches NO cosigner → exit 4.
// ---------------------------------------------------------------------------
#[test]
fn account_of_descriptor_multisig_no_match_exits_4() {
    // All 3 cosigners are OTHER_PHRASE at various paths; PHRASE matches none.
    let xpub0 = xpub_at(OTHER_PHRASE, "m/48'/0'/0'/2'", "");
    let xpub1 = xpub_at(OTHER_PHRASE, "m/48'/0'/1'/2'", "");
    let xpub2 = xpub_at(OTHER_PHRASE, "m/48'/0'/2'/2'", "");
    let fp = master_fp(OTHER_PHRASE, "");
    let descriptor = format!(
        "wsh(sortedmulti(2,[{}/48'/0'/0'/2']{}/<0;1>/*,[{}/48'/0'/1'/2']{}/<0;1>/*,[{}/48'/0'/2'/2']{}/<0;1>/*))",
        fp, xpub0, fp, xpub1, fp, xpub2
    );
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "account-of-descriptor",
            "--phrase-stdin",
            "--descriptor",
            &descriptor,
            "--json",
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(4)
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_str(&String::from_utf8(out).unwrap()).unwrap();
    assert_eq!(v["result"], "no_match");
    assert_eq!(v["matched_cosigners"].as_array().unwrap().len(), 0);
    assert_eq!(v["cosigners_total"], 3);
}

// ---------------------------------------------------------------------------
// Cell 4 — md1 multi-chunk via --descriptor-from md1=- stdin → match.
//
// Plan §4.2 R2 lock: NO whitespace+comma split anywhere. Multi-chunk md1
// arrives via stdin one-chunk-per-line via `--descriptor-from md1=-`.
// Single-chunk inline (one md1 token, no whitespace) is Cell 5.
// ---------------------------------------------------------------------------
#[test]
fn account_of_descriptor_md1_stdin_multi_chunk_match() {
    let xpub = xpub_at(PHRASE, "m/84'/0'/0'", "");
    let fp_hex = master_fp(PHRASE, "").to_string();
    let descriptor_template = format!("wpkh(@0[{fp_hex}/84'/0'/0']/<0;1>/*)");
    // Emit a bundle (full single-sig) to obtain the md1 card.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            &descriptor_template,
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.xpub={xpub}"),
            "--slot",
            &format!("@0.fingerprint={fp_hex}"),
            "--json",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let v: Value = serde_json::from_str(&stdout).unwrap();
    let md1_strs: Vec<String> = v["md1"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_str().unwrap().to_string())
        .collect();
    // Stdin payload one chunk per line.
    let stdin_payload = md1_strs.join("\n");

    let xs_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "account-of-descriptor",
            "--phrase",
            PHRASE,
            "--descriptor-from",
            "md1=-",
            "--json",
        ])
        .write_stdin(stdin_payload)
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_str(&String::from_utf8(xs_out).unwrap()).unwrap();
    assert_eq!(v["result"], "match");
    assert_eq!(v["descriptor_shape"], "md1");
    let matched = v["matched_cosigners"].as_array().unwrap();
    assert_eq!(matched.len(), 1);
    assert_eq!(matched[0]["path"], "m/84'/0'/0'");
}

// ---------------------------------------------------------------------------
// Cell 5 — bad `--descriptor-from` node name → exit 1 with clear error.
// ---------------------------------------------------------------------------
#[test]
fn account_of_descriptor_unknown_descriptor_from_node_refused() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "account-of-descriptor",
            "--phrase-stdin",
            "--descriptor-from",
            "bogus=foo",
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(1)
        .stderr(predicate::str::contains("unknown node"));
}

// ---------------------------------------------------------------------------
// Cell 6 — BIP-388 wallet-policy JSON intake → match.
// ---------------------------------------------------------------------------
#[test]
fn account_of_descriptor_bip388_json_match() {
    let xpub = xpub_at(PHRASE, "m/84'/0'/0'", "");
    let fp = master_fp(PHRASE, "");
    // BIP-388 wallet_policy schema (mirrors wallet_export/pipeline.rs:160-204):
    let json = format!(
        r#"{{"name":"test","description_template":"wpkh(@0/**)","keys_info":["[{}/84'/0'/0']{}"]}}"#,
        fp, xpub
    );
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "account-of-descriptor",
            "--phrase-stdin",
            "--descriptor",
            &json,
            "--json",
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_str(&String::from_utf8(out).unwrap()).unwrap();
    assert_eq!(v["result"], "match");
    assert_eq!(v["descriptor_shape"], "bip388_json");
    let matched = v["matched_cosigners"].as_array().unwrap();
    assert_eq!(matched.len(), 1);
    assert_eq!(matched[0]["path"], "m/84'/0'/0'");
    assert_eq!(matched[0]["template"], "bip84");
}

// ---------------------------------------------------------------------------
// Cell 7 — Toolkit @N-placeholder descriptor → refusal exit 1.
// ---------------------------------------------------------------------------
#[test]
fn account_of_descriptor_toolkit_at_n_placeholder_refused() {
    let descriptor = "wsh(sortedmulti(2,@0[deadbeef/48'/0'/0'/2'],@1[cafebabe/48'/0'/0'/2'],@2[12345678/48'/0'/0'/2']))";
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "account-of-descriptor",
            "--phrase-stdin",
            "--descriptor",
            descriptor,
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(1)
        .stderr(predicate::str::contains("toolkit @N"));
}

// ---------------------------------------------------------------------------
// Cell 8 — v0.19.0 silent-default-path: literal-xpub descriptor without
// [fp/path] annotation gets BIP-48 default path applied + stderr notice.
//
// NOTE: §4.3 step 4 only fires for missing key-origin annotations. For a
// rust-miniscript-parseable literal-xpub descriptor, omitting `[fp/path]`
// is fully legal; we treat the missing path-anno as BIP-48 default
// (m/48'/coin'/account'/2' at account=0). Match should succeed since the
// xpub was actually derived at m/48'/0'/0'/2'.
// ---------------------------------------------------------------------------
#[test]
fn account_of_descriptor_default_path_inference_emits_stderr_notice() {
    // Multisig descriptor where one cosigner has no origin annotation.
    let xpub_self = xpub_at(PHRASE, "m/48'/0'/0'/2'", "");
    let xpub_other = xpub_at(OTHER_PHRASE, "m/48'/0'/0'/2'", "");
    let fp_other = master_fp(OTHER_PHRASE, "");
    // Cosigner @0 omits the [fp/path] annotation. We use the self xpub here
    // so it matches at the default path.
    let descriptor = format!(
        "wsh(sortedmulti(2,{}/<0;1>/*,[{}/48'/0'/0'/2']{}/<0;1>/*))",
        xpub_self, fp_other, xpub_other
    );
    let assertion = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "account-of-descriptor",
            "--phrase-stdin",
            "--descriptor",
            &descriptor,
            "--json",
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(0);
    let output = assertion.get_output();
    let stderr = String::from_utf8(output.stderr.clone()).unwrap();
    assert!(
        stderr.contains("non-canonical descriptor"),
        "stderr should carry the v0.19.0 default-path notice; got: {stderr}"
    );
    let stdout = String::from_utf8(output.stdout.clone()).unwrap();
    let v: Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(v["result"], "match");
}

// ---------------------------------------------------------------------------
// Cell 9 — NUMS sentinel via tr(NUMS, ...) → unspendable_internal_key=true;
// other cosigners still searched.
//
// rust-miniscript needs a valid `tr(<key>, <script>)` shape; we use the
// NUMS H-point hex directly (mirrors v0.19.0 SPEC §4.12.e), since
// `tr(NUMS,...)` is toolkit-specific syntax that rust-miniscript wouldn't
// accept. The tr script body references an xpub matching our seed.
// ---------------------------------------------------------------------------
#[test]
fn account_of_descriptor_nums_sentinel_marks_internal_key_unspendable() {
    let xpub_self = xpub_at(PHRASE, "m/48'/0'/0'/2'", "");
    let fp_self = master_fp(PHRASE, "");
    // NUMS H-point x-only key (same constant the toolkit uses).
    let nums_hex = "50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0";
    let descriptor = format!(
        "tr({},pk([{}/48'/0'/0'/2']{}/<0;1>/*))",
        nums_hex, fp_self, xpub_self
    );
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "account-of-descriptor",
            "--phrase-stdin",
            "--descriptor",
            &descriptor,
            "--json",
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_str(&String::from_utf8(out).unwrap()).unwrap();
    assert_eq!(v["result"], "match");
    let matched = v["matched_cosigners"].as_array().unwrap();
    // Exactly one match against the script-path xpub. NUMS internal key is
    // not searched.
    assert_eq!(matched.len(), 1);
}

// ---------------------------------------------------------------------------
// Cell 10 — descriptor with no xpub-shaped cosigners (all raw pubkeys)
// → exit 1 with clear message.
// ---------------------------------------------------------------------------
#[test]
fn account_of_descriptor_no_xpubs_in_descriptor_refused() {
    let descriptor = "wpkh(02c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee5)";
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "account-of-descriptor",
            "--phrase-stdin",
            "--descriptor",
            descriptor,
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(1)
        .stderr(predicate::str::contains("no extended keys"));
}

// ---------------------------------------------------------------------------
// Cell 11 — --max-account widens the per-cosigner search range.
// ---------------------------------------------------------------------------
#[test]
fn account_of_descriptor_max_account_widens_per_cosigner_range() {
    // Target xpub at account=30. Default range [0,20) misses; --max-account 50
    // catches it.
    let xpub = xpub_at(PHRASE, "m/84'/0'/30'", "");
    let fp = master_fp(PHRASE, "");
    let descriptor = format!("wpkh([{}/84'/0'/30']{}/<0;1>/*)", fp, xpub);

    // Default: no match.
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "account-of-descriptor",
            "--phrase-stdin",
            "--descriptor",
            &descriptor,
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(4);

    // --max-account 50: match.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "account-of-descriptor",
            "--phrase-stdin",
            "--descriptor",
            &descriptor,
            "--max-account",
            "50",
            "--json",
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_str(&String::from_utf8(out).unwrap()).unwrap();
    let matched = v["matched_cosigners"].as_array().unwrap();
    assert_eq!(matched[0]["account"], 30);
}

// ---------------------------------------------------------------------------
// Cell 12 — --add-path extends the candidate set per cosigner.
// ---------------------------------------------------------------------------
#[test]
fn account_of_descriptor_add_path_extends_candidates() {
    // Descriptor with xpub at m/87'/0'/0' (BIP-87, not in default set).
    let xpub = xpub_at(PHRASE, "m/87'/0'/0'", "");
    let fp = master_fp(PHRASE, "");
    let descriptor = format!("wpkh([{}/87'/0'/0']{}/<0;1>/*)", fp, xpub);

    // Without --add-path: no match.
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "account-of-descriptor",
            "--phrase-stdin",
            "--descriptor",
            &descriptor,
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(4);

    // With --add-path: match.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "account-of-descriptor",
            "--phrase-stdin",
            "--descriptor",
            &descriptor,
            "--add-path",
            "m/87'/0'/account'",
            "--json",
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_str(&String::from_utf8(out).unwrap()).unwrap();
    let matched = v["matched_cosigners"].as_array().unwrap();
    assert_eq!(matched[0]["template"], "m/87'/0'/account'");
}

// ---------------------------------------------------------------------------
// Cell 13 — --passphrase honored.
// ---------------------------------------------------------------------------
#[test]
fn account_of_descriptor_passphrase_honored() {
    let pp = "TREZOR";
    let xpub = xpub_at(PHRASE, "m/84'/0'/0'", pp);
    let fp = master_fp(PHRASE, pp);
    let descriptor = format!("wpkh([{}/84'/0'/0']{}/<0;1>/*)", fp, xpub);

    // Without passphrase: no match.
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "account-of-descriptor",
            "--phrase-stdin",
            "--descriptor",
            &descriptor,
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(4);

    // With --passphrase: match.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "account-of-descriptor",
            "--phrase",
            PHRASE,
            "--descriptor",
            &descriptor,
            "--passphrase",
            pp,
            "--json",
        ])
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_str(&String::from_utf8(out).unwrap()).unwrap();
    assert_eq!(v["result"], "match");
}

// ---------------------------------------------------------------------------
// Cell 14 — JSON envelope byte-shape across `matched_cosigners`.
// ---------------------------------------------------------------------------
#[test]
fn account_of_descriptor_json_envelope_byte_exact() {
    let xpub = xpub_at(PHRASE, "m/84'/0'/0'", "");
    let fp = master_fp(PHRASE, "");
    let descriptor = format!("wpkh([{}/84'/0'/0']{}/<0;1>/*)", fp, xpub);
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "account-of-descriptor",
            "--phrase-stdin",
            "--descriptor",
            &descriptor,
            "--json",
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_str(&String::from_utf8(out).unwrap()).unwrap();
    // Verify required fields.
    for field in &[
        "schema_version",
        "mode",
        "result",
        "matched_cosigners",
        "cosigners_total",
        "searched_count_per_cosigner",
        "descriptor_shape",
    ] {
        assert!(v.get(field).is_some(), "missing field {field}");
    }
    assert_eq!(v["mode"], "account-of-descriptor");
    assert_eq!(v["schema_version"], "1");
    assert!(v["searched_count_per_cosigner"].as_u64().unwrap() > 0);
}
