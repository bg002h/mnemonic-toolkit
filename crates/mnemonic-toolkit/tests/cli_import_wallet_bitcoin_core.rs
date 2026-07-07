//! Phase 3 — Bitcoin Core `listdescriptors` parser integration tests.
//!
//! Per `design/IMPLEMENTATION_PLAN_wallet_import_v0_26_0.md` §3.2-§3.14. Tests
//! the library boundary via the CLI scaffold (`cmd/import_wallet.rs`) extended
//! in this phase to dispatch `--format bitcoin-core` to `BitcoinCoreParser`
//! and to expose a `--select-descriptor` filter for multi-entry blobs.
//!
//! Self-contained: no dependency on adjacent repos or external network. The
//! testnet fixture xpubs are reused from `cli_import_wallet_bsms.rs`; the
//! mainnet fixture xpubs are reused from `cli_export_wallet_jade.rs` to keep
//! the corpus internally consistent.

use assert_cmd::Command;
use miniscript::descriptor::checksum::Engine as ChecksumEngine;
use miniscript::{DefiniteDescriptorKey, Descriptor, DescriptorPublicKey};
use std::path::PathBuf;
use std::str::FromStr;

// ---- mainnet fixtures (lifted from cli_export_wallet_jade.rs) ----

const MAINNET_FP_A: &str = "b8688df1";
const MAINNET_XPUB_A: &str = "xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX";
const MAINNET_FP_B: &str = "28645006";
const MAINNET_XPUB_B: &str = "xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6";
const MAINNET_FP_C: &str = "5436d724";
const MAINNET_XPUB_C: &str = "xpub6Buxw9MmbkJr4iAw8SACNci2hQNuPCMwt9P7HkK62ZQAW9UcJaQ2bc6ARD892TToQQ9Rp6AHujHxBLXqAsvn5fRnLfnhKSRfz8qtaoyKUYx";

// ---- testnet fixtures (lifted from cli_import_wallet_bsms.rs) ----

const TESTNET_FP_A: &str = "704c7836";
const TESTNET_XPUB_A: &str = "tpubDEgS9fUEpucKatmvKAv21v8nViHxR6rsV7ohMWK4YjsWd4EWT3w8YzMgMEvNrDfsUANbid74WRFpr3Gym8UHBSLnqg6b1Lzvibw87cLSctC";
const TESTNET_FP_B: &str = "97139860";
const TESTNET_XPUB_B: &str = "tpubDFiXyf7zmBhQrSHoAQB6SmMpF3rfSihAxQGMdQUtZfE8HWHkWLLNLTiYpMzvHnFiTmuUSYieHUYv4tFguzmiHeDrYV8TtWGCWt5qpqox4w3";

/// Compute BIP-380 checksum for a descriptor body (no trailing `#xxx`).
fn checksum(desc_without_hash: &str) -> String {
    let mut eng = ChecksumEngine::new();
    eng.input(desc_without_hash).expect("ascii-only");
    eng.checksum()
}

/// Build a single-entry Bitcoin Core blob with one descriptor body. Computes
/// the BIP-380 checksum dynamically. Adds optional dropped-fields per arg.
fn build_core_single(
    desc: &str,
    active: bool,
    internal: bool,
    range: Option<(u64, u64)>,
    include_dropped: bool,
) -> String {
    let cs = checksum(desc);
    let range_str = match range {
        Some((lo, hi)) => format!(",\n      \"range\": [{lo}, {hi}]"),
        None => String::new(),
    };
    let dropped = if include_dropped {
        ",\n      \"timestamp\": \"now\",\n      \"next\": 5,\n      \"next_index\": 5"
    } else {
        ""
    };
    format!(
        "{{\n  \"wallet_name\": \"test\",\n  \"descriptors\": [\n    {{\n      \"desc\": \"{desc}#{cs}\",\n      \"active\": {active},\n      \"internal\": {internal}{range_str}{dropped}\n    }}\n  ]\n}}\n"
    )
}

/// Build a multi-entry Bitcoin Core blob from per-entry descriptors. Each
/// entry pre-built with `desc`, `active`, `internal`, `range`.
struct CoreEntry<'a> {
    desc: &'a str,
    active: bool,
    internal: bool,
}

fn build_core_multi(entries: &[CoreEntry<'_>]) -> String {
    let mut body = String::new();
    body.push_str("{\n  \"wallet_name\": \"multi\",\n  \"descriptors\": [\n");
    for (i, e) in entries.iter().enumerate() {
        let cs = checksum(e.desc);
        body.push_str("    {\n");
        body.push_str(&format!("      \"desc\": \"{}#{}\",\n", e.desc, cs));
        body.push_str(&format!("      \"active\": {},\n", e.active));
        body.push_str(&format!("      \"internal\": {},\n", e.internal));
        body.push_str("      \"range\": [0, 1000]\n");
        if i + 1 < entries.len() {
            body.push_str("    },\n");
        } else {
            body.push_str("    }\n");
        }
    }
    body.push_str("  ]\n}\n");
    body
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from("tests/fixtures/wallet_import").join(name)
}

/// Run import-wallet with bitcoin-core blob piped on stdin.
fn run_core_stdin(blob: &str) -> assert_cmd::assert::Assert {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["import-wallet", "--blob", "-", "--format", "bitcoin-core"])
        .write_stdin(blob.to_string())
        .assert()
}

/// Run import-wallet with bitcoin-core blob piped on stdin and a
/// `--select-descriptor` value.
fn run_core_stdin_select(blob: &str, select: &str) -> assert_cmd::assert::Assert {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--blob",
            "-",
            "--format",
            "bitcoin-core",
            "--select-descriptor",
            select,
        ])
        .write_stdin(blob.to_string())
        .assert()
}

/// Run import-wallet against a fixture file with select.
fn run_core_file_select(path: &PathBuf, select: &str) -> assert_cmd::assert::Assert {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["import-wallet", "--blob"])
        .arg(path)
        .args(["--format", "bitcoin-core", "--select-descriptor", select])
        .assert()
}

// ============================================================================
// bitcoin-core-receive-change-pair-merge (SPEC/PLAN §8) — shared oracle
// helpers. The anti-C1 funds-safety oracle: independently derive addresses
// from a descriptor via rust-miniscript directly (mirrors
// `prop_backup_restore_roundtrip.rs::derive_receive`), NEVER by re-deriving
// through the toolkit's own synthesis path (that would be tautological).
// ============================================================================

/// Derive `count` addresses at increasing indices from a NON-multipath,
/// single-path descriptor (used against the ORIGINAL split `/N/*` entries,
/// which are never multipath).
fn derive_addresses(desc: &str, count: u32, network: bitcoin::Network) -> Vec<String> {
    let d = Descriptor::<DescriptorPublicKey>::from_str(desc)
        .unwrap_or_else(|e| panic!("descriptor must parse: {desc}: {e}"));
    assert!(
        !d.is_multipath(),
        "derive_addresses expects a single-path descriptor: {desc}"
    );
    (0..count)
        .map(|i| {
            let def: Descriptor<DefiniteDescriptorKey> = if d.has_wildcard() {
                d.clone().derive_at_index(i).unwrap()
            } else {
                Descriptor::<DefiniteDescriptorKey>::try_from(d.clone()).unwrap()
            };
            def.address(network).unwrap().to_string()
        })
        .collect()
}

/// Derive `count` addresses at increasing indices from chain `chain` (0 =
/// external/receive, 1 = internal/change) of a MULTIPATH `<a;b>/*`
/// descriptor (used against the MERGED bundle's `descriptor` field).
fn derive_multipath_chain_addresses(
    desc: &str,
    chain: usize,
    count: u32,
    network: bitcoin::Network,
) -> Vec<String> {
    let d = Descriptor::<DescriptorPublicKey>::from_str(desc)
        .unwrap_or_else(|e| panic!("descriptor must parse: {desc}: {e}"));
    assert!(d.is_multipath(), "expected a multipath descriptor: {desc}");
    let mut singles = d.into_single_descriptors().unwrap();
    assert!(
        chain < singles.len(),
        "chain {chain} out of range ({} single descriptors): {desc}",
        singles.len()
    );
    let single = singles.remove(chain);
    (0..count)
        .map(|i| {
            let def: Descriptor<DefiniteDescriptorKey> = single.clone().derive_at_index(i).unwrap();
            def.address(network).unwrap().to_string()
        })
        .collect()
}

/// Round-trip a (possibly-merged) descriptor through `bundle --descriptor`
/// (concrete-descriptor mode, §4.3's `bundle_run_concrete_descriptor` path —
/// accepts inline `[fp/path]xpub` keys with or without a trailing `#csum`)
/// to synthesize md1/mk1 cards, then feeds those cards back through
/// `verify-bundle --descriptor` and asserts `result: ok`. SPEC §8.4/§8.10 —
/// a secondary regression net alongside (never a substitute for) the
/// address-independent-derivation oracle above.
fn assert_descriptor_verify_bundle_ok(descriptor_with_csum: &str, network: &str) {
    let bundle_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            descriptor_with_csum,
            "--network",
            network,
            "--json",
        ])
        .assert()
        .success();
    let v: serde_json::Value =
        serde_json::from_slice(&bundle_out.get_output().stdout).expect("valid bundle JSON");

    let mut args: Vec<String> = vec![
        "verify-bundle".into(),
        "--descriptor".into(),
        descriptor_with_csum.into(),
        "--network".into(),
        network.into(),
    ];
    for chunk in v["md1"].as_array().expect("md1 array") {
        args.push("--md1".into());
        args.push(chunk.as_str().unwrap().to_string());
    }
    // `mk1` is `MkField`: `Single(Vec<String>)` (flat chunk array, single-sig,
    // n==1) or `Multi(Vec<Vec<String>>)` (array-of-arrays, one inner array per
    // cosigner, n>=2) — untagged serde shape. Handle both uniformly.
    for entry in v["mk1"].as_array().expect("mk1 array") {
        match entry {
            serde_json::Value::String(s) => {
                args.push("--mk1".into());
                args.push(s.clone());
            }
            serde_json::Value::Array(inner) => {
                for chunk in inner {
                    args.push("--mk1".into());
                    args.push(chunk.as_str().expect("mk1 chunk is a string").to_string());
                }
            }
            other => panic!("unexpected mk1 entry shape: {other}"),
        }
    }
    args.push("--json".into());

    let verify_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&args)
        .assert()
        .success();
    let verify_val: serde_json::Value =
        serde_json::from_slice(&verify_out.get_output().stdout).expect("valid verify-bundle JSON");
    assert_eq!(
        verify_val["result"].as_str(),
        Some("ok"),
        "verify-bundle must return result=ok on the merged-pair bundle; got {verify_val}"
    );
}

