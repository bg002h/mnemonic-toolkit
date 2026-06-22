//! #28 phase 2 — `mnemonic restore --md1 <keyless-multisig-template>` COMPLETION
//! integration tests (P3a — CANONICAL multisig: wsh(multi/sortedmulti),
//! sh(wsh(...))).
//!
//! Funds-safety / silent-wrong-wallet class. The make-or-break gates are NOT
//! exit-0; they are:
//!   - the completed addresses == an INDEPENDENT rust-miniscript golden (the
//!     wallet derived directly from the cosigner account xpubs at the canonical
//!     BIP-48 origin — NOT md-codec reconstruction);
//!   - the §7 floors (no `--from`, an unsupplied slot, a swapped `@N`, a
//!     duplicate cosigner key, a too-weak `--expect-wallet-id` prefix all
//!     REFUSE);
//!   - the search resolves the UNIQUE `@N`→key assignment (id-search +
//!     address-search), and a wrong assignment NO-MATCHES.
//!
//! The template md1 is emitted by `bundle --md1-form=template`; the cosigner
//! cards are emitted by `bundle --md1-form=policy` (per-cosigner mk1) so the
//! completion reads REAL mk1 origin paths.

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
// An outsider seed not part of the wallet (for the "supplied seed is not a
// cosigner" floor).
const SEED_OUTSIDER: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

fn mnemonic() -> Command {
    Command::cargo_bin("mnemonic").expect("mnemonic binary builds")
}

/// Extract md1 string(s) from `bundle` text stdout (lines under `# md1`).
fn md1_lines(stdout: &str) -> Vec<String> {
    section_lines(stdout, "# md1")
}

