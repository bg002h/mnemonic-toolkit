//! `mnemonic export-wallet` format adapters.
//!
//! Submodule tree per `design/SPEC_export_wallet_v0_8.md` §12:
//! - `pipeline` — canonical descriptor build (multipath `<0;1>` form) and
//!   the `--descriptor` → BIP-388 `wallet_policy` interop pipeline.
//! - `bitcoin_core` — Bitcoin Core `importdescriptors` JSON emitter.
//! - `bip388` — BIP-388 `wallet_policy` JSON emitter.
//! - `coldcard` — Coldcard generic-wallet JSON + multisig text emitter.
//! - `jade` — Blockstream Jade multisig-text emitter.
//! - `sparrow` — Sparrow wallet JSON emitter.
//! - `specter` — Specter Desktop wallet JSON emitter.
//! - `electrum` — Electrum wallet-file JSON emitter.
//! - `green` — Blockstream Green text emitter.
//!
//! This module-root file holds the cross-format shared surface: byte-exact
//! refusal text constants, watch-only validators, `TaprootInternalKey` /
//! `TimestampArg` shared types, and the `NUMS_XONLY_HEX` BIP-341 reference
//! NUMS point.

mod bip388;
mod bitcoin_core;
mod bsms;
mod coldcard;
mod electrum;
mod green;
mod jade;
mod pipeline;
mod sparrow;
mod specter;

pub(crate) use bip388::Bip388Emitter;
pub(crate) use bitcoin_core::BitcoinCoreEmitter;
pub use bsms::BsmsForm;
pub(crate) use bsms::BsmsEmitter;
// v0.28.0 P8B (plan-doc §S.8) — re-export the per-script-type discriminator
// helper for use by `ToolkitError::BsmsTaprootRefused`'s message rendering
// at `error.rs::message`. Keeping the helper next to `bsms.rs::emit` (the
// sole construction site) preserves locality; the re-export here is the
// minimum surface needed for `error.rs` to render the variant.
pub(crate) use bsms::script_type_short_name;
pub(crate) use coldcard::ColdcardEmitter;
pub(crate) use electrum::ElectrumEmitter;
pub(crate) use green::GreenEmitter;
pub(crate) use jade::JadeEmitter;
pub(crate) use pipeline::build_descriptor_string;
pub(crate) use sparrow::SparrowEmitter;
pub(crate) use specter::SpecterEmitter;

use crate::error::ToolkitError;
use crate::network::CliNetwork;
use crate::slot_input::{SlotInput, SlotSubkey};
use crate::synthesize::ResolvedSlot;
use crate::template::CliTemplate;
use miniscript::{Descriptor as MsDescriptor, DescriptorPublicKey};
use serde_json::{json, Value};

/// v0.8 SPEC §3 byte-exact refusal text for secret-bearing slot inputs.
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

/// SPEC v0.8 §12 — script-type enum local to `crate::wallet_export`. Richer
/// than `crate::cmd::convert::ScriptType` (which is single-sig-only and scoped
/// to the `(Xpub, Address)` derivation edge in `cmd/convert.rs`). The new
/// per-format emitters dispatch on this enum (single vs multisig + envelope
/// flavor) to decide `chain`/`format`/SLIP-132 variant per format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Phase 0 adds the enum; Phase 1+ emitters consume it.
pub(crate) enum WalletScriptType {
    P2pkh,          // bip44
    P2shP2wpkh,     // bip49
    P2wpkh,         // bip84
    P2tr,           // bip86 (singlesig only — refused for Coldcard per SPEC §5.1)
    P2shMulti,      // legacy multisig (sh(multi(...)) / sh(sortedmulti(...)))
    P2shP2wshMulti, // sh-wsh-multi / sh-wsh-sortedmulti
    P2wshMulti,     // wsh-multi / wsh-sortedmulti
    P2trMulti,      // tr-multi-a / tr-sortedmulti-a
}

