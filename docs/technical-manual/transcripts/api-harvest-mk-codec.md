# mk-codec API surface harvest

| Field | Value |
|---|---|
| Crate | mk-codec |
| Version | 0.2.2 |
| Source root | /scratch/code/shibboleth/mnemonic-key/crates/mk-codec |
| HEAD commit | e8782fd7d1e47f5531ab777820e9371d5fa9dd08 |
| Rust edition | 2024 (workspace) |
| MSRV | 1.85 (workspace `rust-version`) |
| `cargo doc` status | builds; 3 rustdoc warnings (private intra-doc link to `bch_decode`, two unresolved-link warnings in `string_layer/bch.rs:506` and `:610`) |

## Feature flags

| Flag | Gates | Default |
|---|---|---|
| `gen-vectors` | Optional `dep:serde_json`; required-features for the `gen_mk_vectors` binary at `src/bin/gen_mk_vectors.rs` | OFF |

No `#[cfg(feature = ...)]` attributes appear in any `src/` file (verified by `grep -rn '#[cfg(feature' src/`). Feature-gating is entirely at the Cargo.toml `[dependencies]` / `[[bin]] required-features` level.

## Dependencies (public-facing — types/traits appearing in public signatures)

- `bitcoin = "0.32"` — `bitcoin::bip32::{DerivationPath, Fingerprint, Xpub, ChildNumber, ChainCode}`, `bitcoin::secp256k1::PublicKey`, `bitcoin::NetworkKind`, `bitcoin::hashes::{Hash, sha256}` appear in public signatures (`KeyCard.origin_path`, `KeyCard.origin_fingerprint`, `KeyCard.xpub`, `reconstruct_xpub`, `XpubCompact::from_xpub`).
- `thiserror = "2.0"` — `#[derive(Error)]` on `enum Error`. The `Display` and `std::error::Error` impls are part of the public surface but the macro generates them; no direct `thiserror` types appear in signatures.
- `bech32 = "0.11"` — declared in Cargo.toml but no `bech32::*` types appear in any `pub` signature. (Internally used; flag-only review confirms zero `pub`-level leakage. See "Notes for chapter author".)
- `getrandom = "0.2"` — used internally by `string_layer::pipeline::fresh_chunk_set_id`; no `getrandom` types in public signatures.

## Public modules (top-level)

`src/lib.rs:37-41`:

- `pub mod bytecode`
- `pub mod consts`
- `pub mod error`
- `pub mod key_card`
- `pub mod string_layer`

`pub(crate)` siblings (not part of public surface, listed for completeness):

- `string_layer::bch_decode` — `pub(crate)`, BM/Forney decoder impl detail; not user-facing.
- `bytecode::test_helpers` — `pub(crate)`, `#[cfg(test)]`-only fixture.
- `string_layer::pipeline` — `mod pipeline;` (private), re-exports the public `encode` / `decode` / `encode_with_chunk_set_id` via `string_layer::mod.rs`.

## Public surface by module

### `mk_codec` (crate root, `src/lib.rs`)

#### Re-exports (`pub use`)

From `consts` (`src/lib.rs:43-48`):
`CHUNKED_FRAGMENT_LONG_BYTES`, `CHUNKED_FRAGMENT_REGULAR_BYTES`, `CROSS_CHUNK_HASH_BYTES`, `GENERATOR_FAMILY`, `HRP`, `MAX_CHUNKS`, `MAX_PATH_COMPONENTS`, `MK_LONG_CONST`, `MK_REGULAR_CONST`, `NUMS_DOMAIN`, `ORIGIN_FINGERPRINT_BYTES`, `POLICY_ID_STUB_BYTES`, `SINGLE_STRING_LONG_BYTES`, `SINGLE_STRING_REGULAR_BYTES`, `XPUB_COMPACT_BYTES`.

From `error` (`src/lib.rs:49`): `Error`, `Result`.

From `key_card` (`src/lib.rs:50`): `KeyCard`, `decode`, `encode`, `encode_with_chunk_set_id`.

#### Functions

(All top-level `pub fn` are re-exports — there are no functions defined directly in `lib.rs`.)

#### Types / Traits / Constants

(All top-level types/constants are re-exports.)

---

### `mk_codec::consts` (`src/consts.rs`)

#### Constants

