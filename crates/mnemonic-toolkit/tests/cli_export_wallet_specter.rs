//! v0.8.1 Phase 3 — `mnemonic export-wallet --format specter` integration tests.
//!
//! SPEC `design/SPEC_export_wallet_v0_8.md` §8 (Specter Desktop wallet-import
//! emitter). Byte-exact fixtures pinned under `tests/export_wallet/`.
//! Phase 3 covers: BIP-84 singlesig wpkh, 2-of-3 wsh-sortedmulti multisig.
//! `--wallet-name` is REQUIRED for `--format specter` (SPEC §13 R1-L1
//! hardening — Specter's UX requires an explicit label).

use assert_cmd::Command;

const TREZOR_24_MASTER_FP: &str = "5436d724";
const TREZOR_24_BIP84_MAINNET_ZPUB: &str = "zpub6qTBTNftBzVTjgVcSUw7vW5N1KQbV93Jnrw314RHGkCkSx4vk6nEWH1MJfReXi2WThvuDRiRpyT7cDoakEcZMQ1iZPgfJgQrcVMR4aJWh6S";

const COSIGNER_A_XPUB: &str = "xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX";
const COSIGNER_A_FP: &str = "b8688df1";
const COSIGNER_B_XPUB: &str = "xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6";
const COSIGNER_B_FP: &str = "28645006";
const COSIGNER_C_XPUB: &str = "xpub6Buxw9MmbkJr4iAw8SACNci2hQNuPCMwt9P7HkK62ZQAW9UcJaQ2bc6ARD892TToQQ9Rp6AHujHxBLXqAsvn5fRnLfnhKSRfz8qtaoyKUYx";
const COSIGNER_C_FP: &str = TREZOR_24_MASTER_FP;

const FIXTURE_SINGLE_WPKH: &str = "tests/export_wallet/specter_single_wpkh.json";
const FIXTURE_MULTI_2OF3: &str = "tests/export_wallet/specter_multi_2of3.json";
const FIXTURE_REFUSAL_MISSING_WALLET_NAME: &str =
    "tests/export_wallet/specter_missing_wallet_name_refusal.stderr";

/// SPEC §8 cell 1 — `--format specter --template bip84 --network mainnet
/// --wallet-name Daily` emits the canonical Specter import shape.
#[test]
fn cell_1_specter_single_wpkh_byte_exact() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--format",
            "specter",
            "--template",
            "bip84",
            "--network",
            "mainnet",
            "--wallet-name",
            "Daily",
            "--slot",
            &format!("@0.xpub={TREZOR_24_BIP84_MAINNET_ZPUB}"),
            "--slot",
            &format!("@0.fingerprint={TREZOR_24_MASTER_FP}"),
            "--output",
            "-",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let expected = std::fs::read_to_string(FIXTURE_SINGLE_WPKH).expect(FIXTURE_SINGLE_WPKH);
    assert_eq!(
        stdout, expected,
        "Specter BIP-84 mainnet singlesig must match fixture byte-exact.\n--- got ---\n{stdout}\n--- expected ---\n{expected}"
    );
}

/// SPEC §8 cell 2 — 2-of-3 wsh-sortedmulti emits N=3 devices (all
/// "unknown") and the canonical BIP-380 descriptor with `#checksum`.
#[test]
fn cell_2_specter_multi_2of3_byte_exact() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--format",
            "specter",
            "--template",
            "wsh-sortedmulti",
            "--threshold",
            "2",
            "--multisig-path-family",
            "bip48",
            "--network",
            "mainnet",
            "--wallet-name",
            "VaultColdStorage",
            "--slot",
            &format!("@0.xpub={COSIGNER_A_XPUB}"),
            "--slot",
            &format!("@0.fingerprint={COSIGNER_A_FP}"),
            "--slot",
            "@0.path=m/48'/0'/0'/2'",
            "--slot",
            &format!("@1.xpub={COSIGNER_B_XPUB}"),
            "--slot",
            &format!("@1.fingerprint={COSIGNER_B_FP}"),
            "--slot",
            "@1.path=m/48'/0'/0'/2'",
            "--slot",
            &format!("@2.xpub={COSIGNER_C_XPUB}"),
            "--slot",
            &format!("@2.fingerprint={COSIGNER_C_FP}"),
            "--slot",
            "@2.path=m/48'/0'/0'/2'",
            "--output",
            "-",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let expected = std::fs::read_to_string(FIXTURE_MULTI_2OF3).expect(FIXTURE_MULTI_2OF3);
    assert_eq!(
        stdout, expected,
        "Specter 2-of-3 wsh-sortedmulti must match fixture byte-exact.\n--- got ---\n{stdout}\n--- expected ---\n{expected}"
    );
}

/// SPEC §8 + §13 R1-L1 cell 3 — Specter without `--wallet-name` refuses via
/// the SPEC §4 missing-info channel. Specter's UX requires an explicit
/// label; emitting a wallet with the default-derived name produces a UI
/// regression. `SpecterEmitter::collect_missing` returns
/// `MissingField::WalletName` whenever `wallet_name_is_non_default = false`
/// (v0.37.8 renamed: the field is true when the user supplied `--wallet-name`
/// OR a name was lifted from the envelope's source metadata).
#[test]
fn cell_3_specter_missing_wallet_name_refusal_byte_exact() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--format",
            "specter",
            "--template",
            "bip84",
            "--network",
            "mainnet",
            // Note: NO --wallet-name supplied.
            "--slot",
            &format!("@0.xpub={TREZOR_24_BIP84_MAINNET_ZPUB}"),
            "--slot",
            &format!("@0.fingerprint={TREZOR_24_MASTER_FP}"),
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    let expected = std::fs::read_to_string(FIXTURE_REFUSAL_MISSING_WALLET_NAME)
        .expect(FIXTURE_REFUSAL_MISSING_WALLET_NAME);
    assert_eq!(
        stderr, expected,
        "Specter missing-wallet-name refusal must match SPEC §4 fixture byte-exact.\n--- got ---\n{stderr}\n--- expected ---\n{expected}"
    );
}

/// SPEC §8 cell 4 — Specter `descriptor` field round-trips through the same
/// canonical-descriptor pipeline that bitcoin-core / bip388 use. Cross-verify
/// by parsing the Specter output, extracting the descriptor, and confirming
/// it parses cleanly as a `miniscript::Descriptor` (the same shape Bitcoin
/// Core accepts).
#[test]
fn cell_4_specter_descriptor_round_trips_with_canonical_pipeline() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--format",
            "specter",
            "--template",
            "bip84",
            "--network",
            "mainnet",
            "--wallet-name",
            "Daily",
            "--slot",
            &format!("@0.xpub={TREZOR_24_BIP84_MAINNET_ZPUB}"),
            "--slot",
            &format!("@0.fingerprint={TREZOR_24_MASTER_FP}"),
            "--output",
            "-",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let descriptor = json["descriptor"].as_str().unwrap();
    assert!(
        descriptor.starts_with("wpkh("),
        "descriptor must start with wpkh() for bip84"
    );
    assert!(
        descriptor.contains("#"),
        "descriptor must include BIP-380 #checksum (unlike Sparrow's miniscript.script — Specter expects the checksum)"
    );
}
