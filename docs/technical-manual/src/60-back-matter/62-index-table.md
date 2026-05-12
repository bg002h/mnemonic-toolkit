# Index

This table mirrors the page-numbered alphabetical index emitted by `makeindex` in the PDF render path. Every `\index{TERM}` marker placed in `src/**/*.md` MUST have a matching row below; the `tests/lint.sh` bidirectional check enforces this.

The rows are sorted alphabetically (case-insensitive). Add new rows as you add new `\index{}` markers.

| Term | Section |
|---|---|
| `@N` | [Conventions and Notation](#conventions-and-notation) |
| `abandon test mnemonic` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address), [Bundle Anatomy](#bundle-anatomy) |
| `address derivation` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `Alternative (use-site)` | [md-codec Rust API](#md-codec-rust-api) |
| `auto-dispatch` | [md1 Wire Format](#md1-wire-format) |
| `base58check` | [Network and Addressing](#network-and-addressing) |
| `BCH code` | [codex32 and BCH](#codex32-and-bch) |
| `bech32` | [codex32 and BCH](#codex32-and-bch) |
| `bifurcation (BIP-388 enforcement)` | [Anti-Collision Invariants](#anti-collision-invariants) |
| `BIP 32 master fingerprint` | [mk1 Wire Format](#mk1-wire-format) |
| `BIP-173` | [codex32 and BCH](#codex32-and-bch) |
| `BIP-32` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `BIP-341` | [md1 Wire Format](#md1-wire-format) |
| `BIP-380` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `BIP-388` | [Introduction](#introduction) |
| `BIP-388 distinct-key` | [Anti-Collision Invariants](#anti-collision-invariants) |
| `BIP-389` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address), [Bundle Anatomy](#bundle-anatomy) |
| `BIP-389 multipath` | [Conventions and Notation](#conventions-and-notation) |
| `BIP-39 entropy` | [ms1 Wire Format](#ms1-wire-format) |
| `BIP-39 mnemonic` | [ms1 Wire Format](#ms1-wire-format) |
| `BIP-39 wordlist` | [ms1 Wire Format](#ms1-wire-format) |
| `BIP-44` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `BIP-48` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `BIP-49` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `BIP-84` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `BIP-86` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `BIP-93` | [codex32 and BCH](#codex32-and-bch) |
| `BIP-93 design distance` | [codex32 and BCH](#codex32-and-bch) |
| `BitReader` | [md-codec Rust API](#md-codec-rust-api) |
| `BitWriter` | [md-codec Rust API](#md-codec-rust-api) |
| `Body (md-codec)` | [md-codec Rust API](#md-codec-rust-api) |
| `Body::KeyArg` | [md1 Wire Format](#md1-wire-format) |
| `Body::MultiKeys` | [md1 Wire Format](#md1-wire-format) |
| `Body::Variable` | [md1 Wire Format](#md1-wire-format) |
| `bundle` | [Bundle Anatomy](#bundle-anatomy) |
| `bundle envelope` | [Bundle Anatomy](#bundle-anatomy) |
| `bundle JSON envelope` | [Bundle Anatomy](#bundle-anatomy) |
| `bundle mode` | [Bundle Anatomy](#bundle-anatomy) |
| `bytecode header (mk1)` | [mk1 Wire Format](#mk1-wire-format) |
| `canonical_origin` | [md-codec Rust API](#md-codec-rust-api) |
| `canonicality rules` | [md1 Wire Format](#md1-wire-format) |
| `cascade-skip` | [Anti-Collision Invariants](#anti-collision-invariants) |
| `chunk_set_id` | [Bundle Anatomy](#bundle-anatomy) |
| `chunk_set_id (md1)` | [md1 Wire Format](#md1-wire-format) |
| `chunk_set_id (mk1)` | [mk1 Wire Format](#mk1-wire-format) |
| `chunk_set_id binding` | [Anti-Collision Invariants](#anti-collision-invariants) |
| `chunked header (md1)` | [md1 Wire Format](#md1-wire-format) |
| `ChunkHeader` | [md-codec Rust API](#md-codec-rust-api) |
| `CKDpub` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `codex32` | [codex32 and BCH](#codex32-and-bch) |
| `compact-73` | [mk1 Wire Format](#mk1-wire-format) |
| `compute_md1_encoding_id` | [md-codec Rust API](#md-codec-rust-api) |
| `compute_wallet_descriptor_template_id` | [md-codec Rust API](#md-codec-rust-api) |
| `compute_wallet_policy_id` | [md-codec Rust API](#md-codec-rust-api) |
| `ContextKind` | [md-codec Rust API](#md-codec-rust-api) |
| `cosigner` | [Introduction](#introduction) |
| `cosigner-mapping diagnostic` | [Anti-Collision Invariants](#anti-collision-invariants) |
| `cross-card binding` | [The m-format Star](#the-m-format-star) |
| `cross-card binding (bundle)` | [Anti-Collision Invariants](#anti-collision-invariants) |
| `cross_chunk_hash` | [mk1 Wire Format](#mk1-wire-format) |
| `decode_md1_string` | [md-codec Rust API](#md-codec-rust-api) |
| `decode_payload` | [md-codec Rust API](#md-codec-rust-api) |
| `derivation (md1)` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `derive (Cargo feature)` | [md-codec Rust API](#md-codec-rust-api) |
| `Descriptor (md-codec)` | [md-codec Rust API](#md-codec-rust-api) |
| `descriptor truncation` | [Bundle Anatomy](#bundle-anatomy) |
| `Descriptor::derive_address` | [md-codec Rust API](#md-codec-rust-api) |
| `DescriptorPublicKey` | [Shape Coverage](#shape-coverage) |
| `distinct-key rule` | [Anti-Collision Invariants](#anti-collision-invariants) |
| `divergent_paths` | [md1 Wire Format](#md1-wire-format) |
| `encode_md1_string` | [md-codec Rust API](#md-codec-rust-api) |
| `encode_payload` | [md-codec Rust API](#md-codec-rust-api) |
| `engraving card` | [Bundle Anatomy](#bundle-anatomy) |
| `Error (md-codec)` | [md-codec Rust API](#md-codec-rust-api) |
| `Error::AddressDerivationFailed` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `Error::AltCountOutOfRange` | [md-codec Rust API](#md-codec-rust-api) |
| `Error::BitStreamTruncated` | [md-codec Rust API](#md-codec-rust-api) |
| `Error::ChainIndexOutOfRange` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `Error::ChildCountOutOfRange` | [md-codec Rust API](#md-codec-rust-api) |
| `Error::ChunkCountExceedsMax` | [md-codec Rust API](#md-codec-rust-api) |
| `Error::ChunkCountOutOfRange` | [md-codec Rust API](#md-codec-rust-api) |
| `Error::ChunkedHeaderMalformed` | [mk1 Wire Format](#mk1-wire-format) |
| `Error::ChunkHeaderChunkedFlagMissing` | [md-codec Rust API](#md-codec-rust-api) |
| `Error::ChunkIndexGap` | [md-codec Rust API](#md-codec-rust-api) |
| `Error::ChunkIndexOutOfRange` | [md-codec Rust API](#md-codec-rust-api) |
| `Error::ChunkSetEmpty` | [md-codec Rust API](#md-codec-rust-api) |
| `Error::ChunkSetIdMismatch` | [md1 Wire Format](#md1-wire-format) |
| `Error::ChunkSetIdOutOfRange` | [md-codec Rust API](#md-codec-rust-api) |
| `Error::ChunkSetIncomplete` | [md-codec Rust API](#md-codec-rust-api) |
| `Error::ChunkSetInconsistent` | [md-codec Rust API](#md-codec-rust-api) |
| `Error::Codex32DecodeError` | [md-codec Rust API](#md-codec-rust-api) |
| `Error::Codex32EncodeError` | [md-codec Rust API](#md-codec-rust-api) |
| `Error::CrossChunkHashMismatch` | [mk1 Wire Format](#mk1-wire-format) |
| `Error::DecodeRecursionDepthExceeded` | [md-codec Rust API](#md-codec-rust-api) |
| `Error::DivergentPathCountMismatch` | [md-codec Rust API](#md-codec-rust-api) |
| `Error::EmptyTlvEntry` | [md-codec Rust API](#md-codec-rust-api) |
| `Error::ForbiddenTapTreeLeaf` | [md1 Wire Format](#md1-wire-format) |
| `Error::HardenedPublicDerivation` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `Error::InvalidPathIndicator` | [mk1 Wire Format](#mk1-wire-format) |
| `Error::InvalidPresenceByte` | [md-codec Rust API](#md-codec-rust-api) |
| `Error::InvalidXpubBytes` | [md-codec Rust API](#md-codec-rust-api) |
| `Error::InvalidXpubVersion` | [mk1 Wire Format](#mk1-wire-format) |
| `Error::KeyCountOutOfRange` | [md-codec Rust API](#md-codec-rust-api) |
| `Error::KGreaterThanN` | [md-codec Rust API](#md-codec-rust-api) |
| `Error::MalformedHeader` | [md-codec Rust API](#md-codec-rust-api) |
| `Error::MissingExplicitOrigin` | [md-codec Rust API](#md-codec-rust-api) |
| `Error::MissingPubkey` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `Error::MultipathAltCountMismatch` | [md-codec Rust API](#md-codec-rust-api) |
| `Error::NUMSSentinelConflict` | [md1 Wire Format](#md1-wire-format) |
| `Error::OperatorContextViolation` | [md1 Wire Format](#md1-wire-format) |
| `Error::OverrideOrderViolation` | [md-codec Rust API](#md-codec-rust-api) |
| `Error::PathDepthExceeded` | [md-codec Rust API](#md-codec-rust-api) |
| `Error::PayloadLengthMismatch` | [ms1 Wire Format](#ms1-wire-format) |
| `Error::PlaceholderFirstOccurrenceOutOfOrder` | [md-codec Rust API](#md-codec-rust-api) |
| `Error::PlaceholderIndexOutOfRange` | [md-codec Rust API](#md-codec-rust-api) |
| `Error::PlaceholderNotReferenced` | [md-codec Rust API](#md-codec-rust-api) |
| `Error::ReservedBitsSet` | [mk1 Wire Format](#mk1-wire-format) |
| `Error::ReservedPrefixViolation` | [ms1 Wire Format](#ms1-wire-format) |
| `Error::ReservedTagNotEmittedInV01` | [ms1 Wire Format](#ms1-wire-format) |
| `Error::ShareIndexNotSecret` | [ms1 Wire Format](#ms1-wire-format) |
| `Error::TagInvalidAlphabet` | [ms1 Wire Format](#ms1-wire-format) |
| `Error::TagOutOfRange` | [md-codec Rust API](#md-codec-rust-api) |
| `Error::ThresholdNotZero` | [ms1 Wire Format](#ms1-wire-format) |
| `Error::ThresholdOutOfRange` | [md-codec Rust API](#md-codec-rust-api) |
| `Error::TlvLengthExceedsRemaining` | [md-codec Rust API](#md-codec-rust-api) |
| `Error::TlvOrderingViolation` | [md-codec Rust API](#md-codec-rust-api) |
| `Error::UnexpectedStringLength` | [ms1 Wire Format](#ms1-wire-format) |
| `Error::UnknownTag` | [ms1 Wire Format](#ms1-wire-format) |
| `Error::UnsupportedCardType` | [mk1 Wire Format](#mk1-wire-format) |
| `Error::UnsupportedVersion` | [mk1 Wire Format](#mk1-wire-format) |
| `Error::VarintOverflow` | [md-codec Rust API](#md-codec-rust-api) |
| `Error::WireVersionMismatch` | [md1 Wire Format](#md1-wire-format) |
| `expand_per_at_n` | [md-codec Rust API](#md-codec-rust-api) |
| `ExpandedKey` | [md-codec Rust API](#md-codec-rust-api) |
| `fingerprint_flag` | [mk1 Wire Format](#mk1-wire-format) |
| `Fingerprints TLV` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `forked-BCH boundary` | [The m-format Star](#the-m-format-star) |
| `generator polynomial` | [codex32 and BCH](#codex32-and-bch) |
| `GF(32)` | [codex32 and BCH](#codex32-and-bch) |
| `GF(32) interpolation` | [Future Shares](#future-shares) |
| `h-notation` | [Anti-Collision Invariants](#anti-collision-invariants) |
| `hardened apostrophe folding` | [Anti-Collision Invariants](#anti-collision-invariants) |
| `Header (md-codec)` | [md-codec Rust API](#md-codec-rust-api) |
| `HRP` | [Conventions and Notation](#conventions-and-notation) |
| `HRP-mixing` | [codex32 and BCH](#codex32-and-bch) |
| `interpolate_at (rust-codex32)` | [Future Shares](#future-shares) |
| `is_nums` | [md1 Wire Format](#md1-wire-format) |
| `key_index` | [md1 Wire Format](#md1-wire-format) |
| `KeyCard` | [mk1 Wire Format](#mk1-wire-format) |
| `kiw` | [Conventions and Notation](#conventions-and-notation) |
| `LEB128` | [mk1 Wire Format](#mk1-wire-format) |
| `Legacy (script context)` | [Shape Coverage](#shape-coverage) |
| `long code` | [codex32 and BCH](#codex32-and-bch) |
| `LP4-ext varint` | [md1 Wire Format](#md1-wire-format) |
| `m-format constellation` | [Introduction](#introduction) |
| `mainnet` | [mk1 Wire Format](#mk1-wire-format) |
| `MAX_DECODE_DEPTH` | [md-codec Rust API](#md-codec-rust-api) |
| `MAX_PATH_COMPONENTS` | [md-codec Rust API](#md-codec-rust-api) |
| `md-codec` | [md-codec Rust API](#md-codec-rust-api) |
| `md-codec v0.32.0` | [md-codec Rust API](#md-codec-rust-api) |
| `md1` | [Introduction](#introduction) |
| `md1_xpub_match` | [Anti-Collision Invariants](#anti-collision-invariants) |
| `Md1EncodingId` | [md1 Wire Format](#md1-wire-format) |
| `md_codec (crate)` | [md-codec Rust API](#md-codec-rust-api) |
| `md_codec::bitstream` | [md-codec Rust API](#md-codec-rust-api) |
| `md_codec::canonical_origin` | [md-codec Rust API](#md-codec-rust-api) |
| `md_codec::canonicalize` | [md-codec Rust API](#md-codec-rust-api) |
| `md_codec::chunk` | [md-codec Rust API](#md-codec-rust-api) |
| `md_codec::codex32` | [md-codec Rust API](#md-codec-rust-api) |
| `md_codec::decode` | [md-codec Rust API](#md-codec-rust-api) |
| `md_codec::derive` | [md-codec Rust API](#md-codec-rust-api) |
| `md_codec::encode` | [md-codec Rust API](#md-codec-rust-api) |
| `md_codec::error` | [md-codec Rust API](#md-codec-rust-api) |
| `md_codec::header` | [md-codec Rust API](#md-codec-rust-api) |
| `md_codec::identity` | [md-codec Rust API](#md-codec-rust-api) |
| `md_codec::origin_path` | [md-codec Rust API](#md-codec-rust-api) |
| `md_codec::phrase` | [md-codec Rust API](#md-codec-rust-api) |
| `md_codec::tag` | [md-codec Rust API](#md-codec-rust-api) |
| `md_codec::tlv` | [md-codec Rust API](#md-codec-rust-api) |
| `md_codec::to_miniscript` | [md-codec Rust API](#md-codec-rust-api) |
| `md_codec::tree` | [md-codec Rust API](#md-codec-rust-api) |
| `md_codec::use_site_path` | [md-codec Rust API](#md-codec-rust-api) |
| `md_codec::validate` | [md-codec Rust API](#md-codec-rust-api) |
| `md_codec::varint` | [md-codec Rust API](#md-codec-rust-api) |
| `MD_REGULAR_CONST` | [codex32 and BCH](#codex32-and-bch) |
| `mk1` | [mk1 Wire Format](#mk1-wire-format) |
| `mk1 chunked-card grouping` | [Future Shares](#future-shares) |
| `MK_LONG_CONST` | [mk1 Wire Format](#mk1-wire-format) |
| `MK_REGULAR_CONST` | [mk1 Wire Format](#mk1-wire-format) |
| `MkField` | [Bundle Anatomy](#bundle-anatomy) |
| `mnemonic-toolkit` | [Introduction](#introduction) |
| `ms1` | [ms1 Wire Format](#ms1-wire-format) |
| `ms1 dense layout` | [Bundle Anatomy](#bundle-anatomy) |
| `ms1 four-case table` | [Anti-Collision Invariants](#anti-collision-invariants) |
| `multi-family bodies` | [md1 Wire Format](#md1-wire-format) |
| `multipath` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `multipath alternative` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `multiplicity (multiset)` | [Anti-Collision Invariants](#anti-collision-invariants) |
| `multiset semantics` | [Anti-Collision Invariants](#anti-collision-invariants) |
| `MultisigHybrid` | [Bundle Anatomy](#bundle-anatomy) |
| `MultisigInfo` | [Bundle Anatomy](#bundle-anatomy) |
| `MultisigMultiSource` | [Bundle Anatomy](#bundle-anatomy) |
| `MultisigWatchOnly` | [Bundle Anatomy](#bundle-anatomy) |
| `Node (md-codec)` | [md-codec Rust API](#md-codec-rust-api) |
| `node_to_descriptor` | [Shape Coverage](#shape-coverage) |
| `node_to_miniscript` | [Shape Coverage](#shape-coverage) |
| `NUMS` | [md1 Wire Format](#md1-wire-format) |
| `NUMS H-point` | [Shape Coverage](#shape-coverage) |
| `origin path` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `OriginPath` | [md1 Wire Format](#md1-wire-format) |
| `OriginPathOverrides TLV` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `PathComponent` | [md-codec Rust API](#md-codec-rust-api) |
| `PathDecl` | [md-codec Rust API](#md-codec-rust-api) |
| `PathDeclPaths` | [md-codec Rust API](#md-codec-rust-api) |
| `Payload::Entr` | [ms1 Wire Format](#ms1-wire-format) |
| `PBKDF2-HMAC-SHA512` | [ms1 Wire Format](#ms1-wire-format) |
| `Phrase` | [md-codec Rust API](#md-codec-rust-api) |
| `pkh` | [Shape Coverage](#shape-coverage) |
| `placeholder (@N)` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `policy_id_stub` | [mk1 Wire Format](#mk1-wire-format) |
| `polymod` | [codex32 and BCH](#codex32-and-bch) |
| `privacy-preserving mode` | [mk1 Wire Format](#mk1-wire-format) |
| `Pubkeys TLV` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `re_emit_bits` | [md-codec Rust API](#md-codec-rust-api) |
| `read_node` | [md-codec Rust API](#md-codec-rust-api) |
| `read_varint` | [md-codec Rust API](#md-codec-rust-api) |
| `regular code` | [codex32 and BCH](#codex32-and-bch) |
| `render_codex32_grouped` | [md-codec Rust API](#md-codec-rust-api) |
| `reserved-prefix byte (ms1)` | [ms1 Wire Format](#ms1-wire-format) |
| `reserved-prefix byte (v0.2)` | [Future Shares](#future-shares) |
| `RESERVED_TAG_TABLE` | [ms1 Wire Format](#ms1-wire-format) |
| `rust-codex32` | [ms1 Wire Format](#ms1-wire-format) |
| `script (BIP-388)` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `secp256k1` | [mk1 Wire Format](#mk1-wire-format) |
| `secret-bearing slot` | [Bundle Anatomy](#bundle-anatomy) |
| `Segwitv0 (script context)` | [Shape Coverage](#shape-coverage) |
| `sh (legacy)` | [Shape Coverage](#shape-coverage) |
| `sh(multi)` | [Shape Coverage](#shape-coverage) |
| `sh(wpkh)` | [Shape Coverage](#shape-coverage) |
| `SHA-256` | [md1 Wire Format](#md1-wire-format) |
| `share-set grouping` | [Future Shares](#future-shares) |
| `single-string header (md1)` | [md1 Wire Format](#md1-wire-format) |
| `SINGLE_STRING_PAYLOAD_BIT_LIMIT` | [md-codec Rust API](#md-codec-rust-api) |
| `SingleSigFull` | [Bundle Anatomy](#bundle-anatomy) |
| `SingleSigWatchOnly` | [Bundle Anatomy](#bundle-anatomy) |
| `standard-path table (mk1)` | [mk1 Wire Format](#mk1-wire-format) |
| `string-layer header (mk1)` | [mk1 Wire Format](#mk1-wire-format) |
| `Tag (md-codec)` | [md-codec Rust API](#md-codec-rust-api) |
| `Tag (ms1)` | [ms1 Wire Format](#ms1-wire-format) |
| `Tag::Check` | [md1 Wire Format](#md1-wire-format) |
| `Tag::ENTR` | [ms1 Wire Format](#ms1-wire-format) |
| `Tag::Multi` | [md1 Wire Format](#md1-wire-format) |
| `Tag::OriginPaths` | [md1 Wire Format](#md1-wire-format) |
| `Tag::Pkh` | [md1 Wire Format](#md1-wire-format) |
| `Tag::PkH` | [md1 Wire Format](#md1-wire-format) |
| `Tag::PkK` | [md1 Wire Format](#md1-wire-format) |
| `Tag::Sh` | [md1 Wire Format](#md1-wire-format) |
| `Tag::Thresh` | [md1 Wire Format](#md1-wire-format) |
| `Tag::Tr` | [md1 Wire Format](#md1-wire-format) |
| `Tag::Wpkh` | [md1 Wire Format](#md1-wire-format) |
| `Tag::Wsh` | [md1 Wire Format](#md1-wire-format) |
| `Tap (script context)` | [Shape Coverage](#shape-coverage) |
| `tap-leaf miniscript` | [Shape Coverage](#shape-coverage) |
| `taproot internal key` | [md1 Wire Format](#md1-wire-format) |
| `TapTree` | [Shape Coverage](#shape-coverage) |
| `target residue` | [codex32 and BCH](#codex32-and-bch) |
| `template (md1)` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `Terminal::Multi` | [Shape Coverage](#shape-coverage) |
| `testnet` | [mk1 Wire Format](#mk1-wire-format) |
| `Threshold (ms-codec v0.2)` | [Future Shares](#future-shares) |
| `TLV section` | [md1 Wire Format](#md1-wire-format) |
| `TLV_PUBKEYS` | [md-codec Rust API](#md-codec-rust-api) |
| `TlvSection` | [md-codec Rust API](#md-codec-rust-api) |
| `to_miniscript_descriptor` | [md-codec Rust API](#md-codec-rust-api) |
| `to_miniscript_descriptor` | [Shape Coverage](#shape-coverage) |
| `tr (key-path)` | [Shape Coverage](#shape-coverage) |
| `tr (multi-leaf)` | [Shape Coverage](#shape-coverage) |
| `tr (NUMS)` | [Shape Coverage](#shape-coverage) |
| `tr (single-leaf)` | [Shape Coverage](#shape-coverage) |
| `Unshared Secret form` | [ms1 Wire Format](#ms1-wire-format) |
| `unwrap_string` | [md-codec Rust API](#md-codec-rust-api) |
| `use-site path` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `use-site-path declaration` | [md1 Wire Format](#md1-wire-format) |
| `UseSitePath` | [md-codec Rust API](#md-codec-rust-api) |
| `v0.1 → v0.2-shares migration` | [Future Shares](#future-shares) |
| `validate_presence_byte` | [md-codec Rust API](#md-codec-rust-api) |
| `verify-bundle` | [Bundle Anatomy](#bundle-anatomy) |
| `VerifyCheck` | [Bundle Anatomy](#bundle-anatomy) |
| `walker normalisation` | [md1 Wire Format](#md1-wire-format) |
| `Wallet Instance ID` | [mk1 Wire Format](#mk1-wire-format) |
| `WalletDescriptorTemplateId` | [md-codec Rust API](#md-codec-rust-api) |
| `WalletPolicyId` | [md-codec Rust API](#md-codec-rust-api) |
| `watch-only slot` | [Bundle Anatomy](#bundle-anatomy) |
| `wildcard (BIP-389)` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `wire format` | [md1 Wire Format](#md1-wire-format) |
| `wpkh` | [Shape Coverage](#shape-coverage) |
| `wrap_payload` | [md-codec Rust API](#md-codec-rust-api) |
| `write_node` | [md-codec Rust API](#md-codec-rust-api) |
| `write_varint` | [md-codec Rust API](#md-codec-rust-api) |
| `wsh (miniscript)` | [Shape Coverage](#shape-coverage) |
| `xpub` | [Shape Coverage](#shape-coverage) |
| `XpubNotInPolicy` | [Anti-Collision Invariants](#anti-collision-invariants) |
