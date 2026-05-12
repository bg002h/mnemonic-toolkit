# Index

This table mirrors the page-numbered alphabetical index emitted by `makeindex` in the PDF render path. Every `\index{TERM}` marker placed in `src/**/*.md` MUST have a matching row below; the `tests/lint.sh` bidirectional check enforces this.

The rows are sorted alphabetically (case-insensitive). Add new rows as you add new `\index{}` markers.

| Term | Section |
|---|---|
| `@N` | [Conventions and Notation](#conventions-and-notation) |
| `abandon test mnemonic` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address), [Bundle Anatomy](#bundle-anatomy) |
| `address derivation` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `auto-dispatch` | [md1 Wire Format](#md1-wire-format) |
| `base58check` | [Network and Addressing](#network-and-addressing) |
| `BCH code` | [codex32 and BCH](#codex32-and-bch) |
| `bech32` | [codex32 and BCH](#codex32-and-bch) |
| `bifurcation (BIP-388 enforcement)` | [Anti-Collision Invariants](#anti-collision-invariants) |
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
| `BIP-388 distinct-key` | [Anti-Collision Invariants](#anti-collision-invariants) |
| `BIP-389` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address), [Bundle Anatomy](#bundle-anatomy) |
| `BIP-389 multipath` | [Conventions and Notation](#conventions-and-notation) |
| `BIP-39 entropy` | [ms1 Wire Format](#ms1-wire-format) |
| `BIP-39 mnemonic` | [ms1 Wire Format](#ms1-wire-format) |
| `BIP-39 wordlist` | [ms1 Wire Format](#ms1-wire-format) |
| `BIP-93` | [codex32 and BCH](#codex32-and-bch) |
| `BIP-93 design distance` | [codex32 and BCH](#codex32-and-bch) |
| `Body::KeyArg` | [md1 Wire Format](#md1-wire-format) |
| `Body::MultiKeys` | [md1 Wire Format](#md1-wire-format) |
| `Body::Variable` | [md1 Wire Format](#md1-wire-format) |
| `bundle` | [Bundle Anatomy](#bundle-anatomy) |
| `bundle envelope` | [Bundle Anatomy](#bundle-anatomy) |
| `bundle JSON envelope` | [Bundle Anatomy](#bundle-anatomy) |
| `bundle mode` | [Bundle Anatomy](#bundle-anatomy) |
| `bytecode header (mk1)` | [mk1 Wire Format](#mk1-wire-format) |
| `canonicality rules` | [md1 Wire Format](#md1-wire-format) |
| `cascade-skip` | [Anti-Collision Invariants](#anti-collision-invariants) |
| `chunk_set_id` | [Bundle Anatomy](#bundle-anatomy) |
| `chunk_set_id binding` | [Anti-Collision Invariants](#anti-collision-invariants) |
| `chunked header (md1)` | [md1 Wire Format](#md1-wire-format) |
| `chunk_set_id (md1)` | [md1 Wire Format](#md1-wire-format) |
| `chunk_set_id (mk1)` | [mk1 Wire Format](#mk1-wire-format) |
| `CKDpub` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `codex32` | [codex32 and BCH](#codex32-and-bch) |
| `compact-73` | [mk1 Wire Format](#mk1-wire-format) |
| `cosigner` | [Introduction](#introduction) |
| `cosigner-mapping diagnostic` | [Anti-Collision Invariants](#anti-collision-invariants) |
| `cross-card binding` | [The m-format Star](#the-m-format-star) |
| `cross-card binding (bundle)` | [Anti-Collision Invariants](#anti-collision-invariants) |
| `cross_chunk_hash` | [mk1 Wire Format](#mk1-wire-format) |
| `derivation (md1)` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `descriptor truncation` | [Bundle Anatomy](#bundle-anatomy) |
| `DescriptorPublicKey` | [Shape Coverage](#shape-coverage) |
| `distinct-key rule` | [Anti-Collision Invariants](#anti-collision-invariants) |
| `divergent_paths` | [md1 Wire Format](#md1-wire-format) |
| `engraving card` | [Bundle Anatomy](#bundle-anatomy) |
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
| `GF(32) interpolation` | [Future Shares](#future-shares) |
| `h-notation` | [Anti-Collision Invariants](#anti-collision-invariants) |
| `hardened apostrophe folding` | [Anti-Collision Invariants](#anti-collision-invariants) |
| `HRP` | [Conventions and Notation](#conventions-and-notation) |
| `HRP-mixing` | [codex32 and BCH](#codex32-and-bch) |
| `interpolate_at (rust-codex32)` | [Future Shares](#future-shares) |
| `is_nums` | [md1 Wire Format](#md1-wire-format) |
| `KeyCard` | [mk1 Wire Format](#mk1-wire-format) |
| `key_index` | [md1 Wire Format](#md1-wire-format) |
| `kiw` | [Conventions and Notation](#conventions-and-notation) |
| `LEB128` | [mk1 Wire Format](#mk1-wire-format) |
| `Legacy (script context)` | [Shape Coverage](#shape-coverage) |
| `long code` | [codex32 and BCH](#codex32-and-bch) |
| `LP4-ext varint` | [md1 Wire Format](#md1-wire-format) |
| `mainnet` | [mk1 Wire Format](#mk1-wire-format) |
| `md1` | [Introduction](#introduction) |
| `md1_xpub_match` | [Anti-Collision Invariants](#anti-collision-invariants) |
| `Md1EncodingId` | [md1 Wire Format](#md1-wire-format) |
| `MD_REGULAR_CONST` | [codex32 and BCH](#codex32-and-bch) |
| `m-format constellation` | [Introduction](#introduction) |
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
| `node_to_descriptor` | [Shape Coverage](#shape-coverage) |
| `node_to_miniscript` | [Shape Coverage](#shape-coverage) |
| `NUMS` | [md1 Wire Format](#md1-wire-format) |
| `NUMS H-point` | [Shape Coverage](#shape-coverage) |
| `origin path` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `OriginPath` | [md1 Wire Format](#md1-wire-format) |
| `OriginPathOverrides TLV` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `Payload::Entr` | [ms1 Wire Format](#ms1-wire-format) |
| `PBKDF2-HMAC-SHA512` | [ms1 Wire Format](#ms1-wire-format) |
| `pkh` | [Shape Coverage](#shape-coverage) |
| `placeholder (@N)` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `policy_id_stub` | [mk1 Wire Format](#mk1-wire-format) |
| `polymod` | [codex32 and BCH](#codex32-and-bch) |
| `privacy-preserving mode` | [mk1 Wire Format](#mk1-wire-format) |
| `Pubkeys TLV` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `regular code` | [codex32 and BCH](#codex32-and-bch) |
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
| `SingleSigFull` | [Bundle Anatomy](#bundle-anatomy) |
| `SingleSigWatchOnly` | [Bundle Anatomy](#bundle-anatomy) |
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
| `to_miniscript_descriptor` | [Shape Coverage](#shape-coverage) |
| `tr (key-path)` | [Shape Coverage](#shape-coverage) |
| `tr (multi-leaf)` | [Shape Coverage](#shape-coverage) |
| `tr (NUMS)` | [Shape Coverage](#shape-coverage) |
| `tr (single-leaf)` | [Shape Coverage](#shape-coverage) |
| `Unshared Secret form` | [ms1 Wire Format](#ms1-wire-format) |
| `use-site path` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `use-site-path declaration` | [md1 Wire Format](#md1-wire-format) |
| `v0.1 → v0.2-shares migration` | [Future Shares](#future-shares) |
| `verify-bundle` | [Bundle Anatomy](#bundle-anatomy) |
| `VerifyCheck` | [Bundle Anatomy](#bundle-anatomy) |
| `walker normalisation` | [md1 Wire Format](#md1-wire-format) |
| `Wallet Instance ID` | [mk1 Wire Format](#mk1-wire-format) |
| `watch-only slot` | [Bundle Anatomy](#bundle-anatomy) |
| `wildcard (BIP-389)` | [Descriptor to Miniscript to Address](#descriptor-to-miniscript-to-address) |
| `wire format` | [md1 Wire Format](#md1-wire-format) |
| `wpkh` | [Shape Coverage](#shape-coverage) |
| `wsh (miniscript)` | [Shape Coverage](#shape-coverage) |
| `xpub` | [Shape Coverage](#shape-coverage) |
| `XpubNotInPolicy` | [Anti-Collision Invariants](#anti-collision-invariants) |
