//! `mk-codec` — reference implementation of the **Mnemonic Key (MK)** backup format.
//!
//! Status: v0.1 implementation in progress. Wire format is locked
//! per the closure design at
//! `docs/superpowers/specs/2026-04-29-mk1-open-questions-closure-design.md`.
//! The bytecode and string-layer encoders/decoders land in subsequent
//! phases of `design/IMPLEMENTATION_PLAN_mk_v0_1.md`.
//!
//! See for the design surface:
//!
//! - `design/SPEC_mk_v0_1.md` — wire-format spec (post-closure)
//! - `design/DECISIONS.md` — rolling decisions log including D-1..D-15 and Q-1..Q-10 closures
//! - `design/IMPLEMENTATION_PLAN_mk_v0_1.md` — v0.1 implementation plan
//! - `design/FOLLOWUPS.md` — deferred items, pre-BIP-submission audit gates, cross-repo coordination
//! - `bip/bip-mnemonic-key.mediawiki` — BIP draft
//!
//! See for related sibling project:
//!
//! - [`bg002h/descriptor-mnemonic`](https://github.com/bg002h/descriptor-mnemonic) —
//!   the MD policy-template format and its `md-codec` reference implementation. MK
//!   is designed to engrave alongside MD policy cards for foreign-xpub multisig
//!   recovery.
//!
//! # Eventual factoring
//!
//! Per `design/DECISIONS.md` D-13, this crate initially **forks** the
//! BCH primitives from the sibling `md-codec`. The shared codex32-derived
//! plumbing extracts to a third crate (`mc-codex32`, likely a third
//! sibling repo) once both formats are implementation-validated; the
//! trigger condition (closure Q-9) is "both md-codec and mk-codec at v1.0
//! with cross-validated conformance vectors and stable public APIs."
//! Until then, fork-from-md-codec; both implementations carry their own
//! BCH-primitives copy.

#![cfg_attr(not(test), deny(missing_docs))]

pub mod bytecode;
pub mod consts;
pub mod error;
pub mod key_card;
pub mod string_layer;
pub mod test_vectors;

pub use consts::{
    CHUNKED_FRAGMENT_LONG_BYTES, CHUNKED_FRAGMENT_REGULAR_BYTES, CROSS_CHUNK_HASH_BYTES,
    GENERATOR_FAMILY, HRP, MAX_CHUNKS, MAX_PATH_COMPONENTS, MK_LONG_CONST, MK_REGULAR_CONST,
    NUMS_DOMAIN, ORIGIN_FINGERPRINT_BYTES, POLICY_ID_STUB_BYTES, SINGLE_STRING_LONG_BYTES,
    SINGLE_STRING_REGULAR_BYTES, XPUB_COMPACT_BYTES,
};
pub use error::{Error, Result};
pub use key_card::{KeyCard, decode, encode, encode_with_chunk_set_id};
