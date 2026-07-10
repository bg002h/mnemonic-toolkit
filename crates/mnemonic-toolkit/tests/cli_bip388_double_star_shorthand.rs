//! Cycle C — SPEC `bip388-double-star-shorthand-support`.
//!
//! Accept the BIP-388 `/**` combined-wildcard shorthand on descriptor intake
//! by expanding a final-use-site `/**` → `/<0;1>/*` BEFORE the parser,
//! instead of hard-rejecting it (`/**` ≡ `/<0;1>/*`, receive=chain0,
//! change=chain1). Covers the surfaces NOT already exercised by the
//! repurposed cells in `parse_descriptor.rs` (§7.1) and
//! `cli_import_wallet_descriptor.rs` (§7.2).
//!
//! Funds property (SPEC §6): an expanded `/**` MUST produce output
//! BYTE-IDENTICAL to the explicit `/<0;1>/*` spelling on every surface, both
//! for successful outputs AND error/exit behavior (compare-cost).

use assert_cmd::Command;
use serde_json::Value;

fn bin() -> Command {
    Command::cargo_bin("mnemonic").expect("binary built")
}

/// Two mainnet account xpubs, reused across the descriptor test corpus
/// (`cli_import_wallet_descriptor.rs`, `cli_bip388_policy_intake.rs`).
const A: &str = "xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX";
const B: &str = "xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6";
const FP_A: &str = "704c7836";
const FP_B: &str = "97139860";

/// Run `bundle --descriptor <desc> --network mainnet --json`; return the
/// parsed JSON with the raw-echo `descriptor` field nulled out (SPEC-known
/// pass-through, not part of the funds property — the field verbatim-echoes
/// whatever the caller supplied, so `/**` vs `/<0;1>/*` legitimately differ
/// there and nowhere else).
fn bundle_json_descriptor(desc: &str) -> Value {
    let out = bin()
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
    let mut v: Value =
        serde_json::from_slice(&out.get_output().stdout).expect("bundle --json output");
    v["descriptor"] = Value::Null;
    v
}

/// Same as `bundle_json_descriptor` but with explicit `--slot` cosigner
/// material (AtN-form descriptors carry no inline keys).
fn bundle_json_descriptor_with_slots(desc: &str, slots: &[&str]) -> Value {
    let mut args: Vec<&str> = vec!["bundle", "--descriptor", desc, "--network", "mainnet"];
    for s in slots {
        args.push("--slot");
        args.push(s);
    }
    args.push("--json");
    let out = bin().args(&args).assert().success();
    let mut v: Value =
        serde_json::from_slice(&out.get_output().stdout).expect("bundle --json output");
    v["descriptor"] = Value::Null;
    v
}

// ── §7.3 — concrete-xpub equivalence oracle (funds anchor) ─────────────────

#[test]
fn bundle_concrete_wpkh_double_star_equals_explicit_multipath() {
    let shorthand = format!("wpkh([{FP_A}/84'/0'/0']{A}/**)");
    let explicit = format!("wpkh([{FP_A}/84'/0'/0']{A}/<0;1>/*)");
    assert_eq!(
        bundle_json_descriptor(&shorthand),
        bundle_json_descriptor(&explicit),
        "singlesig wpkh: `/**` bundle must equal the explicit `/<0;1>/*` bundle"
    );
}

#[test]
fn bundle_concrete_tr_double_star_equals_explicit_multipath() {
    let shorthand = format!("tr([{FP_A}/86'/0'/0']{A}/**)");
    let explicit = format!("tr([{FP_A}/86'/0'/0']{A}/<0;1>/*)");
    assert_eq!(
        bundle_json_descriptor(&shorthand),
        bundle_json_descriptor(&explicit),
        "taproot tr: `/**` bundle must equal the explicit `/<0;1>/*` bundle"
    );
}

