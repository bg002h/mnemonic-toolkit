//! v0.28.0 Phase P6 — Electrum 4.x wallet-file ingest parser.
//!
//! Per `design/SPEC_wallet_import_v0_28_0.md` §11.6 (with P6A in-phase
//! correction to the `wallet_type` value-set; see SPEC §11.6 intro).
//!
//! **NAMESPACE TRAP (SPEC §1.4):** this module is the wallet-file INGEST
//! surface. Sibling modules with similar names:
//! - `crate::electrum` (`src/electrum.rs`) — native Electrum seed-phrase
//!   codec (HMAC-SHA512 prefix dispatch + per-wordlist base-N mapping);
//!   UNCHANGED in v0.28.0.
//! - `crate::wallet_export::electrum` (`wallet_export/electrum.rs`) — the
//!   inverse-of-this-module wallet-file EMIT surface; UNCHANGED in v0.28.0.
//! - `crate::wallet_import::electrum` (THIS FILE) — wallet-file INGEST;
//!   NEW in v0.28.0 Phase P6.
//!
//! ## Wire shape (Electrum 4.x JSON wallet-file)
//!
//! Singlesig (`wallet_type: "standard"`):
//! ```json
//! {
//!   "seed_version": 17,
//!   "wallet_type": "standard",
//!   "use_encryption": false,
//!   "keystore": {
//!     "type": "bip32",
//!     "xpub": "zpub6...",
//!     "derivation": "m/84'/0'/0'",
//!     "root_fingerprint": "5436d724",
//!     "label": "Daily"
//!   }
//! }
//! ```
//!
//! Multisig (`wallet_type: "<k>of<n>"` regex `(\d+)of(\d+)`):
//! ```json
//! {
//!   "seed_version": 17,
//!   "wallet_type": "2of4",
//!   "use_encryption": false,
//!   "x1/": { "type": "bip32", "xpub": "Zpub...", "derivation": "m/48'/0'/0'/2'", "root_fingerprint": "...", "label": "..." },
//!   "x2/": { ... },
//!   "x3/": { ... },
//!   "x4/": { ... }
//! }
//! ```
//!
//! Refusals (`2fa` / `imported` / `use_encryption: true`) per SPEC §11.6.1.
//!
//! ## Phase P6A scope
//!
//! Parser skeleton + sniff impl + provenance metadata struct decls + sniff
//! unit tests. `parse()` returns `Err(BadInput("P6B: parse not yet wired"))`
//! — Phase P6B installs the real body; Phase P6C flips the
//! `cmd/import_wallet.rs` dispatch sites.

use super::{ImportProvenance, ParsedImport, WalletFormatParser};
use crate::error::ToolkitError;
use serde_json::Value;
use std::io::Write;

/// SPEC §11.6 — Electrum 4.x wallet-file ingest parser.
pub(crate) struct ElectrumParser;

/// SPEC §11.6 — `wallet_type` discriminator (post-P6A correction).
///
/// Values:
/// - `Standard` — `wallet_type: "standard"` (singlesig).
/// - `Multisig { k, n }` — `wallet_type` matches `(\d+)of(\d+)` per
///   `electrum/util.py::multisig_type`. Mirrors the toolkit's own emit
///   at `wallet_export/electrum.rs:141` (`format!("{k}of{n}")`).
///
/// Refused variants (`2fa`, `imported`) do NOT produce an
/// `ElectrumWalletType` — they error out before provenance construction
/// per SPEC §11.6.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ElectrumWalletType {
    Standard,
    /// `k`-of-`n` multisig per `electrum/util.py::multisig_type` regex
    /// `(\d+)of(\d+)`. P6A in-phase SPEC correction.
    #[allow(dead_code)] // fields read by P6B parse + P6C envelope emitter
    Multisig { k: u8, n: u8 },
}

