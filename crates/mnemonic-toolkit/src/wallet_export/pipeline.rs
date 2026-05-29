//! SPEC §4 + v0.8 §7 — canonical descriptor build pipeline (multipath form),
//! plus the SPEC v0.8 §6 `--descriptor` → BIP-388 `wallet_policy` interop
//! transformer.

use super::{TaprootInternalKey, NUMS_XONLY_HEX};
use crate::error::ToolkitError;
use crate::network::CliNetwork;
use crate::synthesize::ResolvedSlot;
use crate::template::CliTemplate;
use miniscript::{Descriptor as MsDescriptor, DescriptorPublicKey};
use serde_json::{json, Value};
use std::str::FromStr;

/// SPEC §4 + v0.8 §7: build the canonical descriptor string (with `#checksum`)
/// from template + resolved slots. Multipath form `<0;1>` for receive+change.
/// `taproot_internal_key` is required for `tr-multi-a` / `tr-sortedmulti-a` and
/// ignored for other templates.
pub(crate) fn build_descriptor_string(
    template: CliTemplate,
    slots: &[ResolvedSlot],
    k: u8,
    network: CliNetwork,
    account: u32,
    taproot_internal_key: Option<TaprootInternalKey>,
) -> Result<String, ToolkitError> {
    let s =
        build_descriptor_string_inner(template, slots, k, network, account, taproot_internal_key)?;
    let parsed = MsDescriptor::<DescriptorPublicKey>::from_str(&s)
        .map_err(|e| ToolkitError::DescriptorParse(format!("export-wallet descriptor parse: {e}")))?;
    Ok(parsed.to_string())
}

pub(super) fn key_origin_str(slot: &ResolvedSlot, fallback_path: &str) -> String {
    // For a slot with an origin path, the bracketed annotation is rebuilt from
    // the typed fingerprint+path (`bracketed_origin()`). For the pathless
    // (degenerate / WIF) slot, `bracketed_origin()` would be just `[fp]`, which
    // drops the BIP-32 origin — so we keep the path-bearing `fallback_path`
    // branch (the template-derived default). miniscript wants no `m/` prefix
    // inside `[fp/...]`. See SPEC_path_raw_bracketed_bare_unification.md §5 C4.
    if slot.origin_path_bare().is_empty() {
        let fp = slot.fingerprint.to_string().to_lowercase();
        let raw = fallback_path
            .trim_start_matches("m/")
            .trim_start_matches('m')
            .trim_start_matches('/');
        format!("[{fp}/{raw}]")
    } else {
        slot.bracketed_origin()
    }
}

pub(super) fn template_origin_path_no_m(
    template: CliTemplate,
    network: CliNetwork,
    account: u32,
) -> String {
    let s = template.origin_path_str(network, account);
    s.trim_start_matches("m/").trim_start_matches('m').to_string()
}

fn build_descriptor_string_inner(
    template: CliTemplate,
    slots: &[ResolvedSlot],
    k: u8,
    network: CliNetwork,
    account: u32,
    taproot_internal_key: Option<TaprootInternalKey>,
) -> Result<String, ToolkitError> {
    if slots.is_empty() {
        return Err(ToolkitError::BadInput(
            "export-wallet: at least one --slot @N.xpub=... required".into(),
        ));
    }
    let fallback = template_origin_path_no_m(template, network, account);

    // Single-sig templates: bip44 → pkh, bip49 → sh(wpkh(...)), bip84 → wpkh,
    // bip86 → tr(...).
    let key_segs: Vec<String> = slots
        .iter()
        .map(|s| {
            let origin = key_origin_str(s, &fallback);
            format!("{origin}{}/<0;1>/*", s.xpub)
        })
        .collect();

    let s = match template {
        CliTemplate::Bip44 => format!("pkh({})", key_segs[0]),
        CliTemplate::Bip49 => format!("sh(wpkh({}))", key_segs[0]),
        CliTemplate::Bip84 => format!("wpkh({})", key_segs[0]),
        CliTemplate::Bip86 => format!("tr({})", key_segs[0]),
        CliTemplate::WshMulti => format!("wsh(multi({k},{}))", key_segs.join(",")),
        CliTemplate::WshSortedMulti => {
            format!("wsh(sortedmulti({k},{}))", key_segs.join(","))
        }
        CliTemplate::ShWshMulti => {
            format!("sh(wsh(multi({k},{})))", key_segs.join(","))
        }
        CliTemplate::ShWshSortedMulti => {
            format!("sh(wsh(sortedmulti({k},{})))", key_segs.join(","))
        }
        CliTemplate::TrMultiA | CliTemplate::TrSortedMultiA => {
            build_tr_multi_a_descriptor(template, &key_segs, k, taproot_internal_key)?
        }
    };
    Ok(s)
}

