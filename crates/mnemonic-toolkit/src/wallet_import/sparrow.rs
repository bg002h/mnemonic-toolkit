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
//! Parse strategy:
//! - The `defaultPolicy.miniscript.script` field already carries the FULL
//!   descriptor body with `wpkh(...)` / `wsh(...)` / `sh(wsh(...))` / `tr(...)`
//!   wrapping (Sparrow's emit at `wallet_export/sparrow.rs:185-220` builds the
//!   wrapped form per `CliTemplate`). The script uses `@N/**` cosigner
//!   placeholders (BIP-389 multipath shorthand) — NOT the
//!   `[fp/path]xpub/<0;1>/*` concrete-keys form.
//! - For each `keystores[i]`, substitute `@i/**` with
//!   `[fp/derivation_no_m_prefix]xpub/<0;1>/*` to produce a concrete-keys
//!   descriptor matching `pipeline::concrete_keys_to_placeholders`'s input
//!   contract.
//! - Feed the substituted descriptor through the same `concrete_keys_to_
//!   placeholders` → `parse_descriptor::parse_descriptor` pipeline that BSMS
//!   and Bitcoin Core use. Network detection mirrors BSMS (`network_from_
//!   origins` via BIP-48 coin-type on first cosigner origin path).
//! - Taproot import (v0.31.1 + v0.31.2): Sparrow's taproot emit covers
//!   both shapes — taproot MULTISIG (`tr(NUMS, multi_a(...))` /
//!   `tr(NUMS, sortedmulti_a(...))`) ships as descriptor-passthrough with
//!   concrete `[fp/path]xpub` keys embedded directly; taproot SINGLESIG
//!   (Bip86: `tr(@0/**)`) ships in template-mode with placeholder.
//!   v0.31.1 Cycle 8 introduced the Step 6 path-split for the
//!   descriptor-passthrough branch (`has_tr && !has_at_placeholder` →
//!   skip Step 5 substitution; feed `script_template` directly).
//!   v0.31.2 Cycle 9 collapsed the taproot-singlesig narrow refusal into
//!   the general template-mode substitution branch (the existing Step 5
//!   loop handles `tr(@0/**)` → `tr([fp/86'/0'/0']xpub/<0;1>/*)` cleanly).

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
/// (P1C wiring).
#[derive(Debug, Clone)]
pub(crate) struct SparrowSourceMetadata {
    pub(crate) label: Option<String>,
    pub(crate) policy_type: SparrowPolicyType,
    pub(crate) script_type: String,
    pub(crate) dropped_fields: Vec<String>,
}

/// SPEC §11.1 — Sparrow's `policyType` discriminant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SparrowPolicyType {
    Single,
    Multi,
}

impl SparrowPolicyType {
    pub(crate) fn from_str(s: &str) -> Option<Self> {
        match s {
            "SINGLE" => Some(Self::Single),
            "MULTI" => Some(Self::Multi),
            _ => None,
        }
    }
}

