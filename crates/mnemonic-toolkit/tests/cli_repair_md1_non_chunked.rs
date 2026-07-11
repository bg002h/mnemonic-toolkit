//! `mnemonic repair --md1 <non-chunked-md1>` CLI-level coverage.
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
//!
//! **v0.86.0 (`toolkit-v0860-demote`, funds-adjacent) UPDATE:** a TOUCHED
//! correction of a non-chunked single-string md1 no longer exits 5
//! (confident) — it has no cross-chunk/content-id oracle (the v0.35.0
//! bypass skips `reassemble`'s content-id check entirely for this shape),
//! so it is now demoted to an exit-4 VERIFY-ME candidate, matching the
//! ms1-substitution-correction precedent (Cycle F). A CHUNKED-of-1 md1
//! (the `mnemonic bundle` / `--md1-form=template` shape) retains the
//! oracle and still stays exit 5 — see the boundary test below.

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
/// now demotes to exit 4 (VERIFY-ME candidate; v0.86.0) but still recovers
/// the original non-chunked string on stdout.
///
/// On md-codec 0.34.0 this exited 2 (`wire-format version mismatch: got 2,
/// expected 4`). On md-codec 0.35.0 (through toolkit v0.85.0) it exited 5.
/// From toolkit v0.86.0 it exits 4 (see module doc).
#[test]
fn non_chunked_md1_single_error_repair_exits_4_and_recovers() {
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
        .code(4)
        .stdout(predicate::str::contains("# Repair report"))
        .stdout(predicate::str::contains(VALID_NON_CHUNKED_MD1));
}

/// Positive demote test (v0.86.0, `toolkit-v0860-demote` acceptance #1a):
/// a touched non-chunked md1 correction reports the `Unverified` reason on
/// stderr AND (via `--json`) `verdict: "candidate"` (`cmd/repair.rs`'s
/// `verdict_str` — `SetVerify::Unverified` maps to `"candidate"`).
#[test]
fn non_chunked_md1_demote_reason_and_json_verdict_candidate() {
    let bad = flip_at(VALID_NON_CHUNKED_MD1, 3);

    // Text-form: exit 4 + stdout report + stderr reason mentioning the
    // non-chunked/no-oracle rationale (the demote's OWN reason string, NOT
    // the mk1 reassembly-incomplete reason).
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["repair", "--md1", &bad])
        .assert()
        .code(4)
        .stdout(predicate::str::contains("# Repair report"))
        .stderr(predicate::str::contains("UNVERIFIED"))
        .stderr(predicate::str::contains("non-chunked single-string md1"))
        .stderr(predicate::str::contains("cross-chunk/content-id oracle"));

    // JSON-form: same exit code, `verdict: "candidate"`.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["repair", "--md1", &bad, "--json"])
        .assert()
        .code(4)
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON envelope");
    assert_eq!(v["kind"], "md1");
    assert_eq!(v["verdict"], "candidate");
    assert_eq!(v["corrected_chunks"][0], VALID_NON_CHUNKED_MD1);
}

/// chunked-of-1 BOUNDARY test (v0.86.0 acceptance #1b — the oracle
/// boundary): the `mnemonic bundle` / `--md1-form=template` shape
/// (chunked-flag bit == 1, count == 1) retains the cross-chunk/content-id
/// oracle (it falls through to `reassemble`, not the non-chunked bypass),
/// so a touched correction MUST stay exit 5 — demoting on `count == 1`
/// alone (rather than reading the chunked-flag bit) would have wrongly
/// caught this shape too. Constructed via `md_codec::chunk::split` (the
/// SAME function `mnemonic bundle` uses to emit md1 cards) on the fixture
/// descriptor decoded from `VALID_NON_CHUNKED_MD1`.
#[test]
fn chunked_of_1_md1_single_error_repair_stays_exit_5() {
    let descriptor = md_codec::decode_md1_string(VALID_NON_CHUNKED_MD1)
        .expect("decode non-chunked fixture into a Descriptor");
    let chunked =
        md_codec::chunk::split(&descriptor).expect("split the fixture descriptor into chunks");
    assert_eq!(
        chunked.len(),
        1,
        "fixture descriptor must be small enough to split to exactly 1 chunk (chunked-of-1)"
    );
    let original = &chunked[0];
    let bad = flip_at(original, 3);
    assert_ne!(
        &bad, original,
        "sanity: corruption must actually change the string"
    );

    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["repair", "--md1", &bad])
        .assert()
        .code(5)
        .stdout(predicate::str::contains("# Repair report"))
        .stdout(predicate::str::contains(original.as_str()));
}
