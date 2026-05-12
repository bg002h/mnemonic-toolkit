//! `mnemonic export-wallet` format adapters.
//!
//! Submodule tree per `design/SPEC_export_wallet_v0_8.md` §12:
//! - `pipeline` — canonical descriptor build (multipath `<0;1>` form) and
//!   the `--descriptor` → BIP-388 `wallet_policy` interop pipeline.
//! - `bitcoin_core` — Bitcoin Core `importdescriptors` JSON emitter.
//! - `bip388` — BIP-388 `wallet_policy` JSON emitter.
//!
//! This module-root file holds the cross-format shared surface: byte-exact
//! refusal text constants (§3 watch-only, §7 format-stub), watch-only
//! validators, `TaprootInternalKey` / `TimestampArg` shared types, and the
//! `NUMS_XONLY_HEX` BIP-341 reference NUMS point.

mod bip388;
mod bitcoin_core;
mod pipeline;

pub(crate) use bip388::format_bip388_wallet_policy;
pub(crate) use bitcoin_core::format_bitcoin_core_importdescriptors;
pub(crate) use pipeline::{build_descriptor_string, descriptor_to_bip388_wallet_policy};

use crate::error::ToolkitError;
use crate::slot_input::{SlotInput, SlotSubkey};
use crate::synthesize::ResolvedSlot;
use serde_json::{json, Value};

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
    pub(super) fn to_json(self) -> Value {
        match self {
            TimestampArg::Now => json!("now"),
            TimestampArg::Unix(n) => json!(n),
        }
    }
}
