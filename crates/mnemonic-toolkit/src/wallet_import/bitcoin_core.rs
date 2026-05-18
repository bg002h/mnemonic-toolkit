//! Bitcoin Core `listdescriptors` parser.
//!
//! Per `design/SPEC_wallet_import_v0_26_0.md` §5. Accepts the JSON shape:
//!
//! ```json
//! {
//!   "wallet_name": "<name>",
//!   "descriptors": [
//!     {
//!       "desc": "<descriptor>#<checksum>",
//!       "timestamp": <int|"now">,
//!       "active": <bool>,
//!       "internal": <bool>,
//!       "range": [<int>, <int>],
//!       "next": <int>,
//!       "next_index": <int>
//!     }, ...
//!   ]
//! }
//! ```
//!
//! Each `descriptors[i]` is parsed via the same adapter + `parse_descriptor`
//! pipeline as BSMS (`pipeline::concrete_keys_to_placeholders` →
//! `parse_descriptor::parse_descriptor`). Per-entry metadata (`active`,
//! `internal`, `range`) is preserved in `ParsedImport.source_metadata`;
//! wallet-state fields (`timestamp`, `next`, `next_index`) are dropped from
//! the bundle output with a single stderr NOTICE per SPEC §2.4.
//!
//! Per SPEC §5.2 step 2.a: `desc` containing the literal substring `xprv`
//! is refused with `ImportWalletXprvForbidden` (exit 2) — Bitcoin Core's
//! `listdescriptors true` form returns xprv-bearing entries that the
//! toolkit must not consume.
//!
//! Network detection mirrors BSMS (§4.2 step 8 = §7.0.a locked): inspect
//! the BIP-48 coin-type child number on the FIRST cosigner's origin path.
//! Per-entry coin-type heterogeneity within a single `desc` body is rejected
//! (same rule as BSMS); cross-entry coin-type heterogeneity (e.g.,
//! descriptors[0] mainnet, descriptors[1] testnet) is NOT enforced at the
//! parser level — each `ParsedImport` carries its own `network` field per
//! SPEC §8.1, and the CLI dispatch may emit per-bundle network metadata.

use super::{
    pipeline::concrete_keys_to_placeholders, validate_watch_only_resolved, BsmsAuditFields,
    CoreSourceMetadata, ParsedImport, WalletFormatParser,
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

pub(crate) struct BitcoinCoreParser;

/// Vendor-marker keys that ALSO appear at the top level of competing wallet
/// vendor blobs (Specter, Sparrow, etc.). Their presence overrides any Core
/// match in `sniff` — keeps `sniff` conservative per SPEC §6.1.2 lock.
#[allow(dead_code)] // consumed by `sniff`; the trait method is `#[allow(dead_code)]`-gated until Phase 5 wires the dispatcher.
const VENDOR_MARKER_KEYS: &[&str] = &["chain", "policy", "version", "bipname", "extendedPublicKey"];

impl WalletFormatParser for BitcoinCoreParser {
    fn sniff(blob: &[u8]) -> bool {
        // SPEC §6.1 item 2:
        // 1. Trimmed-leading-whitespace starts with `{`.
        // 2. `serde_json::from_slice::<Value>` succeeds.
        // 3. Top-level value is an object with a `descriptors` key whose
        //    value is a non-empty array.
        // 4. Each `descriptors[i]` is an object with a `desc: String` field.
        // 5. No vendor-specific marker keys present at top level.
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
        // Conservative absence-check against competing vendor markers.
        for marker in VENDOR_MARKER_KEYS {
            if obj.contains_key(*marker) {
                return false;
            }
        }
        let descriptors = match obj.get("descriptors").and_then(|v| v.as_array()) {
            Some(a) => a,
            None => return false,
        };
        if descriptors.is_empty() {
            return false;
        }
        // Every entry must be an object with a `desc: String`.
        descriptors.iter().all(|entry| {
            entry
                .as_object()
                .and_then(|o| o.get("desc"))
                .and_then(|d| d.as_str())
                .is_some()
        })
    }

    fn parse(blob: &[u8], stderr: &mut dyn Write) -> Result<Vec<ParsedImport>, ToolkitError> {
        // SPEC §5.2 step 1: JSON-parse.
        let value: Value = serde_json::from_slice(blob).map_err(|e| {
            ToolkitError::ImportWalletParse(format!(
                "import-wallet: bitcoin-core: parse error: invalid JSON: {e}"
            ))
        })?;
        let obj = value.as_object().ok_or_else(|| {
            ToolkitError::ImportWalletParse(
                "import-wallet: bitcoin-core: parse error: top-level JSON value is not an object"
                    .to_string(),
            )
        })?;
        let descriptors = obj
            .get("descriptors")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                ToolkitError::ImportWalletParse(
                    "import-wallet: bitcoin-core: parse error: missing or non-array top-level `descriptors` key"
                        .to_string(),
                )
            })?;
        if descriptors.is_empty() {
            return Err(ToolkitError::ImportWalletParse(
                "import-wallet: bitcoin-core: parse error: top-level `descriptors` array is empty; no bundles to emit"
                    .to_string(),
            ));
        }

        // SPEC §5.2 step 2.d: aggregate dropped-field names across all entries
        // and emit ONE stderr NOTICE if any are present (avoids N notices for
        // an N-entry blob; the field-set is uniform per Core output anyway).
        let mut aggregate_dropped: Vec<&'static str> = Vec::new();
        for entry in descriptors {
            let eobj = entry.as_object().ok_or_else(|| {
                ToolkitError::ImportWalletParse(
                    "import-wallet: bitcoin-core: parse error: descriptors[i] is not an object"
                        .to_string(),
                )
            })?;
            for f in ["timestamp", "next", "next_index"] {
                if eobj.contains_key(f) && !aggregate_dropped.contains(&f) {
                    aggregate_dropped.push(f);
                }
            }
        }
        if !aggregate_dropped.is_empty() {
            writeln!(
                stderr,
                "notice: import-wallet: bitcoin-core: dropped wallet-state fields {:?}: not preserved in bundle output (key-state only)",
                aggregate_dropped
            )
            .map_err(ToolkitError::Io)?;
        }

        // SPEC §5.2 step 2: per-entry parse loop.
        let mut out: Vec<ParsedImport> = Vec::with_capacity(descriptors.len());
        for (i, entry) in descriptors.iter().enumerate() {
            out.push(parse_entry(i, entry)?);
        }
        Ok(out)
    }
}