/// Extract the per-cosigner mk1 string(s) from `bundle` text stdout. The
/// multisig emit prints `# mk1[i] (...)` headers; collect every `mk1`-prefixed
/// line grouped per header.
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
            } else if t.is_empty() {
                // header section break
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

/// Derive a mainnet account xpub + master fingerprint at `path_str` from a
/// BIP-39 phrase.
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

/// Emit a keyless multisig template md1 for N cosigners (each at the canonical
/// origin for `script`/`account`). Returns the md1 chunk(s).
fn emit_template_md1(script: &str, threshold: &str, cosigners: &[(&str, u32)]) -> Vec<String> {
    let mut args: Vec<String> = vec![
        "bundle".into(),
        "--network".into(),
        "mainnet".into(),
        "--template".into(),
        script.into(),
        "--threshold".into(),
        threshold.into(),
        "--md1-form".into(),
        "template".into(),
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
    let out = mnemonic().args(&args).assert().success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    md1_lines(&stdout)
}

/// The printed WalletPolicyId (full hex) the template emit advisory records.
fn emit_template_wallet_id(script: &str, threshold: &str, cosigners: &[(&str, u32)]) -> String {
    let mut args: Vec<String> = vec![
        "bundle".into(),
        "--network".into(),
        "mainnet".into(),
        "--template".into(),
        script.into(),
        "--threshold".into(),
        threshold.into(),
        "--md1-form".into(),
        "template".into(),
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
    let out = mnemonic().args(&args).assert().success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    let line = stderr
        .lines()
        .find(|l| l.contains("wallet-id (hex)"))
        .unwrap_or_else(|| panic!("no wallet-id (hex) line in: {stderr}"));
    line.split(':').next_back().unwrap().trim().to_string()
}

/// Emit a single cosigner mk1 card (multisig policy form) at the canonical
/// origin so the completion can read its REAL origin path.
fn emit_cosigner_mk1(
    script: &str,
    threshold: &str,
    cosigners: &[(&str, u32)],
    which: usize,
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
        "policy".into(),
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
    let out = mnemonic().args(&args).assert().success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let groups = mk1_groups(&stdout);
    groups
        .get(which)
        .unwrap_or_else(|| panic!("no mk1 group {which} in: {stdout}"))
        .clone()
}

/// INDEPENDENT golden: build the watch-only wsh/sh-wsh multisig descriptor
/// directly from the cosigner account xpubs at the canonical BIP-48 origin
/// (rust-miniscript, NOT md-codec reconstruction), and derive the first
/// `count` receive addresses.
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
        // origin = [fp/48'/0'/account'/{2'|1'}], xkey then /<0;1>/* multipath.
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

/// Helper to push `--md1 <chunk>` for every chunk.
fn push_md1(args: &mut Vec<String>, md1: &[String]) {
    for c in md1 {
        args.push("--md1".into());
        args.push(c.clone());
    }
}

// ===========================================================================
// id-search: the unique assignment → completed addresses == independent golden.
// ===========================================================================

#[test]
fn wsh_sortedmulti_id_search_completes_to_golden() {
    let cos = &[(SEED_A, 0u32), (SEED_B, 0u32)];
    let md1 = emit_template_md1("wsh-sortedmulti", "2", cos);
    let id = emit_template_wallet_id("wsh-sortedmulti", "2", cos);
    let mk1_b = emit_cosigner_mk1("wsh-sortedmulti", "2", cos, 1);

    let mut args = vec!["restore".into(), "--network".into(), "mainnet".into()];
    push_md1(&mut args, &md1);
    args.extend([
        "--from".into(),
        format!("phrase={SEED_A}"),
        "--account".into(),
        "0".into(),
        "--expect-wallet-id".into(),
        id,
        "--count".into(),
        "3".into(),
        "--json".into(),
    ]);
    // cosigner B supplied unassigned (mk1) — the search places it.
    for c in &mk1_b {
        args.push("--cosigner".into());
        args.push(c.clone());
    }
    let got = restore_addresses(&args);
    let golden = golden_addresses("wsh-sortedmulti", 2, cos, true, 3);
    assert_eq!(
        got, golden,
        "id-search completion must match the independent golden"
    );
}

#[test]
fn wsh_multi_id_search_order_dependent_completes_to_golden() {
    // multi (order-DEPENDENT): the search must find the UNIQUE permutation that
    // reproduces the recorded id; the completed addresses match the golden in
    // that exact slot order.
    let cos = &[(SEED_A, 0u32), (SEED_B, 0u32)];
    let md1 = emit_template_md1("wsh-multi", "2", cos);
    let id = emit_template_wallet_id("wsh-multi", "2", cos);
    let mk1_b = emit_cosigner_mk1("wsh-multi", "2", cos, 1);

    let mut args = vec!["restore".into(), "--network".into(), "mainnet".into()];
    push_md1(&mut args, &md1);
    args.extend([
        "--from".into(),
        format!("phrase={SEED_A}"),
        "--account".into(),
        "0".into(),
        "--expect-wallet-id".into(),
        id,
        "--count".into(),
        "2".into(),
        "--json".into(),
    ]);
    for c in &mk1_b {
        args.push("--cosigner".into());
        args.push(c.clone());
    }
    let got = restore_addresses(&args);
    let golden = golden_addresses("wsh-multi", 2, cos, false, 2);
    assert_eq!(
        got, golden,
        "wsh-multi id-search must match the golden in the resolved order"
    );
}

#[test]
fn sh_wsh_multi_id_search_completes_to_golden() {
    let cos = &[(SEED_A, 0u32), (SEED_B, 0u32)];
    let md1 = emit_template_md1("sh-wsh-multi", "2", cos);
    let id = emit_template_wallet_id("sh-wsh-multi", "2", cos);
    let mk1_b = emit_cosigner_mk1("sh-wsh-multi", "2", cos, 1);

    let mut args = vec!["restore".into(), "--network".into(), "mainnet".into()];
    push_md1(&mut args, &md1);
    args.extend([
        "--from".into(),
        format!("phrase={SEED_A}"),
        "--account".into(),
        "0".into(),
        "--expect-wallet-id".into(),
        id,
        "--count".into(),
        "2".into(),
        "--json".into(),
    ]);
    for c in &mk1_b {
        args.push("--cosigner".into());
        args.push(c.clone());
    }
    let got = restore_addresses(&args);
    let golden = golden_addresses("sh-wsh-multi", 2, cos, false, 2);
    assert_eq!(
        got, golden,
        "sh(wsh(multi)) id-search must match the golden"
    );
}

// ===========================================================================
// address-search: a known receive address resolves the assignment, incl. a
// non-zero index found within range.
// ===========================================================================

#[test]
fn wsh_sortedmulti_address_search_completes_to_golden() {
    let cos = &[(SEED_A, 0u32), (SEED_B, 0u32)];
    let md1 = emit_template_md1("wsh-sortedmulti", "2", cos);
    let mk1_b = emit_cosigner_mk1("wsh-sortedmulti", "2", cos, 1);
    let golden = golden_addresses("wsh-sortedmulti", 2, cos, true, 3);

    let mut args = vec!["restore".into(), "--network".into(), "mainnet".into()];
    push_md1(&mut args, &md1);
    args.extend([
        "--from".into(),
        format!("phrase={SEED_A}"),
        "--account".into(),
        "0".into(),
        "--search-address".into(),
        golden[0].clone(),
        "--count".into(),
        "3".into(),
        "--json".into(),
    ]);
    for c in &mk1_b {
        args.push("--cosigner".into());
        args.push(c.clone());
    }
    let got = restore_addresses(&args);
    assert_eq!(
        got, golden,
        "address-search completion must match the independent golden"
    );
}

#[test]
fn wsh_multi_address_search_finds_nonzero_index() {
    // A target at index 2 (NOT 0) must be found within the default 0..20 range.
    let cos = &[(SEED_A, 0u32), (SEED_B, 0u32)];
    let md1 = emit_template_md1("wsh-multi", "2", cos);
    let mk1_b = emit_cosigner_mk1("wsh-multi", "2", cos, 1);
    let golden = golden_addresses("wsh-multi", 2, cos, false, 3);

    let mut args = vec!["restore".into(), "--network".into(), "mainnet".into()];
    push_md1(&mut args, &md1);
    args.extend([
        "--from".into(),
        format!("phrase={SEED_A}"),
        "--account".into(),
        "0".into(),
        "--search-address".into(),
        golden[2].clone(), // index 2 target
        "--count".into(),
        "3".into(),
        "--json".into(),
    ]);
    for c in &mk1_b {
        args.push("--cosigner".into());
        args.push(c.clone());
    }
    let got = restore_addresses(&args);
    assert_eq!(
        got, golden,
        "a non-zero-index target must resolve the assignment"
    );
}

#[test]
fn exact_pool_address_search_byte_regression_guard() {
    // I-5 regression guard: the v0.60.0 EXACT-pool (`pool.len()==n`)
    // address-search path passes `early_exit=false` and MUST be byte-unchanged
    // by the P2 early-exit gate. A 2-of-2 {A@0, B@0} resolved EXACTLY (via
    // `--account 0`, no over-supply) → the n! path, full-scan. The completed
    // addresses must equal the independent golden, proving the gate did not
    // perturb the exact path.
    let cos = &[(SEED_A, 0u32), (SEED_B, 0u32)];
    let md1 = emit_template_md1("wsh-multi", "2", cos);
    let golden = golden_addresses("wsh-multi", 2, cos, false, 3);
    let mk1_b = emit_cosigner_mk1("wsh-multi", "2", cos, 1);

    let mut args = vec!["restore".into(), "--network".into(), "mainnet".into()];
    push_md1(&mut args, &md1);
    args.extend([
        "--from".into(),
        format!("phrase={SEED_A}"),
        "--account".into(),
        "0".into(),
        "--search-address".into(),
        golden[0].clone(),
        "--count".into(),
        "3".into(),
        "--json".into(),
    ]);
    for c in &mk1_b {
        args.push("--cosigner".into());
        args.push(c.clone());
    }
    let got = restore_addresses(&args);
    assert_eq!(
        got, golden,
        "the v0.60.0 EXACT-pool address-search path must be byte-unchanged (early_exit=false)"
    );
}

// ===========================================================================
// Floor 1(i): --from REQUIRED — a no-seed multisig template completion refuses.
// ===========================================================================

#[test]
fn floor_no_from_refuses() {
    let cos = &[(SEED_A, 0u32), (SEED_B, 0u32)];
    let md1 = emit_template_md1("wsh-sortedmulti", "2", cos);
    let mut args = vec!["restore".into(), "--network".into(), "mainnet".into()];
    push_md1(&mut args, &md1);
    // no --from
    let assert = mnemonic().args(&args).assert().failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("--from"),
        "no-seed multisig template completion must name --from: {stderr}"
    );
}

// ===========================================================================
// Floor 1(ii): every slot supplied — an unsupplied cosigner refuses.
// ===========================================================================

#[test]
fn floor_unsupplied_slot_refuses() {
    // 2-of-3 wallet; only the own seed (@0) supplied + ONE cosigner — the 3rd
    // slot has no key → refuse (cannot complete).
    let cos = &[(SEED_A, 0u32), (SEED_B, 0u32), (SEED_C, 0u32)];
    let md1 = emit_template_md1("wsh-sortedmulti", "2", cos);
    let id = emit_template_wallet_id("wsh-sortedmulti", "2", cos);
    let mk1_b = emit_cosigner_mk1("wsh-sortedmulti", "2", cos, 1);
    // deliberately DO NOT supply cosigner C (@2).

    let mut args = vec!["restore".into(), "--network".into(), "mainnet".into()];
    push_md1(&mut args, &md1);
    args.extend([
        "--from".into(),
        format!("phrase={SEED_A}"),
        "--account".into(),
        "0".into(),
        "--expect-wallet-id".into(),
        id,
    ]);
    for c in &mk1_b {
        args.push("--cosigner".into());
        args.push(c.clone());
    }
    mnemonic().args(&args).assert().failure();
}

// ===========================================================================
// Floor 1(iii): a swapped @N / wrong key set → no-match → refuse.
// ===========================================================================

#[test]
fn floor_wrong_cosigner_no_match_refuses() {
    // The recorded id is for {A,B}; supply {A, OUTSIDER} → no assignment of the
    // supplied keys reproduces the id → refuse.
    let cos = &[(SEED_A, 0u32), (SEED_B, 0u32)];
    let md1 = emit_template_md1("wsh-sortedmulti", "2", cos);
    let id = emit_template_wallet_id("wsh-sortedmulti", "2", cos);
    // an mk1 for the OUTSIDER at @1's slot position/origin (wrong key).
    let cos_bad = &[(SEED_A, 0u32), (SEED_OUTSIDER, 0u32)];
    let mk1_outsider = emit_cosigner_mk1("wsh-sortedmulti", "2", cos_bad, 1);

    let mut args = vec!["restore".into(), "--network".into(), "mainnet".into()];
    push_md1(&mut args, &md1);
    args.extend([
        "--from".into(),
        format!("phrase={SEED_A}"),
        "--account".into(),
        "0".into(),
        "--expect-wallet-id".into(),
        id,
    ]);
    for c in &mk1_outsider {
        args.push("--cosigner".into());
        args.push(c.clone());
    }
    mnemonic().args(&args).assert().failure();
}

// ===========================================================================
// Floor 2: @0==@1 duplicate cosigner key → refuse (a 2-of-3 secretly 2-of-2).
// ===========================================================================

#[test]
fn floor_duplicate_cosigner_key_refuses() {
    // A valid 2-of-3 {A,B,C} wallet, but at RESTORE the operator supplies the
    // SAME cosigner mk1 (B) for BOTH cosigner slots (a 2-of-3 secretly a
    // 2-of-2). The duplicate-key floor must reject before the search.
    let cos = &[(SEED_A, 0u32), (SEED_B, 0u32), (SEED_C, 0u32)];
    let md1 = emit_template_md1("wsh-sortedmulti", "2", cos);
    let id = emit_template_wallet_id("wsh-sortedmulti", "2", cos);
    let mk1_b = emit_cosigner_mk1("wsh-sortedmulti", "2", cos, 1);

    let mut args = vec!["restore".into(), "--network".into(), "mainnet".into()];
    push_md1(&mut args, &md1);
    args.extend([
        "--from".into(),
        format!("phrase={SEED_A}"),
        "--account".into(),
        "0".into(),
        "--expect-wallet-id".into(),
        id,
    ]);
    // supply the SAME cosigner mk1 TWICE (two unassigned cosigners,
    // byte-identical key) — the duplicate-key floor must fire.
    for c in &mk1_b {
        args.push("--cosigner".into());
        args.push(c.clone());
    }
    for c in &mk1_b {
        args.push("--cosigner".into());
        args.push(c.clone());
    }
    let assert = mnemonic().args(&args).assert().failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.to_lowercase().contains("duplicate") || stderr.to_lowercase().contains("identical"),
        "duplicate cosigner keys must be named: {stderr}"
    );
}

// ===========================================================================
// Floor 5: a too-weak --expect-wallet-id prefix for the multisig SEARCH refuses.
// ===========================================================================

#[test]
fn floor_weak_id_prefix_refuses() {
    // A 4-byte (8-hex) prefix is too weak for an N!-space multisig search.
    let cos = &[(SEED_A, 0u32), (SEED_B, 0u32)];
    let md1 = emit_template_md1("wsh-sortedmulti", "2", cos);
    let id = emit_template_wallet_id("wsh-sortedmulti", "2", cos);
    let weak = id[..8].to_string(); // 4 bytes
    let mk1_b = emit_cosigner_mk1("wsh-sortedmulti", "2", cos, 1);

    let mut args = vec!["restore".into(), "--network".into(), "mainnet".into()];
    push_md1(&mut args, &md1);
    args.extend([
        "--from".into(),
        format!("phrase={SEED_A}"),
        "--account".into(),
        "0".into(),
        "--expect-wallet-id".into(),
        weak,
    ]);
    for c in &mk1_b {
        args.push("--cosigner".into());
        args.push(c.clone());
    }
    let assert = mnemonic().args(&args).assert().failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.to_lowercase().contains("prefix")
            || stderr.to_lowercase().contains("weak")
            || stderr.to_lowercase().contains("bytes"),
        "a too-weak id prefix must be named: {stderr}"
    );
}