// ============================================================================
// §3.2 — core_single_descriptor_wpkh_happy_path
// ============================================================================

#[test]
fn core_single_descriptor_wpkh_happy_path() {
    let desc = format!("wpkh([{MAINNET_FP_A}/84'/0'/0']{MAINNET_XPUB_A}/<0;1>/*)");
    let blob = build_core_single(&desc, true, false, Some((0, 1000)), false);
    let out = run_core_stdin(&blob).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("bundles=1"), "stdout: {stdout}");
    assert!(stdout.contains("cosigners=1"), "stdout: {stdout}");
    assert!(stdout.contains("network=mainnet"), "stdout: {stdout}");
    assert!(stdout.contains("entropy=none"), "stdout: {stdout}");
    // ParsedImport.threshold is `None` for single-sig (no thresh/multi).
    assert!(stdout.contains("threshold=none"), "stdout: {stdout}");
    // Core entries set bsms_audit=None.
    assert!(stdout.contains("bsms_audit=none"), "stdout: {stdout}");
}

// ============================================================================
// descriptor-origin-extraction-dedup — h-form hardened origin accepted
// ============================================================================

#[test]
fn core_single_descriptor_hform_hardened_path_accepted() {
    // FOLLOWUP descriptor-origin-extraction-dedup + import-parser-hform-origin-
    // tolerance: an `h`-form hardened origin (`84h/0h/0h`, as some Core/Sparrow
    // exports emit) must parse identically to the apostrophe form. Before the
    // dedup the bitcoin-core parser's apostrophe-only `origin_capture_regex`
    // refused it ("no origin annotations in descriptor") even though the upstream
    // placeholder-build + descriptor-parse already accept h-form; routing
    // `build_slot_fields` through the canonical h-form-widened `key_regex` fixes
    // it. RED against the pre-dedup binary, GREEN after.
    let desc = format!("wpkh([{MAINNET_FP_A}/84h/0h/0h]{MAINNET_XPUB_A}/<0;1>/*)");
    let blob = build_core_single(&desc, true, false, Some((0, 1000)), false);
    let out = run_core_stdin(&blob).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("bundles=1"), "stdout: {stdout}");
    assert!(stdout.contains("cosigners=1"), "stdout: {stdout}");
    assert!(stdout.contains("network=mainnet"), "stdout: {stdout}");
}

// ============================================================================
// §3.3 — core_multi_descriptor_emit_all
// ============================================================================

#[test]
fn core_multi_descriptor_emit_all() {
    // 4 entries: BIP-84 receive + change, BIP-49 receive + change.
    // Cycle A: a FIXED use-site step (`/0/*`, `/1/*`) is un-representable in
    // md1 and now rejects at lex (residue-reject floor). This cell's PURPOSE
    // is the 4-entry emit-all count, which is orthogonal to the collapse
    // regression (covered by dedicated reject tests) — swap to per-key-
    // identical `<0;1>/*` multipath entries to preserve the entry count
    // without relying on the fixed-step form (plan-R0 M-a).
    let d0 = format!("wpkh([{MAINNET_FP_A}/84'/0'/0']{MAINNET_XPUB_A}/<0;1>/*)");
    let d1 = d0.clone();
    let d2 = format!("sh(wpkh([{MAINNET_FP_B}/49'/0'/0']{MAINNET_XPUB_B}/<0;1>/*))");
    let d3 = d2.clone();
    let blob = build_core_multi(&[
        CoreEntry {
            desc: &d0,
            active: true,
            internal: false,
        },
        CoreEntry {
            desc: &d1,
            active: true,
            internal: true,
        },
        CoreEntry {
            desc: &d2,
            active: false,
            internal: false,
        },
        CoreEntry {
            desc: &d3,
            active: false,
            internal: true,
        },
    ]);
    let out = run_core_stdin_select(&blob, "all").success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("bundles=4"), "stdout: {stdout}");
}

// ============================================================================
// §3.4 — core_select_descriptor_by_index
// ============================================================================

#[test]
fn core_select_descriptor_by_index() {
    // Cycle A Group B swap (plan-R0 M-a): per-key-identical `<0;1>/*` entries
    // preserve the 4-entry select-by-index coverage without the now-rejected
    // fixed use-site step.
    let d0 = format!("wpkh([{MAINNET_FP_A}/84'/0'/0']{MAINNET_XPUB_A}/<0;1>/*)");
    let d1 = d0.clone();
    let d2 = format!("sh(wpkh([{MAINNET_FP_B}/49'/0'/0']{MAINNET_XPUB_B}/<0;1>/*))");
    let d3 = d2.clone();
    let blob = build_core_multi(&[
        CoreEntry {
            desc: &d0,
            active: true,
            internal: false,
        },
        CoreEntry {
            desc: &d1,
            active: true,
            internal: true,
        },
        CoreEntry {
            desc: &d2,
            active: false,
            internal: false,
        },
        CoreEntry {
            desc: &d3,
            active: false,
            internal: true,
        },
    ]);
    let out = run_core_stdin_select(&blob, "2").success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("bundles=1"), "stdout: {stdout}");
    // The selected entry is index 2 (sh(wpkh)) -> fingerprint MAINNET_FP_B.
    assert!(stdout.contains(MAINNET_FP_B), "stdout: {stdout}");
}

// ============================================================================
// §3.5 — core_select_descriptor_active_receive
// ============================================================================

#[test]
fn core_select_descriptor_active_receive() {
    // Cycle A Group B swap (plan-R0 M-a): selection is driven by the
    // per-entry `active`/`internal` JSON fields, not by descriptor content —
    // per-key-identical `<0;1>/*` entries preserve this cell's assertion.
    let d0 = format!("wpkh([{MAINNET_FP_A}/84'/0'/0']{MAINNET_XPUB_A}/<0;1>/*)");
    let d1 = d0.clone();
    let d2 = format!("sh(wpkh([{MAINNET_FP_B}/49'/0'/0']{MAINNET_XPUB_B}/<0;1>/*))");
    let d3 = d2.clone();
    let blob = build_core_multi(&[
        CoreEntry {
            desc: &d0,
            active: true,
            internal: false,
        }, // receive
        CoreEntry {
            desc: &d1,
            active: true,
            internal: true,
        }, // change
        CoreEntry {
            desc: &d2,
            active: false,
            internal: false,
        },
        CoreEntry {
            desc: &d3,
            active: false,
            internal: true,
        },
    ]);
    let out = run_core_stdin_select(&blob, "active-receive").success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    // Only d0 is active+!internal.
    assert!(stdout.contains("bundles=1"), "stdout: {stdout}");
    assert!(stdout.contains(MAINNET_FP_A), "stdout: {stdout}");
}

// ============================================================================
// §3.6 — core_select_descriptor_active_change
// ============================================================================

#[test]
fn core_select_descriptor_active_change() {
    // Cycle A Group B swap (plan-R0 M-a): selection is driven by the
    // per-entry `active`/`internal` JSON fields, not by descriptor content —
    // per-key-identical `<0;1>/*` entries preserve this cell's assertion.
    let d0 = format!("wpkh([{MAINNET_FP_A}/84'/0'/0']{MAINNET_XPUB_A}/<0;1>/*)");
    let d1 = d0.clone();
    let d2 = format!("sh(wpkh([{MAINNET_FP_B}/49'/0'/0']{MAINNET_XPUB_B}/<0;1>/*))");
    let d3 = d2.clone();
    let blob = build_core_multi(&[
        CoreEntry {
            desc: &d0,
            active: true,
            internal: false,
        },
        CoreEntry {
            desc: &d1,
            active: true,
            internal: true,
        }, // change
        CoreEntry {
            desc: &d2,
            active: false,
            internal: false,
        },
        CoreEntry {
            desc: &d3,
            active: false,
            internal: true,
        },
    ]);
    let out = run_core_stdin_select(&blob, "active-change").success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    // Only d1 is active+internal.
    assert!(stdout.contains("bundles=1"), "stdout: {stdout}");
}

// ============================================================================
// §3.7 — core_multisig_wsh_sortedmulti_2_of_3
// ============================================================================

#[test]
fn core_multisig_wsh_sortedmulti_2_of_3() {
    // Cycle A Group B swap (plan-R0 M-a): the incidental fixed `/0/*` step is
    // orthogonal to this cell's multisig-parse assertion; swap to `<0;1>/*`.
    let desc = format!(
        "wsh(sortedmulti(2,[{MAINNET_FP_A}/48'/0'/0'/2']{MAINNET_XPUB_A}/<0;1>/*,[{MAINNET_FP_B}/48'/0'/0'/2']{MAINNET_XPUB_B}/<0;1>/*,[{MAINNET_FP_C}/48'/0'/0'/2']{MAINNET_XPUB_C}/<0;1>/*))"
    );
    let blob = build_core_single(&desc, true, false, Some((0, 1000)), false);
    let out = run_core_stdin(&blob).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("cosigners=3"), "stdout: {stdout}");
    assert!(stdout.contains("threshold=2"), "stdout: {stdout}");
    assert!(stdout.contains("network=mainnet"), "stdout: {stdout}");
}

// ============================================================================
// §3.8 — core_multipath_split_to_receive_change
// ============================================================================

#[test]
fn core_multipath_split_to_receive_change() {
    // Single entry with BIP-389 multipath `<0;1>/*`. Default Core listdescriptors
    // output uses this form. The parser should accept the single-entry blob
    // without rejecting the multipath syntax.
    let desc = format!("wpkh([{MAINNET_FP_A}/84'/0'/0']{MAINNET_XPUB_A}/<0;1>/*)");
    let blob = build_core_single(&desc, true, false, Some((0, 1000)), false);
    let out = run_core_stdin(&blob).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("bundles=1"), "stdout: {stdout}");
    assert!(stdout.contains("cosigners=1"), "stdout: {stdout}");
}

