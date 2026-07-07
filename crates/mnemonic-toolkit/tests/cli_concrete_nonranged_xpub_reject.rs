//! FOLLOWUP `concrete-nonranged-xpub-implied-wildcard` (Cycle D, v0.79.0).
//!
//! `design/SPEC_concrete_nonranged_xpub_implied_wildcard.md` §6;
//! `design/IMPLEMENTATION_PLAN_concrete_nonranged_xpub_implied_wildcard.md` Phase P0.
//!
//! A concrete non-ranged descriptor `wpkh([fp/path]xpub)` — no `/…` derivation
//! suffix at all — was previously SILENTLY ranged to `wpkh(@0/*)` on encode
//! (md1 always wildcards the use-site), engraving a materially DIFFERENT
//! wallet than the one named, and `verify-bundle --descriptor` FALSE-PASSED
//! (exit 0) against that wrong card (same class as Cycle A's C1). The fix
//! rejects at the `concrete_keys_to_placeholders` substitution layer — the
//! ONLY place that knows a `[fp/path]xpub` came from a REAL concrete key
//! (vs. a hand-typed bare `@N` template, which is structurally unreachable
//! here since `key_regex` never matches `@N`).
//!
//! Fixtures: reused REAL (non-synthetic) mainnet xpubs already load-bearing
//! elsewhere in this test corpus — `FP_A`/`XPUB_A` and `FP_B`/`XPUB_B` are
//! the same constants as `cli_bip388_double_star_shorthand.rs` (there:
//! `FP_A`/`A`, `FP_B`/`B`), reused across `84'/0'/0'` (wpkh), `86'/0'/0'`
//! (taproot), and `48'/0'/0'/2'` (multisig) origins — `key_regex`'s path
//! group has no semantic tie to the account purpose, only syntax.

use assert_cmd::Command;
use serde_json::Value;

const FP_A: &str = "704c7836";
const XPUB_A: &str = "xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX";
const FP_B: &str = "97139860";
const XPUB_B: &str = "xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6";

fn bin() -> Command {
    Command::cargo_bin("mnemonic").expect("binary built")
}

/// Flatten a `bundle --json` `md1` array into `--md1 <chunk>` flag pairs.
fn md1_flags(v: &Value) -> Vec<String> {
    let mut out = Vec::new();
    for chunk in v["md1"].as_array().expect("md1 array") {
        out.push("--md1".into());
        out.push(chunk.as_str().unwrap().to_string());
    }
    out
}

/// Flatten a `bundle --json` `mk1` field (flat OR array-of-arrays, singlesig
/// vs. multisig) into `--mk1 <chunk>` flag pairs.
fn mk1_flags(v: &Value) -> Vec<String> {
    let mut out = Vec::new();
    for el in v["mk1"].as_array().expect("mk1 array") {
        match el {
            Value::String(s) => {
                out.push("--mk1".into());
                out.push(s.clone());
            }
            Value::Array(inner) => {
                for chunk in inner {
                    out.push("--mk1".into());
                    out.push(chunk.as_str().unwrap().to_string());
                }
            }
            other => panic!("unexpected mk1 element shape: {other:?}"),
        }
    }
    out
}

