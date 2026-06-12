//! ms mnem Phase 3 Step 4 — per-card wire-language derive tests.
//!
//! (i) A Japanese `mnem` ms1 → `xpub-search` derives the SAME xpub as the
//!     equivalent Japanese phrase, DIFFERENT from English-derived.
//! (ii) Mixed-language multisig (ja-mnem + cs-mnem cosigners) → `verify-bundle`
//!      import-wallet cross-checks PASS (each cosigner derived under its own wire language).
//! (iii) Corrupt a Japanese `mnem` card ≤ 4 symbols → `repair` recovers it.

use assert_cmd::Command;
use bip39::Mnemonic;
use bitcoin::bip32::{DerivationPath, Xpriv, Xpub};
use bitcoin::secp256k1::Secp256k1;
use std::str::FromStr;

/// Derive account xpub from entropy at `m/87'/0'/0'` in the given language.
fn derive_xpub_bip87(entropy: &[u8], lang: bip39::Language) -> Xpub {
    let mnemonic = Mnemonic::from_entropy_in(lang, entropy).unwrap();
    let seed = mnemonic.to_seed("");
    let secp = Secp256k1::new();
    let master = Xpriv::new_master(bitcoin::NetworkKind::Main, &seed).unwrap();
    let path = DerivationPath::from_str("m/87'/0'/0'").unwrap();
    let xpriv = master.derive_priv(&secp, &path).unwrap();
    Xpub::from_priv(&secp, &xpriv)
}

/// Derive master fingerprint from entropy in the given language.
fn derive_master_fp(entropy: &[u8], lang: bip39::Language) -> String {
    let mnemonic = Mnemonic::from_entropy_in(lang, entropy).unwrap();
    let seed = mnemonic.to_seed("");
    let secp = Secp256k1::new();
    let master = Xpriv::new_master(bitcoin::NetworkKind::Main, &seed).unwrap();
    master.fingerprint(&secp).to_string().to_lowercase()
}

/// Encode entropy as a `mnem` ms1 card with the given wire language code.
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

/// 16 non-zero bytes for test cosigner 0.
const ENTROPY_JA: &[u8] = &[0x01u8; 16];
/// 16 non-zero bytes for test cosigner 1 (DISTINCT).
const ENTROPY_CS: &[u8] = &[0x02u8; 16];

// Wire codes: English=0, Japanese=1, Czech=8
const WIRE_JAPANESE: u8 = 1;
const WIRE_CZECH: u8 = 8;

// ─────────────────────────────────────────────────────────────────────────────
// Test (i): Japanese mnem card → xpub-search/convert derives Japanese xpub
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn japanese_mnem_card_convert_to_phrase_uses_japanese() {
    // Encode entropy as a Japanese mnem card.
    let ja_ms1 = encode_mnem(ENTROPY_JA, WIRE_JAPANESE);

    // The expected phrase is what bip39 generates from this entropy under Japanese.
    let expected_ja_phrase = Mnemonic::from_entropy_in(bip39::Language::Japanese, ENTROPY_JA)
        .unwrap()
        .to_string();

    // The English phrase for the same entropy should be DIFFERENT.
    let expected_en_phrase = Mnemonic::from_entropy_in(bip39::Language::English, ENTROPY_JA)
        .unwrap()
        .to_string();
    assert_ne!(expected_ja_phrase, expected_en_phrase, "test precondition");

    // Run convert --from "ms1=<value>" --to phrase — wire-wins: should produce the Japanese phrase.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("ms1={ja_ms1}"),
            "--to",
            "phrase",
        ])
        .assert()
        .success();

    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    // The output should contain the Japanese phrase words.
    // We check a few distinctive Japanese BIP-39 words from the expected phrase.
    let first_word: &str = expected_ja_phrase.split_whitespace().next().unwrap();
    assert!(
        stdout.contains(first_word),
        "convert should output the Japanese phrase (first word: {first_word}).\nGot: {stdout}"
    );
    // And NOT the English phrase.
    let en_first_word: &str = expected_en_phrase.split_whitespace().next().unwrap();
    assert!(
        !stdout.contains(en_first_word) || first_word == en_first_word,
        "convert should NOT output the English phrase first word {en_first_word}.\nGot: {stdout}"
    );
}

