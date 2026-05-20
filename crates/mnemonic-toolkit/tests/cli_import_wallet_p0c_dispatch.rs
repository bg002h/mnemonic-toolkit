//! v0.28.0 Phase P0C — CLI dispatch pre-stub regression cells.
//!
//! Per plan-doc `/home/bcg/.claude/plans/unified-meandering-sundae.md` P0C
//! row and §B.2 #6 8-site enumeration. P0C pre-stubs the dispatch surface
//! for the 6 new formats (sparrow, specter, coldcard, coldcard-multisig,
//! jade, electrum) via `unimplemented!()` arms at Sites 2 and 4. Each
//! per-parser P{N}C sub-phase flips ONE format's arms across all 8 sites
//! to real dispatch; these cells become regression guards for "the
//! pre-stub arm fires when the per-parser dispatch is NOT yet wired".
//!
//! ## What these cells verify
//!
//! - Each `--format <new>` value is ACCEPTED by clap's PossibleValuesParser
//!   (i.e., NOT rejected as "invalid value" by clap itself; the panic comes
//!   from the dispatch site at `cmd/import_wallet.rs` Site 2, not from arg
//!   parsing).
//! - The dispatch arm at Site 2 panics via `unimplemented!()` with a phase
//!   tag (`P{N}C: format <new-format> not yet wired`).
//! - The existing `--format bsms` + `--format bitcoin-core` arms are
//!   regression-untouched (smoke-tested as a control here; dedicated parity
//!   coverage lives in `cli_import_wallet_bsms.rs` +
//!   `cli_import_wallet_bitcoin_core.rs`).
//!
//! ## Cell shape
//!
//! `assert_cmd` does NOT propagate the `unimplemented!()` panic-message text
//! through `stderr` reliably (subprocess panics on stderr but message
//! framing varies by harness). The cells assert exit-NON-zero +
//! exit-NOT-`SUCCESS` and treat that as the regression guard. A finer-grain
//! "stderr contains 'P{N}C: format <X> not yet wired'" assertion is
//! intentionally NOT pinned here — per-parser P{N}C sub-phases REPLACE
//! these cells with happy-path parse cells anyway, so over-pinning the
//! panic text creates a delete-on-arrival regression-cell.

use assert_cmd::Command;
use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from("tests/fixtures/wallet_import").join(name)
}

/// Invoke `mnemonic import-wallet` with the given args.
///
/// `assert_cmd::Command::assert()` consumes the process; we DON'T call
/// `.success()` here because the unimplemented arms panic with a non-zero
/// exit code.
fn run_import(args: &[&str]) -> assert_cmd::assert::Assert {
    let mut cmd = Command::cargo_bin("mnemonic").unwrap();
    cmd.arg("import-wallet").args(args).assert()
}

// ============================================================================
// `--format <new>` arms panic via unimplemented!()  (Site 2 in plan-doc §B.2 #6)
// ============================================================================

// v0.28.0 Phase P1C: `p0c_format_sparrow_panics_unimplemented` REMOVED —
// the dispatch arm no longer panics; sparrow parse is live. Happy-path
// integration coverage lives in `tests/cli_import_wallet_sparrow.rs` per
// the P0C-cell-replacement-on-P{N}C-flip contract documented in this
// file's header.

#[test]
fn p0c_format_specter_panics_unimplemented() {
    let p = fixture_path("bsms-2line-sortedmulti-2of2.txt");
    let out = run_import(&["--blob", p.to_str().unwrap(), "--format", "specter"]).failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("P2C") || stderr.contains("specter"),
        "stderr should mention P2C or specter on unimplemented dispatch; got: {stderr}"
    );
}

#[test]
fn p0c_format_coldcard_panics_unimplemented() {
    let p = fixture_path("bsms-2line-sortedmulti-2of2.txt");
    let out = run_import(&["--blob", p.to_str().unwrap(), "--format", "coldcard"]).failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("P3C") || stderr.contains("coldcard"),
        "stderr should mention P3C or coldcard on unimplemented dispatch; got: {stderr}"
    );
}

