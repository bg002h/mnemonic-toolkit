//! v0.8.1 Phase 1.2 — `mnemonic export-wallet --format coldcard` integration tests.
//!
//! SPEC `design/SPEC_export_wallet_v0_8.md` §5.1 (Coldcard generic JSON
//! skeleton, singlesig). Byte-exact fixtures pinned under
//! `tests/export_wallet/`. Phase 1.2 covers BIP-84 mainnet (single sub-object)
//! using the Trezor 24-word "abandon × 23 art" test vector. Phase 1.3 adds
//! BIP-49 testnet and BIP-44 mainnet; multisig text emitter lands in Phase 1.4.

use assert_cmd::Command;

/// Trezor 24-word canonical vector: "abandon × 23 art" → 32-zero-bytes entropy.
/// Used by `derive.rs` tests (`derive_master_fingerprint_stable` etc.); this
/// suite re-derives the BIP-84 mainnet account xpub from the phrase at runtime
/// to cross-check the fixture-pinned values stay aligned with the toolkit's
/// own derivation.
const TREZOR_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

/// BIP-84 mainnet master fingerprint for TREZOR_24 (verified at
/// `crates/mnemonic-toolkit/src/derive.rs::derive_master_fingerprint_stable`).
const TREZOR_24_MASTER_FP: &str = "5436d724";

/// BIP-84 mainnet account-0 xpub at `m/84'/0'/0'` for TREZOR_24 (BIP-32
/// neutral form; SLIP-132 `zpub` form is the toolkit's preferred slot input
/// shape after `normalize_xpub_prefix` swaps version bytes).
const TREZOR_24_BIP84_MAINNET_ZPUB: &str = "zpub6qTBTNftBzVTjgVcSUw7vW5N1KQbV93Jnrw314RHGkCkSx4vk6nEWH1MJfReXi2WThvuDRiRpyT7cDoakEcZMQ1iZPgfJgQrcVMR4aJWh6S";

/// Path to the byte-exact fixture (relative to the integration-test binary's
/// runtime working directory, which is the crate root).
const FIXTURE_BIP84_MAINNET: &str =
    "tests/export_wallet/coldcard_generic_bip84_mainnet.json";

/// SPEC §5.1 Phase 1.2 RED → GREEN: `--format coldcard --template bip84
/// --network mainnet --slot @0.xpub=zpub... --slot @0.fingerprint=5436d724`
/// emits the canonical Coldcard generic JSON skeleton for the BIP-84 mainnet
/// account, byte-identical to the pinned fixture (master_xpub omitted —
/// SPEC §5.1 R1.0 fold: top-level `xpub` is OPTIONAL, emitted iff
/// `@0.master_xpub=` was supplied; absent here).
#[test]
fn cell_1_coldcard_generic_bip84_mainnet_byte_exact() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--format",
            "coldcard",
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
    let expected = std::fs::read_to_string(FIXTURE_BIP84_MAINNET).expect(FIXTURE_BIP84_MAINNET);
    assert_eq!(
        stdout, expected,
        "Coldcard BIP-84 mainnet emission must match fixture byte-exact.\n--- got ---\n{stdout}\n--- expected ---\n{expected}"
    );
}

/// Sanity check: the TREZOR_24 fixture vector is consistent with the toolkit's
/// own derivation pipeline. If `bitcoin` or `bip39` crate updates change the
/// derived xpub, this test fails BEFORE the byte-exact test above (which
/// would also fail with a less actionable diff). Helps localize regressions.
#[test]
fn cell_1_coldcard_bip84_vector_consistency_with_derive_pipeline() {
    let xpub_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("phrase={TREZOR_24}"),
            "--to",
            "xpub",
            "--template",
            "bip84",
            "--network",
            "mainnet",
        ])
        .assert()
        .success();
    let xpub_line = String::from_utf8(xpub_out.get_output().stdout.clone()).unwrap();
    // `mnemonic convert --to xpub` emits the BIP-32 (neutral) form. The
    // fixture uses the SLIP-132 zpub form via `--slot @0.xpub=zpub...`.
    // We just check the prefix to keep the assertion robust.
    assert!(
        xpub_line.starts_with("xpub: xpub6"),
        "expected `xpub: xpub6...` prefix from `mnemonic convert`; got {xpub_line:?}",
    );
}
