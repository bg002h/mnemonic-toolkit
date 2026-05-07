//! `mnemonic export-wallet` format adapters.
//!
//! Realizes `design/SPEC_export_wallet_v0_7.md` §3 (watch-only refusal),
//! §4 (descriptor pipeline), §5 (Bitcoin Core importdescriptors),
//! §6 (BIP-388 wallet_policy), §7 (Sparrow/Specter stubs).

use crate::error::ToolkitError;
use crate::network::CliNetwork;
use crate::slot_input::{SlotInput, SlotSubkey};
use crate::synthesize::ResolvedSlot;
use crate::template::CliTemplate;
use miniscript::{Descriptor as MsDescriptor, DescriptorPublicKey};
use serde_json::{json, Value};
use std::str::FromStr;

/// SPEC §3 byte-exact refusal text for secret-bearing slot inputs.
pub const REFUSAL_SECRET_INPUT: &str =
    "mnemonic export-wallet is watch-only by definition; supply only xpub/fingerprint/path slots. To produce an artifact that includes secret material, use 'mnemonic bundle'.";

/// SPEC §7 byte-exact refusal text for sparrow/specter format stubs.
pub fn format_stub_message(name: &str) -> String {
    format!(
        "--format <{name}> is deferred to v0.8 if user demand surfaces; use --format bitcoin-core or --format bip388 instead."
    )
}

/// Refusal text for `tr-multi-a` / `tr-sortedmulti-a` templates under
/// `export-wallet`. v0.8 unblocked these templates via `--taproot-internal-key`;
/// this helper is retained for the (now-unreachable) `ToolkitError`
/// variant message and preserves the v0.7 byte-exact refusal text in case a
/// downstream consumer parses it.
#[allow(dead_code)]
pub fn taproot_multisig_unsupported_message(name: &str) -> String {
    format!(
        "--template <{name}> is not yet supported by 'mnemonic export-wallet' (taproot internal-key designation deferred to v0.8); use 'mnemonic bundle' for taproot multisig artifacts."
    )
}

/// SPEC v0.8 §7 — taproot internal-key designation for `tr-multi-a` /
/// `tr-sortedmulti-a` templates. Selected by `--taproot-internal-key`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaprootInternalKey {
    /// BIP-341 reference NUMS x-only point ("nothing-up-my-sleeve"). Use
    /// when no key-path spend is desired; the script path enforces the
    /// multisig leaf set, and the key-path is provably unspendable.
    Nums,
    /// Cosigner index `@N` is the key-path internal key. Cosigner N is
    /// removed from the multi_a leaf set; remaining cosigners form the
    /// (k-of-(n-1)) script path.
    Cosigner(u8),
}

/// SPEC v0.8 §7 — BIP-341 reference NUMS x-only point. Format used in
/// descriptor expressions: 64-char lowercase hex (32 raw x-only bytes).
/// Origin: this is the canonical NUMS point published in the BIP-341
/// supplementary material and adopted by Bitcoin wallets that produce
/// unspendable taproot key-path keys.
pub(crate) const NUMS_XONLY_HEX: &str =
    "50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0";

/// SPEC §3: refuse phrase / entropy / xprv / wif subkeys. Pre-`resolve_slots`
/// fast path — runs on the user-supplied raw slot inputs to short-circuit
/// before any work. The SPEC-mandated invariant ("validator runs on the
/// resolved-slot set") is additionally enforced by `validate_watch_only_resolved`
/// after `bundle::resolve_slots` returns.
pub(crate) fn validate_watch_only(slots: &[SlotInput]) -> Result<(), ToolkitError> {
    for s in slots {
        if matches!(
            s.subkey,
            SlotSubkey::Phrase | SlotSubkey::Entropy | SlotSubkey::Xprv | SlotSubkey::Wif
        ) {
            return Err(ToolkitError::ExportWalletSecretInput);
        }
    }
    Ok(())
}

/// SPEC §3 post-`resolve_slots` invariant — asserts that no resolved slot
/// carries entropy material. `phrase=` / `entropy=` slots populate
/// `ResolvedSlot.entropy`; `xprv=` slots are refused upstream by the
/// pre-resolve fast path before reaching `resolve_slots`. `wif=` slots can
/// only be supplied at single-sig N=1 in the slot grammar but populate
/// `ResolvedSlot.entropy` with the wif marker; the pre-resolve check catches
/// them, this post-resolve check is the SPEC-stated invariant.
pub(crate) fn validate_watch_only_resolved(
    resolved: &[ResolvedSlot],
) -> Result<(), ToolkitError> {
    if resolved.iter().any(|r| r.entropy.is_some()) {
        return Err(ToolkitError::ExportWalletSecretInput);
    }
    Ok(())
}

/// SPEC §5 timestamp argument: "now" sentinel or unix seconds.
#[derive(Debug, Clone, Copy)]
pub(crate) enum TimestampArg {
    Now,
    Unix(i64),
}

impl TimestampArg {
    fn to_json(self) -> Value {
        match self {
            TimestampArg::Now => json!("now"),
            TimestampArg::Unix(n) => json!(n),
        }
    }
}

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

