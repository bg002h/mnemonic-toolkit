//! SPEC §6 + v0.8 §7 — BIP-388 `wallet_policy` JSON emitter.

use super::pipeline::{key_origin_str, template_origin_path_no_m};
use super::{TaprootInternalKey, NUMS_XONLY_HEX};
use crate::error::ToolkitError;
use crate::network::CliNetwork;
use crate::synthesize::ResolvedSlot;
use crate::template::CliTemplate;
use serde_json::{json, Value};

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
