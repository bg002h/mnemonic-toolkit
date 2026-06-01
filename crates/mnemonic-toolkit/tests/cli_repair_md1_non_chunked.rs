//! Smoke test: `mnemonic repair --md1 <non-chunked-md1>` on a single-string
//! (non-chunked) md1 exits 5 (correction applied) and recovers the original.
//!
//! Background: On md-codec 0.34.0 this path was broken — a non-chunked md1
//! (the form emitted by plain `md encode` for small payloads) would exit 2
//! with `wire-format version mismatch: got 2, expected 4`.  md-codec 0.35.0
//! added a non-chunked-form pre-pass in `decode_with_correction` that routes
//! `strings.len() == 1` inputs whose header sentinel indicates non-chunked
//! directly into `decode_payload`, fixing the path.  The toolkit's
//! `repair_via_md_codec` delegation in `repair.rs` consumes the broadened API
//! transparently once the dep is bumped to 0.35.
//!
//! Fixture: `md1yqpqqxqq8xtwhw4xwn4qh` — the canonical v0.30 single-string
//! md1 produced by `md encode "wpkh(@0/<0;1>/*)"`; a non-chunked form.
//!
//! FOLLOWUP corrected: `md-codec-decode-with-correction-supports-non-chunked-md1`
//! was marked `resolved v0.24.0 cycle` prematurely; the required 0.34→0.35 pin
//! bump was never applied until the output-class-advisory Phase-2 / Tier-0 fold.
//! This test is the regression guard filed at that correction.

use assert_cmd::Command;
use predicates::prelude::*;

/// The canonical non-chunked md1 fixture (`md encode "wpkh(@0/<0;1>/*)"`).
const VALID_NON_CHUNKED_MD1: &str = "md1yqpqqxqq8xtwhw4xwn4qh";

/// Helper: flip the bech32 char at `pos` (0-indexed within the data portion
/// after the `1` separator) to the next char in the bech32 alphabet (cyclic).
/// Mirrors the identical helper in `cli_repair.rs` and `cli_auto_repair.rs`.
fn flip_at(s: &str, pos: usize) -> String {
    const ALPHABET: &str = "qpzry9x8gf2tvdw0s3jn54khce6mua7l";
    let sep = s.rfind('1').unwrap();
    let (prefix, rest) = s.split_at(sep + 1);
    let mut chars: Vec<char> = rest.chars().collect();
    let was = chars[pos];
    let was_idx = ALPHABET.find(was).unwrap();
    let new_idx = (was_idx + 1) % 32;
    chars[pos] = ALPHABET.chars().nth(new_idx).unwrap();
    let mut out = String::from(prefix);
    for c in chars {
        out.push(c);
    }
    out
}

/// Tier-0 smoke test: non-chunked md1 with one error → `mnemonic repair --md1`
/// exits 5 (correction applied) and recovers the original non-chunked string.
///
/// On md-codec 0.34.0 this exits 2 (`wire-format version mismatch: got 2,
/// expected 4`).  On md-codec 0.35.0 it exits 5 and recovers correctly.
#[test]
fn non_chunked_md1_single_error_repair_exits_5_and_recovers() {
    // Flip position 3 in the data portion ('q' → 'p').
    // Corrupted: md1yqppqxqq8xtwhw4xwn4qh  (position 3: q→p)
    let bad = flip_at(VALID_NON_CHUNKED_MD1, 3);
    assert_eq!(
        bad, "md1yqppqxqq8xtwhw4xwn4qh",
        "sanity-check: corrupted fixture must match expected value"
    );

    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["repair", "--md1", &bad])
        .assert()
        .code(5)
        .stdout(predicate::str::contains("# Repair report"))
        .stdout(predicate::str::contains(VALID_NON_CHUNKED_MD1));
}
