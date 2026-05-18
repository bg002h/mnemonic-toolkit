//! Phase 5 — `mnemonic import-wallet` seed overlay (SPEC §8.3).
//!
//! Per `design/IMPLEMENTATION_PLAN_wallet_import_v0_26_0.md` §5.7-§5.11. The
//! pipeline-under-test:
//!   --ms1 <ms-encoded entropy> | --slot @N.phrase=<BIP-39 phrase>
//!     ↓ ms_codec::decode | bip39::Mnemonic::parse
//!     ↓ entropy → mnemonic → seed → master xpriv → derive_priv(path) → Xpub
//!     ↓ compare against blob.cosigners[i].xpub
//!     ↓ match → attach entropy; mismatch → exit 4 ImportWalletSeedMismatch
//!
//! ## Ground-truth seed
//!
//! The Trezor canonical 24-word vector "abandon × 23 art" yields 32-zero-
//! bytes entropy. Empty-passphrase master derivation gives master
//! fingerprint `5436d724`. Known xpubs (verified live):
//!
//!   - `m/48'/0'/0'/2'` (BIP-48 multisig segwit, account 0):
//!     `xpub6E79FaRWLSJCAgA2jDHRvyrWKwT6aSmR685zptzyYPvmUd44omcxZ1NAzDtbdFBvEADjcVbV4NzTDwQeU6oiSV9KGiMSWhjANZjbfUHkm3Y`
//!
//! The ms1-encoded form of the 32-zero entropy:
//!   `ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqcwugpdxtfme2w`

use assert_cmd::Command;
use miniscript::descriptor::checksum::Engine as ChecksumEngine;
use std::path::PathBuf;

const TREZOR_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
const TREZOR_24_MS1: &str =
    "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqcwugpdxtfme2w";
const TREZOR_24_FP: &str = "5436d724";
/// TREZOR_24 → `m/48'/0'/0'/2'` (BIP-48 multisig segwit, account 0, mainnet).
/// Verified live via a standalone derivation harness (bitcoin = 0.32,
/// bip39 = 2; `Xpriv::new_master(NetworkKind::Main, seed) → derive_priv`).
const TREZOR_24_XPUB_BIP48: &str = "xpub6E79FaRWLSJCAgA2jDHRvyrWKwT6aSmR685zptzyYPvmUd44omcxZ1NAzDtbdFBvEADjcVbV4NzTDwQeU6oiSV9KGiMSWhjANZjbfUHkm3Y";

/// Build a BSMS 2-line blob from a descriptor body. Computes a fresh
/// BIP-380 checksum so the blob is fully self-consistent.
fn bsms_2line(body: &str) -> String {
    let mut e = ChecksumEngine::new();
    e.input(body).expect("checksum input must be ASCII");
    let csum = e.checksum();
    format!("BSMS 1.0\n{body}#{csum}\n")
}

/// Build a 1-of-1 watch-only wsh(sortedmulti) BSMS blob using the TREZOR_24
/// seed's xpub at the BIP-48 m/48'/0'/0'/2' path. The descriptor uses
/// `sortedmulti(1, ...)` so it has a `threshold` and a single cosigner —
/// the simplest shape that exercises the multi-cosigner seed-overlay code
/// path with a known seed.
fn flagship_1of1_blob() -> String {
    let body =
        format!("wsh(sortedmulti(1,[{TREZOR_24_FP}/48'/0'/0'/2']{TREZOR_24_XPUB_BIP48}/<0;1>/*))");
    bsms_2line(&body)
}

// ============================================================================
// Cell 1 — happy path: --ms1 derives correctly + entropy attached.
// ============================================================================

#[test]
fn seed_overlay_ms1_match_success() {
    let blob = flagship_1of1_blob();
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--blob",
            "-",
            "--format",
            "bsms",
            "--ms1",
            TREZOR_24_MS1,
            "--json",
        ])
        .write_stdin(blob)
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let val: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let env = &val.as_array().unwrap()[0];
    let cosigner0 = &env["bundle"]["cosigners"].as_array().unwrap()[0];
    assert_eq!(
        cosigner0["has_entropy"].as_bool(),
        Some(true),
        "cosigner[0].has_entropy must be true after successful overlay; env: {env}"
    );
    assert_eq!(
        cosigner0["fingerprint"].as_str(),
        Some(TREZOR_24_FP),
        "cosigner fingerprint must be the TREZOR_24 fp"
    );
}

// ============================================================================
// Cell 2 — mismatch: wrong --ms1 → exit 4 + stderr template.
// ============================================================================

