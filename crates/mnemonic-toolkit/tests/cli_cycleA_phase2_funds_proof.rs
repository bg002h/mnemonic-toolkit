//! Cycle A Phase 2 — funds-proof regressions (born-green).
//!
//! `design/SPEC_cycleA_descriptor_use_site_collapse.md` §1/§6/§8;
//! `design/IMPLEMENTATION_PLAN_cycleA_descriptor_use_site_collapse.md` Phase 2.
//!
//! Phase 1 already shipped the fix (the `lex_placeholders` unconsumed-residue
//! reject, `parse_descriptor.rs`) and its per-shape unit/CLI reject coverage.
//! These tests are ADDITIVE: they LOCK the end-to-end *funds* behavior the fix
//! is actually for, so any future regression here is a wrong-address /
//! false-pass funds bug, not a mere validation gap.
//!
//! - 2a: `verify-bundle` no longer FALSE-PASSES (exit 0) on a concrete
//!   fixed-use-site-step descriptor — the exact mechanism SPEC §1 names
//!   (verify-bundle re-parsed the user's descriptor through the SAME
//!   collapsing lexer as encode, so it silently agreed a wrong card was
//!   right). Primary = concrete-descriptor verify fork → exit 2 /
//!   `DescriptorParse` (plan-R0 I-B). Secondary (optional) = the `@N`-template
//!   verify fork → exit 4 / `DescriptorReparseFailed`.
//! - 2b: the BIP-84 oracle. POSITIVE — a correctly-encoded `<0;1>/*`
//!   single-sig card for `abandon×11 about`, taken through the exact pipeline
//!   that had the bug (`bundle --descriptor` → `concrete_keys_to_placeholders`
//!   → `lex_placeholders`), restores via `restore --md1` ALONE to the true
//!   BIP-84 first receive address. NEGATIVE — the fixed `/0/*` step that would
//!   have collapsed to the WRONG address can no longer be encoded at all.

use assert_cmd::Command;
use bip39::Mnemonic;
use bitcoin::bip32::{DerivationPath, Xpriv, Xpub};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::{Address, KnownHrp, NetworkKind};
use serde_json::Value;
use std::str::FromStr;

// BIP-39 test vector "abandon × 11 about" — the same phrase already pinned as
// the BIP-84 oracle in `cli_restore.rs` (`TREZOR_12`/`FIRST_RECV_BIP84`).
const TREZOR_12: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

/// Authoritative BIP-84 test-vector first receive address (account 0, chain 0,
/// index 0 — `m/84'/0'/0'/0/0`). Already pinned independently at
/// `cli_restore.rs:26` (`FIRST_RECV_BIP84`) via `restore --from phrase=...
/// --template bip84`, which drives the canonical-template synthesis engine
/// directly and never touches `lex_placeholders`. This file re-proves the SAME
/// oracle value through the DIFFERENT pipeline that had the bug: a concrete
/// `bundle --descriptor` (→ `concrete_keys_to_placeholders` →
/// `lex_placeholders`) followed by `restore --md1`.
const FIRST_RECV_BIP84: &str = "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu";

/// The WRONG address the PRE-FIX collapse bug would have restored (SPEC §1):
/// `wildcard_for` (`to_miniscript.rs:133-140`) ALWAYS emits a bare wildcard use
/// site, so a `/0/*` descriptor's dropped `/0` step meant "first receive"
/// walked ONE derivation step from the account xpub (`account_xpub/0`)
/// instead of TWO (`account_xpub/0/0`). Independently re-derived (bypassing
/// the toolkit entirely) in `collapsed_wrong_oracle_value_independently_confirmed`
/// below, rather than trusted from the SPEC text alone.
const COLLAPSED_WRONG_FIRST_RECV: &str = "bc1q8vph849lf3e9rrj85hsxrzlv949rtahe794k6p";

fn mnemonic() -> Command {
    Command::cargo_bin("mnemonic").unwrap()
}

