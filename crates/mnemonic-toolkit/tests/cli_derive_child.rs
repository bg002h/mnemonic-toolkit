//! v0.7 Phase 6 — `mnemonic derive-child` integration tests.
//!
//! SPEC `design/SPEC_derive_child_v0_7.md` §6: 6 reference-vector cells
//! (BIP-39×2, HD-Seed WIF, XPRV, HEX, PWD BASE64, PWD BASE85) + 3 refusal
//! cells (unsupported app, bip39 length out-of-range, hd-seed length
//! not-applicable).
//!
//! All reference vectors come verbatim from BIP-85 §"Test Vectors"
//! (<https://github.com/bitcoin/bips/blob/master/bip-0085.mediawiki#test-vectors>),
//! all using the spec-provided master xprv.

use assert_cmd::Command;

const MASTER_XPRV: &str =
    "xprv9s21ZrQH143K2LBWUUQRFXhucrQqBpKdRRxNVq2zBqsx8HVqFk2uYo8kmbaLLHRdqtQpUm98uKfu3vca1LqdGhUtyoFnCNkfmXRyPXLjbKb";

/// SPEC §6 cell 1 — BIP-85 BIP-39 12-English-word reference vector.
/// Path m/83696968'/39'/0'/12'/0' → "girl mad pet galaxy egg matter matrix prison refuse sense ordinary nose".
#[test]
fn cell_1_bip39_12_words_reference_vector() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "derive-child",
            "--from",
            &format!("xprv={MASTER_XPRV}"),
            "--application",
            "bip39",
            "--length",
            "12",
            "--index",
            "0",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(
        stdout,
        "girl mad pet galaxy egg matter matrix prison refuse sense ordinary nose\n",
    );
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("warning: secret material on stdout"),
        "secret-on-stdout warning expected; got stderr: {stderr:?}"
    );
}

/// SPEC §6 cell 2 — BIP-85 BIP-39 18-English-word reference vector.
/// Path m/83696968'/39'/0'/18'/0'.
#[test]
fn cell_2_bip39_18_words_reference_vector() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "derive-child",
            "--from",
            &format!("xprv={MASTER_XPRV}"),
            "--application",
            "bip39",
            "--length",
            "18",
            "--index",
            "0",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(
        stdout,
        "near account window bike charge season chef number sketch tomorrow excuse sniff circle vital hockey outdoor supply token\n",
    );
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("warning: secret material on stdout"),
        "secret-on-stdout warning expected; got stderr: {stderr:?}"
    );
}

/// v0.8.0 cycle Phase 3 — BIP-85 vector 85.3 (24-word BIP-39
/// reference vector). Path m/83696968'/39'/0'/24'/0'. Closes the
/// v0.7.1 SPEC §5 carry-over (BIP-85 7/9 → 8/9; only 85.9 DICE
/// remains as a refusal cell).
///
/// Cycle SPEC: `mnemonic-toolkit/design/SPEC_test_vector_audit_v0_8_0.md` §2.
#[test]
fn cell_2b_bip39_24_words_reference_vector() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "derive-child",
            "--from",
            &format!("xprv={MASTER_XPRV}"),
            "--application",
            "bip39",
            "--length",
            "24",
            "--index",
            "0",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(
        stdout,
        "puppy ocean match cereal symbol another shed magic wrap hammer bulb intact gadget divorce twin tonight reason outdoor destroy simple truth cigar social volcano\n",
    );
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("warning: secret material on stdout"),
        "secret-on-stdout warning expected; got stderr: {stderr:?}"
    );
}

/// SPEC §6 cell 3 — BIP-85 HD-Seed WIF reference vector.
/// Path m/83696968'/2'/0' → WIF Kzyv4uF39d4Jrw2W7UryTHwZr1zQVNk4dAFyqE6BuMrMh1Za7uhp.
/// `--length` is required at clap level for SPEC §2 grammar-uniformity but
/// ignored at validation when `0` (the sentinel); any non-zero value
/// triggers the SPEC §7 not-applicable refusal (cell 9).
#[test]
fn cell_3_hd_seed_wif_reference_vector() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "derive-child",
            "--from",
            &format!("xprv={MASTER_XPRV}"),
            "--application",
            "hd-seed",
            "--length",
            "0",
            "--index",
            "0",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(
        stdout,
        "Kzyv4uF39d4Jrw2W7UryTHwZr1zQVNk4dAFyqE6BuMrMh1Za7uhp\n",
    );
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("warning: secret material on stdout"),
        "secret-on-stdout warning expected; got stderr: {stderr:?}"
    );
}