- `pub const HRP: &str = "mk"` — HRP for `mk1` strings (BIP 173 separator `1` follows). `src/consts.rs:9`.
- `pub const NUMS_DOMAIN: &[u8] = b"shibbolethnumskey"` — Domain string for NUMS-derived target constants (closure Q-1). `src/consts.rs:15`.
- `pub const MK_REGULAR_CONST: u128 = 0x1062435f91072fa5c` — Top 65 bits of `SHA-256(NUMS_DOMAIN)`. Regular-code target residue. `src/consts.rs:18`.
- `pub const MK_LONG_CONST: u128 = 0x41890d7e441cbe97273` — Top 75 bits of `SHA-256(NUMS_DOMAIN)`. Long-code target residue. `src/consts.rs:21`.
- `pub const MAX_PATH_COMPONENTS: u8 = 10` — Maximum components in an explicit-path encoding (closure Q-3). `src/consts.rs:27`.
- `pub const SINGLE_STRING_REGULAR_BYTES: usize = 48` — Single-string regular-code payload bytes. `src/consts.rs:30`.
- `pub const SINGLE_STRING_LONG_BYTES: usize = 56` — Single-string long-code payload bytes. `src/consts.rs:33`.
- `pub const CHUNKED_FRAGMENT_REGULAR_BYTES: usize = 45` — Chunked-fragment regular-code payload bytes per chunk. `src/consts.rs:36`.
- `pub const CHUNKED_FRAGMENT_LONG_BYTES: usize = 53` — Chunked-fragment long-code payload bytes per chunk. `src/consts.rs:39`.
- `pub const MAX_CHUNKS: u8 = 32` — Maximum chunks per card. `src/consts.rs:42`.
- `pub const CROSS_CHUNK_HASH_BYTES: usize = 4` — Cross-chunk integrity hash size in bytes. `src/consts.rs:45`.
- `pub const GENERATOR_FAMILY: &str = "mk-codec 0.2"` — Family-stable generator string (closure Q-10) for vector-corpus SHA-256 anchoring. `src/consts.rs:50`.
- `pub const XPUB_COMPACT_BYTES: usize = 73` — Compact-73 xpub byte size (closure Q-7). `src/consts.rs:53`.
- `pub const POLICY_ID_STUB_BYTES: usize = 4` — Policy ID stub size in bytes (closure Q-2). `src/consts.rs:56`.
- `pub const ORIGIN_FINGERPRINT_BYTES: usize = 4` — Origin fingerprint size in bytes. `src/consts.rs:59`.

---

### `mk_codec::error` (`src/error.rs`)

#### Types

- `pub enum Error` — `#[non_exhaustive] #[derive(Debug, Error)]`. All errors `mk-codec` can produce. 22 variants. `src/error.rs:20`. (Full enumeration in the Error taxonomy table below.)
- `pub type Result<T> = core::result::Result<T, Error>` — Crate `Result` alias. `src/error.rs:165`.

---

### `mk_codec::key_card` (`src/key_card.rs`)

#### Types

- `pub struct KeyCard` — `#[non_exhaustive] #[derive(Debug, Clone, PartialEq, Eq)]`. In-memory representation of one decoded MK card. `src/key_card.rs:24`.
  - `pub policy_id_stubs: Vec<[u8; 4]>` — Policy ID stubs declaring which MD-encoded policy template(s) this xpub serves. Top 4 bytes of `SHA-256(canonical_bytecode)` of each policy. Non-empty post-decode.
  - `pub origin_fingerprint: Option<Fingerprint>` — Master-key fingerprint per BIP 380 `[fp/...]`. Optional per closure Q-8.
  - `pub origin_path: DerivationPath` — Derivation path from master to `xpub`. Encoded either via 1-byte standard-path indicator or via `0xFE` explicit-path escape hatch.
  - `pub xpub: Xpub` — The BIP 32 extended public key (reconstructed at decode time with `depth = component_count(origin_path)`, `child_number = last_component(origin_path)`).

#### Functions

- `pub fn KeyCard::new(policy_id_stubs: Vec<[u8; 4]>, origin_fingerprint: Option<Fingerprint>, origin_path: DerivationPath, xpub: Xpub) -> Self` — Construct a `KeyCard` from its four owned fields. Intentionally permissive; field-level validation lives at encode time. `src/key_card.rs:79`.
- `pub fn encode(card: &KeyCard) -> Result<Vec<String>>` — Encode a `KeyCard` into one or more `mk1`-prefixed strings. Multi-chunk path draws a fresh 20-bit `chunk_set_id` from the system CSPRNG. `src/key_card.rs:99`. Delegates to `string_layer::encode`.
- `pub fn encode_with_chunk_set_id(card: &KeyCard, chunk_set_id: u32) -> Result<Vec<String>>` — Like `encode` with an explicit `chunk_set_id` override. `chunk_set_id` MUST fit in 20 bits (`0..=0x000F_FFFF`); else returns `Error::ChunkedHeaderMalformed`. `src/key_card.rs:109`. Delegates to `string_layer::encode_with_chunk_set_id`.
- `pub fn decode(strings: &[&str]) -> Result<KeyCard>` — Decode one or more `mk1`-prefixed strings into a `KeyCard`. `src/key_card.rs:114`. Delegates to `string_layer::decode`.

---

### `mk_codec::bytecode` (`src/bytecode/mod.rs`)

#### Re-exports (`pub use`)

- From `decode`: `decode_bytecode` (`src/bytecode/mod.rs:27`).
- From `encode`: `encode_bytecode` (`src/bytecode/mod.rs:28`).
- From `header`: `BytecodeHeader` (`src/bytecode/mod.rs:29`).
- From `path`: `STANDARD_PATHS`, `decode_path`, `encode_path`, `lookup_indicator`, `lookup_path` (`src/bytecode/mod.rs:30`).
- From `xpub_compact`: `XpubCompact`, `decode_xpub_compact`, `encode_xpub_compact`, `reconstruct_xpub` (`src/bytecode/mod.rs:31`).

#### Sub-modules

`pub mod decode;` `pub mod encode;` `pub mod header;` `pub mod path;` `pub mod xpub_compact;` (all in `src/bytecode/mod.rs:18-22`).

---

### `mk_codec::bytecode::encode` (`src/bytecode/encode.rs`)

#### Functions