// ============================================================================
// §3.9 — core_xprv_rejected_exit_2
// ============================================================================

#[test]
fn core_xprv_rejected_exit_2() {
    // Bitcoin Core `listdescriptors true` includes xprv-bearing descriptors;
    // toolkit must refuse. The actual descriptor checksum will never be
    // computed (the xprv-substring check fires before parse).
    let blob = "{\n  \"wallet_name\": \"sk\",\n  \"descriptors\": [\n    {\n      \"desc\": \"wpkh([b8688df1/84'/0'/0']xprvAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA/<0;1>/*)#aaaaaaaa\",\n      \"active\": true,\n      \"internal\": false\n    }\n  ]\n}\n";
    let assert = run_core_stdin(blob).failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let code = assert.get_output().status.code().unwrap_or(-1);
    assert_eq!(code, 2, "expected exit 2; stderr: {stderr}");
    assert!(
        stderr.contains("xprv"),
        "expected xprv-refusal text; stderr: {stderr}"
    );
    assert!(
        stderr.contains("listdescriptors") && stderr.contains("without `true`"),
        "expected helpful rerun template; stderr: {stderr}"
    );
}

// ============================================================================
// Phase 3 R0 architect C1 fold — testnet `tprv` must also be refused
// ============================================================================

#[test]
fn phase3_c1_fold_core_tprv_rejected_exit_2() {
    // `bitcoin-cli -signet listdescriptors true` emits `tprv...` keys, not
    // `xprv...`. The prior `desc.contains("xprv")` check let `tprv` slip
    // through (downstream parse failed with a misleading "no xpub keys
    // found" message). Post-fold: any extended-private-key prefix
    // (xprv/tprv/yprv/Yprv/zprv/Zprv/uprv/Uprv/vprv/Vprv) must hit the
    // helpful `ImportWalletXprvForbidden` template.
    let blob = "{\n  \"wallet_name\": \"signet\",\n  \"descriptors\": [\n    {\n      \"desc\": \"wpkh([b8688df1/84'/1'/0']tprvAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA/<0;1>/*)#bbbbbbbb\",\n      \"active\": true,\n      \"internal\": false\n    }\n  ]\n}\n";
    let assert = run_core_stdin(blob).failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let code = assert.get_output().status.code().unwrap_or(-1);
    assert_eq!(
        code, 2,
        "tprv must hit ImportWalletXprvForbidden (exit 2); stderr: {stderr}"
    );
    assert!(
        stderr.contains("listdescriptors") && stderr.contains("without `true`"),
        "expected helpful rerun template even for tprv; stderr: {stderr}"
    );
}

#[test]
fn phase3_c1_fold_core_slip132_zprv_rejected_exit_2() {
    // SLIP-132 BIP-84-private-form prefix `zprv` — also refused.
    let blob = "{\n  \"wallet_name\": \"sk\",\n  \"descriptors\": [\n    {\n      \"desc\": \"wpkh([b8688df1/84'/0'/0']zprvAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA/<0;1>/*)#cccccccc\",\n      \"active\": true,\n      \"internal\": false\n    }\n  ]\n}\n";
    let assert = run_core_stdin(blob).failure();
    let code = assert.get_output().status.code().unwrap_or(-1);
    assert_eq!(code, 2, "zprv must also be refused");
}

// ============================================================================
// Phase 3 R0 architect I1 fold — xprv substring in BIP-380 checksum must NOT
// false-positive (checksum alphabet contains x/p/r/v independently).
// ============================================================================

#[test]
fn phase3_i1_fold_xprv_substring_in_checksum_does_not_false_positive() {
    // Build a benign xpub descriptor whose checksum field is set to a
    // hand-crafted string containing the 4-char run `xprv`. Since the
    // checksum is invalid, the parse will fail — but with the BIP-380
    // checksum-validation error, NOT with `ImportWalletXprvForbidden`.
    // The fold strips `#<csum>` before the substring scan so the
    // checksum cannot trigger the xprv-refusal path.
    let blob = "{\n  \"wallet_name\": \"sk\",\n  \"descriptors\": [\n    {\n      \"desc\": \"wpkh([b8688df1/84'/0'/0']xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX/<0;1>/*)#xprvqzzz\",\n      \"active\": true,\n      \"internal\": false\n    }\n  ]\n}\n";
    let assert = run_core_stdin(blob).failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let code = assert.get_output().status.code().unwrap_or(-1);
    assert_eq!(code, 2, "expected parse-error tier; stderr: {stderr}");
    // Must NOT route through the xprv-refusal template — that would be
    // the false-positive class this regression cell defends against.
    assert!(
        !stderr.contains("listdescriptors") || !stderr.contains("without `true`"),
        "BIP-380 checksum substring `xprv` must not false-positive ImportWalletXprvForbidden; stderr: {stderr}"
    );
    // Should route through the BIP-380 checksum-validation error path.
    assert!(
        stderr.contains("BIP-380") || stderr.contains("checksum"),
        "expected BIP-380 checksum-validation error; stderr: {stderr}"
    );
}

// ============================================================================
// §3.10 — core_dropped_fields_notice
// ============================================================================

#[test]
fn core_dropped_fields_notice() {
    // Include `timestamp: "now"` + `next: 5` + `next_index: 5`; assert NOTICE
    // fires on stderr; exit 0.
    let desc = format!("wpkh([{MAINNET_FP_A}/84'/0'/0']{MAINNET_XPUB_A}/<0;1>/*)");
    let blob = build_core_single(&desc, true, false, Some((0, 1000)), true);
    let out = run_core_stdin(&blob).success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    // SPEC §2.4 NOTICE template (template_groups: "wallet-state fields").
    assert!(
        stderr.contains("dropped wallet-state fields"),
        "expected dropped-fields NOTICE; stderr: {stderr}"
    );
    // The NOTICE should name at least one of the dropped fields.
    assert!(
        stderr.contains("timestamp") || stderr.contains("next"),
        "expected dropped field name; stderr: {stderr}"
    );
}

// ============================================================================
// §3.11 — core_invalid_json_exit_2
// ============================================================================

#[test]
fn core_invalid_json_exit_2() {
    let blob = "{ this is not valid json at all";
    let assert = run_core_stdin(blob).failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let code = assert.get_output().status.code().unwrap_or(-1);
    assert_eq!(code, 2, "expected exit 2 (parse error); stderr: {stderr}");
    assert!(
        stderr.contains("parse error"),
        "expected parse-error template; stderr: {stderr}"
    );
}

// ============================================================================
// §3.12 — core_missing_descriptors_key_exit_2
// ============================================================================

#[test]
fn core_missing_descriptors_key_exit_2() {
    // Valid JSON but no top-level `descriptors` key.
    let blob = "{\"wallet_name\": \"only\"}\n";
    let assert = run_core_stdin(blob).failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let code = assert.get_output().status.code().unwrap_or(-1);
    assert_eq!(code, 2, "expected exit 2; stderr: {stderr}");
    assert!(
        stderr.contains("parse error") && stderr.contains("descriptors"),
        "expected descriptors-key error template; stderr: {stderr}"
    );
}

// ============================================================================
// §3.13 — core_empty_descriptors_array_exit_2
// ============================================================================

#[test]
fn core_empty_descriptors_array_exit_2() {
    let blob = "{\"descriptors\": []}\n";
    let assert = run_core_stdin(blob).failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let code = assert.get_output().status.code().unwrap_or(-1);
    assert_eq!(code, 2, "expected exit 2; stderr: {stderr}");
    assert!(
        stderr.contains("parse error") && stderr.contains("empty"),
        "expected empty-array error template; stderr: {stderr}"
    );
}

// ============================================================================
// §3.14 — core_testnet_tpub_network_detected
// ============================================================================

#[test]
fn core_testnet_tpub_network_detected() {
    // All-tpub descriptors with BIP-48 coin_type=1' on the origin path.
    // Cycle A Group B swap (plan-R0 M-a): swap the incidental fixed `/0/*`
    // step to `<0;1>/*`; this cell's assertion is network detection, not the
    // use-site shape.
    let desc = format!(
        "wsh(sortedmulti(2,[{TESTNET_FP_A}/48'/1'/0'/2']{TESTNET_XPUB_A}/<0;1>/*,[{TESTNET_FP_B}/48'/1'/0'/2']{TESTNET_XPUB_B}/<0;1>/*))"
    );
    let blob = build_core_single(&desc, true, false, Some((0, 1000)), false);
    let out = run_core_stdin(&blob).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("network=testnet"), "stdout: {stdout}");
}

// ============================================================================
// Extra cell — sniff smoke (true on Core blob, false on Specter-like blob)
// ============================================================================

#[test]
fn core_sniff_smoke() {
    // Library boundary: BitcoinCoreParser::sniff is not exposed via CLI in
    // Phase 3 (Phase 5 wires the sniff dispatcher), so this cell pins the
    // sniff predicate by going through the parse path with `--format
    // bitcoin-core`. A Core-shaped blob with NO vendor-marker keys parses
    // success; a Specter-like blob (top-level `descriptor` key + `label` +
    // `devices`) does not match `descriptors: [array]` shape, so parse fails
    // exit 2.
    let core_blob = build_core_single(
        &format!("wpkh([{MAINNET_FP_A}/84'/0'/0']{MAINNET_XPUB_A}/<0;1>/*)"),
        true,
        false,
        None,
        false,
    );
    run_core_stdin(&core_blob).success();

    // Specter-shaped blob: `descriptor` (singular) + `label` + `devices` keys.
    // Lacks top-level `descriptors` array. Pre-v0.28.7: parse error (exit 2).
    // Post-v0.28.7 Slug 3 (cross-format mismatch matrix completion): the
    // bitcoin-core arm now refuses Specter-shaped blobs via the more
    // specific ImportWalletFormatMismatch (exit 1), since the sniffer
    // identifies the blob as Specter and the user-supplied --format is
    // bitcoin-core.
    let specter_like = "{\"label\":\"Daily\",\"blockheight\":0,\"descriptor\":\"wpkh([b8688df1/84'/0'/0']xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX/<0;1>/*)#00lx6ere\",\"devices\":[\"unknown\"]}";
    let assert = run_core_stdin(specter_like).failure();
    let code = assert.get_output().status.code().unwrap_or(-1);
    assert_eq!(code, 1, "post-v0.28.7 Slug 3: bitcoin-core arm refuses Specter-shaped blob via ImportWalletFormatMismatch (exit 1)");
}

