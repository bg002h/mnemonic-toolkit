//! v0.26.0 round-trip discipline helpers per SPEC §7.
//!
//! Two canonicalize helpers (one per format) + a unified-diff helper.
//! Canonicalization is **semantic, not byte-exact**: the helpers strip
//! optional whitespace, drop fields that cannot be regenerated from a
//! toolkit bundle alone (BSMS audit fields; Core `timestamp`/`next`/
//! `next_index`), and re-checksum descriptor bodies so a checksum-typo
//! variant of the same descriptor still semantic-matches its re-emitted
//! form.
//!
//! Per SPEC §7.3.1 (BSMS) + §7.3.2 (Bitcoin Core).
//!
//! **Concrete keys, no @N placeholders.** Unlike `bsms::BsmsParser::parse`
//! which substitutes `[fp/path]xpub` → `[fp/path]@N` to feed the toolkit's
//! placeholder pipeline, canonicalization operates on the raw third-party
//! descriptor (concrete `[fp/path]xpub` keys preserved). The BIP-380
//! checksum is recomputed via miniscript's `ChecksumEngine` after a
//! parse + render cycle through `Descriptor::<DescriptorPublicKey>`. This
//! normalizes any cosmetic differences in the descriptor body (whitespace,
//! checksum hash itself) while preserving key payload + origin annotation.

use crate::error::ToolkitError;
use miniscript::descriptor::checksum::Engine as ChecksumEngine;
use miniscript::{Descriptor as MsDescriptor, DescriptorPublicKey};
use serde_json::Value;
use std::collections::BTreeMap;
use std::str::FromStr;

/// SPEC §7.3.1 — canonicalize a BSMS Round-2 blob for semantic round-trip
/// comparison.
///
/// 1. CRLF → LF normalize.
/// 2. Strip trailing whitespace per line.
/// 3. Parse descriptor body via `MsDescriptor::<DescriptorPublicKey>::from_str`;
///    re-render via `to_string()`; re-checksum via miniscript's
///    `ChecksumEngine`.
/// 4. Drop audit lines (token, signature, first_address, derivation_path).
/// 5. Re-emit canonical form: `BSMS 1.0\n<re-rendered-descriptor>#<re-checksum>\n`.
pub(crate) fn canonicalize_bsms(blob: &[u8]) -> Result<String, ToolkitError> {
    let text = std::str::from_utf8(blob).map_err(|e| {
        ToolkitError::ImportWalletParse(format!("canonicalize_bsms: blob is not valid UTF-8: {e}"))
    })?;

    // Step 1: CRLF → LF.
    let normalized = text.replace("\r\n", "\n");

    // Step 2: split on LF + strip trailing whitespace per line. (Leading
    // whitespace is significant inside the descriptor; we only trim
    // trailing.)
    let lines: Vec<&str> = normalized
        .split('\n')
        .map(|l| l.trim_end_matches([' ', '\t']))
        .collect();

    // Drop trailing empty entries (a trailing `\n` yields a single empty
    // tail element). Empty lines in the middle of the blob will cause the
    // 2/6 line-count match below to fail; here we just tolerate the
    // trailing newline.
    let mut tail_idx = lines.len();
    while tail_idx > 0 && lines[tail_idx - 1].is_empty() {
        tail_idx -= 1;
    }
    let lines = &lines[..tail_idx];

    if lines.is_empty() {
        return Err(ToolkitError::ImportWalletParse(
            "canonicalize_bsms: empty blob after normalize".to_string(),
        ));
    }

    let header = lines[0];
    if header != "BSMS 1.0" {
        return Err(ToolkitError::ImportWalletParse(format!(
            "canonicalize_bsms: expected header `BSMS 1.0`, got {header:?}"
        )));
    }

    // Step 3: locate the descriptor body. 2-line shape: line 1.
    // 6-line shape: line 2 (line 1 is the token, line 2 is the
    // descriptor). Audit lines 1/3/4/5 (i.e., token + path + first-
    // address + signature) are dropped per step 4.
    let descriptor_with_csum = match lines.len() {
        2 => lines[1],
        6 => lines[2],
        other => {
            return Err(ToolkitError::ImportWalletParse(format!(
                "canonicalize_bsms: expected 2 or 6 lines, got {other}"
            )));
        }
    };

    let canonical_desc = recanonicalize_descriptor(descriptor_with_csum)?;

    // Step 5: re-emit canonical form (always 2-line shape; audit lines
    // dropped per SPEC §7.3.1 step 4).
    Ok(format!("BSMS 1.0\n{canonical_desc}\n"))
}

/// SPEC §7.3.2 — canonicalize a Bitcoin Core `listdescriptors` JSON blob
/// for semantic round-trip comparison.
///
/// 1. Parse JSON via `serde_json`.
/// 2. For each `descriptors[i]`:
///    - `desc`: re-checksum after parse + render.
///    - `active`, `internal`, `range`: preserved.
///    - `timestamp`, `next`, `next_index`: dropped from compare.
/// 3. `wallet_name`: preserved (metadata).
/// 4. Re-serialize with keys sorted alphabetically + 2-space indent +
///    trailing newline.
///
/// Implementation note: the Core export emitter (`wallet_export/bitcoin_core.rs`)
/// emits a top-level JSON array (one entry per multipath-split desc) without
/// the `wallet_name` wrapper, while the importer accepts the `listdescriptors`
/// RPC envelope `{ wallet_name, descriptors: [...] }`. Canonicalize handles
/// BOTH shapes so import-side fixtures + export-side emit can round-trip
/// against each other.
pub(crate) fn canonicalize_bitcoin_core(blob: &[u8]) -> Result<String, ToolkitError> {
    let value: Value = serde_json::from_slice(blob).map_err(|e| {
        ToolkitError::ImportWalletParse(format!("canonicalize_bitcoin_core: invalid JSON: {e}"))
    })?;

    // Normalize to a canonical envelope:
    //   { wallet_name: Option<String>, descriptors: [entry, ...] }
    // Bare-array form (export emitter shape) is hoisted; object form
    // preserves wallet_name and uses its `descriptors` field directly.
    let (wallet_name, entries): (Option<String>, Vec<Value>) = match value {
        Value::Array(arr) => (None, arr),
        Value::Object(map) => {
            let wn = map
                .get("wallet_name")
                .and_then(|v| v.as_str())
                .map(String::from);
            let entries = map
                .get("descriptors")
                .and_then(|v| v.as_array())
                .cloned()
                .ok_or_else(|| {
                    ToolkitError::ImportWalletParse(
                        "canonicalize_bitcoin_core: object form missing `descriptors` array"
                            .to_string(),
                    )
                })?;
            (wn, entries)
        }
        _ => {
            return Err(ToolkitError::ImportWalletParse(
                "canonicalize_bitcoin_core: top-level JSON must be object or array".to_string(),
            ));
        }
    };

    let mut canonical_entries: Vec<Value> = Vec::with_capacity(entries.len());
    for entry in entries {
        let obj = entry.as_object().ok_or_else(|| {
            ToolkitError::ImportWalletParse(
                "canonicalize_bitcoin_core: descriptors[i] is not an object".to_string(),
            )
        })?;

        let desc_with_csum = obj.get("desc").and_then(|v| v.as_str()).ok_or_else(|| {
            ToolkitError::ImportWalletParse(
                "canonicalize_bitcoin_core: descriptors[i].desc is missing or not a string"
                    .to_string(),
            )
        })?;
        let canonical_desc = recanonicalize_descriptor(desc_with_csum)?;

        // Build a fresh entry with only the preserved fields. We use
        // BTreeMap for alphabetic-key ordering at serialize time (BTreeMap
        // serializes its key-value pairs in key-sorted order via serde_json).
        let mut canonical: BTreeMap<String, Value> = BTreeMap::new();
        canonical.insert("desc".to_string(), Value::String(canonical_desc));
        if let Some(active) = obj.get("active") {
            canonical.insert("active".to_string(), active.clone());
        }
        if let Some(internal) = obj.get("internal") {
            canonical.insert("internal".to_string(), internal.clone());
        }
        if let Some(range) = obj.get("range") {
            canonical.insert("range".to_string(), range.clone());
        }
        // SPEC §7.3.2: timestamp, next, next_index DROPPED from compare.

        canonical_entries.push(serde_json::to_value(&canonical).map_err(|e| {
            ToolkitError::ImportWalletParse(format!(
                "canonicalize_bitcoin_core: re-serialize entry: {e}"
            ))
        })?);
    }

    // Build the canonical envelope as a BTreeMap for sorted-key ordering.
    let mut envelope: BTreeMap<String, Value> = BTreeMap::new();
    envelope.insert("descriptors".to_string(), Value::Array(canonical_entries));
    if let Some(wn) = wallet_name {
        envelope.insert("wallet_name".to_string(), Value::String(wn));
    }

    let mut text = serde_json::to_string_pretty(&envelope).map_err(|e| {
        ToolkitError::ImportWalletParse(format!("canonicalize_bitcoin_core: pretty-print: {e}"))
    })?;
    text.push('\n');
    Ok(text)
}

