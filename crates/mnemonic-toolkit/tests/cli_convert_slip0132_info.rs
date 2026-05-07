//! v0.6.2 §11.b — SLIP-0132 input-normalization stderr info-line UX.
//!
//! Pins the canonical info-line text and the "no info-line when input is
//! already neutral" suppression. RED on master pre-Phase 3; GREEN once the
//! emission is wired into `convert.rs::compute_outputs`.

use assert_cmd::Command;

const TREZOR_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
const TREZOR_24_BIP84_MAINNET_XPUB: &str = "xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9";
const TREZOR_24_BIP84_MAINNET_ZPUB: &str = "zpub6qTBTNftBzVTjgVcSUw7vW5N1KQbV93Jnrw314RHGkCkSx4vk6nEWH1MJfReXi2WThvuDRiRpyT7cDoakEcZMQ1iZPgfJgQrcVMR4aJWh6S";
const TREZOR_24_BIP84_MAINNET_YPUB: &str = "ypub6Wcv9hzy3JwytPJVc89ViQyrqMG9YX3oskQpDfXPtjpsPrFhVScftDMDHTU4XoNb44p6Tx7sNK6ZiwC22YCYZAL7h3zEimbNLmHmg3ura5Q";
const TREZOR_24_BIP84_MAINNET_BIG_Y: &str = "Ypub6hX1GwjQcGWMJxTsYncUYVKfZ9JQksjQC24V8vnwGWfH22pcFqzwyLD96ARY6EbVHXt5LY7zFXV4r6onKmMVheSnXX6e8B4NEVZeFwGCnJp";
const TREZOR_24_BIP84_MAINNET_BIG_Z: &str = "Zpub72MGacQKkx3qAFezP9Q6kaRAj7SrhViu78ahvKgpeX3A58dqWWAWbPsH7NP869FQhAzt61iYiBqcjPRM3TmWVt8PPro4i5srWDdHeY2Vy6o";

const TREZOR_12_BIP84_TESTNET_VPUB: &str = "vpub5Y6cjg78GGuNLsaPhmYsiw4gYX3HoQiRBiSwDaBXKUafCt9bNwWQiitDk5VZ5BVxYnQdwoTyXSs2JHRPAgjAvtbBrf8ZhDYe2jWAqvZVnsc";
const TREZOR_12_BIP84_TESTNET_UPUB: &str = "upub5DGMS1SD7bMtVaPGsQmFWqyBNYtqrnivGbviSBHdwUCn9nLN8HLr6fE5isXy5Gr399HqCKsR4nWUQzopSzKA8euazKS97Jj9m1SXTNjmvtM";

/// Derived from `TREZOR_12_BIP84_TESTNET_VPUB` by decode-swap-reencode with
/// the SLIP-0132 testnet `Vpub` version prefix `0x02 0x57 0x54 0x83`. The
/// `--xpub-prefix` clap flag rejects testnet variant strings (mainnet-only
/// flag values per SPEC §11.a), so this is hand-derived rather than emitted
/// by the CLI.
const TREZOR_12_BIP84_TESTNET_BIG_V: &str = "Vpub5izhruqZqETjmSjmeS1rZ1QVGK5Z1mQ1Vz6c8qT4hFR4q4iW9Ltgoqk9YnT2dcirnFUcpPU6QfFXRT39Tut85Nhrh8Ey6d1dvTn3RqK6nRC";
/// Derived from `TREZOR_12_BIP84_TESTNET_UPUB` by decode-swap-reencode with
/// the SLIP-0132 testnet `Upub` version prefix `0x02 0x42 0x89 0xEF`.
const TREZOR_12_BIP84_TESTNET_BIG_U: &str = "Upub5QASZFAegYvFv9Yep5EELvJz6Lw759QWasaPMSZBKF3BmxuGtgj8Bn61XaVSdi4wNcMp4usXwztyYARakDU7H92FpnYYWiC9ejiQ3DE4Wxx";

const TREZOR_24_BIP84_ACCT_XPUB_FINGERPRINT: &str = "2bd87e08";

/// Build the SPEC §5.5.a info-line for a recognized SLIP-0132 input prefix.
/// Variant determines the neutral form: mainnet → xpub, testnet → tpub.
fn info_line(variant: &str) -> String {
    let neutral = match variant {
        "ypub" | "Ypub" | "zpub" | "Zpub" => "xpub",
        "upub" | "Upub" | "vpub" | "Vpub" => "tpub",
        _ => unreachable!(
            "info_line: unknown variant {variant:?} (must be one of: ypub, Ypub, zpub, Zpub, upub, Upub, vpub, Vpub)"
        ),
    };
    format!(
        "info: normalized {variant} input to neutral {neutral} (encoding-only; no key change). Re-emit with --xpub-prefix {variant} if you need the SLIP-0132 form.\n"
    )
}

// ============================================================================
// Matrix cell #1: zpub → --to xpub emits info-line on stderr.
// ============================================================================
#[test]
fn convert_zpub_to_xpub_emits_info_line() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("xpub={TREZOR_24_BIP84_MAINNET_ZPUB}"),
            "--to",
            "xpub",
        ])
        .assert()
        .success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert_eq!(stderr, info_line("zpub"));
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(stdout, format!("xpub: {TREZOR_24_BIP84_MAINNET_XPUB}\n"));
}

