//! ms mnem Phase 3 Step 5+6 — emit + preserve tests.
//!
//! (i)   Non-English phrase `bundle` → emitted ms1 is `mnem` (length ∈ {51,58,64,70,77}),
//!       and decodes back to the same phrase+language.
//! (ii)  A ja `mnem`-source `bundle --import-json` re-emits ja `mnem`.
//! (iii) Mixed-language multisig `--import-json` (ja-mnem ms1[0] + en-entr ms1[1])
//!       → re-emits ms1[0] mnem-ja + ms1[1] entr.
//! (iv)  English golden byte-identity: a known English phrase → a FIXED ms1 literal
//!       (byte-identical to v0.38.4).
//! (v)   inspect <ja mnem ms1> reports `payload_kind: Mnem` + `language: japanese`.

use assert_cmd::Command;
use bip39::Mnemonic;

// ─── fixtures ─────────────────────────────────────────────────────────────────

/// A checksum-valid 12-word English phrase (all-zeros entropy).
const ENGLISH_12: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

/// The well-known ms1 for the all-zeros 12-word English phrase (entr format, 50 chars).
/// This is the v0.38.4 GOLDEN — must be byte-identical post-Step-5.
const ENGLISH_MS1_GOLDEN: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";

/// Mnem ms1 lengths indexed by entropy byte-count (16,20,24,28,32 → 51,58,64,70,77).
const VALID_MNEM_STR_LENGTHS: &[usize] = &[51, 58, 64, 70, 77];

/// 16-byte entropy for ja cosigner (0x01 × 16).
const ENTROPY_JA_HEX: &str = "01010101010101010101010101010101";
/// Wire code for Japanese = 1.
const WIRE_JAPANESE: u8 = 1;
/// Wire code for English = 0 (used in preconditions via encode_entr, not encode_mnem).
#[allow(dead_code)]
const WIRE_ENGLISH: u8 = 0;

fn encode_mnem(entropy: &[u8], wire_lang: u8) -> String {
    ms_codec::encode(
        ms_codec::Tag::ENTR,
        &ms_codec::Payload::Mnem {
            language: wire_lang,
            entropy: entropy.to_vec(),
        },
    )
    .expect("ms_codec::encode mnem")
}

fn encode_entr(entropy: &[u8]) -> String {
    ms_codec::encode(
        ms_codec::Tag::ENTR,
        &ms_codec::Payload::Entr(entropy.to_vec()),
    )
    .expect("ms_codec::encode entr")
}

fn entropy_bytes(hex: &str) -> Vec<u8> {
    hex::decode(hex).expect("hex decode")
}

fn derive_xpub_bip84(entropy: &[u8], lang: bip39::Language) -> bitcoin::bip32::Xpub {
    use bitcoin::bip32::{DerivationPath, Xpriv, Xpub};
    use bitcoin::secp256k1::Secp256k1;
    use std::str::FromStr;
    let mnemonic = Mnemonic::from_entropy_in(lang, entropy).unwrap();
    let seed = mnemonic.to_seed("");
    let secp = Secp256k1::new();
    let master = Xpriv::new_master(bitcoin::NetworkKind::Main, &seed).unwrap();
    let path = DerivationPath::from_str("m/84'/0'/0'").unwrap();
    let xpriv = master.derive_priv(&secp, &path).unwrap();
    Xpub::from_priv(&secp, &xpriv)
}

fn derive_master_fp(entropy: &[u8], lang: bip39::Language) -> String {
    use bitcoin::bip32::Xpriv;
    use bitcoin::secp256k1::Secp256k1;
    let mnemonic = Mnemonic::from_entropy_in(lang, entropy).unwrap();
    let seed = mnemonic.to_seed("");
    let secp = Secp256k1::new();
    let master = Xpriv::new_master(bitcoin::NetworkKind::Main, &seed).unwrap();
    master.fingerprint(&secp).to_string().to_lowercase()
}

