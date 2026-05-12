//! SPEC v0.8 §5 — Coldcard wallet-import emitter.
//!
//! Two artifact flavors (SPEC §5.1, §5.2):
//! - Generic JSON skeleton (singlesig templates: bip44/bip49/bip84).
//! - Multisig text (multisig templates; byte-identical bytes are accepted by
//!   Jade per §6, so the §6 Jade emitter delegates here).
//!
//! Phase 1.2 lands the bip84 mainnet singlesig path only; bip44/bip49 land in
//! Phase 1.3, multisig text lands in Phase 1.4. The single-flavor stubs that
//! refuse with byte-exact pointers (§5.1 `bip86`, §5.2 `tr-multi-a`) land
//! alongside their respective phases.

use super::{EmitInputs, MissingField, WalletFormatEmitter};
use crate::error::ToolkitError;
use serde::Serialize;

/// SPEC v0.8 §5 — `WalletFormatEmitter` impl for `--format coldcard`.
/// Phase 1.2: only the singlesig generic JSON skeleton (template bip44 /
/// bip49 / bip84) is implemented; multisig + bip86 refusals + jade
/// delegation arrive in later commits.
pub(crate) struct ColdcardEmitter;

impl WalletFormatEmitter for ColdcardEmitter {
    fn collect_missing(_inputs: &EmitInputs) -> Vec<MissingField> {
        // Phase 1.2 placeholder: the singlesig emit path uses only
        // resolved-slot data that the upstream validators already
        // guarantee. Future phases that add multisig + bip86 refusal +
        // missing-fingerprint refusal will populate this.
        Vec::new()
    }

    fn emit(inputs: &EmitInputs) -> Result<String, ToolkitError> {
        emit_coldcard_generic_json(inputs)
    }

    fn extension() -> &'static str {
        "json"
    }
}

/// Top-level Coldcard generic-export JSON shape. Field order matches the
/// canonical upstream sample (`firmware/docs/generic-wallet-export.md`):
/// `chain`, `xfp`, optionally `xpub`, `account`, one of `bipNN`. Using
/// `#[derive(Serialize)]` (not `serde_json::Map`) so the output order is
/// guaranteed regardless of whether the crate-level `preserve_order` feature
/// is enabled.
#[derive(Serialize)]
struct ColdcardGenericJson<'a> {
    chain: &'static str,
    xfp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    xpub: Option<String>,
    account: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    bip44: Option<ColdcardSubDerivation<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bip49: Option<ColdcardSubDerivation<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bip84: Option<ColdcardSubDerivation<'a>>,
}

/// Per-derivation sub-object (`bip44` / `bip49` / `bip84`). The `_pub` field
/// (SLIP-132 form) is absent for `bip44` (legacy p2pkh has no SLIP-132
/// variant) but present for `bip49` and `bip84`.
#[derive(Serialize)]
struct ColdcardSubDerivation<'a> {
    name: &'static str,
    deriv: String,
    xfp: String,
    xpub: String,
    #[serde(rename = "_pub", skip_serializing_if = "Option::is_none")]
    slip132_pub: Option<String>,
    first: String,
    #[serde(skip)]
    _marker: std::marker::PhantomData<&'a ()>,
}

