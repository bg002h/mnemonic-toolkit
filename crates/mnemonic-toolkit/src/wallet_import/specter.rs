//! v0.28.0 Phase P2 — Specter-DIY wallet-JSON parser.
//!
//! Per `design/SPEC_wallet_import_v0_28_0.md` §11.2.
//!
//! Specter Desktop's wallet-export JSON shape (canonical authority:
//! <https://github.com/cryptoadvance/specter-desktop/blob/master/src/cryptoadvance/specter/util/wallet_importer.py>)
//! is a single JSON object with four load-bearing top-level fields:
//!
//! ```json
//! {
//!   "label": "<wallet display name>",
//!   "blockheight": <integer rescan-start block>,
//!   "descriptor": "<BIP-380 descriptor with #checksum suffix>",
//!   "devices": [
//!     {"type": "<vendor>", "label": "<device display name>"},
//!     ...
//!   ]
//! }
//! ```
//!
//! Distinctive sniff marker: top-level `blockheight` integer. No other
//! supported format carries this field at the JSON top level
//! (`VENDOR_MARKER_KEYS` in `bitcoin_core.rs:81` excludes any blob carrying
//! `blockheight` from Bitcoin Core's positive sniff per SPEC §6.1.1).
//!
//! Legacy-shape note: the `wallet_export/specter.rs` emitter in this crate
//! produces `devices: Vec<&'static str>` (string array — `["unknown"]`).
//! Newer Specter exports use the object-form `[{"type":..., "label":...}]`.
//! The sniff predicate is shape-tolerant on the `devices` element type (any
//! JSON array satisfies sniff); the parse impl (P2B) handles both shapes.
//!
//! Phase P2A scope: parser skeleton + sniff impl + provenance metadata
//! struct decls + sniff unit tests. `parse()` returns
//! `Err(BadInput("P2B: parse not yet wired"))` — Phase P2B installs the
//! real body; Phase P2C flips the 8 `cmd/import_wallet.rs` dispatch sites
//! from `unimplemented!()` to `SpecterParser::parse`.

use super::{ParsedImport, WalletFormatParser};
use crate::error::ToolkitError;
use serde_json::Value;
use std::io::Write;

/// SPEC §11.2 — Specter-DIY wallet-import parser.
pub(crate) struct SpecterParser;

/// SPEC §11.2 — per-blob provenance metadata for a Specter-DIY parse.
/// Carried on `ImportProvenance::Specter(...)`; preserved for `--json`
/// envelope `source_metadata` emit (Phase P2B integration).
#[derive(Debug, Clone)]
#[allow(dead_code)] // P2A scaffolding; fields read by P2B parse impl + envelope emit.
pub(crate) struct SpecterSourceMetadata {
    /// Top-level `label` (wallet display name).
    pub(crate) label: String,
    /// Top-level `blockheight` (rescan-start block; 0 if absent).
    pub(crate) blockheight: u64,
    /// Per-cosigner device hints. Length matches the descriptor's cosigner
    /// count for multisig; length 1 for singlesig. Each entry is a
    /// `SpecterDeviceMarker` (object-form) or a normalized
    /// `{type: "<vendor>", label: ""}` projection from legacy string-form.
    pub(crate) devices: Vec<SpecterDeviceMarker>,
    /// Top-level fields encountered in the blob but not preserved on the
    /// import-side provenance (mirrors `CoreSourceMetadata.dropped_fields`).
    pub(crate) dropped_fields: Vec<String>,
}

/// SPEC §11.2 — per-cosigner device hint from a Specter wallet JSON.
///
/// Two shapes are tolerated at parse time:
/// - **Object form** (newer Specter exports): `{"type": "<vendor>", "label": "<name>"}`.
/// - **String form** (older / toolkit-side emit): `"<vendor>"` — normalized to
///   `{type: <vendor>, label: ""}` during parse.
#[derive(Debug, Clone)]
#[allow(dead_code)] // P2A scaffolding; fields read by P2B parse impl + envelope emit.
pub(crate) struct SpecterDeviceMarker {
    /// Hardware-wallet type identifier (e.g., `"coldcard"`, `"trezor"`,
    /// `"ledger"`, `"unknown"`). Specter does not normalize the vendor
    /// vocabulary; the toolkit preserves whatever string is on the blob.
    pub(crate) device_type: String,
    /// User-supplied display label for the device. Empty string when the
    /// blob used the legacy `["<vendor>"]` string-array shape.
    pub(crate) label: String,
}

