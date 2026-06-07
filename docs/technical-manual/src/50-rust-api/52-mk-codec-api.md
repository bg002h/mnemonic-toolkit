# mk-codec Rust API

This chapter is the reference for the `mk-codec`\index{mk-codec} crate's public surface at v0.2.2\index{mk-codec v0.2.2} (HEAD `e8782fd` in `bg002h/mnemonic-key`). It enumerates every public module, function, type, constant, and error variant. The wire format these APIs encode/decode is §II.2; this chapter is the library API only — the `mk-cli` binary lives in a sibling crate and its surface is covered in the end-user manual, not Part V. For the normative wire spec, see `mnemonic-key/design/SPEC_mk_v0_1.md` and the in-tree BIP draft at `mnemonic-key/bip/bip-mnemonic-key.mediawiki`.

## V.2.1 Crate purpose

`mk-codec`\index{mk\_codec (crate)} is the reference encoder/decoder for the mk1 xpub-card format (HRP `mk`). The crate ingests a BIP-32 `Xpub`\index{Xpub (BIP-32)} plus an origin path and one or more 4-byte policy-ID stubs, bit-packs them as canonical bytecode, then wraps the result as one or more codex32-style strings protected by HRP-mixed BCH checksums. Decode is the inverse: BCH error-correction up to `t=4` substitutions per chunk, chunk reassembly with cross-chunk SHA-256 integrity, and reconstruction of a full `bitcoin::bip32::Xpub` from the 73-byte compact form. The crate is library-only; `mk-cli` is a sibling binary crate out of Part V scope. Pre-1.0 reference status; the v0.1 wire format is locked, but the Rust API may shift on any 0.X bump.

## V.2.2 Feature flags

(from `crates/mk-codec/Cargo.toml::features`.)

| Flag | Default | Gates | Implied deps |
|---|---|---|---|
| `gen-vectors`\index{gen-vectors (Cargo feature)} | no (`default = []` implicit) | the `gen_mk_vectors` binary at `src/bin/gen_mk_vectors.rs` | `dep:serde_json` |

The library API is unconditional: there are no `#[cfg(feature = ...)]` attributes anywhere under `src/` (verified by grep). `gen-vectors` is a binary-target-only gate that pulls in `serde_json` for emitting the canonical vector corpus. Library consumers can leave it off and never miss anything.

```toml
mk-codec = "0.2"
```

## V.2.3 Public API by module

Thirteen public modules: five top-level (`bytecode`, `consts`, `error`, `key_card`, `string_layer`; `src/lib.rs`) plus eight sub-modules under `bytecode` and `string_layer`. Re-exports at the crate root pull the most commonly-used items into `mk_codec::`:

```rust
pub use consts::{
    CHUNKED_FRAGMENT_LONG_BYTES, CHUNKED_FRAGMENT_REGULAR_BYTES,
    CROSS_CHUNK_HASH_BYTES, GENERATOR_FAMILY, HRP, MAX_CHUNKS,
    MAX_PATH_COMPONENTS, MK_LONG_CONST, MK_REGULAR_CONST, NUMS_DOMAIN,
    ORIGIN_FINGERPRINT_BYTES, POLICY_ID_STUB_BYTES,
    SINGLE_STRING_LONG_BYTES, SINGLE_STRING_REGULAR_BYTES,
    XPUB_COMPACT_BYTES,
};
pub use error::{Error, Result};
pub use key_card::{KeyCard, decode, encode, encode_with_chunk_set_id};
```

(`crates/mk-codec/src/lib.rs`.) No foreign types are re-exported; consumers needing `bitcoin::bip32::Xpub` or `bitcoin::bip32::DerivationPath` add `bitcoin = "0.32"` separately.

### V.2.3.1 `consts`\index{mk\_codec::consts}

Crate-wide constants (SPEC v0.1 closure questions Q-1, Q-2, Q-3, Q-7, Q-10; `consts.rs`).

| Item | Signature | Semantics | Source |
|---|---|---|---|
| `HRP`\index{HRP (mk1)} | `pub const HRP: &str = "mk"` | mk1 HRP (BIP 173 separator `1` follows) | `consts.rs::HRP` |
| `NUMS_DOMAIN`\index{NUMS\_DOMAIN} | `pub const NUMS_DOMAIN: &[u8] = b"shibbolethnumskey"` | domain string for NUMS-derived target residues | `consts.rs::NUMS_DOMAIN` |
| `MK_REGULAR_CONST`\index{MK\_REGULAR\_CONST} | `pub const MK_REGULAR_CONST: u128 = 0x1062435f91072fa5c` | top 65 bits of `SHA-256(NUMS_DOMAIN)`; regular-code target residue | `consts.rs::MK_REGULAR_CONST` |
| `MK_LONG_CONST`\index{MK\_LONG\_CONST} | `pub const MK_LONG_CONST: u128 = 0x41890d7e441cbe97273` | top 75 bits of `SHA-256(NUMS_DOMAIN)`; long-code target residue | `consts.rs::MK_LONG_CONST` |
| `MAX_PATH_COMPONENTS`\index{MAX\_PATH\_COMPONENTS (mk-codec)} | `pub const MAX_PATH_COMPONENTS: u8 = 10` | maximum components in an explicit-path encoding | `consts.rs::MAX_PATH_COMPONENTS` |
| `SINGLE_STRING_REGULAR_BYTES`\index{SINGLE\_STRING\_REGULAR\_BYTES} | `pub const SINGLE_STRING_REGULAR_BYTES: usize = 48` | single-string regular-code payload bytes | `consts.rs::SINGLE_STRING_REGULAR_BYTES` |
| `SINGLE_STRING_LONG_BYTES`\index{SINGLE\_STRING\_LONG\_BYTES} | `pub const SINGLE_STRING_LONG_BYTES: usize = 56` | single-string long-code payload bytes | `consts.rs::SINGLE_STRING_LONG_BYTES` |
| `CHUNKED_FRAGMENT_REGULAR_BYTES`\index{CHUNKED\_FRAGMENT\_REGULAR\_BYTES} | `pub const CHUNKED_FRAGMENT_REGULAR_BYTES: usize = 45` | chunked-fragment regular-code payload bytes per chunk | `consts.rs::CHUNKED_FRAGMENT_REGULAR_BYTES` |
| `CHUNKED_FRAGMENT_LONG_BYTES`\index{CHUNKED\_FRAGMENT\_LONG\_BYTES} | `pub const CHUNKED_FRAGMENT_LONG_BYTES: usize = 53` | chunked-fragment long-code payload bytes per chunk | `consts.rs::CHUNKED_FRAGMENT_LONG_BYTES` |
| `MAX_CHUNKS`\index{MAX\_CHUNKS} | `pub const MAX_CHUNKS: u8 = 32` | maximum chunks per card | `consts.rs::MAX_CHUNKS` |
| `CROSS_CHUNK_HASH_BYTES`\index{CROSS\_CHUNK\_HASH\_BYTES} | `pub const CROSS_CHUNK_HASH_BYTES: usize = 4` | cross-chunk integrity hash size in bytes | `consts.rs::CROSS_CHUNK_HASH_BYTES` |
| `GENERATOR_FAMILY`\index{GENERATOR\_FAMILY} | `pub const GENERATOR_FAMILY: &str = "mk-codec 0.2"` | family-stable BCH HRP-mixing token; rolls only on minor/major bumps | `consts.rs::GENERATOR_FAMILY` |
| `XPUB_COMPACT_BYTES`\index{XPUB\_COMPACT\_BYTES} | `pub const XPUB_COMPACT_BYTES: usize = 73` | compact-73 xpub byte size | `consts.rs::XPUB_COMPACT_BYTES` |
| `POLICY_ID_STUB_BYTES`\index{POLICY\_ID\_STUB\_BYTES} | `pub const POLICY_ID_STUB_BYTES: usize = 4` | policy-ID stub size in bytes | `consts.rs::ORIGIN_FINGERPRINT_BYTES` |
| `ORIGIN_FINGERPRINT_BYTES`\index{ORIGIN\_FINGERPRINT\_BYTES} | `pub const ORIGIN_FINGERPRINT_BYTES: usize = 4` | origin fingerprint size in bytes | `consts.rs::ORIGIN_FINGERPRINT_BYTES` |

