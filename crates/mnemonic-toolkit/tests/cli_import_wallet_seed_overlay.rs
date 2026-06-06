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
    // v0.27.0 envelope: `bundle` is full BundleJson. For 1-of-1 the
    // master_fingerprint surfaces at bundle.master_fingerprint; per-slot
    // entropy presence is encoded by `bundle.ms1[i]` being non-empty per
    // SPEC §5.8.
    let bundle = &env["bundle"];
    assert_eq!(
        bundle["master_fingerprint"].as_str(),
        Some(TREZOR_24_FP),
        "bundle.master_fingerprint must be the TREZOR_24 fp; env: {env}"
    );
    let ms1_0 = bundle["ms1"].as_array().expect("bundle.ms1 must be array")[0]
        .as_str()
        .expect("ms1[0] must be string");
    assert!(
        !ms1_0.is_empty(),
        "bundle.ms1[0] must be non-empty (entropy attached) after successful overlay; got {ms1_0:?}"
    );
    assert_eq!(
        bundle["mode"].as_str(),
        Some("full"),
        "bundle.mode must be \"full\" when any cosigner has entropy"
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
    let bundle = &val.as_array().unwrap()[0]["bundle"];
    let ms1_0 = bundle["ms1"].as_array().unwrap()[0].as_str().unwrap();
    assert!(
        ms1_0.is_empty(),
        "bundle.ms1[0] must be empty (watch-only sentinel) under --ms1 \"\"; got {ms1_0:?}"
    );
    assert_eq!(bundle["mode"].as_str(), Some("watch-only"));
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
    let bundle = &val.as_array().unwrap()[0]["bundle"];
    let ms1_0 = bundle["ms1"].as_array().unwrap()[0].as_str().unwrap();
    assert!(
        !ms1_0.is_empty(),
        "bundle.ms1[0] must be non-empty after --slot @0.phrase= overlay; got {ms1_0:?}"
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
    let bundle = &val.as_array().unwrap()[0]["bundle"];
    let ms1_0 = bundle["ms1"].as_array().unwrap()[0].as_str().unwrap();
    assert!(
        !ms1_0.is_empty(),
        "env-var sentinel resolution must drive a successful overlay; ms1[0] should carry entropy"
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
    // v0.27.0 envelope: `bundle.ms1` is the length-N SPEC §5.8 array;
    // entry [i] non-empty ↔ cosigner i has entropy. `bundle.multisig.cosigners`
    // carries the per-cosigner CosignerEntry vec for N>1.
    let bundle = &val.as_array().unwrap()[0]["bundle"];
    let cosigners = bundle["multisig"]["cosigners"]
        .as_array()
        .expect("bundle.multisig.cosigners array")
        .clone();
    assert_eq!(cosigners.len(), 3, "expected 3 cosigners");
    let ms1 = bundle["ms1"].as_array().expect("bundle.ms1 array");
    assert_eq!(ms1.len(), 3, "expected bundle.ms1 length 3");
    let has_entropy: Vec<bool> = ms1
        .iter()
        .map(|s| !s.as_str().unwrap().is_empty())
        .collect();
    assert_eq!(
        has_entropy,
        vec![true, false, true],
        "expected [true, false, true]; got {has_entropy:?}; ms1={ms1:?}"
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

// ============================================================================
// v0.27.1 Phase 4 PR-#26 coverage gap fold (I12 + I13 + I14)
// ============================================================================

/// I12 — `--ms1` AND `--slot @i.phrase=` for the SAME cosigner index conflict
/// per `wallet_import/overlay.rs::apply_seed_overlay` BadInput template
/// ("cosigner {i} has both --ms1 and --slot @{i}.phrase= supplied"). Regression
/// guard against silent precedence-change.
#[test]
fn seed_overlay_ms1_and_slot_phrase_for_same_cosigner_conflict() {
    let blob = flagship_1of1_blob();
    let slot_arg = format!("@0.phrase={TREZOR_24}");
    let assertion = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--blob",
            "-",
            "--format",
            "bsms",
            "--ms1",
            TREZOR_24_MS1,
            "--slot",
            &slot_arg,
        ])
        .write_stdin(blob)
        .assert()
        .failure();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("cosigner 0") && (stderr.contains("--ms1") || stderr.contains("--slot")),
        "expected per-cosigner conflict diagnostic naming cosigner 0; got: {stderr}"
    );
}

/// I13 — phrase-overlay mismatch via `--slot @0.phrase=<wrong-phrase>` (not
/// `--ms1`). The symmetric Source::Phrase code path through
/// `apply_seed_overlay` mismatch arm; the existing
/// `seed_overlay_ms1_mismatch_exit_4` only exercises the Source::Ms1 path.
#[test]
fn seed_overlay_slot_phrase_mismatch_exit_4() {
    let blob = flagship_1of1_blob();
    // BIP-39 valid 24-word phrase whose derived xpub at m/48'/0'/0'/2'
    // differs from TREZOR_24's. (Standard BIP-39 alternative.)
    const WRONG_24: &str = "legal winner thank year wave sausage worth useful legal winner thank year wave sausage worth useful legal winner thank year wave sausage worth title";
    let slot_arg = format!("@0.phrase={WRONG_24}");
    let assertion = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--blob",
            "-",
            "--format",
            "bsms",
            "--slot",
            &slot_arg,
        ])
        .write_stdin(blob)
        .assert()
        .failure();
    let code = assertion.get_output().status.code().unwrap_or(-1);
    assert_eq!(code, 4, "phrase mismatch must exit 4 (sibling to BundleMismatch)");
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.to_lowercase().contains("mismatch") || stderr.to_lowercase().contains("xpub"),
        "expected mismatch diagnostic; got: {stderr}"
    );
}

