//! `mnemonic xpub-search path-of-xpub` integration tests (v0.26.0 P1).
//!
//! Per `design/PLAN_v0_26_0_xpub_search.md` §10 C1 (commit-numbered phase P1).
//! Realizes plan §3 (P1 SPEC) — seed-intake polymorphism (`--phrase` /
//! `--phrase-stdin` / `--ms1` / `--ms1-stdin` / positional ms1) + target
//! intake polymorphism (xpub | mk1) + candidate-iteration over BIP-44/49/
//! 84/86 single-sig and BIP-48 multisig templates at script_type 1'/2'/3'
//! × account-range + add-path extensions.
//!
//! Test design (TDD):
//! - 19 integration cells via `assert_cmd::Command`.
//! - 1 unit cell for the `XpubSearchEnvelope` serde round-trip.
//!
//! Fixtures: BIP-39 test vector `abandon × 11 about` is the universal master.
//! All expected xpubs are derived at runtime via `bitcoin::bip32` primitives —
//! no hardcoded ground-truth constants we couldn't reproduce.

use assert_cmd::Command;
use bip39::Mnemonic;
use bitcoin::bip32::{DerivationPath, Xpriv, Xpub};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::{base58, NetworkKind};
use predicates::prelude::*;
use std::str::FromStr;

/// The canonical BIP-39 12-word test vector. Entropy is 16 zero bytes.
const PHRASE: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

/// Compute the xpub at `path` for the test phrase + optional passphrase.
fn xpub_at(path: &str, passphrase: &str) -> Xpub {
    let mnemonic = Mnemonic::parse_in(bip39::Language::English, PHRASE).unwrap();
    let seed = mnemonic.to_seed(passphrase);
    let secp = Secp256k1::new();
    let master = Xpriv::new_master(NetworkKind::Main, &seed).unwrap();
    let dp = DerivationPath::from_str(path).unwrap();
    let xpriv = master.derive_priv(&secp, &dp).unwrap();
    Xpub::from_priv(&secp, &xpriv)
}

/// Re-encode an xpub with a SLIP-0132 mainnet `zpub` version prefix
/// (`0x04B24746`).
fn xpub_as_zpub(xp: &Xpub) -> String {
    let raw = xp.encode();
    let mut swapped = raw.to_vec();
    swapped[0..4].copy_from_slice(&[0x04, 0xB2, 0x47, 0x46]);
    base58::encode_check(&swapped)
}

/// Re-encode an xpub as the multisig `Zpub` (BIP-48 wsh) mainnet variant
/// (`0x02AA7ED3`).
fn xpub_as_multisig_zpub(xp: &Xpub) -> String {
    let raw = xp.encode();
    let mut swapped = raw.to_vec();
    swapped[0..4].copy_from_slice(&[0x02, 0xAA, 0x7E, 0xD3]);
    base58::encode_check(&swapped)
}

/// A second BIP-39 vector for "different seed → no match" cells.
const OTHER_PHRASE: &str =
    "legal winner thank year wave sausage worth useful legal winner thank yellow";

fn other_xpub_at(path: &str) -> Xpub {
    let mnemonic = Mnemonic::parse_in(bip39::Language::English, OTHER_PHRASE).unwrap();
    let seed = mnemonic.to_seed("");
    let secp = Secp256k1::new();
    let master = Xpriv::new_master(NetworkKind::Main, &seed).unwrap();
    let dp = DerivationPath::from_str(path).unwrap();
    let xpriv = master.derive_priv(&secp, &dp).unwrap();
    Xpub::from_priv(&secp, &xpriv)
}

// ---------------------------------------------------------------------------
// Cell 1 — phrase + zpub target at m/84'/0'/0' → match
// ---------------------------------------------------------------------------
#[test]
fn path_of_xpub_phrase_zpub_match_bip84() {
    let target = xpub_as_zpub(&xpub_at("m/84'/0'/0'", ""));
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "path-of-xpub",
            "--phrase-stdin",
            "--target-xpub",
            &target,
            "--json",
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();
    let body = String::from_utf8(out).unwrap();
    let v: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(v["mode"], "path-of-xpub");
    assert_eq!(v["result"], "match");
    assert_eq!(v["path"], "m/84'/0'/0'");
    assert_eq!(v["template"], "bip84");
    assert_eq!(v["account"], 0);
    assert_eq!(v["schema_version"], "1");
}

