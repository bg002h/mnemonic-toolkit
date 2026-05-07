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
/// `export-wallet` v0.7. Internal-key designation (NUMS vs key-path key) for
/// `tr(<internal-key>, multi_a(...))` is deferred to v0.8.
pub fn taproot_multisig_unsupported_message(name: &str) -> String {
    format!(
        "--template <{name}> is not yet supported by 'mnemonic export-wallet' (taproot internal-key designation deferred to v0.8); use 'mnemonic bundle' for taproot multisig artifacts."
    )
}

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

/// SPEC §4: build the canonical descriptor string (with `#checksum`) from
/// template + resolved slots. Multipath form `<0;1>` for receive+change.
pub(crate) fn build_descriptor_string(
    template: CliTemplate,
    slots: &[ResolvedSlot],
    k: u8,
    network: CliNetwork,
    account: u32,
) -> Result<String, ToolkitError> {
    let s = build_descriptor_string_inner(template, slots, k, network, account)?;
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
        CliTemplate::TrMultiA => format!("tr(multi_a({k},{}))", key_segs.join(",")),
        CliTemplate::TrSortedMultiA => {
            format!("tr(sortedmulti_a({k},{}))", key_segs.join(","))
        }
    };
    Ok(s)
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

/// SPEC §6: emit BIP-388 `wallet_policy` JSON. `description_template` uses
/// `@N/**` placeholders; `keys_info` is `[fp/path]xpub` strings in slot-index
/// order.
pub(crate) fn format_bip388_wallet_policy(
    template: CliTemplate,
    slots: &[ResolvedSlot],
    k: u8,
    network: CliNetwork,
    account: u32,
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
        CliTemplate::TrMultiA => format!("tr(multi_a({k},{}))", placeholders.join(",")),
        CliTemplate::TrSortedMultiA => {
            format!("tr(sortedmulti_a({k},{}))", placeholders.join(","))
        }
    };

    Ok(json!({
        "name": template.human_name(),
        "description_template": description_template,
        "keys_info": keys_info,
    }))
}

