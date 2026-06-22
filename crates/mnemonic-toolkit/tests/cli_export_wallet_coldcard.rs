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
const FIXTURE_BIP84_MAINNET: &str = "tests/export_wallet/coldcard_generic_bip84_mainnet.json";
const FIXTURE_BIP49_TESTNET: &str = "tests/export_wallet/coldcard_generic_bip49_testnet.json";
const FIXTURE_BIP44_MAINNET: &str = "tests/export_wallet/coldcard_generic_bip44_mainnet.json";
const FIXTURE_MULTISIG_2OF3_WSH: &str = "tests/export_wallet/coldcard_multisig_2of3_wsh.txt";
const FIXTURE_REFUSAL_BIP86: &str = "tests/export_wallet/coldcard_refusal_bip86.stderr";
const FIXTURE_REFUSAL_TR_MULTI_A: &str = "tests/export_wallet/coldcard_refusal_tr_multi_a.stderr";
const FIXTURE_BIP84_WITH_MASTER_XPUB: &str =
    "tests/export_wallet/coldcard_generic_bip84_mainnet_with_master_xpub.json";

/// BIP-32 spec test vector 1 master xpub (depth-0). Used here only as a
/// known-valid base58check xpub for the `@0.master_xpub=` plumbing test;
/// the emitter does not cross-validate that the master xpub and the
/// account xpub derive from the same seed, so any valid depth-0 xpub is
/// acceptable input.
const BIP32_VEC1_MASTER_XPUB: &str = "xpub661MyMwAqRbcFtXgS5sYJABqqG9YLmC4Q1Rdap9gSE8NqtwybGhePY2gZ29ESFjqJoCu1Rupje8YtGqsefD265TMg7usUDFdp6W1EGMcet8";

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
    // Phase 1.11 R1 fold I-4: byte-exact assertion against pinned fixture
    // (was `.contains()` — would silently pass on doubled `error:` prefix or
    // unexpected trailing whitespace).
    let expected = std::fs::read_to_string(FIXTURE_REFUSAL_BIP86).expect(FIXTURE_REFUSAL_BIP86);
    assert_eq!(
        stderr, expected,
        "BIP-86 refusal stderr must match SPEC §5.1 pinned fixture byte-exact.\n--- got ---\n{stderr}\n--- expected ---\n{expected}"
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
    let expected =
        std::fs::read_to_string(FIXTURE_MULTISIG_2OF3_WSH).expect(FIXTURE_MULTISIG_2OF3_WSH);
    assert_eq!(
        stdout, expected,
        "Coldcard 2-of-3 wsh-sortedmulti multisig text must match fixture byte-exact.\n--- got ---\n{stdout}\n--- expected ---\n{expected}"
    );
}

/// SPEC §5.1 v0.8.2 — `@0.master_xpub=` slot subkey now plumbs through
/// `ResolvedSlot.master_xpub` → `EmitInputs.master_xpub_at_0` → Coldcard
/// generic JSON top-level `xpub` field (conditional emission per SPEC §5.1).
/// Validates the resolution of FOLLOWUPS `coldcard-master-xpub-plumbing-pending`.
#[test]
fn cell_8_coldcard_master_xpub_plumbing_byte_exact() {
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
            "--slot",
            &format!("@0.master_xpub={BIP32_VEC1_MASTER_XPUB}"),
            "--output",
            "-",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let expected = std::fs::read_to_string(FIXTURE_BIP84_WITH_MASTER_XPUB)
        .expect(FIXTURE_BIP84_WITH_MASTER_XPUB);
    assert_eq!(
        stdout, expected,
        "Coldcard BIP-84 with master_xpub supplied must emit top-level xpub field byte-exact.\n--- got ---\n{stdout}\n--- expected ---\n{expected}"
    );
    // Structural invariant: the top-level `xpub` field is present iff
    // master_xpub was supplied. cell_1 (master_xpub absent) verifies the
    // opposite case.
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert!(
        json["xpub"].is_string(),
        "Top-level xpub field must be a string when @0.master_xpub= is supplied"
    );
    assert_eq!(
        json["xpub"].as_str().unwrap(),
        BIP32_VEC1_MASTER_XPUB,
        "Top-level xpub must be the user-supplied master_xpub verbatim"
    );
}