/// SPEC §7.4 — unified-diff (RFC standard) between two strings. Used for
/// the `roundtrip.diff` envelope field + stderr WARNING body.
///
/// Returns the empty string for byte-identical inputs (no diff to render).
/// Header is fixed to `--- input` / `+++ output`.
pub(crate) fn unified_diff(old: &str, new: &str) -> String {
    similar::TextDiff::from_lines(old, new)
        .unified_diff()
        .header("input", "output")
        .to_string()
}

/// Strip an optional trailing `#<checksum>` from a descriptor, parse via
/// `MsDescriptor::<DescriptorPublicKey>::from_str`, render via `to_string()`,
/// and append the freshly computed BIP-380 checksum.
///
/// This is the per-descriptor heart of both canonicalize helpers. By
/// round-tripping through `Display`, we normalize any cosmetic differences
/// (whitespace, capitalization within hex, etc.) and emit a deterministic
/// `<body>#<checksum>` form.
///
/// On error (parse / re-checksum), returns `ImportWalletParse`.
fn recanonicalize_descriptor(desc_with_csum: &str) -> Result<String, ToolkitError> {
    // Strip any existing checksum suffix BEFORE parsing — miniscript's
    // `from_str` accepts both forms (with or without `#<csum>`), but we
    // want to ensure the re-rendered form carries a freshly computed
    // checksum unconditionally.
    let body_no_csum = match desc_with_csum.rsplit_once('#') {
        Some((body, _)) => body,
        None => desc_with_csum,
    };

    let parsed = MsDescriptor::<DescriptorPublicKey>::from_str(body_no_csum).map_err(|e| {
        ToolkitError::ImportWalletParse(format!("canonicalize: descriptor parse failed: {e}"))
    })?;

    let rendered = parsed.to_string();
    // miniscript's `Display` impl for `Descriptor` already includes a
    // `#<csum>` suffix; strip it and re-compute via `ChecksumEngine` so
    // the result is deterministic across miniscript versions that might
    // emit subtly different `Display` output for the same descriptor.
    let body_after_display = match rendered.rsplit_once('#') {
        Some((b, _)) => b,
        None => rendered.as_str(),
    };

    let mut eng = ChecksumEngine::new();
    eng.input(body_after_display).map_err(|e| {
        ToolkitError::ImportWalletParse(format!(
            "canonicalize: checksum engine input rejected (descriptor body non-ASCII?): {e}"
        ))
    })?;
    let csum = eng.checksum();
    Ok(format!("{body_after_display}#{csum}"))
}

// =============================================================================
// v0.28.0 Phase P0C — canonicalize skeletons for the 6 new wallet-import formats.
//
// Per plan-doc P0C row (R0 I2 + R1-M5 fold): each new parser's B-sub-phase
// scope includes its `canonicalize_<format>` helper. The bodies here are
// `Err(ToolkitError::BadInput("not yet implemented; <format> ingest lands
// in Phase P{N}B".into()))` stubs that satisfy the import dispatch surface
// at `cmd/import_wallet.rs:432-435` (Site 6 in plan-doc §B.2 #6). Per-parser
// P{N}B sub-phases replace the body with a real semantic-canonicalize per
// SPEC §11.x; this signature does not change.
//
// At P0C the skeletons are unreachable in practice (Site 2 + Site 4 panic
// earlier on `--format <new>`, and auto-sniff can't yield a new-format
// verdict until per-parser P{N}A wires the SniffOutcome variant). Returning
// `Err(BadInput)` is the defensive shape — should anything reach a skeleton
// in violation of that contract, it surfaces as a typed BadInput rather
// than a silent empty-string roundtrip.
//
// Per-parser P{N}B → P{N}C ordering: P{N}B installs the real body here;
// P{N}C flips the corresponding Site 2 + Site 4 dispatch arms to invoke
// the format's parser. The Site 6 dispatch already routes via these symbols
// (alphabetical import block at `cmd/import_wallet.rs:50-65`), so P{N}B's
// body-swap is structurally complete the moment the new body lands.
// =============================================================================