/// SPEC §6 cell 4 — BIP-85 XPRV reference vector.
/// Path m/83696968'/32'/0'. `--length 0` sentinel per SPEC §2 (see cell 3).
#[test]
fn cell_4_xprv_reference_vector() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "derive-child",
            "--from",
            &format!("xprv={MASTER_XPRV}"),
            "--application",
            "xprv",
            "--length",
            "0",
            "--index",
            "0",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(
        stdout,
        "xprv9s21ZrQH143K2srSbCSg4m4kLvPMzcWydgmKEnMmoZUurYuBuYG46c6P71UGXMzmriLzCCBvKQWBUv3vPB3m1SATMhp3uEjXHJ42jFg7myX\n",
    );
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("warning: secret material on stdout"),
        "secret-on-stdout warning expected; got stderr: {stderr:?}"
    );
}

/// SPEC §6 cell 5 — BIP-85 HEX reference vector.
/// Path m/83696968'/128169'/64'/0' → 64 hex bytes per spec.
#[test]
fn cell_5_hex_reference_vector() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "derive-child",
            "--from",
            &format!("xprv={MASTER_XPRV}"),
            "--application",
            "hex",
            "--length",
            "64",
            "--index",
            "0",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(
        stdout,
        "492db4698cf3b73a5a24998aa3e9d7fa96275d85724a91e71aa2d645442f878555d078fd1f1f67e368976f04137b1f7a0d19232136ca50c44614af72b5582a5c\n",
    );
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("warning: secret material on stdout"),
        "secret-on-stdout warning expected; got stderr: {stderr:?}"
    );
}

/// SPEC §6 cell 6a — BIP-85 PWD BASE64 reference vector.
/// Path m/83696968'/707764'/21'/0' → "dKLoepugzdVJvdL56ogNV".
#[test]
fn cell_6a_pwd_base64_reference_vector() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "derive-child",
            "--from",
            &format!("xprv={MASTER_XPRV}"),
            "--application",
            "password-base64",
            "--length",
            "21",
            "--index",
            "0",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(stdout, "dKLoepugzdVJvdL56ogNV\n");
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("warning: secret material on stdout"),
        "secret-on-stdout warning expected; got stderr: {stderr:?}"
    );
}

/// SPEC §6 cell 6b — BIP-85 PWD BASE85 reference vector.
/// Path m/83696968'/707785'/12'/0' → "_s`{TW89)i4`".
#[test]
fn cell_6b_pwd_base85_reference_vector() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "derive-child",
            "--from",
            &format!("xprv={MASTER_XPRV}"),
            "--application",
            "password-base85",
            "--length",
            "12",
            "--index",
            "0",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(stdout, "_s`{TW89)i4`\n");
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("warning: secret material on stdout"),
        "secret-on-stdout warning expected; got stderr: {stderr:?}"
    );
}

/// SPEC v0.8 §7 — refusal: --application rsa / rsa-gpg are still out-of-scope
/// (Phase 6 RSA-crate security spike: RUSTSEC-2023-0071 unpatched). v0.8 lifts
/// `dice` to in-scope; the rsa-family tokens stay in the deferred list.
#[test]
fn cell_7_unsupported_application_rsa_refusal() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "derive-child",
            "--from",
            &format!("xprv={MASTER_XPRV}"),
            "--application",
            "rsa",
            "--length",
            "32",
            "--index",
            "0",
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("--application <rsa|rsa-gpg> is out-of-scope")
            && stderr.contains("RUSTSEC-2023-0071"),
        "stderr missing v0.8 RSA refusal text: {stderr:?}",
    );
}

/// SPEC §7 — refusal: --length 16 invalid for bip39 (valid is 12|15|18|21|24).
#[test]
fn cell_8_bip39_length_out_of_range_refusal() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "derive-child",
            "--from",
            &format!("xprv={MASTER_XPRV}"),
            "--application",
            "bip39",
            "--length",
            "16",
            "--index",
            "0",
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert_eq!(
        stderr.trim(),
        "error: --length 16 out of range for --application bip39 (valid: 12 | 15 | 18 | 21 | 24 words)",
    );
}

