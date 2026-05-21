//! v0.31.3 — `mnemonic verify-bundle --slot @N.seedqr=<digit-string>`
//! integration tests. Mirrors the bundle.rs seedqr-slot consumer
//! coverage on the verification path.

use assert_cmd::Command;

const TREZOR_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
const TREZOR_24_DIGITS: &str = "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000102";

#[test]
fn verify_bundle_seedqr_slot_byte_equal_to_phrase() {
    let fixture =
        std::fs::read_to_string("tests/vectors/v0_1/bip84-mainnet.txt").expect("fixture exists");
    let ms1 = fixture
        .lines()
        .find(|l| l.starts_with("ms1") && !l.contains(' '))
        .expect("compact ms1 line")
        .to_string();
    let mk1: Vec<String> = fixture
        .lines()
        .filter(|l| l.starts_with("mk1") && !l.contains(' ') && !l.contains('-'))
        .map(String::from)
        .collect();
    let md1: Vec<String> = fixture
        .lines()
        .filter(|l| l.starts_with("md1") && !l.contains(' ') && !l.contains('-'))
        .map(String::from)
        .collect();
    assert!(!mk1.is_empty() && !md1.is_empty());

    let common_args = |slot: String| -> Vec<String> {
        let mut args: Vec<String> = vec![
            "verify-bundle".into(),
            "--slot".into(),
            slot,
            "--network".into(),
            "mainnet".into(),
            "--template".into(),
            "bip84".into(),
            "--ms1".into(),
            ms1.clone(),
        ];
        for s in &mk1 {
            args.push("--mk1".into());
            args.push(s.clone());
        }
        for s in &md1 {
            args.push("--md1".into());
            args.push(s.clone());
        }
        args
    };

    let via_phrase = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&common_args(format!("@0.phrase={TREZOR_24}")))
        .assert()
        .success();

    let via_seedqr = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&common_args(format!("@0.seedqr={TREZOR_24_DIGITS}")))
        .assert()
        .success();

    assert_eq!(
        via_phrase.get_output().stdout,
        via_seedqr.get_output().stdout,
        "verify-bundle stdout must be byte-equal between --slot @0.phrase=... and \
         --slot @0.seedqr=... (seedqr decode materializes the identical phrase)"
    );
}

#[test]
fn verify_bundle_seedqr_slot_invalid_digit_count_refused() {
    let bad_digits = "0".repeat(47);
    let assertion = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "verify-bundle",
            "--slot",
            &format!("@0.seedqr={bad_digits}"),
            "--network",
            "mainnet",
            "--template",
            "bip84",
            "--ms1",
            "ms1-stub",
            "--mk1",
            "mk1-stub",
            "--md1",
            "md1-stub",
        ])
        .assert()
        .failure();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("seedqr") && stderr.contains("invalid digit count"),
        "expected seedqr decode error in verify-bundle path; got: {stderr}"
    );
}