#[test]
fn bundle_concrete_sortedmulti_double_star_expands_both_keys_equals_explicit() {
    // Multisig: BOTH `/**` occurrences must expand (per-key, not "last-in-
    // string") — this is the multi-key backstop for the expander's
    // terminator-anchored precision (SPEC §5).
    let shorthand =
        format!("wsh(sortedmulti(2,[{FP_A}/48'/0'/0'/2']{A}/**,[{FP_B}/48'/0'/0'/2']{B}/**))");
    let explicit = format!(
        "wsh(sortedmulti(2,[{FP_A}/48'/0'/0'/2']{A}/<0;1>/*,[{FP_B}/48'/0'/0'/2']{B}/<0;1>/*))"
    );
    assert_eq!(
        bundle_json_descriptor(&shorthand),
        bundle_json_descriptor(&explicit),
        "2-of-2 sortedmulti: `/**` bundle must equal the explicit `/<0;1>/*` bundle \
         (both cosigners' `/**` must expand)"
    );
}

// ── §7.4 — AtN-form oracle ──────────────────────────────────────────────────

#[test]
fn bundle_atn_form_multisig_double_star_equals_explicit_multipath() {
    let slots = [
        format!("@0.xpub={A}"),
        format!("@0.fingerprint={FP_A}"),
        format!("@1.xpub={B}"),
        format!("@1.fingerprint={FP_B}"),
    ];
    let slot_refs: Vec<&str> = slots.iter().map(String::as_str).collect();
    let shorthand =
        bundle_json_descriptor_with_slots("wsh(sortedmulti(2,@0/**,@1/**))", &slot_refs);
    let explicit =
        bundle_json_descriptor_with_slots("wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*))", &slot_refs);
    assert_eq!(
        shorthand, explicit,
        "AtN-form `@N/**` bundle must equal the AtN-form `@N/<0;1>/*` bundle"
    );
}

#[test]
fn verify_bundle_atn_form_double_star_accepts() {
    let slots = [
        format!("@0.xpub={A}"),
        format!("@0.fingerprint={FP_A}"),
        format!("@1.xpub={B}"),
        format!("@1.fingerprint={FP_B}"),
    ];
    let descriptor = "wsh(sortedmulti(2,@0/**,@1/**))";
    let mut bundle_args: Vec<&str> =
        vec!["bundle", "--descriptor", descriptor, "--network", "mainnet"];
    for s in &slots {
        bundle_args.push("--slot");
        bundle_args.push(s);
    }
    bundle_args.push("--json");
    let bundle_out = bin().args(&bundle_args).assert().success();
    let stdout = String::from_utf8(bundle_out.get_output().stdout.clone()).unwrap();

    let tmpdir = tempfile::tempdir().unwrap();
    let path = tmpdir.path().join("bundle.json");
    std::fs::write(&path, &stdout).unwrap();

    let mut verify_args: Vec<&str> = vec![
        "verify-bundle",
        "--descriptor",
        descriptor,
        "--network",
        "mainnet",
    ];
    for s in &slots {
        verify_args.push("--slot");
        verify_args.push(s);
    }
    let path_str = path.to_str().unwrap();
    verify_args.push("--bundle-json");
    verify_args.push(path_str);

    let v = bin().args(&verify_args).assert().success();
    let vo = v.get_output();
    let report = format!(
        "{}{}",
        String::from_utf8_lossy(&vo.stdout),
        String::from_utf8_lossy(&vo.stderr)
    );
    assert!(
        report.contains("result: ok"),
        "AtN-form `@N/**` verify-bundle must be ok, got:\n{report}"
    );
}

