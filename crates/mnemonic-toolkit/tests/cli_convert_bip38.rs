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

// V4 (BIP-38 §"Test vectors" §"Encryption when EC multiply flag is not used",
// vector 4): TestingOneTwoThree, compressed.
const V4_PASS: &str = "TestingOneTwoThree";
const V4_WIF: &str = "L44B5gGEpqEDRS9vVPz7QT35jcBG2r3CZwSwQ4fCewXAhAhqGVpP";
const V4_BIP38: &str = "6PYNKZ1EAgYgmQfmNVamxyXVWHzK5s6DGhwP4J5o44cvXdoY7sRzhtpUeo";

// SPEC_V3 (BIP-38 §"Test vectors" vector 3): no compression, Unicode-NFC
// passphrase encoded as U+03D2 + U+0301 + U+0000 + U+10400 + U+1F4A9.
// (The spec includes a U+0000 NULL between U+0301 and U+10400; after NFC
// normalization the byte sequence is 0xcf9300f0909080f09f92a9.)
// Source: <https://github.com/bitcoin/bips/blob/master/bip-0038.mediawiki>.
const SPEC_V4_PASS: &str = "\u{03D2}\u{0301}\u{0000}\u{10400}\u{1F4A9}";
const SPEC_V4_WIF: &str = "5Jajm8eQ22H3pGWLEVCXyvND8dQZhiQhoLJNKjYXk9roUFTMSZ4";
const SPEC_V4_BIP38: &str = "6PRW5o9FLp4gJDDVqJQKJFTpMvdsSGJxMYHtHaQBF3ooa8mwD69bapcDQn";