/// Build an import-wallet JSON envelope (array format) from a bundle JSON value.
/// Wraps the flat `bundle --json` output into the import-wallet envelope format:
/// `[{"schema_version": "1", "source_format": "bsms", "bundle": {...}}]`
fn wrap_bundle_as_envelope(bundle_v: &serde_json::Value) -> String {
    let envelope = serde_json::json!([{
        "schema_version": "1",
        "source_format": "bsms",
        "bundle": bundle_v
    }]);
    serde_json::to_string(&envelope).expect("serialize envelope")
}

// ─────────────────────────────────────────────────────────────────────────────
// Test (i): non-English phrase bundle → mnem ms1 + round-trip
// ─────────────────────────────────────────────────────────────────────────────

/// Generate a checksum-valid 12-word Japanese phrase from 0x01×16 entropy.
fn japanese_12_phrase() -> String {
    Mnemonic::from_entropy_in(bip39::Language::Japanese, &[0x01u8; 16])
        .unwrap()
        .to_string()
}

#[test]
fn japanese_phrase_bundle_emits_mnem_ms1() {
    let ja_phrase = japanese_12_phrase();

    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--slot",
            &format!("@0.phrase={ja_phrase}"),
            "--language",
            "japanese",
            "--template",
            "bip84",
            "--network",
            "mainnet",
            "--no-engraving-card",
            "--json",
        ])
        .assert()
        .success();

    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    let ms1_arr = v["ms1"].as_array().expect("ms1 array");
    let ms1_val = ms1_arr[0].as_str().unwrap_or("");

    // Must be mnem length (51 for 12-word / 16-byte entropy).
    assert!(
        VALID_MNEM_STR_LENGTHS.contains(&ms1_val.len()),
        "emitted ms1 must be mnem length, got len={} val={ms1_val:?}",
        ms1_val.len()
    );
    assert_eq!(ms1_val.len(), 51, "12-word → 51 chars mnem");

    // Decode the emitted ms1 → must decode to mnem, language=japanese.
    let (_tag, payload) = ms_codec::decode(ms1_val).expect("ms1 must decode");
    match payload {
        ms_codec::Payload::Mnem { language, .. } => {
            assert_eq!(
                language, WIRE_JAPANESE,
                "wire language must be japanese (1)"
            );
        }
        other => panic!("expected Mnem payload, got {other:?}"),
    }
}

