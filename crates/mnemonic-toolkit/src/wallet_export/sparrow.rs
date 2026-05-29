//! SPEC v0.8 §7 — Sparrow Wallet wallet-import emitter.
//!
//! Format reference (canonical model used by Sparrow's wallet-import path):
//! <https://github.com/sparrowwallet/drongo/blob/master/src/main/java/com/sparrowwallet/drongo/wallet/Wallet.java>.
//!
//! Shape: a single JSON object with these fields, in this order, mirroring
//! the upstream serializer:
//! `name`, `network`, `policyType`, `scriptType`, `defaultPolicy`, `keystores`.
//!
//! - `policyType`: `"SINGLE"` (singlesig) / `"MULTI"` (multisig).
//! - `scriptType`: `P2PKH` / `P2SH_P2WPKH` / `P2WPKH` / `P2TR` (singlesig)
//!   or `P2SH` / `P2SH_P2WSH` / `P2WSH` / `P2TR` (multisig — taproot multisig
//!   keeps `P2TR`, distinguished by descriptor passthrough in the miniscript).
//! - `defaultPolicy`: `{ name: "Default", miniscript: { script: "..." } }`.
//!   Sparrow's `Policy` class has exactly two serialized fields and threshold
//!   is implicit in the miniscript `multi(K, ...)` / `sortedmulti(K, ...)`
//!   argument count (SPEC §7 / R1-I1: do NOT emit a `numSignaturesRequired`
//!   field — it's a derived getter, not JSON-serialized).
//! - `keystores`: 1 element for SINGLE, N elements for MULTI, slot-index order.
//! - `masterFingerprint`: lowercase 8-hex.
//! - `extendedPublicKey`: BIP-32 form (NEVER SLIP-132 — Sparrow rejects).

use super::{EmitInputs, MissingField, WalletFormatEmitter, WalletScriptType};
use crate::error::ToolkitError;
use serde::Serialize;

/// SPEC v0.8 §7 — `WalletFormatEmitter` impl for `--format sparrow`.
pub(crate) struct SparrowEmitter;

impl WalletFormatEmitter for SparrowEmitter {
    fn collect_missing(inputs: &EmitInputs) -> Vec<MissingField> {
        // SPEC §7 + §13: Sparrow refuses multisig templates without explicit
        // `--threshold`. The dispatch site auto-defaults `threshold = N` for
        // emitters that accept the K=N default (BitcoinCore / BIP-388 /
        // Coldcard / Jade), but for Sparrow the K is published in the
        // `defaultPolicy.miniscript.script` `multi(K, ...)` argument and
        // silently defaulting would emit a wallet that LOOKS like K=N was
        // intentional. Use `threshold_user_supplied` (set by the dispatch
        // site to `args.threshold.is_some()`) as the discriminator —
        // `inputs.threshold` itself is always Some by this point.
        let mut out = Vec::new();
        if let Some(t) = inputs.template {
            if t.is_multisig() && !inputs.threshold_user_supplied {
                out.push(MissingField::Threshold);
            }
        }
        out
    }

    fn emit(inputs: &EmitInputs) -> Result<String, ToolkitError> {
        emit_sparrow_wallet_json(inputs)
    }

    fn extension() -> &'static str {
        "json"
    }
}

#[derive(Serialize)]
struct SparrowWallet<'a> {
    name: &'a str,
    network: &'static str,
    #[serde(rename = "policyType")]
    policy_type: &'static str,
    #[serde(rename = "scriptType")]
    script_type: &'static str,
    #[serde(rename = "defaultPolicy")]
    default_policy: SparrowPolicy,
    keystores: Vec<SparrowKeystore<'a>>,
}

#[derive(Serialize)]
struct SparrowPolicy {
    name: &'static str,
    miniscript: SparrowMiniscript,
}

#[derive(Serialize)]
struct SparrowMiniscript {
    script: String,
}

#[derive(Serialize)]
struct SparrowKeystore<'a> {
    label: &'a str,
    source: &'static str,
    #[serde(rename = "walletModel")]
    wallet_model: &'static str,
    #[serde(rename = "keyDerivation")]
    key_derivation: SparrowKeyDerivation,
    #[serde(rename = "extendedPublicKey")]
    extended_public_key: String,
}

#[derive(Serialize)]
struct SparrowKeyDerivation {
    #[serde(rename = "masterFingerprint")]
    master_fingerprint: String,
    derivation: String,
}

/// SPEC v0.8 §7 — Sparrow wallet JSON emitter.
pub(crate) fn emit_sparrow_wallet_json(inputs: &EmitInputs) -> Result<String, ToolkitError> {
    let template = inputs.template.ok_or_else(|| {
        ToolkitError::BadInput(
            "--format sparrow requires --template; descriptor passthrough is not supported by Sparrow's file-import surface".into(),
        )
    })?;

    if inputs.resolved_slots.is_empty() {
        return Err(ToolkitError::BadInput(
            "--format sparrow: at least one --slot @N.xpub=... required".into(),
        ));
    }

    let policy_type = if template.is_multisig() { "MULTI" } else { "SINGLE" };
    let script_type_str = sparrow_script_type(inputs.script_type);
    let network = sparrow_network(inputs.network);
    let script = build_miniscript_script(inputs, template)?;

    let keystores: Vec<SparrowKeystore> = inputs
        .resolved_slots
        .iter()
        .map(|s| SparrowKeystore {
            label: inputs.wallet_name,
            source: "SW_WATCH",
            wallet_model: "SPARROW",
            key_derivation: SparrowKeyDerivation {
                master_fingerprint: s.fingerprint.to_string().to_lowercase(),
                derivation: normalize_derivation(&s.origin_path_bare(), template, inputs),
            },
            extended_public_key: s.xpub.to_string(),
        })
        .collect();

    let wallet = SparrowWallet {
        name: inputs.wallet_name,
        network,
        policy_type,
        script_type: script_type_str,
        default_policy: SparrowPolicy {
            name: "Default",
            miniscript: SparrowMiniscript { script },
        },
        keystores,
    };

    serde_json::to_string_pretty(&wallet)
        .map_err(|e| ToolkitError::BadInput(format!("--format sparrow: serialize: {e}")))
}

