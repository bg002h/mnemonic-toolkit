//! v0.6.1 SPEC §11 + §11.a — SLIP-0132 prefix-tolerant input + `--xpub-prefix` output.
//!
//! Unit tests for the byte-level swap mechanics live in `src/slip0132.rs::tests`.
//! These integration tests cover CLI plumbing: input normalization at the
//! `convert.rs::compute_outputs` Xpub-source branch, the `--xpub-prefix` clap
//! parser, the post-compute output swap, the `--network`-required-when-non-default
//! refusal, and the silent-ignore policy on non-xpub targets.

use assert_cmd::Command;

const TREZOR_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
const TREZOR_24_BIP84_MAINNET_XPUB: &str = "xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9";
const TREZOR_24_BIP84_MAINNET_ZPUB: &str = "zpub6qTBTNftBzVTjgVcSUw7vW5N1KQbV93Jnrw314RHGkCkSx4vk6nEWH1MJfReXi2WThvuDRiRpyT7cDoakEcZMQ1iZPgfJgQrcVMR4aJWh6S";
const TREZOR_24_BIP84_MAINNET_YPUB: &str = "ypub6Wcv9hzy3JwytPJVc89ViQyrqMG9YX3oskQpDfXPtjpsPrFhVScftDMDHTU4XoNb44p6Tx7sNK6ZiwC22YCYZAL7h3zEimbNLmHmg3ura5Q";
const TREZOR_24_BIP84_MAINNET_BIG_Y: &str = "Ypub6hX1GwjQcGWMJxTsYncUYVKfZ9JQksjQC24V8vnwGWfH22pcFqzwyLD96ARY6EbVHXt5LY7zFXV4r6onKmMVheSnXX6e8B4NEVZeFwGCnJp";
const TREZOR_24_BIP84_MAINNET_BIG_Z: &str = "Zpub72MGacQKkx3qAFezP9Q6kaRAj7SrhViu78ahvKgpeX3A58dqWWAWbPsH7NP869FQhAzt61iYiBqcjPRM3TmWVt8PPro4i5srWDdHeY2Vy6o";

/// Trezor 12-word + BIP-84 testnet account 0 → tpub (verified canonical) +
/// derived testnet SLIP-0132 forms (computed via the impl; the slip0132 unit
/// tests pin the byte-level swap mechanics).
const TREZOR_12: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const TREZOR_12_BIP84_TESTNET_TPUB: &str = "tpubDC8msFGeGuwnKG9Upg7DM2b4DaRqg3CUZa5g8v2SRQ6K4NSkxUgd7HsL2XVWbVm39yBA4LAxysQAm397zwQSQoQgewGiYZqrA9DsP4zbQ1M";
const TREZOR_12_BIP84_TESTNET_VPUB: &str = "vpub5Y6cjg78GGuNLsaPhmYsiw4gYX3HoQiRBiSwDaBXKUafCt9bNwWQiitDk5VZ5BVxYnQdwoTyXSs2JHRPAgjAvtbBrf8ZhDYe2jWAqvZVnsc";
const TREZOR_12_BIP84_TESTNET_UPUB: &str = "upub5DGMS1SD7bMtVaPGsQmFWqyBNYtqrnivGbviSBHdwUCn9nLN8HLr6fE5isXy5Gr399HqCKsR4nWUQzopSzKA8euazKS97Jj9m1SXTNjmvtM";

/// Hash160 of the BIP-84 account-level xpub's own pubkey. NOT the wallet's
/// master fingerprint (`5436d724` for the trezor-24 seed); this is the
/// fingerprint of the account xpub node itself, used by `convert --from xpub
/// --to fingerprint`.
const TREZOR_24_BIP84_ACCT_XPUB_FINGERPRINT: &str = "2bd87e08";

/// Wallet master fingerprint for the trezor-24 seed; emitted by `convert
/// --from phrase --to fingerprint --template bip84` (the BIP-39-rooted
/// derivation path returns the master fingerprint via `derive_bip32_from_entropy`).
const TREZOR_24_MASTER_FINGERPRINT: &str = "5436d724";

// ============================================================================
// §11 input normalizer cross-cut at convert.rs::compute_outputs (Xpub branch)
// ============================================================================

#[test]
fn input_normalizer_zpub_to_fingerprint_matches_xpub() {
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
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(
        stdout,
        format!("fingerprint: {TREZOR_24_BIP84_ACCT_XPUB_FINGERPRINT}\n")
    );
}

#[test]
fn input_normalizer_ypub_to_fingerprint_matches_xpub() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("xpub={TREZOR_24_BIP84_MAINNET_YPUB}"),
            "--to",
            "fingerprint",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(
        stdout,
        format!("fingerprint: {TREZOR_24_BIP84_ACCT_XPUB_FINGERPRINT}\n")
    );
}

#[test]
fn input_normalizer_big_z_to_xpub_normalizes_to_neutral() {
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
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(
        stdout,
        format!("xpub: {TREZOR_24_BIP84_MAINNET_XPUB}\n")
    );
}

#[test]
fn input_normalizer_big_y_to_xpub_normalizes_to_neutral() {
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
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(
        stdout,
        format!("xpub: {TREZOR_24_BIP84_MAINNET_XPUB}\n")
    );
}