### V.2.3.2 `error`\index{mk\_codec::error}

Error taxonomy (22 variants; full table in §V.2.4). Two public types:

| Item | Signature | Semantics | Source |
|---|---|---|---|
| `Error`\index{Error (mk-codec)} | `pub enum Error { ... }` (`#[non_exhaustive] #[derive(Debug, Error)]`) | 22 variants | `error.rs::Error` |
| `Result<T>`\index{Result (mk-codec)} | `pub type Result<T> = core::result::Result<T, Error>` | crate alias | `error.rs::Result` |

### V.2.3.3 `key_card`\index{mk\_codec::key\_card}

`KeyCard` struct and the public encode/decode pipeline entry points (`key_card.rs`).

| Item | Signature | Semantics | Source |
|---|---|---|---|
| `KeyCard`\index{KeyCard} | `pub struct KeyCard { pub policy_id_stubs: Vec<[u8; 4]>, pub origin_fingerprint: Option<Fingerprint>, pub origin_path: DerivationPath, pub xpub: Xpub }` (`#[non_exhaustive] #[derive(Debug, Clone, PartialEq, Eq)]`) | one decoded mk card | `key_card.rs::KeyCard` |
| `KeyCard::new`\index{KeyCard::new} | `fn new(policy_id_stubs: Vec<[u8; 4]>, origin_fingerprint: Option<Fingerprint>, origin_path: DerivationPath, xpub: Xpub) -> Self` | construct from owned fields. **Intentionally permissive** — field-level invariants enforced at encode time | `key_card.rs::KeyCard::new` |
| `encode`\index{encode (mk-codec)} | `fn encode(card: &KeyCard) -> Result<Vec<String>>` | encode a `KeyCard` into one or more mk1-prefixed strings; multi-chunk path draws a fresh 20-bit `chunk_set_id` from the OS CSPRNG. Delegates to `string_layer::encode` | `key_card.rs::encode` |
| `encode_with_chunk_set_id`\index{encode\_with\_chunk\_set\_id} | `fn encode_with_chunk_set_id(card: &KeyCard, chunk_set_id: u32) -> Result<Vec<String>>` | as `encode` with explicit `chunk_set_id ∈ 0..=0x000F_FFFF`; oversize returns `ChunkedHeaderMalformed`. Delegates to `string_layer::encode_with_chunk_set_id` | `key_card.rs::encode_with_chunk_set_id` |
| `decode`\index{decode (mk-codec)} | `fn decode(strings: &[&str]) -> Result<KeyCard>` | decode one or more mk1-prefixed strings into a `KeyCard`. Delegates to `string_layer::decode` | `key_card.rs::decode` |

```rust
use bitcoin::bip32::{DerivationPath, Fingerprint, Xpub};
use mk_codec::{KeyCard, encode, decode};
use std::str::FromStr;
let card = KeyCard::new(
    vec![*b"abcd"],
    Some(Fingerprint::from([0u8; 4])),
    DerivationPath::from_str("m/84'/0'/0'").unwrap(),
    xpub,
);
let strings: Vec<String> = encode(&card)?;
let refs: Vec<&str> = strings.iter().map(String::as_str).collect();
let round_tripped: KeyCard = decode(&refs)?;
assert_eq!(card, round_tripped);
```

### V.2.3.4 `bytecode`\index{mk\_codec::bytecode}

Bytecode-layer parent module (`bytecode/mod.rs`). Re-exports (`mod.rs`): `decode_bytecode`, `encode_bytecode`, `BytecodeHeader`, `STANDARD_PATHS`, `decode_path`, `encode_path`, `lookup_indicator`, `lookup_path`, `XpubCompact`, `decode_xpub_compact`, `encode_xpub_compact`, `reconstruct_xpub`. Sub-modules: `decode`, `encode`, `header`, `path`, `xpub_compact` (§V.2.3.5–§V.2.3.9).

### V.2.3.5 `bytecode::encode`\index{mk\_codec::bytecode::encode}

Canonical-bytecode encoder (`bytecode/encode.rs`).

| Item | Signature | Semantics | Source |
|---|---|---|---|
| `encode_bytecode`\index{encode\_bytecode} | `fn encode_bytecode(card: &KeyCard) -> Result<Vec<u8>>` | encode a `KeyCard` to canonical bytecode (pre-chunking). Rejects empty / >255 `policy_id_stubs` with `InvalidPolicyIdStubCount` | `bytecode/encode.rs::encode_bytecode` |