/// SPEC §11.3 + §7 — canonicalize a Coldcard single-sig generic-JSON blob
/// for semantic round-trip comparison.
///
/// Strategy mirrors `canonicalize_bitcoin_core`'s "preserve key fields,
/// drop ephemeral, sort keys" pattern but adapted for Coldcard's
/// `chain` / `xfp` / `account` / `bip*` / `xpub` shape:
///
/// 1. Parse JSON top-level (must be object).
/// 2. Preserve: `chain`, `xfp` (uppercased), `account` (defaulting to 0),
///    and the SELECTED dominant-BIP block per §11.3.1 dominance order.
/// 3. Within the selected BIP block: preserve `name`, `deriv`, `xfp`,
///    `xpub`. Drop `_pub` (SLIP-132 alternate; the BIP-32 `xpub` carries
///    the canonical key) and `first` (address string is a derived view,
///    not key state).
/// 4. Drop competing-BIP blocks (superseded by dominance).
/// 5. Drop legacy top-level `xpub` if a `bip*` block was selected.
/// 6. Drop `bip48_*` blocks (multisig-context hints).
/// 7. Re-serialize with alphabetic key ordering + 2-space indent +
///    trailing newline.
pub(crate) fn canonicalize_coldcard(blob: &[u8]) -> Result<String, ToolkitError> {
    let value: Value = serde_json::from_slice(blob).map_err(|e| {
        ToolkitError::ImportWalletParse(format!("canonicalize_coldcard: invalid JSON: {e}"))
    })?;
    let obj = value.as_object().ok_or_else(|| {
        ToolkitError::ImportWalletParse(
            "canonicalize_coldcard: top-level JSON must be an object".to_string(),
        )
    })?;

    // §11.3.1 dominance: pick the highest-priority bip* block present.
    let dominant_key: Option<&'static str> = {
        let mut picked: Option<&'static str> = None;
        for key in ["bip86", "bip84", "bip49", "bip44"] {
            if obj.contains_key(key) {
                picked = Some(key);
                break;
            }
        }
        picked
    };

    // Canonical envelope (BTreeMap → alphabetic key ordering at serialize
    // time per `canonicalize_bitcoin_core` precedent).
    let mut envelope: BTreeMap<String, Value> = BTreeMap::new();

    // Required field: chain.
    let chain = obj.get("chain").and_then(|v| v.as_str()).ok_or_else(|| {
        ToolkitError::ImportWalletParse(
            "canonicalize_coldcard: missing or non-string top-level `chain` field".to_string(),
        )
    })?;
    envelope.insert("chain".to_string(), Value::String(chain.to_string()));

    // Required field: xfp (uppercase normalize).
    let xfp = obj.get("xfp").and_then(|v| v.as_str()).ok_or_else(|| {
        ToolkitError::ImportWalletParse(
            "canonicalize_coldcard: missing or non-string top-level `xfp` field".to_string(),
        )
    })?;
    envelope.insert("xfp".to_string(), Value::String(xfp.to_uppercase()));

    // Optional field: account (default 0).
    let account = obj
        .get("account")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    envelope.insert(
        "account".to_string(),
        Value::Number(serde_json::Number::from(account)),
    );

    // If a bip* block is selected, preserve it (with internal canonicalization).
    if let Some(key) = dominant_key {
        let block = obj.get(key).unwrap(); // checked above
        let block_obj = block.as_object().ok_or_else(|| {
            ToolkitError::ImportWalletParse(format!(
                "canonicalize_coldcard: `{key}` block is not an object"
            ))
        })?;
        let mut canonical_block: BTreeMap<String, Value> = BTreeMap::new();
        // Preserve: name, deriv, xfp, xpub.
        for f in ["name", "deriv", "xpub"] {
            if let Some(v) = block_obj.get(f) {
                canonical_block.insert(f.to_string(), v.clone());
            }
        }
        // xfp inside the block: uppercase normalize for byte-stability.
        if let Some(inner_xfp) = block_obj.get("xfp").and_then(|v| v.as_str()) {
            canonical_block.insert(
                "xfp".to_string(),
                Value::String(inner_xfp.to_uppercase()),
            );
        }
        // _pub + first deliberately DROPPED (ephemeral / derived).
        envelope.insert(
            key.to_string(),
            serde_json::to_value(&canonical_block).map_err(|e| {
                ToolkitError::ImportWalletParse(format!(
                    "canonicalize_coldcard: re-serialize bip block: {e}"
                ))
            })?,
        );
    } else if obj.contains_key("xpub") {
        // Legacy Mk1/Mk2: preserve top-level xpub.
        envelope.insert(
            "xpub".to_string(),
            Value::String(
                obj.get("xpub")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        ToolkitError::ImportWalletParse(
                            "canonicalize_coldcard: top-level `xpub` must be a string".to_string(),
                        )
                    })?
                    .to_string(),
            ),
        );
    }
    // bip48_* deliberately DROPPED (multisig hints; not part of singlesig
    // semantic round-trip).

    let mut text = serde_json::to_string_pretty(&envelope).map_err(|e| {
        ToolkitError::ImportWalletParse(format!("canonicalize_coldcard: pretty-print: {e}"))
    })?;
    text.push('\n');
    Ok(text)
}

/// SPEC §11.4 — canonicalize a Coldcard multisig text blob.
/// Body lands in Phase P4B.
pub(crate) fn canonicalize_coldcard_multisig(_blob: &[u8]) -> Result<String, ToolkitError> {
    Err(ToolkitError::BadInput(
        "canonicalize_coldcard_multisig: not yet implemented; \
         coldcard-multisig ingest lands in Phase P4B"
            .into(),
    ))
}

/// SPEC §11.6 — canonicalize an Electrum wallet JSON blob.
/// Body lands in Phase P6B.
pub(crate) fn canonicalize_electrum(_blob: &[u8]) -> Result<String, ToolkitError> {
    Err(ToolkitError::BadInput(
        "canonicalize_electrum: not yet implemented; electrum ingest lands in Phase P6B".into(),
    ))
}

/// SPEC §11.5 — canonicalize a Jade multisig-file JSON wrapper.
/// Body lands in Phase P5B.
pub(crate) fn canonicalize_jade(_blob: &[u8]) -> Result<String, ToolkitError> {
    Err(ToolkitError::BadInput(
        "canonicalize_jade: not yet implemented; jade ingest lands in Phase P5B".into(),
    ))
}

/// SPEC §11.1 — canonicalize a Sparrow wallet JSON blob.
/// Body lands in Phase P1B.
pub(crate) fn canonicalize_sparrow(_blob: &[u8]) -> Result<String, ToolkitError> {
    Err(ToolkitError::BadInput(
        "canonicalize_sparrow: not yet implemented; sparrow ingest lands in Phase P1B".into(),
    ))
}

