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
//! Both shapes are accepted at sniff + parse time; the parser normalizes
//! string-form into `SpecterDeviceMarker { device_type: <string>, label: "" }`.
//!
//! Parse semantics (Phase P2B):
//! 1. JSON-parse blob; extract the 4 required fields per sniff contract.
//! 2. Reject any `descriptor` carrying an extended-private-key prefix per
//!    SPEC §11.2 + Phase P3 R0 architect C1 fold (mirrors
//!    `bitcoin_core::parse_entry` xprv-forbid rule).
//! 3. Validate the BIP-380 checksum on the descriptor body.
//! 4. Run the concrete-keys → `@N` adapter + `parse_descriptor` pipeline
//!    (same shape as BSMS / Core).
//! 5. Build per-cosigner `ResolvedSlot` (entropy: None — Specter export is
//!    watch-only by construction; seed overlay applies at the CLI dispatch
//!    layer per `cmd/import_wallet.rs::apply_seed_overlay`).
//! 6. Infer network from the FIRST cosigner's BIP-48 coin-type child number
//!    (mirrors `bsms::network_from_origins` + `bitcoin_core::network_from_origins`).
//! 7. Normalize the `devices` array into a `Vec<SpecterDeviceMarker>` —
//!    object-form `{type, label}` decoded verbatim; legacy string-form
//!    projected to `{type: <string>, label: ""}`. Length must match the
//!    descriptor's cosigner count; an explicit error fires otherwise.
//! 8. Collect any top-level fields outside `{label, blockheight, descriptor,
//!    devices}` into `dropped_fields` (preserved on provenance for
//!    `--json` envelope `source_metadata` emit).
//!
//! Phase P2C wires the 8 `cmd/import_wallet.rs` dispatch sites to invoke
//! this parser; until then it is unreachable from the CLI (Site 2 panics
//! first on `--format specter` via `unimplemented!("P2C: ...")`, and the
//! auto-sniff path's `None =>` arm hits an `unreachable!()` catch-all for
//! `SniffOutcome::Specter`).

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
use std::str::FromStr;
use std::sync::OnceLock;

/// SPEC §11.2 — Specter-DIY wallet-import parser.
pub(crate) struct SpecterParser;

/// SPEC §11.2 — per-blob provenance metadata for a Specter-DIY parse.
/// Carried on `ImportProvenance::Specter(...)`; preserved for `--json`
/// envelope `source_metadata` emit (Phase P2C integration adds the envelope
/// read; the `#[allow(dead_code)]` lifts at P2C).
#[derive(Debug, Clone)]
#[allow(dead_code)] // P2B: fields populated by parse impl; P2C wires envelope-emit reads.
pub(crate) struct SpecterSourceMetadata {
    /// Top-level `label` (wallet display name).
    pub(crate) label: String,
    /// Top-level `blockheight` (rescan-start block; 0 if absent or non-integer).
    pub(crate) blockheight: u64,
    /// Per-cosigner device hints. Length matches the descriptor's cosigner
    /// count for multisig; length 1 for singlesig. Each entry is a
    /// `SpecterDeviceMarker` (object-form) or a normalized
    /// `{device_type: "<vendor>", label: ""}` projection from legacy string-form.
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
///   `{device_type: <vendor>, label: ""}` during parse.
#[derive(Debug, Clone)]
#[allow(dead_code)] // P2B: fields populated by parse impl; P2C wires envelope-emit reads.
pub(crate) struct SpecterDeviceMarker {
    /// Hardware-wallet type identifier (e.g., `"coldcard"`, `"trezor"`,
    /// `"ledger"`, `"unknown"`). Specter does not normalize the vendor
    /// vocabulary; the toolkit preserves whatever string is on the blob.
    pub(crate) device_type: String,
    /// User-supplied display label for the device. Empty string when the
    /// blob used the legacy `["<vendor>"]` string-array shape.
    pub(crate) label: String,
}

/// SPEC §11.2 load-bearing top-level fields. Any key outside this set lands
/// in `SpecterSourceMetadata::dropped_fields` for envelope-side audit.
const KNOWN_TOP_LEVEL_FIELDS: &[&str] = &["label", "blockheight", "descriptor", "devices"];

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

