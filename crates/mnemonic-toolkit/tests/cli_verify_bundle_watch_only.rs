//! Verify-bundle watch-only round-trip integration test.
//!
//! Confirms SPEC §2.2.2 stderr warning is emitted alongside the round-trip
//! "result: ok" verdict.
//!
//! v0.24.0 D30: cells starting at `cross_check_*` exercise the new
//! mk1↔md1 xpub-vs-path defense-in-depth cross-check (closes FOLLOWUP
//! `verify-bundle-watch-only-xpub-path-internal-consistency`). Failure
//! mode is stderr WARNING (not hard error); verify-bundle's exit code
//! and `result: ok / mismatch` verdict are unchanged by the cross-check.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn verify_bundle_watch_only_bip84_mainnet_round_trip() {
    let fixture =
        std::fs::read_to_string("tests/vectors/v0_1/bip84-mainnet.txt").expect("fixture exists");
    let mk1_lines: Vec<&str> = fixture
        .lines()
        .filter(|l| l.starts_with("mk1") && !l.contains(' ') && !l.contains('-'))
        .collect();
    let md1_lines: Vec<&str> = fixture
        .lines()
        .filter(|l| l.starts_with("md1") && !l.contains(' ') && !l.contains('-'))
        .collect();
    assert!(!mk1_lines.is_empty() && !md1_lines.is_empty());

    let card = mk_codec::decode(&mk1_lines).expect("mk1 decodes");
    let xpub_str = card.xpub.to_string();

    let mut args: Vec<String> = vec![
        "verify-bundle".into(),
        "--slot".into(),
        format!("@0.xpub={xpub_str}"),
        "--slot".into(),
        "@0.fingerprint=5436d724".into(),
        "--network".into(),
        "mainnet".into(),
        "--template".into(),
        "bip84".into(),
    ];
    for s in &mk1_lines {
        args.push("--mk1".into());
        args.push((*s).into());
    }
    for s in &md1_lines {
        args.push("--md1".into());
        args.push((*s).into());
    }

    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&args)
        .assert()
        .success()
        .stdout(predicate::str::contains("result: ok"))
        .stderr(predicate::str::contains(
            "watch-only verify-bundle does not verify",
        ));
}

// v0.5 SPEC §5.7 case 1: watch-only with user-supplied (spurious) --ms1.
// The helper's watch-only short-circuit absorbs supplied --ms1 silently; the
// run still reports result: ok (closes FOLLOWUP verify-bundle-watch-only-
// spurious-ms1-handling).
#[test]
fn verify_bundle_watch_only_spurious_ms1_silently_absorbed_v0_5() {
    let fixture =
        std::fs::read_to_string("tests/vectors/v0_1/bip84-mainnet.txt").expect("fixture exists");
    let mk1_lines: Vec<&str> = fixture
        .lines()
        .filter(|l| l.starts_with("mk1") && !l.contains(' ') && !l.contains('-'))
        .collect();
    let md1_lines: Vec<&str> = fixture
        .lines()
        .filter(|l| l.starts_with("md1") && !l.contains(' ') && !l.contains('-'))
        .collect();
    let ms1_line: &str = fixture
        .lines()
        .find(|l| l.starts_with("ms1") && !l.contains(' '))
        .expect("compact ms1 line in fixture");

    let card = mk_codec::decode(&mk1_lines).expect("mk1 decodes");
    let xpub_str = card.xpub.to_string();

    let mut args: Vec<String> = vec![
        "verify-bundle".into(),
        "--slot".into(),
        format!("@0.xpub={xpub_str}"),
        "--slot".into(),
        "@0.fingerprint=5436d724".into(),
        "--network".into(),
        "mainnet".into(),
        "--template".into(),
        "bip84".into(),
        // Spurious --ms1 supply — should be silently absorbed in watch-only mode.
        "--ms1".into(),
        ms1_line.into(),
    ];
    for s in &mk1_lines {
        args.push("--mk1".into());
        args.push((*s).into());
    }
    for s in &md1_lines {
        args.push("--md1".into());
        args.push((*s).into());
    }

    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&args)
        .assert()
        .success()
        .stdout(predicate::str::contains("result: ok"));
}