/// Fields the toolkit's `ParsedImport` preserves verbatim from a Sparrow blob.
/// Any top-level / nested envelope field not in this set goes into
/// `SparrowSourceMetadata.dropped_fields` and drives a stderr NOTICE per
/// SPEC §2.4. Top-level `name`/`network`/`policyType`/`scriptType`/
/// `defaultPolicy`/`keystores` are preserved (`name` lives on metadata.label,
/// the rest are intrinsic to provenance + descriptor); other top-level fields
/// (Sparrow private metadata: `birthDate`, `gapLimit`, `mixConfig`, etc.) are
/// surfaced as dropped.
const SPARROW_PRESERVED_TOP_LEVEL_KEYS: &[&str] = &[
    "name",
    "network",
    "policyType",
    "scriptType",
    "defaultPolicy",
    "keystores",
];

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

    /// SPEC §11.1 — parse a Sparrow wallet JSON blob.
    ///
    /// Steps:
    /// 1. JSON-parse + top-level object check.
    /// 2. Extract envelope fields (`name`, `policyType`, `scriptType`,
    ///    `defaultPolicy.miniscript.script`, `keystores`).
    /// 3. Validate `policyType` ↔ `keystores.len()`: SINGLE requires N=1;
    ///    MULTI requires N≥2.
    /// 4. Per-keystore: extract `masterFingerprint`, `derivation`, `xpub`.
    /// 5. Substitute `@i/**` → `[fp/derivation_no_m]xpub/<0;1>/*` in the
    ///    miniscript script (skipped under v0.31.1 descriptor-passthrough
    ///    mode where `script_template` already carries concrete keys).
    /// 6. Path-split (v0.31.1 + v0.31.2): `has_tr && !has_at_placeholder`
    ///    = descriptor-passthrough (taproot multisig; skip Step 5).
    ///    Otherwise = substitute. Taproot singlesig (Bip86 `tr(@0/**)`)
    ///    takes the substitution branch (v0.31.2 Cycle 9).
    /// 7. Feed through `concrete_keys_to_placeholders` → `parse_descriptor`.
    /// 8. Build `ResolvedSlot` cosigners with origin + xpub typed values.
    /// 9. Emit stderr NOTICE per SPEC §2.4 listing dropped envelope fields.
    /// 10. Wrap in `ParsedImport` with `ImportProvenance::Sparrow(...)`.
    fn parse(blob: &[u8], stderr: &mut dyn Write) -> Result<Vec<ParsedImport>, ToolkitError> {
        // Step 1: JSON parse.
        let value: Value = serde_json::from_slice(blob).map_err(|e| {
            ToolkitError::ImportWalletParse(format!(
                "import-wallet: sparrow: parse error: invalid JSON: {e}"
            ))
        })?;
        let obj = value.as_object().ok_or_else(|| {
            ToolkitError::ImportWalletParse(
                "import-wallet: sparrow: parse error: top-level JSON value is not an object"
                    .to_string(),
            )
        })?;

        // Step 2: envelope-field extraction.
        let name_opt = obj.get("name").and_then(|v| v.as_str()).map(String::from);
        let policy_type_str = obj
            .get("policyType")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                ToolkitError::ImportWalletParse(
                    "import-wallet: sparrow: parse error: missing or non-string top-level `policyType`".to_string(),
                )
            })?;
        let policy_type = SparrowPolicyType::from_str(policy_type_str).ok_or_else(|| {
            ToolkitError::ImportWalletParse(format!(
                "import-wallet: sparrow: parse error: `policyType` must be \"SINGLE\" or \"MULTI\", got {policy_type_str:?}"
            ))
        })?;
        let script_type = obj
            .get("scriptType")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                ToolkitError::ImportWalletParse(
                    "import-wallet: sparrow: parse error: missing or non-string top-level `scriptType`".to_string(),
                )
            })?
            .to_string();
        let script_template = obj
            .get("defaultPolicy")
            .and_then(|v| v.as_object())
            .and_then(|m| m.get("miniscript"))
            .and_then(|v| v.as_object())
            .and_then(|m| m.get("script"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                ToolkitError::ImportWalletParse(
                    "import-wallet: sparrow: parse error: missing or non-string `defaultPolicy.miniscript.script`".to_string(),
                )
            })?
            .to_string();
        let keystores_arr = obj
            .get("keystores")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                ToolkitError::ImportWalletParse(
                    "import-wallet: sparrow: parse error: missing or non-array top-level `keystores`".to_string(),
                )
            })?;
        if keystores_arr.is_empty() {
            return Err(ToolkitError::ImportWalletParse(
                "import-wallet: sparrow: parse error: `keystores` array is empty".to_string(),
            ));
        }

        // Step 3: policyType ↔ keystores.len() consistency.
        let n = keystores_arr.len();
        match (policy_type, n) {
            (SparrowPolicyType::Single, 1) => {}
            (SparrowPolicyType::Single, other) => {
                return Err(ToolkitError::ImportWalletParse(format!(
                    "import-wallet: sparrow: parse error: `policyType` is SINGLE but `keystores` has {other} entries (expected 1)"
                )));
            }
            (SparrowPolicyType::Multi, other) if other < 2 => {
                return Err(ToolkitError::ImportWalletParse(format!(
                    "import-wallet: sparrow: parse error: `policyType` is MULTI but `keystores` has only {other} entries (expected ≥2)"
                )));
            }
            (SparrowPolicyType::Multi, _) => {}
        }

        // Step 4: per-keystore field extraction.
        let mut keystores: Vec<KeystoreParts> = Vec::with_capacity(n);
        for (i, ks) in keystores_arr.iter().enumerate() {
            keystores.push(parse_keystore(i, ks)?);
        }

        // Step 6 (v0.31.2 Cycle 9): full taproot import via path-split.
        //
        // Sparrow's emit at `wallet_export/sparrow.rs` ships taproot in TWO
        // shapes:
        // - MULTISIG (`tr-multi-a` / `tr-sortedmulti-a`) → DESCRIPTOR-
        //   PASSTHROUGH: concrete `[fp/path]xpub` keys embedded in
        //   `script_template` directly (no `@N/**` placeholders). Skip
        //   Step 5 substitution; feed `script_template` directly.
        // - SINGLESIG (`Bip86`: `tr(@0/**)`) → TEMPLATE-MODE: standard
        //   `@N/**` placeholder substitution (v0.31.2; collapsed Cycle 8's
        //   narrow refusal). Produces `tr([fp/86'/0'/0']xpub/<0;1>/*)`
        //   which `concrete_keys_to_placeholders` + `parse_descriptor`
        //   accept cleanly.
        //
        // Detection:
        //   has_tr             = script_template.contains("tr(")
        //   has_at_placeholder = regex `@\d+/**` (any digit index)
        //   descriptor-passthrough (taproot multisig) = has_tr && !has_at_placeholder
        //                       → skip Step 5 substitution; feed script_template directly.
        //   otherwise (template-mode; non-taproot OR taproot singlesig)
        //                       → run Step 5 substitution.
        //
        // v0.31.4 (Cycle 11) defensive widening: `has_at_placeholder`
        // was a literal substring check for `@0/**`. Sparrow's current
        // emit at `wallet_export/sparrow.rs:230` always indexes from
        // `(0..n)` so `@0/**` is always present in template-mode
        // blobs, but a hypothetical future emit-side change (e.g.,
        // 2-of-2 with cosigner indexing starting at 1) would have
        // silently mis-classified `wpkh(@1/**)` as descriptor-passthrough.
        // The regex `@\d+/**` matches any digit index. Mirrors the
        // `sparrow.rs:566` precedent (`Regex::new(r"@\d+(?:/\*\*)?")`).
        //
        // Closes `sparrow-taproot-descriptor-passthrough-import-support`
        // (Cycle 8) + `sparrow-taproot-singlesig-template-mode-import`
        // (Cycle 9) + `sparrow-import-detection-regex-defensive-widening`
        // (Cycle 11).
        let has_tr = script_template.contains("tr(");
        let has_at_placeholder = regex::Regex::new(r"@\d+/\*\*")
            .expect("at-placeholder regex is a fixed string literal")
            .is_match(&script_template);
        let is_descriptor_passthrough = has_tr && !has_at_placeholder;

        // Step 5: substitute `@i/**` → `[fp/derivation_no_m]xpub/<0;1>/*`.
        // Apply longest-N first to avoid prefix collisions when N ≥ 10
        // (`@1` is a prefix of `@10`). Mirrors
        // `cmd/xpub_search/descriptor_intake.rs:212-218` discipline.
        //
        // Skipped for descriptor-passthrough mode (taproot multisig): the
        // script_template already carries concrete `[fp/path]xpub` keys;
        // substitution is a no-op + the leftover-placeholder regex below
        // also no-ops (no `@N/**` present to leave behind).
        let substituted = if is_descriptor_passthrough {
            script_template.clone()
        } else {
            let mut substituted = script_template.clone();
            let mut indices: Vec<usize> = (0..n).collect();
            indices.sort_by_key(|i| std::cmp::Reverse(i.to_string().len()));
            for i in indices {
                let placeholder = format!("@{i}/**");
                let ks = &keystores[i];
                // Strip leading `m/` from derivation; brackets carry path
                // sans the `m` prefix per BIP-380 / BIP-389.
                let path_no_m = ks
                    .derivation
                    .strip_prefix("m/")
                    .unwrap_or(ks.derivation.as_str().strip_prefix('m').unwrap_or(&ks.derivation));
                let bracketed = if path_no_m.is_empty() {
                    format!("[{fp}]{xpub}/<0;1>/*", fp = ks.master_fingerprint, xpub = ks.xpub)
                } else {
                    format!(
                        "[{fp}/{path}]{xpub}/<0;1>/*",
                        fp = ks.master_fingerprint,
                        path = path_no_m,
                        xpub = ks.xpub,
                    )
                };
                substituted = substituted.replace(&placeholder, &bracketed);
            }
            substituted
        };
        // Sanity: no leftover `@N/**` placeholders. (If the script_template
        // refers to an `@i` beyond `keystores.len()`, the substitution would
        // leave it intact — surface as a parse error rather than feeding
        // garbage downstream.)
        if let Some(stray) = leftover_placeholder_regex().find(&substituted) {
            return Err(ToolkitError::ImportWalletParse(format!(
                "import-wallet: sparrow: parse error: descriptor template references {} but keystores carries only {} entries",
                stray.as_str(),
                n
            )));
        }

        // Step 7: feed through the existing pipeline.
        let (placeholder_form, parsed_keys, parsed_fingerprints) =
            concrete_keys_to_placeholders(&substituted).map_err(|e| {
                ToolkitError::ImportWalletParse(e.message().replacen(
                    "import-wallet: bsms:",
                    "import-wallet: sparrow:",
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
                "import-wallet: sparrow: parse error: {}",
                e.message()
            ))
        })?;

        // Step 8: build ResolvedSlot vec.
        let origins =
            crate::wallet_import::pipeline::extract_origin_components(&substituted, "sparrow")?;
        let network = network_from_origins(&origins)?;
        let mut cosigners: Vec<ResolvedSlot> = Vec::with_capacity(parsed_keys.len());
        for (i, _) in parsed_keys.iter().enumerate() {
            let (xpub, fp, path) = build_slot_fields(&substituted, i)?;
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

        // Threshold extraction — for MULTI scripts (`multi(K,...)`,
        // `sortedmulti(K,...)`); SINGLE has no threshold token.
        let threshold = extract_threshold_local(&substituted)?;

        // Step 9: dropped-field detection + stderr NOTICE.
        let mut dropped_fields: Vec<String> = Vec::new();
        for (k, _) in obj.iter() {
            if !SPARROW_PRESERVED_TOP_LEVEL_KEYS.contains(&k.as_str()) {
                dropped_fields.push(k.clone());
            }
        }
        if !dropped_fields.is_empty() {
            writeln!(
                stderr,
                "notice: import-wallet: sparrow: dropped envelope fields {}: not preserved in bundle output (key-state only)",
                dropped_fields.join(", ")
            )
            .map_err(ToolkitError::Io)?;
        }

        let source_metadata = SparrowSourceMetadata {
            label: name_opt,
            policy_type,
            script_type,
            dropped_fields,
        };

        // Reconstruct the "original_descriptor" the toolkit propagates into
        // the v0.27.0 BundleJson envelope (`bundle.descriptor` wire-shape).
        // For Sparrow this is the substituted concrete-keys descriptor with
        // a freshly-computed BIP-380 checksum — Sparrow's wire shape stores
        // the placeholder form (`@N/**`); the envelope expects the
        // concrete-keys form. Re-checksum via miniscript's ChecksumEngine
        // for byte-determinism. On checksum-engine failure (non-ASCII / odd
        // chars), fall back to the substituted body without checksum
        // (downstream BundleJson does not crash on missing `#csum`).
        let original_descriptor = match recompute_descriptor_checksum(&substituted) {
            Ok(s) => s,
            Err(_) => substituted.clone(),
        };

        Ok(vec![ParsedImport {
            descriptor,
            original_descriptor,
            cosigners,
            network,
            threshold,
            provenance: ImportProvenance::Sparrow(source_metadata),
        }])
    }
}