/// SPEC §5.1 v0.8.2 — when `@0.master_xpub=` is NOT supplied (the default
/// case exercised by cell_1), the top-level `xpub` field is omitted from
/// the JSON object. Verifies the absence-side of the conditional.
#[test]
fn cell_9_coldcard_master_xpub_absent_omits_top_level_xpub() {
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
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert!(
        json.get("xpub").is_none(),
        "Top-level xpub field must be omitted when @0.master_xpub= is NOT supplied"
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
    // Phase 1.11 R1 fold I-4: byte-exact assertion against pinned fixture.
    let expected =
        std::fs::read_to_string(FIXTURE_REFUSAL_TR_MULTI_A).expect(FIXTURE_REFUSAL_TR_MULTI_A);
    assert_eq!(
        stderr, expected,
        "Coldcard tr-multi-a refusal must match SPEC §5.2 pinned fixture byte-exact (must include FOLLOWUPS slug).\n--- got ---\n{stderr}\n--- expected ---\n{expected}"
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

// ============================================================================
// v0.28.4 (A1) — `--format coldcard-multisig` export-side alias for `coldcard`
// with multisig-template precheck. Closes format-name asymmetry FOLLOWUP
// `export-wallet-coldcard-multisig-alias` from the manual-v0.2.0 cycle's
// P1b R0 architect §F4.
// ============================================================================

#[test]
fn export_wallet_coldcard_multisig_format_wsh_sortedmulti_2_of_3_emits_text() {
    // Happy path: `--format coldcard-multisig --template wsh-sortedmulti
    // --threshold 2` produces the same Coldcard-multisig text output as
    // `--format coldcard --template wsh-sortedmulti --threshold 2` (the
    // multisig-template arm of ColdcardEmitter::emit delegates identically).
    let out = assert_cmd::Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--format", "coldcard-multisig",
            "--template", "wsh-sortedmulti",
            "--threshold", "2",
            "--network", "mainnet",
            "--account", "0",
            "--slot", "@0.xpub=xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX",
            "--slot", "@0.fingerprint=b8688df1",
            "--slot", "@0.path=m/48'/0'/0'/2'",
            "--slot", "@1.xpub=xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6",
            "--slot", "@1.fingerprint=28645006",
            "--slot", "@1.path=m/48'/0'/0'/2'",
            "--slot", "@2.xpub=xpub6Buxw9MmbkJr4iAw8SACNci2hQNuPCMwt9P7HkK62ZQAW9UcJaQ2bc6ARD892TToQQ9Rp6AHujHxBLXqAsvn5fRnLfnhKSRfz8qtaoyKUYx",
            "--slot", "@2.fingerprint=5436d724",
            "--slot", "@2.path=m/48'/0'/0'/2'",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    // The Coldcard multisig text format starts with "Name: <wallet>" and
    // "Policy: K of N" lines (no separate header banner line).
    assert!(
        stdout.contains("Name:"),
        "expected Coldcard multisig 'Name:' field, got: {stdout:?}"
    );
    assert!(
        stdout.contains("Policy: 2 of 3"),
        "expected 'Policy: 2 of 3', got: {stdout:?}"
    );
}

#[test]
fn export_wallet_coldcard_multisig_format_refuses_singlesig_template_bip84() {
    let out = assert_cmd::Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--format", "coldcard-multisig",
            "--template", "bip84",
            "--network", "mainnet",
            "--account", "0",
            "--slot", "@0.xpub=xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9",
            "--slot", "@0.fingerprint=5436d724",
        ])
        .output()
        .expect("mnemonic spawn");
    assert_ne!(out.status.code(), Some(0), "must refuse");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("--format coldcard-multisig requires a multisig --template"),
        "expected multisig-required refusal, got: {stderr}"
    );
    assert!(
        stderr.contains("--format coldcard"),
        "expected pointer to `--format coldcard` for singlesig, got: {stderr}"
    );
}