- `pub fn encode_bytecode(card: &KeyCard) -> Result<Vec<u8>>` — Encode a `KeyCard` to its canonical bytecode form (pre-chunking). `src/bytecode/encode.rs:21`. Rejects empty / >255 `policy_id_stubs` with `Error::InvalidPolicyIdStubCount`.

---

### `mk_codec::bytecode::decode` (`src/bytecode/decode.rs`)

#### Functions

- `pub fn decode_bytecode(bytes: &[u8]) -> Result<KeyCard>` — Decode canonical bytecode (pre-chunking) into a `KeyCard`. Surfaces every SPEC §4 bytecode-layer validity rule via a unique `Error` variant. `src/bytecode/decode.rs:19`.

---

### `mk_codec::bytecode::header` (`src/bytecode/header.rs`)

#### Types

- `pub struct BytecodeHeader` — `#[derive(Debug, Clone, Copy, PartialEq, Eq)]`. Parsed mk1 bytecode header. `src/bytecode/header.rs:30`.
  - `pub version: u8` — Version field, range 0..=15.
  - `pub fingerprint_flag: bool` — Bit 2: when `true`, `origin_fingerprint` is present.

#### Methods

- `pub fn BytecodeHeader::parse(byte: u8) -> Result<Self>` — Parse a single byte. Rejects unknown versions (`Error::UnsupportedVersion`) and any reserved bits set (`Error::ReservedBitsSet`). `src/bytecode/header.rs:40`.
- `pub fn BytecodeHeader::to_byte(self) -> u8` — Serialize the header to its single-byte wire form. `src/bytecode/header.rs:55`.

Note: `BytecodeHeader` is *not* `#[non_exhaustive]` (compare `KeyCard`, `Error`, `StringLayerHeader`, `CorrectionResult`, `DecodedString`, `ChunkFragment`, which all are). See Notes section.

---

### `mk_codec::bytecode::path` (`src/bytecode/path.rs`)

#### Constants

- `pub const EXPLICIT_PATH_INDICATOR: u8 = 0xFE` — Indicator byte for an explicit (non-standard-table) path. `src/bytecode/path.rs:28`.
- `pub const STANDARD_PATHS: &[(u8, &str)]` — Standard-table dictionary entries: 14 entries (7 mainnet `0x01`..=`0x07`, 7 testnet `0x11`..=`0x17`). `0x16` (BIP 48 testnet nested-segwit multisig, `m/48'/1'/0'/1'`) was added in mk-codec v0.2.0. `src/bytecode/path.rs:38`.

#### Functions

- `pub fn lookup_indicator(indicator: u8) -> Option<DerivationPath>` — Look up a standard-table indicator → `DerivationPath`. Returns `None` for reserved values (`0x00`, `0x08..=0x10`, `0x18..=0xFD`, `0xFF`). `src/bytecode/path.rs:60`.
- `pub fn lookup_path(path: &DerivationPath) -> Option<u8>` — Look up `DerivationPath` → standard-table indicator. Returns `None` if not in the dictionary (encoder falls through to explicit-path). Structural comparison. `src/bytecode/path.rs:72`.
- `pub fn encode_path(path: &DerivationPath) -> Vec<u8>` — Encode a path: 1-byte standard-table indicator if available, else explicit-path escape hatch (`0xFE` + count + LEB128 components). `src/bytecode/path.rs:85`.
- `pub fn decode_path(cursor: &mut &[u8]) -> Result<DerivationPath>` — Decode a path field starting at `*cursor` (advances). `src/bytecode/path.rs:101`. Emits `Error::InvalidPathIndicator` / `Error::PathTooDeep` / `Error::InvalidPathComponent` / `Error::UnexpectedEnd`.

---

### `mk_codec::bytecode::xpub_compact` (`src/bytecode/xpub_compact.rs`)

#### Types

- `pub struct XpubCompact` — `#[derive(Debug, Clone, PartialEq, Eq)]`. 73-byte compact form (closure Q-7). `src/bytecode/xpub_compact.rs:32`.
  - `pub version: [u8; 4]` — 4-byte BIP 32 version prefix.
  - `pub parent_fingerprint: [u8; 4]` — 4-byte parent-key fingerprint.
  - `pub chain_code: [u8; 32]` — 32-byte BIP 32 chain code.
  - `pub public_key: [u8; 33]` — 33-byte compressed secp256k1 public key.

Note: `XpubCompact` is *not* `#[non_exhaustive]`. See Notes section.

#### Methods / Functions

- `pub fn XpubCompact::from_xpub(xpub: &Xpub) -> Self` — Build a compact form from a full BIP 32 `Xpub`. `src/bytecode/xpub_compact.rs:45`.
- `pub fn reconstruct_xpub(compact: &XpubCompact, origin_path: &DerivationPath) -> Result<Xpub>` — Reconstruct a full BIP 32 `Xpub` from a compact form + origin path. `depth := component_count(origin_path)`, `child_number := last_component(origin_path)`. `src/bytecode/xpub_compact.rs:85`. Emits `Error::InvalidXpubVersion` / `Error::InvalidXpubPublicKey`. Internal `expect("origin_path must be non-empty per SPEC §3.5")` on empty path (callers must enforce).
- `pub fn encode_xpub_compact(compact: &XpubCompact, out: &mut Vec<u8>)` — Encode a compact form to its 73-byte wire layout. `src/bytecode/xpub_compact.rs:109`.
- `pub fn decode_xpub_compact(cursor: &mut &[u8]) -> Result<XpubCompact>` — Decode 73 bytes into a compact form. `src/bytecode/xpub_compact.rs:117`. Emits `Error::UnexpectedEnd` / `Error::InvalidXpubVersion`.