/// `(fingerprint_hex, account_xpub)` for `TREZOR_12` at the BIP-84 account-0
/// path (`m/84'/0'/0'`), computed independently via the `bitcoin`/`bip39`
/// crates (mirrors `cli_cross_start_convergence.rs::derive_account_xpub` /
/// `cli_xpub_search_address_of_xpub.rs::account_xpub_at`), so every value
/// this file asserts is provably derived, not copy-pasted from elsewhere.
fn bip84_account0() -> (String, Xpub) {
    let secp = Secp256k1::new();
    let m = Mnemonic::parse_in(bip39::Language::English, TREZOR_12).unwrap();
    let seed = m.to_seed("");
    let master = Xpriv::new_master(NetworkKind::Main, &seed).unwrap();
    let fp = master.fingerprint(&secp).to_string().to_lowercase();
    let dp = DerivationPath::from_str("m/84'/0'/0'").unwrap();
    let xpriv = master.derive_priv(&secp, &dp).unwrap();
    let xpub = Xpub::from_priv(&secp, &xpriv);
    (fp, xpub)
}

/// Render a mainnet P2WPKH address for an xpub (no further derivation).
fn p2wpkh_addr(xpub: &Xpub) -> String {
    Address::p2wpkh(&xpub.to_pub(), KnownHrp::Mainnet).to_string()
}

/// Flatten a bundle JSON `mk1` field into `--mk1 <chunk>` flag pairs. Single-sig
/// bundles emit a FLAT `Vec<String>`; multisig bundles emit an array-of-arrays
/// (one inner array per cosigner) — handle both shapes (mirrors
/// `cli_restore_multisig.rs`'s `mk1_per` decoding).
fn mk1_flags(v: &Value) -> Vec<String> {
    let mut out = Vec::new();
    for el in v["mk1"].as_array().expect("mk1 array") {
        match el {
            Value::String(s) => {
                out.push("--mk1".into());
                out.push(s.clone());
            }
            Value::Array(inner) => {
                for chunk in inner {
                    out.push("--mk1".into());
                    out.push(chunk.as_str().unwrap().to_string());
                }
            }
            other => panic!("unexpected mk1 element shape: {other:?}"),
        }
    }
    out
}

fn md1_flags(v: &Value) -> Vec<String> {
    let mut out = Vec::new();
    for chunk in v["md1"].as_array().expect("md1 array") {
        out.push("--md1".into());
        out.push(chunk.as_str().unwrap().to_string());
    }
    out
}

// ============================================================================
// 2a — verify-bundle false-pass closure (the headline regression, SPEC §1,
// plan-R0 I-B / M-7).
// ============================================================================

/// PRIMARY: the concrete-descriptor verify fork
/// (`verify_bundle.rs::descriptor_mode_verify_run` →
/// `descriptor_concrete_to_resolved_slots` → `parse_descriptor` →
/// `lex_placeholders`) now REJECTS a fixed `/0/*` use-site step at RE-PARSE,
/// BEFORE any supplied `--md1`/`--mk1` card is ever compared. Pre-fix, this
/// exact re-parse silently collapsed `/0/*` to `/*` — identically to encode —
/// so verify-bundle FALSE-PASSED (exit 0) even though the user's descriptor
/// and the card encoded different wallets. Proving the closure needs no
/// "wrong card" fixture: because the reject fires before comparison, ANY
/// syntactically-valid card set suffices (built here from the SAME wallet's
/// correctly-encoded `<0;1>/*` descriptor, for realism).
#[test]
fn verify_bundle_concrete_fixed_use_site_step_rejects_before_card_comparison() {
    let (fp, acct_xpub) = bip84_account0();
    let valid_desc = format!("wpkh([{fp}/84'/0'/0']{acct_xpub}/<0;1>/*)");

    let produced = mnemonic()
        .args([
            "bundle",
            "--descriptor",
            &valid_desc,
            "--network",
            "mainnet",
            "--json",
            "--no-engraving-card",
        ])
        .assert()
        .success();
    let v: Value = serde_json::from_slice(&produced.get_output().stdout).unwrap();

    // The fixed-step descriptor: SAME wallet, `/0/*` instead of `/<0;1>/*` —
    // the exact shape SPEC §1 names as the false-pass site.
    let concrete_fixed = format!("wpkh([{fp}/84'/0'/0']{acct_xpub}/0/*)");
    let mut args: Vec<String> = vec![
        "verify-bundle".into(),
        "--descriptor".into(),
        concrete_fixed,
        "--network".into(),
        "mainnet".into(),
    ];
    args.extend(md1_flags(&v));
    args.extend(mk1_flags(&v));

    let out = mnemonic().args(&args).output().unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(
        !out.status.success(),
        "verify-bundle must now REJECT a concrete /0/* descriptor (pre-fix: a \
         silent false-pass, exit 0); stderr: {stderr}"
    );
    assert_eq!(
        out.status.code(),
        Some(2),
        "must be exit 2 / DescriptorParse — the concrete-descriptor verify \
         fork (plan-R0 I-B); exit 4 is only the @N-template verify fork; \
         stderr: {stderr}"
    );
    assert!(
        stderr.contains("multipath") && stderr.contains("<a;b>"),
        "stderr must point at the multipath remedy: {stderr}"
    );
}

