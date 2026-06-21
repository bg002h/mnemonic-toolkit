//! cycle-4 convergence (toolkit 0.62.1) — characterization test for the
//! ms-codec 0.5.0 cross-share polynomial-consistency check (M6) as it surfaces
//! through `mnemonic ms-shares combine` after the pin bump.
//!
//! Background (BRAINSTORM/PLAN `*_cycle4_codec_funds_fixes.md` §6): codex32
//! K-of-N (BIP-93) carries NO digest share, so a same-id-but-inconsistent set
//! (shares from DIFFERENT splits that happen to share hrp/id/threshold/length)
//! previously combined to a SILENT WRONG secret. ms-codec 0.5.0 added a
//! membership check: interpolate the secret from the first k shares, then verify
//! every EXTRA supplied share lies on that polynomial; any mismatch →
//! `Error::InconsistentShareSet`.
//!
//! Toolkit lockstep (SILENT — no compiler catch): the pin bump adds
//!   - `ms_codec_exit_code`: explicit `InconsistentShareSet => 2` arm (funds /
//!     format-violation class) — without it the `_ => 1` wildcard would route to
//!     exit 1.
//!   - `friendly_ms_codec`: explicit prose arm ("inconsistent share set …").
//!
//! Irreducible limit (documented, not tested as detectable): an EXACTLY-k mixed
//! pair is NOT detectable — any k points define *a* polynomial. M6 closes only
//! the detectable case (any over-threshold set not all-on-one-curve). Hence the
//! reject fixture is an n>k set `[A1, A2, B3]`.
//!
//! Provenance precision: this specific `[A1, A2, B3]` fixture also happens to be
//! rejected by the OLD ms-codec 0.4.4 — but for an INCIDENTAL reason (a
//! reserved-prefix-byte mismatch on the wrongly-interpolated secret), NOT the M6
//! membership check, and with a different message/exit reason. The general
//! silent-wrong-secret class (combine returning `Ok(wrong)` at exit 0) is real
//! and was EMPIRICALLY reproduced at the codec level via mutation testing in
//! `design/agent-reports/cycle4-trackB-whole-diff-review.md`. What THIS toolkit
//! test pins is the new ACCURATE surfacing — `InconsistentShareSet` → exit 2
//! with the "inconsistent share set" prose — which the old codec did NOT emit
//! (verified non-vacuous against an old-codec binary in the convergence review).
//!
//! Fixtures: deterministic same-id 2-of-3 share sets for two DIFFERENT secrets
//! A (16×0x11) and B (16×0x33), built with a fixed id "aaaa" so the headers
//! match but the polynomials differ. Generated once via codex32's public
//! `from_seed`/`interpolate_at` (mirroring ms-codec's `same_id_2_of_n` test
//! helper) and baked here as literals (the construction is CSPRNG-free →
//! reproducible). Verified against the built binary at write time.

use assert_cmd::Command;

// Secret A (16 bytes of 0x11), 2-of-3, id "aaaa".
const A1: &str = "ms12aaaaqyg3zyg3zyg3zyg3zyg3zyg3zyg3qpzwna5afu3ksg";
const A2: &str = "ms12aaaapwujstkjstkjstkjstkjstkjstkjpzxt8d3addtd3q";
const A3: &str = "ms12aaaazsfh06ah06ah06ah06ah06ah06ahz82yj57aphvfjc";
// Secret B (16 bytes of 0x33), 2-of-3, SAME id "aaaa" → same header, different
// polynomial. B3 is the over-threshold share that fails A's membership check.
const B3: &str = "ms12aaaazfkwahrwahrwahrwahrwahrwahrwzq2mek7vawaukk";

// Secret A is 16 bytes of 0x11 → entropy hex is "11" × 16.
const SECRET_A_ENTROPY_HEX: &str = "11111111111111111111111111111111";

fn combine(args: &[&str]) -> (String, String, i32) {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["ms-shares", "combine"])
        .args(args)
        .output()
        .unwrap();
    (
        String::from_utf8(out.stdout).unwrap(),
        String::from_utf8(out.stderr).unwrap(),
        out.status.code().unwrap_or(-1),
    )
}

/// The funds-safety RED→GREEN: an n>k same-id set `[A1, A2, B3]` (matching
/// headers, B3 off A's polynomial) → exit 2 with the inconsistent-share-set
/// prose. Pre-0.5.0 this silently recovered a WRONG secret at exit 0.
#[test]
fn combine_inconsistent_same_id_set_exits_2_with_prose() {
    let (stdout, stderr, exit) = combine(&[
        "--share", A1, "--share", A2, "--share", B3, "--to", "entropy",
    ]);
    assert_eq!(exit, 2, "inconsistent set must exit 2; stderr: {stderr}");
    assert!(
        stderr.contains("inconsistent share set"),
        "expected inconsistent-share-set prose; got: {stderr}"
    );
    // Funds-safety: no secret leaked to stdout on the reject path.
    assert!(
        !stdout.contains(SECRET_A_ENTROPY_HEX),
        "rejected combine must not emit any secret on stdout; got: {stdout}"
    );
}

/// Positive control (hard invariant §6.0): a valid EXACTLY-k consistent set
/// `[A1, A2]` still recovers the correct secret A, exit 0 — no regression from
/// the truncate-to-k + membership-check rewrite.
#[test]
fn combine_valid_exactly_k_still_recovers_secret() {
    let (stdout, stderr, exit) = combine(&["--share", A1, "--share", A2, "--to", "entropy"]);
    assert_eq!(
        exit, 0,
        "valid exactly-k combine must exit 0; stderr: {stderr}"
    );
    assert!(
        stdout.contains(SECRET_A_ENTROPY_HEX),
        "valid combine must recover secret A ({SECRET_A_ENTROPY_HEX}); got: {stdout}"
    );
}

/// Positive control: a valid n>k ALL-consistent set `[A1, A2, A3]` recovers the
/// same secret A at exit 0 — the extra share passes the membership check, so the
/// over-supplied legitimate case is not regressed.
#[test]
fn combine_valid_n_gt_k_all_consistent_still_recovers_secret() {
    let (stdout, stderr, exit) = combine(&[
        "--share", A1, "--share", A2, "--share", A3, "--to", "entropy",
    ]);
    assert_eq!(
        exit, 0,
        "valid n>k all-consistent combine must exit 0; stderr: {stderr}"
    );
    assert!(
        stdout.contains(SECRET_A_ENTROPY_HEX),
        "valid n>k combine must recover secret A; got: {stdout}"
    );
}
