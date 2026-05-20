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

    // Step 3: locate the descriptor body.
    // - 2-line shape: line 1 carries the descriptor.
    // - 4-line shape (v0.28.0; BIP-129-canonical Round-2): line 1 is the
    //   descriptor; lines 2-3 are path-restrictions + first-address (dropped
    //   per step 4 — the canonical form is always re-emitted as 2-line).
    // - 6-line shape (DEPRECATED in v0.28.0): line 2 is the descriptor;
    //   lines 1/3/4/5 (token + path + first-address + signature) are dropped.
    let descriptor_with_csum = match lines.len() {
        2 => lines[1],
        4 => lines[1],
        6 => lines[2],
        other => {
            return Err(ToolkitError::ImportWalletParse(format!(
                "canonicalize_bsms: expected 2, 4, or 6 lines, got {other}"
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

/// SPEC §11.3 — canonicalize a Coldcard single-sig generic-JSON blob for
/// semantic round-trip comparison.
///
/// Strategy:
/// 1. JSON-parse.
/// 2. Drop fields the toolkit doesn't preserve on the bundle (any top-level
///    key not in `COLDCARD_PRESERVED_TOP_LEVEL_KEYS`).
/// 3. Re-emit in alphabetical top-level key order via `BTreeMap`. Mirrors
///    `canonicalize_specter` / `canonicalize_sparrow` shape.
///
/// The canonicalization is **semantic, not byte-exact**: two blobs that
/// differ only in top-level key order / dropped extra fields canonicalize
/// to the same string. Per-bipN sub-objects are preserved verbatim (their
/// internal key order is wire-determined by Coldcard firmware emit; we do
/// not reorder them).
pub(crate) fn canonicalize_coldcard(blob: &[u8]) -> Result<String, ToolkitError> {
    use crate::wallet_import::coldcard::COLDCARD_PRESERVED_TOP_LEVEL_KEYS;

    let value: Value = serde_json::from_slice(blob).map_err(|e| {
        ToolkitError::ImportWalletParse(format!("canonicalize_coldcard: invalid JSON: {e}"))
    })?;
    let obj = value.as_object().ok_or_else(|| {
        ToolkitError::ImportWalletParse(
            "canonicalize_coldcard: top-level JSON value is not an object".to_string(),
        )
    })?;

    // Required fields: chain + xfp.
    if obj.get("chain").and_then(|v| v.as_str()).is_none() {
        return Err(ToolkitError::ImportWalletParse(
            "canonicalize_coldcard: missing top-level `chain`".to_string(),
        ));
    }
    if obj.get("xfp").and_then(|v| v.as_str()).is_none() {
        return Err(ToolkitError::ImportWalletParse(
            "canonicalize_coldcard: missing top-level `xfp`".to_string(),
        ));
    }

    let mut canonical: BTreeMap<String, Value> = BTreeMap::new();
    for (k, v) in obj.iter() {
        if COLDCARD_PRESERVED_TOP_LEVEL_KEYS.contains(&k.as_str()) {
            canonical.insert(k.clone(), v.clone());
        }
    }

    let mut text = serde_json::to_string_pretty(&canonical).map_err(|e| {
        ToolkitError::ImportWalletParse(format!("canonicalize_coldcard: pretty-print: {e}"))
    })?;
    text.push('\n');
    Ok(text)
}

/// SPEC §11.4 — canonicalize a Coldcard multisig text blob for semantic
/// round-trip comparison.
///
/// Strategy:
/// 1. CRLF → LF normalize + strip trailing whitespace per line + drop
///    comment lines (`# …`) + drop blank lines for diffing purposes.
/// 2. Parse via `coldcard_multisig::parse_text` to recover the typed
///    header fields + cosigner list (with effective per-cosigner XFPs
///    via the SPEC §11.4.1 truth table).
/// 3. Re-emit in a deterministic canonical form: shared-derivation shape,
///    cosigners sorted lex by xpub (mirrors the toolkit's emit at
///    `wallet_export/coldcard.rs:339` sortedmulti rule), top-level XFP
///    header DROPPED (redundant with per-cosigner `<XFP>: <xpub>` lines).
///
/// The canonicalization is **semantic, not byte-exact**: two blobs that
/// parse to the same wallet (regardless of cosigner ordering, comment
/// lines, CRLF vs LF, XFP-header presence, dash vs space in `Policy:`)
/// canonicalize to the same string.
pub(crate) fn canonicalize_coldcard_multisig(blob: &[u8]) -> Result<String, ToolkitError> {
    use crate::wallet_import::coldcard_multisig::{
        parse_text, ColdcardMsFormat,
    };

    // Re-parse via the dedicated parser; stderr WARNING (xfp divergence)
    // is swallowed for canonicalization — it does not affect the canonical
    // form (effective XFP per truth-table is what's emitted).
    let mut sink: Vec<u8> = Vec::new();
    let parsed = parse_text(blob, &mut sink)?;

    let meta = match &parsed.provenance {
        crate::wallet_import::ImportProvenance::ColdcardMultisig(m) => m,
        _ => {
            return Err(ToolkitError::ImportWalletParse(
                "canonicalize_coldcard_multisig: parser returned non-ColdcardMultisig provenance"
                    .to_string(),
            ));
        }
    };

    // Effective per-cosigner XFP from parsed.cosigners (already applied
    // the truth table). Sort by xpub lex (sortedmulti convention).
    let mut cosigner_lines: Vec<String> = parsed
        .cosigners
        .iter()
        .map(|c| {
            format!(
                "{xfp}: {xpub}",
                xfp = c.fingerprint.to_string().to_uppercase(),
                xpub = c.xpub
            )
        })
        .collect();
    cosigner_lines.sort();

    // Shared derivation path: rebuild from the first cosigner's path
    // (canonicalization ASSUMES homogeneous derivation; the parser already
    // accepted heterogeneous paths but they would canonicalize awkwardly.
    // For SPEC §11.4 the shared `Derivation:` field is the canonical form).
    let derivation_str = format!("m{}", path_components_for_canonical(&parsed.cosigners[0].path));

    let format_str = match meta.script_format {
        ColdcardMsFormat::P2wsh => "P2WSH",
        ColdcardMsFormat::P2shP2wsh => "P2SH-P2WSH",
        ColdcardMsFormat::P2sh => "P2SH",
    };

    let mut out = String::new();
    out.push_str(&format!("Name: {}\n", meta.name));
    out.push_str(&format!("Policy: {} of {}\n", meta.policy.k, meta.policy.n));
    out.push_str(&format!("Derivation: {}\n", derivation_str));
    out.push_str(&format!("Format: {}\n", format_str));
    out.push('\n');
    for line in cosigner_lines {
        out.push_str(&line);
        out.push('\n');
    }
    Ok(out)
}

/// Render a DerivationPath as `/N'/N'/...` components (the form used in
/// the `Derivation:` header value, minus the leading `m`). Mirrors the
/// path-string conversion at `wallet_import/coldcard_multisig.rs`'s
/// `derivation_path_components` but operates on the typed
/// `bitcoin::bip32::DerivationPath`.
fn path_components_for_canonical(path: &bitcoin::bip32::DerivationPath) -> String {
    use bitcoin::bip32::ChildNumber;
    let mut s = String::new();
    for comp in path.into_iter() {
        match comp {
            ChildNumber::Hardened { index } => s.push_str(&format!("/{}'", index)),
            ChildNumber::Normal { index } => s.push_str(&format!("/{}", index)),
        }
    }
    s
}

/// SPEC §11.6 — canonicalize an Electrum 4.x wallet JSON blob for semantic
/// round-trip comparison.
///
/// Strategy:
/// 1. JSON-parse + top-level object check.
/// 2. Preserve `ELECTRUM_PRESERVED_TOP_LEVEL_KEYS` (seed_version, wallet_type,
///    use_encryption, keystore) + dynamic `xN/` per-cosigner keys (matched
///    via the same predicate the parser uses).
/// 3. Re-emit via BTreeMap (alphabetical key ordering) + JSON pretty-print.
///
/// The canonicalization is **semantic, not byte-exact**: two blobs that
/// differ only in top-level key order / dropped extra fields canonicalize
/// to the same string. The `keystore` / `xN/` sub-objects are preserved
/// verbatim (their internal key order is wire-determined by Electrum's
/// Python dict serialization; we do NOT reorder them at this layer because
/// nested sub-object key order is not load-bearing for Electrum's loader).
pub(crate) fn canonicalize_electrum(blob: &[u8]) -> Result<String, ToolkitError> {
    use crate::wallet_import::electrum::{
        classify_wallet_type, ElectrumWalletType, ELECTRUM_PRESERVED_TOP_LEVEL_KEYS,
    };

    let value: Value = serde_json::from_slice(blob).map_err(|e| {
        ToolkitError::ImportWalletParse(format!("canonicalize_electrum: invalid JSON: {e}"))
    })?;
    let obj = value.as_object().ok_or_else(|| {
        ToolkitError::ImportWalletParse(
            "canonicalize_electrum: top-level JSON value is not an object".to_string(),
        )
    })?;

    // Required fields: seed_version + wallet_type.
    if obj.get("seed_version").and_then(|v| v.as_u64()).is_none() {
        return Err(ToolkitError::ImportWalletParse(
            "canonicalize_electrum: missing or non-integer top-level `seed_version`".to_string(),
        ));
    }
    let wt_str = obj.get("wallet_type").and_then(|v| v.as_str()).ok_or_else(|| {
        ToolkitError::ImportWalletParse(
            "canonicalize_electrum: missing or non-string top-level `wallet_type`".to_string(),
        )
    })?;

    // For multisig blobs, derive cosigner count `n` from wallet_type so we
    // know which `xN/` keys to preserve. For non-multisig (standard / 2fa /
    // imported / unknown), n=0 → no `xN/` keys preserved.
    let n: usize = match classify_wallet_type(wt_str) {
        Some(ElectrumWalletType::Multisig { n, .. }) => n as usize,
        _ => 0,
    };

    let mut canonical: BTreeMap<String, Value> = BTreeMap::new();
    for (k, v) in obj.iter() {
        if ELECTRUM_PRESERVED_TOP_LEVEL_KEYS.contains(&k.as_str()) {
            canonical.insert(k.clone(), v.clone());
            continue;
        }
        // Preserve `xN/` cosigner sub-objects for the matched n.
        if is_canonical_cosigner_key(k, n) {
            canonical.insert(k.clone(), v.clone());
        }
    }

    let mut text = serde_json::to_string_pretty(&canonical).map_err(|e| {
        ToolkitError::ImportWalletParse(format!("canonicalize_electrum: pretty-print: {e}"))
    })?;
    text.push('\n');
    Ok(text)
}

/// Local predicate: `k` matches `xN/` for `1 <= N <= n`. Mirrors
/// `wallet_import::electrum::is_multisig_cosigner_key` (private to that
/// module; re-implemented here to keep roundtrip.rs from depending on
/// internal helpers).
fn is_canonical_cosigner_key(k: &str, n: usize) -> bool {
    if n == 0 {
        return false;
    }
    let stripped = match k.strip_prefix('x').and_then(|s| s.strip_suffix('/')) {
        Some(s) => s,
        None => return false,
    };
    let parsed: usize = match stripped.parse() {
        Ok(v) => v,
        Err(_) => return false,
    };
    parsed >= 1 && parsed <= n
}

/// SPEC §11.5 — canonicalize a Jade multisig-file JSON wrapper.
/// Body lands in Phase P5B.
pub(crate) fn canonicalize_jade(_blob: &[u8]) -> Result<String, ToolkitError> {
    Err(ToolkitError::BadInput(
        "canonicalize_jade: not yet implemented; jade ingest lands in Phase P5B".into(),
    ))
}

/// SPEC §11.1 — canonicalize a Sparrow wallet JSON blob for semantic
/// round-trip comparison.
///
/// Strategy:
/// 1. JSON-parse + top-level object check.
/// 2. Project to a canonical alphabetical-key form (BTreeMap → serialize):
///    - Top-level keys preserved: `name`, `network`, `policyType`,
///      `scriptType`, `defaultPolicy`, `keystores` (the preserved-fields set
///      mirrors `sparrow::SPARROW_PRESERVED_TOP_LEVEL_KEYS`).
///    - All other top-level keys (Sparrow's private metadata: `birthDate`,
///      `gapLimit`, `mixConfig`, etc.) DROPPED.
///    - `defaultPolicy` rebuilt with default `name: "Default"` if absent;
///      `miniscript` rebuilt with only the `script` field preserved.
///    - Each `keystores[i]` rebuilt with only `label`, `source`,
///      `walletModel`, `keyDerivation` (which preserves only
///      `masterFingerprint` + `derivation`), and `extendedPublicKey`.
///    - `extendedPublicKey` passed through the SLIP-132 normalizer for
///      canonical neutral-prefix form.
///
/// Dropped fields (analogous to Core's `timestamp`/`next`/`next_index`):
/// any top-level key NOT in the preserved set is dropped (e.g.,
/// Sparrow's `birthDate`, `gapLimit`, `mixConfig`).
pub(crate) fn canonicalize_sparrow(blob: &[u8]) -> Result<String, ToolkitError> {
    let value: Value = serde_json::from_slice(blob).map_err(|e| {
        ToolkitError::ImportWalletParse(format!("canonicalize_sparrow: invalid JSON: {e}"))
    })?;
    let obj = value.as_object().ok_or_else(|| {
        ToolkitError::ImportWalletParse(
            "canonicalize_sparrow: top-level JSON value is not an object".to_string(),
        )
    })?;

    // BTreeMap → deterministic alphabetical key ordering at serialize time.
    let mut canonical: BTreeMap<String, Value> = BTreeMap::new();

    if let Some(name) = obj.get("name") {
        canonical.insert("name".to_string(), name.clone());
    }
    if let Some(network) = obj.get("network") {
        canonical.insert("network".to_string(), network.clone());
    }
    let policy_type = obj.get("policyType").ok_or_else(|| {
        ToolkitError::ImportWalletParse(
            "canonicalize_sparrow: missing top-level `policyType`".to_string(),
        )
    })?;
    canonical.insert("policyType".to_string(), policy_type.clone());

    let script_type = obj.get("scriptType").ok_or_else(|| {
        ToolkitError::ImportWalletParse(
            "canonicalize_sparrow: missing top-level `scriptType`".to_string(),
        )
    })?;
    canonical.insert("scriptType".to_string(), script_type.clone());

    // defaultPolicy: rebuild as canonical (alphabetical) BTreeMap nesting.
    let default_policy_obj = obj
        .get("defaultPolicy")
        .and_then(|v| v.as_object())
        .ok_or_else(|| {
            ToolkitError::ImportWalletParse(
                "canonicalize_sparrow: missing or non-object `defaultPolicy`".to_string(),
            )
        })?;
    let miniscript_obj = default_policy_obj
        .get("miniscript")
        .and_then(|v| v.as_object())
        .ok_or_else(|| {
            ToolkitError::ImportWalletParse(
                "canonicalize_sparrow: missing or non-object `defaultPolicy.miniscript`"
                    .to_string(),
            )
        })?;
    let script_str = miniscript_obj
        .get("script")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            ToolkitError::ImportWalletParse(
                "canonicalize_sparrow: missing or non-string `defaultPolicy.miniscript.script`"
                    .to_string(),
            )
        })?
        .to_string();
    let mut canonical_miniscript: BTreeMap<String, Value> = BTreeMap::new();
    canonical_miniscript.insert("script".to_string(), Value::String(script_str));
    let mut canonical_default_policy: BTreeMap<String, Value> = BTreeMap::new();
    let policy_name = default_policy_obj
        .get("name")
        .cloned()
        .unwrap_or_else(|| Value::String("Default".to_string()));
    canonical_default_policy.insert("name".to_string(), policy_name);
    canonical_default_policy.insert(
        "miniscript".to_string(),
        serde_json::to_value(&canonical_miniscript).map_err(|e| {
            ToolkitError::ImportWalletParse(format!(
                "canonicalize_sparrow: serialize miniscript: {e}"
            ))
        })?,
    );
    canonical.insert(
        "defaultPolicy".to_string(),
        serde_json::to_value(&canonical_default_policy).map_err(|e| {
            ToolkitError::ImportWalletParse(format!(
                "canonicalize_sparrow: serialize defaultPolicy: {e}"
            ))
        })?,
    );

    // keystores: rebuild each entry as canonical BTreeMap. Drop any
    // non-preserved keystore field (e.g., `passphrase`).
    let keystores = obj
        .get("keystores")
        .and_then(|v| v.as_array())
        .ok_or_else(|| {
            ToolkitError::ImportWalletParse(
                "canonicalize_sparrow: missing or non-array `keystores`".to_string(),
            )
        })?;
    let mut canonical_keystores: Vec<Value> = Vec::with_capacity(keystores.len());
    for (i, ks) in keystores.iter().enumerate() {
        let kobj = ks.as_object().ok_or_else(|| {
            ToolkitError::ImportWalletParse(format!(
                "canonicalize_sparrow: keystores[{i}] is not an object"
            ))
        })?;
        let mut ck: BTreeMap<String, Value> = BTreeMap::new();
        if let Some(v) = kobj.get("label") {
            ck.insert("label".to_string(), v.clone());
        }
        if let Some(v) = kobj.get("source") {
            ck.insert("source".to_string(), v.clone());
        }
        if let Some(v) = kobj.get("walletModel") {
            ck.insert("walletModel".to_string(), v.clone());
        }
        let key_derivation = kobj
            .get("keyDerivation")
            .and_then(|v| v.as_object())
            .ok_or_else(|| {
                ToolkitError::ImportWalletParse(format!(
                    "canonicalize_sparrow: keystores[{i}].keyDerivation missing or not object"
                ))
            })?;
        let mut ckd: BTreeMap<String, Value> = BTreeMap::new();
        if let Some(fp) = key_derivation.get("masterFingerprint") {
            ckd.insert("masterFingerprint".to_string(), fp.clone());
        }
        if let Some(d) = key_derivation.get("derivation") {
            ckd.insert("derivation".to_string(), d.clone());
        }
        ck.insert(
            "keyDerivation".to_string(),
            serde_json::to_value(&ckd).map_err(|e| {
                ToolkitError::ImportWalletParse(format!(
                    "canonicalize_sparrow: serialize keystores[{i}].keyDerivation: {e}"
                ))
            })?,
        );
        // Pass xpub through SLIP-132 normalizer for canonical neutral form.
        // Failure to normalize → preserve verbatim (canonicalize is best-
        // effort; the parse layer surfaces invalid xpubs separately).
        if let Some(xpub_str) = kobj.get("extendedPublicKey").and_then(|v| v.as_str()) {
            let neutral = match crate::slip0132::normalize_xpub_prefix(xpub_str) {
                Ok((n, _)) => n,
                Err(_) => xpub_str.to_string(),
            };
            ck.insert("extendedPublicKey".to_string(), Value::String(neutral));
        }
        canonical_keystores.push(serde_json::to_value(&ck).map_err(|e| {
            ToolkitError::ImportWalletParse(format!(
                "canonicalize_sparrow: serialize keystores[{i}]: {e}"
            ))
        })?);
    }
    canonical.insert("keystores".to_string(), Value::Array(canonical_keystores));

    let mut text = serde_json::to_string_pretty(&canonical).map_err(|e| {
        ToolkitError::ImportWalletParse(format!("canonicalize_sparrow: pretty-print: {e}"))
    })?;
    text.push('\n');
    Ok(text)
}

/// SPEC §11.2 — canonicalize a Specter-DIY wallet JSON blob.
///
/// Re-emit in alphabetical top-level key order via `BTreeMap`. Preserves the
/// four load-bearing fields (`label`, `blockheight`, `descriptor`, `devices`);
/// drops any other top-level key (parser-side `SPECTER_PRESERVED_TOP_LEVEL_KEYS`
/// surfaces dropped fields in `SpecterSourceMetadata.dropped_fields` for
/// the `--json` envelope). Mirrors `canonicalize_sparrow` shape +
/// `canonicalize_bitcoin_core` shape (top-level alphabetical, nested
/// preserved-key projection).
pub(crate) fn canonicalize_specter(blob: &[u8]) -> Result<String, ToolkitError> {
    let value: Value = serde_json::from_slice(blob).map_err(|e| {
        ToolkitError::ImportWalletParse(format!("canonicalize_specter: invalid JSON: {e}"))
    })?;
    let obj = value.as_object().ok_or_else(|| {
        ToolkitError::ImportWalletParse(
            "canonicalize_specter: top-level JSON value is not an object".to_string(),
        )
    })?;

    let mut canonical: BTreeMap<String, Value> = BTreeMap::new();

    let label = obj.get("label").ok_or_else(|| {
        ToolkitError::ImportWalletParse(
            "canonicalize_specter: missing top-level `label`".to_string(),
        )
    })?;
    canonical.insert("label".to_string(), label.clone());

    let blockheight = obj.get("blockheight").ok_or_else(|| {
        ToolkitError::ImportWalletParse(
            "canonicalize_specter: missing top-level `blockheight`".to_string(),
        )
    })?;
    canonical.insert("blockheight".to_string(), blockheight.clone());

    let descriptor = obj.get("descriptor").ok_or_else(|| {
        ToolkitError::ImportWalletParse(
            "canonicalize_specter: missing top-level `descriptor`".to_string(),
        )
    })?;
    canonical.insert("descriptor".to_string(), descriptor.clone());

    let devices = obj.get("devices").ok_or_else(|| {
        ToolkitError::ImportWalletParse(
            "canonicalize_specter: missing top-level `devices`".to_string(),
        )
    })?;
    // Preserve devices verbatim (both legacy string-form + modern object-form
    // are intrinsic to the blob shape; the parser normalizes them, but
    // canonicalize re-emits the on-disk shape).
    canonical.insert("devices".to_string(), devices.clone());

    let mut text = serde_json::to_string_pretty(&canonical).map_err(|e| {
        ToolkitError::ImportWalletParse(format!("canonicalize_specter: pretty-print: {e}"))
    })?;
    text.push('\n');
    Ok(text)
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
    // canonicalize_coldcard cells (Phase P3B)
    // =========================================================================

    /// SPEC §11.3 — canonicalize_coldcard on a bip84 fixture produces a JSON
    /// object with alphabetically-sorted top-level keys + trailing newline.
    /// Preserved-field set: chain, xfp, xpub, account, bip44, bip49, bip84,
    /// bip86, bip48_1, bip48_2.
    #[test]
    fn canonicalize_coldcard_preserves_required_fields() {
        let blob = std::fs::read(
            "tests/fixtures/wallet_import/coldcard-singlesig-bip84-mainnet.json",
        )
        .expect("bip84 fixture readable");
        let canonical = canonicalize_coldcard(&blob).unwrap();
        assert!(canonical.contains("\"chain\":"));
        assert!(canonical.contains("\"xfp\":"));
        assert!(canonical.contains("\"bip84\":"));
        assert!(canonical.contains("\"account\":"));
        assert!(canonical.ends_with('\n'), "trailing newline required");
    }

    /// canonicalize_coldcard is idempotent — re-canonicalize equals canonicalize.
    #[test]
    fn canonicalize_coldcard_idempotent() {
        let blob = std::fs::read(
            "tests/fixtures/wallet_import/coldcard-singlesig-bip84-mainnet.json",
        )
        .expect("fixture readable");
        let once = canonicalize_coldcard(&blob).unwrap();
        let twice = canonicalize_coldcard(once.as_bytes()).unwrap();
        assert_eq!(once, twice, "canonicalize_coldcard must be idempotent");
    }

    /// canonicalize_coldcard drops non-preserved top-level fields.
    #[test]
    fn canonicalize_coldcard_drops_non_preserved_top_level_fields() {
        let blob = br#"{
            "chain":"BTC","xfp":"B8688DF1","account":0,
            "bip84":{"name":"p2wpkh","deriv":"m/84'/0'/0'","xfp":"B8688DF1","xpub":"xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX"},
            "this_field_dropped":"x",
            "and_this_too":42
        }"#;
        let canonical = canonicalize_coldcard(blob).unwrap();
        assert!(!canonical.contains("this_field_dropped"));
        assert!(!canonical.contains("and_this_too"));
        assert!(canonical.contains("\"chain\":"));
    }

    /// canonicalize_coldcard reorders top-level keys alphabetically. The
    /// preserved key set is {account, bip44, bip49, bip48_1, bip48_2, bip84,
    /// bip86, chain, xfp, xpub} → alphabetical order moves `chain` after
    /// `bip86` etc. Two blobs with the same content but different on-disk
    /// key ordering canonicalize identically.
    #[test]
    fn canonicalize_coldcard_alphabetizes_keys_across_orderings() {
        let blob_order_a = br#"{
            "chain":"BTC","xfp":"B8688DF1","account":0,
            "bip84":{"name":"p2wpkh","deriv":"m/84'/0'/0'","xfp":"B8688DF1","xpub":"xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX"}
        }"#;
        // Same fields, different on-disk top-level order.
        let blob_order_b = br#"{
            "bip84":{"name":"p2wpkh","deriv":"m/84'/0'/0'","xfp":"B8688DF1","xpub":"xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX"},
            "account":0,"xfp":"B8688DF1","chain":"BTC"
        }"#;
        let canon_a = canonicalize_coldcard(blob_order_a).unwrap();
        let canon_b = canonicalize_coldcard(blob_order_b).unwrap();
        assert_eq!(canon_a, canon_b, "ordering-only differences must canonicalize identically");
    }

    /// canonicalize_coldcard refuses invalid JSON with parse-class error.
    #[test]
    fn canonicalize_coldcard_invalid_json_returns_parse_error() {
        let err = canonicalize_coldcard(b"{not json").unwrap_err();
        match err {
            ToolkitError::ImportWalletParse(_) => {}
            other => panic!("expected ImportWalletParse, got: {other:?}"),
        }
    }

    /// canonicalize_coldcard refuses missing-chain with parse-class error.
    #[test]
    fn canonicalize_coldcard_missing_chain_refused() {
        let blob = br#"{"xfp":"B8688DF1","bip84":{"name":"p2wpkh","xpub":"xpub..."}}"#;
        let err = canonicalize_coldcard(blob).unwrap_err();
        match err {
            ToolkitError::ImportWalletParse(msg) => {
                assert!(msg.contains("chain"), "msg must cite chain; got: {msg}");
            }
            other => panic!("expected ImportWalletParse, got: {other:?}"),
        }
    }

    /// P4B canonicalize: round-trip a fixture and verify the canonical form
    /// is stable (canonicalize-of-canonicalize == canonicalize). The skeleton
    /// regression cell that previously sat here is replaced — the body is
    /// no longer a stub.
    #[test]
    fn canonicalize_coldcard_multisig_idempotent() {
        let blob = std::fs::read(
            "tests/fixtures/wallet_import/coldcard-ms-2of3-p2wsh-with-xfp.txt",
        )
        .expect("fixture file readable");
        let c1 = canonicalize_coldcard_multisig(&blob).unwrap();
        let c2 = canonicalize_coldcard_multisig(c1.as_bytes()).unwrap();
        assert_eq!(c1, c2, "canonicalize_coldcard_multisig must be idempotent");
    }

    /// P4B canonicalize: with-XFP-header fixture and without-XFP-header
    /// fixture canonicalize to the same string (header dropped; same
    /// per-cosigner XFPs + same cosigners + same headers otherwise).
    #[test]
    fn canonicalize_coldcard_multisig_with_and_without_xfp_header_match() {
        let with_blob = std::fs::read(
            "tests/fixtures/wallet_import/coldcard-ms-2of3-p2wsh-with-xfp.txt",
        )
        .expect("with-xfp fixture readable");
        let without_blob = std::fs::read(
            "tests/fixtures/wallet_import/coldcard-ms-2of3-p2wsh-no-xfp.txt",
        )
        .expect("no-xfp fixture readable");
        let c_with = canonicalize_coldcard_multisig(&with_blob).unwrap();
        let c_without = canonicalize_coldcard_multisig(&without_blob).unwrap();
        assert_eq!(
            c_with, c_without,
            "with-XFP-header and without-XFP-header fixtures must canonicalize identically"
        );
    }

    /// P4B canonicalize: 3-of-5 fixture canonicalizes to a stable form
    /// containing all 5 cosigners (sorted lex by xpub).
    #[test]
    fn canonicalize_coldcard_multisig_3of5_stable() {
        let blob = std::fs::read("tests/fixtures/wallet_import/coldcard-ms-3of5-p2wsh.txt")
            .expect("3of5 fixture readable");
        let c = canonicalize_coldcard_multisig(&blob).unwrap();
        assert!(c.starts_with("Name: TestMs3of5\n"));
        assert!(c.contains("Policy: 3 of 5\n"));
        assert!(c.contains("Format: P2WSH\n"));
        assert!(c.contains("Derivation: m/48'/0'/0'/2'\n"));
        // Counts of XFP-prefixed cosigner lines == 5.
        let cosigner_line_count = c.lines().filter(|l| l.contains(": xpub")).count();
        assert_eq!(cosigner_line_count, 5, "got:\n{c}");
    }

    /// P4B canonicalize: comment lines + CRLF + dash-form Policy are all
    /// stripped/normalized — three variants of the same underlying wallet
    /// canonicalize identically.
    #[test]
    fn canonicalize_coldcard_multisig_cosmetic_variants_match() {
        let xpub_a = "xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX";
        let xpub_b = "xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6";
        let xpub_c = "xpub6Buxw9MmbkJr4iAw8SACNci2hQNuPCMwt9P7HkK62ZQAW9UcJaQ2bc6ARD892TToQQ9Rp6AHujHxBLXqAsvn5fRnLfnhKSRfz8qtaoyKUYx";

        let plain = format!(
            "Name: T\n\
Policy: 2 of 3\n\
Derivation: m/48'/0'/0'/2'\n\
Format: P2WSH\n\
\n\
34A3A4F1: {xpub_a}\n\
FF9DFBCF: {xpub_b}\n\
B7F7DFEA: {xpub_c}\n"
        );
        let with_comments = format!(
            "# exported from Coldcard\n\
# wallet xfp = 34A3A4F1\n\
Name: T\n\
Policy: 2-of-3\n\
Derivation: m/48'/0'/0'/2'\n\
Format: P2WSH\n\
\n\
34A3A4F1: {xpub_a}\n\
FF9DFBCF: {xpub_b}\n\
B7F7DFEA: {xpub_c}\n"
        );
        let crlf = plain.replace('\n', "\r\n");

        let c_plain = canonicalize_coldcard_multisig(plain.as_bytes()).unwrap();
        let c_comments = canonicalize_coldcard_multisig(with_comments.as_bytes()).unwrap();
        let c_crlf = canonicalize_coldcard_multisig(crlf.as_bytes()).unwrap();
        assert_eq!(c_plain, c_comments);
        assert_eq!(c_plain, c_crlf);
    }

    /// P4B canonicalize: invalid blob surfaces as `ImportWalletParse` (NOT
    /// the prior P0C-stub `BadInput`); the helper signature contract per
    /// SPEC §7-style helpers (canonicalize errors are parse-class).
    #[test]
    fn canonicalize_coldcard_multisig_invalid_blob_returns_parse_error() {
        let err = canonicalize_coldcard_multisig(b"not a coldcard ms file").unwrap_err();
        match err {
            ToolkitError::ImportWalletParse(_) => {}
            other => panic!("expected ImportWalletParse, got: {other:?}"),
        }
    }

    // ===========================================================================
    // canonicalize_sparrow cells (Phase P1B)
    // ===========================================================================

    /// SPEC §11.1 — canonicalize_sparrow on a minimal SINGLE wallet produces
    /// a JSON object with alphabetically-sorted top-level keys. Verifies the
    /// preserved-field set: name, network, policyType, scriptType,
    /// defaultPolicy, keystores.
    #[test]
    fn canonicalize_sparrow_single_preserves_required_fields() {
        let blob = br#"{
            "name":"bip84-0","network":"mainnet","policyType":"SINGLE","scriptType":"P2WPKH",
            "defaultPolicy":{"name":"Default","miniscript":{"script":"wpkh(@0/**)"}},
            "keystores":[{
                "label":"bip84-0","source":"SW_WATCH","walletModel":"SPARROW",
                "keyDerivation":{"masterFingerprint":"5436d724","derivation":"m/84'/0'/0'"},
                "extendedPublicKey":"xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9"
            }]
        }"#;
        let canonical = canonicalize_sparrow(blob).unwrap();
        // All required top-level keys present.
        for key in ["name", "network", "policyType", "scriptType", "defaultPolicy", "keystores"] {
            assert!(canonical.contains(&format!("\"{key}\"")), "missing key {key}: {canonical}");
        }
        // Round-trip via serde_json to verify the canonical form is itself
        // valid JSON with the required top-level keys in a serde_json::Map
        // (which preserves insertion order from BTreeMap → alphabetical).
        let parsed: serde_json::Value =
            serde_json::from_str(&canonical).expect("canonical output must be valid JSON");
        let obj = parsed.as_object().expect("top-level must be object");
        let top_level_keys: Vec<&str> = obj.keys().map(|s| s.as_str()).collect();
        assert_eq!(
            top_level_keys,
            vec!["defaultPolicy", "keystores", "name", "network", "policyType", "scriptType"],
            "top-level keys must be alphabetically ordered in canonical form"
        );
    }

    /// SPEC §11.1 — canonicalize_sparrow is idempotent (re-canonicalize
    /// produces byte-identical output).
    #[test]
    fn canonicalize_sparrow_idempotent() {
        let blob = br#"{
            "name":"bip84-0","network":"mainnet","policyType":"SINGLE","scriptType":"P2WPKH",
            "defaultPolicy":{"name":"Default","miniscript":{"script":"wpkh(@0/**)"}},
            "keystores":[{
                "label":"bip84-0","source":"SW_WATCH","walletModel":"SPARROW",
                "keyDerivation":{"masterFingerprint":"5436d724","derivation":"m/84'/0'/0'"},
                "extendedPublicKey":"xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9"
            }]
        }"#;
        let once = canonicalize_sparrow(blob).unwrap();
        let twice = canonicalize_sparrow(once.as_bytes()).unwrap();
        assert_eq!(once, twice, "canonicalize_sparrow must be idempotent");
    }

    /// SPEC §11.1 — extra top-level fields (Sparrow's `birthDate`,
    /// `gapLimit`) are DROPPED from the canonical form.
    #[test]
    fn canonicalize_sparrow_drops_non_preserved_top_level_fields() {
        let blob = br#"{
            "name":"x","network":"mainnet","policyType":"SINGLE","scriptType":"P2WPKH",
            "defaultPolicy":{"name":"Default","miniscript":{"script":"wpkh(@0/**)"}},
            "keystores":[{
                "label":"x","source":"SW_WATCH","walletModel":"SPARROW",
                "keyDerivation":{"masterFingerprint":"5436d724","derivation":"m/84'/0'/0'"},
                "extendedPublicKey":"xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9"
            }],
            "birthDate":1717000000,
            "gapLimit":20,
            "mixConfig":{"mixers":[]}
        }"#;
        let canonical = canonicalize_sparrow(blob).unwrap();
        assert!(!canonical.contains("birthDate"), "birthDate must be dropped: {canonical}");
        assert!(!canonical.contains("gapLimit"), "gapLimit must be dropped: {canonical}");
        assert!(!canonical.contains("mixConfig"), "mixConfig must be dropped: {canonical}");
    }

    /// SPEC §11.1 — multisig canonicalize preserves all keystores entries in
    /// declaration order with alphabetical sub-key ordering.
    #[test]
    fn canonicalize_sparrow_multi_preserves_keystore_ordering() {
        let blob = br#"{
            "name":"wsh-sortedmulti-0","network":"mainnet","policyType":"MULTI","scriptType":"P2WSH",
            "defaultPolicy":{"name":"Default","miniscript":{"script":"wsh(sortedmulti(2,@0/**,@1/**,@2/**))"}},
            "keystores":[
                {"label":"k1","source":"SW_WATCH","walletModel":"SPARROW",
                 "keyDerivation":{"masterFingerprint":"b8688df1","derivation":"m/48'/0'/0'/2'"},
                 "extendedPublicKey":"xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX"},
                {"label":"k2","source":"SW_WATCH","walletModel":"SPARROW",
                 "keyDerivation":{"masterFingerprint":"28645006","derivation":"m/48'/0'/0'/2'"},
                 "extendedPublicKey":"xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6"},
                {"label":"k3","source":"SW_WATCH","walletModel":"SPARROW",
                 "keyDerivation":{"masterFingerprint":"5436d724","derivation":"m/48'/0'/0'/2'"},
                 "extendedPublicKey":"xpub6Buxw9MmbkJr4iAw8SACNci2hQNuPCMwt9P7HkK62ZQAW9UcJaQ2bc6ARD892TToQQ9Rp6AHujHxBLXqAsvn5fRnLfnhKSRfz8qtaoyKUYx"}
            ]
        }"#;
        let canonical = canonicalize_sparrow(blob).unwrap();
        // All 3 fingerprints present in declaration order.
        let p1 = canonical.find("b8688df1").expect("k1 fp");
        let p2 = canonical.find("28645006").expect("k2 fp");
        let p3 = canonical.find("5436d724").expect("k3 fp");
        assert!(p1 < p2 && p2 < p3, "keystore ordering must be preserved: {canonical}");
    }

    /// SPEC §11.1 — malformed JSON returns ImportWalletParse.
    #[test]
    fn canonicalize_sparrow_malformed_json_typed_error() {
        let err = canonicalize_sparrow(b"not json").unwrap_err();
        assert!(matches!(err, ToolkitError::ImportWalletParse(ref m) if m.contains("invalid JSON")));
    }

    /// SPEC §11.1 — bare-array top-level returns ImportWalletParse.
    #[test]
    fn canonicalize_sparrow_bare_array_typed_error() {
        let err = canonicalize_sparrow(b"[]").unwrap_err();
        assert!(matches!(err, ToolkitError::ImportWalletParse(ref m) if m.contains("top-level JSON value is not an object")));
    }

    // ========================================================================
    // canonicalize_electrum (P6B): real body cells. The pre-P6B skeleton-shape
    // test (`canonicalize_electrum_skeleton_returns_not_yet_implemented`) is
    // replaced by the cells below — the body now mirrors `canonicalize_coldcard`
    // (BTreeMap-backed alphabetical key reorder + 4 preserved top-level fields
    // + dynamic `xN/` cosigner keys for multisig).
    // ========================================================================

    #[test]
    fn canonicalize_electrum_standard_reorders_alphabetically() {
        let src = br#"{
            "wallet_type": "standard",
            "seed_version": 17,
            "use_encryption": false,
            "keystore": {"type": "bip32", "xpub": "zpub6qTBTNftBzVTjgVcSUw7vW5N1KQbV93Jnrw314RHGkCkSx4vk6nEWH1MJfReXi2WThvuDRiRpyT7cDoakEcZMQ1iZPgfJgQrcVMR4aJWh6S", "derivation": "m/84'/0'/0'", "root_fingerprint": "5436d724", "label": "Daily"}
        }"#;
        let canon = canonicalize_electrum(src).unwrap();
        // Alphabetical: keystore, seed_version, use_encryption, wallet_type.
        let keystore_idx = canon.find("\"keystore\"").unwrap();
        let seedv_idx = canon.find("\"seed_version\"").unwrap();
        let useenc_idx = canon.find("\"use_encryption\"").unwrap();
        let wtype_idx = canon.find("\"wallet_type\"").unwrap();
        assert!(keystore_idx < seedv_idx);
        assert!(seedv_idx < useenc_idx);
        assert!(useenc_idx < wtype_idx);
    }

    #[test]
    fn canonicalize_electrum_drops_extra_top_level_fields() {
        let src = br#"{
            "seed_version": 17,
            "wallet_type": "standard",
            "use_encryption": false,
            "keystore": {"type": "bip32", "xpub": "zpub...", "derivation": "m/84'/0'/0'", "root_fingerprint": "5436d724"},
            "addresses": {"receiving": ["bc1q..."]},
            "labels": {"tx1": "label"}
        }"#;
        let canon = canonicalize_electrum(src).unwrap();
        assert!(
            !canon.contains("addresses") && !canon.contains("labels"),
            "extra top-level fields must be dropped; got: {canon}"
        );
    }

    #[test]
    fn canonicalize_electrum_multisig_preserves_xn_keys() {
        let src = br#"{
            "seed_version": 17,
            "wallet_type": "2of3",
            "use_encryption": false,
            "x1/": {"xpub": "Zpub1", "derivation": "m/48'/0'/0'/2'"},
            "x2/": {"xpub": "Zpub2", "derivation": "m/48'/0'/0'/2'"},
            "x3/": {"xpub": "Zpub3", "derivation": "m/48'/0'/0'/2'"}
        }"#;
        let canon = canonicalize_electrum(src).unwrap();
        assert!(canon.contains("\"x1/\""), "x1/ must be preserved; got: {canon}");
        assert!(canon.contains("\"x2/\""), "x2/ must be preserved; got: {canon}");
        assert!(canon.contains("\"x3/\""), "x3/ must be preserved; got: {canon}");
    }

    #[test]
    fn canonicalize_electrum_multisig_drops_xn_keys_above_n() {
        // wallet_type "2of3" → preserve x1/, x2/, x3/ only. An out-of-band x4/
        // (e.g., user added a phantom cosigner manually) is dropped.
        let src = br#"{
            "seed_version": 17,
            "wallet_type": "2of3",
            "use_encryption": false,
            "x1/": {"xpub": "Zpub1"},
            "x2/": {"xpub": "Zpub2"},
            "x3/": {"xpub": "Zpub3"},
            "x4/": {"xpub": "Zpub4-phantom"}
        }"#;
        let canon = canonicalize_electrum(src).unwrap();
        assert!(canon.contains("\"x3/\""));
        assert!(!canon.contains("\"x4/\""), "phantom x4/ must be dropped; got: {canon}");
    }

    #[test]
    fn canonicalize_electrum_invalid_json_returns_parse_error() {
        let err = canonicalize_electrum(b"{not json").unwrap_err();
        match err {
            ToolkitError::ImportWalletParse(msg) => {
                assert!(msg.contains("invalid JSON"), "msg must cite invalid JSON; got: {msg}");
            }
            other => panic!("expected ImportWalletParse, got: {other:?}"),
        }
    }

    #[test]
    fn canonicalize_electrum_missing_seed_version_returns_parse_error() {
        let src = br#"{"wallet_type": "standard"}"#;
        let err = canonicalize_electrum(src).unwrap_err();
        match err {
            ToolkitError::ImportWalletParse(msg) => {
                assert!(msg.contains("seed_version"), "msg must cite seed_version; got: {msg}");
            }
            other => panic!("expected ImportWalletParse, got: {other:?}"),
        }
    }

    #[test]
    fn canonicalize_electrum_ends_with_trailing_newline() {
        let src = br#"{"seed_version":17,"wallet_type":"standard","use_encryption":false,"keystore":{}}"#;
        let canon = canonicalize_electrum(src).unwrap();
        assert!(canon.ends_with('\n'), "canonical form must end with trailing newline");
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

    // ========================================================================
    // canonicalize_specter (P2B): real body cells. The pre-P2B skeleton-shape
    // test (`canonicalize_specter_skeleton_returns_not_yet_implemented`) is
    // replaced by the cells below — the body now mirrors `canonicalize_sparrow`
    // (BTreeMap-backed alphabetical key reorder + 4 preserved top-level fields).
    // ========================================================================

    #[test]
    fn canonicalize_specter_reorders_top_level_keys_alphabetically() {
        // Source order: label, blockheight, descriptor, devices (already
        // alphabetical — but Specter Desktop's wire shape often differs).
        // BTreeMap pretty-print emits keys in alphabetical order regardless
        // of source-blob order.
        let src = br#"{
  "descriptor": "wpkh([5436d724/84'/0'/0']xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9/<0;1>/*)#00lx6ere",
  "label": "Daily",
  "blockheight": 800000,
  "devices": ["unknown"]
}"#;
        let canon = canonicalize_specter(src).unwrap();
        // First key in canonical form is alphabetical-first: "blockheight".
        let bh_idx = canon.find("\"blockheight\"").unwrap();
        let descriptor_idx = canon.find("\"descriptor\"").unwrap();
        let devices_idx = canon.find("\"devices\"").unwrap();
        let label_idx = canon.find("\"label\"").unwrap();
        assert!(bh_idx < descriptor_idx);
        assert!(descriptor_idx < devices_idx);
        assert!(devices_idx < label_idx);
    }

    #[test]
    fn canonicalize_specter_drops_extra_top_level_fields() {
        let src = br#"{
            "label":"x","blockheight":0,
            "descriptor":"wpkh([5436d724/84'/0'/0']xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9/<0;1>/*)#00lx6ere",
            "devices":["unknown"],
            "extra_metadata":"this should be dropped"
        }"#;
        let canon = canonicalize_specter(src).unwrap();
        assert!(
            !canon.contains("extra_metadata"),
            "extra_metadata must be dropped from canonical form; got: {canon}"
        );
    }

    #[test]
    fn canonicalize_specter_invalid_json_returns_parse_error() {
        let err = canonicalize_specter(b"{not json").unwrap_err();
        match err {
            ToolkitError::ImportWalletParse(msg) => {
                assert!(msg.contains("specter"), "msg must cite format; got: {msg}");
                assert!(msg.contains("invalid JSON"), "msg must cite JSON shape; got: {msg}");
            }
            other => panic!("expected ImportWalletParse, got: {other:?}"),
        }
    }

    #[test]
    fn canonicalize_specter_missing_descriptor_returns_parse_error() {
        let src = br#"{"label":"x","blockheight":0,"devices":[]}"#;
        let err = canonicalize_specter(src).unwrap_err();
        match err {
            ToolkitError::ImportWalletParse(msg) => {
                assert!(
                    msg.contains("descriptor"),
                    "msg must cite missing descriptor; got: {msg}"
                );
            }
            other => panic!("expected ImportWalletParse, got: {other:?}"),
        }
    }

    #[test]
    fn canonicalize_specter_ends_with_trailing_newline() {
        let src = br#"{"label":"x","blockheight":0,"descriptor":"wpkh([5436d724/84'/0'/0']xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9/<0;1>/*)#00lx6ere","devices":[]}"#;
        let canon = canonicalize_specter(src).unwrap();
        assert!(canon.ends_with('\n'), "canonical form must end with trailing newline; got tail: {:?}", &canon[canon.len().saturating_sub(5)..]);
    }

    #[test]
    fn skeleton_canonicalize_helpers_accept_empty_blob() {
        // Empty blob is a degenerate input; remaining skeletons must still
        // return the BadInput("not yet implemented") shape (not panic, not
        // Ok, not a different error class). This pins the "shape-only"
        // contract.
        //
        // Note: `coldcard`, `coldcard-multisig`, `electrum`, `sparrow`,
        // `specter` are OMITTED from this list — their canonicalize bodies
        // are no longer skeletons (P3B, P4B, P6B, P1B, P2B respectively).
        // Other format skeletons stay on this list until their per-parser
        // P{N}B phase.
        //
        // With only Jade remaining as a skeleton at v0.28.0 Phase P6C, this
        // loop iterates a single element — preserved as a `for` shape (not
        // collapsed to a direct call) so the per-format roll-call discipline
        // is mechanical when P5B lands.
        #[allow(clippy::single_element_loop)]
        for (name, result) in [
            ("jade", canonicalize_jade(b"")),
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
