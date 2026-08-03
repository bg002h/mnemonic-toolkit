//! v0.97.0 — `verify-bundle` descriptor-INPUT errors exit 2, not 4.
//!
//! SPEC: `design/SPEC_verify_bundle_descriptor_exit_tier.md` (R0 GREEN round 3).
//!
//! Exit 4 is this project's BundleMismatch / VERIFY-ME tier — "a result with no
//! self-oracle you must confirm out-of-band". A descriptor that fails to parse
//! produced no result at all: nothing was compared against any card. Before
//! v0.97.0 a typo on `--descriptor` was reported in that tier, and the GUI
//! rendered an amber VERIFY-ME badge for it.
//!
//! One cell per REACHABLE flipped site (SPEC §6). The site-per-cell rule exists
//! because R0 round 1 found that pinning only two of them would let a
//! half-applied fix ship with the whole suite green.
//!
//! Exit 4 still means, on this path: cards mismatched, `result: partial`
//! (v0.88.0 dead-card), or `Bip388VerifyDistinctness`. Those are NOT touched —
//! `no_collateral_*` below pins that.

use assert_cmd::Command;

const P0: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const P1: &str = "legal winner thank year wave sausage worth useful legal winner thank yellow";

fn bin() -> Command {
    Command::cargo_bin("mnemonic").unwrap()
}

/// Empty card sentinels satisfy clap's required `--mk1`/`--md1` without
/// affecting the outcome: every cell here rejects during descriptor intake,
/// before any card is consulted.
fn verify(desc: &str, extra: &[&str]) -> std::process::Output {
    let mut c = bin();
    c.args([
        "verify-bundle",
        "--network",
        "mainnet",
        "--mk1",
        "",
        "--md1",
        "",
        "--descriptor",
        desc,
    ]);
    c.args(extra);
    c.output().unwrap()
}

fn emit(desc: &str, extra: &[&str]) -> std::process::Output {
    let mut c = bin();
    c.args(["bundle", "--network", "mainnet", "--descriptor", desc]);
    c.args(extra);
    c.output().unwrap()
}

/// T1 — site `:1445` (`lex_placeholders`). Hardened multipath alternative.
#[test]
fn t1_lex_reject_exits_2() {
    let out = verify(
        "wsh(multi(2,@0/<0h;1h>/*,@1/<0;1>/*))",
        &[
            "--slot",
            &format!("@0.phrase={P0}"),
            "--slot",
            &format!("@1.phrase={P1}"),
        ],
    );
    assert_eq!(
        out.status.code(),
        Some(2),
        "lex reject must be input-tier (exit 2)"
    );
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("multipath"), "stderr: {err}");
    assert!(
        !err.contains("re-parse failed"),
        "wrapper must be gone: {err}"
    );
}

/// T2 — site `:1449` (`resolve_placeholders`). Non-dense placeholder set.
#[test]
fn t2_resolve_gap_exits_2() {
    let out = verify(
        "wsh(multi(2,@0/<0;1>/*,@2/<0;1>/*))",
        &[
            "--slot",
            &format!("@0.phrase={P0}"),
            "--slot",
            &format!("@1.phrase={P1}"),
        ],
    );
    assert_eq!(out.status.code(), Some(2), "resolve gap must be input-tier");
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("dense") || err.contains("not present"),
        "stderr: {err}"
    );
}

/// T3 — site `:1468` (canonicity probe). Lex+resolve pass; the probe fails.
#[test]
fn t3_probe_failure_exits_2() {
    let out = verify(
        "wsh(pk(@0/<0;1>/*),pk(@1/<0;1>/*))",
        &[
            "--slot",
            &format!("@0.phrase={P0}"),
            "--slot",
            &format!("@1.phrase={P1}"),
        ],
    );
    assert_eq!(
        out.status.code(),
        Some(2),
        "probe failure must be input-tier"
    );
}

