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
        .stderr(predicates::str::contains("wrong token or tampered ciphertext"));
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
