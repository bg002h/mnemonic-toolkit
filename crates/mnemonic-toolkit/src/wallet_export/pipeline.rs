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
    // DELIBERATELY STRICT, and the only strict descriptor parse left on the
    // `export-wallet` path (PLAN `PLAN_wallet_file_export.md` Phase 1). It is
    // NOT an admission point for `sigless-branch`: this canonicalizes the
    // BUILDER's own output for the `--scheme`/`--slot` arm, and a
    // builder-produced descriptor cannot carry a sigless branch — every
    // template is a `pk`/`multi`/`multi_a`/`sortedmulti` quorum. The uniform
    // admission gate still runs on the result, and simply never fires there.
    //
    // Relaxing it would be a behaviour change nobody has ruled on: this
    // function is also `restore`'s canonicalizer, and Phase 1 makes no
    // behaviour change to `restore`.
    let parsed = MsDescriptor::<DescriptorPublicKey>::from_str(&s).map_err(|e| {
        ToolkitError::DescriptorParse(format!("export-wallet descriptor parse: {e}"))
    })?;
    Ok(parsed.to_string())
}

pub(super) fn key_origin_str(slot: &ResolvedSlot) -> String {
    // Bug hunt 2026-11-25 toolkit#2: the bracketed origin is rebuilt ONLY from
    // what the slot itself carries (typed fingerprint + path). A pathless slot
    // renders as a bare `[fp]`. This used to substitute the TEMPLATE DEFAULT
    // path for a pathless slot, so `restore --md1` printed
    // `[fp/87'/0'/0']<key>` for a key whose card origin was empty and which
    // does not live there: a signer following that origin derives a different
    // key. A default path belongs only where the caller itself derived the key
    // at it, and such a slot already carries that path.
    // miniscript wants no `m/` prefix inside `[fp/...]`.
    slot.bracketed_origin()
}

fn build_descriptor_string_inner(
    template: CliTemplate,
    slots: &[ResolvedSlot],
    k: u8,
    // Unused since bug hunt 2026-11-25 toolkit#2: they only fed the template
    // default that `key_origin_str` used to substitute for a pathless slot.
    _network: CliNetwork,
    _account: u32,
    taproot_internal_key: Option<TaprootInternalKey>,
) -> Result<String, ToolkitError> {
    if slots.is_empty() {
        return Err(ToolkitError::BadInput(
            "export-wallet: at least one --slot @N.xpub=... required".into(),
        ));
    }
    // Single-sig templates: bip44 → pkh, bip49 → sh(wpkh(...)), bip84 → wpkh,
    // bip86 → tr(...).
    let key_segs: Vec<String> = slots
        .iter()
        .map(|s| {
            let origin = key_origin_str(s);
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
            "internal: tr-multi-a / tr-sortedmulti-a reached without --taproot-internal-key".into(),
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
            format!("tr({NUMS_XONLY_HEX},{leaf_op}({k},{}))", key_segs.join(","),)
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
/// derivation suffixes are refused. A `tr` whose internal key is an
/// origin-less literal key refuses first (the ruled literal-key text), and
/// keys whose public keys are not pairwise distinct refuse last (BIP-388
/// "Additional rules"); `alternatives` is the format list those texts name.
/// The BIP-388 wallet-policy `name` used when no name is carried — the value
/// `export-wallet --descriptor` emitted unconditionally before
/// `bip388-policy-name-lossy-roundtrip` (v0.53.8). Also the `build-descriptor`
/// `--emit-spec bip388` default.
pub(crate) const DEFAULT_BIP388_POLICY_NAME: &str = "imported-descriptor";

pub(crate) fn descriptor_to_bip388_wallet_policy(
    canonical_descriptor: &str,
    name: &str,
    alternatives: &str,
) -> Result<Value, ToolkitError> {
    // Lenient: a re-parse of an already-admitted string (PLAN
    // `PLAN_wallet_file_export.md` Phase 1, F-2). `bip388` is a TRANSFORMING
    // emitter — it rewrites the descriptor into a `@N/**` template — and it is
    // reached only AFTER the admission gate, so a keyless leaf must survive
    // this parse rather than be refused here without an affordance
    // (round-1 finding I4(b)).
    let parsed = crate::parse_descriptor::parse_descriptor_lenient(canonical_descriptor)
        .map_err(|e| ToolkitError::DescriptorParse(format!("--descriptor parse: {e}")))?;
    if let MsDescriptor::Tr(tr) = &parsed {
        if let DescriptorPublicKey::Single(single) = tr.internal_key() {
            if single.origin.is_none() {
                return Err(super::bip388::bip388_literal_key_refusal(
                    &tr.internal_key().to_string(),
                ));
            }
        }
    }
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
    let mut xpubs: Vec<bitcoin::bip32::Xpub> = Vec::new();
    for pk in parsed.iter_pk() {
        let full = pk.to_string();
        let stripped = strip_multipath_suffix(&full)?;
        if let DescriptorPublicKey::MultiXPub(mx) = &pk {
            // A step between the xkey and `/<0;1>/*` (e.g. `xpub/5/<0;1>/*`)
            // would leave `xpub/5` in keys_info: no BIP-388 form (bip-0388
            // L172, L180-187).
            if mx.derivation_paths.paths().iter().any(|p| p.len() != 1) {
                return Err(super::bip388::bip388_extra_step_refusal(
                    &full,
                    alternatives,
                ));
            }
            xpubs.push(mx.xkey);
        }
        keys_info.push(stripped);
        full_key_strs.push(full);
    }
    super::bip388::check_bip388_distinct_keys(&xpubs, alternatives)?;

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
        "name": name,
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
            entropy: None,
            master_xpub: None,
            language: None,
            _entropy_pin: None,
        }
    }

    // Bug hunt 2026-11-25 toolkit#2 (supersedes T5(e)): a pathless slot
    // renders as a bare `[fp]` — never with a substituted template default.
    #[test]
    fn key_origin_str_pathless_slot_never_guesses_a_path() {
        let pathless = slot_with_path(DerivationPath::default());
        assert_eq!(key_origin_str(&pathless), "[deadbeef]");
    }

    // A path-bearing slot renders from its typed path.
    #[test]
    fn key_origin_str_path_bearing_slot_uses_bracketed_origin() {
        let slot = slot_with_path(DerivationPath::from_str("48'/0'/0'/2'").unwrap());
        assert_eq!(key_origin_str(&slot), "[deadbeef/48'/0'/0'/2']");
    }
}
