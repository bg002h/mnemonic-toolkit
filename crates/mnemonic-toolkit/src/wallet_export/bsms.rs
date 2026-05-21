//! SPEC v0.27.0 §3.5 — BIP-129 BSMS Round-2 emitter.
//!
//! Emits two output shapes:
//! - **4-line (default; BIP-129-canonical Round-2 plaintext):**
//!   ```text
//!   BSMS 1.0
//!   <descriptor>#<checksum>
//!   <path-restrictions>
//!   <first-address>
//!   ```
//!   Line 3 is the path-restrictions string per §3.5.1; line 4 is the
//!   wallet's first address at `/0/0` derived via
//!   `crate::derive_address::derive_first_address`.
//! - **2-line (lenient excerpt — symmetric with the v0.26.0 lenient input
//!   parser at `wallet_import/bsms.rs:95-102`):**
//!   ```text
//!   BSMS 1.0
//!   <descriptor>#<checksum>
//!   ```
//!
//! Form selection: `--bsms-form 2-line|4-line` on `export-wallet` (default
//! `4-line`).
//!
//! Taproot descriptors (`tr(...)`) are explicitly refused — BIP-386 is not in
//! BIP-129 §1 prerequisites. The refusal points users at `--format
//! bitcoin-core` or `--format sparrow` for taproot watch-only setup.
//!
//! v0.27.0 ingest does **not** add a 4-line lenient parser — closing the full
//! round-trip with a 4-line-faithful parser is tracked by FOLLOWUP
//! `bsms-bip129-full-cutover` (v0.28+). The 4-line emit produces output the
//! v0.26.0 2-line and 6-line lenient parser cannot ingest verbatim (their
//! shape grammars are different); the 2-line emit, conversely, is byte-
//! exact-idempotent under v0.26.0's `BsmsParser::parse` for the 2-line case.

use super::{EmitInputs, MissingField, WalletFormatEmitter, WalletScriptType};
use crate::derive_address::derive_first_address;
use crate::error::ToolkitError;
use crate::network::CliNetwork;
use clap::ValueEnum;
use miniscript::{Descriptor as MsDescriptor, DescriptorPublicKey, ForEachKey};
use std::str::FromStr;

/// SPEC v0.27.0 §3.5 — `--bsms-form` CLI value enum. `4-line` is the
/// BIP-129-canonical Round-2 plaintext shape; `2-line` is the lenient
/// excerpt symmetric with the v0.26.0 parser's lenient input form.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, ValueEnum)]
pub enum BsmsForm {
    #[value(name = "2-line")]
    TwoLine,
    #[value(name = "4-line")]
    #[default]
    FourLine,
}

pub(crate) struct BsmsEmitter;

impl WalletFormatEmitter for BsmsEmitter {
    fn collect_missing(_inputs: &EmitInputs) -> Vec<MissingField> {
        // BSMS surfaces refusals as `ToolkitError::BadInput` inside `emit()`
        // with pointer text — matches the Jade emitter's contract.
        Vec::new()
    }

