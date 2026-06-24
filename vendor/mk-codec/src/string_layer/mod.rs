//! String layer — `KeyCard` ↔ `Vec<String>` (BCH + chunked-header reassembly).
//!
//! Wraps the bytecode-layer codec ([`crate::bytecode`]) with the
//! BIP 93–derived BCH error-correction layer (forked from `md-codec` per
//! `design/DECISIONS.md` D-13) and the closure-locked string-layer header
//! structure (per Q-5 — 2-symbol single-string + 8-symbol chunked).
//!
//! Submodules:
//!
//! - [`bch`]: BCH polymod, syndrome decoder, and bech32-alphabet helpers.
//! - [`bch_decode`]: syndrome-based BM/Forney decoder (impl detail of
//!   `bch`). `pub(crate)` for cross-module test fixture access; not part
//!   of the user-facing API.
//! - [`header`]: 5-bit-symbol-aligned `StringLayerHeader`
//!   (`SingleString` + `Chunked` variants).
//! - [`chunk`]: stream split + reassemble with `cross_chunk_hash`
//!   integrity check.
//!
//! The public entry points wired up here ([`encode`] / [`encode_with_chunk_set_id`]
//! / [`decode`]) are the layer-3 boundary; `crate::key_card` re-exports them.

pub mod bch;
// v0.3.1: promoted from `pub(crate)` so downstream consumers (toolkit
// `repair` feature) can call `decode_regular_errors` / `decode_long_errors`
// for non-mk HRPs (ms, md — all 3 share the BIP-93 BCH(93,80,8) generator).
pub mod bch_decode;
pub mod chunk;
pub mod header;

mod pipeline;

pub use bch::{
    ALPHABET, BchCode, CaseStatus, CorrectionResult, DecodedString, SEPARATOR, bch_correct_long,
    bch_correct_regular, bch_create_checksum_long, bch_create_checksum_regular, bch_verify_long,
    bch_verify_regular, bytes_to_5bit, case_check, decode_string, encode_5bit_to_string,
    five_bit_to_bytes, hrp_expand,
};
pub use chunk::{reassemble_from_chunks, split_into_chunks};
pub use header::StringLayerHeader;
pub use pipeline::{decode, encode, encode_with_chunk_set_id};