/// Per-keystore raw fields extracted from `keystores[i]`.
#[derive(Debug)]
struct KeystoreParts {
    master_fingerprint: String,
    derivation: String,
    xpub: String,
}

fn parse_keystore(i: usize, ks: &Value) -> Result<KeystoreParts, ToolkitError> {
    let obj = ks.as_object().ok_or_else(|| {
        ToolkitError::ImportWalletParse(format!(
            "import-wallet: sparrow: parse error: keystores[{i}] is not an object"
        ))
    })?;
    let key_derivation = obj
        .get("keyDerivation")
        .and_then(|v| v.as_object())
        .ok_or_else(|| {
            ToolkitError::ImportWalletParse(format!(
                "import-wallet: sparrow: parse error: keystores[{i}].keyDerivation missing or not an object"
            ))
        })?;
    let master_fingerprint = key_derivation
        .get("masterFingerprint")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            ToolkitError::ImportWalletParse(format!(
                "import-wallet: sparrow: parse error: keystores[{i}].keyDerivation.masterFingerprint missing or not a string"
            ))
        })?
        .to_string();
    // Sparrow's masterFingerprint is lowercase 8-hex by emit convention; be
    // lenient — accept any 8-hex (case-insensitive) but reject anything that
    // doesn't parse as a u32 byte sequence.
    if master_fingerprint.len() != 8
        || !master_fingerprint.chars().all(|c| c.is_ascii_hexdigit())
    {
        return Err(ToolkitError::ImportWalletParse(format!(
            "import-wallet: sparrow: parse error: keystores[{i}].keyDerivation.masterFingerprint must be 8 hex chars, got {master_fingerprint:?}"
        )));
    }
    let derivation = key_derivation
        .get("derivation")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            ToolkitError::ImportWalletParse(format!(
                "import-wallet: sparrow: parse error: keystores[{i}].keyDerivation.derivation missing or not a string"
            ))
        })?
        .to_string();
    let xpub = obj
        .get("extendedPublicKey")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            ToolkitError::ImportWalletParse(format!(
                "import-wallet: sparrow: parse error: keystores[{i}].extendedPublicKey missing or not a string"
            ))
        })?
        .to_string();
    Ok(KeystoreParts {
        master_fingerprint,
        derivation,
        xpub,
    })
}

