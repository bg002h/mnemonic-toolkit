//! v0.8.1 Phase 1.5 — `mnemonic export-wallet --format jade` integration tests.
//!
//! SPEC `design/SPEC_export_wallet_v0_8.md` §6 (Blockstream Jade wallet-import
//! emitter). Multisig path is byte-identical to Coldcard's §5.2 multisig text
//! (verified by `cell_1_jade_multisig_byte_equal_to_coldcard`). Singlesig
//! refuses; taproot-multisig refuses pending Jade firmware support.

use assert_cmd::Command;

const TREZOR_24_MASTER_FP: &str = "5436d724";
const TREZOR_24_BIP84_MAINNET_ZPUB: &str = "zpub6qTBTNftBzVTjgVcSUw7vW5N1KQbV93Jnrw314RHGkCkSx4vk6nEWH1MJfReXi2WThvuDRiRpyT7cDoakEcZMQ1iZPgfJgQrcVMR4aJWh6S";

const COSIGNER_A_XPUB: &str = "xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX";
const COSIGNER_A_FP: &str = "b8688df1";
const COSIGNER_B_XPUB: &str = "xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6";
const COSIGNER_B_FP: &str = "28645006";
const COSIGNER_C_XPUB: &str = "xpub6Buxw9MmbkJr4iAw8SACNci2hQNuPCMwt9P7HkK62ZQAW9UcJaQ2bc6ARD892TToQQ9Rp6AHujHxBLXqAsvn5fRnLfnhKSRfz8qtaoyKUYx";
const COSIGNER_C_FP: &str = TREZOR_24_MASTER_FP;

const FIXTURE_JADE_MULTISIG_2OF3_WSH: &str =
    "tests/export_wallet/jade_multisig_2of3_wsh.txt";
const FIXTURE_COLDCARD_MULTISIG_2OF3_WSH: &str =
    "tests/export_wallet/coldcard_multisig_2of3_wsh.txt";

fn run_jade_multisig() -> String {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--format",
            "jade",
            "--template",
            "wsh-sortedmulti",
            "--threshold",
            "2",
            "--multisig-path-family",
            "bip48",
            "--network",
            "mainnet",
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
    String::from_utf8(out.get_output().stdout.clone()).unwrap()
}

/// SPEC §6 Phase 1.5 — `--format jade` 2-of-3 wsh-sortedmulti emission matches
/// the pinned Jade fixture byte-exact.
#[test]
fn cell_1_jade_multisig_2of3_wsh_byte_exact() {
    let stdout = run_jade_multisig();
    let expected = std::fs::read_to_string(FIXTURE_JADE_MULTISIG_2OF3_WSH)
        .expect(FIXTURE_JADE_MULTISIG_2OF3_WSH);
    assert_eq!(
        stdout, expected,
        "Jade 2-of-3 wsh-sortedmulti emission must match fixture byte-exact.\n--- got ---\n{stdout}\n--- expected ---\n{expected}"
    );
}

/// SPEC §6 — the Jade multisig text and Coldcard multisig text fixtures are
/// byte-identical (SPEC mandates this; Jade's `register_multisig.multisig_file`
/// is exactly the Coldcard shape).
#[test]
fn cell_2_jade_multisig_byte_equal_to_coldcard_fixture() {
    let jade = std::fs::read_to_string(FIXTURE_JADE_MULTISIG_2OF3_WSH)
        .expect(FIXTURE_JADE_MULTISIG_2OF3_WSH);
    let coldcard = std::fs::read_to_string(FIXTURE_COLDCARD_MULTISIG_2OF3_WSH)
        .expect(FIXTURE_COLDCARD_MULTISIG_2OF3_WSH);
    assert_eq!(
        jade, coldcard,
        "Jade and Coldcard multisig text fixtures must be byte-identical per SPEC §6"
    );
}

/// SPEC §6 — and the Jade emitter output and Coldcard emitter output are
/// byte-identical at runtime (not just via the pinned files). Cross-emitter
/// equivalence is the load-bearing invariant.
#[test]
fn cell_3_jade_emitter_output_byte_equal_to_coldcard_emitter_output() {
    let jade_stdout = run_jade_multisig();
    let coldcard_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--format",
            "coldcard",
            "--template",
            "wsh-sortedmulti",
            "--threshold",
            "2",
            "--multisig-path-family",
            "bip48",
            "--network",
            "mainnet",
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
    let coldcard_stdout = String::from_utf8(coldcard_out.get_output().stdout.clone()).unwrap();
    assert_eq!(
        jade_stdout, coldcard_stdout,
        "Jade and Coldcard multisig text emissions must be byte-identical (Jade delegates to Coldcard §5.2)"
    );
}

/// SPEC §6 — singlesig templates REFUSE under `--format jade` (Jade selects
/// address type on-device; no file-import surface for singlesig watch-only).
#[test]
fn cell_4_jade_singlesig_refuses_byte_exact() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--format",
            "jade",
            "--template",
            "bip84",
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.xpub={TREZOR_24_BIP84_MAINNET_ZPUB}"),
            "--slot",
            &format!("@0.fingerprint={TREZOR_24_MASTER_FP}"),
        ])
        .assert()
        .failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    // SPEC §6 byte-exact stderr with a SINGLE `error:` prefix. R1-C1 fold:
    // earlier test used `.contains(...)` which silently passed when the
    // emitter source double-prefixed the message.
    let expected = "error: mnemonic export-wallet --format jade emits multisig wallet config only; for singlesig setups Jade reads the seed on-device. Use --format coldcard for a singlesig JSON or --format bitcoin-core for a descriptor.";
    assert_eq!(
        stderr.trim_end(),
        expected,
        "Jade singlesig refusal must match SPEC §6 byte-exact (single `error:` prefix).\n--- got ---\n{stderr}"
    );
}

/// SPEC §6 — taproot multisig REFUSES under `--format jade` per FOLLOWUPS
/// `jade-tr-multi-a-pending-firmware`.
#[test]
fn cell_5_jade_tr_multi_a_refuses() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--format",
            "jade",
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
    assert!(
        stderr.contains("jade-tr-multi-a-pending-firmware"),
        "Jade tr-multi-a refusal must cite the FOLLOWUPS slug.\n--- got ---\n{stderr}"
    );
}
