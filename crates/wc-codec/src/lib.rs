//! `wc-codec` — the **Word-Card value engine** for the m-format constellation.
//!
//! This crate implements the codec-agnostic RS / RAID / sync / word engine that
//! turns a `(SourceKind, version, payload)` triple into an engravable BIP-39
//! word sequence and back (see `design/IMPLEMENTATION_PLAN_word_card_encoding.md`).
//!
//! **P1/P2 scope (so far):**
//! - [`field`]: `GF(2^11)` arithmetic with the frozen primitive polynomial
//!   `x^11 + x^2 + 1` and primitive element `α = x` (plan §3);
//! - [`wordmap`]: the BIP-39 English symbol ↔ word map (symbol value == 11-bit
//!   index), sourced from the `bip39` crate as the single source of truth;
//! - [`regroup`]: bit-precise MSB-first 8 ↔ 11 regrouping (plan §4.1);
//! - [`pad`]: the frozen stripe zero-padding rule (plan §4.1 / M4);
//! - [`rs`] (**P2**): the systematic evaluation-form Reed–Solomon value layer
//!   — encode (interpolate + evaluate), decode (Gao partial-GCD with erasure
//!   puncturing), append-only prefix-extensible parity (plan §3 / §4.1).
//! - [`sync`] (**P3**): the structural sync / checkpoint layer — checkpoint word
//!   codec (marker + block-index mod 8 + CRC-5), `interleave` (insert
//!   checkpoints), and `sync_classify` (trichotomy + realignment + bounded
//!   single-deletion candidates / whole-block erasures), plan §4.3.
//!
//! The header/integrity/stop-sign (P4), RAID (P5), and the toolkit adapter (P6)
//! are intentionally NOT present yet. The toolkit crate does not depend on
//! `wc-codec` until P6.

pub mod field;
pub mod pad;
mod poly;
pub mod regroup;
pub mod rs;
pub mod sync;
pub mod wordmap;
