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
//! `internal`, `range`) is preserved via `ParsedImport::source_metadata()`
//! accessor; backed by `ImportProvenance::BitcoinCore(...)`;
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
    pipeline::concrete_keys_to_placeholders, validate_watch_only_resolved, CoreSourceMetadata,
    ImportProvenance, ParsedImport, WalletFormatParser,
};
use crate::error::ToolkitError;
use crate::parse_descriptor;
use crate::synthesize::{xpub_to_65, ResolvedSlot};
use bitcoin::bip32::{ChildNumber, DerivationPath, Fingerprint, Xpub};
use regex::Regex;
use serde_json::Value;
use std::io::Write;
use std::sync::OnceLock;

pub(crate) struct BitcoinCoreParser;

/// Vendor-marker keys that ALSO appear at the top level of competing wallet
/// vendor blobs (Specter, Sparrow, Coldcard, Jade, Electrum, etc.). Their
/// presence at top level overrides any Core match in `sniff` — keeps `sniff`
/// conservative per `SPEC_wallet_import_v0_26_0.md` §6.1.2 lock and the
/// v0.28.0 amendment at `SPEC_wallet_import_v0_28_0.md` §6.1.1 (Q4 lock).
///
/// v0.28.0 P0A additions absorb markers for Phases P1-P6 parsers:
/// - `seed_version`, `wallet_type` — Electrum wallet (SPEC §11.6)
/// - `policyType`, `defaultPolicy`, `keystores` — Sparrow Wallet (SPEC §11.1)
/// - `devices`, `blockheight` — Specter (SPEC §11.2; `label` deliberately
///   omitted per R0 I3 fold — Specter positive sniff uses `blockheight` +
///   `devices` + `descriptor` + `label`, but `label` is generic enough that
///   a legitimate Core blob carrying a top-level `label` key should not be
///   excluded; Specter is still strongly disambiguated by `blockheight`)
/// - `multisig_file` — Blockstream Jade (SPEC §11.5; the top-level reply
///   field of Jade's `get_registered_multisig` RPC. R0 I4 fold removed
///   `register_multisig` from this list — that's the RPC command name,
///   not an on-disk JSON field, verified via Blockstream/Jade docs)
///
/// Note: Coldcard generic-JSON (`chain`, `xfp`, `bipN`) is already covered
/// by the `chain` exclusion (v0.26.0 original); ColdcardMultisig is a text
/// format (NOT JSON) and never reaches this JSON-sniff path.
const VENDOR_MARKER_KEYS: &[&str] = &[
    // v0.26.0 originals (Bitcoin Core / generic-vendor exclusion):
    "chain",
    "policy",
    "version",
    "bipname",
    "extendedPublicKey",
    // v0.28.0 P0A additions (per-format vendor markers; R1 fold):
    "seed_version",
    "wallet_type",
    "policyType",
    "defaultPolicy",
    "keystores",
    "devices",
    "blockheight",
    "multisig_file",
];

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
        // SPEC §5.1 + Phase 3 R0 I2 fold: extract `wallet_name` from envelope
        // (metadata-only; preserved for Phase 4 canonicalize + Phase 5 --json
        // envelope). Absent / non-string → None.
        let wallet_name = obj
            .get("wallet_name")
            .and_then(|v| v.as_str())
            .map(str::to_string);

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
        // Phase 3 R0 M2 fold: join(", ") instead of {:?} Debug for clean
        // user-facing stderr (no brackets/double-quotes).
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
                "notice: import-wallet: bitcoin-core: dropped wallet-state fields {}: not preserved in bundle output (key-state only)",
                aggregate_dropped.join(", ")
            )
            .map_err(ToolkitError::Io)?;
        }

        // SPEC §5.2 step 2: per-entry parse loop.
        //
        // `internal` provenance (SPEC_bitcoin_core_receive_change_pair_merge.md
        // §5) is now threaded EXPLICITLY per entry rather than read inside
        // `parse_entry` itself: a passthrough entry (this loop, P0) always
        // carries `Some(parse_bool_field(eobj, "internal")?)`; only the P1
        // merge pre-pass's synthesized entries carry `None`.
        let mut out: Vec<ParsedImport> = Vec::with_capacity(descriptors.len());
        for (i, entry) in descriptors.iter().enumerate() {
            let eobj = entry.as_object().ok_or_else(|| {
                ToolkitError::ImportWalletParse(format!(
                    "import-wallet: bitcoin-core: parse error: descriptors[{i}] is not an object"
                ))
            })?;
            let internal = Some(parse_bool_field(eobj, "internal")?);
            out.push(parse_entry(i, entry, wallet_name.clone(), internal)?);
        }
        Ok(out)
    }
}