// ---------------------------------------------------------------------------
// Cell 2 — phrase + xpub target at m/86'/0'/3' → match account=3
// ---------------------------------------------------------------------------
#[test]
fn path_of_xpub_phrase_xpub_match_bip86_account3() {
    let target = xpub_at("m/86'/0'/3'", "").to_string();
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "path-of-xpub",
            "--phrase-stdin",
            "--target-xpub",
            &target,
            "--json",
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_str(&String::from_utf8(out).unwrap()).unwrap();
    assert_eq!(v["path"], "m/86'/0'/3'");
    assert_eq!(v["template"], "bip86");
    assert_eq!(v["account"], 3);
}

// ---------------------------------------------------------------------------
// Cell 3 — SLIP-0132 normalize: zpub variant preserved in JSON
// ---------------------------------------------------------------------------
#[test]
fn path_of_xpub_slip0132_normalize_zpub_variant_preserved() {
    let target = xpub_as_zpub(&xpub_at("m/84'/0'/0'", ""));
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "path-of-xpub",
            "--phrase-stdin",
            "--target-xpub",
            &target,
            "--json",
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_str(&String::from_utf8(out).unwrap()).unwrap();
    assert_eq!(v["target_xpub_variant"], "zpub");
    let canonical = v["target_xpub_canonical"].as_str().unwrap();
    assert!(
        canonical.starts_with("xpub"),
        "canonical must start with 'xpub' after normalization; got {canonical}"
    );
}

// ---------------------------------------------------------------------------
// Cell 4 — different seed → no match → exit 4
// ---------------------------------------------------------------------------
#[test]
fn path_of_xpub_no_match_returns_exit_4() {
    let target = other_xpub_at("m/84'/0'/0'").to_string();
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "path-of-xpub",
            "--phrase-stdin",
            "--target-xpub",
            &target,
            "--json",
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(4)
        .get_output()
        .stdout
        .clone();
    // JSON envelope on no-match still emitted.
    let v: serde_json::Value = serde_json::from_str(&String::from_utf8(out).unwrap()).unwrap();
    assert_eq!(v["mode"], "path-of-xpub");
    assert_eq!(v["result"], "no_match");
}

// ---------------------------------------------------------------------------
// Cell 5 — --min-account 5 --number-of-accounts 3 searches [5, 8)
// ---------------------------------------------------------------------------
#[test]
fn path_of_xpub_min_account_5_number_of_accounts_3_searches_5_to_8() {
    // Target at account=2 should NOT match (out of [5, 8)).
    let target_a2 = xpub_at("m/84'/0'/2'", "").to_string();
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "path-of-xpub",
            "--phrase-stdin",
            "--target-xpub",
            &target_a2,
            "--min-account",
            "5",
            "--number-of-accounts",
            "3",
            "--json",
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(4);

    // Target at account=6 SHOULD match.
    let target_a6 = xpub_at("m/84'/0'/6'", "").to_string();
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "path-of-xpub",
            "--phrase-stdin",
            "--target-xpub",
            &target_a6,
            "--min-account",
            "5",
            "--number-of-accounts",
            "3",
            "--json",
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_str(&String::from_utf8(out).unwrap()).unwrap();
    assert_eq!(v["account"], 6);
}

// ---------------------------------------------------------------------------
// Cell 6 — --max-account 50 widens the range; target at 30 matches.
// ---------------------------------------------------------------------------
#[test]
fn path_of_xpub_max_account_overrides_number_of_accounts() {
    let target = xpub_at("m/84'/0'/30'", "").to_string();
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "path-of-xpub",
            "--phrase-stdin",
            "--target-xpub",
            &target,
            "--min-account",
            "0",
            "--number-of-accounts",
            "5",
            "--max-account",
            "50",
            "--json",
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_str(&String::from_utf8(out).unwrap()).unwrap();
    assert_eq!(v["account"], 30);
}

