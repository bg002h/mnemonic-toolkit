//! Task 11 — cross-surface masked-`older()` advisory regression test.
//!
//! The per-surface tests (`cli_compare_cost.rs`, `cli_bundle_full.rs`,
//! `cli_export_wallet_older_advisory.rs`) each pin one surface's hook in
//! isolation. THIS file is the cross-surface contract: it asserts the four
//! invariants that must hold *uniformly* across every advisory surface, so a
//! future hook added/edited on one surface cannot silently drift from the
//! others. (SPEC_older_timelock_advisory / PLAN Task 11.)
//!
//! Invariants pinned here:
//!   1. Fires + non-blocking — the advisory surfaces on `compare-cost`,
//!      `bundle`, AND `export-wallet` for the canonical masked policy, and each
//!      surface still exits 0 (an already-deployed wallet is never refused over
//!      a consensus-masked `older()`).
//!   2. Clean-512s false-positive guard — a clean 512-second-unit timelock
//!      `older(4194305)` (0x400001) emits NO advisory (guards a fat-fingered
//!      bit-22 mask in `older_consensus_masked`).
//!   3. Operand-keyed dedup — the same literal twice collapses to ONE advisory
//!      line; two DISTINCT masked literals (both mask-to-0) keep TWO lines.
//!   4. `--json` stdout cleanliness — the advisory is on stderr; the stdout
//!      JSON payload never contains the advisory text.

use assert_cmd::Command;

fn bin() -> Command {
    Command::cargo_bin("mnemonic").expect("binary built")
}

// ── Key constants (lifted verbatim from the per-surface tests) ───────────────

/// Concrete secp256k1 pubkeys for `--descriptor` surfaces (abstract labels like
/// `K0` only parse under `--miniscript`). Same constants as
/// `cli_compare_cost.rs` / `cli_export_wallet_older_advisory.rs`.
const KEY_A: &str = "02998512205ec6a5cdb77d5b4f7de63c560d1e846162612ee178c49e7b6cc44fb9";
const KEY_B: &str = "03999999999999999999999999999999999999999999999999999999999999999d";

/// Watch-only cosigner xpubs + fingerprints for `bundle --descriptor` slots
/// (lifted from `cli_bundle_full.rs`). Content is advisory-irrelevant; only the
/// descriptor's `older()` operand drives the hook.
const MASKED_FP_A: &str = "b8688df1";
const MASKED_XPUB_A: &str = "xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX";
const MASKED_FP_B: &str = "28645006";
const MASKED_XPUB_B: &str = "xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6";

/// Surface-stable advisory prefix. Every advisory message begins with
/// `advisory: older(` (see `timelock_advisory.rs::TimelockAdvisory::message`),
/// so counting this substring counts advisory lines independent of the
/// per-operand body text.
const ADVISORY_PREFIX: &str = "advisory: older";
/// The canonical masked operand's full advisory substring.
const MASKED_OLDER_65536: &str = "advisory: older(65536) is consensus-masked";

/// The canonical masked policy with CONCRETE keys (for `--descriptor`
/// surfaces): `older(65536)` (0x10000 — stray bit-16, low-16 value 0 → masked)
/// alongside a clean `older(2016)` → exactly one advisory.
fn canonical_masked_descriptor() -> String {
    format!("wsh(andor(pk({KEY_A}),older(65536),and_v(v:pk({KEY_B}),older(2016))))")
}

/// The standard `bundle --descriptor` slot block for the two-cosigner masked
/// multisig descriptor used below.
fn bundle_masked_args(descriptor: &str) -> Vec<String> {
    vec![
        "bundle".into(),
        "--descriptor".into(),
        descriptor.into(),
        "--network".into(),
        "mainnet".into(),
        "--slot".into(),
        format!("@0.xpub={MASKED_XPUB_A}"),
        "--slot".into(),
        format!("@0.fingerprint={MASKED_FP_A}"),
        "--slot".into(),
        "@0.path=48'/0'/0'/2'".into(),
        "--slot".into(),
        format!("@1.xpub={MASKED_XPUB_B}"),
        "--slot".into(),
        format!("@1.fingerprint={MASKED_FP_B}"),
        "--slot".into(),
        "@1.path=48'/0'/0'/2'".into(),
        "--no-engraving-card".into(),
    ]
}

// ── Invariant 1: fires + non-blocking on every surface ───────────────────────

/// `compare-cost --descriptor <masked>` → advisory on stderr, exit 0.
#[test]
fn fires_and_non_blocking_compare_cost() {
    let desc = canonical_masked_descriptor();
    let out = bin()
        .args(["compare-cost", "--descriptor", &desc])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(0),
        "compare-cost: masked older() must be advisory, not fatal; stderr: {stderr}"
    );
    assert!(
        stderr.contains(MASKED_OLDER_65536),
        "compare-cost: expected masked-older advisory on stderr; got: {stderr}"
    );
}

/// `bundle --descriptor <masked-multisig>` → advisory on stderr, exit 0
/// (Adapter-A, Site 1).
#[test]
fn fires_and_non_blocking_bundle() {
    // multi(2,...) wrapper so the descriptor resolves against the two slots.
    let descriptor = "wsh(and_v(v:multi(2,@0/<0;1>/*,@1/<0;1>/*),older(65536)))";
    let out = bin()
        .args(bundle_masked_args(descriptor))
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(0),
        "bundle: masked older() must be advisory, not fatal; stderr: {stderr}"
    );
    assert!(
        stderr.contains(MASKED_OLDER_65536),
        "bundle: expected masked-older advisory on stderr; got: {stderr}"
    );
}

