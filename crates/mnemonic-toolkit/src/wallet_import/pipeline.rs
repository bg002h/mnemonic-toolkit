//! Concrete-keys → `@N`-placeholder adapter (inverse of
//! `wallet_export::pipeline::descriptor_to_bip388_wallet_policy`).
//!
//! Per SPEC §4.2 step 5: lex `[fp/path]xpub` occurrences out of a third-party
//! descriptor body, assign sequential `@N` placeholders preserving
//! declaration order, and produce `(ParsedKey, ParsedFingerprint)` pairs that
//! feed `parse_descriptor::parse_descriptor`.
//!
//! Per SPEC §4.3: ordering is the literal first-occurrence ordering in the
//! descriptor body. `sortedmulti(N, @0, @1, ..., @M)`'s lexicographic sort
//! at render time is orthogonal to this placeholder-binding step — the input
//! order is preserved at `@N` substitution; the render-time sort is a
//! `Display`-impl operation in miniscript that does not touch the
//! TLV-level ordering.

use crate::error::ToolkitError;
use crate::parse_descriptor::{ParsedFingerprint, ParsedKey};
use crate::slip0132::normalize_xpub_prefix;
use crate::synthesize::{xpub_to_65, ResolvedSlot};
use bitcoin::bip32::{DerivationPath, Fingerprint, Xpub};
use md_codec::Descriptor as MdDescriptor;
use regex::Regex;
use std::str::FromStr;
use std::sync::OnceLock;

/// SPEC §4.2 step 5 regex: `[fp/path]xpub`. Accepts SLIP-132 prefix
/// variants (`xpub|tpub|ypub|Ypub|zpub|Zpub|upub|Upub|vpub|Vpub`) — the xpub
/// string is canonicalized via `slip0132::normalize_xpub_prefix` before
/// payload extraction. The `path` capture is anchored by `/` + decimal digits
/// optionally followed by a hardened `'` mark.
///
/// Note: the literal regex below uses `[xtyzuvYZUV]` for the first prefix
/// char to match the 10 accepted SLIP-132 prefixes plus xpub/tpub. The
/// downstream `Xpub::from_str` accepts the neutralized form returned by
/// `normalize_xpub_prefix`; SLIP-132 mainnet variants neutralize to `xpub`,
/// testnet variants neutralize to `tpub`.
fn key_regex() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"\[([0-9a-fA-F]{8})((?:/\d+(?:'|h)?)+)\]([xtyzuvYZUV]pub[A-HJ-NP-Za-km-z1-9]+)")
            .expect("key_regex is a fixed string literal")
    })
}

/// Lift every `[fp/path]xpub` origin tuple from a concrete descriptor body via
/// the canonical (h-form-widened) `key_regex`, in declaration order. Shared by
/// all `wallet_import` parsers (FOLLOWUP `descriptor-origin-extraction-dedup`),
/// replacing the former per-parser `extract_origin_components` + apostrophe-only
/// `origin_capture_regex` copies — so every parser now tolerates `h`-form
/// hardened origins (resolves `import-parser-hform-origin-tolerance`).
/// `format_name` is the per-parser error prefix. Empty result → error.
pub(crate) fn extract_origin_components(
    body: &str,
    format_name: &str,
) -> Result<Vec<(Fingerprint, DerivationPath, String)>, ToolkitError> {
    let mut out = Vec::new();
    for cap in key_regex().captures_iter(body) {
        let fp_hex = cap.get(1).expect("group 1").as_str();
        let path_raw_inner = cap.get(2).expect("group 2").as_str();
        let xpub_str = cap.get(3).expect("group 3").as_str();

        let mut fp_bytes = [0u8; 4];
        for i in 0..4 {
            fp_bytes[i] = u8::from_str_radix(&fp_hex[i * 2..i * 2 + 2], 16).map_err(|e| {
                ToolkitError::ImportWalletParse(format!(
                    "import-wallet: {format_name}: parse error: fingerprint hex: {e}"
                ))
            })?;
        }
        let fp = Fingerprint::from(fp_bytes);
        let path = DerivationPath::from_str(&format!("m{path_raw_inner}")).map_err(|e| {
            ToolkitError::ImportWalletParse(format!(
                "import-wallet: {format_name}: parse error: derivation-path parse: {e}"
            ))
        })?;
        out.push((fp, path, xpub_str.to_string()));
    }
    if out.is_empty() {
        return Err(ToolkitError::ImportWalletParse(format!(
            "import-wallet: {format_name}: parse error: no origin annotations in descriptor"
        )));
    }
    Ok(out)
}

