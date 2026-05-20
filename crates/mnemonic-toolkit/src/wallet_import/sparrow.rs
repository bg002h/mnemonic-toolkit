//! Sparrow Wallet wallet-import parser.
//!
//! Per `design/SPEC_wallet_import_v0_28_0.md` §11.1. Accepts the JSON shape
//! that Sparrow's `Wallet.toJSON()` emits — the inverse of
//! `wallet_export/sparrow.rs`'s `emit_sparrow_wallet_json`:
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
//! Sniff (SPEC §11.1) is positive-marker on `policyType` + `scriptType` +
//! `defaultPolicy.miniscript.script` + `keystores`. Vendor markers are
//! sufficient to disambiguate Sparrow from Bitcoin Core / Specter / other
//! JSON formats — no false-positive co-fire risk with other §11 parsers.
//!
//! Phase P1A wires `sniff` + the `SparrowSourceMetadata` provenance struct;
//! `parse` is intentionally `unimplemented!("P1B: parse not yet wired")` until
//! Phase P1B lands the body. Wiring discipline mirrors P4A (Coldcard
//! multisig): the sniff slot in `wallet_import/sniff.rs::sniff_format`'s
//! `votes` array is flipped from `false` → `SparrowParser::sniff(blob)` in
//! this phase; the parse-side dispatch at `cmd/import_wallet.rs` stays on
//! `unimplemented!("P1C: parse not yet wired")` until P1C.

use super::{ImportProvenance, ParsedImport, WalletFormatParser};
use crate::error::ToolkitError;
use serde_json::Value;
use std::io::Write;

/// SPEC §11.1 — Sparrow Wallet wallet-import parser.
///
/// Phase P1A ships sniff + struct definitions only; `parse` lands at P1B.
pub(crate) struct SparrowParser;

/// SPEC §11.1 — provenance metadata for a parsed Sparrow wallet blob.
///
/// Mirrors the shape of `CoreSourceMetadata` (per-blob envelope fields the
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
///
/// Fields populated by `SparrowParser::parse` (P1B) and consumed by the
/// `cmd::import_wallet::emit_json_envelope` `sparrow_source_metadata` field
/// (P1C wiring). The struct + variants exist at P1A for downstream-consumer
/// reference + dispatch stitching but are not yet constructed by any wired
/// call site (the `#[allow(dead_code)]` on the `ImportProvenance::Sparrow`
/// variant covers this interim per the P4A ColdcardMultisig precedent at
/// `wallet_import/mod.rs:83-84`).
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct SparrowSourceMetadata {
    pub(crate) label: Option<String>,
    pub(crate) policy_type: SparrowPolicyType,
    pub(crate) script_type: String,
    pub(crate) dropped_fields: Vec<String>,
}

/// SPEC §11.1 — Sparrow's `policyType` discriminant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) enum SparrowPolicyType {
    Single,
    Multi,
}

impl SparrowPolicyType {
    /// Map Sparrow's wire-form `policyType` string into the typed variant.
    /// Returns `None` for unrecognized values (rejected by the parser as
    /// `ImportWalletParse` exit 2). Phase P1B consumes this; P1A exposes
    /// the helper so `cfg(test)` paths can construct values too.
    #[allow(dead_code)]
    pub(crate) fn from_str(s: &str) -> Option<Self> {
        match s {
            "SINGLE" => Some(Self::Single),
            "MULTI" => Some(Self::Multi),
            _ => None,
        }
    }
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

    /// SPEC §11.1 — parse a Sparrow wallet JSON blob. Body lands in Phase P1B.
    ///
    /// At P1A the body is `unimplemented!("P1B: parse not yet wired")` so
    /// that any accidental invocation (e.g. via `cmd/import_wallet.rs`'s
    /// `--format sparrow` arm before P1C wires it) panics with an
    /// unambiguous "P1B" marker rather than failing silently or returning
    /// garbage.
    fn parse(_blob: &[u8], _stderr: &mut dyn Write) -> Result<Vec<ParsedImport>, ToolkitError> {
        unimplemented!("P1B: SparrowParser::parse not yet wired (sniff lands at P1A)")
    }
}

