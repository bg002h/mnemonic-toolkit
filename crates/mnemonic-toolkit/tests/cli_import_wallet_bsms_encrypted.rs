//! v0.31.0 — BIP-129 encrypted Round-2 ingest integration tests.
//!
//! These tests exercise the orchestrator-side decrypt path in
//! `cmd/import_wallet.rs`. The crypto primitives are exhaustively
//! tested at the library level by `bsms_crypto::tests` (Cycle 7a;
//! 20 cells incl. TV-3 cross-validation against BIP-129).
//!
//! NOTE on TV-3: BIP-129 TV-3 is a Round-1 KEY record (per BIP-129
//! §Test Vectors "STANDARD Encryption — Signer 1"). The current
//! `BsmsParser` handles Round-2 (4-line / 6-line descriptor shape).
//! The decrypt-success-then-parse-refusal boundary is documented in
//! `tv3_decrypt_emits_notice_advisory`. A future cycle (FOLLOWUP
//! `bsms-encryption-round1-decrypt-then-verify`) adds Round-1
//! decrypt-then-verify integration.

use assert_cmd::Command;
use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/wallet_import")
        .join(name)
}

fn mnemonic() -> Command {
    Command::cargo_bin("mnemonic").expect("mnemonic binary builds")
}

// ──────────────────────────────────────────────────────────────────────
// Happy path (decrypt + MAC verify succeed; parser refuses TV-3 5-line
// Round-1 — the documented boundary)
// ──────────────────────────────────────────────────────────────────────

#[test]
fn tv3_decrypt_emits_notice_advisory() {
    let blob = fixture_path("bsms-encrypted-standard-tv3.dat");
    let token = fixture_path("bsms-encrypted-standard-tv3-token.hex");
    let assertion = mnemonic()
        .args(["import-wallet", "--format", "bsms"])
        .args(["--blob"])
        .arg(&blob)
        .args(["--bsms-encryption-token"])
        .arg(&token)
        .assert();
    let output = assertion.get_output();
    let stderr = String::from_utf8(output.stderr.clone()).unwrap();
    assert!(
        stderr.contains("BIP-129 encrypted Round-2 envelope decrypted")
            && stderr.contains("MAC verified")
            && stderr.contains("token width 16 hex chars"),
        "expected decrypt-success NOTICE on stderr; got: {stderr}"
    );
}

// ──────────────────────────────────────────────────────────────────────
// MAC verify failure (wrong token)
// ──────────────────────────────────────────────────────────────────────

#[test]
fn wrong_token_yields_mac_mismatch_exit_2() {
    let blob = fixture_path("bsms-encrypted-standard-tv3.dat");
    let tmp = tempfile::NamedTempFile::new().unwrap();
    // TV-3 token ends in b7; flip last hex char → b8.
    std::fs::write(tmp.path(), b"a54044308ceac9b8\n").unwrap();
    mnemonic()
        .args(["import-wallet", "--format", "bsms"])
        .args(["--blob"])
        .arg(&blob)
        .args(["--bsms-encryption-token"])
        .arg(tmp.path())
        .assert()
        .failure()
        .code(2)
        .stderr(predicates::str::contains("MAC verification failed"))
        .stderr(predicates::str::contains(
            "wrong token or tampered ciphertext",
        ));
}

// ──────────────────────────────────────────────────────────────────────
// Token format validation
// ──────────────────────────────────────────────────────────────────────

#[test]
fn token_with_invalid_hex_chars_refused() {
    let blob = fixture_path("bsms-encrypted-standard-tv3.dat");
    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), b"not-valid-hex!!!\n").unwrap();
    mnemonic()
        .args(["import-wallet", "--format", "bsms"])
        .args(["--blob"])
        .arg(&blob)
        .args(["--bsms-encryption-token"])
        .arg(tmp.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "token file contents not valid hex",
        ));
}

