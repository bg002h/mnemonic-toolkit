//! v0.6.2 §11.b — SLIP-0132 input-normalization stderr info-line in bundle mode.
//!
//! Pins stderr ordering: `info-line(s) → engraving card → secret-on-stdout
//! warning (conditional)` per SPEC §5.5.a. Watch-only paths suppress the
//! warning. RED on master pre-Phase 3 (info-line emission not yet wired in
//! `bundle.rs::resolve_slots` / `bundle.rs::bundle_run_unified_descriptor`).

use assert_cmd::Command;
use bip39::{Language, Mnemonic};
use bitcoin::base58;
use bitcoin::bip32::{DerivationPath, Xpriv, Xpub};
use bitcoin::secp256k1::Secp256k1;
use std::str::FromStr;

const TREZOR_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
const TREZOR_FP_HEX: &str = "5436d724";
const TREZOR_24_BIP84_MAINNET_XPUB: &str = "xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9";
const TREZOR_24_BIP84_MAINNET_ZPUB: &str = "zpub6qTBTNftBzVTjgVcSUw7vW5N1KQbV93Jnrw314RHGkCkSx4vk6nEWH1MJfReXi2WThvuDRiRpyT7cDoakEcZMQ1iZPgfJgQrcVMR4aJWh6S";

const INFO_LINE_ZPUB_MAINNET: &str =
    "info: normalized zpub input to neutral xpub (encoding-only; no key change). Re-emit with --xpub-prefix zpub if you need the SLIP-0132 form.\n";
const INFO_LINE_BIG_Y_MAINNET: &str =
    "info: normalized Ypub input to neutral xpub (encoding-only; no key change). Re-emit with --xpub-prefix Ypub if you need the SLIP-0132 form.\n";

const SECRET_WARNING: &str =
    "warning: secret material on stdout — consider redirecting (e.g., '> file.txt' or '| age -e ...')";

fn engraving_card_offset(stderr: &str) -> Option<usize> {
    stderr.find("# === Wallet bundle:")
}
fn secret_warning_offset(stderr: &str) -> Option<usize> {
    stderr.find(SECRET_WARNING)
}

/// Derive `(xpub, fingerprint, path)` for a phrase + derivation path on
/// mainnet. Pattern lifted from `cli_bundle_multisig.rs`.
fn derive_mainnet(phrase: &str, path_str: &str) -> (Xpub, String, String) {
    let secp = Secp256k1::new();
    let m = Mnemonic::parse_in(Language::English, phrase).unwrap();
    let seed = m.to_seed("");
    let master = Xpriv::new_master(bitcoin::NetworkKind::Main, &seed).unwrap();
    let fp = master.fingerprint(&secp);
    let path = DerivationPath::from_str(path_str).unwrap();
    let xpriv = master.derive_priv(&secp, &path).unwrap();
    let xpub = Xpub::from_priv(&secp, &xpriv);
    (xpub, fp.to_string().to_lowercase(), path_str.to_string())
}

/// Re-encode an xpub with the SLIP-0132 multisig-`Ypub` (mainnet) version
/// prefix `0x02 0x95 0xB4 0x3F`. Reaches into `bitcoin::base58` directly so
/// the test does not depend on the in-crate `slip0132::apply_xpub_prefix`
/// (which is `pub(crate)`).
fn to_big_y_mainnet(xpub: &Xpub) -> String {
    let mut raw = xpub.encode();
    raw[0..4].copy_from_slice(&[0x02, 0x95, 0xB4, 0x3F]);
    base58::encode_check(&raw)
}

// ============================================================================
// Matrix cell #5: descriptor-mode watch-only with zpub slot
//   → info-line → engraving card; warning suppressed.
// ============================================================================
#[test]
fn bundle_descriptor_watch_only_zpub_emits_info_line_then_card_no_warning() {
    let descriptor = "wpkh(@0/<0;1>/*)";
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            descriptor,
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.xpub={TREZOR_24_BIP84_MAINNET_ZPUB}"),
            "--slot",
            &format!("@0.fingerprint={TREZOR_FP_HEX}"),
        ])
        .assert()
        .success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    let info_idx = stderr
        .find(INFO_LINE_ZPUB_MAINNET)
        .unwrap_or_else(|| panic!("expected info-line in stderr; got: {stderr:?}"));
    let card_idx = engraving_card_offset(&stderr)
        .unwrap_or_else(|| panic!("expected engraving card in stderr; got: {stderr:?}"));
    assert!(
        info_idx < card_idx,
        "info-line must precede engraving card; info_idx={info_idx} card_idx={card_idx} stderr={stderr:?}"
    );
    assert!(
        secret_warning_offset(&stderr).is_none(),
        "watch-only must NOT emit secret warning; got: {stderr:?}"
    );
}