/// SPEC §7: scriptType discriminant mirrored from Sparrow's `ScriptType` enum
/// (`drongo/.../wallet/ScriptType.java`).
fn sparrow_script_type(t: WalletScriptType) -> &'static str {
    match t {
        WalletScriptType::P2pkh => "P2PKH",
        WalletScriptType::P2shP2wpkh => "P2SH_P2WPKH",
        WalletScriptType::P2wpkh => "P2WPKH",
        WalletScriptType::P2tr => "P2TR",
        WalletScriptType::P2shMulti => "P2SH",
        WalletScriptType::P2shP2wshMulti => "P2SH_P2WSH",
        WalletScriptType::P2wshMulti => "P2WSH",
        // Sparrow's enum keeps taproot multisig as "P2TR"; the script-path
        // multi_a / sortedmulti_a is conveyed via the miniscript expression
        // (descriptor-passthrough), not by a separate scriptType discriminant.
        WalletScriptType::P2trMulti => "P2TR",
    }
}

/// SPEC §7: network strings Sparrow accepts.
fn sparrow_network(network: crate::network::CliNetwork) -> &'static str {
    use crate::network::CliNetwork::*;
    match network {
        Mainnet => "mainnet",
        Testnet => "testnet",
        Signet => "signet",
        Regtest => "regtest",
    }
}

/// SPEC §7: `defaultPolicy.miniscript.script`. For taproot multisig, use the
/// canonical descriptor directly (descriptor-passthrough — Sparrow understands
/// the full BIP-388 form). For all other templates, build the placeholder
/// expression with `@N/**` cosigner refs.
fn build_miniscript_script(
    inputs: &EmitInputs,
    template: crate::template::CliTemplate,
) -> Result<String, ToolkitError> {
    use crate::template::CliTemplate;
    let n = inputs.resolved_slots.len();
    match template {
        CliTemplate::Bip44 => Ok("pkh(@0/**)".to_string()),
        CliTemplate::Bip49 => Ok("sh(wpkh(@0/**))".to_string()),
        CliTemplate::Bip84 => Ok("wpkh(@0/**)".to_string()),
        CliTemplate::Bip86 => Ok("tr(@0/**)".to_string()),
        CliTemplate::WshMulti => Ok(format!("wsh({})", multi_arg("multi", inputs, n)?)),
        CliTemplate::WshSortedMulti => {
            Ok(format!("wsh({})", multi_arg("sortedmulti", inputs, n)?))
        }
        CliTemplate::ShWshMulti => Ok(format!("sh(wsh({}))", multi_arg("multi", inputs, n)?)),
        CliTemplate::ShWshSortedMulti => Ok(format!(
            "sh(wsh({}))",
            multi_arg("sortedmulti", inputs, n)?
        )),
        // Taproot multisig: descriptor passthrough per SPEC §7 trailing
        // paragraph. Sparrow's `defaultPolicy.miniscript.script` field is a
        // bare miniscript policy expression (not a BIP-380 descriptor with
        // checksum) — for every non-taproot template above we emit
        // `wpkh(@0/**)` / `wsh(sortedmulti(K,...))` with no `#checksum`.
        // Strip the canonical descriptor's `#<8-char>` suffix before
        // emitting so the taproot path matches the same shape contract
        // (Phase 2 R1 fold C-1: keeping the checksum would break Sparrow's
        // policy parser, which substring-matches on `script` for policy
        // detection).
        CliTemplate::TrMultiA | CliTemplate::TrSortedMultiA => {
            let desc: &str = &inputs.canonical_descriptor;
            let script = desc.rfind('#').map_or(desc, |pos| &desc[..pos]);
            Ok(script.to_string())
        }
    }
}

fn multi_arg(kind: &str, inputs: &EmitInputs, n: usize) -> Result<String, ToolkitError> {
    let k = inputs.threshold.ok_or_else(|| {
        ToolkitError::ExportWalletMissingFields {
            format: "sparrow",
            missing: vec![MissingField::Threshold],
        }
    })?;
    let placeholders: Vec<String> = (0..n).map(|i| format!("@{i}/**")).collect();
    Ok(format!("{kind}({k},{})", placeholders.join(",")))
}

/// Derivation field: normalize the per-slot bare origin path (multisig; from
/// `ResolvedSlot::origin_path_bare()`) or the template's singlesig origin path.
/// Sparrow expects `m/...` form without trailing slash and without descriptor
/// wildcards.
fn normalize_derivation(
    bare_path: &str,
    template: crate::template::CliTemplate,
    inputs: &EmitInputs,
) -> String {
    if template.is_multisig() {
        if bare_path.starts_with('m') {
            // Covers both `m/...` and bare `m`.
            bare_path.to_string()
        } else if bare_path.is_empty() {
            // Fall back to the multisig family default (BIP-48 wsh or BIP-87).
            template.origin_path_str(inputs.network, inputs.account)
        } else {
            format!("m/{bare_path}")
        }
    } else {
        // Singlesig: always use the template-derived origin path.
        template.origin_path_str(inputs.network, inputs.account)
    }
}