#[test]
fn token_with_wrong_width_refused() {
    let blob = fixture_path("bsms-encrypted-standard-tv3.dat");
    // 20-hex-char token = 10 bytes (neither STANDARD nor EXTENDED).
    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), b"abcdef0123456789abcd\n").unwrap();
    mnemonic()
        .args(["import-wallet", "--format", "bsms"])
        .args(["--blob"])
        .arg(&blob)
        .args(["--bsms-encryption-token"])
        .arg(tmp.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "token must be 8 bytes STANDARD (16 hex chars) or 16 bytes EXTENDED (32 hex chars)",
        ));
}

#[test]
fn extended_mode_32_hex_token_passes_width_check() {
    // 32-hex-char token = 16 bytes (EXTENDED mode width). The wire blob
    // can't actually be decrypted (it's a STANDARD-mode TV-3) so MAC
    // verify will fail — but the WIDTH check passes, exercising the
    // EXTENDED-mode acceptance path.
    let blob = fixture_path("bsms-encrypted-standard-tv3.dat");
    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), b"108a2360adb302774eb521daebbeda5e\n").unwrap();
    mnemonic()
        .args(["import-wallet", "--format", "bsms"])
        .args(["--blob"])
        .arg(&blob)
        .args(["--bsms-encryption-token"])
        .arg(tmp.path())
        .assert()
        .failure()
        .code(2)
        // The 32-char width passes; MAC verify fails (wrong token for this wire).
        .stderr(predicates::str::contains("MAC verification failed"))
        .stderr(predicates::str::contains("token width 32 hex chars"));
}

#[test]
fn token_uppercase_hex_gets_lowercased() {
    // read_bsms_token lowercases the input. An uppercase TOKEN file
    // produces the same decryption as the lowercase canonical form,
    // because BIP-129 + Coinkite Python use lowercase hex throughout
    // and our normalization aligns.
    let blob = fixture_path("bsms-encrypted-standard-tv3.dat");
    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), b"A54044308CEAC9B7\n").unwrap();
    let assertion = mnemonic()
        .args(["import-wallet", "--format", "bsms"])
        .args(["--blob"])
        .arg(&blob)
        .args(["--bsms-encryption-token"])
        .arg(tmp.path())
        .assert();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("BIP-129 encrypted Round-2 envelope decrypted"),
        "uppercase TOKEN should be lowercased by read_bsms_token + decrypt cleanly; got: {stderr}"
    );
}

// ──────────────────────────────────────────────────────────────────────
// Wire blob format validation
// ──────────────────────────────────────────────────────────────────────

#[test]
fn wire_blob_not_hex_refused() {
    let token = fixture_path("bsms-encrypted-standard-tv3-token.hex");
    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), b"not-valid-hex-blob!!!\n").unwrap();
    mnemonic()
        .args(["import-wallet", "--format", "bsms"])
        .args(["--blob"])
        .arg(tmp.path())
        .args(["--bsms-encryption-token"])
        .arg(&token)
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "encrypted Round-2 wire is not valid hex",
        ));
}

#[test]
fn wire_blob_mac_only_no_ciphertext_refused() {
    // 32-byte wire = exactly MAC, no ciphertext.
    let token = fixture_path("bsms-encrypted-standard-tv3-token.hex");
    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(
        tmp.path(),
        b"fbdbdb64e6a8231c342131d9f13dcd5a954b4c5021658fa5afcb3fc74dc82706",
    )
    .unwrap();
    mnemonic()
        .args(["import-wallet", "--format", "bsms"])
        .args(["--blob"])
        .arg(tmp.path())
        .args(["--bsms-encryption-token"])
        .arg(&token)
        .assert()
        .failure()
        .stderr(predicates::str::contains("too short"));
}

// ──────────────────────────────────────────────────────────────────────
// Stdin handling
// ──────────────────────────────────────────────────────────────────────