// ============================================================================
// Matrix cell #6: descriptor-mode watch-only with all-neutral xpubs
//   → engraving card only; no info-line, no warning.
// ============================================================================
#[test]
fn bundle_descriptor_watch_only_neutral_xpub_no_info_line() {
    let descriptor = "wpkh(@0/<0;1>/*)";
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            descriptor,
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.xpub={TREZOR_24_BIP84_MAINNET_XPUB}"),
            "--slot",
            &format!("@0.fingerprint={TREZOR_FP_HEX}"),
        ])
        .assert()
        .success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        !stderr.contains("info: normalized"),
        "neutral xpub input must not emit info-line; got: {stderr:?}"
    );
    assert!(
        engraving_card_offset(&stderr).is_some(),
        "engraving card must be present; got: {stderr:?}"
    );
    assert!(
        secret_warning_offset(&stderr).is_none(),
        "watch-only must not emit secret warning; got: {stderr:?}"
    );
}

// ============================================================================
// Matrix cell #7: full bundle (BIP-39 phrase + zpub cosigner) →
//   info-line → engraving card → secret warning.
// Two-cosigner sortedmulti where @0 is a phrase slot (secret-bearing) and
// @1 is a zpub slot (triggers normalizer + info-line).
// ============================================================================
#[test]
fn bundle_multisig_full_zpub_cosigner_emits_info_then_card_then_warning() {
    let path = "m/48'/0'/0'/2'";
    let (xpub_b, fp_b, _) = derive_mainnet(
        "legal winner thank year wave sausage worth useful legal winner thank yellow",
        path,
    );
    // Re-encode cosigner B as Ypub (multisig SLIP-0132). Used to ensure the
    // input differs from neutral xpub so the normalizer fires.
    let big_y_b = {
        let mut raw = xpub_b.encode();
        // Ypub mainnet multisig prefix.
        raw[0..4].copy_from_slice(&[0x02, 0x95, 0xB4, 0x3F]);
        base58::encode_check(&raw)
    };

    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--network",
            "mainnet",
            "--template",
            "wsh-sortedmulti",
            "--threshold",
            "2",
            "--slot",
            &format!("@0.phrase={TREZOR_24}"),
            "--slot",
            &format!("@1.xpub={big_y_b}"),
            "--slot",
            &format!("@1.fingerprint={fp_b}"),
            "--slot",
            &format!("@1.path={path}"),
        ])
        .assert()
        .success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    let info_idx = stderr
        .find(INFO_LINE_BIG_Y_MAINNET)
        .unwrap_or_else(|| panic!("expected Ypub info-line; stderr: {stderr:?}"));
    let card_idx = engraving_card_offset(&stderr)
        .unwrap_or_else(|| panic!("expected engraving card; stderr: {stderr:?}"));
    let warn_idx = secret_warning_offset(&stderr)
        .unwrap_or_else(|| panic!("full bundle must emit secret warning; stderr: {stderr:?}"));
    assert!(
        info_idx < card_idx && card_idx < warn_idx,
        "stderr ordering must be info → card → warning; got info_idx={info_idx} card_idx={card_idx} warn_idx={warn_idx} stderr={stderr:?}"
    );
}

// ============================================================================
// Matrix cell #8: full bundle (phrase only, no SLIP-0132 input) →
// engraving card → secret warning; no info-line.
// ============================================================================
#[test]
fn bundle_full_phrase_only_no_info_line_card_then_warning() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--slot",
            &format!("@0.phrase={TREZOR_24}"),
            "--network",
            "mainnet",
            "--template",
            "bip84",
        ])
        .assert()
        .success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        !stderr.contains("info: normalized"),
        "no SLIP-0132 input → no info-line; got: {stderr:?}"
    );
    let card_idx = engraving_card_offset(&stderr)
        .unwrap_or_else(|| panic!("expected engraving card; stderr: {stderr:?}"));
    let warn_idx = secret_warning_offset(&stderr)
        .unwrap_or_else(|| panic!("expected secret warning; stderr: {stderr:?}"));
    assert!(
        card_idx < warn_idx,
        "engraving card must precede secret warning; card_idx={card_idx} warn_idx={warn_idx}"
    );
}