// ===========================================================================
// Multi-account own: --account 0,1 resolves the OWN slots at two accounts.
// (degrade2-class scope is P3b; here a canonical multisig where the SAME seed
//  is two cosigners at accounts 0 and 1.)
// ===========================================================================

#[test]
fn multi_account_own_resolves_both_slots() {
    // 2-of-2 where BOTH keys come from SEED_A — one at account 0, one at
    // account 1. `--account 0,1` derives both own keys; the search places them.
    let cos = &[(SEED_A, 0u32), (SEED_A, 1u32)];
    let md1 = emit_template_md1("wsh-multi", "2", cos);
    let id = emit_template_wallet_id("wsh-multi", "2", cos);
    let golden = golden_addresses("wsh-multi", 2, cos, false, 2);

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
    // No --cosigner: both slots are OWN (the seed at accounts 0 and 1).
    let got = restore_addresses(&args);
    assert_eq!(
        got, golden,
        "multi-account own (--account 0,1) must resolve both own slots"
    );
}

// ===========================================================================
// P2 (own-account subset-search): `--own-account-max K` over-supplies the OWN
// candidates (own seed derived at accounts 0..K) and the engine resolves the
// unique own-account→slot assignment over the enlarged pool. This SUPERSEDES
// the I-1 (#28 P3a) "refuse --own-account-max" gate (the genuine subset-search
// has landed). The make-or-break gate is COMPLETION to an INDEPENDENT
// rust-miniscript golden — esp. when the operator's own account is NOT 0.
// ===========================================================================

#[test]
fn own_account_max_completes_at_nonzero_account() {
    // THE HEADLINE GATE. A 2-of-2 where the operator's OWN key lives at account
    // 3 (NOT 0) and the cosigner is SEED_B@0. The operator does not recall their
    // own account → `--own-account-max 5` derives own@{0,1,2,3,4} (5 own
    // candidates) and the subset-search must select own@3 + the cosigner and
    // complete to the INDEPENDENT golden.
    let cos = &[(SEED_A, 3u32), (SEED_B, 0u32)];
    let md1 = emit_template_md1("wsh-multi", "2", cos);
    let id = emit_template_wallet_id("wsh-multi", "2", cos);
    let golden = golden_addresses("wsh-multi", 2, cos, false, 2);
    let mk1_b = emit_cosigner_mk1("wsh-multi", "2", cos, 1);

    let mut args = vec!["restore".into(), "--network".into(), "mainnet".into()];
    push_md1(&mut args, &md1);
    args.extend([
        "--from".into(),
        format!("phrase={SEED_A}"),
        "--own-account-max".into(),
        "5".into(),
        "--expect-wallet-id".into(),
        id,
        "--count".into(),
        "2".into(),
        "--json".into(),
    ]);
    for c in &mk1_b {
        args.push("--cosigner".into());
        args.push(c.clone());
    }
    let got = restore_addresses(&args);
    assert_eq!(
        got, golden,
        "--own-account-max subset-search must resolve a NON-ZERO own account to the golden"
    );
}

#[test]
fn own_account_max_address_search_finds_nonzero_account() {
    // The early-exit (address-search) over-supply path: own@2 of a 2-of-2,
    // resolved via --search-address over the own-anchored space.
    let cos = &[(SEED_A, 2u32), (SEED_B, 0u32)];
    let md1 = emit_template_md1("wsh-sortedmulti", "2", cos);
    let golden = golden_addresses("wsh-sortedmulti", 2, cos, true, 3);
    let mk1_b = emit_cosigner_mk1("wsh-sortedmulti", "2", cos, 1);

    let mut args = vec!["restore".into(), "--network".into(), "mainnet".into()];
    push_md1(&mut args, &md1);
    args.extend([
        "--from".into(),
        format!("phrase={SEED_A}"),
        "--own-account-max".into(),
        "4".into(),
        "--search-address".into(),
        golden[0].clone(),
        "--count".into(),
        "3".into(),
        "--json".into(),
    ]);
    for c in &mk1_b {
        args.push("--cosigner".into());
        args.push(c.clone());
    }
    let got = restore_addresses(&args);
    assert_eq!(
        got, golden,
        "--own-account-max address-search (early-exit) must resolve a non-zero own account"
    );
}

#[test]
fn own_account_max_alone_passes() {
    // I-4 regression guard: `--own-account-max K` ALONE (no explicit --account)
    // must PASS clap parse (the `--account` default_value="0" must NOT trip the
    // `conflicts_with` mutex — clap ignores the default). It completes a real
    // own@0 2-of-2 over the over-supply space.
    let cos = &[(SEED_A, 0u32), (SEED_B, 0u32)];
    let md1 = emit_template_md1("wsh-multi", "2", cos);
    let id = emit_template_wallet_id("wsh-multi", "2", cos);
    let golden = golden_addresses("wsh-multi", 2, cos, false, 2);
    let mk1_b = emit_cosigner_mk1("wsh-multi", "2", cos, 1);

    let mut args = vec!["restore".into(), "--network".into(), "mainnet".into()];
    push_md1(&mut args, &md1);
    args.extend([
        "--from".into(),
        format!("phrase={SEED_A}"),
        "--own-account-max".into(),
        "3".into(),
        "--expect-wallet-id".into(),
        id,
        "--count".into(),
        "2".into(),
        "--json".into(),
    ]);
    for c in &mk1_b {
        args.push("--cosigner".into());
        args.push(c.clone());
    }
    let got = restore_addresses(&args);
    assert_eq!(
        got, golden,
        "--own-account-max alone (no --account) must pass clap and complete"
    );
}

