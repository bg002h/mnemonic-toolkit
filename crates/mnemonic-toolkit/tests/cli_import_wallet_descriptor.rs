//! C5 — `import-wallet --format descriptor`: generic commented-descriptor intake.
//!
//! Reads a watch-only descriptor from text, tolerating leading `#`-comment lines +
//! blank lines (subsumes `export-wallet --format green`/`--format descriptor`
//! output + hand-written/foreign commented descriptors). Singlesig AND multisig.
//! Explicit-only (no auto-sniff). Checksum TOLERANT (validate-if-present).
//!
//! Ships v0.58.0. FOLLOWUP `import-wallet-format-descriptor`.

use assert_cmd::Command;
use serde_json::Value;

fn bin() -> Command {
    Command::cargo_bin("mnemonic").expect("binary built")
}

/// Two origin-annotated mainnet account xpubs (from cli_bip388_policy_intake.rs).
const A: &str = "xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX";
const B: &str = "xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6";

/// `import-wallet --format descriptor --blob -` with the given stdin text.
fn import_descriptor(stdin: &str) -> assert_cmd::assert::Assert {
    bin()
        .args(["import-wallet", "--format", "descriptor", "--blob", "-"])
        .write_stdin(stdin.to_string())
        .assert()
}

// ── Positive cells ────────────────────────────────────────────────────────────

/// Singlesig, checksum-LESS, with leading `#`-comments + a blank line (the green
/// 3-line export shape). Confirms tolerant-checksum + comment-strip.
#[test]
fn singlesig_commented_checksumless_imports() {
    let blob = format!(
        "# Blockstream Green - Watch-only import (singlesig)\n# Help: https://example\n\nwpkh([704c7836/84'/0'/0']{A}/<0;1>/*)\n"
    );
    let out = import_descriptor(&blob).success();
    let s = String::from_utf8_lossy(&out.get_output().stdout);
    assert!(s.contains("cosigners=1"), "singlesig → 1 cosigner: {s}");
    assert!(s.contains("network=mainnet"), "mainnet: {s}");
}

/// Multisig 2-of-2 sortedmulti with origin-annotated keys → 2 cosigners,
/// threshold 2. The "more general than green" proof (green-export refuses
/// multisig; descriptor-import accepts it).
#[test]
fn multisig_sortedmulti_imports_with_threshold() {
    let blob = format!(
        "# 2-of-2 vault\nwsh(sortedmulti(2,[704c7836/48'/0'/0'/2']{A}/<0;1>/*,[97139860/48'/0'/0'/2']{B}/<0;1>/*))\n"
    );
    let out = import_descriptor(&blob).success();
    let s = String::from_utf8_lossy(&out.get_output().stdout);
    assert!(s.contains("cosigners=2"), "2-of-2 → 2 cosigners: {s}");
    assert!(s.contains("threshold=2"), "threshold 2: {s}");
}

/// Real round-trip WITH a checksum: `export-wallet --format descriptor` emits a
/// `<desc>#csum`; re-importing it via `--format descriptor` succeeds (validates
/// the present checksum).
#[test]
fn export_descriptor_roundtrips_back_through_import() {
    let exported = bin()
        .args([
            "export-wallet",
            "--descriptor",
            &format!("wpkh([704c7836/84'/0'/0']{A}/<0;1>/*)"),
            "--format",
            "descriptor",
        ])
        .assert()
        .success();
    let out = String::from_utf8(exported.get_output().stdout.clone()).unwrap();
    // The descriptor line carries a `#csum`.
    let desc_line = out
        .lines()
        .find(|l| l.starts_with("wpkh("))
        .expect("a descriptor line");
    assert!(
        desc_line.contains('#'),
        "export carries a checksum: {desc_line}"
    );
    let reimport = import_descriptor(&format!("# round-trip\n{desc_line}\n")).success();
    let s = String::from_utf8_lossy(&reimport.get_output().stdout);
    assert!(
        s.contains("cosigners=1") && s.contains("network=mainnet"),
        "{s}"
    );
}

/// `--json` envelope reports `source_format: "descriptor"`, watch-only.
#[test]
fn json_envelope_source_format_descriptor() {
    let blob = format!("wpkh([704c7836/84'/0'/0']{A}/<0;1>/*)\n");
    let out = bin()
        .args([
            "import-wallet",
            "--format",
            "descriptor",
            "--blob",
            "-",
            "--json",
        ])
        .write_stdin(blob)
        .assert()
        .success();
    let v: Value = serde_json::from_slice(&out.get_output().stdout).expect("valid JSON");
    // `--json` emits an array (one element per parsed bundle); source_format is
    // per-element.
    assert_eq!(v[0]["source_format"], "descriptor", "source_format: {v}");
    assert_eq!(v[0]["bundle"]["mode"], "watch-only", "watch-only: {v}");
}