### V.2.3.6 `bytecode::decode`\index{mk\_codec::bytecode::decode}

Canonical-bytecode decoder (`bytecode/decode.rs`).

| Item | Signature | Semantics | Source |
|---|---|---|---|
| `decode_bytecode`\index{decode\_bytecode} | `fn decode_bytecode(bytes: &[u8]) -> Result<KeyCard>` | decode canonical bytecode into a `KeyCard`. Every SPEC §4 bytecode-layer validity rule surfaces as a unique `Error` variant | `bytecode/decode.rs::decode_bytecode` |

### V.2.3.7 `bytecode::header`\index{mk\_codec::bytecode::header}

Single-byte bytecode header parser (`bytecode/header.rs`).

| Item | Signature | Semantics | Source |
|---|---|---|---|
| `BytecodeHeader`\index{BytecodeHeader} | `pub struct BytecodeHeader { pub version: u8, pub fingerprint_flag: bool }` (`#[derive(Debug, Clone, Copy, PartialEq, Eq)]`) | parsed mk1 bytecode header | `bytecode/header.rs::BytecodeHeader` |
| `BytecodeHeader::parse` | `fn parse(byte: u8) -> Result<Self>` | parse one byte; rejects unknown version (`UnsupportedVersion`) or reserved bits (`ReservedBitsSet`) | `bytecode/header.rs::BytecodeHeader::parse` |
| `BytecodeHeader::to_byte` | `fn to_byte(self) -> u8` | serialize to single-byte wire form | `bytecode/header.rs::BytecodeHeader::to_byte` |

`BytecodeHeader` is **NOT** `#[non_exhaustive]`. See §V.2.7.

### V.2.3.8 `bytecode::path`\index{mk\_codec::bytecode::path}

Origin-path codec: standard-table dictionary + `0xFE` explicit-path escape hatch (`bytecode/path.rs`).

| Item | Signature | Semantics | Source |
|---|---|---|---|
| `EXPLICIT_PATH_INDICATOR`\index{EXPLICIT\_PATH\_INDICATOR} | `pub const EXPLICIT_PATH_INDICATOR: u8 = 0xFE` | indicator byte for an explicit (non-standard-table) path | `bytecode/path.rs::EXPLICIT_PATH_INDICATOR` |
| `STANDARD_PATHS`\index{STANDARD\_PATHS} | `pub const STANDARD_PATHS: &[(u8, &str)]` | 14 entries: 7 mainnet (`0x01`..=`0x07`), 7 testnet (`0x11`..=`0x17`). `0x16` (BIP 48 testnet nested-segwit multisig, `m/48'/1'/0'/1'`) added in v0.2.0. **mk1-internal** dictionary (not a sibling mirror — see §V.2.7) | `bytecode/path.rs::STANDARD_PATHS` |
| `lookup_indicator`\index{lookup\_indicator} | `fn lookup_indicator(indicator: u8) -> Option<DerivationPath>` | standard-table indicator → `DerivationPath`; `None` for reserved values | `bytecode/path.rs::lookup_indicator` |
| `lookup_path`\index{lookup\_path} | `fn lookup_path(path: &DerivationPath) -> Option<u8>` | `DerivationPath` → standard-table indicator; `None` triggers fallthrough to explicit-path. Structural comparison | `bytecode/path.rs::lookup_path` |
| `encode_path`\index{encode\_path} | `fn encode_path(path: &DerivationPath) -> Vec<u8>` | 1-byte indicator if available, else `0xFE` + count + LEB128 components | `bytecode/path.rs::encode_path` |
| `decode_path`\index{decode\_path} | `fn decode_path(cursor: &mut &[u8]) -> Result<DerivationPath>` | decode a path field at `*cursor` (advances). Emits `InvalidPathIndicator`, `PathTooDeep`, `InvalidPathComponent`, `UnexpectedEnd` | `bytecode/path.rs::decode_path` |

### V.2.3.9 `bytecode::xpub_compact`\index{mk\_codec::bytecode::xpub\_compact}

73-byte compact xpub form + reconstruction (`bytecode/xpub_compact.rs`).

| Item | Signature | Semantics | Source |
|---|---|---|---|
| `XpubCompact`\index{XpubCompact} | `pub struct XpubCompact { pub version: [u8; 4], pub parent_fingerprint: [u8; 4], pub chain_code: [u8; 32], pub public_key: [u8; 33] }` (`#[derive(Debug, Clone, PartialEq, Eq)]`) | 73-byte compact form (closure Q-7) | `bytecode/xpub_compact.rs::XpubCompact` |
| `XpubCompact::from_xpub`\index{XpubCompact::from\_xpub} | `fn from_xpub(xpub: &Xpub) -> Self` | build compact form from full BIP-32 `Xpub` | `bytecode/xpub_compact.rs::XpubCompact::from_xpub` |
| `reconstruct_xpub`\index{reconstruct\_xpub} | `fn reconstruct_xpub(compact: &XpubCompact, origin_path: &DerivationPath) -> Result<Xpub>` | reconstruct full `Xpub` (depth ← `len(origin_path)`, child_number ← last component). Emits `InvalidXpubVersion`, `InvalidXpubPublicKey`. **Panics** on empty path — see §V.2.7 | `bytecode/xpub_compact.rs::reconstruct_xpub` |
| `encode_xpub_compact`\index{encode\_xpub\_compact} | `fn encode_xpub_compact(compact: &XpubCompact, out: &mut Vec<u8>)` | append 73-byte wire layout to `out` | `bytecode/xpub_compact.rs::encode_xpub_compact` |
| `decode_xpub_compact`\index{decode\_xpub\_compact} | `fn decode_xpub_compact(cursor: &mut &[u8]) -> Result<XpubCompact>` | decode 73 bytes. Emits `UnexpectedEnd`, `InvalidXpubVersion` | `bytecode/xpub_compact.rs::decode_xpub_compact` |

`XpubCompact` is **NOT** `#[non_exhaustive]`. See §V.2.7.

```rust
use mk_codec::bytecode::{XpubCompact, reconstruct_xpub};
let compact = XpubCompact::from_xpub(&xpub);
// CALLER must validate path is non-empty before calling reconstruct_xpub:
assert!(!origin_path.as_ref().is_empty());
let rebuilt = reconstruct_xpub(&compact, &origin_path)?;
assert_eq!(rebuilt, xpub);
```

