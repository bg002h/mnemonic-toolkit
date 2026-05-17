//! v0.20.0 F1 regression corpus — `verify-bundle` round-trip for multi-cosigner
//! bundles. Pre-fix, all cosigners' mk1 chunks share `chunk_set_id` because the
//! four n>1 derivation sites (synthesize.rs `:246` synthesize_descriptor /
//! `:391` synthesize_multisig_full / `:561` synthesize_multisig_watch_only /
//! `:754` synthesize_unified) all derived csi from a shared `policy_id` stub.
//! Post-fix, each cosigner's chunks get a distinct csi derived from
//! `<loop_var>.xpub.fingerprint().to_bytes()`.
//!
//! Cells 1-3 round-trip canonical wsh-sortedmulti (template mode → synthesize_unified)
//! and non-canonical wsh(andor(...)) (descriptor mode → synthesize_descriptor)
//! via `verify-bundle --bundle-json` and via flat `--mk1` repetition. Cell 4 is
//! a `--self-check` sanity guard (Phase 0 gap-B: --self-check bypasses csi-grouping
//! and passes pre-fix; kept as sanity, not regression guard). Cell 5 pins single-sig
//! n=1 byte-identity against a pre-fix hardcoded fixture (single-sig path is
//! unchanged: synthesize_unified:737 and synthesize_descriptor:228 both pass `&stub`
//! not `&stubs[0]`).

use assert_cmd::Command;
use std::io::Write;

const TREZOR_12_ZERO: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const BIP39_TEST_2: &str =
    "legal winner thank year wave sausage worth useful legal winner thank yellow";
const BIP39_TEST_3: &str =
    "letter advice cage absurd amount doctor acoustic avoid letter advice cage above";

/// Cell 1 — canonical 2-of-2 wsh-sortedmulti via `--template` (synthesize_unified:754 path).
/// Emit bundle JSON, pipe back through verify-bundle --bundle-json.
/// Pre-fix: `result: mismatch` with `mk1_decode[0..1]: fail`.
/// Post-fix: `result: ok`.
#[test]
fn canonical_wsh_sortedmulti_round_trips_via_bundle_json() {
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
            "--multisig-path-family",
            "bip48",
            "--slot",
            &format!("@0.phrase={TREZOR_12_ZERO}"),
            "--slot",
            &format!("@1.phrase={BIP39_TEST_2}"),
            "--json",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(bundle_out.get_output().stdout.clone()).unwrap();

    let tmpdir = tempfile::tempdir().unwrap();
    let path = tmpdir.path().join("bundle.json");
    std::fs::File::create(&path)
        .unwrap()
        .write_all(stdout.as_bytes())
        .unwrap();

    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "verify-bundle",
            "--network",
            "mainnet",
            "--template",
            "wsh-sortedmulti",
            "--threshold",
            "2",
            "--multisig-path-family",
            "bip48",
            "--slot",
            &format!("@0.phrase={TREZOR_12_ZERO}"),
            "--slot",
            &format!("@1.phrase={BIP39_TEST_2}"),
            "--bundle-json",
            path.to_str().unwrap(),
        ])
        .assert()
        .success();
}

/// Cell 2 — same 2-of-2 bundle but verify via flat `--mk1` argv repetition
/// (`--mk1 c0 --mk1 c1 --mk1 c2 --mk1 c3`). Confirms the alternative intake
/// path works once csi is unique per cosigner.
#[test]
fn canonical_wsh_sortedmulti_round_trips_via_flat_mk1_repetition() {
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
            "--multisig-path-family",
            "bip48",
            "--slot",
            &format!("@0.phrase={TREZOR_12_ZERO}"),
            "--slot",
            &format!("@1.phrase={BIP39_TEST_2}"),
            "--json",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(bundle_out.get_output().stdout.clone()).unwrap();
    let bundle: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    // Extract flat mk1 chunk list + ms1 + md1.
    let mk1_outer = bundle["mk1"].as_array().expect("mk1 array");
    let mut mk1_flat: Vec<String> = Vec::new();
    for inner in mk1_outer {
        for chunk in inner.as_array().expect("inner mk1 array") {
            mk1_flat.push(chunk.as_str().unwrap().to_string());
        }
    }
    let ms1_arr: Vec<String> = bundle["ms1"]
        .as_array()
        .expect("ms1 array")
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    let md1_arr: Vec<String> = bundle["md1"]
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
        "--multisig-path-family".into(),
        "bip48".into(),
        "--slot".into(),
        format!("@0.phrase={TREZOR_12_ZERO}"),
        "--slot".into(),
        format!("@1.phrase={BIP39_TEST_2}"),
    ];
    for m in &mk1_flat {
        args.push("--mk1".into());
        args.push(m.clone());
    }
    for m in &ms1_arr {
        args.push("--ms1".into());
        args.push(m.clone());
    }
    for m in &md1_arr {
        args.push("--md1".into());
        args.push(m.clone());
    }

    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&args)
        .assert()
        .success();
}