// ============================================================================
// Matrix cell #2: already-neutral xpub → --to xpub emits NO info-line.
// ============================================================================
#[test]
fn convert_neutral_xpub_to_xpub_emits_no_info_line() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("xpub={TREZOR_24_BIP84_MAINNET_XPUB}"),
            "--to",
            "xpub",
        ])
        .assert()
        .success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        !stderr.contains("info: normalized"),
        "neutral input must not emit normalization info-line; got stderr: {stderr:?}"
    );
}

// ============================================================================
// Matrix cell #3: zpub → --to xpub --xpub-prefix Ypub still emits info-line.
// ============================================================================
#[test]
fn convert_zpub_to_xpub_with_output_prefix_still_emits_info_line() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("xpub={TREZOR_24_BIP84_MAINNET_ZPUB}"),
            "--to",
            "xpub",
            "--network",
            "mainnet",
            "--xpub-prefix",
            "Ypub",
        ])
        .assert()
        .success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert_eq!(stderr, info_line("zpub"));
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(stdout, format!("xpub: {TREZOR_24_BIP84_MAINNET_BIG_Y}\n"));
}

// ============================================================================
// Matrix cell #4: zpub → --to fingerprint emits info-line.
// ============================================================================
#[test]
fn convert_zpub_to_fingerprint_emits_info_line() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("xpub={TREZOR_24_BIP84_MAINNET_ZPUB}"),
            "--to",
            "fingerprint",
        ])
        .assert()
        .success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert_eq!(stderr, info_line("zpub"));
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(
        stdout,
        format!("fingerprint: {TREZOR_24_BIP84_ACCT_XPUB_FINGERPRINT}\n")
    );
}

// ============================================================================
// Variant coverage — 4 mainnet variants on convert --to xpub.
// Each asserts the info-line substitutes the variant string verbatim incl. case.
// ============================================================================
#[test]
fn convert_variant_coverage_ypub_mainnet() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("xpub={TREZOR_24_BIP84_MAINNET_YPUB}"),
            "--to",
            "xpub",
        ])
        .assert()
        .success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert_eq!(stderr, info_line("ypub"));
}

#[test]
fn convert_variant_coverage_big_y_mainnet() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("xpub={TREZOR_24_BIP84_MAINNET_BIG_Y}"),
            "--to",
            "xpub",
        ])
        .assert()
        .success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert_eq!(stderr, info_line("Ypub"));
}

#[test]
fn convert_variant_coverage_zpub_mainnet() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("xpub={TREZOR_24_BIP84_MAINNET_ZPUB}"),
            "--to",
            "xpub",
        ])
        .assert()
        .success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert_eq!(stderr, info_line("zpub"));
}

#[test]
fn convert_variant_coverage_big_z_mainnet() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("xpub={TREZOR_24_BIP84_MAINNET_BIG_Z}"),
            "--to",
            "xpub",
        ])
        .assert()
        .success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert_eq!(stderr, info_line("Zpub"));
}

#[test]
fn convert_variant_coverage_upub_testnet() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("xpub={TREZOR_12_BIP84_TESTNET_UPUB}"),
            "--to",
            "xpub",
        ])
        .assert()
        .success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert_eq!(stderr, info_line("upub"));
}

#[test]
fn convert_variant_coverage_big_u_testnet() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("xpub={TREZOR_12_BIP84_TESTNET_BIG_U}"),
            "--to",
            "xpub",
        ])
        .assert()
        .success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert_eq!(stderr, info_line("Upub"));
}

#[test]
fn convert_variant_coverage_vpub_testnet() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("xpub={TREZOR_12_BIP84_TESTNET_VPUB}"),
            "--to",
            "xpub",
        ])
        .assert()
        .success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert_eq!(stderr, info_line("vpub"));
}

#[test]
fn convert_variant_coverage_big_v_testnet() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("xpub={TREZOR_12_BIP84_TESTNET_BIG_V}"),
            "--to",
            "xpub",
        ])
        .assert()
        .success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert_eq!(stderr, info_line("Vpub"));
}

// ============================================================================
// Matrix cell #10: --json mode still emits info-line on stderr; stdout is JSON.
// ============================================================================
#[test]
fn convert_json_mode_emits_info_line_on_stderr() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("xpub={TREZOR_24_BIP84_MAINNET_ZPUB}"),
            "--to",
            "xpub",
            "--json",
        ])
        .assert()
        .success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert_eq!(stderr, info_line("zpub"));
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON envelope");
    assert!(v.is_object(), "stdout must be a JSON object; got {v:?}");
}

/// Phrase input does not exercise the SLIP-0132 normalizer (no version-prefix
/// in a phrase). This test is GREEN throughout and pins the negative case.
#[test]
fn convert_phrase_input_emits_no_info_line() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("phrase={TREZOR_24}"),
            "--to",
            "xpub",
            "--network",
            "mainnet",
            "--template",
            "bip84",
        ])
        .assert()
        .success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        !stderr.contains("info: normalized"),
        "phrase input has no SLIP-0132 prefix to normalize; got stderr: {stderr:?}"
    );
}
