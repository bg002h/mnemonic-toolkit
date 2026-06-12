//! v0.8.1 Phase 2 — `mnemonic export-wallet --format sparrow` integration tests.
//!
//! SPEC `design/SPEC_export_wallet_v0_8.md` §7 (Sparrow Wallet wallet-import
//! emitter). Byte-exact fixtures pinned under `tests/export_wallet/`.
//! Phase 2 covers: BIP-84 singlesig wpkh, BIP-86 singlesig p2tr, and 2-of-3
//! wsh-sortedmulti multisig. Plus the SPEC §4 missing-threshold refusal
//! channel (first emitter to exercise `ToolkitError::ExportWalletMissingFields`
//! end-to-end through the CLI).

use assert_cmd::Command;

const TREZOR_24_MASTER_FP: &str = "5436d724";
const TREZOR_24_BIP84_MAINNET_ZPUB: &str = "zpub6qTBTNftBzVTjgVcSUw7vW5N1KQbV93Jnrw314RHGkCkSx4vk6nEWH1MJfReXi2WThvuDRiRpyT7cDoakEcZMQ1iZPgfJgQrcVMR4aJWh6S";
const TREZOR_24_BIP86_MAINNET_XPUB: &str = "xpub6CAYwo2AfKJy1cdFGBAgLvCrZULhEkZ9C9s4GGXwXzHvNPguMWBcVrGEDjP2ZJdX92gVWLeLrNVVmipTrKqrwMy2eT282xKEyHMbPDrcD9e";

const COSIGNER_A_XPUB: &str = "xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX";
const COSIGNER_A_FP: &str = "b8688df1";
const COSIGNER_B_XPUB: &str = "xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6";
const COSIGNER_B_FP: &str = "28645006";
const COSIGNER_C_XPUB: &str = "xpub6Buxw9MmbkJr4iAw8SACNci2hQNuPCMwt9P7HkK62ZQAW9UcJaQ2bc6ARD892TToQQ9Rp6AHujHxBLXqAsvn5fRnLfnhKSRfz8qtaoyKUYx";
const COSIGNER_C_FP: &str = TREZOR_24_MASTER_FP;

const FIXTURE_SINGLE_WPKH: &str = "tests/export_wallet/sparrow_single_wpkh.json";
const FIXTURE_SINGLE_TR: &str = "tests/export_wallet/sparrow_single_tr.json";
const FIXTURE_MULTI_2OF3_WSH: &str = "tests/export_wallet/sparrow_multi_2of3_wsh_sortedmulti.json";
const FIXTURE_REFUSAL_MISSING_THRESHOLD: &str =
    "tests/export_wallet/sparrow_missing_threshold_refusal.stderr";
const FIXTURE_TR_MULTI_A_NUMS_2OF3: &str = "tests/export_wallet/sparrow_tr_multi_a_nums_2of3.json";

/// SPEC §7 cell 1 — `--format sparrow --template bip84 --network mainnet`
/// emits the canonical Sparrow wallet JSON for the BIP-84 mainnet account.
#[test]
fn cell_1_sparrow_single_wpkh_byte_exact() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--format",
            "sparrow",
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
    let expected = std::fs::read_to_string(FIXTURE_SINGLE_WPKH).expect(FIXTURE_SINGLE_WPKH);
    assert_eq!(
        stdout, expected,
        "Sparrow BIP-84 mainnet singlesig must match fixture byte-exact.\n--- got ---\n{stdout}\n--- expected ---\n{expected}"
    );
}

/// SPEC §7 cell 2 — `--format sparrow --template bip86` emits the p2tr
/// singlesig shape with `scriptType: P2TR` and `script: tr(@0/**)`.
#[test]
fn cell_2_sparrow_single_tr_byte_exact() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--format",
            "sparrow",
            "--template",
            "bip86",
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.xpub={TREZOR_24_BIP86_MAINNET_XPUB}"),
            "--slot",
            &format!("@0.fingerprint={TREZOR_24_MASTER_FP}"),
            "--output",
            "-",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let expected = std::fs::read_to_string(FIXTURE_SINGLE_TR).expect(FIXTURE_SINGLE_TR);
    assert_eq!(
        stdout, expected,
        "Sparrow BIP-86 mainnet singlesig p2tr must match fixture byte-exact.\n--- got ---\n{stdout}\n--- expected ---\n{expected}"
    );
}