/// I14 — `--ms1` with malformed bech32 hits the `Err(_)` arm at
/// `overlay.rs:133-137` ("ms_codec decode failed"). The strictly-non-entropy
/// branch at overlay.rs:128-132 ("decoded payload is not entropy") is
/// structurally unreachable from user input because `ms_codec::Payload` v0.2.0
/// has only the `Entr` variant (the enum is `#[non_exhaustive]` for future
/// expansion); this cell pins the adjacent decode-Err coverage gap that
/// `apply_seed_overlay` shares with the non-entropy arm. If a future ms-codec
/// version adds Payload variants, a Phase-N cycle can extend this cell to
/// also feed a non-Entr payload card.
#[test]
fn seed_overlay_ms1_decode_error_rejected_with_pointer_text() {
    let blob = flagship_1of1_blob();
    // Valid ms1 HRP prefix to pass upstream `validate_flag_hrp("--ms1", "ms", ...)`,
    // but garbage bech32 body so `ms_codec::decode` rejects.
    const MALFORMED_MS1: &str = "ms1qpzqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqy";
    let assertion = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--blob",
            "-",
            "--format",
            "bsms",
            "--ms1",
            MALFORMED_MS1,
        ])
        .write_stdin(blob)
        .assert()
        .failure();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.to_lowercase().contains("decode")
            || stderr.to_lowercase().contains("ms_codec")
            || stderr.to_lowercase().contains("checksum")
            || stderr.to_lowercase().contains("invalid"),
        "expected ms_codec decode-failure diagnostic; got: {stderr}"
    );
}

// ============================================================================
// FOLLOWUP `import-wallet-ms1-argv-advisory-gap` — secret-in-argv advisory for
// inline `--ms1` AND its twin `--slot @N.phrase` (per-leak-site, actual index).
// ============================================================================

#[test]
fn ms1_inline_value_fires_argv_advisory() {
    let blob = flagship_1of1_blob();
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet", "--blob", "-", "--format", "bsms", "--ms1", TREZOR_24_MS1, "--json",
        ])
        .write_stdin(blob)
        .assert()
        .success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("secret material on argv (--ms1)"),
        "inline --ms1 must fire the argv-leak advisory; got: {stderr:?}"
    );
}

#[test]
fn ms1_env_sentinel_no_argv_advisory() {
    // `@env:VAR` is NOT an argv leak — the advisory must NOT fire (and the
    // @env:-skip must read the RAW pre-rebind value, not the resolved secret).
    let blob = flagship_1of1_blob();
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet", "--blob", "-", "--format", "bsms", "--ms1",
            "@env:MNEMONIC_TEST_MS1_ENV", "--json",
        ])
        .env("MNEMONIC_TEST_MS1_ENV", TREZOR_24_MS1)
        .write_stdin(blob)
        .assert()
        .success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        !stderr.contains("secret material on argv (--ms1)"),
        "an @env: --ms1 must NOT fire the argv advisory; got: {stderr:?}"
    );
}

#[test]
fn slot_phrase_inline_fires_argv_advisory_with_actual_index() {
    // `--slot @0.phrase=<inline>` is the same @env:-only argv-secret surface;
    // advisory fires per-leak-site with the ACTUAL index. It fires at top-of-run
    // BEFORE validation, so this asserts only the advisory (exit not asserted).
    let blob = flagship_1of1_blob();
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet", "--blob", "-", "--format", "bsms",
            "--slot",
            "@0.phrase=abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
            "--json",
        ])
        .write_stdin(blob)
        .assert();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("secret material on argv (--slot @0.phrase=)"),
        "inline --slot @0.phrase must fire the argv advisory with the actual index; got: {stderr:?}"
    );
}
