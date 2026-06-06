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

use super::{
    pipeline::concrete_keys_to_placeholders, validate_watch_only_resolved, ImportProvenance,
    ParsedImport, WalletFormatParser,
};
use crate::error::ToolkitError;
use crate::parse_descriptor;
use crate::synthesize::{xpub_to_65, ResolvedSlot};
use bitcoin::bip32::{ChildNumber, DerivationPath, Fingerprint, Xpub};
use regex::Regex;
use serde_json::Value;
use std::io::Write;
use std::sync::OnceLock;

/// SPEC §11.2 — Specter-DIY wallet-import parser.
pub(crate) struct SpecterParser;

/// SPEC §11.2 — per-blob provenance metadata for a Specter-DIY parse.
/// Carried on `ImportProvenance::Specter(...)`; preserved for `--json`
/// envelope `source_metadata` emit (Phase P2B integration).
#[derive(Debug, Clone)]
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
pub(crate) struct SpecterDeviceMarker {
    /// Hardware-wallet type identifier (e.g., `"coldcard"`, `"trezor"`,
    /// `"ledger"`, `"unknown"`). Specter does not normalize the vendor
    /// vocabulary; the toolkit preserves whatever string is on the blob.
    pub(crate) device_type: String,
    /// User-supplied display label for the device. Empty string when the
    /// blob used the legacy `["<vendor>"]` string-array shape.
    pub(crate) label: String,
}