/// Smoke-helper used by P1B parse impl + P1C dispatch wiring. P1A pre-declares
/// the `ImportProvenance::Sparrow(SparrowSourceMetadata)` constructor to keep
/// the variant-add diff bounded to `mod.rs` alphabetical insertion (see
/// `wallet_import/mod.rs::ImportProvenance::Sparrow`). The helper itself is
/// unused at P1A; `#[allow(dead_code)]` keeps the warning surface clean.
#[allow(dead_code)]
pub(crate) fn build_provenance(meta: SparrowSourceMetadata) -> ImportProvenance {
    ImportProvenance::Sparrow(meta)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===========================================================================
    // SNIFF cells (P1A)
    // ===========================================================================

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
    /// Parsing taproot is refused at P1B; sniff is permissive (the format
    /// IS Sparrow's; the refusal happens later at parse time).
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

    /// Negative-sniff cell: a Bitcoin Core listdescriptors blob has no
    /// `policyType` marker.
    #[test]
    fn sniff_false_on_bitcoin_core_blob() {
        let blob = br#"{"wallet_name":"x","descriptors":[{"desc":"wpkh(xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX/<0;1>/*)#00000000"}]}"#;
        assert!(!SparrowParser::sniff(blob));
    }

    /// Negative-sniff cell: a Specter blob has `blockheight` + `devices` but
    /// no Sparrow markers.
    #[test]
    fn sniff_false_on_specter_blob() {
        let blob = br#"{"label":"my-wallet","blockheight":700000,"descriptor":"wpkh(xpub...)","devices":[{"type":"coldcard","label":"cc"}]}"#;
        assert!(!SparrowParser::sniff(blob));
    }

    /// Negative-sniff cell: empty `keystores` array → not Sparrow per #5.
    #[test]
    fn sniff_false_on_empty_keystores() {
        let blob = br#"{"policyType":"SINGLE","scriptType":"P2WPKH",
                        "defaultPolicy":{"miniscript":{"script":"wpkh(@0/**)"}},
                        "keystores":[]}"#;
        assert!(!SparrowParser::sniff(blob));
    }

    /// Negative-sniff cell: missing nested `defaultPolicy.miniscript.script`
    /// → not Sparrow per #4.
    #[test]
    fn sniff_false_on_missing_nested_script() {
        let blob = br#"{"policyType":"SINGLE","scriptType":"P2WPKH",
                        "defaultPolicy":{"miniscript":{}},
                        "keystores":[{"x":1}]}"#;
        assert!(!SparrowParser::sniff(blob));
    }

    /// Negative-sniff cell: `policyType` present but value outside
    /// {SINGLE, MULTI} → not Sparrow per #2.
    #[test]
    fn sniff_false_on_unrecognized_policy_type_value() {
        let blob = br#"{"policyType":"NOVEL","scriptType":"P2WPKH",
                        "defaultPolicy":{"miniscript":{"script":"wpkh(@0/**)"}},
                        "keystores":[{"x":1}]}"#;
        assert!(!SparrowParser::sniff(blob));
    }

    /// Negative-sniff cell: top-level is a bare JSON array (not an object).
    #[test]
    fn sniff_false_on_bare_array() {
        let blob = br#"[{"policyType":"SINGLE"}]"#;
        assert!(!SparrowParser::sniff(blob));
    }

    /// Negative-sniff cell: completely non-JSON random text.
    #[test]
    fn sniff_false_on_random_text() {
        assert!(!SparrowParser::sniff(b"not a wallet blob\n"));
    }

    /// Negative-sniff cell: empty blob.
    #[test]
    fn sniff_false_on_empty_blob() {
        assert!(!SparrowParser::sniff(b""));
    }

    // ===========================================================================
    // P1A scaffolding: parse is `unimplemented!()` until P1B
    // ===========================================================================

    /// Regression guard: `SparrowParser::parse` panics with a P1B marker at
    /// Phase P1A. P1B flips the body to a real impl; P1B's reviewer-loop
    /// REPLACES this cell.
    #[test]
    #[should_panic(expected = "P1B")]
    fn parse_panics_until_p1b() {
        let mut stderr: Vec<u8> = Vec::new();
        let _ = SparrowParser::parse(b"{}", &mut stderr);
    }

    /// Provenance constructor accepts a `SparrowSourceMetadata` and yields
    /// the `ImportProvenance::Sparrow(_)` variant. Pinned at P1A so P1B's
    /// downstream parse-result wiring is bounded.
    #[test]
    fn build_provenance_yields_sparrow_variant() {
        let meta = SparrowSourceMetadata {
            label: Some("test".to_string()),
            policy_type: SparrowPolicyType::Single,
            script_type: "P2WPKH".to_string(),
            dropped_fields: Vec::new(),
        };
        let prov = build_provenance(meta);
        assert!(matches!(prov, ImportProvenance::Sparrow(_)));
    }

    /// SparrowPolicyType::from_str covers both wire-form values + rejects
    /// unrecognized strings.
    #[test]
    fn sparrow_policy_type_from_str_matrix() {
        assert_eq!(SparrowPolicyType::from_str("SINGLE"), Some(SparrowPolicyType::Single));
        assert_eq!(SparrowPolicyType::from_str("MULTI"), Some(SparrowPolicyType::Multi));
        assert_eq!(SparrowPolicyType::from_str("single"), None);
        assert_eq!(SparrowPolicyType::from_str(""), None);
        assert_eq!(SparrowPolicyType::from_str("NOVEL"), None);
    }
}
