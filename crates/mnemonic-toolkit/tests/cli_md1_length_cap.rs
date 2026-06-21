//! cycle-4 convergence (toolkit 0.62.1) — characterization tests for the
//! md-codec 0.38.0 codex32 regular-code LENGTH CAPS (H6 / M4 / I1) as they
//! surface through the toolkit CLI after the pin bump.
//!
//! Background (BRAINSTORM/PLAN `*_cycle4_codec_funds_fixes.md`): the codex32
//! regular code is BCH(93,80,8) — 80 data symbols + 13 checksum = 93. md-codec
//! 0.38.0 added three fail-closed caps:
//!   - H6 (encode): `PayloadTooLongForSingleString` — over-80-data-symbol
//!     single-string encode rejected.
//!   - M4 (correcting decode): `ChunkSymbolCountOutOfRange` — over-93-symbol
//!     chunk rejected before BCH correction (degree aliasing, β has order 93).
//!   - I1 (non-correcting decode): `StringSymbolCountOutOfRange` — over-93-symbol
//!     single string rejected before the length-agnostic BCH verify.
//!
//! The toolkit pin-bump PATCH routes all three to **exit 2** in
//! `md_codec_exit_code` (an exhaustive match — the bump was compile-forced to
//! add the arms) and renders prose in `friendly_md_codec`.
//!
//! Toolkit-surface reality (verified empirically at write time, recorded so the
//! divergence from the codec-level exit code is intentional and not a bug):
//!   - `mnemonic inspect <over-93 md1>` → exit 2. This is the path where the I1
//!     `unwrap_string` cap surfaces THROUGH `md_codec_exit_code` (exit 2) — the
//!     genuine "non-correcting cap at the user-facing decode surface" the plan
//!     targets.
//!   - `mnemonic repair --md1 <over-93 md1>` → exit 2, but via the toolkit's OWN
//!     `repair.rs` length-band classifier (BIP-93 [94,95] reserved-invalid /
//!     [96,108] long-code-undefined / out-of-range), NOT the codec's M4 cap. The
//!     exit code matches the plan; the mechanism is repair-local (pre-existing).
//!   - `mnemonic restore --md1 <over-93 md1>` → exit 1 (NOT 2). The restore path
//!     wraps every md1-reassemble failure as `bad()` → `ToolkitError::BadInput`
//!     (`cmd/restore.rs` `--md1 decode: …`), a PRE-EXISTING design choice that
//!     predates cycle-4 and is identical on `origin/master`. The I1 cap MESSAGE
//!     still renders (the codec rejects), but restore down-classifies the exit
//!     code. Re-routing restore's md1-decode errors to exit 2 would change the
//!     exit code of EVERY md1 decode error on that path (out of scope for a
//!     pin-bump PATCH). The exit-2 surfacing of the cap is asserted via
//!     `inspect` (above); restore's exit-1 is asserted here as the documented
//!     current behavior so a future intentional change is a deliberate update.

use assert_cmd::Command;
use predicates::prelude::*;

fn bin() -> Command {
    Command::cargo_bin("mnemonic").unwrap()
}

/// An over-93-symbol md1 data part (94 valid bech32 chars). Over the 93-symbol
/// codex32 regular-code ceiling regardless of checksum validity — the length
/// cap fires before BCH verification on the non-correcting path.
fn over_93_md1(n: usize) -> String {
    format!("md1{}", "q".repeat(n))
}

// --- (b) the I1 non-correcting cap at its exit-2 surface: `inspect` -----------

/// `mnemonic inspect <over-93 md1>` → exit 2 with the `StringSymbolCountOutOfRange`
/// prose. This is the I1 cap surfacing through `md_codec_exit_code` (the plan's
/// "non-correcting cap on the restore/inspect path" — `inspect` is where it
/// reaches exit 2; restore down-classifies to exit 1, asserted separately).
#[test]
fn inspect_over_93_symbol_md1_exits_2_with_cap_prose() {
    bin()
        .args(["inspect", &over_93_md1(94)])
        .assert()
        .code(2)
        .stderr(predicates::str::contains(
            "the codex32 regular code caps a string at 93",
        ));
}

/// A longer over-93 string (100 symbols) also rejects at exit 2 — confirms the
/// cap is a true ceiling, not a single off-by-one boundary.
#[test]
fn inspect_well_over_93_symbol_md1_exits_2() {
    bin()
        .args(["inspect", &over_93_md1(100)])
        .assert()
        .code(2)
        .stderr(predicates::str::contains("100 symbols"));
}

/// Positive control: an in-domain (<= 93-symbol) md1 does NOT trip the length
/// cap. `VALID_MD1_CHUNK0` is a real 64-data-symbol md1 chunk (one of a 3-chunk
/// set); inspecting it alone fails with an UNRELATED "chunk set incomplete"
/// error — proving the symbol-count cap is not over-rejecting in-domain lengths.
#[test]
fn inspect_in_domain_md1_does_not_trip_length_cap() {
    const VALID_MD1_CHUNK0: &str =
        "md1fgdxlpqpqpm6jzzqqvqpdqw0za5zs4gyy55aq4vsmnhy4s6wyaypu34c7raqu8np";
    // 64 data symbols — well under the 93 ceiling.
    assert!(VALID_MD1_CHUNK0.len() - 3 <= 93);
    bin()
        .args(["inspect", VALID_MD1_CHUNK0])
        .assert()
        .stderr(predicates::str::contains("caps a string at 93").not());
}

// --- (a) `repair --md1 <over-93>` → exit 2 (toolkit length-band reject) --------

/// `mnemonic repair --md1 <94-symbol md1>` → exit 2. 94 falls in BIP-93's
/// reserved-invalid band [94, 95]; the toolkit's `repair.rs` length classifier
/// rejects it at exit 2 (independent of the codec's M4 cap, which sits behind
/// `decode_with_correction`).
#[test]
fn repair_md1_reserved_invalid_band_exits_2() {
    bin()
        .args(["repair", "--md1", &over_93_md1(94)])
        .assert()
        .code(2)
        .stderr(predicates::str::contains("reserved-invalid band"));
}

/// `mnemonic repair --md1 <100-symbol md1>` → exit 2. 100 is in the long-code
/// band [96, 108], undefined for HRP `md` in this codec — repair rejects at
/// exit 2.
#[test]
fn repair_md1_long_code_band_exits_2() {
    bin()
        .args(["repair", "--md1", &over_93_md1(100)])
        .assert()
        .code(2)
        .stderr(predicates::str::contains("long BCH code"));
}

// --- restore exit-1 down-classification (documented pre-existing behavior) -----

/// `mnemonic restore --md1 <over-93 md1>` → exit 1 (NOT 2). The I1 cap MESSAGE
/// renders (the codec rejects the over-length word) but the restore path wraps
/// the reassemble error as `bad()` → `BadInput` (exit 1) — a PRE-EXISTING design
/// choice (identical on `origin/master`). Asserted as current behavior so any
/// future re-routing to exit 2 is a deliberate, reviewed change.
#[test]
fn restore_over_93_symbol_md1_exits_1_pre_existing_bad_input() {
    bin()
        .args(["restore", "--md1", &over_93_md1(94)])
        .assert()
        .code(1)
        .stderr(predicates::str::contains(
            "the codex32 regular code caps a string at 93",
        ));
}