/// SPEC §11.2 — canonicalize a Specter wallet JSON blob.
/// Body lands in Phase P2B.
pub(crate) fn canonicalize_specter(_blob: &[u8]) -> Result<String, ToolkitError> {
    Err(ToolkitError::BadInput(
        "canonicalize_specter: not yet implemented; specter ingest lands in Phase P2B".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Canonical xpubs reused from the rest of the test suite. Kept inline
    // (no fixture lookups) for unit-test simplicity.
    const FP_A: &str = "b8688df1";
    const XPUB_A: &str = "xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX";
    const FP_B: &str = "28645006";
    const XPUB_B: &str = "xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6";

    fn dummy_csum(body: &str) -> String {
        let mut e = ChecksumEngine::new();
        e.input(body).unwrap();
        e.checksum()
    }

    /// Build a 2-line BSMS blob with a freshly computed checksum.
    fn bsms_2line(desc: &str) -> String {
        let cs = dummy_csum(desc);
        format!("BSMS 1.0\n{desc}#{cs}\n")
    }

    /// Build a 6-line BSMS blob with a freshly computed checksum + arbitrary
    /// audit fields.
    fn bsms_6line(desc: &str) -> String {
        let cs = dummy_csum(desc);
        format!("BSMS 1.0\n00112233aabbccdd\n{desc}#{cs}\nm/48'/0'/0'/2'\nbc1qexample\nH/sig=\n")
    }

    #[test]
    fn canonicalize_bsms_drops_audit_lines() {
        let desc =
            format!("wsh(sortedmulti(2,[{FP_A}/48'/0'/0'/2']{XPUB_A}/<0;1>/*,[{FP_B}/48'/0'/0'/2']{XPUB_B}/<0;1>/*))");
        let blob_2 = bsms_2line(&desc);
        let blob_6 = bsms_6line(&desc);
        let c2 = canonicalize_bsms(blob_2.as_bytes()).unwrap();
        let c6 = canonicalize_bsms(blob_6.as_bytes()).unwrap();
        // Audit lines must be dropped: 6-line canonicalizes to the same
        // form as 2-line.
        assert_eq!(c2, c6);
    }

    #[test]
    fn canonicalize_bsms_normalizes_crlf() {
        let desc = format!("wpkh([{FP_A}/84'/0'/0']{XPUB_A}/<0;1>/*)");
        let blob = bsms_2line(&desc);
        let blob_crlf = blob.replace('\n', "\r\n");
        let a = canonicalize_bsms(blob.as_bytes()).unwrap();
        let b = canonicalize_bsms(blob_crlf.as_bytes()).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn canonicalize_bsms_strips_trailing_whitespace() {
        let desc = format!("wpkh([{FP_A}/84'/0'/0']{XPUB_A}/<0;1>/*)");
        let blob = bsms_2line(&desc);
        // Append trailing whitespace on each line. `canonicalize_bsms`
        // strips both ` ` and `\t` from the end of every line per
        // SPEC §7.3.1 step 2.
        let blob_ws = blob.replace('\n', "  \t\n");
        let a = canonicalize_bsms(blob.as_bytes()).unwrap();
        let b = canonicalize_bsms(blob_ws.as_bytes()).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn canonicalize_bsms_recomputes_checksum() {
        // Feed a blob with a deliberately-incorrect checksum suffix; the
        // canonicalize step must produce the same output as the correctly-
        // checksummed blob (because the body is re-checksummed via
        // ChecksumEngine).
        let desc = format!("wpkh([{FP_A}/84'/0'/0']{XPUB_A}/<0;1>/*)");
        let cs = dummy_csum(&desc);
        let good = format!("BSMS 1.0\n{desc}#{cs}\n");
        let bad = format!("BSMS 1.0\n{desc}#xxxxxxxx\n");
        let a = canonicalize_bsms(good.as_bytes()).unwrap();
        let b = canonicalize_bsms(bad.as_bytes()).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn canonicalize_bsms_rejects_missing_header() {
        let bad = b"NOT BSMS\nwhatever\n";
        assert!(canonicalize_bsms(bad).is_err());
    }

    #[test]
    fn canonicalize_bsms_rejects_wrong_line_count() {
        let bad = b"BSMS 1.0\nfoo\nbar\nbaz\n"; // 4 non-empty lines, neither 2 nor 6.
        assert!(canonicalize_bsms(bad).is_err());
    }

    #[test]
    fn canonicalize_core_object_form_drops_dropped_fields() {
        let desc = format!("wpkh([{FP_A}/84'/0'/0']{XPUB_A}/0/*)");
        let cs = dummy_csum(&desc);
        let with_extras = format!(
            "{{\n  \"wallet_name\": \"a\",\n  \"descriptors\": [\n    {{\n      \"desc\": \"{desc}#{cs}\",\n      \"active\": true,\n      \"internal\": false,\n      \"range\": [0, 1000],\n      \"timestamp\": \"now\",\n      \"next\": 5,\n      \"next_index\": 5\n    }}\n  ]\n}}\n"
        );
        let without_extras = format!(
            "{{\n  \"wallet_name\": \"a\",\n  \"descriptors\": [\n    {{\n      \"desc\": \"{desc}#{cs}\",\n      \"active\": true,\n      \"internal\": false,\n      \"range\": [0, 1000]\n    }}\n  ]\n}}\n"
        );
        let a = canonicalize_bitcoin_core(with_extras.as_bytes()).unwrap();
        let b = canonicalize_bitcoin_core(without_extras.as_bytes()).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn canonicalize_core_object_and_array_match_when_no_wallet_name() {
        // Array-form blob (export emitter shape) vs object-form with the
        // same descriptors. They differ on wallet_name, so we test the
        // bare-array case against an object-form WITHOUT wallet_name.
        let desc = format!("wpkh([{FP_A}/84'/0'/0']{XPUB_A}/0/*)");
        let cs = dummy_csum(&desc);
        let array_form = format!(
            "[\n  {{\n    \"desc\": \"{desc}#{cs}\",\n    \"active\": true,\n    \"internal\": false,\n    \"range\": [0, 1000]\n  }}\n]\n"
        );
        let object_form_no_wn = format!(
            "{{\n  \"descriptors\": [\n    {{\n      \"desc\": \"{desc}#{cs}\",\n      \"active\": true,\n      \"internal\": false,\n      \"range\": [0, 1000]\n    }}\n  ]\n}}\n"
        );
        let a = canonicalize_bitcoin_core(array_form.as_bytes()).unwrap();
        let b = canonicalize_bitcoin_core(object_form_no_wn.as_bytes()).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn canonicalize_core_keys_sorted_alphabetically() {
        let desc = format!("wpkh([{FP_A}/84'/0'/0']{XPUB_A}/0/*)");
        let cs = dummy_csum(&desc);
        let blob = format!(
            "{{\n  \"descriptors\": [\n    {{\n      \"range\": [0, 1000],\n      \"internal\": false,\n      \"desc\": \"{desc}#{cs}\",\n      \"active\": true\n    }}\n  ]\n}}\n"
        );
        let canonical = canonicalize_bitcoin_core(blob.as_bytes()).unwrap();
        // Verify entry-level keys appear in alphabetic order: active, desc,
        // internal, range.
        let active_idx = canonical.find("\"active\"").unwrap();
        let desc_idx = canonical.find("\"desc\"").unwrap();
        let internal_idx = canonical.find("\"internal\"").unwrap();
        let range_idx = canonical.find("\"range\"").unwrap();
        assert!(active_idx < desc_idx);
        assert!(desc_idx < internal_idx);
        assert!(internal_idx < range_idx);
    }

    #[test]
    fn canonicalize_core_recomputes_checksum() {
        let desc = format!("wpkh([{FP_A}/84'/0'/0']{XPUB_A}/0/*)");
        let cs = dummy_csum(&desc);
        let good = format!(
            "{{\n  \"descriptors\": [\n    {{\n      \"desc\": \"{desc}#{cs}\"\n    }}\n  ]\n}}\n"
        );
        let bad = format!(
            "{{\n  \"descriptors\": [\n    {{\n      \"desc\": \"{desc}#xxxxxxxx\"\n    }}\n  ]\n}}\n"
        );
        let a = canonicalize_bitcoin_core(good.as_bytes()).unwrap();
        let b = canonicalize_bitcoin_core(bad.as_bytes()).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn canonicalize_core_rejects_invalid_json() {
        assert!(canonicalize_bitcoin_core(b"not json").is_err());
    }

    #[test]
    fn unified_diff_empty_on_identical_input() {
        assert_eq!(unified_diff("foo\nbar\n", "foo\nbar\n"), "");
    }

    #[test]
    fn unified_diff_nonempty_on_difference() {
        let d = unified_diff("foo\nbar\n", "foo\nbaz\n");
        assert!(d.contains("--- input"));
        assert!(d.contains("+++ output"));
        assert!(d.contains("-bar"));
        assert!(d.contains("+baz"));
    }

    // ========================================================================
    // Fixture-based semantic round-trip cells per SPEC §7.2 + §7.3.
    //
    // These tests read the static fixture files vendored at
    // `crates/mnemonic-toolkit/tests/fixtures/wallet_import/` and exercise
    // the canonicalize helpers against semantic-equivalent variants of each
    // fixture. Pattern per cell:
    //   1. Read fixture bytes.
    //   2. Build a semantically-equivalent variant (CRLF flip, audit-line
    //      injection, key reordering, etc.).
    //   3. Assert `canonicalize(fixture) == canonicalize(variant)`.
    //
    // The crate's integration-test layout uses `tests/fixtures/<path>`
    // relative to the package root; from inside `src/`, the same files
    // resolve via `env!("CARGO_MANIFEST_DIR")`.
    // ========================================================================

    fn read_fixture(name: &str) -> Vec<u8> {
        let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("tests");
        path.push("fixtures");
        path.push("wallet_import");
        path.push(name);
        std::fs::read(&path).unwrap_or_else(|e| panic!("read fixture {}: {e}", path.display()))
    }

    #[test]
    fn fixture_bsms_2line_sortedmulti_2of2_canonicalize_stable() {
        // Sanity: canonicalize is idempotent on a clean fixture.
        let blob = read_fixture("bsms-2line-sortedmulti-2of2.txt");
        let c1 = canonicalize_bsms(&blob).unwrap();
        let c2 = canonicalize_bsms(c1.as_bytes()).unwrap();
        assert_eq!(c1, c2, "canonicalize must be idempotent");
        assert!(c1.starts_with("BSMS 1.0\n"));
        assert!(c1.contains("wsh(sortedmulti(2,"));
    }

    #[test]
    fn fixture_bsms_2line_sortedmulti_2of3_canonicalize_stable() {
        let blob = read_fixture("bsms-2line-sortedmulti-2of3.txt");
        let c = canonicalize_bsms(&blob).unwrap();
        assert!(c.contains("wsh(sortedmulti(2,"));
        // 3 cosigners → 3 origin annotations present.
        let origin_count = c.matches('[').count();
        assert_eq!(origin_count, 3, "expected 3 origin annotations; got: {c}");
    }

    #[test]
    fn fixture_bsms_2line_multi_2of2_canonicalize_stable() {
        // Bare `multi(...)` (declaration-order, not sortedmulti).
        let blob = read_fixture("bsms-2line-multi-2of2.txt");
        let c = canonicalize_bsms(&blob).unwrap();
        assert!(c.contains("sh(multi(2,"));
    }

    #[test]
    fn fixture_bsms_2line_multi_2of3_canonicalize_preserves_declaration_order() {
        // R0 M3 gap-fill: bare `multi(...)` 2-of-3 with cosigners declared
        // in NON-lex order. The canonicalize helper must preserve the
        // declaration order (re-rendering via miniscript's `Display`
        // emits keys in their parsed order for bare `multi(...)`; only
        // `sortedmulti` lex-sorts).
        //
        // Declared xpub-string order in the fixture:
        //   xpub6F... (b8688df1), xpub6B... (5436d724), xpub6D... (28645006)
        // Lex order on xpub byte-strings would be:
        //   xpub6B... < xpub6D... < xpub6F...
        // i.e. lex order = 5436d724, 28645006, b8688df1. The fixture's
        // declaration order differs from lex order in every position;
        // canonicalize must NOT sort.
        let blob = read_fixture("bsms-2line-multi-2of3.txt");
        let c = canonicalize_bsms(&blob).unwrap();

        // Locate each xpub byte-string in the canonical output; assert
        // they appear in DECLARATION order (NOT lex order).
        let pos_xpub_f = c
            .find("xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX")
            .unwrap_or_else(|| panic!("expected xpub6F... in canonical output; got: {c}"));
        let pos_xpub_b = c
            .find("xpub6Buxw9MmbkJr4iAw8SACNci2hQNuPCMwt9P7HkK62ZQAW9UcJaQ2bc6ARD892TToQQ9Rp6AHujHxBLXqAsvn5fRnLfnhKSRfz8qtaoyKUYx")
            .unwrap_or_else(|| panic!("expected xpub6B... in canonical output; got: {c}"));
        let pos_xpub_d = c
            .find("xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6")
            .unwrap_or_else(|| panic!("expected xpub6D... in canonical output; got: {c}"));

        // Declaration order: F < B < D (positionally in the canonical
        // string). Lex order would have been B < D < F.
        assert!(
            pos_xpub_f < pos_xpub_b && pos_xpub_b < pos_xpub_d,
            "canonicalize must preserve declaration order for bare multi(); got positions F={pos_xpub_f}, B={pos_xpub_b}, D={pos_xpub_d}"
        );

        // Idempotency: canon(canon(x)) == canon(x).
        let c2 = canonicalize_bsms(c.as_bytes()).unwrap();
        assert_eq!(c, c2, "canonicalize must be idempotent");

        // 3 cosigner origin annotations present.
        assert_eq!(
            c.matches('[').count(),
            3,
            "expected 3 origin annotations; got: {c}"
        );
    }

    #[test]
    fn fixture_bsms_2line_decay_144_canonicalize_drops_audit() {
        // Decaying-multisig shape. Build a 6-line variant by injecting
        // audit lines; assert it canonicalizes to the same form as the
        // 2-line vendored fixture.
        let blob_2 = read_fixture("bsms-2line-decay-144.txt");
        let c2 = canonicalize_bsms(&blob_2).unwrap();
        // The decay-144 fixture's descriptor body is line index 1.
        let txt = std::str::from_utf8(&blob_2).unwrap();
        let lines: Vec<&str> = txt.split('\n').collect();
        let desc = lines[1];
        let blob_6 =
            format!("BSMS 1.0\n00112233aabbccdd\n{desc}\nm/48'/1'/3'/2'\nbc1qexample\nH/sig=\n");
        let c6 = canonicalize_bsms(blob_6.as_bytes()).unwrap();
        assert_eq!(c2, c6, "audit lines must be dropped");
    }

    #[test]
    fn fixture_bsms_1of1_singlesig_canonicalize_stable() {
        let blob = read_fixture("bsms-1of1-singlesig.txt");
        let c = canonicalize_bsms(&blob).unwrap();
        assert!(c.contains("wpkh(["));
        // Single cosigner → 1 origin annotation.
        assert_eq!(c.matches('[').count(), 1);
    }

    #[test]
    fn fixture_bsms_shwsh_2of3_canonicalize_stable() {
        let blob = read_fixture("bsms-shwsh-2of3.txt");
        let c = canonicalize_bsms(&blob).unwrap();
        assert!(c.contains("sh(wsh(sortedmulti(2,"));
    }

    #[test]
    fn fixture_bsms_testnet_2of2_canonicalize_preserves_tpub() {
        let blob = read_fixture("bsms-testnet-2of2.txt");
        let c = canonicalize_bsms(&blob).unwrap();
        // tpub keys must be preserved through canonicalize (testnet
        // identifier survives the parse → render cycle).
        assert!(c.contains("tpubD"), "testnet tpub must survive: {c}");
    }

    #[test]
    fn fixture_bsms_2line_decaying_multisig_32768_canonicalize_stable() {
        // Pre-existing Phase 2 fixture; ensure it still canonicalizes.
        let blob = read_fixture("bsms_2line_decaying_multisig_32768.txt");
        let c = canonicalize_bsms(&blob).unwrap();
        assert!(c.contains("sln:older(32768)"));
    }

    #[test]
    fn fixture_bsms_crlf_variant_matches_lf_variant() {
        // Read 2-line fixture, transform to CRLF, assert canonicalize
        // produces the same output as the original LF form.
        let blob = read_fixture("bsms-2line-sortedmulti-2of2.txt");
        let lf = std::str::from_utf8(&blob).unwrap();
        let crlf = lf.replace('\n', "\r\n");
        let a = canonicalize_bsms(&blob).unwrap();
        let b = canonicalize_bsms(crlf.as_bytes()).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn fixture_bsms_extra_trailing_newlines_match() {
        // Append a few extra trailing newlines; canonicalize must produce
        // the same output (strip_trailing_empty rule).
        let blob = read_fixture("bsms-2line-sortedmulti-2of2.txt");
        let with_extra = {
            let mut v = blob.clone();
            v.extend_from_slice(b"\n\n\n");
            v
        };
        let a = canonicalize_bsms(&blob).unwrap();
        let b = canonicalize_bsms(&with_extra).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn fixture_core_bip84_mainnet_canonicalize_drops_timestamp() {
        let with_ts = read_fixture("core-bip84-mainnet.json");
        let txt = std::str::from_utf8(&with_ts).unwrap();
        // Strip the timestamp field manually + the trailing comma on
        // `range` so the JSON is still valid.
        let stripped = txt
            .replace(",\n      \"timestamp\": \"now\"\n", "\n")
            .replace("\"range\": [0, 1000],\n", "\"range\": [0, 1000]\n");
        let a = canonicalize_bitcoin_core(&with_ts).unwrap();
        let b = canonicalize_bitcoin_core(stripped.as_bytes()).unwrap();
        assert_eq!(a, b, "timestamp must be dropped from canonical form");
    }

    #[test]
    fn fixture_core_bip49_mainnet_canonicalize_stable() {
        // BIP-49 fixture has 2 entries (receive + change); both must
        // appear in the canonical output.
        let blob = read_fixture("core-bip49-mainnet.json");
        let c = canonicalize_bitcoin_core(&blob).unwrap();
        // 2 entries → 2 `desc:` occurrences.
        assert_eq!(c.matches("\"desc\":").count(), 2);
        assert!(c.contains("sh(wpkh("));
    }

    #[test]
    fn fixture_core_multisig_2of3_canonicalize_preserves_keys() {
        let blob = read_fixture("core-multisig-2of3.json");
        let c = canonicalize_bitcoin_core(&blob).unwrap();
        // 3 cosigner fingerprints all preserved.
        assert!(c.contains("b8688df1"));
        assert!(c.contains("28645006"));
        assert!(c.contains("5436d724"));
    }

    #[test]
    fn fixture_core_testnet_bip84_canonicalize_preserves_tpub() {
        let blob = read_fixture("core-testnet-bip84.json");
        let c = canonicalize_bitcoin_core(&blob).unwrap();
        assert!(c.contains("tpubD"), "testnet tpub must survive: {c}");
    }

    #[test]
    fn fixture_core_multi_bip84_canonicalize_drops_wallet_state() {
        // Existing Phase 3 fixture with no `timestamp`/`next`/`next_index`
        // but with 4 entries; canonicalize must preserve all 4 + drop
        // nothing (nothing to drop).
        let blob = read_fixture("core-multi-bip84.json");
        let c = canonicalize_bitcoin_core(&blob).unwrap();
        assert_eq!(c.matches("\"desc\":").count(), 4);
    }

    #[test]
    fn fixture_core_key_reordering_irrelevant() {
        // Read a Core fixture, manually re-order ENTRY-level keys
        // (active/desc/internal/range/timestamp scrambled), then
        // canonicalize. Output must be byte-identical because
        // canonicalize sorts alphabetically.
        let blob = read_fixture("core-bip84-mainnet.json");
        let original = canonicalize_bitcoin_core(&blob).unwrap();

        // Hand-build a scrambled-key variant of the same data.
        let scrambled = r#"{
  "wallet_name": "bip84_mainnet",
  "descriptors": [
    {
      "timestamp": "now",
      "range": [0, 1000],
      "internal": false,
      "active": true,
      "desc": "wpkh([b8688df1/84'/0'/0']xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX/<0;1>/*)#5ql5mvwg"
    }
  ]
}
"#;
        let scrambled_canonical = canonicalize_bitcoin_core(scrambled.as_bytes()).unwrap();
        assert_eq!(
            original, scrambled_canonical,
            "alphabetic key sort must normalize entry-level key order"
        );
    }

    #[test]
    fn unified_diff_byte_exact_branch_short_circuits() {
        // SPEC §7.4: `diff` is `Some(...)` iff `byte_exact == false`.
        // The helper itself does not gate on byte_exact (the caller does),
        // but for identical inputs it must produce the empty string so
        // the caller can decide cheaply.
        let blob = read_fixture("bsms-2line-sortedmulti-2of2.txt");
        let s = std::str::from_utf8(&blob).unwrap();
        assert_eq!(unified_diff(s, s), "");
    }

    // =========================================================================
    // v0.28.0 Phase P0C — skeleton-canonicalize cells.
    //
    // Each new format's `canonicalize_<format>` helper returns
    // `Err(ToolkitError::BadInput("not yet implemented; <format> ingest lands
    // in Phase P{N}B"))` per plan-doc P0C row. Per-parser P{N}B replaces the
    // body with the real semantic-canonicalize implementation; these cells
    // become regression guards for the skeleton-shape contract and will be
    // REPLACED (not augmented) at P{N}B with format-specific happy-path
    // canonicalize cells.
    //
    // Pinning the error template here defends against accidental
    // "early-flip" of a skeleton body without the matching SPEC §11.x parse
    // contract landing first.
    // =========================================================================

    // =========================================================================
    // v0.28.0 Phase P3B — canonicalize_coldcard semantic round-trip cells.
    // =========================================================================

    /// SPEC §11.3 + §7 round-trip: a well-formed BIP-84 BTC blob
    /// canonicalizes deterministically. Re-canonicalizing the output
    /// produces the same bytes (idempotence — required for `byte_exact`
    /// to be meaningful per SPEC §7.4).
    #[test]
    fn canonicalize_coldcard_bip84_idempotent() {
        let blob = br#"{
  "chain": "BTC",
  "xfp": "5436D724",
  "account": 0,
  "bip84": {
    "name": "p2wpkh",
    "deriv": "m/84'/0'/0'",
    "xfp": "127EA0E6",
    "xpub": "xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9",
    "_pub": "zpub6qTBTNftBzVTjgVcSUw7vW5N1KQbV93Jnrw314RHGkCkSx4vk6nEWH1MJfReXi2WThvuDRiRpyT7cDoakEcZMQ1iZPgfJgQrcVMR4aJWh6S",
    "first": "bc1qzmtrqsfuaf6l6kkcsseumq26ukaphfj9skkug6"
  }
}
"#;
        let first = canonicalize_coldcard(blob).unwrap();
        let second = canonicalize_coldcard(first.as_bytes()).unwrap();
        assert_eq!(first, second, "canonicalize must be idempotent");
    }

    /// `_pub` (SLIP-132 alternate) + `first` (derived address) are
    /// ephemeral and MUST be dropped by canonicalize (semantic
    /// round-trip, not byte-exact).
    #[test]
    fn canonicalize_coldcard_drops_pub_alternate_and_first_address() {
        let with_extras = br#"{"chain":"BTC","xfp":"5436D724","account":0,"bip84":{"name":"p2wpkh","deriv":"m/84'/0'/0'","xfp":"127EA0E6","xpub":"xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9","_pub":"zpub6qTBTNftBzVTjgVcSUw7vW5N1KQbV93Jnrw314RHGkCkSx4vk6nEWH1MJfReXi2WThvuDRiRpyT7cDoakEcZMQ1iZPgfJgQrcVMR4aJWh6S","first":"bc1qfoo"}}"#;
        let without_extras = br#"{"chain":"BTC","xfp":"5436D724","account":0,"bip84":{"name":"p2wpkh","deriv":"m/84'/0'/0'","xfp":"127EA0E6","xpub":"xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9"}}"#;
        let c1 = canonicalize_coldcard(with_extras).unwrap();
        let c2 = canonicalize_coldcard(without_extras).unwrap();
        assert_eq!(c1, c2, "canonicalize must drop _pub + first");
        assert!(!c1.contains("_pub"), "_pub must be dropped");
        assert!(!c1.contains("first"), "first must be dropped");
        assert!(!c1.contains("zpub"), "SLIP-132 alternate value gone");
    }

    /// Competing-BIP blocks are superseded by dominance order; canonicalize
    /// drops them so a blob carrying bip84+bip49+bip44 round-trips to the
    /// same form as a bip84-only blob.
    #[test]
    fn canonicalize_coldcard_dominance_drops_competing_blocks() {
        let multi_bip = br#"{"chain":"BTC","xfp":"5436D724","account":0,"bip44":{"name":"p2pkh","deriv":"m/44'/0'/0'","xfp":"127EA0E6","xpub":"xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9"},"bip49":{"name":"p2wpkh-p2sh","deriv":"m/49'/0'/0'","xfp":"127EA0E6","xpub":"xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9"},"bip84":{"name":"p2wpkh","deriv":"m/84'/0'/0'","xfp":"127EA0E6","xpub":"xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9"}}"#;
        let single_bip = br#"{"chain":"BTC","xfp":"5436D724","account":0,"bip84":{"name":"p2wpkh","deriv":"m/84'/0'/0'","xfp":"127EA0E6","xpub":"xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9"}}"#;
        let c1 = canonicalize_coldcard(multi_bip).unwrap();
        let c2 = canonicalize_coldcard(single_bip).unwrap();
        assert_eq!(c1, c2, "dominance must drop bip44+bip49 in favor of bip84");
        assert!(
            !c1.contains("\"bip44\"") && !c1.contains("\"bip49\""),
            "competing-BIP block names must be absent in canonical form"
        );
    }

    /// `xfp` casing normalizes to uppercase regardless of source casing.
    #[test]
    fn canonicalize_coldcard_xfp_uppercase_normalize() {
        let lower = br#"{"chain":"BTC","xfp":"5436d724","account":0,"bip84":{"name":"p2wpkh","deriv":"m/84'/0'/0'","xfp":"127ea0e6","xpub":"xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9"}}"#;
        let upper = br#"{"chain":"BTC","xfp":"5436D724","account":0,"bip84":{"name":"p2wpkh","deriv":"m/84'/0'/0'","xfp":"127EA0E6","xpub":"xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9"}}"#;
        let c1 = canonicalize_coldcard(lower).unwrap();
        let c2 = canonicalize_coldcard(upper).unwrap();
        assert_eq!(c1, c2, "xfp casing must normalize to uppercase");
        assert!(c1.contains("5436D724"), "outer xfp uppercase");
        assert!(c1.contains("127EA0E6"), "inner xfp uppercase");
    }

    /// `account` field defaults to 0 when absent.
    #[test]
    fn canonicalize_coldcard_account_defaults_to_zero() {
        let no_account = br#"{"chain":"BTC","xfp":"5436D724","bip84":{"name":"p2wpkh","deriv":"m/84'/0'/0'","xfp":"127EA0E6","xpub":"xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9"}}"#;
        let with_zero = br#"{"chain":"BTC","xfp":"5436D724","account":0,"bip84":{"name":"p2wpkh","deriv":"m/84'/0'/0'","xfp":"127EA0E6","xpub":"xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9"}}"#;
        let c1 = canonicalize_coldcard(no_account).unwrap();
        let c2 = canonicalize_coldcard(with_zero).unwrap();
        assert_eq!(c1, c2, "absent account defaults to 0");
    }

    /// Top-level keys sort alphabetically: account < bip84 < chain < xfp.
    /// Asserts the rendered key order via the SEQUENCE of TOP-LEVEL keys
    /// (parsed back via serde_json::Value to walk the serialized form),
    /// avoiding false matches against inner `"xfp"` inside the bip84
    /// sub-object.
    #[test]
    fn canonicalize_coldcard_keys_sorted_alphabetically() {
        let blob = br#"{"chain":"BTC","xfp":"5436D724","account":0,"bip84":{"name":"p2wpkh","deriv":"m/84'/0'/0'","xfp":"127EA0E6","xpub":"xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9"}}"#;
        let c = canonicalize_coldcard(blob).unwrap();
        // Parse top-level lines and extract the FIRST occurrence of each
        // outer key at column 2 (2-space indent + opening quote).
        let i_account = c.find("\n  \"account\"").unwrap();
        let i_bip84 = c.find("\n  \"bip84\"").unwrap();
        let i_chain = c.find("\n  \"chain\"").unwrap();
        let i_xfp = c.find("\n  \"xfp\"").unwrap();
        assert!(
            i_account < i_bip84 && i_bip84 < i_chain && i_chain < i_xfp,
            "top-level keys must sort alphabetically: account < bip84 < chain < xfp; got: {c}"
        );
    }

    /// Non-JSON or non-object blob → ImportWalletParse error.
    #[test]
    fn canonicalize_coldcard_rejects_non_object() {
        let arr = b"[1, 2, 3]";
        let err = canonicalize_coldcard(arr).unwrap_err();
        match err {
            ToolkitError::ImportWalletParse(msg) => assert!(
                msg.contains("top-level JSON must be an object"),
                "got: {msg}"
            ),
            other => panic!("expected ImportWalletParse, got: {other:?}"),
        }
    }

    #[test]
    fn canonicalize_coldcard_multisig_skeleton_returns_not_yet_implemented() {
        let err = canonicalize_coldcard_multisig(b"any blob").unwrap_err();
        match err {
            ToolkitError::BadInput(msg) => {
                assert!(msg.contains("not yet implemented"));
                assert!(msg.contains("P4B"), "msg must cite Phase P4B; got: {msg}");
                assert!(
                    msg.contains("coldcard-multisig"),
                    "msg must cite format; got: {msg}"
                );
            }
            other => panic!("expected BadInput, got: {other:?}"),
        }
    }

    #[test]
    fn canonicalize_electrum_skeleton_returns_not_yet_implemented() {
        let err = canonicalize_electrum(b"any blob").unwrap_err();
        match err {
            ToolkitError::BadInput(msg) => {
                assert!(msg.contains("not yet implemented"));
                assert!(msg.contains("P6B"), "msg must cite Phase P6B; got: {msg}");
                assert!(msg.contains("electrum"), "msg must cite format; got: {msg}");
            }
            other => panic!("expected BadInput, got: {other:?}"),
        }
    }

    #[test]
    fn canonicalize_jade_skeleton_returns_not_yet_implemented() {
        let err = canonicalize_jade(b"any blob").unwrap_err();
        match err {
            ToolkitError::BadInput(msg) => {
                assert!(msg.contains("not yet implemented"));
                assert!(msg.contains("P5B"), "msg must cite Phase P5B; got: {msg}");
                assert!(msg.contains("jade"), "msg must cite format; got: {msg}");
            }
            other => panic!("expected BadInput, got: {other:?}"),
        }
    }

    #[test]
    fn canonicalize_sparrow_skeleton_returns_not_yet_implemented() {
        let err = canonicalize_sparrow(b"any blob").unwrap_err();
        match err {
            ToolkitError::BadInput(msg) => {
                assert!(msg.contains("not yet implemented"));
                assert!(msg.contains("P1B"), "msg must cite Phase P1B; got: {msg}");
                assert!(msg.contains("sparrow"), "msg must cite format; got: {msg}");
            }
            other => panic!("expected BadInput, got: {other:?}"),
        }
    }

    #[test]
    fn canonicalize_specter_skeleton_returns_not_yet_implemented() {
        let err = canonicalize_specter(b"any blob").unwrap_err();
        match err {
            ToolkitError::BadInput(msg) => {
                assert!(msg.contains("not yet implemented"));
                assert!(msg.contains("P2B"), "msg must cite Phase P2B; got: {msg}");
                assert!(msg.contains("specter"), "msg must cite format; got: {msg}");
            }
            other => panic!("expected BadInput, got: {other:?}"),
        }
    }

    #[test]
    fn skeleton_canonicalize_helpers_accept_empty_blob() {
        // Empty blob is a degenerate input; skeletons (formats whose parse
        // impl has not yet landed) must still return the
        // BadInput("not yet implemented") shape (not panic, not Ok, not a
        // different error class). This pins the "shape-only" contract.
        //
        // Phase P3B fold: `canonicalize_coldcard` is now real; an empty
        // blob hits the JSON-parse error path (`ImportWalletParse`) — see
        // `canonicalize_coldcard_rejects_non_object` for the empty-blob
        // refusal test on the real impl.
        for (name, result) in [
            ("coldcard-multisig", canonicalize_coldcard_multisig(b"")),
            ("electrum", canonicalize_electrum(b"")),
            ("jade", canonicalize_jade(b"")),
            ("sparrow", canonicalize_sparrow(b"")),
            ("specter", canonicalize_specter(b"")),
        ] {
            assert!(
                matches!(result, Err(ToolkitError::BadInput(ref m)) if m.contains("not yet implemented")),
                "{name} skeleton must return BadInput(not yet implemented) on empty blob; got: {result:?}"
            );
        }
    }

    #[test]
    fn unified_diff_renders_descriptor_diff() {
        // Two semantically-different BSMS blobs (different threshold);
        // diff must contain both `-` and `+` markers.
        let blob_2of2 = read_fixture("bsms-2line-sortedmulti-2of2.txt");
        let blob_2of3 = read_fixture("bsms-2line-sortedmulti-2of3.txt");
        let a = std::str::from_utf8(&blob_2of2).unwrap();
        let b = std::str::from_utf8(&blob_2of3).unwrap();
        let d = unified_diff(a, b);
        assert!(d.contains("--- input"));
        assert!(d.contains("+++ output"));
        // The two fixtures differ in cosigner count → diff must contain
        // descriptor-body change markers.
        assert!(d
            .lines()
            .any(|l| l.starts_with('-') && !l.starts_with("---")));
        assert!(d
            .lines()
            .any(|l| l.starts_with('+') && !l.starts_with("+++")));
    }
}