fn parse_entry(idx: usize, entry: &Value) -> Result<ParsedImport, ToolkitError> {
    let eobj = entry.as_object().ok_or_else(|| {
        ToolkitError::ImportWalletParse(format!(
            "import-wallet: bitcoin-core: parse error: descriptors[{idx}] is not an object"
        ))
    })?;

    let desc_with_csum = eobj
        .get("desc")
        .and_then(|d| d.as_str())
        .ok_or_else(|| {
            ToolkitError::ImportWalletParse(format!(
                "import-wallet: bitcoin-core: parse error: descriptors[{idx}].desc is missing or not a string"
            ))
        })?;

    // SPEC §5.2 step 2.a: refuse xprv-bearing descriptors. Substring match is
    // sufficient — xprv... base58 strings cannot collide with the rest of a
    // legitimate xpub descriptor body (xpub-prefix variants are `xpub|tpub|
    // ypub|Ypub|zpub|Zpub|upub|Upub|vpub|Vpub`; none contain `xprv`).
    if desc_with_csum.contains("xprv") {
        return Err(ToolkitError::ImportWalletXprvForbidden);
    }

    // SPEC §5.2 step 2.b: same adapter + parse_descriptor pipeline as BSMS.
    // Validate the BIP-380 checksum up-front via miniscript so a bad
    // checksum surfaces as ImportWalletParse rather than a downstream
    // DescriptorParse (consistent with BSMS error template).
    let descriptor_body_no_csum = miniscript::descriptor::checksum::verify_checksum(
        desc_with_csum,
    )
    .map_err(|e| {
        ToolkitError::ImportWalletParse(format!(
            "import-wallet: bitcoin-core: parse error: descriptors[{idx}]: BIP-380 checksum validation failed: {e}"
        ))
    })?;

    let (placeholder_form, parsed_keys, parsed_fingerprints) =
        concrete_keys_to_placeholders(descriptor_body_no_csum).map_err(|e| {
            // Re-tag the BSMS error template prefix as bitcoin-core for the
            // user-facing message.
            ToolkitError::ImportWalletParse(e.message().replacen(
                "import-wallet: bsms:",
                "import-wallet: bitcoin-core:",
                1,
            ))
        })?;

    let descriptor =
        parse_descriptor::parse_descriptor(&placeholder_form, &parsed_keys, &parsed_fingerprints)
            .map_err(|e| {
            ToolkitError::ImportWalletParse(format!(
                "import-wallet: bitcoin-core: parse error: descriptors[{idx}]: {}",
                e.message()
            ))
        })?;

    let origins = extract_origin_components(descriptor_body_no_csum)?;
    let network = network_from_origins(&origins, idx)?;

    let mut cosigners: Vec<ResolvedSlot> = Vec::with_capacity(parsed_keys.len());
    for (slot_idx, _) in parsed_keys.iter().enumerate() {
        let (xpub, fp, path, path_raw) = build_slot_fields(descriptor_body_no_csum, slot_idx, idx)?;
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

    let threshold = extract_threshold(descriptor_body_no_csum);

    let active = eobj
        .get("active")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let internal = eobj
        .get("internal")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let range = parse_range_field(eobj.get("range"))?;

    let mut dropped_fields: Vec<String> = Vec::new();
    for f in ["timestamp", "next", "next_index"] {
        if eobj.contains_key(f) {
            dropped_fields.push(f.to_string());
        }
    }

    let source_metadata = Some(CoreSourceMetadata {
        active,
        internal,
        range,
        dropped_fields,
    });

    Ok(ParsedImport {
        descriptor,
        cosigners,
        network,
        threshold,
        bsms_audit: None::<BsmsAuditFields>,
        source_metadata,
    })
}

/// Decode the optional `range` field — Bitcoin Core emits a 2-element integer
/// array `[lo, hi]`. Returns `Ok(None)` if absent (Core may omit `range` for
/// non-ranged descriptors); errors if the shape is unexpected.
fn parse_range_field(v: Option<&Value>) -> Result<Option<(u64, u64)>, ToolkitError> {
    let v = match v {
        Some(v) => v,
        None => return Ok(None),
    };
    if v.is_null() {
        return Ok(None);
    }
    let arr = v.as_array().ok_or_else(|| {
        ToolkitError::ImportWalletParse(
            "import-wallet: bitcoin-core: parse error: `range` must be a [lo, hi] array"
                .to_string(),
        )
    })?;
    if arr.len() != 2 {
        return Err(ToolkitError::ImportWalletParse(format!(
            "import-wallet: bitcoin-core: parse error: `range` must have exactly 2 elements, got {}",
            arr.len()
        )));
    }
    let lo = arr[0].as_u64().ok_or_else(|| {
        ToolkitError::ImportWalletParse(
            "import-wallet: bitcoin-core: parse error: `range[0]` must be a non-negative integer"
                .to_string(),
        )
    })?;
    let hi = arr[1].as_u64().ok_or_else(|| {
        ToolkitError::ImportWalletParse(
            "import-wallet: bitcoin-core: parse error: `range[1]` must be a non-negative integer"
                .to_string(),
        )
    })?;
    Ok(Some((lo, hi)))
}

/// Per-cosigner origin tuple lifted out of the descriptor body via a shared
/// `[fp/path]xpub` regex. Returned in declaration order. Mirrors
/// `bsms::extract_origin_components` (kept separate so the error-message
/// prefix carries the correct format tag).
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
                    "import-wallet: bitcoin-core: parse error: fingerprint hex: {e}"
                ))
            })?;
        }
        let fp = Fingerprint::from(fp_bytes);
        let path = DerivationPath::from_str(&format!("m{path_raw_inner}")).map_err(|e| {
            ToolkitError::ImportWalletParse(format!(
                "import-wallet: bitcoin-core: parse error: derivation-path parse: {e}"
            ))
        })?;
        let path_raw = format!("[{fp_hex}{path_raw_inner}]");
        out.push((fp, path, path_raw, xpub_str.to_string()));
    }
    if out.is_empty() {
        return Err(ToolkitError::ImportWalletParse(
            "import-wallet: bitcoin-core: parse error: no origin annotations in descriptor"
                .to_string(),
        ));
    }
    Ok(out)
}

