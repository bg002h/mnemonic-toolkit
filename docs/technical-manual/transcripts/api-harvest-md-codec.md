# md-codec API surface harvest

| Field | Value |
|---|---|
| Crate | `md-codec` (library name: `md_codec`) |
| Version | 0.32.0 |
| Source root | `/scratch/code/shibboleth/descriptor-mnemonic/crates/md-codec` |
| HEAD commit | `df1ed24b77cd7def3b9ffdd1aefa6d0566ab511b` |
| Rust edition | 2024 (inherited from workspace) |
| MSRV | `rust-version = "1.85"` (inherited from workspace) |
| License | MIT (workspace) |
| `cargo doc --no-deps --all-features -p md-codec` | builds clean (no warnings/errors) |

## Feature flags

(from `Cargo.toml:21-23`)

| Flag | Default | Gates | Implied deps |
|---|---|---|---|
| `derive` | yes (in `default = ["derive"]`) | `pub mod to_miniscript`, `derive::xpub_from_tlv_bytes`, `Descriptor::derive_address` | `dep:miniscript` |

No other features defined. `default = ["derive"]`. Pure-codec consumers opt out via `default-features = false`.

## Dependencies (public-facing only — types/traits appearing in public signatures)

(from `Cargo.toml:25-29` plus walked `pub fn` signatures)

- **`bitcoin`** (workspace pin) — surfaced in:
  - `derive::xpub_from_tlv_bytes(...) -> Result<bitcoin::bip32::Xpub, Error>` (feature-gated `derive`; `pub(crate)`, not in public API but visible through type aliasing)
  - `Descriptor::derive_address(..., network: bitcoin::Network) -> Result<bitcoin::Address<bitcoin::address::NetworkUnchecked>, Error>` (feature-gated `derive`)
- **`miniscript`** (workspace pin, optional, gated by `derive` feature) — surfaced in:
  - `to_miniscript::to_miniscript_descriptor(...) -> Result<miniscript::Descriptor<miniscript::descriptor::DescriptorPublicKey>, Error>`
- **`thiserror = "2.0"`** — used internally to derive `Error`; surfaces only via `#[derive(Error)]` (`std::error::Error` impl auto-derived; `Display` via `#[error("...")]`).
- **`bip39`** (workspace pin) — internal only (`Phrase::from_id_bytes` calls `bip39::Mnemonic::from_entropy`), no `bip39` types in the public API. `Phrase` wraps `[String; 12]`.

No `pub use` re-exports of foreign types.

## Public modules (top-level)

(from `src/lib.rs:15-37`)

20 public modules (19 unconditional; `to_miniscript` requires `derive` feature) + one private (`mod bch`).

| Module | Path | Feature gate |
|---|---|---|
| `bitstream` | `src/bitstream.rs` | — |
| `canonical_origin` | `src/canonical_origin.rs` | — |
| `canonicalize` | `src/canonicalize.rs` | — |
| `chunk` | `src/chunk.rs` | — |
| `codex32` | `src/codex32.rs` | — |
| `decode` | `src/decode.rs` | — |
| `derive` | `src/derive.rs` | — (module declaration unconditional; entire body `#[cfg(feature = "derive")]`) |
| `encode` | `src/encode.rs` | — |
| `error` | `src/error.rs` | — |
| `header` | `src/header.rs` | — |
| `identity` | `src/identity.rs` | — |
| `origin_path` | `src/origin_path.rs` | — |
| `phrase` | `src/phrase.rs` | — |
| `tag` | `src/tag.rs` | — |
| `tlv` | `src/tlv.rs` | — |
| `to_miniscript` | `src/to_miniscript.rs` | `derive` (whole module `#[cfg(feature = "derive")]`) |
| `tree` | `src/tree.rs` | — |
| `use_site_path` | `src/use_site_path.rs` | — |
| `validate` | `src/validate.rs` | — |
| `varint` | `src/varint.rs` | — |

(`mod bch` at `src/lib.rs:15` is private — out of scope.)

## Public surface by module

### `md_codec` (crate root, `src/lib.rs`)

#### Re-exports (`pub use`)

(`src/lib.rs:39-52`)

- `pub use canonicalize::canonicalize_placeholder_indices;` (line 39)
- `pub use chunk::{ChunkHeader, derive_chunk_set_id, reassemble, split};` (line 40)
- `pub use decode::{decode_md1_string, decode_payload};` (line 41)
- `pub use encode::{Descriptor, encode_md1_string, encode_payload};` (line 42)
- `pub use error::Error;` (line 43)
- `pub use header::Header;` (line 44)
- `pub use identity::{Md1EncodingId, WalletDescriptorTemplateId, WalletPolicyId, compute_md1_encoding_id, compute_wallet_descriptor_template_id, compute_wallet_policy_id, validate_presence_byte};` (lines 45-48)
- `pub use origin_path::{OriginPath, PathComponent, PathDecl, PathDeclPaths};` (line 49)
- `pub use phrase::Phrase;` (line 50)
- `pub use tag::Tag;` (line 51)
- `pub use tlv::TlvSection;` (line 52)

No functions, types, traits, type aliases, or constants declared directly at the crate root.

### `md_codec::bitstream` (`src/bitstream.rs`)

Module-level doc: "Bit-aligned reader and writer. Per spec §4.6: bits are packed MSB-first into bytes…" (`bitstream.rs:1-5`).

#### Types

- `pub struct BitWriter { /* private fields */ }` (`bitstream.rs:11-16`) — MSB-first bit packer. Implements `Default`.
- `pub struct BitReader<'a> { /* private fields */ }` (`bitstream.rs:89-96`) — MSB-first bit unpacker over a borrowed byte slice.

#### Functions (free)

- `pub fn re_emit_bits(dst: &mut BitWriter, src_bytes: &[u8], bit_len: usize) -> Result<(), Error>` (`bitstream.rs:220`) — Read `bit_len` MSB-first bits from `src_bytes`, append to `dst`. Used by TLV re-encoder.

#### `impl BitWriter` (`bitstream.rs:18-84`)

- `pub fn new() -> Self` (line 20)
- `pub fn write_bits(&mut self, value: u64, count: usize)` (line 29) — write `count` bits from LSB-aligned `value` MSB-first
- `pub fn bit_len(&self) -> usize` (line 72) — total bits written
- `pub fn into_bytes(self) -> Vec<u8>` (line 81) — consume; final byte zero-padded

#### `impl<'a> BitReader<'a>` (`bitstream.rs:98-209`)