#[test]
fn token_via_stdin_works() {
    let blob = fixture_path("bsms-encrypted-standard-tv3.dat");
    let assertion = mnemonic()
        .args(["import-wallet", "--format", "bsms"])
        .args(["--blob"])
        .arg(&blob)
        .args(["--bsms-encryption-token", "-"])
        .write_stdin("a54044308ceac9b7")
        .assert();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("BIP-129 encrypted Round-2 envelope decrypted"),
        "stdin-token path should produce decrypt-success NOTICE; got: {stderr}"
    );
}

#[test]
fn both_blob_and_token_stdin_refused() {
    // R0 I2: stdin-contention guard. When both --blob=- and
    // --bsms-encryption-token=- are supplied, the orchestrator MUST
    // refuse explicitly (otherwise the blob-read consumes stdin and
    // the token-read sees EOF, producing a misleading "not valid hex"
    // error).
    mnemonic()
        .args(["import-wallet", "--format", "bsms"])
        .args(["--blob", "-"])
        .args(["--bsms-encryption-token", "-"])
        .write_stdin("ignored")
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "--blob=- and --bsms-encryption-token=- cannot both read from stdin",
        ));
}

// ──────────────────────────────────────────────────────────────────────
// No-token path (no --bsms-encryption-token; should refuse encrypted)
// ──────────────────────────────────────────────────────────────────────

#[test]
fn encrypted_blob_without_token_refused_at_parser() {
    // Without --bsms-encryption-token, an encrypted blob doesn't have
    // the `BSMS 1.0` header so it doesn't auto-sniff as BSMS. With
    // --format bsms explicit, the parser hits its existing
    // header-required refusal path.
    let blob = fixture_path("bsms-encrypted-standard-tv3.dat");
    mnemonic()
        .args(["import-wallet", "--format", "bsms"])
        .args(["--blob"])
        .arg(&blob)
        .assert()
        .failure();
}

// ──────────────────────────────────────────────────────────────────────
// No regression on plaintext BSMS (without --bsms-encryption-token)
// ──────────────────────────────────────────────────────────────────────

#[test]
fn plaintext_2line_multi_2of3_no_regression() {
    // Pre-v0.31.0 plaintext BSMS Round-2 blob still imports successfully
    // without --bsms-encryption-token. Sanity check: behavior unchanged.
    let blob = fixture_path("bsms-2line-multi-2of3.txt");
    mnemonic()
        .args(["import-wallet", "--format", "bsms"])
        .args(["--blob"])
        .arg(&blob)
        .args(["--json"])
        .assert()
        .success();
}

// ──────────────────────────────────────────────────────────────────────
// v0.32.1 — encrypted Round-1 decrypt-then-verify (--bsms-round1)
// ──────────────────────────────────────────────────────────────────────

const TV3_TOKEN_HEX: &str = "a54044308ceac9b7";

/// Helper: decrypt the TV-3 encrypted Round-1 wire to recover the
/// plaintext 5-line BSMS record (via the library primitives).
fn tv3_decrypted_plaintext() -> String {
    use mnemonic_toolkit::bsms_crypto::{decrypt, derive_encryption_key};
    let wire_hex =
        std::fs::read_to_string(fixture_path("bsms-encrypted-standard-tv3.dat")).unwrap();
    let wire = hex::decode(wire_hex.trim()).unwrap();
    let (mac, ct) = wire.split_at(32);
    let token_raw = hex::decode(TV3_TOKEN_HEX).unwrap();
    let enc_key = derive_encryption_key(&token_raw);
    let iv: [u8; 16] = mac[..16].try_into().unwrap();
    let pt = decrypt(ct, &enc_key, &iv).unwrap();
    String::from_utf8(pt.to_vec()).unwrap()
}

