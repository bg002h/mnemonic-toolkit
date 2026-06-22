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
        // SPEC §4 missing-info refusals are conceptually the right channel
        // for "this format does not support this template" — but for
        // Coldcard the user-facing refusal pointer ("use --format
        // bitcoin-core / sparrow for taproot watch-only setup") is
        // substantially more helpful than the generic §4 bullet
        // ("format_template_compatibility (this format does not represent
        // the resolved template)"). The per-template incompat refusals are
        // therefore surfaced as `ToolkitError::BadInput` with byte-exact
        // pointer text from inside `emit()`. By the time `emit()` runs,
        // `resolve_slots` has already backfilled per-slot fields (xpub /
        // fingerprint / path) and the dispatch site has set `threshold`
        // for multisig, so the genuine per-slot / global missing-info case
        // is compile-time-impossible. Phase 3 SpecterEmitter is the first
        // emitter that genuinely populates `MissingField::WalletName`.
        Vec::new()
    }

    fn emit(inputs: &EmitInputs) -> Result<String, ToolkitError> {
        use crate::template::CliTemplate;
        match inputs.template {
            Some(
                CliTemplate::WshMulti
                | CliTemplate::WshSortedMulti
                | CliTemplate::ShWshMulti
                | CliTemplate::ShWshSortedMulti
                | CliTemplate::TrMultiA
                | CliTemplate::TrSortedMultiA,
            ) => emit_coldcard_multisig_text(inputs),
            _ => emit_coldcard_generic_json(inputs),
        }
    }

    fn extension() -> &'static str {
        "json"
    }
}

