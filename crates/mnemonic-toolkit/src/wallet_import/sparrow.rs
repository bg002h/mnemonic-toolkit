//! Sparrow Wallet wallet-import parser.
//!
//! Per `design/SPEC_wallet_import_v0_28_0.md` §11.1 (Phase P1A scaffolding;
//! P1B installs the parse impl). Accepts the JSON shape that Sparrow's
//! `Wallet.toJSON()` emits — the inverse of `wallet_export/sparrow.rs`'s
//! `emit_sparrow_wallet_json`:
//!
//! ```json
//! {
//!   "name": "<wallet-label>",
//!   "network": "mainnet|testnet|signet|regtest",
//!   "policyType": "SINGLE|MULTI",
//!   "scriptType": "P2WPKH|P2TR|P2WSH|P2SH_P2WSH|...",
//!   "defaultPolicy": {
//!     "name": "Default",
//!     "miniscript": { "script": "<miniscript-policy-expr>" }
//!   },
//!   "keystores": [
//!     {
//!       "label": "...",
//!       "source": "SW_WATCH|...",
//!       "walletModel": "SPARROW|...",
//!       "keyDerivation": {
//!         "masterFingerprint": "<lowercase 8-hex>",
//!         "derivation": "m/..."
//!       },
//!       "extendedPublicKey": "<xpub-or-tpub>"
//!     }, ...
//!   ]
//! }
//! ```
//!
//! Sniff is positive-marker-based on `policyType` + `scriptType` +
//! `defaultPolicy.miniscript.script` + `keystores`. Vendor markers are
//! sufficient to disambiguate Sparrow from Bitcoin Core / Specter / other
//! JSON formats — no false-positive co-fire risk with other §11 parsers.
//!
//! Parse impl is deferred to Phase P1B; the current `parse` returns
//! `Err(BadInput("not yet implemented"))` so the trait bound is satisfied
//! while P1A only wires sniff into `wallet_import::sniff::sniff_format`.

use super::{ParsedImport, WalletFormatParser};
use crate::error::ToolkitError;
use serde_json::Value;
use std::io::Write;

pub(crate) struct SparrowParser;

/// SPEC §11.1 — provenance metadata for a parsed Sparrow wallet blob.
///
/// Mirrors the shape of `CoreSourceMetadata` (per-entry envelope fields the
/// toolkit preserves for `--json` envelope emission + round-trip canonicalize)
/// adapted to Sparrow's wire shape. Fields:
///
/// - `label`: top-level `name` field if present (Sparrow's wallet-label).
///   `Option<String>` because Sparrow blobs in the wild may omit it.
/// - `policy_type`: `Single` for `policyType: "SINGLE"`, `Multi` for `"MULTI"`.
///   Enum-typed rather than verbatim string so invalid values are rejected
///   at parse time.
/// - `script_type`: verbatim `scriptType` string (`"P2WPKH"`, `"P2TR"`,
///   `"P2WSH"`, `"P2SH_P2WSH"`, etc.). Verbatim because Sparrow's
///   `ScriptType` enum carries display strings that downstream consumers
///   may want to surface unchanged.
/// - `dropped_fields`: names of envelope fields present in the source blob
///   but not preserved in `ParsedImport`. Drives the per-blob NOTICE per
///   SPEC §2.4 (analogous to `CoreSourceMetadata.dropped_fields`).
#[derive(Debug, Clone)]
#[allow(dead_code)] // P1A: fields populated in P1B.
pub(crate) struct SparrowSourceMetadata {
    pub(crate) label: Option<String>,
    pub(crate) policy_type: SparrowPolicyType,
    pub(crate) script_type: String,
    pub(crate) dropped_fields: Vec<String>,
}

/// SPEC §11.1 — Sparrow's `policyType` discriminant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // P1A: variants constructed in P1B.
pub(crate) enum SparrowPolicyType {
    Single,
    Multi,
}