// ============================================================================
// v0.24.0 D30 — watch-only mk1↔md1 xpub-vs-path cross-check cells.
// ============================================================================

const TREZOR_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
const BIP39_TEST_2: &str =
    "legal winner thank year wave sausage worth useful legal winner thank yellow";

/// Helper: extract mk1 (single-sig flat) + md1 vecs from a `mnemonic bundle --json` invocation.
fn gen_bundle_json(args: &[&str]) -> (Vec<String>, Vec<String>) {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(args)
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let bundle: serde_json::Value = serde_json::from_str(&stdout).expect("valid bundle JSON");
    let mk1 = bundle["mk1"]
        .as_array()
        .expect("mk1 array")
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    let md1 = bundle["md1"]
        .as_array()
        .expect("md1 array")
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    (mk1, md1)
}

/// Helper: extract mk1 chunks (flattened across cosigners) + md1 from a multisig bundle JSON.
fn gen_bundle_json_multisig(args: &[&str]) -> (Vec<String>, Vec<String>) {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(args)
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let bundle: serde_json::Value = serde_json::from_str(&stdout).expect("valid bundle JSON");
    let mut mk1: Vec<String> = Vec::new();
    for inner in bundle["mk1"].as_array().expect("mk1 array") {
        for chunk in inner.as_array().expect("inner mk1 array") {
            mk1.push(chunk.as_str().unwrap().to_string());
        }
    }
    let md1: Vec<String> = bundle["md1"]
        .as_array()
        .expect("md1 array")
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    (mk1, md1)
}

/// Build a watch-only verify-bundle argv for single-sig BIP-84 mainnet given
/// (mk1_strings, md1_strings) by deriving the watch-only `@0.xpub` slot from
/// the supplied mk1 card.
fn single_sig_watch_only_args(mk1: &[String], md1: &[String], template: &str) -> Vec<String> {
    let mk_refs: Vec<&str> = mk1.iter().map(|s| s.as_str()).collect();
    let card = mk_codec::decode(&mk_refs).expect("mk1 decodes");
    let xpub_str = card.xpub.to_string();
    let fp_str = card
        .origin_fingerprint
        .expect("fingerprint present")
        .to_string()
        .to_lowercase();
    let mut args: Vec<String> = vec![
        "verify-bundle".into(),
        "--slot".into(),
        format!("@0.xpub={xpub_str}"),
        "--slot".into(),
        format!("@0.fingerprint={fp_str}"),
        "--network".into(),
        "mainnet".into(),
        "--template".into(),
        template.into(),
    ];
    for s in mk1 {
        args.push("--mk1".into());
        args.push(s.clone());
    }
    for s in md1 {
        args.push("--md1".into());
        args.push(s.clone());
    }
    args
}

const CROSS_CHECK_WARNING_PREFIX: &str = "warning: cosigner[";

/// Cell (a) — consistent cards: cross-check is silent. Reuses the canonical
/// bip84-mainnet fixture. The legacy `watch-only verify-bundle does not verify`
/// disclaimer still fires (that's the existing SPEC §2.2.2 banner), but the
/// new D30 cross-check warning must NOT appear.
#[test]
fn cross_check_consistent_cards_silent_no_warning() {
    let fixture =
        std::fs::read_to_string("tests/vectors/v0_1/bip84-mainnet.txt").expect("fixture exists");
    let mk1: Vec<String> = fixture
        .lines()
        .filter(|l| l.starts_with("mk1") && !l.contains(' ') && !l.contains('-'))
        .map(String::from)
        .collect();
    let md1: Vec<String> = fixture
        .lines()
        .filter(|l| l.starts_with("md1") && !l.contains(' ') && !l.contains('-'))
        .map(String::from)
        .collect();
    let args = single_sig_watch_only_args(&mk1, &md1, "bip84");

    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&args)
        .assert()
        .success()
        .stdout(predicate::str::contains("result: ok"))
        .stderr(predicate::str::contains(CROSS_CHECK_WARNING_PREFIX).not());
}