fn parse_entry(
    idx: usize,
    entry: &Value,
    wallet_name: Option<String>,
    internal: Option<bool>,
) -> Result<ParsedImport, ToolkitError> {
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

    // SPEC §5.2 step 2.a (Phase 3 R0 architect C1+I1 folds): refuse any
    // extended-private-key prefix, not just literal "xprv". Bitcoin Core's
    // `listdescriptors true` on testnet/signet/regtest emits `tprv`; SLIP-132
    // defines `yprv|Yprv|zprv|Zprv|uprv|Uprv|vprv|Vprv` private-key prefix
    // variants. None were caught by the prior `contains("xprv")` check.
    // Strip the BIP-380 `#<csum>` trailer before the substring scan so the
    // checksum's bech32-style alphabet (which can contain the 4-char run
    // `xprv` stochastically at probability ~5e-6 per descriptor) cannot
    // false-positive a benign xpub descriptor.
    let body_for_xprv_check = match desc_with_csum.rsplit_once('#') {
        Some((body, _csum)) => body,
        None => desc_with_csum,
    };
    if xprv_prefix_regex().is_match(body_for_xprv_check) {
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

    let origins = crate::wallet_import::pipeline::extract_origin_components(
        descriptor_body_no_csum,
        "bitcoin-core",
    )?;
    let network = network_from_origins(&origins, idx)?;

    let mut cosigners: Vec<ResolvedSlot> = Vec::with_capacity(parsed_keys.len());
    for (slot_idx, _) in parsed_keys.iter().enumerate() {
        let (xpub, fp, path) = build_slot_fields(descriptor_body_no_csum, slot_idx, idx)?;
        debug_assert_eq!(xpub_to_65(&xpub), parsed_keys[slot_idx].payload);
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

    // cycle-5 S-NET (axis 2 / H15): per-entry, each decoded xpub's NetworkKind
    // must agree with this entry's coin-type-derived network.
    crate::wallet_import::pipeline::assert_slots_network_agrees(
        &cosigners,
        network,
        "import: bitcoin-core",
    )?;

    let threshold = extract_threshold(descriptor_body_no_csum)?;

    // v0.27.1 Phase 2 I4 fold: distinguish "absent" (default false) from
    // "shape-wrong" (typed parse error). The prior pattern
    // `.and_then(.as_bool).unwrap_or(false)` silently flipped non-bool inputs
    // ("active": "true", `1`, etc.) to false, which downstream
    // `--select-descriptor active-*` reported as "no active-* descriptor
    // found" — a misleading user-facing error. Mirrors `parse_range_field`'s
    // shape-strictness precedent.
    let active = parse_bool_field(eobj, "active")?;
    // `internal` is now threaded explicitly by the caller (see `parse`'s loop
    // and, from P1, `merge_receive_change_pairs`) rather than read here —
    // `Some(bool)` for a passthrough entry, `None` for a pre-pass-merged
    // entry. NEVER inferred from the multipath shape of `desc` itself.
    let range = parse_range_field(eobj.get("range"))?;

    let mut dropped_fields: Vec<String> = Vec::new();
    for f in ["timestamp", "next", "next_index"] {
        if eobj.contains_key(f) {
            dropped_fields.push(f.to_string());
        }
    }

    let source_metadata = CoreSourceMetadata {
        active,
        internal,
        range,
        dropped_fields,
        wallet_name,
    };

    Ok(ParsedImport {
        descriptor,
        original_descriptor: desc_with_csum.to_string(),
        cosigners,
        network,
        threshold,
        provenance: ImportProvenance::BitcoinCore(source_metadata),
    })
}

/// Decode the optional `range` field — Bitcoin Core emits a 2-element integer
/// array `[lo, hi]`. Returns `Ok(None)` if absent (Core may omit `range` for
/// non-ranged descriptors); errors if the shape is unexpected.
/// v0.27.1 Phase 2 I4 helper. Mirrors `parse_range_field`'s shape-strictness:
/// absent or `null` → `Ok(false)` (default); present + non-bool → `Err` with
/// pointer text naming the field.
fn parse_bool_field(
    eobj: &serde_json::Map<String, Value>,
    field: &str,
) -> Result<bool, ToolkitError> {
    match eobj.get(field) {
        None => Ok(false),
        Some(Value::Null) => Ok(false),
        Some(Value::Bool(b)) => Ok(*b),
        Some(other) => Err(ToolkitError::ImportWalletParse(format!(
            "import-wallet: bitcoin-core: parse error: `{field}` must be boolean, got {}",
            kind_of(other)
        ))),
    }
}

/// Compact JSON type label used by `parse_bool_field` error templates.
fn kind_of(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

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

fn build_slot_fields(
    descriptor_body: &str,
    slot_idx: usize,
    entry_idx: usize,
) -> Result<(Xpub, Fingerprint, DerivationPath), ToolkitError> {
    let origins =
        crate::wallet_import::pipeline::extract_origin_components(descriptor_body, "bitcoin-core")?;
    let (fp, path, xpub_str) = origins.into_iter().nth(slot_idx).ok_or_else(|| {
        ToolkitError::ImportWalletParse(format!(
            "import-wallet: bitcoin-core: parse error: descriptors[{entry_idx}]: slot index {slot_idx} out of range"
        ))
    })?;
    crate::wallet_import::pipeline::finalize_slot_fields(fp, path, &xpub_str, "bitcoin-core")
}

fn network_from_origins(
    origins: &[(Fingerprint, DerivationPath, String)],
    entry_idx: usize,
) -> Result<bitcoin::Network, ToolkitError> {
    if origins.is_empty() {
        return Err(ToolkitError::ImportWalletParse(format!(
            "import-wallet: bitcoin-core: parse error: descriptors[{entry_idx}]: no origins to infer network from"
        )));
    }
    let coin_types: Vec<u32> = origins
        .iter()
        .map(|(_, p, _)| coin_type_from_path(p, entry_idx))
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
/// at the top-level miniscript context. Returns `Ok(None)` for single-key
/// shapes (no thresh/multi token found); `Err` for u8 overflow.
///
/// v0.27.1 Phase 2 I6 fold: previously returned `Option<u8>`, silently
/// mapping u8 overflow (e.g. `thresh(256, …)`) to `None` — which downstream
/// rendered as `"threshold": null`, presenting a "no-threshold" descriptor
/// when the input was actually malformed. Now distinguishes "no thresh token"
/// from "thresh argument failed u8 parse" via the typed Result.
///
/// Mirrors `bsms::extract_threshold`.
pub(super) fn extract_threshold(descriptor_body: &str) -> Result<Option<u8>, ToolkitError> {
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
            "import-wallet: bitcoin-core: parse error: thresh/multi argument `{arg}` exceeds u8 range (>255 cosigners not supported): {e}"
        ))
    })
}