// ============================================================================
// Extra cell — fixture-file round-trip (uses the vendored multi-bip84 file)
// ============================================================================

#[test]
fn core_fixture_file_multi_bip84_all() {
    let p = fixture_path("core-multi-bip84.json");
    let out = run_core_file_select(&p, "all").success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    // The fixture has 4 entries (receive + change, two script types).
    assert!(stdout.contains("bundles=4"), "stdout: {stdout}");
}

// ============================================================================
// v0.27.1 Phase 2 PR-#26 fold — shape-mismatch silent defaults
// ============================================================================

/// Phase 2 I4 fold cell — `"active": "true"` (string instead of bool) must
/// surface as a typed parse error, not silently flip to `active: false` and
/// produce a misleading downstream "no active-* descriptor found" error.
#[test]
fn bitcoin_core_active_non_boolean_errors_with_pointer_text() {
    let desc = format!("wpkh([{MAINNET_FP_A}/84'/0'/0']{MAINNET_XPUB_A}/<0;1>/*)");
    let cs = checksum(&desc);
    let blob = format!(
        "{{\n  \"descriptors\": [\n    {{\n      \"desc\": \"{desc}#{cs}\",\n      \"active\": \"true\",\n      \"internal\": false\n    }}\n  ]\n}}\n"
    );
    let assertion = run_core_stdin(&blob).failure();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("`active` must be boolean"),
        "expected shape-strict diagnostic naming `active`; got: {stderr}"
    );
}

/// Phase 2 I4 fold cell — `"internal": 1` (number instead of bool) must
/// reject symmetric with `active`.
#[test]
fn bitcoin_core_internal_non_boolean_errors_with_pointer_text() {
    let desc = format!("wpkh([{MAINNET_FP_A}/84'/0'/0']{MAINNET_XPUB_A}/<0;1>/*)");
    let cs = checksum(&desc);
    let blob = format!(
        "{{\n  \"descriptors\": [\n    {{\n      \"desc\": \"{desc}#{cs}\",\n      \"active\": true,\n      \"internal\": 1\n    }}\n  ]\n}}\n"
    );
    let assertion = run_core_stdin(&blob).failure();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("`internal` must be boolean"),
        "expected shape-strict diagnostic naming `internal`; got: {stderr}"
    );
}

/// Phase 2 I4 fold cell — regression guard that ABSENT (vs shape-wrong)
/// `active` keeps the prior default-false behavior. Mirrors `parse_range_field`'s
/// absent-vs-shape-wrong split. R0 M3 fold: verify the resulting envelope
/// materializes `active: false` / `internal: false`, not just that parse
/// succeeded. Use --json to access source_metadata in the envelope.
#[test]
fn bitcoin_core_active_absent_defaults_false() {
    let desc = format!("wpkh([{MAINNET_FP_A}/84'/0'/0']{MAINNET_XPUB_A}/<0;1>/*)");
    let cs = checksum(&desc);
    // No `active` or `internal` keys — both default to false.
    let blob = format!(
        "{{\n  \"descriptors\": [\n    {{\n      \"desc\": \"{desc}#{cs}\"\n    }}\n  ]\n}}\n"
    );
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--blob",
            "-",
            "--format",
            "bitcoin-core",
            "--json",
        ])
        .write_stdin(blob)
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let envelope: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON envelope");
    let arr = envelope.as_array().expect("envelope is array");
    assert_eq!(arr.len(), 1, "expected one entry; got: {stdout}");
    let meta = &arr[0]["source_metadata"];
    assert_eq!(
        meta["active"],
        serde_json::Value::Bool(false),
        "absent `active` must default to false; envelope: {stdout}"
    );
    assert_eq!(
        meta["internal"],
        serde_json::Value::Bool(false),
        "absent `internal` must default to false; envelope: {stdout}"
    );
}

/// Phase 2 I6 fold cell — `thresh()` argument exceeding u8 range (>255
/// cosigners) must surface as a typed parse error, not silently render as
/// `"threshold": null` per FOLLOWUP `pr-26-shape-mismatch-silent-defaults`.
#[test]
fn bitcoin_core_thresh_overflow_errors_clearly() {
    // sortedmulti is the practical multisig surface; thresh(256, …) would
    // require 256 keys which is implausible at the test-fixture level. We
    // construct a synthetic descriptor body that hits the u8 overflow path
    // via the regex match — the descriptor parse may also fail downstream,
    // but the overflow rejection fires first.
    let desc = format!("wsh(sortedmulti(256,[{MAINNET_FP_A}/48'/0'/0'/2']{MAINNET_XPUB_A}/<0;1>/*,[{MAINNET_FP_B}/48'/0'/0'/2']{MAINNET_XPUB_B}/<0;1>/*))");
    let cs = checksum(&desc);
    let blob = format!(
        "{{\n  \"descriptors\": [\n    {{\n      \"desc\": \"{desc}#{cs}\",\n      \"active\": true,\n      \"internal\": false\n    }}\n  ]\n}}\n"
    );
    let assertion = run_core_stdin(&blob).failure();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    // Either the overflow rejection fires (preferred path), or the descriptor
    // parser rejects 256 cosigners first. Both are correct refusals; we
    // accept either diagnostic as proof that the silent `threshold: null`
    // path is closed.
    assert!(
        stderr.contains("exceeds u8 range")
            || stderr.contains("256")
            || stderr.to_lowercase().contains("threshold"),
        "expected u8-overflow or 256-cosigner diagnostic; got: {stderr}"
    );
}

// ============================================================================
// v0.27.1 Phase 4 PR-#26 I15 — --select-descriptor matrix
// ============================================================================

/// I15(a) — out-of-range numeric index on a multi-entry blob → exit 1
/// (`SelectDescriptor::ByIndex(N)` arm of `apply_select_descriptor` at
/// wallet_import/mod.rs). Single happy-path index cell already covered in
/// existing suite (`core_select_index_1`); this cell pins the failure path.
#[test]
fn core_select_index_out_of_range_errors() {
    // 4-entry blob from existing fixture.
    let p = fixture_path("core-multi-bip84.json");
    let assertion = run_core_file_select(&p, "99").failure();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("99") && stderr.to_lowercase().contains("range"),
        "expected OOB diagnostic naming index 99; got: {stderr}"
    );
}

/// I15(b) — `--select-descriptor active-receive` against a blob where NO
/// entry satisfies `active && !internal` → exit 1.
#[test]
fn core_select_active_receive_no_match_errors() {
    let desc = format!("wpkh([{MAINNET_FP_A}/84'/0'/0']{MAINNET_XPUB_A}/<0;1>/*)");
    // Only inactive entry — no active-receive candidate.
    let blob = build_core_single(&desc, false, false, None, false);
    let assertion = run_core_stdin_select(&blob, "active-receive").failure();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("active-receive") || stderr.contains("no active"),
        "expected no-active-receive diagnostic; got: {stderr}"
    );
}

/// I15(c) — malformed `--select-descriptor` value → exit (clap-parse or
/// `parse_select` rejection). Locks the rejection template.
#[test]
fn core_select_malformed_value_errors() {
    let desc = format!("wpkh([{MAINNET_FP_A}/84'/0'/0']{MAINNET_XPUB_A}/<0;1>/*)");
    let blob = build_core_single(&desc, true, false, None, false);
    let assertion = run_core_stdin_select(&blob, "garbage_value").failure();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("garbage_value")
            || stderr.to_lowercase().contains("invalid value")
            || stderr.to_lowercase().contains("--select-descriptor"),
        "expected malformed-value diagnostic; got: {stderr}"
    );
}

// ============================================================================
// v0.28.0 Phase 10 — Bitcoin Core fixture-corpus expansion
//
// Adds 8 parse-only fixture cells against new vendored Core blobs per the
// plan-doc §S.9 owner-phase tags + plan-doc Phase 10 sub-phase table:
//
// - P10A: 4 new Core fixtures (`core-bip44-mainnet.json`,
//   `core-bip86-mainnet.json`, `core-wsh-sortedmulti-3of5.json`,
//   `core-multipath-0-1.json`). Parse-only cells.
// - P10B: 4 more Core fixtures (`core-explicit-active-false.json`,
//   `core-mainnet-receive-change-pair.json`,
//   `core-multipath-receive-change-pair.json`, `core-empty-descriptors-array.json`).
//   Parse-only + sniff-negative cells.
//
// Each cell loads the fixture from disk via `fixture_path` and asserts the
// stdout shape; no new test infrastructure required (existing helpers
// `run_core_file_select` + `Command::cargo_bin` + `fixture_path` suffice).
//
// Scope discipline: fixture-corpus only. No SPEC §6/§10 contract changes; no
// new parser flags. Per plan-doc §S.9, the BSMS-side corpus expansion is
// owned by Instance G3 (P9A/P9B); the per-vendor-format corpora are owned by
// their respective per-parser instances (A/B/C/D/E/F).
// ============================================================================

// ----------------------------------------------------------------------------
// P10A — 4 new Core fixtures, parse-only cells
// ----------------------------------------------------------------------------

/// P10A.1 — `core-bip44-mainnet.json`: P2PKH BIP-44 single-sig mainnet
/// (legacy script-type, `pkh(...)` descriptor wrapper, `m/44'/0'/0'` origin).
/// Pins the parser's acceptance of `pkh()` (legacy P2PKH) — Core ships this
/// for "legacy wallet" descriptors. Mirrors `core-bip49-mainnet.json`'s
/// existing happy-path shape but with the legacy `pkh()` wrapper instead of
/// the SegWit-v0-nested `sh(wpkh())`.
#[test]
fn core_fixture_file_bip44_mainnet_parses() {
    let p = fixture_path("core-bip44-mainnet.json");
    let out = run_core_file_select(&p, "all").success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("bundles=1"), "stdout: {stdout}");
    assert!(stdout.contains("cosigners=1"), "stdout: {stdout}");
    assert!(stdout.contains("network=mainnet"), "stdout: {stdout}");
    // ParsedImport.threshold is `None` for single-sig (no thresh/multi).
    assert!(stdout.contains("threshold=none"), "stdout: {stdout}");
    // Core entries set bsms_audit=None.
    assert!(stdout.contains("bsms_audit=none"), "stdout: {stdout}");
    // The fixture's [fp/...] origin uses MAINNET_FP_A (b8688df1).
    assert!(stdout.contains(MAINNET_FP_A), "stdout: {stdout}");
}