#[test]
fn account_and_own_account_max_conflict() {
    // I-4: `--account L` + `--own-account-max K` together → clap `conflicts_with`
    // parse error (the toolkit maps clap usage errors to exit 64), BEFORE any
    // work. `--own-account-max K` ALONE passes (own_account_max_alone_passes);
    // the explicit `--account` is what trips the mutex.
    let cos = &[(SEED_A, 0u32), (SEED_B, 0u32)];
    let md1 = emit_template_md1("wsh-multi", "2", cos);

    let mut args = vec!["restore".into(), "--network".into(), "mainnet".into()];
    push_md1(&mut args, &md1);
    args.extend([
        "--from".into(),
        format!("phrase={SEED_A}"),
        "--account".into(),
        "0,1".into(),
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
        "--account + --own-account-max must be a clap conflict: {stderr}"
    );
}

// ===========================================================================
// §5a premise-violation refusals (own-only) — all fail SAFE.
// ===========================================================================

#[test]
fn own_only_over_supplied_cosigners_refuses() {
    // §5a: own-only mode (default; no --search-cosigner-subset) with MORE
    // cosigner cards than the wallet has (M'>M) → REFUSE UP FRONT, naming
    // --search-cosigner-subset (the P3 flag). Distinct from the legacy
    // pool>n refuse — this is the new own-only-needs-exact-cosigners gate.
    let cos = &[(SEED_A, 0u32), (SEED_B, 0u32)];
    let md1 = emit_template_md1("wsh-sortedmulti", "2", cos);
    let id = emit_template_wallet_id("wsh-sortedmulti", "2", cos);
    let mk1_b = emit_cosigner_mk1("wsh-sortedmulti", "2", cos, 1);
    // an extra (outsider) cosigner card → over-supplied cosigners.
    let cos_extra = &[(SEED_A, 0u32), (SEED_OUTSIDER, 0u32)];
    let mk1_extra = emit_cosigner_mk1("wsh-sortedmulti", "2", cos_extra, 1);

    let mut args = vec!["restore".into(), "--network".into(), "mainnet".into()];
    push_md1(&mut args, &md1);
    args.extend([
        "--from".into(),
        format!("phrase={SEED_A}"),
        "--own-account-max".into(),
        "3".into(),
        "--expect-wallet-id".into(),
        id,
    ]);
    for c in &mk1_b {
        args.push("--cosigner".into());
        args.push(c.clone());
    }
    for c in &mk1_extra {
        args.push("--cosigner".into());
        args.push(c.clone());
    }
    let assert = mnemonic().args(&args).assert().failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let low = stderr.to_lowercase();
    assert!(
        low.contains("own-only") && low.contains("--search-cosigner-subset"),
        "over-supplied cosigners in own-only must refuse naming --search-cosigner-subset: {stderr}"
    );
    assert!(
        !low.contains("no match"),
        "the refusal must be an INPUT error, not a search NO-MATCH: {stderr}"
    );
}

#[test]
fn own_only_under_supplied_cosigners_no_match() {
    // §5a: under-supplied cosigners (M'<M) → NO-MATCH refuse (the engine cannot
    // fill every slot). A 3-of-3 with only ONE cosigner card supplied + the
    // over-supplied own range.
    let cos = &[(SEED_A, 0u32), (SEED_B, 0u32), (SEED_C, 0u32)];
    let md1 = emit_template_md1("wsh-multi", "3", cos);
    let id = emit_template_wallet_id("wsh-multi", "3", cos);
    let mk1_b = emit_cosigner_mk1("wsh-multi", "3", cos, 1);

    let mut args = vec!["restore".into(), "--network".into(), "mainnet".into()];
    push_md1(&mut args, &md1);
    args.extend([
        "--from".into(),
        format!("phrase={SEED_A}"),
        "--own-account-max".into(),
        "3".into(),
        "--expect-wallet-id".into(),
        id,
    ]);
    // Only ONE cosigner (B); C is missing → under-supply.
    for c in &mk1_b {
        args.push("--cosigner".into());
        args.push(c.clone());
    }
    let assert = mnemonic().args(&args).assert().failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let low = stderr.to_lowercase();
    assert!(
        low.contains("cosigner") || low.contains("slot") || low.contains("no match"),
        "under-supplied cosigners must refuse (NO-MATCH / not-enough-keys): {stderr}"
    );
}

#[test]
fn own_account_max_at_account_mutex_with_explicit_cosigner_assignment() {
    // §2 open-point 7: explicit `--cosigner @N=` assignment ⊕ subset-search
    // (`--own-account-max`) → BadInput.
    let cos = &[(SEED_A, 0u32), (SEED_B, 0u32)];
    let md1 = emit_template_md1("wsh-multi", "2", cos);
    let path = canonical_path("wsh-multi", 0);
    let (xpub_b, _fp_b) = xpub_at(SEED_B, &path);

    let mut args = vec!["restore".into(), "--network".into(), "mainnet".into()];
    push_md1(&mut args, &md1);
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
        "explicit @N= ⊕ --own-account-max must be a BadInput mutex: {stderr}"
    );
}

// ===========================================================================
// §6 hard ceilings.
// ===========================================================================

#[test]
fn own_account_max_ceiling_refuses() {
    // §6: K_own > 256 → BadInput (hard ceiling).
    let cos = &[(SEED_A, 0u32), (SEED_B, 0u32)];
    let md1 = emit_template_md1("wsh-multi", "2", cos);
    let id = emit_template_wallet_id("wsh-multi", "2", cos);
    let mk1_b = emit_cosigner_mk1("wsh-multi", "2", cos, 1);

    let mut args = vec!["restore".into(), "--network".into(), "mainnet".into()];
    push_md1(&mut args, &md1);
    args.extend([
        "--from".into(),
        format!("phrase={SEED_A}"),
        "--own-account-max".into(),
        "300".into(),
        "--expect-wallet-id".into(),
        id,
    ]);
    for c in &mk1_b {
        args.push("--cosigner".into());
        args.push(c.clone());
    }
    let assert = mnemonic().args(&args).assert().failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let low = stderr.to_lowercase();
    assert!(
        low.contains("256") || low.contains("own-account-max"),
        "K_own > 256 must refuse via the hard ceiling: {stderr}"
    );
}

#[test]
fn own_account_max_short_id_prefix_refuses() {
    // The worked prefix sizing: a too-short --expect-wallet-id over the LARGER
    // over-supply space must refuse for-weakness. A 4-byte (8-hex) prefix that
    // is accepted for an n!-space (the exact-pool floor_weak test uses the same)
    // must be REJECTED for the larger s_own space.
    let cos = &[(SEED_A, 0u32), (SEED_B, 0u32)];
    let md1 = emit_template_md1("wsh-multi", "2", cos);
    let id = emit_template_wallet_id("wsh-multi", "2", cos);
    let weak = id[..8].to_string(); // 4 bytes
    let mk1_b = emit_cosigner_mk1("wsh-multi", "2", cos, 1);

    let mut args = vec!["restore".into(), "--network".into(), "mainnet".into()];
    push_md1(&mut args, &md1);
    args.extend([
        "--from".into(),
        format!("phrase={SEED_A}"),
        "--own-account-max".into(),
        "32".into(),
        "--expect-wallet-id".into(),
        weak,
    ]);
    for c in &mk1_b {
        args.push("--cosigner".into());
        args.push(c.clone());
    }
    let assert = mnemonic().args(&args).assert().failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let low = stderr.to_lowercase();
    assert!(
        low.contains("prefix") || low.contains("weak") || low.contains("bytes"),
        "a too-short id prefix over the over-supply space must refuse: {stderr}"
    );
}

// ===========================================================================
// M-1 (P3a R0 fold): the own-origin deviation reproduces a DEFAULT-family
// (BIP-87) wallet. The toolkit's multisig emit DEFAULTS to BIP-87
// (`m/87'/coin'/acct'`), but `canonical_origin(tree)` ALWAYS returns the BIP-48
// origin for `wsh(...)` shapes (it is structural). So a wallet emitted at BIP-87
// can only be reproduced by reading the cosigner's actual family off its mk1 and
// substituting the own account — NOT the BIP-48 canonical fallback. This pins
// the deviation's CENTRAL claim, which all explicit-BIP-48 vectors leave untested.
// ===========================================================================

/// Emit a keyless multisig template md1 at the DEFAULT (BIP-87) family
/// (`m/87'/0'/account'`). Returns the md1 chunk(s).
fn emit_template_md1_bip87(threshold: &str, cosigners: &[(&str, u32)]) -> Vec<String> {
    bundle_bip87_args("template", threshold, cosigners, |args, stdout| {
        let _ = args;
        md1_lines(stdout)
    })
}

/// The printed WalletPolicyId (full hex) for the BIP-87 template emit.
fn emit_template_wallet_id_bip87(threshold: &str, cosigners: &[(&str, u32)]) -> String {
    let args = bundle_bip87_arg_vec("template", threshold, cosigners);
    let out = mnemonic().args(&args).assert().success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    let line = stderr
        .lines()
        .find(|l| l.contains("wallet-id (hex)"))
        .unwrap_or_else(|| panic!("no wallet-id (hex) line in: {stderr}"));
    line.split(':').next_back().unwrap().trim().to_string()
}