/// SPEC §7 — refusal: --length not applicable for hd-seed (output is fixed-size).
#[test]
fn cell_9_hd_seed_length_not_applicable_refusal() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "derive-child",
            "--from",
            &format!("xprv={MASTER_XPRV}"),
            "--application",
            "hd-seed",
            "--length",
            "32",
            "--index",
            "0",
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert_eq!(
        stderr.trim(),
        "error: --length not applicable for --application <hd-seed|xprv> (output is fixed-size)",
    );
}

/// SPEC §7 — refusal: --length not applicable for xprv (output is fixed-size).
/// Mirrors cell 9 for the xprv branch of the not-applicable family.
#[test]
fn cell_9b_xprv_length_not_applicable_refusal() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "derive-child",
            "--from",
            &format!("xprv={MASTER_XPRV}"),
            "--application",
            "xprv",
            "--length",
            "32",
            "--index",
            "0",
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert_eq!(
        stderr.trim(),
        "error: --length not applicable for --application <hd-seed|xprv> (output is fixed-size)",
    );
}

// ============================================================================
// SPEC v0.8 §3 — Item #5: phrase-master input
// ============================================================================

/// Trezor's canonical zero-entropy 12-word mnemonic. Self-consistency test
/// below derives the corresponding master xprv in-test and uses it to cross-
/// validate `--from phrase=` against `--from xprv=`.
const ZERO_PHRASE: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

/// Compute the mainnet master xprv for `phrase` + `passphrase` extension,
/// matching what `derive-child --from phrase=` does internally.
fn master_xprv_for(phrase: &str, passphrase: &str) -> String {
    use bip39::{Language, Mnemonic};
    use bitcoin::bip32::Xpriv;
    use bitcoin::NetworkKind;
    let mnemonic = Mnemonic::parse_in(Language::English, phrase).unwrap();
    let seed = mnemonic.to_seed(passphrase);
    Xpriv::new_master(NetworkKind::Main, &seed).unwrap().to_string()
}

#[test]
fn phrase_master_matches_xprv_master_bip39_12_words() {
    // Compute the xprv from the phrase, then derive from BOTH `xprv=` and
    // `phrase=` and assert outputs match. Cross-validates that derive-child
    // performs the phrase → master conversion identically to the reference.
    let derived_xprv = master_xprv_for(ZERO_PHRASE, "");

    let from_xprv = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "derive-child",
            "--from",
            &format!("xprv={derived_xprv}"),
            "--application",
            "bip39",
            "--length",
            "12",
            "--index",
            "0",
        ])
        .assert()
        .success();
    let from_phrase = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "derive-child",
            "--from",
            &format!("phrase={ZERO_PHRASE}"),
            "--application",
            "bip39",
            "--length",
            "12",
            "--index",
            "0",
        ])
        .assert()
        .success();
    assert_eq!(
        String::from_utf8(from_xprv.get_output().stdout.clone()).unwrap(),
        String::from_utf8(from_phrase.get_output().stdout.clone()).unwrap(),
    );
}

#[test]
fn phrase_master_with_passphrase_diverges_from_empty_extension() {
    // Different BIP-39 extensions ⇒ different master xprvs ⇒ different outputs.
    let no_pass = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "derive-child",
            "--from",
            &format!("phrase={ZERO_PHRASE}"),
            "--application",
            "bip39",
            "--length",
            "12",
            "--index",
            "0",
        ])
        .assert()
        .success();
    let with_pass = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "derive-child",
            "--from",
            &format!("phrase={ZERO_PHRASE}"),
            "--application",
            "bip39",
            "--length",
            "12",
            "--index",
            "0",
            "--passphrase",
            "extension",
        ])
        .assert()
        .success();
    assert_ne!(
        String::from_utf8(no_pass.get_output().stdout.clone()).unwrap(),
        String::from_utf8(with_pass.get_output().stdout.clone()).unwrap(),
    );
}

// ============================================================================
// SPEC v0.8 §4 — Item #6: BIP-85 language code dispatch
// ============================================================================

