//! v0.28.0 Phase P3 — Coldcard single-sig wallet.json parser.
//!
//! Per `design/SPEC_wallet_import_v0_28_0.md` §11.3.
//!
//! Coldcard's generic-wallet-export JSON shape (canonical authority:
//! `coldcard/firmware/shared/generic_wallet_export.py` +
//! `coldcard/firmware/docs/generic-wallet-export.md` at upstream master) is
//! a single JSON object with these top-level fields:
//!
//! ```json
//! {
//!   "chain": "BTC"   // or "XTN"
//!   "xfp": "<8-hex-uppercase-master-fingerprint>",
//!   "xpub": "<optional-master-xpub>",
//!   "account": <u32 account number>,
//!   "bip44": { "name": "p2pkh",        "deriv": "m/44'/<coin>'/<acct>'", "xfp": "<hex>", "xpub": "<account-xpub>", "first": "<addr>" },
//!   "bip49": { "name": "p2wpkh-p2sh",  "deriv": "m/49'/<coin>'/<acct>'", "xfp": "<hex>", "xpub": "<account-xpub>", "_pub": "<ypub>", "first": "<addr>" },
//!   "bip84": { "name": "p2wpkh",       "deriv": "m/84'/<coin>'/<acct>'", "xfp": "<hex>", "xpub": "<account-xpub>", "_pub": "<zpub>", "first": "<addr>" },
//!   "bip86": { "name": "p2tr",         "deriv": "m/86'/<coin>'/<acct>'", "xfp": "<hex>", "xpub": "<account-xpub>", "first": "<addr>" },
//!   "bip48_1": { ... },   // BIP-48 multisig hint (P2SH-P2WSH); IGNORED by single-sig parser
//!   "bip48_2": { ... }    // BIP-48 multisig hint (P2WSH); IGNORED by single-sig parser
//! }
//! ```
//!
//! Firmware-variance handling (SPEC §11.3 firmware-variance table):
//! - Mk1/Mk2 pre-2022: `xpub` top-level only (single BIP-44 wallet).
//! - Mk3+: per-bipNN sub-objects (`bip44`/`bip49`/`bip84`).
//! - Mk4+: adds `bip86` (taproot).
//! - Q (2024+): adds `bip48_1`/`bip48_2` multisig hints.
//!
//! Sniff signature (SPEC §11.3 Q3 lock, R0 I8 relaxed): top-level JSON
//! object with `chain ∈ {BTC, XTN}` + `xfp` + at-least-one-of
//! `{xpub, bip44, bip49, bip84, bip86, bip48_1, bip48_2}`.
//!
//! Parse contract:
//! 1. JSON-parse + top-level object check.
//! 2. Extract `chain` → network (BTC → mainnet, XTN → testnet).
//! 3. Extract `xfp` → master fingerprint.
//! 4. Dominant-BIP selection per SPEC §11.3.1: bip86 > bip84 > bip49 > bip44
//!    (with top-level `xpub` legacy-firmware fallback inferring BIP from
//!    SLIP-132 prefix).
//! 5. Build synthetic descriptor: `<wrapper>([xfp/deriv_no_m]xpub/<0;1>/*)#<csum>`
//!    where wrapper ∈ {pkh, sh(wpkh), wpkh, tr} per dominant BIP.
//! 6. Feed through `concrete_keys_to_placeholders` → `parse_descriptor`.
//! 7. Build single ResolvedSlot.
//! 8. Wrap in `ParsedImport` with `ImportProvenance::Coldcard(...)`.
//!
//! Phase P3A scope: parser skeleton + sniff impl + provenance metadata
//! struct decls + sniff unit tests. `parse()` returns
//! `Err(BadInput("P3B: parse not yet wired"))` — Phase P3B installs the
//! real body; Phase P3C flips the `cmd/import_wallet.rs` dispatch sites.

use super::{ParsedImport, WalletFormatParser};
use crate::error::ToolkitError;
use serde_json::Value;
use std::io::Write;

/// SPEC §11.3 — Coldcard single-sig wallet.json parser.
pub(crate) struct ColdcardParser;