/// Top-level keys preserved on the Specter envelope by the toolkit's parse.
/// Any other top-level field surfaces in `SpecterSourceMetadata.dropped_fields`
/// and drives a stderr NOTICE per SPEC §2.4. Mirrors
/// `SPARROW_PRESERVED_TOP_LEVEL_KEYS` in `wallet_import/sparrow.rs:129`.
const SPECTER_PRESERVED_TOP_LEVEL_KEYS: &[&str] =
    &["label", "blockheight", "descriptor", "devices"];

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
        // (1) label: string.
        if obj.get("label").and_then(|v| v.as_str()).is_none() {
            return false;
        }
        // (2) blockheight: integer (u64 or i64; floats rejected).
        let blockheight_ok = obj
            .get("blockheight")
            .map(|v| v.is_u64() || v.is_i64())
            .unwrap_or(false);
        if !blockheight_ok {
            return false;
        }
        // (3) descriptor: string.
        if obj.get("descriptor").and_then(|v| v.as_str()).is_none() {
            return false;
        }
        // (4) devices: array (element shape validated at parse time).
        if obj.get("devices").and_then(|v| v.as_array()).is_none() {
            return false;
        }
        true
    }

    /// SPEC §11.2 — parse a Specter-DIY wallet JSON blob.
    ///
    /// Specter's `descriptor` field is the FULL concrete-keys form
    /// (`wpkh([fp/path]xpub/<0;1>/*)`, `wsh(sortedmulti(K, [fp/path]xpub/...))`,
    /// etc.) — NOT the `@N/**` placeholder form Sparrow uses. So the parse
    /// reduces to:
    ///
    /// 1. JSON-parse + top-level object check.
    /// 2. Extract envelope fields (`label`, `blockheight`, `descriptor`, `devices`).
    /// 3. Feed `descriptor` through `concrete_keys_to_placeholders` →
    ///    `parse_descriptor::parse_descriptor` (same pipeline BSMS + Bitcoin Core use).
    /// 4. Build `ResolvedSlot` cosigners with origin + xpub typed values.
    /// 5. Extract threshold from `multi(K, ...)` / `sortedmulti(K, ...)`.
    /// 6. Normalize devices array (object-form OR string-form → `SpecterDeviceMarker`).
    /// 7. Emit stderr NOTICE per SPEC §2.4 listing dropped envelope fields.
    /// 8. Wrap in `ParsedImport` with `ImportProvenance::Specter(...)`.
    fn parse(blob: &[u8], stderr: &mut dyn Write) -> Result<Vec<ParsedImport>, ToolkitError> {
        // Step 1: JSON parse.
        let value: Value = serde_json::from_slice(blob).map_err(|e| {
            ToolkitError::ImportWalletParse(format!(
                "import-wallet: specter: parse error: invalid JSON: {e}"
            ))
        })?;
        let obj = value.as_object().ok_or_else(|| {
            ToolkitError::ImportWalletParse(
                "import-wallet: specter: parse error: top-level JSON value is not an object"
                    .to_string(),
            )
        })?;

        // Step 2: envelope-field extraction.
        let label = obj
            .get("label")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                ToolkitError::ImportWalletParse(
                    "import-wallet: specter: parse error: missing or non-string top-level `label`"
                        .to_string(),
                )
            })?
            .to_string();
        let blockheight = obj
            .get("blockheight")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| {
                ToolkitError::ImportWalletParse(
                    "import-wallet: specter: parse error: missing or non-integer top-level `blockheight`"
                        .to_string(),
                )
            })?;
        let descriptor_str = obj
            .get("descriptor")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                ToolkitError::ImportWalletParse(
                    "import-wallet: specter: parse error: missing or non-string top-level `descriptor`"
                        .to_string(),
                )
            })?
            .to_string();
        let devices_arr = obj.get("devices").and_then(|v| v.as_array()).ok_or_else(|| {
            ToolkitError::ImportWalletParse(
                "import-wallet: specter: parse error: missing or non-array top-level `devices`"
                    .to_string(),
            )
        })?;

        // Step 3a: validate BIP-380 checksum on the ORIGINAL descriptor body
        // (concrete `[fp/path]xpub` keys present), mirroring the BSMS pattern
        // at `wallet_import/bsms.rs:195-207`. The downstream `parse_descriptor`
        // pipeline runs on the placeholder form (post-substitution) where the
        // original checksum no longer applies. Returns the body sans
        // `#<checksum>` suffix; the placeholder adapter consumes this
        // stripped form.
        let descriptor_body_no_csum =
            miniscript::descriptor::checksum::verify_checksum(&descriptor_str).map_err(|e| {
                ToolkitError::ImportWalletParse(format!(
                    "import-wallet: specter: parse error: BIP-380 checksum validation failed: {e}"
                ))
            })?;

        // Step 3b: feed descriptor through the concrete-keys pipeline.
        let (placeholder_form, parsed_keys, parsed_fingerprints) =
            concrete_keys_to_placeholders(descriptor_body_no_csum).map_err(|e| {
                ToolkitError::ImportWalletParse(e.message().replacen(
                    "import-wallet: bsms:",
                    "import-wallet: specter:",
                    1,
                ))
            })?;

        let descriptor = parse_descriptor::parse_descriptor(
            &placeholder_form,
            &parsed_keys,
            &parsed_fingerprints,
        )
        .map_err(|e| {
            ToolkitError::ImportWalletParse(format!(
                "import-wallet: specter: parse error: {}",
                e.message()
            ))
        })?;

        // Step 4: build ResolvedSlot vec from origins.
        let origins =
            crate::wallet_import::pipeline::extract_origin_components(&descriptor_str, "specter")?;
        let network = network_from_origins(&origins)?;
        let mut cosigners: Vec<ResolvedSlot> = Vec::with_capacity(parsed_keys.len());
        for (i, _) in parsed_keys.iter().enumerate() {
            let (xpub, fp, path) = build_slot_fields(&descriptor_str, i)?;
            debug_assert_eq!(xpub_to_65(&xpub), parsed_keys[i].payload);
            cosigners.push(ResolvedSlot {
                xpub,
                fingerprint: fp,
                path,
                entropy: None,
                master_xpub: None,
                language: None,
                _entropy_pin: None,
            });
        }
        validate_watch_only_resolved(&cosigners)?;

        // Step 5: threshold extraction (multisig only; singlesig → None).
        let threshold = extract_threshold_local(&descriptor_str)?;

        // Step 6: normalize devices array.
        let mut devices: Vec<SpecterDeviceMarker> = Vec::with_capacity(devices_arr.len());
        for (i, d) in devices_arr.iter().enumerate() {
            devices.push(parse_device(i, d)?);
        }

        // Step 7: dropped-field detection + stderr NOTICE.
        let mut dropped_fields: Vec<String> = Vec::new();
        for (k, _) in obj.iter() {
            if !SPECTER_PRESERVED_TOP_LEVEL_KEYS.contains(&k.as_str()) {
                dropped_fields.push(k.clone());
            }
        }
        if !dropped_fields.is_empty() {
            writeln!(
                stderr,
                "notice: import-wallet: specter: dropped envelope fields {}: not preserved in bundle output (key-state only)",
                dropped_fields.join(", ")
            )
            .map_err(ToolkitError::Io)?;
        }

        let source_metadata = SpecterSourceMetadata {
            label,
            blockheight,
            devices,
            dropped_fields,
        };

        // Step 8: rebuild original_descriptor with a fresh BIP-380 checksum
        // (Specter's wire shape carries the descriptor with `#csum` suffix,
        // but the toolkit re-emits for byte-determinism). On checksum-engine
        // failure (non-ASCII / odd chars), fall back to the verbatim
        // descriptor string (downstream BundleJson does not crash on a
        // missing `#csum`).
        let original_descriptor = match recompute_descriptor_checksum(&descriptor_str) {
            Ok(s) => s,
            Err(_) => descriptor_str.clone(),
        };

        Ok(vec![ParsedImport {
            descriptor,
            original_descriptor,
            cosigners,
            network,
            threshold,
            provenance: ImportProvenance::Specter(source_metadata),
        }])
    }
}