---

### `mk_codec::string_layer` (`src/string_layer/mod.rs`)

#### Re-exports (`pub use`)

- From `bch` (`src/string_layer/mod.rs:29-34`): `ALPHABET`, `BchCode`, `CaseStatus`, `CorrectionResult`, `DecodedString`, `SEPARATOR`, `bch_correct_long`, `bch_correct_regular`, `bch_create_checksum_long`, `bch_create_checksum_regular`, `bch_verify_long`, `bch_verify_regular`, `bytes_to_5bit`, `case_check`, `decode_string`, `encode_5bit_to_string`, `five_bit_to_bytes`, `hrp_expand`.
- From `chunk` (`src/string_layer/mod.rs:35`): `reassemble_from_chunks`, `split_into_chunks`.
- From `header` (`src/string_layer/mod.rs:36`): `StringLayerHeader`.
- From `pipeline` (`src/string_layer/mod.rs:37`): `decode`, `encode`, `encode_with_chunk_set_id`.

#### Sub-modules

`pub mod bch;` `pub mod chunk;` `pub mod header;` (`src/string_layer/mod.rs:22, 24, 25`).
`pub(crate) mod bch_decode;` and `mod pipeline;` are internal.

---

### `mk_codec::string_layer::bch` (`src/string_layer/bch.rs`)

#### Constants

- `pub const ALPHABET: &[u8; 32]` — bech32 32-character alphabet in 5-bit-value order: `b"qpzry9x8gf2tvdw0s3jn54khce6mua7l"`. `src/string_layer/bch.rs:39`.
- `pub const SEPARATOR: char = '1'` — bech32 separator character between HRP and data-part (BIP 173 §3). `src/string_layer/bch.rs:109`.
- `pub const GEN_REGULAR: [u128; 5]` — BCH polymod constants for regular checksum (BCH(93,80,8)) from BIP 93. `src/string_layer/bch.rs:173`.
- `pub const POLYMOD_INIT: u128 = 0x23181b3` — Initial residue value for both polymod algorithms (BIP 93). `src/string_layer/bch.rs:185`.
- `pub const REGULAR_SHIFT: u32 = 60` — Right-shift to extract top 5 bits from 65-bit regular-code residue. `src/string_layer/bch.rs:191`.
- `pub const REGULAR_MASK: u128 = 0x0fffffffffffffff` — Mask preserving low 60 bits of 65-bit regular-code residue. `src/string_layer/bch.rs:194`.
- `pub const GEN_LONG: [u128; 5]` — BCH polymod constants for long checksum (BCH(108,93,8)) from BIP 93. `src/string_layer/bch.rs:203`.
- `pub const LONG_SHIFT: u32 = 70` — Right-shift to extract top 5 bits from 75-bit long-code residue. `src/string_layer/bch.rs:215`.
- `pub const LONG_MASK: u128 = 0x3fffffffffffffffff` — Mask preserving low 70 bits of 75-bit long-code residue. `src/string_layer/bch.rs:218`.

#### Types

- `pub enum BchCode` — `#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]`. Which BCH code variant a string uses. Variants: `Regular` (BCH(93,80,8), 13-char checksum), `Long` (BCH(108,93,8), 15-char checksum). `src/string_layer/bch.rs:27`.
- `pub enum CaseStatus` — `#[derive(Debug, Clone, Copy, PartialEq, Eq)]`. Result of a case check. Variants: `Lower` (all-lowercase or no letters), `Upper` (all-uppercase), `Mixed` (both — invalid). `src/string_layer/bch.rs:155`.
- `pub struct CorrectionResult` — `#[non_exhaustive] #[derive(Debug, Clone, PartialEq, Eq)]`. Result of a successful BCH decode+correct. `src/string_layer/bch.rs:364`.
  - `pub data: Vec<u8>` — Corrected `data_with_checksum` slice.
  - `pub corrections_applied: usize` — Number of substitutions applied (0 = clean).
  - `pub corrected_positions: Vec<usize>` — Indices into `data` of substituted positions.
- `pub struct DecodedString` — `#[non_exhaustive] #[derive(Debug, Clone, PartialEq, Eq)]`. Result of a successful mk1 string decode at the BCH layer. `src/string_layer/bch.rs:570`.
  - `pub code: BchCode` — Detected BCH code variant.
  - `pub corrections_applied: usize` — Number of substitution errors corrected.
  - `pub corrected_positions: Vec<usize>` — Indices into data-part of corrected positions.
  - `pub data_with_checksum: Vec<u8>` — Full post-correction 5-bit symbol sequence (data part + checksum).

#### Methods

- `pub fn DecodedString::data(&self) -> &[u8]` — Data part as 5-bit values with trailing checksum stripped. `src/string_layer/bch.rs:594`.
- `pub fn DecodedString::corrected_char_at(&self, char_position: usize) -> char` — Look up corrected bech32 character at given position in the data part. Panics if position ≥ `data_with_checksum.len()`. `src/string_layer/bch.rs:622`.