/// SPEC §11.3 — `chain` field discriminator. `BTC` → mainnet, `XTN` → testnet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) enum ColdcardChain {
    Btc,
    Xtn,
}

impl ColdcardChain {
    /// Map to `bitcoin::Network`.
    pub(crate) fn to_network(self) -> bitcoin::Network {
        match self {
            ColdcardChain::Btc => bitcoin::Network::Bitcoin,
            ColdcardChain::Xtn => bitcoin::Network::Testnet,
        }
    }
}

/// SPEC §11.3.1 — Coldcard's dominant-BIP selection result. The single-sig
/// parser picks ONE of these per blob; bip48_* multisig hints are explicitly
/// excluded (multisig case is `--format coldcard-multisig`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) enum ColdcardBip {
    Bip44,
    Bip49,
    Bip84,
    Bip86,
}

impl ColdcardBip {
    /// JSON sub-object key for this BIP (e.g., `"bip44"`).
    pub(crate) fn as_json_key(self) -> &'static str {
        match self {
            ColdcardBip::Bip44 => "bip44",
            ColdcardBip::Bip49 => "bip49",
            ColdcardBip::Bip84 => "bip84",
            ColdcardBip::Bip86 => "bip86",
        }
    }
}

/// SPEC §11.3 — per-blob provenance metadata for a Coldcard single-sig parse.
/// Carried on `ImportProvenance::Coldcard(...)`; preserved for `--json`
/// envelope `coldcard_source_metadata` emit (P3C wiring).
///
/// P3A interim: the variant is constructed by P3B's `parse()` body + the
/// `ImportProvenance::Coldcard(...)` enum slot lands at P3C; the
/// `#[allow(dead_code)]` annotations come off as the wiring lands.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct ColdcardSourceMetadata {
    /// Top-level `chain` (BTC | XTN).
    pub(crate) chain: ColdcardChain,
    /// Top-level `xfp` decoded to 4-byte master fingerprint.
    pub(crate) xfp: [u8; 4],
    /// Dominant-BIP block selected per SPEC §11.3.1 (bip86 > bip84 > bip49 > bip44).
    pub(crate) bip_derivation: ColdcardBip,
    /// Top-level `account` field (u32). Default 0 if absent (legacy firmware).
    pub(crate) raw_account: u32,
    /// Top-level fields encountered in the blob but not preserved on the
    /// import-side provenance (mirrors `CoreSourceMetadata.dropped_fields`).
    pub(crate) dropped_fields: Vec<String>,
}

/// Top-level keys preserved on the Coldcard envelope by the toolkit's parse.
/// Any other top-level field surfaces in `ColdcardSourceMetadata.dropped_fields`
/// and drives a stderr NOTICE per SPEC §2.4. Mirrors
/// `SPECTER_PRESERVED_TOP_LEVEL_KEYS`.
///
/// P3A interim: consumed by P3B's parse body for dropped-field detection;
/// `#[allow(dead_code)]` comes off when P3B wires it.
#[allow(dead_code)]
pub(crate) const COLDCARD_PRESERVED_TOP_LEVEL_KEYS: &[&str] = &[
    "chain", "xfp", "xpub", "account", "bip44", "bip49", "bip84", "bip86",
    "bip48_1", "bip48_2",
];

impl WalletFormatParser for ColdcardParser {
    /// SPEC §11.3 (Q3 lock) sniff: top-level JSON object containing ALL of:
    /// (1) `chain ∈ {"BTC", "XTN"}` as a string,
    /// (2) `xfp` as a string,
    /// (3) at-least-one-of `{xpub, bip44, bip49, bip84, bip86, bip48_1, bip48_2}`.
    ///
    /// The disjunction in (3) absorbs Coldcard firmware variance — different
    /// firmware versions emit different combinations of per-BIP derivation
    /// blocks.
    fn sniff(blob: &[u8]) -> bool {
        let trimmed = trim_leading_ws(blob);
        if !trimmed.starts_with(b"{") {
            return false;
        }
        let value: Value = match serde_json::from_slice(blob) {
            Ok(v) => v,
            Err(_) => return false,
        };
        let obj = match value.as_object() {
            Some(o) => o,
            None => return false,
        };
        // (1) chain: must be string "BTC" or "XTN".
        let chain_ok = obj
            .get("chain")
            .and_then(|v| v.as_str())
            .map(|s| s == "BTC" || s == "XTN")
            .unwrap_or(false);
        if !chain_ok {
            return false;
        }
        // (2) xfp: must be string.
        if obj.get("xfp").and_then(|v| v.as_str()).is_none() {
            return false;
        }
        // (3) at-least-one-of: xpub | bip44 | bip49 | bip84 | bip86 | bip48_1 | bip48_2.
        let has_derivation_marker = [
            "xpub", "bip44", "bip49", "bip84", "bip86", "bip48_1", "bip48_2",
        ]
        .iter()
        .any(|k| obj.contains_key(*k));
        if !has_derivation_marker {
            return false;
        }
        true
    }