### V.2.3.10 `string_layer`\index{mk\_codec::string\_layer}

String-layer parent module (`string_layer/mod.rs`). Re-exports (`string_layer/mod.rs`): from `bch` — `ALPHABET`, `BchCode`, `CaseStatus`, `CorrectionResult`, `DecodedString`, `SEPARATOR`, `bch_correct_long`, `bch_correct_regular`, `bch_create_checksum_long`, `bch_create_checksum_regular`, `bch_verify_long`, `bch_verify_regular`, `bytes_to_5bit`, `case_check`, `decode_string`, `encode_5bit_to_string`, `five_bit_to_bytes`, `hrp_expand`; from `chunk` — `reassemble_from_chunks`, `split_into_chunks`; from `header` — `StringLayerHeader`; from private `pipeline` — `decode`, `encode`, `encode_with_chunk_set_id`. Sub-modules: `bch`, `chunk`, `header` (§V.2.3.11–§V.2.3.13). `bch_decode` is `pub(crate)` and `pipeline` is private — neither is part of the public surface.

Pipeline-level entry points (re-exported here, at `key_card`, and at the crate root):

| Item | Signature | Semantics | Source |
|---|---|---|---|
| `string_layer::encode` | `fn encode(card: &KeyCard) -> Result<Vec<String>>` | full encode pipeline; multi-chunk path draws `chunk_set_id` from `getrandom::getrandom`. **Panics** on CSPRNG failure — see §V.2.7 | `string_layer/pipeline.rs::encode` |
| `string_layer::encode_with_chunk_set_id` | `fn encode_with_chunk_set_id(card: &KeyCard, chunk_set_id: u32) -> Result<Vec<String>>` | as above with explicit id; rejects oversize with `ChunkedHeaderMalformed` | `string_layer/pipeline.rs::encode_with_chunk_set_id` |
| `string_layer::decode` | `fn decode(strings: &[&str]) -> Result<KeyCard>` | full decode pipeline; supports single + chunked; rejects mixing with `MixedHeaderTypes`; empty list returns `ChunkedHeaderMalformed` | `string_layer/pipeline.rs::decode` |

### V.2.3.11 `string_layer::bch`\index{mk\_codec::string\_layer::bch}

HRP-mixed BCH primitives (forked from BIP-93; `string_layer/bch.rs`). **Forked, not shared** — see §V.2.7.

Constants:

| Item | Signature | Semantics | Source |
|---|---|---|---|
| `ALPHABET`\index{ALPHABET (bech32)} | `pub const ALPHABET: &[u8; 32] = b"qpzry9x8gf2tvdw0s3jn54khce6mua7l"` | bech32 32-character alphabet in 5-bit-value order | `bch.rs::ALPHABET` |
| `SEPARATOR`\index{SEPARATOR (bech32)} | `pub const SEPARATOR: char = '1'` | bech32 separator (BIP 173 §3) | `bch.rs::SEPARATOR` |
| `GEN_REGULAR`\index{GEN\_REGULAR} | `pub const GEN_REGULAR: [u128; 5]` | BCH(93,80,8) polymod constants (BIP 93) | `bch.rs::GEN_REGULAR` |
| `POLYMOD_INIT`\index{POLYMOD\_INIT} | `pub const POLYMOD_INIT: u128 = 0x23181b3` | initial residue (BIP 93) | `bch.rs::POLYMOD_INIT` |
| `REGULAR_SHIFT`\index{REGULAR\_SHIFT} | `pub const REGULAR_SHIFT: u32 = 60` | right-shift to extract top 5 bits of 65-bit residue | `bch.rs::REGULAR_SHIFT` |
| `REGULAR_MASK`\index{REGULAR\_MASK} | `pub const REGULAR_MASK: u128 = 0x0fffffffffffffff` | low-60-bit mask | `bch.rs::REGULAR_MASK` |
| `GEN_LONG`\index{GEN\_LONG} | `pub const GEN_LONG: [u128; 5]` | BCH(108,93,8) polymod constants (BIP 93) | `bch.rs::GEN_LONG` |
| `LONG_SHIFT`\index{LONG\_SHIFT} | `pub const LONG_SHIFT: u32 = 70` | right-shift to extract top 5 bits of 75-bit residue | `bch.rs::LONG_SHIFT` |
| `LONG_MASK`\index{LONG\_MASK} | `pub const LONG_MASK: u128 = 0x3fffffffffffffffff` | low-70-bit mask | `bch.rs::LONG_MASK` |

Types:

| Item | Signature | Semantics | Source |
|---|---|---|---|
| `BchCode`\index{BchCode} | `pub enum BchCode { Regular, Long }` (`Hash` + standard derives; exhaustive) | which BCH code variant a string uses | `bch.rs::BchCode` |
| `CaseStatus`\index{CaseStatus} | `pub enum CaseStatus { Lower, Upper, Mixed }` (exhaustive) | case-check result | `bch.rs::CaseStatus` |
| `CorrectionResult`\index{CorrectionResult} | `pub struct CorrectionResult { pub data: Vec<u8>, pub corrections_applied: usize, pub corrected_positions: Vec<usize> }` (`#[non_exhaustive]`) | successful BCH decode+correct | `bch.rs::CorrectionResult` |
| `DecodedString`\index{DecodedString} | `pub struct DecodedString { pub code: BchCode, pub corrections_applied: usize, pub corrected_positions: Vec<usize>, pub data_with_checksum: Vec<u8> }` (`#[non_exhaustive]`) | successful mk1 string decode at the BCH layer | `bch.rs::DecodedString` |
| `DecodedString::data` | `fn data(&self) -> &[u8]` | data part as 5-bit values with trailing checksum stripped (chars after `"mk1"`) | `bch.rs::DecodedString::data` |
| `DecodedString::corrected_char_at` | `fn corrected_char_at(&self, char_position: usize) -> char` | corrected bech32 char at given position. **Panics** if `position >= data_with_checksum.len()` | `bch.rs::DecodedString::corrected_char_at` |

Functions:

| Item | Signature | Semantics | Source |
|---|---|---|---|
| `bytes_to_5bit`\index{bytes\_to\_5bit} | `fn bytes_to_5bit(bytes: &[u8]) -> Vec<u8>` | 8-bit → 5-bit (zero-padded) | `bch.rs::bytes_to_5bit` |
| `five_bit_to_bytes`\index{five\_bit\_to\_bytes} | `fn five_bit_to_bytes(values: &[u8]) -> Option<Vec<u8>>` | 5-bit → 8-bit; `None` for out-of-range or nonzero pad | `bch.rs::five_bit_to_bytes` |
| `bch_code_for_length`\index{bch\_code\_for\_length} | `fn bch_code_for_length(data_part_len: usize) -> Option<BchCode>` | `Regular` for 14..=93, `Long` for 96..=108, `None` for the 94..=95 reserved gap or out-of-range | `bch.rs::bch_code_for_length` |
| `case_check`\index{case\_check} | `fn case_check(s: &str) -> CaseStatus` | all-lower / all-upper / mixed | `bch.rs::case_check` |
| `hrp_expand`\index{hrp\_expand} | `fn hrp_expand(hrp: &str) -> Vec<u8>` | BIP 173 HRP-expansion; output length `2*hrp.len() + 1` for ASCII | `bch.rs::hrp_expand` |
| `bch_create_checksum_regular`\index{bch\_create\_checksum\_regular} | `fn bch_create_checksum_regular(hrp: &str, data: &[u8]) -> [u8; 13]` | 13-symbol regular checksum | `bch.rs::bch_create_checksum_regular` |
| `bch_verify_regular`\index{bch\_verify\_regular} | `fn bch_verify_regular(hrp: &str, data_with_checksum: &[u8]) -> bool` | verify regular checksum | `bch.rs::bch_verify_regular` |
| `bch_create_checksum_long`\index{bch\_create\_checksum\_long} | `fn bch_create_checksum_long(hrp: &str, data: &[u8]) -> [u8; 15]` | 15-symbol long checksum | `bch.rs::bch_create_checksum_long` |
| `bch_verify_long`\index{bch\_verify\_long} | `fn bch_verify_long(hrp: &str, data_with_checksum: &[u8]) -> bool` | verify long checksum | `bch.rs::bch_verify_long` |
| `bch_correct_regular`\index{bch\_correct\_regular} | `fn bch_correct_regular(hrp: &str, data_with_checksum: &[u8]) -> Result<CorrectionResult, Error>` | BM/Forney decoder; up to `t=4` substitutions for BCH(93,80,8); re-verifies post-correction. Emits `BchUncorrectable` | `bch.rs::bch_correct_regular` |
| `bch_correct_long`\index{bch\_correct\_long} | `fn bch_correct_long(hrp: &str, data_with_checksum: &[u8]) -> Result<CorrectionResult, Error>` | long-code analog | `bch.rs::bch_correct_long` |
| `encode_5bit_to_string`\index{encode\_5bit\_to\_string} | `fn encode_5bit_to_string(data_5bit: &[u8]) -> Result<String, Error>` | encode 5-bit symbols as complete mk1 string. Auto-selects regular/long. Output begins `"mk1"`. Emits `InvalidStringLength` | `bch.rs::encode_5bit_to_string` |
| `decode_string`\index{decode\_string} | `fn decode_string(s: &str) -> Result<DecodedString, Error>` | full BCH-layer decode with `t=4` correction. Emits `MixedCase`, `InvalidHrp`, `InvalidStringLength`, `InvalidChar`, `BchUncorrectable` | `bch.rs::decode_string` |

```rust
use mk_codec::string_layer::{
    bytes_to_5bit, encode_5bit_to_string, decode_string,
};
let bits5 = bytes_to_5bit(&bytecode);
let card = encode_5bit_to_string(&bits5)?;       // "mk1..."
let parsed = decode_string(&card)?;
assert_eq!(parsed.corrections_applied, 0);
```

### V.2.3.12 `string_layer::header`\index{mk\_codec::string\_layer::header}

Per-string header (`string_layer/header.rs`).

| Item | Signature | Semantics | Source |
|---|---|---|---|
| `SINGLE_HEADER_SYMBOLS`\index{SINGLE\_HEADER\_SYMBOLS} | `pub const SINGLE_HEADER_SYMBOLS: usize = 2` | 5-bit symbols in single-string header | `string_layer/header.rs::SINGLE_HEADER_SYMBOLS` |
| `CHUNKED_HEADER_SYMBOLS`\index{CHUNKED\_HEADER\_SYMBOLS} | `pub const CHUNKED_HEADER_SYMBOLS: usize = 8` | 5-bit symbols in chunked header | `string_layer/header.rs::CHUNKED_HEADER_SYMBOLS` |
| `MAX_CHUNK_SET_ID`\index{MAX\_CHUNK\_SET\_ID} | `pub const MAX_CHUNK_SET_ID: u32 = (1 << 20) - 1` | maximum 20-bit `chunk_set_id` | `string_layer/header.rs::MAX_CHUNK_SET_ID` |
| `VERSION_V0_1`\index{VERSION\_V0\_1} | `pub const VERSION_V0_1: u8 = 0x00` | format-version field emitted in v0.1 | `string_layer/header.rs::VERSION_V0_1` |
| `StringLayerHeader`\index{StringLayerHeader} | `pub enum StringLayerHeader { SingleString { version: u8 }, Chunked { version: u8, chunk_set_id: u32, total_chunks: u8, chunk_index: u8 } }` (`#[non_exhaustive]`) | per-string header. `total_chunks` is 1-based; wire encoding is `count − 1` | `string_layer/header.rs::StringLayerHeader` |
| `StringLayerHeader::to_5bit_symbols` | `fn to_5bit_symbols(self) -> Vec<u8>` | emit 2 or 8 symbols | `string_layer/header.rs::StringLayerHeader::to_5bit_symbols` |
| `StringLayerHeader::from_5bit_symbols` | `fn from_5bit_symbols(symbols: &[u8]) -> Result<(Self, usize)>` | parse leading header; returns `(header, consumed)`. Emits `UnexpectedEnd`, `UnsupportedVersion`, `UnsupportedCardType`, `ChunkedHeaderMalformed` | `string_layer/header.rs::StringLayerHeader::from_5bit_symbols` |
| `StringLayerHeader::is_chunked` | `fn is_chunked(self) -> bool` | discriminant predicate | `string_layer/header.rs::StringLayerHeader::is_chunked` |

### V.2.3.13 `string_layer::chunk`\index{mk\_codec::string\_layer::chunk}

Chunked-card framing + reassembly (`string_layer/chunk.rs`).

