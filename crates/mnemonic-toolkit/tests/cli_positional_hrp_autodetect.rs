//! v0.24.0 §2.C.1 — positional `<STRING>...` intake with toolkit-internal
//! HRP-autodetect routing on `repair` + `inspect` + `verify-bundle`.
//!
//! Realizes plan §4.E.test-coverage:
//!   - positional single-HRP (repair / inspect)
//!   - positional mixed-HRP (D35 mutex drop allows this on repair / inspect)
//!   - mixed positional + flag (combined ordering)
//!   - unknown-HRP positional → `ToolkitError::UnknownHrp`
//!   - mismatched-HRP flag still rejects (D34 regression)
//!   - I3 regression: `verify-bundle --bundle-json X ms1xxx` → clap-error
//!     via `conflicts_with = "bundle_json"`
//!   - verify-bundle positional happy-path (watch-only single-sig)
//!
//! Fixtures: canonical `abandon × 11 about` toolkit-emitted bundle (same as
//! `cli_repair.rs` / `cli_inspect.rs` / `cli_auto_repair.rs`) plus the
//! `tests/vectors/v0_1/bip84-mainnet.txt` watch-only fixture for verify-bundle.

use assert_cmd::Command;
use predicates::prelude::*;

const VALID_MS1: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
const VALID_MK1_CHUNK0: &str = "mk1qprsqhpqqsq3cqtsleeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8qp3yxg3dpe854wq4";
const VALID_MK1_CHUNK1: &str = "mk1qprsqhpp0f30mtxzd65mvwcur9usdatwuqvq6z70r9nwrgk6xn6l8gy6nwa2n977sw6zh34rma0nh";
const VALID_MD1_CHUNK0: &str = "md1fgdxlpqpqpm6jzzqqvqpdqw0za5zs4gyy55aq4vsmnhy4s6wyaypu34c7raqu8np";
const VALID_MD1_CHUNK1: &str = "md1fgdxlpqf2zcgefcpupmel75q5435j7seugaj5jr7qyur6vt76es5cdeyrq7zdy0d";
const VALID_MD1_CHUNK2: &str = "md1fgdxlpq3xa2dk8vwpj7gx74hwqxqdp083jehp5tdrfa0n5zdfkqcdlrvnh5r62jn";

/// Helper: deterministically flip the bech32 character at `pos` (within
/// the data-part) to the next char in the bech32 alphabet (cyclic).
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

// ============================================================================
// repair — positional intake
// ============================================================================

/// Single-HRP positional: `mnemonic repair ms1xxx` → routes to ms1, emits
/// the corrected chunk (exit 5).
#[test]
fn repair_positional_single_ms1_routes_correctly() {
    let bad = flip_at(VALID_MS1, 17);
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["repair", &bad])
        .assert()
        .code(5)
        .stdout(predicate::str::contains("# Repair report"))
        .stdout(predicate::str::contains(
            "ms1 chunk 0: 1 correction at position 17",
        ))
        .stdout(predicate::str::contains(VALID_MS1));
}

/// Single-HRP positional: a single mk1 chunk via positional.
#[test]
fn repair_positional_single_mk1_routes_correctly() {
    // Already-valid mk1 chunk → exit 0, pass-through on stdout.
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["repair", VALID_MK1_CHUNK0])
        .assert()
        .code(0)
        .stdout(predicate::str::contains(VALID_MK1_CHUNK0));
}

/// Mixed-HRP positional (D35 mutex drop allows this): supply one ms1 + one mk1
/// + one md1 (all already-valid) → exit 0 with all three on stdout.
#[test]
fn repair_positional_mixed_hrp_d35_allows() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "repair",
            VALID_MS1,
            VALID_MK1_CHUNK0,
            VALID_MK1_CHUNK1,
            VALID_MD1_CHUNK0,
            VALID_MD1_CHUNK1,
            VALID_MD1_CHUNK2,
        ])
        .assert()
        .code(0)
        .stdout(predicate::str::contains(VALID_MS1))
        .stdout(predicate::str::contains(VALID_MK1_CHUNK0))
        .stdout(predicate::str::contains(VALID_MD1_CHUNK0));
}

