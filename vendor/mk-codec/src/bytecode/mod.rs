//! Bytecode layer — `KeyCard` ↔ canonical bytecode (pre-chunking).
//!
//! Per `design/SPEC_mk_v0_1.md` §3 and `bip/bip-mnemonic-key.mediawiki`
//! §"Bytecode layer". The bytecode is the byte sequence emitted between
//! the string-layer chunk header and the cross-chunk integrity hash;
//! the string-layer (BCH, HRP-mixing, chunk reassembly) lives in
//! `crate::string_layer` (Phase 5).
//!
//! Submodules:
//!
//! - [`header`]: 1-byte bytecode header (version + fingerprint flag)
//! - [`path`]: standard-table dictionary + `0xFE` explicit-path codec
//! - [`xpub_compact`]: 73-byte compact xpub form with depth/child_number
//!   reconstruction from `origin_path`
//! - [`encode`]: top-level `KeyCard → Vec<u8>` encoder
//! - [`decode`]: top-level `Vec<u8> → KeyCard` decoder

pub mod decode;
pub mod encode;
pub mod header;
pub mod path;
pub mod xpub_compact;

#[cfg(test)]
pub(crate) mod test_helpers;

pub use decode::decode_bytecode;
pub use encode::encode_bytecode;
pub use header::BytecodeHeader;
pub use path::{STANDARD_PATHS, decode_path, encode_path, lookup_indicator, lookup_path};
pub use xpub_compact::{XpubCompact, decode_xpub_compact, encode_xpub_compact, reconstruct_xpub};