/// `@N` leftover-detection regex. Used as a sanity guard after the per-
/// keystore substitution pass — any leftover `@N` means the script template
/// referred to an index outside `keystores.len()`.
fn leftover_placeholder_regex() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"@\d+(?:/\*\*)?").expect("leftover_placeholder_regex is a fixed string literal")
    })
}

fn build_slot_fields(
    descriptor_body: &str,
    slot_idx: usize,
) -> Result<(Xpub, Fingerprint, DerivationPath), ToolkitError> {
    let origins =
        crate::wallet_import::pipeline::extract_origin_components(descriptor_body, "sparrow")?;
    let (fp, path, xpub_str) = origins.into_iter().nth(slot_idx).ok_or_else(|| {
        ToolkitError::ImportWalletParse(format!(
            "import-wallet: sparrow: parse error: slot index {slot_idx} out of range"
        ))
    })?;
    crate::wallet_import::pipeline::finalize_slot_fields(fp, path, &xpub_str, "sparrow")
}

fn network_from_origins(
    origins: &[(Fingerprint, DerivationPath, String)],
) -> Result<bitcoin::Network, ToolkitError> {
    if origins.is_empty() {
        return Err(ToolkitError::ImportWalletParse(
            "import-wallet: sparrow: parse error: no origins to infer network from".to_string(),
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
                "import-wallet: sparrow: cosigner {i} has coin-type {ct}, cosigner 0 has coin-type {first}; all cosigners must share a coin-type"
            )));
        }
    }
    match first {
        0 => Ok(bitcoin::Network::Bitcoin),
        1 => Ok(bitcoin::Network::Testnet),
        other => Err(ToolkitError::ImportWalletParse(format!(
            "import-wallet: sparrow: parse error: unsupported coin-type {other} on origin path; only 0 (mainnet) and 1 (testnet) supported per BIP-48"
        ))),
    }
}