/// Cell 3 — non-canonical 2-of-2 `wsh(andor(pkh(@0),after(N),pk(@1)))` via
/// `--descriptor` (descriptor mode → synthesize_descriptor:246 path). Exercises
/// v0.19.0 default-path inference combined with F1 csi-uniqueness fix.
#[test]
fn non_canonical_wsh_andor_round_trips_via_bundle_json() {
    let descriptor = "wsh(andor(pkh(@0),after(12000000),pk(@1)))";
    let bundle_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            descriptor,
            "--network",
            "mainnet",
            "--account",
            "0",
            "--slot",
            &format!("@0.phrase={TREZOR_12_ZERO}"),
            "--slot",
            &format!("@1.phrase={BIP39_TEST_2}"),
            "--json",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(bundle_out.get_output().stdout.clone()).unwrap();

    let tmpdir = tempfile::tempdir().unwrap();
    let path = tmpdir.path().join("bundle.json");
    std::fs::File::create(&path)
        .unwrap()
        .write_all(stdout.as_bytes())
        .unwrap();

    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "verify-bundle",
            "--descriptor",
            descriptor,
            "--network",
            "mainnet",
            "--account",
            "0",
            "--slot",
            &format!("@0.phrase={TREZOR_12_ZERO}"),
            "--slot",
            &format!("@1.phrase={BIP39_TEST_2}"),
            "--bundle-json",
            path.to_str().unwrap(),
        ])
        .assert()
        .success();
}

/// Cell 4 — `--self-check` sanity cell. Per Phase 0 gap-B, `--self-check` for
/// multisig at `bundle.rs:1478-1504` decodes each per-cosigner chunk-vector
/// separately via `mk_codec::decode(&strs)` and bypasses the csi-grouping logic
/// in `verify_bundle::emit_multisig_checks`. Self-check passes today (pre-fix)
/// AND post-fix. This cell pins that self-check keeps working through the F1
/// change. NOT a regression guard for F1.
#[test]
fn self_check_canonical_multisig_passes_both_pre_and_post_fix() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--network",
            "mainnet",
            "--template",
            "wsh-sortedmulti",
            "--threshold",
            "2",
            "--multisig-path-family",
            "bip48",
            "--slot",
            &format!("@0.phrase={TREZOR_12_ZERO}"),
            "--slot",
            &format!("@1.phrase={BIP39_TEST_2}"),
            "--self-check",
        ])
        .assert()
        .success();
}

/// Cell 5 — single-sig n=1 forward-compat guard. Single-sig path in both
/// `synthesize_unified:725-739` and `synthesize_descriptor:216-230` derives csi
/// from `&stub` (bare), NOT `&stubs[0]` — so the F1 fix does not touch single-sig
/// csi. Bundle output bytes must remain byte-identical against a pre-fix capture.
///
/// Fixture provenance: captured 2026-05-17 from pre-fix toolkit (commit dbd3728)
/// via:
/// ```
/// mnemonic bundle --network mainnet --template bip84 \
///     --slot '@0.phrase=...' --json --no-engraving-card
/// ```
#[test]
fn single_sig_csi_unchanged_byte_identical_to_pre_fix_fixture() {
    const PRE_FIX_SINGLE_SIG_BUNDLE_JSON: &str = include_str!(
        "fixtures/v0_20_0_single_sig_bip84_bundle.json"
    );

    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--network",
            "mainnet",
            "--template",
            "bip84",
            "--slot",
            &format!("@0.phrase={TREZOR_12_ZERO}"),
            "--json",
            "--no-engraving-card",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(
        stdout.trim(),
        PRE_FIX_SINGLE_SIG_BUNDLE_JSON.trim(),
        "single-sig n=1 csi-derivation must be unchanged by F1; bundle bytes diverged from pre-fix fixture"
    );
}