impl WalletFormatParser for SpecterParser {
    /// SPEC §11.2 sniff: top-level JSON object containing all of
    /// `{label, blockheight, descriptor, devices}` where `blockheight` is an
    /// integer and `devices` is an array. The integer-shape check on
    /// `blockheight` is the distinctive disambiguator — no other supported
    /// vendor format carries an integer field with this name at the JSON
    /// top level.
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
        // SPEC §11.2 positive sniff: REQUIRE all four load-bearing fields.
        // `label`: any string. `blockheight`: integer (number with no
        // fractional component; serde_json::Value::is_u64 || is_i64 ⇒ true).
        // `descriptor`: any string. `devices`: any array.
        let label_ok = obj.get("label").map(Value::is_string).unwrap_or(false);
        let blockheight_ok = obj
            .get("blockheight")
            .map(|v| v.is_u64() || v.is_i64())
            .unwrap_or(false);
        let descriptor_ok = obj.get("descriptor").map(Value::is_string).unwrap_or(false);
        let devices_ok = obj.get("devices").map(Value::is_array).unwrap_or(false);
        label_ok && blockheight_ok && descriptor_ok && devices_ok
    }

    /// Phase P2A skeleton — parse impl lands in P2B per plan-doc §S.2.
    fn parse(_blob: &[u8], _stderr: &mut dyn Write) -> Result<Vec<ParsedImport>, ToolkitError> {
        Err(ToolkitError::BadInput(
            "P2B: specter parse not yet wired (skeleton lives at wallet_import/specter.rs)".into(),
        ))
    }
}

