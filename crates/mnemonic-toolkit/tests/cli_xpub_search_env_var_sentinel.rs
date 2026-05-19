//! v0.26.0 — `@env:<VAR>` sentinel integration tests for the `xpub-search`
//! umbrella subcommands. Closes the architect-review C1 finding flagged by
//! the merge-coordinator: PR #24's xpub-search added `--phrase` + `--passphrase`
//! secret-bearing surfaces that PR #25's cross-cutting `@env:VAR` resolver
//! did NOT cover. This file pins the symmetric wire-up.
//!
//! Coverage: 3 modes (`path-of-xpub`, `account-of-descriptor`, `passphrase-of-xpub`)
//! × 2 surfaces (`--phrase`, `--passphrase`) — happy-path + missing-var-exit-1.
//! `address-of-xpub` has no seed material; skipped.
//!
//! Pattern mirrors `tests/cli_env_var_sentinel.rs` (Phase 1 cross-cutting tests
//! for bundle/verify-bundle/convert) and the per-subcommand argv-leak
//! advisory skip behavior on env-sentinel-bearing flags.

use assert_cmd::Command;
use bip39::Mnemonic;
use bitcoin::bip32::{DerivationPath, Xpriv, Xpub};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::NetworkKind;
use predicates::prelude::*;
use std::str::FromStr;

const PHRASE: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

/// Derive xpub at `path` for the test PHRASE + passphrase. Mirrors the
/// helper in `cli_xpub_search_path_of_xpub.rs:31`.
fn xpub_at(path: &str, passphrase: &str) -> Xpub {
    let mnemonic = Mnemonic::parse_in(bip39::Language::English, PHRASE).unwrap();
    let seed = mnemonic.to_seed(passphrase);
    let secp = Secp256k1::new();
    let master = Xpriv::new_master(NetworkKind::Main, &seed).unwrap();
    let dp = DerivationPath::from_str(path).unwrap();
    let xpriv = master.derive_priv(&secp, &dp).unwrap();
    Xpub::from_priv(&secp, &xpriv)
}

// ============================================================================
// path-of-xpub × {--phrase, --passphrase} × {happy, unset}
// ============================================================================

#[test]
fn path_of_xpub_env_phrase_happy_path() {
    let target = xpub_at("m/84'/0'/0'", "").to_string();
    Command::cargo_bin("mnemonic")
        .unwrap()
        .env("XPS_PHRASE", PHRASE)
        .args([
            "xpub-search",
            "path-of-xpub",
            "--phrase",
            "@env:XPS_PHRASE",
            "--target-xpub",
            &target,
            "--json",
        ])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("\"result\":\"match\""))
        // Argv-leak advisory MUST NOT fire when sentinel is used.
        .stderr(predicate::str::contains("secret material on argv").not());
}

#[test]
fn path_of_xpub_env_phrase_unset_fails() {
    let target = xpub_at("m/84'/0'/0'", "").to_string();
    Command::cargo_bin("mnemonic")
        .unwrap()
        .env_remove("XPS_PHRASE_UNSET")
        .args([
            "xpub-search",
            "path-of-xpub",
            "--phrase",
            "@env:XPS_PHRASE_UNSET",
            "--target-xpub",
            &target,
        ])
        .assert()
        .code(1)
        .stderr(predicate::str::contains(
            "--phrase: env-var XPS_PHRASE_UNSET referenced by sentinel is not set",
        ));
}