/// Mixed positional + flag: `mnemonic repair --ms1 ms1xxx mk1yyy` → both
/// routed correctly; ms1 from --ms1 flag combined with mk1 from positional.
#[test]
fn repair_mixed_positional_and_flag_combined_routing() {
    let bad_ms1 = flip_at(VALID_MS1, 17);
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["repair", "--ms1", &bad_ms1, VALID_MK1_CHUNK0])
        .assert()
        .code(5)
        .stdout(predicate::str::contains(VALID_MS1))
        .stdout(predicate::str::contains(VALID_MK1_CHUNK0))
        .stdout(predicate::str::contains(
            "ms1 chunk 0: 1 correction at position 17",
        ));
}

/// Unknown-HRP positional: `mnemonic repair abc1xxx` → exit 2 with
/// `ToolkitError::UnknownHrp` (message cites "expected one of: ms1, mk1, md1").
#[test]
fn repair_positional_unknown_hrp_rejects_with_unknown_hrp() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["repair", "abc1xxxxxx"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains(
            "does not begin with a recognized HRP prefix",
        ))
        .stderr(predicate::str::contains("ms1"))
        .stderr(predicate::str::contains("mk1"))
        .stderr(predicate::str::contains("md1"));
}

/// D34 regression: `mnemonic repair --ms1 mk1xxx` still rejects strictly
/// with `ToolkitError::HrpMismatch`. Mismatched-HRP via typed flag is the
/// load-bearing D34/I5 invariant we did NOT loosen with D35.
#[test]
fn repair_flag_value_mismatched_hrp_rejects_d34() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["repair", "--ms1", VALID_MK1_CHUNK0])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("--ms1"))
        .stderr(predicate::str::contains("expects a value with HRP 'ms'"))
        .stderr(predicate::str::contains("got 'mk'"));
}

/// Symmetric D34: --mk1 with an ms1 value.
#[test]
fn repair_mk1_flag_with_ms1_value_rejects_d34() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["repair", "--mk1", VALID_MS1])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("--mk1"))
        .stderr(predicate::str::contains("expects a value with HRP 'mk'"))
        .stderr(predicate::str::contains("got 'ms'"));
}

/// At-least-one constraint: invoking `mnemonic repair` with NO args
/// (neither flag nor positional) → clap rejects per the
/// `required_unless_present_any` on the positional.
#[test]
fn repair_no_args_clap_rejects() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["repair"])
        .assert()
        .failure();
}

// ============================================================================
// inspect — positional intake
// ============================================================================

/// Single-HRP positional: `mnemonic inspect mk1xxx mk1yyy` → routes to mk1,
/// emits the standard mk1 text-form summary.
#[test]
fn inspect_positional_single_mk1_routes_correctly() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["inspect", VALID_MK1_CHUNK0, VALID_MK1_CHUNK1])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("kind: mk1"))
        .stdout(predicate::str::contains("origin_path: m/84'/0'/0'"))
        .stdout(predicate::str::contains("xpub: xpub"));
}

/// Mixed-HRP positional (D35 mutex drop): inspect one ms1 + one mk1 in
/// a single invocation → emits both summaries.
#[test]
fn inspect_positional_mixed_hrp_d35_allows() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "inspect",
            VALID_MS1,
            VALID_MK1_CHUNK0,
            VALID_MK1_CHUNK1,
        ])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("kind: ms1"))
        .stdout(predicate::str::contains("kind: mk1"));
}

/// Mixed positional + flag.
#[test]
fn inspect_mixed_positional_and_flag_combined_routing() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["inspect", "--ms1", VALID_MS1, VALID_MK1_CHUNK0, VALID_MK1_CHUNK1])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("kind: ms1"))
        .stdout(predicate::str::contains("kind: mk1"));
}

/// Unknown-HRP positional on inspect.
#[test]
fn inspect_positional_unknown_hrp_rejects_with_unknown_hrp() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["inspect", "abc1xxxxxx"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains(
            "does not begin with a recognized HRP prefix",
        ));
}