fn coin_type_from_path(path: &DerivationPath) -> Result<u32, ToolkitError> {
    let comps: Vec<&ChildNumber> = path.into_iter().collect();
    if comps.len() < 2 {
        return Err(ToolkitError::ImportWalletParse(format!(
            "import-wallet: sparrow: parse error: origin path has only {} components; need ≥2 for BIP-48 coin-type inference",
            comps.len()
        )));
    }
    match comps[1] {
        ChildNumber::Hardened { index } => Ok(*index),
        ChildNumber::Normal { index } => Err(ToolkitError::ImportWalletParse(format!(
            "import-wallet: sparrow: parse error: coin-type component {index} is not hardened; BIP-48 requires `<coin_type>'`"
        ))),
    }
}

/// Extract K from `multi(K, ...)` / `sortedmulti(K, ...)` at the top-level
/// miniscript context. Returns `Ok(None)` for SINGLE descriptors (`wpkh(...)`,
/// `pkh(...)`, etc.). Mirrors `bsms::extract_threshold` / `bitcoin_core::
/// extract_threshold`. Local to Sparrow so error prefixes are tagged
/// correctly.
fn extract_threshold_local(descriptor_body: &str) -> Result<Option<u8>, ToolkitError> {
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
            "import-wallet: sparrow: parse error: multi argument `{arg}` exceeds u8 range (>255 cosigners not supported): {e}"
        ))
    })
}

