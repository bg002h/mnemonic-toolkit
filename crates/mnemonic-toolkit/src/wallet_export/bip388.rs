//! SPEC §6 + v0.8 §7 — BIP-388 `wallet_policy` JSON emitter.

use super::pipeline::{
    descriptor_to_bip388_wallet_policy, key_origin_str, template_origin_path_no_m,
};
use super::{EmitInputs, MissingField, TaprootInternalKey, WalletFormatEmitter, NUMS_XONLY_HEX};
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
            // Descriptor passthrough. v0.53.8 (`bip388-policy-name-lossy-
            // roundtrip`): the policy `name` rides `inputs.wallet_name` (lifted
            // from the source `--descriptor` policy JSON; default
            // "imported-descriptor"), so a named policy round-trips.
            descriptor_to_bip388_wallet_policy(
                &inputs.canonical_descriptor,
                inputs.wallet_name,
                BIP388_EXPORT_ALTERNATIVES,
            )?
        };
        serde_json::to_string_pretty(&value)
            .map_err(|e| ToolkitError::Internal(format!("export-wallet json: {e}")))
    }

    fn extension() -> &'static str {
        "json"
    }
}

/// The ruled refusal for a literal (non-`@N`) key in a BIP-388 policy
/// (BIP-388 L310: a non-KP key is invalid).
/// (FOLLOWUPS `bip388-literal-nums-key-refusal-text`; sweep P25).
pub(crate) fn bip388_literal_key_refusal(hex: &str) -> ToolkitError {
    ToolkitError::BadInput(format!(
        "--format bip388 cannot express the literal key {hex} (a taproot internal key with \
         no origin, e.g. the BIP-341 NUMS key): a BIP-388 wallet policy allows only @N key \
         placeholders; use --format descriptor or --format bitcoin-core for this wallet"
    ))
}

/// A descriptor key with a derivation step between its extended key and
/// `/<0;1>/*` has no BIP-388 form: a KEY_INFO is an extended public key with
/// no path after it (bip-0388 L172, L180-187).
pub(crate) fn bip388_extra_step_refusal(key: &str, alternatives: &str) -> ToolkitError {
    ToolkitError::BadInput(format!(
        "--format bip388 cannot express the key {key} (a derivation step between the \
         extended public key and `/<0;1>/*`): a BIP-388 key information entry is an \
         extended public key with no path after it; use {alternatives} for this wallet"
    ))
}

/// The alternatives named by the bip388 refusals on export-wallet / restore.
pub(crate) const BIP388_EXPORT_ALTERNATIVES: &str = "--format descriptor or --format bitcoin-core";
/// The alternatives named on build-descriptor (no bitcoin-core format there).
pub(crate) const BIP388_BUILD_ALTERNATIVES: &str = "--format descriptor";

/// BIP-388 "Additional rules": the public keys of `keys_info` must be
/// pairwise distinct (BIP-388 L193). Compares the deserialized public key (not
/// the origin).
pub(crate) fn check_bip388_distinct_keys(
    xpubs: &[bitcoin::bip32::Xpub],
    alternatives: &str,
) -> Result<(), ToolkitError> {
    for i in 0..xpubs.len() {
        for j in (i + 1)..xpubs.len() {
            if xpubs[i].public_key == xpubs[j].public_key {
                return Err(ToolkitError::BadInput(format!(
                    "--format bip388 cannot express a wallet that uses one public key more than \
                     once (keys_info @{i} and @{j}: {}): a BIP-388 wallet policy requires \
                     pairwise distinct keys; use {alternatives} for this wallet",
                    xpubs[i]
                )));
            }
        }
    }
    Ok(())
}

/// SPEC §6 + v0.8 §7: emit BIP-388 `wallet_policy` JSON. `description_template`
/// uses `@N/**` placeholders; `keys_info` is `[fp/path]xpub` strings in
/// slot-index order. Taproot multisig with cosigner-internal key uses `@N/**`
/// for the internal key; with NUMS it refuses (`bip388_literal_key_refusal`:
/// BIP-388 allows only `@N` key placeholders). Keys whose public keys are not
/// pairwise distinct refuse (`check_bip388_distinct_keys`).
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
                ToolkitError::Internal(
                    "internal: tr-multi-a wallet_policy needs --taproot-internal-key".into(),
                )
            })?;
            match internal {
                // BIP-388 allows only `@N` key placeholders (a literal key is
                // invalid, bip-0388 line 310): refuse with the ruled text
                // shared with the descriptor arm (sweep P25).
                TaprootInternalKey::Nums => {
                    return Err(bip388_literal_key_refusal(NUMS_XONLY_HEX));
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
                    format!("tr(@{idx}/**,{leaf_op}({k},{leaf_str}))",)
                }
            }
        }
    };

    let xpubs: Vec<bitcoin::bip32::Xpub> = slots.iter().map(|s| s.xpub).collect();
    check_bip388_distinct_keys(&xpubs, BIP388_EXPORT_ALTERNATIVES)?;

    Ok(json!({
        "name": template.human_name(),
        "description_template": description_template,
        "keys_info": keys_info,
    }))
}