/// Helper: re-encrypt a plaintext Round-1 record with an arbitrary token
/// (computing MAC + IV per BIP-129) → hex `MAC || ciphertext` wire.
fn reencrypt_with_token(plaintext: &str, token_hex: &str) -> String {
    use mnemonic_toolkit::bsms_crypto::{
        compute_mac, derive_encryption_key, derive_hmac_key, encrypt,
    };
    let token_raw = hex::decode(token_hex).unwrap();
    let enc_key = derive_encryption_key(&token_raw);
    let hmac_key = derive_hmac_key(&enc_key);
    let mac = compute_mac(&hmac_key, token_hex, plaintext.as_bytes());
    let iv: [u8; 16] = mac[..16].try_into().unwrap();
    let ct = encrypt(plaintext.as_bytes(), &enc_key, &iv);
    hex::encode([&mac[..], &ct[..]].concat())
}

/// Helper: re-encrypt with the TV-3 token (back-compat shim for the
/// Cycle-15 cell).
fn reencrypt_with_tv3_token(plaintext: &str) -> String {
    reencrypt_with_token(plaintext, TV3_TOKEN_HEX)
}

/// Write `contents` to a fresh temp file, returning the keep-alive handle
/// (caller binds it so the file outlives the command).
fn temp_with(contents: &str) -> tempfile::NamedTempFile {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), contents.as_bytes()).unwrap();
    tmp
}

#[test]
fn tv3_round1_decrypt_then_verify() {
    // The TV-3 encrypted Round-1 record, fed via --bsms-round1 + token,
    // decrypts + MAC-verifies + BIP-322-verifies (verified=true).
    let dat = fixture_path("bsms-encrypted-standard-tv3.dat");
    let token = fixture_path("bsms-encrypted-standard-tv3-token.hex");
    let assertion = mnemonic()
        .args(["import-wallet", "--bsms-round1"])
        .arg(&dat)
        .args(["--bsms-encryption-token"])
        .arg(&token)
        .args(["--json"])
        .assert()
        .success();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("BIP-129 encrypted Round-1 record 0 decrypted")
            && stderr.contains("MAC verified"),
        "expected Round-1 decrypt NOTICE; got: {stderr}"
    );
    let stdout = String::from_utf8(assertion.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let v = &json["bsms_round1_verifications"][0];
    assert_eq!(
        v["signature_verified"], true,
        "TV-3 Round-1 must verify; got: {stdout}"
    );
}

#[test]
fn round1_encrypted_without_token_refused() {
    let dat = fixture_path("bsms-encrypted-standard-tv3.dat");
    let assertion = mnemonic()
        .args(["import-wallet", "--bsms-round1"])
        .arg(&dat)
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("looks encrypted")
            && stderr.contains("no --bsms-encryption-token was supplied"),
        "expected no-token refusal; got: {stderr}"
    );
}

#[test]
fn round1_encrypted_wrong_token_mac_mismatch() {
    let dat = fixture_path("bsms-encrypted-standard-tv3.dat");
    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), b"a54044308ceac9b8\n").unwrap(); // flipped last char
    mnemonic()
        .args(["import-wallet", "--bsms-round1"])
        .arg(&dat)
        .args(["--bsms-encryption-token"])
        .arg(tmp.path())
        .assert()
        .failure()
        .code(2)
        .stderr(predicates::str::contains("MAC verification failed"));
}

