//! BIP-39 §"Test Vectors" — full pinned reference corpus (English).
//!
//! Source: <https://raw.githubusercontent.com/trezor/python-mnemonic/master/vectors.json>
//! at SHA `b57a5ad77a981e743f4167ab2f7927a55c1e82a8` (retrieved 2026-05-07).
//! BIP-39 §Test Vectors delegates to this corpus. Each row is a 4-tuple
//! `[entropy_hex, mnemonic, seed_hex, xprv]`; passphrase is the literal
//! string `"TREZOR"` for every Trezor vector.
//!
//! v0.7.1 Phase 1.B pinned 6 of 24 english cells via hand-rolled tests;
//! v0.8 Phase 8 lifts to a parametric loop covering all 24 cells via the
//! vendored `bip39_trezor_vectors.json` corpus (`include_str!`-loaded so
//! cargo rebuilds the test on corpus changes). Closes the v0.7.1
//! FOLLOWUP `18-remaining-bip39-trezor-corpus-vectors`.
//!
//! Each cell pins the full BIP-39 quad:
//!  1. CLI: `phrase → entropy` (decodes mnemonic + recomputes hex entropy).
//!  2. CLI: `entropy → phrase` (re-encodes with checksum).
//!  3. Lib: `Mnemonic::to_seed("TREZOR")` against the spec seed bytes —
//!     exercises the PBKDF2-HMAC-SHA512 surface with non-empty passphrase.
//!  4. Lib: `Xpriv::new_master(Main, &seed)` against the spec master xprv —
//!     pins the BIP-39 → BIP-32 hand-off.
//!
//! Note: the toolkit's `mnemonic convert --to xprv` returns the
//! template-derived *account* xpriv, not the master xpriv; the master-xprv
//! pin is therefore exercised at the library level (same crates the toolkit
//! uses internally — `bip39` + `bitcoin`).

use assert_cmd::Command;
use bip39::Mnemonic;
use bitcoin::bip32::Xpriv;
use bitcoin::NetworkKind;
use serde_json::Value;

const TREZOR_PASSPHRASE: &str = "TREZOR";

const CORPUS_JSON: &str = include_str!("bip39_trezor_vectors.json");

/// Pin the full BIP-39 quad for one Trezor vector.
fn assert_bip39_quad(
    label: &str,
    entropy_hex: &str,
    phrase: &str,
    seed_hex: &str,
    expected_master_xprv: &str,
) {
    // (1) CLI phrase → entropy.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("phrase={phrase}"),
            "--to",
            "entropy",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(
        stdout,
        format!("entropy: {entropy_hex}\n"),
        "phrase→entropy mismatch ({label})"
    );

    // (2) CLI entropy → phrase.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("entropy={entropy_hex}"),
            "--to",
            "phrase",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(
        stdout,
        format!("phrase: {phrase}\n"),
        "entropy→phrase mismatch ({label})"
    );

    // (3) Lib seed pin under TREZOR passphrase (PBKDF2-HMAC-SHA512 path).
    let mnemonic = Mnemonic::parse_in(bip39::Language::English, phrase).expect("phrase parses");
    let seed = mnemonic.to_seed(TREZOR_PASSPHRASE);
    assert_eq!(hex::encode(seed), seed_hex, "seed mismatch ({label})");

    // (4) Lib master xprv pin (BIP-39 → BIP-32 hand-off).
    let master = Xpriv::new_master(NetworkKind::Main, &seed).expect("Xpriv::new_master");
    assert_eq!(
        master.to_string(),
        expected_master_xprv,
        "master xprv mismatch ({label})"
    );
}

/// SPEC v0.8 Phase 8 — exhaustive BIP-39 English Trezor corpus pin (all 24
/// vectors as a single parametric test). v0.7.1 carried 6 hand-pinned cells;
/// this loop covers the full corpus, closing the `18-remaining-bip39-trezor-
/// corpus-vectors` FOLLOWUP.
#[test]
fn bip39_trezor_english_corpus_full() {
    let parsed: Value = serde_json::from_str(CORPUS_JSON).expect("corpus JSON parses");
    let rows = parsed["english"].as_array().expect("english array present");
    assert_eq!(rows.len(), 24, "expected 24 english Trezor vectors");
    for (i, row) in rows.iter().enumerate() {
        let arr = row.as_array().expect("row is a 4-tuple");
        assert_eq!(arr.len(), 4, "row {i} has unexpected shape");
        let entropy_hex = arr[0].as_str().unwrap();
        let phrase = arr[1].as_str().unwrap();
        let seed_hex = arr[2].as_str().unwrap();
        let expected_master_xprv = arr[3].as_str().unwrap();
        let label = format!("english[{i}]");
        assert_bip39_quad(&label, entropy_hex, phrase, seed_hex, expected_master_xprv);
    }
}