    fn parse(blob: &[u8], stderr: &mut dyn Write) -> Result<Vec<ParsedImport>, ToolkitError> {
        // SPEC §11.2 step 1: JSON-parse.
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

        // SPEC §11.2 step 2: extract `label` (required string).
        let label = obj
            .get("label")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                ToolkitError::ImportWalletParse(
                    "import-wallet: specter: parse error: missing or non-string `label`".to_string(),
                )
            })?
            .to_string();

        // SPEC §11.2 step 3: extract `blockheight` (required integer; u64).
        // Negative integers (i64 < 0) are rejected — blockheights are
        // monotonically increasing non-negative counters; a negative value
        // signals a malformed export.
        let blockheight = match obj.get("blockheight") {
            Some(v) if v.is_u64() => v.as_u64().expect("u64 check above"),
            Some(v) if v.is_i64() => {
                let i = v.as_i64().expect("i64 check above");
                if i < 0 {
                    return Err(ToolkitError::ImportWalletParse(format!(
                        "import-wallet: specter: parse error: negative `blockheight` {i}; \
                         must be a non-negative integer"
                    )));
                }
                i as u64
            }
            Some(_) | None => {
                return Err(ToolkitError::ImportWalletParse(
                    "import-wallet: specter: parse error: missing or non-integer `blockheight`"
                        .to_string(),
                ));
            }
        };

        // SPEC §11.2 step 4: extract `descriptor` (required string).
        let desc_with_csum = obj
            .get("descriptor")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                ToolkitError::ImportWalletParse(
                    "import-wallet: specter: parse error: missing or non-string `descriptor`"
                        .to_string(),
                )
            })?;

        // SPEC §11.2 step 5 (mirrors `bitcoin_core::parse_entry` xprv-forbid):
        // refuse any extended-private-key prefix BIP-32 + SLIP-132. Strip the
        // BIP-380 `#<csum>` trailer first so the checksum's bech32-style
        // alphabet cannot stochastically false-positive (Phase 3 I1 lift).
        let body_for_xprv_check = match desc_with_csum.rsplit_once('#') {
            Some((body, _csum)) => body,
            None => desc_with_csum,
        };
        if xprv_prefix_regex().is_match(body_for_xprv_check) {
            return Err(ToolkitError::ImportWalletXprvForbidden);
        }

        // SPEC §11.2 step 6: validate BIP-380 checksum via miniscript.
        // Returns the descriptor body sans `#<csum>` suffix; the placeholder
        // adapter consumes this stripped form (same as bsms/bitcoin_core).
        let descriptor_body_no_csum =
            miniscript::descriptor::checksum::verify_checksum(desc_with_csum).map_err(|e| {
                ToolkitError::ImportWalletParse(format!(
                    "import-wallet: specter: parse error: BIP-380 checksum validation failed: {e}"
                ))
            })?;

        // SPEC §11.2 step 7: concrete-keys → @N adapter + parse_descriptor.
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

        // SPEC §11.2 step 8: per-cosigner origin extraction + network inference.
        let origins = extract_origin_components(descriptor_body_no_csum)?;
        let network = network_from_origins(&origins)?;

        let mut cosigners: Vec<ResolvedSlot> = Vec::with_capacity(parsed_keys.len());
        for (slot_idx, _) in parsed_keys.iter().enumerate() {
            let (xpub, fp, path, path_raw) =
                build_slot_fields(descriptor_body_no_csum, slot_idx)?;
            debug_assert_eq!(xpub_to_65(&xpub), parsed_keys[slot_idx].payload);
            cosigners.push(ResolvedSlot {
                xpub,
                fingerprint: fp,
                path,
                path_raw,
                entropy: None,
                master_xpub: None,
                _entropy_pin: None,
            });
        }

        validate_watch_only_resolved(&cosigners)?;

        let threshold = extract_threshold(descriptor_body_no_csum)?;

        // SPEC §11.2 step 9: devices array normalization.
        let devices_json = obj
            .get("devices")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                ToolkitError::ImportWalletParse(
                    "import-wallet: specter: parse error: missing or non-array `devices`"
                        .to_string(),
                )
            })?;
        let devices = normalize_devices(devices_json, cosigners.len(), stderr)?;

        // SPEC §11.2 step 10: collect non-{label,blockheight,descriptor,devices}
        // top-level keys into `dropped_fields` (mirrors
        // `CoreSourceMetadata.dropped_fields` convention).
        let mut dropped_fields: Vec<String> = Vec::new();
        for k in obj.keys() {
            if !KNOWN_TOP_LEVEL_FIELDS.contains(&k.as_str()) && !dropped_fields.contains(k) {
                dropped_fields.push(k.clone());
            }
        }
        if !dropped_fields.is_empty() {
            writeln!(
                stderr,
                "notice: import-wallet: specter: dropped unrecognized top-level fields {}: not preserved in bundle output (key-state only)",
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

        Ok(vec![ParsedImport {
            descriptor,
            original_descriptor: desc_with_csum.to_string(),
            cosigners,
            network,
            threshold,
            provenance: ImportProvenance::Specter(source_metadata),
        }])
    }
}