| Item | Signature | Semantics | Source |
|---|---|---|---|
| `MAX_CHUNKABLE_BYTECODE`\index{MAX\_CHUNKABLE\_BYTECODE} | `pub const MAX_CHUNKABLE_BYTECODE: usize = MAX_CHUNKS * CHUNKED_FRAGMENT_LONG_BYTES − CROSS_CHUNK_HASH_BYTES` | `= 32 × 53 − 4 = 1692 bytes` | `string_layer/chunk.rs::MAX_CHUNKABLE_BYTECODE` |
| `ChunkFragment`\index{ChunkFragment} | `pub struct ChunkFragment { pub header: StringLayerHeader, pub fragment: Vec<u8> }` (`#[non_exhaustive]`) | one chunk's header + payload bytes | `string_layer/chunk.rs::ChunkFragment` |
| `split_into_chunks`\index{split\_into\_chunks} | `fn split_into_chunks(canonical_bytecode: &[u8], chunk_set_id: u32) -> Result<Vec<ChunkFragment>>` | split + append cross-chunk hash. Byte-deterministic in `(bytecode, chunk_set_id)`. Emits `ChunkedHeaderMalformed`, `CardPayloadTooLarge` | `string_layer/chunk.rs::split_into_chunks` |
| `reassemble_from_chunks`\index{reassemble\_from\_chunks} | `fn reassemble_from_chunks(chunks: Vec<ChunkFragment>) -> Result<Vec<u8>>` | inverse of `split_into_chunks`; validates SPEC §4 rules 11-13 + cross-chunk hash; accepts any chunk order (sorts internally). Emits `ChunkedHeaderMalformed`, `ChunkSetIdMismatch`, `MixedHeaderTypes`, `CrossChunkHashMismatch` | `string_layer/chunk.rs::reassemble_from_chunks` |

```rust
use mk_codec::bytecode::encode_bytecode;
use mk_codec::string_layer::{split_into_chunks, reassemble_from_chunks};
let bytes = encode_bytecode(&card)?;
let chunks = split_into_chunks(&bytes, 0x000A_BCDE)?;
let restored = reassemble_from_chunks(chunks)?;
assert_eq!(bytes, restored);
```

## V.2.4 Error taxonomy

`pub enum Error` from `crates/mk-codec/src/error.rs` — 22 variants (lines 20-162), `#[non_exhaustive] #[derive(Debug, Error)]`. Grouped by emit-site cluster; within each group, ordered by source line.

### String-layer (codex32 plumbing, HRP, chunk-header)

| Variant | Display | Emitted by | Source |
|---|---|---|---|
| `InvalidHrp(String)`\index{Error::InvalidHrp} | `invalid HRP: {0}` | `string_layer::bch::decode_string` | `bch.rs::decode_string` |
| `MixedCase`\index{Error::MixedCase} | `mixed case in input string` | `string_layer::bch::decode_string` | `bch.rs::decode_string` |
| `InvalidStringLength(usize)`\index{Error::InvalidStringLength} | `invalid data-part length: {0}` | `bch::encode_5bit_to_string`, `bch::decode_string` | `bch.rs::encode_5bit_to_string, bch.rs::decode_string` |
| `InvalidChar { ch, position }`\index{Error::InvalidChar} | `invalid character {ch} at position {position}` | `bch::decode_string` | `bch.rs::decode_string` |
| `BchUncorrectable(String)`\index{Error::BchUncorrectable} | `BCH uncorrectable: {0}` | `bch::bch_correct_regular`, `bch::bch_correct_long` | `bch.rs::bch_correct_regular, bch.rs::bch_correct_long` |
| `UnsupportedCardType(u8)`\index{Error::UnsupportedCardType} | `unsupported card type: 0x{0:02x}` | `header::StringLayerHeader::from_5bit_symbols` | `string_layer/header.rs::StringLayerHeader::from_5bit_symbols` |
| `MalformedPayloadPadding`\index{Error::MalformedPayloadPadding} | `malformed payload padding (5-bit symbols don't byte-align)` | `string_layer::pipeline::decode` | `pipeline.rs::decode` |
| `ChunkSetIdMismatch`\index{Error::ChunkSetIdMismatch (mk-codec)} | `chunk_set_id mismatch across chunks` | `chunk::reassemble_from_chunks` | `string_layer/chunk.rs::reassemble_from_chunks` |
| `ChunkedHeaderMalformed(String)`\index{Error::ChunkedHeaderMalformed} | `chunked-header malformed: {0}` | `header::from_5bit_symbols`, `chunk::split_into_chunks`, `chunk::reassemble_from_chunks`, `pipeline::encode_bytecode_stream`, `pipeline::decode` | `string_layer/header.rs::StringLayerHeader::from_5bit_symbols`; `chunk.rs::split_into_chunks, chunk.rs::reassemble_from_chunks`; `pipeline.rs::encode_bytecode_stream, pipeline.rs::decode` |
| `MixedHeaderTypes`\index{Error::MixedHeaderTypes} | `mixed string-layer header types in input list` | `chunk::reassemble_from_chunks`, `pipeline::decode` | `chunk.rs::reassemble_from_chunks`; `pipeline.rs::decode` |
| `CrossChunkHashMismatch`\index{Error::CrossChunkHashMismatch} | `cross-chunk integrity hash mismatch` | `chunk::reassemble_from_chunks` | `string_layer/chunk.rs::reassemble_from_chunks` |

### Bytecode-layer

