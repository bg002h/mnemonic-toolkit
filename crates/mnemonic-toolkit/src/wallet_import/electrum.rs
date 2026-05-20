//! Electrum 4.x wallet-file ingest (`--format electrum`).
//!
//! Per `design/SPEC_wallet_import_v0_28_0.md` §11.6. Inverse of
//! `wallet_export/electrum.rs` (the export-side emitter). Parses Electrum's
//! Python-dict-serialized JSON wallet file. Accepts the post-upgrade
//! canonical shape only; pre-Electrum-4.x wallets must be opened in Electrum
//! 4.x first so the loader's `_convert_*` migration chain rewrites legacy
//! values (see §11.6 Electrum-version scoping note).
//!
//! ## Namespace disambiguation (SPEC §1.4)
//!
//! Three `electrum`-named modules coexist in the toolkit:
//!
//! - `crate::electrum` — Electrum native-seed-format codec (HMAC-SHA512
//!   prefix dispatch). Unrelated to wallet-file ingest. UNCHANGED in v0.28.0.
//! - `crate::wallet_export::electrum` — Electrum wallet-file EMIT. UNCHANGED.
//! - `crate::wallet_import::electrum` — THIS MODULE. Wallet-file INGEST
//!   (inverse of wallet_export::electrum). NEW in v0.28.0 P6.
//!
//! ## Sniff signature (P6A)
//!
//! Sniff matches when ALL of:
//! 1. Blob parses as JSON.
//! 2. Top-level value is an object.
//! 3. Object has a `seed_version` integer field in {11..=71} (per Electrum
//!    upstream's `NEW_SEED_VERSION=11` floor + `FINAL_SEED_VERSION=71` ceiling
//!    at `electrum/wallet_db.py`).
//! 4. Object has a `wallet_type` string field in the accepted enumeration
//!    (`"standard"`, `"<k>of<n>"` pattern via regex, `"2fa"`, `"imported"`).
//!
//! Sniff returns `true` for the refused variants (`"2fa"` / `"imported"` /
//! encrypted) so the parser arm is reached and the user-facing refusal
//! template fires; sniff is a routing decision, not an admission decision.
//!
//! ## Parse contract (P6B — body lands in Phase P6B)
//!
//! See `parse()` doc-comment for the per-`wallet_type` dispatch table.

use super::{WalletFormatParser, ParsedImport};
use crate::error::ToolkitError;
use serde_json::Value;
use std::io::Write;
use std::sync::OnceLock;

/// SPEC §11.6 — Electrum wallet-file parser.
pub(crate) struct ElectrumParser;

/// SPEC §11.6 — per-format provenance for Electrum-ingested bundles. Holds
/// non-bundle metadata (seed_version, wallet_type, wallet_name,
/// dropped_fields) preserved for the `--json` envelope's `source_metadata`
/// surface (parallel to v0.26.0's `CoreSourceMetadata`).
///
/// `#[allow(dead_code)]` is the P6A→P6B handoff window: the struct is
/// declared here so Phase P6B's parse body can populate it, and Phase P6C
/// can wire it into `ImportProvenance::Electrum(...)`. Until then no caller
/// constructs it. Tests at the bottom of this module exercise the
/// constructor + field-access surfaces so the contract is pinned.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct ElectrumSourceMetadata {
    /// `seed_version` integer from the top-level Electrum wallet object.
    /// Range {11..=71} (Electrum 4.x current; per FOLLOWUP
    /// `electrum-final-seed-version-drift`, the upper bound drifts upstream).
    /// Widened from `u8` (P0A SPEC draft) to `u32` for consistency with
    /// `wallet_export/electrum.rs::ELECTRUM_SEED_VERSION_PIN: u32` and to
    /// absorb future upstream drift past 255 without a field-type churn.
    pub(crate) seed_version: u32,
    /// Parsed `wallet_type` discriminator. See enum doc.
    pub(crate) wallet_type: ElectrumWalletType,
    /// Optional human-readable wallet name. Electrum's wallet file does NOT
    /// carry a top-level `wallet_name` field (the value is implicit in the
    /// on-disk filename); this slot is reserved for future Electrum-version
    /// changes. P6 leaves it `None`.
    pub(crate) wallet_name: Option<String>,
    /// Electrum wallet-state field names present in the source that were
    /// dropped from the bundle output (e.g., `addr_history`, `addresses`,
    /// `channels`, `transactions` — runtime state not reconstructable from
    /// the key-material alone). Drives a single stderr NOTICE per SPEC §2.4
    /// (analogous to `CoreSourceMetadata::dropped_fields`).
    pub(crate) dropped_fields: Vec<String>,
}

