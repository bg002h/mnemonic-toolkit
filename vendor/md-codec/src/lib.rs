//! # `md-codec`
//!
//! Reference implementation of the **Mnemonic Descriptor (MD)** format —
//! an engravable backup format for [BIP 388 wallet policies][bip388].
//!
//! [bip388]: https://github.com/bitcoin/bips/blob/master/bip-0388.mediawiki
//!
//! v0.30 wire format: bit-aligned payload, sparse per-`@N` TLV overrides,
//! 5-bit single-payload header (4-bit version=4 + `divergent_paths` flag),
//! 6-bit bytecode tag space, decoder auto-dispatch between single and chunked
//! payloads via the first 5-bit symbol's bit 0, symbol-aligned codex32
//! wrapping with HRP `"md"`. See `design/SPEC_v0_30_wire_format.md` for the
//! normative spec.

pub mod bch;
pub mod bch_decode;

pub mod bitstream;
pub mod canonical_origin;
pub mod canonicalize;
pub mod chunk;
pub mod codex32;
pub mod decode;
pub mod derive;
pub mod encode;
pub mod error;
pub mod header;
pub mod identity;
mod nums;
pub mod origin_path;
pub mod phrase;
// The `@N`-template renderer is pure AST string-walking (no miniscript/derive
// dependency); it sources the NUMS H-point from the ungated `nums` module, so
// it is unconditional — available with or without the `derive` feature.
pub mod render;
pub mod tag;
pub mod test_vectors;
pub mod tlv;
#[cfg(feature = "derive")]
pub mod to_miniscript;
pub mod tree;
pub mod use_site_path;
pub mod validate;
pub mod varint;

pub use canonicalize::canonicalize_placeholder_indices;
pub use chunk::{
    ChunkHeader, CorrectionDetail, decode_with_correction, derive_chunk_set_id, reassemble, split,
};
pub use decode::{decode_md1_string, decode_payload};
pub use encode::{Descriptor, encode_md1_string, encode_payload};
pub use error::Error;
pub use header::Header;
pub use identity::{
    Md1EncodingId, WalletDescriptorTemplateId, WalletPolicyId, compute_md1_encoding_id,
    compute_wallet_descriptor_template_id, compute_wallet_policy_id, validate_presence_byte,
};
pub use origin_path::{OriginPath, PathComponent, PathDecl, PathDeclPaths};
pub use phrase::Phrase;
pub use render::{RenderError, descriptor_to_template};
pub use tag::Tag;
pub use tlv::TlvSection;
#[cfg(feature = "derive")]
pub use to_miniscript::{
    has_hardened_use_site, to_miniscript_descriptor, to_miniscript_descriptor_multipath,
};
