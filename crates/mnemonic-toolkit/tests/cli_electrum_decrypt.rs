//! v0.33.0 — `mnemonic electrum-decrypt` integration tests.
//!
//! Surfaces the `electrum_crypto::decrypt_field` primitive (Format A field
//! decryption: base64 `iv || aes-cbc(plaintext+PKCS7)`, key=sha256d(password)).
//! The TV (`TV_CIPHERTEXT` / `test-password` / `"hello world"`) is the
//! cross-impl-validated Cycle-6a vector.

use assert_cmd::Command;

// Cross-impl-validated Cycle-6a vector (electrum_crypto.rs tests).
const TV_CIPHERTEXT: &str = "ABEiM0RVZneImaq7zN3u/zY0181f7qAY/NWiVQFLdHE=";
const TV_PASSWORD: &str = "test-password";
const TV_PLAINTEXT: &str = "hello world";

fn mnemonic() -> Command {
    Command::cargo_bin("mnemonic").expect("mnemonic binary builds")
}

fn temp_with(contents: &str) -> tempfile::NamedTempFile {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), contents.as_bytes()).unwrap();
    tmp
}

#[test]
fn decrypt_inline_password_happy_path() {
    let assertion = mnemonic()
        .args([
            "electrum-decrypt",
            "--ciphertext",
            TV_CIPHERTEXT,
            "--decrypt-password",
            TV_PASSWORD,
        ])
        .assert()
        .success()
        .stdout(format!("{TV_PLAINTEXT}\n"));
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("secret material on argv (--decrypt-password")
            && stderr.contains("warning: stdout carries private key material (can spend)"),
        "expected inline argv-leakage + stdout-class advisories; got: {stderr}"
    );
}

#[test]
fn decrypt_password_stdin_happy_path() {
    let assertion = mnemonic()
        .args([
            "electrum-decrypt",
            "--ciphertext",
            TV_CIPHERTEXT,
            "--decrypt-password-stdin",
        ])
        .write_stdin(TV_PASSWORD)
        .assert()
        .success()
        .stdout(format!("{TV_PLAINTEXT}\n"));
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        !stderr.contains("secret material on argv"),
        "stdin password must NOT emit the argv-leakage advisory; got: {stderr}"
    );
}

#[test]
fn decrypt_password_file_happy_path() {
    let pw = temp_with(TV_PASSWORD);
    mnemonic()
        .args([
            "electrum-decrypt",
            "--ciphertext",
            TV_CIPHERTEXT,
            "--decrypt-password-file",
        ])
        .arg(pw.path())
        .assert()
        .success()
        .stdout(format!("{TV_PLAINTEXT}\n"));
}

#[test]
fn decrypt_wrong_password_refused() {
    let assertion = mnemonic()
        .args([
            "electrum-decrypt",
            "--ciphertext",
            TV_CIPHERTEXT,
            "--decrypt-password",
            "wrong-password",
        ])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("decryption failed (wrong password or corrupted ciphertext)"),
        "expected unified wrong-password message; got: {stderr}"
    );
}

#[test]
fn decrypt_bad_base64_refused() {
    let assertion = mnemonic()
        .args([
            "electrum-decrypt",
            "--ciphertext",
            "not-valid-base64!!!",
            "--decrypt-password",
            TV_PASSWORD,
        ])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("not valid base64"),
        "expected base64 error; got: {stderr}"
    );
}

#[test]
fn decrypt_no_password_required_clap_error() {
    let assertion = mnemonic()
        .args(["electrum-decrypt", "--ciphertext", TV_CIPHERTEXT])
        .assert()
        .failure()
        .code(64); // EX_USAGE (clap ArgGroup required)
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("required") || stderr.contains("decrypt-password"),
        "expected ArgGroup-required error; got: {stderr}"
    );
}

#[test]
fn decrypt_two_password_forms_conflict() {
    mnemonic()
        .args([
            "electrum-decrypt",
            "--ciphertext",
            TV_CIPHERTEXT,
            "--decrypt-password",
            TV_PASSWORD,
            "--decrypt-password-stdin",
        ])
        .write_stdin(TV_PASSWORD)
        .assert()
        .failure()
        .code(64); // EX_USAGE (clap ArgGroup multiple=false conflict)
}

#[test]
fn decrypt_ciphertext_stdin_and_password_stdin_refused() {
    let assertion = mnemonic()
        .args([
            "electrum-decrypt",
            "--ciphertext",
            "-",
            "--decrypt-password-stdin",
        ])
        .write_stdin(TV_PASSWORD)
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("cannot both read from stdin"),
        "expected single-stdin refusal; got: {stderr}"
    );
}

#[test]
fn ciphertext_stdin_happy_path() {
    let pw = temp_with(TV_PASSWORD);
    mnemonic()
        .args([
            "electrum-decrypt",
            "--ciphertext",
            "-",
            "--decrypt-password-file",
        ])
        .arg(pw.path())
        .write_stdin(TV_CIPHERTEXT)
        .assert()
        .success()
        .stdout(format!("{TV_PLAINTEXT}\n"));
}

#[test]
fn decrypt_json_envelope() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    mnemonic()
        .args([
            "electrum-decrypt",
            "--ciphertext",
            TV_CIPHERTEXT,
            "--decrypt-password",
            TV_PASSWORD,
            "--json-out",
        ])
        .arg(tmp.path())
        .assert()
        .success();
    let json: serde_json::Value =
        serde_json::from_reader(std::fs::File::open(tmp.path()).unwrap()).unwrap();
    assert_eq!(json["schema_version"], "1");
    assert_eq!(json["operation"], "electrum-decrypt");
    assert_eq!(json["plaintext"], TV_PLAINTEXT);
    assert!(
        json.get("password").is_none(),
        "envelope must NOT echo the password"
    );
    assert!(json.get("decrypt_password").is_none());
}

#[test]
fn decrypt_realistic_seed_fixture() {
    // R0 Q9 — beyond the toy TV: mint a realistic Electrum-seed-shaped
    // plaintext via the library `encrypt_field` (deterministic test IV),
    // then decrypt it through the CLI and assert round-trip.
    use mnemonic_toolkit::electrum_crypto::encrypt_field;
    let seed =
        "wild father tree among universe such mobile favorite target dynamic credit identify";
    let password = b"correct horse battery staple";
    let iv: [u8; 16] = [
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
        0x10,
    ];
    let ciphertext = encrypt_field(seed, password, &iv);
    let pw = temp_with("correct horse battery staple");
    mnemonic()
        .args([
            "electrum-decrypt",
            "--ciphertext",
            &ciphertext,
            "--decrypt-password-file",
        ])
        .arg(pw.path())
        .assert()
        .success()
        .stdout(format!("{seed}\n"));
}

#[test]
fn json_out_world_readable_advisory() {
    // R0 I2 — --json-out to a 0o644 path emits the world-readable advisory.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("electrum_plaintext.json");
        std::fs::write(&path, b"{}").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
        let assertion = mnemonic()
            .args([
                "electrum-decrypt",
                "--ciphertext",
                TV_CIPHERTEXT,
                "--decrypt-password",
                TV_PASSWORD,
                "--json-out",
            ])
            .arg(&path)
            .assert()
            .success();
        let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
        assert!(
            stderr.to_lowercase().contains("readable") || stderr.contains("permission"),
            "expected world-readable advisory; got: {stderr}"
        );
    }
}
