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

// ============================================================================
// v0.22.1 D20 — JSON-context auto-fire output cells
// ============================================================================

/// Cell 24: convert --json + corrupted ms1 → auto-fire emits a JSON
/// envelope on stdout (NOT text-form) with the D20 discriminator fields.
#[test]
fn cell_24_convert_json_context_auto_fire_emits_json_envelope() {
    let bad = flip_at(VALID_MS1, 17);
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--json",
            "--from",
            &format!("ms1={bad}"),
            "--to",
            "phrase",
        ])
        .assert()
        .code(5)
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    // Should NOT contain text-form report markers.
    assert!(!s.contains("# Repair report"), "JSON context must not emit text-form headers; got: {s}");
    // Should parse as a single JSON envelope.
    let v: serde_json::Value = serde_json::from_str(s.trim()).expect("valid JSON envelope");
    assert_eq!(v["schema_version"], "1");
    assert_eq!(v["auto_repair_short_circuit"], true);
    assert_eq!(v["exit_code"], 5);
    assert_eq!(v["kind"], "ms1");
    assert_eq!(v["corrected_chunks"][0], VALID_MS1);
    assert_eq!(v["repairs"][0]["chunk_index"], 0);
    assert_eq!(v["repairs"][0]["corrected_positions"][0]["position"], 17);
}

/// Cell 25: inspect --json + corrupted md1 → auto-fire emits a JSON
/// envelope on stdout. (md1 is the auto-fire-observable kind for inspect
/// because md-codec has no internal correction; the mk1 path is preempted
/// by mk-codec's internal correction per cell 20a.)
#[test]
fn cell_25_inspect_json_context_auto_fire_emits_json_envelope() {
    let bad_chunk0 = flip_at(VALID_MD1_CHUNK0, 20);
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "inspect",
            "--json",
            "--md1",
            &bad_chunk0,
            "--md1",
            VALID_MD1_CHUNK1,
            "--md1",
            VALID_MD1_CHUNK2,
        ])
        .assert()
        .code(5)
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    assert!(!s.contains("# Repair report"));
    let v: serde_json::Value = serde_json::from_str(s.trim()).expect("valid JSON envelope");
    assert_eq!(v["schema_version"], "1");
    assert_eq!(v["auto_repair_short_circuit"], true);
    assert_eq!(v["exit_code"], 5);
    assert_eq!(v["kind"], "md1");
    assert_eq!(v["corrected_chunks"][0], VALID_MD1_CHUNK0);
    assert_eq!(v["repairs"][0]["chunk_index"], 0);
    assert_eq!(v["repairs"][0]["corrected_positions"][0]["position"], 20);
}

/// Cell 26: D20 schema pin. Verify full envelope structure (all 6
/// top-level fields present with the documented types) for a known-shape
/// invocation. Distinct from cells 24/25 which assert specific FIELD
/// values — this cell asserts the SCHEMA itself stays stable.
#[test]
fn cell_26_d20_json_envelope_schema_v1_pin() {
    let bad = flip_at(VALID_MS1, 17);
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--json",
            "--from",
            &format!("ms1={bad}"),
            "--to",
            "phrase",
        ])
        .assert()
        .code(5)
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    let v: serde_json::Value = serde_json::from_str(s.trim()).unwrap();

    // Type assertions for each top-level field (presence + type, NOT order —
    // serde_json::Value is BTreeMap-backed and loses key order on parse).
    assert!(v["schema_version"].is_string());
    assert!(v["auto_repair_short_circuit"].is_boolean());
    assert!(v["exit_code"].is_number());
    assert!(v["kind"].is_string());
    assert!(v["corrected_chunks"].is_array());
    assert!(v["repairs"].is_array());
    // Exhaustive field-set check — fails if a future change ADDS or REMOVES
    // a top-level field without updating the schema-pin.
    let obj = v.as_object().unwrap();
    let mut keys: Vec<&str> = obj.keys().map(|s| s.as_str()).collect();
    keys.sort();
    assert_eq!(
        keys,
        vec![
            "auto_repair_short_circuit",
            "corrected_chunks",
            "exit_code",
            "kind",
            "repairs",
            "schema_version",
        ],
        "D20 schema-v1 top-level field-set pin (alphabetically sorted)"
    );

    // Per-detail field-set check.
    let detail = &v["repairs"][0];
    let mut detail_keys: Vec<&str> = detail.as_object().unwrap().keys().map(|s| s.as_str()).collect();
    detail_keys.sort();
    assert_eq!(
        detail_keys,
        vec!["chunk_index", "corrected_chunk", "corrected_positions", "original_chunk"],
        "D20 repair-detail field-set pin (alphabetically sorted)"
    );

    let pos = &detail["corrected_positions"][0];
    let mut pos_keys: Vec<&str> = pos.as_object().unwrap().keys().map(|s| s.as_str()).collect();
    pos_keys.sort();
    assert_eq!(pos_keys, vec!["now", "position", "was"], "D20 position field-set pin");

    // Serialized field-order pin (the raw output IS ordered per serde
    // struct-field order). schema_version must come first; the discriminator
    // pair (auto_repair_short_circuit + exit_code) follows; the data fields
    // (kind, corrected_chunks, repairs) close.
    let raw_order_check = |needle_a: &str, needle_b: &str| {
        let ia = s.find(needle_a).expect(needle_a);
        let ib = s.find(needle_b).expect(needle_b);
        ia < ib
    };
    assert!(raw_order_check("\"schema_version\"", "\"auto_repair_short_circuit\""));
    assert!(raw_order_check("\"auto_repair_short_circuit\"", "\"exit_code\""));
    assert!(raw_order_check("\"exit_code\"", "\"kind\""));
    assert!(raw_order_check("\"kind\"", "\"corrected_chunks\""));
    assert!(raw_order_check("\"corrected_chunks\"", "\"repairs\""));
}