#[test]
fn path_of_xpub_env_passphrase_happy_path() {
    // Derive target under passphrase "swordfish" so the env-resolved
    // value is load-bearing — wrong passphrase → no_match (exit 4).
    let target = xpub_at("m/84'/0'/0'", "swordfish").to_string();
    Command::cargo_bin("mnemonic")
        .unwrap()
        .env("XPS_PASSPHRASE", "swordfish")
        .args([
            "xpub-search",
            "path-of-xpub",
            "--phrase-stdin",
            "--passphrase",
            "@env:XPS_PASSPHRASE",
            "--target-xpub",
            &target,
            "--json",
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(0)
        .stdout(predicate::str::contains("\"result\":\"match\""))
        .stderr(predicate::str::contains("secret material on argv (--passphrase)").not());
}

#[test]
fn path_of_xpub_env_passphrase_unset_fails() {
    let target = xpub_at("m/84'/0'/0'", "").to_string();
    Command::cargo_bin("mnemonic")
        .unwrap()
        .env_remove("XPS_PASSPHRASE_UNSET")
        .args([
            "xpub-search",
            "path-of-xpub",
            "--phrase-stdin",
            "--passphrase",
            "@env:XPS_PASSPHRASE_UNSET",
            "--target-xpub",
            &target,
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(1)
        .stderr(predicate::str::contains(
            "--passphrase: env-var XPS_PASSPHRASE_UNSET referenced by sentinel is not set",
        ));
}

// ============================================================================
// account-of-descriptor × {--phrase, --passphrase} × {happy, unset}
// ============================================================================
//
// Descriptor fixture: a single-sig wpkh descriptor with the test PHRASE's
// `m/84'/0'/0'` xpub. account-of-descriptor scans the standard candidates
// + add-paths × accounts looking for which cosigner role(s) the seed plays.
//
// We build the descriptor string with the literal xpub at m/84'/0'/0' so
// the test phrase is the matching seed at account=0.

fn wpkh_descriptor_at_account_0() -> String {
    let xp = xpub_at("m/84'/0'/0'", "").to_string();
    // No `[fp/path]` annotation; v0.19.0 silent-default-path inference applies.
    format!("wpkh({xp}/<0;1>/*)")
}

#[test]
fn account_of_descriptor_env_phrase_happy_path() {
    let desc = wpkh_descriptor_at_account_0();
    Command::cargo_bin("mnemonic")
        .unwrap()
        .env("AOD_PHRASE", PHRASE)
        .args([
            "xpub-search",
            "account-of-descriptor",
            "--phrase",
            "@env:AOD_PHRASE",
            "--descriptor",
            &desc,
            "--json",
        ])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("\"result\":\"match\""))
        .stderr(predicate::str::contains("secret material on argv (--phrase)").not());
}

#[test]
fn account_of_descriptor_env_passphrase_unset_fails() {
    let desc = wpkh_descriptor_at_account_0();
    Command::cargo_bin("mnemonic")
        .unwrap()
        .env_remove("AOD_PASSPHRASE_UNSET")
        .args([
            "xpub-search",
            "account-of-descriptor",
            "--phrase-stdin",
            "--passphrase",
            "@env:AOD_PASSPHRASE_UNSET",
            "--descriptor",
            &desc,
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(1)
        .stderr(predicate::str::contains(
            "--passphrase: env-var AOD_PASSPHRASE_UNSET referenced by sentinel is not set",
        ));
}

// ============================================================================
// passphrase-of-xpub × {--phrase, --passphrase} × {happy, unset}
// ============================================================================

#[test]
fn passphrase_of_xpub_env_phrase_happy_path() {
    let target = xpub_at("m/84'/0'/0'", "secret-pw").to_string();
    Command::cargo_bin("mnemonic")
        .unwrap()
        .env("POX_PHRASE", PHRASE)
        .args([
            "xpub-search",
            "passphrase-of-xpub",
            "--phrase",
            "@env:POX_PHRASE",
            "--passphrase",
            "secret-pw",
            "--target-xpub",
            &target,
            "--json",
        ])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("\"result\":\"match\""))
        .stderr(predicate::str::contains("secret material on argv (--phrase)").not());
}

#[test]
fn passphrase_of_xpub_env_passphrase_happy_path() {
    let target = xpub_at("m/84'/0'/0'", "secret-pw").to_string();
    Command::cargo_bin("mnemonic")
        .unwrap()
        .env("POX_PASSPHRASE", "secret-pw")
        .args([
            "xpub-search",
            "passphrase-of-xpub",
            "--phrase-stdin",
            "--passphrase",
            "@env:POX_PASSPHRASE",
            "--target-xpub",
            &target,
            "--json",
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(0)
        .stdout(predicate::str::contains("\"result\":\"match\""))
        .stderr(predicate::str::contains("secret material on argv (--passphrase)").not());
}
