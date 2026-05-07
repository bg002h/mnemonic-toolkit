//! v0.7 Phase 1 — `mnemonic convert` BIP-38 encrypt/decrypt edges.
//! Reference vectors: BIP-38 spec §"Test vectors", non-EC-multiplied form.
//! <https://github.com/bitcoin/bips/blob/master/bip-0038.mediawiki>

use assert_cmd::Command;

// --- BIP-38 spec test vectors (non-EC-multiplied) ---
//
// V1: no compression, passphrase "TestingOneTwoThree"
const V1_PASS: &str = "TestingOneTwoThree";
const V1_WIF: &str = "5KN7MzqK5wt2TP1fQCYyHBtDrXdJuXbUzm4A9rKAteGu3Qi5CVR";
const V1_BIP38: &str = "6PRVWUbkzzsbcVac2qwfssoUJAN1Xhrg6bNk8J7Nzm5H7kxEbn2Nh2ZoGg";

// V2: no compression, passphrase "Satoshi"
const V2_PASS: &str = "Satoshi";
const V2_WIF: &str = "5HtasZ6ofTHP6HCwTqTkLDuLQisYPah7aUnSKfC7h4hMUVw2gi5";
const V2_BIP38: &str = "6PRNFFkZc2NZ6dJqFfhRoFNMR9Lnyj7dYGrzdgXXVMXcxoKTePPX1dWByq";

// V3: compression, passphrase "TestingOneTwoThree"
const V3_PASS: &str = "TestingOneTwoThree";
const V3_WIF: &str = "L44B5gGEpqEDRS9vVPz7QT35jcBG2r3CZwSwQ4fCewXAhAhqGVpP";
const V3_BIP38: &str = "6PYNKZ1EAgYgmQfmNVamxyXVWHzK5s6DGhwP4J5o44cvXdoY7sRzhtpUeo";

/// Helper: extract the value from `<node>: <value>\n` stdout.
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
// (Wif, Bip38) — encrypt
// ============================================================================

#[test]
fn encrypt_wif_to_bip38_vector1_no_compression() {
    let out = convert_value(&[
        "convert",
        "--from",
        &format!("wif={V1_WIF}"),
        "--to",
        "bip38",
        "--passphrase",
        V1_PASS,
    ]);
    assert_eq!(out, V1_BIP38);
}

#[test]
fn encrypt_wif_to_bip38_vector2_no_compression() {
    let out = convert_value(&[
        "convert",
        "--from",
        &format!("wif={V2_WIF}"),
        "--to",
        "bip38",
        "--passphrase",
        V2_PASS,
    ]);
    assert_eq!(out, V2_BIP38);
}

#[test]
fn encrypt_wif_to_bip38_vector3_compressed() {
    let out = convert_value(&[
        "convert",
        "--from",
        &format!("wif={V3_WIF}"),
        "--to",
        "bip38",
        "--passphrase",
        V3_PASS,
    ]);
    assert_eq!(out, V3_BIP38);
}

// ============================================================================
// (Bip38, Wif) — decrypt
// ============================================================================

#[test]
fn decrypt_bip38_to_wif_vector1_no_compression() {
    let out = convert_value(&[
        "convert",
        "--from",
        &format!("bip38={V1_BIP38}"),
        "--to",
        "wif",
        "--passphrase",
        V1_PASS,
    ]);
    assert_eq!(out, V1_WIF);
}

#[test]
fn decrypt_bip38_to_wif_vector2_no_compression() {
    let out = convert_value(&[
        "convert",
        "--from",
        &format!("bip38={V2_BIP38}"),
        "--to",
        "wif",
        "--passphrase",
        V2_PASS,
    ]);
    assert_eq!(out, V2_WIF);
}

#[test]
fn decrypt_bip38_to_wif_vector3_compressed() {
    let out = convert_value(&[
        "convert",
        "--from",
        &format!("bip38={V3_BIP38}"),
        "--to",
        "wif",
        "--passphrase",
        V3_PASS,
    ]);
    assert_eq!(out, V3_WIF);
}

// ============================================================================
// Refusals — SPEC v0.7 §3.d
// ============================================================================

#[test]
fn refusal_wif_to_bip38_no_passphrase() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("wif={V1_WIF}"),
            "--to",
            "bip38",
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert_eq!(
        stderr,
        "error: --from <bip38|wif> --to <wif|bip38> requires --passphrase (BIP-38 encryption is passphrase-driven).\n"
    );
}

#[test]
fn refusal_bip38_to_wif_no_passphrase() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("bip38={V1_BIP38}"),
            "--to",
            "wif",
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert_eq!(
        stderr,
        "error: --from <bip38|wif> --to <wif|bip38> requires --passphrase (BIP-38 encryption is passphrase-driven).\n"
    );
}

#[test]
fn refusal_bip38_to_wif_wrong_passphrase() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("bip38={V1_BIP38}"),
            "--to",
            "wif",
            "--passphrase",
            "WRONG",
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert_eq!(
        stderr,
        "error: BIP-38 decryption failed: passphrase does not match the encrypted key (per BIP-38 §\"Decryption\" address-hash check).\n"
    );
}

#[test]
fn refusal_bip38_to_bip38_identity() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("bip38={V1_BIP38}"),
            "--to",
            "bip38",
            "--passphrase",
            V1_PASS,
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert_eq!(
        stderr,
        "error: --from bip38 --to bip38 is an identity pivot. To re-encrypt with a different passphrase, decrypt to wif then re-encrypt.\n"
    );
}

// ============================================================================
// Composite phrase → bip38 (via wif intermediate)
// ============================================================================

#[test]
fn composite_phrase_to_bip38_via_wif() {
    // Trezor zero-entropy 12-word phrase, BIP-84 derivation path m/84'/0'/0'/0/0,
    // mainnet. The same phrase + path drives a deterministic WIF; we verify
    // that the BIP-38 decrypt of the emitted ciphertext recovers that WIF.
    const PHRASE: &str =
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    const BIP38_PASS: &str = "encrypt-pass-12345";

    let bip38_out = convert_value(&[
        "convert",
        "--from",
        &format!("phrase={PHRASE}"),
        "--to",
        "bip38",
        "--path",
        "m/84'/0'/0'/0/0",
        "--passphrase",
        BIP38_PASS,
    ]);
    assert!(bip38_out.starts_with("6P"), "BIP-38 ciphertext must start with 6P; got {bip38_out:?}");

    // Verify by decrypting back; the recovered WIF must match the direct
    // phrase → wif path with the same passphrase.
    let direct_wif = convert_value(&[
        "convert",
        "--from",
        &format!("phrase={PHRASE}"),
        "--to",
        "wif",
        "--path",
        "m/84'/0'/0'/0/0",
        "--passphrase",
        BIP38_PASS,
    ]);
    let recovered_wif = convert_value(&[
        "convert",
        "--from",
        &format!("bip38={bip38_out}"),
        "--to",
        "wif",
        "--passphrase",
        BIP38_PASS,
    ]);
    assert_eq!(recovered_wif, direct_wif);
}