/// SPEC v0.8 §5.1 — Coldcard generic JSON skeleton, singlesig templates only.
/// Top-level `xpub` is emitted iff `@0.master_xpub=` was supplied; otherwise
/// the field is omitted from the JSON object per §5.1 conditional-emission
/// fold (commit 284f349).
pub(crate) fn emit_coldcard_generic_json(inputs: &EmitInputs) -> Result<String, ToolkitError> {
    use crate::template::CliTemplate;
    use bitcoin::bip32::ChildNumber;
    use bitcoin::secp256k1::Secp256k1;
    use bitcoin::Address;

    let template = inputs.template.ok_or_else(|| {
        ToolkitError::BadInput(
            "--format coldcard requires --template (bip44 / bip49 / bip84); pass a recognized template or use a different format for descriptor passthrough".into(),
        )
    })?;

    // bip86 is not in the upstream Coldcard generic-wallet-export schema
    // (verified against firmware/docs/generic-wallet-export.md at upstream
    // master); refuse with the §5.1 byte-exact pointer.
    if matches!(template, CliTemplate::Bip86) {
        return Err(ToolkitError::BadInput(
            "--format coldcard does not yet support BIP-86 (P2TR) — Coldcard's generic-wallet-export schema documents only bip44/bip49/bip84. Use --format bitcoin-core (descriptor) or --format sparrow for taproot watch-only setup.".into(),
        ));
    }

    let (sub_object_name, sub_slot): (&'static str, ColdcardSubSlot) = match template {
        CliTemplate::Bip44 => ("p2pkh", ColdcardSubSlot::Bip44),
        CliTemplate::Bip49 => ("p2wpkh-p2sh", ColdcardSubSlot::Bip49),
        CliTemplate::Bip84 => ("p2wpkh", ColdcardSubSlot::Bip84),
        // Multisig templates: Phase 1.4 wires the §5.2 multisig text emitter.
        // Until then, refuse cleanly so the v0.7 byte-exact contract holds.
        _ => {
            return Err(ToolkitError::BadInput(format!(
                "--format coldcard does not yet support --template {} (Phase 1.4 wires Coldcard multisig text)",
                template.human_name(),
            )));
        }
    };

    // Single-slot input — singlesig templates have exactly one resolved slot.
    if inputs.resolved_slots.len() != 1 {
        return Err(ToolkitError::BadInput(format!(
            "--format coldcard + singlesig template expects exactly one --slot @0 input; got {}",
            inputs.resolved_slots.len(),
        )));
    }
    let slot = &inputs.resolved_slots[0];

    // ResolvedSlot already carries a parsed Xpub (BIP-32 form post-slip0132
    // normalization). Re-use directly.
    let xpub = slot.xpub;

    // bipNN.xfp is the parent fingerprint of the account xpub (BIP-32
    // serialization bytes 5–8). NOT the master fingerprint at top-level.
    let parent_fp_upper = xpub.parent_fingerprint.to_string().to_uppercase();

    // bipNN.first = address at /0/0 relative to the account xpub.
    let secp = Secp256k1::verification_only();
    let recv = xpub
        .derive_pub(
            &secp,
            &[
                ChildNumber::from_normal_idx(0).unwrap(),
                ChildNumber::from_normal_idx(0).unwrap(),
            ],
        )
        .map_err(|e| {
            ToolkitError::DescriptorParse(format!(
                "--format coldcard: derive_pub /0/0: {e}",
            ))
        })?;
    let first_addr = match template {
        CliTemplate::Bip44 => Address::p2pkh(&recv.to_pub(), inputs.network.network_kind()).to_string(),
        CliTemplate::Bip49 => Address::p2shwpkh(&recv.to_pub(), inputs.network.network_kind()).to_string(),
        CliTemplate::Bip84 => Address::p2wpkh(&recv.to_pub(), inputs.network.known_hrp()).to_string(),
        _ => unreachable!("singlesig templates only by match-arm above"),
    };

    // bipNN._pub: SLIP-132 variant. Coldcard omits `_pub` for bip44 (legacy
    // p2pkh has no SLIP-132 variant); emits zpub for bip84 and ypub for bip49.
    let slip132_pub: Option<String> = match template {
        CliTemplate::Bip44 => None,
        CliTemplate::Bip49 => Some(crate::slip0132::apply_xpub_prefix(
            &xpub,
            crate::slip0132::XpubPrefix::Ypub,
            inputs.network,
        )),
        CliTemplate::Bip84 => Some(crate::slip0132::apply_xpub_prefix(
            &xpub,
            crate::slip0132::XpubPrefix::Zpub,
            inputs.network,
        )),
        _ => unreachable!(),
    };

    // bipNN.deriv: m/<purpose>'/<coin>'/<account>'.
    let deriv = template.origin_path_str(inputs.network, inputs.account);

    let sub = ColdcardSubDerivation {
        name: sub_object_name,
        deriv,
        xfp: parent_fp_upper,
        xpub: xpub.to_string(),
        slip132_pub,
        first: first_addr,
        _marker: std::marker::PhantomData,
    };

    let top = ColdcardGenericJson {
        chain: chain_string(inputs.network),
        xfp: slot.fingerprint.to_string().to_uppercase(),
        // SPEC §5.1: top-level xpub emitted iff @0.master_xpub= was supplied.
        // Phase 1.2: synthesize.rs does not yet forward MasterXpub slot inputs
        // into ResolvedSlot, so this is unconditionally None for now. A
        // follow-on commit will plumb the field through.
        xpub: None,
        account: inputs.account,
        bip44: matches!(sub_slot, ColdcardSubSlot::Bip44).then(|| sub_clone(&sub)),
        bip49: matches!(sub_slot, ColdcardSubSlot::Bip49).then(|| sub_clone(&sub)),
        bip84: matches!(sub_slot, ColdcardSubSlot::Bip84).then_some(sub),
    };

    serde_json::to_string_pretty(&top)
        .map_err(|e| ToolkitError::BadInput(format!("--format coldcard: serialize: {e}")))
}

/// Which `bipNN` field to populate in the top-level struct.
enum ColdcardSubSlot {
    Bip44,
    Bip49,
    Bip84,
}

/// Cheap clone helper for the case where the sub-object goes into a non-bip84
/// field. `ColdcardSubDerivation` does not derive `Clone` because the
/// PhantomData lifetime parameter complicates the derive; this manual clone
/// works because all field types are owned/Copy.
fn sub_clone<'a>(s: &ColdcardSubDerivation<'a>) -> ColdcardSubDerivation<'a> {
    ColdcardSubDerivation {
        name: s.name,
        deriv: s.deriv.clone(),
        xfp: s.xfp.clone(),
        xpub: s.xpub.clone(),
        slip132_pub: s.slip132_pub.clone(),
        first: s.first.clone(),
        _marker: std::marker::PhantomData,
    }
}

/// `chain` field value per Coldcard's canonical schema: BTC mainnet, XTN
/// testnet+signet, XRT regtest.
fn chain_string(network: crate::network::CliNetwork) -> &'static str {
    use crate::network::CliNetwork::*;
    match network {
        Mainnet => "BTC",
        Testnet | Signet => "XTN",
        Regtest => "XRT",
    }
}