/// P10A.2 — `core-bip86-mainnet.json`: P2TR BIP-86 single-sig mainnet
/// (taproot key-path-only, `tr(xpub)` descriptor wrapper, `m/86'/0'/0'`
/// origin). Pins the parser's acceptance of `tr()` key-path-only — Core
/// ships this for taproot single-sig wallets. The miniscript adapter
/// permits `tr(xpub/...)` without a TapTree; threshold is None (single-sig).
#[test]
fn core_fixture_file_bip86_mainnet_parses() {
    let p = fixture_path("core-bip86-mainnet.json");
    let out = run_core_file_select(&p, "all").success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("bundles=1"), "stdout: {stdout}");
    assert!(stdout.contains("cosigners=1"), "stdout: {stdout}");
    assert!(stdout.contains("network=mainnet"), "stdout: {stdout}");
    assert!(stdout.contains("threshold=none"), "stdout: {stdout}");
    assert!(stdout.contains("bsms_audit=none"), "stdout: {stdout}");
    assert!(stdout.contains(MAINNET_FP_A), "stdout: {stdout}");
}

/// P10A.3 — `core-wsh-sortedmulti-3of5.json`: 3-of-5 wsh-sortedmulti
/// mainnet. Larger threshold + larger cosigner count than the existing
/// 2-of-3 fixture (`core-multisig-2of3.json`); pins the parser scales
/// beyond BIP-48 §"Multisig Public Key Derivation Path" common cases.
///
/// Three cosigners use the existing mainnet xpubs (A/B/C from
/// `cli_export_wallet_jade.rs`); two additional cosigners use BIP-32
/// Test Vector 1 and Test Vector 2 root xpubs (publicly-known, distinct
/// from the toolkit's test corpus to give 5 unique keys with stable
/// fingerprints `deadbeef` and `cafebabe`).
#[test]
fn core_fixture_file_wsh_sortedmulti_3of5_parses() {
    let p = fixture_path("core-wsh-sortedmulti-3of5.json");
    let out = run_core_file_select(&p, "all").success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("bundles=1"), "stdout: {stdout}");
    assert!(stdout.contains("cosigners=5"), "stdout: {stdout}");
    assert!(stdout.contains("threshold=3"), "stdout: {stdout}");
    assert!(stdout.contains("network=mainnet"), "stdout: {stdout}");
    // All 5 fingerprints surface in stdout.
    for fp in [
        MAINNET_FP_A,
        MAINNET_FP_B,
        MAINNET_FP_C,
        "deadbeef",
        "cafebabe",
    ] {
        assert!(stdout.contains(fp), "fp {fp} missing from stdout: {stdout}");
    }
}

/// P10A.4 — `core-multipath-0-1.json`: explicit BIP-389 `<0;1>/*` multipath
/// in a single-entry blob. Pins the parser's acceptance of the canonical
/// Core `listdescriptors`-default emit shape. (The existing
/// `core_multipath_split_to_receive_change` cell exercises the same shape
/// via `build_core_single`; this cell adds a fixture-FILE round-trip to
/// validate disk-on-disk consumption.)
#[test]
fn core_fixture_file_multipath_0_1_parses() {
    let p = fixture_path("core-multipath-0-1.json");
    let out = run_core_file_select(&p, "all").success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("bundles=1"), "stdout: {stdout}");
    assert!(stdout.contains("cosigners=1"), "stdout: {stdout}");
    assert!(stdout.contains("network=mainnet"), "stdout: {stdout}");
    assert!(stdout.contains(MAINNET_FP_A), "stdout: {stdout}");
}

// ----------------------------------------------------------------------------
// P10B — 4 more Core fixtures: parse-only + sniff-negative cells
// ----------------------------------------------------------------------------

/// P10B.1 — `core-explicit-active-false.json`: a single-entry blob with
/// `active: false` explicit. Pins the parser's `active` field passthrough
/// without coercion. The bundle parses cleanly (active flag is a passthrough
/// piece of provenance metadata, not a filter); the entry surfaces as
/// `active=false` in the stdout breakdown.
#[test]
fn core_fixture_file_explicit_active_false_parses() {
    let p = fixture_path("core-explicit-active-false.json");
    let out = run_core_file_select(&p, "all").success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("bundles=1"), "stdout: {stdout}");
    assert!(stdout.contains("cosigners=1"), "stdout: {stdout}");
    // active=false surfaces in the per-bundle breakdown.
    assert!(stdout.contains("active=false"), "stdout: {stdout}");
}

/// P10B.2 — `core-mainnet-receive-change-pair.json`: two-entry blob with
/// `/0/*` (receive, active+!internal) + `/1/*` (change, active+internal)
/// — the legacy Core shape pre-BIP-389-multipath.
///
/// SPEC_bitcoin_core_receive_change_pair_merge.md §8.2 FLIP: this same-key
/// receive/change pair now MERGES into one `<0;1>/*` multipath bundle via
/// the parse-time pre-pass, restoring standard Bitcoin Core `listdescriptors`
/// import. Was (Cycle A v0.76.0, pre-merge): exit-2 reject at entry 0 with
/// the hand-combine-then-`--format descriptor` workaround message. The
/// fixture file itself is KEPT UNCHANGED — it is both the canonical
/// legacy-split shape this cell now proves auto-recombines AND the fixture
/// the pair-merge FOLLOWUP always intended to consume.
#[test]
fn core_fixture_file_mainnet_receive_change_pair_parses() {
    let p = fixture_path("core-mainnet-receive-change-pair.json");
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["import-wallet", "--blob"])
        .arg(&p)
        .args([
            "--format",
            "bitcoin-core",
            "--select-descriptor",
            "all",
            "--json",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let envelope: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON envelope");
    let arr = envelope.as_array().expect("envelope array");
    assert_eq!(
        arr.len(),
        1,
        "receive+change pair must merge to ONE bundle; envelope: {stdout}"
    );
    let desc = arr[0]["bundle"]["descriptor"]
        .as_str()
        .expect("bundle.descriptor must be present");
    assert!(desc.contains("<0;1>/*"), "merged descriptor: {desc}");
    assert!(desc.contains(MAINNET_FP_A), "merged descriptor: {desc}");
    let (body, csum) = desc
        .rsplit_once('#')
        .expect("merged descriptor carries a checksum");
    assert_eq!(
        checksum(body),
        csum,
        "merged descriptor's checksum must itself validate: {desc}"
    );
}

/// P10B.3 — `core-multipath-receive-change-pair.json`: two-entry blob where
/// each entry already carries a BIP-389 `<0;1>/*` multipath (one wpkh
/// active+!internal, one sh(wpkh) active+internal). Distinct from P10B.2
/// in that each entry is itself multipath-shaped — a hybrid Core layout
/// some wallets emit when they combine legacy + segwit accounts. Parser
/// must accept this without conflating the two entries.
#[test]
fn core_fixture_file_multipath_receive_change_pair_parses() {
    let p = fixture_path("core-multipath-receive-change-pair.json");
    let out = run_core_file_select(&p, "all").success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("bundles=2"), "stdout: {stdout}");
    assert!(stdout.contains("network=mainnet"), "stdout: {stdout}");
    // Both fingerprints (BIP-84 entry uses MAINNET_FP_A; BIP-49 entry uses
    // MAINNET_FP_B) surface in stdout.
    assert!(stdout.contains(MAINNET_FP_A), "stdout: {stdout}");
    assert!(stdout.contains(MAINNET_FP_B), "stdout: {stdout}");

    // §8.13 STRENGTHEN (SPEC_bitcoin_core_receive_change_pair_merge.md §8.13,
    // R0-round-1 I1) — REGRESSION-LOCK, not a red-first driver: this fixture's
    // two entries are BOTH already-`<0;1>/*` multipath (distinct keys/scripts,
    // MAINNET_FP_A wpkh vs MAINNET_FP_B sh(wpkh)) — never merge candidates,
    // and never touched by the P1 pre-pass. `--select-descriptor
    // active-receive` must return exactly the ONE `internal:false` entry
    // (MAINNET_FP_A) and `active-change` exactly the ONE `internal:true`
    // entry (MAINNET_FP_B). This proves `internal` provenance is read from
    // the EXPLICIT per-entry field (`Some(bool)`), NOT inferred from the
    // already-multipath `<0;1>/*` shape both entries share (which a
    // shape-based-`None` implementation would conflate with a pre-pass-merged
    // entry and therefore satisfy BOTH filters for BOTH entries).
    let recv = run_core_file_select(&p, "active-receive").success();
    let recv_stdout = String::from_utf8(recv.get_output().stdout.clone()).unwrap();
    assert!(
        recv_stdout.contains("bundles=1"),
        "active-receive must return exactly one bundle; stdout: {recv_stdout}"
    );
    assert!(recv_stdout.contains(MAINNET_FP_A), "stdout: {recv_stdout}");
    assert!(
        !recv_stdout.contains(MAINNET_FP_B),
        "active-receive must NOT include the internal:true entry; stdout: {recv_stdout}"
    );

    let chg = run_core_file_select(&p, "active-change").success();
    let chg_stdout = String::from_utf8(chg.get_output().stdout.clone()).unwrap();
    assert!(
        chg_stdout.contains("bundles=1"),
        "active-change must return exactly one bundle; stdout: {chg_stdout}"
    );
    assert!(chg_stdout.contains(MAINNET_FP_B), "stdout: {chg_stdout}");
    assert!(
        !chg_stdout.contains(MAINNET_FP_A),
        "active-change must NOT include the internal:false entry; stdout: {chg_stdout}"
    );
}