/// `export-wallet --descriptor <masked> --format descriptor` → advisory on
/// stderr, exit 0 (Adapter-B, Site 1 — the passthrough path that exits 0; the
/// per-surface file separately pins Site 2's fire-before-taproot-refuse).
#[test]
fn fires_and_non_blocking_export_wallet() {
    let desc = canonical_masked_descriptor();
    let out = bin()
        .args(["export-wallet", "--descriptor", &desc, "--format", "descriptor"])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(0),
        "export-wallet: masked older() must be advisory, not fatal; stderr: {stderr}"
    );
    assert!(
        stderr.contains(MASKED_OLDER_65536),
        "export-wallet: expected masked-older advisory on stderr; got: {stderr}"
    );
}

// ── Invariant 2: clean-512s false-positive guard ─────────────────────────────

/// `older(4194305)` = 0x400001 — bit-22 (the 512-second type flag) set with a
/// clean non-zero low-16 value (1) → a VALID 512-second relative timelock, NOT
/// masked. The advisory must NOT fire. Guards a fat-fingered bit-22 mask in
/// `older_consensus_masked` (which would mis-flag every 512-second lock).
#[test]
fn clean_512s_lock_no_false_positive_advisory() {
    // miniscript surface is enough: the predicate is shared by all surfaces.
    let out = bin()
        .args([
            "compare-cost",
            "--miniscript",
            "and_v(v:pk(K0),older(4194305))",
        ])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(0),
        "clean 512s lock must succeed; stderr: {stderr}"
    );
    assert!(
        !stderr.contains(ADVISORY_PREFIX),
        "clean 512-second older(4194305) must NOT emit an older() advisory; got: {stderr}"
    );
}

// ── Invariant 3: operand-keyed dedup ─────────────────────────────────────────

/// Same masked literal twice (`older(65536)` in both arms) → the advisory line
/// appears EXACTLY ONCE (operand-keyed dedup via `older_advisories_ms`'s
/// `seen` set).
#[test]
fn dedup_same_literal_twice_one_advisory() {
    let out = bin()
        .args([
            "compare-cost",
            "--miniscript",
            "andor(pk(K0),older(65536),and_v(v:pk(K1),older(65536)))",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stderr = String::from_utf8_lossy(&out.stderr);
    let count = stderr.matches(ADVISORY_PREFIX).count();
    assert_eq!(
        count, 1,
        "duplicate older(65536) operand must collapse to one advisory; got {count}: {stderr}"
    );
}

/// Two DISTINCT masked literals (`older(65536)` = 0x10000 and `older(131072)` =
/// 0x20000 — both stray-bit, low-16 value 0 → both mask-to-0) keep TWO advisory
/// lines: dedup is keyed on the OPERAND, not on the (identical) mask
/// consequence.
#[test]
fn dedup_two_distinct_literals_two_advisories() {
    let out = bin()
        .args([
            "compare-cost",
            "--miniscript",
            "andor(pk(K0),older(65536),and_v(v:pk(K1),older(131072)))",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stderr = String::from_utf8_lossy(&out.stderr);
    let count = stderr.matches(ADVISORY_PREFIX).count();
    assert_eq!(
        count, 2,
        "two distinct masked operands (65536, 131072) must keep two advisories; got {count}: {stderr}"
    );
    // Both operands named explicitly (not just two lines of the same text).
    assert!(
        stderr.contains("advisory: older(65536)"),
        "missing older(65536) advisory line: {stderr}"
    );
    assert!(
        stderr.contains("advisory: older(131072)"),
        "missing older(131072) advisory line: {stderr}"
    );
}

// ── Invariant 4: --json stdout cleanliness ───────────────────────────────────

/// `compare-cost --json` with a masked operand → the advisory is on STDERR; the
/// stdout JSON payload does NOT contain the advisory text (guards a future
/// regression that inlines the advisory into the structured stdout envelope).
#[test]
fn json_stdout_clean_advisory_on_stderr_compare_cost() {
    let desc = canonical_masked_descriptor();
    let out = bin()
        .args(["compare-cost", "--descriptor", &desc, "--json"])
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(0),
        "compare-cost --json: stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    // stdout must be valid JSON and carry NO advisory text.
    let _: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("compare-cost --json emits valid JSON on stdout");
    assert!(
        !stdout.contains(ADVISORY_PREFIX),
        "advisory must NOT leak into stdout JSON; stdout: {stdout}"
    );
    assert!(
        stderr.contains(MASKED_OLDER_65536),
        "advisory must be on stderr under --json; stderr: {stderr}"
    );
}

/// Same stdout-cleanliness contract on `bundle --json` (a second `--json`
/// surface; export-wallet has no own `--json` shape for the descriptor
/// passthrough path, so the two structured-stdout surfaces are compare-cost and
/// bundle).
#[test]
fn json_stdout_clean_advisory_on_stderr_bundle() {
    let descriptor = "wsh(and_v(v:multi(2,@0/<0;1>/*,@1/<0;1>/*),older(65536)))";
    let mut args = bundle_masked_args(descriptor);
    args.push("--json".into());
    let out = bin().args(args).output().unwrap();
    assert_eq!(
        out.status.code(),
        Some(0),
        "bundle --json: stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stdout.contains(ADVISORY_PREFIX),
        "advisory must NOT leak into bundle --json stdout; stdout: {stdout}"
    );
    assert!(
        stderr.contains(MASKED_OLDER_65536),
        "advisory must be on stderr under bundle --json; stderr: {stderr}"
    );
}