/// OPTIONAL secondary (SPEC D2 / plan-R0 I-B): the `@N`-template verify fork
/// (`lex_placeholders` called directly on the raw `--descriptor` string,
/// `verify_bundle.rs:1375`) wraps the SAME residue reject as
/// `DescriptorReparseFailed{detail}` → exit 4 — a DIFFERENT exit code than the
/// concrete fork above (per-path error variant, plan-R0 I-B correction).
#[test]
fn verify_bundle_at_n_template_fixed_use_site_step_rejects_exit_4() {
    let (fp, acct_xpub) = bip84_account0();
    let placeholder_desc = format!("wpkh(@0[{fp}/84'/0'/0']/0/*)");

    let out = mnemonic()
        .args([
            "verify-bundle",
            "--descriptor",
            &placeholder_desc,
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.xpub={acct_xpub}"),
            // Reject fires inside `lex_placeholders` before any card is
            // consulted, so empty sentinels satisfy clap's required
            // `--mk1`/`--md1` without affecting the outcome (mirrors
            // `cli_non_canonical_descriptor.rs`'s empty-sentinel pattern).
            "--mk1",
            "",
            "--md1",
            "",
        ])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(
        !out.status.success(),
        "the @N-template verify fork must also reject a fixed /0/* use-site \
         step; stderr: {stderr}"
    );
    assert_eq!(
        out.status.code(),
        Some(4),
        "must be exit 4 / DescriptorReparseFailed — the @N-template verify \
         fork (distinct from the concrete fork's exit 2); stderr: {stderr}"
    );
    assert!(
        stderr.contains("re-parse failed") && stderr.contains("multipath"),
        "stderr must name the re-parse failure + the multipath remedy: {stderr}"
    );
}

// ============================================================================
// 2b — BIP-84 oracle (SPEC §1/§8; plan Phase 2).
// ============================================================================

/// Sanity-check the SPEC's cited "collapsed wrong" oracle value is itself
/// correct — external-fact verification, not a trust-the-draft-doc citation:
/// independently re-derive `account_xpub/0` (bypassing the toolkit entirely)
/// and confirm it equals `COLLAPSED_WRONG_FIRST_RECV`, and that it is
/// DISJOINT from the true first receive (`account_xpub/0/0`).
#[test]
fn collapsed_wrong_oracle_value_independently_confirmed() {
    let (_fp, acct_xpub) = bip84_account0();
    let secp = Secp256k1::new();

    let collapsed_child = acct_xpub
        .derive_pub(&secp, &DerivationPath::from_str("m/0").unwrap())
        .unwrap();
    let collapsed_addr = p2wpkh_addr(&collapsed_child);
    assert_eq!(
        collapsed_addr, COLLAPSED_WRONG_FIRST_RECV,
        "independent re-derivation of account_xpub/0 (the ONE-step-short \
         derivation the pre-fix collapse bug performed) must match the SPEC \
         §1-cited wrong oracle value"
    );

    let true_child = acct_xpub
        .derive_pub(&secp, &DerivationPath::from_str("m/0/0").unwrap())
        .unwrap();
    let true_addr = p2wpkh_addr(&true_child);
    assert_eq!(
        true_addr, FIRST_RECV_BIP84,
        "account_xpub/0/0 (the TRUE two-step first-receive derivation) must \
         match the authoritative BIP-84 oracle"
    );
    assert_ne!(
        collapsed_addr, true_addr,
        "the collapsed and true first-receive addresses must be DISJOINT \
         (SPEC §1: xpub/0/* derives xpub/0/i; xpub/* derives xpub/i)"
    );
}