/// P10B.4 — `core-empty-descriptors-array.json`: NEGATIVE case. Top-level
/// `descriptors: []` (empty array). Must refuse with exit 2 under
/// `--format bitcoin-core`. Pairs with `core_empty_descriptors_array_exit_2`
/// (the existing stdin-based assertion) to extend coverage to fixture-FILE
/// consumption.
#[test]
fn core_fixture_file_empty_descriptors_array_refused_exit_2() {
    let p = fixture_path("core-empty-descriptors-array.json");
    // Use a builder that does NOT pass `--select-descriptor` (we want the
    // bare `--format bitcoin-core` dispatch to surface the empty-array
    // refusal early in parse, not in the select-filter).
    let assertion = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["import-wallet", "--blob"])
        .arg(&p)
        .args(["--format", "bitcoin-core"])
        .assert()
        .failure();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    let code = assertion.get_output().status.code().unwrap_or(-1);
    assert_eq!(code, 2, "expected exit 2; stderr: {stderr}");
    assert!(
        stderr.contains("parse error") && stderr.contains("empty"),
        "expected empty-array error template; stderr: {stderr}"
    );
}

/// P10B.4-sniff — companion sniff-negative cell for the empty-descriptors
/// fixture. When `--format` is omitted, the `sniff_format` orchestrator
/// (post-P0D consult-all-then-count, sniff.rs:74-105) consults
/// `BitcoinCoreParser::sniff` (`bitcoin_core.rs:91-97`), which returns
/// `false` on an empty `descriptors: []` array. With all other parser
/// sniffs at v0.28.0 cutover still pre-stubbed to `false`, the verdict is
/// `SniffOutcome::NoMatch` → caller emits `ImportWalletAmbiguousFormat`
/// exit 1 with the "could not detect format" template (per SPEC §6.2).
#[test]
fn core_fixture_file_empty_descriptors_array_sniff_no_match() {
    let p = fixture_path("core-empty-descriptors-array.json");
    let assertion = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["import-wallet", "--blob"])
        .arg(&p)
        .assert()
        .failure();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    let code = assertion.get_output().status.code().unwrap_or(-1);
    // SniffOutcome::NoMatch → exit 1 with the auto-detect failure template
    // (distinct from the parse-time exit-2 path the previous cell pins).
    assert_eq!(code, 1, "expected exit 1 (sniff NoMatch); stderr: {stderr}");
    assert!(
        stderr.contains("could not detect format"),
        "expected sniff-NoMatch template; stderr: {stderr}"
    );
}

// ============================================================================
// Consensus-masked older() advisory (SPEC_older_timelock_advisory, Task 4)
// ============================================================================

/// A wsh miniscript descriptor carrying a BIP-68 consensus-masked relative
/// timelock (`older(65536)` — bit 16 is outside the low-16-bit value field,
/// so consensus masks it to an effective value of 0). Importing it must emit
/// the non-blocking advisory on stderr while still succeeding (exit 0).
#[test]
fn core_masked_older_emits_advisory() {
    // Cycle A Group B swap (plan-R0 M-a): the incidental fixed `/0/*` step is
    // orthogonal to this cell's older()-advisory assertion; swap to `<0;1>/*`.
    let desc = format!(
        "wsh(and_v(v:multi(2,[{MAINNET_FP_A}/48'/0'/0'/2']{MAINNET_XPUB_A}/<0;1>/*,[{MAINNET_FP_B}/48'/0'/0'/2']{MAINNET_XPUB_B}/<0;1>/*),older(65536)))"
    );
    let blob = build_core_single(&desc, true, false, Some((0, 1000)), false);
    let assert = run_core_stdin(&blob).success();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("advisory: older(65536) is consensus-masked"),
        "expected consensus-masked older() advisory on stderr; stderr: {stderr}"
    );
}

/// Clean-input counterpart: `older(2016)` is a valid 16-bit relative timelock
/// (no stray bits, non-zero value), so NO advisory is emitted. The import
/// still succeeds. Guards against the hook firing on clean operands.
#[test]
fn core_clean_older_emits_no_advisory() {
    // Cycle A Group B swap (plan-R0 M-a): swap the incidental fixed `/0/*`
    // step to `<0;1>/*`; unaffected by the clean-older() advisory assertion.
    let desc = format!(
        "wsh(and_v(v:multi(2,[{MAINNET_FP_A}/48'/0'/0'/2']{MAINNET_XPUB_A}/<0;1>/*,[{MAINNET_FP_B}/48'/0'/0'/2']{MAINNET_XPUB_B}/<0;1>/*),older(2016)))"
    );
    let blob = build_core_single(&desc, true, false, Some((0, 1000)), false);
    let assert = run_core_stdin(&blob).success();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(
        !stderr.contains("advisory: older"),
        "clean older(2016) must NOT emit an older() advisory; stderr: {stderr}"
    );
}

// ============================================================================
// Cycle A — residue-reject floor (CRITICAL funds fix): a bitcoin-core
// single-entry blob carrying a FIXED use-site step (`/0/*`) is un-
// representable in md1 (a fixed step silently collapsed to a bare `/*`,
// encoding a DIFFERENT wallet — SPEC_cycleA_descriptor_use_site_collapse.md
// §1). `parse_descriptor::lex_placeholders`'s residue-reject floor now
// catches this at import time (`parse_entry` calls `parse_descriptor`
// per-entry, `bitcoin_core.rs`), before any card is ever built. This is the
// dedicated per-surface reject test (plan Phase 1a) — the sole coverage of
// this reject shape, which is what licenses the Group-B `<0;1>/*` fixture
// swaps elsewhere in this file (the collapse regression they used to
// incidentally cover is proven here instead).
// ============================================================================

#[test]
fn core_fixed_use_site_step_rejected_with_workaround() {
    let desc = format!("wpkh([{MAINNET_FP_A}/84'/0'/0']{MAINNET_XPUB_A}/0/*)");
    let blob = build_core_single(&desc, true, false, Some((0, 1000)), false);
    let assertion = run_core_stdin(&blob).failure();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    let code = assertion.get_output().status.code().unwrap_or(-1);
    assert_eq!(
        code, 2,
        "fixed use-site step `/0/*` must reject exit 2; stderr: {stderr}"
    );
    assert!(
        stderr.contains("bitcoin-core"),
        "expected the reject to be scoped to the bitcoin-core surface; stderr: {stderr}"
    );
    assert!(
        stderr.contains("multipath") && stderr.contains("<a;b>"),
        "expected the multipath `/<a;b>/*` remedy pointer; stderr: {stderr}"
    );
    assert!(
        stderr.contains("/0/*"),
        "expected the reject to name the offending residue; stderr: {stderr}"
    );
}

/// Companion: Core's standard receive+change TWO-entry export (`/0/*` +
/// `/1/*`, distinct `internal` flags) also MERGES (SPEC
/// `bitcoin_core_receive_change_pair_merge.md` §8.3 FLIP) into one `<0;1>/*`
/// bundle. Distinct from `core_fixture_file_mainnet_receive_change_pair_parses`
/// (fixture-file variant of the same shape) — this is the inline-blob
/// stdin-driven twin. Renamed from `core_receive_change_pair_rejected_with_
/// workaround` (the pre-flip name asserted the exit-2 reject; keeping that
/// name after flipping the assertion to merge-accept would be misleading).
#[test]
fn core_receive_change_pair_merges_inline_blob() {
    let d0 = format!("wpkh([{MAINNET_FP_A}/84'/0'/0']{MAINNET_XPUB_A}/0/*)");
    let d1 = format!("wpkh([{MAINNET_FP_A}/84'/0'/0']{MAINNET_XPUB_A}/1/*)");
    let blob = build_core_multi(&[
        CoreEntry {
            desc: &d0,
            active: true,
            internal: false,
        },
        CoreEntry {
            desc: &d1,
            active: true,
            internal: true,
        },
    ]);
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--blob",
            "-",
            "--format",
            "bitcoin-core",
            "--select-descriptor",
            "all",
            "--json",
        ])
        .write_stdin(blob)
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let envelope: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON envelope");
    let arr = envelope.as_array().expect("envelope array");
    assert_eq!(
        arr.len(),
        1,
        "receive+change pair must merge to ONE bundle; envelope: {stdout}"
    );
    let desc = arr[0]["bundle"]["descriptor"]
        .as_str()
        .expect("bundle.descriptor must be present");
    assert!(desc.contains("<0;1>/*"), "merged descriptor: {desc}");
}

// ============================================================================
// bitcoin-core-receive-change-pair-merge (SPEC_bitcoin_core_receive_change_
// pair_merge.md §8) — the parse-time pre-pass that recombines a same-key
// Core receive/change split pair into one `<a;b>/*` multipath entry.
// ============================================================================

/// §8.1 LINCHPIN funds oracle — distinct-key `/0/*` (FP_A) + `/1/*` (FP_B)
/// looks receive/change-SHAPED (fixed step, steps differ, internal flags
/// disagree) but the KEYS DIFFER — distinct keys are different wallets.
/// MUST NOT merge; refused with the §7 differentiated near-miss message
/// (exit 2), not a silent merge and not the generic floor-reject text.
#[test]
fn core_receive_change_distinct_keys_must_not_merge() {
    let d0 = format!("wpkh([{MAINNET_FP_A}/84'/0'/0']{MAINNET_XPUB_A}/0/*)");
    let d1 = format!("wpkh([{MAINNET_FP_B}/84'/0'/0']{MAINNET_XPUB_B}/1/*)");
    let blob = build_core_multi(&[
        CoreEntry {
            desc: &d0,
            active: true,
            internal: false,
        },
        CoreEntry {
            desc: &d1,
            active: true,
            internal: true,
        },
    ]);
    let assertion = run_core_stdin_select(&blob, "all").failure();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    let code = assertion.get_output().status.code().unwrap_or(-1);
    assert_eq!(
        code, 2,
        "distinct-key receive/change-shaped near-miss must reject exit 2; stderr: {stderr}"
    );
    assert!(
        stderr.contains("distinct keys are different wallets") || stderr.contains("keys/origins differ"),
        "expected the §7 differentiated near-miss message, not a generic floor reject; stderr: {stderr}"
    );
}