fn key_origin_str(slot: &ResolvedSlot, fallback_path: &str) -> String {
    let fp = slot.fingerprint.to_string().to_lowercase();
    // path_raw may include leading "m/" or not; miniscript wants no "m/" prefix
    // inside `[fp/...]`. Strip it.
    let raw = if slot.path_raw.is_empty() {
        fallback_path.trim_start_matches("m/").trim_start_matches('m').to_string()
    } else {
        slot.path_raw.trim_start_matches("m/").trim_start_matches('m').to_string()
    };
    let raw = raw.trim_start_matches('/');
    format!("[{fp}/{raw}]")
}

fn template_origin_path_no_m(template: CliTemplate, network: CliNetwork, account: u32) -> String {
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

/// SPEC §5: emit Bitcoin Core `importdescriptors` JSON. Multipath `<0;1>`
/// splits into 2 entries (receive `internal: false`, change `internal: true`).
pub(crate) fn format_bitcoin_core_importdescriptors(
    canonical_descriptor: &str,
    range: (u32, u32),
    timestamp: TimestampArg,
    _bitcoin_core_version: u8,
) -> Result<Value, ToolkitError> {
    let parsed = MsDescriptor::<DescriptorPublicKey>::from_str(canonical_descriptor)
        .map_err(|e| ToolkitError::DescriptorParse(format!("export-wallet re-parse: {e}")))?;

    let entries: Vec<Value> = if parsed.is_multipath() {
        let parts = parsed
            .clone()
            .into_single_descriptors()
            .map_err(|e| ToolkitError::DescriptorParse(format!("multipath split: {e}")))?;
        if parts.len() != 2 {
            return Err(ToolkitError::DescriptorParse(format!(
                "expected 2 multipath splits (receive/change), got {}",
                parts.len()
            )));
        }
        parts
            .into_iter()
            .enumerate()
            .map(|(i, p)| {
                json!({
                    "desc": p.to_string(),
                    "active": true,
                    "internal": i == 1,
                    "range": [range.0, range.1],
                    "timestamp": timestamp.to_json(),
                })
            })
            .collect()
    } else {
        vec![json!({
            "desc": parsed.to_string(),
            "active": true,
            "internal": false,
            "range": [range.0, range.1],
            "timestamp": timestamp.to_json(),
        })]
    };

    Ok(Value::Array(entries))
}

/// SPEC §6 + v0.8 §7: emit BIP-388 `wallet_policy` JSON. `description_template`
/// uses `@N/**` placeholders; `keys_info` is `[fp/path]xpub` strings in
/// slot-index order. Taproot multisig with cosigner-internal key uses `@N/**`
/// for the internal key; with NUMS, the literal hex is embedded directly.
pub(crate) fn format_bip388_wallet_policy(
    template: CliTemplate,
    slots: &[ResolvedSlot],
    k: u8,
    network: CliNetwork,
    account: u32,
    taproot_internal_key: Option<TaprootInternalKey>,
) -> Result<Value, ToolkitError> {
    let fallback = template_origin_path_no_m(template, network, account);
    let keys_info: Vec<String> = slots
        .iter()
        .map(|s| format!("{}{}", key_origin_str(s, &fallback), s.xpub))
        .collect();

    let n = slots.len();
    let placeholders: Vec<String> = (0..n).map(|i| format!("@{i}/**")).collect();

    let description_template = match template {
        CliTemplate::Bip44 => format!("pkh({})", placeholders[0]),
        CliTemplate::Bip49 => format!("sh(wpkh({}))", placeholders[0]),
        CliTemplate::Bip84 => format!("wpkh({})", placeholders[0]),
        CliTemplate::Bip86 => format!("tr({})", placeholders[0]),
        CliTemplate::WshMulti => format!("wsh(multi({k},{}))", placeholders.join(",")),
        CliTemplate::WshSortedMulti => {
            format!("wsh(sortedmulti({k},{}))", placeholders.join(","))
        }
        CliTemplate::ShWshMulti => {
            format!("sh(wsh(multi({k},{})))", placeholders.join(","))
        }
        CliTemplate::ShWshSortedMulti => {
            format!("sh(wsh(sortedmulti({k},{})))", placeholders.join(","))
        }
        CliTemplate::TrMultiA | CliTemplate::TrSortedMultiA => {
            let leaf_op = match template {
                CliTemplate::TrMultiA => "multi_a",
                CliTemplate::TrSortedMultiA => "sortedmulti_a",
                _ => unreachable!(),
            };
            let internal = taproot_internal_key.ok_or_else(|| {
                ToolkitError::BadInput(
                    "internal: tr-multi-a wallet_policy needs --taproot-internal-key".into(),
                )
            })?;
            match internal {
                TaprootInternalKey::Nums => {
                    format!(
                        "tr({NUMS_XONLY_HEX},{leaf_op}({k},{}))",
                        placeholders.join(","),
                    )
                }
                TaprootInternalKey::Cosigner(idx) => {
                    let leaf_placeholders: Vec<&String> = placeholders
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| *i != idx as usize)
                        .map(|(_, p)| p)
                        .collect();
                    let leaf_str = leaf_placeholders
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(",");
                    format!(
                        "tr(@{idx}/**,{leaf_op}({k},{leaf_str}))",
                    )
                }
            }
        }
    };

    Ok(json!({
        "name": template.human_name(),
        "description_template": description_template,
        "keys_info": keys_info,
    }))
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