/// Finalize one extracted origin tuple → typed slot fields: SLIP-0132-neutralize
/// the xpub prefix, then decode to a typed `Xpub`. Shared finalize half of the
/// former per-parser `build_slot_fields` (FOLLOWUP
/// `descriptor-origin-extraction-dedup`). The decode is a defensive guard — the
/// same key was already decoded by `concrete_keys_to_placeholders` upstream — so
/// the generic (slot-context-free) error message is invisible in practice.
pub(crate) fn finalize_slot_fields(
    fp: Fingerprint,
    path: DerivationPath,
    xpub_str: &str,
    format_name: &str,
) -> Result<(Xpub, Fingerprint, DerivationPath), ToolkitError> {
    let (neutral, _variant) = normalize_xpub_prefix(xpub_str)?;
    let xpub = Xpub::from_str(&neutral).map_err(|e| {
        ToolkitError::ImportWalletParse(format!(
            "import-wallet: {format_name}: parse error: xpub decode: {e}"
        ))
    })?;
    Ok((xpub, fp, path))
}

/// Cheap `@\d`-presence probe (the toolkit's `@N` placeholder form). NEW.
fn at_n_probe() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"@\d").expect("AT_N_PROBE literal"))
}

/// `@N`-form probe for callers that must NOT trigger the rule-4 origin
/// error (export-wallet passthrough accepts origin-less concrete). SPEC §3.4.
pub(crate) fn is_at_n_form(s: &str) -> bool {
    at_n_probe().is_match(s)
}

/// Which descriptor form a user string is. Discriminant only — no payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DescriptorForm {
    /// `@N`-placeholder template (keys sourced per-surface).
    AtN,
    /// Bare-concrete form with inline `[fp/path]xpub` keys.
    Concrete,
}

/// Classify a descriptor string via cheap probes. Pure; no conversion.
/// Rule 1: both probes → mixed error. 2: `@\d` only → AtN. 3: key_regex
/// only → Concrete. 4: neither → origin-required error (md-codec is NOT
/// reached on this branch, so the error originates here — SPEC §3.1).
pub(crate) fn classify_descriptor_form(input: &str) -> Result<DescriptorForm, ToolkitError> {
    let has_at_n = at_n_probe().is_match(input);
    let has_concrete = key_regex().is_match(input);
    match (has_at_n, has_concrete) {
        (true, true) => Err(ToolkitError::DescriptorParse(
            "descriptor mixes @N placeholders with inline keys; use one form".into(),
        )),
        (true, false) => Ok(DescriptorForm::AtN),
        (false, true) => Ok(DescriptorForm::Concrete),
        (false, false) => Err(ToolkitError::DescriptorParse(
            "descriptor has neither @N placeholders nor [fp/path]-annotated keys; \
             concrete descriptors must carry a key origin, e.g. [<fp>/84h/0h/0h]xpub…"
                .into(),
        )),
    }
}

/// Strict BIP-388 wallet-policy schema — the exact inverse-side mirror of the
/// emitter at `wallet_export::pipeline::descriptor_to_bip388_wallet_policy`.
/// `deny_unknown_fields`. (Moved here from `cmd/xpub_search/descriptor_intake.rs`
/// so the policy→descriptor expansion is single-sourced and reusable by the
/// `export-wallet`/`bundle` `--descriptor` consumers.)
#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct BipPolicyJson {
    // Deserialized-but-unread: the `_name` underscore silences `dead_code`; the
    // `#[serde(rename = "name")]` is LOAD-BEARING — without it, `deny_unknown_fields`
    // would reject the real `"name"` JSON key and demand a `"_name"` key, breaking
    // every real policy.
    #[serde(rename = "name")]
    _name: String,
    description_template: String,
    keys_info: Vec<String>,
}