    fn emit(inputs: &EmitInputs) -> Result<String, ToolkitError> {
        // BIP-129 §1 prerequisites: BIP-32 + BIP-39/BIP-85 seeds + BIP-43
        // purpose values. BIP-386 (taproot) is not in the list. Refuse
        // before any descriptor parse so the failure points at the format
        // mismatch (more helpful than a downstream miniscript parse error).
        //
        // v0.28.0 P8A/P8B (plan-doc §S.8): the refusal carries a
        // `ToolkitError::BsmsTaprootRefused { script_type }` payload so the
        // per-script-type discriminator (P2tr vs P2trMulti) surfaces in stderr,
        // along with a BIP-386 status note + FOLLOWUP slug pointer for
        // upstream-watch + alternative-format hints. Real taproot emit
        // remains upstream-blocked on BIP-129 §1 adding BIP-386; tracked at
        // FOLLOWUP `bsms-taproot-emit`.
        if matches!(
            inputs.script_type,
            WalletScriptType::P2tr | WalletScriptType::P2trMulti
        ) {
            return Err(ToolkitError::BsmsTaprootRefused {
                script_type: inputs.script_type,
            });
        }

        // Lines 1 + 2 are shared between the 2-line and 4-line shapes. Line 2
        // is `EmitInputs.canonical_descriptor` verbatim — its type
        // `CheckedDescriptor<'_>` (added v0.28.3 / A2) enforces the BIP-380
        // `#<8-char-csum>` suffix invariant at construction time. Pre-v0.28.3
        // the invariant was enforced by convention at construction sites;
        // post-v0.28.3 it's compile-time-guaranteed. `Deref<Target = str>`
        // means this binding continues to work as `&str` for `format!`.
        let line1 = "BSMS 1.0";
        let line2 = inputs.canonical_descriptor;

        let body = match inputs.bsms_form {
            BsmsForm::TwoLine => format!("{line1}\n{line2}"),
            BsmsForm::FourLine => {
                // Parse the canonical descriptor once for path-restrictions
                // discrimination + first-address derivation. Re-parsing here
                // (rather than threading a `&Descriptor` into `EmitInputs`)
                // keeps the cross-emitter contract minimal — other formats
                // do their own parse where needed (e.g., bitcoin_core.rs:48).
                let parsed = MsDescriptor::<DescriptorPublicKey>::from_str(
                    &inputs.canonical_descriptor,
                )
                .map_err(|e| {
                    ToolkitError::DescriptorParse(format!(
                        "--format bsms 4-line: descriptor re-parse: {e}"
                    ))
                })?;

                let line3 = path_restrictions_line(&parsed);
                let line4 = derive_first_address(&parsed, network_to_bitcoin(inputs.network))?;

                format!("{line1}\n{line2}\n{line3}\n{line4}")
            }
        };

        Ok(body)
    }

    fn extension() -> &'static str {
        "txt"
    }
}

/// SPEC §3.5.1 — line-3 path-restrictions emit rule.
///
/// Strategy: walk every cosigner key structurally via
/// `Descriptor::for_each_key`, extract each key's path-suffix, and decide
/// based on the unique set:
/// - All keys carry `<0;1>/*` → emit `/0/*,/1/*` (canonical multipath).
/// - All keys carry `/0/*` (single receive branch) → emit `/0/*`.
/// - Any other shape OR mixed per-key suffixes → `No path restrictions`
///   per BIP-129 §Round 2 (the path-restrictions field is the wallet's
///   coordinator-declared addressable scope; emitting one that does not
///   apply to all cosigners would misrepresent the bundle). A hand-rolled
///   `[fp/path]xpub/0/*,/1/*` per-key suffix falls into this arm — the
///   toolkit does NOT special-case the unioned single-branch shape
///   because miniscript's Display canonicalizes the multipath form to
///   `<0;1>/*`; if a future input shape needs it, file a FOLLOWUP.
///
/// The structural walk uses miniscript's canonical key `Display` form;
/// closes the architect-flagged divergent-multipath false-positive that
/// the prior string-contains heuristic carried.
fn path_restrictions_line(parsed: &MsDescriptor<DescriptorPublicKey>) -> &'static str {
    let mut suffixes: Vec<Option<String>> = Vec::new();
    parsed.for_each_key(|k| {
        suffixes.push(extract_key_suffix(&k.to_string()));
        true
    });
    if suffixes.is_empty() {
        return "No path restrictions";
    }
    let canonical_multipath = suffixes.iter().all(|s| s.as_deref() == Some("/<0;1>/*"));
    if canonical_multipath {
        return "/0/*,/1/*";
    }
    let receive_only = suffixes.iter().all(|s| s.as_deref() == Some("/0/*"));
    if receive_only {
        return "/0/*";
    }
    "No path restrictions"
}

