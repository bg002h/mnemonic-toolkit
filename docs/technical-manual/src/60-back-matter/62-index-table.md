# Index

This table mirrors the page-numbered alphabetical index emitted by `makeindex` in the PDF render path. Every `\index{TERM}` marker placed in `src/**/*.md` MUST have a matching row below; the `tests/lint.sh` bidirectional check enforces this.

The rows are sorted alphabetically (case-insensitive). Add new rows as you add new `\index{}` markers.

| Term | Section |
|---|---|
| `@N` | [Conventions and Notation](#conventions-and-notation) |
| `abandon test mnemonic` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `address derivation` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `auto-dispatch` | [md1 Wire Format](#md1-wire-format) |
| `BCH code` | [codex32 and BCH](#codex32-and-bch) |
| `bech32` | [codex32 and BCH](#codex32-and-bch) |
| `BIP 32 master fingerprint` | [mk1 Wire Format](#mk1-wire-format) |
| `BIP-32` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `BIP-44` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `BIP-48` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `BIP-49` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `BIP-84` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `BIP-86` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `BIP-173` | [codex32 and BCH](#codex32-and-bch) |
| `BIP-341` | [md1 Wire Format](#md1-wire-format) |
| `BIP-380` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `BIP-388` | [Introduction](#introduction) |
| `BIP-389` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `BIP-389 multipath` | [Conventions and Notation](#conventions-and-notation) |
| `BIP-39 entropy` | [ms1 Wire Format](#ms1-wire-format) |
| `BIP-39 mnemonic` | [ms1 Wire Format](#ms1-wire-format) |
| `BIP-39 wordlist` | [ms1 Wire Format](#ms1-wire-format) |
| `BIP-93` | [codex32 and BCH](#codex32-and-bch) |
| `BIP-93 design distance` | [codex32 and BCH](#codex32-and-bch) |
| `Body::KeyArg` | [md1 Wire Format](#md1-wire-format) |
| `Body::MultiKeys` | [md1 Wire Format](#md1-wire-format) |
| `Body::Variable` | [md1 Wire Format](#md1-wire-format) |
| `bytecode header (mk1)` | [mk1 Wire Format](#mk1-wire-format) |
| `canonicality rules` | [md1 Wire Format](#md1-wire-format) |
| `chunked header (md1)` | [md1 Wire Format](#md1-wire-format) |
| `chunk_set_id (md1)` | [md1 Wire Format](#md1-wire-format) |
| `chunk_set_id (mk1)` | [mk1 Wire Format](#mk1-wire-format) |
| `CKDpub` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `codex32` | [codex32 and BCH](#codex32-and-bch) |
| `compact-73` | [mk1 Wire Format](#mk1-wire-format) |
| `cosigner` | [Introduction](#introduction) |
| `cross-card binding` | [The m-format Star](#the-m-format-star) |
| `cross_chunk_hash` | [mk1 Wire Format](#mk1-wire-format) |
| `derivation (md1)` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `divergent_paths` | [md1 Wire Format](#md1-wire-format) |
| `Error::AddressDerivationFailed` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `Error::ChainIndexOutOfRange` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `Error::ChunkedHeaderMalformed` | [mk1 Wire Format](#mk1-wire-format) |
| `Error::ChunkSetIdMismatch` | [md1 Wire Format](#md1-wire-format) |
| `Error::CrossChunkHashMismatch` | [mk1 Wire Format](#mk1-wire-format) |
| `Error::ForbiddenTapTreeLeaf` | [md1 Wire Format](#md1-wire-format) |
| `Error::HardenedPublicDerivation` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `Error::InvalidPathIndicator` | [mk1 Wire Format](#mk1-wire-format) |
| `Error::InvalidXpubVersion` | [mk1 Wire Format](#mk1-wire-format) |
| `Error::MissingPubkey` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `Error::NUMSSentinelConflict` | [md1 Wire Format](#md1-wire-format) |
| `Error::OperatorContextViolation` | [md1 Wire Format](#md1-wire-format) |
| `Error::PayloadLengthMismatch` | [ms1 Wire Format](#ms1-wire-format) |
| `Error::ReservedBitsSet` | [mk1 Wire Format](#mk1-wire-format) |
| `Error::ReservedPrefixViolation` | [ms1 Wire Format](#ms1-wire-format) |
| `Error::ReservedTagNotEmittedInV01` | [ms1 Wire Format](#ms1-wire-format) |
| `Error::ShareIndexNotSecret` | [ms1 Wire Format](#ms1-wire-format) |
| `Error::TagInvalidAlphabet` | [ms1 Wire Format](#ms1-wire-format) |
| `Error::ThresholdNotZero` | [ms1 Wire Format](#ms1-wire-format) |
| `Error::UnexpectedStringLength` | [ms1 Wire Format](#ms1-wire-format) |
| `Error::UnknownTag` | [ms1 Wire Format](#ms1-wire-format) |
| `Error::UnsupportedCardType` | [mk1 Wire Format](#mk1-wire-format) |
| `Error::UnsupportedVersion` | [mk1 Wire Format](#mk1-wire-format) |
| `Error::WireVersionMismatch` | [md1 Wire Format](#md1-wire-format) |
| `Fingerprints TLV` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `fingerprint_flag` | [mk1 Wire Format](#mk1-wire-format) |
| `forked-BCH boundary` | [The m-format Star](#the-m-format-star) |
| `generator polynomial` | [codex32 and BCH](#codex32-and-bch) |
| `GF(32)` | [codex32 and BCH](#codex32-and-bch) |
| `HRP` | [Conventions and Notation](#conventions-and-notation) |
| `HRP-mixing` | [codex32 and BCH](#codex32-and-bch) |
| `is_nums` | [md1 Wire Format](#md1-wire-format) |
| `KeyCard` | [mk1 Wire Format](#mk1-wire-format) |
| `key_index` | [md1 Wire Format](#md1-wire-format) |
| `kiw` | [Conventions and Notation](#conventions-and-notation) |
| `LEB128` | [mk1 Wire Format](#mk1-wire-format) |
| `long code` | [codex32 and BCH](#codex32-and-bch) |
| `LP4-ext varint` | [md1 Wire Format](#md1-wire-format) |
| `mainnet` | [mk1 Wire Format](#mk1-wire-format) |
| `md1` | [Introduction](#introduction) |
| `Md1EncodingId` | [md1 Wire Format](#md1-wire-format) |
| `MD_REGULAR_CONST` | [codex32 and BCH](#codex32-and-bch) |
| `m-format constellation` | [Introduction](#introduction) |
| `mk1` | [mk1 Wire Format](#mk1-wire-format) |
| `MK_LONG_CONST` | [mk1 Wire Format](#mk1-wire-format) |
| `MK_REGULAR_CONST` | [mk1 Wire Format](#mk1-wire-format) |
| `mnemonic-toolkit` | [Introduction](#introduction) |
| `ms1` | [ms1 Wire Format](#ms1-wire-format) |
| `multi-family bodies` | [md1 Wire Format](#md1-wire-format) |
| `multipath` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `multipath alternative` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `NUMS` | [md1 Wire Format](#md1-wire-format) |
| `origin path` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `OriginPath` | [md1 Wire Format](#md1-wire-format) |
| `OriginPathOverrides TLV` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `Payload::Entr` | [ms1 Wire Format](#ms1-wire-format) |
| `PBKDF2-HMAC-SHA512` | [ms1 Wire Format](#ms1-wire-format) |
| `placeholder (@N)` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `policy_id_stub` | [mk1 Wire Format](#mk1-wire-format) |
| `polymod` | [codex32 and BCH](#codex32-and-bch) |
| `privacy-preserving mode` | [mk1 Wire Format](#mk1-wire-format) |
| `Pubkeys TLV` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `regular code` | [codex32 and BCH](#codex32-and-bch) |
| `reserved-prefix byte (ms1)` | [ms1 Wire Format](#ms1-wire-format) |
| `RESERVED_TAG_TABLE` | [ms1 Wire Format](#ms1-wire-format) |
| `rust-codex32` | [ms1 Wire Format](#ms1-wire-format) |
| `script (BIP-388)` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `secp256k1` | [mk1 Wire Format](#mk1-wire-format) |
| `SHA-256` | [md1 Wire Format](#md1-wire-format) |
| `single-string header (md1)` | [md1 Wire Format](#md1-wire-format) |
| `standard-path table (mk1)` | [mk1 Wire Format](#mk1-wire-format) |
| `string-layer header (mk1)` | [mk1 Wire Format](#mk1-wire-format) |
| `Tag::Check` | [md1 Wire Format](#md1-wire-format) |
| `Tag::ENTR` | [ms1 Wire Format](#ms1-wire-format) |
| `Tag (ms1)` | [ms1 Wire Format](#ms1-wire-format) |
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
| `taproot internal key` | [md1 Wire Format](#md1-wire-format) |
| `target residue` | [codex32 and BCH](#codex32-and-bch) |
| `template (md1)` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `testnet` | [mk1 Wire Format](#mk1-wire-format) |
| `TLV section` | [md1 Wire Format](#md1-wire-format) |
| `Unshared Secret form` | [ms1 Wire Format](#ms1-wire-format) |
| `use-site path` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `use-site-path declaration` | [md1 Wire Format](#md1-wire-format) |
| `walker normalisation` | [md1 Wire Format](#md1-wire-format) |
| `Wallet Instance ID` | [mk1 Wire Format](#mk1-wire-format) |
| `wildcard (BIP-389)` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `wire format` | [md1 Wire Format](#md1-wire-format) |