/// §8.4 ANTI-C1 ORACLE (non-tautological) — import the same-key `/0/*` +
/// `/1/*` pair, merge to `<0;1>/*`, then INDEPENDENTLY derive addresses from
/// the ORIGINAL split descriptors (external from `.../0/*`, internal from
/// `.../1/*`) via rust-miniscript directly and assert they equal the merged
/// bundle's chain-0 / chain-1 addresses for BOTH chains. Anchors on the
/// pre-merge truth, not on a hand-authored `<0;1>` (which would be the same
/// construction the merge itself performs — tautological). Plus:
/// `verify-bundle` PASSES on the merged output.
#[test]
fn core_merged_pair_addresses_match_original_split() {
    let orig_recv = format!("wpkh([{MAINNET_FP_A}/84'/0'/0']{MAINNET_XPUB_A}/0/*)");
    let orig_chg = format!("wpkh([{MAINNET_FP_A}/84'/0'/0']{MAINNET_XPUB_A}/1/*)");
    let expected_recv = derive_addresses(&orig_recv, 3, bitcoin::Network::Bitcoin);
    let expected_chg = derive_addresses(&orig_chg, 3, bitcoin::Network::Bitcoin);

    let blob = build_core_multi(&[
        CoreEntry {
            desc: &orig_recv,
            active: true,
            internal: false,
        },
        CoreEntry {
            desc: &orig_chg,
            active: true,
            internal: true,
        },
    ]);
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--blob",
            "-",
            "--format",
            "bitcoin-core",
            "--json",
        ])
        .write_stdin(blob)
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let envelope: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON envelope");
    let arr = envelope.as_array().expect("envelope array");
    assert_eq!(
        arr.len(),
        1,
        "receive+change pair must merge to ONE bundle; envelope: {stdout}"
    );
    let merged_desc = arr[0]["bundle"]["descriptor"]
        .as_str()
        .expect(
            "bundle.descriptor must be present -- the oracle must NOT fall back to a \
             re-authored <0;1> when --json yields no descriptor",
        )
        .to_string();
    assert!(
        merged_desc.contains("<0;1>/*"),
        "merged descriptor: {merged_desc}"
    );

    let got_recv = derive_multipath_chain_addresses(&merged_desc, 0, 3, bitcoin::Network::Bitcoin);
    let got_chg = derive_multipath_chain_addresses(&merged_desc, 1, 3, bitcoin::Network::Bitcoin);
    assert_eq!(
        got_recv, expected_recv,
        "merged chain-0 addresses must match the ORIGINAL /0/* split"
    );
    assert_eq!(
        got_chg, expected_chg,
        "merged chain-1 addresses must match the ORIGINAL /1/* split"
    );

    assert_descriptor_verify_bundle_ok(&merged_desc, "mainnet");
}

/// §8.5 — `--select-descriptor active-receive` AND `active-change` each
/// return the ONE merged bundle (once each; no double-emit).
#[test]
fn core_merged_pair_select_receive_and_change_both_match() {
    let d0 = format!("wpkh([{MAINNET_FP_A}/84'/0'/0']{MAINNET_XPUB_A}/0/*)");
    let d1 = format!("wpkh([{MAINNET_FP_A}/84'/0'/0']{MAINNET_XPUB_A}/1/*)");
    let blob = build_core_multi(&[
        CoreEntry {
            desc: &d0,
            active: true,
            internal: false,
        },
        CoreEntry {
            desc: &d1,
            active: true,
            internal: true,
        },
    ]);

    let recv = run_core_stdin_select(&blob, "active-receive").success();
    let recv_stdout = String::from_utf8(recv.get_output().stdout.clone()).unwrap();
    assert!(
        recv_stdout.contains("bundles=1"),
        "active-receive must return the one merged bundle; stdout: {recv_stdout}"
    );

    let chg = run_core_stdin_select(&blob, "active-change").success();
    let chg_stdout = String::from_utf8(chg.get_output().stdout.clone()).unwrap();
    assert!(
        chg_stdout.contains("bundles=1"),
        "active-change must return the SAME one merged bundle; stdout: {chg_stdout}"
    );
}

/// §8.6 — a single lone `/0/*` entry with no receive/change partner still
/// hits the existing generic fixed-step floor reject (exit 2) — the merge
/// pre-pass never touches an unpaired entry.
#[test]
fn core_lone_receive_fixed_step_still_rejects() {
    let d0 = format!("wpkh([{MAINNET_FP_A}/84'/0'/0']{MAINNET_XPUB_A}/0/*)");
    let blob = build_core_single(&d0, true, false, Some((0, 1000)), false);
    let assertion = run_core_stdin(&blob).failure();
    let code = assertion.get_output().status.code().unwrap_or(-1);
    assert_eq!(code, 2, "lone fixed-step entry must still floor-reject");
}

/// §8.7 — two same-key entries with the IDENTICAL final step (cond. 4 fails:
/// steps do not differ) do not pair; both are left unmerged and floor-reject.
#[test]
fn core_pair_same_step_does_not_merge() {
    let d0 = format!("wpkh([{MAINNET_FP_A}/84'/0'/0']{MAINNET_XPUB_A}/0/*)");
    let d1 = d0.clone();
    let blob = build_core_multi(&[
        CoreEntry {
            desc: &d0,
            active: true,
            internal: false,
        },
        CoreEntry {
            desc: &d1,
            active: true,
            internal: true,
        },
    ]);
    let assertion = run_core_stdin_select(&blob, "all").failure();
    let code = assertion.get_output().status.code().unwrap_or(-1);
    assert_eq!(code, 2, "same-step pair must not merge (cond. 4 fails)");
}

/// §8.8 — two same-key entries with differing steps but AGREEING `internal`
/// flags (cond. 5 fails: both `false`) do not pair.
#[test]
fn core_pair_both_internal_false_does_not_merge() {
    let d0 = format!("wpkh([{MAINNET_FP_A}/84'/0'/0']{MAINNET_XPUB_A}/0/*)");
    let d1 = format!("wpkh([{MAINNET_FP_A}/84'/0'/0']{MAINNET_XPUB_A}/1/*)");
    let blob = build_core_multi(&[
        CoreEntry {
            desc: &d0,
            active: true,
            internal: false,
        },
        CoreEntry {
            desc: &d1,
            active: true,
            internal: false,
        },
    ]);
    let assertion = run_core_stdin_select(&blob, "all").failure();
    let code = assertion.get_output().status.code().unwrap_or(-1);
    assert_eq!(
        code, 2,
        "internal-agreeing pair must not merge (cond. 5 fails)"
    );
}

/// §8.9 — three same-key entries sharing the grouping key (cond. 6 fails:
/// ambiguous, not exactly two) do not merge; a NOTICE names the ambiguity and
/// all three fall through to the generic floor reject (exit 2, each still
/// carries a fixed step).
#[test]
fn core_three_entries_sharing_key_ambiguous_no_merge() {
    let d0 = format!("wpkh([{MAINNET_FP_A}/84'/0'/0']{MAINNET_XPUB_A}/0/*)");
    let d1 = format!("wpkh([{MAINNET_FP_A}/84'/0'/0']{MAINNET_XPUB_A}/1/*)");
    let d2 = format!("wpkh([{MAINNET_FP_A}/84'/0'/0']{MAINNET_XPUB_A}/2/*)");
    let blob = build_core_multi(&[
        CoreEntry {
            desc: &d0,
            active: true,
            internal: false,
        },
        CoreEntry {
            desc: &d1,
            active: true,
            internal: true,
        },
        CoreEntry {
            desc: &d2,
            active: false,
            internal: false,
        },
    ]);
    let assertion = run_core_stdin_select(&blob, "all").failure();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    let code = assertion.get_output().status.code().unwrap_or(-1);
    assert_eq!(
        code, 2,
        "3-way ambiguous share must still floor-reject; stderr: {stderr}"
    );
    assert!(
        stderr.contains("ambiguous"),
        "expected the ambiguity NOTICE on stderr; stderr: {stderr}"
    );
}

/// §8.10 — `wsh(sortedmulti(2,...))` all-keys `/0/*` + all-keys `/1/*` merges
/// to a single multisig `<0;1>/*`; addresses independently derived from the
/// original split multisig descriptors for both chains (guards a misfired
/// per-key replacement). `verify-bundle` PASSES.
#[test]
fn core_multisig_receive_change_pair_merges() {
    let orig_recv = format!(
        "wsh(sortedmulti(2,[{MAINNET_FP_A}/48'/0'/0'/2']{MAINNET_XPUB_A}/0/*,[{MAINNET_FP_B}/48'/0'/0'/2']{MAINNET_XPUB_B}/0/*,[{MAINNET_FP_C}/48'/0'/0'/2']{MAINNET_XPUB_C}/0/*))"
    );
    let orig_chg = format!(
        "wsh(sortedmulti(2,[{MAINNET_FP_A}/48'/0'/0'/2']{MAINNET_XPUB_A}/1/*,[{MAINNET_FP_B}/48'/0'/0'/2']{MAINNET_XPUB_B}/1/*,[{MAINNET_FP_C}/48'/0'/0'/2']{MAINNET_XPUB_C}/1/*))"
    );
    let expected_recv = derive_addresses(&orig_recv, 3, bitcoin::Network::Bitcoin);
    let expected_chg = derive_addresses(&orig_chg, 3, bitcoin::Network::Bitcoin);

    let blob = build_core_multi(&[
        CoreEntry {
            desc: &orig_recv,
            active: true,
            internal: false,
        },
        CoreEntry {
            desc: &orig_chg,
            active: true,
            internal: true,
        },
    ]);
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--blob",
            "-",
            "--format",
            "bitcoin-core",
            "--json",
        ])
        .write_stdin(blob)
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let envelope: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON envelope");
    let arr = envelope.as_array().expect("envelope array");
    assert_eq!(arr.len(), 1, "envelope: {stdout}");
    let merged_desc = arr[0]["bundle"]["descriptor"]
        .as_str()
        .expect("bundle.descriptor must be present")
        .to_string();
    assert!(
        merged_desc.contains("<0;1>/*"),
        "merged descriptor: {merged_desc}"
    );

    let got_recv = derive_multipath_chain_addresses(&merged_desc, 0, 3, bitcoin::Network::Bitcoin);
    let got_chg = derive_multipath_chain_addresses(&merged_desc, 1, 3, bitcoin::Network::Bitcoin);
    assert_eq!(
        got_recv, expected_recv,
        "chain-0 (receive) address mismatch"
    );
    assert_eq!(got_chg, expected_chg, "chain-1 (change) address mismatch");

    assert_descriptor_verify_bundle_ok(&merged_desc, "mainnet");
}