/// D34 regression on inspect: `--md1 ms1xxx` rejects strictly.
#[test]
fn inspect_md1_flag_with_ms1_value_rejects_d34() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["inspect", "--md1", VALID_MS1])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("--md1"))
        .stderr(predicate::str::contains("expects a value with HRP 'md'"))
        .stderr(predicate::str::contains("got 'ms'"));
}

// ============================================================================
// verify-bundle — positional intake + I3 regression
// ============================================================================

/// I3 regression: `mnemonic verify-bundle --bundle-json foo.json ms1xxx`
/// → clap rejects with `conflicts_with = "bundle_json"` error text. This is
/// the LOAD-BEARING invariant per I3 fold: positional must be mutually
/// exclusive with `--bundle-json` to preserve the existing cards-vs-bundle
/// XOR semantic.
#[test]
fn verify_bundle_bundle_json_xor_positional_clap_rejects_per_i3() {
    let tmpdir = tempfile::tempdir().unwrap();
    let path = tmpdir.path().join("dummy.json");
    std::fs::write(&path, "{}").unwrap();

    let assert = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "verify-bundle",
            "--network",
            "mainnet",
            "--template",
            "bip84",
            "--bundle-json",
            path.to_str().unwrap(),
            VALID_MS1,
        ])
        .assert()
        .failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    // clap's `conflicts_with` error text mentions both flags.
    assert!(
        stderr.contains("cannot be used with") || stderr.contains("conflict"),
        "expected clap conflicts-with error, got: {stderr}"
    );
}

/// Positional happy-path on verify-bundle: drive the watch-only bip84-mainnet
/// fixture via positional `<STRING>...` (no `--mk1` / `--md1` flags).
#[test]
fn verify_bundle_positional_watch_only_round_trip() {
    let fixture =
        std::fs::read_to_string("tests/vectors/v0_1/bip84-mainnet.txt").expect("fixture exists");
    let mk1: Vec<String> = fixture
        .lines()
        .filter(|l| l.starts_with("mk1") && !l.contains(' ') && !l.contains('-'))
        .map(String::from)
        .collect();
    let md1: Vec<String> = fixture
        .lines()
        .filter(|l| l.starts_with("md1") && !l.contains(' ') && !l.contains('-'))
        .map(String::from)
        .collect();
    assert!(!mk1.is_empty() && !md1.is_empty(), "fixture must yield mk1+md1");

    // Derive the watch-only @0.xpub + @0.fingerprint slots from mk1.
    let mk_refs: Vec<&str> = mk1.iter().map(|s| s.as_str()).collect();
    let card = mk_codec::decode(&mk_refs).expect("mk1 decodes");
    let xpub_str = card.xpub.to_string();
    let fp_str = card
        .origin_fingerprint
        .expect("fingerprint present")
        .to_string()
        .to_lowercase();

    let mut argv: Vec<String> = vec![
        "verify-bundle".into(),
        "--slot".into(),
        format!("@0.xpub={xpub_str}"),
        "--slot".into(),
        format!("@0.fingerprint={fp_str}"),
        "--network".into(),
        "mainnet".into(),
        "--template".into(),
        "bip84".into(),
    ];
    // Pass mk1 + md1 strings positionally — no --mk1 / --md1 flags.
    for s in &mk1 {
        argv.push(s.clone());
    }
    for s in &md1 {
        argv.push(s.clone());
    }

    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&argv)
        .assert()
        .success()
        .stdout(predicate::str::contains("result: ok"));
}