/// SPEC v0.8 §7 — assemble `tr(<internal-key>, multi_a(K, leaves...))` per
/// the chosen `taproot_internal_key` designation.
fn build_tr_multi_a_descriptor(
    template: CliTemplate,
    key_segs: &[String],
    k: u8,
    taproot_internal_key: Option<TaprootInternalKey>,
) -> Result<String, ToolkitError> {
    let internal = taproot_internal_key.ok_or_else(|| {
        ToolkitError::BadInput(
            "internal: tr-multi-a / tr-sortedmulti-a reached without --taproot-internal-key"
                .into(),
        )
    })?;
    let leaf_op = match template {
        CliTemplate::TrMultiA => "multi_a",
        CliTemplate::TrSortedMultiA => "sortedmulti_a",
        _ => unreachable!("non-tr-multi-a template in build_tr_multi_a_descriptor"),
    };
    Ok(match internal {
        TaprootInternalKey::Nums => {
            // NUMS internal key: all cosigners stay in the multi_a leaf.
            format!(
                "tr({NUMS_XONLY_HEX},{leaf_op}({k},{}))",
                key_segs.join(","),
            )
        }
        TaprootInternalKey::Cosigner(idx) => {
            // Cosigner N is the key-path key; remaining N-1 cosigners are
            // the multi_a leaves. Bounds-checked in the caller.
            let internal_seg = key_segs.get(idx as usize).ok_or_else(|| {
                ToolkitError::BadInput(format!(
                    "--taproot-internal-key @{idx} out of range (only {} cosigners)",
                    key_segs.len(),
                ))
            })?;
            let leaf_segs: Vec<&String> = key_segs
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != idx as usize)
                .map(|(_, s)| s)
                .collect();
            let leaf_str = leaf_segs
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(",");
            format!("tr({internal_seg},{leaf_op}({k},{leaf_str}))")
        }
    })
}

