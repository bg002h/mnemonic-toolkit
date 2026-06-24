//! `ms-codec` — reference implementation of the **ms1** backup format (HRP `ms`).
//!
//! ms1 is a Bitcoin self-custody backup format for BIP-39 entropy, layered atop
//! BIP-93 codex32 — vendored inline from Andrew Poelstra's `rust-codex32` (CC0)
//! at [`crate::codex32`] (Cycle-B, shape A; see `src/codex32/`). Designed for
//! steel-plate engraving alongside sibling formats `mk1` (xpubs) and `md1`
//! (descriptors). Every wire-format decision is judged against "does this make
//! a steel-plate backup more correct, or less?"
//!
//! See [`SPEC_ms_v0_1.md`](../../design/SPEC_ms_v0_1.md) for the full wire-format
//! specification and [`MIGRATION.md`](../../MIGRATION.md) for the v0.1 → v0.2
//! K-of-N share-encoding migration contract.
//!
//! # Quickstart
//!
//! ```
//! use ms_codec::{encode, decode, Payload, Tag};
//!
//! let entropy = vec![0xAAu8; 16]; // 12-word BIP-39 entropy
//! let s = encode(Tag::ENTR, &Payload::Entr(entropy.clone())).unwrap();
//! assert_eq!(s.len(), 50); // 12-word entr = 50-char ms1 string
//!
//! let (tag, payload) = decode(&s).unwrap();
//! assert_eq!(tag, Tag::ENTR);
//! assert_eq!(payload, Payload::Entr(entropy));
//! ```
//!
//! # v0.1 scope
//!
//! - **In:** BIP-39 entropy (16/20/24/28/32 B). Tag: `entr`.
//! - **Out:** Direct BIP-32 master seed (64 B) and serialized xpriv (78 B) —
//!   reserved-not-emitted in v0.1; deferred to v0.2+ with separate framing
//!   (they overflow BIP-93 codex32's length brackets when prepended with
//!   the v0.2-migration prefix byte). The master-seed backup use case is
//!   preserved via the application-layer routing
//!   `BIP-39 phrase → entropy → ms1 entr → engrave → recover → BIP-39 mnemonic
//!   → PBKDF2 → master seed`. See SPEC §1.2.

#![cfg_attr(not(test), deny(missing_docs))]

pub mod bch;
pub mod bch_decode;
pub mod codex32; // vendored BIP-93 codex32 (CC0, inlined; see src/codex32/ + LICENSE)
pub mod consts;
pub mod decode;
pub mod encode;
pub mod error;
pub mod inspect;
pub mod payload;
pub mod shares;
pub mod tag;

mod envelope; // crate-private; v0.2-migration seam

pub use decode::{decode, decode_with_correction, CorrectionDetail};
pub use encode::encode;
pub use error::{Error, Result};
pub use inspect::{inspect, InspectKind, InspectReport};
pub use payload::{Payload, PayloadKind};
pub use shares::{combine_shares, encode_shares, Threshold};
pub use tag::Tag;