    /// SPEC §11.3 — parse a Coldcard single-sig wallet JSON blob. P3A skeleton
    /// returns `Err(BadInput("P3B: parse not yet wired"))`; the real body
    /// lands in Phase P3B.
    fn parse(_blob: &[u8], _stderr: &mut dyn Write) -> Result<Vec<ParsedImport>, ToolkitError> {
        Err(ToolkitError::BadInput(
            "P3B: coldcard parse not yet wired".to_string(),
        ))
    }
}

/// Strip ASCII leading whitespace before checking for `{` prefix. Mirrors
/// the helper in `wallet_import/specter.rs:503` (`trim_leading_ws`).
fn trim_leading_ws(blob: &[u8]) -> &[u8] {
    let mut i = 0;
    while i < blob.len()
        && (blob[i] == b' ' || blob[i] == b'\t' || blob[i] == b'\n' || blob[i] == b'\r')
    {
        i += 1;
    }
    &blob[i..]
}

// =============================================================================
// P3A — `dead_code` allows on items wired by P3B/P3C.
//
// The `ImportProvenance::Coldcard(...)` variant lands in P3C (it does not
// exist at the enum yet); the `ColdcardSourceMetadata` struct + variant
// helpers are constructed in P3B's `parse()` body. Until P3B/P3C land,
// these items are reachable only from sniff_format + this module's tests.
// The `#[allow(dead_code)]` annotations document the P3A-only interim
// state — they come off as the wiring lands.
// =============================================================================

#[allow(dead_code)]
const _COLDCARD_CHAIN_USED_BY_P3B: fn(ColdcardChain) -> bitcoin::Network = ColdcardChain::to_network;