#[test]
fn round1_plaintext_still_verifies_no_misclassify() {
    // A plaintext 5-line Round-1 record via --bsms-round1 (no token) still
    // verifies — the encrypted-detection must NOT mis-classify plaintext
    // (it starts with the `BSMS 1.0` header → not all-hex).
    let plaintext_fixture = "tests/fixtures/bsms_round1/tv1-no-encryption-pubkey-signer1.bsms";
    let assertion = mnemonic()
        .args([
            "import-wallet",
            "--bsms-round1",
            plaintext_fixture,
            "--json",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(assertion.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(
        json["bsms_round1_verifications"][0]["signature_verified"],
        true
    );
}

#[test]
fn round1_encrypted_decrypt_ok_but_sig_fail() {
    // R0 M2 — an encrypted Round-1 record that decrypts + MAC-verifies OK
    // but whose plaintext has a corrupted BIP-322 signature: lenient mode
    // → NOTICE + Failed status (exit 0); --bsms-verify-strict → fatal.
    let plaintext = tv3_decrypted_plaintext();
    // Corrupt the base64 SIG (last line): flip its first char to a
    // different valid base64 char (keeps the 5-line shape + valid base64).
    let mut lines: Vec<&str> = plaintext.lines().collect();
    let sig = lines.pop().unwrap();
    let first = sig.chars().next().unwrap();
    let flipped = if first == 'A' { 'B' } else { 'A' };
    let bad_sig: String = std::iter::once(flipped)
        .chain(sig.chars().skip(1))
        .collect();
    let corrupted = format!("{}\n{}\n", lines.join("\n"), bad_sig);
    let wire_hex = reencrypt_with_tv3_token(&corrupted);

    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), wire_hex.as_bytes()).unwrap();
    let token = fixture_path("bsms-encrypted-standard-tv3-token.hex");

    // Lenient: decrypt OK, NOTICE, verify-failed status, exit 0.
    let lenient = mnemonic()
        .args(["import-wallet", "--bsms-round1"])
        .arg(tmp.path())
        .args(["--bsms-encryption-token"])
        .arg(&token)
        .args(["--json"])
        .assert()
        .success();
    let stderr = String::from_utf8(lenient.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("record 0 decrypted") && stderr.contains("signature verification failed"),
        "expected decrypt-OK + verify-FAIL NOTICE; got: {stderr}"
    );

    // Strict: same record → fatal.
    mnemonic()
        .args(["import-wallet", "--bsms-round1"])
        .arg(tmp.path())
        .args(["--bsms-encryption-token"])
        .arg(&token)
        .args(["--bsms-verify-strict"])
        .assert()
        .failure();
}

// ──────────────────────────────────────────────────────────────────────
// v0.32.2 — per-Signer tokens (repeatable --bsms-encryption-token)
// ──────────────────────────────────────────────────────────────────────

const TOKEN_B_HEX: &str = "b1b2b3b4b5b6b7b8";

#[test]
fn per_signer_two_tokens_two_records_positional() {
    // Two encrypted Round-1 records, each with its own token, paired
    // positionally (token[i] ↔ record[i]). Both decrypt + verify.
    let pt = tv3_decrypted_plaintext();
    let rec0 = temp_with(&reencrypt_with_token(&pt, TV3_TOKEN_HEX));
    let rec1 = temp_with(&reencrypt_with_token(&pt, TOKEN_B_HEX));
    let tok0 = temp_with(TV3_TOKEN_HEX);
    let tok1 = temp_with(TOKEN_B_HEX);
    let assertion = mnemonic()
        .args(["import-wallet", "--bsms-round1"])
        .arg(rec0.path())
        .args(["--bsms-round1"])
        .arg(rec1.path())
        .args(["--bsms-encryption-token"])
        .arg(tok0.path())
        .args(["--bsms-encryption-token"])
        .arg(tok1.path())
        .args(["--json"])
        .assert()
        .success();
    let stdout = String::from_utf8(assertion.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(
        json["bsms_round1_verifications"][0]["signature_verified"],
        true
    );
    assert_eq!(
        json["bsms_round1_verifications"][1]["signature_verified"],
        true
    );
}

#[test]
fn single_token_shared_still_decrypts_all() {
    // One token shared across TWO encrypted records (backward-compatible).
    let pt = tv3_decrypted_plaintext();
    let rec0 = temp_with(&reencrypt_with_token(&pt, TV3_TOKEN_HEX));
    let rec1 = temp_with(&reencrypt_with_token(&pt, TV3_TOKEN_HEX));
    let tok = temp_with(TV3_TOKEN_HEX);
    let assertion = mnemonic()
        .args(["import-wallet", "--bsms-round1"])
        .arg(rec0.path())
        .args(["--bsms-round1"])
        .arg(rec1.path())
        .args(["--bsms-encryption-token"])
        .arg(tok.path())
        .args(["--json"])
        .assert()
        .success();
    let stdout = String::from_utf8(assertion.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(
        json["bsms_round1_verifications"][0]["signature_verified"],
        true
    );
    assert_eq!(
        json["bsms_round1_verifications"][1]["signature_verified"],
        true
    );
}

#[test]
fn per_signer_token_count_mismatch_refused() {
    // 2 records + 3 tokens → N>1 ≠ record count → refuse.
    let pt = tv3_decrypted_plaintext();
    let rec0 = temp_with(&reencrypt_with_token(&pt, TV3_TOKEN_HEX));
    let rec1 = temp_with(&reencrypt_with_token(&pt, TOKEN_B_HEX));
    let tok0 = temp_with(TV3_TOKEN_HEX);
    let tok1 = temp_with(TOKEN_B_HEX);
    let tok2 = temp_with(TV3_TOKEN_HEX);
    let assertion = mnemonic()
        .args(["import-wallet", "--bsms-round1"])
        .arg(rec0.path())
        .args(["--bsms-round1"])
        .arg(rec1.path())
        .args(["--bsms-encryption-token"])
        .arg(tok0.path())
        .args(["--bsms-encryption-token"])
        .arg(tok1.path())
        .args(["--bsms-encryption-token"])
        .arg(tok2.path())
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("counts must match for positional pairing"),
        "expected count-mismatch refusal; got: {stderr}"
    );
}

#[test]
fn per_signer_mixed_plaintext_encrypted_refused() {
    // 2 tokens but record[1] is plaintext → per-Signer mode requires all
    // records encrypted → refuse.
    let pt = tv3_decrypted_plaintext();
    let rec0 = temp_with(&reencrypt_with_token(&pt, TV3_TOKEN_HEX));
    let rec1 = temp_with(&pt); // plaintext
    let tok0 = temp_with(TV3_TOKEN_HEX);
    let tok1 = temp_with(TOKEN_B_HEX);
    let assertion = mnemonic()
        .args(["import-wallet", "--bsms-round1"])
        .arg(rec0.path())
        .args(["--bsms-round1"])
        .arg(rec1.path())
        .args(["--bsms-encryption-token"])
        .arg(tok0.path())
        .args(["--bsms-encryption-token"])
        .arg(tok1.path())
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("per-Signer mode requires every record to be encrypted"),
        "expected mixed-mode refusal; got: {stderr}"
    );
}

#[test]
fn per_signer_multi_token_with_encrypted_blob_refused() {
    // N>1 tokens + an encrypted Round-2 --blob → ambiguous → refuse.
    // (Reached only after the Round-1 records verify.)
    let pt = tv3_decrypted_plaintext();
    let rec0 = temp_with(&reencrypt_with_token(&pt, TV3_TOKEN_HEX));
    let rec1 = temp_with(&reencrypt_with_token(&pt, TOKEN_B_HEX));
    let blob = fixture_path("bsms-encrypted-standard-tv3.dat");
    let tok0 = temp_with(TV3_TOKEN_HEX);
    let tok1 = temp_with(TOKEN_B_HEX);
    let assertion = mnemonic()
        .args(["import-wallet", "--format", "bsms", "--bsms-round1"])
        .arg(rec0.path())
        .args(["--bsms-round1"])
        .arg(rec1.path())
        .args(["--blob"])
        .arg(&blob)
        .args(["--bsms-encryption-token"])
        .arg(tok0.path())
        .args(["--bsms-encryption-token"])
        .arg(tok1.path())
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("Round-2 --blob decrypt requires exactly one --bsms-encryption-token"),
        "expected multi-token-blob refusal; got: {stderr}"
    );
}