// ---------------------------------------------------------------------------
// Cell 7 — --add-path m/87'/0'/account' substitution → BIP-87 path matched.
// ---------------------------------------------------------------------------
#[test]
fn path_of_xpub_add_path_bip87() {
    let target = xpub_at("m/87'/0'/2'", "").to_string();
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "path-of-xpub",
            "--phrase-stdin",
            "--target-xpub",
            &target,
            "--add-path",
            "m/87'/0'/account'",
            "--json",
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_str(&String::from_utf8(out).unwrap()).unwrap();
    assert_eq!(v["path"], "m/87'/0'/2'");
    assert_eq!(v["account"], 2);
    // Template name for add-path entries is the literal template string.
    assert_eq!(v["template"], "m/87'/0'/account'");
}

// ---------------------------------------------------------------------------
// Cell 8 — --add-path with no `account` token: searched once at exact path.
// ---------------------------------------------------------------------------
#[test]
fn path_of_xpub_add_path_no_account_token_searched_once() {
    let target = xpub_at("m/9999'/0'/0'", "").to_string();
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "path-of-xpub",
            "--phrase-stdin",
            "--target-xpub",
            &target,
            "--add-path",
            "m/9999'/0'/0'",
            "--json",
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_str(&String::from_utf8(out).unwrap()).unwrap();
    assert_eq!(v["path"], "m/9999'/0'/0'");
    // account field should be `null` for no-account-token templates.
    assert!(
        v["account"].is_null(),
        "account must be null for token-less add-path; got {:?}",
        v["account"]
    );
}

// ---------------------------------------------------------------------------
// Cell 9 — `--phrase-stdin` intake works.
// ---------------------------------------------------------------------------
#[test]
fn path_of_xpub_phrase_stdin() {
    let target = xpub_at("m/84'/0'/0'", "").to_string();
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "path-of-xpub",
            "--phrase-stdin",
            "--target-xpub",
            &target,
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(0);
}

// ---------------------------------------------------------------------------
// Cell 10 — `--ms1` intake works.
// ---------------------------------------------------------------------------
#[test]
fn path_of_xpub_ms1_intake() {
    // Encode the test phrase's entropy (16 zero bytes) as an ms1 card.
    let mnemonic = Mnemonic::parse_in(bip39::Language::English, PHRASE).unwrap();
    let entropy = mnemonic.to_entropy();
    let ms1 = ms_codec::encode(
        ms_codec::Tag::ENTR,
        &ms_codec::Payload::Entr(entropy.clone()),
    )
    .expect("ms_codec::encode");

    let target = xpub_at("m/84'/0'/0'", "").to_string();
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "path-of-xpub",
            "--ms1",
            &ms1,
            "--target-xpub",
            &target,
        ])
        .assert()
        .code(0);
}

// ---------------------------------------------------------------------------
// Cell 11 — `--passphrase` alters the master; matching target requires
//   matching passphrase.
// ---------------------------------------------------------------------------
#[test]
fn path_of_xpub_passphrase_alters_match() {
    let target_with_pp = xpub_at("m/84'/0'/0'", "TREZOR").to_string();

    // With matching passphrase → match.
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "path-of-xpub",
            "--phrase-stdin",
            "--passphrase",
            "TREZOR",
            "--target-xpub",
            &target_with_pp,
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(0);

    // Without passphrase against the same passphrase-derived xpub → no match.
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "path-of-xpub",
            "--phrase-stdin",
            "--target-xpub",
            &target_with_pp,
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(4);
}

