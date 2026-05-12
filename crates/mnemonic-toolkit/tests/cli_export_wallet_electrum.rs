//! v0.8.1 Phase 4 — `mnemonic export-wallet --format electrum` integration tests.
//!
//! SPEC `design/SPEC_export_wallet_v0_8.md` §9 (Electrum wallet-db emitter).
//! Byte-exact fixtures pinned under `tests/export_wallet/`. Phase 4 covers:
//! singlesig BIP-84 (SLIP-132 `zpub` form), 2-of-4 wsh-sortedmulti multisig
//! (SLIP-132 multisig `Zpub` form), and the SPEC §9.2 tr-multi-a refusal.
//!
//! `ELECTRUM_SEED_VERSION_PIN` is currently `17` per Phase 4 step 0 deferral
//! (FOLLOWUPS `electrum-seed-version-spike-pending`).

use assert_cmd::Command;

const TREZOR_24_MASTER_FP: &str = "5436d724";
const TREZOR_24_BIP84_MAINNET_ZPUB: &str = "zpub6qTBTNftBzVTjgVcSUw7vW5N1KQbV93Jnrw314RHGkCkSx4vk6nEWH1MJfReXi2WThvuDRiRpyT7cDoakEcZMQ1iZPgfJgQrcVMR4aJWh6S";

const COSIGNER_A_XPUB: &str = "xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX";
const COSIGNER_A_FP: &str = "b8688df1";
const COSIGNER_B_XPUB: &str = "xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6";
const COSIGNER_B_FP: &str = "28645006";
const COSIGNER_C_XPUB: &str = "xpub6Buxw9MmbkJr4iAw8SACNci2hQNuPCMwt9P7HkK62ZQAW9UcJaQ2bc6ARD892TToQQ9Rp6AHujHxBLXqAsvn5fRnLfnhKSRfz8qtaoyKUYx";
const COSIGNER_C_FP: &str = TREZOR_24_MASTER_FP;
/// Derived from "test test test test test test test test test test test junk"
/// (Hardhat-default test mnemonic) at the bip48 wsh path.
const COSIGNER_D_XPUB: &str = "xpub6Bv8ayijom26yJ1wZ62h4X1smfYBfNeNtGujxw6vaY4zq4Tw4cn2oV8qZmjnuVxh56oSe21r7V8r9LjZjArFh3QRZQbgzgLcfjVikZNa86W";
const COSIGNER_D_FP: &str = "16a93ed0";

const FIXTURE_SINGLE: &str = "tests/export_wallet/electrum_single.json";
const FIXTURE_MULTI_2OF4: &str = "tests/export_wallet/electrum_multi_2of4.json";
const FIXTURE_REFUSAL_TR_MULTI_A: &str = "tests/export_wallet/electrum_tr_multi_a_refusal.stderr";

/// SPEC §9.1 cell 1 — Electrum singlesig standard wallet: BIP-84 mainnet,
/// `wallet_type: "standard"`, SLIP-132 `zpub` form in `keystore.xpub`.
#[test]
fn cell_1_electrum_single_bip84_byte_exact() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--format",
            "electrum",
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
    let expected = std::fs::read_to_string(FIXTURE_SINGLE).expect(FIXTURE_SINGLE);
    assert_eq!(
        stdout, expected,
        "Electrum BIP-84 mainnet singlesig must match fixture byte-exact (zpub form, seed_version 17).\n--- got ---\n{stdout}\n--- expected ---\n{expected}"
    );
}

/// SPEC §9.2 cell 2 — Electrum 2-of-4 wsh-sortedmulti multisig:
/// `wallet_type: "2of4"`, four `x1/`..`x4/` keystores, each with SLIP-132
/// multisig `Zpub` form xpub (capital Z indicates wsh multisig).
#[test]
fn cell_2_electrum_multi_2of4_wsh_byte_exact() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--format",
            "electrum",
            "--template",
            "wsh-sortedmulti",
            "--threshold",
            "2",
            "--multisig-path-family",
            "bip48",
            "--network",
            "mainnet",
            "--wallet-name",
            "VaultCold",
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
            "--slot",
            &format!("@3.xpub={COSIGNER_D_XPUB}"),
            "--slot",
            &format!("@3.fingerprint={COSIGNER_D_FP}"),
            "--slot",
            "@3.path=m/48'/0'/0'/2'",
            "--output",
            "-",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let expected = std::fs::read_to_string(FIXTURE_MULTI_2OF4).expect(FIXTURE_MULTI_2OF4);
    assert_eq!(
        stdout, expected,
        "Electrum 2-of-4 wsh-sortedmulti must match fixture byte-exact (capital Zpub form).\n--- got ---\n{stdout}\n--- expected ---\n{expected}"
    );
}

/// SPEC §9.2 cell 3 — Electrum + tr-multi-a REFUSES per FOLLOWUPS
/// `electrum-tr-multi-a-pending-libsecp-taproot`. Byte-exact stderr.
#[test]
fn cell_3_electrum_tr_multi_a_refuses_byte_exact() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--format",
            "electrum",
            "--template",
            "tr-multi-a",
            "--threshold",
            "2",
            "--multisig-path-family",
            "bip87",
            "--network",
            "mainnet",
            "--taproot-internal-key",
            "nums",
            "--slot",
            &format!("@0.xpub={COSIGNER_A_XPUB}"),
            "--slot",
            &format!("@0.fingerprint={COSIGNER_A_FP}"),
            "--slot",
            &format!("@1.xpub={COSIGNER_B_XPUB}"),
            "--slot",
            &format!("@1.fingerprint={COSIGNER_B_FP}"),
            "--slot",
            &format!("@2.xpub={COSIGNER_C_XPUB}"),
            "--slot",
            &format!("@2.fingerprint={COSIGNER_C_FP}"),
        ])
        .assert()
        .failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    let expected = std::fs::read_to_string(FIXTURE_REFUSAL_TR_MULTI_A)
        .expect(FIXTURE_REFUSAL_TR_MULTI_A);
    assert_eq!(
        stderr, expected,
        "Electrum tr-multi-a refusal must match fixture byte-exact (cites FOLLOWUPS slug).\n--- got ---\n{stderr}\n--- expected ---\n{expected}"
    );
}

/// SPEC §9 cell 4 — SLIP-132 round-trip via user-supplied SLIP-132 input.
/// Supply `--slot @0.xpub=zpub...` and verify the emitter's `keystore.xpub`
/// is the same zpub form (script-type × network matched).
#[test]
fn cell_4_electrum_slip132_round_trip_bip84() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--format",
            "electrum",
            "--template",
            "bip84",
            "--network",
            "mainnet",
            "--wallet-name",
            "RoundTrip",
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
    let emitted_xpub = json["keystore"]["xpub"].as_str().unwrap();
    assert_eq!(
        emitted_xpub, TREZOR_24_BIP84_MAINNET_ZPUB,
        "Electrum singlesig bip84 mainnet must emit zpub form matching user input (SLIP-132 round-trip).\n--- got ---\n{emitted_xpub}\n--- expected ---\n{TREZOR_24_BIP84_MAINNET_ZPUB}"
    );
}