| Variant | Display | Emitted by | Source |
|---|---|---|---|
| `UnsupportedVersion(u8)`\index{Error::UnsupportedVersion (mk-codec)} | `unsupported version: {0}` | `bytecode::header::BytecodeHeader::parse`, `string_layer::header::from_5bit_symbols` | `bytecode/header.rs::BytecodeHeader::parse`; `string_layer/header.rs::StringLayerHeader::from_5bit_symbols` |
| `ReservedBitsSet`\index{Error::ReservedBitsSet} | `reserved bits set in bytecode header` | `bytecode::header::BytecodeHeader::parse` | `bytecode/header.rs::BytecodeHeader::parse` |
| `InvalidPolicyIdStubCount`\index{Error::InvalidPolicyIdStubCount} | `policy_id_stub_count must be >= 1` | `bytecode::encode::encode_bytecode`, `bytecode::decode::decode_bytecode` | `bytecode/encode.rs::encode_bytecode`; `bytecode/decode.rs::decode_bytecode` |
| `InvalidPathIndicator(u8)`\index{Error::InvalidPathIndicator} | `invalid path indicator byte: 0x{0:02x}` | `bytecode::path::decode_path` | `bytecode/path.rs::decode_path` |
| `PathTooDeep(u8)`\index{Error::PathTooDeep} | `path too deep: {0} components (max 10)` | `bytecode::path::decode_path` → `decode_explicit_path` | `bytecode/path.rs::decode_explicit_path` |
| `InvalidPathComponent(String)`\index{Error::InvalidPathComponent} | `invalid path component: {0}` | `bytecode::path::decode_path` | `bytecode/path.rs::decode_explicit_path, ::leb128_decode_u32` |
| `InvalidXpubVersion(u32)`\index{Error::InvalidXpubVersion} | `invalid xpub version: 0x{0:08x}` | `bytecode::xpub_compact::version_to_network` | `bytecode/xpub_compact.rs::version_to_network` |
| `InvalidXpubPublicKey(String)`\index{Error::InvalidXpubPublicKey} | `invalid xpub public key: {0}` | `bytecode::xpub_compact::reconstruct_xpub` | `bytecode/xpub_compact.rs::reconstruct_xpub` |
| `UnexpectedEnd`\index{Error::UnexpectedEnd (mk-codec)} | `unexpected end of bytecode` | `bytecode::decode::decode_bytecode`, `bytecode::path::decode_path`, `bytecode::xpub_compact::decode_xpub_compact`, `string_layer::header::from_5bit_symbols` | `bytecode/decode.rs::read_u8, ::read_array`; `bytecode/path.rs::read_u8`; `bytecode/xpub_compact.rs::decode_xpub_compact`; `string_layer/header.rs::StringLayerHeader::from_5bit_symbols` |
| `TrailingBytes`\index{Error::TrailingBytes} | `trailing bytes after xpub` | `bytecode::decode::decode_bytecode` | `bytecode/decode.rs::decode_bytecode` |
| `CardPayloadTooLarge { bytecode_len, max_supported }`\index{Error::CardPayloadTooLarge} | `card payload too large: bytecode_len = {…} > max_supported = {…}` | `string_layer::chunk::split_into_chunks` | `string_layer/chunk.rs::split_into_chunks` |

(Variant count = 22.)

## V.2.5 Integration patterns

### V.2.5.1 Encoder pipeline

`KeyCard` → bytecode → BCH-protected mk1 string(s).

- Build a `KeyCard` (typically obtained from `mnemonic-toolkit` or `mk-cli`; the field-level invariants — `policy_id_stubs` non-empty, path depth ≤ `MAX_PATH_COMPONENTS = 10` — are NOT checked by `KeyCard::new`; they fire at encode time).
- Call `encode(&card)`. Internally:
  1. `bytecode::encode::encode_bytecode(&card)` bit-packs to canonical bytecode, rejecting invalid stub counts with `InvalidPolicyIdStubCount`.
  2. If bytecode fits in `SINGLE_STRING_LONG_BYTES = 56` bytes (after header), `string_layer::pipeline` emits one mk1 string via `string_layer::bch::encode_5bit_to_string`.
  3. Otherwise it chunks: draws a 20-bit `chunk_set_id` from `getrandom::getrandom` (or accepts an explicit one via `encode_with_chunk_set_id`), invokes `chunk::split_into_chunks` (appending a 4-byte `SHA-256(canonical_bytecode)[0..4]` cross-chunk hash), and emits one mk1 string per chunk.
- For maximum reproducibility (e.g. test vectors), call `encode_with_chunk_set_id(&card, fixed_id)` — same input + same id ⇒ byte-identical chunked output.

```rust
use mk_codec::{encode, encode_with_chunk_set_id, KeyCard};
let strings: Vec<String> = encode(&card)?;        // OS CSPRNG for chunk_set_id
let deterministic = encode_with_chunk_set_id(&card, 0x000A_BCDE)?;
```

### V.2.5.2 Decoder pipeline

mk1 string(s) → BCH-correction → bytecode → `KeyCard`.

- `decode(&[&str])` is the single entry point. It:
  1. Parses each string's case + HRP (`bch::case_check`, `bch::decode_string`).
  2. Runs BCH error-correction up to `t=4` substitutions per string (`bch_correct_regular` / `bch_correct_long`); successful corrections surface on the inner `DecodedString::corrected_positions` but are not exposed by the top-level pipeline.
  3. Parses the per-string `StringLayerHeader`; rejects mixed `SingleString`/`Chunked` input with `MixedHeaderTypes`.
  4. For chunked input, calls `chunk::reassemble_from_chunks` (any order; validates index gaps/duplicates, `chunk_set_id` agreement, cross-chunk SHA-256 hash).
  5. Calls `bytecode::decode::decode_bytecode` on the reassembled canonical bytecode (`KeyCard` reconstruction via `bytecode::xpub_compact::reconstruct_xpub`).
- Empty input list returns `ChunkedHeaderMalformed`.

```rust
use mk_codec::decode;
let card = decode(&[s1, s2, s3])?;      // chunked
let card = decode(&[single])?;          // single-string
```

### V.2.5.3 Chunked reassembly

`split_into_chunks` is byte-deterministic in `(canonical_bytecode, chunk_set_id)`. Each chunk carries its own BCH checksum, so per-chunk damage is locally detectable. The shared 20-bit `chunk_set_id` binds them to one bytecode; the trailing 4-byte cross-chunk SHA-256 hash binds the reassembled byte-string to its declared identity.

- `reassemble_from_chunks` accepts chunks in any order (sorts internally by `chunk_index`).
- Mixed-bundle inputs surface as `ChunkSetIdMismatch` (different `chunk_set_id` across chunks) or `CrossChunkHashMismatch` (correct ids but reassembled bytes hash to the wrong 4-byte prefix).
- `MixedHeaderTypes` fires when a `SingleString`-header chunk is mixed into a chunked set.

## V.2.6 Versioning and MSRV

- Crate version: **0.2.2** (HEAD `e8782fd`).
- Rust edition: **2024** (inherited from workspace `Cargo.toml`).
- MSRV: **1.85** (`rust-version` inherited from workspace).
- License: **MIT**.
- Public semver promise: **none**. Pre-1.0 reference implementation; any 0.X bump may break. The v0.1 wire format is locked (v0.2.x additions are wire-additive: `STANDARD_PATHS` entry `0x16` added in v0.2.0; pre-v0.2 decoders reject it as `InvalidPathIndicator(0x16)`). The `GENERATOR_FAMILY = "mk-codec 0.2"` token rolls only on minor/major bumps — v0.2.0 → v0.2.2 keeps it constant.