fn trim_leading_ws(blob: &[u8]) -> &[u8] {
    let mut i = 0;
    while i < blob.len()
        && (blob[i] == b' ' || blob[i] == b'\t' || blob[i] == b'\n' || blob[i] == b'\r')
    {
        i += 1;
    }
    &blob[i..]
}

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // Sniff: positive cases (SPEC §11.2 — all four marker fields present + typed)
    // -------------------------------------------------------------------------

    #[test]
    fn sniff_true_on_canonical_specter_singlesig() {
        // Minimal positive: object-form devices array, blockheight integer,
        // label + descriptor strings. Mirrors `wallet_export/specter.rs`
        // emit shape with newer object-form devices.
        let blob = br#"{
  "label": "daily",
  "blockheight": 800000,
  "descriptor": "wpkh([deadbeef/84'/0'/0']xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX/<0;1>/*)#abcdefgh",
  "devices": [{"type": "coldcard", "label": "primary"}]
}"#;
        assert!(SpecterParser::sniff(blob));
    }

    #[test]
    fn sniff_true_on_legacy_string_devices_array() {
        // Older Specter exports (and this crate's `wallet_export/specter.rs`
        // emitter at v0.27.x) use a string-array `devices: ["unknown"]`
        // instead of the object-form. Sniff must accept both shapes
        // (devices: array — element type unconstrained at sniff time).
        let blob = br#"{
  "label": "daily",
  "blockheight": 0,
  "descriptor": "wpkh([deadbeef/84'/0'/0']xpub.../<0;1>/*)#abcdefgh",
  "devices": ["unknown"]
}"#;
        assert!(SpecterParser::sniff(blob));
    }

    #[test]
    fn sniff_true_on_blockheight_zero() {
        // `blockheight: 0` is the default emit value from
        // `wallet_export/specter.rs:67` — must be a valid sniff.
        let blob = br#"{"label":"x","blockheight":0,"descriptor":"wpkh(xpub.../<0;1>/*)#abcdefgh","devices":[{"type":"unknown","label":""}]}"#;
        assert!(SpecterParser::sniff(blob));
    }

    #[test]
    fn sniff_true_with_extra_top_level_fields() {
        // Specter is liberal in what it emits; the toolkit's positive sniff
        // checks the 4 required markers and ignores unrecognized siblings
        // (P2B's parser collects them into dropped_fields).
        let blob = br#"{
  "label": "daily",
  "blockheight": 800000,
  "descriptor": "wpkh(xpub.../<0;1>/*)#abcdefgh",
  "devices": [{"type":"coldcard","label":"primary"}],
  "unknown_specter_field": "ignored at sniff time",
  "another_field": 42
}"#;
        assert!(SpecterParser::sniff(blob));
    }

    #[test]
    fn sniff_true_on_multisig_devices_length_n() {
        // Multisig Specter wallet: devices array length = cosigner count.
        // Sniff doesn't validate length; it only checks the array shape.
        let blob = br#"{
  "label": "2of3 vault",
  "blockheight": 750000,
  "descriptor": "wsh(sortedmulti(2,[a]xpub.../<0;1>/*,[b]xpub.../<0;1>/*,[c]xpub.../<0;1>/*))#abcdefgh",
  "devices": [
    {"type":"coldcard","label":"sig1"},
    {"type":"trezor","label":"sig2"},
    {"type":"ledger","label":"sig3"}
  ]
}"#;
        assert!(SpecterParser::sniff(blob));
    }

    // -------------------------------------------------------------------------
    // Sniff: negative cases (SPEC §11.2 — missing marker / wrong type)
    // -------------------------------------------------------------------------

    #[test]
    fn sniff_false_on_missing_blockheight() {
        // `blockheight` is the distinctive Specter marker per SPEC §11.2 +
        // §6.1.1 VENDOR_MARKER_KEYS. Without it, sniff must reject.
        let blob = br#"{"label":"x","descriptor":"wpkh(xpub...)#abcdefgh","devices":[{"type":"unknown","label":""}]}"#;
        assert!(!SpecterParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_string_blockheight() {
        // Blockheight as STRING (not integer) — fails the integer-shape
        // check. This guards against accidentally accepting non-Specter
        // blobs that happen to carry the literal key `"blockheight"` with
        // an inappropriate type.
        let blob = br#"{"label":"x","blockheight":"800000","descriptor":"wpkh(xpub...)#abcdefgh","devices":[]}"#;
        assert!(!SpecterParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_float_blockheight() {
        // serde_json::Number::is_u64 + is_i64 both return false for floats.
        // Floats are not legitimate blockheights and should not satisfy sniff.
        let blob = br#"{"label":"x","blockheight":800000.5,"descriptor":"wpkh(xpub...)#abcdefgh","devices":[]}"#;
        assert!(!SpecterParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_missing_label() {
        let blob = br#"{"blockheight":800000,"descriptor":"wpkh(xpub...)#abcdefgh","devices":[]}"#;
        assert!(!SpecterParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_missing_descriptor() {
        let blob = br#"{"label":"x","blockheight":800000,"devices":[]}"#;
        assert!(!SpecterParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_missing_devices() {
        let blob = br#"{"label":"x","blockheight":800000,"descriptor":"wpkh(xpub...)#abcdefgh"}"#;
        assert!(!SpecterParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_devices_not_array() {
        // Devices as object (instead of array) fails sniff.
        let blob = br#"{"label":"x","blockheight":800000,"descriptor":"wpkh(xpub...)#abcdefgh","devices":{"k":"v"}}"#;
        assert!(!SpecterParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_label_not_string() {
        // Label as integer instead of string.
        let blob = br#"{"label":42,"blockheight":800000,"descriptor":"wpkh(xpub...)#abcdefgh","devices":[]}"#;
        assert!(!SpecterParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_descriptor_not_string() {
        let blob = br#"{"label":"x","blockheight":800000,"descriptor":42,"devices":[]}"#;
        assert!(!SpecterParser::sniff(blob));
    }

    // -------------------------------------------------------------------------
    // Sniff: cross-format negative — must NOT match other vendor blobs
    // -------------------------------------------------------------------------

    #[test]
    fn sniff_false_on_bsms_blob() {
        // BSMS Round-2 2-line shape; not JSON.
        let blob = b"BSMS 1.0\nwpkh([deadbeef/84'/0'/0']xpub...)/0/*#abcdefgh\n";
        assert!(!SpecterParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_bitcoin_core_listdescriptors() {
        // Bitcoin Core `listdescriptors` envelope: wallet_name + descriptors[].
        // Lacks blockheight / descriptor / devices — multiple sniff markers
        // absent.
        let blob = br#"{"wallet_name":"a","descriptors":[{"desc":"wpkh(xpub...)#abcdefgh"}]}"#;
        assert!(!SpecterParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_random_text() {
        assert!(!SpecterParser::sniff(b"some random text\n"));
    }

    #[test]
    fn sniff_false_on_empty_blob() {
        assert!(!SpecterParser::sniff(b""));
    }

    #[test]
    fn sniff_false_on_top_level_array() {
        // Top-level must be an object (Specter exports never use a bare array).
        let blob = br#"[{"label":"x","blockheight":0,"descriptor":"...","devices":[]}]"#;
        assert!(!SpecterParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_invalid_json() {
        let blob = br#"{not valid json"#;
        assert!(!SpecterParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_top_level_string() {
        let blob = br#""just a string""#;
        assert!(!SpecterParser::sniff(blob));
    }

    #[test]
    fn sniff_tolerates_leading_whitespace() {
        // Sniff inspects whitespace-trimmed blob for `{` prefix, but the
        // serde_json parse itself accepts leading whitespace. Both layers
        // must agree.
        let blob = br#"
  {
    "label":"x","blockheight":800000,
    "descriptor":"wpkh(xpub...)#abcdefgh","devices":[]
  }
"#;
        assert!(SpecterParser::sniff(blob));
    }

    // -------------------------------------------------------------------------
    // Skeleton parse impl — Phase P2A returns BadInput pending P2B.
    // -------------------------------------------------------------------------

    #[test]
    fn parse_skeleton_returns_p2b_not_yet_wired() {
        let blob = br#"{"label":"x","blockheight":0,"descriptor":"wpkh(xpub...)#abcdefgh","devices":[]}"#;
        let mut stderr = Vec::new();
        let err = SpecterParser::parse(blob, &mut stderr).unwrap_err();
        match err {
            ToolkitError::BadInput(msg) => {
                assert!(
                    msg.contains("P2B"),
                    "skeleton must cite Phase P2B; got: {msg}"
                );
                assert!(
                    msg.contains("not yet wired"),
                    "skeleton must say not yet wired; got: {msg}"
                );
            }
            other => panic!("expected BadInput, got: {other:?}"),
        }
    }
}
