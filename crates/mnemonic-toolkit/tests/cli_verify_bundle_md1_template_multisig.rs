//! #28 phase 2 — `mnemonic verify-bundle --md1 <keyless-multisig-template>`
//! COMPLETION + binding integration tests (P4 / Slice 3).
//!
//! verify-bundle gains the SAME completion intake restore has (`--from` +
//! `--cosigner` + the search options) and runs the SAME permutation-search
//! completion engine (P3a canonical + P3b general). It then ALSO asserts the
//! card↔template-id binding (the supplied keyless md1 + the N template-form mk1
//! stubs bind to the recomposed wallet's `WalletDescriptorTemplateId`) and
//! surfaces the completed `WalletPolicyId` + first address.
//!
//! Funds-safety / silent-wrong-wallet class. The make-or-break gates are NOT
//! exit-0; they are:
//!   - the recomposed first address == an INDEPENDENT rust-miniscript golden;
//!   - PARITY: the completed id/address verify-bundle reports == what `restore`
//!     reports for the SAME inputs (the two surfaces never diverge);
//!   - the binding checks (md1_template_match + mk1_template_stub_bind) pass for
//!     the genuine cards and FAIL on a cross-mix;
//!   - the floors: no `--from` refuses; a wrong/outsider cosigner NO-MATCHes
//!     (exit 4), never a silent OK.
//!
//! The template md1 + the template-form mk1 stubs are emitted by
//! `bundle --md1-form=template`; the per-cosigner mk1s carrying REAL origins for
//! completion are emitted by `bundle --md1-form=policy` (mirrors the restore
//! test). Goldens are anchored OUTSIDE the toolkit synth path (rust-miniscript).

use assert_cmd::Command;
use bip39::Mnemonic;
use bitcoin::bip32::{DerivationPath, Xpriv, Xpub};
use bitcoin::secp256k1::Secp256k1;
use miniscript::{Descriptor, DescriptorPublicKey};
use std::str::FromStr;

const SEED_A: &str = "legal winner thank year wave sausage worth useful legal winner thank yellow";
const SEED_B: &str =
    "letter advice cage absurd amount doctor acoustic avoid letter advice cage above";
const SEED_C: &str = "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong";
const SEED_OUTSIDER: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

fn mnemonic() -> Command {
    Command::cargo_bin("mnemonic").expect("mnemonic binary builds")
}

/// Extract md1 string(s) from `bundle` text stdout (lines under `# md1`).
fn md1_lines(stdout: &str) -> Vec<String> {
    section_lines(stdout, "# md1")
}

/// Extract the per-cosigner mk1 string(s) from `bundle` text stdout.
fn mk1_groups(stdout: &str) -> Vec<Vec<String>> {
    let mut groups: Vec<Vec<String>> = Vec::new();
    let mut cur: Option<Vec<String>> = None;
    for line in stdout.lines() {
        if line.starts_with("# mk1") {
            if let Some(g) = cur.take() {
                if !g.is_empty() {
                    groups.push(g);
                }
            }
            cur = Some(Vec::new());
            continue;
        }
        if let Some(g) = cur.as_mut() {
            let t = line.trim();
            if t.starts_with("mk1") {
                g.push(t.to_string());
            }
        }
    }
    if let Some(g) = cur.take() {
        if !g.is_empty() {
            groups.push(g);
        }
    }
    groups
}