## V.2.7 Notes for advanced users

- **BCH primitives are forked from BIP-93, not shared.** `string_layer::bch` is a fork of md-codec's BCH code retargeted to HRP `"mk"` and the NUMS-derived residues `MK_REGULAR_CONST` / `MK_LONG_CONST`. The originally-planned `mc-codex32` shared crate was retired on 2026-05-03 (`mc-codex32-extraction-retired-2026-05-03` in `mnemonic-key/design/FOLLOWUPS.md`); md1↔mk1 BCH stays forked indefinitely. The full BCH primitive surface is public — `GEN_REGULAR`, `GEN_LONG`, `POLYMOD_INIT`, `REGULAR_SHIFT`/`REGULAR_MASK`, `LONG_SHIFT`/`LONG_MASK`, `hrp_expand`, `bch_create_checksum_*`, `bch_verify_*`, `bch_correct_*`, `bch_code_for_length` — so callers can in principle build mk-codec-compatible string-layer codecs without going through `KeyCard`. See §I.3 for the HRP-mixing pattern and the BIP-93 differences.
- **Path dictionary is mk1-internal (standalone).** `STANDARD_PATHS` is the 14-entry mk1 path table; it is **not** a mirror of any md-codec table. The md1↔mk1 path-dictionary mirror invariant was retired post-md-codec v0.11 (which dropped path dictionaries entirely; paths in md1 are now encoded explicitly via `OriginPath`). The mk1 dictionary remains, but downstream tooling should treat it as mk1-only. The `bytecode::path` module's doc-comment (`bytecode/path.rs`) is the authoritative source.
- **Inconsistent `#[non_exhaustive]` policy.** `KeyCard`, `Error`, `StringLayerHeader`, `CorrectionResult`, `DecodedString`, `ChunkFragment` ARE marked `#[non_exhaustive]`. `BchCode`, `CaseStatus`, `BytecodeHeader`, and `XpubCompact` are NOT. Exhaustive struct-literal construction (or `match`-without-`_`) on the unmarked four is brittle — a future field/variant addition would be a breaking change without the attribute as a warning aid. Not necessarily a defect (header / compact-xpub fields are wire-locked at v0.1; the two enums are stable-by-design) but worth surfacing.
- **`reconstruct_xpub` panics on empty `origin_path`.** The implementation contains `.expect("origin_path must be non-empty per SPEC §3.5")` (`bytecode/xpub_compact.rs::reconstruct_xpub`). The pipeline guarantees non-emptiness because standard-table indicators resolve to paths with ≥3 components and explicit-path encoding requires `count ≥ 1`. **External callers using `XpubCompact` directly (outside `decode`) must pre-check** `!origin_path.as_ref().is_empty()`. Out-of-spec failure mode; not a `Result`.
- **`encode` / `encode_with_chunk_set_id` panic on OS CSPRNG failure.** `string_layer::pipeline::fresh_chunk_set_id` panics with `"OS CSPRNG must be available for mk1 encode"` if `getrandom::getrandom` returns an error (`string_layer/pipeline.rs::fresh_chunk_set_id`). Documented behaviour, but a non-`Result` failure mode in the encode path. Callers running in adversarial environments (e.g. exotic embedded targets without entropy access) should pre-check `getrandom::getrandom` themselves or use `encode_with_chunk_set_id` with a caller-supplied id.
- **`DecodedString::corrected_char_at` panics on out-of-range position** (`>= data_with_checksum.len()`). Documented as a `# Panics` clause in the doc-comment (`string_layer/bch.rs::DecodedString::corrected_char_at`).
- **`KeyCard::new` is intentionally permissive.** Field-level invariants (non-empty `policy_id_stubs`, `origin_path` depth ≤ `MAX_PATH_COMPONENTS`, well-formed `xpub`) are enforced at encode time, not at construction. Round-trip safety holds only post-encode; `KeyCard::new` accepts invalid inputs that the encoder will subsequently reject.
- **`Error` is `#[non_exhaustive]`.** External `match` arms must include a `_ =>` arm. Standard for the m-format family.
- **`bech32 = "0.11"` is declared but unused in public signatures.** A grep across `src/` for `bech32::` yields zero matches in any `pub` signature; the crate uses its own bech32-alphabet primitives (`ALPHABET`, `bytes_to_5bit`, etc.). The declaration is retained as a reserved dependency — do not assume it is dropped in a future patch release.
- **Three pre-existing `cargo doc` warnings.** Documentation-only, not user-affecting: (1) private intra-doc link to `bch_decode` from `string_layer/mod.rs` (the module is `pub(crate)`); (2) unresolved link `Error::InvalidStringLength` from `string_layer/bch.rs` (missing `crate::` prefix); (3) unresolved link `crate::Correction::corrected` from `string_layer/bch.rs` (a stale copy-from-md-codec doc-comment; the mk-codec equivalent is `DecodedString::corrected_char_at`). Candidates for a future doc-only patch release.
- **Stale `"md1"` doc-strings in `DecodedString`.** Two doc-comments at `string_layer/bch.rs` and `string_layer/bch.rs` mention `"md1"` where they should say `"mk1"` — leftover from the md-codec fork. mk1 strings begin with HRP `"mk"` + separator `'1'` = `"mk1"`. The wire format is unaffected; only the doc-comments drift.

## Cross-references

- §I.3 — codex32 and BCH (HRP-mixing background for the BIP-93 fork these APIs ship).
- §II.2 — mk1 wire format (the bit-level layout these APIs encode/decode).
- §V.1 — md-codec (sibling crate; the BCH lineage forks from md-codec's implementation).
- Worked example: `cargo run --quiet --manifest-path docs/technical-manual/examples/Cargo.toml --example mk-codec-api-roundtrip` — source at `docs/technical-manual/examples/examples/mk-codec-api-roundtrip.rs`; transcript pair at `docs/technical-manual/transcripts/mk-codec-api-roundtrip.{cmd,out}`.

<!-- cspell-additions: (none — every new term is taken from the existing manual or harvest doc-comments; the harvest's review-cycle gate guarantees vocabulary alignment) -->