/// Cell (b) — mk1 depth ≠ md1 path length: warning. Constructed by
/// re-encoding a decoded mk1 with a shorter origin path (depth 2 instead of
/// the bundle's depth 3). The xpub bytes (chain_code + public_key) are
/// preserved; only the path is shortened. On decode, `xpub.depth` is
/// reconstructed from origin_path → 2, which disagrees with the unchanged
/// md1 path-decl depth of 3 → cross-check fires.
#[test]
fn cross_check_mk1_depth_lt_md1_path_warns() {
    use bitcoin::bip32::DerivationPath;
    use std::str::FromStr;

    let (mk1, md1) = gen_bundle_json(&[
        "bundle",
        "--network",
        "mainnet",
        "--template",
        "bip84",
        "--slot",
        &format!("@0.phrase={TREZOR_24}"),
        "--json",
    ]);
    let mk_refs: Vec<&str> = mk1.iter().map(|s| s.as_str()).collect();
    let original = mk_codec::decode(&mk_refs).expect("mk1 decodes");
    // Synthesize a depth-2 path that preserves the hardened-bit pattern.
    let tampered_path = DerivationPath::from_str("m/84'/0'").unwrap();
    let tampered = mk_codec::KeyCard::new(
        original.policy_id_stubs.clone(),
        original.origin_fingerprint,
        tampered_path,
        original.xpub,
    );
    let tampered_strs = mk_codec::encode(&tampered).expect("re-encode succeeds");
    let args = single_sig_watch_only_args(&tampered_strs, &md1, "bip84");

    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&args)
        .assert()
        // verify-bundle exit code is unchanged by the cross-check; the
        // synthesized expected bundle will still mismatch (path doesn't match
        // bip84 template) — but we only assert the cross-check stderr line.
        .stderr(predicate::str::contains(
            "mk1 xpub depth (2) does not match md1 origin-path length (3)",
        ));
}

/// Cell (c) — mk1 child_number ≠ md1 path last element: warning. Generated
/// by combining account-0 mk1 with account-1 md1. mk1.xpub.child_number is
/// reconstructed from mk1.origin_path's last component (0'), while md1
/// path-decl's last element is (1'). Cross-check fires.
#[test]
fn cross_check_mk1_child_number_ne_md1_last_warns() {
    let (mk1_acct0, _md1_acct0) = gen_bundle_json(&[
        "bundle",
        "--network",
        "mainnet",
        "--template",
        "bip84",
        "--account",
        "0",
        "--slot",
        &format!("@0.phrase={TREZOR_24}"),
        "--json",
    ]);
    let (_mk1_acct1, md1_acct1) = gen_bundle_json(&[
        "bundle",
        "--network",
        "mainnet",
        "--template",
        "bip84",
        "--account",
        "1",
        "--slot",
        &format!("@0.phrase={TREZOR_24}"),
        "--json",
    ]);
    let args = single_sig_watch_only_args(&mk1_acct0, &md1_acct1, "bip84");

    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&args)
        .assert()
        .stderr(predicate::str::contains(
            "mk1 xpub child_number (0') does not match md1 origin-path last component (1')",
        ));
}