#[test]
fn seed_overlay_ms1_mismatch_exit_4() {
    let blob = flagship_1of1_blob();
    // Use a different 32-byte-entropy ms1 (all-ones isn't representable
    // directly, but the 16-byte vector from the repair tests is).
    // Per project convention, decoding `ms10entr…[16-byte]` produces a
    // different entropy → different master fingerprint → different xpub
    // at m/48'/0'/0'/2' → mismatch.
    const WRONG_MS1_16BYTE_ZERO: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
    let assertion = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--blob",
            "-",
            "--format",
            "bsms",
            "--ms1",
            WRONG_MS1_16BYTE_ZERO,
        ])
        .write_stdin(blob)
        .assert()
        .failure()
        .code(4);
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("cosigner 0:")
            && stderr.contains("supplied seed produces xpub")
            && stderr.contains("blob declares"),
        "expected ImportWalletSeedMismatch stderr template, got: {stderr}"
    );
}

// ============================================================================
// Cell 3 — partial watch-only: empty-string sentinel preserves watch-only.
// ============================================================================

#[test]
fn seed_overlay_empty_string_sentinel_preserves_watch_only() {
    // Single-cosigner blob; pass `--ms1 ""` (empty-string sentinel per
    // v0.25.1) → cosigner stays watch-only + stderr NOTICE.
    let blob = flagship_1of1_blob();
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--blob",
            "-",
            "--format",
            "bsms",
            "--ms1",
            "",
            "--json",
        ])
        .write_stdin(blob)
        .assert()
        .success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("cosigner 0 ms1 supplied as empty-string sentinel")
            || stderr.contains("treated as watch-only"),
        "expected v0.25.1 empty-sentinel NOTICE in stderr, got: {stderr}"
    );
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let val: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let cosigner0 = &val.as_array().unwrap()[0]["bundle"]["cosigners"][0];
    assert_eq!(cosigner0["has_entropy"].as_bool(), Some(false));
}

// ============================================================================
// Cell 4 — --slot @N.phrase= equivalent path.
// ============================================================================

#[test]
fn seed_overlay_via_slot_subkey_phrase() {
    let blob = flagship_1of1_blob();
    let slot_arg = format!("@0.phrase={TREZOR_24}");
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--blob",
            "-",
            "--format",
            "bsms",
            "--slot",
            &slot_arg,
            "--json",
        ])
        .write_stdin(blob)
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let val: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let cosigner0 = &val.as_array().unwrap()[0]["bundle"]["cosigners"][0];
    assert_eq!(
        cosigner0["has_entropy"].as_bool(),
        Some(true),
        "cosigner[0].has_entropy must be true after --slot @0.phrase= overlay"
    );
}

// ============================================================================
// Cell 5 — @env:VAR sentinel path.
// ============================================================================

#[test]
fn seed_overlay_env_var_sentinel() {
    let blob = flagship_1of1_blob();
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .env("MNEMONIC_TEST_IMPORT_WALLET_MS1_0", TREZOR_24_MS1)
        .args([
            "import-wallet",
            "--blob",
            "-",
            "--format",
            "bsms",
            "--ms1",
            "@env:MNEMONIC_TEST_IMPORT_WALLET_MS1_0",
            "--json",
        ])
        .write_stdin(blob)
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let val: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let cosigner0 = &val.as_array().unwrap()[0]["bundle"]["cosigners"][0];
    assert_eq!(
        cosigner0["has_entropy"].as_bool(),
        Some(true),
        "env-var sentinel resolution must drive a successful overlay"
    );
}

// ============================================================================
// Cell 6 — @env:VAR unset → exit 1 EnvVarMissing
// ============================================================================

#[test]
fn seed_overlay_env_var_unset_exit_1() {
    let blob = flagship_1of1_blob();
    let assertion = Command::cargo_bin("mnemonic")
        .unwrap()
        .env_remove("MNEMONIC_TEST_IMPORT_WALLET_MS1_NEVER_SET")
        .args([
            "import-wallet",
            "--blob",
            "-",
            "--format",
            "bsms",
            "--ms1",
            "@env:MNEMONIC_TEST_IMPORT_WALLET_MS1_NEVER_SET",
        ])
        .write_stdin(blob)
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("--ms1: env-var") && stderr.contains("is not set"),
        "expected EnvVarMissing stderr template, got: {stderr}"
    );
}

// ============================================================================
// Cell 7 — --slot @N.entropy= (non-phrase subkey) → exit 1 BadInput.
// ============================================================================

#[test]
fn seed_overlay_slot_non_phrase_subkey_rejected() {
    let blob = flagship_1of1_blob();
    // import-wallet only accepts the `phrase` subkey on `--slot`. Other
    // subkeys (entropy / wif / xprv) must be rejected at clap-parse-time
    // or in `run()`.
    let assertion = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--blob",
            "-",
            "--format",
            "bsms",
            "--slot",
            "@0.entropy=00000000000000000000000000000000",
        ])
        .write_stdin(blob)
        .assert()
        .failure();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("only the `phrase` subkey is supported"),
        "expected import-wallet subkey gate, got: {stderr}"
    );
}

