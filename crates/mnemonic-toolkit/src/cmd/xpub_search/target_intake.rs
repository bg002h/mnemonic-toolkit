//! Target intake helper for `xpub-search` modes (P1/P4).
//!
//! `--target-xpub <value>` accepts:
//!   - A bare SLIP-0132 xpub (any of `xpub|tpub|ypub|Ypub|zpub|Zpub|upub|Upub|vpub|Vpub`).
//!   - An mk1 bech32 card carrying an xpub.
//!
//! Multisig SLIP-0132 variants (`Ypub`/`Zpub`/`Upub`/`Vpub`) are accepted in
//! P1 path-of-xpub per plan §3.1 R0 I8 fold — they represent a cosigner xpub
//! at a multisig path, and P1's search includes BIP-48 multisig templates.

use crate::error::ToolkitError;
use crate::slip0132::normalize_xpub_prefix;
use bitcoin::bip32::Xpub;
use std::str::FromStr;

/// Resolve a `--target-xpub <value>` argument to a (canonical xpub,
/// original variant signal) pair.
///
/// Returns `(Xpub, Option<&'static str>)`. The second element is the
/// SLIP-0132 prefix signal (`"zpub"`, `"Zpub"`, etc.) when the input was
/// alt-prefixed and normalized to xpub/tpub; `None` for already-canonical
/// xpub/tpub input AND for mk1 cards (those carry xpubs directly).
pub fn resolve_target_xpub(value: &str) -> Result<(Xpub, Option<&'static str>), ToolkitError> {
    // Case-insensitive PROBE (v0.53.3 audit M11); the original tokens pass
    // to mk-codec, the case authority (it lowercase-normalizes; rejects mixed).
    if value.to_lowercase().starts_with("mk1") {
        // mk1 card route: tokenize whitespace (mk1 cards may have multiple
        // chunks separated by spaces) and decode via mk_codec.
        let tokens: Vec<&str> = value.split_whitespace().collect();
        let card = mk_codec::decode(&tokens).map_err(ToolkitError::from)?;
        // The mk_codec card carries its xpub as a `bitcoin::bip32::Xpub`
        // already. No variant signal (mk1 carries canonical xpub bytes).
        Ok((card.xpub, None))
    } else {
        // SLIP-0132 route: normalize alt-prefixes to xpub/tpub then parse.
        let (canonical_str, variant) = normalize_xpub_prefix(value)?;
        let xpub = Xpub::from_str(&canonical_str).map_err(ToolkitError::from)?;
        Ok((xpub, variant))
    }
}