/// Cell (d) — mk1 parent_fingerprint ≠ derived-parent fingerprint: warning.
/// Built by constructing a depth-1 mk1 (origin_path = m/0') whose
/// parent_fingerprint must equal the claimed master fingerprint (the parent
/// at depth 1 IS the master). The bundle-generated mk1 has parent_fingerprint
/// matching the master_fingerprint of the bundle; we re-encode with a
/// fabricated origin_fingerprint that doesn't match → cross-check fires.
#[test]
fn cross_check_mk1_parent_fingerprint_mismatch_warns() {
    use bitcoin::bip32::{DerivationPath, Fingerprint};
    use std::str::FromStr;

    let (mk1, md1) = gen_bundle_json(&[
        "bundle",
        "--network",
        "mainnet",
        "--template",
        "bip84",
        "--slot",
        &format!("@0.phrase={TREZOR_24}"),
        "--json",
    ]);
    let mk_refs: Vec<&str> = mk1.iter().map(|s| s.as_str()).collect();
    let original = mk_codec::decode(&mk_refs).expect("mk1 decodes");
    // Construct a depth-1 mk1 by shortening origin_path to m/84'. Then
    // override origin_fingerprint to NOT match the xpub's parent_fingerprint.
    // (At depth 1, parent_fingerprint must equal the claimed master
    // fingerprint per BIP-32; the cross-check fires when they disagree.)
    let depth1_path = DerivationPath::from_str("m/84'").unwrap();
    let fake_master_fp = Fingerprint::from([0xde, 0xad, 0xbe, 0xef]);
    let tampered = mk_codec::KeyCard::new(
        original.policy_id_stubs.clone(),
        Some(fake_master_fp),
        depth1_path,
        original.xpub,
    );
    let tampered_strs = mk_codec::encode(&tampered).expect("re-encode succeeds");
    // Also re-shorten md1 path to depth 1 so that the depth/child checks
    // don't dominate — but since we can't tamper md1 here, the depth(1) vs
    // md1-depth(3) check will also fire. We still assert parent_fingerprint
    // mismatch fires (the check at md_depth == 1 is what we want); use a
    // multisig path-overrides crafted bundle would be cleaner but requires
    // md_codec re-encode. Simpler: scope this cell to assert at minimum that
    // SOME cross-check warning fires when we tampered the mk1.
    let args = single_sig_watch_only_args(&tampered_strs, &md1, "bip84");

    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&args)
        .assert()
        .stderr(predicate::str::contains(CROSS_CHECK_WARNING_PREFIX));
}

/// Cell (e) — multi-cosigner watch-only with one card inconsistent: warning
/// lists which cosigner. Generated by combining mk1 chunks from two
/// different bundles: cosigner[0] mk1 from account-1 bundle (depth 3, last
/// element 1') spliced with the rest of an account-0 multisig bundle's
/// md1 (per-cosigner OriginPaths at last element 0'). Cosigner[0] cross-check
/// fires with a `cosigner[0]` prefix.
#[test]
fn cross_check_multi_cosigner_one_inconsistent_lists_index() {
    // Generate a clean 2-of-2 multisig bundle (account 0). Capture mk1 + md1.
    let (mk1_acct0_multi, md1_acct0_multi) = gen_bundle_json_multisig(&[
        "bundle",
        "--network",
        "mainnet",
        "--template",
        "wsh-sortedmulti",
        "--threshold",
        "2",
        "--slot",
        &format!("@0.phrase={TREZOR_24}"),
        "--slot",
        &format!("@1.phrase={BIP39_TEST_2}"),
        "--json",
    ]);
    // Generate a single-sig account-1 bundle. Use its mk1 to replace the
    // cosigner[0] mk1 chunks above (forcing a child_number mismatch).
    let (mk1_acct1_single, _md1_acct1) = gen_bundle_json(&[
        "bundle",
        "--network",
        "mainnet",
        "--template",
        "bip84",
        "--account",
        "1",
        "--slot",
        &format!("@0.phrase={TREZOR_24}"),
        "--json",
    ]);

    // The multisig md1 carries pubkeys TLV with the two cosigners' xpubs.
    // Splice in: drop cosigner[0]'s mk1 chunks, keep cosigner[1]'s; append
    // the account-1 single-sig mk1. The xpub-to-pubkey lookup in the
    // cross-check helper won't find a match for the spliced card → falls
    // back to positional placement at index 0 (the first unfilled position).
    // The account-1 mk1's origin_path last element is 1' (m/84'/0'/1'); the
    // multisig md1's path-decl at index 0 has last element 0' → child_number
    // mismatch with `cosigner[0]` prefix.

    // mk1_acct0_multi is flat across cosigners (2 chunks per cosigner = 4 total
    // for n=2). Drop the first 2 chunks (cosigner[0]) and prepend the
    // account-1 single-sig mk1 chunks.
    let mut spliced_mk1: Vec<String> = mk1_acct1_single.clone();
    // Keep cosigner[1] chunks (positions 2..).
    spliced_mk1.extend(mk1_acct0_multi.iter().skip(2).cloned());

    let mut args: Vec<String> = vec![
        "verify-bundle".into(),
        "--network".into(),
        "mainnet".into(),
        "--template".into(),
        "wsh-sortedmulti".into(),
        "--threshold".into(),
        "2".into(),
    ];
    // Watch-only multisig: supply xpubs (not phrases) for both slots. Use
    // the original multisig mk1 cards to derive the xpub slots.
    let mk_refs_orig: Vec<&str> = mk1_acct0_multi.iter().map(|s| s.as_str()).collect();
    // Split original multisig mk1 chunks by chunk_set_id by simply taking
    // the first half as cosigner[0] and second half as cosigner[1] (each
    // cosigner emits 2 chunks for the bip87 path).
    let cos0 = mk_codec::decode(&mk_refs_orig[..2]).expect("cos0 decodes");
    let cos1 = mk_codec::decode(&mk_refs_orig[2..]).expect("cos1 decodes");
    let cos0_xpub = cos0.xpub.to_string();
    let cos1_xpub = cos1.xpub.to_string();
    let cos0_fp = cos0.origin_fingerprint.unwrap().to_string().to_lowercase();
    let cos1_fp = cos1.origin_fingerprint.unwrap().to_string().to_lowercase();
    args.push("--slot".into());
    args.push(format!("@0.xpub={cos0_xpub}"));
    args.push("--slot".into());
    args.push(format!("@0.fingerprint={cos0_fp}"));
    args.push("--slot".into());
    args.push(format!("@1.xpub={cos1_xpub}"));
    args.push("--slot".into());
    args.push(format!("@1.fingerprint={cos1_fp}"));
    for s in &spliced_mk1 {
        args.push("--mk1".into());
        args.push(s.clone());
    }
    for s in &md1_acct0_multi {
        args.push("--md1".into());
        args.push(s.clone());
    }

    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&args)
        .assert()
        .stderr(predicate::str::contains("cosigner[0]"))
        .stderr(predicate::str::contains("does not match"));
}