/// SPEC v0.8 §12 — map a `CliTemplate` to the corresponding `WalletScriptType`.
/// Used by emitters that operate on the template path (`--template`).
pub(crate) fn script_type_from_template(t: &CliTemplate) -> WalletScriptType {
    match t {
        CliTemplate::Bip44 => WalletScriptType::P2pkh,
        CliTemplate::Bip49 => WalletScriptType::P2shP2wpkh,
        CliTemplate::Bip84 => WalletScriptType::P2wpkh,
        CliTemplate::Bip86 => WalletScriptType::P2tr,
        CliTemplate::WshMulti | CliTemplate::WshSortedMulti => WalletScriptType::P2wshMulti,
        CliTemplate::ShWshMulti | CliTemplate::ShWshSortedMulti => {
            WalletScriptType::P2shP2wshMulti
        }
        CliTemplate::TrMultiA | CliTemplate::TrSortedMultiA => WalletScriptType::P2trMulti,
    }
}

/// SPEC v0.8 §12 — map a parsed `Descriptor` to the corresponding
/// `WalletScriptType`. Used by emitters that operate on the descriptor path
/// (`--descriptor`). The detection looks at the outermost wrapper plus, for
/// `tr(...)`, a substring check for `multi_a(` / `sortedmulti_a(` to discriminate
/// `P2tr` from `P2trMulti`. Returns `BadInput` for bare scripts (outside the
/// BIP-388 wallet-policy surface).
pub(crate) fn script_type_from_descriptor(
    d: &MsDescriptor<DescriptorPublicKey>,
) -> Result<WalletScriptType, ToolkitError> {
    use miniscript::descriptor::ShInner;
    use miniscript::Descriptor::*;
    match d {
        Pkh(_) => Ok(WalletScriptType::P2pkh),
        Wpkh(_) => Ok(WalletScriptType::P2wpkh),
        Sh(s) => match s.as_inner() {
            ShInner::Wpkh(_) => Ok(WalletScriptType::P2shP2wpkh),
            ShInner::Wsh(_) => Ok(WalletScriptType::P2shP2wshMulti),
            // Post-miniscript-#915, `sortedmulti` no longer has its own
            // `ShInner` variant; it surfaces as `Terminal::SortedMulti` inside
            // `ShInner::Ms`. Either way the wallet-format classification is
            // the same legacy `P2shMulti` (e.g., `sh(multi(K,...))` or
            // `sh(sortedmulti(K,...))`).
            ShInner::Ms(_) => Ok(WalletScriptType::P2shMulti),
        },
        Wsh(_) => Ok(WalletScriptType::P2wshMulti),
        Tr(t) => {
            // Phase 0 heuristic: render the tr descriptor and check for
            // `multi_a(` / `sortedmulti_a(` substrings to discriminate
            // taproot-multisig from taproot-singlesig. Miniscript's Display
            // is deterministic; structural walking can replace this in a
            // later phase (Phase 4 Electrum / Phase 2 Sparrow) if a corner
            // case demands it.
            let rendered = t.to_string();
            if rendered.contains("multi_a(") || rendered.contains("sortedmulti_a(") {
                Ok(WalletScriptType::P2trMulti)
            } else {
                Ok(WalletScriptType::P2tr)
            }
        }
        Bare(_) => Err(ToolkitError::DescriptorParse(
            "wallet-export descriptor must have a top-level Pkh/Wpkh/Sh/Wsh/Tr wrapper (bare scripts are outside the BIP-388 wallet-policy surface)".into(),
        )),
    }
}

/// SPEC v0.8 §4 — missing-info refusal field enumeration. Per the SPEC:
/// per-slot fields are discriminants 1-3 (`MasterFingerprint`, `DerivationPath`,
/// `Xpub`); globals are 4-7 (`ScriptType`, `Threshold`, `WalletName`,
/// `IncompatibleFormatForTemplate`). The deterministic refusal order surfaces
/// globals first (sorted by enum discriminant 4 → 5 → 6 → 7), then per-slot
/// entries grouped by enum discriminant (1, 2, 3) and ordered by slot index
/// within each discriminant.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)] // Phase 0 adds the enum; Phase 1+ emitters populate it.
pub(crate) enum MissingField {
    MasterFingerprint { slot: u8 },
    DerivationPath { slot: u8 },
    Xpub { slot: u8 },
    ScriptType,
    Threshold,
    WalletName,
    IncompatibleFormatForTemplate,
}