/// Re-render a concrete-keys descriptor with a freshly computed BIP-380
/// checksum, mirroring `roundtrip::recanonicalize_descriptor`'s shape.
/// Used for the `original_descriptor` field on `ParsedImport`.
fn recompute_descriptor_checksum(body: &str) -> Result<String, ToolkitError> {
    use miniscript::descriptor::checksum::Engine as ChecksumEngine;
    let body_no_csum = match body.rsplit_once('#') {
        Some((b, _)) => b,
        None => body,
    };
    let mut eng = ChecksumEngine::new();
    eng.input(body_no_csum).map_err(|e| {
        ToolkitError::ImportWalletParse(format!(
            "import-wallet: sparrow: checksum engine input rejected: {e}"
        ))
    })?;
    let csum = eng.checksum();
    Ok(format!("{body_no_csum}#{csum}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===========================================================================
    // SNIFF cells (P1A; preserved for P1B's reviewer-loop)
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

    #[test]
    fn sniff_false_on_bitcoin_core_blob() {
        let blob = br#"{"wallet_name":"x","descriptors":[{"desc":"wpkh(xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX/<0;1>/*)#00000000"}]}"#;
        assert!(!SparrowParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_specter_blob() {
        let blob = br#"{"label":"my-wallet","blockheight":700000,"descriptor":"wpkh(xpub...)","devices":[{"type":"coldcard","label":"cc"}]}"#;
        assert!(!SparrowParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_empty_keystores() {
        let blob = br#"{"policyType":"SINGLE","scriptType":"P2WPKH",
                        "defaultPolicy":{"miniscript":{"script":"wpkh(@0/**)"}},
                        "keystores":[]}"#;
        assert!(!SparrowParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_missing_nested_script() {
        let blob = br#"{"policyType":"SINGLE","scriptType":"P2WPKH",
                        "defaultPolicy":{"miniscript":{}},
                        "keystores":[{"x":1}]}"#;
        assert!(!SparrowParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_unrecognized_policy_type_value() {
        let blob = br#"{"policyType":"NOVEL","scriptType":"P2WPKH",
                        "defaultPolicy":{"miniscript":{"script":"wpkh(@0/**)"}},
                        "keystores":[{"x":1}]}"#;
        assert!(!SparrowParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_bare_array() {
        let blob = br#"[{"policyType":"SINGLE"}]"#;
        assert!(!SparrowParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_random_text() {
        assert!(!SparrowParser::sniff(b"not a wallet blob\n"));
    }

    #[test]
    fn sniff_false_on_empty_blob() {
        assert!(!SparrowParser::sniff(b""));
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

    // ===========================================================================
    // PARSE cells (P1B)
    // ===========================================================================

    fn parse(blob: &[u8]) -> Result<Vec<ParsedImport>, ToolkitError> {
        let mut stderr = Vec::new();
        SparrowParser::parse(blob, &mut stderr)
    }

    fn parse_capturing_stderr(blob: &[u8]) -> (Result<Vec<ParsedImport>, ToolkitError>, String) {
        let mut stderr = Vec::new();
        let r = SparrowParser::parse(blob, &mut stderr);
        (r, String::from_utf8(stderr).unwrap_or_default())
    }

    /// Parse: SINGLE / P2WPKH happy-path. Verifies cosigner count + network
    /// + provenance variant.
    #[test]
    fn parse_single_wpkh_mainnet_happy_path() {
        let blob = br#"{
            "name":"bip84-0","network":"mainnet","policyType":"SINGLE","scriptType":"P2WPKH",
            "defaultPolicy":{"name":"Default","miniscript":{"script":"wpkh(@0/**)"}},
            "keystores":[{
                "label":"bip84-0","source":"SW_WATCH","walletModel":"SPARROW",
                "keyDerivation":{"masterFingerprint":"5436d724","derivation":"m/84'/0'/0'"},
                "extendedPublicKey":"xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9"
            }]
        }"#;
        let parsed = parse(blob).unwrap();
        assert_eq!(parsed.len(), 1);
        let p = &parsed[0];
        assert_eq!(p.cosigners.len(), 1);
        assert_eq!(p.network, bitcoin::Network::Bitcoin);
        assert_eq!(p.threshold, None);
        assert!(matches!(p.provenance, ImportProvenance::Sparrow(_)));
        if let ImportProvenance::Sparrow(meta) = &p.provenance {
            assert_eq!(meta.label.as_deref(), Some("bip84-0"));
            assert_eq!(meta.policy_type, SparrowPolicyType::Single);
            assert_eq!(meta.script_type, "P2WPKH");
            assert!(meta.dropped_fields.is_empty());
        } else {
            panic!("provenance");
        }
    }

    /// Parse: MULTI 2-of-3 P2WSH sortedmulti happy-path. Verifies cosigner
    /// count + threshold + per-slot xpub matches.
    #[test]
    fn parse_multi_2of3_p2wsh_sortedmulti_happy_path() {
        let blob = br#"{
            "name":"wsh-sortedmulti-0","network":"mainnet","policyType":"MULTI","scriptType":"P2WSH",
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
        let parsed = parse(blob).unwrap();
        assert_eq!(parsed.len(), 1);
        let p = &parsed[0];
        assert_eq!(p.cosigners.len(), 3);
        assert_eq!(p.network, bitcoin::Network::Bitcoin);
        assert_eq!(p.threshold, Some(2));
        // Cosigner ordering preserved (declaration order from `keystores`).
        assert_eq!(p.cosigners[0].fingerprint.to_string(), "b8688df1");
        assert_eq!(p.cosigners[1].fingerprint.to_string(), "28645006");
        assert_eq!(p.cosigners[2].fingerprint.to_string(), "5436d724");
        if let ImportProvenance::Sparrow(meta) = &p.provenance {
            assert_eq!(meta.policy_type, SparrowPolicyType::Multi);
            assert_eq!(meta.script_type, "P2WSH");
        }
    }

    /// v0.31.2 Cycle 9 — taproot singlesig template-mode imports via the
    /// standard substitution path. Previously refused at Cycle 8 narrow
    /// refusal block (now removed); `tr(@0/**)` substitutes to
    /// `tr([fp/86'/0'/0']xpub.../<0;1>/*)` which the pipeline accepts.
    #[test]
    fn parse_p2tr_singlesig_imports_via_substitution() {
        let blob = br#"{
            "name":"bip86-0","network":"mainnet","policyType":"SINGLE","scriptType":"P2TR",
            "defaultPolicy":{"name":"Default","miniscript":{"script":"tr(@0/**)"}},
            "keystores":[{
                "label":"bip86-0","source":"SW_WATCH","walletModel":"SPARROW",
                "keyDerivation":{"masterFingerprint":"5436d724","derivation":"m/86'/0'/0'"},
                "extendedPublicKey":"xpub6CAYwo2AfKJy1cdFGBAgLvCrZULhEkZ9C9s4GGXwXzHvNPguMWBcVrGEDjP2ZJdX92gVWLeLrNVVmipTrKqrwMy2eT282xKEyHMbPDrcD9e"
            }]
        }"#;
        let parsed = parse(blob).unwrap();
        assert_eq!(parsed.len(), 1, "Bip86 emits exactly one ParsedImport");
        let p = &parsed[0];
        assert_eq!(p.cosigners.len(), 1);
        // Sparrow Bip86 singlesig: derivation path m/86'/0'/0' → BIP-86 mainnet.
        assert!(
            p.cosigners[0].bracketed_origin().contains("86'"),
            "cosigner origin must reflect m/86'/0'/0'; got: {}",
            p.cosigners[0].bracketed_origin()
        );
    }

    /// Refusal: SINGLE policyType with 2 keystores → invalid.
    #[test]
    fn parse_single_with_multi_keystores_refused() {
        let blob = br#"{
            "policyType":"SINGLE","scriptType":"P2WPKH",
            "defaultPolicy":{"miniscript":{"script":"wpkh(@0/**)"}},
            "keystores":[
                {"keyDerivation":{"masterFingerprint":"deadbeef","derivation":"m/84'/0'/0'"},
                 "extendedPublicKey":"xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9"},
                {"keyDerivation":{"masterFingerprint":"feedface","derivation":"m/84'/0'/0'"},
                 "extendedPublicKey":"xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9"}
            ]
        }"#;
        let err = parse(blob).unwrap_err();
        let msg = format!("{err:?}");
        assert!(
            msg.contains("SINGLE") && msg.contains("2 entries"),
            "expected SINGLE-with-2-keystores refusal; got: {msg}"
        );
    }

    /// Refusal: MULTI policyType with 1 keystore → invalid.
    #[test]
    fn parse_multi_with_single_keystore_refused() {
        let blob = br#"{
            "policyType":"MULTI","scriptType":"P2WSH",
            "defaultPolicy":{"miniscript":{"script":"wsh(sortedmulti(1,@0/**))"}},
            "keystores":[
                {"keyDerivation":{"masterFingerprint":"deadbeef","derivation":"m/48'/0'/0'/2'"},
                 "extendedPublicKey":"xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX"}
            ]
        }"#;
        let err = parse(blob).unwrap_err();
        let msg = format!("{err:?}");
        assert!(
            msg.contains("MULTI") && msg.contains("only 1"),
            "expected MULTI-with-1-keystore refusal; got: {msg}"
        );
    }

    /// Refusal: malformed masterFingerprint (not 8 hex).
    #[test]
    fn parse_malformed_fingerprint_refused() {
        let blob = br#"{
            "policyType":"SINGLE","scriptType":"P2WPKH",
            "defaultPolicy":{"miniscript":{"script":"wpkh(@0/**)"}},
            "keystores":[{
                "keyDerivation":{"masterFingerprint":"not-hex","derivation":"m/84'/0'/0'"},
                "extendedPublicKey":"xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9"
            }]
        }"#;
        let err = parse(blob).unwrap_err();
        let msg = format!("{err:?}");
        assert!(
            msg.contains("masterFingerprint must be 8 hex chars"),
            "expected fingerprint-format refusal; got: {msg}"
        );
    }

    /// Stderr NOTICE: dropped fields surface via SPEC §2.4 template.
    #[test]
    fn parse_emits_notice_for_dropped_fields() {
        let blob = br#"{
            "name":"bip84-0","network":"mainnet","policyType":"SINGLE","scriptType":"P2WPKH",
            "defaultPolicy":{"miniscript":{"script":"wpkh(@0/**)"}},
            "keystores":[{
                "label":"bip84-0",
                "keyDerivation":{"masterFingerprint":"5436d724","derivation":"m/84'/0'/0'"},
                "extendedPublicKey":"xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9"
            }],
            "birthDate":1717000000,
            "gapLimit":20
        }"#;
        let (res, stderr) = parse_capturing_stderr(blob);
        let parsed = res.unwrap();
        assert_eq!(parsed.len(), 1);
        assert!(
            stderr.contains("notice: import-wallet: sparrow: dropped envelope fields")
                && stderr.contains("birthDate")
                && stderr.contains("gapLimit"),
            "expected stderr NOTICE listing both dropped fields; got: {stderr}"
        );
        if let ImportProvenance::Sparrow(meta) = &parsed[0].provenance {
            assert!(meta.dropped_fields.iter().any(|f| f == "birthDate"));
            assert!(meta.dropped_fields.iter().any(|f| f == "gapLimit"));
        }
    }

    /// Testnet network detection via BIP-48 coin-type=1.
    #[test]
    fn parse_testnet_network_inferred_from_coin_type_one() {
        let blob = br#"{
            "name":"testnet","network":"testnet","policyType":"SINGLE","scriptType":"P2WPKH",
            "defaultPolicy":{"miniscript":{"script":"wpkh(@0/**)"}},
            "keystores":[{
                "keyDerivation":{"masterFingerprint":"704c7836","derivation":"m/84'/1'/0'"},
                "extendedPublicKey":"tpubDEgS9fUEpucKatmvKAv21v8nViHxR6rsV7ohMWK4YjsWd4EWT3w8YzMgMEvNrDfsUANbid74WRFpr3Gym8UHBSLnqg6b1Lzvibw87cLSctC"
            }]
        }"#;
        let parsed = parse(blob).unwrap();
        assert_eq!(parsed[0].network, bitcoin::Network::Testnet);
    }

    // ===========================================================================
    // Fixture-driven cells: load tests/fixtures/wallet_import/sparrow-*.json
    // and round-trip via the parse pipeline. Fixtures owner per plan-doc P1B
    // row.
    // ===========================================================================

    fn load_fixture(name: &str) -> Vec<u8> {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/wallet_import")
            .join(name);
        std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
    }

    #[test]
    fn fixture_singlesig_p2wpkh_parses_clean() {
        let blob = load_fixture("sparrow-singlesig-p2wpkh.json");
        let parsed = parse(&blob).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].cosigners.len(), 1);
        assert_eq!(parsed[0].network, bitcoin::Network::Bitcoin);
        assert_eq!(parsed[0].threshold, None);
    }

    /// v0.31.4 (Cycle 11) defensive widening regression — the Step 6
    /// `has_at_placeholder` discriminator regex `@\d+/**` must match
    /// every template-mode placeholder shape regardless of cosigner
    /// index, AND must NOT match descriptor-passthrough shapes that
    /// don't contain an `@N/**` placeholder. Closes
    /// `sparrow-import-detection-regex-defensive-widening`.
    #[test]
    fn at_placeholder_regex_matches_only_template_mode_shapes() {
        let re = regex::Regex::new(r"@\d+/\*\*").expect("regex literal");
        // Positive cases — any digit index, any wrapper.
        for s in &[
            "@0/**",
            "@1/**",
            "@10/**",
            "wpkh(@0/**)",
            "wpkh(@1/**)",
            "wsh(sortedmulti(2,@0/**,@1/**,@2/**))",
            "tr(@0/**)",
        ] {
            assert!(
                re.is_match(s),
                "regex should match template-mode shape {s:?}"
            );
        }
        // Negative cases — descriptor-passthrough OR malformed.
        for s in &[
            "",
            "@/**",
            "@0/*",
            "@a/**",
            "tr([5436d724/86'/0'/0']xpub6CAYwo2AfKJy1cdFGBAgLvCrZULhEkZ9C9s4GGXwXzHvNPguMWBcVrGEDjP2ZJdX92gVWLeLrNVVmipTrKqrwMy2eT282xKEyHMbPDrcD9e/<0;1>/*)",
            "wsh(multi(2,[fp/path]xpub.../<0;1>/*,[fp/path]xpub.../<0;1>/*))",
        ] {
            assert!(
                !re.is_match(s),
                "regex should NOT match non-template-mode shape {s:?}"
            );
        }
    }

    /// v0.31.4 backward-compat regression — the v0.31.3 `@0/**`
    /// substring path still routes template-mode shapes through
    /// Step 5 substitution after the regex widening. Locks the
    /// no-behavior-change claim under the current Sparrow emit
    /// invariant (`wallet_export/sparrow.rs:230` indexes from
    /// `(0..n)`).
    #[test]
    fn parse_at_0_placeholder_still_routes_to_template_mode_substitution() {
        // sparrow-singlesig-p2wpkh.json carries `wpkh(@0/**)` (the
        // existing fixture used at fixture_singlesig_p2wpkh_parses_clean).
        // After the v0.31.4 regex widening, this fixture must continue
        // to route through template-mode substitution (NOT
        // descriptor-passthrough) — the substituted descriptor body
        // carries the concrete `[fp/path]xpub/<0;1>/*` form.
        let blob = load_fixture("sparrow-singlesig-p2wpkh.json");
        let parsed = parse(&blob).unwrap();
        assert_eq!(parsed.len(), 1);
        let p = &parsed[0];
        // BIP-84 origin path m/84'/0'/0' substituted into the descriptor.
        assert!(
            p.cosigners[0].bracketed_origin().contains("84'"),
            "BIP-84 origin path must survive template-mode substitution; got: {}",
            p.cosigners[0].bracketed_origin()
        );
        assert_eq!(p.threshold, None);
    }

    #[test]
    fn fixture_multisig_2of3_sortedmulti_parses_clean() {
        let blob = load_fixture("sparrow-multisig-2of3-p2wsh-sortedmulti.json");
        let parsed = parse(&blob).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].cosigners.len(), 3);
        assert_eq!(parsed[0].threshold, Some(2));
    }

    #[test]
    fn fixture_multisig_2of3_multi_ordered_parses_clean() {
        let blob = load_fixture("sparrow-multisig-2of3-p2wsh-multi-ordered.json");
        let parsed = parse(&blob).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].cosigners.len(), 3);
        assert_eq!(parsed[0].threshold, Some(2));
    }

    #[test]
    fn fixture_singlesig_p2sh_p2wpkh_parses_clean() {
        let blob = load_fixture("sparrow-singlesig-p2sh-p2wpkh.json");
        let parsed = parse(&blob).unwrap();
        assert_eq!(parsed.len(), 1);
        if let ImportProvenance::Sparrow(meta) = &parsed[0].provenance {
            assert_eq!(meta.script_type, "P2SH_P2WPKH");
        }
    }

    #[test]
    fn fixture_malformed_missing_script_refused() {
        let blob = load_fixture("sparrow-malformed-missing-script.json");
        let err = parse(&blob).unwrap_err();
        let msg = format!("{err:?}");
        assert!(
            msg.contains("defaultPolicy.miniscript.script"),
            "expected missing-script refusal; got: {msg}"
        );
    }

    /// SLIP-132 zpub xpub variants are normalized to neutral form by the
    /// existing pipeline.
    #[test]
    fn parse_zpub_variant_normalized() {
        let blob = br#"{
            "policyType":"SINGLE","scriptType":"P2WPKH",
            "defaultPolicy":{"miniscript":{"script":"wpkh(@0/**)"}},
            "keystores":[{
                "keyDerivation":{"masterFingerprint":"5436d724","derivation":"m/84'/0'/0'"},
                "extendedPublicKey":"zpub6qTBTNftBzVTjgVcSUw7vW5N1KQbV93Jnrw314RHGkCkSx4vk6nEWH1MJfReXi2WThvuDRiRpyT7cDoakEcZMQ1iZPgfJgQrcVMR4aJWh6S"
            }]
        }"#;
        let parsed = parse(blob).unwrap();
        assert_eq!(parsed.len(), 1);
        // Slot xpub is normalized to neutral `xpub` form internally; the
        // round-trip via `Display` re-renders as `xpub`.
        assert!(
            parsed[0].cosigners[0].xpub.to_string().starts_with("xpub6"),
            "zpub must normalize to xpub; got: {}",
            parsed[0].cosigners[0].xpub
        );
    }
}