#[allow(dead_code)]
const _COLDCARD_BIP_USED_BY_P3B: fn(ColdcardBip) -> &'static str = ColdcardBip::as_json_key;

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // Sniff: positive cases (SPEC §11.3 Q3 lock — chain + xfp + ≥1 derivation marker)
    // -------------------------------------------------------------------------

    #[test]
    fn sniff_true_on_modern_bip84_blob() {
        let blob = br#"{
            "chain": "BTC",
            "xfp": "B8688DF1",
            "account": 0,
            "bip84": {
                "name": "p2wpkh",
                "deriv": "m/84'/0'/0'",
                "xfp": "B8688DF1",
                "xpub": "xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX",
                "_pub": "zpubDFXrR8dxAH7gFqHkw9JvNXqVkPiTMfb4P4n2RvBT3PSnD3iJWHsodaR7g2ND2VPiR1iCqXcLqCCdKM7ZN3Hh3hQrFqdjsLkhBwYHbLAQt2T",
                "first": "bc1qjyf0xzn0eyl9d0glujytdq2t5kdq0u4lcj6xtg"
            }
        }"#;
        assert!(ColdcardParser::sniff(blob));
    }

    #[test]
    fn sniff_true_on_legacy_xpub_only_blob() {
        // SPEC §11.3 firmware-variance table: Mk1/Mk2 firmware emitted only
        // a top-level `xpub` (no per-bipN blocks). Sniff still accepts via
        // the at-least-one-of disjunction.
        let blob = br#"{
            "chain": "BTC",
            "xfp": "B8688DF1",
            "xpub": "xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX"
        }"#;
        assert!(ColdcardParser::sniff(blob));
    }

    #[test]
    fn sniff_true_on_multi_bip_blob() {
        // Modern Coldcard firmware emits BOTH bip44, bip49, bip84 in one
        // envelope (and optionally bip86 / bip48_*). Sniff is shape-only —
        // dominant-BIP selection happens at parse time per SPEC §11.3.1.
        let blob = br#"{
            "chain": "BTC",
            "xfp": "B8688DF1",
            "account": 0,
            "bip44": {"name":"p2pkh","deriv":"m/44'/0'/0'","xfp":"B8688DF1","xpub":"xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX","first":"1FRBR4iY3XQhytKwgZmnjJyCSGsXHm9gBL"},
            "bip49": {"name":"p2wpkh-p2sh","deriv":"m/49'/0'/0'","xfp":"B8688DF1","xpub":"xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX","_pub":"ypubDExampleYpub","first":"3FZ..."},
            "bip84": {"name":"p2wpkh","deriv":"m/84'/0'/0'","xfp":"B8688DF1","xpub":"xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX","_pub":"zpubDExampleZpub","first":"bc1q..."}
        }"#;
        assert!(ColdcardParser::sniff(blob));
    }

    #[test]
    fn sniff_true_on_bip86_taproot_blob() {
        // Mk4+ firmware adds bip86 (taproot).
        let blob = br#"{
            "chain": "BTC",
            "xfp": "B8688DF1",
            "account": 0,
            "bip86": {"name":"p2tr","deriv":"m/86'/0'/0'","xfp":"B8688DF1","xpub":"xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX","first":"bc1p..."}
        }"#;
        assert!(ColdcardParser::sniff(blob));
    }

    #[test]
    fn sniff_true_on_testnet_xtn_blob() {
        let blob = br#"{
            "chain": "XTN",
            "xfp": "704C7836",
            "account": 0,
            "bip84": {"name":"p2wpkh","deriv":"m/84'/1'/0'","xfp":"704C7836","xpub":"tpubDEgS9fUEpucKatmvKAv21v8nViHxR6rsV7ohMWK4YjsWd4EWT3w8YzMgMEvNrDfsUANbid74WRFpr3Gym8UHBSLnqg6b1Lzvibw87cLSctC","_pub":"vpubDExampleVpub","first":"tb1q..."}
        }"#;
        assert!(ColdcardParser::sniff(blob));
    }

    #[test]
    fn sniff_true_on_bip48_only_blob() {
        // Q firmware adds bip48_1/bip48_2 multisig hints. They are sniff-positive
        // markers but the single-sig parser ignores them at parse time (multisig
        // case is `--format coldcard-multisig`). Sniff is shape-only — at-least-
        // one-of disjunction means bip48_* alone satisfies sniff.
        let blob = br#"{
            "chain": "BTC",
            "xfp": "B8688DF1",
            "account": 0,
            "bip48_2": {"name":"p2wsh","deriv":"m/48'/0'/0'/2'","xfp":"B8688DF1","xpub":"xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX"}
        }"#;
        assert!(ColdcardParser::sniff(blob));
    }

    #[test]
    fn sniff_tolerates_leading_whitespace() {
        let blob = br#"
        {
            "chain": "BTC",
            "xfp": "B8688DF1",
            "xpub": "xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX"
        }"#;
        assert!(ColdcardParser::sniff(blob));
    }

    // -------------------------------------------------------------------------
    // Sniff: negative cases
    // -------------------------------------------------------------------------

    #[test]
    fn sniff_false_on_missing_chain() {
        let blob = br#"{"xfp":"B8688DF1","xpub":"xpub..."}"#;
        assert!(!ColdcardParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_chain_main_not_btc() {
        // Bitcoin Core uses `"chain": "main"` — Coldcard uses `"BTC"`. The
        // sniff must reject `main` to keep the format-disambiguation strict.
        let blob = br#"{"chain":"main","xfp":"B8688DF1","xpub":"xpub..."}"#;
        assert!(!ColdcardParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_chain_test_not_xtn() {
        let blob = br#"{"chain":"test","xfp":"704C7836","xpub":"tpub..."}"#;
        assert!(!ColdcardParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_missing_xfp() {
        let blob = br#"{"chain":"BTC","xpub":"xpub..."}"#;
        assert!(!ColdcardParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_xfp_not_string() {
        // `xfp` is conventionally an uppercase hex string, NOT an integer.
        // Coldcard firmware emits it as a string; the sniff must reject the
        // wrong-type form to keep format disambiguation strict.
        let blob = br#"{"chain":"BTC","xfp":3094905841,"xpub":"xpub..."}"#;
        assert!(!ColdcardParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_chain_only_no_derivation_marker() {
        // chain + xfp alone is not enough — need at least one of the
        // derivation markers per the Q3 disjunction.
        let blob = br#"{"chain":"BTC","xfp":"B8688DF1","account":0}"#;
        assert!(!ColdcardParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_invalid_json() {
        let blob = br#"{"chain":"BTC","xfp":"B8688DF1","xpub":"xpub..."#;
        assert!(!ColdcardParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_top_level_array() {
        let blob = br#"[{"chain":"BTC","xfp":"B8688DF1","xpub":"xpub..."}]"#;
        assert!(!ColdcardParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_empty_blob() {
        assert!(!ColdcardParser::sniff(b""));
    }

    #[test]
    fn sniff_false_on_random_text() {
        assert!(!ColdcardParser::sniff(b"some random text\n"));
    }

    // -------------------------------------------------------------------------
    // Sniff: cross-format negative — must NOT match other vendor blobs
    // -------------------------------------------------------------------------

    #[test]
    fn sniff_false_on_bsms_blob() {
        let blob = b"BSMS 1.0\nwpkh([deadbeef/84'/0'/0']xpub.../0/*)#abcdefgh\n";
        assert!(!ColdcardParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_bitcoin_core_listdescriptors() {
        // Bitcoin Core `listdescriptors` lacks `chain` and `xfp` keys; the
        // sniff must reject.
        let blob = br#"{"wallet_name":"a","descriptors":[{"desc":"wpkh(xpub.../<0;1>/*)#abcdefgh"}]}"#;
        assert!(!ColdcardParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_specter_blob() {
        // Specter carries `label`+`blockheight`+`descriptor`+`devices`; lacks
        // Coldcard's `chain`+`xfp`.
        let blob = br#"{
            "label":"Daily","blockheight":800000,
            "descriptor":"wpkh([5436d724/84'/0'/0']xpub.../<0;1>/*)#abcdefgh",
            "devices":[{"type":"coldcard","label":"primary"}]
        }"#;
        assert!(!ColdcardParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_sparrow_blob() {
        // Sparrow lacks `chain`+`xfp` at top level.
        let blob = br#"{
            "policyType":"SINGLE","scriptType":"P2WPKH",
            "defaultPolicy":{"miniscript":{"script":"wpkh(@0/**)"}},
            "keystores":[{"keyDerivation":{"masterFingerprint":"5436d724","derivation":"m/84'/0'/0'"},
                "extendedPublicKey":"xpub..."}]
        }"#;
        assert!(!ColdcardParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_coldcard_multisig_text() {
        // Coldcard multisig is text-shape (not JSON) — leads with `Name:`.
        let blob = b"Name: ms-2of3\nPolicy: 2 of 3\nDerivation: m/48'/0'/0'/2'\nFormat: P2WSH\n";
        assert!(!ColdcardParser::sniff(blob));
    }

    // -------------------------------------------------------------------------
    // P3A skeleton parse: returns BadInput "not yet wired"
    // -------------------------------------------------------------------------

    #[test]
    fn parse_skeleton_returns_p3b_not_yet_wired_badinput() {
        let blob = br#"{"chain":"BTC","xfp":"B8688DF1","xpub":"xpub..."}"#;
        let mut stderr = Vec::new();
        let err = ColdcardParser::parse(blob, &mut stderr).unwrap_err();
        match err {
            ToolkitError::BadInput(msg) => {
                assert!(msg.contains("P3B"), "msg must cite Phase P3B; got: {msg}");
                assert!(msg.contains("coldcard"), "msg must cite format; got: {msg}");
            }
            other => panic!("expected BadInput, got: {other:?}"),
        }
    }
}