impl MissingField {
    /// Sort key for the SPEC §4 deterministic-order rule:
    /// `(group, enum_discriminant, slot)` where `group = 0` for globals
    /// (sorted first) and `group = 1` for per-slot.
    fn sort_key(&self) -> (u8, u8, u8) {
        match self {
            // Globals (group 0) — sorted by enum discriminant 4 → 7.
            MissingField::ScriptType => (0, 4, 0),
            MissingField::Threshold => (0, 5, 0),
            MissingField::WalletName => (0, 6, 0),
            MissingField::IncompatibleFormatForTemplate => (0, 7, 0),
            // Per-slot (group 1) — sorted by (discriminant, slot).
            MissingField::MasterFingerprint { slot } => (1, 1, *slot),
            MissingField::DerivationPath { slot } => (1, 2, *slot),
            MissingField::Xpub { slot } => (1, 3, *slot),
        }
    }

    fn bullet_line(&self) -> String {
        match self {
            MissingField::MasterFingerprint { slot } => format!(
                "master_fingerprint for slot @{slot} (supply via --slot @{slot}.fingerprint=<8-hex>)"
            ),
            MissingField::DerivationPath { slot } => format!(
                "derivation_path for slot @{slot} (supply via --slot @{slot}.path=<m/...> or use --template)"
            ),
            MissingField::Xpub { slot } => format!(
                "xpub for slot @{slot} (supply via --slot @{slot}.xpub=<base58>)"
            ),
            MissingField::ScriptType => "script_type (supply --template <bip44|bip49|bip84|bip86|wsh-sortedmulti|...> or --descriptor)".to_string(),
            MissingField::Threshold => "threshold (multisig templates require --threshold <K>)".to_string(),
            MissingField::WalletName => "wallet_name (supply --wallet-name <STRING>)".to_string(),
            MissingField::IncompatibleFormatForTemplate => "format_template_compatibility (this format does not represent the resolved template)".to_string(),
        }
    }
}

/// SPEC v0.8 §4 — sole site of byte-exact missing-info refusal-text
/// construction. `format` is the `--format <NAME>` string (e.g., `"coldcard"`).
/// `missing` is the unsorted set of missing fields per the calling emitter's
/// `collect_missing`; this function sorts deterministically and emits the
/// SPEC-pinned refusal shape. `user_text()` for `ToolkitError::ExportWalletMissingFields`
/// calls this directly and does NOT concatenate per-format header constants
/// separately — the SPEC mandates this is the unique construction site.
#[allow(dead_code)] // Phase 0 adds the function; Phase 1+ emitters route through ExportWalletMissingFields.
pub(crate) fn build_missing_fields_refusal(format: &str, missing: &[MissingField]) -> String {
    let mut sorted: Vec<&MissingField> = missing.iter().collect();
    sorted.sort_by_key(|f| f.sort_key());
    // NOTE: no leading `"error: "` — `ToolkitError::Display` (`error.rs:410`)
    // prepends that prefix uniformly for every error. The SPEC §4 byte-exact
    // shape pins a SINGLE prefix; including it here would produce
    // `error: error: ...` once `ExportWalletMissingFields` is wired.
    let mut s = format!(
        "mnemonic export-wallet --format {format} requires the following missing fields:\n"
    );
    for f in sorted {
        s.push_str("  - ");
        s.push_str(&f.bullet_line());
        s.push('\n');
    }
    s.push_str("Re-invoke with all missing fields supplied.");
    s
}

/// SPEC v0.8 §12 — shared trait every `--format` emitter implements.
///
/// `collect_missing`: per-format predicate that walks `EmitInputs` and
/// returns the set of `MissingField` entries this format requires but the
/// inputs do not provide. Returning a non-empty `Vec` lets the caller surface
/// the §4 byte-exact missing-info refusal (via
/// `ToolkitError::ExportWalletMissingFields` → `build_missing_fields_refusal`).
///
/// `emit`: produce the final byte-exact output string for the wallet-import
/// artifact. Returns `String` (not `Value`) because all six new formats and
/// the two existing formats produce text; JSON formats thin-wrap their
/// `serde_json::Value` builder with `to_string_pretty`. The caller writes
/// the returned bytes directly to stdout / `--output <path>`.
///
/// `extension`: file-extension hint for `--output <path>` validation /
/// suggestion. `"json"` for Bitcoin Core / BIP-388 / Sparrow / Specter /
/// Electrum / Coldcard generic; `"txt"` for Coldcard multisig / Jade / Green.
#[allow(dead_code)] // Phase 0 adds the trait; cmd::export_wallet::run wires it next.
pub(crate) trait WalletFormatEmitter {
    fn collect_missing(inputs: &EmitInputs) -> Vec<MissingField>;
    fn emit(inputs: &EmitInputs) -> Result<String, ToolkitError>;
    fn extension() -> &'static str;
}

