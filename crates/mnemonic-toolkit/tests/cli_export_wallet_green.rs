//! v0.8.1 Phase 5 — `mnemonic export-wallet --format green` integration tests.
//!
//! SPEC `design/SPEC_export_wallet_v0_8.md` §10 (Blockstream Green
//! wallet-import emitter). Thin 3-line text emitter for singlesig;
//! multisig refuses with FOLLOWUPS pointer.

use assert_cmd::Command;

const TREZOR_24_MASTER_FP: &str = "5436d724";
const TREZOR_24_BIP84_MAINNET_ZPUB: &str = "zpub6qTBTNftBzVTjgVcSUw7vW5N1KQbV93Jnrw314RHGkCkSx4vk6nEWH1MJfReXi2WThvuDRiRpyT7cDoakEcZMQ1iZPgfJgQrcVMR4aJWh6S";

const COSIGNER_A_XPUB: &str = "xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX";
const COSIGNER_A_FP: &str = "b8688df1";
const COSIGNER_B_XPUB: &str = "xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6";
const COSIGNER_B_FP: &str = "28645006";
const COSIGNER_C_XPUB: &str = "xpub6Buxw9MmbkJr4iAw8SACNci2hQNuPCMwt9P7HkK62ZQAW9UcJaQ2bc6ARD892TToQQ9Rp6AHujHxBLXqAsvn5fRnLfnhKSRfz8qtaoyKUYx";
const COSIGNER_C_FP: &str = TREZOR_24_MASTER_FP;

const FIXTURE_DESCRIPTOR: &str = "tests/export_wallet/green_descriptor.txt";
const FIXTURE_REFUSAL_MULTISIG: &str = "tests/export_wallet/green_multisig_refusal.stderr";

/// SPEC §10 cell 1 — Green singlesig: 3-line text (2 comment lines + canonical
/// descriptor with `#checksum`). Bytes-exact match with pinned fixture.
#[test]
fn cell_1_green_singlesig_byte_exact() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--format",
            "green",
            "--template",
            "bip84",
            "--network",
            "mainnet",
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
    let expected = std::fs::read_to_string(FIXTURE_DESCRIPTOR).expect(FIXTURE_DESCRIPTOR);
    assert_eq!(
        stdout, expected,
        "Green singlesig descriptor file must match fixture byte-exact.\n--- got ---\n{stdout}\n--- expected ---\n{expected}"
    );
}

/// SPEC §10 cell 2 — Green multisig REFUSES (Green's multisig surface is
/// server-mediated via Green Multisig Shield, not file-import). Byte-exact
/// stderr citing FOLLOWUPS slug.
#[test]
fn cell_2_green_multisig_refuses_byte_exact() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--format",
            "green",
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
    let expected =
        std::fs::read_to_string(FIXTURE_REFUSAL_MULTISIG).expect(FIXTURE_REFUSAL_MULTISIG);
    assert_eq!(
        stderr, expected,
        "Green multisig refusal must match SPEC §10 fixture byte-exact (cites FOLLOWUPS slug).\n--- got ---\n{stderr}\n--- expected ---\n{expected}"
    );
}

/// v0.28.7 Slug 2 cell 4 — Green descriptor-mode (--from-import-json) REFUSES
/// multisig. Previously the refusal was gated on `inputs.template.is_some()`,
/// so descriptor-mode multisig (where template==None) silently produced output.
/// v0.28.7 changes the guard to `inputs.script_type.is_multisig()` which fires
/// for both template-mode and descriptor-mode.
///
/// FOLLOWUP `green-emitter-multisig-refusal-template-only` (resolved v0.28.7).
#[test]
fn cell_4_green_descriptor_mode_multisig_refuses() {
    // Step 1: import a multisig coldcard-multisig fixture to get the JSON envelope.
    let import_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--format",
            "coldcard-multisig",
            "--blob",
            "tests/fixtures/wallet_import/coldcard-ms-2of3-p2wsh-with-xfp.txt",
            "--json",
        ])
        .output()
        .expect("mnemonic import-wallet spawn");
    assert!(
        import_out.status.success(),
        "coldcard-multisig import must succeed; stderr: {}",
        String::from_utf8_lossy(&import_out.stderr)
    );

    // Step 2: export-wallet --format green --from-import-json - (stdin = envelope JSON).
    let export_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--format",
            "green",
            "--from-import-json",
            "-",
        ])
        .write_stdin(import_out.stdout)
        .output()
        .expect("mnemonic export-wallet spawn");
    assert_ne!(
        export_out.status.code(),
        Some(0),
        "descriptor-mode multisig must refuse, got success; stderr: {}",
        String::from_utf8_lossy(&export_out.stderr)
    );
    let stderr = String::from_utf8_lossy(&export_out.stderr);
    assert!(
        stderr.contains("does not support multisig"),
        "expected multisig-refusal stderr, got: {stderr}"
    );
}

/// SPEC §10 cell 3 — Green emits the canonical descriptor verbatim (includes
/// `#checksum`). Cross-verifies that the singlesig file body matches the
/// descriptor that bitcoin-core / specter would emit for the same inputs.
#[test]
fn cell_3_green_descriptor_includes_checksum() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--format",
            "green",
            "--template",
            "bip84",
            "--network",
            "mainnet",
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
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3, "Green file must be exactly 3 lines");
    assert!(
        lines[0].starts_with("# Blockstream Green"),
        "line 1 must be the header comment"
    );
    assert!(
        lines[1].starts_with("# Help: https://"),
        "line 2 must be the Help URL comment"
    );
    assert!(
        lines[2].starts_with("wpkh(") && lines[2].contains('#'),
        "line 3 must be the canonical descriptor with #checksum"
    );
}
