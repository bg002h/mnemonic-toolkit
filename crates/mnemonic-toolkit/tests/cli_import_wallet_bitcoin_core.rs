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
// §3.3 — core_multi_descriptor_emit_all
// ============================================================================

#[test]
fn core_multi_descriptor_emit_all() {
    // 4 entries: BIP-84 receive + change, BIP-49 receive + change.
    let d0 = format!("wpkh([{MAINNET_FP_A}/84'/0'/0']{MAINNET_XPUB_A}/0/*)");
    let d1 = format!("wpkh([{MAINNET_FP_A}/84'/0'/0']{MAINNET_XPUB_A}/1/*)");
    let d2 = format!("sh(wpkh([{MAINNET_FP_B}/49'/0'/0']{MAINNET_XPUB_B}/0/*))");
    let d3 = format!("sh(wpkh([{MAINNET_FP_B}/49'/0'/0']{MAINNET_XPUB_B}/1/*))");
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
    let d0 = format!("wpkh([{MAINNET_FP_A}/84'/0'/0']{MAINNET_XPUB_A}/0/*)");
    let d1 = format!("wpkh([{MAINNET_FP_A}/84'/0'/0']{MAINNET_XPUB_A}/1/*)");
    let d2 = format!("sh(wpkh([{MAINNET_FP_B}/49'/0'/0']{MAINNET_XPUB_B}/0/*))");
    let d3 = format!("sh(wpkh([{MAINNET_FP_B}/49'/0'/0']{MAINNET_XPUB_B}/1/*))");
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
    let d0 = format!("wpkh([{MAINNET_FP_A}/84'/0'/0']{MAINNET_XPUB_A}/0/*)");
    let d1 = format!("wpkh([{MAINNET_FP_A}/84'/0'/0']{MAINNET_XPUB_A}/1/*)");
    let d2 = format!("sh(wpkh([{MAINNET_FP_B}/49'/0'/0']{MAINNET_XPUB_B}/0/*))");
    let d3 = format!("sh(wpkh([{MAINNET_FP_B}/49'/0'/0']{MAINNET_XPUB_B}/1/*))");
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
    let d0 = format!("wpkh([{MAINNET_FP_A}/84'/0'/0']{MAINNET_XPUB_A}/0/*)");
    let d1 = format!("wpkh([{MAINNET_FP_A}/84'/0'/0']{MAINNET_XPUB_A}/1/*)");
    let d2 = format!("sh(wpkh([{MAINNET_FP_B}/49'/0'/0']{MAINNET_XPUB_B}/0/*))");
    let d3 = format!("sh(wpkh([{MAINNET_FP_B}/49'/0'/0']{MAINNET_XPUB_B}/1/*))");
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
    let desc = format!(
        "wsh(sortedmulti(2,[{MAINNET_FP_A}/48'/0'/0'/2']{MAINNET_XPUB_A}/0/*,[{MAINNET_FP_B}/48'/0'/0'/2']{MAINNET_XPUB_B}/0/*,[{MAINNET_FP_C}/48'/0'/0'/2']{MAINNET_XPUB_C}/0/*))"
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
    let desc = format!(
        "wsh(sortedmulti(2,[{TESTNET_FP_A}/48'/1'/0'/2']{TESTNET_XPUB_A}/0/*,[{TESTNET_FP_B}/48'/1'/0'/2']{TESTNET_XPUB_B}/0/*))"
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
    // Lacks top-level `descriptors` array -> parse error.
    let specter_like = "{\"label\":\"Daily\",\"blockheight\":0,\"descriptor\":\"wpkh([b8688df1/84'/0'/0']xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX/<0;1>/*)#00lx6ere\",\"devices\":[\"unknown\"]}";
    let assert = run_core_stdin(specter_like).failure();
    let code = assert.get_output().status.code().unwrap_or(-1);
    assert_eq!(code, 2);
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