/// True iff `s` (trimmed) begins with `{` — the BIP-388 wallet-policy-JSON sniff.
/// MUST be checked BEFORE `is_at_n_form` / `classify_descriptor_form`: a raw
/// policy JSON matches the `@\d` probe (its `description_template`) AND the
/// `key_regex` probe (its `keys_info`), so an unguarded policy would trip the
/// mixed-form / @N-refusal paths.
pub(crate) fn is_bip388_policy_shape(s: &str) -> bool {
    s.trim_start().starts_with('{')
}

/// Expand a BIP-388 wallet-policy JSON `{name, description_template, keys_info}`
/// into a concrete multipath descriptor STRING by substituting each `@N/**` →
/// `keys_info[N] + "/<0;1>/*"`. Pure string-in/string-out (no network/account/
/// stderr). The exact inverse of the emitter's `@N/**` substitution.
///
/// Replaces longest-N-first by **digit-count** (`@10` before `@1`) to mirror the
/// emitter inverse — over-defensive here since `/**` is part of every replaced
/// token (so `@1` can never be a substring of `@10/**`), but kept faithful to
/// the original `descriptor_intake` logic. After substitution, any residual
/// `@N` means the template referenced an index ≥ `keys_info.len()` → refuse
/// (rather than feed a half-substituted string to the downstream parser).
pub(crate) fn expand_bip388_policy(json: &str) -> Result<String, ToolkitError> {
    let parsed: BipPolicyJson = serde_json::from_str(json).map_err(|e| {
        ToolkitError::BadInput(format!(
            "--descriptor BIP-388 JSON parse failed: {e}; expected fields {{name, description_template, keys_info}}"
        ))
    })?;
    let mut template = parsed.description_template.clone();
    let mut indices: Vec<usize> = (0..parsed.keys_info.len()).collect();
    indices.sort_by_key(|n| std::cmp::Reverse(n.to_string().len()));
    for n in indices {
        let placeholder = format!("@{n}/**");
        let key = format!("{}/<0;1>/*", parsed.keys_info[n]);
        template = template.replace(&placeholder, &key);
    }
    if is_at_n_form(&template) {
        return Err(ToolkitError::DescriptorParse(
            "BIP-388 policy template references @N beyond keys_info[..]".into(),
        ));
    }
    Ok(template)
}

/// Extract the BIP-388 wallet-policy `name` from a policy JSON, for the
/// `--format bip388` round-trip name-preservation (`bip388-policy-name-lossy-
/// roundtrip`). Returns `None` for a missing/empty name OR malformed JSON —
/// **by contract, this NEVER errors**: the caller (`export-wallet`'s
/// `--descriptor` path) calls `expand_bip388_policy` immediately after, which
/// surfaces the real parse error. So do not error-check this result.
pub(crate) fn bip388_policy_name(json: &str) -> Option<String> {
    #[derive(serde::Deserialize)]
    struct NameOnly {
        name: String,
    }
    serde_json::from_str::<NameOnly>(json)
        .ok()
        .map(|n| n.name)
        .filter(|n| !n.is_empty())
}

