//! `--format bitcoin-core-addresses` — an `addr()` watch list, the ONE route
//! into Bitcoin Core that this wallet actually survives.
//!
//! Realizes Phase 1b of `mnemonic-engrave/design/PLAN_wallet_file_export.md`.
//!
//! # Why this format exists
//!
//! The wallet's tier-4 spend path requires no signature. Bitcoin Core refuses
//! `importdescriptors` on that basis — *"is not sane: witnesses without
//! signature exist"* — on **every version through v31.1**, and unlike our side
//! the rule is **non-waivable** there: there is no flag. So
//! `--format bitcoin-core` emits a shape-perfect array that Core rejects
//! per-entry, and `--allow sigless-branch` converts an export-time refusal into
//! an import-time one.
//!
//! An `addr()` descriptor carries no spend policy at all — just a
//! scriptPubKey — so there is nothing for the sanity rule to object to. You can
//! **watch** this wallet in Core; you cannot **describe** it to Core. This
//! module is that distinction, made executable.
//!
//! # The shape, and why each field is what it is
//!
//! Each entry is a NON-RANGED, INACTIVE watch entry (the same shape
//! [`super::bitcoin_core::import_array_single`] uses for `nostr --import
//! readonly`):
//!
//! * **no `range`** — Core errors *"Range should not be specified for an
//!   un-ranged descriptor"*; `addr()` is not a range.
//! * **`active: false`** — an active descriptor is one Core derives from, and
//!   there is nothing here to derive from.
//! * **`internal`** — `false` for the receive chain, `true` for the change
//!   chain. Both chains are emitted: a change-blind watch wallet silently
//!   under-reports the balance.
//! * **`label`** — on the RECEIVE entries only, carrying the in-band caveat.
//!   Core refuses `label` together with `internal: true` (*"Internal addresses
//!   should not have a label"*, verified on v27.0), so a label on a change
//!   entry is not a cosmetic slip: it fails the import.
//!
//! No non-Core keys are emitted. Core v27.0 was measured to tolerate unknown
//! keys on an entry object, but that tolerance is undocumented, and a
//! funds-adjacent artifact should not depend on a behaviour no release note
//! promises.
//!
//! # What this does NOT do
//!
//! It does not make the DESCRIPTOR route work. Nothing here, in the help text,
//! or in any message may say otherwise.

use super::{EmitInputs, MissingField, WalletFormatEmitter};
use crate::error::ToolkitError;
use serde_json::{json, Value};

/// Addresses per chain when `--count` is not supplied.
///
/// **20, the BIP-44 gap limit.** This artifact's whole limitation is that it is
/// a fixed window which will not extend, so the default window should be at
/// least as wide as the gap every wallet uses to decide an account is empty.
/// (`mnemonic addresses --count` defaults to 10; it is an inspection tool, not
/// a watch window, and the two defaults are deliberately different.)
pub(crate) const DEFAULT_ADDRESS_COUNT: u32 = 20;

/// BIP-380 descriptor checksum for a body with no `#csum` suffix.
///
/// Same engine `Descriptor::to_string` uses, so an `addr()` string — which
/// rust-miniscript has no `Descriptor` variant for and therefore cannot
/// render — gets the identical checksum Core computes. Cross-checked two ways:
/// a Core-computed value is pinned in
/// `tests/cli_export_wallet_bitcoin_core_addresses.rs`, and the live gate in
/// `tests/bitcoind_addr_import.rs` re-derives every entry's checksum through
/// `getdescriptorinfo`.
fn checksummed(body_no_csum: &str) -> Result<String, ToolkitError> {
    use miniscript::descriptor::checksum::Engine as ChecksumEngine;
    let mut eng = ChecksumEngine::new();
    eng.input(body_no_csum).map_err(|e| {
        ToolkitError::BadInput(format!(
            "export-wallet: bitcoin-core-addresses: checksum engine rejected {body_no_csum:?}: {e}"
        ))
    })?;
    let csum = eng.checksum();
    Ok(format!("{body_no_csum}#{csum}"))
}