/// Parse one `devices[i]` element into a `SpecterDeviceMarker`. Tolerates
/// both the legacy `string` shape (toolkit's own emit at
/// `wallet_export/specter.rs:55`) and the modern object shape
/// (`{type: ..., label: ...}`).
fn parse_device(i: usize, d: &Value) -> Result<SpecterDeviceMarker, ToolkitError> {
    // Object form: {"type": "<vendor>", "label": "<name>"}
    if let Some(obj) = d.as_object() {
        let device_type = obj
            .get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                ToolkitError::ImportWalletParse(format!(
                    "import-wallet: specter: parse error: devices[{i}].type missing or not a string"
                ))
            })?
            .to_string();
        // `label` is optional on the object form; default to empty.
        let label = obj
            .get("label")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        return Ok(SpecterDeviceMarker { device_type, label });
    }
    // String form: `"<vendor>"` (legacy / toolkit-side emit).
    if let Some(s) = d.as_str() {
        return Ok(SpecterDeviceMarker {
            device_type: s.to_string(),
            label: String::new(),
        });
    }
    Err(ToolkitError::ImportWalletParse(format!(
        "import-wallet: specter: parse error: devices[{i}] must be an object or string, got {d:?}"
    )))
}

fn build_slot_fields(
    descriptor_body: &str,
    slot_idx: usize,
) -> Result<(Xpub, Fingerprint, DerivationPath), ToolkitError> {
    let origins =
        crate::wallet_import::pipeline::extract_origin_components(descriptor_body, "specter")?;
    let (fp, path, xpub_str) = origins.into_iter().nth(slot_idx).ok_or_else(|| {
        ToolkitError::ImportWalletParse(format!(
            "import-wallet: specter: parse error: slot index {slot_idx} out of range"
        ))
    })?;
    crate::wallet_import::pipeline::finalize_slot_fields(fp, path, &xpub_str, "specter")
}

fn network_from_origins(
    origins: &[(Fingerprint, DerivationPath, String)],
) -> Result<bitcoin::Network, ToolkitError> {
    if origins.is_empty() {
        return Err(ToolkitError::ImportWalletParse(
            "import-wallet: specter: parse error: no origins to infer network from".to_string(),
        ));
    }
    let coin_types: Vec<u32> = origins
        .iter()
        .map(|(_, p, _)| coin_type_from_path(p))
        .collect::<Result<Vec<_>, _>>()?;
    let first = coin_types[0];
    for (i, ct) in coin_types.iter().enumerate().skip(1) {
        if *ct != first {
            return Err(ToolkitError::ImportWalletParse(format!(
                "import-wallet: specter: cosigner {i} has coin-type {ct}, cosigner 0 has coin-type {first}; all cosigners must share a coin-type"
            )));
        }
    }
    match first {
        0 => Ok(bitcoin::Network::Bitcoin),
        1 => Ok(bitcoin::Network::Testnet),
        other => Err(ToolkitError::ImportWalletParse(format!(
            "import-wallet: specter: parse error: unsupported coin-type {other} on origin path; only 0 (mainnet) and 1 (testnet) supported per BIP-48"
        ))),
    }
}