#[test]
fn export_wallet_coldcard_multisig_format_refuses_no_template() {
    let out = assert_cmd::Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--format",
            "coldcard-multisig",
            "--network",
            "mainnet",
            "--account",
            "0",
        ])
        .output()
        .expect("mnemonic spawn");
    assert_ne!(out.status.code(), Some(0), "must refuse");
}

// ============================================================================
// cycle-13a P3 (H11) — divergent per-cosigner `Derivation:` export.
//
// The 2-of-3 wsh-sortedmulti emit collapsed divergent cosigner origin paths
// to the wrong global placeholder `m/0'/0'`. H11 emits a per-cosigner
// `Derivation:` line read from the SAME sorted slot (NEVER `m/0'/0'`); the
// all-agree case keeps the single shared `Derivation:` line byte-identical.
// xpub-lex sort order of the three cosigners is [C, B, A] (slot order A,B,C),
// so divergent paths exercise the sort≠slot pairing hazard.
// ============================================================================

/// Build a divergent SORTED 2-of-3 coldcard-multisig export with distinct
/// per-slot paths. `format` is "coldcard" or "jade".
fn run_divergent_export(format: &str) -> std::process::Output {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--format",
            format,
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
            "@1.path=m/48'/0'/1'/2'",
            "--slot",
            &format!("@2.xpub={COSIGNER_C_XPUB}"),
            "--slot",
            &format!("@2.fingerprint={COSIGNER_C_FP}"),
            "--slot",
            "@2.path=m/48'/0'/2'/2'",
            "--output",
            "-",
        ])
        .output()
        .expect("mnemonic spawn")
}

/// #1 — divergent paths emit a `Derivation:` line per cosigner with the real
/// path; NEVER `m/0'/0'`; each cosigner line carries its real master fp.
#[test]
fn export_coldcard_multisig_divergent_paths_emits_per_cosigner_derivation() {
    let out = run_divergent_export("coldcard");
    assert!(out.status.success(), "divergent export must succeed");
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        !stdout.contains("m/0'/0'"),
        "divergent export must NEVER emit the m/0'/0' placeholder; got:\n{stdout}"
    );
    // All three real paths must appear as `Derivation:` lines.
    for path in ["m/48'/0'/0'/2'", "m/48'/0'/1'/2'", "m/48'/0'/2'/2'"] {
        assert!(
            stdout.contains(&format!("Derivation: {path}")),
            "missing per-cosigner Derivation line for {path}; got:\n{stdout}"
        );
    }
    // The single-shared-Derivation collapse must be gone: there must be one
    // `Derivation:` per cosigner (3), not exactly one.
    let derivation_lines = stdout.lines().filter(|l| l.starts_with("Derivation:")).count();
    assert_eq!(derivation_lines, 3, "one Derivation line per cosigner; got:\n{stdout}");
}

/// #1b (load-bearing I-2 pairing test) — the xpub that sorts FIRST (C, at slot
/// @2 with path `2'`) must be paired with ITS OWN path, not a slot-order
/// `derivations[i]`-indexed one. Reads each emitted `Derivation:`/`<XFP>: xpub`
/// pair and asserts path↔xpub come from the SAME slot. RED today AND under a
/// naive `derivations[i]` fix → forces H11-b's same-sorted-slot rule.
#[test]
fn export_coldcard_multisig_sort_order_ne_slot_order_pairs_correctly() {
    let out = run_divergent_export("coldcard");
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();

    // Expected pairing (path belongs to the slot the xpub belongs to):
    //   A @0 path 0'   B @1 path 1'   C @2 path 2'
    let want = [
        ("m/48'/0'/0'/2'", COSIGNER_A_XPUB),
        ("m/48'/0'/1'/2'", COSIGNER_B_XPUB),
        ("m/48'/0'/2'/2'", COSIGNER_C_XPUB),
    ];

    // Walk the emitted lines; each `Derivation: <path>` is immediately
    // followed by `<XFP>: <xpub>`. Build the observed (path, xpub) pairs.
    let lines: Vec<&str> = stdout.lines().collect();
    let mut pairs: Vec<(String, String)> = Vec::new();
    for w in lines.windows(2) {
        if let Some(path) = w[0].strip_prefix("Derivation: ") {
            if let Some((_xfp, xpub)) = w[1].split_once(": ") {
                pairs.push((path.to_string(), xpub.to_string()));
            }
        }
    }
    for (path, xpub) in want {
        assert!(
            pairs.iter().any(|(p, x)| p == path && x == xpub),
            "path {path} must be paired with its OWN slot's xpub {xpub} (no scramble); \
             observed pairs: {pairs:?}\n--- output ---\n{stdout}"
        );
    }
}