/// Convert a descriptor body bearing concrete `[fp/path]xpub` keys into the
/// placeholder form `[fp/path]@N` + accompanying `(ParsedKey,
/// ParsedFingerprint)` pairs for `parse_descriptor::parse_descriptor`.
///
/// The replacement preserves the `[fp/path]` origin annotation so that the
/// downstream `lex_placeholders` + `resolve_placeholders` pipeline can
/// consume the `@N` syntax with origin-path metadata intact. The trailing
/// multipath / range suffix (e.g., `/<0;1>/*`) is preserved by virtue of
/// being outside the regex match.
pub(crate) fn concrete_keys_to_placeholders(
    descriptor: &str,
) -> Result<(String, Vec<ParsedKey>, Vec<ParsedFingerprint>), ToolkitError> {
    let re = key_regex();
    let mut keys: Vec<ParsedKey> = Vec::new();
    let mut fingerprints: Vec<ParsedFingerprint> = Vec::new();
    let mut placeholder_form = String::with_capacity(descriptor.len());
    let mut last_end = 0usize;
    let mut idx: u8 = 0;

    for cap in re.captures_iter(descriptor) {
        let m = cap.get(0).expect("group 0 is always present");
        placeholder_form.push_str(&descriptor[last_end..m.start()]);

        let fp_hex = cap.get(1).expect("group 1 captured").as_str();
        let path = cap.get(2).expect("group 2 captured").as_str();
        let xpub_str = cap.get(3).expect("group 3 captured").as_str();

        // SLIP-132 → neutral (xpub|tpub) canonicalization; rejects non-78-byte
        // base58check payloads and unknown version prefixes.
        let (neutral_xpub_str, _variant) = normalize_xpub_prefix(xpub_str)?;
        let xpub = Xpub::from_str(&neutral_xpub_str).map_err(|e| {
            ToolkitError::ImportWalletParse(format!(
                "import-wallet: bsms: parse error: xpub decode failed for key @{idx}: {e}"
            ))
        })?;

        let fp_bytes = parse_fp_hex(fp_hex).map_err(|e| {
            ToolkitError::ImportWalletParse(format!(
                "import-wallet: bsms: parse error: fingerprint decode failed for key @{idx}: {e}"
            ))
        })?;

        keys.push(ParsedKey {
            i: idx,
            payload: crate::synthesize::xpub_to_65(&xpub),
        });
        fingerprints.push(ParsedFingerprint {
            i: idx,
            fp: fp_bytes,
        });

        // Substitute the `[fp/path]xpub` literal with `@N[fp/path]`. The
        // `lex_placeholders` regex (parse_descriptor.rs:69) expects the
        // annotation to FOLLOW `@N` (capture group order: `@N[fp/path]
        // /<multipath>/*`), not precede it.
        placeholder_form.push('@');
        placeholder_form.push_str(&idx.to_string());
        placeholder_form.push('[');
        placeholder_form.push_str(fp_hex);
        placeholder_form.push_str(path);
        placeholder_form.push(']');

        last_end = m.end();
        idx = idx.checked_add(1).ok_or_else(|| {
            ToolkitError::ImportWalletParse(
                "import-wallet: bsms: parse error: more than 256 keys (placeholder @N overflow)"
                    .to_string(),
            )
        })?;
    }
    placeholder_form.push_str(&descriptor[last_end..]);

    if keys.is_empty() {
        return Err(ToolkitError::ImportWalletParse(
            "import-wallet: bsms: parse error: no [fp/path]xpub keys found in descriptor"
                .to_string(),
        ));
    }
    Ok((placeholder_form, keys, fingerprints))
}

/// Bare-concrete (checksum-stripped) descriptor body → (parsed md_codec
/// Descriptor, watch-only ResolvedSlots). Mirrors bsms.rs:219-265; recovers
/// the full Xpub + path from the original base58 (the ParsedKey [u8;65]
/// payload is lossy). SPEC §3.2.
pub(crate) fn descriptor_concrete_to_resolved_slots(
    body: &str,
) -> Result<(MdDescriptor, Vec<ResolvedSlot>), ToolkitError> {
    // Remap the converter's hard-coded "import-wallet: bsms:" prefix to a
    // neutral DescriptorParse (the caller is bundle/verify-bundle).
    let (placeholder_form, keys, fps) = concrete_keys_to_placeholders(body).map_err(|e| {
        ToolkitError::DescriptorParse(
            e.message()
                .replace("import-wallet: bsms: parse error: ", ""),
        )
    })?;
    let descriptor = crate::parse_descriptor::parse_descriptor(&placeholder_form, &keys, &fps)
        .map_err(|e| ToolkitError::DescriptorParse(e.message()))?;

    let mut slots: Vec<ResolvedSlot> = Vec::with_capacity(keys.len());
    for (idx, cap) in key_regex().captures_iter(body).enumerate() {
        let fp_hex = cap.get(1).expect("group 1").as_str();
        let path_inner = cap.get(2).expect("group 2").as_str();
        let xpub_str = cap.get(3).expect("group 3").as_str();
        let fp_bytes = parse_fp_hex(fp_hex).map_err(|e| {
            ToolkitError::DescriptorParse(format!("fingerprint hex for slot {idx}: {e}"))
        })?;
        let path = DerivationPath::from_str(&format!("m{path_inner}"))
            .map_err(|e| ToolkitError::DescriptorParse(format!("derivation path: {e}")))?;
        let (neutral, _variant) = normalize_xpub_prefix(xpub_str)?;
        let xpub = Xpub::from_str(&neutral)
            .map_err(|e| ToolkitError::DescriptorParse(format!("xpub decode: {e}")))?;
        debug_assert_eq!(xpub_to_65(&xpub), keys[idx].payload);
        slots.push(ResolvedSlot {
            xpub,
            fingerprint: Fingerprint::from(fp_bytes),
            path,
            entropy: None,
            master_xpub: None,
            language: None,
            _entropy_pin: None,
        });
    }
    Ok((descriptor, slots))
}