impl WalletFormatParser for SparrowParser {
    /// SPEC §11.1 — positive-marker sniff. Returns `true` iff the blob:
    ///
    /// 1. Parses as JSON whose top-level value is an object.
    /// 2. Contains `policyType` ∈ {`"SINGLE"`, `"MULTI"`} at top level.
    /// 3. Contains `scriptType` (any string) at top level.
    /// 4. Contains `defaultPolicy.miniscript.script` (nested string).
    /// 5. Contains a non-empty `keystores` array at top level.
    ///
    /// All five must hold; absence of any single marker → `false`. The
    /// `policyType` value-set check (#2) rejects blobs that contain the
    /// `policyType` key but with an unrecognized value (defense-in-depth
    /// against the Bitcoin Core vendor-marker exclusion: if Core's
    /// `policyType` set ever drifts, this sniff stays strict).
    fn sniff(blob: &[u8]) -> bool {
        let value: Value = match serde_json::from_slice(blob) {
            Ok(v) => v,
            Err(_) => return false,
        };
        let obj = match value.as_object() {
            Some(o) => o,
            None => return false,
        };
        // (2) policyType ∈ {SINGLE, MULTI}
        let policy_type_ok = obj
            .get("policyType")
            .and_then(|v| v.as_str())
            .map(|s| s == "SINGLE" || s == "MULTI")
            .unwrap_or(false);
        if !policy_type_ok {
            return false;
        }
        // (3) scriptType present + string-typed.
        if obj.get("scriptType").and_then(|v| v.as_str()).is_none() {
            return false;
        }
        // (4) defaultPolicy.miniscript.script present + string-typed.
        let nested_script_ok = obj
            .get("defaultPolicy")
            .and_then(|v| v.as_object())
            .and_then(|m| m.get("miniscript"))
            .and_then(|v| v.as_object())
            .and_then(|m| m.get("script"))
            .and_then(|v| v.as_str())
            .is_some();
        if !nested_script_ok {
            return false;
        }
        // (5) keystores is a non-empty array.
        let keystores_ok = obj
            .get("keystores")
            .and_then(|v| v.as_array())
            .map(|a| !a.is_empty())
            .unwrap_or(false);
        if !keystores_ok {
            return false;
        }
        true
    }

    /// SPEC §11.1 — parse impl LANDS in Phase P1B. P1A returns BadInput so the
    /// trait bound is satisfied while sniff is wired into `sniff_format`.
    fn parse(_blob: &[u8], _stderr: &mut dyn Write) -> Result<Vec<ParsedImport>, ToolkitError> {
        Err(ToolkitError::BadInput(
            "sparrow parse: not yet implemented; landing in Phase P1B".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// SPEC §11.1 — positive-sniff cell: minimal SINGLE wallet.
    #[test]
    fn sniff_true_on_minimal_single_blob() {
        let blob = br#"{
            "name":"bip84-0",
            "network":"mainnet",
            "policyType":"SINGLE",
            "scriptType":"P2WPKH",
            "defaultPolicy":{"name":"Default","miniscript":{"script":"wpkh(@0/**)"}},
            "keystores":[{
                "label":"bip84-0","source":"SW_WATCH","walletModel":"SPARROW",
                "keyDerivation":{"masterFingerprint":"5436d724","derivation":"m/84'/0'/0'"},
                "extendedPublicKey":"xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9"
            }]
        }"#;
        assert!(SparrowParser::sniff(blob));
    }

    /// SPEC §11.1 — positive-sniff cell: 2-of-3 P2WSH multisig.
    #[test]
    fn sniff_true_on_minimal_multi_blob() {
        let blob = br#"{
            "name":"wsh-sortedmulti-0",
            "network":"mainnet",
            "policyType":"MULTI",
            "scriptType":"P2WSH",
            "defaultPolicy":{"name":"Default","miniscript":{"script":"wsh(sortedmulti(2,@0/**,@1/**,@2/**))"}},
            "keystores":[
                {"label":"wsh","source":"SW_WATCH","walletModel":"SPARROW",
                 "keyDerivation":{"masterFingerprint":"b8688df1","derivation":"m/48'/0'/0'/2'"},
                 "extendedPublicKey":"xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX"},
                {"label":"wsh","source":"SW_WATCH","walletModel":"SPARROW",
                 "keyDerivation":{"masterFingerprint":"28645006","derivation":"m/48'/0'/0'/2'"},
                 "extendedPublicKey":"xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6"},
                {"label":"wsh","source":"SW_WATCH","walletModel":"SPARROW",
                 "keyDerivation":{"masterFingerprint":"5436d724","derivation":"m/48'/0'/0'/2'"},
                 "extendedPublicKey":"xpub6Buxw9MmbkJr4iAw8SACNci2hQNuPCMwt9P7HkK62ZQAW9UcJaQ2bc6ARD892TToQQ9Rp6AHujHxBLXqAsvn5fRnLfnhKSRfz8qtaoyKUYx"}
            ]
        }"#;
        assert!(SparrowParser::sniff(blob));
    }

    /// SPEC §11.1 — positive-sniff cell: taproot singlesig (P2TR scriptType).
    #[test]
    fn sniff_true_on_p2tr_blob() {
        let blob = br#"{
            "name":"bip86-0","network":"mainnet","policyType":"SINGLE","scriptType":"P2TR",
            "defaultPolicy":{"name":"Default","miniscript":{"script":"tr(@0/**)"}},
            "keystores":[{
                "label":"bip86-0","source":"SW_WATCH","walletModel":"SPARROW",
                "keyDerivation":{"masterFingerprint":"5436d724","derivation":"m/86'/0'/0'"},
                "extendedPublicKey":"xpub6CAYwo2AfKJy1cdFGBAgLvCrZULhEkZ9C9s4GGXwXzHvNPguMWBcVrGEDjP2ZJdX92gVWLeLrNVVmipTrKqrwMy2eT282xKEyHMbPDrcD9e"
            }]
        }"#;
        assert!(SparrowParser::sniff(blob));
    }