#[test]
fn japanese_phrase_bundle_ms1_round_trips_to_phrase() {
    let ja_phrase = japanese_12_phrase();

    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--slot",
            &format!("@0.phrase={ja_phrase}"),
            "--language",
            "japanese",
            "--template",
            "bip84",
            "--network",
            "mainnet",
            "--no-engraving-card",
            "--json",
        ])
        .assert()
        .success();

    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    let ms1_val = v["ms1"].as_array().unwrap()[0].as_str().unwrap();

    // Convert the mnem ms1 back to a phrase — must recover the original ja phrase.
    let out2 = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("ms1={ms1_val}"),
            "--to",
            "phrase",
        ])
        .assert()
        .success();

    let stdout2 = String::from_utf8(out2.get_output().stdout.clone()).unwrap();
    // The phrase line starts with "phrase: ".
    let recovered = stdout2
        .lines()
        .find(|l| l.starts_with("phrase:"))
        .and_then(|l| l.split_once(':').map(|x| x.1))
        .map(|s| s.trim())
        .unwrap_or(stdout2.trim());
    assert_eq!(
        recovered, ja_phrase,
        "ms1 round-trip must recover the original Japanese phrase"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Test (ii): ja mnem-source bundle --import-json re-emits ja mnem
// ─────────────────────────────────────────────────────────────────────────────

/// Test that a bundle --import-json with a ja mnem ms1 re-emits the same ja mnem.
/// Uses a singlesig wpkh descriptor bundle with a known Japanese xpub.
#[test]
fn ja_mnem_source_import_json_re_emits_ja_mnem() {
    let entropy_ja = entropy_bytes(ENTROPY_JA_HEX);
    let ja_ms1 = encode_mnem(&entropy_ja, WIRE_JAPANESE);
    assert_eq!(ja_ms1.len(), 51, "ja mnem precondition");

    // Known singlesig bip84 xpub and fingerprint for 0x01×16 entropy, Japanese.
    let ja_xpub = "xpub6DWUhHbeZgL1RRZd7CvmzamidoxSaYLArMiqzVXUYsKCzUZAVRBvM2Xz5a5mcrUZCihgsyoq6Y9ExvV3RcgXLUT7T2w8QDBRfQPKT8WkK7A";
    let ja_fp = "7ae3af71";
    // Build singlesig wpkh descriptor.
    let desc_body = format!("wpkh([{ja_fp}/84'/0'/0']{ja_xpub}/<0;1>/*)");
    use miniscript::descriptor::checksum::Engine as CsEngine;
    let mut ce = CsEngine::new();
    ce.input(&desc_body).expect("ascii");
    let csum = ce.checksum();
    let descriptor = format!("{desc_body}#{csum}");

    // Build a watch-only bundle using the descriptor (so bundle JSON has descriptor != null).
    let watch_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--slot",
            &format!("@0.xpub={ja_xpub}"),
            "--slot",
            &format!("@0.fingerprint={ja_fp}"),
            "--descriptor",
            &descriptor,
            "--network",
            "mainnet",
            "--no-engraving-card",
            "--json",
        ])
        .assert()
        .success();
    let watch_stdout = String::from_utf8(watch_out.get_output().stdout.clone()).unwrap();
    let mut bundle_v: serde_json::Value = serde_json::from_str(&watch_stdout).expect("watch JSON");

    // Inject the ja mnem ms1 into the bundle.
    bundle_v["ms1"] = serde_json::json!([ja_ms1]);
    bundle_v["mode"] = serde_json::json!("full");

    // Wrap as import-wallet envelope and feed to bundle --import-json.
    let envelope_str = wrap_bundle_as_envelope(&bundle_v);

    let rebundle_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--import-json",
            "-",
            "--network",
            "mainnet",
            "--json",
        ])
        .write_stdin(envelope_str.as_bytes())
        .assert()
        .success();

    let rebundle_stdout = String::from_utf8(rebundle_out.get_output().stdout.clone()).unwrap();
    let rebundle_v: serde_json::Value =
        serde_json::from_str(&rebundle_stdout).expect("rebundle JSON");
    let reemitted_ms1 = rebundle_v["ms1"].as_array().unwrap()[0].as_str().unwrap();

    // The re-emitted ms1 must be the same mnem card as the original.
    assert_eq!(
        reemitted_ms1, ja_ms1,
        "re-emitted ms1 must be identical to the original ja mnem ms1"
    );
    assert_eq!(
        reemitted_ms1.len(),
        51,
        "re-emitted ms1 must be mnem (51 chars)"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Test (iii): mixed-language multisig --import-json → ms1[0] mnem-ja + ms1[1] entr
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn mixed_language_import_json_re_emits_mnem_ja_and_entr() {
    let entropy_ja = entropy_bytes(ENTROPY_JA_HEX);
    let entropy_en: Vec<u8> = vec![0x02u8; 16]; // cosigner 1: English

    let ja_ms1 = encode_mnem(&entropy_ja, WIRE_JAPANESE);
    let en_ms1 = encode_entr(&entropy_en);

    // Verify preconditions: ja is mnem (51), en is entr (50).
    assert_eq!(ja_ms1.len(), 51, "ja ms1 precondition");
    assert_eq!(en_ms1.len(), 50, "en ms1 precondition");

    // Derive xpubs and fingerprints at m/84'/0'/0' (bip84 path).
    let ja_xpub = derive_xpub_bip84(&entropy_ja, bip39::Language::Japanese);
    let en_xpub = derive_xpub_bip84(&entropy_en, bip39::Language::English);
    let ja_fp = derive_master_fp(&entropy_ja, bip39::Language::Japanese);
    let en_fp = derive_master_fp(&entropy_en, bip39::Language::English);

    // Build a 2-of-2 wsh(sortedmulti) descriptor with BIP-84 paths.
    let descriptor_body = format!(
        "wsh(sortedmulti(2,[{ja_fp}/84'/0'/0']{ja_xpub}/<0;1>/*,[{en_fp}/84'/0'/0']{en_xpub}/<0;1>/*))"
    );
    // Compute descriptor checksum.
    use miniscript::descriptor::checksum::Engine as CsEngine;
    let mut ce = CsEngine::new();
    ce.input(&descriptor_body).expect("ascii descriptor");
    let csum = ce.checksum();
    let descriptor = format!("{descriptor_body}#{csum}");

    // Build a watch-only 2-of-2 bundle from the descriptor.
    let watch_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--slot",
            &format!("@0.xpub={ja_xpub}"),
            "--slot",
            &format!("@0.fingerprint={ja_fp}"),
            "--slot",
            &format!("@1.xpub={en_xpub}"),
            "--slot",
            &format!("@1.fingerprint={en_fp}"),
            "--descriptor",
            &descriptor,
            "--network",
            "mainnet",
            "--no-engraving-card",
            "--json",
        ])
        .assert()
        .success();
    let watch_stdout = String::from_utf8(watch_out.get_output().stdout.clone()).unwrap();
    let mut bundle_v: serde_json::Value = serde_json::from_str(&watch_stdout).expect("watch JSON");

    // Inject ms1 cards into the bundle.ms1 array.
    bundle_v["ms1"] = serde_json::json!([ja_ms1, en_ms1]);
    bundle_v["mode"] = serde_json::json!("full");

    // Wrap as import-wallet envelope and feed to bundle --import-json.
    let envelope_str = wrap_bundle_as_envelope(&bundle_v);

    let rebundle_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--import-json",
            "-",
            "--network",
            "mainnet",
            "--json",
        ])
        .write_stdin(envelope_str.as_bytes())
        .assert()
        .success();

    let rebundle_stdout = String::from_utf8(rebundle_out.get_output().stdout.clone()).unwrap();
    let rebundle_v: serde_json::Value =
        serde_json::from_str(&rebundle_stdout).expect("rebundle JSON");
    let result_ms1 = rebundle_v["ms1"].as_array().expect("result ms1 array");

    let reemitted_0 = result_ms1[0].as_str().unwrap_or("");
    let reemitted_1 = result_ms1[1].as_str().unwrap_or("");

    assert_eq!(reemitted_0, ja_ms1, "ms1[0] must be the same ja mnem card");
    assert_eq!(reemitted_0.len(), 51, "ms1[0] must be mnem (51)");
    assert_eq!(reemitted_1, en_ms1, "ms1[1] must be the same en entr card");
    assert_eq!(reemitted_1.len(), 50, "ms1[1] must be entr (50)");
}

