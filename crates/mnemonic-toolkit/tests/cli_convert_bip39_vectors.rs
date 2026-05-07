//! BIP-39 §"Test Vectors" — pinned reference corpus subset.
//!
//! Source: <https://raw.githubusercontent.com/trezor/python-mnemonic/master/vectors.json>
//! at SHA `b57a5ad77a981e743f4167ab2f7927a55c1e82a8` (retrieved 2026-05-07).
//! BIP-39 §Test Vectors delegates to this corpus. Each row is a 4-tuple
//! `[entropy_hex, mnemonic, seed_hex, xprv]`; passphrase is the literal
//! string `"TREZOR"` for every Trezor vector.
//!
//! Phase 1.B closes 6 of 24 english cells in the audit matrix
//! (`design/agent-reports/v0_7_1-bip-test-vector-audit-matrix.md` BIP-39
//! row). Selection: 2 × 12-word + 2 × 24-word + 2 × non-trivial entropy
//! edges. The remaining 18 cells stay MISSING for v0.8 carry.
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

const TREZOR_PASSPHRASE: &str = "TREZOR";

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
        .args(["convert", "--from", &format!("phrase={phrase}"), "--to", "entropy"])
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
        .args(["convert", "--from", &format!("entropy={entropy_hex}"), "--to", "phrase"])
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

// ---------------------------------------------------------------------------
// 12-word entries (zero-entropy + all-FF edges).
// ---------------------------------------------------------------------------

#[test]
fn bip39_trezor_v01_12word_zero_entropy() {
    assert_bip39_quad(
        "v01",
        "00000000000000000000000000000000",
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        "c55257c360c07c72029aebc1b53c05ed0362ada38ead3e3e9efa3708e53495531f09a6987599d18264c1e1c92f2cf141630c7a3c4ab7c81b2f001698e7463b04",
        "xprv9s21ZrQH143K3h3fDYiay8mocZ3afhfULfb5GX8kCBdno77K4HiA15Tg23wpbeF1pLfs1c5SPmYHrEpTuuRhxMwvKDwqdKiGJS9XFKzUsAF",
    );
}

#[test]
fn bip39_trezor_v04_12word_all_ff_entropy() {
    assert_bip39_quad(
        "v04",
        "ffffffffffffffffffffffffffffffff",
        "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong",
        "ac27495480225222079d7be181583751e86f571027b0497b5b5d11218e0a8a13332572917f0f8e5a589620c6f15b11c61dee327651a14c34e18231052e48c069",
        "xprv9s21ZrQH143K2V4oox4M8Zmhi2Fjx5XK4Lf7GKRvPSgydU3mjZuKGCTg7UPiBUD7ydVPvSLtg9hjp7MQTYsW67rZHAXeccqYqrsx8LcXnyd",
    );
}

// ---------------------------------------------------------------------------
// 24-word entries (zero-entropy + all-FF edges).
// ---------------------------------------------------------------------------

#[test]
fn bip39_trezor_v09_24word_zero_entropy() {
    assert_bip39_quad(
        "v09",
        "0000000000000000000000000000000000000000000000000000000000000000",
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art",
        "bda85446c68413707090a52022edd26a1c9462295029f2e60cd7c4f2bbd3097170af7a4d73245cafa9c3cca8d561a7c3de6f5d4a10be8ed2a5e608d68f92fcc8",
        "xprv9s21ZrQH143K32qBagUJAMU2LsHg3ka7jqMcV98Y7gVeVyNStwYS3U7yVVoDZ4btbRNf4h6ibWpY22iRmXq35qgLs79f312g2kj5539ebPM",
    );
}

#[test]
fn bip39_trezor_v12_24word_all_ff_entropy() {
    assert_bip39_quad(
        "v12",
        "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
        "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo vote",
        "dd48c104698c30cfe2b6142103248622fb7bb0ff692eebb00089b32d22484e1613912f0a5b694407be899ffd31ed3992c456cdf60f5d4564b8ba3f05a69890ad",
        "xprv9s21ZrQH143K2WFF16X85T2QCpndrGwx6GueB72Zf3AHwHJaknRXNF37ZmDrtHrrLSHvbuRejXcnYxoZKvRquTPyp2JiNG3XcjQyzSEgqCB",
    );
}

// ---------------------------------------------------------------------------
// Non-trivial entropy edges.
// ---------------------------------------------------------------------------

#[test]
fn bip39_trezor_v13_12word_nontrivial_entropy() {
    assert_bip39_quad(
        "v13",
        "9e885d952ad362caeb4efe34a8e91bd2",
        "ozone drill grab fiber curtain grace pudding thank cruise elder eight picnic",
        "274ddc525802f7c828d8ef7ddbcdc5304e87ac3535913611fbbfa986d0c9e5476c91689f9c8a54fd55bd38606aa6a8595ad213d4c9c9f9aca3fb217069a41028",
        "xprv9s21ZrQH143K2oZ9stBYpoaZ2ktHj7jLz7iMqpgg1En8kKFTXJHsjxry1JbKH19YrDTicVwKPehFKTbmaxgVEc5TpHdS1aYhB2s9aFJBeJH",
    );
}

#[test]
fn bip39_trezor_v15_24word_nontrivial_entropy() {
    assert_bip39_quad(
        "v15",
        "68a79eaca2324873eacc50cb9c6eca8cc68ea5d936f98787c60c7ebc74e6ce7c",
        "hamster diagram private dutch cause delay private meat slide toddler razor book happy fancy gospel tennis maple dilemma loan word shrug inflict delay length",
        "64c87cde7e12ecf6704ab95bb1408bef047c22db4cc7491c4271d170a1b213d20b385bc1588d9c7b38f1b39d415665b8a9030c9ec653d75e65f847d8fc1fc440",
        "xprv9s21ZrQH143K2XTAhys3pMNcGn261Fi5Ta2Pw8PwaVPhg3D8DWkzWQwjTJfskj8ofb81i9NP2cUNKxwjueJHHMQAnxtivTA75uUFqPFeWzk",
    );
}