/// The in-band caveat, carried by every receive entry's `label`.
///
/// PLAN Phase 1b Acceptance: *"the emitted artifact states its own address
/// count and the no-derivation caveat in-band — a consumer who loads it must be
/// able to see from the file alone that it is a fixed list which will not
/// extend past the exported gap."*
///
/// ASCII only, on purpose: this string travels through a JSON file, an RPC
/// argument, Core's address book and back out of `getaddressesbylabel`.
fn caveat_label(wallet_name: &str, count: u32, change_count: u32, single_path: bool) -> String {
    let last = count.saturating_sub(1);
    let mut s = format!(
        "{wallet_name}: mnemonic bitcoin-core-addresses FIXED LIST of \
         {count} receive + {change_count} change addresses (indices 0-{last}). "
    );
    if single_path {
        s.push_str(
            "The descriptor is single-path, so it has no change chain and none was emitted. ",
        );
    }
    s.push_str(
        "NO DERIVATION: this file holds addresses, not the wallet descriptor, so Bitcoin \
         Core cannot extend past the exported gap. Re-export with a larger --count before \
         the last index is used.",
    );
    s
}

/// SPEC v0.8 §12 — `WalletFormatEmitter` impl for `--format
/// bitcoin-core-addresses`.
pub(crate) struct BitcoinCoreAddressesEmitter;

impl WalletFormatEmitter for BitcoinCoreAddressesEmitter {
    fn collect_missing(_inputs: &EmitInputs) -> Vec<MissingField> {
        // Everything this format needs is in the canonical descriptor.
        Vec::new()
    }

    fn emit(inputs: &EmitInputs) -> Result<String, ToolkitError> {
        let value = format_addr_list(
            &inputs.canonical_descriptor,
            inputs.address_count,
            inputs.network.to_bitcoin_network(),
            inputs.wallet_name,
            inputs.timestamp,
        )?;
        serde_json::to_string_pretty(&value)
            .map_err(|e| ToolkitError::BadInput(format!("export-wallet json: {e}")))
    }

    fn extension() -> &'static str {
        "json"
    }
}

/// Build the `importdescriptors` array: `count` receive entries, then `count`
/// change entries (or none, for a single-path descriptor).
pub(crate) fn format_addr_list(
    canonical_descriptor: &str,
    count: u32,
    network: bitcoin::Network,
    wallet_name: &str,
    timestamp: super::TimestampArg,
) -> Result<Value, ToolkitError> {
    if count == 0 {
        return Err(ToolkitError::BadInput(
            "--count 0 would emit an empty watch list, which imports successfully and \
             watches nothing. Supply --count >= 1."
                .to_string(),
        ));
    }

    // Re-parse of an ALREADY-ADMITTED string, so it is lenient (PLAN Phase 1
    // finding F-2: "every downstream re-parse becomes a lenient parse of an
    // already-admitted string"). This format exists FOR sigless wallets, so a
    // strict re-parse here would refuse exactly the input it is built for.
    let parsed = crate::parse_descriptor::parse_descriptor_lenient(canonical_descriptor)
        .map_err(|e| ToolkitError::DescriptorParse(format!("export-wallet re-parse: {e}")))?;

    let single_path = !parsed.is_multipath();
    if !single_path {
        // Mirror `format_bitcoin_core_importdescriptors`'s guard: the canonical
        // shape is `<0;1>`, and a 3+-branch descriptor has no receive/change
        // reading we could assert.
        let branches = parsed
            .clone()
            .into_single_descriptors()
            .map_err(|e| ToolkitError::DescriptorParse(format!("multipath split: {e}")))?
            .len();
        if branches != 2 {
            return Err(ToolkitError::DescriptorParse(format!(
                "expected 2 multipath splits (receive/change), got {branches}"
            )));
        }
    }

    let receive = crate::derive_address::derive_chain_addresses(
        &parsed,
        0,
        count,
        network,
        "bitcoin-core-addresses receive",
    )?;
    let change = if single_path {
        Vec::new()
    } else {
        crate::derive_address::derive_chain_addresses(
            &parsed,
            1,
            count,
            network,
            "bitcoin-core-addresses change",
        )?
    };

    let label = caveat_label(wallet_name, count, change.len() as u32, single_path);

    let mut entries: Vec<Value> = Vec::with_capacity(receive.len() + change.len());
    for a in &receive {
        entries.push(json!({
            "desc": checksummed(&format!("addr({a})"))?,
            "active": false,
            "internal": false,
            "timestamp": timestamp.to_json(),
            "label": label,
        }));
    }
    for a in &change {
        // NO `label` here: Core refuses `label` with `internal: true`.
        entries.push(json!({
            "desc": checksummed(&format!("addr({a})"))?,
            "active": false,
            "internal": true,
            "timestamp": timestamp.to_json(),
        }));
    }
    Ok(Value::Array(entries))
}