// ─────────────────────────────────────────────────────────────────────────────
// Test (iv): English golden byte-identity
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn english_phrase_convert_ms1_golden_byte_identity() {
    // The English phrase MUST produce the exact GOLDEN ms1 (v0.38.4 byte-identical).
    // This gates the back-compat invariant: the Entr branch is untouched by Step 5.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("phrase={ENGLISH_12}"),
            "--language",
            "english",
            "--to",
            "ms1",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let ms1_val = stdout
        .lines()
        .find(|l| l.trim_start().starts_with("ms1:"))
        .and_then(|l| l.split_once(':').map(|x| x.1))
        .map(|s| s.trim())
        .unwrap_or_else(|| stdout.trim());
    assert_eq!(
        ms1_val, ENGLISH_MS1_GOLDEN,
        "English ms1 must be byte-identical to the v0.38.4 golden\n\
         got:      {ms1_val}\n\
         expected: {ENGLISH_MS1_GOLDEN}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Test (v) Step 6: inspect <ja mnem ms1> reports mnem + language japanese
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn inspect_ja_mnem_ms1_reports_kind_and_language() {
    let entropy_ja = entropy_bytes(ENTROPY_JA_HEX);
    let ja_ms1 = encode_mnem(&entropy_ja, WIRE_JAPANESE);

    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["inspect", "--ms1", &ja_ms1])
        .assert()
        .success();

    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    // Must report payload_kind: Mnem.
    assert!(
        stdout.contains("payload_kind: Mnem"),
        "inspect must report payload_kind: Mnem\nGot: {stdout}"
    );
    // Must report language: japanese.
    assert!(
        stdout.contains("language: japanese"),
        "inspect must report language: japanese\nGot: {stdout}"
    );
}

#[test]
fn inspect_ja_mnem_ms1_json_includes_language_field() {
    let entropy_ja = entropy_bytes(ENTROPY_JA_HEX);
    let ja_ms1 = encode_mnem(&entropy_ja, WIRE_JAPANESE);

    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["inspect", "--ms1", &ja_ms1, "--json"])
        .assert()
        .success();

    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("inspect JSON");
    // JSON must include language field = "japanese".
    assert_eq!(
        v["language"].as_str(),
        Some("japanese"),
        "inspect JSON must include language: japanese\nGot: {v}"
    );
    assert_eq!(
        v["payload_kind"].as_str(),
        Some("Mnem"),
        "inspect JSON must report payload_kind: Mnem\nGot: {v}"
    );
}