#### Functions

- `pub fn bytes_to_5bit(bytes: &[u8]) -> Vec<u8>` — Convert 8-bit bytes to 5-bit values (zero-padded). `src/string_layer/bch.rs:56`.
- `pub fn five_bit_to_bytes(values: &[u8]) -> Option<Vec<u8>>` — Convert 5-bit values back to bytes. Returns `None` for out-of-range values or nonzero pad bits. `src/string_layer/bch.rs:78`.
- `pub fn bch_code_for_length(data_part_len: usize) -> Option<BchCode>` — Determine `BchCode` variant from total data-part length per BIP 93. `Regular` for 14..=93, `Long` for 96..=108, `None` for 94..=95 reserved-invalid gap or out-of-range. `src/string_layer/bch.rs:117`.
- `pub fn case_check(s: &str) -> CaseStatus` — Check whether a string is all-lowercase, all-uppercase, or mixed. `src/string_layer/bch.rs:132`.
- `pub fn hrp_expand(hrp: &str) -> Vec<u8>` — BIP 173-style HRP-expansion producing the 5-bit-symbol prelude. Length is `2*hrp.len() + 1` for ASCII HRPs. `src/string_layer/bch.rs:252`.
- `pub fn bch_create_checksum_regular(hrp: &str, data: &[u8]) -> [u8; 13]` — Compute the 13-character BCH checksum for the regular code. `src/string_layer/bch.rs:294`.
- `pub fn bch_verify_regular(hrp: &str, data_with_checksum: &[u8]) -> bool` — Verify a regular-code BCH checksum. `src/string_layer/bch.rs:312`.
- `pub fn bch_create_checksum_long(hrp: &str, data: &[u8]) -> [u8; 15]` — Compute the 15-character BCH checksum for the long code. `src/string_layer/bch.rs:326`.
- `pub fn bch_verify_long(hrp: &str, data_with_checksum: &[u8]) -> bool` — Verify a long-code BCH checksum. `src/string_layer/bch.rs:343`.
- `pub fn bch_correct_regular(hrp: &str, data_with_checksum: &[u8]) -> Result<CorrectionResult, crate::Error>` — Attempt to correct a regular-code BCH-checksummed string with up to four substitutions (full `t = 4` capacity of BCH(93, 80, 8)). Syndrome-based BM/Forney decoder; re-verifies post-correction. `src/string_layer/bch.rs:392`. Emits `Error::BchUncorrectable`.
- `pub fn bch_correct_long(hrp: &str, data_with_checksum: &[u8]) -> Result<CorrectionResult, crate::Error>` — Long-code analog. `src/string_layer/bch.rs:450`.
- `pub fn encode_5bit_to_string(data_5bit: &[u8]) -> Result<String, crate::Error>` — Encode a 5-bit-symbol data stream as a complete mk1 string. Auto-selects regular/long BCH code from resulting data-part length. Returns full string starting with `crate::consts::HRP` + BIP 173 separator (`"mk1"`). `src/string_layer/bch.rs:515`. Emits `Error::InvalidStringLength`.
- `pub fn decode_string(s: &str) -> Result<DecodedString, crate::Error>` — Decode an mk1 string with full BCH error correction up to `t = 4`. `src/string_layer/bch.rs:648`. Emits `Error::MixedCase`, `Error::InvalidHrp`, `Error::InvalidStringLength`, `Error::InvalidChar`, `Error::BchUncorrectable`.

---

### `mk_codec::string_layer::header` (`src/string_layer/header.rs`)

#### Constants

- `pub const SINGLE_HEADER_SYMBOLS: usize = 2` — Number of 5-bit symbols in the single-string header. `src/string_layer/header.rs:20`.
- `pub const CHUNKED_HEADER_SYMBOLS: usize = 8` — Number of 5-bit symbols in the chunked header. `src/string_layer/header.rs:24`.
- `pub const MAX_CHUNK_SET_ID: u32 = (1 << 20) - 1` — Maximum allowed value of `chunk_set_id` (20-bit field). `src/string_layer/header.rs:27`.
- `pub const VERSION_V0_1: u8 = 0x00` — Format-version field value emitted in v0.1. `src/string_layer/header.rs:30`.

#### Types

- `pub enum StringLayerHeader` — `#[non_exhaustive] #[derive(Debug, Clone, Copy, PartialEq, Eq)]`. String-layer header for one mk1 chunk. `src/string_layer/header.rs:35`. Variants:
  - `SingleString { version: u8 }` — Card fits in one mk1 string; no chunking.
  - `Chunked { version: u8, chunk_set_id: u32, total_chunks: u8, chunk_index: u8 }` — One chunk in a multi-chunk encoding. `total_chunks` is in `1..=MAX_CHUNKS`; wire encoding is `count - 1` (off-by-one to fit 5-bit field).

#### Methods

- `pub fn StringLayerHeader::to_5bit_symbols(self) -> Vec<u8>` — Emit this header as a sequence of 5-bit symbols (2 or 8). `src/string_layer/header.rs:67`.
- `pub fn StringLayerHeader::from_5bit_symbols(symbols: &[u8]) -> Result<(Self, usize)>` — Parse a header off the front of a 5-bit-symbol stream. Returns parsed header and consumed-symbol count. `src/string_layer/header.rs:120`. Emits `Error::UnexpectedEnd`, `Error::UnsupportedVersion`, `Error::UnsupportedCardType`, `Error::ChunkedHeaderMalformed`.
- `pub fn StringLayerHeader::is_chunked(self) -> bool` — Returns `true` if this header is the `Chunked` variant. `src/string_layer/header.rs:179`.