/// SPEC §11.6 — per-blob provenance metadata for an Electrum parse.
/// Carried on `ImportProvenance::Electrum(...)`; preserved for `--json`
/// envelope `electrum_source_metadata` emit (P6C wiring).
#[derive(Debug, Clone)]
#[allow(dead_code)] // fields constructed by P6B, consumed by P6C envelope emitter
pub(crate) struct ElectrumSourceMetadata {
    /// Top-level `seed_version` (Electrum's wallet-db version pin; integer
    /// in {11..71} at v0.28.0 cutover, FINAL_SEED_VERSION drifts upward per
    /// upstream releases — see `wallet_export/electrum.rs::ELECTRUM_SEED_VERSION_PIN`
    /// FOLLOWUP `electrum-final-seed-version-drift`).
    pub(crate) seed_version: u64,
    /// Decoded `wallet_type` (singlesig vs k-of-n multisig).
    pub(crate) wallet_type: ElectrumWalletType,
    /// Top-level wallet label (best-effort: derived from `keystore.label`
    /// for singlesig, or `x1/.label` for multisig). `None` if absent or
    /// the (singlesig) label is empty.
    pub(crate) wallet_name: Option<String>,
    /// Top-level fields encountered in the blob but not preserved on the
    /// import-side provenance (mirrors `CoreSourceMetadata.dropped_fields`).
    pub(crate) dropped_fields: Vec<String>,
}

/// Top-level keys preserved on the Electrum envelope by the toolkit's parse.
/// Any other top-level field surfaces in `ElectrumSourceMetadata.dropped_fields`
/// and drives a stderr NOTICE per SPEC §2.4. Mirrors
/// `COLDCARD_PRESERVED_TOP_LEVEL_KEYS`.
///
/// Note: multisig per-cosigner keys `x1/`, `x2/`, ..., `xN/` are dynamic
/// (N = cosigner count) and tested separately via prefix match in
/// `dropped_fields` computation.
#[allow(dead_code)] // consumed by P6B / canonicalize_electrum
pub(crate) const ELECTRUM_PRESERVED_TOP_LEVEL_KEYS: &[&str] = &[
    "seed_version",
    "wallet_type",
    "use_encryption",
    "keystore",
];

/// SPEC §11.6 — sniff seed_version range. Electrum's `_convert_version_*`
/// chain accepts `seed_version >= 12` (with rejections at 14 / 51 per
/// `wallet_db.py`), so the sniff accepts a generous {11..71+} band to
/// absorb future FINAL_SEED_VERSION drift without re-pinning the sniff.
/// The lower bound is intentionally inclusive at 11 to allow one notch of
/// pre-12 tolerance (NoMatch via `seed_version: 10` is a footgun for
/// hand-edited blobs). The upper bound 71 matches current FINAL_SEED_VERSION
/// but is treated as a soft ceiling — values >71 ARE still accepted at sniff
/// time (the parse-time post-validation re-checks the range and emits a
/// stderr NOTICE if seed_version >71 — handled at P6B).
const SNIFF_SEED_VERSION_MIN: u64 = 11;

/// SPEC §11.6 — sniff `wallet_type` value: matches `(\d+)of(\d+)` regex
/// per `electrum/util.py::multisig_type`. Returns `Some((k, n))` on
/// match, `None` otherwise. Used by sniff + parse contracts.
///
/// Implementation is a hand-rolled state-machine equivalent to the
/// Python regex `(\d+)of(\d+)` anchored at the START (Python's `re.match`
/// is start-anchored by default). The end is NOT anchored — Electrum's
/// `re.match` returns on partial-prefix match, so trailing garbage is
/// tolerated (though canonical Electrum wallet files never carry it).
pub(crate) fn parse_multisig_wallet_type(s: &str) -> Option<(u8, u8)> {
    let bytes = s.as_bytes();
    let mut i = 0usize;
    // First digit run.
    let k_start = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i == k_start {
        return None;
    }
    let k_str = &s[k_start..i];
    // Literal "of".
    if i + 2 > bytes.len() || &bytes[i..i + 2] != b"of" {
        return None;
    }
    i += 2;
    // Second digit run.
    let n_start = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i == n_start {
        return None;
    }
    let n_str = &s[n_start..i];
    let k = k_str.parse::<u8>().ok()?;
    let n = n_str.parse::<u8>().ok()?;
    Some((k, n))
}