#[test]
fn inspect_english_entr_ms1_no_language_field() {
    // English entr cards must NOT have a language field in inspect output.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["inspect", "--ms1", ENGLISH_MS1_GOLDEN])
        .assert()
        .success();

    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(
        !stdout.contains("language:"),
        "English entr ms1 must NOT have language field\nGot: {stdout}"
    );

    let out_json = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["inspect", "--ms1", ENGLISH_MS1_GOLDEN, "--json"])
        .assert()
        .success();
    let stdout_json = String::from_utf8(out_json.get_output().stdout.clone()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&stdout_json).expect("inspect JSON");
    assert!(
        v["language"].is_null(),
        "English entr inspect JSON must have null/absent language\nGot: {v}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Test (vi): C1 regression — descriptor-@N path must emit mnem for non-English
// ─────────────────────────────────────────────────────────────────────────────
//
// Regression guard for the "third emit path" bug found at end-of-cycle review:
// `bundle --descriptor "wpkh(@0)" --slot "@0.phrase=<ja>" --language japanese`
// was emitting a 50-char `entr` card (language-stripped) instead of a 51-char
// `mnem` card, because synthesize_descriptor had no run_language fallback.
// The fix adds `run_language: bip39::Language` to synthesize_descriptor and
// uses `c.language.unwrap_or(run_language)` — symmetric with synthesize_unified.

/// C1 regression: `--descriptor "wpkh(@0)"` + `--slot @0.phrase=<ja>` + `--language japanese`
/// MUST emit a 51-char mnem card, not a 50-char entr card.
///
/// The `--descriptor "wpkh(@0)"` form routes through `bundle_run_unified_descriptor`
/// (the @N placeholder path). Pre-fix this path had `language: None` on the cosigner
/// and `synthesize_descriptor` had no `run_language` fallback, so non-English slots
/// silently fell through to `Payload::Entr` (language-stripped). Post-fix it falls
/// back to `run_language` = Japanese → emits `Payload::Mnem`.
#[test]
fn descriptor_placeholder_japanese_phrase_emits_mnem_ms1() {
    let ja_phrase = japanese_12_phrase();

    // Use the bare @0 placeholder descriptor (non-canonical, no embedded xpub).
    // bundle_run_unified_descriptor handles this path: it derives the xpub from
    // the phrase, so the phrase is the only required slot input.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            "wpkh(@0)",
            "--slot",
            &format!("@0.phrase={ja_phrase}"),
            "--language",
            "japanese",
            "--network",
            "mainnet",
            "--no-engraving-card",
            "--json",
        ])
        .assert()
        .success();

    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    let ms1_arr = v["ms1"].as_array().expect("ms1 array");
    let ms1_val = ms1_arr[0].as_str().unwrap_or("");

    // CRITICAL: must be mnem length (51 for 12-word / 16-byte entropy).
    // Pre-fix this was 50 (entr). Post-fix it must be 51 (mnem).
    assert_eq!(
        ms1_val.len(),
        51,
        "descriptor-@N + ja phrase MUST emit mnem (51 chars), not entr (50);\n\
         got len={} val={ms1_val:?}",
        ms1_val.len()
    );
    assert!(
        VALID_MNEM_STR_LENGTHS.contains(&ms1_val.len()),
        "ms1 length {} is not a valid mnem length",
        ms1_val.len()
    );

    // Decode and assert Mnem payload with wire language = japanese (1).
    let (_tag, payload) = ms_codec::decode(ms1_val).expect("ms1 must decode");
    match payload {
        ms_codec::Payload::Mnem { language, .. } => {
            assert_eq!(
                language, WIRE_JAPANESE,
                "wire language must be japanese (1)"
            );
        }
        other => panic!(
            "descriptor-@N + ja phrase: expected Mnem payload, got {other:?}\n\
             ms1 = {ms1_val:?}"
        ),
    }
}