/// SPEC §7 cell 3 — 2-of-3 wsh-sortedmulti emits `policyType: MULTI`,
/// `scriptType: P2WSH`, `script: wsh(sortedmulti(2,@0/**,@1/**,@2/**))`,
/// and N=3 keystores each with the cosigner's xpub + fingerprint + path.
#[test]
fn cell_3_sparrow_multi_2of3_wsh_sortedmulti_byte_exact() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--format",
            "sparrow",
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
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let expected = std::fs::read_to_string(FIXTURE_MULTI_2OF3_WSH).expect(FIXTURE_MULTI_2OF3_WSH);
    assert_eq!(
        stdout, expected,
        "Sparrow 2-of-3 wsh-sortedmulti must match fixture byte-exact.\n--- got ---\n{stdout}\n--- expected ---\n{expected}"
    );
}

/// SPEC §4 + §7 + §13 cell 4 — Sparrow multisig WITHOUT `--threshold` triggers
/// the SPEC §4 missing-info refusal channel. This is the first emitter to
/// exercise `ToolkitError::ExportWalletMissingFields` end-to-end through the
/// CLI (Coldcard + Jade route their format-template incompat via the more
/// helpful `BadInput` pointer; Sparrow uses the §4 channel because the
/// missing field is GENUINELY missing user input, not format-template
/// incompatibility).
#[test]
fn cell_4_sparrow_missing_threshold_refusal_byte_exact() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--format",
            "sparrow",
            "--template",
            "wsh-sortedmulti",
            // Note: NO --threshold supplied.
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
        .failure()
        .code(2);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    let expected = std::fs::read_to_string(FIXTURE_REFUSAL_MISSING_THRESHOLD)
        .expect(FIXTURE_REFUSAL_MISSING_THRESHOLD);
    assert_eq!(
        stderr, expected,
        "Sparrow missing-threshold refusal must match SPEC §4 fixture byte-exact.\n--- got ---\n{stderr}\n--- expected ---\n{expected}"
    );
}

/// SPEC §7 cell 5 — Sparrow + tr-multi-a uses descriptor-passthrough for the
/// miniscript script. Phase 2 R1 C-1 fold: the `#checksum` suffix is stripped
/// before placement in `defaultPolicy.miniscript.script` (Sparrow's policy
/// parser expects a bare miniscript expression, not a BIP-380 descriptor).
/// Byte-exact assertion against the pinned fixture (R1-I1 fold: the earlier
/// structural-only rationale was factually incorrect — BIP-380 checksums are
/// deterministic over fixed xpub inputs and the fixture is perfectly
/// byte-pinneable).
#[test]
fn cell_5_sparrow_tr_multi_a_nums_2of3_byte_exact() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--format",
            "sparrow",
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
            "--output",
            "-",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let expected =
        std::fs::read_to_string(FIXTURE_TR_MULTI_A_NUMS_2OF3).expect(FIXTURE_TR_MULTI_A_NUMS_2OF3);
    assert_eq!(
        stdout, expected,
        "Sparrow tr-multi-a NUMS 2-of-3 must match fixture byte-exact (with #checksum stripped per C-1).\n--- got ---\n{stdout}\n--- expected ---\n{expected}"
    );
    // Additional invariant: SPEC §7 says script must NOT include BIP-380
    // `#checksum` suffix (Sparrow's policy parser would reject it).
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let script = json["defaultPolicy"]["miniscript"]["script"]
        .as_str()
        .unwrap();
    assert!(
        !script.contains('#'),
        "miniscript.script must NOT include BIP-380 #checksum suffix (Phase 2 R1 C-1); got {script:?}",
    );
}