// ============================================================================
// Cell 8 — multi-cosigner skip-middle: --ms1 a --ms1 "" --ms1 c on 2-of-3 blob.
// Plan §5.10: 3-cosigner blob; supply --ms1 for cosigner 0 + 2 only; assert
// cosigner 1 stays watch-only. Exercises the multi-cosigner-with-middle-skip
// case that the empty-string-sentinel-on-1-of-1 cell above does not cover.
//
// Provenance of the three BIP-39 seeds:
//   - "abandon × 11 about" → fp `73c5da0a` → ms10entrsqqqqq...cj9sxraq34v7f
//     (BIP-39 12-word test vector; appears in tests/cli_verify_bundle_multi_cosigner_mk1.rs:21)
//   - "legal winner × ... thank yellow" → fp `b8688df1` → ms10entrsqplh7lml...
//     (BIP-39 12-word test vector; same file:24)
//   - "letter advice × ... cage above" → fp `28645006` → ms10entrsqzqgpq...
//     (BIP-39 12-word test vector; same file:26)
//
// Xpubs at BIP-87 path `m/87'/0'/0'` derived live via
//   `mnemonic bundle --template wsh-sortedmulti --multisig-path-family bip87
//    --slot @N.phrase=... --json` (see `cosigners[]` in the output). BSMS
// blob below is hand-rolled at the same path family; BIP-380 checksum
// `4wup4at0` discovered via the toolkit's "expected <csum>" stderr template.
// ============================================================================

const SKIP_MIDDLE_MS1_0: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
const SKIP_MIDDLE_MS1_2: &str = "ms10entrsqzqgpqyqszqgpqyqszqgpqyqszqqlfm7mep84hunu";

fn skip_middle_3of3_blob() -> String {
    let body = "wsh(sortedmulti(2,\
[73c5da0a/87'/0'/0']xpub6DBjiYnc4ewKti13Q1L35bqdodw5z3VGJnf516B3icHrEGEUcCuCG5GVQDZtH8Xmsyt3Fs9YDNwLaqjUbbRidwXZ6sxufZcr4VqqzrXvicM/<0;1>/*,\
[b8688df1/87'/0'/0']xpub6CbhrPzY2z7NcCGCGjLAJLq8iRyjUfwmdXQs66MxTVUReKqb9DpLnVJ5D1qpatZjUuPGTyxf5TYU1vA34YFE9FHB4TvfYmokYLVsyEFZFt9/<0;1>/*,\
[28645006/87'/0'/0']xpub6DB7HNqw6CZojxN85NuFTPWZhi2FagSnexPS1rv3nYQhngkmdHgb7iebYvTFmFKKDA3ozf5yezDsCH6cXAw3WZijviSZtZC2hjHn2uazz4z/<0;1>/*))";
    bsms_2line(body)
}

#[test]
fn seed_overlay_multi_cosigner_skip_middle() {
    let blob = skip_middle_3of3_blob();
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--blob",
            "-",
            "--format",
            "bsms",
            "--ms1",
            SKIP_MIDDLE_MS1_0,
            "--ms1",
            "",
            "--ms1",
            SKIP_MIDDLE_MS1_2,
            "--json",
        ])
        .write_stdin(blob)
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let val: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let cosigners = val.as_array().unwrap()[0]["bundle"]["cosigners"]
        .as_array()
        .expect("bundle.cosigners array")
        .clone();
    assert_eq!(cosigners.len(), 3, "expected 3 cosigners");
    let has_entropy: Vec<bool> = cosigners
        .iter()
        .map(|c| c["has_entropy"].as_bool().unwrap())
        .collect();
    assert_eq!(
        has_entropy,
        vec![true, false, true],
        "expected [true, false, true]; got {has_entropy:?}; cosigners={cosigners:?}"
    );
    // Sanity: middle cosigner stays watch-only via the empty-string sentinel.
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("cosigner 1 ms1 supplied as empty-string sentinel")
            || stderr.contains("treated as watch-only"),
        "expected v0.25.1 empty-sentinel NOTICE for cosigner 1, got: {stderr}"
    );
}

/// Path for any vendored fixture (currently unused by the cells above; the
/// blob fixtures are constructed in-test so a path-getter is kept for the
/// future cells that switch to vendored fixtures with named seeds).
#[allow(dead_code)]
fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from("tests/fixtures/wallet_import").join(name)
}
