# Index

This table mirrors the page-numbered alphabetical index emitted by `makeindex` in the PDF render path. Every `\index{TERM}` marker placed in `src/**/*.md` MUST have a matching row below; the `tests/lint.sh` bidirectional check enforces this.

The rows are sorted alphabetically (case-insensitive). Add new rows as you add new `\index{}` markers.

| Term | Section |
|---|---|
| `abandon test mnemonic` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address), [Bundle Anatomy](#bundle-anatomy) |
| `address derivation` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `ALPHABET (bech32)` | [mk-codec Rust API](#mk-codec-rust-api) |
| `Alternative (use-site)` | [md-codec Rust API](#md-codec-rust-api) |
| `auto-dispatch` | [md1 Wire Format](#md1-wire-format) |
| `base58check` | [Network and Addressing](#network-and-addressing) |
| `BCH code` | [codex32 and BCH](#codex32-and-bch) |
| `bch_code_for_length` | [mk-codec Rust API](#mk-codec-rust-api) |
| `BchCode` | [mk-codec Rust API](#mk-codec-rust-api) |
| `bch_correct_long` | [mk-codec Rust API](#mk-codec-rust-api) |
| `bch_correct_regular` | [mk-codec Rust API](#mk-codec-rust-api) |
| `bch_create_checksum_long` | [mk-codec Rust API](#mk-codec-rust-api) |
| `bch_create_checksum_regular` | [mk-codec Rust API](#mk-codec-rust-api) |
| `bch_verify_long` | [mk-codec Rust API](#mk-codec-rust-api) |
| `bch_verify_regular` | [mk-codec Rust API](#mk-codec-rust-api) |
| `bech32` | [codex32 and BCH](#codex32-and-bch) |
| `bifurcation (BIP-388 enforcement)` | [Anti-Collision Invariants](#anti-collision-invariants) |
| `binary-only crate` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `bind_descriptor_keys` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `BIP-173` | [codex32 and BCH](#codex32-and-bch) |
| `BIP-32` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `BIP 32 master fingerprint` | [mk1 Wire Format](#mk1-wire-format) |
| `BIP-341` | [md1 Wire Format](#md1-wire-format) |
| `BIP-380` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `BIP-388 distinct-key` | [Anti-Collision Invariants](#anti-collision-invariants) |
| `BIP-388` | [Introduction](#introduction) |
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
| `BitcoinErrorKind` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `BitReader` | [md-codec Rust API](#md-codec-rust-api) |
| `BitWriter` | [md-codec Rust API](#md-codec-rust-api) |
| `Body::KeyArg` | [md1 Wire Format](#md1-wire-format) |
| `Body (md-codec)` | [md-codec Rust API](#md-codec-rust-api) |
| `Body::MultiKeys` | [md1 Wire Format](#md1-wire-format) |
| `Body::Variable` | [md1 Wire Format](#md1-wire-format) |
| `build_descriptor` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `Bundle::any_secret_bearing` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `bundle` | [Bundle Anatomy](#bundle-anatomy) |
| `bundle envelope` | [Bundle Anatomy](#bundle-anatomy) |
| `BundleInputForCard` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `bundle JSON envelope` | [Bundle Anatomy](#bundle-anatomy) |
| `BundleJson` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `bundle mode` | [Bundle Anatomy](#bundle-anatomy) |
| `BundleMode` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `Bundle (toolkit)` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `bytecode header (mk1)` | [mk1 Wire Format](#mk1-wire-format) |
| `BytecodeHeader` | [mk-codec Rust API](#mk-codec-rust-api) |
| `bytes_to_5bit` | [mk-codec Rust API](#mk-codec-rust-api) |
| `canonicality rules` | [md1 Wire Format](#md1-wire-format) |
| `canonical_origin` | [md-codec Rust API](#md-codec-rust-api) |
| `cascade-skip` | [Anti-Collision Invariants](#anti-collision-invariants) |
| `case_check` | [mk-codec Rust API](#mk-codec-rust-api) |
| `CaseStatus` | [mk-codec Rust API](#mk-codec-rust-api) |
| `check_key_vector_distinctness` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `check_no_concurrent_stdin` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `CHECKSUM_LEN_SHORT` | [ms-codec Rust API](#ms-codec-rust-api) |
| `chunk_5char` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `CHUNKED_FRAGMENT_LONG_BYTES` | [mk-codec Rust API](#mk-codec-rust-api) |
| `CHUNKED_FRAGMENT_REGULAR_BYTES` | [mk-codec Rust API](#mk-codec-rust-api) |
| `chunked header (md1)` | [md1 Wire Format](#md1-wire-format) |
| `CHUNKED_HEADER_SYMBOLS` | [mk-codec Rust API](#mk-codec-rust-api) |
| `ChunkFragment` | [mk-codec Rust API](#mk-codec-rust-api) |
| `ChunkHeader` | [md-codec Rust API](#md-codec-rust-api) |
| `chunk_md1` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `chunk_set_id binding` | [Anti-Collision Invariants](#anti-collision-invariants) |
| `chunk_set_id` | [Bundle Anatomy](#bundle-anatomy) |
| `chunk_set_id (md1)` | [md1 Wire Format](#md1-wire-format) |
| `chunk_set_id (mk1)` | [mk1 Wire Format](#mk1-wire-format) |
| `CKDpub` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `CliLanguage` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `CliNetwork` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `CliTemplate::bip48_script_type` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `CliTemplate::derivation_path` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `CliTemplate::human_name` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `CliTemplate::is_multisig` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `CliTemplate::md_origin_path` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `CliTemplate::origin_path_str` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `CliTemplate::wrapper_node` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `codex32` | [codex32 and BCH](#codex32-and-bch) |
| `compact-73` | [mk1 Wire Format](#mk1-wire-format) |
| `compute_md1_encoding_id` | [md-codec Rust API](#md-codec-rust-api) |
| `compute_wallet_descriptor_template_id` | [md-codec Rust API](#md-codec-rust-api) |
| `compute_wallet_policy_id` | [md-codec Rust API](#md-codec-rust-api) |
| `ContextKind` | [md-codec Rust API](#md-codec-rust-api) |
| `CorrectionResult` | [mk-codec Rust API](#mk-codec-rust-api) |
| `CosignerEntry` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `cosigner` | [Introduction](#introduction) |
| `CosignerKeyInfo` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `cosigner-mapping diagnostic` | [Anti-Collision Invariants](#anti-collision-invariants) |
| `CosignerSpec` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `cross-card binding (bundle)` | [Anti-Collision Invariants](#anti-collision-invariants) |
| `cross-card binding` | [The m-format Star](#the-m-format-star) |
| `CROSS_CHUNK_HASH_BYTES` | [mk-codec Rust API](#mk-codec-rust-api) |
| `cross_chunk_hash` | [mk1 Wire Format](#mk1-wire-format) |
| `decode_bytecode` | [mk-codec Rust API](#mk-codec-rust-api) |
| `DecodedString` | [mk-codec Rust API](#mk-codec-rust-api) |
| `decode_md1_string` | [md-codec Rust API](#md-codec-rust-api) |
| `decode (mk-codec)` | [mk-codec Rust API](#mk-codec-rust-api) |
| `decode (ms-codec)` | [ms-codec Rust API](#ms-codec-rust-api) |
| `decode_path` | [mk-codec Rust API](#mk-codec-rust-api) |
| `decode_payload` | [md-codec Rust API](#md-codec-rust-api) |
| `decode_string` | [mk-codec Rust API](#mk-codec-rust-api) |
| `decode_xpub_compact` | [mk-codec Rust API](#mk-codec-rust-api) |
| `derivation (md1)` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `derive (Cargo feature)` | [md-codec Rust API](#md-codec-rust-api) |
| `DerivedAccount` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `derive_full` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `derive (module)` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `DescriptorBinding` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `Descriptor::derive_address` | [md-codec Rust API](#md-codec-rust-api) |
| `Descriptor::is_wallet_policy` | [md-codec Rust API](#md-codec-rust-api) |
| `Descriptor::key_index_width` | [md-codec Rust API](#md-codec-rust-api) |
| `Descriptor (md-codec)` | [md-codec Rust API](#md-codec-rust-api) |
| `DescriptorMode` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `DescriptorPublicKey` | [Shape Coverage](#shape-coverage) |
| `descriptor truncation` | [Bundle Anatomy](#bundle-anatomy) |
| `detect_bundle_mode` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `distinct-key rule` | [Anti-Collision Invariants](#anti-collision-invariants) |
| `divergent_paths` | [md1 Wire Format](#md1-wire-format) |
| `encode_5bit_to_string` | [mk-codec Rust API](#mk-codec-rust-api) |
| `encode_bytecode` | [mk-codec Rust API](#mk-codec-rust-api) |
| `encode_md1_string` | [md-codec Rust API](#md-codec-rust-api) |
| `encode (mk-codec)` | [mk-codec Rust API](#mk-codec-rust-api) |
| `encode (ms-codec)` | [ms-codec Rust API](#ms-codec-rust-api) |
| `encode_path` | [mk-codec Rust API](#mk-codec-rust-api) |
| `encode_payload` | [md-codec Rust API](#md-codec-rust-api) |
| `encode_with_chunk_set_id` | [mk-codec Rust API](#mk-codec-rust-api) |
| `encode_xpub_compact` | [mk-codec Rust API](#mk-codec-rust-api) |
| `engraving card` | [Bundle Anatomy](#bundle-anatomy) |
| `engraving_card_unified` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `Error::AddressDerivationFailed` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `Error::AltCountOutOfRange` | [md-codec Rust API](#md-codec-rust-api) |
| `Error::BchUncorrectable` | [mk-codec Rust API](#mk-codec-rust-api) |
| `Error::BitStreamTruncated` | [md-codec Rust API](#md-codec-rust-api) |
| `Error::CardPayloadTooLarge` | [mk-codec Rust API](#mk-codec-rust-api) |
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
| `Error::ChunkSetIdMismatch (mk-codec)` | [mk-codec Rust API](#mk-codec-rust-api) |
| `Error::ChunkSetIdOutOfRange` | [md-codec Rust API](#md-codec-rust-api) |
| `Error::ChunkSetIncomplete` | [md-codec Rust API](#md-codec-rust-api) |
| `Error::ChunkSetInconsistent` | [md-codec Rust API](#md-codec-rust-api) |
| `Error::Codex32DecodeError` | [md-codec Rust API](#md-codec-rust-api) |
| `Error::Codex32EncodeError` | [md-codec Rust API](#md-codec-rust-api) |
| `Error::Codex32` | [ms-codec Rust API](#ms-codec-rust-api) |
| `Error::CrossChunkHashMismatch` | [mk1 Wire Format](#mk1-wire-format) |
| `Error::DecodeRecursionDepthExceeded` | [md-codec Rust API](#md-codec-rust-api) |
| `Error::DivergentPathCountMismatch` | [md-codec Rust API](#md-codec-rust-api) |
| `Error::EmptyTlvEntry` | [md-codec Rust API](#md-codec-rust-api) |
| `Error::ForbiddenTapTreeLeaf` | [md1 Wire Format](#md1-wire-format) |
| `Error::HardenedPublicDerivation` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `Error::InvalidChar` | [mk-codec Rust API](#mk-codec-rust-api) |
| `Error::InvalidHrp` | [mk-codec Rust API](#mk-codec-rust-api) |
| `Error::InvalidPathComponent` | [mk-codec Rust API](#mk-codec-rust-api) |
| `Error::InvalidPathIndicator` | [mk1 Wire Format](#mk1-wire-format) |
| `Error::InvalidPolicyIdStubCount` | [mk-codec Rust API](#mk-codec-rust-api) |
| `Error::InvalidPresenceByte` | [md-codec Rust API](#md-codec-rust-api) |
| `Error::InvalidStringLength` | [mk-codec Rust API](#mk-codec-rust-api) |
| `Error::InvalidXpubBytes` | [md-codec Rust API](#md-codec-rust-api) |
| `Error::InvalidXpubPublicKey` | [mk-codec Rust API](#mk-codec-rust-api) |
| `Error::InvalidXpubVersion` | [mk1 Wire Format](#mk1-wire-format) |
| `Error::KeyCountOutOfRange` | [md-codec Rust API](#md-codec-rust-api) |
| `Error::KGreaterThanN` | [md-codec Rust API](#md-codec-rust-api) |
| `Error::MalformedHeader` | [md-codec Rust API](#md-codec-rust-api) |
| `Error::MalformedPayloadPadding` | [mk-codec Rust API](#mk-codec-rust-api) |
| `Error (md-codec)` | [md-codec Rust API](#md-codec-rust-api) |
| `Error::MissingExplicitOrigin` | [md-codec Rust API](#md-codec-rust-api) |
| `Error::MissingPubkey` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `Error::MixedCase` | [mk-codec Rust API](#mk-codec-rust-api) |
| `Error::MixedHeaderTypes` | [mk-codec Rust API](#mk-codec-rust-api) |
| `Error (mk-codec)` | [mk-codec Rust API](#mk-codec-rust-api) |
| `error (module)` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `Error (ms-codec)` | [ms-codec Rust API](#ms-codec-rust-api) |
| `Error::MultipathAltCountMismatch` | [md-codec Rust API](#md-codec-rust-api) |
| `Error::NUMSSentinelConflict` | [md1 Wire Format](#md1-wire-format) |
| `Error::OperatorContextViolation` | [md1 Wire Format](#md1-wire-format) |
| `Error::OverrideOrderViolation` | [md-codec Rust API](#md-codec-rust-api) |
| `Error::PathDepthExceeded` | [md-codec Rust API](#md-codec-rust-api) |
| `Error::PathTooDeep` | [mk-codec Rust API](#mk-codec-rust-api) |
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
| `Error::TrailingBytes` | [mk-codec Rust API](#mk-codec-rust-api) |
| `Error::UnexpectedEnd (mk-codec)` | [mk-codec Rust API](#mk-codec-rust-api) |
| `Error::UnexpectedStringLength` | [ms1 Wire Format](#ms1-wire-format) |
| `Error::UnknownTag` | [ms1 Wire Format](#ms1-wire-format) |
| `Error::UnsupportedCardType` | [mk1 Wire Format](#mk1-wire-format) |
| `Error::UnsupportedVersion` | [mk1 Wire Format](#mk1-wire-format) |
| `Error::UnsupportedVersion (mk-codec)` | [mk-codec Rust API](#mk-codec-rust-api) |
| `Error::VarintOverflow` | [md-codec Rust API](#md-codec-rust-api) |
| `Error::WireVersionMismatch` | [md1 Wire Format](#md1-wire-format) |
| `Error::WrongHrp` | [ms-codec Rust API](#ms-codec-rust-api) |
| `ExpandedKey` | [md-codec Rust API](#md-codec-rust-api) |
| `expand_per_at_n` | [md-codec Rust API](#md-codec-rust-api) |
| `EXPLICIT_PATH_INDICATOR` | [mk-codec Rust API](#mk-codec-rust-api) |
| `fingerprint_flag` | [mk1 Wire Format](#mk1-wire-format) |
| `Fingerprints TLV` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `five_bit_to_bytes` | [mk-codec Rust API](#mk-codec-rust-api) |
| `forked-BCH boundary` | [The m-format Star](#the-m-format-star) |
| `format (module)` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `GENERATOR_FAMILY` | [mk-codec Rust API](#mk-codec-rust-api) |
| `generator polynomial` | [codex32 and BCH](#codex32-and-bch) |
| `GEN_LONG` | [mk-codec Rust API](#mk-codec-rust-api) |
| `GEN_REGULAR` | [mk-codec Rust API](#mk-codec-rust-api) |
| `gen-vectors (Cargo feature)` | [mk-codec Rust API](#mk-codec-rust-api) |
| `GF(32)` | [codex32 and BCH](#codex32-and-bch) |
| `GF(32) interpolation` | [Future Shares](#future-shares) |
| `hardened apostrophe folding` | [Anti-Collision Invariants](#anti-collision-invariants) |
| `Header (md-codec)` | [md-codec Rust API](#md-codec-rust-api) |
| `Header::WF_REDESIGN_VERSION` | [md-codec Rust API](#md-codec-rust-api) |
| `h-notation` | [Anti-Collision Invariants](#anti-collision-invariants) |
| `HRP` | [Conventions and Notation](#conventions-and-notation) |
| `hrp_expand` | [mk-codec Rust API](#mk-codec-rust-api) |
| `HRP-mixing` | [codex32 and BCH](#codex32-and-bch) |
| `HRP (mk1)` | [mk-codec Rust API](#mk-codec-rust-api) |
| `HRP (ms1)` | [ms-codec Rust API](#ms-codec-rust-api) |
| `inspect` | [ms-codec Rust API](#ms-codec-rust-api) |
| `InspectReport` | [ms-codec Rust API](#ms-codec-rust-api) |
| `interpolate_at (rust-codex32)` | [Future Shares](#future-shares) |
| `is_nums` | [md1 Wire Format](#md1-wire-format) |
| `KeyCard` | [mk1 Wire Format](#mk1-wire-format) |
| `KeyCard::new` | [mk-codec Rust API](#mk-codec-rust-api) |
| `key_index` | [md1 Wire Format](#md1-wire-format) |
| `kiw` | [Conventions and Notation](#conventions-and-notation) |
| `LEB128` | [mk1 Wire Format](#mk1-wire-format) |
| `Legacy (script context)` | [Shape Coverage](#shape-coverage) |
| `lex_placeholders` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `long code` | [codex32 and BCH](#codex32-and-bch) |
| `LONG_MASK` | [mk-codec Rust API](#mk-codec-rust-api) |
| `LONG_SHIFT` | [mk-codec Rust API](#mk-codec-rust-api) |
| `lookup_indicator` | [mk-codec Rust API](#mk-codec-rust-api) |
| `lookup_path` | [mk-codec Rust API](#mk-codec-rust-api) |
| `LP4-ext varint` | [md1 Wire Format](#md1-wire-format) |
| `mainnet` | [mk1 Wire Format](#mk1-wire-format) |
| `MAX_ALT_COUNT` | [md-codec Rust API](#md-codec-rust-api) |
| `MAX_CHUNKABLE_BYTECODE` | [mk-codec Rust API](#mk-codec-rust-api) |
| `MAX_CHUNK_SET_ID` | [mk-codec Rust API](#mk-codec-rust-api) |
| `MAX_CHUNKS` | [mk-codec Rust API](#mk-codec-rust-api) |
| `MAX_DECODE_DEPTH` | [md-codec Rust API](#md-codec-rust-api) |
| `MAX_PATH_COMPONENTS` | [md-codec Rust API](#md-codec-rust-api) |
| `MAX_PATH_COMPONENTS (mk-codec)` | [mk-codec Rust API](#mk-codec-rust-api) |
| `Md1EncodingId` | [md1 Wire Format](#md1-wire-format) |
| `md1` | [Introduction](#introduction) |
| `md1_xpub_match` | [Anti-Collision Invariants](#anti-collision-invariants) |
| `md_codec::bitstream` | [md-codec Rust API](#md-codec-rust-api) |
| `md_codec::canonicalize` | [md-codec Rust API](#md-codec-rust-api) |
| `md_codec::canonical_origin` | [md-codec Rust API](#md-codec-rust-api) |
| `md_codec::chunk` | [md-codec Rust API](#md-codec-rust-api) |
| `md_codec::codex32` | [md-codec Rust API](#md-codec-rust-api) |
| `md_codec (crate)` | [md-codec Rust API](#md-codec-rust-api) |
| `md_codec::decode` | [md-codec Rust API](#md-codec-rust-api) |
| `md_codec::derive` | [md-codec Rust API](#md-codec-rust-api) |
| `md_codec::encode` | [md-codec Rust API](#md-codec-rust-api) |
| `md_codec::error` | [md-codec Rust API](#md-codec-rust-api) |
| `md_codec::header` | [md-codec Rust API](#md-codec-rust-api) |
| `md_codec::identity` | [md-codec Rust API](#md-codec-rust-api) |
| `md-codec` | [md-codec Rust API](#md-codec-rust-api) |
| `md_codec::origin_path` | [md-codec Rust API](#md-codec-rust-api) |
| `md_codec::phrase` | [md-codec Rust API](#md-codec-rust-api) |
| `md_codec::tag` | [md-codec Rust API](#md-codec-rust-api) |
| `md_codec::tlv` | [md-codec Rust API](#md-codec-rust-api) |
| `md_codec::to_miniscript` | [md-codec Rust API](#md-codec-rust-api) |
| `md_codec::tree` | [md-codec Rust API](#md-codec-rust-api) |
| `md_codec::use_site_path` | [md-codec Rust API](#md-codec-rust-api) |
| `md-codec v0.32.0` | [md-codec Rust API](#md-codec-rust-api) |
| `md_codec::validate` | [md-codec Rust API](#md-codec-rust-api) |
| `md_codec::varint` | [md-codec Rust API](#md-codec-rust-api) |
| `MD_REGULAR_CONST` | [codex32 and BCH](#codex32-and-bch) |
| `m-format constellation` | [Introduction](#introduction) |
| `MIN_ALT_COUNT` | [md-codec Rust API](#md-codec-rust-api) |
| `mk1 chunked-card grouping` | [Future Shares](#future-shares) |
| `mk1` | [mk1 Wire Format](#mk1-wire-format) |
| `mk_codec::bytecode::decode` | [mk-codec Rust API](#mk-codec-rust-api) |
| `mk_codec::bytecode::encode` | [mk-codec Rust API](#mk-codec-rust-api) |
| `mk_codec::bytecode::header` | [mk-codec Rust API](#mk-codec-rust-api) |
| `mk_codec::bytecode` | [mk-codec Rust API](#mk-codec-rust-api) |
| `mk_codec::bytecode::path` | [mk-codec Rust API](#mk-codec-rust-api) |
| `mk_codec::bytecode::xpub_compact` | [mk-codec Rust API](#mk-codec-rust-api) |
| `mk_codec::consts` | [mk-codec Rust API](#mk-codec-rust-api) |
| `mk_codec (crate)` | [mk-codec Rust API](#mk-codec-rust-api) |
| `mk_codec::error` | [mk-codec Rust API](#mk-codec-rust-api) |
| `mk_codec::key_card` | [mk-codec Rust API](#mk-codec-rust-api) |
| `mk-codec` | [mk-codec Rust API](#mk-codec-rust-api) |
| `mk_codec::string_layer::bch` | [mk-codec Rust API](#mk-codec-rust-api) |
| `mk_codec::string_layer::chunk` | [mk-codec Rust API](#mk-codec-rust-api) |
| `mk_codec::string_layer::header` | [mk-codec Rust API](#mk-codec-rust-api) |
| `mk_codec::string_layer` | [mk-codec Rust API](#mk-codec-rust-api) |
| `mk-codec v0.2.2` | [mk-codec Rust API](#mk-codec-rust-api) |
| `MkField` | [Bundle Anatomy](#bundle-anatomy) |
| `MK_LONG_CONST` | [mk1 Wire Format](#mk1-wire-format) |
| `MK_REGULAR_CONST` | [mk1 Wire Format](#mk1-wire-format) |
| `mnemonic_toolkit (crate)` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `mnemonic-toolkit` | [Introduction](#introduction) |
| `mnemonic-toolkit v0.8.0` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `ms1 dense layout` | [Bundle Anatomy](#bundle-anatomy) |
| `ms1 four-case table` | [Anti-Collision Invariants](#anti-collision-invariants) |
| `ms1` | [ms1 Wire Format](#ms1-wire-format) |
| `ms_codec::consts` | [ms-codec Rust API](#ms-codec-rust-api) |
| `ms_codec (crate)` | [ms-codec Rust API](#ms-codec-rust-api) |
| `ms_codec::decode` | [ms-codec Rust API](#ms-codec-rust-api) |
| `ms_codec::encode` | [ms-codec Rust API](#ms-codec-rust-api) |
| `ms_codec::error` | [ms-codec Rust API](#ms-codec-rust-api) |
| `ms_codec::inspect` | [ms-codec Rust API](#ms-codec-rust-api) |
| `ms-codec` | [ms-codec Rust API](#ms-codec-rust-api) |
| `ms_codec::payload` | [ms-codec Rust API](#ms-codec-rust-api) |
| `ms_codec::tag` | [ms-codec Rust API](#ms-codec-rust-api) |
| `ms-codec v0.1.1` | [ms-codec Rust API](#ms-codec-rust-api) |
| `MsField` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `multi-family bodies` | [md1 Wire Format](#md1-wire-format) |
| `multipath alternative` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `multipath` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `multiplicity (multiset)` | [Anti-Collision Invariants](#anti-collision-invariants) |
| `multiset semantics` | [Anti-Collision Invariants](#anti-collision-invariants) |
| `MultisigHybrid` | [Bundle Anatomy](#bundle-anatomy) |
| `MultisigInfo` | [Bundle Anatomy](#bundle-anatomy) |
| `MultisigMultiSource` | [Bundle Anatomy](#bundle-anatomy) |
| `MultisigPathFamily` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `MultisigWatchOnly` | [Bundle Anatomy](#bundle-anatomy) |
| `@N` | [Conventions and Notation](#conventions-and-notation) |
| `Node (md-codec)` | [md-codec Rust API](#md-codec-rust-api) |
| `node_to_descriptor` | [Shape Coverage](#shape-coverage) |
| `node_to_miniscript` | [Shape Coverage](#shape-coverage) |
| `NUMS_DOMAIN` | [mk-codec Rust API](#mk-codec-rust-api) |
| `NUMS H-point` | [Shape Coverage](#shape-coverage) |
| `NUMS` | [md1 Wire Format](#md1-wire-format) |
| `ORIGIN_FINGERPRINT_BYTES` | [mk-codec Rust API](#mk-codec-rust-api) |
| `origin path` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `OriginPath` | [md1 Wire Format](#md1-wire-format) |
| `OriginPathOverrides TLV` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `parse_cosigners_file` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `parse_cosigner_spec` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `parse_descriptor (function)` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `parse_descriptor (module)` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `ParsedFingerprint` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `ParsedKey` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `ParseError (slot)` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `parse_master_fingerprint` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `parse (module)` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `parse_slot_input` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `parse_xpub_prefix_arg` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `PathComponent` | [md-codec Rust API](#md-codec-rust-api) |
| `PathDecl` | [md-codec Rust API](#md-codec-rust-api) |
| `PathDeclPaths` | [md-codec Rust API](#md-codec-rust-api) |
| `Payload::as_bytes` | [ms-codec Rust API](#ms-codec-rust-api) |
| `Payload::Entr` | [ms1 Wire Format](#ms1-wire-format) |
| `Payload::kind` | [ms-codec Rust API](#ms-codec-rust-api) |
| `PayloadKind` | [ms-codec Rust API](#ms-codec-rust-api) |
| `Payload (ms-codec)` | [ms-codec Rust API](#ms-codec-rust-api) |
| `Payload::validate` | [ms-codec Rust API](#ms-codec-rust-api) |
| `PBKDF2-HMAC-SHA512` | [ms1 Wire Format](#ms1-wire-format) |
| `Phrase::from_id_bytes` | [md-codec Rust API](#md-codec-rust-api) |
| `Phrase` | [md-codec Rust API](#md-codec-rust-api) |
| `pkh` | [Shape Coverage](#shape-coverage) |
| `placeholder (@N)` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `PlaceholderOccurrence` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `POLICY_ID_STUB_BYTES` | [mk-codec Rust API](#mk-codec-rust-api) |
| `policy_id_stub` | [mk1 Wire Format](#mk1-wire-format) |
| `polymod` | [codex32 and BCH](#codex32-and-bch) |
| `POLYMOD_INIT` | [mk-codec Rust API](#mk-codec-rust-api) |
| `pre_check_template_n` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `pre_check_threshold` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `privacy-preserving mode` | [mk1 Wire Format](#mk1-wire-format) |
| `Pubkeys TLV` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `read_node` | [md-codec Rust API](#md-codec-rust-api) |
| `read_phrase_input` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `read_varint` | [md-codec Rust API](#md-codec-rust-api) |
| `reassemble_from_chunks` | [mk-codec Rust API](#mk-codec-rust-api) |
| `reconstruct_xpub` | [mk-codec Rust API](#mk-codec-rust-api) |
| `re_emit_bits` | [md-codec Rust API](#md-codec-rust-api) |
| `regular code` | [codex32 and BCH](#codex32-and-bch) |
| `REGULAR_MASK` | [mk-codec Rust API](#mk-codec-rust-api) |
| `REGULAR_SHIFT` | [mk-codec Rust API](#mk-codec-rust-api) |
| `render_codex32_grouped` | [md-codec Rust API](#md-codec-rust-api) |
| `RESERVED_NOT_EMITTED_V01` | [ms-codec Rust API](#ms-codec-rust-api) |
| `reserved-prefix byte (ms1)` | [ms1 Wire Format](#ms1-wire-format) |
| `reserved-prefix byte (v0.2)` | [Future Shares](#future-shares) |
| `RESERVED_PREFIX (ms1)` | [ms-codec Rust API](#ms-codec-rust-api) |
| `RESERVED_TAG_TABLE` | [ms1 Wire Format](#ms1-wire-format) |
| `ResolvedPlaceholders` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `ResolvedSlot::is_secret_bearing` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `ResolvedSlot` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `resolve_placeholders` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `Result (mk-codec)` | [mk-codec Rust API](#mk-codec-rust-api) |
| `Result (ms-codec)` | [ms-codec Rust API](#ms-codec-rust-api) |
| `rust-codex32` | [ms1 Wire Format](#ms1-wire-format) |
| `script (BIP-388)` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `ScriptCtx` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `secp256k1` | [mk1 Wire Format](#mk1-wire-format) |
| `secret-bearing slot` | [Bundle Anatomy](#bundle-anatomy) |
| `Segwitv0 (script context)` | [Shape Coverage](#shape-coverage) |
| `SEPARATOR (bech32)` | [mk-codec Rust API](#mk-codec-rust-api) |
| `SHA-256` | [md1 Wire Format](#md1-wire-format) |
| `SHARE_INDEX_V01` | [ms-codec Rust API](#ms-codec-rust-api) |
| `share-set grouping` | [Future Shares](#future-shares) |
| `sh (legacy)` | [Shape Coverage](#shape-coverage) |
| `sh(multi)` | [Shape Coverage](#shape-coverage) |
| `sh(wpkh)` | [Shape Coverage](#shape-coverage) |
| `SINGLE_HEADER_SYMBOLS` | [mk-codec Rust API](#mk-codec-rust-api) |
| `SingleSigFull` | [Bundle Anatomy](#bundle-anatomy) |
| `SingleSigWatchOnly` | [Bundle Anatomy](#bundle-anatomy) |
| `single-string header (md1)` | [md1 Wire Format](#md1-wire-format) |
| `SINGLE_STRING_LONG_BYTES` | [mk-codec Rust API](#mk-codec-rust-api) |
| `SINGLE_STRING_PAYLOAD_BIT_LIMIT` | [md-codec Rust API](#md-codec-rust-api) |
| `SINGLE_STRING_REGULAR_BYTES` | [mk-codec Rust API](#mk-codec-rust-api) |
| `SlotCardBlock` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `SlotInput` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `SlotSubkey` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `split_into_chunks` | [mk-codec Rust API](#mk-codec-rust-api) |
| `STANDARD_PATHS` | [mk-codec Rust API](#mk-codec-rust-api) |
| `standard-path table (mk1)` | [mk1 Wire Format](#mk1-wire-format) |
| `string-layer header (mk1)` | [mk1 Wire Format](#mk1-wire-format) |
| `StringLayerHeader` | [mk-codec Rust API](#mk-codec-rust-api) |
| `substitute_synthetic` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `synthesize (module)` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `synthesize_unified` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `synthetic_xpub_for` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `Tag::as_bytes` | [ms-codec Rust API](#ms-codec-rust-api) |
| `Tag::as_str` | [ms-codec Rust API](#ms-codec-rust-api) |
| `Tag::Check` | [md1 Wire Format](#md1-wire-format) |
| `Tag::ENTR` | [ms1 Wire Format](#ms1-wire-format) |
| `TAG_ENTR` | [ms-codec Rust API](#ms-codec-rust-api) |
| `Tag::from_raw_bytes` | [ms-codec Rust API](#ms-codec-rust-api) |
| `Tag (md-codec)` | [md-codec Rust API](#md-codec-rust-api) |
| `Tag (ms1)` | [ms1 Wire Format](#ms1-wire-format) |
| `Tag (ms-codec)` | [ms-codec Rust API](#ms-codec-rust-api) |
| `Tag::Multi` | [md1 Wire Format](#md1-wire-format) |
| `Tag::OriginPaths` | [md1 Wire Format](#md1-wire-format) |
| `Tag::Pkh` | [md1 Wire Format](#md1-wire-format) |
| `Tag::PkH` | [md1 Wire Format](#md1-wire-format) |
| `Tag::PkK` | [md1 Wire Format](#md1-wire-format) |
| `Tag::Sh` | [md1 Wire Format](#md1-wire-format) |
| `Tag::Thresh` | [md1 Wire Format](#md1-wire-format) |
| `Tag::Tr` | [md1 Wire Format](#md1-wire-format) |
| `Tag::try_new` | [ms-codec Rust API](#ms-codec-rust-api) |
| `Tag::Wpkh` | [md1 Wire Format](#md1-wire-format) |
| `Tag::Wsh` | [md1 Wire Format](#md1-wire-format) |
| `tap-leaf miniscript` | [Shape Coverage](#shape-coverage) |
| `taproot internal key` | [md1 Wire Format](#md1-wire-format) |
| `Tap (script context)` | [Shape Coverage](#shape-coverage) |
| `TapTree` | [Shape Coverage](#shape-coverage) |
| `target residue` | [codex32 and BCH](#codex32-and-bch) |
| `template (md1)` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `template (module)` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `TemplateOrDescriptor` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `Terminal::Multi` | [Shape Coverage](#shape-coverage) |
| `testnet` | [mk1 Wire Format](#mk1-wire-format) |
| `Threshold (ms-codec v0.2)` | [Future Shares](#future-shares) |
| `THRESHOLD_V01` | [ms-codec Rust API](#ms-codec-rust-api) |
| `TLV_FINGERPRINTS` | [md-codec Rust API](#md-codec-rust-api) |
| `TLV_ORIGIN_PATH_OVERRIDES` | [md-codec Rust API](#md-codec-rust-api) |
| `TLV_PUBKEYS` | [md-codec Rust API](#md-codec-rust-api) |
| `TLV section` | [md1 Wire Format](#md1-wire-format) |
| `TlvSection` | [md-codec Rust API](#md-codec-rust-api) |
| `TLV_USE_SITE_PATH_OVERRIDES` | [md-codec Rust API](#md-codec-rust-api) |
| `to_miniscript_descriptor` | [md-codec Rust API](#md-codec-rust-api) |
| `to_miniscript_descriptor` | [Shape Coverage](#shape-coverage) |
| `ToolkitError` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `tr (key-path)` | [Shape Coverage](#shape-coverage) |
| `tr (multi-leaf)` | [Shape Coverage](#shape-coverage) |
| `tr (NUMS)` | [Shape Coverage](#shape-coverage) |
| `tr (single-leaf)` | [Shape Coverage](#shape-coverage) |
| `Unshared Secret form` | [ms1 Wire Format](#ms1-wire-format) |
| `unwrap_string` | [md-codec Rust API](#md-codec-rust-api) |
| `use-site-path declaration` | [md1 Wire Format](#md1-wire-format) |
| `use-site path` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `UseSitePath` | [md-codec Rust API](#md-codec-rust-api) |
| `v0.1 → v0.2-shares migration` | [Future Shares](#future-shares) |
| `validate_explicit_origin_required` | [md-codec Rust API](#md-codec-rust-api) |
| `validate_multipath_consistency` | [md-codec Rust API](#md-codec-rust-api) |
| `validate_placeholder_usage` | [md-codec Rust API](#md-codec-rust-api) |
| `validate_presence_byte` | [md-codec Rust API](#md-codec-rust-api) |
| `validate_slot_set` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `validate_tap_script_tree` | [md-codec Rust API](#md-codec-rust-api) |
| `validate_xpub_bytes` | [md-codec Rust API](#md-codec-rust-api) |
| `VALID_ENTR_LENGTHS` | [ms-codec Rust API](#ms-codec-rust-api) |
| `VALID_STR_LENGTHS` | [ms-codec Rust API](#ms-codec-rust-api) |
| `verify-bundle` | [Bundle Anatomy](#bundle-anatomy) |
| `VerifyBundleJson` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `VerifyCheck` | [Bundle Anatomy](#bundle-anatomy) |
| `VERSION_V0_1` | [mk-codec Rust API](#mk-codec-rust-api) |
| `walker normalisation` | [md1 Wire Format](#md1-wire-format) |
| `walk_root` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `WalletDescriptorTemplateId` | [md-codec Rust API](#md-codec-rust-api) |
| `Wallet Instance ID` | [mk1 Wire Format](#mk1-wire-format) |
| `WalletPolicyId` | [md-codec Rust API](#md-codec-rust-api) |
| `WalletPolicyId::to_phrase` | [md-codec Rust API](#md-codec-rust-api) |
| `watch-only slot` | [Bundle Anatomy](#bundle-anatomy) |
| `wildcard (BIP-389)` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `wire format` | [md1 Wire Format](#md1-wire-format) |
| `wpkh` | [Shape Coverage](#shape-coverage) |
| `wrap_payload` | [md-codec Rust API](#md-codec-rust-api) |
| `write_node` | [md-codec Rust API](#md-codec-rust-api) |
| `write_varint` | [md-codec Rust API](#md-codec-rust-api) |
| `wsh (miniscript)` | [Shape Coverage](#shape-coverage) |
| `Xpub (BIP-32)` | [mk-codec Rust API](#mk-codec-rust-api) |
| `XPUB_COMPACT_BYTES` | [mk-codec Rust API](#mk-codec-rust-api) |
| `XpubCompact::from_xpub` | [mk-codec Rust API](#mk-codec-rust-api) |
| `XpubCompact` | [mk-codec Rust API](#mk-codec-rust-api) |
| `XpubNotInPolicy` | [Anti-Collision Invariants](#anti-collision-invariants) |
| `XpubPrefix` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
| `xpub` | [Shape Coverage](#shape-coverage) |
| `xpub_to_65` | [mnemonic-toolkit Rust API](#mnemonic-toolkit-rust-api) |