/// `bundle --descriptor <desc>` then `verify-bundle --descriptor <desc>`
/// against the SAME produced cards — the standard positive round-trip
/// oracle (mirrors `cli_descriptor_concrete.rs`).
fn bundle_then_verify_round_trip(desc: &str) {
    let produced = bin()
        .args([
            "bundle",
            "--descriptor",
            desc,
            "--network",
            "mainnet",
            "--json",
        ])
        .assert()
        .success();
    let v: Value = serde_json::from_slice(&produced.get_output().stdout).unwrap();
    let mut args: Vec<String> = vec![
        "verify-bundle".into(),
        "--descriptor".into(),
        desc.into(),
        "--network".into(),
        "mainnet".into(),
    ];
    args.extend(md1_flags(&v));
    args.extend(mk1_flags(&v));
    let out = bin().args(&args).output().unwrap();
    assert!(
        out.status.success(),
        "round-trip verify must succeed for {desc}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

// ============================================================================
// §6.1 — REJECT: bundle a concrete non-ranged singlesig descriptor.
// ============================================================================

#[test]
fn bundle_concrete_wpkh_no_derivation_suffix_rejects() {
    let desc = format!("wpkh([{FP_A}/84'/0'/0']{XPUB_A})");
    let out = bin()
        .args([
            "bundle",
            "--descriptor",
            &desc,
            "--network",
            "mainnet",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(
        !out.status.success(),
        "a concrete non-ranged xpub must be REJECTED (was: silently ranged to @0/*)"
    );
    assert_eq!(
        out.status.code(),
        Some(2),
        "exit 2 / DescriptorParse (bundle concrete fork)"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("@0"),
        "must name the offending key @0: {stderr}"
    );
    assert!(
        stderr.contains("no derivation suffix"),
        "must state the un-representable-non-ranged reason: {stderr}"
    );
    assert!(
        stderr.contains("/*") && stderr.contains("/<0;1>/*"),
        "must point at BOTH remedies (ranged `/*` / multipath `/<0;1>/*`): {stderr}"
    );
    assert!(
        !stderr.contains("import-wallet: bsms:"),
        "bundle's concrete fork must remap the prefix (no bsms: leak): {stderr}"
    );
}

// ============================================================================
// §6.2 — FUNDS ANCHOR: the verify-bundle false-pass is genuinely CLOSED, at
// re-parse, BEFORE any card comparison.
// ============================================================================

#[test]
fn verify_bundle_concrete_no_derivation_false_pass_closed() {
    // The non-ranged spelling can no longer be bundled directly (§6.1), so
    // build the SAME wallet's card via the ranged `/*` spelling — this is
    // the IDENTICAL card the pre-fix silent-ranging bug would have produced
    // from the non-ranged spelling (md1 always wildcards the use-site).
    let ranged_desc = format!("wpkh([{FP_A}/84'/0'/0']{XPUB_A}/*)");
    let produced = bin()
        .args([
            "bundle",
            "--descriptor",
            &ranged_desc,
            "--network",
            "mainnet",
            "--json",
        ])
        .assert()
        .success();
    let v: Value = serde_json::from_slice(&produced.get_output().stdout).unwrap();

    let non_ranged_desc = format!("wpkh([{FP_A}/84'/0'/0']{XPUB_A})");
    let mut args: Vec<String> = vec![
        "verify-bundle".into(),
        "--descriptor".into(),
        non_ranged_desc,
        "--network".into(),
        "mainnet".into(),
    ];
    args.extend(md1_flags(&v));
    args.extend(mk1_flags(&v));

    let out = bin().args(&args).output().unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(
        !out.status.success(),
        "verify-bundle must no longer FALSE-PASS (pre-fix: exit 0 / `result: ok` against a \
         card encoding a materially different — ranged — wallet): {stderr}"
    );
    assert_eq!(
        out.status.code(),
        Some(2),
        "exit 2 / DescriptorParse — the concrete-descriptor verify fork: {stderr}"
    );
    // THE FUNDS ANCHOR: assert the PARSE-REJECT message text, not merely a
    // non-zero exit code (a card-comparison mismatch would ALSO exit != 0
    // with different text). This proves the reject fired at RE-PARSE,
    // before `verify_emit_from_expected` ever compared the supplied cards.
    assert!(
        stderr.contains("@0") && stderr.contains("no derivation suffix"),
        "must be the re-parse REJECT message (proves the reject fires BEFORE card \
         comparison), not a card-mismatch message: {stderr}"
    );
    assert!(
        !stderr.contains("result: ok"),
        "must never emit the false-pass ok report: {stderr}"
    );
    assert!(
        !stderr.contains("import-wallet: bsms:"),
        "verify-bundle's concrete fork must remap the prefix (no bsms: leak): {stderr}"
    );
}

// ============================================================================
// §6.3/§6.4 — ACCEPT (regression): ranged spellings still bundle + verify.
// ============================================================================

#[test]
fn bundle_concrete_wpkh_ranged_single_path_round_trips() {
    bundle_then_verify_round_trip(&format!("wpkh([{FP_A}/84'/0'/0']{XPUB_A}/*)"));
}

#[test]
fn bundle_concrete_wpkh_multipath_round_trips() {
    bundle_then_verify_round_trip(&format!("wpkh([{FP_A}/84'/0'/0']{XPUB_A}/<0;1>/*)"));
}

// ============================================================================
// §6.5 — ACCEPT (regression): hand-typed LITERAL @N template is UNAFFECTED —
// it routes via the AtN direct-lex path and never reaches `key_regex` /
// `concrete_keys_to_placeholders` (structurally unreachable, not merely
// untested). Companion to the unit-level pin
// `lex_residue_floor_accepts_bare_at_n_d1_deferred` (parse_descriptor.rs),
// kept green + unmodified.
// ============================================================================

#[test]
fn bundle_atn_bare_template_unaffected_by_new_check() {
    let out = bin()
        .args([
            "bundle",
            "--descriptor",
            "wpkh(@0)",
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.xpub={XPUB_A}"),
            "--slot",
            &format!("@0.fingerprint={FP_A}"),
            "--json",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "a hand-typed bare @N template must be UNAFFECTED by the new concrete-key check: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

// ============================================================================
// §6.6 — REJECT: multisig with ONE non-ranged concrete key (first cosigner).
// ============================================================================

#[test]
fn bundle_multisig_first_key_nonranged_rejects_names_at0() {
    let desc = format!(
        "wsh(sortedmulti(2,[{FP_A}/48'/0'/0'/2']{XPUB_A},[{FP_B}/48'/0'/0'/2']{XPUB_B}/*))"
    );
    let out = bin()
        .args([
            "bundle",
            "--descriptor",
            &desc,
            "--network",
            "mainnet",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(
        !out.status.success(),
        "one non-ranged cosigner among ranged ones must still reject"
    );
    assert_eq!(out.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("@0") && stderr.contains("no derivation suffix"),
        "must name the offending @0 cosigner: {stderr}"
    );
}

// ============================================================================
// §6.7 — the Cycle-A fixed-step floor is UNCHANGED: `/0/*` still rejects,
// via the PRE-EXISTING residue floor, NOT the new check (the new check sees
// the leading `/` and passes through).
// ============================================================================

#[test]
fn bundle_concrete_fixed_step_still_rejects_via_cycle_a_floor_not_new_check() {
    let desc = format!("wpkh([{FP_A}/84'/0'/0']{XPUB_A}/0/*)");
    let out = bin()
        .args([
            "bundle",
            "--descriptor",
            &desc,
            "--network",
            "mainnet",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(!out.status.success(), "a fixed /0/* step must still reject");
    assert_eq!(out.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("multipath") && stderr.contains("<a;b>"),
        "must be the pre-existing Cycle-A floor reject message: {stderr}"
    );
    assert!(
        !stderr.contains("no derivation suffix"),
        "the NEW check must NOT fire here (the key IS followed by `/`): {stderr}"
    );
}

// ============================================================================
// §6.8 — import-wallet: both `--format descriptor` (remapped prefix) and
// `--format bsms` (own-format prefix, legitimately KEEPS `import-wallet:
// bsms:` — M3 scope) reject a concrete non-ranged key at the same choke
// point.
// ============================================================================

#[test]
fn import_wallet_format_descriptor_concrete_nonranged_rejects() {
    let blob = format!("wpkh([{FP_A}/84'/0'/0']{XPUB_A})\n");
    let out = bin()
        .args(["import-wallet", "--format", "descriptor", "--blob", "-"])
        .write_stdin(blob)
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert_eq!(out.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("import-wallet: descriptor:"),
        "the bsms: prefix must remap to descriptor: for this surface: {stderr}"
    );
    assert!(
        stderr.contains("@0") && stderr.contains("no derivation suffix"),
        "{stderr}"
    );
}

#[test]
fn import_wallet_format_bsms_concrete_nonranged_rejects() {
    let desc = format!("wpkh([{FP_A}/84'/0'/0']{XPUB_A})");
    let blob = format!("BSMS 1.0\n{desc}\n");
    let out = bin()
        .args(["import-wallet", "--format", "bsms", "--blob", "-"])
        .write_stdin(blob)
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert_eq!(out.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&out.stderr);
    // M3: bsms format LEGITIMATELY keeps its own "import-wallet: bsms:"
    // prefix — no remap for its own native surface.
    assert!(
        stderr.contains("import-wallet: bsms:"),
        "bsms format keeps its own prefix (no remap expected): {stderr}"
    );
    assert!(
        stderr.contains("@0") && stderr.contains("no derivation suffix"),
        "{stderr}"
    );
}

// ============================================================================
// §6.9 — taproot (M1): the mechanism is script-agnostic (keyed on
// `key_regex`, fires in any key position).
// ============================================================================

#[test]
fn bundle_taproot_concrete_no_derivation_rejects_names_at0() {
    let desc = format!("tr([{FP_A}/86'/0'/0']{XPUB_A})");
    let out = bin()
        .args([
            "bundle",
            "--descriptor",
            &desc,
            "--network",
            "mainnet",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(
        !out.status.success(),
        "taproot concrete non-ranged must reject"
    );
    assert_eq!(out.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("@0") && stderr.contains("no derivation suffix"),
        "{stderr}"
    );
}

#[test]
fn bundle_taproot_concrete_multipath_round_trips() {
    bundle_then_verify_round_trip(&format!("tr([{FP_A}/86'/0'/0']{XPUB_A}/<0;1>/*)"));
}