// ---------------------------------------------------------------------------
// Cell 12 — multisig SLIP-0132 variant accepted; matches under BIP-48 path.
// ---------------------------------------------------------------------------
#[test]
fn path_of_xpub_multisig_variant_zpub_accepted_searches_multisig_paths() {
    let target = xpub_as_multisig_zpub(&xpub_at("m/48'/0'/0'/2'", ""));
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "path-of-xpub",
            "--phrase-stdin",
            "--target-xpub",
            &target,
            "--json",
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_str(&String::from_utf8(out).unwrap()).unwrap();
    assert_eq!(v["path"], "m/48'/0'/0'/2'");
    assert_eq!(v["template"], "bip48-wsh");
    assert_eq!(v["target_xpub_variant"], "Zpub");
}

// ---------------------------------------------------------------------------
// Cell 13 — invalid phrase → exit 1.
// ---------------------------------------------------------------------------
#[test]
fn path_of_xpub_invalid_phrase_exits_1() {
    let target = xpub_at("m/84'/0'/0'", "").to_string();
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "path-of-xpub",
            "--phrase",
            "not a valid bip39 phrase here at all really",
            "--target-xpub",
            &target,
        ])
        .assert()
        .code(1);
}

// ---------------------------------------------------------------------------
// Cell 14 — invalid xpub → exit 1.
// ---------------------------------------------------------------------------
#[test]
fn path_of_xpub_invalid_xpub_exits_1() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "path-of-xpub",
            "--phrase-stdin",
            "--target-xpub",
            "xpubINVALIDinvalidinvalid",
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(1);
}

// ---------------------------------------------------------------------------
// Cell 15 (Cycle F `ms1-repair-demote-to-candidate` FLIP — SPEC §5.3): a
// corrupted ms1 no longer short-circuits under xpub-search's auto-repair —
// the candidate correction is demoted, so the ORIGINAL decode error
// surfaces (`ms_codec::Error::Codex32` invalid-checksum ⇒ `ms_codec_exit_code`
// ⇒ exit 1, via `ToolkitError::Bip39`/`MsCodec` mapping), NOT exit 5, NOT
// exit 4/2 — plus the standalone-inline I2 advisory on stderr. Confirmed by
// running the binary.
// ---------------------------------------------------------------------------
#[test]
fn path_of_xpub_ms1_decode_failure_auto_fires() {
    // Build a valid ms1 then flip a single character.
    let mnemonic = Mnemonic::parse_in(bip39::Language::English, PHRASE).unwrap();
    let entropy = mnemonic.to_entropy();
    let ms1 = ms_codec::encode(
        ms_codec::Tag::ENTR,
        &ms_codec::Payload::Entr(entropy.clone()),
    )
    .expect("ms_codec::encode");
    let bad = {
        const ALPHABET: &str = "qpzry9x8gf2tvdw0s3jn54khce6mua7l";
        let sep = ms1.rfind('1').unwrap();
        let (prefix, rest) = ms1.split_at(sep + 1);
        let mut chars: Vec<char> = rest.chars().collect();
        // Flip position 17 in the data-part.
        let was = chars[17];
        let was_idx = ALPHABET.find(was).unwrap();
        chars[17] = ALPHABET.chars().nth((was_idx + 1) % 32).unwrap();
        let mut out = String::from(prefix);
        for c in chars {
            out.push(c);
        }
        out
    };

    let target = xpub_at("m/84'/0'/0'", "").to_string();
    Command::cargo_bin("mnemonic")
        .unwrap()
        .env("MNEMONIC_FORCE_TTY", "1")
        .args([
            "xpub-search",
            "path-of-xpub",
            "--ms1",
            &bad,
            "--target-xpub",
            &target,
        ])
        .assert()
        .code(1)
        .stdout(predicate::str::contains("# Repair report").not())
        .stderr(predicate::str::contains(
            "candidate correction exists but a seed card cannot be self-verified",
        ))
        .stderr(predicate::str::contains("mnemonic repair --ms1"));
}