/// POSITIVE — a correctly-encoded `<0;1>/*` single-sig card for
/// `abandon×11 about`, taken through the EXACT pipeline that had the bug
/// (`bundle --descriptor` → `concrete_keys_to_placeholders` →
/// `lex_placeholders`), restores via `restore --md1` ALONE (no `--from`
/// needed — the md1 Policy form embeds the real xpub; this is a pure
/// watch-only round trip) to the TRUE BIP-84 first receive address — never
/// the collapsed wrong one. This is a DIRECT first-receive-address assertion
/// (not a proxy): `restore --md1` prints a `first recv:` line, and the
/// reconstructed descriptor's use-site is asserted as the PRESERVED `<0;1>/*`
/// multipath (not silently collapsed).
#[test]
fn bundle_descriptor_multipath_restores_to_true_bip84_first_receive() {
    let (fp, acct_xpub) = bip84_account0();
    let valid_desc = format!("wpkh([{fp}/84'/0'/0']{acct_xpub}/<0;1>/*)");

    let produced = mnemonic()
        .args([
            "bundle",
            "--descriptor",
            &valid_desc,
            "--network",
            "mainnet",
            "--json",
            "--no-engraving-card",
        ])
        .assert()
        .success();
    let v: Value = serde_json::from_slice(&produced.get_output().stdout).unwrap();

    let mut args: Vec<String> = vec!["restore".into(), "--network".into(), "mainnet".into()];
    args.extend(md1_flags(&v));

    let out = mnemonic().args(&args).output().unwrap();
    assert!(
        out.status.success(),
        "restore --md1 alone must succeed (watch-only, no --from needed): {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("<0;1>/*"),
        "reconstructed descriptor must show the PRESERVED multipath use-site \
         (not collapsed to a bare `/*`): {stdout}"
    );
    assert!(
        stdout.contains(FIRST_RECV_BIP84),
        "must derive the TRUE BIP-84 first receive address \
         ({FIRST_RECV_BIP84}), not the collapsed wrong one \
         ({COLLAPSED_WRONG_FIRST_RECV}): {stdout}"
    );
    assert!(
        !stdout.contains(COLLAPSED_WRONG_FIRST_RECV),
        "must NEVER produce the collapsed wrong address: {stdout}"
    );
}

/// NEGATIVE — the fixed `/0/*` use-site step that would have collapsed to
/// `COLLAPSED_WRONG_FIRST_RECV` above can no longer be encoded AT ALL:
/// `bundle --descriptor` hard-rejects at encode (exit 2), so no card
/// producing that wrong address can ever reach an engraving plate. This
/// overlaps Phase-1's `cli_cross_start_convergence.rs` a4/a5 Group-A reject
/// cells in mechanism (same residue-reject floor); this cell is additive in
/// that it ties the reject DIRECTLY to the address-oracle values proven
/// above, in the same file as the positive oracle.
#[test]
fn bundle_descriptor_fixed_use_site_step_cannot_encode_the_collapsed_wallet() {
    let (fp, acct_xpub) = bip84_account0();
    let collapsing_desc = format!("wpkh([{fp}/84'/0'/0']{acct_xpub}/0/*)");

    let out = mnemonic()
        .args([
            "bundle",
            "--descriptor",
            &collapsing_desc,
            "--network",
            "mainnet",
            "--json",
            "--no-engraving-card",
        ])
        .output()
        .unwrap();
    assert!(
        !out.status.success(),
        "a fixed /0/* use-site step must be REJECTED at encode, not silently \
         collapsed into a card for the wrong wallet"
    );
    assert_eq!(out.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("multipath") && stderr.contains("<a;b>"),
        "{stderr}"
    );
}
