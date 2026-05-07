//! v0.7 Phase 3 — `mnemonic convert` Electrum native seed format.
//! Reference vectors: `design/agent-reports/v0_7-phase-3-electrum-corpus-spike.md`
//! (sourced from spesmilo/electrum `tests/test_mnemonic.py` +
//! `tests/test_wallet_vertical.py::test_electrum_seed_2fa_segwit`).

use assert_cmd::Command;

// --- Reference phrases (verified GREEN in corpus spike) ---
//
// All 4 phrases below are the canonical Electrum reference corpus entries,
// one per SeedVersion (01/100/101/102), sourced from spesmilo/electrum:
// - 01, 100, 101 from `tests/test_mnemonic.py::Test_seeds.mnemonics`.
// - 102 from `tests/test_wallet_vertical.py::test_electrum_seed_2fa_segwit`.
// HMAC-SHA512 prefix predictions cross-validated in v0.7 Phase 3 spike
// (`design/agent-reports/v0_7-phase-3-electrum-corpus-spike.md`).

const STANDARD_PHRASE: &str =
    "cram swing cover prefer miss modify ritual silly deliver chunk behind inform able";
const STANDARD_HEX: &str = "2738290a29d0c8b7523ac6ea9c63370191";

const SEGWIT_PHRASE: &str =
    "wild father tree among universe such mobile favorite target dynamic credit identify";
const SEGWIT_HEX: &str = "0708661136ef5411cf61f6e07fcfd4efd8";

const STANDARD_2FA_PHRASE: &str =
    "science dawn member doll dutch real can brick knife deny drive list";
const SEGWIT_2FA_PHRASE: &str =
    "universe topic remind silver february ranch shine worth innocent cattle enhance wise";

/// Helper: extract value from `<node>: <value>\n` stdout.
fn convert_value(args: &[&str]) -> String {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(args)
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let line = stdout.trim();
    let colon = line.find(": ").expect("convert output must be '<node>: <value>'");
    line[colon + 2..].to_string()
}

// ============================================================================
// Decode happy paths — Standard + Segwit
// ============================================================================

#[test]
fn decode_standard_phrase_to_entropy() {
    let out = convert_value(&[
        "convert",
        "--from",
        &format!("electrum-phrase={STANDARD_PHRASE}"),
        "--to",
        "entropy",
    ]);
    assert_eq!(out, STANDARD_HEX);
}

#[test]
fn decode_segwit_phrase_to_entropy() {
    let out = convert_value(&[
        "convert",
        "--from",
        &format!("electrum-phrase={SEGWIT_PHRASE}"),
        "--to",
        "entropy",
    ]);
    assert_eq!(out, SEGWIT_HEX);
}

// ============================================================================
// Encode happy paths — Standard + Segwit (with --electrum-version flag)
// ============================================================================

#[test]
fn encode_entropy_to_standard_phrase() {
    let out = convert_value(&[
        "convert",
        "--from",
        &format!("entropy={STANDARD_HEX}"),
        "--to",
        "electrum-phrase",
    ]);
    assert_eq!(out, STANDARD_PHRASE);
}

#[test]
fn encode_entropy_to_segwit_phrase_via_flag() {
    let out = convert_value(&[
        "convert",
        "--from",
        &format!("entropy={SEGWIT_HEX}"),
        "--to",
        "electrum-phrase",
        "--electrum-version",
        "segwit",
    ]);
    assert_eq!(out, SEGWIT_PHRASE);
}

#[test]
fn encode_explicit_standard_flag_matches_default() {
    // `--electrum-version standard` is a no-op equivalent of the default.
    let out = convert_value(&[
        "convert",
        "--from",
        &format!("entropy={STANDARD_HEX}"),
        "--to",
        "electrum-phrase",
        "--electrum-version",
        "standard",
    ]);
    assert_eq!(out, STANDARD_PHRASE);
}

// ============================================================================
// Round-trip via entropy
// ============================================================================