#[test]
fn verify_bundle_concrete_double_star_accepts_against_explicit_reference_bundle() {
    // Cross-spelling oracle: bundle the EXPLICIT `/<0;1>/*` form as the
    // reference (pre-existing, not expander-derived — non-tautological),
    // then verify-bundle it against a `/**`-spelled --descriptor. Both
    // spellings must describe the SAME wallet.
    let explicit = format!("wpkh([{FP_A}/84'/0'/0']{A}/<0;1>/*)");
    let shorthand = format!("wpkh([{FP_A}/84'/0'/0']{A}/**)");

    let bundle_out = bin()
        .args([
            "bundle",
            "--descriptor",
            &explicit,
            "--network",
            "mainnet",
            "--json",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(bundle_out.get_output().stdout.clone()).unwrap();
    let tmpdir = tempfile::tempdir().unwrap();
    let path = tmpdir.path().join("bundle.json");
    std::fs::write(&path, &stdout).unwrap();

    let v = bin()
        .args([
            "verify-bundle",
            "--descriptor",
            &shorthand,
            "--network",
            "mainnet",
            "--bundle-json",
            path.to_str().unwrap(),
        ])
        .assert()
        .success();
    let vo = v.get_output();
    let report = format!(
        "{}{}",
        String::from_utf8_lossy(&vo.stdout),
        String::from_utf8_lossy(&vo.stderr)
    );
    assert!(
        report.contains("result: ok"),
        "verify-bundle --descriptor `/**` against an explicit-`/<0;1>/*`-sourced \
         bundle must be ok, got:\n{report}"
    );
}

// ── §7.5 — xpub-search literal-xpub `/**` parses ────────────────────────────

#[test]
fn xpub_search_account_of_descriptor_double_star_matches() {
    use bip39::Mnemonic;
    use bitcoin::bip32::{DerivationPath, Xpriv, Xpub};
    use bitcoin::secp256k1::Secp256k1;
    use std::str::FromStr;

    const PHRASE: &str = "abandon abandon abandon abandon abandon abandon abandon abandon \
         abandon abandon abandon about";

    let mnemonic = Mnemonic::parse_in(bip39::Language::English, PHRASE).unwrap();
    let seed = mnemonic.to_seed("");
    let secp = Secp256k1::new();
    let master = Xpriv::new_master(bitcoin::NetworkKind::Main, &seed).unwrap();
    let fp = master.fingerprint(&secp);
    let dp = DerivationPath::from_str("m/84'/0'/0'").unwrap();
    let xpriv = master.derive_priv(&secp, &dp).unwrap();
    let xpub = Xpub::from_priv(&secp, &xpriv);

    let shorthand = format!("wpkh([{fp}/84'/0'/0']{xpub}/**)");
    let explicit = format!("wpkh([{fp}/84'/0'/0']{xpub}/<0;1>/*)");

    let run = |desc: &str| -> Value {
        let out = bin()
            .args([
                "xpub-search",
                "account-of-descriptor",
                "--phrase-stdin",
                "--descriptor",
                desc,
                "--json",
            ])
            .write_stdin(PHRASE)
            .assert()
            .success();
        serde_json::from_slice(&out.get_output().stdout).expect("valid JSON")
    };

    let v_shorthand = run(&shorthand);
    let v_explicit = run(&explicit);
    assert_eq!(v_shorthand["result"], "match", "{v_shorthand}");
    assert_eq!(
        v_shorthand, v_explicit,
        "xpub-search JSON for `/**` must equal the explicit `/<0;1>/*` JSON"
    );
}

// ── §7.6 — BSMS round-2 `/**` accepts + equals `/<0;1>/*` ───────────────────

#[test]
fn import_wallet_bsms_double_star_accepts_equals_explicit_multipath() {
    let shorthand = format!("BSMS 1.0\nwpkh([{FP_A}/84'/0'/0']{A}/**)\n");
    let explicit = format!("BSMS 1.0\nwpkh([{FP_A}/84'/0'/0']{A}/<0;1>/*)\n");

    let run = |blob: &str| -> Value {
        let out = bin()
            .args(["import-wallet", "--format", "bsms", "--blob", "-", "--json"])
            .write_stdin(blob.to_string())
            .assert()
            .success();
        let mut v: Value = serde_json::from_slice(&out.get_output().stdout).expect("valid JSON");
        // The bundle.descriptor field verbatim-echoes the checksum-refreshed
        // USER body (still `/**`-spelled) — SPEC-known pass-through, not part
        // of the funds property.
        v[0]["bundle"]["descriptor"] = Value::Null;
        v
    };

    let v_shorthand = run(&shorthand);
    let v_explicit = run(&explicit);
    assert_eq!(
        v_shorthand, v_explicit,
        "BSMS round-2 `/**` envelope must equal the explicit `/<0;1>/*` envelope \
         (modulo the raw-echo descriptor field)"
    );
    // §7.11 (folded in) — the roundtrip/canonicalize field must be CLEAN, not
    // the bogus "canonicalize_failed" a soft-failing raw `from_str` on `/**`
    // would previously have produced.
    let status = v_shorthand[0]["roundtrip"]["status"].as_str();
    assert_ne!(
        status,
        Some("canonicalize_failed"),
        "BSMS `--json` roundtrip must NOT report canonicalize_failed for `/**`: {v_shorthand}"
    );
}

// ── §7.8 — JSON `@N/**` (BIP-388 wallet-policy) regression: untouched ───────

#[test]
fn bip388_policy_json_at_n_double_star_still_works() {
    // The JSON-template `@N/**` path (`expand_bip388_policy`) is a DIFFERENT
    // code path from the literal-string expander added this cycle — this
    // cell pins that it is UNCHANGED (no double-expansion / regression).
    let policy = format!(
        r#"{{"name":"test-vault","description_template":"wsh(sortedmulti(2,@0/**,@1/**))","keys_info":["[{FP_A}/48'/0'/0'/2']{A}","[{FP_B}/48'/0'/0'/2']{B}"]}}"#
    );
    bin()
        .args([
            "bundle",
            "--descriptor",
            &policy,
            "--network",
            "mainnet",
            "--json",
        ])
        .assert()
        .success();
}

// ── §7.11 — export-wallet acceptance + compare-cost error-equivalence ──────

#[test]
fn export_wallet_double_star_equals_explicit_multipath() {
    let shorthand = format!("wpkh([{FP_A}/84'/0'/0']{A}/**)");
    let explicit = format!("wpkh([{FP_A}/84'/0'/0']{A}/<0;1>/*)");

    let run = |desc: &str| -> (Vec<u8>, Vec<u8>) {
        let out = bin()
            .args([
                "export-wallet",
                "--descriptor",
                desc,
                "--format",
                "descriptor",
            ])
            .assert()
            .success();
        let o = out.get_output();
        (o.stdout.clone(), o.stderr.clone())
    };

    let (stdout_shorthand, stderr_shorthand) = run(&shorthand);
    let (stdout_explicit, stderr_explicit) = run(&explicit);
    // export-wallet's stdout is a fresh miniscript re-render (`d.to_string()`),
    // NOT an echo of the user's literal text — so THIS surface's full stdout
    // is genuinely byte-identical between spellings (closes the
    // import-accepts/export-rejects asymmetry).
    assert_eq!(
        stdout_shorthand, stdout_explicit,
        "export-wallet stdout for `/**` must equal the explicit `/<0;1>/*` stdout"
    );
    assert_eq!(stderr_shorthand, stderr_explicit);
}

#[test]
fn compare_cost_double_star_wpkh_rejects_identically_as_unsupported_wrapper() {
    // Cycle G (`compare-cost-multipath-descriptor-unsupported`) UPDATE (NOT
    // invert — SPEC §2 I1): the split-first fix gets a multipath descriptor
    // PAST derivation, but compare-cost still rejects `wpkh` regardless of
    // multipath (unsupported wrapper — `strip.rs` only supports
    // `wsh`/`sh(wsh)`/single-leaf `tr`). So `/**` and `/<0;1>/*` now fail
    // IDENTICALLY with the NEW `UnsupportedWrapper` error, NOT the OLD
    // "multipath key cannot be a DerivedDescriptorKey" derivation error —
    // pinning that multipath now gets past derivation while wpkh stays
    // unsupported. The `/**` ≡ `/<0;1>/*` equivalence still holds.
    let shorthand = format!("wpkh([{FP_A}/84'/0'/0']{A}/**)");
    let explicit = format!("wpkh([{FP_A}/84'/0'/0']{A}/<0;1>/*)");

    let run = |desc: &str| -> (i32, Vec<u8>) {
        let out = bin()
            .args(["compare-cost", "--descriptor", desc])
            .assert()
            .failure();
        let o = out.get_output();
        (o.status.code().unwrap_or(-1), o.stderr.clone())
    };

    let (code_shorthand, stderr_shorthand) = run(&shorthand);
    let (code_explicit, stderr_explicit) = run(&explicit);
    assert_eq!(code_shorthand, code_explicit);
    assert_eq!(
        stderr_shorthand, stderr_explicit,
        "`/**` compare-cost error must be byte-identical to the explicit `/<0;1>/*` error"
    );
    let stderr_str = String::from_utf8_lossy(&stderr_shorthand);
    assert!(
        !stderr_str.contains("multipath key cannot be a DerivedDescriptorKey"),
        "multipath must now get PAST derivation (pins the split-first fix), got: {stderr_str}"
    );
    assert!(
        stderr_str.contains("unsupported wrapper"),
        "expected the wpkh UnsupportedWrapper rejection, got: {stderr_str}"
    );
    assert!(
        !stderr_str.contains("invalid child number format"),
        "must NOT be the pre-fix raw miniscript parse error (that would mean `/**` \
         is still unexpanded): {stderr_str}"
    );
}

/// Cycle G SPEC §2/§4.4 — a `wsh`-wrapped multipath descriptor is ACCEPTED
/// (split to the receive branch, index 0, before derivation — mirroring
/// `derive_address.rs`), and its cost is byte-identical to the single-path
/// `/0/*` equivalent. `/**` inherits acceptance for free (it pre-expands to
/// `/<0;1>/*` upstream, Cycle C) — all three spellings must cost identically.
#[test]
fn compare_cost_wsh_multipath_accepted_and_cost_equals_singlepath() {
    let fetch_conditions = |desc: &str| -> Value {
        let out = bin()
            .args(["compare-cost", "--descriptor", desc, "--json"])
            .assert()
            .success();
        let v: Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
        v["conditions"].clone()
    };

    let multipath = format!("wsh(pk([{FP_A}/84'/0'/0']{A}/<0;1>/*))");
    let doublestar = format!("wsh(pk([{FP_A}/84'/0'/0']{A}/**))");
    let singlepath = format!("wsh(pk([{FP_A}/84'/0'/0']{A}/0/*))");

    let conds_multipath = fetch_conditions(&multipath);
    let conds_doublestar = fetch_conditions(&doublestar);
    let conds_singlepath = fetch_conditions(&singlepath);

    assert_eq!(
        conds_multipath, conds_singlepath,
        "explicit `/<0;1>/*` multipath compare-cost must equal the single-path `/0/*` cost"
    );
    assert_eq!(
        conds_doublestar, conds_singlepath,
        "`/**` shorthand compare-cost must equal the single-path `/0/*` cost"
    );
}

/// Cycle G SPEC §2/§4.6 (M4) — malformed multipath (inconsistent branch
/// counts across keys within one `wsh(multi(...))`) errors cleanly, no
/// panic. rust-miniscript's descriptor-string PARSE step itself rejects a
/// mismatched multipath (`Error::MultipathDescLenMismatch`) BEFORE
/// `is_multipath()` / `into_single_descriptors()` are ever reached, so this
/// surfaces via the pre-existing `CompareCostError::Parse` path (exit 2).
#[test]
fn compare_cost_malformed_multipath_inconsistent_branch_counts_errors_cleanly() {
    let desc =
        format!("wsh(multi(2,[{FP_A}/84'/0'/0']{A}/<0;1>/*,[{FP_B}/84'/0'/0']{B}/<0;1;2>/*))");
    let out = bin()
        .args(["compare-cost", "--descriptor", &desc])
        .output()
        .unwrap();
    assert!(
        !out.status.success(),
        "a mismatched-branch-count multipath descriptor must be rejected, not silently \
         truncated to the shorter key's branch count"
    );
    assert_eq!(
        out.status.code(),
        Some(2),
        "expected the CompareCostError::Parse exit code; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

// ── §7.7 — precision guard (CLI-level end-to-end check) ─────────────────────

#[test]
fn bundle_triple_star_still_rejects_end_to_end() {
    // `/***` is NOT the BIP-388 shorthand (extra `*` breaks the terminator
    // anchor) — must still hard-reject through the real CLI surface, not
    // just at the unit level (`parse_descriptor::tests::
    // expand_literal_double_star_ignores_triple_star`).
    let desc = format!("wpkh([{FP_A}/84'/0'/0']{A}/***)");
    bin()
        .args([
            "bundle",
            "--descriptor",
            &desc,
            "--network",
            "mainnet",
            "--json",
        ])
        .assert()
        .failure();
}