/// D34 regression on verify-bundle (architect review C1 fold): passing a
/// mismatched-HRP value to a typed flag — e.g. `--ms1 mk1xxx` — must reject
/// with `ToolkitError::HrpMismatch` rather than dropping through to the
/// sibling-codec parse with no flag-name attribution. Mirrors
/// `repair_flag_value_mismatched_hrp_rejects_d34` +
/// `inspect_md1_flag_with_ms1_value_rejects_d34` for the third subcommand.
///
/// We pair the bad `--ms1` with valid `--mk1` + `--md1` from the watch-only
/// fixture so clap's `required_unless_present_any` gate is satisfied before
/// `run()` executes the D34 validation.
#[test]
fn verify_bundle_flag_value_mismatched_hrp_rejects_d34() {
    let fixture =
        std::fs::read_to_string("tests/vectors/v0_1/bip84-mainnet.txt").expect("fixture exists");
    let mk1: Vec<String> = fixture
        .lines()
        .filter(|l| l.starts_with("mk1") && !l.contains(' ') && !l.contains('-'))
        .map(String::from)
        .collect();
    let md1: Vec<String> = fixture
        .lines()
        .filter(|l| l.starts_with("md1") && !l.contains(' ') && !l.contains('-'))
        .map(String::from)
        .collect();
    assert!(!mk1.is_empty() && !md1.is_empty(), "fixture must yield mk1+md1");

    let mut argv: Vec<String> = vec![
        "verify-bundle".into(),
        "--network".into(),
        "mainnet".into(),
        "--template".into(),
        "bip84".into(),
        // The D34 trigger — `--ms1` flag carrying a value with a non-`ms1` HRP.
        "--ms1".into(),
        VALID_MK1_CHUNK0.into(),
    ];
    for s in &mk1 {
        argv.push("--mk1".into());
        argv.push(s.clone());
    }
    for s in &md1 {
        argv.push("--md1".into());
        argv.push(s.clone());
    }

    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&argv)
        .assert()
        .code(2)
        .stderr(predicate::str::contains("--ms1"))
        .stderr(predicate::str::contains("expects a value with HRP 'ms'"))
        .stderr(predicate::str::contains("got 'mk'"));
}

/// v0.53.3 (audit M11) INVERSION of the v0.24.0 I5 pin: `validate_flag_hrp`'s
/// deliberate case-mismatch rejection is RELAXED — the codecs are the
/// authority on case (BIP-173 uppercase-QR cards exist in the wild), so a
/// consistent-case `--ms1 MS1XXX` is no longer rejected at flag-validation
/// time. The 6-char fixture instead dies in the repair path's own
/// `parse_chunk` pre-gate (which lowercases + length-gates, so it NEVER
/// reaches ms-codec) with the parse-step marker. The historical I5 concern
/// ("expected 'ms' got 'ms'") stays moot: no case-mismatch message exists
/// anymore, and the true-HRP-mismatch path still compares lowercased
/// prefixes (see `repair_flag_value_mismatched_hrp_rejects_d34`).
#[test]
fn validate_flag_hrp_case_mismatch_relaxed_reaches_parse_step() {
    let assert = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["repair", "--ms1", "MS1XXX"])
        .assert()
        .code(2);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    // The relaxation's negative marker: no case-mismatch rejection fires.
    assert!(
        !stderr.contains("case mismatch"),
        "the I5 case-mismatch rejection was relaxed (codecs own case); \
         got stderr: {stderr}"
    );
    // The positive marker: the value passed flag validation and failed in
    // the toolkit's own parse_chunk pre-gate (6 chars never reach ms-codec).
    assert!(
        stderr.contains("parse failed before correction could run"),
        "expected the parse-step marker, got: {stderr}"
    );
    // The M3 secret-in-argv advisory for the inline --ms1 value fires
    // independently of the error (asserted separately — pre-relaxation a
    // bare `contains("--ms1")` would have passed VACUOUSLY via this line).
    assert!(
        stderr.contains("warning: secret material on argv (--ms1)"),
        "expected the --ms1 argv-leak advisory, got: {stderr}"
    );
}

/// Verify-bundle: positional unknown-HRP rejects with `ToolkitError::UnknownHrp`.
#[test]
fn verify_bundle_positional_unknown_hrp_rejects() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "verify-bundle",
            "--network",
            "mainnet",
            "--template",
            "bip84",
            "--slot",
            "@0.xpub=xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V",
            "abc1xxxxxx",
        ])
        .assert()
        .code(2)
        .stderr(predicate::str::contains(
            "does not begin with a recognized HRP prefix",
        ));
}