/// SPEC §11.6 — accepted `wallet_type` value-shape discriminator.
///
/// The literal string `"multisig"` is NEVER stored as `wallet_type` by
/// Electrum 4.x (the `_convert_wallet_type` upgrade chain rewrites legacy
/// values to either `"standard"` or to the `<k>of<n>` pattern, validated by
/// `multisig_type()` at `electrum/util.py` via regex `r'(\d+)of(\d+)'`).
/// `ElectrumWalletType::Multisig` carries the parsed `(k, n)` directly to
/// faithfully mirror what's on disk.
///
/// Refused wallet-type variants (`"2fa"` / `"imported"`) do not reach this
/// enum — they error out at parse-time via SPEC §11.6.1 refusal templates
/// before a provenance is constructed.
///
/// `#[allow(dead_code)]` is the P6A→P6B handoff window — see
/// `ElectrumSourceMetadata`'s doc-comment for the same rationale.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ElectrumWalletType {
    /// `wallet_type: "standard"` — singlesig wallet (single `keystore` field
    /// at top level). Covers post-upgrade `"old"`, `"xpub"`, `"bip44"`, and
    /// hardware-wallet legacy values that map to `"standard"` per
    /// `_convert_wallet_type`.
    Standard,
    /// `wallet_type: "<k>of<n>"` — multisig wallet (per-cosigner `x1/`,
    /// `x2/`, ... fields at top level). `k` ≥ 1, `n` ≤ 15 per Electrum's
    /// `multisig_type` regex; not bounds-validated here (caller may sanity-
    /// check separately at parse time).
    Multisig { k: u8, n: u8 },
}

impl WalletFormatParser for ElectrumParser {
    /// SPEC §11.6 sniff signature. Returns `true` when the blob looks like
    /// an Electrum 4.x wallet file (post-upgrade canonical shape). Does NOT
    /// discriminate between the parse-able (`standard` / `<k>of<n>`) and
    /// refused (`2fa` / `imported`) sub-shapes — that's a parse-arm
    /// distinction. Encrypted wallets are also sniff-positive (the
    /// `use_encryption: true` top-level field still lives alongside
    /// `seed_version` / `wallet_type`); the parse arm refuses them.
    fn sniff(blob: &[u8]) -> bool {
        let value: Value = match serde_json::from_slice(blob) {
            Ok(v) => v,
            Err(_) => return false,
        };
        let obj = match value.as_object() {
            Some(o) => o,
            None => return false,
        };

        // seed_version: integer in {11..=71}.
        let sv_ok = obj
            .get("seed_version")
            .and_then(Value::as_u64)
            .map(|n| (11..=71).contains(&n))
            .unwrap_or(false);
        if !sv_ok {
            return false;
        }

        // wallet_type: string in accepted set.
        let wt = match obj.get("wallet_type").and_then(Value::as_str) {
            Some(s) => s,
            None => return false,
        };
        match wt {
            "standard" | "2fa" | "imported" => true,
            other => multisig_type_regex().is_match(other),
        }
    }

