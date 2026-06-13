//! `mnemonic xpub-search account-of-descriptor` masked-`older()` advisory wiring
//! tests (PLAN_older_timelock_advisory Task 10).
//!
//! Two funnels in `cmd/xpub_search/descriptor_intake.rs`:
//! - **Literal funnel (Adapter B):** `--descriptor wsh(andor(...older(65536)...))`
//!   → parsed via rust-miniscript → `older_advisories_descriptor` → stderr advisory,
//!   exit 0.
//! - **md1-card funnel (Adapter A, A-raw-card):** a real `older(65536)` md1 card
//!   (generated via `bundle --descriptor ...`) fed back to `xpub-search` via the
//!   md1-card path → `older_advisories_tree` → stderr advisory, exit 0.
//!   (`older(65536)` is bit-31-clear so it encodes normally; this proves the
//!   A-raw-card hook WIRING — bit-31 reachability itself is unit-tested in Task 3.)
//!
//! Fixture: BIP-39 vector `abandon × 11 about` is the universal master.

use assert_cmd::Command;
use bip39::Mnemonic;
use bitcoin::bip32::{DerivationPath, Fingerprint, Xpriv, Xpub};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::NetworkKind;
use serde_json::Value;
use std::str::FromStr;

const PHRASE: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

const OTHER_PHRASE: &str =
    "legal winner thank year wave sausage worth useful legal winner thank yellow";

/// The masked-older advisory substring shared by both funnels (SPEC §5 Masked form).
const ADVISORY_SUBSTR: &str = "older(65536) is consensus-masked";

/// Compute the xpub at `path` for the test phrase.
fn xpub_at(phrase: &str, path: &str) -> Xpub {
    let mnemonic = Mnemonic::parse_in(bip39::Language::English, phrase).unwrap();
    let seed = mnemonic.to_seed("");
    let secp = Secp256k1::new();
    let master = Xpriv::new_master(NetworkKind::Main, &seed).unwrap();
    let dp = DerivationPath::from_str(path).unwrap();
    let xpriv = master.derive_priv(&secp, &dp).unwrap();
    Xpub::from_priv(&secp, &xpriv)
}

/// Master fingerprint of `phrase`.
fn master_fp(phrase: &str) -> Fingerprint {
    let mnemonic = Mnemonic::parse_in(bip39::Language::English, phrase).unwrap();
    let seed = mnemonic.to_seed("");
    let secp = Secp256k1::new();
    let master = Xpriv::new_master(NetworkKind::Main, &seed).unwrap();
    Xpub::from_priv(&secp, &master).fingerprint()
}

/// A 2-of-2 `wsh(andor(...))` literal descriptor carrying a masked `older(65536)`.
/// cosigner @0 = PHRASE (the match), @1 = OTHER_PHRASE, both at m/48'/0'/0'/2'.
fn masked_literal_descriptor() -> String {
    let xpub0 = xpub_at(PHRASE, "m/48'/0'/0'/2'");
    let xpub1 = xpub_at(OTHER_PHRASE, "m/48'/0'/0'/2'");
    let fp0 = master_fp(PHRASE);
    let fp1 = master_fp(OTHER_PHRASE);
    // andor(pk(A), older(65536), pk(B)): A normal-spends; after a (consensus-masked)
    // relative timelock B can recover. older(65536) is consensus-masked → advisory.
    format!(
        "wsh(andor(pk([{}/48'/0'/0'/2']{}/<0;1>/*),older(65536),pk([{}/48'/0'/0'/2']{}/<0;1>/*)))",
        fp0, xpub0, fp1, xpub1
    )
}

/// A clean (un-masked) `wsh(andor(...older(2016)...))` literal descriptor.
fn clean_literal_descriptor() -> String {
    let xpub0 = xpub_at(PHRASE, "m/48'/0'/0'/2'");
    let xpub1 = xpub_at(OTHER_PHRASE, "m/48'/0'/0'/2'");
    let fp0 = master_fp(PHRASE);
    let fp1 = master_fp(OTHER_PHRASE);
    format!(
        "wsh(andor(pk([{}/48'/0'/0'/2']{}/<0;1>/*),older(2016),pk([{}/48'/0'/0'/2']{}/<0;1>/*)))",
        fp0, xpub0, fp1, xpub1
    )
}