/// v0.28.0 Phase P4C UPDATE: this cell was originally the P0C-stub
/// regression guard ("--format coldcard-multisig panics unimplemented");
/// post-P4C the dispatch is wired, so the same BSMS-blob input now
/// surfaces an `ImportWalletFormatMismatch` (supplied=coldcard-multisig,
/// sniffed=bsms) instead of a panic. The cell is preserved at the same
/// location as a dispatch-surface regression guard for the
/// post-P4C-wiring semantic.
#[test]
fn p0c_format_coldcard_multisig_dispatches_format_mismatch_post_p4c() {
    let p = fixture_path("bsms-2line-sortedmulti-2of2.txt");
    let out = run_import(&[
        "--blob",
        p.to_str().unwrap(),
        "--format",
        "coldcard-multisig",
    ])
    .failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("coldcard-multisig") && stderr.contains("bsms"),
        "stderr should cite format mismatch (supplied=coldcard-multisig vs sniffed=bsms); \
         got: {stderr}"
    );
}

#[test]
fn p0c_format_jade_panics_unimplemented() {
    let p = fixture_path("bsms-2line-sortedmulti-2of2.txt");
    let out = run_import(&["--blob", p.to_str().unwrap(), "--format", "jade"]).failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("P5C") || stderr.contains("jade"),
        "stderr should mention P5C or jade on unimplemented dispatch; got: {stderr}"
    );
}

#[test]
fn p0c_format_electrum_panics_unimplemented() {
    let p = fixture_path("bsms-2line-sortedmulti-2of2.txt");
    let out = run_import(&["--blob", p.to_str().unwrap(), "--format", "electrum"]).failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("P6C") || stderr.contains("electrum"),
        "stderr should mention P6C or electrum on unimplemented dispatch; got: {stderr}"
    );
}

// ============================================================================
// PossibleValuesParser ACCEPTS all 8 alphabetical formats (Site 1)
// ============================================================================

#[test]
fn p0c_format_arg_rejects_out_of_set_value() {
    // clap's PossibleValuesParser rejects `gobbledygook` (NOT in the 8-value
    // set). Failure must come from clap-arg-parse — NOT from a dispatch-site
    // panic. The fallback `Some(other) =>` BadInput arm at the dispatch site
    // is unreachable via clap (PossibleValuesParser rejects first); the
    // fallback remains as defense-in-depth.
    let p = fixture_path("bsms-2line-sortedmulti-2of2.txt");
    let out = run_import(&[
        "--blob",
        p.to_str().unwrap(),
        "--format",
        "gobbledygook",
    ])
    .failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("invalid value")
            || stderr.contains("possible values")
            || stderr.contains("gobbledygook"),
        "clap should reject out-of-set --format value; stderr: {stderr}"
    );
}

// ============================================================================
// Existing v0.26.0 dispatch arms are regression-untouched (Site 2 + Site 4)
// ============================================================================

#[test]
fn p0c_existing_format_bsms_still_dispatches() {
    // Smoke: `--format bsms` still routes through `BsmsParser::parse` and
    // produces the expected summary. Full coverage in
    // `tests/cli_import_wallet_bsms.rs`; this cell is the P0C-local
    // regression-guard.
    let p = fixture_path("bsms-2line-sortedmulti-2of2.txt");
    let out = run_import(&["--blob", p.to_str().unwrap(), "--format", "bsms"]).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(
        stdout.contains("cosigners="),
        "format=bsms must still parse + emit summary; stdout: {stdout}"
    );
}

#[test]
fn p0c_existing_format_bitcoin_core_still_dispatches() {
    let p = fixture_path("core-bip84-mainnet.json");
    let out = run_import(&[
        "--blob",
        p.to_str().unwrap(),
        "--format",
        "bitcoin-core",
    ])
    .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(
        stdout.contains("cosigners=") || stdout.contains("bundles="),
        "format=bitcoin-core must still parse + emit summary; stdout: {stdout}"
    );
}

// ============================================================================
// Stderr templates enumerate all 8 formats (Site 3 — only template strings
// changed at P0C; the auto-sniff arm bodies are untouched per plan-doc)
// ============================================================================

#[test]
fn p0c_no_match_stderr_template_lists_8_formats() {
    // Send a blob that sniff_format classifies as NoMatch; the stderr
    // template now enumerates all 8 supported `--format` values.
    use std::io::Write;
    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    writeln!(tmp, "totally random not-a-wallet text").unwrap();
    let p = tmp.path();

    let out = run_import(&["--blob", p.to_str().unwrap()]).failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("could not detect format"),
        "stderr should contain NoMatch template; got: {stderr}"
    );
    // Enumeration check: each of the 8 formats appears in the template.
    for fmt in &[
        "bitcoin-core",
        "bsms",
        "coldcard",
        "coldcard-multisig",
        "electrum",
        "jade",
        "sparrow",
        "specter",
    ] {
        assert!(
            stderr.contains(fmt),
            "NoMatch stderr template should enumerate `{fmt}`; got: {stderr}"
        );
    }
}