/// C1 regression round-trip: the mnem card emitted from `--descriptor "wpkh(@0)"` +
/// `--slot @0.phrase=<ja>` + `--language japanese` round-trips back to the original
/// Japanese phrase.
#[test]
fn descriptor_placeholder_japanese_phrase_ms1_round_trips() {
    let ja_phrase = japanese_12_phrase();

    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            "wpkh(@0)",
            "--slot",
            &format!("@0.phrase={ja_phrase}"),
            "--language",
            "japanese",
            "--network",
            "mainnet",
            "--no-engraving-card",
            "--json",
        ])
        .assert()
        .success();

    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    let ms1_val = v["ms1"].as_array().unwrap()[0].as_str().unwrap();

    // Convert back to phrase → must recover the original ja phrase.
    let out2 = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("ms1={ms1_val}"),
            "--to",
            "phrase",
        ])
        .assert()
        .success();

    let stdout2 = String::from_utf8(out2.get_output().stdout.clone()).unwrap();
    let recovered = stdout2
        .lines()
        .find(|l| l.starts_with("phrase:"))
        .and_then(|l| l.split_once(':').map(|x| x.1))
        .map(|s| s.trim())
        .unwrap_or(stdout2.trim());
    assert_eq!(
        recovered, ja_phrase,
        "descriptor-@N mnem ms1 must round-trip back to the original Japanese phrase"
    );
}

