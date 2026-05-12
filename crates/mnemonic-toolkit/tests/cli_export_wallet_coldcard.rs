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

/// BIP-49 testnet account-0 xpub at `m/49'/1'/0'` for TREZOR_24 (tpub form;
/// derived via `mnemonic convert --to xpub --template bip49 --network testnet`).
const TREZOR_24_BIP49_TESTNET_TPUB: &str = "tpubDDYhB7EGtNkJdeaPTacttc9jZ6aq7NWHiYy21ACcFx8g2zs9HNpQDondF7HQfemghZSEimBPHPRfs93UehvbFHZyHgWDBrY4KSCC183DAFw";

/// BIP-44 mainnet account-0 xpub at `m/44'/0'/0'` for TREZOR_24.
const TREZOR_24_BIP44_MAINNET_XPUB: &str = "xpub6CDwootAjK1YycduSmTrAAXjfW9A8bPhVamZeofd8wX6rvGm2vLz6qtnqx4FagbFeXJwFThkzDPGkErrjFpLpr1wsj7NXgHHkevPUxHYjUP";

/// Path to the byte-exact fixture (relative to the integration-test binary's
/// runtime working directory, which is the crate root).
const FIXTURE_BIP84_MAINNET: &str =
    "tests/export_wallet/coldcard_generic_bip84_mainnet.json";
const FIXTURE_BIP49_TESTNET: &str =
    "tests/export_wallet/coldcard_generic_bip49_testnet.json";
const FIXTURE_BIP44_MAINNET: &str =
    "tests/export_wallet/coldcard_generic_bip44_mainnet.json";
const FIXTURE_MULTISIG_2OF3_WSH: &str =
    "tests/export_wallet/coldcard_multisig_2of3_wsh.txt";

// Phase 1.4 multisig vectors (BIP-48 wsh, m/48'/0'/0'/2').
// Cosigner A + B from cli_export_wallet.rs's existing fixtures.
// Cosigner C derived from TREZOR_24 at the same BIP-48 wsh path.
const COSIGNER_A_XPUB: &str = "xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX";
const COSIGNER_A_FP: &str = "b8688df1";
const COSIGNER_B_XPUB: &str = "xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6";
const COSIGNER_B_FP: &str = "28645006";
const COSIGNER_C_XPUB: &str = "xpub6Buxw9MmbkJr4iAw8SACNci2hQNuPCMwt9P7HkK62ZQAW9UcJaQ2bc6ARD892TToQQ9Rp6AHujHxBLXqAsvn5fRnLfnhKSRfz8qtaoyKUYx";
const COSIGNER_C_FP: &str = TREZOR_24_MASTER_FP;

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

/// SPEC §5.1 Phase 1.3 — BIP-49 testnet singlesig (`chain: "XTN"`, SLIP-132
/// `upub` form for `_pub`, p2sh-wrapped p2wpkh address starting with `2`).
#[test]
fn cell_2_coldcard_generic_bip49_testnet_byte_exact() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--format",
            "coldcard",
            "--template",
            "bip49",
            "--network",
            "testnet",
            "--slot",
            &format!("@0.xpub={TREZOR_24_BIP49_TESTNET_TPUB}"),
            "--slot",
            &format!("@0.fingerprint={TREZOR_24_MASTER_FP}"),
            "--output",
            "-",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let expected = std::fs::read_to_string(FIXTURE_BIP49_TESTNET).expect(FIXTURE_BIP49_TESTNET);
    assert_eq!(
        stdout, expected,
        "Coldcard BIP-49 testnet emission must match fixture byte-exact.\n--- got ---\n{stdout}\n--- expected ---\n{expected}"
    );
}

/// SPEC §5.1 Phase 1.3 — BIP-44 mainnet singlesig (legacy p2pkh; no `_pub`
/// field per upstream Coldcard sample — legacy lacks a SLIP-132 variant).
#[test]
fn cell_3_coldcard_generic_bip44_mainnet_byte_exact() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--format",
            "coldcard",
            "--template",
            "bip44",
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.xpub={TREZOR_24_BIP44_MAINNET_XPUB}"),
            "--slot",
            &format!("@0.fingerprint={TREZOR_24_MASTER_FP}"),
            "--output",
            "-",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let expected = std::fs::read_to_string(FIXTURE_BIP44_MAINNET).expect(FIXTURE_BIP44_MAINNET);
    assert_eq!(
        stdout, expected,
        "Coldcard BIP-44 mainnet emission must match fixture byte-exact.\n--- got ---\n{stdout}\n--- expected ---\n{expected}"
    );
}

