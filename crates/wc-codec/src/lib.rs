//! `wc-codec` — the **Word-Card value engine** for the m-format constellation.
//!
//! This crate implements the codec-agnostic RS / RAID / sync / word engine that
//! turns a `(SourceKind, version, payload)` triple into an engravable BIP-39
//! word sequence and back (see `design/IMPLEMENTATION_PLAN_word_card_encoding.md`).
//!
//! **P1 scope (this commit):** the *foundation* layer only —
//! - [`field`]: `GF(2^11)` arithmetic with the frozen primitive polynomial
//!   `x^11 + x^2 + 1` and primitive element `α = x` (plan §3);
//! - [`wordmap`]: the BIP-39 English symbol ↔ word map (symbol value == 11-bit
//!   index), sourced from the `bip39` crate as the single source of truth;
//! - [`regroup`]: bit-precise MSB-first 8 ↔ 11 regrouping (plan §4.1);
//! - [`pad`]: the frozen stripe zero-padding rule (plan §4.1 / M4).
//!
//! The RS value layer (P2), sync layer (P3), header/integrity/stop-sign (P4),
//! RAID (P5), and the toolkit adapter (P6) are intentionally NOT present yet.
//! The toolkit crate does not depend on `wc-codec` until P6.

pub mod field;
pub mod pad;
pub mod regroup;
pub mod wordmap;