---

### `mk_codec::string_layer::chunk` (`src/string_layer/chunk.rs`)

#### Constants

- `pub const MAX_CHUNKABLE_BYTECODE: usize = (MAX_CHUNKS as usize) * CHUNKED_FRAGMENT_LONG_BYTES - CROSS_CHUNK_HASH_BYTES` — Maximum canonical-bytecode length that can be chunked under v0.1 (= 32*53 - 4 = 1692 bytes). `src/string_layer/chunk.rs:21`.

#### Types

- `pub struct ChunkFragment` — `#[non_exhaustive] #[derive(Debug, Clone, PartialEq, Eq)]`. One chunk's worth of split output: a parsed header + its fragment bytes. `src/string_layer/chunk.rs:27`.
  - `pub header: StringLayerHeader` — The string-layer header that prefixes this chunk on the wire.
  - `pub fragment: Vec<u8>` — The raw fragment payload bytes for this chunk.

#### Functions

- `pub fn split_into_chunks(canonical_bytecode: &[u8], chunk_set_id: u32) -> Result<Vec<ChunkFragment>>` — Split canonical bytecode into chunks, appending cross-chunk integrity hash. Byte-deterministic in `(canonical_bytecode, chunk_set_id)`. `src/string_layer/chunk.rs:50`. Emits `Error::ChunkedHeaderMalformed` (chunk_set_id > 20 bits), `Error::CardPayloadTooLarge`.
- `pub fn reassemble_from_chunks(chunks: Vec<ChunkFragment>) -> Result<Vec<u8>>` — Reassemble canonical bytecode from parsed chunks. Validates SPEC §4 rules 11-13 + cross-chunk hash. Accepts chunks in any order (sorts internally). `src/string_layer/chunk.rs:109`. Emits `Error::ChunkedHeaderMalformed`, `Error::ChunkSetIdMismatch`, `Error::MixedHeaderTypes`, `Error::CrossChunkHashMismatch`.

---

### `mk_codec::string_layer` — pipeline-level entry points (re-exported from private `string_layer::pipeline`)

These are the public encode/decode boundary; also re-exported at `mk_codec::key_card` and at the crate root.

- `pub fn encode(card: &KeyCard) -> Result<Vec<String>>` — Encode a `KeyCard` into one or more `mk1`-prefixed strings. Multi-chunk path draws `chunk_set_id` from `getrandom::getrandom` (OS CSPRNG); panics with "OS CSPRNG must be available for mk1 encode" if entropy read fails. `src/string_layer/pipeline.rs:56`.
- `pub fn encode_with_chunk_set_id(card: &KeyCard, chunk_set_id: u32) -> Result<Vec<String>>` — Like `encode` with explicit `chunk_set_id` override. `src/string_layer/pipeline.rs:67`. Emits `Error::ChunkedHeaderMalformed` if value exceeds 20 bits.
- `pub fn decode(strings: &[&str]) -> Result<KeyCard>` — Decode one or more `mk1`-prefixed strings into a `KeyCard`. Supports both single-string and chunked inputs; rejects mixing with `Error::MixedHeaderTypes`. `src/string_layer/pipeline.rs:118`. Empty input list returns `Error::ChunkedHeaderMalformed`.

---

## Error taxonomy

22 variants on `Error` (all in `src/error.rs:20`; `#[non_exhaustive]`). "Emitted by" lists non-test call sites in `src/` only.