/// Normalize the JSON `devices` array into `Vec<SpecterDeviceMarker>`.
/// Accepts BOTH legacy string-form (`["coldcard"]`) and object-form
/// (`[{"type": "coldcard", "label": "primary"}]`).
///
/// **Length policy (SPEC §11.2 step 9):** devices array length should equal
/// the descriptor's cosigner count. The toolkit's own `wallet_export/specter.rs`
/// emitter at line 62-63 enforces this invariant on output. On INPUT we
/// surface a stderr NOTICE for `devices.len() != cosigner_count` (lenient)
/// and pad/truncate to the cosigner count so the provenance struct's
/// invariant (1-to-1 with cosigner slots) holds. Some Specter firmware
/// variants reportedly emit `devices: []` for watch-only-via-descriptor
/// exports; we tolerate that shape by padding with `unknown` placeholders.
fn normalize_devices(
    devices_json: &[Value],
    cosigner_count: usize,
    stderr: &mut dyn Write,
) -> Result<Vec<SpecterDeviceMarker>, ToolkitError> {
    let mut out: Vec<SpecterDeviceMarker> = Vec::with_capacity(devices_json.len().max(cosigner_count));
    for (i, entry) in devices_json.iter().enumerate() {
        match entry {
            // Legacy string-form: project to {device_type: <s>, label: ""}.
            Value::String(s) => out.push(SpecterDeviceMarker {
                device_type: s.clone(),
                label: String::new(),
            }),
            // Modern object-form: extract `type` + `label`. Per Specter's
            // wallet_importer.py, `type` is required + `label` is optional
            // (defaults to empty string).
            Value::Object(map) => {
                let device_type = map
                    .get("type")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        ToolkitError::ImportWalletParse(format!(
                            "import-wallet: specter: parse error: devices[{i}]: missing or non-string `type`"
                        ))
                    })?
                    .to_string();
                let label = map
                    .get("label")
                    .and_then(|v| v.as_str())
                    .map(str::to_string)
                    .unwrap_or_default();
                out.push(SpecterDeviceMarker { device_type, label });
            }
            other => {
                return Err(ToolkitError::ImportWalletParse(format!(
                    "import-wallet: specter: parse error: devices[{i}] is neither a string nor an object (got {})",
                    json_kind(other)
                )));
            }
        }
    }
    // Length-vs-cosigner-count: lenient. Emit NOTICE; pad or truncate.
    if out.len() != cosigner_count {
        writeln!(
            stderr,
            "notice: import-wallet: specter: devices array length {} differs from cosigner count {}; provenance devices vector normalized to match cosigner slots",
            out.len(),
            cosigner_count
        )
        .map_err(ToolkitError::Io)?;
    }
    match out.len().cmp(&cosigner_count) {
        std::cmp::Ordering::Less => {
            let pad = cosigner_count - out.len();
            for _ in 0..pad {
                out.push(SpecterDeviceMarker {
                    device_type: "unknown".to_string(),
                    label: String::new(),
                });
            }
        }
        std::cmp::Ordering::Greater => {
            out.truncate(cosigner_count);
        }
        std::cmp::Ordering::Equal => {}
    }
    Ok(out)
}