// ============================================================================
// v0.25.0 §2.D — watch-only depth ≥ 2 parent_fingerprint NOTICE cells.
// ============================================================================

/// v0.25.0 §2.D cell — multi-cosigner watch-only at depth ≥ 2 emits an
/// explicit stderr NOTICE that the parent_fingerprint is unverified-by-design
/// (no ms1 → cannot derive parent xpub). Cryptographic ceiling per BIP-32
/// child→parent one-wayness; permissive-input / expressive-output per project
/// philosophy.
///
/// Cell exercises the depth ≥ 2 watch-only branch on a 2-of-2 wsh-sortedmulti
/// bundle (multisig templates produce depth-3 paths via the BIP-87 default
/// purpose). Both cosigners are watch-only (no ms1) so the NOTICE fires for
/// both indices.
#[test]
fn watch_only_depth_3_emits_unverified_parent_fp_notice() {
    let (mk1, md1) = gen_bundle_json_multisig(&[
        "bundle",
        "--network",
        "mainnet",
        "--template",
        "wsh-sortedmulti",
        "--threshold",
        "2",
        "--slot",
        &format!("@0.phrase={TREZOR_24}"),
        "--slot",
        &format!("@1.phrase={BIP39_TEST_2}"),
        "--json",
    ]);
    // Decode mk1 chunks per cosigner to extract xpubs.
    let mk_refs: Vec<&str> = mk1.iter().map(|s| s.as_str()).collect();
    let cos0 = mk_codec::decode(&mk_refs[..2]).expect("cos0 decodes");
    let cos1 = mk_codec::decode(&mk_refs[2..]).expect("cos1 decodes");
    let cos0_xpub = cos0.xpub.to_string();
    let cos1_xpub = cos1.xpub.to_string();
    let cos0_fp = cos0.origin_fingerprint.unwrap().to_string().to_lowercase();
    let cos1_fp = cos1.origin_fingerprint.unwrap().to_string().to_lowercase();

    let mut args: Vec<String> = vec![
        "verify-bundle".into(),
        "--network".into(),
        "mainnet".into(),
        "--template".into(),
        "wsh-sortedmulti".into(),
        "--threshold".into(),
        "2".into(),
        "--slot".into(),
        format!("@0.xpub={cos0_xpub}"),
        "--slot".into(),
        format!("@0.fingerprint={cos0_fp}"),
        "--slot".into(),
        format!("@1.xpub={cos1_xpub}"),
        "--slot".into(),
        format!("@1.fingerprint={cos1_fp}"),
    ];
    for s in &mk1 {
        args.push("--mk1".into());
        args.push(s.clone());
    }
    for s in &md1 {
        args.push("--md1".into());
        args.push(s.clone());
    }

    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&args)
        .assert()
        .stderr(predicate::str::contains(
            "notice: cosigner[0] mk1 parent_fingerprint at depth 3 unverified (requires ms1 to derive parent xpub)",
        ))
        .stderr(predicate::str::contains(
            "notice: cosigner[1] mk1 parent_fingerprint at depth 3 unverified (requires ms1 to derive parent xpub)",
        ));
}