/// Top-level Coldcard generic-export JSON shape. Field order matches the
/// canonical upstream sample (`firmware/docs/generic-wallet-export.md`):
/// `chain`, `xfp`, optionally `xpub`, `account`, one of `bipNN`. SPEC v0.8 §5.1
/// pins this order intentionally to mirror upstream byte-for-byte; emitting
/// fields in any other order would still be valid JSON but would deviate from
/// the reference sample that Coldcard firmware parses. Using `#[derive(Serialize)]`
/// (not `serde_json::Map`) so the output order is guaranteed regardless of
/// whether the crate-level `preserve_order` feature is enabled.
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
            ToolkitError::DescriptorParse(format!("--format coldcard: derive_pub /0/0: {e}",))
        })?;
    let first_addr = match template {
        CliTemplate::Bip44 => {
            Address::p2pkh(recv.to_pub(), inputs.network.network_kind()).to_string()
        }
        CliTemplate::Bip49 => {
            Address::p2shwpkh(&recv.to_pub(), inputs.network.network_kind()).to_string()
        }
        CliTemplate::Bip84 => {
            Address::p2wpkh(&recv.to_pub(), inputs.network.known_hrp()).to_string()
        }
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
        // v0.8.2 plumbing (FOLLOWUPS `coldcard-master-xpub-plumbing-pending`,
        // now resolved): `inputs.master_xpub_at_0` is `Some(Xpub)` when the
        // user supplied `--slot @0.master_xpub=<base58>`, `None` otherwise.
        xpub: inputs.master_xpub_at_0.as_ref().map(|x| x.to_string()),
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

/// SPEC v0.8 §5.2 — Coldcard multisig text emitter. Format reference:
/// <https://coldcard.com/docs/multisig>. Jade's `register_multisig.multisig_file`
/// accepts byte-identical text (SPEC §6); Phase 1.5 wires `JadeEmitter` to
/// delegate here directly.
pub(crate) fn emit_coldcard_multisig_text(inputs: &EmitInputs) -> Result<String, ToolkitError> {
    use crate::template::CliTemplate;

    let template = inputs.template.ok_or_else(|| {
        ToolkitError::BadInput("--format coldcard multisig text requires --template".into())
    })?;

    // tr-multi-a / tr-sortedmulti-a — refuse pending Coldcard firmware support
    // for BIP-388 / BIP-341 taproot multisig (track FOLLOWUPS entry
    // `coldcard-tr-multi-a-pending-firmware`).
    if matches!(
        template,
        CliTemplate::TrMultiA | CliTemplate::TrSortedMultiA
    ) {
        return Err(ToolkitError::BadInput(format!(
            "--format coldcard does not yet support --template {} — Coldcard firmware does not currently ingest taproot multisig text exports (tracked by FOLLOWUPS coldcard-tr-multi-a-pending-firmware). Use --format bitcoin-core (descriptor) or --format sparrow for taproot multisig watch-only setup.",
            template.human_name(),
        )));
    }

    // SPEC §5.2 `Format` field: P2WSH for wsh; P2SH-P2WSH for sh-wsh.
    // The legacy P2SH (bare `sh(multi(...))`) row is reserved but not in
    // the toolkit's current template set.
    let format_str = match template {
        CliTemplate::WshMulti | CliTemplate::WshSortedMulti => "P2WSH",
        CliTemplate::ShWshMulti | CliTemplate::ShWshSortedMulti => "P2SH-P2WSH",
        _ => unreachable!("non-multisig templates routed to generic JSON via emit()"),
    };

    let threshold = inputs.threshold.ok_or_else(|| {
        ToolkitError::BadInput("--format coldcard multisig text requires --threshold <K>".into())
    })?;
    let cosigner_count = inputs.resolved_slots.len();
    if cosigner_count < 2 {
        return Err(ToolkitError::BadInput(format!(
            "--format coldcard multisig text requires at least 2 cosigners (--slot @0..@N); got {cosigner_count}",
        )));
    }
    if (threshold as usize) < 1 || (threshold as usize) > cosigner_count {
        return Err(ToolkitError::BadInput(format!(
            "--threshold {threshold} out of range for {cosigner_count} cosigners",
        )));
    }

    // Wallet name: SPEC §5.2 truncates to ≤20 Unicode scalar values
    // (chars). `.truncate(20)` would slice at byte 20 and panic when that
    // byte lands inside a multi-byte UTF-8 sequence; `chars().take(20)`
    // operates on scalar values and is safe for non-ASCII input. The result
    // is the user-intuitive "first 20 characters".
    let name: String = inputs.wallet_name.chars().take(20).collect();

    // Per-slot bare origin (v0.37.9 — `ResolvedSlot::origin_path_bare()`)
    // carries the user-supplied origin or template-derived default; both
    // forms are normalized to `m/...` here.
    let normalize_path = |p: &str| -> String {
        if p.starts_with('m') {
            // Covers both `m/...` and bare `m` (Coldcard accepts both).
            p.to_string()
        } else if p.is_empty() {
            String::new()
        } else {
            format!("m/{p}")
        }
    };

    // SPEC §5.2: sortedmulti → lex-sort cosigners by xpub; multi → slot-index
    // order. cycle-13a H11-b (I-2 funds-safety pairing): build the SINGLE
    // sorted slot vector here and read EACH cosigner's path / fingerprint /
    // xpub from the SAME sorted slot below. NEVER index a separate slot-order
    // `derivations[i]` vector against this sorted loop — that scrambles
    // path↔xpub whenever sort-order ≠ slot-order (worse than `m/0'/0'`).
    let mut cosigners: Vec<&crate::synthesize::ResolvedSlot> =
        inputs.resolved_slots.iter().collect();
    if matches!(
        template,
        CliTemplate::WshSortedMulti | CliTemplate::ShWshSortedMulti
    ) {
        cosigners.sort_by(|a, b| a.xpub.to_string().cmp(&b.xpub.to_string()));
    }

    // cycle-13a H11-d (funds-safety): refuse rather than substitute a
    // placeholder origin into a steel backup. Empty origin = no faithful
    // per-cosigner export possible.
    if cosigners
        .iter()
        .any(|s| normalize_path(&s.origin_path_bare()).is_empty())
    {
        return Err(ToolkitError::BadInput(
            "--format coldcard multisig text: at least one cosigner has an empty origin \
             derivation path — a faithful per-cosigner export is not possible. Supply each \
             cosigner's origin path (`--slot @N.path=m/...`) so the exported `Derivation:` \
             line is correct; the toolkit refuses to substitute a placeholder origin into a \
             multisig wallet file."
                .to_string(),
        ));
    }

    // cycle-13a H11-a/b/c: emit a single shared `Derivation:` line when ALL
    // cosigner origins agree (byte-identical to the pre-cycle-13a happy path);
    // otherwise emit a per-cosigner `Derivation: <path>` immediately before
    // each `<XFP_master>: <xpub>` line, each read from the SAME sorted slot.
    let sorted_paths: Vec<String> = cosigners
        .iter()
        .map(|s| normalize_path(&s.origin_path_bare()))
        .collect();
    let homogeneous = sorted_paths.windows(2).all(|w| w[0] == w[1]);

    // Assemble the text. SPEC §5.2 line order:
    //   Name: ...
    //   Policy: K of N
    //   [Derivation: m/...]            (shared form only)
    //   Format: P2WSH | P2SH-P2WSH | P2SH
    //   [Derivation: m/...]            (per-cosigner form: before each xpub)
    //   <XFP>: xpub6...                (one per cosigner)
    //
    // The trait emit contract is "return the text body; the call-site adds
    // exactly one trailing newline via `writeln!`". So we join lines with
    // `\n` and let the caller (`cmd::export_wallet::run`) terminate.
    let mut lines: Vec<String> = Vec::with_capacity(4 + 2 * cosigners.len());
    lines.push(format!("Name: {name}"));
    lines.push(format!("Policy: {threshold} of {cosigner_count}"));
    if homogeneous {
        // All agree → single shared `Derivation:` line (the common case).
        lines.push(format!("Derivation: {}", sorted_paths[0]));
    }
    lines.push(format!("Format: {format_str}"));
    for (cs, path) in cosigners.iter().zip(sorted_paths.iter()) {
        if !homogeneous {
            // Divergent → emit this sorted slot's OWN path immediately before
            // its xpub (H11-b same-sorted-slot pairing — `path` is the path of
            // THIS `cs`, both read from the same zipped sorted slot).
            lines.push(format!("Derivation: {path}"));
        }
        // XFP uppercase 8-hex; xpub in BIP-32 base58 form (NOT SLIP-132 per
        // SPEC §5.2 bullet on cosigner-line shape).
        let xfp = cs.fingerprint.to_string().to_uppercase();
        lines.push(format!("{xfp}: {}", cs.xpub));
    }
    Ok(lines.join("\n"))
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

#[cfg(test)]
mod cycle13a_h11_tests {
    //! cycle-13a P3 (H11-d) — `emit_coldcard_multisig_text` refuses (never
    //! substitutes a placeholder origin) when any cosigner has an empty origin
    //! derivation path. This case is unreachable via the public CLI (the
    //! multisig template family always assigns a path; descriptor import
    //! refuses origin-less descriptors), so it is exercised at the emitter
    //! boundary with hand-built pathless slots.

    use super::emit_coldcard_multisig_text;
    use crate::error::ToolkitError;
    use crate::synthesize::ResolvedSlot;
    use crate::template::CliTemplate;
    use crate::wallet_export::{CheckedDescriptor, EmitInputs, TimestampArg, WalletScriptType};
    use bitcoin::bip32::{DerivationPath, Fingerprint, Xpub};
    use std::str::FromStr;

    const XPUB_A: &str = "xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX";
    const XPUB_B: &str = "xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6";
    // A structurally-valid 2-of-2 wsh sortedmulti descriptor (the body is not
    // re-validated against the slots by the emitter — it only needs the
    // BIP-380 `#csum` suffix to satisfy `CheckedDescriptor`).
    const DESC_2OF2: &str = "wsh(sortedmulti(2,xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX/<0;1>/*,xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6/<0;1>/*))#dummycsm";

    fn pathless_slot(xpub: &str) -> ResolvedSlot {
        ResolvedSlot {
            xpub: Xpub::from_str(xpub).unwrap(),
            fingerprint: Fingerprint::from_str("deadbeef").unwrap(),
            path: DerivationPath::default(), // empty origin → origin_path_bare() == ""
            entropy: None,
            master_xpub: None,
            language: None,
            _entropy_pin: None,
        }
    }

    /// #3 — all slots have empty `origin_path_bare()` → refuse via `BadInput`
    /// (exit 1); message names the empty-origin cause; NO `m/0'/0'` anywhere.
    #[test]
    fn export_coldcard_multisig_empty_origin_refuses() {
        let slots = vec![pathless_slot(XPUB_A), pathless_slot(XPUB_B)];
        let inputs = EmitInputs {
            canonical_descriptor: CheckedDescriptor::new(DESC_2OF2).unwrap(),
            resolved_slots: &slots,
            template: Some(CliTemplate::WshSortedMulti),
            script_type: WalletScriptType::P2wshMulti,
            network: crate::network::CliNetwork::Mainnet,
            account: 0,
            threshold: Some(2),
            threshold_user_supplied: true,
            master_xpub_at_0: None,
            wallet_name: "t",
            wallet_name_is_non_default: false,
            taproot_internal_key: None,
            range: (0, 0),
            timestamp: TimestampArg::Now,
            bitcoin_core_version: 0,
            bsms_form: Default::default(),
        };
        let err = emit_coldcard_multisig_text(&inputs).unwrap_err();
        match &err {
            ToolkitError::BadInput(msg) => {
                assert!(
                    msg.contains("empty origin"),
                    "H11-d refusal must name the empty-origin cause; got: {msg}"
                );
                assert!(
                    !msg.contains("m/0'/0'"),
                    "H11-d must never emit the m/0'/0' placeholder; got: {msg}"
                );
            }
            other => panic!("expected BadInput (exit 1), got: {other:?}"),
        }
        // Exit-code class: BadInput → exit 1.
        assert_eq!(err.exit_code(), 1, "BadInput must map to exit 1");
    }
}