/// Cell 6 — 3-cosigner non-canonical wsh(andor(...)) round-trip (the
/// non_canonical_default_path_self_check_round_trips test's descriptor, but
/// via verify-bundle --bundle-json rather than --self-check, to expose the
/// csi-collision in descriptor mode for n>2 cosigners).
#[test]
fn non_canonical_3_of_3_wsh_andor_round_trips_via_bundle_json() {
    let descriptor = "wsh(andor(pkh(@0),after(12000000),or_i(and_v(v:pkh(@1),older(4032)),and_v(v:pkh(@2),older(32768)))))";
    let bundle_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            descriptor,
            "--network",
            "mainnet",
            "--language",
            "english",
            "--account",
            "0",
            "--slot",
            &format!("@0.phrase={TREZOR_12_ZERO}"),
            "--slot",
            &format!("@1.phrase={BIP39_TEST_2}"),
            "--slot",
            &format!("@2.phrase={BIP39_TEST_3}"),
            "--json",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(bundle_out.get_output().stdout.clone()).unwrap();

    let tmpdir = tempfile::tempdir().unwrap();
    let path = tmpdir.path().join("bundle.json");
    std::fs::File::create(&path)
        .unwrap()
        .write_all(stdout.as_bytes())
        .unwrap();

    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "verify-bundle",
            "--descriptor",
            descriptor,
            "--network",
            "mainnet",
            "--language",
            "english",
            "--account",
            "0",
            "--slot",
            &format!("@0.phrase={TREZOR_12_ZERO}"),
            "--slot",
            &format!("@1.phrase={BIP39_TEST_2}"),
            "--slot",
            &format!("@2.phrase={BIP39_TEST_3}"),
            "--bundle-json",
            path.to_str().unwrap(),
        ])
        .assert()
        .success();
}

/// Cell 7 — v0.21.0 SPEC §5.8 per-slot ms1 emission regression guard.
/// Descriptor-mode 3-of-3 wsh(andor(...)) bundle with phrases supplied for ALL
/// three slots must emit `ms1[i]` populated for i ∈ {0, 1, 2} (no `""`
/// sentinels). Pre-v0.21.0 emitted `ms1 = ["ms1...", "", ""]` per the legacy
/// "v0.3 descriptor-mode contract" pinning to @0 only; SPEC §5.8 emission rule
/// now mandates per-slot emission. Exercises the full bundle → verify-bundle
/// --bundle-json round-trip per `[[feedback-verify-bundle-round-trip-per-phase-r0-scope]]`.
#[test]
fn descriptor_mode_3_of_3_emits_per_slot_ms1_post_v0_21() {
    let descriptor = "wsh(andor(pkh(@0),after(12000000),or_i(and_v(v:pkh(@1),older(4032)),and_v(v:pkh(@2),older(32768)))))";
    let bundle_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            descriptor,
            "--network",
            "mainnet",
            "--language",
            "english",
            "--account",
            "0",
            "--slot",
            &format!("@0.phrase={TREZOR_12_ZERO}"),
            "--slot",
            &format!("@1.phrase={BIP39_TEST_2}"),
            "--slot",
            &format!("@2.phrase={BIP39_TEST_3}"),
            "--json",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(bundle_out.get_output().stdout.clone()).unwrap();
    let bundle: serde_json::Value = serde_json::from_str(&stdout).expect("bundle is valid JSON");

    // SPEC §5.8 emission rule — ms1 dense vec of length N with every
    // phrase-bearing slot populated; no empty-string sentinels.
    let ms1 = bundle["ms1"].as_array().expect("ms1 is an array");
    assert_eq!(ms1.len(), 3, "descriptor-mode 3-cosigner bundle emits len-3 ms1");
    for (i, entry) in ms1.iter().enumerate() {
        let s = entry.as_str().expect("ms1[i] is a string");
        assert!(
            s.starts_with("ms1"),
            "ms1[{i}] must be a populated ms1 string per SPEC §5.8; got {s:?}"
        );
    }
    // The 3 entries must be distinct (each slot carries its own entropy bytes).
    assert_ne!(ms1[0], ms1[1]);
    assert_ne!(ms1[1], ms1[2]);
    assert_ne!(ms1[0], ms1[2]);

    // Round-trip via verify-bundle --bundle-json — all 3+6N = 21 checks must
    // pass (ms1_decode[0..2] + ms1_entropy_match[0..2] now all `ok`, not
    // `skipped: watch-only slot`).
    let tmpdir = tempfile::tempdir().unwrap();
    let path = tmpdir.path().join("bundle.json");
    std::fs::File::create(&path)
        .unwrap()
        .write_all(stdout.as_bytes())
        .unwrap();

    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "verify-bundle",
            "--descriptor",
            descriptor,
            "--network",
            "mainnet",
            "--language",
            "english",
            "--account",
            "0",
            "--slot",
            &format!("@0.phrase={TREZOR_12_ZERO}"),
            "--slot",
            &format!("@1.phrase={BIP39_TEST_2}"),
            "--slot",
            &format!("@2.phrase={BIP39_TEST_3}"),
            "--bundle-json",
            path.to_str().unwrap(),
        ])
        .assert()
        .success();
}