// ---------------------------------------------------------------------------
// Cell 16 — `--no-auto-repair` disables auto-fire (exit != 5).
// ---------------------------------------------------------------------------
#[test]
fn path_of_xpub_no_auto_repair_flag_disables_auto_fire() {
    let mnemonic = Mnemonic::parse_in(bip39::Language::English, PHRASE).unwrap();
    let entropy = mnemonic.to_entropy();
    let ms1 = ms_codec::encode(
        ms_codec::Tag::ENTR,
        &ms_codec::Payload::Entr(entropy.clone()),
    )
    .expect("ms_codec::encode");
    let bad = {
        const ALPHABET: &str = "qpzry9x8gf2tvdw0s3jn54khce6mua7l";
        let sep = ms1.rfind('1').unwrap();
        let (prefix, rest) = ms1.split_at(sep + 1);
        let mut chars: Vec<char> = rest.chars().collect();
        let was = chars[17];
        let was_idx = ALPHABET.find(was).unwrap();
        chars[17] = ALPHABET.chars().nth((was_idx + 1) % 32).unwrap();
        let mut out = String::from(prefix);
        for c in chars {
            out.push(c);
        }
        out
    };

    let target = xpub_at("m/84'/0'/0'", "").to_string();
    let assertion = Command::cargo_bin("mnemonic")
        .unwrap()
        .env("MNEMONIC_FORCE_TTY", "1")
        .args([
            "--no-auto-repair",
            "xpub-search",
            "path-of-xpub",
            "--ms1",
            &bad,
            "--target-xpub",
            &target,
        ])
        .assert()
        .failure();
    let code = assertion.get_output().status.code().unwrap();
    assert_ne!(
        code, 5,
        "--no-auto-repair must not trigger exit 5; got {code}"
    );
}

// ---------------------------------------------------------------------------
// Cell 17 — JSON envelope shape, including `target_xpub_variant: null` for
//   canonical xpub input.
// ---------------------------------------------------------------------------
#[test]
fn path_of_xpub_json_envelope_byte_exact_match() {
    let target = xpub_at("m/84'/0'/0'", "").to_string();
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "path-of-xpub",
            "--phrase-stdin",
            "--target-xpub",
            &target,
            "--json",
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();
    let body = String::from_utf8(out).unwrap();
    let v: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(v["schema_version"], "1");
    assert_eq!(v["mode"], "path-of-xpub");
    assert_eq!(v["result"], "match");
    assert_eq!(v["path"], "m/84'/0'/0'");
    assert_eq!(v["template"], "bip84");
    assert_eq!(v["account"], 0);
    assert!(
        v["target_xpub_variant"].is_null(),
        "canonical xpub input must serialize target_xpub_variant as null; got {:?}",
        v["target_xpub_variant"]
    );
    assert!(v["searched_count"].is_number());
}

// ---------------------------------------------------------------------------
// Cell 18 — argv-leak advisory on inline `--phrase`.
// ---------------------------------------------------------------------------
#[test]
fn path_of_xpub_argv_leak_advisory_on_inline_phrase() {
    let target = xpub_at("m/84'/0'/0'", "").to_string();
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "path-of-xpub",
            "--phrase",
            PHRASE,
            "--target-xpub",
            &target,
        ])
        .assert()
        .code(0)
        .stderr(predicate::str::contains(
            "secret material on argv (--phrase)",
        ));
}

// ---------------------------------------------------------------------------
// Cell 19 — positional ms1 intake (no flag).
// ---------------------------------------------------------------------------
#[test]
fn path_of_xpub_positional_ms1_works() {
    let mnemonic = Mnemonic::parse_in(bip39::Language::English, PHRASE).unwrap();
    let entropy = mnemonic.to_entropy();
    let ms1 = ms_codec::encode(
        ms_codec::Tag::ENTR,
        &ms_codec::Payload::Entr(entropy.clone()),
    )
    .expect("ms_codec::encode");

    let target = xpub_at("m/84'/0'/0'", "").to_string();
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "path-of-xpub",
            "--target-xpub",
            &target,
            &ms1,
        ])
        .assert()
        .code(0);
}

// NOTE: the `XpubSearchEnvelope` serde round-trip unit cell lives in the
// module itself at `crates/mnemonic-toolkit/src/cmd/xpub_search/mod.rs`
// `#[cfg(test)] mod tests` (cmd modules are binary-private; no lib re-
// export for `cmd::xpub_search`).
