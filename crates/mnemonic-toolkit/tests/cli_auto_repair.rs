//! Integration tests for v0.22.0 auto-fire short-circuit (Phase 5 scope).
//!
//! Per `design/IMPLEMENTATION_PLAN_repair_v0_22.md` §4.4 reduced-scope:
//!   - convert (--from ms1=…) auto-fire — cell 19
//!   - convert (--from mk1=…) auto-fire — cell 20
//!   - inspect (--ms1) auto-fire (cell 18 from §4.3, owned here now)
//!   - `--no-auto-repair` suppresses both convert and inspect auto-fire — cell 22
//!   - bundle --self-check NOT auto-firing — cell 23 (per D16)
//!
//! verify-bundle auto-fire (cell 21 in original plan) is DEFERRED to v0.22.1
//! per the FOLLOWUP `verify-bundle-auto-fire-helper-refactor` (helper signature
//! cascade through 10 callers is high-risk for the v0.22.0 single-shot tag).
//!
//! Fixtures: same `abandon × 11 about` toolkit-emitted bundle as other v0.22
//! cells.

use assert_cmd::Command;
use predicates::prelude::*;

const VALID_MS1: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
const VALID_MK1_CHUNK0: &str = "mk1qprsqhpqqsq3cqtsleeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8qp3yxg3dpe854wq4";
const VALID_MK1_CHUNK1: &str = "mk1qprsqhpp0f30mtxzd65mvwcur9usdatwuqvq6z70r9nwrgk6xn6l8gy6nwa2n977sw6zh34rma0nh";
const EXPECTED_XPUB: &str = "xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V";

const VALID_MD1_CHUNK0: &str = "md1fgdxlpqpqpm6jzzqqvqpdqw0za5zs4gyy55aq4vsmnhy4s6wyaypu34c7raqu8np";
const VALID_MD1_CHUNK1: &str = "md1fgdxlpqf2zcgefcpupmel75q5435j7seugaj5jr7qyur6vt76es5cdeyrq7zdy0d";
const VALID_MD1_CHUNK2: &str = "md1fgdxlpq3xa2dk8vwpj7gx74hwqxqdp083jehp5tdrfa0n5zdfkqcdlrvnh5r62jn";

fn flip_at(chunk: &str, pos: usize) -> String {
    const ALPHABET: &str = "qpzry9x8gf2tvdw0s3jn54khce6mua7l";
    let sep = chunk.rfind('1').unwrap();
    let (prefix, rest) = chunk.split_at(sep + 1);
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

/// Cell 19: convert --from ms1=<1-error> --to phrase → exit 5 + repair
/// report on stdout + corrected ms1 emitted.
#[test]
fn cell_19_convert_auto_fire_ms1_one_substitution() {
    let bad = flip_at(VALID_MS1, 17);
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["convert", "--from", &format!("ms1={bad}"), "--to", "phrase"])
        .assert()
        .code(5)
        .stdout(predicate::str::contains("# Repair report"))
        .stdout(predicate::str::contains(
            "ms1 chunk 0: 1 correction at position 17",
        ))
        .stdout(predicate::str::contains(VALID_MS1))
        .stderr(predicate::str::contains(
            "repair: applied 1 correction across 1 chunk",
        ));
}

/// Cell 20a: layering note for mk1 auto-fire.
///
/// `mk-codec` ALREADY does internal BCH correction at the same t=4 capacity
/// as the toolkit's repair primitive (per `mk_codec::Error::BchUncorrectable`
/// being the explicit "beyond-capacity" variant). A 1-char corrupted mk1 is
/// silently fixed inside `mk_codec::decode`, so the toolkit's auto-fire
/// short-circuit never gets called — `convert --from mk1=<1-error>` exits 0
/// with the xpub projection emitted, NOT exit 5 with a repair report.
///
/// This cell asserts the observable behavior: 1-char-corrupted mk1 still
/// produces the correct xpub via mk-codec's internal correction. Truly
/// unrepairable mk1 (>4 errors per chunk) surfaces as `BchUncorrectable`
/// which is the same beyond-capacity ceiling the toolkit's repair primitive
/// would also reject. The auto-fire wiring itself is exercised via the ms1
/// cell 19 (codex32 has no internal correction; only the toolkit's
/// `repair::try_repair_and_short_circuit` fires there).
#[test]
fn cell_20a_mk1_internal_correction_preempts_auto_fire() {
    let bad_chunk1 = flip_at(VALID_MK1_CHUNK1, 25);
    let mk1_value = format!("{VALID_MK1_CHUNK0} {bad_chunk1}");
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["convert", "--from", &format!("mk1={mk1_value}"), "--to", "xpub"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains(EXPECTED_XPUB))
        // No repair report — mk-codec's internal correction silently fixed it.
        .stdout(predicate::str::contains("# Repair report").not());
}