// ── Negative cells ────────────────────────────────────────────────────────────

/// Explicit-only: a bare descriptor WITHOUT `--format descriptor` does NOT
/// auto-sniff (the parser is absent from the sniff votes) → "could not detect".
#[test]
fn bare_descriptor_without_format_refuses_no_sniff() {
    bin()
        .args(["import-wallet", "--blob", "-"])
        .write_stdin(format!("wpkh([704c7836/84'/0'/0']{A}/<0;1>/*)\n"))
        .assert()
        .failure();
}

/// A file with NO descriptor line (only comments/blanks) → refuse loudly.
#[test]
fn no_descriptor_line_refused() {
    import_descriptor("# only a comment\n\n# another\n")
        .failure()
        .stderr(predicates::str::contains("no descriptor line"));
}

/// A file with TWO descriptor lines → refuse loudly.
#[test]
fn two_descriptor_lines_refused() {
    let blob =
        format!("wpkh([704c7836/84'/0'/0']{A}/<0;1>/*)\nwpkh([97139860/84'/0'/0']{B}/<0;1>/*)\n");
    import_descriptor(&blob)
        .failure()
        .stderr(predicates::str::contains("expected a single descriptor"));
}

/// A BAD checksum is refused (tolerant ≠ ignored — a present checksum is validated).
#[test]
fn bad_checksum_refused() {
    let blob = format!("wpkh([704c7836/84'/0'/0']{A}/<0;1>/*)#deadbeef\n");
    import_descriptor(&blob)
        .failure()
        .stderr(predicates::str::contains("checksum"));
}

// ── Cycle A residue-reject floor (CRITICAL funds fix) ─────────────────────────
//
// I-1 (Phase-1 R0 fold): `import-wallet --format descriptor` is a lex-surface —
// its concrete descriptor flows through `concrete_keys_to_placeholders` →
// `parse_descriptor::parse_descriptor` (descriptor.rs step 3), so a FIXED
// use-site step (`/0/*`) or the BIP-389 combined shorthand (`/**`) is
// un-representable in md1 (SPEC_cycleA_descriptor_use_site_collapse.md §1/§6)
// and rejects at the shared residue-reject floor. These cells lock the
// per-surface CLI behavior + message (born-green — the lexer already rejects).

/// A single `/0/*` descriptor (fixed use-site step) hard-rejects exit 2 with
/// the multipath-remedy message. Checksum-LESS (the parser is tolerant), so
/// the reject fires at the residue floor, not the checksum validator.
#[test]
fn descriptor_fixed_use_site_step_rejected_with_multipath_remedy() {
    let blob = format!("wpkh([704c7836/84'/0'/0']{A}/0/*)\n");
    let assertion = import_descriptor(&blob).failure();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    let code = assertion.get_output().status.code().unwrap_or(-1);
    assert_eq!(
        code, 2,
        "fixed use-site step `/0/*` must reject exit 2; stderr: {stderr}"
    );
    assert!(
        stderr.contains("descriptor"),
        "reject must be scoped to the descriptor surface; stderr: {stderr}"
    );
    assert!(
        stderr.contains("multipath") && stderr.contains("<a;b>"),
        "expected the multipath `/<a;b>/*` remedy pointer; stderr: {stderr}"
    );
    assert!(
        stderr.contains("/0/*"),
        "reject must name the offending fixed-step residue; stderr: {stderr}"
    );
}

/// The BIP-389 combined `/**` shorthand (HIGHEST-impact case — mainstream
/// wallet export form) hard-rejects exit 2. The `wild` group eats only `/*`,
/// leaving a stray residue → reject; the message must name the `/**` shorthand
/// and the multipath remedy (plan-R0 I-D).
#[test]
fn descriptor_double_star_shorthand_rejected_with_multipath_remedy() {
    let blob = format!("wpkh([704c7836/84'/0'/0']{A}/**)\n");
    let assertion = import_descriptor(&blob).failure();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    let code = assertion.get_output().status.code().unwrap_or(-1);
    assert_eq!(
        code, 2,
        "the `/**` shorthand must reject exit 2; stderr: {stderr}"
    );
    assert!(
        stderr.contains("descriptor"),
        "reject must be scoped to the descriptor surface; stderr: {stderr}"
    );
    assert!(
        stderr.contains("/**"),
        "reject must name the `/**` shorthand explicitly; stderr: {stderr}"
    );
    assert!(
        stderr.contains("multipath") && stderr.contains("<a;b>"),
        "expected the multipath `/<a;b>/*` remedy pointer; stderr: {stderr}"
    );
}