    /// Negative-sniff cell: a BSMS 4-line blob has no JSON shape.
    #[test]
    fn sniff_false_on_bsms_blob() {
        let blob = b"BSMS 1.0\nwpkh([deadbeef/84'/0'/0']xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9/<0;1>/*)#00000000\n";
        assert!(!SparrowParser::sniff(blob));
    }

    /// Negative-sniff cell: a Bitcoin Core listdescriptors envelope has
    /// `descriptors` array but no `policyType` / `keystores` / nested
    /// miniscript script.
    #[test]
    fn sniff_false_on_bitcoin_core_blob() {
        let blob = br#"{"wallet_name":"x","descriptors":[{"desc":"wpkh(xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX/<0;1>/*)#00000000"}]}"#;
        assert!(!SparrowParser::sniff(blob));
    }

    /// Negative-sniff cell: a Specter blob carries `label` + `blockheight` +
    /// `descriptor` + `devices` but NOT Sparrow's positive markers.
    #[test]
    fn sniff_false_on_specter_blob() {
        let blob = br#"{"label":"my-wallet","blockheight":700000,"descriptor":"wpkh(xpub...)","devices":[{"type":"coldcard","label":"cc"}]}"#;
        assert!(!SparrowParser::sniff(blob));
    }

    /// Negative-sniff cell: empty keystores array fails sniff (positive-marker
    /// #5 requires non-empty).
    #[test]
    fn sniff_false_on_empty_keystores() {
        let blob = br#"{"policyType":"SINGLE","scriptType":"P2WPKH",
                        "defaultPolicy":{"miniscript":{"script":"wpkh(@0/**)"}},
                        "keystores":[]}"#;
        assert!(!SparrowParser::sniff(blob));
    }

    /// Negative-sniff cell: missing `defaultPolicy.miniscript.script` fails
    /// sniff (positive-marker #4).
    #[test]
    fn sniff_false_on_missing_nested_script() {
        let blob = br#"{"policyType":"SINGLE","scriptType":"P2WPKH",
                        "defaultPolicy":{"miniscript":{}},
                        "keystores":[{"x":1}]}"#;
        assert!(!SparrowParser::sniff(blob));
    }

    /// Negative-sniff cell: unrecognized `policyType` value fails sniff
    /// (positive-marker #2 value-set check).
    #[test]
    fn sniff_false_on_unrecognized_policy_type_value() {
        let blob = br#"{"policyType":"NOVEL","scriptType":"P2WPKH",
                        "defaultPolicy":{"miniscript":{"script":"wpkh(@0/**)"}},
                        "keystores":[{"x":1}]}"#;
        assert!(!SparrowParser::sniff(blob));
    }

    /// Negative-sniff cell: bare-array top-level fails sniff (positive-marker
    /// #1 requires object).
    #[test]
    fn sniff_false_on_bare_array() {
        let blob = br#"[{"policyType":"SINGLE"}]"#;
        assert!(!SparrowParser::sniff(blob));
    }

    /// Negative-sniff cell: random non-JSON text fails sniff.
    #[test]
    fn sniff_false_on_random_text() {
        assert!(!SparrowParser::sniff(b"not a wallet blob\n"));
    }

    /// Negative-sniff cell: empty blob fails sniff.
    #[test]
    fn sniff_false_on_empty_blob() {
        assert!(!SparrowParser::sniff(b""));
    }

    /// P1A: parse impl returns BadInput placeholder until P1B lands.
    #[test]
    fn parse_returns_not_yet_implemented_in_p1a() {
        let blob = br#"{"policyType":"SINGLE"}"#;
        let mut stderr = Vec::new();
        let err = SparrowParser::parse(blob, &mut stderr).unwrap_err();
        let msg = format!("{err:?}");
        assert!(
            msg.contains("not yet implemented") && msg.contains("P1B"),
            "expected P1B-deferral message; got: {msg}"
        );
    }
}