#[test]
fn input_normalizer_testnet_vpub_to_xpub_normalizes_to_tpub() {
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
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(
        stdout,
        format!("xpub: {TREZOR_12_BIP84_TESTNET_TPUB}\n")
    );
}

#[test]
fn input_normalizer_testnet_upub_to_xpub_normalizes_to_tpub() {
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
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(
        stdout,
        format!("xpub: {TREZOR_12_BIP84_TESTNET_TPUB}\n")
    );
}

// ============================================================================
// §11.a `--xpub-prefix` output emission
// ============================================================================

#[test]
fn output_xpub_prefix_zpub_mainnet() {
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
            "--xpub-prefix",
            "zpub",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(
        stdout,
        format!("xpub: {TREZOR_24_BIP84_MAINNET_ZPUB}\n")
    );
}

#[test]
fn output_xpub_prefix_big_z_mainnet() {
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
            "--xpub-prefix",
            "Zpub",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(
        stdout,
        format!("xpub: {TREZOR_24_BIP84_MAINNET_BIG_Z}\n")
    );
}

#[test]
fn output_xpub_prefix_ypub_and_big_y_mainnet() {
    for (variant, expected) in [
        ("ypub", TREZOR_24_BIP84_MAINNET_YPUB),
        ("Ypub", TREZOR_24_BIP84_MAINNET_BIG_Y),
    ] {
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
                "--xpub-prefix",
                variant,
            ])
            .assert()
            .success();
        let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
        assert_eq!(stdout, format!("xpub: {expected}\n"));
    }
}

#[test]
fn output_xpub_prefix_default_xpub_is_neutral() {
    // --xpub-prefix xpub IS the default; output must equal the no-flag emission.
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
            "--xpub-prefix",
            "xpub",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(
        stdout,
        format!("xpub: {TREZOR_24_BIP84_MAINNET_XPUB}\n")
    );
}

#[test]
fn output_xpub_prefix_testnet_zpub_emits_vpub() {
    // Testnet variants are network-context-derived: --xpub-prefix zpub +
    // --network testnet → vpub (BIP-84 testnet single-sig).
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("phrase={TREZOR_12}"),
            "--to",
            "xpub",
            "--network",
            "testnet",
            "--template",
            "bip84",
            "--xpub-prefix",
            "zpub",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(
        stdout,
        format!("xpub: {TREZOR_12_BIP84_TESTNET_VPUB}\n")
    );
}

// ============================================================================
// Refusals + silent-ignore
// ============================================================================

#[test]
fn refusal_xpub_prefix_non_default_without_network() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("phrase={TREZOR_24}"),
            "--to",
            "xpub",
            "--template",
            "bip84",
            "--xpub-prefix",
            "zpub",
            // intentionally omit --network
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert_eq!(
        stderr,
        "error: --xpub-prefix <variant> requires explicit --network (cannot infer mainnet vs. testnet swap from defaults).\n"
    );
}

#[test]
fn refusal_unknown_extended_key_version_prefix() {
    // 78-byte buffer prefixed with a bogus 4-byte version, base58check-encoded.
    // Hand-rolled to avoid linking the bin's slip0132 module.
    use bitcoin::base58;
    let mut raw = [0u8; 78];
    raw[0..4].copy_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]);
    let bogus = base58::encode_check(&raw);

    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["convert", "--from", &format!("xpub={bogus}"), "--to", "fingerprint"])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert_eq!(
        stderr,
        "error: unknown extended-key version prefix: deadbeef\n"
    );
}

#[test]
fn xpub_prefix_silently_ignored_on_non_xpub_target() {
    // --xpub-prefix zpub with --to fingerprint: no xpub target, so the flag is
    // silently ignored. Output is the plain fingerprint; no extra stderr noise.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("phrase={TREZOR_24}"),
            "--to",
            "fingerprint",
            "--network",
            "mainnet",
            "--template",
            "bip84",
            "--xpub-prefix",
            "zpub",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(stdout, format!("fingerprint: {TREZOR_24_MASTER_FINGERPRINT}\n"));
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        !stderr.contains("--xpub-prefix"),
        "no --xpub-prefix-related stderr should appear when target has no xpub; got: {stderr:?}"
    );
}

// ============================================================================
// §11.a round-trip property
// ============================================================================

#[test]
fn round_trip_xpub_to_zpub_to_xpub_via_two_invocations() {
    // First invocation: neutral xpub → zpub.
    let zpub_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("xpub={TREZOR_24_BIP84_MAINNET_XPUB}"),
            "--to",
            "xpub",
            "--xpub-prefix",
            "zpub",
            "--network",
            "mainnet",
        ])
        .assert()
        .success();
    let zpub_stdout = String::from_utf8(zpub_out.get_output().stdout.clone()).unwrap();
    let zpub_str = zpub_stdout.trim().trim_start_matches("xpub: ");
    assert!(zpub_str.starts_with("zpub"));

    // Second invocation: zpub → xpub (input normalizer swaps back).
    let xpub_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("xpub={zpub_str}"),
            "--to",
            "xpub",
        ])
        .assert()
        .success();
    let xpub_stdout = String::from_utf8(xpub_out.get_output().stdout.clone()).unwrap();
    assert_eq!(
        xpub_stdout,
        format!("xpub: {TREZOR_24_BIP84_MAINNET_XPUB}\n")
    );
}