fn section_lines(stdout: &str, header: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut in_sec = false;
    for line in stdout.lines() {
        if line.starts_with(header) {
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

/// Derive a mainnet account xpub + master fingerprint at `path_str`.
fn xpub_at(phrase: &str, path_str: &str) -> (Xpub, String) {
    let secp = Secp256k1::new();
    let m = Mnemonic::parse_in(bip39::Language::English, phrase).unwrap();
    let seed = m.to_seed("");
    let master = Xpriv::new_master(bitcoin::NetworkKind::Main, &seed).unwrap();
    let fp = master.fingerprint(&secp);
    let path = DerivationPath::from_str(path_str).unwrap();
    let xpriv = master.derive_priv(&secp, &path).unwrap();
    let xpub = Xpub::from_priv(&secp, &xpriv);
    (xpub, fp.to_string().to_lowercase())
}

/// The canonical BIP-48 origin for a given wsh/sh-wsh multisig at `account`.
fn canonical_path(script: &str, account: u32) -> String {
    match script {
        "wsh-multi" | "wsh-sortedmulti" => format!("48'/0'/{account}'/2'"),
        "sh-wsh-multi" | "sh-wsh-sortedmulti" => format!("48'/0'/{account}'/1'"),
        other => panic!("unknown script {other}"),
    }
}

/// Build the `bundle` arg vector for a canonical multisig template/policy.
fn canonical_bundle_args(
    form: &str,
    script: &str,
    threshold: &str,
    cosigners: &[(&str, u32)],
) -> Vec<String> {
    let mut args: Vec<String> = vec![
        "bundle".into(),
        "--network".into(),
        "mainnet".into(),
        "--template".into(),
        script.into(),
        "--threshold".into(),
        threshold.into(),
        "--md1-form".into(),
        form.into(),
        "--group-size".into(),
        "0".into(),
        "--no-engraving-card".into(),
    ];
    for (idx, (phrase, account)) in cosigners.iter().enumerate() {
        let path = canonical_path(script, *account);
        let (xpub, fp) = xpub_at(phrase, &path);
        args.push("--slot".into());
        args.push(format!("@{idx}.xpub={xpub}"));
        args.push("--slot".into());
        args.push(format!("@{idx}.fingerprint={fp}"));
        args.push("--slot".into());
        args.push(format!("@{idx}.path={path}"));
    }
    args
}

/// Emit the keyless template md1 for a canonical multisig.
fn emit_template_md1(script: &str, threshold: &str, cosigners: &[(&str, u32)]) -> Vec<String> {
    let args = canonical_bundle_args("template", script, threshold, cosigners);
    let out = mnemonic().args(&args).assert().success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    md1_lines(&stdout)
}

/// Emit the N TEMPLATE-form mk1 STUB cards (the engraved cards that bind via the
/// template-id) — every group, in slot order.
fn emit_template_mk1_stubs(
    script: &str,
    threshold: &str,
    cosigners: &[(&str, u32)],
) -> Vec<Vec<String>> {
    let args = canonical_bundle_args("template", script, threshold, cosigners);
    let out = mnemonic().args(&args).assert().success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    mk1_groups(&stdout)
}

/// The printed WalletPolicyId (full hex) the template emit advisory records.
fn emit_template_wallet_id(script: &str, threshold: &str, cosigners: &[(&str, u32)]) -> String {
    let args = canonical_bundle_args("template", script, threshold, cosigners);
    let out = mnemonic().args(&args).assert().success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    let line = stderr
        .lines()
        .find(|l| l.contains("wallet-id (hex)"))
        .unwrap_or_else(|| panic!("no wallet-id (hex) line in: {stderr}"));
    line.split(':').next_back().unwrap().trim().to_string()
}

/// Emit a single cosigner mk1 card (POLICY form) carrying the REAL origin so the
/// completion reads its actual origin path.
fn emit_cosigner_mk1(
    script: &str,
    threshold: &str,
    cosigners: &[(&str, u32)],
    which: usize,
) -> Vec<String> {
    let args = canonical_bundle_args("policy", script, threshold, cosigners);
    let out = mnemonic().args(&args).assert().success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let groups = mk1_groups(&stdout);
    groups
        .get(which)
        .unwrap_or_else(|| panic!("no mk1 group {which} in: {stdout}"))
        .clone()
}

/// INDEPENDENT golden for a canonical wsh/sh-wsh multisig (rust-miniscript).
fn golden_addresses(
    script: &str,
    threshold: u32,
    cosigners: &[(&str, u32)],
    sorted: bool,
    count: u32,
) -> Vec<String> {
    let mut key_strs: Vec<String> = Vec::new();
    for (phrase, account) in cosigners {
        let path = canonical_path(script, *account);
        let (xpub, fp) = xpub_at(phrase, &path);
        let origin = path.replace('\'', "h");
        key_strs.push(format!("[{fp}/{origin}]{xpub}/<0;1>/*"));
    }
    let inner = if sorted {
        format!("sortedmulti({threshold},{})", key_strs.join(","))
    } else {
        format!("multi({threshold},{})", key_strs.join(","))
    };
    let desc_str = match script {
        "wsh-multi" | "wsh-sortedmulti" => format!("wsh({inner})"),
        "sh-wsh-multi" | "sh-wsh-sortedmulti" => format!("sh(wsh({inner}))"),
        other => panic!("unknown script {other}"),
    };
    let desc = Descriptor::<DescriptorPublicKey>::from_str(&desc_str)
        .unwrap_or_else(|e| panic!("golden descriptor parse {desc_str}: {e}"));
    let receive = desc.clone().into_single_descriptors().unwrap().remove(0);
    (0..count)
        .map(|i| {
            receive
                .derive_at_index(i)
                .unwrap()
                .address(bitcoin::Network::Bitcoin)
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

fn push_mk1_stubs(args: &mut Vec<String>, stubs: &[Vec<String>]) {
    for g in stubs {
        for c in g {
            args.push("--mk1".into());
            args.push(c.clone());
        }
    }
}

fn push_cosigners(args: &mut Vec<String>, cards: &[Vec<String>]) {
    for g in cards {
        for c in g {
            args.push("--cosigner".into());
            args.push(c.clone());
        }
    }
}

/// Parse a verify-bundle `--json` envelope.
fn verify_json(args: &[String]) -> serde_json::Value {
    let out = mnemonic().args(args).assert().success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("verify-bundle --json parse: {e}; stdout: {stdout}"))
}

/// The restore-reported first address for the SAME inputs (parity oracle).
fn restore_first_address(args: &[String]) -> String {
    let out = mnemonic().args(args).assert().success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let j: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("restore --json parse: {e}; stdout: {stdout}"));
    j["wallets"][0]["first_addresses"][0]
        .as_str()
        .unwrap()
        .to_string()
}

/// Decode a chunk-form md1 set and RE-ENCODE it as a single NON-chunked md1
/// string (the bare `md encode` form; encode_md1_string always emits the
/// single-payload/non-chunked form — the shape `bundle` never emits).
fn to_nonchunked(chunk_form_md1: &[String]) -> String {
    let refs: Vec<&str> = chunk_form_md1.iter().map(String::as_str).collect();
    let d = md_codec::chunk::reassemble(&refs).expect("chunk-form md1 decodes");
    md_codec::encode_md1_string(&d).expect("re-encode as a single non-chunked md1")
}

#[test]
fn verify_bundle_nonchunked_multisig_template_routes_no_from() {
    // A keyless 2-of-2 wsh-sortedmulti TEMPLATE, re-encoded non-chunked, supplied
    // WITHOUT --from, must REACH verify_multisig_template and refuse naming the
    // seed requirement (proving Facet 1 routed it). Today it falls THROUGH the
    // chunk-form-only classify gate → the general dispatch errors differently
    // (no "--from/seed" refusal). Keyless 2-of-2 template is < 400 bits → fits
    // a single non-chunked string.
    let cos = &[(SEED_A, 0u32), (SEED_B, 0u32)];
    let md1 = emit_template_md1("wsh-sortedmulti", "2", cos);
    let stubs = emit_template_mk1_stubs("wsh-sortedmulti", "2", cos);
    let single = to_nonchunked(&md1);
    // --mk1 is clap-required alongside --md1 (verify_bundle.rs:183); supply the
    // template-form stubs (as the chunked sibling `..._no_from_refuses` does) so
    // the invocation reaches the classify gate. WITHOUT --from.
    let mut args = vec!["verify-bundle".into(), "--network".into(), "mainnet".into()];
    push_md1(&mut args, &[single]);
    push_mk1_stubs(&mut args, &stubs);
    let assert = mnemonic().args(&args).assert().failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("--from") || stderr.contains("seed"),
        "non-chunked multisig template must route to verify_multisig_template and \
         name the --from/seed requirement (proves Facet 1 routed): {stderr}"
    );
}

#[test]
fn verify_bundle_nonchunked_multisig_template_verifies_ok() {
    // OUT-3 "free ride": a non-chunked keyless multisig template, completed via
    // --from + --cosigner, verifies GREEN through the (already id-based) WDT-id
    // compare (:937-941). Proves Facet 1 alone closes the multisig leg.
    let cos = &[(SEED_A, 0u32), (SEED_B, 0u32)];
    let md1 = emit_template_md1("wsh-sortedmulti", "2", cos);
    let stubs = emit_template_mk1_stubs("wsh-sortedmulti", "2", cos);
    let id = emit_template_wallet_id("wsh-sortedmulti", "2", cos);
    let mk1_b = emit_cosigner_mk1("wsh-sortedmulti", "2", cos, 1);

    let mut args = vec!["verify-bundle".into(), "--network".into(), "mainnet".into()];
    push_md1(&mut args, &[to_nonchunked(&md1)]); // <-- NON-chunked single string
    push_mk1_stubs(&mut args, &stubs);
    args.extend([
        "--from".into(),
        format!("phrase={SEED_A}"),
        "--account".into(),
        "0".into(),
        "--expect-wallet-id".into(),
        id,
        "--json".into(),
    ]);
    push_cosigners(&mut args, &[mk1_b]);

    let j = verify_json(&args);
    assert_eq!(
        j["result"], "ok",
        "non-chunked multisig template must verify OK: {j}"
    );
    let by = |n: &str| {
        j["checks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["name"] == n)
            .unwrap()["passed"]
            .clone()
    };
    assert_eq!(
        by("md1_template_match"),
        true,
        "md1_template_match must pass: {j}"
    );
}

// ===========================================================================
// 1. canonical multisig template, id-search → recomposes + binds + parity.
// ===========================================================================

#[test]
fn verify_bundle_canonical_multisig_template_id_search_ok() {
    let cos = &[(SEED_A, 0u32), (SEED_B, 0u32)];
    let md1 = emit_template_md1("wsh-sortedmulti", "2", cos);
    let stubs = emit_template_mk1_stubs("wsh-sortedmulti", "2", cos);
    let id = emit_template_wallet_id("wsh-sortedmulti", "2", cos);
    let mk1_b = emit_cosigner_mk1("wsh-sortedmulti", "2", cos, 1);
    let golden = golden_addresses("wsh-sortedmulti", 2, cos, true, 1);

    let mut args = vec!["verify-bundle".into(), "--network".into(), "mainnet".into()];
    push_md1(&mut args, &md1);
    push_mk1_stubs(&mut args, &stubs);
    args.extend([
        "--from".into(),
        format!("phrase={SEED_A}"),
        "--account".into(),
        "0".into(),
        "--expect-wallet-id".into(),
        id.clone(),
        "--json".into(),
    ]);
    push_cosigners(&mut args, &[mk1_b.clone()]);

    let j = verify_json(&args);
    assert_eq!(j["result"], "ok", "verify-bundle envelope: {j}");
    assert_eq!(
        j["first_receive"].as_str().unwrap(),
        golden[0],
        "recomposed first address must equal the independent golden"
    );
    // binding checks present + passing.
    let checks = j["checks"].as_array().unwrap();
    let by = |name: &str| -> bool {
        checks
            .iter()
            .find(|c| c["name"] == name)
            .unwrap_or_else(|| panic!("missing check {name}: {j}"))["passed"]
            .as_bool()
            .unwrap()
    };
    assert!(
        by("md1_template_match"),
        "md1_template_match must pass: {j}"
    );
    assert!(
        by("mk1_template_stub_bind"),
        "mk1_template_stub_bind must pass: {j}"
    );

    // PARITY: restore the SAME inputs and compare the first address.
    let mut rargs = vec!["restore".into(), "--network".into(), "mainnet".into()];
    push_md1(&mut rargs, &md1);
    rargs.extend([
        "--from".into(),
        format!("phrase={SEED_A}"),
        "--account".into(),
        "0".into(),
        "--expect-wallet-id".into(),
        id,
        "--json".into(),
    ]);
    push_cosigners(&mut rargs, &[mk1_b]);
    let restore_addr = restore_first_address(&rargs);
    assert_eq!(
        j["first_receive"].as_str().unwrap(),
        restore_addr,
        "verify-bundle and restore must report the SAME first address (funds-safety parity)"
    );
}

// ===========================================================================
// 1b. (P4 R0 M-1 fold) ORDER-DEPENDENT wsh-multi template, id-search → recompose
//     == golden + restore PARITY. The other id-search happy-path
//     (`..._id_search_ok`) uses wsh-SORTEDMULTI (order-independent); restore's
//     suite covers wsh-multi positively but verify only had it in the negative
//     cross-mix. This closes the verify-side parity matrix symmetrically: a
//     wsh-multi id-search must resolve the UNIQUE permutation that reproduces the
//     recorded id, and verify must report the SAME address restore does.
// ===========================================================================

#[test]
fn verify_bundle_canonical_wsh_multi_template_id_search_ok() {
    let cos = &[(SEED_A, 0u32), (SEED_B, 0u32)];
    let md1 = emit_template_md1("wsh-multi", "2", cos);
    let stubs = emit_template_mk1_stubs("wsh-multi", "2", cos);
    let id = emit_template_wallet_id("wsh-multi", "2", cos);
    let mk1_b = emit_cosigner_mk1("wsh-multi", "2", cos, 1);
    // golden built on the wsh-MULTI (order-dependent) tree, in slot order {A,B}.
    let golden = golden_addresses("wsh-multi", 2, cos, false, 1);

    let mut args = vec!["verify-bundle".into(), "--network".into(), "mainnet".into()];
    push_md1(&mut args, &md1);
    push_mk1_stubs(&mut args, &stubs);
    args.extend([
        "--from".into(),
        format!("phrase={SEED_A}"),
        "--account".into(),
        "0".into(),
        "--expect-wallet-id".into(),
        id.clone(),
        "--json".into(),
    ]);
    push_cosigners(&mut args, &[mk1_b.clone()]);

    let j = verify_json(&args);
    assert_eq!(j["result"], "ok", "wsh-multi verify envelope: {j}");
    assert_eq!(
        j["first_receive"].as_str().unwrap(),
        golden[0],
        "wsh-multi recomposed first address must equal the independent golden in the resolved order"
    );
    // binding checks present + passing (the order-dependent shape binds too).
    let checks = j["checks"].as_array().unwrap();
    let by = |name: &str| -> bool {
        checks
            .iter()
            .find(|c| c["name"] == name)
            .unwrap_or_else(|| panic!("missing check {name}: {j}"))["passed"]
            .as_bool()
            .unwrap()
    };
    assert!(
        by("md1_template_match"),
        "md1_template_match must pass: {j}"
    );
    assert!(
        by("mk1_template_stub_bind"),
        "mk1_template_stub_bind must pass: {j}"
    );

    // PARITY: restore the SAME inputs and compare the first address.
    let mut rargs = vec!["restore".into(), "--network".into(), "mainnet".into()];
    push_md1(&mut rargs, &md1);
    rargs.extend([
        "--from".into(),
        format!("phrase={SEED_A}"),
        "--account".into(),
        "0".into(),
        "--expect-wallet-id".into(),
        id,
        "--json".into(),
    ]);
    push_cosigners(&mut rargs, &[mk1_b]);
    let restore_addr = restore_first_address(&rargs);
    assert_eq!(
        j["first_receive"].as_str().unwrap(),
        restore_addr,
        "wsh-multi verify-bundle and restore must report the SAME first address (funds-safety parity)"
    );
}

// ===========================================================================
// 3. address-search parity with restore.
// ===========================================================================

#[test]
fn verify_bundle_multisig_template_address_search_ok() {
    let cos = &[(SEED_A, 0u32), (SEED_B, 0u32)];
    let md1 = emit_template_md1("wsh-sortedmulti", "2", cos);
    let stubs = emit_template_mk1_stubs("wsh-sortedmulti", "2", cos);
    let mk1_b = emit_cosigner_mk1("wsh-sortedmulti", "2", cos, 1);
    let golden = golden_addresses("wsh-sortedmulti", 2, cos, true, 1);

    let mut args = vec!["verify-bundle".into(), "--network".into(), "mainnet".into()];
    push_md1(&mut args, &md1);
    push_mk1_stubs(&mut args, &stubs);
    args.extend([
        "--from".into(),
        format!("phrase={SEED_A}"),
        "--account".into(),
        "0".into(),
        "--search-address".into(),
        golden[0].clone(),
        "--json".into(),
    ]);
    push_cosigners(&mut args, &[mk1_b]);

    let j = verify_json(&args);
    assert_eq!(j["result"], "ok", "address-search envelope: {j}");
    assert_eq!(
        j["first_receive"].as_str().unwrap(),
        golden[0],
        "address-search recompose must match the golden"
    );
}

// ===========================================================================
// 4. no --from → refuse, naming --from.
// ===========================================================================

#[test]
fn verify_bundle_multisig_template_no_from_refuses() {
    let cos = &[(SEED_A, 0u32), (SEED_B, 0u32)];
    let md1 = emit_template_md1("wsh-sortedmulti", "2", cos);
    let stubs = emit_template_mk1_stubs("wsh-sortedmulti", "2", cos);

    let mut args = vec!["verify-bundle".into(), "--network".into(), "mainnet".into()];
    push_md1(&mut args, &md1);
    push_mk1_stubs(&mut args, &stubs);
    // no --from
    let assert = mnemonic().args(&args).assert().failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("--from"),
        "no-seed multisig template verify must name --from: {stderr}"
    );
    assert!(
        !stderr.contains("template-only"),
        "the refusal must be the completion floor, not the pre-P4 routing reject: {stderr}"
    );
}

// ===========================================================================
// 5. wrong/outsider cosigner → NO-MATCH (exit 4), never a silent OK.
// ===========================================================================

#[test]
fn verify_bundle_multisig_template_wrong_cosigner_no_match() {
    // Recorded id is for {A,B}; supply an OUTSIDER cosigner → no assignment
    // reproduces the id → mismatch/refuse (exit 4), NEVER a silent OK.
    let cos = &[(SEED_A, 0u32), (SEED_B, 0u32)];
    let md1 = emit_template_md1("wsh-sortedmulti", "2", cos);
    let stubs = emit_template_mk1_stubs("wsh-sortedmulti", "2", cos);
    let id = emit_template_wallet_id("wsh-sortedmulti", "2", cos);
    // an outsider mk1 at @1's origin (wrong key).
    let cos_bad = &[(SEED_A, 0u32), (SEED_OUTSIDER, 0u32)];
    let mk1_outsider = emit_cosigner_mk1("wsh-sortedmulti", "2", cos_bad, 1);

    let mut args = vec!["verify-bundle".into(), "--network".into(), "mainnet".into()];
    push_md1(&mut args, &md1);
    push_mk1_stubs(&mut args, &stubs);
    args.extend([
        "--from".into(),
        format!("phrase={SEED_A}"),
        "--account".into(),
        "0".into(),
        "--expect-wallet-id".into(),
        id,
    ]);
    push_cosigners(&mut args, &[mk1_outsider]);
    let assert = mnemonic().args(&args).assert();
    let out = assert.get_output();
    assert_ne!(
        out.status.code(),
        Some(0),
        "a wrong cosigner must never produce a silent OK"
    );
    let stdout = String::from_utf8(out.stdout.clone()).unwrap();
    assert!(
        !stdout.contains("\"result\":\"ok\"") && !stdout.contains("\"result\": \"ok\""),
        "no OK result for an outsider cosigner: {stdout}"
    );
}

// ===========================================================================
// 6. cross-mix binding: a md1/mk1 of a DIFFERENT template SHAPE fails the
//    template-id binding (the binding gate is non-vacuous).
//
// NB: the `WalletDescriptorTemplateId` is STRUCTURAL — a wsh-sortedmulti 2-of-2
// has the same template-id regardless of WHICH cosigner keys are in it. So
// supplying a {A,C} md1 + {A,B} keys would (correctly) bind + complete to the
// {A,B} wallet (the cosigner identity is gated by the completion SEARCH, not the
// shape stub). A genuine binding failure is a DIFFERENT shape: here a `wsh-multi`
// (order-DEPENDENT) md1 + a `wsh-sortedmulti` recorded id + the {A,B} keys — the
// recomposed wallet's template-id (built on the wsh-multi tree) must NOT match...
// actually completion builds on the SUPPLIED md1's tree, so to break BINDING we
// supply an md1 whose template the recorded id can never satisfy: a 2-of-2
// `wsh-multi` md1 but search for the `wsh-sortedmulti` id of {A,B}. The search
// builds wsh-multi candidates whose id ≠ the sortedmulti id → NO-MATCH (exit 4),
// never a silent OK. (A successful completion ALWAYS shares the template-id with
// the supplied md1 by construction, so md1_template_match cannot fail on a
// completed wallet — the funds-safety gate is the completion search itself.)
// ===========================================================================

#[test]
fn verify_bundle_multisig_template_binding_cross_mix_fails() {
    let cos = &[(SEED_A, 0u32), (SEED_B, 0u32)];
    // a wsh-MULTI (order-dependent) md1 + stubs.
    let md1_multi = emit_template_md1("wsh-multi", "2", cos);
    let stubs_multi = emit_template_mk1_stubs("wsh-multi", "2", cos);
    // but a recorded id for the wsh-SORTEDMULTI {A,B} wallet (a different shape).
    let id_sorted = emit_template_wallet_id("wsh-sortedmulti", "2", cos);
    let mk1_b = emit_cosigner_mk1("wsh-multi", "2", cos, 1);

    let mut args = vec!["verify-bundle".into(), "--network".into(), "mainnet".into()];
    push_md1(&mut args, &md1_multi);
    push_mk1_stubs(&mut args, &stubs_multi);
    args.extend([
        "--from".into(),
        format!("phrase={SEED_A}"),
        "--account".into(),
        "0".into(),
        "--expect-wallet-id".into(),
        id_sorted,
        "--json".into(),
    ]);
    push_cosigners(&mut args, &[mk1_b]);

    // The wsh-multi tree can never reproduce the wsh-sortedmulti id → NO-MATCH
    // (exit 4), never a silent OK.
    let assert = mnemonic().args(&args).assert();
    let out = assert.get_output();
    assert_ne!(
        out.status.code(),
        Some(0),
        "a cross-shape bundle must never verify OK"
    );
    let stdout = String::from_utf8(out.stdout.clone()).unwrap();
    assert!(
        !stdout.contains("\"result\":\"ok\"") && !stdout.contains("\"result\": \"ok\""),
        "no OK result for a cross-shape mix: {stdout}"
    );
}

// ===========================================================================
// 2. GENERAL policy (degrade2-class) template id-search → completes + verifies.
//    Shape: wsh(or_i(pk(@0),and_v(v:pk(@1),pk(@2)))) at BIP-84 origins.
// ===========================================================================

fn bip84_origin(account: u32) -> String {
    format!("84'/0'/{account}'")
}

/// Build the general-policy descriptor string for the P3b shape.
fn general_desc(slots: &[(&str, u32)]) -> String {
    assert_eq!(slots.len(), 3, "the general shape has exactly 3 keys");
    let mut keys: Vec<String> = Vec::new();
    for (phrase, account) in slots {
        let path = bip84_origin(*account);
        let (xpub, fp) = xpub_at(phrase, &path);
        let origin = path.replace('\'', "h");
        keys.push(format!("[{fp}/{origin}]{xpub}/<0;1>/*"));
    }
    format!(
        "wsh(or_i(pk({}),and_v(v:pk({}),pk({}))))",
        keys[0], keys[1], keys[2]
    )
}

fn emit_general_bundle(form: &str, desc: &str) -> String {
    let out = mnemonic()
        .args([
            "bundle",
            "--network",
            "mainnet",
            "--md1-form",
            form,
            "--group-size",
            "0",
            "--no-engraving-card",
            "--descriptor",
            desc,
        ])
        .assert()
        .success();
    String::from_utf8(out.get_output().stdout.clone()).unwrap()
}

fn emit_general_template_md1(desc: &str) -> Vec<String> {
    md1_lines(&emit_general_bundle("template", desc))
}

fn emit_general_template_mk1_stubs(desc: &str) -> Vec<Vec<String>> {
    mk1_groups(&emit_general_bundle("template", desc))
}

fn emit_general_template_wallet_id(desc: &str) -> String {
    let out = mnemonic()
        .args([
            "bundle",
            "--network",
            "mainnet",
            "--md1-form",
            "template",
            "--group-size",
            "0",
            "--no-engraving-card",
            "--descriptor",
            desc,
        ])
        .assert()
        .success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    let line = stderr
        .lines()
        .find(|l| l.contains("wallet-id (hex)"))
        .unwrap_or_else(|| panic!("no wallet-id (hex) line in: {stderr}"));
    line.split(':').next_back().unwrap().trim().to_string()
}

fn emit_general_cosigner_mk1(desc: &str, which: usize) -> Vec<String> {
    let groups = mk1_groups(&emit_general_bundle("policy", desc));
    groups
        .get(which)
        .unwrap_or_else(|| panic!("no general mk1 group {which}"))
        .clone()
}

fn general_golden_addresses(desc: &str, count: u32) -> Vec<String> {
    let parsed = Descriptor::<DescriptorPublicKey>::from_str(desc)
        .unwrap_or_else(|e| panic!("golden general descriptor parse {desc}: {e}"));
    let receive = parsed.into_single_descriptors().unwrap().remove(0);
    (0..count)
        .map(|i| {
            receive
                .derive_at_index(i)
                .unwrap()
                .address(bitcoin::Network::Bitcoin)
                .unwrap()
                .to_string()
        })
        .collect()
}

// ===========================================================================
// P4 — verify-bundle exposes the own-account subset-search flags
// (`--own-account-max` + `--search-cosigner-subset`), threaded into the SAME
// shared `complete_multisig_template` engine restore uses. The make-or-break
// gate is PARITY: the recomposed first address verify-bundle reports ==
// restore's == an INDEPENDENT rust-miniscript golden (verify == restore, same
// engine). The flag refusals mirror restore's (clap conflict, @N= mutex, the
// §6 ceiling, wrong-cosigner NO-MATCH).
// ===========================================================================

/// Build the restore parity arg-vector matching a verify-bundle subset-search
/// invocation: same md1, --from, the subset-search flags, --expect-wallet-id,
/// and cosigners. (verify-bundle's `--account` is scalar; restore's is a list,
/// so the over-supply tests use `--own-account-max` on BOTH — no `--account`.)
/// `extra_flags` carries the subset-search flag(s) (`--own-account-max K` and/or
/// `--search-cosigner-subset`) so restore drives the SAME engine path.
fn restore_parity_args(
    md1: &[String],
    from_phrase: &str,
    extra_flags: &[String],
    id: &str,
    cosigners: &[Vec<String>],
) -> Vec<String> {
    let mut rargs = vec!["restore".into(), "--network".into(), "mainnet".into()];
    push_md1(&mut rargs, md1);
    rargs.push("--from".into());
    rargs.push(format!("phrase={from_phrase}"));
    rargs.extend(extra_flags.iter().cloned());
    rargs.extend(["--expect-wallet-id".into(), id.to_string(), "--json".into()]);
    push_cosigners(&mut rargs, cosigners);
    rargs
}

#[test]
fn verify_bundle_own_account_max_completes_at_nonzero_account() {
    // P4 HEADLINE: a 2-of-2 where the operator's OWN key lives at account 3
    // (NOT 0). verify-bundle with `--own-account-max 5` (NEW on verify-bundle)
    // derives own@{0..4}, the subset-search selects own@3 + the cosigner, and
    // the recomposed first address == the INDEPENDENT golden. PARITY: restore
    // with the SAME inputs reports the SAME address.
    let cos = &[(SEED_A, 3u32), (SEED_B, 0u32)];
    let md1 = emit_template_md1("wsh-multi", "2", cos);
    let stubs = emit_template_mk1_stubs("wsh-multi", "2", cos);
    let id = emit_template_wallet_id("wsh-multi", "2", cos);
    let mk1_b = emit_cosigner_mk1("wsh-multi", "2", cos, 1);
    let golden = golden_addresses("wsh-multi", 2, cos, false, 1);

    let mut args = vec!["verify-bundle".into(), "--network".into(), "mainnet".into()];
    push_md1(&mut args, &md1);
    push_mk1_stubs(&mut args, &stubs);
    args.extend([
        "--from".into(),
        format!("phrase={SEED_A}"),
        "--own-account-max".into(),
        "5".into(),
        "--expect-wallet-id".into(),
        id.clone(),
        "--json".into(),
    ]);
    push_cosigners(&mut args, &[mk1_b.clone()]);

    let j = verify_json(&args);
    assert_eq!(j["result"], "ok", "own-account-max verify envelope: {j}");
    assert_eq!(
        j["first_receive"].as_str().unwrap(),
        golden[0],
        "verify-bundle --own-account-max must resolve a NON-ZERO own account to the golden"
    );
    let checks = j["checks"].as_array().unwrap();
    assert!(
        checks
            .iter()
            .any(|c| c["name"] == "md1_template_match" && c["passed"] == true),
        "md1_template_match must pass: {j}"
    );

    // PARITY: restore the SAME inputs (own-only over-supply) → SAME address.
    let extra = vec!["--own-account-max".to_string(), "5".to_string()];
    let rargs = restore_parity_args(&md1, SEED_A, &extra, &id, &[mk1_b]);
    let restore_addr = restore_first_address(&rargs);
    assert_eq!(
        j["first_receive"].as_str().unwrap(),
        restore_addr,
        "verify-bundle and restore must report the SAME first address over the subset-search \
         (verify == restore, same engine)"
    );
}

#[test]
fn verify_bundle_search_cosigner_subset_completes() {
    // P4: verify-bundle with `--search-cosigner-subset` (NEW on verify-bundle) +
    // an OVER-SUPPLIED cosigner pool (the 2 real cosigners + 1 outsider) must
    // select the correct {B,C} subset and recompose to the INDEPENDENT golden,
    // in PARITY with restore P3.
    let cos = &[(SEED_A, 0u32), (SEED_B, 0u32), (SEED_C, 0u32)];
    let md1 = emit_template_md1("wsh-multi", "2", cos);
    let stubs = emit_template_mk1_stubs("wsh-multi", "2", cos);
    let id = emit_template_wallet_id("wsh-multi", "2", cos);
    let golden = golden_addresses("wsh-multi", 2, cos, false, 1);
    let mk1_b = emit_cosigner_mk1("wsh-multi", "2", cos, 1);
    let mk1_c = emit_cosigner_mk1("wsh-multi", "2", cos, 2);
    let cos_outsider = &[(SEED_A, 0u32), (SEED_OUTSIDER, 0u32)];
    let mk1_outsider = emit_cosigner_mk1("wsh-multi", "2", cos_outsider, 1);

    let mut args = vec!["verify-bundle".into(), "--network".into(), "mainnet".into()];
    push_md1(&mut args, &md1);
    push_mk1_stubs(&mut args, &stubs);
    args.extend([
        "--from".into(),
        format!("phrase={SEED_A}"),
        "--search-cosigner-subset".into(),
        "--expect-wallet-id".into(),
        id.clone(),
        "--json".into(),
    ]);
    push_cosigners(
        &mut args,
        &[mk1_b.clone(), mk1_c.clone(), mk1_outsider.clone()],
    );

    let j = verify_json(&args);
    assert_eq!(
        j["result"], "ok",
        "search-cosigner-subset verify envelope: {j}"
    );
    assert_eq!(
        j["first_receive"].as_str().unwrap(),
        golden[0],
        "verify-bundle --search-cosigner-subset must select the correct cosigner subset and \
         recompose to the golden"
    );

    // PARITY: restore with the SAME over-supplied pool reports the SAME address.
    let extra = vec!["--search-cosigner-subset".to_string()];
    let rargs = restore_parity_args(&md1, SEED_A, &extra, &id, &[mk1_b, mk1_c, mk1_outsider]);
    let restore_addr = restore_first_address(&rargs);
    assert_eq!(
        j["first_receive"].as_str().unwrap(),
        restore_addr,
        "verify-bundle --search-cosigner-subset must agree with restore (same engine)"
    );
}

#[test]
fn verify_bundle_own_account_max_alone_passes() {
    // I-4 regression guard on verify-bundle: `--own-account-max K` ALONE (no
    // explicit --account) must PASS clap (the scalar `--account` default_value
    // must NOT trip the `conflicts_with` mutex). It completes a real own@0 2-of-2.
    let cos = &[(SEED_A, 0u32), (SEED_B, 0u32)];
    let md1 = emit_template_md1("wsh-multi", "2", cos);
    let stubs = emit_template_mk1_stubs("wsh-multi", "2", cos);
    let id = emit_template_wallet_id("wsh-multi", "2", cos);
    let golden = golden_addresses("wsh-multi", 2, cos, false, 1);
    let mk1_b = emit_cosigner_mk1("wsh-multi", "2", cos, 1);

    let mut args = vec!["verify-bundle".into(), "--network".into(), "mainnet".into()];
    push_md1(&mut args, &md1);
    push_mk1_stubs(&mut args, &stubs);
    args.extend([
        "--from".into(),
        format!("phrase={SEED_A}"),
        "--own-account-max".into(),
        "3".into(),
        "--expect-wallet-id".into(),
        id,
        "--json".into(),
    ]);
    push_cosigners(&mut args, &[mk1_b]);

    let j = verify_json(&args);
    assert_eq!(j["result"], "ok", "own-account-max-alone verify: {j}");
    assert_eq!(
        j["first_receive"].as_str().unwrap(),
        golden[0],
        "--own-account-max alone (no --account) must pass clap and complete on verify-bundle"
    );
}

#[test]
fn verify_bundle_account_and_own_account_max_conflict() {
    // I-4: `--account N` + `--own-account-max K` together on verify-bundle → clap
    // `conflicts_with` parse error (exit 64), BEFORE any work. (`--account` is
    // SCALAR on verify-bundle — the conflict is the same.)
    let cos = &[(SEED_A, 0u32), (SEED_B, 0u32)];
    let md1 = emit_template_md1("wsh-multi", "2", cos);
    let stubs = emit_template_mk1_stubs("wsh-multi", "2", cos);

    let mut args = vec!["verify-bundle".into(), "--network".into(), "mainnet".into()];
    push_md1(&mut args, &md1);
    push_mk1_stubs(&mut args, &stubs);
    args.extend([
        "--from".into(),
        format!("phrase={SEED_A}"),
        "--account".into(),
        "0".into(),
        "--own-account-max".into(),
        "5".into(),
        "--expect-wallet-id".into(),
        "deadbeef".into(),
    ]);
    let assert = mnemonic().args(&args).assert().code(64);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let low = stderr.to_lowercase();
    assert!(
        low.contains("cannot be used with") || low.contains("conflict"),
        "--account + --own-account-max must be a clap conflict on verify-bundle: {stderr}"
    );
}

#[test]
fn verify_bundle_own_account_max_at_account_mutex_with_explicit_cosigner_assignment() {
    // §2 open-point 7 (mirrored on verify-bundle): explicit `--cosigner @N=`
    // assignment ⊕ subset-search (`--own-account-max`) → BadInput.
    let cos = &[(SEED_A, 0u32), (SEED_B, 0u32)];
    let md1 = emit_template_md1("wsh-multi", "2", cos);
    let stubs = emit_template_mk1_stubs("wsh-multi", "2", cos);
    let path = canonical_path("wsh-multi", 0);
    let (xpub_b, _fp_b) = xpub_at(SEED_B, &path);

    let mut args = vec!["verify-bundle".into(), "--network".into(), "mainnet".into()];
    push_md1(&mut args, &md1);
    push_mk1_stubs(&mut args, &stubs);
    args.extend([
        "--from".into(),
        format!("phrase={SEED_A}"),
        "--own-account-max".into(),
        "3".into(),
        "--cosigner".into(),
        format!("@1={xpub_b}"),
    ]);
    let assert = mnemonic().args(&args).assert().failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let low = stderr.to_lowercase();
    assert!(
        low.contains("@n=") || low.contains("explicit") || low.contains("subset-search"),
        "explicit @N= ⊕ --own-account-max must be a BadInput mutex on verify-bundle: {stderr}"
    );
}

#[test]
fn verify_bundle_own_account_max_ceiling_refuses() {
    // §6 (mirrored on verify-bundle): K_own > 256 → BadInput (hard ceiling).
    let cos = &[(SEED_A, 0u32), (SEED_B, 0u32)];
    let md1 = emit_template_md1("wsh-multi", "2", cos);
    let stubs = emit_template_mk1_stubs("wsh-multi", "2", cos);
    let id = emit_template_wallet_id("wsh-multi", "2", cos);
    let mk1_b = emit_cosigner_mk1("wsh-multi", "2", cos, 1);

    let mut args = vec!["verify-bundle".into(), "--network".into(), "mainnet".into()];
    push_md1(&mut args, &md1);
    push_mk1_stubs(&mut args, &stubs);
    args.extend([
        "--from".into(),
        format!("phrase={SEED_A}"),
        "--own-account-max".into(),
        "300".into(),
        "--expect-wallet-id".into(),
        id,
    ]);
    push_cosigners(&mut args, &[mk1_b]);
    let assert = mnemonic().args(&args).assert().failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let low = stderr.to_lowercase();
    assert!(
        low.contains("256") || low.contains("own-account-max"),
        "K_own > 256 must refuse via the hard ceiling on verify-bundle: {stderr}"
    );
}

#[test]
fn verify_bundle_search_cosigner_subset_wrong_cosigner_no_match() {
    // §5a anti-vacuity (mirrored on verify-bundle): an over-supplied pool that
    // LACKS a true cosigner → NO-MATCH (exit ≠ 0), never a silent OK.
    let cos = &[(SEED_A, 0u32), (SEED_B, 0u32), (SEED_C, 0u32)];
    let md1 = emit_template_md1("wsh-multi", "2", cos);
    let stubs = emit_template_mk1_stubs("wsh-multi", "2", cos);
    let id = emit_template_wallet_id("wsh-multi", "2", cos);
    let mk1_b = emit_cosigner_mk1("wsh-multi", "2", cos, 1);
    // TWO outsider cards (C is missing) → the {B,C} subset is unreachable.
    let cos_outsider = &[(SEED_A, 0u32), (SEED_OUTSIDER, 0u32)];
    let mk1_outsider1 = emit_cosigner_mk1("wsh-multi", "2", cos_outsider, 1);
    let cos_outsider2 = &[(SEED_A, 0u32), (SEED_C, 1u32)]; // C at a WRONG account
    let mk1_outsider2 = emit_cosigner_mk1("wsh-multi", "2", cos_outsider2, 1);

    let mut args = vec!["verify-bundle".into(), "--network".into(), "mainnet".into()];
    push_md1(&mut args, &md1);
    push_mk1_stubs(&mut args, &stubs);
    args.extend([
        "--from".into(),
        format!("phrase={SEED_A}"),
        "--search-cosigner-subset".into(),
        "--expect-wallet-id".into(),
        id,
    ]);
    push_cosigners(&mut args, &[mk1_b, mk1_outsider1, mk1_outsider2]);
    let assert = mnemonic().args(&args).assert();
    let out = assert.get_output();
    assert_ne!(
        out.status.code(),
        Some(0),
        "a pool lacking a true cosigner must never produce a silent OK on verify-bundle"
    );
    let stdout = String::from_utf8(out.stdout.clone()).unwrap();
    assert!(
        !stdout.contains("\"result\":\"ok\"") && !stdout.contains("\"result\": \"ok\""),
        "no OK result for a pool lacking a true cosigner: {stdout}"
    );
}

#[test]
fn verify_bundle_general_policy_template_id_search_ok() {
    let slots = &[(SEED_A, 0u32), (SEED_B, 1u32), (SEED_C, 2u32)];
    let desc = general_desc(slots);
    let md1 = emit_general_template_md1(&desc);
    // sanity: genuinely general + keyless.
    let md1_refs: Vec<&str> = md1.iter().map(|s| s.as_str()).collect();
    let decoded = md_codec::chunk::reassemble(&md1_refs).expect("general template decodes");
    assert!(
        md_codec::canonical_origin::canonical_origin(&decoded.tree).is_none(),
        "the shape MUST be non-canonical (general)"
    );
    assert!(!decoded.is_wallet_policy());
    assert_eq!(decoded.n, 3);

    let stubs = emit_general_template_mk1_stubs(&desc);
    let id = emit_general_template_wallet_id(&desc);
    let golden = general_golden_addresses(&desc, 1);

    let mut args = vec!["verify-bundle".into(), "--network".into(), "mainnet".into()];
    push_md1(&mut args, &md1);
    push_mk1_stubs(&mut args, &stubs);
    args.extend([
        "--from".into(),
        format!("phrase={SEED_A}"),
        "--account".into(),
        "0".into(),
        "--expect-wallet-id".into(),
        id,
        "--json".into(),
    ]);
    // the @1 (B) + @2 (C) cosigners (unassigned) carrying real BIP-84 origins.
    let mk1_b = emit_general_cosigner_mk1(&desc, 1);
    let mk1_c = emit_general_cosigner_mk1(&desc, 2);
    push_cosigners(&mut args, &[mk1_b, mk1_c]);

    let j = verify_json(&args);
    assert_eq!(j["result"], "ok", "general-policy verify envelope: {j}");
    assert_eq!(
        j["first_receive"].as_str().unwrap(),
        golden[0],
        "general-policy recompose must match the independent golden"
    );
    let checks = j["checks"].as_array().unwrap();
    assert!(
        checks
            .iter()
            .any(|c| c["name"] == "md1_template_match" && c["passed"] == true),
        "general md1_template_match must pass: {j}"
    );
}