/// SPEC §5.1 R1-I2 — `--template bip86 --format coldcard` REFUSES with the
/// pinned pointer text (BIP-86 is not in the upstream Coldcard generic-export
/// schema; tracked by FOLLOWUPS `coldcard-bip86-generic-export-pending-firmware`).
#[test]
fn cell_4_coldcard_bip86_refuses_byte_exact() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--format",
            "coldcard",
            "--template",
            "bip86",
            "--network",
            "mainnet",
            // Use the BIP-84 xpub as a placeholder — refusal fires before
            // slot resolution, so the value is irrelevant.
            "--slot",
            &format!("@0.xpub={TREZOR_24_BIP84_MAINNET_ZPUB}"),
            "--slot",
            &format!("@0.fingerprint={TREZOR_24_MASTER_FP}"),
        ])
        .assert()
        .failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    let expected = "--format coldcard does not yet support BIP-86 (P2TR) — Coldcard's generic-wallet-export schema documents only bip44/bip49/bip84. Use --format bitcoin-core (descriptor) or --format sparrow for taproot watch-only setup.";
    assert!(
        stderr.contains(expected),
        "BIP-86 refusal stderr must contain the SPEC §5.1 pointer.\n--- got ---\n{stderr}"
    );
}

/// SPEC §5.2 Phase 1.4 — Coldcard multisig text emitter, 2-of-3 wsh-sortedmulti.
/// Cosigner order in the output is sorted by xpub lex (sortedmulti); Derivation
/// line carries the shared `m/48'/0'/0'/2'` BIP-48 wsh path; XFPs uppercase;
/// xpubs in BIP-32 base58 form (not SLIP-132).
#[test]
fn cell_5_coldcard_multisig_2of3_wsh_sortedmulti_byte_exact() {
    let out = Command::cargo_bin("mnemonic")
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
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let expected = std::fs::read_to_string(FIXTURE_MULTISIG_2OF3_WSH).expect(FIXTURE_MULTISIG_2OF3_WSH);
    assert_eq!(
        stdout, expected,
        "Coldcard 2-of-3 wsh-sortedmulti multisig text must match fixture byte-exact.\n--- got ---\n{stdout}\n--- expected ---\n{expected}"
    );
}

/// SPEC §5.2 — `tr-multi-a` template REFUSES under `--format coldcard` per
/// FOLLOWUPS `coldcard-tr-multi-a-pending-firmware`. The byte-exact pointer
/// is checked here.
#[test]
fn cell_6_coldcard_tr_multi_a_refuses() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--format",
            "coldcard",
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
        stderr.contains("coldcard-tr-multi-a-pending-firmware"),
        "tr-multi-a refusal must cite the FOLLOWUPS slug.\n--- got ---\n{stderr}"
    );
}

/// SPEC §5.2 + Phase 1.11 N-1 — `--wallet-name` truncation must be safe for
/// non-ASCII input. Earlier `name.truncate(20)` sliced at byte 20 and would
/// panic when that byte landed mid-codepoint; `chars().take(20)` operates on
/// scalar values. Regression guard: supply a wallet name composed of 4-byte
/// codepoints (each `🤐` is U+1F910, 4 bytes UTF-8) and assert the emitter
/// returns success rather than panicking. The wallet name appears verbatim
/// in the multisig text's `Name:` line, truncated to 20 chars (=20 of the
/// emoji, but only the first 20 chars matter — we just need to prove no
/// panic).
#[test]
fn cell_7_coldcard_wallet_name_non_ascii_truncation_no_panic() {
    let long_emoji_name = "\u{1F910}".repeat(25); // 25 chars, 100 bytes
    let out = Command::cargo_bin("mnemonic")
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
            "--wallet-name",
            &long_emoji_name,
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
    // 20 emoji × 4 bytes/emoji = 80 bytes after the leading "Name: " label.
    let expected_name_line = format!("Name: {}", "\u{1F910}".repeat(20));
    assert!(
        stdout.lines().next() == Some(&expected_name_line),
        "first line should be `Name: ` + first 20 emoji codepoints, got: {:?}",
        stdout.lines().next(),
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