| Variant | Doc-comment (first line) | Emitted by |
|---|---|---|
| `InvalidHrp(String)` | HRP is not `mk` or input is not a valid bech32-shaped string. | `string_layer::bch::decode_string` (`bch.rs:658`, `bch.rs:663`) |
| `MixedCase` | Input string mixes ASCII upper- and lower-case in its data part. | `string_layer::bch::decode_string` (`bch.rs:652`) |
| `InvalidStringLength(usize)` | Input string's data-part length is not a valid mk1 length. | `string_layer::bch::encode_5bit_to_string` (`bch.rs:532`); `string_layer::bch::decode_string` (`bch.rs:667`) |
| `InvalidChar { ch, position }` | Input string's data part contains a character not in bech32 alphabet. | `string_layer::bch::decode_string` (`bch.rs:672`, `bch.rs:676`) |
| `BchUncorrectable(String)` | BCH checksum could not be corrected within per-code-variant substitution capacity. | `string_layer::bch::bch_correct_regular` (`bch.rs:424`, `bch.rs:440`); `string_layer::bch::bch_correct_long` (`bch.rs:478`, `bch.rs:493`) |
| `UnsupportedCardType(u8)` | Chunk-header card-type byte not in {0x00 SingleString, 0x01 Chunked}. | `string_layer::header::StringLayerHeader::from_5bit_symbols` (`header.rs:174`) |
| `MalformedPayloadPadding` | 5-bit payload symbols don't byte-align after BCH verification. | `string_layer::pipeline::decode` (`pipeline.rs:132`) |
| `ChunkSetIdMismatch` | Chunks have inconsistent `chunk_set_id` values. | `string_layer::chunk::reassemble_from_chunks` (`chunk.rs:150`) |
| `ChunkedHeaderMalformed(String)` | Chunked-header malformed (bad total_chunks, chunk_index, gaps, duplicates, oversized chunk_set_id, empty input list). | `string_layer::header::StringLayerHeader::from_5bit_symbols` (`header.rs:155`, `header.rs:160`); `string_layer::chunk::split_into_chunks` (`chunk.rs:55`); `string_layer::chunk::reassemble_from_chunks` (`chunk.rs:111`, `chunk.rs:124`, `chunk.rs:132`, `chunk.rs:153`, `chunk.rs:159`, `chunk.rs:164`, `chunk.rs:185`, `chunk.rs:191`); `string_layer::pipeline::encode_bytecode_stream` (`pipeline.rs:88`); `string_layer::pipeline::decode` (`pipeline.rs:120`) |
| `MixedHeaderTypes` | Multi-string input mixes `SingleString` and `Chunked` headers. | `string_layer::chunk::reassemble_from_chunks` (`chunk.rs:176`); `string_layer::pipeline::decode` (`pipeline.rs:139`) |
| `CrossChunkHashMismatch` | Reassembled bytecode's trailing 4-byte `cross_chunk_hash` doesn't match `SHA-256(canonical_bytecode)[0..4]`. | `string_layer::chunk::reassemble_from_chunks` (`chunk.rs:200`) |
| `UnsupportedVersion(u8)` | Bytecode-header version != 0 in v0.1. | `bytecode::header::BytecodeHeader::parse` (`header.rs:43`); `string_layer::header::StringLayerHeader::from_5bit_symbols` (`header.rs:126`) |
| `ReservedBitsSet` | A reserved bit in the bytecode header was set. | `bytecode::header::BytecodeHeader::parse` (`header.rs:46`) |
| `InvalidPolicyIdStubCount` | `policy_id_stub_count == 0` (spec requires ≥ 1). | `bytecode::encode::encode_bytecode` (`encode.rs:23`, `encode.rs:26`); `bytecode::decode::decode_bytecode` (`decode.rs:27`) |
| `InvalidPathIndicator(u8)` | Origin-path indicator byte is outside the standard table or in the reserved range. | `bytecode::path::decode_path` (`path.rs:109`) |
| `PathTooDeep(u8)` | Explicit path declared `component_count > MAX_PATH_COMPONENTS`. | `bytecode::path::decode_path` → `decode_explicit_path` (`path.rs:115`) |
| `InvalidPathComponent(String)` | A path component's encoded value is invalid (BIP 32 range or hardened-bit issue). | `bytecode::path::decode_path` (`path.rs:122`, `path.rs:125`, `path.rs:158`, `path.rs:164`) |
| `InvalidXpubVersion(u32)` | xpub `version` field doesn't match a known network's xpub prefix. | `bytecode::xpub_compact::version_to_network` (`xpub_compact.rs:67`) |
| `InvalidXpubPublicKey(String)` | xpub `public_key` bytes do not parse as a valid compressed secp256k1 point. | `bytecode::xpub_compact::reconstruct_xpub` (`xpub_compact.rs:97`) |
| `UnexpectedEnd` | Decoder hit end-of-stream mid-field. | `bytecode::decode::decode_bytecode` → `read_u8` / `read_array` (`decode.rs:60`, `decode.rs:69`); `bytecode::path::decode_path` → `read_u8` (`path.rs:173`); `bytecode::xpub_compact::decode_xpub_compact` (`xpub_compact.rs:119`); `string_layer::header::StringLayerHeader::from_5bit_symbols` (`header.rs:122`, `header.rs:136`) |
| `TrailingBytes` | Decoder finished consuming all expected fields but bytes remain. | `bytecode::decode::decode_bytecode` (`decode.rs:47`) |
| `CardPayloadTooLarge { bytecode_len, max_supported }` | Canonical bytecode + cross-chunk hash exceeds v0.1 capacity (32 × 53 − 4 = 1692 bytes). | `string_layer::chunk::split_into_chunks` (`chunk.rs:60`) |

## Feature-gated items

| Item | Feature | Path |
|---|---|---|
| `gen_mk_vectors` binary | `gen-vectors` (`required-features`) | `src/bin/gen_mk_vectors.rs` |
| `serde_json` dependency | `gen-vectors` (`dep:serde_json`) | Cargo.toml `[dependencies]` |

No `#[cfg(feature = ...)]` items in the library `src/` tree — feature gating is binary-target-only.

## Notes for chapter author (Phase 4.2)