- `pub fn new(bytes: &'a [u8]) -> Self` (line 101) — bit_limit = `bytes.len() * 8`
- `pub fn with_bit_limit(bytes: &'a [u8], bit_limit: usize) -> Self` (line 113) — explicit limit
- `pub fn read_bits(&mut self, count: usize) -> Result<u64, Error>` (line 123) — LSB-aligned result
- `pub fn remaining_bits(&self) -> usize` (line 164)
- `pub fn is_exhausted(&self) -> bool` (line 169)
- `pub fn save_position(&self) -> usize` (line 176) — snapshot for rollback
- `pub fn restore_position(&mut self, saved: usize)` (line 181)

(Note: `bit_position`, `save_bit_limit`, `set_bit_limit_for_scope`, `restore_bit_limit` are `pub(crate)` — internal.)

#### Constants / type aliases / traits

None.

### `md_codec::canonical_origin` (`src/canonical_origin.rs`)

Module-level doc: "Canonical-origin map per spec §4 (v0.13 wallet-policy layer)…" with table mapping wrapper shape → BIP path.

#### Functions

- `pub fn canonical_origin(tree: &Node) -> Option<OriginPath>` (`canonical_origin.rs:45`) — Returns canonical BIP path from wrapper shape; `None` for shapes requiring explicit overrides. Table:
  - `pkh(@N)` → `m/44'/0'/0'`
  - `wpkh(@N)` → `m/84'/0'/0'`
  - `tr(@N)` keypath-only → `m/86'/0'/0'`
  - `wsh(multi|sortedmulti)` → `m/48'/0'/0'/2'`
  - `sh(wsh(multi|sortedmulti))` → `m/48'/0'/0'/1'`
  - else → `None`

#### Types / constants / traits

None.

(`is_wsh_inner_multi` at line 38 is `pub(crate)` — internal.)

### `md_codec::canonicalize` (`src/canonicalize.rs`)

Module-level doc: "BIP 388 placeholder-ordering canonicalization per spec v0.13 §6.1, plus per-`@N` canonical-fill expansion per §5.3 / §6.3."

#### Functions

- `pub fn canonicalize_placeholder_indices(d: &mut Descriptor) -> Result<(), Error>` (`canonicalize.rs:168`) — Reshape `d` in place so tree first-occurrence sequence is `[0, 1, …, n-1]`. Atomically permutes tree indices, divergent path decl, and all per-`@N` TLV maps. Idempotent.
- `pub fn expand_per_at_n(d: &Descriptor) -> Result<Vec<ExpandedKey>, Error>` (`canonicalize.rs:420`) — Resolve each `@N` into a fully-populated `ExpandedKey` by composing per-`@N` TLV overrides with descriptor-level baselines (origin path / use-site path / fp / xpub). Precondition: caller must have canonicalized indices (or `d` came from decoder).

#### Types

- `pub struct ExpandedKey { ... }` (`canonicalize.rs:337-350`) — Fields all `pub`:
  - `pub idx: u8` (line 340)
  - `pub origin_path: OriginPath` (line 342)
  - `pub use_site_path: UseSitePath` (line 344)
  - `pub fingerprint: Option<[u8; 4]>` (line 346)
  - `pub xpub: Option<[u8; 65]>` (line 349)

### `md_codec::chunk` (`src/chunk.rs`)

Module-level doc: "Chunk header per SPEC v0.30 §2.2. Encodes the 37-bit chunked wire-format header…"

#### Types

- `pub struct ChunkHeader { ... }` (`chunk.rs:13-23`) — 37-bit chunked wire header. Fields:
  - `pub version: u8` (line 16) — v0.30 = 4
  - `pub chunk_set_id: u32` (line 18) — 20-bit
  - `pub count: u8` (line 20) — 1..=64
  - `pub index: u8` (line 22) — `< count`

#### Functions (free)

- `pub fn derive_chunk_set_id(id: &Md1EncodingId) -> u32` (`chunk.rs:168`) — Top 20 bits of the 16-byte hash, MSB-first.
- `pub fn split(d: &Descriptor) -> Result<Vec<String>, Error>` (`chunk.rs:228`) — Split a `Descriptor` into N codex32 md1 strings, each carrying a chunk header + payload slice.
- `pub fn reassemble(strings: &[&str]) -> Result<Descriptor, Error>` (`chunk.rs:298`) — Reverse of `split`; unwrap each string, parse 37-bit chunk header, validate consistency, sort by index, decode reassembled payload.

#### `impl ChunkHeader` (`chunk.rs:25-79`)

- `pub fn write(&self, w: &mut BitWriter) -> Result<(), Error>` (line 29) — 37 bits
- `pub fn read(r: &mut BitReader) -> Result<Self, Error>` (line 60)

#### Constants