/// C1 regression advisory: for a non-English `--descriptor "wpkh(@0)"` bundle
/// (which now correctly emits mnem), the §6.3 language-loss advisory MUST be
/// suppressed (the card is self-describing — no language loss).
#[test]
fn descriptor_placeholder_japanese_phrase_advisory_suppressed() {
    let ja_phrase = japanese_12_phrase();

    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            "wpkh(@0)",
            "--slot",
            &format!("@0.phrase={ja_phrase}"),
            "--language",
            "japanese",
            "--network",
            "mainnet",
            "--no-engraving-card",
            "--json",
        ])
        .assert()
        .success();

    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();

    // The §6.3 advisory fires when a slot emits entr in a non-English run context.
    // Post-fix: the slot emits mnem (self-describing) → advisory must be suppressed.
    // The advisory text contains "Advisory" or "non-English" per non_english_seed_advisory().
    assert!(
        !stderr.contains("Advisory"),
        "descriptor-@N ja mnem: §6.3 advisory must be suppressed (card is self-describing)\n\
         got stderr: {stderr:?}"
    );
    assert!(
        !stderr.contains("non-English"),
        "descriptor-@N ja mnem: no non-English advisory expected in stderr\n\
         got stderr: {stderr:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Test (vii): C2 regression — import-json entr card MUST stay entr under --language
// ─────────────────────────────────────────────────────────────────────────────
//
// Regression guard for the C1-fold regression introduced in 80f78fc:
// `bundle --import-json <envelope-with-English-entr-ms1> --language japanese`
// was silently re-emitting the entr card as a 51-char Japanese `mnem` card,
// corrupting the wire language. An entr card is language-AGNOSTIC; re-labeling
// it as Japanese is a silent lie.
//
// Root cause: the Entr arm in bundle_run_from_import_json set slot.language=None,
// so synthesize_descriptor's `unwrap_or(run_language)` inherited --language
// (japanese) and emitted mnem instead of entr. Fix: set language=English
// explicitly for Entr wire cards.

/// C2 regression: `bundle --import-json` with an English `entr` ms1[0] + `--language japanese`
/// MUST re-emit the byte-identical 50-char `entr` card, NOT a 51-char `mnem` card.
///
/// This test is RED against pre-fix code (50-char entr → 51-char mnem) and GREEN after.
#[test]
fn import_json_entr_card_stays_entr_under_language_japanese() {
    // Use all-zeros 16-byte entropy → well-known English entr card.
    let entropy_en: Vec<u8> = vec![0x00u8; 16];
    let en_ms1 = encode_entr(&entropy_en);
    assert_eq!(
        en_ms1.len(),
        50,
        "English entr precondition: must be 50 chars"
    );

    // Decode and confirm it IS an Entr payload before injecting.
    let (_tag, payload_pre) = ms_codec::decode(&en_ms1).expect("en_ms1 must decode");
    assert!(
        matches!(payload_pre, ms_codec::Payload::Entr(_)),
        "precondition: en_ms1 must be Payload::Entr, got {payload_pre:?}"
    );

    // Derive the xpub and fingerprint from the all-zeros entropy at bip84 path.
    let xpub = derive_xpub_bip84(&entropy_en, bip39::Language::English);
    let fp = derive_master_fp(&entropy_en, bip39::Language::English);

    // Build singlesig wpkh descriptor.
    let desc_body = format!("wpkh([{fp}/84'/0'/0']{xpub}/<0;1>/*)");
    use miniscript::descriptor::checksum::Engine as CsEngine;
    let mut ce = CsEngine::new();
    ce.input(&desc_body).expect("ascii descriptor");
    let csum = ce.checksum();
    let descriptor = format!("{desc_body}#{csum}");

    // Build a watch-only bundle using the descriptor so bundle JSON has descriptor != null.
    let watch_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--slot",
            &format!("@0.xpub={xpub}"),
            "--slot",
            &format!("@0.fingerprint={fp}"),
            "--descriptor",
            &descriptor,
            "--network",
            "mainnet",
            "--no-engraving-card",
            "--json",
        ])
        .assert()
        .success();
    let watch_stdout = String::from_utf8(watch_out.get_output().stdout.clone()).unwrap();
    let mut bundle_v: serde_json::Value = serde_json::from_str(&watch_stdout).expect("watch JSON");

    // Inject the English entr ms1 into the bundle.
    bundle_v["ms1"] = serde_json::json!([en_ms1]);
    bundle_v["mode"] = serde_json::json!("full");

    // Wrap as import-wallet envelope.
    let envelope_str = wrap_bundle_as_envelope(&bundle_v);

    // Re-bundle with --language japanese — the entr card must stay entr.
    let rebundle_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--import-json",
            "-",
            "--language",
            "japanese",
            "--network",
            "mainnet",
            "--json",
        ])
        .write_stdin(envelope_str.as_bytes())
        .assert()
        .success();

    let rebundle_stdout = String::from_utf8(rebundle_out.get_output().stdout.clone()).unwrap();
    let rebundle_v: serde_json::Value =
        serde_json::from_str(&rebundle_stdout).expect("rebundle JSON");
    let reemitted_ms1 = rebundle_v["ms1"].as_array().unwrap()[0].as_str().unwrap();

    // CRITICAL: must be byte-identical to the original 50-char entr card.
    // Pre-fix this was a 51-char mnem card (language corrupted to japanese).
    assert_eq!(
        reemitted_ms1,
        en_ms1,
        "C2 regression: entr card under --language japanese MUST be byte-identical to original\n\
         got:      {reemitted_ms1} (len={})\n\
         expected: {en_ms1} (len=50)",
        reemitted_ms1.len()
    );
    assert_eq!(
        reemitted_ms1.len(),
        50,
        "C2 regression: re-emitted entr card must be 50 chars (entr), not 51 (mnem);\n\
         got {reemitted_ms1:?}"
    );

    // Decode and confirm Payload::Entr — not Mnem.
    let (_tag, payload) = ms_codec::decode(reemitted_ms1).expect("re-emitted ms1 must decode");
    assert!(
        matches!(payload, ms_codec::Payload::Entr(_)),
        "C2 regression: re-emitted card must be Payload::Entr, got {payload:?}\n\
         ms1 = {reemitted_ms1:?}"
    );
}