/// Match any extended-private-key prefix per BIP-32 + SLIP-132 (Phase 3 R0
/// architect C1 fold). Mainnet `xprv`, testnet `tprv`, SLIP-132
/// `yprv|Yprv|zprv|Zprv|uprv|Uprv|vprv|Vprv`. The trailing
/// `[A-HJ-NP-Za-km-z1-9]+` ensures we match an actual base58check key body
/// rather than the literal 4-char prefix substring (BIP-380 checksum
/// false-positive guard, I1 fold).
fn xprv_prefix_regex() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"[xtyzuvYZUV]prv[A-HJ-NP-Za-km-z1-9]+")
            .expect("xprv_prefix_regex is a fixed string literal")
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

    /// v0.27.1 Phase 2 R0 M1 fold: guarantee coverage of the
    /// `extract_threshold` u8-overflow branch. Mirrors `bsms::tests` cell.
    #[test]
    fn extract_threshold_u8_overflow_is_typed_error() {
        // Body without thresh/multi → Ok(None).
        let r = extract_threshold("wpkh(@0)").unwrap();
        assert_eq!(r, None);

        // Body with multi(2,…) → Ok(Some(2)).
        let r = extract_threshold("sh(multi(2,@0,@1,@2))").unwrap();
        assert_eq!(r, Some(2));

        // Body with sortedmulti(256,…) → Err (u8 overflow).
        let err = extract_threshold("wsh(sortedmulti(256,@0,@1))").unwrap_err();
        let msg = format!("{err:?}");
        assert!(
            msg.contains("exceeds u8 range") && msg.contains("256"),
            "expected u8-overflow diagnostic naming 256; got: {msg}"
        );
    }

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