// ---------------------------------------------------------------------------
// (a) literal funnel: --descriptor wsh(andor(...older(65536)...)) → stderr
//     advisory + exit 0.
// ---------------------------------------------------------------------------
#[test]
fn xpub_search_literal_funnel_masked_older_emits_advisory() {
    let descriptor = masked_literal_descriptor();
    let assertion = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "account-of-descriptor",
            "--phrase-stdin",
            "--descriptor",
            &descriptor,
            "--json",
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(0);
    let output = assertion.get_output();
    let stderr = String::from_utf8(output.stderr.clone()).unwrap();
    assert!(
        stderr.contains(ADVISORY_SUBSTR),
        "literal funnel must emit the masked-older advisory on stderr; got: {stderr}"
    );
    // Non-blocking: the search still ran and matched.
    let stdout = String::from_utf8(output.stdout.clone()).unwrap();
    let v: Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(v["result"], "match");
    assert_eq!(v["descriptor_shape"], "literal_xpub");
}

// ---------------------------------------------------------------------------
// (b) md1-card funnel (A-raw-card): generate a real older(65536) md1 card via
//     `bundle --descriptor`, feed it to xpub-search → stderr advisory + exit 0.
//     Proves the md1-card → older_advisories_tree hook wiring.
// ---------------------------------------------------------------------------
#[test]
fn xpub_search_md1_card_funnel_masked_older_emits_advisory() {
    let xpub = xpub_at(PHRASE, "m/84'/0'/0'");
    let fp_hex = master_fp(PHRASE).to_string();
    // Single-sig + masked older(65536) recovery leg. Template uses @0 placeholder
    // for the searchable cosigner; the older() literal is fixed.
    let descriptor_template =
        format!("wsh(and_v(v:pk(@0[{fp_hex}/84'/0'/0']/<0;1>/*),older(65536)))");
    // Emit a bundle to obtain the md1 card (older(65536) is bit-31-clear → encodes
    // normally).
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            &descriptor_template,
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.xpub={xpub}"),
            "--slot",
            &format!("@0.fingerprint={fp_hex}"),
            "--json",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let v: Value = serde_json::from_str(&stdout).unwrap();
    let md1_strs: Vec<String> = v["md1"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_str().unwrap().to_string())
        .collect();
    let stdin_payload = md1_strs.join("\n");

    // Feed the md1 card(s) back to xpub-search via the md1-card funnel.
    let assertion = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "account-of-descriptor",
            "--phrase",
            PHRASE,
            "--descriptor-from",
            "md1=-",
            "--json",
        ])
        .write_stdin(stdin_payload)
        .assert()
        .code(0);
    let output = assertion.get_output();
    let stderr = String::from_utf8(output.stderr.clone()).unwrap();
    assert!(
        stderr.contains(ADVISORY_SUBSTR),
        "md1-card funnel must emit the masked-older advisory on stderr; got: {stderr}"
    );
    let stdout = String::from_utf8(output.stdout.clone()).unwrap();
    let v: Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(v["result"], "match");
    assert_eq!(v["descriptor_shape"], "md1");
}

// ---------------------------------------------------------------------------
// Clean case: a literal descriptor with a clean older(2016) → NO advisory.
// ---------------------------------------------------------------------------
#[test]
fn xpub_search_literal_funnel_clean_older_no_advisory() {
    let descriptor = clean_literal_descriptor();
    let assertion = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "account-of-descriptor",
            "--phrase-stdin",
            "--descriptor",
            &descriptor,
            "--json",
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(0);
    let output = assertion.get_output();
    let stderr = String::from_utf8(output.stderr.clone()).unwrap();
    assert!(
        !stderr.contains("advisory: older"),
        "clean older(2016) must NOT emit any older advisory; got: {stderr}"
    );
    let stdout = String::from_utf8(output.stdout.clone()).unwrap();
    let v: Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(v["result"], "match");
}