/// Compact JSON type label used by error templates.
fn json_kind(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// Per-cosigner origin tuple lifted out of the descriptor body via the
/// shared `[fp/path]xpub` regex. Returned in declaration order. Mirrors
/// `bsms::extract_origin_components` + `bitcoin_core::extract_origin_components`
/// (kept separate so the error-message prefix carries the `specter` tag).
fn extract_origin_components(
    descriptor_body: &str,
) -> Result<Vec<(Fingerprint, DerivationPath, String, String)>, ToolkitError> {
    let re = origin_capture_regex();
    let mut out = Vec::new();
    for cap in re.captures_iter(descriptor_body) {
        let fp_hex = cap.get(1).expect("group 1").as_str();
        let path_raw_inner = cap.get(2).expect("group 2").as_str();
        let xpub_str = cap.get(3).expect("group 3").as_str();

        let mut fp_bytes = [0u8; 4];
        for i in 0..4 {
            fp_bytes[i] = u8::from_str_radix(&fp_hex[i * 2..i * 2 + 2], 16).map_err(|e| {
                ToolkitError::ImportWalletParse(format!(
                    "import-wallet: specter: parse error: fingerprint hex: {e}"
                ))
            })?;
        }
        let fp = Fingerprint::from(fp_bytes);
        let path = DerivationPath::from_str(&format!("m{path_raw_inner}")).map_err(|e| {
            ToolkitError::ImportWalletParse(format!(
                "import-wallet: specter: parse error: derivation-path parse: {e}"
            ))
        })?;
        let path_raw = format!("[{fp_hex}{path_raw_inner}]");
        out.push((fp, path, path_raw, xpub_str.to_string()));
    }
    if out.is_empty() {
        return Err(ToolkitError::ImportWalletParse(
            "import-wallet: specter: parse error: no origin annotations in descriptor".to_string(),
        ));
    }
    Ok(out)
}

fn build_slot_fields(
    descriptor_body: &str,
    slot_idx: usize,
) -> Result<(Xpub, Fingerprint, DerivationPath, String), ToolkitError> {
    let origins = extract_origin_components(descriptor_body)?;
    let (fp, path, path_raw, xpub_str) = origins.into_iter().nth(slot_idx).ok_or_else(|| {
        ToolkitError::ImportWalletParse(format!(
            "import-wallet: specter: parse error: slot index {slot_idx} out of range"
        ))
    })?;
    let (neutral, _variant) = crate::slip0132::normalize_xpub_prefix(&xpub_str)?;
    let xpub = Xpub::from_str(&neutral).map_err(|e| {
        ToolkitError::ImportWalletParse(format!(
            "import-wallet: specter: parse error: xpub decode for slot {slot_idx}: {e}"
        ))
    })?;
    Ok((xpub, fp, path, path_raw))
}

fn network_from_origins(
    origins: &[(Fingerprint, DerivationPath, String, String)],
) -> Result<bitcoin::Network, ToolkitError> {
    if origins.is_empty() {
        return Err(ToolkitError::ImportWalletParse(
            "import-wallet: specter: parse error: no origins to infer network from".to_string(),
        ));
    }
    let coin_types: Vec<u32> = origins
        .iter()
        .map(|(_, p, _, _)| coin_type_from_path(p))
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

/// Extract K from `thresh(K, ...)` / `multi(K, ...)` / `sortedmulti(K, ...)`
/// at the top-level miniscript context. Returns `Ok(None)` for single-key
/// shapes; `Err` for u8 overflow. Mirrors `bsms::extract_threshold` +
/// `bitcoin_core::extract_threshold`.
fn extract_threshold(descriptor_body: &str) -> Result<Option<u8>, ToolkitError> {
    static R: OnceLock<Regex> = OnceLock::new();
    let re = R.get_or_init(|| {
        Regex::new(r"(?:thresh|multi|sortedmulti)\((\d+)\s*,").expect("threshold regex is fixed")
    });
    let cap = match re.captures(descriptor_body) {
        Some(c) => c,
        None => return Ok(None),
    };
    let arg = cap.get(1).expect("regex has capture group 1").as_str();
    arg.parse::<u8>().map(Some).map_err(|e| {
        ToolkitError::ImportWalletParse(format!(
            "import-wallet: specter: parse error: thresh/multi argument `{arg}` exceeds u8 range (>255 cosigners not supported): {e}"
        ))
    })
}

/// Match any extended-private-key prefix per BIP-32 + SLIP-132 (Phase 3 R0
/// architect C1 fold; lifted from `bitcoin_core::xprv_prefix_regex`).
fn xprv_prefix_regex() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"[xtyzuvYZUV]prv[A-HJ-NP-Za-km-z1-9]+")
            .expect("xprv_prefix_regex is a fixed string literal")
    })
}

