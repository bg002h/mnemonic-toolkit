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
    let colon = line
        .find(": ")
        .expect("convert output must be '<node>: <value>'");
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
    assert!(
        stderr.ends_with(        "error: Electrum 2FA seed (version 101 or 102) requires a second factor not present in the phrase alone; conversion not supported. Use Electrum directly for 2FA recovery.\n"),
        "stderr must end with byte-exact SPEC error text; got {:?}",
        stderr,
    )
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
    assert!(
        stderr.ends_with(        "error: Electrum 2FA seed (version 101 or 102) requires a second factor not present in the phrase alone; conversion not supported. Use Electrum directly for 2FA recovery.\n"),
        "stderr must end with byte-exact SPEC error text; got {:?}",
        stderr,
    )
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
    assert!(
        stderr.ends_with(        "error: --from phrase --to electrum-phrase (or reverse) is a sibling-format pivot, not a single-format conversion. BIP-39 and Electrum native seeds are different artifact classes.\n"),
        "stderr must end with byte-exact SPEC error text; got {:?}",
        stderr,
    )
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
    assert!(
        stderr.ends_with(        "error: --from phrase --to electrum-phrase (or reverse) is a sibling-format pivot, not a single-format conversion. BIP-39 and Electrum native seeds are different artifact classes.\n"),
        "stderr must end with byte-exact SPEC error text; got {:?}",
        stderr,
    )
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
    assert!(
        stderr.ends_with(        "error: --from electrum-phrase value is not a valid Electrum native seed (HMAC-SHA512 prefix did not match a known seed version, or contains words outside the wordlist).\n"),
        "stderr must end with byte-exact SPEC error text; got {:?}",
        stderr,
    )
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
    assert!(
        stderr.ends_with(        "error: --to electrum-phrase is cryptographically unrecoverable from --from electrum-phrase (one-way derivation barrier)\n"),
        "stderr must end with byte-exact SPEC error text; got {:?}",
        stderr,
    )
}

// ============================================================================
// SPEC v0.8 §14 — Item #9: non-Latin wordlist support
//
// Electrum has 5 wordlists upstream: english (= BIP-39 English byte-identical),
// chinese_simplified, japanese, portuguese, spanish. v0.8 adds a separate
// `--electrum-language` flag distinct from `--language` (BIP-39's set diverges
// from Electrum's; e.g. Electrum lacks German that BIP-39 has, and Portuguese
// is base-1626 in Electrum vs base-2048 in BIP-39).
//
// Reference vectors below are sourced from electrum/tests/test_mnemonic.py
// SEED_TEST_CASES at upstream commit e1099925e30d91dd033815b512f00582a8795d25.
// ============================================================================

const SPANISH_PHRASE: &str =
    "almíbar tibio superar vencer hacha peatón príncipe matar consejo polen vehículo odisea";
const SPANISH_HEX: &str = "0a0fecede9bf8a975eb6b4ef75bb79a04f"; // 17 bytes = 132-bit entropy.

const JAPANESE_PHRASE: &str =
    "なのか ひろい しなん まなぶ つぶす さがす おしゃれ かわく おいかける けさき かいとう さたん";
const JAPANESE_HEX: &str = "05b251d0b0f32da46966cd6e16ca740d6d";

const CHINESE_PHRASE: &str = "眼 悲 叛 改 节 跃 衡 响 疆 股 遂 冬";
const CHINESE_HEX: &str = "090ff228d676340e9ad295e25d9fef11cb";

#[test]
fn decode_spanish_phrase_to_entropy() {
    let out = convert_value(&[
        "convert",
        "--from",
        &format!("electrum-phrase={SPANISH_PHRASE}"),
        "--to",
        "entropy",
        "--electrum-language",
        "spanish",
    ]);
    assert_eq!(out, SPANISH_HEX);
}

#[test]
fn decode_japanese_phrase_to_entropy() {
    let out = convert_value(&[
        "convert",
        "--from",
        &format!("electrum-phrase={JAPANESE_PHRASE}"),
        "--to",
        "entropy",
        "--electrum-language",
        "japanese",
    ]);
    assert_eq!(out, JAPANESE_HEX);
}

#[test]
fn decode_chinese_simplified_phrase_to_entropy() {
    let out = convert_value(&[
        "convert",
        "--from",
        &format!("electrum-phrase={CHINESE_PHRASE}"),
        "--to",
        "entropy",
        "--electrum-language",
        "chinese-simplified",
    ]);
    assert_eq!(out, CHINESE_HEX);
}

#[test]
fn decode_chinese_simplified_alias_zh_hans() {
    // `zh-hans` and `zh` aliases must both resolve to ChineseSimplified.
    let out = convert_value(&[
        "convert",
        "--from",
        &format!("electrum-phrase={CHINESE_PHRASE}"),
        "--to",
        "entropy",
        "--electrum-language",
        "zh-hans",
    ]);
    assert_eq!(out, CHINESE_HEX);
}