#[test]
fn round_trip_standard_phrase_via_entropy() {
    let entropy = convert_value(&[
        "convert",
        "--from",
        &format!("electrum-phrase={STANDARD_PHRASE}"),
        "--to",
        "entropy",
    ]);
    let phrase = convert_value(&[
        "convert",
        "--from",
        &format!("entropy={entropy}"),
        "--to",
        "electrum-phrase",
        "--electrum-version",
        "standard",
    ]);
    assert_eq!(phrase, STANDARD_PHRASE);
}

#[test]
fn round_trip_segwit_phrase_via_entropy() {
    let entropy = convert_value(&[
        "convert",
        "--from",
        &format!("electrum-phrase={SEGWIT_PHRASE}"),
        "--to",
        "entropy",
    ]);
    let phrase = convert_value(&[
        "convert",
        "--from",
        &format!("entropy={entropy}"),
        "--to",
        "electrum-phrase",
        "--electrum-version",
        "segwit",
    ]);
    assert_eq!(phrase, SEGWIT_PHRASE);
}

// ============================================================================
// Refusals — SPEC §3.d / §14
// ============================================================================

#[test]
fn refusal_standard_2fa_decode() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("electrum-phrase={STANDARD_2FA_PHRASE}"),
            "--to",
            "entropy",
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert_eq!(
        stderr,
        "error: Electrum 2FA seed (version 101 or 102) requires a second factor not present in the phrase alone; conversion not supported. Use Electrum directly for 2FA recovery.\n"
    );
}

#[test]
fn refusal_segwit_2fa_decode() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("electrum-phrase={SEGWIT_2FA_PHRASE}"),
            "--to",
            "entropy",
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert_eq!(
        stderr,
        "error: Electrum 2FA seed (version 101 or 102) requires a second factor not present in the phrase alone; conversion not supported. Use Electrum directly for 2FA recovery.\n"
    );
}

#[test]
fn refusal_phrase_to_electrum_phrase_sibling_pivot() {
    // BIP-39 valid phrase (12 words) — toolkit must intercept as sibling pivot
    // before any decode attempt.
    let bip39_phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("phrase={bip39_phrase}"),
            "--to",
            "electrum-phrase",
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert_eq!(
        stderr,
        "error: --from phrase --to electrum-phrase (or reverse) is a sibling-format pivot, not a single-format conversion. BIP-39 and Electrum native seeds are different artifact classes.\n"
    );
}

#[test]
fn refusal_electrum_phrase_to_phrase_sibling_pivot() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("electrum-phrase={STANDARD_PHRASE}"),
            "--to",
            "phrase",
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert_eq!(
        stderr,
        "error: --from phrase --to electrum-phrase (or reverse) is a sibling-format pivot, not a single-format conversion. BIP-39 and Electrum native seeds are different artifact classes.\n"
    );
}

#[test]
fn refusal_invalid_electrum_phrase_format() {
    // Random text — fails HMAC prefix dispatch.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            "electrum-phrase=this is not a real electrum seed phrase at all just garbage tokens",
            "--to",
            "entropy",
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert_eq!(
        stderr,
        "error: --from electrum-phrase value is not a valid Electrum native seed (HMAC-SHA512 prefix did not match a known seed version, or contains words outside the wordlist).\n"
    );
}

#[test]
fn refusal_electrum_version_2fa_at_arg_parse() {
    // `--electrum-version standard-2fa` and `101`/`102` are rejected by
    // the value parser, never reaching the encode arm.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("entropy={STANDARD_HEX}"),
            "--to",
            "electrum-phrase",
            "--electrum-version",
            "standard-2fa",
        ])
        .assert()
        .failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("--electrum-version \"standard-2fa\" is refused"),
        "stderr did not mention the refusal: {stderr:?}"
    );
}

#[test]
fn refusal_electrum_phrase_to_electrum_phrase_identity() {
    // Identity-pivot — caught by the catch-all one-way refusal.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("electrum-phrase={STANDARD_PHRASE}"),
            "--to",
            "electrum-phrase",
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    // Catch-all refusal taxonomy (one-way barrier message).
    assert_eq!(
        stderr,
        "error: --to electrum-phrase is cryptographically unrecoverable from --from electrum-phrase (one-way derivation barrier)\n"
    );
}