/// Cell 20b: md1 auto-fire via `inspect`. `md-codec` does NOT have internal
/// BCH correction (its `bch.rs` only `bch_verify_regular`s), so a 1-char
/// corruption in any md1 chunk surfaces as `md_codec::Error::Codex32DecodeError`
/// and the toolkit's auto-fire short-circuit fires.
#[test]
fn cell_20b_inspect_auto_fire_md1_one_substitution() {
    let bad_chunk0 = flip_at(VALID_MD1_CHUNK0, 20);
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "inspect",
            "--md1",
            &bad_chunk0,
            "--md1",
            VALID_MD1_CHUNK1,
            "--md1",
            VALID_MD1_CHUNK2,
        ])
        .assert()
        .code(5)
        .stdout(predicate::str::contains("# Repair report"))
        .stdout(predicate::str::contains(
            "md1 chunk 0: 1 correction at position 20",
        ))
        .stdout(predicate::str::contains(VALID_MD1_CHUNK0));
}

/// Cell 18b: inspect auto-fire on corrupted ms1 (formerly cell 18 in §4.3,
/// now lives here since auto-fire wiring is Phase 5).
#[test]
fn cell_18b_inspect_auto_fire_on_corrupted_ms1() {
    let bad = flip_at(VALID_MS1, 17);
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["inspect", "--ms1", &bad])
        .assert()
        .code(5)
        .stdout(predicate::str::contains("# Repair report"))
        .stdout(predicate::str::contains(VALID_MS1))
        .stderr(predicate::str::contains(
            "repair: applied 1 correction across 1 chunk",
        ));
}

/// Cell 22: --no-auto-repair suppresses auto-fire on both convert and inspect.
/// Exit code reverts to the pre-cycle typed-codec-error policy.
#[test]
fn cell_22_no_auto_repair_suppresses_short_circuit_on_convert_and_inspect() {
    let bad = flip_at(VALID_MS1, 17);

    // convert with --no-auto-repair → typed MsCodec error, NOT exit 5.
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "--no-auto-repair",
            "convert",
            "--from",
            &format!("ms1={bad}"),
            "--to",
            "phrase",
        ])
        .assert()
        .failure()
        .code(predicate::ne(5))
        .stdout(predicate::str::contains("# Repair report").not())
        .stderr(predicate::str::contains("error:"));

    // inspect with --no-auto-repair → same shape.
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["--no-auto-repair", "inspect", "--ms1", &bad])
        .assert()
        .failure()
        .code(predicate::ne(5))
        .stdout(predicate::str::contains("# Repair report").not())
        .stderr(predicate::str::contains("error:"));
}

/// Cell 23: bundle --self-check NOT auto-firing per D16. Synthetic
/// corruption is impossible to inject through the bundle path (the
/// toolkit synthesizes all three cards itself), so this cell asserts the
/// negative shape: a successful bundle invocation produces a clean exit-0
/// run with NO repair-report text anywhere (because auto-fire is wired
/// only into convert + inspect, not bundle).
#[test]
fn cell_23_bundle_self_check_does_not_auto_fire() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--network",
            "mainnet",
            "--template",
            "bip84",
            "--slot",
            "@0.phrase=abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
            "--account",
            "0",
            "--json",
            "--no-engraving-card",
        ])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("# Repair report").not())
        .stderr(predicate::str::contains("repair:").not());
}
