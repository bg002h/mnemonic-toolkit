//! SPEC §6 + v0.8 §7 — BIP-388 `wallet_policy` JSON emitter.

use super::pipeline::{descriptor_to_bip388_wallet_policy, key_origin_str, template_origin_path_no_m};
use super::{
    EmitInputs, MissingField, TaprootInternalKey, WalletFormatEmitter, NUMS_XONLY_HEX,
};
use crate::error::ToolkitError;
use crate::network::CliNetwork;
use crate::synthesize::ResolvedSlot;
use crate::template::CliTemplate;
use serde_json::{json, Value};

/// SPEC v0.8 §12 — `WalletFormatEmitter` impl for `--format bip388`.
/// Two paths:
/// - Template path (`EmitInputs.template` is `Some`): render directly via
///   `format_bip388_wallet_policy` using `@N/**` placeholders.
/// - Descriptor passthrough (`EmitInputs.template` is `None`): re-extract
///   `[fp/path]xpub` keys via `pipeline::descriptor_to_bip388_wallet_policy`.
///
/// Both branches return a `Value`; the trait emit pretty-prints to `String`.
pub(crate) struct Bip388Emitter;

impl WalletFormatEmitter for Bip388Emitter {
    fn collect_missing(_inputs: &EmitInputs) -> Vec<MissingField> {
        // BIP-388 wallet_policy is tolerant: the template path always has
        // populated resolved_slots (validated upstream), and the descriptor
        // path re-extracts keys deterministically. Missing pieces surface
        // as descriptor / BadInput errors rather than §4 missing-info refusals.
        Vec::new()
    }

    fn emit(inputs: &EmitInputs) -> Result<String, ToolkitError> {
        let value = if let Some(template) = inputs.template {
            // Template path: render using `@N/**` placeholders.
            // Threshold defaults to 1 for singlesig (matches v0.7 behavior).
            let k = inputs.threshold.unwrap_or(1);
            format_bip388_wallet_policy(
                template,
                inputs.resolved_slots,
                k,
                inputs.network,
                inputs.account,
                inputs.taproot_internal_key,
            )?
        } else {
            // Descriptor passthrough.
            descriptor_to_bip388_wallet_policy(inputs.canonical_descriptor)?
        };
        serde_json::to_string_pretty(&value)
            .map_err(|e| ToolkitError::BadInput(format!("export-wallet json: {e}")))
    }

    fn extension() -> &'static str {
        "json"
    }
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