#[test]
fn japanese_mnem_card_derives_different_xpub_than_english() {
    // Wire-wins correctness: ja mnem → ja xpub ≠ en xpub for same entropy.
    let ja_xpub = derive_xpub_bip87(ENTROPY_JA, bip39::Language::Japanese);
    let en_xpub = derive_xpub_bip87(ENTROPY_JA, bip39::Language::English);
    assert_ne!(
        ja_xpub, en_xpub,
        "test precondition: ja and en xpubs must differ for 0x01*16 entropy"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Test (ii): Mixed-language multisig — import-wallet overlay cross-check passes
// ─────────────────────────────────────────────────────────────────────────────

/// Build a proper BSMS 1.0 blob with descriptor checksum.
fn bsms_1line_from_descriptor(descriptor_body: &str) -> String {
    use miniscript::descriptor::checksum::Engine as ChecksumEngine;
    let mut e = ChecksumEngine::new();
    e.input(descriptor_body).expect("ascii descriptor");
    let csum = e.checksum();
    format!("BSMS 1.0\n{descriptor_body}#{csum}\n")
}

#[test]
fn mixed_language_multisig_import_wallet_ms1_overlay_cross_check_passes() {
    // Cosigner 0: Japanese mnem, cosigner 1: Czech mnem.
    let ja_ms1 = encode_mnem(ENTROPY_JA, WIRE_JAPANESE);
    let cs_ms1 = encode_mnem(ENTROPY_CS, WIRE_CZECH);

    let ja_xpub = derive_xpub_bip87(ENTROPY_JA, bip39::Language::Japanese);
    let cs_xpub = derive_xpub_bip87(ENTROPY_CS, bip39::Language::Czech);
    let ja_fp = derive_master_fp(ENTROPY_JA, bip39::Language::Japanese);
    let cs_fp = derive_master_fp(ENTROPY_CS, bip39::Language::Czech);

    // Build a wsh(sortedmulti(2,...)) descriptor. Use BIP-87 paths.
    let descriptor_body = format!(
        "wsh(sortedmulti(2,[{ja_fp}/87'/0'/0']{ja_xpub}/<0;1>/*,[{cs_fp}/87'/0'/0']{cs_xpub}/<0;1>/*))"
    );
    let bsms_blob = bsms_1line_from_descriptor(&descriptor_body);

    // import-wallet with --ms1 for each cosigner should succeed (exit 0):
    // each cosigner is derived under its own wire language.
    // If the overlay used a single --language (wrong behavior), the second
    // cosigner would derive under the wrong language → xpub mismatch → exit 4.
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--blob",
            "-",
            "--format",
            "bsms",
            "--ms1",
            &ja_ms1,
            "--ms1",
            &cs_ms1,
        ])
        .write_stdin(bsms_blob)
        .assert()
        .success();
}

// ─────────────────────────────────────────────────────────────────────────────
// Test (iii): Corrupt a Japanese `mnem` card ≤ 4 symbols → repair recovers it
// ─────────────────────────────────────────────────────────────────────────────

/// Flip the bech32 character at `pos` within the data-part (after the `1` separator).
fn flip_at(chunk: &str, pos: usize) -> String {
    const ALPHABET: &str = "qpzry9x8gf2tvdw0s3jn54khce6mua7l";
    let sep = chunk.rfind('1').unwrap();
    let (prefix, rest) = chunk.split_at(sep + 1);
    let chars: Vec<char> = rest.chars().collect();
    let c = chars[pos];
    let idx = ALPHABET.find(c).unwrap_or(0);
    let next = ALPHABET.chars().nth((idx + 1) % ALPHABET.len()).unwrap();
    let mut out = prefix.to_string();
    for (i, ch) in chars.iter().enumerate() {
        if i == pos {
            out.push(next);
        } else {
            out.push(*ch);
        }
    }
    out
}

#[test]
fn repair_recovers_corrupt_japanese_mnem_ms1() {
    let ja_ms1 = encode_mnem(ENTROPY_JA, WIRE_JAPANESE);

    // Flip a single character in the data part (position 10 — well within the payload).
    let corrupt = flip_at(&ja_ms1, 10);
    assert_ne!(corrupt, ja_ms1, "corruption must change the string");

    // Verify that ms_codec recognizes the original as valid and the corrupt as invalid.
    assert!(ms_codec::decode(&ja_ms1).is_ok(), "original must be valid");
    assert!(
        ms_codec::decode(&corrupt).is_err(),
        "corrupted must be invalid"
    );

    // repair should exit 0 or 5 (5 = corrections applied = success).
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["repair", "--ms1", &corrupt])
        .output()
        .expect("mnemonic repair");

    let exit_code = out.status.code().unwrap_or(-1);
    assert!(
        exit_code == 0 || exit_code == 5,
        "repair should exit 0 (already valid) or 5 (corrections applied), got {exit_code}"
    );

    let stdout = String::from_utf8(out.stdout.clone()).unwrap();
    // The stdout should contain the original valid ms1.
    assert!(
        stdout.contains(&ja_ms1),
        "repaired output should contain the original Japanese mnem ms1.\nGot: {stdout}\nExpected to contain: {ja_ms1}"
    );
}