/// v0.28.3 (A2): compile-time enforcement of the `EmitInputs.canonical_descriptor`
/// BIP-380 `#<8-char-csum>` suffix invariant. Pre-v0.28.3 the invariant was
/// documented at `wallet_export/bsms.rs:86-90` and enforced only by convention
/// at construction sites; a future code path that built `EmitInputs` from a
/// stripped-body descriptor would silently regress BSMS L2 + Specter
/// `descriptor` JSON field + Green plaintext (latent class surfaced by F9).
///
/// `CheckedDescriptor::new` validates the suffix and returns `Result` on
/// failure; `Deref<Target = str>` lets consumers continue to bind via
/// `inputs.canonical_descriptor` with auto-deref to `&str`.
#[derive(Debug, Clone, Copy)]
pub(crate) struct CheckedDescriptor<'a>(&'a str);

impl<'a> CheckedDescriptor<'a> {
    /// Construct a `CheckedDescriptor` from a descriptor string that MUST
    /// end with `#<8-char-csum>` per BIP-380. Returns `Err(BadInput)` if
    /// the suffix is missing, the wrong length, or not ASCII-alphanumeric.
    pub(crate) fn new(desc: &'a str) -> Result<Self, crate::error::ToolkitError> {
        let pos = desc.rfind('#').ok_or_else(|| {
            crate::error::ToolkitError::BadInput(format!(
                "CheckedDescriptor: missing BIP-380 `#<csum>` suffix in: {desc:?}"
            ))
        })?;
        let csum = &desc[pos + 1..];
        if csum.len() != 8 {
            return Err(crate::error::ToolkitError::BadInput(format!(
                "CheckedDescriptor: BIP-380 checksum must be 8 chars, got {} in: {desc:?}",
                csum.len()
            )));
        }
        if !csum.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err(crate::error::ToolkitError::BadInput(format!(
                "CheckedDescriptor: BIP-380 checksum must be ASCII-alphanumeric, got {csum:?} in: {desc:?}"
            )));
        }
        Ok(Self(desc))
    }

    /// Return the underlying descriptor string (with `#<csum>` suffix).
    #[allow(dead_code)] // Available for future callers; not used by current emitters (Deref covers them).
    pub(crate) fn as_str(&self) -> &'a str {
        self.0
    }
}

impl std::ops::Deref for CheckedDescriptor<'_> {
    type Target = str;
    fn deref(&self) -> &str {
        self.0
    }
}

impl std::fmt::Display for CheckedDescriptor<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// SPEC v0.8 §12 — single struct threaded through `WalletFormatEmitter::emit`
/// carrying all data each per-format emitter needs.
/// Built in `cmd::export_wallet::run` after template + slot resolution and
/// watch-only validation; the resulting reference is borrowed by emitters.
#[allow(dead_code)] // Phase 0 adds the struct; Phase 1+ emitters consume it.
pub(crate) struct EmitInputs<'a> {
    /// Canonical descriptor with BIP-380 `#<8-char-csum>` suffix. The
    /// `CheckedDescriptor<'_>` newtype (defined above) enforces the
    /// suffix at construction time; consumers bind via auto-deref to
    /// `&str` (e.g., `let line2 = inputs.canonical_descriptor;` in
    /// `bsms.rs`).
    pub canonical_descriptor: CheckedDescriptor<'a>,
    /// Resolved slots in slot-index order. Empty when `--descriptor` was used
    /// without `--template` (descriptor-passthrough path).
    pub resolved_slots: &'a [ResolvedSlot],
    /// `None` when `--descriptor` was used without `--template`.
    pub template: Option<CliTemplate>,
    /// Derived via `script_type_from_template` or `script_type_from_descriptor`.
    pub script_type: WalletScriptType,
    pub network: CliNetwork,
    pub account: u32,
    /// `Some(K)` for multisig templates; `None` for singlesig.
    pub threshold: Option<u8>,
    /// `true` when the user explicitly supplied `--threshold`. Phase 2's
    /// `SparrowEmitter::collect_missing` checks this flag: Sparrow refuses
    /// multisig templates without explicit threshold because its
    /// `defaultPolicy.miniscript.script` field uses `multi(K, ...)` /
    /// `sortedmulti(K, ...)` and silently defaulting `K = N` would publish
    /// a single-no-threshold Sparrow wallet that bypasses the K-of-N
    /// signing rule (UX rationale per SPEC §13 missing-threshold-refusal
    /// fixture row).
    pub threshold_user_supplied: bool,
    /// v0.8.2 SPEC §5.1 — depth-0 master xpub for slot @0 (`@0.master_xpub=`),
    /// when supplied. Consumed by `--format coldcard` singlesig emitter to
    /// populate the top-level `xpub` field; silently ignored by other
    /// formats (per the per-format ignored-input contract). `None` when the
    /// slot subkey was absent.
    pub master_xpub_at_0: Option<bitcoin::bip32::Xpub>,
    /// Resolved wallet name. For the template path, falls back to
    /// `<template-human-name>-<account>` when `--wallet-name` is absent;
    /// for the descriptor path, falls back to `"imported-descriptor"`.
    pub wallet_name: &'a str,
    /// `true` when the user explicitly supplied `--wallet-name`. Phase 3's
    /// `SpecterEmitter::collect_missing` checks this field: Specter requires
    /// an explicit wallet name (UX rationale per SPEC §13 R1-L1 fold).
    pub wallet_name_was_user_supplied: bool,
    pub taproot_internal_key: Option<TaprootInternalKey>,
    pub range: (u32, u32),
    pub timestamp: TimestampArg,
    pub bitcoin_core_version: u8,
    /// SPEC v0.27.0 §3.5 — `--bsms-form` selection for `--format bsms`.
    /// Silently ignored by every other emitter (per the per-format
    /// ignored-input contract).
    pub bsms_form: BsmsForm,
}