/// SPEC v0.8 §4 — `--language japanese` selects BIP-85 language code 1 +
/// the Japanese wordlist. Output should be a Japanese-wordlist phrase
/// distinct from the English default for the same master + index.
#[test]
fn bip39_japanese_diverges_from_english() {
    let english = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "derive-child",
            "--from",
            &format!("xprv={MASTER_XPRV}"),
            "--application",
            "bip39",
            "--length",
            "12",
            "--index",
            "0",
        ])
        .assert()
        .success();
    let japanese = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "derive-child",
            "--from",
            &format!("xprv={MASTER_XPRV}"),
            "--application",
            "bip39",
            "--length",
            "12",
            "--index",
            "0",
            "--language",
            "japanese",
        ])
        .assert()
        .success();
    let en_out = String::from_utf8(english.get_output().stdout.clone()).unwrap();
    let ja_out = String::from_utf8(japanese.get_output().stdout.clone()).unwrap();
    assert_ne!(en_out, ja_out);
    // Sanity check that Japanese output is in fact non-ASCII.
    assert!(!ja_out.is_ascii(), "expected non-ASCII Japanese output; got {ja_out:?}");
}

#[test]
fn bip39_portuguese_refused_no_bip85_code() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "derive-child",
            "--from",
            &format!("xprv={MASTER_XPRV}"),
            "--application",
            "bip39",
            "--length",
            "12",
            "--index",
            "0",
            "--language",
            "portuguese",
        ])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("portuguese is not assigned a BIP-85 path code"),
        "stderr did not mention BIP-85 code refusal: {stderr:?}",
    );
}

// ============================================================================
// SPEC v0.8 §4 — Item #7: testnet network emission
// ============================================================================

/// SPEC v0.8 §4 — `--network testnet` emits hd-seed WIF with `c…` prefix
/// (testnet compressed) instead of the mainnet `K…`/`L…` prefix.
#[test]
fn hd_seed_wif_testnet_prefix() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "derive-child",
            "--from",
            &format!("xprv={MASTER_XPRV}"),
            "--application",
            "hd-seed",
            "--length",
            "0",
            "--index",
            "0",
            "--network",
            "testnet",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let wif = stdout.trim();
    assert!(
        wif.starts_with('c'),
        "testnet WIF must start with 'c'; got {wif:?}",
    );
}

#[test]
fn xprv_child_testnet_prefix() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "derive-child",
            "--from",
            &format!("xprv={MASTER_XPRV}"),
            "--application",
            "xprv",
            "--length",
            "0",
            "--index",
            "0",
            "--network",
            "testnet",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let xprv = stdout.trim();
    assert!(
        xprv.starts_with("tprv"),
        "testnet xprv must start with 'tprv'; got {xprv:?}",
    );
}

// ============================================================================
// SPEC v0.8 §3 — Item #8: stdin master xprv
// ============================================================================

/// SPEC v0.8 §3 — `--from xprv=-` reads the master from stdin.
#[test]
fn xprv_from_stdin_matches_argv_master() {
    let from_argv = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "derive-child",
            "--from",
            &format!("xprv={MASTER_XPRV}"),
            "--application",
            "bip39",
            "--length",
            "12",
            "--index",
            "0",
        ])
        .assert()
        .success();
    let from_stdin = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "derive-child",
            "--from",
            "xprv=-",
            "--application",
            "bip39",
            "--length",
            "12",
            "--index",
            "0",
        ])
        .write_stdin(MASTER_XPRV.as_bytes())
        .assert()
        .success();
    assert_eq!(
        String::from_utf8(from_argv.get_output().stdout.clone()).unwrap(),
        String::from_utf8(from_stdin.get_output().stdout.clone()).unwrap(),
    );
}

// ============================================================================
// SPEC v0.8 §4 — Item #7 (DICE only; RSA + RSA-GPG deferred per Phase 6 spike)
// ============================================================================

/// SPEC v0.8 §4 + BIP-85 v1.3.0 §"DICE" — d6 reference vector at index 0.
/// Path m/83696968'/89101'/6'/10'/0' → "1,0,0,2,0,1,5,5,2,4".
#[test]
fn dice_d6_10_rolls_reference_vector() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "derive-child",
            "--from",
            &format!("xprv={MASTER_XPRV}"),
            "--application",
            "dice",
            "--length",
            "10",
            "--index",
            "0",
            "--dice-sides",
            "6",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(stdout, "1,0,0,2,0,1,5,5,2,4\n");
}

/// SPEC v0.8 §4 — `--application dice` requires `--dice-sides`.
#[test]
fn dice_missing_dice_sides_refused() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "derive-child",
            "--from",
            &format!("xprv={MASTER_XPRV}"),
            "--application",
            "dice",
            "--length",
            "10",
            "--index",
            "0",
        ])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("--application dice requires --dice-sides"),
        "stderr missing dice-sides refusal: {stderr:?}",
    );
}