- **`#[non_exhaustive]` policy is inconsistent.** `KeyCard`, `Error`, `StringLayerHeader`, `CorrectionResult`, `DecodedString`, `ChunkFragment` are `#[non_exhaustive]`; `BytecodeHeader` and `XpubCompact` are not. The latter two would prevent external struct-literal construction breakage on field addition. Not necessarily a defect — header / compact-xpub fields are wire-locked at v0.1 — but worth surfacing as a stability-API note.
- **`reconstruct_xpub` panics on empty `origin_path`** via `.expect("origin_path must be non-empty per SPEC §3.5")` (`xpub_compact.rs:95`). Doc-comment says "caller responsibility; the spec guarantees this since standard-table indicators have ≥3 components and explicit-path encoding requires `count ≥ 1`." External callers using `XpubCompact` directly (outside the layered decoder pipeline) MUST validate path non-emptiness; the chapter should call this out.
- **`encode` / `encode_with_chunk_set_id` panic on entropy failure.** `string_layer::pipeline::fresh_chunk_set_id` panics with "OS CSPRNG must be available for mk1 encode" if `getrandom::getrandom` fails (`pipeline.rs:47`). Documented behaviour, but worth noting as a non-`Result` failure mode in encode-path callers.
- **`DecodedString::corrected_char_at` panics on out-of-range position** (>= `data_with_checksum.len()`). Documented as a `# Panics` clause in the doc-comment.
- **HRP-mixed BCH is forked from BIP 93, not shared.** Per closure D-13 and the lib.rs preamble, `string_layer::bch` is a *fork* of `md-codec`'s BCH code with mk1-specific HRP (`"mk"`) and NUMS target residues (`MK_REGULAR_CONST` / `MK_LONG_CONST`). The plan for a shared `mc-codex32` crate was retired 2026-05-03 per the toolkit CLAUDE.md cross-repo note; mk1 BCH stays forked indefinitely. The public surface exposes `GEN_REGULAR`, `GEN_LONG`, `POLYMOD_INIT`, `REGULAR_SHIFT`/`MASK`, `LONG_SHIFT`/`MASK`, `hrp_expand`, `bch_create_checksum_*`, `bch_verify_*`, `bch_correct_*`, `bch_code_for_length` — the full BCH primitive surface is public, so callers could in principle build mk-codec-compatible string-layer codecs without going through `KeyCard`.
- **Path dictionary is mk1-internal (standalone), no longer mirrored.** Per `bytecode/path.rs:8-16`, md1 v0.11+ dropped path dictionaries entirely (`design/SPEC_v0_11_wire_format.md` §1.4); the v0.10.x-era mirror invariant `path-dictionary-mirror-stewardship` is retired. **mk-codec's `STANDARD_PATHS` constant (`bytecode/path.rs:38`, 14 entries; mainnet `0x01`..=`0x07`, testnet `0x11`..=`0x17`) is therefore standalone, not a sibling-mirror.** `0x16` was added in mk-codec v0.2.0 (wire-additive — pre-v0.2 decoders reject as `Error::InvalidPathIndicator(0x16)`). The public functions `lookup_indicator`, `lookup_path`, `encode_path`, `decode_path`, and the `EXPLICIT_PATH_INDICATOR = 0xFE` const all touch the path-dictionary encoding/decoding surface; flag for the chapter that these are mk1-only.
- **`bech32` dependency is declared but not used in public signatures.** `Cargo.toml:27` declares `bech32 = "0.11"` but a `pub`-level grep across `src/` finds no `bech32::*` type in any public signature. The crate uses its own bech32-alphabet primitives (`ALPHABET`, `bytes_to_5bit`, etc.). Verify whether `bech32` is actually used internally or could be dropped — but per the CLAUDE.md note on reserved-deps, do not drop without user confirmation. (md-codec uses a similar pattern.)
- **`cargo doc` warnings (3) are documentation-only and pre-existing:**
  - Private intra-doc link to `bch_decode` from `string_layer/mod.rs:11` (the module is `pub(crate)`).
  - Unresolved link `Error::InvalidStringLength` from `string_layer/bch.rs:506` (the linked item exists in `crate::error::Error::InvalidStringLength` but the doc-comment path is missing the `crate::` prefix).
  - Unresolved link `crate::Correction::corrected` from `string_layer/bch.rs:610` (refers to md-codec's `Correction.corrected` field; in mk-codec the equivalent is `DecodedString::corrected_char_at`; doc-comment is a copy from md-codec and not adapted).
- **Visible `"md1"` strings in `DecodedString` doc-comments — stale copy-from-md-codec.** `string_layer/bch.rs:575` reads `Indices into the data-part (chars after "md1") of any corrected positions.`; `bch.rs:603` reads `data part (chars after the "md1" HRP+separator).`. mk1 strings use HRP `"mk"` / prefix `"mk1"` (`consts.rs:9: pub const HRP: &str = "mk"`). Chapter 4.2 draft must substitute `"mk1"` when quoting either doc-comment. Source fix is outside harvest scope (candidate for mid-cycle mk1 cross-repo FOLLOWUP).
- **Const naming convention drift:** `GENERATOR_FAMILY = "mk-codec 0.2"` (`consts.rs:50`) embeds the family-stable minor version. The string itself only rolls on minor/major bumps (not patch), so v0.2.0 → v0.2.2 keeps the same token — verified consistent.
- **`KeyCard::new` is the only public constructor** but is intentionally permissive — invariants (`policy_id_stubs` non-empty, path depth ≤ `MAX_PATH_COMPONENTS`) are enforced at encode time, not construction time. Chapter should highlight that round-trip-safety guarantees only apply post-`encode` round-trip; raw `KeyCard::new` accepts invalid inputs that the encoder will reject.
- **`Error` is `#[non_exhaustive]`** — external callers' exhaustive `match` arms must include a `_ =>` arm. This is standard for the family but worth a callout in API stability notes.