fn parse_fp_hex(s: &str) -> Result<[u8; 4], String> {
    if s.len() != 8 {
        return Err(format!("fingerprint must be 8 hex chars; got {}", s.len()));
    }
    let mut out = [0u8; 4];
    for i in 0..4 {
        out[i] =
            u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).map_err(|e| format!("hex parse: {e}"))?;
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// T4 (`bip388-policy-name-lossy-roundtrip`) — the name extractor returns
    /// the policy name, and `None` (NOT an error) for empty-name / malformed.
    #[test]
    fn bip388_policy_name_extracts_or_none() {
        assert_eq!(
            bip388_policy_name(r#"{"name":"test-vault","description_template":"wpkh(@0/**)","keys_info":["x"]}"#),
            Some("test-vault".to_string())
        );
        // empty name → None
        assert_eq!(
            bip388_policy_name(r#"{"name":"","description_template":"wpkh(@0/**)","keys_info":["x"]}"#),
            None
        );
        // missing name field → None
        assert_eq!(bip388_policy_name(r#"{"description_template":"wpkh(@0/**)"}"#), None);
        // malformed JSON → None (never errors; expand surfaces the real error)
        assert_eq!(bip388_policy_name("not json {{{"), None);
    }

    #[test]
    fn two_keys_preserve_declaration_order() {
        // Synthetic testnet inputs (lifted from the user's flagship BSMS blob).
        // Replacement uses literal `[fp/path]@N` form for downstream lex.
        let desc = "wsh(thresh(2,pkh([704c7836/48'/1'/3'/2']tpubDEgS9fUEpucKatmvKAv21v8nViHxR6rsV7ohMWK4YjsWd4EWT3w8YzMgMEvNrDfsUANbid74WRFpr3Gym8UHBSLnqg6b1Lzvibw87cLSctC/<0;1>/*),s:pk([97139860/48'/1'/2'/2']tpubDFiXyf7zmBhQrSHoAQB6SmMpF3rfSihAxQGMdQUtZfE8HWHkWLLNLTiYpMzvHnFiTmuUSYieHUYv4tFguzmiHeDrYV8TtWGCWt5qpqox4w3/<0;1>/*),sln:older(32768)))";
        let (placeholder, keys, fps) = concrete_keys_to_placeholders(desc).unwrap();
        assert_eq!(keys.len(), 2);
        assert_eq!(fps.len(), 2);
        assert_eq!(keys[0].i, 0);
        assert_eq!(keys[1].i, 1);
        // Declaration order: @0 was the pkh slot, @1 was the s:pk slot.
        assert_eq!(fps[0].fp, [0x70, 0x4c, 0x78, 0x36]);
        assert_eq!(fps[1].fp, [0x97, 0x13, 0x98, 0x60]);
        // Origin annotation preserved (`@N[fp/path]` form matches
        // `lex_placeholders` regex at parse_descriptor.rs:69).
        assert!(placeholder.contains("@0[704c7836/48'/1'/3'/2']/<0;1>/*"));
        assert!(placeholder.contains("@1[97139860/48'/1'/2'/2']/<0;1>/*"));
    }

    #[test]
    fn no_keys_errors() {
        let desc = "wsh(thresh(2,older(144),older(288)))";
        let err = concrete_keys_to_placeholders(desc).unwrap_err();
        assert!(matches!(err, ToolkitError::ImportWalletParse(_)));
    }

    #[test]
    fn hform_hardened_paths_accepted() {
        // Core/Sparrow emit `h`-form (`/48h/1h/...`); the converter must
        // accept it identically to apostrophe form.
        let hform = "wsh(sortedmulti(2,[704c7836/48h/1h/3h/2h]tpubDEgS9fUEpucKatmvKAv21v8nViHxR6rsV7ohMWK4YjsWd4EWT3w8YzMgMEvNrDfsUANbid74WRFpr3Gym8UHBSLnqg6b1Lzvibw87cLSctC/<0;1>/*,[97139860/48h/1h/2h/2h]tpubDFiXyf7zmBhQrSHoAQB6SmMpF3rfSihAxQGMdQUtZfE8HWHkWLLNLTiYpMzvHnFiTmuUSYieHUYv4tFguzmiHeDrYV8TtWGCWt5qpqox4w3/<0;1>/*))";
        let (placeholder, keys, fps) = concrete_keys_to_placeholders(hform).unwrap();
        assert_eq!(keys.len(), 2);
        assert_eq!(fps[0].fp, [0x70, 0x4c, 0x78, 0x36]);
        // The h-form path string is preserved verbatim into the @N form.
        assert!(placeholder.contains("@0[704c7836/48h/1h/3h/2h]/<0;1>/*"), "{placeholder}");
    }

    #[test]
    fn classify_atn_concrete_mixed_garbage() {
        // @N template → AtN.
        assert_eq!(
            classify_descriptor_form("wsh(sortedmulti(2,@0[704c7836/48'/1'/3'/2']/<0;1>/*,@1[97139860/48'/1'/2'/2']/<0;1>/*))").unwrap(),
            DescriptorForm::AtN
        );
        // bare concrete → Concrete.
        assert_eq!(
            classify_descriptor_form("wpkh([704c7836/84'/0'/0']tpubDEgS9fUEpucKatmvKAv21v8nViHxR6rsV7ohMWK4YjsWd4EWT3w8YzMgMEvNrDfsUANbid74WRFpr3Gym8UHBSLnqg6b1Lzvibw87cLSctC/0/*)").unwrap(),
            DescriptorForm::Concrete
        );
        // mixed @N + inline xpub → error (rule 1).
        let mixed = "wsh(sortedmulti(2,@0[704c7836/48'/1'/3'/2']/<0;1>/*,[97139860/48'/1'/2'/2']tpubDFiXyf7zmBhQrSHoAQB6SmMpF3rfSihAxQGMdQUtZfE8HWHkWLLNLTiYpMzvHnFiTmuUSYieHUYv4tFguzmiHeDrYV8TtWGCWt5qpqox4w3/<0;1>/*))";
        assert!(classify_descriptor_form(mixed).unwrap_err().message().contains("mixes @N"));
        // origin-less / keyless → rule-4 origin-required error.
        let err = classify_descriptor_form("wpkh(0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798)").unwrap_err();
        assert!(err.message().contains("must carry a key origin"), "{}", err.message());
    }

    #[test]
    fn concrete_to_resolved_slots_recovers_typed_fields() {
        let body = "wsh(sortedmulti(2,[704c7836/48'/1'/3'/2']tpubDEgS9fUEpucKatmvKAv21v8nViHxR6rsV7ohMWK4YjsWd4EWT3w8YzMgMEvNrDfsUANbid74WRFpr3Gym8UHBSLnqg6b1Lzvibw87cLSctC/<0;1>/*,[97139860/48'/1'/2'/2']tpubDFiXyf7zmBhQrSHoAQB6SmMpF3rfSihAxQGMdQUtZfE8HWHkWLLNLTiYpMzvHnFiTmuUSYieHUYv4tFguzmiHeDrYV8TtWGCWt5qpqox4w3/<0;1>/*))";
        let (_descriptor, slots) = descriptor_concrete_to_resolved_slots(body).unwrap();
        assert_eq!(slots.len(), 2);
        assert_eq!(slots[0].fingerprint, bitcoin::bip32::Fingerprint::from([0x70, 0x4c, 0x78, 0x36]));
        assert_eq!(slots[0].path, bitcoin::bip32::DerivationPath::from_str("m/48'/1'/3'/2'").unwrap());
        assert_eq!(slots[1].fingerprint, bitcoin::bip32::Fingerprint::from([0x97, 0x13, 0x98, 0x60]));
        assert!(slots.iter().all(|s| s.entropy.is_none()));
    }

    #[test]
    fn concrete_helper_error_drops_bsms_prefix() {
        let err = descriptor_concrete_to_resolved_slots("wsh(thresh(2,older(144),older(288)))").unwrap_err();
        assert!(!err.message().contains("bsms"), "leaked converter prefix: {}", err.message());
    }

    // ---- BIP-388 wallet-policy → concrete-descriptor expansion (Cycle D) ----

    #[test]
    fn is_bip388_policy_shape_detects_leading_brace() {
        assert!(is_bip388_policy_shape("{\"name\":\"x\"}"));
        assert!(is_bip388_policy_shape("  \n  {\"name\":\"x\"}")); // leading whitespace
        assert!(!is_bip388_policy_shape("wsh(multi(2,@0/**,@1/**))"));
        assert!(!is_bip388_policy_shape("md1qpwmxpzqqsrd"));
    }

    #[test]
    fn expand_bip388_policy_substitutes_each_at_n() {
        // First-pins the substitution output (the pre-existing descriptor_intake
        // cells are detect_shape-only; R0-r1 I-2).
        let json = r#"{"name":"vault","description_template":"wsh(sortedmulti(2,@0/**,@1/**))","keys_info":["[704c7836/48'/1'/3'/2']tpubDEgS9fUEpucKatmvKAv21v8nViHxR6rsV7ohMWK4YjsWd4EWT3w8YzMgMEvNrDfsUANbid74WRFpr3Gym8UHBSLnqg6b1Lzvibw87cLSctC","[97139860/48'/1'/2'/2']tpubDFiXyf7zmBhQrSHoAQB6SmMpF3rfSihAxQGMdQUtZfE8HWHkWLLNLTiYpMzvHnFiTmuUSYieHUYv4tFguzmiHeDrYV8TtWGCWt5qpqox4w3"]}"#;
        let out = expand_bip388_policy(json).unwrap();
        assert_eq!(
            out,
            "wsh(sortedmulti(2,[704c7836/48'/1'/3'/2']tpubDEgS9fUEpucKatmvKAv21v8nViHxR6rsV7ohMWK4YjsWd4EWT3w8YzMgMEvNrDfsUANbid74WRFpr3Gym8UHBSLnqg6b1Lzvibw87cLSctC/<0;1>/*,[97139860/48'/1'/2'/2']tpubDFiXyf7zmBhQrSHoAQB6SmMpF3rfSihAxQGMdQUtZfE8HWHkWLLNLTiYpMzvHnFiTmuUSYieHUYv4tFguzmiHeDrYV8TtWGCWt5qpqox4w3/<0;1>/*))"
        );
        assert!(!is_at_n_form(&out), "no residual @N placeholder");
    }

    #[test]
    fn expand_bip388_policy_deny_unknown_fields() {
        let json = r#"{"name":"x","description_template":"wsh(@0/**)","keys_info":["[704c7836/84'/0'/0']tpub"],"extra":1}"#;
        let err = expand_bip388_policy(json).unwrap_err();
        assert!(matches!(err, ToolkitError::BadInput(_)), "{}", err.message());
    }

    #[test]
    fn expand_bip388_policy_longest_n_first_no_clobber() {
        // 11 keys: @10 must map to keys_info[10], unaffected by the @1 pass.
        let placeholders: Vec<String> = (0..11).map(|i| format!("@{i}/**")).collect();
        let keys: Vec<String> = (0..11).map(|i| format!("\"k{i}\"")).collect();
        let json = format!(
            r#"{{"name":"x","description_template":"wsh(multi(6,{}))","keys_info":[{}]}}"#,
            placeholders.join(","),
            keys.join(",")
        );
        let out = expand_bip388_policy(&json).unwrap();
        assert!(out.contains("k10/<0;1>/*"), "{out}");
        assert!(out.contains("k1/<0;1>/*"), "{out}");
        assert!(!is_at_n_form(&out), "residual @N: {out}");
    }

    #[test]
    fn expand_bip388_policy_at_n_beyond_keys_info_refused() {
        // Template references @1 but only one key supplied → residual @1 → refuse
        // (the improved, earlier error vs a downstream miniscript parse failure).
        let json = r#"{"name":"x","description_template":"wsh(multi(2,@0/**,@1/**))","keys_info":["[704c7836/84'/0'/0']tpub"]}"#;
        let err = expand_bip388_policy(json).unwrap_err();
        assert!(matches!(err, ToolkitError::DescriptorParse(_)));
        assert!(err.message().contains("@N beyond keys_info"), "{}", err.message());
    }

    #[test]
    fn expand_bip388_policy_malformed_json_bad_input() {
        let err = expand_bip388_policy("not json").unwrap_err();
        assert!(matches!(err, ToolkitError::BadInput(_)));
    }
}