#[cfg(test)]
mod addr_list_tests {
    use super::*;
    use crate::wallet_export::TimestampArg;

    /// The journey's first receive address for the reasonably-complex wallet,
    /// and the checksum Bitcoin Core v27.0 computed for it via
    /// `getdescriptorinfo "addr(bc1qr6h…)"` on 2026-08-22.
    const JOURNEY_RECV_0: &str = "bc1qr6h5gahcaqa8a35p3ts0d2w6qvhmsn7dhunu5xd9kyculcgz3dwqf266zj";
    const CORE_CSUM_RECV_0: &str = "nf7wvmq9";

    /// The checksum claim is the one thing here we cannot derive from
    /// rust-miniscript's own `Display` (there is no `Descriptor::Addr`), so it
    /// is pinned against a value Core produced.
    #[test]
    fn checksum_agrees_with_bitcoin_core() {
        assert_eq!(
            checksummed(&format!("addr({JOURNEY_RECV_0})")).unwrap(),
            format!("addr({JOURNEY_RECV_0})#{CORE_CSUM_RECV_0}")
        );
    }

    /// A one-character change in the address must change the checksum —
    /// otherwise the "checksum agrees" test above would pass against a
    /// constant.
    #[test]
    fn the_checksum_actually_depends_on_the_address() {
        let a = checksummed(&format!("addr({JOURNEY_RECV_0})")).unwrap();
        let mutated = format!("{}q", &JOURNEY_RECV_0[..JOURNEY_RECV_0.len() - 1]);
        let b = checksummed(&format!("addr({mutated})")).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn count_zero_is_refused() {
        let err = format_addr_list(
            "wpkh([5436d724/84'/0'/0']xpub6Buxw9MmbkJr4iAw8SACNci2hQNuPCMwt9P7HkK62ZQAW9UcJaQ2bc6ARD892TToQQ9Rp6AHujHxBLXqAsvn5fRnLfnhKSRfz8qtaoyKUYx/<0;1>/*)",
            0,
            bitcoin::Network::Bitcoin,
            "w",
            TimestampArg::Unix(0),
        )
        .unwrap_err();
        assert!(err.to_string().contains("--count"), "{err}");
    }

    /// The caveat states the count, both chain sizes and the last index, and
    /// names the lever that extends it.
    #[test]
    fn the_caveat_states_the_window_and_how_to_widen_it() {
        let l = caveat_label("rcw", 20, 20, false);
        assert!(l.contains("20 receive + 20 change"), "{l}");
        assert!(l.contains("indices 0-19"), "{l}");
        assert!(l.contains("FIXED LIST"), "{l}");
        assert!(l.contains("NO DERIVATION"), "{l}");
        assert!(l.contains("--count"), "{l}");
        assert!(
            l.is_ascii(),
            "the label travels through RPC + Core's address book: {l}"
        );
        assert!(!l.contains("single-path"), "{l}");
    }

    /// A single-path descriptor's caveat says WHY there is no change chain,
    /// instead of leaving a reader to infer it from a count of 0.
    #[test]
    fn the_single_path_caveat_explains_the_missing_change_chain() {
        let l = caveat_label("rcw", 4, 0, true);
        assert!(l.contains("4 receive + 0 change"), "{l}");
        assert!(l.contains("single-path"), "{l}");
    }
}