/// Extract the path-suffix from a `DescriptorPublicKey`'s canonical `Display`
/// form. The Display shape is `[<fp>/<origin>]<xpub>[<suffix>]`; suffix
/// begins at the first non-base58 char (`/` or `<`) after the xpub body.
///
/// Returns `None` when the key has no suffix at all (xpub with no trailing
/// path) — distinguishing "no suffix" from "empty suffix" lets the caller's
/// `==` predicates short-circuit cleanly instead of comparing against
/// sentinel empty strings.
fn extract_key_suffix(key_str: &str) -> Option<String> {
    let after_origin = match key_str.rfind(']') {
        Some(i) => &key_str[i + 1..],
        None => key_str,
    };
    let suffix_start = after_origin.find(['/', '<'])?;
    Some(after_origin[suffix_start..].to_string())
}

/// v0.28.0 P8A (plan-doc §S.8) — short-name discriminator for
/// `WalletScriptType::P2tr` / `P2trMulti` used by the BSMS taproot refusal
/// message. Returns:
/// - `"P2tr"` for `WalletScriptType::P2tr` (taproot singlesig — bip86 / `tr(K)`).
/// - `"P2trMulti"` for `WalletScriptType::P2trMulti` (taproot multisig —
///   `tr(IK, multi_a(...))` / `tr(IK, sortedmulti_a(...))`).
///
/// Panics on any other variant — callers must gate the call on a
/// `matches!(inputs.script_type, P2tr | P2trMulti)` predicate (`emit()` does
/// this above). The panic is the load-bearing invariant the caller gate
/// preserves; renaming/removing either taproot variant would be a downstream
/// compiler error that surfaces this contract loudly.
pub(crate) fn script_type_short_name(st: &WalletScriptType) -> &'static str {
    match st {
        WalletScriptType::P2tr => "P2tr",
        WalletScriptType::P2trMulti => "P2trMulti",
        other => panic!(
            "script_type_short_name called with non-taproot variant {other:?}; \
             callers must gate on matches!(_, P2tr | P2trMulti)"
        ),
    }
}

/// Map the toolkit's `CliNetwork` to `bitcoin::Network` for the
/// `derive_address` helper. Mirrors the conversion `network.network_kind()`
/// pattern used by per-format emitters (e.g., coldcard.rs:173) but resolves
/// to the typed `bitcoin::Network` enum that miniscript's
/// `Descriptor::address(network)` consumes.
fn network_to_bitcoin(network: CliNetwork) -> bitcoin::Network {
    match network {
        CliNetwork::Mainnet => bitcoin::Network::Bitcoin,
        CliNetwork::Testnet => bitcoin::Network::Testnet,
        CliNetwork::Signet => bitcoin::Network::Signet,
        CliNetwork::Regtest => bitcoin::Network::Regtest,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// v0.28.0 P8A (plan-doc §S.8) — pin the two valid `script_type_short_name`
    /// returns. Discriminator strings appear verbatim in the v0.28.0
    /// `BsmsTaprootRefused` user-facing message; renaming either silently
    /// would break the corresponding stderr-assertion cells in
    /// `tests/cli_export_wallet_bsms.rs`. Pinning here gives a closer-to-the-
    /// source regression guard.
    #[test]
    fn script_type_short_name_p2tr() {
        assert_eq!(script_type_short_name(&WalletScriptType::P2tr), "P2tr");
    }

    #[test]
    fn script_type_short_name_p2tr_multi() {
        assert_eq!(
            script_type_short_name(&WalletScriptType::P2trMulti),
            "P2trMulti"
        );
    }

    /// Non-taproot variant must panic — preserves the caller-gate contract
    /// documented on the helper. If a future refactor accidentally widens
    /// the caller surface beyond the `matches!(_, P2tr | P2trMulti)` predicate
    /// in `emit()`, this cell fires loudly.
    #[test]
    #[should_panic(expected = "non-taproot variant")]
    fn script_type_short_name_panics_on_non_taproot() {
        let _ = script_type_short_name(&WalletScriptType::P2wshMulti);
    }
}