fn origin_capture_regex() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"\[([0-9a-fA-F]{8})((?:/\d+'?)+)\]([xtyzuvYZUV]pub[A-HJ-NP-Za-km-z1-9]+)")
            .expect("origin_capture_regex is a fixed string literal")
    })
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
        let blob = br#"{"label":"x","blockheight":0,"descriptor":"wpkh(xpub.../<0;1>/*)#abcdefgh","devices":[{"type":"unknown","label":""}]}"#;
        assert!(SpecterParser::sniff(blob));
    }

    #[test]
    fn sniff_true_with_extra_top_level_fields() {
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
    // Sniff: negative cases
    // -------------------------------------------------------------------------

    #[test]
    fn sniff_false_on_missing_blockheight() {
        let blob = br#"{"label":"x","descriptor":"wpkh(xpub...)#abcdefgh","devices":[{"type":"unknown","label":""}]}"#;
        assert!(!SpecterParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_string_blockheight() {
        let blob = br#"{"label":"x","blockheight":"800000","descriptor":"wpkh(xpub...)#abcdefgh","devices":[]}"#;
        assert!(!SpecterParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_float_blockheight() {
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
        let blob = br#"{"label":"x","blockheight":800000,"descriptor":"wpkh(xpub...)#abcdefgh","devices":{"k":"v"}}"#;
        assert!(!SpecterParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_label_not_string() {
        let blob = br#"{"label":42,"blockheight":800000,"descriptor":"wpkh(xpub...)#abcdefgh","devices":[]}"#;
        assert!(!SpecterParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_descriptor_not_string() {
        let blob = br#"{"label":"x","blockheight":800000,"descriptor":42,"devices":[]}"#;
        assert!(!SpecterParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_bsms_blob() {
        let blob = b"BSMS 1.0\nwpkh([deadbeef/84'/0'/0']xpub.../<0;1>/*)#abcdefgh\n";
        assert!(!SpecterParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_bitcoin_core_listdescriptors() {
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
        let blob = br#"
  {
    "label":"x","blockheight":800000,
    "descriptor":"wpkh(xpub...)#abcdefgh","devices":[]
  }
"#;
        assert!(SpecterParser::sniff(blob));
    }

    // -------------------------------------------------------------------------
    // Parse impl: happy path + error cases (Phase P2B)
    // -------------------------------------------------------------------------

    const MAINNET_FP_A: &str = "b8688df1";
    const MAINNET_XPUB_A: &str = "xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX";
    const MAINNET_FP_B: &str = "28645006";
    const MAINNET_XPUB_B: &str = "xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6";
    const MAINNET_FP_C: &str = "5436d724";
    const MAINNET_XPUB_C: &str = "xpub6Buxw9MmbkJr4iAw8SACNci2hQNuPCMwt9P7HkK62ZQAW9UcJaQ2bc6ARD892TToQQ9Rp6AHujHxBLXqAsvn5fRnLfnhKSRfz8qtaoyKUYx";
    const TESTNET_FP_A: &str = "704c7836";
    const TESTNET_XPUB_A: &str = "tpubDEgS9fUEpucKatmvKAv21v8nViHxR6rsV7ohMWK4YjsWd4EWT3w8YzMgMEvNrDfsUANbid74WRFpr3Gym8UHBSLnqg6b1Lzvibw87cLSctC";

    fn checksum(desc_without_hash: &str) -> String {
        let mut eng = miniscript::descriptor::checksum::Engine::new();
        eng.input(desc_without_hash).expect("ascii-only");
        eng.checksum()
    }

    fn build_specter_blob(label: &str, blockheight: u64, descriptor: &str, devices_json: &str) -> String {
        format!(
            "{{\n  \"label\": \"{label}\",\n  \"blockheight\": {blockheight},\n  \"descriptor\": \"{descriptor}\",\n  \"devices\": {devices_json}\n}}\n"
        )
    }

    #[test]
    fn parse_singlesig_p2wpkh_mainnet_happy_path() {
        let body = format!("wpkh([{MAINNET_FP_A}/84'/0'/0']{MAINNET_XPUB_A}/<0;1>/*)");
        let desc = format!("{}#{}", body, checksum(&body));
        let blob = build_specter_blob("daily", 800000, &desc, r#"[{"type":"coldcard","label":"primary"}]"#);
        let mut stderr = Vec::new();
        let parsed = SpecterParser::parse(blob.as_bytes(), &mut stderr).unwrap();
        assert_eq!(parsed.len(), 1);
        let p = &parsed[0];
        assert_eq!(p.cosigners.len(), 1);
        assert_eq!(p.network, bitcoin::Network::Bitcoin);
        assert_eq!(p.threshold, None);
        match &p.provenance {
            ImportProvenance::Specter(meta) => {
                assert_eq!(meta.label, "daily");
                assert_eq!(meta.blockheight, 800000);
                assert_eq!(meta.devices.len(), 1);
                assert_eq!(meta.devices[0].device_type, "coldcard");
                assert_eq!(meta.devices[0].label, "primary");
                assert!(meta.dropped_fields.is_empty());
            }
            other => panic!("expected ImportProvenance::Specter, got: {other:?}"),
        }
    }

    #[test]
    fn parse_multisig_2of3_wsh_sortedmulti_happy_path() {
        let body = format!(
            "wsh(sortedmulti(2,[{MAINNET_FP_A}/48'/0'/0'/2']{MAINNET_XPUB_A}/<0;1>/*,[{MAINNET_FP_B}/48'/0'/0'/2']{MAINNET_XPUB_B}/<0;1>/*,[{MAINNET_FP_C}/48'/0'/0'/2']{MAINNET_XPUB_C}/<0;1>/*))"
        );
        let desc = format!("{}#{}", body, checksum(&body));
        let blob = build_specter_blob(
            "vault",
            850000,
            &desc,
            r#"[{"type":"coldcard","label":"sig1"},{"type":"trezor","label":"sig2"},{"type":"ledger","label":"sig3"}]"#,
        );
        let mut stderr = Vec::new();
        let parsed = SpecterParser::parse(blob.as_bytes(), &mut stderr).unwrap();
        let p = &parsed[0];
        assert_eq!(p.cosigners.len(), 3);
        assert_eq!(p.threshold, Some(2));
        assert_eq!(p.network, bitcoin::Network::Bitcoin);
        match &p.provenance {
            ImportProvenance::Specter(meta) => {
                assert_eq!(meta.devices.len(), 3);
                assert_eq!(meta.devices[0].device_type, "coldcard");
                assert_eq!(meta.devices[1].device_type, "trezor");
                assert_eq!(meta.devices[2].device_type, "ledger");
            }
            other => panic!("expected Specter provenance, got: {other:?}"),
        }
    }

    #[test]
    fn parse_legacy_string_devices_array_normalizes_to_empty_labels() {
        let body = format!("wpkh([{MAINNET_FP_A}/84'/0'/0']{MAINNET_XPUB_A}/<0;1>/*)");
        let desc = format!("{}#{}", body, checksum(&body));
        let blob = build_specter_blob("daily", 0, &desc, r#"["unknown"]"#);
        let mut stderr = Vec::new();
        let parsed = SpecterParser::parse(blob.as_bytes(), &mut stderr).unwrap();
        match &parsed[0].provenance {
            ImportProvenance::Specter(meta) => {
                assert_eq!(meta.devices.len(), 1);
                assert_eq!(meta.devices[0].device_type, "unknown");
                assert_eq!(meta.devices[0].label, "");
            }
            other => panic!("expected Specter provenance, got: {other:?}"),
        }
    }

    #[test]
    fn parse_testnet_blob_infers_network_testnet() {
        let body = format!("wpkh([{TESTNET_FP_A}/84'/1'/0']{TESTNET_XPUB_A}/<0;1>/*)");
        let desc = format!("{}#{}", body, checksum(&body));
        let blob = build_specter_blob("test wallet", 200000, &desc, r#"[{"type":"coldcard","label":"signet device"}]"#);
        let mut stderr = Vec::new();
        let parsed = SpecterParser::parse(blob.as_bytes(), &mut stderr).unwrap();
        assert_eq!(parsed[0].network, bitcoin::Network::Testnet);
    }

    #[test]
    fn parse_dropped_unknown_top_level_fields() {
        let body = format!("wpkh([{MAINNET_FP_A}/84'/0'/0']{MAINNET_XPUB_A}/<0;1>/*)");
        let desc = format!("{}#{}", body, checksum(&body));
        let blob = format!(
            "{{\n  \"label\": \"daily\",\n  \"blockheight\": 0,\n  \"descriptor\": \"{desc}\",\n  \"devices\": [{{\"type\":\"coldcard\",\"label\":\"\"}}],\n  \"unknown_field_x\": \"x\",\n  \"unknown_field_y\": 42\n}}\n"
        );
        let mut stderr = Vec::new();
        let parsed = SpecterParser::parse(blob.as_bytes(), &mut stderr).unwrap();
        let stderr_text = String::from_utf8(stderr).unwrap();
        assert!(
            stderr_text.contains("dropped unrecognized top-level fields"),
            "expected dropped-fields NOTICE; got stderr: {stderr_text}"
        );
        match &parsed[0].provenance {
            ImportProvenance::Specter(meta) => {
                assert_eq!(meta.dropped_fields.len(), 2);
                assert!(meta.dropped_fields.contains(&"unknown_field_x".to_string()));
                assert!(meta.dropped_fields.contains(&"unknown_field_y".to_string()));
            }
            other => panic!("expected Specter provenance, got: {other:?}"),
        }
    }

    #[test]
    fn parse_xprv_descriptor_refused_via_xprv_forbidden() {
        // Substitute an xprv string for the xpub. Since we strip the
        // checksum before regex-matching the xprv prefix, we use a known
        // mainnet xprv prefix `xprv` + base58check tail to trigger the
        // refusal. The checksum is intentionally invalid because the xprv
        // refusal must fire BEFORE checksum validation.
        let desc = "wpkh([b8688df1/84'/0'/0']xprv9s21ZrQH143K3GJpoapnV8SFfukcVBSfeCficPSGfubmSFDxo1kuHnLisriDvSnRRuL2Qrg5ggqHKNVpxR86QEC8w35uxmGoggxtQTPvfUu/<0;1>/*)#invalidcs";
        let blob = build_specter_blob("danger", 0, desc, r#"[{"type":"coldcard","label":""}]"#);
        let mut stderr = Vec::new();
        let err = SpecterParser::parse(blob.as_bytes(), &mut stderr).unwrap_err();
        assert!(matches!(err, ToolkitError::ImportWalletXprvForbidden));
    }

    #[test]
    fn parse_invalid_checksum_rejected() {
        let body = format!("wpkh([{MAINNET_FP_A}/84'/0'/0']{MAINNET_XPUB_A}/<0;1>/*)");
        let desc = format!("{}#deadbeef", body);
        let blob = build_specter_blob("x", 0, &desc, r#"[{"type":"coldcard","label":""}]"#);
        let mut stderr = Vec::new();
        let err = SpecterParser::parse(blob.as_bytes(), &mut stderr).unwrap_err();
        match err {
            ToolkitError::ImportWalletParse(msg) => {
                assert!(
                    msg.contains("BIP-380 checksum validation failed"),
                    "expected checksum error; got: {msg}"
                );
            }
            other => panic!("expected ImportWalletParse, got: {other:?}"),
        }
    }

    #[test]
    fn parse_missing_descriptor_field_rejected() {
        // `descriptor` absent → parse error (sniff also rejects, but the
        // explicit `--format specter` path bypasses sniff and lands here).
        let blob = r#"{"label":"x","blockheight":0,"devices":[]}"#;
        let mut stderr = Vec::new();
        let err = SpecterParser::parse(blob.as_bytes(), &mut stderr).unwrap_err();
        match err {
            ToolkitError::ImportWalletParse(msg) => {
                assert!(
                    msg.contains("missing or non-string `descriptor`"),
                    "got: {msg}"
                );
            }
            other => panic!("expected ImportWalletParse, got: {other:?}"),
        }
    }

    #[test]
    fn parse_negative_blockheight_rejected() {
        let body = format!("wpkh([{MAINNET_FP_A}/84'/0'/0']{MAINNET_XPUB_A}/<0;1>/*)");
        let desc = format!("{}#{}", body, checksum(&body));
        let blob = format!(
            "{{\"label\":\"x\",\"blockheight\":-1,\"descriptor\":\"{desc}\",\"devices\":[{{\"type\":\"coldcard\",\"label\":\"\"}}]}}"
        );
        let mut stderr = Vec::new();
        let err = SpecterParser::parse(blob.as_bytes(), &mut stderr).unwrap_err();
        match err {
            ToolkitError::ImportWalletParse(msg) => {
                assert!(msg.contains("negative `blockheight`"), "got: {msg}");
            }
            other => panic!("expected ImportWalletParse, got: {other:?}"),
        }
    }

    #[test]
    fn parse_devices_length_mismatch_emits_notice_and_normalizes() {
        // Multisig 2-of-3 descriptor but only 1 device entry — provenance
        // devices vector pads to cosigner count.
        let body = format!(
            "wsh(sortedmulti(2,[{MAINNET_FP_A}/48'/0'/0'/2']{MAINNET_XPUB_A}/<0;1>/*,[{MAINNET_FP_B}/48'/0'/0'/2']{MAINNET_XPUB_B}/<0;1>/*,[{MAINNET_FP_C}/48'/0'/0'/2']{MAINNET_XPUB_C}/<0;1>/*))"
        );
        let desc = format!("{}#{}", body, checksum(&body));
        let blob = build_specter_blob("partial", 0, &desc, r#"[{"type":"coldcard","label":"only"}]"#);
        let mut stderr = Vec::new();
        let parsed = SpecterParser::parse(blob.as_bytes(), &mut stderr).unwrap();
        let stderr_text = String::from_utf8(stderr).unwrap();
        assert!(
            stderr_text.contains("devices array length 1 differs from cosigner count 3"),
            "expected length-mismatch NOTICE; got stderr: {stderr_text}"
        );
        match &parsed[0].provenance {
            ImportProvenance::Specter(meta) => {
                assert_eq!(meta.devices.len(), 3);
                assert_eq!(meta.devices[0].device_type, "coldcard"); // preserved
                assert_eq!(meta.devices[1].device_type, "unknown"); // padded
                assert_eq!(meta.devices[2].device_type, "unknown"); // padded
            }
            other => panic!("expected Specter provenance, got: {other:?}"),
        }
    }

    #[test]
    fn parse_devices_object_missing_type_rejected() {
        let body = format!("wpkh([{MAINNET_FP_A}/84'/0'/0']{MAINNET_XPUB_A}/<0;1>/*)");
        let desc = format!("{}#{}", body, checksum(&body));
        let blob = build_specter_blob("x", 0, &desc, r#"[{"label":"only label, no type"}]"#);
        let mut stderr = Vec::new();
        let err = SpecterParser::parse(blob.as_bytes(), &mut stderr).unwrap_err();
        match err {
            ToolkitError::ImportWalletParse(msg) => {
                assert!(msg.contains("devices[0]: missing or non-string `type`"), "got: {msg}");
            }
            other => panic!("expected ImportWalletParse, got: {other:?}"),
        }
    }

    #[test]
    fn parse_invalid_json_rejected() {
        let blob = br#"{not even close to JSON"#;
        let mut stderr = Vec::new();
        let err = SpecterParser::parse(blob, &mut stderr).unwrap_err();
        match err {
            ToolkitError::ImportWalletParse(msg) => {
                assert!(msg.contains("invalid JSON"), "got: {msg}");
            }
            other => panic!("expected ImportWalletParse, got: {other:?}"),
        }
    }

    #[test]
    fn parse_threshold_u8_overflow_typed_error() {
        let r = extract_threshold("wpkh(@0)").unwrap();
        assert_eq!(r, None);

        let r = extract_threshold("wsh(sortedmulti(3,@0,@1,@2,@3))").unwrap();
        assert_eq!(r, Some(3));

        let err = extract_threshold("wsh(sortedmulti(256,@0,@1))").unwrap_err();
        let msg = format!("{err:?}");
        assert!(
            msg.contains("exceeds u8 range") && msg.contains("256"),
            "got: {msg}"
        );
    }

    // -------------------------------------------------------------------------
    // Fixture-based smokes: 4 fixtures land at tests/fixtures/wallet_import/.
    // -------------------------------------------------------------------------

    fn read_fixture(name: &str) -> Vec<u8> {
        let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("tests");
        path.push("fixtures");
        path.push("wallet_import");
        path.push(name);
        std::fs::read(&path).unwrap_or_else(|e| panic!("read fixture {}: {e}", path.display()))
    }

    #[test]
    fn fixture_specter_singlesig_p2wpkh_coldcard_parses() {
        let blob = read_fixture("specter-singlesig-p2wpkh-coldcard.json");
        let mut stderr = Vec::new();
        let parsed = SpecterParser::parse(&blob, &mut stderr).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].cosigners.len(), 1);
        assert_eq!(parsed[0].network, bitcoin::Network::Bitcoin);
    }

    #[test]
    fn fixture_specter_multisig_2of3_wsh_sortedmulti_parses() {
        let blob = read_fixture("specter-multisig-2of3-wsh-sortedmulti.json");
        let mut stderr = Vec::new();
        let parsed = SpecterParser::parse(&blob, &mut stderr).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].cosigners.len(), 3);
        assert_eq!(parsed[0].threshold, Some(2));
    }

    #[test]
    fn fixture_specter_descriptor_with_checksum_parses() {
        let blob = read_fixture("specter-with-checksum.json");
        let mut stderr = Vec::new();
        let parsed = SpecterParser::parse(&blob, &mut stderr).unwrap();
        assert_eq!(parsed.len(), 1);
    }

    #[test]
    fn fixture_specter_blockheight_zero_parses() {
        let blob = read_fixture("specter-blockheight-zero.json");
        let mut stderr = Vec::new();
        let parsed = SpecterParser::parse(&blob, &mut stderr).unwrap();
        match &parsed[0].provenance {
            ImportProvenance::Specter(meta) => {
                assert_eq!(meta.blockheight, 0);
            }
            other => panic!("expected Specter provenance, got: {other:?}"),
        }
    }
}