fn build_slot_fields(
    descriptor_body: &str,
    slot_idx: usize,
    entry_idx: usize,
) -> Result<(Xpub, Fingerprint, DerivationPath, String), ToolkitError> {
    let origins = extract_origin_components(descriptor_body)?;
    let (fp, path, path_raw, xpub_str) = origins.into_iter().nth(slot_idx).ok_or_else(|| {
        ToolkitError::ImportWalletParse(format!(
            "import-wallet: bitcoin-core: parse error: descriptors[{entry_idx}]: slot index {slot_idx} out of range"
        ))
    })?;
    let (neutral, _variant) = crate::slip0132::normalize_xpub_prefix(&xpub_str)?;
    let xpub = Xpub::from_str(&neutral).map_err(|e| {
        ToolkitError::ImportWalletParse(format!(
            "import-wallet: bitcoin-core: parse error: descriptors[{entry_idx}]: xpub decode for slot {slot_idx}: {e}"
        ))
    })?;
    Ok((xpub, fp, path, path_raw))
}

fn network_from_origins(
    origins: &[(Fingerprint, DerivationPath, String, String)],
    entry_idx: usize,
) -> Result<bitcoin::Network, ToolkitError> {
    if origins.is_empty() {
        return Err(ToolkitError::ImportWalletParse(format!(
            "import-wallet: bitcoin-core: parse error: descriptors[{entry_idx}]: no origins to infer network from"
        )));
    }
    let coin_types: Vec<u32> = origins
        .iter()
        .map(|(_, p, _, _)| coin_type_from_path(p, entry_idx))
        .collect::<Result<Vec<_>, _>>()?;
    let first = coin_types[0];
    for (i, ct) in coin_types.iter().enumerate().skip(1) {
        if *ct != first {
            return Err(ToolkitError::ImportWalletParse(format!(
                "import-wallet: bitcoin-core: descriptors[{entry_idx}]: cosigner {i} has coin-type {ct}, cosigner 0 has coin-type {first}; all cosigners must share a coin-type"
            )));
        }
    }
    match first {
        0 => Ok(bitcoin::Network::Bitcoin),
        1 => Ok(bitcoin::Network::Testnet),
        other => Err(ToolkitError::ImportWalletParse(format!(
            "import-wallet: bitcoin-core: parse error: descriptors[{entry_idx}]: unsupported coin-type {other} on origin path; only 0 (mainnet) and 1 (testnet) supported per BIP-48"
        ))),
    }
}