fn coin_type_from_path(path: &DerivationPath) -> Result<u32, ToolkitError> {
    let comps: Vec<&ChildNumber> = path.into_iter().collect();
    if comps.len() < 2 {
        return Err(ToolkitError::ImportWalletParse(format!(
            "import-wallet: specter: parse error: origin path has only {} components; need ≥2 for BIP-48 coin-type inference",
            comps.len()
        )));
    }
    match comps[1] {
        ChildNumber::Hardened { index } => Ok(*index),
        ChildNumber::Normal { index } => Err(ToolkitError::ImportWalletParse(format!(
            "import-wallet: specter: parse error: coin-type component {index} is not hardened; BIP-48 requires `<coin_type>'`"
        ))),
    }
}

/// Extract K from `multi(K, ...)` / `sortedmulti(K, ...)`. Returns `Ok(None)`
/// for singlesig descriptors. Mirrors `sparrow::extract_threshold_local`.
fn extract_threshold_local(descriptor_body: &str) -> Result<Option<u8>, ToolkitError> {
    static R: OnceLock<Regex> = OnceLock::new();
    let re = R.get_or_init(|| {
        Regex::new(r"(?:thresh|multi|sortedmulti|multi_a|sortedmulti_a)\((\d+)\s*,")
            .expect("threshold regex is fixed")
    });
    let cap = match re.captures(descriptor_body) {
        Some(c) => c,
        None => return Ok(None),
    };
    let arg = cap.get(1).expect("regex has capture group 1").as_str();
    arg.parse::<u8>().map(Some).map_err(|e| {
        ToolkitError::ImportWalletParse(format!(
            "import-wallet: specter: parse error: multi argument `{arg}` exceeds u8 range (>255 cosigners not supported): {e}"
        ))
    })
}

/// Re-render a concrete-keys descriptor with a freshly computed BIP-380
/// checksum, mirroring `sparrow::recompute_descriptor_checksum`.
fn recompute_descriptor_checksum(body: &str) -> Result<String, ToolkitError> {
    use miniscript::descriptor::checksum::Engine as ChecksumEngine;
    let body_no_csum = match body.rsplit_once('#') {
        Some((b, _)) => b,
        None => body,
    };
    let mut eng = ChecksumEngine::new();
    eng.input(body_no_csum).map_err(|e| {
        ToolkitError::ImportWalletParse(format!(
            "import-wallet: specter: checksum engine input rejected: {e}"
        ))
    })?;
    let csum = eng.checksum();
    Ok(format!("{body_no_csum}#{csum}"))
}

