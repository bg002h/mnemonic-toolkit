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
fn emit_cosigner_mk1(script: &str, threshold: &str, cosigners: &[(&str, u32)], which: usize) -> Vec<String> {
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
    let receive = desc
        .clone()
        .into_single_descriptors()
        .unwrap()
        .remove(0);
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

    let mut args = vec![
        "restore".into(),
        "--network".into(),
        "mainnet".into(),
    ];
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
    assert_eq!(got, golden, "id-search completion must match the independent golden");
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
    assert_eq!(got, golden, "wsh-multi id-search must match the golden in the resolved order");
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
    assert_eq!(got, golden, "sh(wsh(multi)) id-search must match the golden");
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
    assert_eq!(got, golden, "address-search completion must match the independent golden");
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
    assert_eq!(got, golden, "a non-zero-index target must resolve the assignment");
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
        stderr.to_lowercase().contains("prefix") || stderr.to_lowercase().contains("weak")
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
    assert_eq!(got, golden, "multi-account own (--account 0,1) must resolve both own slots");
}

// ===========================================================================
// I-1 (P3a R0 fold): the over-supply (`pool.len() > n`) is REFUSED LOUDLY at
// input — the permutation engine enumerates only n! placements of the FIRST n
// pool entries, so any pool index ≥ n is NEVER evaluated → a legitimate wallet
// would silently NO-MATCH. The genuine `--own-account-max` subset-search is
// deferred (FOLLOWUP `template-multisig-own-account-range-subset-search`); until
// then the supported invariant is `pool.len() == n` (own keys from the
// `--account` LIST + cosigners exactly fill the N slots). Both the `--account`
// over-supply and the `--own-account-max` flag must refuse with an ACTIONABLE
// message (NOT a confusing NO-MATCH, NOT an exit-via-search).
// ===========================================================================

#[test]
fn own_account_max_flag_refuses_with_actionable_message() {
    // A real 2-of-2 {A@0, B@0}; the operator passes `--own-account-max 3` (the
    // only way to over-supply own keys). Before the fold this derived
    // own@{0,1,2}+cosigner = pool of 4 into n=2 slots and silently NO-MATCHED;
    // after the fold it must refuse LOUDLY at input, naming --account.
    let cos = &[(SEED_A, 0u32), (SEED_B, 0u32)];
    let md1 = emit_template_md1("wsh-sortedmulti", "2", cos);
    let id = emit_template_wallet_id("wsh-sortedmulti", "2", cos);
    let mk1_b = emit_cosigner_mk1("wsh-sortedmulti", "2", cos, 1);

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
    let assert = mnemonic().args(&args).assert().failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let low = stderr.to_lowercase();
    assert!(
        low.contains("own-account-max") && low.contains("--account"),
        "the --own-account-max refusal must name the flag and point at --account: {stderr}"
    );
    assert!(
        !low.contains("no match"),
        "the refusal must be an actionable INPUT error, not a search NO-MATCH: {stderr}"
    );
}

#[test]
fn pool_larger_than_slots_refuses_with_actionable_message() {
    // A real 2-of-2 {A@0, B@0}; the operator over-supplies THREE keys for two
    // slots (own@0 + cosigner B + an extra cosigner C). The pool (3) > n (2);
    // the engine can never place the 3rd → refuse LOUDLY at input.
    let cos = &[(SEED_A, 0u32), (SEED_B, 0u32)];
    let md1 = emit_template_md1("wsh-sortedmulti", "2", cos);
    let id = emit_template_wallet_id("wsh-sortedmulti", "2", cos);
    let mk1_b = emit_cosigner_mk1("wsh-sortedmulti", "2", cos, 1);
    // an extra (outsider) cosigner card → pool over-supply.
    let cos_extra = &[(SEED_A, 0u32), (SEED_OUTSIDER, 0u32)];
    let mk1_extra = emit_cosigner_mk1("wsh-sortedmulti", "2", cos_extra, 1);

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
    for c in &mk1_extra {
        args.push("--cosigner".into());
        args.push(c.clone());
    }
    let assert = mnemonic().args(&args).assert().failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let low = stderr.to_lowercase();
    assert!(
        low.contains("--account") || low.contains("more keys") || low.contains("over-supply")
            || low.contains("exactly"),
        "the over-supply refusal must be actionable: {stderr}"
    );
    assert!(
        !low.contains("no match"),
        "the refusal must be an INPUT error, not a search NO-MATCH: {stderr}"
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
            PathComponent { hardened: true, value: 99 },
            PathComponent { hardened: true, value: 0 },
            PathComponent { hardened: true, value: 0 },
            PathComponent { hardened: true, value: 2 },
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
    assert_eq!(got, golden, "explicit mode (sortedmulti) reproduces the golden");
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
    assert!(addrs[0].starts_with("bc1q"), "single-sig bip84 → bech32 addr: {addrs:?}");
}
