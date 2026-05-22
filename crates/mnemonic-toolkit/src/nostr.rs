//! Nostr-key wrappers — NIP-19 (`npub`/`nsec`) decode, BIP-340 even-y
//! normalization, and Bitcoin address/descriptor/WIF derivation for the
//! `mnemonic nostr` subcommand.
//!
//! A nostr key is a BIP-340 x-only secp256k1 key. Taproot (`p2tr`) is the
//! native mapping — the x-only key IS the taproot internal key, no parity
//! fabrication. Non-taproot (`p2pkh`/`p2wpkh`/`p2sh-p2wpkh`) uses the BIP-340
//! even-y compressed form `02‖x` (mirrors `cost/strip.rs` §11). For `nsec`,
//! the secret is normalized to even-y so the emitted WIF controls the emitted
//! address (see `normalize_to_even_y`).

#![allow(unused_imports)] // skeleton — imports consumed by Tasks A1/A2/A3

use crate::error::ToolkitError;
use bitcoin::secp256k1::{Parity, PublicKey, Secp256k1, SecretKey, Signing, Verification, XOnlyPublicKey};
use bitcoin::CompressedPublicKey;
use zeroize::Zeroizing;