/// Emit a single cosigner mk1 card (policy form) at the BIP-87 family.
fn emit_cosigner_mk1_bip87(
    threshold: &str,
    cosigners: &[(&str, u32)],
    which: usize,
) -> Vec<String> {
    let args = bundle_bip87_arg_vec("policy", threshold, cosigners);
    let out = mnemonic().args(&args).assert().success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let groups = mk1_groups(&stdout);
    groups
        .get(which)
        .unwrap_or_else(|| panic!("no mk1 group {which} in: {stdout}"))
        .clone()
}

/// Build the `bundle` arg vector for a BIP-87 `wsh-sortedmulti` template/policy
/// (each cosigner at `m/87'/0'/account'`). Mirrors `emit_template_md1` but at
/// the DEFAULT path family.
fn bundle_bip87_arg_vec(form: &str, threshold: &str, cosigners: &[(&str, u32)]) -> Vec<String> {
    let mut args: Vec<String> = vec![
        "bundle".into(),
        "--network".into(),
        "mainnet".into(),
        "--template".into(),
        "wsh-sortedmulti".into(),
        "--threshold".into(),
        threshold.into(),
        "--md1-form".into(),
        form.into(),
        "--group-size".into(),
        "0".into(),
        "--no-engraving-card".into(),
    ];
    for (idx, (phrase, account)) in cosigners.iter().enumerate() {
        let path = format!("87'/0'/{account}'");
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

fn bundle_bip87_args<T>(
    form: &str,
    threshold: &str,
    cosigners: &[(&str, u32)],
    extract: impl Fn(&[String], &str) -> T,
) -> T {
    let args = bundle_bip87_arg_vec(form, threshold, cosigners);
    let out = mnemonic().args(&args).assert().success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    extract(&args, &stdout)
}

/// INDEPENDENT golden for a BIP-87 `wsh(sortedmulti(...))` wallet: built directly
/// from the cosigner xpubs at `m/87'/0'/account'` via rust-miniscript.
fn golden_addresses_bip87(threshold: u32, cosigners: &[(&str, u32)], count: u32) -> Vec<String> {
    let mut key_strs: Vec<String> = Vec::new();
    for (phrase, account) in cosigners {
        let path = format!("87'/0'/{account}'");
        let (xpub, fp) = xpub_at(phrase, &path);
        let origin = path.replace('\'', "h");
        key_strs.push(format!("[{fp}/{origin}]{xpub}/<0;1>/*"));
    }
    let desc_str = format!("wsh(sortedmulti({threshold},{}))", key_strs.join(","));
    let desc = Descriptor::<DescriptorPublicKey>::from_str(&desc_str)
        .unwrap_or_else(|e| panic!("golden bip87 descriptor parse {desc_str}: {e}"));
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

#[test]
fn default_family_bip87_id_search_completes_to_golden() {
    // Emit a 2-of-2 sortedmulti at the DEFAULT (BIP-87) family, then complete it
    // via id-search. The own origin must default to the cosigner's BIP-87 family
    // (m/87'/0'/0'), NOT the BIP-48 canonical_origin(tree) — else the own key
    // derives at the wrong path and the search NO-MATCHES.
    let cos = &[(SEED_A, 0u32), (SEED_B, 0u32)];
    let md1 = emit_template_md1_bip87("2", cos);
    let id = emit_template_wallet_id_bip87("2", cos);
    let mk1_b = emit_cosigner_mk1_bip87("2", cos, 1);
    let golden = golden_addresses_bip87(2, cos, 3);

    let mut args = vec!["restore".into(), "--network".into(), "mainnet".into()];
    push_md1(&mut args, &md1);
    args.extend([
        "--from".into(),
        format!("phrase={SEED_A}"),
        "--account".into(),
        "0".into(),
        "--expect-wallet-id".into(),
        id,
        "--count".into(),
        "3".into(),
        "--json".into(),
    ]);
    for c in &mk1_b {
        args.push("--cosigner".into());
        args.push(c.clone());
    }
    let got = restore_addresses(&args);
    assert_eq!(
        got, golden,
        "default-family (BIP-87) id-search must reproduce the BIP-87 golden via the cosigner-family own-origin default"
    );
}

#[test]
fn default_family_bip87_address_search_completes_to_golden() {
    // Same DEFAULT (BIP-87) wallet, completed via address-search.
    let cos = &[(SEED_A, 0u32), (SEED_B, 0u32)];
    let md1 = emit_template_md1_bip87("2", cos);
    let mk1_b = emit_cosigner_mk1_bip87("2", cos, 1);
    let golden = golden_addresses_bip87(2, cos, 3);

    let mut args = vec!["restore".into(), "--network".into(), "mainnet".into()];
    push_md1(&mut args, &md1);
    args.extend([
        "--from".into(),
        format!("phrase={SEED_A}"),
        "--account".into(),
        "0".into(),
        "--search-address".into(),
        golden[0].clone(),
        "--count".into(),
        "3".into(),
        "--json".into(),
    ]);
    for c in &mk1_b {
        args.push("--cosigner".into());
        args.push(c.clone());
    }
    let got = restore_addresses(&args);
    assert_eq!(
        got, golden,
        "default-family (BIP-87) address-search must reproduce the BIP-87 golden"
    );
}

// ===========================================================================
// Non-regression: single-sig template completion (#28 phase 1) still works.
// ===========================================================================

// ===========================================================================
// C1 INVARIANT (funds-safety): the template's CARRIED path_decl is NEVER loaded
// into the completion. A template md1 re-encoded with a deliberately WRONG
// carried origin must STILL complete to the SAME wallet — proving the
// per-slot origins are BUILT FRESH from the supplied keys, not the carried ones.
// ===========================================================================

#[test]
fn carried_origin_never_loaded_into_completion() {
    use md_codec::origin_path::{OriginPath, PathComponent, PathDecl, PathDeclPaths};

    let cos = &[(SEED_A, 0u32), (SEED_B, 0u32)];
    let md1 = emit_template_md1("wsh-sortedmulti", "2", cos);
    let id = emit_template_wallet_id("wsh-sortedmulti", "2", cos);
    let mk1_b = emit_cosigner_mk1("wsh-sortedmulti", "2", cos, 1);
    let golden = golden_addresses("wsh-sortedmulti", 2, cos, true, 2);

    // Decode the emitted (origin-elided) template, then TAMPER the carried
    // path_decl to a deliberately WRONG, non-canonical origin (m/99'/0'/0'/2').
    // (Use Divergent to bypass canonical-elision so the wrong origin is on the
    // wire; the completion must ignore it entirely.)
    let md1_refs: Vec<&str> = md1.iter().map(|s| s.as_str()).collect();
    let mut tampered = md_codec::chunk::reassemble(&md1_refs).expect("template decodes");
    let wrong = OriginPath {
        components: vec![
            PathComponent {
                hardened: true,
                value: 99,
            },
            PathComponent {
                hardened: true,
                value: 0,
            },
            PathComponent {
                hardened: true,
                value: 0,
            },
            PathComponent {
                hardened: true,
                value: 2,
            },
        ],
    };
    tampered.path_decl = PathDecl {
        n: 2,
        paths: PathDeclPaths::Divergent(vec![wrong.clone(), wrong]),
    };
    let tampered_md1 = md_codec::chunk::split(&tampered).expect("tampered template re-encodes");

    let mut args = vec!["restore".into(), "--network".into(), "mainnet".into()];
    push_md1(&mut args, &tampered_md1);
    args.extend([
        "--from".into(),
        format!("phrase={SEED_A}"),
        "--account".into(),
        "0".into(),
        "--expect-wallet-id".into(),
        id,
        "--count".into(),
        "2".into(),
        "--json".into(),
    ]);
    for c in &mk1_b {
        args.push("--cosigner".into());
        args.push(c.clone());
    }
    // The completion BUILDS fresh origins from the supplied keys, so the wrong
    // carried origin is irrelevant → same golden wallet.
    let got = restore_addresses(&args);
    assert_eq!(
        got, golden,
        "the carried (wrong) origin must NOT reach completion — fresh origins reproduce the wallet"
    );
}

// ===========================================================================
// Ambiguous → refuse: id-search where two assignments both match (a too-short
// prefix that happens to be satisfiable by ≥2 placements). Harder to force
// deterministically; instead pin the engine refusal on an over-broad search.
// (The sortedmulti id-search is order-SENSITIVE so it is not ambiguous; the
//  ambiguity floor is unit-tested in permutation_search. Here we pin that a
//  wrong-key set with NO match refuses — the None arm — already covered above.)
// ===========================================================================

// ===========================================================================
// Explicit mode (Mode B): all cosigners assigned via @N= + own from --from;
// builds WITHOUT a search and fires the unverified-assignment warning.
// ===========================================================================

#[test]
fn explicit_assignment_mode_completes_and_warns() {
    let cos = &[(SEED_A, 0u32), (SEED_B, 0u32)];
    let md1 = emit_template_md1("wsh-sortedmulti", "2", cos);
    let golden = golden_addresses("wsh-sortedmulti", 2, cos, true, 2);
    // cosigner B assigned explicitly at @1; own (A) fills @0.
    let mk1_b = emit_cosigner_mk1("wsh-sortedmulti", "2", cos, 1);

    let mut args = vec!["restore".into(), "--network".into(), "mainnet".into()];
    push_md1(&mut args, &md1);
    args.extend([
        "--from".into(),
        format!("phrase={SEED_A}"),
        "--account".into(),
        "0".into(),
        "--count".into(),
        "2".into(),
        "--json".into(),
    ]);
    for c in &mk1_b {
        args.push("--cosigner".into());
        args.push(format!("@1={c}"));
    }
    let out = mnemonic().args(&args).assert().success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.to_lowercase().contains("without verifying")
            || stderr.to_lowercase().contains("wrong assignment"),
        "explicit mode must warn the assignment is unverified: {stderr}"
    );
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let j: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let got: Vec<String> = j["wallets"][0]["first_addresses"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a.as_str().unwrap().to_string())
        .collect();
    // sortedmulti is order-independent → explicit @1=B / @0=A reproduces golden.
    assert_eq!(
        got, golden,
        "explicit mode (sortedmulti) reproduces the golden"
    );
}

#[test]
fn singlesig_template_completion_unchanged() {
    // bip84 single-sig template still completes from --from (phase-1 path).
    let out = mnemonic()
        .args([
            "bundle",
            "--template",
            "bip84",
            "--network",
            "mainnet",
            "--md1-form",
            "template",
            "--group-size",
            "0",
            "--no-engraving-card",
            "--slot",
            &format!("@0.phrase={SEED_A}"),
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let md1 = md1_lines(&stdout);
    let mut args = vec!["restore".into(), "--network".into(), "mainnet".into()];
    push_md1(&mut args, &md1);
    args.extend([
        "--from".into(),
        format!("phrase={SEED_A}"),
        "--account".into(),
        "0".into(),
        "--json".into(),
    ]);
    // single-sig completion must succeed and yield a bc1q address.
    let addrs = restore_addresses(&args);
    assert!(
        addrs[0].starts_with("bc1q"),
        "single-sig bip84 → bech32 addr: {addrs:?}"
    );
}

// ===========================================================================
// P3b — GENERAL / thresh policy completion (non-canonical origins).
//
// P3a covered CANONICAL multisig (wsh(multi/sortedmulti), sh(wsh) — where
// `canonical_origin(tree).is_some()`). P3b extends `restore --md1` completion to
// GENERAL/thresh policies (`canonical_origin(tree).is_none()`), e.g. a
// `wsh(or_i(...))` policy where the keys play DISTINCT spending roles and the
// per-@N origins are non-canonical (here BIP-84 `m/84'/0'/N'` — NOT the BIP-48
// that `canonical_origin` would force).
//
// The chosen shape is GENERAL + ORDER-DEPENDENT + DIVERGENT:
//   wsh(or_i(pk(@0), and_v(v:pk(@1), pk(@2))))
//   @0 = SEED_A at m/84'/0'/0'   (own — single-key OR branch)
//   @1 = SEED_B at m/84'/0'/1'   (cosigner — AND branch)
//   @2 = SEED_C at m/84'/0'/2'   (cosigner — AND branch)
// • GENERAL: the `or_i` combinator → `canonical_origin(tree)` is None (pinned
//   below) → falls through to `run_multisig` (which REFUSES) before P3b.
// • ORDER-DEPENDENT: @0 (alone-spends OR branch) vs @1/@2 (jointly-spend AND
//   branch) are DIFFERENT spending roles — a wrong assignment is a different
//   wallet (not order-independent like sortedmulti).
// • DIVERGENT: three DISTINCT per-@N BIP-84 origins (accounts 0,1,2) → the
//   built `path_decl` is `PathDeclPaths::Divergent` (the C1 general case).
// 3 keys → 3! = 6 permutations (a fast shape).
//
// The template + per-cosigner mk1s are emitted via `bundle --md1-form=
// template|policy --descriptor <full general descriptor>` (NOT `--template`,
// which is canonical-only). The independent golden is built directly from the
// SAME descriptor string via rust-miniscript (NOT an md-codec reconstruction).
// ===========================================================================

/// The own key (@0) BIP-84 origin string (no leading `m/`), e.g. `84'/0'/0'`.
fn bip84_origin(account: u32) -> String {
    format!("84'/0'/{account}'")
}

/// Build the general-policy descriptor string for the P3b shape from controlled
/// BIP-84 seeds. `slots[i] = (phrase, account)` → key `@i` at `m/84'/0'/account'`.
/// Shape: `wsh(or_i(pk(@0), and_v(v:pk(@1), pk(@2))))`.
fn general_desc(slots: &[(&str, u32)]) -> String {
    assert_eq!(slots.len(), 3, "the P3b general shape has exactly 3 keys");
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

/// Emit the keyless general-policy template md1 via `bundle --md1-form=template
/// --descriptor`. Returns the md1 chunk(s).
fn emit_general_template_md1(desc: &str) -> Vec<String> {
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
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    md1_lines(&stdout)
}

/// The printed WalletPolicyId (full hex) the general-policy template emit records.
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

/// Emit one per-cosigner mk1 card (general-policy `--md1-form=policy`) at slot
/// `which` so the completion reads its REAL BIP-84 origin path.
fn emit_general_cosigner_mk1(desc: &str, which: usize) -> Vec<String> {
    let out = mnemonic()
        .args([
            "bundle",
            "--network",
            "mainnet",
            "--md1-form",
            "policy",
            "--group-size",
            "0",
            "--no-engraving-card",
            "--descriptor",
            desc,
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let groups = mk1_groups(&stdout);
    groups
        .get(which)
        .unwrap_or_else(|| panic!("no mk1 group {which} in: {stdout}"))
        .clone()
}

/// INDEPENDENT golden for the general-policy shape: parse the SAME descriptor
/// string with rust-miniscript directly (NOT md-codec reconstruction) and derive
/// the first `count` receive addresses.
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

/// The default P3b general wallet: A@0 / B@1 / C@2 at BIP-84.
fn p3b_slots() -> Vec<(&'static str, u32)> {
    vec![(SEED_A, 0u32), (SEED_B, 1u32), (SEED_C, 2u32)]
}

/// Push the @1 (B) and @2 (C) cosigner mk1s (unassigned) onto the restore args.
fn push_general_cosigners(args: &mut Vec<String>, desc: &str) {
    for which in [1usize, 2usize] {
        for c in &emit_general_cosigner_mk1(desc, which) {
            args.push("--cosigner".into());
            args.push(c.clone());
        }
    }
}

#[test]
fn general_policy_id_search_completes_to_golden() {
    // The headline gate: a general (or_i, order-dependent, divergent-origin)
    // template completes via id-search to the INDEPENDENT rust-miniscript golden.
    let slots = p3b_slots();
    let desc = general_desc(&slots);
    // Sanity: the shape really IS general (non-canonical) + keyless.
    let md1 = emit_general_template_md1(&desc);
    let md1_refs: Vec<&str> = md1.iter().map(|s| s.as_str()).collect();
    let decoded = md_codec::chunk::reassemble(&md1_refs).expect("general template decodes");
    assert!(
        md_codec::canonical_origin::canonical_origin(&decoded.tree).is_none(),
        "the P3b shape MUST be non-canonical (general) — else it is a P3a case"
    );
    assert!(!decoded.is_wallet_policy(), "the template md1 is keyless");
    assert_eq!(decoded.n, 3, "3 distinct @N slots");

    let id = emit_general_template_wallet_id(&desc);
    let golden = general_golden_addresses(&desc, 3);

    let mut args = vec!["restore".into(), "--network".into(), "mainnet".into()];
    push_md1(&mut args, &md1);
    args.extend([
        "--from".into(),
        format!("phrase={SEED_A}"),
        "--account".into(),
        "0".into(),
        "--expect-wallet-id".into(),
        id,
        "--count".into(),
        "3".into(),
        "--json".into(),
    ]);
    push_general_cosigners(&mut args, &desc);
    let got = restore_addresses(&args);
    assert_eq!(
        got, golden,
        "general-policy id-search completion must match the independent golden"
    );
}

#[test]
fn general_policy_address_search_completes_to_golden() {
    // Same general shape, completed via address-search instead of id-search.
    let slots = p3b_slots();
    let desc = general_desc(&slots);
    let md1 = emit_general_template_md1(&desc);
    let golden = general_golden_addresses(&desc, 3);

    let mut args = vec!["restore".into(), "--network".into(), "mainnet".into()];
    push_md1(&mut args, &md1);
    args.extend([
        "--from".into(),
        format!("phrase={SEED_A}"),
        "--account".into(),
        "0".into(),
        "--search-address".into(),
        golden[0].clone(),
        "--count".into(),
        "3".into(),
        "--json".into(),
    ]);
    push_general_cosigners(&mut args, &desc);
    let got = restore_addresses(&args);
    assert_eq!(
        got, golden,
        "general-policy address-search completion must match the independent golden"
    );
}

#[test]
fn general_policy_carried_origin_never_loaded() {
    // C1 for a DIVERGENT general template: the carried per-@N `path_decl` is
    // NEVER loaded into completion. Tamper the carried origins to deliberately
    // WRONG, non-canonical paths (m/99'/0'/N'); the completion must STILL reach
    // the SAME golden — proving the per-slot origins are BUILT FRESH from the
    // supplied keys (own --account / cosigner mk1), not the carried ones.
    use md_codec::origin_path::{OriginPath, PathComponent, PathDecl, PathDeclPaths};

    let slots = p3b_slots();
    let desc = general_desc(&slots);
    let md1 = emit_general_template_md1(&desc);
    let id = emit_general_template_wallet_id(&desc);
    let golden = general_golden_addresses(&desc, 2);

    // Decode, then TAMPER the carried Divergent path_decl to wrong origins.
    let md1_refs: Vec<&str> = md1.iter().map(|s| s.as_str()).collect();
    let mut tampered = md_codec::chunk::reassemble(&md1_refs).expect("general template decodes");
    assert!(
        md_codec::canonical_origin::canonical_origin(&tampered.tree).is_none(),
        "tamper target must be a general (non-canonical) template"
    );
    let wrong = |acct: u32| OriginPath {
        components: vec![
            PathComponent {
                hardened: true,
                value: 99,
            },
            PathComponent {
                hardened: true,
                value: 0,
            },
            PathComponent {
                hardened: true,
                value: acct,
            },
        ],
    };
    tampered.path_decl = PathDecl {
        n: 3,
        paths: PathDeclPaths::Divergent(vec![wrong(0), wrong(1), wrong(2)]),
    };
    let tampered_md1 =
        md_codec::chunk::split(&tampered).expect("tampered general template re-encodes");

    let mut args = vec!["restore".into(), "--network".into(), "mainnet".into()];
    push_md1(&mut args, &tampered_md1);
    args.extend([
        "--from".into(),
        format!("phrase={SEED_A}"),
        "--account".into(),
        "0".into(),
        "--expect-wallet-id".into(),
        id,
        "--count".into(),
        "2".into(),
        "--json".into(),
    ]);
    push_general_cosigners(&mut args, &desc);
    let got = restore_addresses(&args);
    assert_eq!(
        got, golden,
        "the WRONG carried origin must NOT reach completion — fresh per-slot origins reproduce the wallet"
    );
}

#[test]
fn general_policy_wrong_family_no_match() {
    // Anti-vacuity (I-A): the own BIP-84 origin is LOAD-BEARING. The own key (@0)
    // belongs at m/84'/0'/0'. If the operator forces a WRONG own family via
    // --origin (here BIP-48 m/48'/0'/0'/2' — what compute_default_origin_path /
    // canonical_origin would yield), the own key derives at the wrong path, so NO
    // permutation of the supplied keys reproduces the recorded id → REFUSE.
    let slots = p3b_slots();
    let desc = general_desc(&slots);
    let md1 = emit_general_template_md1(&desc);
    let id = emit_general_template_wallet_id(&desc);

    let mut args = vec!["restore".into(), "--network".into(), "mainnet".into()];
    push_md1(&mut args, &md1);
    args.extend([
        "--from".into(),
        format!("phrase={SEED_A}"),
        "--origin".into(),
        "m/48'/0'/0'/2'".into(), // WRONG family (BIP-48, not the wallet's BIP-84)
        "--expect-wallet-id".into(),
        id,
    ]);
    push_general_cosigners(&mut args, &desc);
    // A wrong own family cannot reproduce the wallet → no-match → refuse.
    // Non-vacuity: the refusal must come from the SEARCH (NO MATCH), NOT the
    // pre-P3b "template-only" routing refusal that rejects every general md1.
    let assert = mnemonic().args(&args).assert().failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(
        !stderr.to_lowercase().contains("template-only"),
        "the wrong-family refusal must be a genuine search NO-MATCH (proving the \
         general template was ROUTED to completion + the BIP-84 own origin is \
         load-bearing), not the pre-P3b template-only routing refusal: {stderr}"
    );
}

// --- P3b floors (general shape) --------------------------------------------

#[test]
fn general_policy_floor_no_from_refuses() {
    // Floor 1(i): a no-seed general template completion refuses, naming --from.
    let slots = p3b_slots();
    let desc = general_desc(&slots);
    let md1 = emit_general_template_md1(&desc);
    let mut args = vec!["restore".into(), "--network".into(), "mainnet".into()];
    push_md1(&mut args, &md1);
    // no --from
    let assert = mnemonic().args(&args).assert().failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("--from"),
        "no-seed general template completion must name --from: {stderr}"
    );
}

#[test]
fn general_policy_floor_unsupplied_slot_refuses() {
    // Floor 1(ii): only the own seed + ONE cosigner supplied for a 3-slot general
    // wallet → the 3rd slot has no key → refuse (cannot complete).
    let slots = p3b_slots();
    let desc = general_desc(&slots);
    let md1 = emit_general_template_md1(&desc);
    let id = emit_general_template_wallet_id(&desc);

    let mut args = vec!["restore".into(), "--network".into(), "mainnet".into()];
    push_md1(&mut args, &md1);
    args.extend([
        "--from".into(),
        format!("phrase={SEED_A}"),
        "--account".into(),
        "0".into(),
        "--expect-wallet-id".into(),
        id,
    ]);
    // supply ONLY @1 (B); leave @2 (C) unsupplied.
    for c in &emit_general_cosigner_mk1(&desc, 1) {
        args.push("--cosigner".into());
        args.push(c.clone());
    }
    // Non-vacuity: the refusal must be the every-slot floor (not-enough-keys),
    // NOT the pre-P3b "template-only" routing refusal.
    let assert = mnemonic().args(&args).assert().failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(
        !stderr.to_lowercase().contains("template-only"),
        "the unsupplied-slot refusal must be the every-slot floor (general template \
         ROUTED to completion), not the pre-P3b template-only routing refusal: {stderr}"
    );
}

#[test]
fn general_policy_floor_duplicate_cosigner_key_refuses() {
    // Floor 2: supplying the SAME cosigner mk1 for both cosigner slots collides
    // on key → the duplicate-key floor must reject before the search.
    let slots = p3b_slots();
    let desc = general_desc(&slots);
    let md1 = emit_general_template_md1(&desc);
    let id = emit_general_template_wallet_id(&desc);
    let mk1_b = emit_general_cosigner_mk1(&desc, 1);

    let mut args = vec!["restore".into(), "--network".into(), "mainnet".into()];
    push_md1(&mut args, &md1);
    args.extend([
        "--from".into(),
        format!("phrase={SEED_A}"),
        "--account".into(),
        "0".into(),
        "--expect-wallet-id".into(),
        id,
    ]);
    // supply cosigner B TWICE (byte-identical key) for the two cosigner slots.
    for c in &mk1_b {
        args.push("--cosigner".into());
        args.push(c.clone());
    }
    for c in &mk1_b {
        args.push("--cosigner".into());
        args.push(c.clone());
    }
    let assert = mnemonic().args(&args).assert().failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.to_lowercase().contains("duplicate") || stderr.to_lowercase().contains("identical"),
        "duplicate cosigner keys must be named: {stderr}"
    );
}

// ===========================================================================
// P5 — degrade2-STRUCTURED default-CI differential (SPEC §7 "degrade2 shape
// completes" gate, at a TRACTABLE size).
//
// The full degrade2.desc is an 11-key general policy (timelocks + sha256
// hashlock + several `or_i` branches across many BIP-84 accounts) — its n! is
// huge and the real seeds are unknown. This test builds a FAITHFUL ANALOG from
// the controlled SEED_A/B/C: a `wsh(or_i(...))` carrying ALL the structural
// hallmarks of degrade2 — an `after()` ABSOLUTE timelock, an `older()` RELATIVE
// timelock, a `sha256()` HASHLOCK, an inner `multi(2,...)`, and MULTIPLE BIP-84
// accounts — kept to 4 distinct slots (n! = 24, tractable). Two of the four
// slots are OWN (SEED_A at accounts 0 and 3 → `--account 0,3`), exercising the
// multi-own-account resolution on a general (divergent-origin) shape.
//
// Shape (slot roles):
//   wsh(or_i(
//     and_v(v:after(1000000), and_v(v:sha256(H), pk(@0))),   // @0 own  @ 84'/0'/0'
//     or_i(
//       and_v(v:older(65535), multi(2, @1, @2)),             // @1,@2   @ 84'/0'/{1,2}'
//       and_v(v:pk(@3), after(1893456000))                   // @3 own  @ 84'/0'/3'
//     )
//   ))
//
// • GENERAL: the `or_i` combinators → `canonical_origin(tree)` is None.
// • DIVERGENT: four DISTINCT BIP-84 origins (accounts 0,1,2,3).
// • ORDER-DEPENDENT: the four keys play distinct spending roles.
//
// Oracle (NON-VACUOUS): completed addresses == an INDEPENDENT rust-miniscript
// derivation of the ORIGINAL concrete descriptor (NOT md-codec reconstruction).
// Anti-vacuity: a WRONG key→slot assignment (a swapped @0↔@3, which puts the own
// keys in different roles) derives a DIFFERENT address — proven by
// `degrade2_structured_anti_vacuity_swapped_assignment_differs`.
// ===========================================================================

/// A fixed sha256 hashlock preimage-hash (the degrade2 reference value — opaque
/// to derivation; any 32-byte hex works, this one mirrors the real card).
const DEGRADE2_SHA256: &str = "a84dce40975727c398023cfbd50d5db3b9662375521d0f1ac62dbd829b9a08ad";

/// Build the degrade2-structured analog descriptor from 4 (seed, account) slots
/// at BIP-84 `m/84'/0'/account'`. `slots[i]` → key `@i`.
fn degrade2_desc(slots: &[(&str, u32)]) -> String {
    assert_eq!(slots.len(), 4, "the degrade2 analog has exactly 4 slots");
    let mut keys: Vec<String> = Vec::new();
    for (phrase, account) in slots {
        let path = bip84_origin(*account);
        let (xpub, fp) = xpub_at(phrase, &path);
        let origin = path.replace('\'', "h");
        keys.push(format!("[{fp}/{origin}]{xpub}/<0;1>/*"));
    }
    format!(
        "wsh(or_i(\
           and_v(v:after(1000000),and_v(v:sha256({h}),pk({k0}))),\
           or_i(\
             and_v(v:older(65535),multi(2,{k1},{k2})),\
             and_v(v:pk({k3}),after(1893456000))\
           )\
         ))",
        h = DEGRADE2_SHA256,
        k0 = keys[0],
        k1 = keys[1],
        k2 = keys[2],
        k3 = keys[3],
    )
}

/// The default degrade2-analog wallet: own SEED_A at accounts 0 (@0) and 3 (@3);
/// cosigners SEED_B@1 (@1) and SEED_C@2 (@2).
fn degrade2_slots() -> Vec<(&'static str, u32)> {
    vec![
        (SEED_A, 0u32),
        (SEED_B, 1u32),
        (SEED_C, 2u32),
        (SEED_A, 3u32),
    ]
}

#[test]
fn degrade2_structured_completes_to_golden() {
    let slots = degrade2_slots();
    let desc = degrade2_desc(&slots);

    let md1 = emit_general_template_md1(&desc);
    // Sanity: genuinely general (non-canonical) + keyless + 4 slots.
    let md1_refs: Vec<&str> = md1.iter().map(|s| s.as_str()).collect();
    let decoded = md_codec::chunk::reassemble(&md1_refs).expect("degrade2 template decodes");
    assert!(
        md_codec::canonical_origin::canonical_origin(&decoded.tree).is_none(),
        "the degrade2 analog MUST be a general (non-canonical) policy"
    );
    assert!(!decoded.is_wallet_policy(), "the template md1 is keyless");
    assert_eq!(decoded.n, 4, "4 distinct @N slots");

    let id = emit_general_template_wallet_id(&desc);
    let golden = general_golden_addresses(&desc, 3);

    let mut args = vec!["restore".into(), "--network".into(), "mainnet".into()];
    push_md1(&mut args, &md1);
    args.extend([
        "--from".into(),
        format!("phrase={SEED_A}"),
        // TWO own accounts (the SAME seed at 0 and 3) → multi-own resolution.
        "--account".into(),
        "0,3".into(),
        "--expect-wallet-id".into(),
        id,
        "--count".into(),
        "3".into(),
        "--json".into(),
    ]);
    // The two EXTERNAL cosigners @1 (B) and @2 (C), unassigned.
    for which in [1usize, 2usize] {
        for c in &emit_general_cosigner_mk1(&desc, which) {
            args.push("--cosigner".into());
            args.push(c.clone());
        }
    }
    let got = restore_addresses(&args);
    assert_eq!(
        got, golden,
        "degrade2-structured (after+older+sha256+multi, multi-account own) completion \
         must match the independent rust-miniscript golden"
    );
}

#[test]
fn degrade2_structured_anti_vacuity_swapped_assignment_differs() {
    // NON-VACUITY: the golden oracle is DISCRIMINATING for the degrade2 shape. The
    // two own keys live in DIFFERENT spending roles (@0 = the sha256-gated
    // after(1000000) branch; @3 = the pk + after(1893456000) branch). Swapping
    // @0↔@3 (SEED_A@0 ↔ SEED_A@3) yields a STRUCTURALLY-different wallet whose
    // first address DIFFERS — so a completion that placed the own keys in the
    // wrong roles could not "match the golden" vacuously.
    let correct = degrade2_desc(&[
        (SEED_A, 0u32),
        (SEED_B, 1u32),
        (SEED_C, 2u32),
        (SEED_A, 3u32),
    ]);
    let swapped = degrade2_desc(&[
        (SEED_A, 3u32), // @0 now SEED_A@3
        (SEED_B, 1u32),
        (SEED_C, 2u32),
        (SEED_A, 0u32), // @3 now SEED_A@0
    ]);
    assert_ne!(
        general_golden_addresses(&correct, 1),
        general_golden_addresses(&swapped, 1),
        "the degrade2 golden MUST anchor the @0↔@3 role distinction — a swapped \
         own-key assignment must derive a DIFFERENT address (else the oracle is vacuous)"
    );

    // A COSIGNER-pair swap (@1↔@2) inside the order-dependent multi(2) must ALSO
    // differ (different key order → different witness script → different address).
    let cosigner_swapped = degrade2_desc(&[
        (SEED_A, 0u32),
        (SEED_C, 2u32), // @1 now SEED_C@2
        (SEED_B, 1u32), // @2 now SEED_B@1
        (SEED_A, 3u32),
    ]);
    assert_ne!(
        general_golden_addresses(&correct, 1),
        general_golden_addresses(&cosigner_swapped, 1),
        "a @1↔@2 swap inside the order-dependent multi(2) must also differ"
    );
}