/// Strip ASCII leading whitespace before checking for `{` prefix. Mirrors
/// `wallet_import/bitcoin_core.rs:566`'s helper; we inline a sibling copy here
/// to keep `bitcoin_core.rs::trim_leading_ws` `pub(super)`-free.
fn trim_leading_ws(blob: &[u8]) -> &[u8] {
    let mut i = 0;
    while i < blob.len() && (blob[i] == b' ' || blob[i] == b'\t' || blob[i] == b'\n' || blob[i] == b'\r')
    {
        i += 1;
    }
    &blob[i..]
}

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // Sniff: positive cases (SPEC §11.2 — all 4 markers present)
    // -------------------------------------------------------------------------

    #[test]
    fn sniff_true_on_minimal_singlesig_blob() {
        let blob = br#"{
  "label": "Daily",
  "blockheight": 800000,
  "descriptor": "wpkh([5436d724/84'/0'/0']xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9/<0;1>/*)#00lx6ere",
  "devices": [{"type":"coldcard","label":"primary"}]
}"#;
        assert!(SpecterParser::sniff(blob));
    }

    #[test]
    fn sniff_true_on_blockheight_zero() {
        // `blockheight: 0` is the default emit value from
        // `wallet_export/specter.rs:67` — must be a valid sniff.
        let blob = br#"{"label":"x","blockheight":0,"descriptor":"wpkh(xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9/<0;1>/*)#abcdefgh","devices":[{"type":"unknown","label":""}]}"#;
        assert!(SpecterParser::sniff(blob));
    }

    #[test]
    fn sniff_true_on_legacy_string_devices() {
        // Toolkit-side emitter at `wallet_export/specter.rs:55` produces
        // `devices: Vec<&'static str>` (`["unknown"]`). The sniff doesn't
        // validate device-element shape — that's a parse-time concern.
        let blob = br#"{"label":"x","blockheight":0,"descriptor":"wpkh(xpub.../<0;1>/*)#abcdefgh","devices":["unknown"]}"#;
        assert!(SpecterParser::sniff(blob));
    }

    #[test]
    fn sniff_true_on_empty_devices_array() {
        // Empty array is still an array — sniff is shape-only here.
        let blob = br#"{"label":"x","blockheight":0,"descriptor":"wpkh(xpub.../<0;1>/*)#abcdefgh","devices":[]}"#;
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
        let blob = br#"{"label":"x","descriptor":"wpkh(xpub.../<0;1>/*)#abcdefgh","devices":[{"type":"unknown","label":""}]}"#;
        assert!(!SpecterParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_string_blockheight() {
        // Blockheight as STRING (not integer) — fails the integer-shape
        // check. This guards against accidentally accepting non-Specter
        // blobs that happen to carry the literal key `"blockheight"` with
        // an inappropriate type.
        let blob = br#"{"label":"x","blockheight":"800000","descriptor":"wpkh(xpub.../<0;1>/*)#abcdefgh","devices":[]}"#;
        assert!(!SpecterParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_float_blockheight() {
        // serde_json::Number::is_u64 + is_i64 both return false for floats.
        // Floats are not legitimate blockheights and should not satisfy sniff.
        let blob = br#"{"label":"x","blockheight":800000.5,"descriptor":"wpkh(xpub.../<0;1>/*)#abcdefgh","devices":[]}"#;
        assert!(!SpecterParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_missing_label() {
        let blob = br#"{"blockheight":800000,"descriptor":"wpkh(xpub.../<0;1>/*)#abcdefgh","devices":[]}"#;
        assert!(!SpecterParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_missing_descriptor() {
        let blob = br#"{"label":"x","blockheight":800000,"devices":[]}"#;
        assert!(!SpecterParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_missing_devices() {
        let blob = br#"{"label":"x","blockheight":800000,"descriptor":"wpkh(xpub.../<0;1>/*)#abcdefgh"}"#;
        assert!(!SpecterParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_devices_not_array() {
        // Devices as object (instead of array) fails sniff.
        let blob = br#"{"label":"x","blockheight":800000,"descriptor":"wpkh(xpub.../<0;1>/*)#abcdefgh","devices":{"k":"v"}}"#;
        assert!(!SpecterParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_label_not_string() {
        // Label as integer instead of string.
        let blob = br#"{"label":42,"blockheight":800000,"descriptor":"wpkh(xpub.../<0;1>/*)#abcdefgh","devices":[]}"#;
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
        let blob = b"BSMS 1.0\nwpkh([deadbeef/84'/0'/0']xpub.../0/*)#abcdefgh\n";
        assert!(!SpecterParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_bitcoin_core_listdescriptors() {
        // Bitcoin Core `listdescriptors` envelope: wallet_name + descriptors[].
        // Lacks blockheight / descriptor (top-level) / devices — multiple
        // sniff markers absent.
        let blob = br#"{"wallet_name":"a","descriptors":[{"desc":"wpkh(xpub.../<0;1>/*)#abcdefgh"}]}"#;
        assert!(!SpecterParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_sparrow_blob() {
        // Sparrow has policyType/scriptType/defaultPolicy/keystores, no
        // blockheight + no top-level descriptor.
        let blob = br#"{"policyType":"SINGLE","scriptType":"P2WPKH","defaultPolicy":{"miniscript":{"script":"wpkh(@0/**)"}},"keystores":[{"keyDerivation":{"masterFingerprint":"5436d724","derivation":"m/84'/0'/0'"},"extendedPublicKey":"xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9"}]}"#;
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
    "descriptor":"wpkh(xpub.../<0;1>/*)#abcdefgh","devices":[]
  }
"#;
        assert!(SpecterParser::sniff(blob));
    }

    // ========================================================================
    // PARSE cells (P2B)
    // ========================================================================

    fn parse(blob: &[u8]) -> Result<Vec<ParsedImport>, ToolkitError> {
        let mut stderr = Vec::new();
        SpecterParser::parse(blob, &mut stderr)
    }

    fn parse_capturing_stderr(blob: &[u8]) -> (Result<Vec<ParsedImport>, ToolkitError>, String) {
        let mut stderr = Vec::new();
        let r = SpecterParser::parse(blob, &mut stderr);
        (r, String::from_utf8(stderr).unwrap_or_default())
    }

    /// Parse: singlesig P2WPKH happy-path. Specter's `descriptor` field
    /// is the FULL concrete-keys form — no `@N/**` substitution needed.
    #[test]
    fn parse_singlesig_p2wpkh_mainnet_happy_path() {
        let blob = br#"{
            "label":"Daily",
            "blockheight":800000,
            "descriptor":"wpkh([5436d724/84'/0'/0']xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9/<0;1>/*)#00lx6ere",
            "devices":[{"type":"coldcard","label":"primary"}]
        }"#;
        let parsed = parse(blob).unwrap();
        assert_eq!(parsed.len(), 1);
        let p = &parsed[0];
        assert_eq!(p.cosigners.len(), 1);
        assert_eq!(p.network, bitcoin::Network::Bitcoin);
        assert_eq!(p.threshold, None);
        assert!(matches!(p.provenance, ImportProvenance::Specter(_)));
        if let ImportProvenance::Specter(meta) = &p.provenance {
            assert_eq!(meta.label, "Daily");
            assert_eq!(meta.blockheight, 800000);
            assert_eq!(meta.devices.len(), 1);
            assert_eq!(meta.devices[0].device_type, "coldcard");
            assert_eq!(meta.devices[0].label, "primary");
            assert!(meta.dropped_fields.is_empty());
        } else {
            panic!("provenance");
        }
    }

    /// Parse: multisig 2-of-3 P2WSH sortedmulti happy-path.
    #[test]
    fn parse_multisig_2of3_p2wsh_sortedmulti_happy_path() {
        let blob = br#"{
            "label":"VaultColdStorage",
            "blockheight":750000,
            "descriptor":"wsh(sortedmulti(2,[b8688df1/48'/0'/0'/2']xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX/<0;1>/*,[28645006/48'/0'/0'/2']xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6/<0;1>/*,[5436d724/48'/0'/0'/2']xpub6Buxw9MmbkJr4iAw8SACNci2hQNuPCMwt9P7HkK62ZQAW9UcJaQ2bc6ARD892TToQQ9Rp6AHujHxBLXqAsvn5fRnLfnhKSRfz8qtaoyKUYx/<0;1>/*))#he0ej3xr",
            "devices":[
                {"type":"coldcard","label":"sig1"},
                {"type":"trezor","label":"sig2"},
                {"type":"ledger","label":"sig3"}
            ]
        }"#;
        let parsed = parse(blob).unwrap();
        assert_eq!(parsed.len(), 1);
        let p = &parsed[0];
        assert_eq!(p.cosigners.len(), 3);
        assert_eq!(p.network, bitcoin::Network::Bitcoin);
        assert_eq!(p.threshold, Some(2));
        // Cosigner ordering preserved (declaration order from descriptor).
        assert_eq!(p.cosigners[0].fingerprint.to_string(), "b8688df1");
        assert_eq!(p.cosigners[1].fingerprint.to_string(), "28645006");
        assert_eq!(p.cosigners[2].fingerprint.to_string(), "5436d724");
        if let ImportProvenance::Specter(meta) = &p.provenance {
            assert_eq!(meta.label, "VaultColdStorage");
            assert_eq!(meta.blockheight, 750000);
            assert_eq!(meta.devices.len(), 3);
            assert_eq!(meta.devices[1].device_type, "trezor");
        }
    }

    /// Parse: tolerates legacy string-form `devices: ["unknown"]` shape
    /// (matches toolkit-side `wallet_export/specter.rs:55` emitter).
    #[test]
    fn parse_tolerates_legacy_string_devices_array() {
        let blob = br#"{
            "label":"x",
            "blockheight":0,
            "descriptor":"wpkh([5436d724/84'/0'/0']xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9/<0;1>/*)#00lx6ere",
            "devices":["unknown"]
        }"#;
        let parsed = parse(blob).unwrap();
        assert_eq!(parsed.len(), 1);
        if let ImportProvenance::Specter(meta) = &parsed[0].provenance {
            assert_eq!(meta.devices.len(), 1);
            assert_eq!(meta.devices[0].device_type, "unknown");
            assert_eq!(meta.devices[0].label, "", "string-form devices normalize to empty label");
        }
    }

    /// Stderr NOTICE: dropped fields surface via SPEC §2.4 template.
    #[test]
    fn parse_emits_notice_for_dropped_fields() {
        let blob = br#"{
            "label":"x",
            "blockheight":0,
            "descriptor":"wpkh([5436d724/84'/0'/0']xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9/<0;1>/*)#00lx6ere",
            "devices":[],
            "some_extra_field":"ignored",
            "another_field":42
        }"#;
        let (res, stderr) = parse_capturing_stderr(blob);
        let parsed = res.unwrap();
        assert_eq!(parsed.len(), 1);
        assert!(
            stderr.contains("notice: import-wallet: specter: dropped envelope fields")
                && stderr.contains("some_extra_field")
                && stderr.contains("another_field"),
            "expected stderr NOTICE listing both dropped fields; got: {stderr}"
        );
        if let ImportProvenance::Specter(meta) = &parsed[0].provenance {
            assert!(meta.dropped_fields.iter().any(|f| f == "some_extra_field"));
            assert!(meta.dropped_fields.iter().any(|f| f == "another_field"));
        }
    }

    /// Testnet detection via BIP-48 coin-type=1.
    #[test]
    fn parse_testnet_network_inferred_from_coin_type_one() {
        let blob = br#"{
            "label":"testnet",
            "blockheight":0,
            "descriptor":"wpkh([704c7836/84'/1'/0']tpubDEgS9fUEpucKatmvKAv21v8nViHxR6rsV7ohMWK4YjsWd4EWT3w8YzMgMEvNrDfsUANbid74WRFpr3Gym8UHBSLnqg6b1Lzvibw87cLSctC/<0;1>/*)#r486umak",
            "devices":["unknown"]
        }"#;
        let parsed = parse(blob).unwrap();
        assert_eq!(parsed[0].network, bitcoin::Network::Testnet);
    }

    /// Refusal: malformed JSON.
    #[test]
    fn parse_malformed_json_refused() {
        let blob = br#"{not json"#;
        let err = parse(blob).unwrap_err();
        let msg = format!("{err:?}");
        assert!(
            msg.contains("specter") && msg.contains("invalid JSON"),
            "expected specter-tagged invalid-JSON error; got: {msg}"
        );
    }

    /// Refusal: missing `descriptor` top-level field.
    #[test]
    fn parse_missing_descriptor_refused() {
        let blob = br#"{"label":"x","blockheight":0,"devices":[]}"#;
        let err = parse(blob).unwrap_err();
        let msg = format!("{err:?}");
        assert!(
            msg.contains("specter") && msg.contains("`descriptor`"),
            "expected missing-descriptor error; got: {msg}"
        );
    }

    /// Refusal: descriptor without origin annotation (Specter always emits
    /// `[fp/path]xpub` form per its wallet_importer.py; a bare `xpub...` form
    /// has no fingerprint/path to extract for ResolvedSlot construction).
    /// The descriptor body carries a valid BIP-380 checksum (nczup5a0,
    /// computed for the no-origin form), so the failure surfaces at the
    /// origin-extraction step, NOT the upstream checksum verify.
    #[test]
    fn parse_descriptor_without_origin_refused() {
        let blob = br#"{
            "label":"x","blockheight":0,
            "descriptor":"wpkh(xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9/<0;1>/*)#nczup5a0",
            "devices":[]
        }"#;
        let err = parse(blob).unwrap_err();
        let msg = format!("{err:?}");
        assert!(
            msg.contains("specter") && msg.contains("no [fp/path]xpub keys"),
            "expected no-keys error; got: {msg}"
        );
    }

    /// Refusal: device element neither object nor string.
    #[test]
    fn parse_devices_invalid_element_refused() {
        let blob = br#"{
            "label":"x","blockheight":0,
            "descriptor":"wpkh([5436d724/84'/0'/0']xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9/<0;1>/*)#00lx6ere",
            "devices":[42]
        }"#;
        let err = parse(blob).unwrap_err();
        let msg = format!("{err:?}");
        assert!(
            msg.contains("specter") && msg.contains("devices[0]"),
            "expected devices[0] error; got: {msg}"
        );
    }

    /// SLIP-132 zpub variant normalized to neutral xpub form by the pipeline.
    /// Note: the BIP-380 checksum is computed over the verbatim wire form
    /// (with `zpub` prefix in this case), NOT the post-normalize form, so
    /// the input here uses the computed-for-zpub checksum.
    #[test]
    fn parse_zpub_variant_normalized() {
        let blob = br#"{
            "label":"x","blockheight":0,
            "descriptor":"wpkh([5436d724/84'/0'/0']zpub6qTBTNftBzVTjgVcSUw7vW5N1KQbV93Jnrw314RHGkCkSx4vk6nEWH1MJfReXi2WThvuDRiRpyT7cDoakEcZMQ1iZPgfJgQrcVMR4aJWh6S/<0;1>/*)#yxzx3ag7",
            "devices":["unknown"]
        }"#;
        let parsed = parse(blob).unwrap();
        assert_eq!(parsed.len(), 1);
        assert!(
            parsed[0].cosigners[0].xpub.to_string().starts_with("xpub6"),
            "zpub must normalize to xpub; got: {}",
            parsed[0].cosigners[0].xpub
        );
    }

    // ========================================================================
    // Fixture-driven cells: load tests/fixtures/wallet_import/specter-*.json
    // ========================================================================

    fn load_fixture(name: &str) -> Vec<u8> {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/wallet_import")
            .join(name);
        std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
    }

    #[test]
    fn fixture_singlesig_p2wpkh_parses_clean() {
        let blob = load_fixture("specter-singlesig-p2wpkh.json");
        let parsed = parse(&blob).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].cosigners.len(), 1);
        assert_eq!(parsed[0].network, bitcoin::Network::Bitcoin);
        assert_eq!(parsed[0].threshold, None);
    }

    #[test]
    fn fixture_multisig_2of3_sortedmulti_parses_clean() {
        let blob = load_fixture("specter-multisig-2of3-p2wsh-sortedmulti.json");
        let parsed = parse(&blob).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].cosigners.len(), 3);
        assert_eq!(parsed[0].threshold, Some(2));
    }

    #[test]
    fn fixture_blockheight_zero_parses_clean() {
        let blob = load_fixture("specter-blockheight-zero.json");
        let parsed = parse(&blob).unwrap();
        assert_eq!(parsed.len(), 1);
        if let ImportProvenance::Specter(meta) = &parsed[0].provenance {
            assert_eq!(meta.blockheight, 0);
        }
    }

    #[test]
    fn fixture_descriptor_with_checksum_parses_clean() {
        let blob = load_fixture("specter-descriptor-with-checksum.json");
        let parsed = parse(&blob).unwrap();
        assert_eq!(parsed.len(), 1);
    }
}