/// SPEC v0.8 §6 — `--descriptor` + `--format bip388` interop. Parses the
/// canonical descriptor string, extracts `[fp/path]xpub` keys (stripping the
/// derivation suffix), and emits the BIP-388 wallet_policy JSON with `@N/**`
/// placeholders. Requires the descriptor to use the multipath form
/// `<0;1>/*` for each key (BIP-388's intended receive/change shape); other
/// derivation suffixes are refused.
pub(crate) fn descriptor_to_bip388_wallet_policy(
    canonical_descriptor: &str,
) -> Result<Value, ToolkitError> {
    let parsed = MsDescriptor::<DescriptorPublicKey>::from_str(canonical_descriptor)
        .map_err(|e| ToolkitError::DescriptorParse(format!("--descriptor parse: {e}")))?;
    if !parsed.is_multipath() {
        return Err(ToolkitError::BadInput(
            "--format bip388 requires the --descriptor to use multipath form `/<0;1>/*` (BIP-388 receive/change shape)".into(),
        ));
    }

    // Walk parsed.iter_pk() to collect each DescriptorPublicKey in source
    // order. For each, derive (a) the keys_info form `[fp/path]xpub` and
    // (b) the full descriptor occurrence string `[fp/path]xpub/<0;1>/*`
    // for substitution. Both are deterministic via miniscript's serialization.
    let mut keys_info: Vec<String> = Vec::new();
    let mut full_key_strs: Vec<String> = Vec::new();
    for pk in parsed.iter_pk() {
        let full = pk.to_string();
        let stripped = strip_multipath_suffix(&full)?;
        keys_info.push(stripped);
        full_key_strs.push(full);
    }

    // String-substitute each full key-expression with `@N/**`. We replace
    // longest-first to avoid prefix collisions when two keys share a common
    // prefix (e.g., the same xpub at different derivations).
    let mut template = canonical_descriptor.to_string();
    // Strip the BIP-380 `#checksum` suffix — BIP-388 wallet_policy
    // `description_template` is checksum-free; the wallet recomputes it.
    if let Some(pos) = template.rfind('#') {
        template.truncate(pos);
    }
    let mut indexed: Vec<(usize, &String)> = full_key_strs.iter().enumerate().collect();
    indexed.sort_by_key(|(_, s)| std::cmp::Reverse(s.len()));
    for (i, full) in indexed {
        let placeholder = format!("@{i}/**");
        template = template.replacen(full, &placeholder, 1);
    }

    Ok(json!({
        "name": "imported-descriptor",
        "description_template": template,
        "keys_info": keys_info,
    }))
}

/// Strip the `/<0;1>/*` multipath suffix from a key-expression string.
/// Refuses non-multipath suffixes (e.g., `/0/*`, `/0/0`) since BIP-388's
/// `@N/**` placeholder maps specifically to the receive/change pair.
fn strip_multipath_suffix(full: &str) -> Result<String, ToolkitError> {
    // Expected suffix: "/<0;1>/*" (8 chars).
    const SUFFIX: &str = "/<0;1>/*";
    full.strip_suffix(SUFFIX)
        .ok_or_else(|| {
            ToolkitError::BadInput(format!(
                "--format bip388 requires every descriptor key to end in `/<0;1>/*` for the receive/change pair; got key {full:?}",
            ))
        })
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin::bip32::{DerivationPath, Fingerprint, Xpub};
    use std::str::FromStr;

    const SAMPLE_XPUB: &str = "xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9";

    fn slot_with_path(path: DerivationPath) -> ResolvedSlot {
        ResolvedSlot {
            xpub: Xpub::from_str(SAMPLE_XPUB).unwrap(),
            fingerprint: Fingerprint::from_str("deadbeef").unwrap(),
            path,
            path_raw: String::new(),
            entropy: None,
            master_xpub: None,
            _entropy_pin: None,
        }
    }

    // T5(e) (SPEC_path_raw_bracketed_bare_unification.md §8, R0 C-2) —
    // `key_origin_str` must keep the path-bearing fallback for a pathless
    // (default-path) slot rather than dropping to a bare `[fp]` that would
    // corrupt the exported descriptor key.
    #[test]
    fn key_origin_str_pathless_slot_uses_path_bearing_fallback() {
        let pathless = slot_with_path(DerivationPath::default());
        assert_eq!(
            key_origin_str(&pathless, "84'/0'/0'"),
            "[deadbeef/84'/0'/0']",
            "pathless slot must use the path-bearing fallback, not bare [fp]"
        );
        // also accepts an `m/`-prefixed fallback
        assert_eq!(key_origin_str(&pathless, "m/84'/0'/0'"), "[deadbeef/84'/0'/0']");
    }

    // A path-bearing slot renders from its typed path (ignores the fallback).
    #[test]
    fn key_origin_str_path_bearing_slot_uses_bracketed_origin() {
        let slot = slot_with_path(DerivationPath::from_str("48'/0'/0'/2'").unwrap());
        assert_eq!(key_origin_str(&slot, "84'/0'/0'"), "[deadbeef/48'/0'/0'/2']");
    }
}