#[cfg(test)]
mod checked_descriptor_tests {
    //! v0.28.3 (A2) — unit tests for the `CheckedDescriptor<'_>` newtype
    //! that compile-time-enforces the `EmitInputs.canonical_descriptor`
    //! BIP-380 `#<8-char-csum>` suffix invariant. Forward-looking defensive
    //! engineering per the manual-v0.2.0 cycle's P1b R1 architect §F9
    //! Axis B observation; brainstorm at
    //! `design/BRAINSTORM_followups_abc_release_plan.md`.

    use super::CheckedDescriptor;

    const VALID_DESC: &str = "wpkh([5436d724/84'/0'/0']xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9/<0;1>/*)#tk4vnxy8";

    #[test]
    fn accepts_descriptor_with_canonical_8char_checksum() {
        let checked = CheckedDescriptor::new(VALID_DESC).expect("valid descriptor");
        assert_eq!(checked.as_str(), VALID_DESC);
    }

    #[test]
    fn rejects_missing_checksum_suffix() {
        let desc = "wpkh([5436d724/84'/0'/0']xpub.../<0;1>/*)";
        let err = CheckedDescriptor::new(desc).expect_err("missing checksum must error");
        let msg = format!("{err:?}");
        assert!(
            msg.contains("missing"),
            "expected missing-checksum error, got: {msg}"
        );
    }

    #[test]
    fn rejects_wrong_length_checksum() {
        let desc = "wpkh([5436d724/84'/0'/0']xpub.../<0;1>/*)#abc123";
        let err = CheckedDescriptor::new(desc).expect_err("wrong-length must error");
        let msg = format!("{err:?}");
        assert!(
            msg.contains("must be 8"),
            "expected length-rule error, got: {msg}"
        );
    }

    #[test]
    fn rejects_non_alphanumeric_checksum() {
        // 8 chars but contains non-alphanumeric — no embedded `#` so `rfind`
        // finds only the one delimiter.
        let desc = "wpkh([5436d724/84'/0'/0']xpub.../<0;1>/*)#abc!!!!!";
        let err = CheckedDescriptor::new(desc).expect_err("non-alphanumeric must error");
        let msg = format!("{err:?}");
        assert!(
            msg.contains("alphanumeric"),
            "expected alphanumeric-rule error, got: {msg}"
        );
    }

    #[test]
    fn deref_to_str_for_consumer_compat() {
        let checked = CheckedDescriptor::new(VALID_DESC).expect("valid");
        let s: &str = &checked;
        assert_eq!(s, VALID_DESC);
        assert!(checked.contains("wpkh"));
        assert!(checked.starts_with("wpkh"));
    }
}