#[test]
fn multi_token_zero_records_refused() {
    // R0 I1 gap-h — N>1 tokens with NO --bsms-round1 records → refuse
    // (tokens would otherwise be read + silently discarded). A `--blob`
    // is supplied to satisfy clap's `--blob required_unless bsms_round1`
    // so the runtime gap-h guard (which runs before blob processing) is
    // reached rather than clap's usage error.
    let blob = fixture_path("bsms-encrypted-standard-tv3.dat");
    let tok0 = temp_with(TV3_TOKEN_HEX);
    let tok1 = temp_with(TOKEN_B_HEX);
    let assertion = mnemonic()
        .args(["import-wallet", "--format", "bsms", "--blob"])
        .arg(&blob)
        .args(["--bsms-encryption-token"])
        .arg(tok0.path())
        .args(["--bsms-encryption-token"])
        .arg(tok1.path())
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("require matching --bsms-round1 records; none supplied"),
        "expected gap-h refusal; got: {stderr}"
    );
}

#[test]
fn per_signer_token_i_mac_mismatch_cites_index() {
    // R0 M2 — 2 records + 2 tokens where token[1] is WRONG for record[1]
    // (record[1] encrypted with TOKEN_B but token[1] supplied is the TV-3
    // token) → MAC mismatch on record index 1.
    let pt = tv3_decrypted_plaintext();
    let rec0 = temp_with(&reencrypt_with_token(&pt, TV3_TOKEN_HEX));
    let rec1 = temp_with(&reencrypt_with_token(&pt, TOKEN_B_HEX));
    let tok0 = temp_with(TV3_TOKEN_HEX);
    let tok1_wrong = temp_with(TV3_TOKEN_HEX); // wrong for rec1 (needs TOKEN_B)
    mnemonic()
        .args(["import-wallet", "--bsms-round1"])
        .arg(rec0.path())
        .args(["--bsms-round1"])
        .arg(rec1.path())
        .args(["--bsms-encryption-token"])
        .arg(tok0.path())
        .args(["--bsms-encryption-token"])
        .arg(tok1_wrong.path())
        .assert()
        .failure()
        .code(2)
        .stderr(predicates::str::contains("MAC verification failed"));
}

