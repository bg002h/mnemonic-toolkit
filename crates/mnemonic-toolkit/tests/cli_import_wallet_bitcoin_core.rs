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
use std::path::PathBuf;

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
/// Cycle A (plan-R0 I-C): a FIXED use-site step is un-representable in md1
/// (residue-reject floor) — this legacy non-multipath receive/change pair now
/// HARD-FAILS at entry 0 (`/0/*`) before selection ever runs, with the
/// bitcoin-core interim-limitation workaround message (combine to `<0;1>/*`
/// + `--format descriptor`; automatic recombination is the split-out
/// `bitcoin-core-receive-change-pair-merge` follow-up). The fixture file
/// itself is KEPT UNCHANGED — it is both the canonical legacy-split funds
/// regression this cell now proves closed AND the future INPUT fixture for
/// the pair-merge follow-up.
#[test]
fn core_fixture_file_mainnet_receive_change_pair_parses() {
    let p = fixture_path("core-mainnet-receive-change-pair.json");
    let assertion = run_core_file_select(&p, "all").failure();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    let code = assertion.get_output().status.code().unwrap_or(-1);
    assert_eq!(
        code, 2,
        "legacy non-multipath receive/change pair must reject exit 2; stderr: {stderr}"
    );
    assert!(
        stderr.contains("bitcoin-core"),
        "expected bitcoin-core-scoped reject; stderr: {stderr}"
    );
    assert!(
        stderr.contains("multipath") || stderr.contains("<0;1>") || stderr.contains("<a;b>"),
        "expected the multipath-remedy reject text; stderr: {stderr}"
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
/// `/1/*`, distinct `internal` flags) also rejects — both entries carry a
/// fixed step, so entry 0 rejects before entry 1 (or selection) is ever
/// reached. Distinct from `core_fixture_file_mainnet_receive_change_pair_parses`
/// (fixture-file variant of the same shape) — this is the inline-blob
/// stdin-driven twin.
#[test]
fn core_receive_change_pair_rejected_with_workaround() {
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
    let assertion = run_core_stdin_select(&blob, "all").failure();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    let code = assertion.get_output().status.code().unwrap_or(-1);
    assert_eq!(
        code, 2,
        "Core receive+change split pair must reject exit 2; stderr: {stderr}"
    );
    assert!(
        stderr.contains("bitcoin-core"),
        "expected bitcoin-core-scoped reject; stderr: {stderr}"
    );
}