- `pub const SINGLE_STRING_PAYLOAD_BIT_LIMIT: usize = 64 * 5;` (`chunk.rs:212`) — Threshold above which chunking is required (codex32 regular form's 80-char data-part limit minus HRP/sep/checksum = 64 data symbols × 5 bits = 320 bits).

### `md_codec::codex32` (`src/codex32.rs`)

Module-level doc: "v0.11 ↔ codex32 BCH layer adapter, symbol-aligned per spec §3.1 / D7."

#### Functions

- `pub fn wrap_payload(payload_bytes: &[u8], bit_count: usize) -> Result<String, Error>` (`codex32.rs:67`) — Wrap a byte-padded payload bit stream into a complete codex32 md1 string (HRP `md` + payload + 13-symbol BCH checksum, symbol-aligned).
- `pub fn unwrap_string(s: &str) -> Result<(Vec<u8>, usize), Error>` (`codex32.rs:92`) — Returns `(byte-padded payload bytes, symbol_aligned_bit_count = 5 × data_symbol_count)`. Tolerates whitespace + `-` as visual separators.

#### Constants

- (`pub(crate) const REGULAR_CHECKSUM_SYMBOLS: usize = 13;` at line 18 is `pub(crate)` — internal.)

### `md_codec::decode` (`src/decode.rs`)

Module-level doc: "Top-level decoder per spec §13.2."

#### Functions

- `pub fn decode_payload(bytes: &[u8], total_bits: usize) -> Result<Descriptor, Error>` (`decode.rs:15`) — Decode a `Descriptor` from canonical payload bit stream. Validates header, path decl, use-site path, tree (with `MAX_DECODE_DEPTH=128` cap), TLV section. Decoder-side hardening: top-level wrapper allow-list `{Sh, Wsh, Wpkh, Pkh, Tr}` (else `Error::OperatorContextViolation { context: TopLevel }`). Runs `validate_placeholder_usage`, `validate_multipath_consistency`, `validate_tap_script_tree`, `validate_explicit_origin_required`, `validate_xpub_bytes`.
- `pub fn decode_md1_string(s: &str) -> Result<Descriptor, Error>` (`decode.rs:79`) — Decode from complete codex32 md1 string via `unwrap_string` + `decode_payload`.

### `md_codec::derive` (`src/derive.rs`) — feature `derive`

Module-level doc: "Address derivation (v0.32). v0.32 replaces the v0.14-era hand-rolled 5-shape allow-list with an AST → `miniscript::Descriptor` converter…"

#### Functions

None at module level. The `xpub_from_tlv_bytes` function at line 49 is `pub(crate)` — internal.

#### `impl Descriptor` (`derive.rs:66-133`) — feature-gated `derive`

- `pub fn derive_address(&self, chain: u32, index: u32, network: bitcoin::Network) -> Result<bitcoin::Address<bitcoin::address::NetworkUnchecked>, Error>` (`derive.rs:92`) — Derive address at `(chain, index)`. `chain` selects the use-site multipath alt (e.g., 0=receive, 1=change for `<0;1>/*`). `index` is the wildcard child number. Returns unchecked address; caller calls `.assume_checked()` or `.require_network(network)`. Errors per doc-comment: `MissingPubkey`, `InvalidXpubBytes`, `ChainIndexOutOfRange`, `HardenedPublicDerivation`, `MissingExplicitOrigin`, `AddressDerivationFailed`.

### `md_codec::encode` (`src/encode.rs`)

Module-level doc: "Top-level encoder per spec §13.3."

#### Types

- `pub struct Descriptor { ... }` (`encode.rs:16-28`) — Top-level descriptor parsed/built from a v0.30 wire payload. Fields all `pub`:
  - `pub n: u8` (line 19)
  - `pub path_decl: PathDecl` (line 21)
  - `pub use_site_path: UseSitePath` (line 23)
  - `pub tree: Node` (line 25)
  - `pub tlv: TlvSection` (line 27)

#### `impl Descriptor` (`encode.rs:30-53`)

- `pub fn key_index_width(&self) -> u8` (line 37) — `⌈log₂(n)⌉` per SPEC §7; clamped to 0 at n∈{0,1}.
- `pub fn is_wallet_policy(&self) -> bool` (line 50) — `true` iff `tlv.pubkeys` is `Some(v)` with `!v.is_empty()`.

(Plus `pub fn derive_address` from `derive` module — see §`derive` above.)

#### Functions (free)

- `pub fn encode_payload(d: &Descriptor) -> Result<(Vec<u8>, usize), Error>` (`encode.rs:65`) — Encode a Descriptor; returns `(bytes, total_bit_count)`. Internally clones `d`, canonicalizes placeholder indices, then runs `validate_placeholder_usage`, `validate_multipath_consistency`, `validate_tap_script_tree`.
- `pub fn render_codex32_grouped(s: &str, group_size: usize) -> String` (`encode.rs:98`) — Hyphen-grouped rendering; `group_size = 0` returns input unchanged.
- `pub fn encode_md1_string(d: &Descriptor) -> Result<String, Error>` (`encode.rs:114`) — `encode_payload` + `codex32::wrap_payload`.

### `md_codec::error` (`src/error.rs`)

Module-level doc: "Error variants for the md-codec wire-format codec."

#### Types

- `pub enum ContextKind { TopLevel, TapLeaf, MultiBody }` (`error.rs:9-16`) — Where in the descriptor tree an operator appears (per SPEC v0.30 §11). Derives `Debug, Clone, Copy, PartialEq, Eq`.
- `pub enum Error { ... }` (`error.rs:19-392`) — 43 variants; see "Error taxonomy" section below. Derives `Debug, Error, PartialEq, Eq` (`thiserror::Error`).

### `md_codec::header` (`src/header.rs`)

Module-level doc: "Single-payload header (5 bits) per SPEC v0.30 §2.1."

#### Types

- `pub struct Header { ... }` (`header.rs:16-22`) — Fields all `pub`:
  - `pub version: u8` (line 19) — 4 bits
  - `pub divergent_paths: bool` (line 21) — bit 4

#### `impl Header` (`header.rs:24-50`)

- `pub const WF_REDESIGN_VERSION: u8 = 4;` (line 27)
- `pub fn write(&self, w: &mut BitWriter)` (line 30)
- `pub fn read(r: &mut BitReader) -> Result<Self, Error>` (line 38) — rejects `version != WF_REDESIGN_VERSION` with `Error::WireVersionMismatch`.

### `md_codec::identity` (`src/identity.rs`)

Module-level doc: "Identity computation per spec §8."

#### Types

- `pub struct Md1EncodingId([u8; 16]);` (`identity.rs:15-16`) — Tuple-struct, inner field private. 128-bit canonical identifier for an md1 encoding (spec §8). Derives `Debug, Clone, Copy, PartialEq, Eq, Hash`.
- `pub struct WalletDescriptorTemplateId([u8; 16]);` (`identity.rs:54-55`) — Tuple-struct, inner field private. 128-bit BIP 388 wallet-descriptor-template identifier (spec §8.1, γ-flavor; hashes BIP 388 template content only).
- `pub struct WalletPolicyId([u8; 16]);` (`identity.rs:114-115`) — Tuple-struct, inner field private. 128-bit canonical wallet-policy identifier (spec v0.13 §5.3).

#### `impl Md1EncodingId` (`identity.rs:18-35`)

- `pub fn new(bytes: [u8; 16]) -> Self` (line 20)
- `pub fn as_bytes(&self) -> &[u8; 16]` (line 25)
- `pub fn fingerprint(&self) -> [u8; 4]` (line 30) — first 4 bytes of id

#### `impl WalletDescriptorTemplateId` (`identity.rs:57-67`)

- `pub fn new(bytes: [u8; 16]) -> Self` (line 59)
- `pub fn as_bytes(&self) -> &[u8; 16]` (line 64)

#### `impl WalletPolicyId` (`identity.rs:117-132`)

- `pub fn new(bytes: [u8; 16]) -> Self` (line 119)
- `pub fn as_bytes(&self) -> &[u8; 16]` (line 124)
- `pub fn to_phrase(&self) -> Result<Phrase, Error>` (line 129) — render as 12-word BIP-39 phrase (spec §8.4).

#### Functions (free)

- `pub fn compute_md1_encoding_id(d: &Descriptor) -> Result<Md1EncodingId, Error>` (`identity.rs:39`) — SHA-256 over canonical bit-packed payload bytes (truncated to 16 bytes).
- `pub fn compute_wallet_descriptor_template_id(d: &Descriptor) -> Result<WalletDescriptorTemplateId, Error>` (`identity.rs:71`) — SHA-256 over use-site-path bits ‖ tree bits ‖ optional `UseSitePathOverrides` TLV entry (truncated to 16 bytes).
- `pub fn compute_wallet_policy_id(d: &Descriptor) -> Result<WalletPolicyId, Error>` (`identity.rs:172`) — SHA-256 over canonical template tree ‖ per-`@N` canonical records (presence_byte ‖ varint(path_bits) ‖ path ‖ varint(use_site_bits) ‖ use_site ‖ optional fp ‖ optional xpub). Internally canonicalizes `d` and runs `expand_per_at_n`.
- `pub fn validate_presence_byte(byte: u8) -> Result<(), Error>` (`identity.rs:253`) — Reject `presence_byte` with non-zero reserved bits (bits 2..7). Returns `Error::InvalidPresenceByte`.

### `md_codec::origin_path` (`src/origin_path.rs`)

Module-level doc: "Origin-path-decl block per spec §3.4."

#### Types

- `pub struct PathComponent { pub hardened: bool, pub value: u32 }` (`origin_path.rs:18-24`) — Single BIP-32 path component (e.g., `84'` or `0`). `value` is u31 effective (LP4-ext varint).
- `pub struct OriginPath { pub components: Vec<PathComponent> }` (`origin_path.rs:46-50`) — Sequence of components root→leaf.
- `pub struct PathDecl { pub n: u8, pub paths: PathDeclPaths }` (`origin_path.rs:81-87`) — Path declaration: key count + paths variant.
- `pub enum PathDeclPaths { Shared(OriginPath), Divergent(Vec<OriginPath>) }` (`origin_path.rs:90-96`) — Variants:
  - `Shared(OriginPath)` — single path shared by all n keys (header bit 4 = 0)
  - `Divergent(Vec<OriginPath>)` — n distinct paths (header bit 4 = 1)

#### `impl PathComponent` (`origin_path.rs:26-40`)

- `pub fn write(&self, w: &mut BitWriter) -> Result<(), Error>` (line 28) — 1 hardened bit + LP4-ext varint
- `pub fn read(r: &mut BitReader) -> Result<Self, Error>` (line 35)

#### `impl OriginPath` (`origin_path.rs:52-77`)

- `pub fn write(&self, w: &mut BitWriter) -> Result<(), Error>` (line 54) — 4-bit depth + each component
- `pub fn read(r: &mut BitReader) -> Result<Self, Error>` (line 69)

#### `impl PathDecl` (`origin_path.rs:98-147`)

- `pub fn write(&self, w: &mut BitWriter) -> Result<(), Error>` (line 110) — caller responsible for header bit 4. Errors: `KeyCountOutOfRange`, `DivergentPathCountMismatch`, `PathDepthExceeded`.
- `pub fn read(r: &mut BitReader, divergent_mode: bool) -> Result<Self, Error>` (line 134)

#### Constants

- `pub const MAX_PATH_COMPONENTS: usize = 15;` (`origin_path.rs:43`) — 4-bit depth field max.

### `md_codec::phrase` (`src/phrase.rs`)

Module-level doc: "BIP-39 phrase rendering per spec §8.4."

#### Types

- `pub struct Phrase(pub [String; 12]);` (`phrase.rs:7-10`) — Tuple-struct; inner `[String; 12]` field is `pub`.

#### `impl Phrase` (`phrase.rs:12-26`)

- `pub fn from_id_bytes(id: &[u8; 16]) -> Result<Self, Error>` (line 17) — Render 16 bytes (128 bits) as 12 BIP-39 words via `bip39::Mnemonic::from_entropy`.

#### Trait impls

- `impl std::fmt::Display for Phrase` (`phrase.rs:28-32`) — joins words with single space.

### `md_codec::tag` (`src/tag.rs`)

Module-level doc: "v0.30 Tag enum per SPEC §3. 36 operators in primary 6-bit space (0x00..=0x23)."

#### Types

- `pub enum Tag { ... }` (`tag.rs:14-89`) — 36 variants; derives `Debug, Clone, Copy, PartialEq, Eq, Hash`. Variants (primary 6-bit codes, all `(primary, None)` — extension subspace `0x3F` reserved):

| Code | Variant | Meaning |
|---|---|---|
| `0x00` | `Wpkh` | `wpkh` — P2WPKH descriptor |
| `0x01` | `Tr` | `tr` — Taproot descriptor |
| `0x02` | `Wsh` | `wsh` — P2WSH descriptor |
| `0x03` | `Sh` | `sh` — P2SH descriptor |
| `0x04` | `Pkh` | `pkh` — P2PKH descriptor |
| `0x05` | `TapTree` | Taproot tree node |
| `0x06` | `Multi` | `multi` k-of-n |
| `0x07` | `SortedMulti` | `sortedmulti` |
| `0x08` | `MultiA` | `multi_a` (CHECKSIGADD) |
| `0x09` | `SortedMultiA` | `sortedmulti_a` |
| `0x0A` | `PkK` | miniscript `pk_k` |
| `0x0B` | `PkH` | miniscript `pk_h` |
| `0x0C` | `Check` | `c:` (CHECKSIG) |
| `0x0D` | `Verify` | `v:` (VERIFY) |
| `0x0E` | `Swap` | `s:` (SWAP) |
| `0x0F` | `Alt` | `a:` (TOALTSTACK) |
| `0x10` | `DupIf` | `d:` (DUPIF) |
| `0x11` | `NonZero` | `j:` (NONZERO) |
| `0x12` | `ZeroNotEqual` | `n:` (ZERONOTEQUAL) |
| `0x13` | `AndV` | `and_v` |
| `0x14` | `AndB` | `and_b` |
| `0x15` | `AndOr` | `andor` |
| `0x16` | `OrB` | `or_b` |
| `0x17` | `OrC` | `or_c` |
| `0x18` | `OrD` | `or_d` |
| `0x19` | `OrI` | `or_i` |
| `0x1A` | `Thresh` | `thresh` |
| `0x1B` | `After` | absolute timelock |
| `0x1C` | `Older` | relative timelock |
| `0x1D` | `Sha256` | `sha256` |
| `0x1E` | `Hash160` | `hash160` |
| `0x1F` | `Hash256` | `hash256` (v0.30: primary; v0.x: extension) |
| `0x20` | `Ripemd160` | `ripemd160` (v0.30: primary; v0.x: extension) |
| `0x21` | `RawPkH` | raw public-key hash (v0.30: primary; v0.x: extension) |
| `0x22` | `False` | `0` literal |
| `0x23` | `True` | `1` literal |

Primary range `0x24..=0x3E` reserved; primary `0x3F` is extension prefix with the 4-bit subspace `0x00..=0x0F` entirely reserved in v0.30.

#### `impl Tag` (`tag.rs:93-203`)

- `pub fn write(&self, w: &mut BitWriter)` (line 140) — 6 bits primary (+ 4 if extension; never reached in v0.30)
- `pub fn read(r: &mut BitReader) -> Result<Self, Error>` (line 156) — Consumes 6 bits (or 10 for extension); reserved/extension codes → `Error::TagOutOfRange { primary }`.

(`pub(crate) fn codes(&self) -> (u8, Option<u8>)` at line 98 is `pub(crate)` — internal.)

### `md_codec::tlv` (`src/tlv.rs`)

Module-level doc: "TLV section per spec §3.7 (extended in v0.13 §3.2 with `Pubkeys` and `OriginPathOverrides`)."

#### Constants

- `pub const TLV_USE_SITE_PATH_OVERRIDES: u8 = 0x00;` (`tlv.rs:11`)
- `pub const TLV_FINGERPRINTS: u8 = 0x01;` (`tlv.rs:13`)
- `pub const TLV_PUBKEYS: u8 = 0x02;` (`tlv.rs:16`)
- `pub const TLV_ORIGIN_PATH_OVERRIDES: u8 = 0x03;` (`tlv.rs:19`)

#### Types

- `pub struct TlvSection { ... }` (`tlv.rs:23-39`) — Decoded TLV section. Fields all `pub`:
  - `pub use_site_path_overrides: Option<Vec<(u8, UseSitePath)>>` (line 26)
  - `pub fingerprints: Option<Vec<(u8, [u8; 4])>>` (line 28)
  - `pub pubkeys: Option<Vec<(u8, [u8; 65])>>` (line 32)
  - `pub origin_path_overrides: Option<Vec<(u8, OriginPath)>>` (line 35)
  - `pub unknown: Vec<(u8, Vec<u8>, usize)>` (line 38) — `(tag, payload_bytes, bit_len)` triples; preserves unknown TLVs verbatim per D6 forward-compat.

#### `impl TlvSection` (`tlv.rs:41-307`)

- `pub fn new_empty() -> Self` (line 43)
- `pub fn is_empty(&self) -> bool` (line 57)
- `pub fn write(&self, w: &mut BitWriter, key_index_width: u8) -> Result<(), Error>` (line 86) — ascending tag order. Errors: `EmptyTlvEntry`, `OverrideOrderViolation`, inner encoding errors.
- `pub fn read(r: &mut BitReader, key_index_width: u8, n: u8) -> Result<Self, Error>` (line 212) — Consumes all remaining bits (with ≤7-bit codex32 padding tolerance via rollback).

### `md_codec::to_miniscript` (`src/to_miniscript.rs`) — feature `derive`

Module-level doc: "v0.32 AST → `miniscript::Descriptor<DescriptorPublicKey>` converter. Replaces the v0.14-era hand-rolled 5-shape allow-list with a generic converter that builds a miniscript `Descriptor` from any BIP-388-parseable md1 wire AST."

#### Functions

- `pub fn to_miniscript_descriptor(d: &Descriptor, chain: u32) -> Result<miniscript::Descriptor<miniscript::descriptor::DescriptorPublicKey>, Error>` (`to_miniscript.rs:54`) — Convert md1 `Descriptor` AST → rust-miniscript `Descriptor` for the given chain (multipath alt selector). Trailing `/*` wildcard left for `at_derivation_index` to resolve. Errors: propagated `MissingPubkey`, `InvalidXpubBytes`, `MissingExplicitOrigin` from `expand_per_at_n`; `AddressDerivationFailed` wrapping miniscript layer failures.

(All other items in this module are private helpers: `build_descriptor_public_key`, `origin_path_to_derivation`, `use_site_to_derivation_path`, `node_to_descriptor`, `build_nums_internal_key`, `wsh_inner_to_descriptor`, `sh_inner_to_descriptor`, `tree_to_taptree`, `node_to_miniscript`, `lookup_key`, `build_multi_threshold`, `arity_eq`, `failed`, `into_failed`, `sha256_from_bytes`, `hash256_from_bytes`, `ripemd160_from_bytes`, `hash160_from_bytes`, plus the `NUMS_H_POINT_X_ONLY_HEX` const.)

### `md_codec::tree` (`src/tree.rs`)

Module-level doc: "Tree (operator AST) per spec §3.6 + §6."

#### Types

- `pub struct Node { ... }` (`tree.rs:8-14`) — Operator AST node. Fields all `pub`:
  - `pub tag: Tag` (line 11)
  - `pub body: Body` (line 13)
- `pub enum Body { ... }` (`tree.rs:17-73`) — Body shape determined by `tag`. Variants:
  - `Children(Vec<Node>)` (line 20) — N child nodes (Class 1 fixed-arity)
  - `Variable { k: u8, children: Vec<Node> }` (line 22-29) — `Tag::Thresh` only (post-v0.30 Phase C); encodes `k` + N children
  - `MultiKeys { k: u8, indices: Vec<u8> }` (line 30-40) — Multi-family (`Tag::Multi`, `SortedMulti`, `MultiA`, `SortedMultiA`): k-of-n with raw `kiw`-width key indices (NOT full Nodes)
  - `Tr { is_nums: bool, key_index: u8, tree: Option<Box<Node>> }` (line 41-57) — Taproot body: NUMS flag + (if `!is_nums`) key index + optional tap-script-tree root. `is_nums=true` ⇒ internal key = BIP-341 NUMS H-point (`50929b74…`).
  - `KeyArg { index: u8 }` (line 58-64) — Single key-arg (Pkh, Wpkh, PkK, PkH, multi-family children). Wire bit-width derived from parent Descriptor's `key_index_width()`.
  - `Hash256Body([u8; 32])` (line 65-66) — 256-bit hash literal (Sha256, Hash256)
  - `Hash160Body([u8; 20])` (line 67-68) — 160-bit hash literal (Hash160, Ripemd160, RawPkH)
  - `Timelock(u32)` (line 69-70) — u32 bitcoin-native timelock (After, Older)
  - `Empty` (line 71-72) — False, True

  Both `Node` and `Body` derive `Debug, Clone, PartialEq, Eq`.

#### Functions (free)

- `pub fn write_node(w: &mut BitWriter, node: &Node, key_index_width: u8) -> Result<(), Error>` (`tree.rs:79`) — Encode a `Node` to bit stream. Errors: `ThresholdOutOfRange`, `ChildCountOutOfRange` (k/n outside 1..=32).
- `pub fn read_node(r: &mut BitReader, key_index_width: u8) -> Result<Node, Error>` (`tree.rs:178`) — Decode a `Node` from bit stream; internally threads `depth` counter capped at `MAX_DECODE_DEPTH`. Errors: `Tag::read` propagation, `KGreaterThanN`, `DecodeRecursionDepthExceeded`.

#### Constants

- `pub const MAX_DECODE_DEPTH: u8 = 128;` (`tree.rs:167`) — Hard cap on `read_node` recursion depth (anti-DoS hardening; coincides numerically with BIP-341 `TAPROOT_CONTROL_MAX_NODE_COUNT` but generic across all recursive tags).

### `md_codec::use_site_path` (`src/use_site_path.rs`)

Module-level doc: "Use-site-path-decl block per spec §3.5."

#### Types

- `pub struct Alternative { pub hardened: bool, pub value: u32 }` (`use_site_path.rs:18-24`) — One alternative in a multipath substitution group.
- `pub struct UseSitePath { pub multipath: Option<Vec<Alternative>>, pub wildcard_hardened: bool }` (`use_site_path.rs:48-54`) — Use-site path declaration. `Some(_)` ⇒ has-multipath bit set; `wildcard_hardened` is the trailing `*h` bit.

#### `impl Alternative` (`use_site_path.rs:26-40`)

- `pub fn write(&self, w: &mut BitWriter) -> Result<(), Error>` (line 28)
- `pub fn read(r: &mut BitReader) -> Result<Self, Error>` (line 35)

#### `impl UseSitePath` (`use_site_path.rs:56-117`)

- `pub fn standard_multipath() -> Self` (line 58) — The dominant `<0;1>/*` shape (`Some([Alt(false,0), Alt(false,1)])` + `wildcard_hardened=false`).
- `pub fn write(&self, w: &mut BitWriter) -> Result<(), Error>` (line 80) — Errors: `AltCountOutOfRange` if alt count ∉ 2..=9.
- `pub fn read(r: &mut BitReader) -> Result<Self, Error>` (line 99)

#### Constants

- `pub const MIN_ALT_COUNT: usize = 2;` (`use_site_path.rs:43`)
- `pub const MAX_ALT_COUNT: usize = 9;` (`use_site_path.rs:45`) — 3-bit field encoded as count-2.

### `md_codec::validate` (`src/validate.rs`)

Module-level doc: "Decoder-side validation per spec §7."

#### Functions

- `pub fn validate_placeholder_usage(root: &Node, n: u8) -> Result<(), Error>` (`validate.rs:17`) — Enforces (1) every `@i ∈ 0..n` appears at least once and (2) first occurrences appear in ascending order. Errors: `PlaceholderNotReferenced`, `PlaceholderFirstOccurrenceOutOfOrder`, `PlaceholderIndexOutOfRange`, `NUMSSentinelConflict`.
- `pub fn validate_multipath_consistency(shared: &UseSitePath, overrides: &[(u8, UseSitePath)]) -> Result<(), Error>` (`validate.rs:117`) — All multipath groups must share alt-count. Error: `MultipathAltCountMismatch`.
- `pub fn validate_tap_script_tree(node: &Node) -> Result<(), Error>` (`validate.rs:141`) — All leaves must be permitted (no `Wpkh|Tr|Wsh|Sh|Pkh|Multi|SortedMulti` at tap-tree leaves). Error: `ForbiddenTapTreeLeaf`.
- `pub fn validate_explicit_origin_required(d: &Descriptor) -> Result<(), Error>` (`validate.rs:182`) — When `canonical_origin(&d.tree).is_none()`, every `@N` must have an explicit non-empty origin (via override or path_decl). Error: `MissingExplicitOrigin`.
- `pub fn validate_xpub_bytes(d: &Descriptor) -> Result<(), Error>` (`validate.rs:216`) — Every `Pubkeys` entry's 33-byte compressed-pubkey field (bytes 32..65) parses as a valid secp256k1 point. No-op when `pubkeys` is `None`. Error: `InvalidXpubBytes`.

### `md_codec::varint` (`src/varint.rs`)

Module-level doc: "LP4-ext varint per spec §4.1."

#### Functions

- `pub fn write_varint(writer: &mut BitWriter, value: u32) -> Result<(), Error>` (`varint.rs:15`) — Encoding: `[L: 4][payload: L]`, with `L=15` ⇒ `[L: 4][L_high: 4][payload_low: 14][payload_high: L_high]` (payload bits = `L_high + 14`). Single-extension max value `2^29 - 1`. Error: `VarintOverflow`.
- `pub fn read_varint(reader: &mut BitReader) -> Result<u32, Error>` (`varint.rs:45`)

## Error taxonomy

`pub enum Error` from `src/error.rs` — 43 variants (line 19 to 392). `#[derive(Debug, Error, PartialEq, Eq)]`. Display strings come from `#[error("...")]` attributes; "Emitted by" populated from grep of `Error::Variant` construction sites within `src/`.

| Variant | Doc summary (one-line) | Emitted by (modules) |
|---|---|---|
| `BitStreamTruncated { requested, available }` | Read of `requested` bits with only `available` remaining | `bitstream::BitReader::read_bits` (line 129) |
| `WireVersionMismatch { got }` | Wire-format version field ≠ 4 (v0.30) | `header::Header::read` (43), `chunk::ChunkHeader::read` (63) |
| `MalformedHeader { detail }` | Header malformed (non-version) | **NEVER CONSTRUCTED** in src/ — declared (line 43) but no `Err(Error::MalformedHeader …)` site found. Flag for chapter author. |
| `PathDepthExceeded { got, max }` | Path depth exceeds `MAX_PATH_COMPONENTS = 15` | `origin_path::OriginPath::write` (56) |
| `KeyCountOutOfRange { n }` | `n` outside `1..=32` (SPEC §4) | `origin_path::PathDecl::write` (112) |
| `DivergentPathCountMismatch { n, got }` | Divergent path count ≠ key count | `origin_path::PathDecl::write` (120), `canonicalize::expand_per_at_n` (427) |
| `AltCountOutOfRange { got }` | Multipath alt-count outside `2..=9` (SPEC §8) | `use_site_path::UseSitePath::write` (83) |
| `TagOutOfRange { primary }` | 6-bit tag in reserved `0x24..=0x3E` or extension subspace `0x3F + 0x0..0xF` | `tag::Tag::read` (161, 200) |
| `ThresholdOutOfRange { k }` | Threshold `k` outside `1..=32` | `tree::write_node` (93, 109) |
| `ChildCountOutOfRange { count }` | Child count outside `1..=32` | `tree::write_node` (96, 112) |
| `KGreaterThanN { k, n }` | `k > n` in k-of-n threshold | `tree` decode paths (lines 230, 242) |
| `TlvOrderingViolation { prev, current }` | TLV tag not ascending | `tlv::TlvSection::read` (235) |
| `PlaceholderIndexOutOfRange { idx, n }` | TLV placeholder idx ≥ n | `tlv` reads (325), `canonicalize` (256, 289), `validate::walk_for_placeholders` (47, 73) |
| `OverrideOrderViolation { prev, current }` | Per-`@N` override entries not strictly ascending | `tlv::write` (110, 134, 158, 184), `tlv` read (329) |
| `EmptyTlvEntry { tag }` | TLV entry has zero entries (encoder must omit) | `tlv::write` (101, 125, 151, 175), `tlv::read` (249, 380) |
| `TlvLengthExceedsRemaining { length, remaining }` | Declared length > bits available | `tlv::read` (240) |
| `PlaceholderNotReferenced { idx, n }` | Placeholder `@i` not referenced anywhere | `canonicalize::canonicalize_placeholder_indices` (184), `validate::validate_placeholder_usage` (24) |
| `PlaceholderFirstOccurrenceOutOfOrder { expected_first, got_first }` | First-occurrence ordering broken | `validate::validate_placeholder_usage` (30) |
| `MultipathAltCountMismatch { expected, got }` | Multipaths in template disagree on alt-count | `validate::validate_multipath_consistency` (129) |
| `ForbiddenTapTreeLeaf { tag }` | Forbidden tag at tap-script-tree leaf (§6.3.1) | `validate::walk_tap_tree_leaves` (156) |
| `OperatorContextViolation { tag, context }` | Operator in forbidden context (SPEC §11) | `decode::decode_payload` (40) — TopLevel-allow-list-violator path |
| `ChunkCountOutOfRange { count }` | Chunk count outside `1..=64` | `chunk::ChunkHeader::write` (31) |
| `ChunkIndexOutOfRange { index, count }` | Chunk index ≥ count | `chunk::ChunkHeader::write` (34) |
| `ChunkSetIdOutOfRange { id }` | Chunk-set-id exceeds 20 bits | `chunk::ChunkHeader::write` (40) |
| `ChunkHeaderChunkedFlagMissing` | Chunked-flag bit not set (SPEC §2.2 bit 0 = 1) | `chunk::ChunkHeader::read` (67) |
| `ChunkCountExceedsMax { needed }` | Encoding needs > 64 chunks | `chunk::split` (244) |
| `Codex32DecodeError(String)` | Codex32 decode failure (HRP mismatch, alphabet, BCH) | `codex32::unwrap_string` (96, 109, 116, 123) |
| `Codex32EncodeError(String)` | Codex32 encode failure (BCH layer) | **NEVER CONSTRUCTED** in src/. Declared (line 250) but no construction site found — `wrap_payload` doesn't produce it; the `bch_create_checksum_regular` call is infallible. Flag for chapter author. |
| `ChunkSetEmpty` | No strings provided to reassemble | `chunk::reassemble` (304) |
| `ChunkSetInconsistent` | Chunks disagree on version / chunk-set-id / count | `chunk::reassemble` (341) |
| `ChunkSetIncomplete { got, expected }` | Got fewer chunks than expected | `chunk::reassemble` (345) |
| `ChunkIndexGap { expected, got }` | Gap in chunk index sequence | `chunk::reassemble` (355) |
| `ChunkSetIdMismatch { expected, derived }` | Reassembled-then-derived id ≠ headers' id | `chunk::reassemble` (375) |
| `VarintOverflow { value }` | LP4-ext value > `2^29 - 1` | `varint::write_varint` (31) |
| `MissingExplicitOrigin { idx }` | Non-canonical wrapper without explicit origin for `@idx` | `canonicalize::expand_per_at_n` (454), `validate::validate_explicit_origin_required` (203) |
| `InvalidPresenceByte { reserved_bits }` | `WalletPolicyId` presence byte has non-zero reserved bits 2..7 | `identity::validate_presence_byte` (256) |
| `InvalidXpubBytes { idx }` | Pubkeys TLV's 33-byte pubkey field is not a valid secp256k1 point | `validate::validate_xpub_bytes` (222), `derive::xpub_from_tlv_bytes` (55) |
| `MissingPubkey { idx }` | Address derivation lacks an xpub for `@idx` (template-only or partial) | `to_miniscript::build_descriptor_public_key` (73) |
| `ChainIndexOutOfRange { chain, alt_count }` | `chain` param out of multipath range | `derive::derive_address` (105, 114), `to_miniscript::use_site_to_derivation_path` (117) |
| `HardenedPublicDerivation` | Use-site path requires hardened derivation (BIP-32 forbids on xpub) | `derive::derive_address` (100, 111), `to_miniscript::use_site_to_derivation_path` (122) |
| `AddressDerivationFailed { detail }` | Miniscript-layer failure or converter arity/context mismatch | `derive::derive_address` (123, 128), `to_miniscript::failed` helper (475; called from many sites in the converter) |
| `NUMSSentinelConflict` | `is_nums=false` with `key_index` out of range (SPEC §7+§11) | `validate::walk_for_placeholders` (96), `canonicalize::check_placeholder_bounds` (270) |
| `DecodeRecursionDepthExceeded { depth, max }` | `read_node` recursion exceeded `MAX_DECODE_DEPTH=128` | `tree::read_node_with_depth` (187) |

(Variant count = 43 from the `pub enum Error { ... }` block, lines 20-392, excluding test-only references.)

## Feature-gated items

| Item | Feature | Path | Source |
|---|---|---|---|
| `pub mod to_miniscript` | `derive` | `md_codec::to_miniscript` | `src/lib.rs:32-33` (`#[cfg(feature = "derive")]`) |
| `pub fn to_miniscript_descriptor` | `derive` | `md_codec::to_miniscript::to_miniscript_descriptor` | `src/to_miniscript.rs:1-29` (module under cfg) |
| `pub fn Descriptor::derive_address` | `derive` | `md_codec::Descriptor::derive_address` | `src/derive.rs:66-67` (`#[cfg(feature = "derive")] impl Descriptor`) |
| `pub(crate) fn xpub_from_tlv_bytes` | `derive` | (crate-internal) | `src/derive.rs:48-49` (not pub-API; listed for completeness) |

All other `pub` items are unconditional.

## Notes for chapter author (Phase 4.1)

1. **Two `Error` variants are declared but never constructed in `src/`** — flag for the chapter draft:
   - `Error::MalformedHeader { detail }` (`error.rs:42-46`). No `Err(Error::MalformedHeader …)` site found. Doc-comment claims it's used "when chunked-flag inconsistent with caller context, or chunk-header internal field out of range," but the chunk-header field-range errors all use the dedicated `ChunkCountOutOfRange` / `ChunkIndexOutOfRange` / `ChunkSetIdOutOfRange` variants instead. Either dead code, reserved for future use, or a stale doc-comment. Chapter draft should mention "documented but currently unreachable" or recommend a FOLLOWUP to retire it.
   - `Error::Codex32EncodeError(String)` (`error.rs:249-251`). No construction site. The encode-side path (`codex32::wrap_payload`) calls `bch_create_checksum_regular` which returns a plain `[u8; 13]` (not a `Result`). Similarly "documented but currently unreachable" — flag.

2. **Feature-gating subtlety.** `pub mod to_miniscript` is cfg'd off the whole module declaration in `lib.rs`, but `pub mod derive` is unconditional in `lib.rs:23` even though the module body (`src/derive.rs`) wraps almost everything in `#[cfg(feature = "derive")]` per-item. Net effect with `default-features = false`: `md_codec::derive` exists as an empty-public-API module, while `md_codec::to_miniscript` does not exist at all. Chapter must not document `derive_address` as unconditionally available — it's gated on `derive`.

3. **Multiple `Body` variant kinds carry different placeholder-encoding paths.** Phase 4.1 must not flatten "all bodies hold child Nodes" — `Body::MultiKeys` carries raw `Vec<u8>` indices (NOT child Nodes), per the v0.30 Phase C wire-format change documented in `tree.rs:30-40`. Confusing these breaks any reader.

4. **`Tag::Tr` body has its own dedicated `Body::Tr` variant** (not `Body::Children`). Encodes `is_nums` flag + optional `key_index` + optional tap-script-tree root. The `is_nums=true` case skips writing `key_index` to the wire entirely (SPEC v0.30 §7). Chapter draft must spell this out, especially the NUMS H-point `50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0` (hard-coded x-only at `to_miniscript.rs:34-35`).

5. **`canonical_origin` table** (`src/canonical_origin.rs:8-19`) is small and load-bearing — Phase 4.1 should reproduce it as a table, not narrative prose. Includes notable explicit-only cases: `tr(@N, TapTree)`, `sh(sortedmulti)` (legacy P2SH multi), bare `wsh(@N)`, etc.

6. **`Phrase::from_id_bytes` cannot fail** despite the `Result` return type — the inner `bip39::Mnemonic::from_entropy(id)` is `.expect()`-unwrapped on a static "128-bit entropy is always a valid BIP-39 input" comment (`phrase.rs:18-19`). The `Result<Self, Error>` signature is for API uniformity. Chapter should note this — readers should not feel obliged to handle a non-existent error path.

7. **`Descriptor::derive_address` returns `Address<NetworkUnchecked>`** — chapter should not call it "the address" but explicitly mention that callers must `.assume_checked()` or `.require_network(network)` to lock it. This is the rust-bitcoin v0.32 idiom for separating untrusted parsing from trusted use; carries over here.

8. **`encode_payload` is self-canonicalizing.** Internally clones `d`, calls `canonicalize_placeholder_indices`, then runs validations and writes (`encode.rs:65-92`). Callers do NOT need to canonicalize beforehand; spec §6.1 first-occurrence ordering is enforced inside. (Contrast: `expand_per_at_n` requires the caller to have canonicalized — see `canonicalize.rs:370-376` "Precondition.")

9. **`compute_wallet_policy_id` also self-canonicalizes** (`identity.rs:172-178`). Both top-level identity functions take a `&Descriptor` and copy + canonicalize internally; users don't need to remember the order.

10. **Semver — no declaration found.** `Cargo.toml` doesn't list a published version policy; the v0.32 release notes (per `MEMORY.md` and the recent CHANGELOG) flag the `Error::UnsupportedDerivationShape` removal as breaking. No public semver-stability promise. Chapter should describe this as "pre-1.0 reference implementation; breaking changes possible on any 0.X bump."

11. **`MAX_DECODE_DEPTH = 128`** at `tree.rs:167` is anti-DoS hardening, NOT spec-mandated for non-taproot sites. Coincidence with BIP-341 `TAPROOT_CONTROL_MAX_NODE_COUNT` is incidental — Phase 4.1 should resist inferring a deeper connection. Doc-comment is explicit on this.

12. **No `pub use` of foreign crates.** No `bitcoin::*` or `miniscript::*` is re-exported. Public signatures (notably `Descriptor::derive_address` and `to_miniscript_descriptor`) reference foreign types directly. Chapter readers who want to type those signatures need to add `bitcoin` and `miniscript` crates separately or use `md_codec`'s exported `bitcoin` / `miniscript` re-exports if Phase 4.1 wants to add them (currently absent).

13. **`TlvSection::unknown`** preserves unknown TLVs verbatim through re-encoding (D6 forward-compat). Chapter draft should highlight this — it means a v0.32 decoder forward-encounters an unknown v0.40 tag and round-trips it without loss.

14. **Spec section numbers cited in doc-comments**: many doc-comments cite SPEC sections (§2.1, §3.4, §6.3, §7, §8.1, §11, etc.). These refer to `design/SPEC_v0_30_wire_format.md` in the descriptor-mnemonic repo (not the BIP draft). Chapter author should anchor citations to a single source of truth and possibly include section-number cross-references if the SPEC is published.
