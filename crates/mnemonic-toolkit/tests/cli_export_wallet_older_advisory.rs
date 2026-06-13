//! Task 9 — masked `older()` advisory on `export-wallet` (Adapter-B, two sites).
//!
//! `export-wallet` is an Adapter-B surface. A BIP-68 consensus-masked `older()`
//! operand (a stray bit outside the {low-16, bit-22} window, or a zero 16-bit
//! value such as `older(65536)`) emits a non-blocking advisory on stderr, exit 0
//! — the command never refuses to back up an already-deployed wallet.
//!
//! Two hook sites (PLAN_older_timelock_advisory.md Task 9):
//!   - Site 1: the `--descriptor` passthrough path (after `from_str`, before
//!     `to_string`).
//!   - Site 2: the `--from-import-json` path (after script-type derivation,
//!     BEFORE the taproot early-return — so a masked `older()` is surfaced even
//!     when the command will subsequently refuse a taproot envelope).

use assert_cmd::Command;

fn bin() -> Command {
    Command::cargo_bin("mnemonic").expect("binary built")
}

/// Concrete keys (abstract labels do not parse as a full --descriptor).
/// Lifted from `cli_compare_cost.rs` (KEY_A / KEY_B).
const KEY_A: &str = "02998512205ec6a5cdb77d5b4f7de63c560d1e846162612ee178c49e7b6cc44fb9";
const KEY_B: &str = "03999999999999999999999999999999999999999999999999999999999999999d";
/// BIP-341 NUMS H-point (x-only) for the taproot internal key.
const NUMS: &str = "50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0";

/// Substring of the masked-older advisory (stable across the message body).
const MASKED_OLDER_ADVISORY: &str = "advisory: older(65536) is consensus-masked";

// ── Site 1: --descriptor passthrough ────────────────────────────────────────

/// (a) `export-wallet --descriptor wsh(andor(...older(65536)...)) --format
/// descriptor` → stderr advisory + exit 0. `older(65536)` is masked;
/// `older(2016)` is a clean relative block lock → exactly one advisory.
#[test]
fn export_wallet_descriptor_masked_older_emits_advisory() {
    let desc = format!("wsh(andor(pk({KEY_A}),older(65536),and_v(v:pk({KEY_B}),older(2016))))");
    let out = bin()
        .args([
            "export-wallet",
            "--descriptor",
            &desc,
            "--format",
            "descriptor",
        ])
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(0),
        "masked older() is an advisory, not fatal; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains(MASKED_OLDER_ADVISORY),
        "expected masked-older advisory on stderr; got: {stderr}"
    );
}

/// Clean case (Site 1): a valid relative block lock `older(2016)` (within
/// low-16, no stray bits) → NO advisory on stderr, exit 0.
#[test]
fn export_wallet_descriptor_clean_older_no_advisory() {
    let desc = format!("wsh(and_v(v:pk({KEY_A}),older(2016)))");
    let out = bin()
        .args([
            "export-wallet",
            "--descriptor",
            &desc,
            "--format",
            "descriptor",
        ])
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("consensus-masked"),
        "clean older() must not emit a masked advisory; got: {stderr}"
    );
}

// ── Site 2: --from-import-json (advisory fires BEFORE a later refuse) ─────────

/// Build a minimal `import-wallet --json` envelope carrying `descriptor`. The
/// `--from-import-json` path parses this envelope and derives the script-type
/// from `bundle.descriptor`; the Site-2 hook fires the advisory just after that
/// derivation, before any taproot refuse.
fn envelope_with_descriptor(descriptor: &str) -> String {
    format!(
        r#"[
  {{
    "bundle": {{
      "account": 0,
      "descriptor": "{descriptor}",
      "master_fingerprint": null,
      "md1": [],
      "mk1": [],
      "mode": "watch-only",
      "ms1": [],
      "network": "mainnet",
      "origin_path": null,
      "origin_paths": null,
      "privacy_preserving": false,
      "schema_version": "4",
      "template": null
    }},
    "roundtrip": {{"byte_exact": false, "diff": null, "semantic_match": false, "status": "blocked_no_emitter"}},
    "schema_version": "1",
    "source_format": "bsms"
  }}
]"#
    )
}

/// (b) `--from-import-json` with a masked TAPROOT descriptor. The advisory must
/// fire on stderr BEFORE the taproot early-return refuse — proving the hook is
/// upstream of the refuse. The command then refuses (exit != 0), but the masked
/// `older()` is surfaced regardless.
#[test]
fn export_wallet_from_import_json_masked_older_advisory_fires_before_taproot_refuse() {
    let descriptor = format!("tr({NUMS},and_v(v:pk({KEY_A}),older(65536)))");
    let envelope = envelope_with_descriptor(&descriptor);
    let out = bin()
        .args([
            "export-wallet",
            "--from-import-json",
            "-",
            "--format",
            "descriptor",
        ])
        .write_stdin(envelope)
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    // The advisory fires even though the command subsequently refuses taproot.
    assert!(
        stderr.contains(MASKED_OLDER_ADVISORY),
        "masked-older advisory must surface on stderr even when a later taproot \
         refuse occurs; got: {stderr}"
    );
    // Sanity: the later taproot refuse still happens (non-zero exit), confirming
    // the advisory genuinely precedes it.
    assert_ne!(
        out.status.code(),
        Some(0),
        "taproot --from-import-json must still refuse after the advisory; stderr: {stderr}"
    );
    assert!(
        stderr.contains("taproot descriptors are not yet supported"),
        "expected the taproot refuse after the advisory; got: {stderr}"
    );
}

/// Clean case (Site 2): a clean non-taproot envelope exports successfully with
/// NO advisory on stderr.
#[test]
fn export_wallet_from_import_json_clean_no_advisory() {
    let descriptor = format!("wsh(and_v(v:pk({KEY_A}),older(2016)))");
    let envelope = envelope_with_descriptor(&descriptor);
    let out = bin()
        .args([
            "export-wallet",
            "--from-import-json",
            "-",
            "--format",
            "descriptor",
        ])
        .write_stdin(envelope)
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("consensus-masked"),
        "clean older() must not emit a masked advisory; got: {stderr}"
    );
}