/// SPEC §11.6 — classify a top-level `wallet_type` string into the
/// post-correction value-set. Returns `None` for unrecognized values
/// (including `2fa` / `imported` — those are recognized at parse time
/// for the refusal templates per §11.6.1, but at SNIFF time we accept
/// any non-empty string as "this looks Electrum-shaped" — see
/// `ElectrumParser::sniff` for the sniff predicate).
#[allow(dead_code)] // consumed by P6B parse dispatch
pub(crate) fn classify_wallet_type(s: &str) -> Option<ElectrumWalletType> {
    if s == "standard" {
        return Some(ElectrumWalletType::Standard);
    }
    if let Some((k, n)) = parse_multisig_wallet_type(s) {
        return Some(ElectrumWalletType::Multisig { k, n });
    }
    None
}

impl WalletFormatParser for ElectrumParser {
    /// SPEC §11.6 sniff (P6A correction): top-level JSON object containing
    /// ALL of:
    /// (1) `seed_version` integer in `{SNIFF_SEED_VERSION_MIN..}` (inclusive
    ///     lower bound; upper bound is unbounded at sniff time to absorb
    ///     future Electrum FINAL_SEED_VERSION drift — parse time re-checks
    ///     the ceiling per SPEC §11.6).
    /// (2) `wallet_type` string in the v0.28.0 value-set
    ///     `{"standard", "<k>of<n>", "2fa", "imported"}`. The
    ///     `"<k>of<n>"` regex is recognized per `electrum/util.py::multisig_type`
    ///     `(\d+)of(\d+)`.
    ///
    /// Refusal types (`2fa`, `imported`) ARE matched at sniff time (so
    /// the parse-time refusal stderr template can fire with a clear message)
    /// — they would otherwise vector through `NoMatch` and produce the
    /// generic "could not detect format" template.
    ///
    /// Note: encrypted wallets (`use_encryption: true`) sniff POSITIVE here
    /// (top-level structure is still Electrum-shaped); the refusal lands at
    /// parse time per SPEC §11.6.1.
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
        // (1) seed_version: integer ≥ SNIFF_SEED_VERSION_MIN.
        let sv_ok = obj
            .get("seed_version")
            .and_then(|v| v.as_u64())
            .map(|n| n >= SNIFF_SEED_VERSION_MIN)
            .unwrap_or(false);
        if !sv_ok {
            return false;
        }
        // (2) wallet_type: string in value-set.
        let wt_ok = obj
            .get("wallet_type")
            .and_then(|v| v.as_str())
            .map(|s| {
                s == "standard"
                    || s == "2fa"
                    || s == "imported"
                    || parse_multisig_wallet_type(s).is_some()
            })
            .unwrap_or(false);
        if !wt_ok {
            return false;
        }
        true
    }

    /// SPEC §11.6 parse — P6A skeleton: returns
    /// `Err(BadInput("P6B: parse not yet wired"))`. Phase P6B installs the
    /// real body.
    fn parse(_blob: &[u8], _stderr: &mut dyn Write) -> Result<Vec<ParsedImport>, ToolkitError> {
        Err(ToolkitError::BadInput(
            "wallet_import::electrum::parse: P6B: parse not yet wired".to_string(),
        ))
    }
}

/// Trim ASCII-whitespace bytes (space, tab, CR, LF) from the start of a
/// blob. Mirrors `wallet_import/coldcard.rs:trim_leading_ws`.
fn trim_leading_ws(b: &[u8]) -> &[u8] {
    let mut start = 0usize;
    while start < b.len() && matches!(b[start], b' ' | b'\t' | b'\r' | b'\n') {
        start += 1;
    }
    &b[start..]
}