#[test]
fn two_token_stdin_refused() {
    // Two --bsms-encryption-token=- → single-stdin-per-invocation refusal.
    let rec0 = temp_with(&reencrypt_with_token(
        &tv3_decrypted_plaintext(),
        TV3_TOKEN_HEX,
    ));
    let rec1 = temp_with(&reencrypt_with_token(
        &tv3_decrypted_plaintext(),
        TOKEN_B_HEX,
    ));
    let assertion = mnemonic()
        .args(["import-wallet", "--bsms-round1"])
        .arg(rec0.path())
        .args(["--bsms-round1"])
        .arg(rec1.path())
        .args([
            "--bsms-encryption-token",
            "-",
            "--bsms-encryption-token",
            "-",
        ])
        .write_stdin(TV3_TOKEN_HEX)
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("at most one --bsms-encryption-token=- per invocation"),
        "expected single-stdin refusal; got: {stderr}"
    );
}

// ──────────────────────────────────────────────────────────────────────
// v0.32.3 — Coinkite Python cross-implementation smoke (vendored fixtures)
//
// `bsms-coinkite-xref-round2-2of3.dat` is a hex MAC||ciphertext wire
// produced by the INDEPENDENT Coinkite reference's `bsms.encryption.encrypt`
// (pinned SHA c30abe3a, EXTENDED 16-byte token) over the real
// `bsms-2line-multi-2of3.txt` Round-2 descriptor. See
// `tests/external/README.md` for the regen recipe.
// ──────────────────────────────────────────────────────────────────────

const XREF_TOKEN_HEX: &str = "00112233445566778899aabbccddeeff";