/// T4 — site `:1678` (unsupported `--slot` subkey). Historically a live drift
/// site: the Entropy arm exists because raw entropy used to fall in here.
#[test]
fn t4_unsupported_slot_subkey_exits_2() {
    let out = verify(
        "wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*))",
        &[
            "--slot",
            "@0.wif=L1aW4aubDFB7yfras2S1mN3bqg9nwySY8nkoLmJebSLD5BWv3ENZ",
            "--slot",
            &format!("@1.phrase={P1}"),
        ],
    );
    assert_eq!(
        out.status.code(),
        Some(2),
        "slot-usage error must be input-tier"
    );
}

/// T5 — site `:1371` (`--descriptor-file` read). Parity with emit.
#[test]
fn t5_missing_descriptor_file_exits_2_on_both_surfaces() {
    let missing = "/nonexistent-descriptor-file-for-exit-tier-test.txt";
    let v = bin()
        .args([
            "verify-bundle",
            "--network",
            "mainnet",
            "--mk1",
            "",
            "--md1",
            "",
            "--descriptor-file",
            missing,
        ])
        .output()
        .unwrap();
    assert_eq!(
        v.status.code(),
        Some(2),
        "verify: missing file must be input-tier"
    );
    let e = bin()
        .args([
            "bundle",
            "--network",
            "mainnet",
            "--descriptor-file",
            missing,
        ])
        .output()
        .unwrap();
    assert_eq!(
        e.status.code(),
        Some(2),
        "emit: missing file is already input-tier"
    );
}

/// T6 — the asymmetry that triggered this cycle: identical malformed input,
/// both surfaces, same tier.
#[test]
fn t6_emit_and_verify_agree_on_malformed_descriptor() {
    let desc = "wsh(multi(2,@0/<0h;1h>/*,@1/<0;1>/*))";
    let slots = ["--slot", "@0.phrase=x", "--slot", "@1.phrase=y"];
    let v = verify(desc, &slots);
    let e = emit(desc, &slots);
    assert_eq!(
        (v.status.code(), e.status.code()),
        (Some(2), Some(2)),
        "emit and verify must agree on a malformed descriptor"
    );
}

/// T7 — no collateral: the genuine result-tier stays at exit 4. `--template`
/// with a wrong-passphrase / mismatching card set is the canonical exit-4 case;
/// this cell pins that the tier still exists after the flip.
#[test]
fn t7_no_collateral_result_tier_still_exits_4() {
    // A well-formed descriptor whose cards are empty sentinels => the run gets
    // past intake and fails as a RESULT, not as input.
    let out = verify(
        "wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*))",
        &[
            "--slot",
            &format!("@0.phrase={P0}"),
            "--slot",
            &format!("@1.phrase={P1}"),
        ],
    );
    assert_eq!(
        out.status.code(),
        Some(4),
        "a well-formed descriptor must still reach the RESULT tier, not be re-tiered to input-error"
    );
}

/// T7b — the third preserved exit-4 meaning, which v0.97.0 newly ADVERTISES in
/// the CHANGELOG and manual: `Bip388VerifyDistinctness`.
///
/// SPEC §6 T7 mandated this cell precisely because advertising a contract
/// without pinning it is the shape this cycle exists to close. It is NOT
/// covered elsewhere at the CLI level: `error.rs`'s unit test pins only the
/// enum→4 mapping and would stay green if these verify sites were re-tiered,
/// and `cli_bip388_distinctness.rs` defers verify coverage to a phase that
/// never landed.
#[test]
fn t7b_bip388_distinctness_still_exits_4() {
    // Same phrase in both slots => duplicate (xpub, path) => distinctness fails
    // on the verify surface, pre-card, at exit 4 (a deliberate v0.19.0 §4.11.c
    // per-surface split that this cycle explicitly does NOT touch).
    let out = verify(
        "wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*))",
        &[
            "--slot",
            &format!("@0.phrase={P0}"),
            "--slot",
            &format!("@1.phrase={P0}"),
        ],
    );
    assert_eq!(
        out.status.code(),
        Some(4),
        "BIP-388 distinctness must stay in the RESULT tier (exit 4)"
    );
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("BIP-388 distinct-key rule"), "stderr: {err}");
}