// Placeholder construction site for the `ImportProvenance::Electrum`
// variant — feeds the `#[allow(dead_code)]` budget at P6A. Once P6B lands
// the real parser, this helper is deleted and the variant is constructed
// by `parse()`. P6C wires the alphabetically-positioned envelope arm.
#[allow(dead_code)]
fn _p6a_provenance_construct_placeholder() -> ImportProvenance {
    ImportProvenance::Electrum(ElectrumSourceMetadata {
        seed_version: 17,
        wallet_type: ElectrumWalletType::Standard,
        wallet_name: None,
        dropped_fields: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // sniff tests (P6A scope)
    // ========================================================================

    #[test]
    fn sniff_standard_singlesig_positive() {
        let blob = br#"{
            "seed_version": 17,
            "wallet_type": "standard",
            "use_encryption": false,
            "keystore": {"type": "bip32", "xpub": "zpub6...", "derivation": "m/84'/0'/0'", "root_fingerprint": "5436d724", "label": "Daily"}
        }"#;
        assert!(ElectrumParser::sniff(blob));
    }

    #[test]
    fn sniff_multisig_2of4_positive() {
        let blob = br#"{"seed_version": 17, "wallet_type": "2of4", "use_encryption": false, "x1/": {}}"#;
        assert!(ElectrumParser::sniff(blob));
    }

    #[test]
    fn sniff_multisig_3of5_positive() {
        let blob = br#"{"seed_version": 17, "wallet_type": "3of5"}"#;
        assert!(ElectrumParser::sniff(blob));
    }

    #[test]
    fn sniff_2fa_positive_for_clear_error_message() {
        // SPEC §11.6: 2fa sniff is POSITIVE so the parse-time refusal can
        // fire with a clear "2fa wallets require TrustedCoin..." message
        // (otherwise sniff would NoMatch and surface the generic
        // "could not detect format" template).
        let blob = br#"{"seed_version": 17, "wallet_type": "2fa"}"#;
        assert!(ElectrumParser::sniff(blob));
    }

    #[test]
    fn sniff_imported_positive_for_clear_error_message() {
        let blob = br#"{"seed_version": 17, "wallet_type": "imported"}"#;
        assert!(ElectrumParser::sniff(blob));
    }

    #[test]
    fn sniff_seed_version_below_min_rejected() {
        let blob = br#"{"seed_version": 10, "wallet_type": "standard"}"#;
        assert!(!ElectrumParser::sniff(blob));
    }

    #[test]
    fn sniff_seed_version_above_current_final_still_accepted_at_sniff() {
        // FINAL_SEED_VERSION drifts upward over Electrum releases. Sniff
        // does NOT cap on the upper end — parse-time validation re-checks
        // the ceiling and emits a NOTICE if >71. This lets the sniff stay
        // stable across upstream upgrades.
        let blob = br#"{"seed_version": 99, "wallet_type": "standard"}"#;
        assert!(ElectrumParser::sniff(blob));
    }

    #[test]
    fn sniff_wallet_type_unknown_rejected() {
        let blob = br#"{"seed_version": 17, "wallet_type": "trezor"}"#;
        assert!(!ElectrumParser::sniff(blob));
    }

    #[test]
    fn sniff_wallet_type_missing_rejected() {
        let blob = br#"{"seed_version": 17}"#;
        assert!(!ElectrumParser::sniff(blob));
    }

    #[test]
    fn sniff_seed_version_missing_rejected() {
        let blob = br#"{"wallet_type": "standard"}"#;
        assert!(!ElectrumParser::sniff(blob));
    }

    #[test]
    fn sniff_seed_version_string_rejected() {
        let blob = br#"{"seed_version": "17", "wallet_type": "standard"}"#;
        assert!(!ElectrumParser::sniff(blob));
    }

    #[test]
    fn sniff_top_level_array_rejected() {
        let blob = br#"[{"seed_version": 17, "wallet_type": "standard"}]"#;
        assert!(!ElectrumParser::sniff(blob));
    }

    #[test]
    fn sniff_invalid_json_rejected() {
        let blob = br#"{not json"#;
        assert!(!ElectrumParser::sniff(blob));
    }

    #[test]
    fn sniff_empty_blob_rejected() {
        assert!(!ElectrumParser::sniff(b""));
    }

    #[test]
    fn sniff_leading_whitespace_tolerated() {
        let blob = b"   \n\t{\"seed_version\": 17, \"wallet_type\": \"standard\"}";
        assert!(ElectrumParser::sniff(blob));
    }

    #[test]
    fn sniff_bitcoin_core_descriptors_blob_rejected() {
        // Cross-format guard: a Bitcoin Core listdescriptors blob does NOT
        // carry seed_version / wallet_type; sniff must reject.
        let blob =
            br#"{"wallet_name":"a","descriptors":[{"desc":"wpkh(xpub...)#abcdefgh"}]}"#;
        assert!(!ElectrumParser::sniff(blob));
    }

    #[test]
    fn sniff_coldcard_blob_rejected() {
        // Cross-format guard: a Coldcard generic-wallet-export blob lacks
        // seed_version / wallet_type.
        let blob = br#"{"chain":"BTC","xfp":"B8688DF1","bip84":{"xpub":"zpub..."}}"#;
        assert!(!ElectrumParser::sniff(blob));
    }

    // ========================================================================
    // parse_multisig_wallet_type unit tests
    // ========================================================================

    #[test]
    fn parse_multisig_2of3() {
        assert_eq!(parse_multisig_wallet_type("2of3"), Some((2, 3)));
    }

    #[test]
    fn parse_multisig_15of15_max() {
        assert_eq!(parse_multisig_wallet_type("15of15"), Some((15, 15)));
    }

    #[test]
    fn parse_multisig_3of5() {
        assert_eq!(parse_multisig_wallet_type("3of5"), Some((3, 5)));
    }

    #[test]
    fn parse_multisig_overflow_u8_rejected() {
        // 256of256 overflows u8.
        assert_eq!(parse_multisig_wallet_type("256of256"), None);
    }

    #[test]
    fn parse_multisig_standard_string_rejected() {
        assert_eq!(parse_multisig_wallet_type("standard"), None);
    }

    #[test]
    fn parse_multisig_missing_of_rejected() {
        assert_eq!(parse_multisig_wallet_type("23"), None);
    }

    #[test]
    fn parse_multisig_only_first_digit_run_rejected() {
        assert_eq!(parse_multisig_wallet_type("2of"), None);
    }

    #[test]
    fn parse_multisig_2fa_rejected() {
        assert_eq!(parse_multisig_wallet_type("2fa"), None);
    }

    #[test]
    fn parse_multisig_imported_rejected() {
        assert_eq!(parse_multisig_wallet_type("imported"), None);
    }

    #[test]
    fn parse_multisig_empty_rejected() {
        assert_eq!(parse_multisig_wallet_type(""), None);
    }

    // ========================================================================
    // classify_wallet_type unit tests
    // ========================================================================

    #[test]
    fn classify_standard() {
        assert_eq!(
            classify_wallet_type("standard"),
            Some(ElectrumWalletType::Standard)
        );
    }

    #[test]
    fn classify_multisig_2of3() {
        assert_eq!(
            classify_wallet_type("2of3"),
            Some(ElectrumWalletType::Multisig { k: 2, n: 3 })
        );
    }

    #[test]
    fn classify_2fa_returns_none() {
        // 2fa is a recognized refusal class — classify_wallet_type itself
        // returns None (the refusal-template lookup happens in the parse
        // body, not the classifier). The sniff side accepts 2fa for clear
        // error messaging via a separate predicate.
        assert_eq!(classify_wallet_type("2fa"), None);
    }

    #[test]
    fn classify_imported_returns_none() {
        assert_eq!(classify_wallet_type("imported"), None);
    }

    #[test]
    fn classify_unknown_returns_none() {
        assert_eq!(classify_wallet_type("trezor"), None);
    }

    // ========================================================================
    // parse skeleton smoke
    // ========================================================================

    #[test]
    fn parse_p6a_skeleton_returns_p6b_not_wired_error() {
        let mut sink = Vec::new();
        let err = ElectrumParser::parse(b"{}", &mut sink).unwrap_err();
        let msg = format!("{err}");
        assert!(
            msg.contains("P6B") || msg.contains("not yet wired"),
            "expected P6B not-yet-wired skeleton message; got: {msg}"
        );
    }

    #[test]
    fn provenance_construct_placeholder_smoke() {
        let p = _p6a_provenance_construct_placeholder();
        match p {
            ImportProvenance::Electrum(_) => {}
            other => panic!("expected ImportProvenance::Electrum, got {other:?}"),
        }
    }
}