    /// SPEC §11.6 parse contract — body lands in Phase P6B. The P6A skeleton
    /// emits a typed `BadInput` carrying the P6B sentinel so an explicit
    /// `--format electrum` invocation surfaces a clear "not yet wired" error
    /// rather than a panic.
    fn parse(_blob: &[u8], _stderr: &mut dyn Write) -> Result<Vec<ParsedImport>, ToolkitError> {
        Err(ToolkitError::BadInput(
            "wallet_import::electrum::parse: skeleton only; body lands in Phase P6B".into(),
        ))
    }
}

/// SPEC §11.6 — `wallet_type` multisig pattern regex. Mirrors Electrum's
/// `multisig_type()` at `electrum/util.py`: `r'(\d+)of(\d+)'`. Anchored
/// at both ends (the upstream regex is unanchored via `re.match` which
/// anchors leading-only; we anchor trailing too because Electrum's writer
/// only ever stores the bare pattern with no suffix).
fn multisig_type_regex() -> &'static regex::Regex {
    static R: OnceLock<regex::Regex> = OnceLock::new();
    R.get_or_init(|| regex::Regex::new(r"^(\d+)of(\d+)$").expect("static regex compiles"))
}

/// Parse a `<k>of<n>` `wallet_type` value into `(k, n)`. Returns `None` for
/// non-matching inputs (caller should check via `sniff` or fall through to
/// the SPEC §11.6.1 refusal-template arms first).
#[allow(dead_code)] // Consumed by Phase P6B parse-arm.
pub(crate) fn parse_multisig_wallet_type(s: &str) -> Option<(u8, u8)> {
    let caps = multisig_type_regex().captures(s)?;
    let k = caps.get(1)?.as_str().parse::<u8>().ok()?;
    let n = caps.get(2)?.as_str().parse::<u8>().ok()?;
    Some((k, n))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal `standard` wallet shape — mirrors the toolkit's own
    /// `wallet_export/electrum.rs` emit + matches Electrum 4.x on-disk.
    const STANDARD_BIP84: &str = r#"{
  "seed_version": 17,
  "wallet_type": "standard",
  "use_encryption": false,
  "keystore": {
    "type": "bip32",
    "xpub": "zpub6qTBTNftBzVTjgVcSUw7vW5N1KQbV93Jnrw314RHGkCkSx4vk6nEWH1MJfReXi2WThvuDRiRpyT7cDoakEcZMQ1iZPgfJgQrcVMR4aJWh6S",
    "derivation": "m/84'/0'/0'",
    "root_fingerprint": "5436d724",
    "label": ""
  }
}
"#;

    const MULTISIG_2OF3: &str = r#"{
  "seed_version": 17,
  "use_encryption": false,
  "wallet_type": "2of3",
  "x1/": {"type":"bip32","xpub":"Zpub75ybJh4YZjnMskAAUkpy6uLizWcTTRC91yDtz9RcRwtavi4wHpBPZDEYUu9LoAPb6NQZNqKd6eKqF4FhqgWSaWQdqSt4FmdQkQH9uMmHhSh","derivation":"m/48'/0'/0'/2'","root_fingerprint":"b8688df1","label":""},
  "x2/": {"type":"bip32","xpub":"Zpub74LquwpiAdpsXwRDJp46dQ9BhcoEhk3vPktqwMqGrQYmjRhYQi5mbemCRiHUXVh1Ypu5XRYzbbznqxodCwK5NPeVXAPVAuLGKrr1LUMFmPh","derivation":"m/48'/0'/0'/2'","root_fingerprint":"28645006","label":""},
  "x3/": {"type":"bip32","xpub":"Zpub72UafiS3U4xBBsiYjpCRcsEqm8i4Uo2Y2e5DmoNQALzLEXfyaJ7RvrGNGKznahzYT9T2BdMXiGPZ55NiuVukpcueupHwtfXeRKF3wyH3XDv","derivation":"m/48'/0'/0'/2'","root_fingerprint":"5436d724","label":""}
}
"#;

    // ====== sniff: positive cases ======

    #[test]
    fn sniff_standard_singlesig_matches() {
        assert!(ElectrumParser::sniff(STANDARD_BIP84.as_bytes()));
    }

    #[test]
    fn sniff_multisig_2of3_matches() {
        assert!(ElectrumParser::sniff(MULTISIG_2OF3.as_bytes()));
    }

    #[test]
    fn sniff_2fa_matches_for_routing_to_refusal_arm() {
        // SPEC §11.6: refused variants are sniff-positive so the parser arm
        // is reached and the §11.6.1 refusal template fires (vs falling
        // through to NoMatch with a generic "could not detect format"
        // error).
        let blob = br#"{"seed_version":17,"wallet_type":"2fa","use_encryption":false,"x1/":{}}"#;
        assert!(ElectrumParser::sniff(blob));
    }

    #[test]
    fn sniff_imported_matches_for_routing_to_refusal_arm() {
        let blob = br#"{"seed_version":17,"wallet_type":"imported","use_encryption":false,"addresses":{}}"#;
        assert!(ElectrumParser::sniff(blob));
    }

    #[test]
    fn sniff_encrypted_matches_for_routing_to_refusal_arm() {
        // use_encryption: true does NOT change sniff outcome; sniff
        // recognizes the wallet, parse arm refuses it.
        let blob = br#"{"seed_version":17,"wallet_type":"standard","use_encryption":true,"keystore":"base64-blob..."}"#;
        assert!(ElectrumParser::sniff(blob));
    }

    #[test]
    fn sniff_3of5_multisig_matches() {
        let blob = br#"{"seed_version":17,"wallet_type":"3of5","use_encryption":false}"#;
        assert!(ElectrumParser::sniff(blob));
    }

    #[test]
    fn sniff_seed_version_at_floor_11_matches() {
        let blob = br#"{"seed_version":11,"wallet_type":"standard","use_encryption":false,"keystore":{}}"#;
        assert!(ElectrumParser::sniff(blob));
    }

    #[test]
    fn sniff_seed_version_at_ceiling_71_matches() {
        let blob = br#"{"seed_version":71,"wallet_type":"standard","use_encryption":false,"keystore":{}}"#;
        assert!(ElectrumParser::sniff(blob));
    }

    // ====== sniff: negative cases ======

    #[test]
    fn sniff_no_match_seed_version_below_floor_10() {
        let blob = br#"{"seed_version":10,"wallet_type":"standard"}"#;
        assert!(!ElectrumParser::sniff(blob));
    }

    #[test]
    fn sniff_no_match_seed_version_above_ceiling_72() {
        let blob = br#"{"seed_version":72,"wallet_type":"standard"}"#;
        assert!(!ElectrumParser::sniff(blob));
    }

    #[test]
    fn sniff_no_match_wallet_type_literal_multisig() {
        // SPEC §11.6: the literal string "multisig" is NEVER stored by
        // Electrum (only "<k>of<n>" patterns). A blob carrying it is NOT
        // an Electrum wallet.
        let blob = br#"{"seed_version":17,"wallet_type":"multisig"}"#;
        assert!(!ElectrumParser::sniff(blob));
    }

    #[test]
    fn sniff_no_match_wallet_type_unknown_string() {
        let blob = br#"{"seed_version":17,"wallet_type":"hd_wallet"}"#;
        assert!(!ElectrumParser::sniff(blob));
    }

    #[test]
    fn sniff_no_match_missing_seed_version() {
        let blob = br#"{"wallet_type":"standard","keystore":{}}"#;
        assert!(!ElectrumParser::sniff(blob));
    }

    #[test]
    fn sniff_no_match_missing_wallet_type() {
        let blob = br#"{"seed_version":17,"keystore":{}}"#;
        assert!(!ElectrumParser::sniff(blob));
    }

    #[test]
    fn sniff_no_match_seed_version_not_integer() {
        let blob = br#"{"seed_version":"17","wallet_type":"standard"}"#;
        assert!(!ElectrumParser::sniff(blob));
    }

    #[test]
    fn sniff_no_match_wallet_type_not_string() {
        let blob = br#"{"seed_version":17,"wallet_type":17}"#;
        assert!(!ElectrumParser::sniff(blob));
    }

    #[test]
    fn sniff_no_match_invalid_json() {
        assert!(!ElectrumParser::sniff(b"not json"));
    }

    #[test]
    fn sniff_no_match_empty() {
        assert!(!ElectrumParser::sniff(b""));
    }

    #[test]
    fn sniff_no_match_array_at_top_level() {
        assert!(!ElectrumParser::sniff(br#"[{"seed_version":17}]"#));
    }

    #[test]
    fn sniff_no_match_multisig_pattern_zero_k() {
        // Per Electrum semantics k >= 1; "0of3" is malformed. Our regex
        // matches any digits, so this is sniff-positive at the regex layer
        // — the bounds check is deferred to parse-time. Document that.
        // (If the SPEC tightens to k >= 1 at sniff-time, this test flips.)
        let blob = br#"{"seed_version":17,"wallet_type":"0of3"}"#;
        // Currently sniff-positive (regex matches). Parse will reject.
        assert!(ElectrumParser::sniff(blob));
    }

    #[test]
    fn sniff_no_match_multisig_pattern_with_suffix() {
        // Anchored trailing: `"2of3junk"` does NOT match.
        let blob = br#"{"seed_version":17,"wallet_type":"2of3junk"}"#;
        assert!(!ElectrumParser::sniff(blob));
    }

    // ====== parse: skeleton stub ======

    #[test]
    fn parse_skeleton_returns_p6b_sentinel_error() {
        let mut stderr: Vec<u8> = Vec::new();
        let err = ElectrumParser::parse(STANDARD_BIP84.as_bytes(), &mut stderr)
            .expect_err("P6A skeleton must Err with P6B sentinel");
        let msg = err.to_string();
        assert!(
            msg.contains("P6B"),
            "P6A skeleton error must reference Phase P6B; got: {msg}"
        );
    }

    // ====== parse_multisig_wallet_type helper ======

    #[test]
    fn parse_multisig_wallet_type_2of3() {
        assert_eq!(parse_multisig_wallet_type("2of3"), Some((2, 3)));
    }

    #[test]
    fn parse_multisig_wallet_type_15of15() {
        assert_eq!(parse_multisig_wallet_type("15of15"), Some((15, 15)));
    }

    #[test]
    fn parse_multisig_wallet_type_rejects_non_pattern() {
        assert_eq!(parse_multisig_wallet_type("standard"), None);
        assert_eq!(parse_multisig_wallet_type(""), None);
        assert_eq!(parse_multisig_wallet_type("multisig"), None);
    }

    #[test]
    fn parse_multisig_wallet_type_rejects_overflow() {
        // u8 max is 255; "256of256" parses as numbers but overflows u8.
        assert_eq!(parse_multisig_wallet_type("256of256"), None);
    }

    // ====== ElectrumSourceMetadata construction sanity ======

    #[test]
    fn metadata_standard_construction() {
        let m = ElectrumSourceMetadata {
            seed_version: 17,
            wallet_type: ElectrumWalletType::Standard,
            wallet_name: None,
            dropped_fields: Vec::new(),
        };
        assert_eq!(m.seed_version, 17);
        assert!(matches!(m.wallet_type, ElectrumWalletType::Standard));
    }

    #[test]
    fn metadata_multisig_construction() {
        let m = ElectrumSourceMetadata {
            seed_version: 17,
            wallet_type: ElectrumWalletType::Multisig { k: 2, n: 3 },
            wallet_name: Some("Test".to_string()),
            dropped_fields: vec!["addr_history".to_string()],
        };
        match m.wallet_type {
            ElectrumWalletType::Multisig { k, n } => {
                assert_eq!(k, 2);
                assert_eq!(n, 3);
            }
            _ => panic!("expected Multisig variant"),
        }
    }
}