#[test]
fn coinkite_xref_round2_full_plaintext_byte_equal() {
    // R0 I1 — the strong cross-impl pin: decrypt the Coinkite-generated
    // wire with the toolkit's library primitives and assert the recovered
    // plaintext byte-equals the source plaintext over its FULL length
    // (exercises the keystream across all ~460 bytes, not just the
    // descriptor line).
    use mnemonic_toolkit::bsms_crypto::{decrypt, derive_encryption_key};
    let wire_hex =
        std::fs::read_to_string(fixture_path("bsms-coinkite-xref-round2-2of3.dat")).unwrap();
    let wire = hex::decode(wire_hex.trim()).unwrap();
    let (mac, ct) = wire.split_at(32);
    let token_raw = hex::decode(XREF_TOKEN_HEX).unwrap();
    let enc_key = derive_encryption_key(&token_raw);
    let iv: [u8; 16] = mac[..16].try_into().unwrap();
    let recovered = decrypt(ct, &enc_key, &iv).unwrap();

    let expected = std::fs::read(fixture_path("bsms-2line-multi-2of3.txt")).unwrap();
    assert_eq!(
        recovered.as_slice(),
        expected.as_slice(),
        "Coinkite-encrypted wire must decrypt byte-for-byte to the source plaintext"
    );
}

#[test]
fn coinkite_xref_round2_descriptor_imports() {
    // End-to-end CLI cross-impl: feed the Coinkite wire through
    // import-wallet; assert the imported descriptor equals the descriptor
    // from importing the PLAINTEXT directly. Also the first EXTENDED-token
    // wire that actually DECRYPTS via the CLI.
    let wire = fixture_path("bsms-coinkite-xref-round2-2of3.dat");
    let token = fixture_path("bsms-coinkite-xref-round2-2of3-token.hex");
    let via_cipher = mnemonic()
        .args(["import-wallet", "--format", "bsms"])
        .args(["--blob"])
        .arg(&wire)
        .args(["--bsms-encryption-token"])
        .arg(&token)
        .args(["--json"])
        .assert()
        .success();
    let cipher_stderr = String::from_utf8(via_cipher.get_output().stderr.clone()).unwrap();
    assert!(
        cipher_stderr.contains("BIP-129 encrypted Round-2 envelope decrypted")
            && cipher_stderr.contains("token width 32 hex chars")
            && cipher_stderr.contains("MAC verified"),
        "expected EXTENDED-token decrypt NOTICE; got: {cipher_stderr}"
    );
    let cipher_json: serde_json::Value =
        serde_json::from_slice(&via_cipher.get_output().stdout).unwrap();

    let plaintext = fixture_path("bsms-2line-multi-2of3.txt");
    let via_plain = mnemonic()
        .args(["import-wallet", "--format", "bsms"])
        .args(["--blob"])
        .arg(&plaintext)
        .args(["--json"])
        .assert()
        .success();
    let plain_json: serde_json::Value =
        serde_json::from_slice(&via_plain.get_output().stdout).unwrap();

    assert_eq!(
        cipher_json[0]["bundle"]["descriptor"], plain_json[0]["bundle"]["descriptor"],
        "Coinkite-decrypted import descriptor must equal the plaintext import descriptor"
    );
}

#[test]
fn coinkite_xref_round2_wrong_token_mac_mismatch() {
    let wire = fixture_path("bsms-coinkite-xref-round2-2of3.dat");
    let tmp = tempfile::NamedTempFile::new().unwrap();
    // Flip the last hex char of the EXTENDED token: ...eeff → ...eefe.
    std::fs::write(tmp.path(), b"00112233445566778899aabbccddeefe\n").unwrap();
    mnemonic()
        .args(["import-wallet", "--format", "bsms"])
        .args(["--blob"])
        .arg(&wire)
        .args(["--bsms-encryption-token"])
        .arg(tmp.path())
        .assert()
        .failure()
        .code(2)
        .stderr(predicates::str::contains("MAC verification failed"));
}