// SPEC_V5 (BIP-38 §"Test vectors" vector 5): compression, passphrase "Satoshi".
// Source: <https://github.com/bitcoin/bips/blob/master/bip-0038.mediawiki>.
const SPEC_V5_PASS: &str = "Satoshi";
const SPEC_V5_WIF: &str = "KwYgW8gcxj1JWJXhPSu4Fqwzfhp5Yfi42mdYmMa4XqK7NJxXUSK7";
const SPEC_V5_BIP38: &str = "6PYLtMnXvfG3oJde97zRyLYFZCYizPU5T3LwgdYJz1fRhh16bU7u6PPmY7";

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
fn encrypt_wif_to_bip38_vector4_compressed() {
    let out = convert_value(&[
        "convert",
        "--from",
        &format!("wif={V4_WIF}"),
        "--to",
        "bip38",
        "--passphrase",
        V4_PASS,
    ]);
    assert_eq!(out, V4_BIP38);
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
fn decrypt_bip38_to_wif_vector4_compressed() {
    let out = convert_value(&[
        "convert",
        "--from",
        &format!("bip38={V4_BIP38}"),
        "--to",
        "wif",
        "--passphrase",
        V4_PASS,
    ]);
    assert_eq!(out, V4_WIF);
}

// ============================================================================
// BIP-38 §"Test vectors" SPEC_V3 + SPEC_V5 — Phase 3.A
//
// Source: <https://github.com/bitcoin/bips/blob/master/bip-0038.mediawiki>
// §"Test vectors" §"Encryption when EC multiply flag is not used".
// SPEC_V3: vector 3 (no compression, Unicode-NFC passphrase).
// SPEC_V5: vector 5 (compression, Satoshi passphrase).
// ============================================================================

// SPEC_V3 (BIP-38 vector 3) is `#[ignore]`'d: the spec passphrase contains
// U+0000 (NULL), which POSIX/argv do not permit in command-line arguments.
// The `--passphrase=-` stdin sentinel is also unsuitable here because
// `read_stdin_to_string` applies `.trim()`. v0.8 FOLLOWUP
// `bip38-spec-vector-3-null-byte-passphrase` tracks exposing a NULL-safe
// input channel (raw stdin bytes via `--passphrase-bytes-hex` or similar).
// Cite-only entries below preserve the source URL for matrix completeness.
#[test]
#[ignore = "BIP-38 V3 passphrase contains U+0000; not representable via argv (FOLLOWUP bip38-spec-vector-3-null-byte-passphrase)"]
fn encrypt_wif_to_bip38_spec_vector3_unicode_nfc_passphrase() {
    let out = convert_value(&[
        "convert",
        "--from",
        &format!("wif={SPEC_V4_WIF}"),
        "--to",
        "bip38",
        "--passphrase",
        SPEC_V4_PASS,
    ]);
    assert_eq!(out, SPEC_V4_BIP38);
}

#[test]
#[ignore = "BIP-38 V3 passphrase contains U+0000; not representable via argv (FOLLOWUP bip38-spec-vector-3-null-byte-passphrase)"]
fn decrypt_bip38_to_wif_spec_vector3_unicode_nfc_passphrase() {
    let out = convert_value(&[
        "convert",
        "--from",
        &format!("bip38={SPEC_V4_BIP38}"),
        "--to",
        "wif",
        "--passphrase",
        SPEC_V4_PASS,
    ]);
    assert_eq!(out, SPEC_V4_WIF);
}

#[test]
fn encrypt_wif_to_bip38_spec_vector5_satoshi_compressed() {
    let out = convert_value(&[
        "convert",
        "--from",
        &format!("wif={SPEC_V5_WIF}"),
        "--to",
        "bip38",
        "--passphrase",
        SPEC_V5_PASS,
    ]);
    assert_eq!(out, SPEC_V5_BIP38);
}

#[test]
fn decrypt_bip38_to_wif_spec_vector5_satoshi_compressed() {
    let out = convert_value(&[
        "convert",
        "--from",
        &format!("bip38={SPEC_V5_BIP38}"),
        "--to",
        "wif",
        "--passphrase",
        SPEC_V5_PASS,
    ]);
    assert_eq!(out, SPEC_V5_WIF);
}

// ============================================================================
// BIP-38 §"Test vectors" EC-multiplied DECRYPT — Phase 3.B
//
// Source: <https://github.com/bitcoin/bips/blob/master/bip-0038.mediawiki>
// §"Test vectors" §"Encryption when EC multiply mode is used".
//
// BIP-38 defines two encoding forms: non-EC-multiplied (`6P` prefix; the
// passphrase owner already holds the privkey) and EC-multiplied (`6Pf`
// prefix; an intermediate code derived from the passphrase is combined with
// random entropy by a third party to produce the encrypted privkey). The
// toolkit decrypts both forms transparently via `bip38 = "1.1"`'s `Decrypt`
// impl. ENCRYPT to EC-multiplied form (intermediate-code workflow) is NOT
// yet exposed and is tracked as v0.8 FOLLOWUP
// `bip38-ec-multiplied-encrypt-mode-support`.
//
// SPEC §12 erratum: pre-Phase-3 SPEC text incorrectly claimed `bip38`'s
// `Decrypt` impl rejected EC-multiplied codes. Empirical Phase 3 testing
// confirmed all 4 EC vectors decrypt successfully; SPEC §12 corrected in
// the same Phase 3.B commit.
// ============================================================================

#[test]
fn decrypt_bip38_to_wif_ec_multiplied_vector_ec1_testing_one_two_three() {
    // EC1: passphrase "TestingOneTwoThree", no Lot/Sequence.
    const EC1_PASS: &str = "TestingOneTwoThree";
    const EC1_BIP38: &str = "6PfQu77ygVyJLZjfvMLyhLMQbYnu5uguoJJ4kMCLqWwPEdfpwANVS76gTX";
    const EC1_WIF: &str = "5K4caxezwjGCGfnoPTZ8tMcJBLB7Jvyjv4xxeacadhq8nLisLR2";
    let out = convert_value(&[
        "convert",
        "--from",
        &format!("bip38={EC1_BIP38}"),
        "--to",
        "wif",
        "--passphrase",
        EC1_PASS,
    ]);
    assert_eq!(out, EC1_WIF);
}

#[test]
fn decrypt_bip38_to_wif_ec_multiplied_vector_ec2_satoshi() {
    // EC2: passphrase "Satoshi", no Lot/Sequence.
    const EC2_PASS: &str = "Satoshi";
    const EC2_BIP38: &str = "6PfLGnQs6VZnrNpmVKfjotbnQuaJK4KZoPFrAjx1JMJUa1Ft8gnf5WxfKd";
    const EC2_WIF: &str = "5KJ51SgxWaAYR13zd9ReMhJpwrcX47xTJh2D3fGPG9CM8vkv5sH";
    let out = convert_value(&[
        "convert",
        "--from",
        &format!("bip38={EC2_BIP38}"),
        "--to",
        "wif",
        "--passphrase",
        EC2_PASS,
    ]);
    assert_eq!(out, EC2_WIF);
}

#[test]
fn decrypt_bip38_to_wif_ec_multiplied_vector_ec3_lot_sequence_no_compress() {
    // EC3: passphrase "MOLON LABE", Lot 263183 / Sequence 1, no compression.
    const EC3_PASS: &str = "MOLON LABE";
    const EC3_BIP38: &str = "6PgNBNNzDkKdhkT6uJntUXwwzQV8Rr2tZcbkDcuC9DZRsS6AtHts4Ypo1j";
    const EC3_WIF: &str = "5JLdxTtcTHcfYcmJsNVy1v2PMDx432JPoYcBTVVRHpPaxUrdtf8";
    let out = convert_value(&[
        "convert",
        "--from",
        &format!("bip38={EC3_BIP38}"),
        "--to",
        "wif",
        "--passphrase",
        EC3_PASS,
    ]);
    assert_eq!(out, EC3_WIF);
}

#[test]
fn decrypt_bip38_to_wif_ec_multiplied_vector_ec4_lot_sequence_unicode() {
    // EC4: passphrase "ΜΟΛΩΝ ΛΑΒΕ" (Greek capitals), Lot 806938 / Sequence 1,
    // no compression. Pins both the EC-multiplied decrypt path AND
    // Unicode-NFC normalization of the passphrase under the EC-multiply form.
    const EC4_PASS: &str = "\u{039C}\u{039F}\u{039B}\u{03A9}\u{039D} \u{039B}\u{0391}\u{0392}\u{0395}";
    const EC4_BIP38: &str = "6PgGWtx25kUg8QWvwuJAgorN6k9FbE25rv5dMRwu5SKMnfpfVe5mar2ngH";
    const EC4_WIF: &str = "5KMKKuUmAkiNbA3DazMQiLfDq47qs8MAEThm4yL8R2PhV1ov33D";
    let out = convert_value(&[
        "convert",
        "--from",
        &format!("bip38={EC4_BIP38}"),
        "--to",
        "wif",
        "--passphrase",
        EC4_PASS,
    ]);
    assert_eq!(out, EC4_WIF);
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

#[test]
fn composite_entropy_to_bip38_via_wif() {
    // Trezor zero-entropy 12-word phrase's entropy (BIP-39 reference vector,
    // `abandon × 11 about` → `00000000000000000000000000000000`). BIP-84
    // derivation path m/84'/0'/0'/0/0, mainnet. Mirrors
    // `composite_phrase_to_bip38_via_wif` but exercises the (Entropy, Bip38)
    // arm end-to-end via the CLI.
    const ENTROPY: &str = "00000000000000000000000000000000";
    const BIP38_PASS: &str = "TestingOneTwoThree";

    let bip38_out = convert_value(&[
        "convert",
        "--from",
        &format!("entropy={ENTROPY}"),
        "--to",
        "bip38",
        "--path",
        "m/84'/0'/0'/0/0",
        "--passphrase",
        BIP38_PASS,
    ]);
    assert!(
        bip38_out.starts_with("6P"),
        "BIP-38 ciphertext must start with 6P; got {bip38_out:?}"
    );

    // Decrypt back; recovered WIF must match the direct entropy → wif path
    // with the same passphrase (dual-purpose --passphrase, SPEC §12.b).
    let direct_wif = convert_value(&[
        "convert",
        "--from",
        &format!("entropy={ENTROPY}"),
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

// ============================================================================
// Dual-passphrase semantics (SPEC §12.b) — cross-check
// ============================================================================

#[test]
fn composite_phrase_to_bip38_dual_passphrase_semantics_pinned() {
    // SPEC §12.b: in `phrase → wif → bip38`, --passphrase serves a DUAL
    // purpose: BIP-39 mnemonic extension AND BIP-38 Scrypt key. This test
    // pins that behavior — decrypting a `phrase → bip38 --passphrase X`
    // output yields the WIF derived from the phrase WITH X as mnemonic
    // extension (WIF_B), NOT the WIF derived without an extension (WIF_A).
    //
    // If a future refactor splits the two channels (FOLLOWUP
    // `bip38-distinct-passphrase-flag`), this test must be updated.
    const PHRASE: &str =
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    const PATH: &str = "m/84'/0'/0'/0/0";
    const X: &str = "dual-purpose-passphrase";

    // WIF_A: phrase → wif with EMPTY mnemonic extension.
    let wif_a = convert_value(&[
        "convert",
        "--from",
        &format!("phrase={PHRASE}"),
        "--to",
        "wif",
        "--path",
        PATH,
    ]);

    // WIF_B: phrase → wif with X as mnemonic extension.
    let wif_b = convert_value(&[
        "convert",
        "--from",
        &format!("phrase={PHRASE}"),
        "--to",
        "wif",
        "--path",
        PATH,
        "--passphrase",
        X,
    ]);

    assert_ne!(
        wif_a, wif_b,
        "BIP-39 extension must change derived WIF; if these are equal, the test setup is wrong"
    );

    // BIP38_C: phrase → bip38 with --passphrase X (composite arm).
    let bip38_c = convert_value(&[
        "convert",
        "--from",
        &format!("phrase={PHRASE}"),
        "--to",
        "bip38",
        "--path",
        PATH,
        "--passphrase",
        X,
    ]);

    // Decrypt BIP38_C with X.
    let recovered = convert_value(&[
        "convert",
        "--from",
        &format!("bip38={bip38_c}"),
        "--to",
        "wif",
        "--passphrase",
        X,
    ]);

    // Per SPEC §12.b: recovered must equal WIF_B (X applied to BOTH legs),
    // NOT WIF_A (which would imply X was treated as BIP-38-only).
    assert_eq!(
        recovered, wif_b,
        "SPEC §12.b — composite --passphrase MUST drive both PBKDF2 and Scrypt"
    );
    assert_ne!(
        recovered, wif_a,
        "SPEC §12.b — composite --passphrase MUST NOT bypass PBKDF2 mnemonic extension"
    );
}