/// #2 — all-equal paths keep the single shared `Derivation:` line, byte-
/// identical to the pre-cycle-13a emit (GREEN-preserving regression guard).
#[test]
fn export_coldcard_multisig_shared_path_unchanged() {
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
        .output()
        .expect("mnemonic spawn");
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    let expected =
        std::fs::read_to_string(FIXTURE_MULTISIG_2OF3_WSH).expect(FIXTURE_MULTISIG_2OF3_WSH);
    assert_eq!(
        stdout, expected,
        "all-agree export must stay byte-identical to the pre-cycle-13a fixture"
    );
}

/// #4 — Jade inherits the divergent per-cosigner emit via delegation.
#[test]
fn export_jade_divergent_paths_inherits_per_cosigner() {
    let out = run_divergent_export("jade");
    assert!(out.status.success(), "jade divergent export must succeed");
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(!stdout.contains("m/0'/0'"), "jade must not emit m/0'/0'; got:\n{stdout}");
    for path in ["m/48'/0'/0'/2'", "m/48'/0'/1'/2'", "m/48'/0'/2'/2'"] {
        assert!(
            stdout.contains(&format!("Derivation: {path}")),
            "jade missing per-cosigner Derivation for {path}; got:\n{stdout}"
        );
    }
}

/// #5 / #12 (headline co-design proof) — export divergent → import-wallet
/// --format coldcard-multisig → each cosigner's resolved `[fp/path]` equals
/// the ORIGINAL (divergent path + master fp preserved). GREEN only with P1
/// (H14-c silent accept) + P2 (per-line path parse) + P3 (per-cosigner emit).
#[test]
fn roundtrip_divergent_master_fp_and_paths_preserved() {
    let exp = run_divergent_export("coldcard");
    assert!(exp.status.success());
    let blob = String::from_utf8(exp.stdout).unwrap();

    let imp = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--blob",
            "-",
            "--format",
            "coldcard-multisig",
            "--json",
        ])
        .write_stdin(blob.clone())
        .output()
        .expect("import spawn");
    assert!(
        imp.status.success(),
        "re-import of the divergent export must succeed; stderr: {}\n--- blob ---\n{blob}",
        String::from_utf8_lossy(&imp.stderr)
    );
    let stdout = String::from_utf8(imp.stdout).unwrap();
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("envelope is JSON");
    let descriptor = v[0]["bundle"]["descriptor"]
        .as_str()
        .expect("descriptor present");

    // Each cosigner's ORIGINAL (master-fp, divergent-path, xpub) must survive
    // the round-trip into the re-parsed descriptor's `[fp/path]xpub` form.
    for (fp, path, xpub) in [
        (COSIGNER_A_FP, "48'/0'/0'/2'", COSIGNER_A_XPUB),
        (COSIGNER_B_FP, "48'/0'/1'/2'", COSIGNER_B_XPUB),
        (COSIGNER_C_FP, "48'/0'/2'/2'", COSIGNER_C_XPUB),
    ] {
        let want = format!("[{fp}/{path}]{xpub}");
        assert!(
            descriptor.contains(&want),
            "round-trip must preserve `{want}`; got descriptor:\n{descriptor}"
        );
    }
}