fn coin_type_from_path(path: &DerivationPath, entry_idx: usize) -> Result<u32, ToolkitError> {
    let comps: Vec<&ChildNumber> = path.into_iter().collect();
    if comps.len() < 2 {
        return Err(ToolkitError::ImportWalletParse(format!(
            "import-wallet: bitcoin-core: parse error: descriptors[{entry_idx}]: origin path has only {} components; need ≥2 for BIP-48 coin-type inference",
            comps.len()
        )));
    }
    match comps[1] {
        ChildNumber::Hardened { index } => Ok(*index),
        ChildNumber::Normal { index } => Err(ToolkitError::ImportWalletParse(format!(
            "import-wallet: bitcoin-core: parse error: descriptors[{entry_idx}]: coin-type component {index} is not hardened; BIP-48 requires `<coin_type>'`"
        ))),
    }
}

/// Extract K from `thresh(K, ...)` / `multi(K, ...)` / `sortedmulti(K, ...)`
/// at the top-level miniscript context. Returns `None` for single-key shapes.
/// Mirrors `bsms::extract_threshold`.
fn extract_threshold(descriptor_body: &str) -> Option<u8> {
    static R: OnceLock<Regex> = OnceLock::new();
    let re = R.get_or_init(|| {
        Regex::new(r"(?:thresh|multi|sortedmulti)\((\d+)\s*,").expect("threshold regex is fixed")
    });
    re.captures(descriptor_body)
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse::<u8>().ok())
}

fn origin_capture_regex() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"\[([0-9a-fA-F]{8})((?:/\d+'?)+)\]([xtyzuvYZUV]pub[A-HJ-NP-Za-km-z1-9]+)")
            .expect("origin_capture_regex is a fixed string literal")
    })
}

#[allow(dead_code)] // consumed by `sniff`; same Phase 5 wiring rationale as VENDOR_MARKER_KEYS.
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

    /// SPEC §6.1 item 2: sniff predicate smoke. Pins behavior for the cases
    /// where sniff must return true vs. false. Used by Phase 5's sniff
    /// dispatcher.
    #[test]
    fn sniff_true_on_minimal_core_blob() {
        let blob = br#"{"wallet_name":"x","descriptors":[{"desc":"wpkh(xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX/<0;1>/*)#00000000"}]}"#;
        assert!(BitcoinCoreParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_bsms_blob() {
        let blob = b"BSMS 1.0\nwsh(pk(deadbeef))#00000000\n";
        assert!(!BitcoinCoreParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_specter_blob() {
        // Top-level `chain` is a Specter vendor-marker key per VENDOR_MARKER_KEYS.
        let blob = br#"{"chain":"main","descriptor":"wpkh(xpub...)","label":"daily","devices":["unknown"]}"#;
        assert!(!BitcoinCoreParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_empty_descriptors_array() {
        let blob = br#"{"descriptors":[]}"#;
        assert!(!BitcoinCoreParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_non_object_top_level() {
        assert!(!BitcoinCoreParser::sniff(b"[1, 2, 3]"));
    }

    #[test]
    fn sniff_false_on_entry_missing_desc() {
        let blob = br#"{"descriptors":[{"timestamp":42}]}"#;
        assert!(!BitcoinCoreParser::sniff(blob));
    }
}