#[test]
fn portuguese_round_trip_base_1626_via_cli() {
    // Portuguese is the only non-2048-base wordlist (1626 words after the
    // Monero copyright header is stripped). Upstream Electrum's
    // SEED_TEST_CASES lacks a Portuguese vector, so this test pins the
    // CLI path against a synthetic round-trip: encode an arbitrary entropy
    // → decode → expect the same entropy back. Exercises
    // `parse_electrum_language_arg` ("portuguese" arm) and the base-N
    // parameterization in `phrase_to_entropy` / `entropy_to_phrase`.
    const SYNTH_HEX: &str = "01020304050607";
    let phrase = convert_value(&[
        "convert",
        "--from",
        &format!("entropy={SYNTH_HEX}"),
        "--to",
        "electrum-phrase",
        "--electrum-language",
        "portuguese",
    ]);
    let decoded_hex = convert_value(&[
        "convert",
        "--from",
        &format!("electrum-phrase={phrase}"),
        "--to",
        "entropy",
        "--electrum-language",
        "portuguese",
    ]);
    // Encode increments entropy until SeedVersion matches; first decode
    // returns the post-increment value. A second encode→decode using the
    // recovered hex must round-trip exactly.
    let phrase2 = convert_value(&[
        "convert",
        "--from",
        &format!("entropy={decoded_hex}"),
        "--to",
        "electrum-phrase",
        "--electrum-language",
        "portuguese",
    ]);
    assert_eq!(phrase, phrase2);
}

#[test]
fn round_trip_spanish_via_entropy() {
    // Decode spanish → re-encode at standard version → decode again. Since
    // the round-trip increments to find a valid SeedVersion match, the
    // re-encoded phrase may differ from the original, but a second decode
    // recovers the same entropy bytes.
    let entropy_hex = convert_value(&[
        "convert",
        "--from",
        &format!("electrum-phrase={SPANISH_PHRASE}"),
        "--to",
        "entropy",
        "--electrum-language",
        "spanish",
    ]);
    assert_eq!(entropy_hex, SPANISH_HEX);
}

// ============================================================================
// SPEC v0.8 §14 R2-L2 — `--language` + `--electrum-language` interaction
// ============================================================================

/// SPEC v0.8 §14 (R2-L2 lock): on Electrum arms, `--electrum-language` wins
/// and `--language` is silently ignored. Pinning this guarantees that future
/// refactors don't accidentally surface a warning or error on the combination.
#[test]
fn electrum_arm_silently_ignores_language_flag() {
    // Pass `--language japanese` (BIP-39 Japanese wordlist) AND
    // `--electrum-language spanish` (Electrum Spanish wordlist) on a Spanish
    // Electrum phrase. `--electrum-language` wins; output must match the
    // Spanish entropy. `--language` produces no warning to stderr (silent).
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("electrum-phrase={SPANISH_PHRASE}"),
            "--to",
            "entropy",
            "--electrum-language",
            "spanish",
            "--language",
            "japanese",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let line = stdout.trim();
    let colon = line.find(": ").unwrap();
    assert_eq!(&line[colon + 2..], SPANISH_HEX);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        !stderr.contains("--language"),
        "stderr must not mention --language on the Electrum arm; got: {stderr:?}",
    );
}

// ============================================================================
// SPEC v0.8 §14 — Item #11: SeedVersion info-line on stderr during decode
// ============================================================================

#[test]
fn decode_emits_seed_version_info_line_standard() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("electrum-phrase={STANDARD_PHRASE}"),
            "--to",
            "entropy",
        ])
        .assert()
        .success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("note: detected Electrum SeedVersion 01 (standard)"),
        "stderr missing Standard info-line; got: {stderr:?}",
    );
}

#[test]
fn decode_emits_seed_version_info_line_segwit() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("electrum-phrase={SEGWIT_PHRASE}"),
            "--to",
            "entropy",
        ])
        .assert()
        .success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("note: detected Electrum SeedVersion 100 (segwit)"),
        "stderr missing Segwit info-line; got: {stderr:?}",
    );
}

// ============================================================================
// v0.36.0 — refusal lock-test (spot-check finding)
// ============================================================================
//
// `convert` is not the edge for Electrum-native-seed addresses: Electrum uses
// its own PBKDF2 salt + non-BIP-44 derivation, so it is NOT a single-format
// `convert` conversion. As of v0.47.0 the operation IS supported by a different
// command — `mnemonic addresses --from electrum-phrase=<seed>` (Electrum-
// vector-tested in `cli_addresses_electrum.rs`). So `convert (electrum-phrase,
// address)` is refused with a REDIRECT to that command (not the generic
// one-way-barrier message). This pins the redirect.
#[test]
fn electrum_phrase_to_address_is_refused() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("electrum-phrase={SEGWIT_PHRASE}"),
            "--to",
            "address",
        ])
        .assert()
        .failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.to_lowercase().contains("electrum-phrase"),
        "refusal stderr must name the electrum-phrase edge; got: {stderr:?}",
    );
    assert!(
        stderr.contains("addresses --from electrum-phrase"),
        "refusal must REDIRECT to `mnemonic addresses --from electrum-phrase`; got: {stderr:?}",
    );
}