// ============================================================================
// v0.22.1 Phase 4 — verify-bundle auto-fire with TTY-conditional D18 default
// ============================================================================

/// Helper: synthesize a clean bundle JSON for the canonical test phrase,
/// then corrupt the ms1[0] chunk at the given position. Returns the
/// corrupted JSON as a String suitable for `--bundle-json /dev/stdin`.
fn synth_corrupted_bundle_json(corrupt_pos: usize) -> String {
    let clean_json = Command::cargo_bin("mnemonic")
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
        .get_output()
        .stdout
        .clone();
    let mut v: serde_json::Value = serde_json::from_slice(&clean_json).unwrap();
    let ms1_arr = v["ms1"].as_array().unwrap();
    let chunk = ms1_arr[0].as_str().unwrap().to_string();
    let corrupted = flip_at(&chunk, corrupt_pos);
    v["ms1"][0] = serde_json::Value::String(corrupted);
    serde_json::to_string(&v).unwrap()
}

fn write_temp_json(body: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "mnemonic_v0_22_1_bundle_{}.json",
        std::process::id()
    ));
    std::fs::write(&path, body).unwrap();
    path
}

/// Cell 27: verify-bundle auto-fire happy-path under TTY. The corrupted
/// ms1[0] chunk triggers the D18 TTY-gated auto-fire; output is the D20
/// JSON envelope (we use --json to also exercise that pairing).
#[test]
fn cell_27_verify_bundle_auto_fire_happy_path_tty() {
    let bad_json = synth_corrupted_bundle_json(17);
    let path = write_temp_json(&bad_json);
    Command::cargo_bin("mnemonic")
        .unwrap()
        .env("MNEMONIC_FORCE_TTY", "1")
        .args([
            "verify-bundle",
            "--network",
            "mainnet",
            "--template",
            "bip84",
            "--slot",
            "@0.phrase=abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
            "--bundle-json",
            path.to_str().unwrap(),
        ])
        .assert()
        .code(5)
        .stdout(predicate::str::contains("# Repair report"))
        .stdout(predicate::str::contains(
            "ms1 chunk 0: 1 correction at position 17",
        ));
}

/// Cell 28: --no-auto-repair hard override even under TTY. Falls back to
/// legacy VerifyCheck row + exit 4.
#[test]
fn cell_28_verify_bundle_no_auto_repair_forced_off() {
    let bad_json = synth_corrupted_bundle_json(17);
    let path = write_temp_json(&bad_json);
    Command::cargo_bin("mnemonic")
        .unwrap()
        .env("MNEMONIC_FORCE_TTY", "1")
        .args([
            "--no-auto-repair",
            "verify-bundle",
            "--network",
            "mainnet",
            "--template",
            "bip84",
            "--slot",
            "@0.phrase=abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
            "--bundle-json",
            path.to_str().unwrap(),
        ])
        .assert()
        .code(4)
        .stdout(predicate::str::contains("# Repair report").not())
        .stdout(predicate::str::contains("ms1_decode: fail"));
}

/// Cell 29: piped (non-TTY) preserves legacy VerifyCheck behavior — no
/// auto-fire even though --no-auto-repair was NOT set. Verifies the D18
/// invariant that pipe-context preserves automation contract.
#[test]
fn cell_29_verify_bundle_piped_preserves_legacy() {
    let bad_json = synth_corrupted_bundle_json(17);
    let path = write_temp_json(&bad_json);
    Command::cargo_bin("mnemonic")
        .unwrap()
        // MNEMONIC_FORCE_TTY=0 explicitly forces non-TTY (cargo test stdout
        // is already non-TTY but this makes intent unambiguous).
        .env("MNEMONIC_FORCE_TTY", "0")
        .args([
            "verify-bundle",
            "--network",
            "mainnet",
            "--template",
            "bip84",
            "--slot",
            "@0.phrase=abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
            "--bundle-json",
            path.to_str().unwrap(),
        ])
        .assert()
        .code(4)
        .stdout(predicate::str::contains("# Repair report").not())
        .stdout(predicate::str::contains("ms1_decode: fail"));
}

/// Cell 30: --json + TTY + corrupted bundle → D20 JSON envelope (NOT the
/// VerifyBundleJson check-array envelope; the auto-fire envelope short-
/// circuits before the verify check array is built).
#[test]
fn cell_30_verify_bundle_json_context_under_tty_emits_envelope() {
    let bad_json = synth_corrupted_bundle_json(17);
    let path = write_temp_json(&bad_json);
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .env("MNEMONIC_FORCE_TTY", "1")
        .args([
            "verify-bundle",
            "--json",
            "--network",
            "mainnet",
            "--template",
            "bip84",
            "--slot",
            "@0.phrase=abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
            "--bundle-json",
            path.to_str().unwrap(),
        ])
        .assert()
        .code(5)
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    let v: serde_json::Value = serde_json::from_str(s.trim()).expect("valid JSON envelope");
    // Auto-fire envelope discriminators (D20).
    assert_eq!(v["schema_version"], "1");
    assert_eq!(v["auto_repair_short_circuit"], true);
    assert_eq!(v["exit_code"], 5);
    assert_eq!(v["kind"], "ms1");
    // Should NOT be the VerifyBundleJson schema (which has `result`+`checks`).
    assert!(v["result"].is_null(), "should not be VerifyBundleJson envelope");
    assert!(v["checks"].is_null(), "should not be VerifyBundleJson envelope");
}