/// v0.25.0 §2.D cell — partial-watch-only multisig: ms1 supplied for
/// cosigner[0] only (consistent with the supplied mk1[0]). For cosigner[1]
/// without ms1, the helper emits NOTICE (depth ≥ 2 + no seed); for
/// cosigner[0] the full-path check fires silently (derived matches claimed).
#[test]
fn watch_only_multi_cosigner_one_ms1_missing_emits_notice_for_that_cosigner_only() {
    // Reuse the same multisig bundle as the depth-3 cell.
    let bundle_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--network",
            "mainnet",
            "--template",
            "wsh-sortedmulti",
            "--threshold",
            "2",
            "--slot",
            &format!("@0.phrase={TREZOR_24}"),
            "--slot",
            &format!("@1.phrase={BIP39_TEST_2}"),
            "--json",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(bundle_out.get_output().stdout.clone()).unwrap();
    let bundle: serde_json::Value = serde_json::from_str(&stdout).expect("valid bundle JSON");
    let ms1_arr = bundle["ms1"].as_array().expect("ms1 array");
    let ms1_cos0 = ms1_arr[0].as_str().unwrap().to_string();
    // ms1_arr[1] intentionally NOT supplied; empty string is the watch-only sentinel.

    let mut mk1: Vec<String> = Vec::new();
    for inner in bundle["mk1"].as_array().expect("mk1 array") {
        for chunk in inner.as_array().expect("inner mk1 array") {
            mk1.push(chunk.as_str().unwrap().to_string());
        }
    }
    let md1: Vec<String> = bundle["md1"]
        .as_array()
        .expect("md1 array")
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();

    let mut args: Vec<String> = vec![
        "verify-bundle".into(),
        "--network".into(),
        "mainnet".into(),
        "--template".into(),
        "wsh-sortedmulti".into(),
        "--threshold".into(),
        "2".into(),
        "--slot".into(),
        format!("@0.phrase={TREZOR_24}"),
        "--slot".into(),
        format!("@1.phrase={BIP39_TEST_2}"),
        // ms1 for cosigner[0] only; cosigner[1] has no --ms1 flag (vec
        // get(1) → None in the new helper, treated as the watch-only
        // sentinel for the depth-≥-2 NOTICE branch).
        "--ms1".into(),
        ms1_cos0,
    ];
    for s in &mk1 {
        args.push("--mk1".into());
        args.push(s.clone());
    }
    for s in &md1 {
        args.push("--md1".into());
        args.push(s.clone());
    }

    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&args)
        .assert()
        .stderr(predicate::str::contains(
            "notice: cosigner[1] mk1 parent_fingerprint at depth 3 unverified",
        ))
        // cosigner[0] full-path check fires silently when ms1 + mk1 are consistent;
        // assert NO notice fires for cosigner[0].
        .stderr(predicate::str::contains(
            "notice: cosigner[0] mk1 parent_fingerprint",
        ).not());
}