// ============================================================================
// Matrix cell #9: multi-slot ordering — @0.xpub=zpub, @1.xpub=Ypub →
// stderr emits info-line for zpub then info-line for Ypub then card.
// ============================================================================
#[test]
fn bundle_multisig_two_normalized_slots_emit_info_lines_in_slot_order() {
    let path_a = "m/48'/0'/0'/2'";
    let path_b = "m/48'/0'/0'/2'";
    let (xpub_a, fp_a, _) = derive_mainnet(TREZOR_24, path_a);
    let (xpub_b, fp_b, _) = derive_mainnet(
        "legal winner thank year wave sausage worth useful legal winner thank yellow",
        path_b,
    );
    let zpub_a = {
        let mut raw = xpub_a.encode();
        // zpub mainnet single-sig prefix; encoding-only — content asserted
        // independently of BIP-84 path conventions.
        raw[0..4].copy_from_slice(&[0x04, 0xB2, 0x47, 0x46]);
        base58::encode_check(&raw)
    };
    let big_y_b = to_big_y_mainnet(&xpub_b);

    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--network",
            "mainnet",
            "--template",
            "wsh-sortedmulti",
            "--threshold",
            "2",
            "--slot",
            &format!("@0.xpub={zpub_a}"),
            "--slot",
            &format!("@0.fingerprint={fp_a}"),
            "--slot",
            &format!("@0.path={path_a}"),
            "--slot",
            &format!("@1.xpub={big_y_b}"),
            "--slot",
            &format!("@1.fingerprint={fp_b}"),
            "--slot",
            &format!("@1.path={path_b}"),
        ])
        .assert()
        .success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    let zpub_idx = stderr
        .find(INFO_LINE_ZPUB_MAINNET)
        .unwrap_or_else(|| panic!("expected zpub info-line; stderr: {stderr:?}"));
    let ypub_idx = stderr
        .find(INFO_LINE_BIG_Y_MAINNET)
        .unwrap_or_else(|| panic!("expected Ypub info-line; stderr: {stderr:?}"));
    let card_idx = engraving_card_offset(&stderr)
        .unwrap_or_else(|| panic!("expected engraving card; stderr: {stderr:?}"));
    assert!(
        zpub_idx < ypub_idx && ypub_idx < card_idx,
        "info-lines must appear in slot order then card; zpub_idx={zpub_idx} ypub_idx={ypub_idx} card_idx={card_idx}"
    );
    assert!(
        secret_warning_offset(&stderr).is_none(),
        "watch-only multisig must not emit secret warning; stderr: {stderr:?}"
    );
}

// ============================================================================
// Matrix cell #11: --json watch-only with zpub slot
//   → info-line on stderr; no warning (watch-only); JSON BundleJson on stdout.
// ============================================================================
#[test]
fn bundle_json_watch_only_zpub_emits_info_line_no_warning() {
    let descriptor = "wpkh(@0/<0;1>/*)";
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            descriptor,
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.xpub={TREZOR_24_BIP84_MAINNET_ZPUB}"),
            "--slot",
            &format!("@0.fingerprint={TREZOR_FP_HEX}"),
            "--json",
        ])
        .assert()
        .success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains(INFO_LINE_ZPUB_MAINNET),
        "expected info-line; stderr: {stderr:?}"
    );
    assert!(
        secret_warning_offset(&stderr).is_none(),
        "watch-only must not emit secret warning; stderr: {stderr:?}"
    );
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("valid BundleJson");
    assert_eq!(v["mode"], "watch-only");
}

// ============================================================================
// Matrix cell #12: --no-engraving-card with zpub slot
//   → info-line only on stderr; engraving card suppressed; no warning.
// ============================================================================
#[test]
fn bundle_no_engraving_card_zpub_emits_info_line_only() {
    let descriptor = "wpkh(@0/<0;1>/*)";
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            descriptor,
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.xpub={TREZOR_24_BIP84_MAINNET_ZPUB}"),
            "--slot",
            &format!("@0.fingerprint={TREZOR_FP_HEX}"),
            "--no-engraving-card",
        ])
        .assert()
        .success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains(INFO_LINE_ZPUB_MAINNET),
        "expected info-line; stderr: {stderr:?}"
    );
    assert!(
        engraving_card_offset(&stderr).is_none(),
        "--no-engraving-card must suppress card; stderr: {stderr:?}"
    );
    assert!(
        secret_warning_offset(&stderr).is_none(),
        "watch-only must not emit secret warning; stderr: {stderr:?}"
    );
}