/// §8.11 — within ONE entry, one key uses `/0/*` and another uses `/1/*`
/// (per-key non-uniform step, cond. 7 fails). Never a merge candidate (no
/// partner is even sought); the entry, still carrying fixed steps, hits the
/// existing floor reject.
#[test]
fn core_multisig_partial_split_does_not_merge() {
    let desc = format!(
        "wsh(sortedmulti(2,[{MAINNET_FP_A}/48'/0'/0'/2']{MAINNET_XPUB_A}/0/*,[{MAINNET_FP_B}/48'/0'/0'/2']{MAINNET_XPUB_B}/1/*))"
    );
    let blob = build_core_single(&desc, true, false, Some((0, 1000)), false);
    let assertion = run_core_stdin(&blob).failure();
    let code = assertion.get_output().status.code().unwrap_or(-1);
    assert_eq!(
        code, 2,
        "per-key non-uniform split within one entry must not merge and must floor-reject"
    );
}

/// §8.12 — `--json` merged entry emits `source_metadata.internal: null`;
/// text-summary prints `both`.
#[test]
fn core_merged_json_internal_null() {
    let d0 = format!("wpkh([{MAINNET_FP_A}/84'/0'/0']{MAINNET_XPUB_A}/0/*)");
    let d1 = format!("wpkh([{MAINNET_FP_A}/84'/0'/0']{MAINNET_XPUB_A}/1/*)");
    let blob = build_core_multi(&[
        CoreEntry {
            desc: &d0,
            active: true,
            internal: false,
        },
        CoreEntry {
            desc: &d1,
            active: true,
            internal: true,
        },
    ]);

    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--blob",
            "-",
            "--format",
            "bitcoin-core",
            "--json",
        ])
        .write_stdin(blob.clone())
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let envelope: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON envelope");
    let arr = envelope.as_array().expect("envelope array");
    assert_eq!(arr.len(), 1, "envelope: {stdout}");
    assert_eq!(
        arr[0]["source_metadata"]["internal"],
        serde_json::Value::Null,
        "merged entry's source_metadata.internal must serialize as null; envelope: {stdout}"
    );

    let text_out = run_core_stdin(&blob).success();
    let text_stdout = String::from_utf8(text_out.get_output().stdout.clone()).unwrap();
    assert!(
        text_stdout.contains("internal=both"),
        "text-summary must print `both` for a merged entry; stdout: {text_stdout}"
    );
}

/// §8.14 — a `/5/*` + `/6/*` same-key pair merges to `<5;6>/*` — the ACTUAL
/// step values, never hardcoded 0/1.
#[test]
fn core_nonstandard_steps_merge_uses_actual_values() {
    let d0 = format!("wpkh([{MAINNET_FP_A}/84'/0'/0']{MAINNET_XPUB_A}/5/*)");
    let d1 = format!("wpkh([{MAINNET_FP_A}/84'/0'/0']{MAINNET_XPUB_A}/6/*)");
    let blob = build_core_multi(&[
        CoreEntry {
            desc: &d0,
            active: true,
            internal: false,
        },
        CoreEntry {
            desc: &d1,
            active: true,
            internal: true,
        },
    ]);
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--blob",
            "-",
            "--format",
            "bitcoin-core",
            "--json",
        ])
        .write_stdin(blob)
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let envelope: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON envelope");
    let arr = envelope.as_array().expect("envelope array");
    assert_eq!(arr.len(), 1, "envelope: {stdout}");
    let desc = arr[0]["bundle"]["descriptor"].as_str().unwrap();
    assert!(desc.contains("<5;6>/*"), "merged descriptor: {desc}");
}

/// §8.15 (taproot, mandatory) — split `tr(key/0/*)` + `tr(key/1/*)` (single-
/// key bip86, key-path-only) merges to `tr(key/<0;1>/*)`; P2TR addresses
/// independently derived from the two originals for both chains.
#[test]
fn core_tr_bip86_receive_change_pair_merges() {
    let orig_recv = format!("tr([{MAINNET_FP_A}/86'/0'/0']{MAINNET_XPUB_A}/0/*)");
    let orig_chg = format!("tr([{MAINNET_FP_A}/86'/0'/0']{MAINNET_XPUB_A}/1/*)");
    let expected_recv = derive_addresses(&orig_recv, 3, bitcoin::Network::Bitcoin);
    let expected_chg = derive_addresses(&orig_chg, 3, bitcoin::Network::Bitcoin);

    let blob = build_core_multi(&[
        CoreEntry {
            desc: &orig_recv,
            active: true,
            internal: false,
        },
        CoreEntry {
            desc: &orig_chg,
            active: true,
            internal: true,
        },
    ]);
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--blob",
            "-",
            "--format",
            "bitcoin-core",
            "--json",
        ])
        .write_stdin(blob)
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let envelope: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON envelope");
    let arr = envelope.as_array().expect("envelope array");
    assert_eq!(arr.len(), 1, "envelope: {stdout}");
    let merged_desc = arr[0]["bundle"]["descriptor"]
        .as_str()
        .expect("bundle.descriptor must be present")
        .to_string();
    assert!(
        merged_desc.starts_with("tr("),
        "merged descriptor: {merged_desc}"
    );
    assert!(
        merged_desc.contains("<0;1>/*"),
        "merged descriptor: {merged_desc}"
    );

    let got_recv = derive_multipath_chain_addresses(&merged_desc, 0, 3, bitcoin::Network::Bitcoin);
    let got_chg = derive_multipath_chain_addresses(&merged_desc, 1, 3, bitcoin::Network::Bitcoin);
    assert_eq!(
        got_recv, expected_recv,
        "chain-0 (receive) P2TR address mismatch"
    );
    assert_eq!(
        got_chg, expected_chg,
        "chain-1 (change) P2TR address mismatch"
    );
}

/// §8.15 (script-path `tr` out of scope) — a script-path `tr` (internal key +
/// a tapscript leaf) split pair is OUT of scope (§0 / §4.2 cond. 7): the
/// guard does NOT merge it; it falls to the floor reject (exit 2). LOCKED
/// behavior, not contingent on fixture feasibility.
#[test]
fn core_tr_scriptpath_pair_does_not_merge() {
    let orig_recv = format!(
        "tr([{MAINNET_FP_A}/86'/0'/0']{MAINNET_XPUB_A}/0/*,pk([{MAINNET_FP_B}/86'/0'/0']{MAINNET_XPUB_B}/0/*))"
    );
    let orig_chg = format!(
        "tr([{MAINNET_FP_A}/86'/0'/0']{MAINNET_XPUB_A}/1/*,pk([{MAINNET_FP_B}/86'/0'/0']{MAINNET_XPUB_B}/1/*))"
    );
    // Cheap insurance: both shapes must themselves be valid rust-miniscript
    // (a parse failure would make this cell vacuous, not a real script-path
    // `tr` refusal).
    Descriptor::<DescriptorPublicKey>::from_str(&orig_recv).expect("script-path tr must parse");
    Descriptor::<DescriptorPublicKey>::from_str(&orig_chg).expect("script-path tr must parse");

    let blob = build_core_multi(&[
        CoreEntry {
            desc: &orig_recv,
            active: true,
            internal: false,
        },
        CoreEntry {
            desc: &orig_chg,
            active: true,
            internal: true,
        },
    ]);
    let assertion = run_core_stdin_select(&blob, "all").failure();
    let code = assertion.get_output().status.code().unwrap_or(-1);
    assert_eq!(
        code, 2,
        "script-path tr receive/change pair is out of scope and must not merge"
    );
}

/// §8.16 — a `/0'/*` (hardened final step) + `/1/*` shaped pair: cond. 3
/// excludes the hardened side from candidacy entirely, so no merge is even
/// attempted; both entries floor-reject.
#[test]
fn core_hardened_final_step_does_not_merge() {
    let d0 = format!("wpkh([{MAINNET_FP_A}/84'/0'/0']{MAINNET_XPUB_A}/0'/*)");
    let d1 = format!("wpkh([{MAINNET_FP_A}/84'/0'/0']{MAINNET_XPUB_A}/1/*)");
    let blob = build_core_multi(&[
        CoreEntry {
            desc: &d0,
            active: true,
            internal: false,
        },
        CoreEntry {
            desc: &d1,
            active: true,
            internal: true,
        },
    ]);
    let assertion = run_core_stdin_select(&blob, "all").failure();
    let code = assertion.get_output().status.code().unwrap_or(-1);
    assert_eq!(code, 2, "hardened-final-step shaped pair must not merge");
}

/// §8.17 — a mergeable-shaped pair where entry 0 carries a CORRUPT `#<csum>`
/// is refused BEFORE merge (fail-closed, §4.4/M9) — never silently "repaired"
/// by the checksum-recompute step. The corrupt entry's own re-validation in
/// `parse_entry` surfaces the standard BIP-380 checksum error.
#[test]
fn core_corrupt_input_checksum_not_merged() {
    let d0 = format!("wpkh([{MAINNET_FP_A}/84'/0'/0']{MAINNET_XPUB_A}/0/*)");
    let d1 = format!("wpkh([{MAINNET_FP_A}/84'/0'/0']{MAINNET_XPUB_A}/1/*)");
    let cs0 = checksum(&d0);
    let cs1 = checksum(&d1);
    let bad_cs0: String = cs0.chars().rev().collect();
    assert_ne!(bad_cs0, cs0, "corrupted checksum must actually differ");

    let blob = format!(
        "{{\n  \"descriptors\": [\n    {{\n      \"desc\": \"{d0}#{bad_cs0}\",\n      \"active\": true,\n      \"internal\": false\n    }},\n    {{\n      \"desc\": \"{d1}#{cs1}\",\n      \"active\": true,\n      \"internal\": true\n    }}\n  ]\n}}\n"
    );
    let assertion = run_core_stdin_select(&blob, "all").failure();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    let code = assertion.get_output().status.code().unwrap_or(-1);
    assert_eq!(
        code, 2,
        "corrupt-checksum candidate must not be silently merged; stderr: {stderr}"
    );
    assert!(
        stderr.contains("BIP-380") || stderr.contains("checksum"),
        "expected the standard BIP-380 checksum-validation error; stderr: {stderr}"
    );
}
