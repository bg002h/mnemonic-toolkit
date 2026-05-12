# md-codec Rust API

This chapter is the reference for the `md-codec`\index{md-codec} crate's public surface at v0.32.0\index{md-codec v0.32.0} (HEAD `df1ed24` in `bg002h/descriptor-mnemonic`). It enumerates every public module, function, type, constant, and error variant. The wire format these APIs encode/decode is §II.1; the address-derivation tier they feed is §III.1; this chapter is the library API only. For the normative wire spec, see `descriptor-mnemonic/design/SPEC_v0_30_wire_format.md` and the in-tree BIP draft at `descriptor-mnemonic/bip/bip-mnemonic-descriptor.mediawiki`.

## V.1.1 Crate purpose

`md-codec`\index{md\_codec (crate)} is the reference encoder/decoder for md1 wire-format v0.30 (with v0.31 + v0.32 amendments). The crate produces bit-packed bytecode, wraps it as a codex32-style card string (HRP `md`, BCH-protected), and decodes the reverse — plus optional miniscript-tier address derivation behind the `derive` Cargo feature. The library is the sole HEAD source for the wire format: every BIP-draft section number cited in this chapter resolves to a doc-comment in `crates/md-codec/src/`. Pre-1.0 reference status; breaking changes are possible on any 0.X bump (v0.32 removed `Error::UnsupportedDerivationShape`).

## V.1.2 Feature flags

(from `crates/md-codec/Cargo.toml:21-23`.)

| Flag | Default | Gates | Implied deps |
|---|---|---|---|
| `derive`\index{derive (Cargo feature)} | yes (`default = ["derive"]`) | `pub mod to_miniscript`; `Descriptor::derive_address` | `dep:miniscript` (workspace pin) |

Pure-codec consumers opt out with `default-features = false`:

```toml
md-codec = { version = "0.32", default-features = false }
```

Subtlety: `pub mod derive` is unconditional in `lib.rs`, but the module body wraps every item in `#[cfg(feature = "derive")]`. With the feature off, `md_codec::derive` exists as an empty-public-API module while `md_codec::to_miniscript` does not exist at all. Document `Descriptor::derive_address` and `to_miniscript_descriptor` as feature-gated; everything else is unconditional.

## V.1.3 Public API by module

Twenty public modules (nineteen unconditional, one feature-gated). Re-exports at the crate root pull the most commonly-used items into `md_codec::`:

```rust
pub use canonicalize::canonicalize_placeholder_indices;
pub use chunk::{ChunkHeader, derive_chunk_set_id, reassemble, split};
pub use decode::{decode_md1_string, decode_payload};
pub use encode::{Descriptor, encode_md1_string, encode_payload};
pub use error::Error;
pub use header::Header;
pub use identity::{Md1EncodingId, WalletDescriptorTemplateId, WalletPolicyId,
    compute_md1_encoding_id, compute_wallet_descriptor_template_id,
    compute_wallet_policy_id, validate_presence_byte};
pub use origin_path::{OriginPath, PathComponent, PathDecl, PathDeclPaths};
pub use phrase::Phrase;
pub use tag::Tag;
pub use tlv::TlvSection;
```

(`crates/md-codec/src/lib.rs:39-52`.) No foreign types are re-exported; consumers needing `bitcoin::Address` or `miniscript::Descriptor` add those crates separately.

### V.1.3.1 `bitstream`\index{md\_codec::bitstream}

MSB-first bit packer + reader (BIP draft §"Bit ordering"; `crates/md-codec/src/bitstream.rs:1-5`).

| Item | Signature | Semantics | Source |
|---|---|---|---|
| `BitWriter`\index{BitWriter} | `pub struct BitWriter` (impls `Default`) | MSB-first bit packer | `bitstream.rs:11-16` |
| `BitWriter::new` | `fn new() -> Self` | empty writer | `bitstream.rs:20` |
| `BitWriter::write_bits` | `fn write_bits(&mut self, value: u64, count: usize)` | append `count` bits from LSB-aligned `value` MSB-first | `bitstream.rs:29` |
| `BitWriter::bit_len` | `fn bit_len(&self) -> usize` | total bits written | `bitstream.rs:72` |
| `BitWriter::into_bytes` | `fn into_bytes(self) -> Vec<u8>` | consume; final byte zero-padded | `bitstream.rs:81` |
| `BitReader<'a>`\index{BitReader} | `pub struct BitReader<'a>` | MSB-first bit unpacker over borrowed bytes | `bitstream.rs:89-96` |
| `BitReader::new` | `fn new(bytes: &'a [u8]) -> Self` | bit_limit = `bytes.len() * 8` | `bitstream.rs:101` |
| `BitReader::with_bit_limit` | `fn with_bit_limit(bytes: &'a [u8], bit_limit: usize) -> Self` | explicit bit cap | `bitstream.rs:113` |
| `BitReader::read_bits` | `fn read_bits(&mut self, count: usize) -> Result<u64, Error>` | LSB-aligned result | `bitstream.rs:123` |
| `BitReader::remaining_bits` | `fn remaining_bits(&self) -> usize` | bits left until limit | `bitstream.rs:164` |
| `BitReader::is_exhausted` | `fn is_exhausted(&self) -> bool` | `remaining_bits() == 0` | `bitstream.rs:169` |
| `BitReader::save_position` | `fn save_position(&self) -> usize` | snapshot for rollback | `bitstream.rs:176` |
| `BitReader::restore_position` | `fn restore_position(&mut self, saved: usize)` | restore snapshot | `bitstream.rs:181` |
| `re_emit_bits`\index{re\_emit\_bits} | `fn re_emit_bits(dst: &mut BitWriter, src_bytes: &[u8], bit_len: usize) -> Result<(), Error>` | read `bit_len` MSB-first bits from `src_bytes`, append to `dst` (TLV re-encoder) | `bitstream.rs:220` |

### V.1.3.2 `canonical_origin`\index{md\_codec::canonical\_origin}

Maps wrapper shape → canonical BIP origin path (v0.13 wallet-policy layer; `canonical_origin.rs`).

| Item | Signature | Semantics | Source |
|---|---|---|---|
| `canonical_origin`\index{canonical\_origin} | `fn canonical_origin(tree: &Node) -> Option<OriginPath>` | canonical BIP path for the wrapper shape; `None` ⇒ explicit override required | `canonical_origin.rs:45` |

Wrapper → path table (load-bearing):

| Wrapper | Canonical origin |
|---|---|
| `pkh(@N)` | `m/44'/0'/0'` |
| `wpkh(@N)` | `m/84'/0'/0'` |
| `tr(@N)` keypath-only | `m/86'/0'/0'` |
| `wsh(multi\|sortedmulti)` | `m/48'/0'/0'/2'` |
| `sh(wsh(multi\|sortedmulti))` | `m/48'/0'/0'/1'` |
| anything else (`tr(@N, TapTree)`, `sh(sortedmulti)`, bare `wsh(@N)`, miniscript leaves, etc.) | `None` |

`None` triggers `validate_explicit_origin_required` (see §V.1.3.18).

### V.1.3.3 `canonicalize`\index{md\_codec::canonicalize}

BIP-388 placeholder-ordering canonicalisation + per-`@N` expansion (SPEC v0.13 §6.1, §5.3 / §6.3; `canonicalize.rs`).

| Item | Signature | Semantics | Source |
|---|---|---|---|
| `canonicalize_placeholder_indices` | `fn canonicalize_placeholder_indices(d: &mut Descriptor) -> Result<(), Error>` | permute placeholder indices in place so first-occurrence sequence is `[0..n-1]`. Idempotent. Atomic across tree / divergent path decl / per-`@N` TLV maps | `canonicalize.rs:168` |
| `expand_per_at_n`\index{expand\_per\_at\_n} | `fn expand_per_at_n(d: &Descriptor) -> Result<Vec<ExpandedKey>, Error>` | resolve each `@N` into a fully-populated `ExpandedKey` by overlaying per-`@N` TLV overrides on descriptor-level baselines. **Precondition:** caller has canonicalized indices (or `d` came from decoder) | `canonicalize.rs:420` |
| `ExpandedKey`\index{ExpandedKey} | `pub struct ExpandedKey { pub idx: u8, pub origin_path: OriginPath, pub use_site_path: UseSitePath, pub fingerprint: Option<[u8; 4]>, pub xpub: Option<[u8; 65]> }` | resolved per-`@N` view | `canonicalize.rs:337-350` |

```rust
use md_codec::{canonicalize_placeholder_indices, decode_md1_string};
use md_codec::canonicalize::expand_per_at_n;
let mut d = decode_md1_string(card_str)?;
canonicalize_placeholder_indices(&mut d)?;
let keys = expand_per_at_n(&d)?;
for k in &keys {
    println!("@{} origin={:?} xpub={:?}", k.idx, k.origin_path, k.xpub.is_some());
}
```

### V.1.3.4 `chunk`\index{md\_codec::chunk}

Chunked-card framing (SPEC v0.30 §2.2; `chunk.rs`).

| Item | Signature | Semantics | Source |
|---|---|---|---|
| `ChunkHeader`\index{ChunkHeader} | `pub struct ChunkHeader { pub version: u8, pub chunk_set_id: u32, pub count: u8, pub index: u8 }` | 37-bit chunked wire header | `chunk.rs:13-23` |
| `ChunkHeader::write` | `fn write(&self, w: &mut BitWriter) -> Result<(), Error>` | emit 37 bits | `chunk.rs:29` |
| `ChunkHeader::read` | `fn read(r: &mut BitReader) -> Result<Self, Error>` | parse 37-bit chunk header | `chunk.rs:60` |
| `derive_chunk_set_id` | `fn derive_chunk_set_id(id: &Md1EncodingId) -> u32` | top 20 bits of the 16-byte id, MSB-first | `chunk.rs:168` |
| `split` | `fn split(d: &Descriptor) -> Result<Vec<String>, Error>` | split a `Descriptor` into N codex32 md1 strings; one per chunk | `chunk.rs:228` |
| `reassemble` | `fn reassemble(strings: &[&str]) -> Result<Descriptor, Error>` | inverse of `split`; validates consistency, sorts by index, decodes joined payload | `chunk.rs:298` |
| `SINGLE_STRING_PAYLOAD_BIT_LIMIT`\index{SINGLE\_STRING\_PAYLOAD\_BIT\_LIMIT} | `pub const SINGLE_STRING_PAYLOAD_BIT_LIMIT: usize = 64 * 5;` | threshold above which chunking is required (320 bits) | `chunk.rs:212` |

```rust
use md_codec::{split, reassemble, Descriptor};
let cards: Vec<String> = split(&d)?;
let card_refs: Vec<&str> = cards.iter().map(String::as_str).collect();
let round_tripped: Descriptor = reassemble(&card_refs)?;
```

### V.1.3.5 `codex32`\index{md\_codec::codex32}

v0.11-aligned BCH-protected envelope (SPEC §3.1 / D7; `codex32.rs`).

| Item | Signature | Semantics | Source |
|---|---|---|---|
| `wrap_payload`\index{wrap\_payload} | `fn wrap_payload(payload_bytes: &[u8], bit_count: usize) -> Result<String, Error>` | wrap a byte-padded payload bit stream into a complete codex32 md1 card (HRP `md` + payload + 13-symbol BCH checksum, symbol-aligned) | `codex32.rs:67` |
| `unwrap_string`\index{unwrap\_string} | `fn unwrap_string(s: &str) -> Result<(Vec<u8>, usize), Error>` | returns `(byte-padded payload bytes, symbol_aligned_bit_count)`. Tolerates whitespace + `-` separators | `codex32.rs:92` |

```rust
use md_codec::codex32::{wrap_payload, unwrap_string};
let card = wrap_payload(&bytes, bit_count)?;
let (got_bytes, got_bits) = unwrap_string(&card)?;
```

### V.1.3.6 `decode`\index{md\_codec::decode}

Top-level decoder (SPEC §13.2; `decode.rs`).

| Item | Signature | Semantics | Source |
|---|---|---|---|
| `decode_payload`\index{decode\_payload} | `fn decode_payload(bytes: &[u8], total_bits: usize) -> Result<Descriptor, Error>` | decode `Descriptor` from canonical payload bit stream. Validates header, path-decl, use-site-path, tree (capped at `MAX_DECODE_DEPTH=128`), TLVs. Top-level wrapper allow-list `{Sh, Wsh, Wpkh, Pkh, Tr}` enforced (else `OperatorContextViolation { context: TopLevel }`). Runs all five `validate::*` checks | `decode.rs:15` |
| `decode_md1_string`\index{decode\_md1\_string} | `fn decode_md1_string(s: &str) -> Result<Descriptor, Error>` | `unwrap_string` + `decode_payload` | `decode.rs:79` |

```rust
use md_codec::decode_md1_string;
let d = decode_md1_string("md1yqpqqxqq8xtwhw4xwn4qh")?;
assert_eq!(d.n, 1);
```

### V.1.3.7 `derive`\index{md\_codec::derive} — feature `derive`

Address derivation (v0.32 AST-driven converter, replacing the v0.14-era 5-shape allow-list; `derive.rs`).

| Item | Signature | Semantics | Source |
|---|---|---|---|
| `Descriptor::derive_address`\index{Descriptor::derive\_address} | `fn derive_address(&self, chain: u32, index: u32, network: bitcoin::Network) -> Result<bitcoin::Address<bitcoin::address::NetworkUnchecked>, Error>` | derive address at `(chain, index)`. `chain` selects the multipath alt (e.g. `0`=receive, `1`=change for `<0;1>/*`); `index` is the wildcard child number. Returns `NetworkUnchecked` — caller must `.assume_checked()` or `.require_network(network)` | `derive.rs:92` |

Pre-flight rejections (before invoking the converter): `HardenedPublicDerivation`, `ChainIndexOutOfRange`, `MissingPubkey`, `MissingExplicitOrigin`, `InvalidXpubBytes`, `AddressDerivationFailed`. See §III.1 for the three-tier derivation walk and §III.2 for shape coverage.

```rust
use bitcoin::Network;
use md_codec::decode_md1_string;
let d = decode_md1_string(card)?;
let addr_unchecked = d.derive_address(0, 0, Network::Bitcoin)?;
let addr = addr_unchecked.assume_checked();
```

### V.1.3.8 `encode`\index{md\_codec::encode}

Top-level encoder + `Descriptor` struct (SPEC §13.3; `encode.rs`).

| Item | Signature | Semantics | Source |
|---|---|---|---|
| `Descriptor`\index{Descriptor (md-codec)} | `pub struct Descriptor { pub n: u8, pub path_decl: PathDecl, pub use_site_path: UseSitePath, pub tree: Node, pub tlv: TlvSection }` | top-level v0.30 descriptor parsed/built from a wire payload | `encode.rs:16-28` |
| `Descriptor::key_index_width`\index{Descriptor::key\_index\_width} | `fn key_index_width(&self) -> u8` | `⌈log₂(n)⌉`; clamped to 0 at n∈{0,1} | `encode.rs:37` |
| `Descriptor::is_wallet_policy`\index{Descriptor::is\_wallet\_policy} | `fn is_wallet_policy(&self) -> bool` | `true` iff `tlv.pubkeys.is_some()` with non-empty vec | `encode.rs:50` |
| `encode_payload`\index{encode\_payload} | `fn encode_payload(d: &Descriptor) -> Result<(Vec<u8>, usize), Error>` | encode descriptor → (bytes, total_bit_count). Self-canonicalises internally; runs three `validate::*` checks before write | `encode.rs:65` |
| `render_codex32_grouped`\index{render\_codex32\_grouped} | `fn render_codex32_grouped(s: &str, group_size: usize) -> String` | hyphen-grouped rendering for human display; `group_size = 0` returns input unchanged | `encode.rs:98` |
| `encode_md1_string`\index{encode\_md1\_string} | `fn encode_md1_string(d: &Descriptor) -> Result<String, Error>` | `encode_payload` + `codex32::wrap_payload` | `encode.rs:114` |

```rust
use md_codec::{Descriptor, encode_md1_string};
use md_codec::encode::render_codex32_grouped;
let card: String = encode_md1_string(&d)?;
let grouped = render_codex32_grouped(&card, 4); // "md1y-qpqq-xqq8-xtwh-w4xw-n4qh"
```

### V.1.3.9 `error`\index{md\_codec::error}

Error taxonomy (43 variants; full table in §V.1.4). Two public types:

| Item | Signature | Semantics | Source |
|---|---|---|---|
| `ContextKind`\index{ContextKind} | `pub enum ContextKind { TopLevel, TapLeaf, MultiBody }` | where in the descriptor tree an operator appears (SPEC v0.30 §11) | `error.rs:9-16` |
| `Error`\index{Error (md-codec)} | `pub enum Error { ... }` (`thiserror::Error`; derives `Debug, Error, PartialEq, Eq`) | 43 variants | `error.rs:19-392` |

### V.1.3.10 `header`\index{md\_codec::header}

5-bit single-payload header (SPEC v0.30 §2.1; `header.rs`). Chunked-header is in §V.1.3.4.

| Item | Signature | Semantics | Source |
|---|---|---|---|
| `Header`\index{Header (md-codec)} | `pub struct Header { pub version: u8, pub divergent_paths: bool }` | 4-bit version + 1 bit | `header.rs:16-22` |
| `Header::WF_REDESIGN_VERSION`\index{Header::WF\_REDESIGN\_VERSION} | `pub const WF_REDESIGN_VERSION: u8 = 4;` | v0.30 version literal | `header.rs:27` |
| `Header::write` | `fn write(&self, w: &mut BitWriter)` | emit 5 bits | `header.rs:30` |
| `Header::read` | `fn read(r: &mut BitReader) -> Result<Self, Error>` | parse 5 bits; rejects `version != 4` with `WireVersionMismatch` | `header.rs:38` |

### V.1.3.11 `identity`\index{md\_codec::identity}

Identity computations (SPEC §8; `identity.rs`).

| Item | Signature | Semantics | Source |
|---|---|---|---|
| `Md1EncodingId`\index{Md1EncodingId} | `pub struct Md1EncodingId([u8; 16]);` | 128-bit canonical identifier (SHA-256 of canonical bytecode bytes, truncated to 16) | `identity.rs:15-16` |
| `Md1EncodingId::new` / `as_bytes` / `fingerprint` | `fn new(bytes: [u8; 16]) -> Self` / `fn as_bytes(&self) -> &[u8; 16]` / `fn fingerprint(&self) -> [u8; 4]` | construction; raw access; first 4 bytes | `identity.rs:20, 25, 30` |
| `WalletDescriptorTemplateId`\index{WalletDescriptorTemplateId} | `pub struct WalletDescriptorTemplateId([u8; 16]);` | 128-bit BIP-388 template id (γ-flavor; hashes template content) | `identity.rs:54-55` |
| `WalletDescriptorTemplateId::new` / `as_bytes` | as for `Md1EncodingId` | — | `identity.rs:59, 64` |
| `WalletPolicyId`\index{WalletPolicyId} | `pub struct WalletPolicyId([u8; 16]);` | 128-bit canonical wallet-policy id (SPEC v0.13 §5.3) | `identity.rs:114-115` |
| `WalletPolicyId::new` / `as_bytes` | as above | — | `identity.rs:119, 124` |
| `WalletPolicyId::to_phrase`\index{WalletPolicyId::to\_phrase} | `fn to_phrase(&self) -> Result<Phrase, Error>` | render id as 12-word BIP-39 phrase (SPEC §8.4) | `identity.rs:129` |
| `compute_md1_encoding_id`\index{compute\_md1\_encoding\_id} | `fn compute_md1_encoding_id(d: &Descriptor) -> Result<Md1EncodingId, Error>` | SHA-256 over canonical bit-packed payload bytes (truncated 16) | `identity.rs:39` |
| `compute_wallet_descriptor_template_id`\index{compute\_wallet\_descriptor\_template\_id} | `fn compute_wallet_descriptor_template_id(d: &Descriptor) -> Result<WalletDescriptorTemplateId, Error>` | SHA-256 over use-site-path bits ‖ tree bits ‖ optional `UseSitePathOverrides` (truncated 16) | `identity.rs:71` |
| `compute_wallet_policy_id`\index{compute\_wallet\_policy\_id} | `fn compute_wallet_policy_id(d: &Descriptor) -> Result<WalletPolicyId, Error>` | SHA-256 over canonical template ‖ per-`@N` records. Self-canonicalises + runs `expand_per_at_n` internally | `identity.rs:172` |
| `validate_presence_byte`\index{validate\_presence\_byte} | `fn validate_presence_byte(byte: u8) -> Result<(), Error>` | reject reserved bits 2..7 nonzero; returns `InvalidPresenceByte` | `identity.rs:253` |

```rust
use md_codec::{compute_wallet_policy_id, decode_md1_string};
let d = decode_md1_string(card)?;
let id = compute_wallet_policy_id(&d)?;
let phrase = id.to_phrase()?;
println!("policy_id = {}", phrase);  // 12 BIP-39 words
```

### V.1.3.12 `origin_path`\index{md\_codec::origin\_path}

Origin-path-decl block (SPEC §3.4; `origin_path.rs`).

| Item | Signature | Semantics | Source |
|---|---|---|---|
| `PathComponent`\index{PathComponent} | `pub struct PathComponent { pub hardened: bool, pub value: u32 }` | one BIP-32 component (e.g. `84'`). `value` is u31 (LP4-ext) | `origin_path.rs:18-24` |
| `PathComponent::write` / `read` | `fn write(&self, w: &mut BitWriter) -> Result<(), Error>` / `fn read(r: &mut BitReader) -> Result<Self, Error>` | 1 hardened bit + LP4-ext varint | `origin_path.rs:28, 35` |
| `OriginPath`\index{OriginPath} | `pub struct OriginPath { pub components: Vec<PathComponent> }` | sequence root→leaf | `origin_path.rs:46-50` |
| `OriginPath::write` / `read` | as for `PathComponent` | 4-bit depth + components | `origin_path.rs:54, 69` |
| `PathDecl`\index{PathDecl} | `pub struct PathDecl { pub n: u8, pub paths: PathDeclPaths }` | path declaration (key count + paths variant) | `origin_path.rs:81-87` |
| `PathDeclPaths`\index{PathDeclPaths} | `pub enum PathDeclPaths { Shared(OriginPath), Divergent(Vec<OriginPath>) }` | header bit 4 selects variant | `origin_path.rs:90-96` |
| `PathDecl::write` | `fn write(&self, w: &mut BitWriter) -> Result<(), Error>` | caller responsible for header bit 4 | `origin_path.rs:110` |
| `PathDecl::read` | `fn read(r: &mut BitReader, divergent_mode: bool) -> Result<Self, Error>` | — | `origin_path.rs:134` |
| `MAX_PATH_COMPONENTS`\index{MAX\_PATH\_COMPONENTS} | `pub const MAX_PATH_COMPONENTS: usize = 15;` | 4-bit depth field max | `origin_path.rs:43` |

### V.1.3.13 `phrase`\index{md\_codec::phrase}

BIP-39 phrase rendering (SPEC §8.4; `phrase.rs`).

| Item | Signature | Semantics | Source |
|---|---|---|---|
| `Phrase`\index{Phrase} | `pub struct Phrase(pub [String; 12]);` | tuple-struct holding 12 BIP-39 words | `phrase.rs:7-10` |
| `Phrase::from_id_bytes`\index{Phrase::from\_id\_bytes} | `fn from_id_bytes(id: &[u8; 16]) -> Result<Self, Error>` | render 128 bits as 12 BIP-39 words. **Infallible in practice** — inner `.expect()` cannot fire on 128-bit entropy; `Result` is API-uniform shape | `phrase.rs:17` |
| `impl Display for Phrase` | — | joins words with single space | `phrase.rs:28-32` |

### V.1.3.14 `tag`\index{md\_codec::tag}

v0.30 Tag enum (SPEC §3; `tag.rs`). 36 variants in the 6-bit primary space `0x00..=0x23`; primary `0x24..=0x3E` reserved; primary `0x3F` is the extension prefix with the 4-bit subspace `0x00..=0x0F` fully reserved.

| Item | Signature | Semantics | Source |
|---|---|---|---|
| `Tag`\index{Tag (md-codec)} | `pub enum Tag { Wpkh, Tr, Wsh, Sh, Pkh, TapTree, Multi, SortedMulti, MultiA, SortedMultiA, PkK, PkH, Check, Verify, Swap, Alt, DupIf, NonZero, ZeroNotEqual, AndV, AndB, AndOr, OrB, OrC, OrD, OrI, Thresh, After, Older, Sha256, Hash160, Hash256, Ripemd160, RawPkH, False, True }` | 36 operator tags | `tag.rs:14-89` |
| `Tag::write` | `fn write(&self, w: &mut BitWriter)` | 6 bits (+4 if extension; never reached in v0.30) | `tag.rs:140` |
| `Tag::read` | `fn read(r: &mut BitReader) -> Result<Self, Error>` | parse; reserved/extension → `TagOutOfRange { primary }` | `tag.rs:156` |

The full tag-code table is reproduced in §II.1 §"Tag table (v0.30)" — that table is the authoritative narrative; this chapter does not reprint it.

### V.1.3.15 `tlv`\index{md\_codec::tlv}

TLV section (SPEC §3.7 + v0.13 §3.2; `tlv.rs`).

| Item | Signature | Semantics | Source |
|---|---|---|---|
| `TLV_USE_SITE_PATH_OVERRIDES`\index{TLV\_USE\_SITE\_PATH\_OVERRIDES} | `pub const TLV_USE_SITE_PATH_OVERRIDES: u8 = 0x00;` | tag-code constant | `tlv.rs:11` |
| `TLV_FINGERPRINTS`\index{TLV\_FINGERPRINTS} | `pub const TLV_FINGERPRINTS: u8 = 0x01;` | tag-code constant | `tlv.rs:13` |
| `TLV_PUBKEYS`\index{TLV\_PUBKEYS} | `pub const TLV_PUBKEYS: u8 = 0x02;` | tag-code constant | `tlv.rs:16` |
| `TLV_ORIGIN_PATH_OVERRIDES`\index{TLV\_ORIGIN\_PATH\_OVERRIDES} | `pub const TLV_ORIGIN_PATH_OVERRIDES: u8 = 0x03;` | tag-code constant | `tlv.rs:19` |
| `TlvSection`\index{TlvSection} | `pub struct TlvSection { pub use_site_path_overrides: Option<Vec<(u8, UseSitePath)>>, pub fingerprints: Option<Vec<(u8, [u8; 4])>>, pub pubkeys: Option<Vec<(u8, [u8; 65])>>, pub origin_path_overrides: Option<Vec<(u8, OriginPath)>>, pub unknown: Vec<(u8, Vec<u8>, usize)> }` | decoded TLV section. `unknown` preserves unknown TLVs verbatim (D6 forward-compat: a v0.32 decoder round-trips an unknown v0.40 tag without loss) | `tlv.rs:23-39` |
| `TlvSection::new_empty` | `fn new_empty() -> Self` | empty section (all `None`; `unknown = []`) | `tlv.rs:43` |
| `TlvSection::is_empty` | `fn is_empty(&self) -> bool` | true iff every field is `None`/empty | `tlv.rs:57` |
| `TlvSection::write` | `fn write(&self, w: &mut BitWriter, key_index_width: u8) -> Result<(), Error>` | ascending-tag emission | `tlv.rs:86` |
| `TlvSection::read` | `fn read(r: &mut BitReader, key_index_width: u8, n: u8) -> Result<Self, Error>` | consumes all remaining bits (with ≤7-bit codex32 padding tolerance via rollback) | `tlv.rs:212` |

### V.1.3.16 `to_miniscript`\index{md\_codec::to\_miniscript} — feature `derive`

v0.32 AST → rust-miniscript converter (`to_miniscript.rs`).

| Item | Signature | Semantics | Source |
|---|---|---|---|
| `to_miniscript_descriptor`\index{to\_miniscript\_descriptor} | `fn to_miniscript_descriptor(d: &Descriptor, chain: u32) -> Result<miniscript::Descriptor<miniscript::descriptor::DescriptorPublicKey>, Error>` | convert md1 AST → rust-miniscript `Descriptor` for the given chain. Trailing `/*` wildcard left for `at_derivation_index` to resolve | `to_miniscript.rs:54` |

```rust
use md_codec::to_miniscript::to_miniscript_descriptor;
let ms_desc = to_miniscript_descriptor(&d, 0)?;
let key_at = ms_desc.at_derivation_index(7)?;
let addr = key_at.address(bitcoin::Network::Bitcoin)?;
```

### V.1.3.17 `tree`\index{md\_codec::tree}

Operator AST (SPEC §3.6 + §6; `tree.rs`).

| Item | Signature | Semantics | Source |
|---|---|---|---|
| `Node`\index{Node (md-codec)} | `pub struct Node { pub tag: Tag, pub body: Body }` | operator AST node | `tree.rs:8-14` |
| `Body`\index{Body (md-codec)} | `pub enum Body { Children(Vec<Node>), Variable { k: u8, children: Vec<Node> }, MultiKeys { k: u8, indices: Vec<u8> }, Tr { is_nums: bool, key_index: u8, tree: Option<Box<Node>> }, KeyArg { index: u8 }, Hash256Body([u8; 32]), Hash160Body([u8; 20]), Timelock(u32), Empty }` | body shape determined by `tag` (see below) | `tree.rs:17-73` |
| `write_node`\index{write\_node} | `fn write_node(w: &mut BitWriter, node: &Node, key_index_width: u8) -> Result<(), Error>` | encode node to bit stream | `tree.rs:79` |
| `read_node`\index{read\_node} | `fn read_node(r: &mut BitReader, key_index_width: u8) -> Result<Node, Error>` | decode node; threads internal depth counter capped at `MAX_DECODE_DEPTH` | `tree.rs:178` |
| `MAX_DECODE_DEPTH`\index{MAX\_DECODE\_DEPTH} | `pub const MAX_DECODE_DEPTH: u8 = 128;` | recursion cap (anti-DoS hardening; numerical coincidence with BIP-341 `TAPROOT_CONTROL_MAX_NODE_COUNT` is incidental — generic across all recursive tags) | `tree.rs:167` |

`Body` variants in detail (load-bearing for any AST walker):

- `Children(Vec<Node>)` — fixed-arity `N` child nodes (most operators).
- `Variable { k, children }` — `Tag::Thresh` only (`thresh(k, ...)`). Full child Nodes.
- `MultiKeys { k, indices }` — multi-family (`Multi`, `SortedMulti`, `MultiA`, `SortedMultiA`) carries raw `Vec<u8>` key indices, **not** `Vec<Node>`. Easy to misread. See §II.1 §"Body shapes" for the wire layout.
- `Tr { is_nums, key_index, tree }` — Taproot. `is_nums=true` ⇒ internal key is the BIP-341 NUMS H-point (`50929b74…803ac0`); `key_index` is then suppressed on the wire. `is_nums=false` ⇒ `key_index ∈ 0..n`.
- `KeyArg { index }` — single-key bodies (Pkh, Wpkh, PkK, PkH, plus the indices inside multi-family bodies are flattened to repeated `KeyArg` at AST time per the converter).
- `Hash256Body([u8; 32])` — 256-bit hash literal (Sha256, Hash256).
- `Hash160Body([u8; 20])` — 160-bit hash literal (Hash160, Ripemd160, RawPkH).
- `Timelock(u32)` — u32 timelock (After, Older).
- `Empty` — `False`, `True`.

### V.1.3.18 `use_site_path`\index{md\_codec::use\_site\_path}

Use-site-path-decl block (SPEC §3.5; `use_site_path.rs`).

| Item | Signature | Semantics | Source |
|---|---|---|---|
| `Alternative`\index{Alternative (use-site)} | `pub struct Alternative { pub hardened: bool, pub value: u32 }` | one multipath alternative | `use_site_path.rs:18-24` |
| `Alternative::write` / `read` | — | bit-level read/write | `use_site_path.rs:28, 35` |
| `UseSitePath`\index{UseSitePath} | `pub struct UseSitePath { pub multipath: Option<Vec<Alternative>>, pub wildcard_hardened: bool }` | use-site declaration | `use_site_path.rs:48-54` |
| `UseSitePath::standard_multipath` | `fn standard_multipath() -> Self` | the dominant `<0;1>/*` shape | `use_site_path.rs:58` |
| `UseSitePath::write` | `fn write(&self, w: &mut BitWriter) -> Result<(), Error>` | rejects alt-count ∉ `2..=9` with `AltCountOutOfRange` | `use_site_path.rs:80` |
| `UseSitePath::read` | `fn read(r: &mut BitReader) -> Result<Self, Error>` | — | `use_site_path.rs:99` |
| `MIN_ALT_COUNT`\index{MIN\_ALT\_COUNT} | `pub const MIN_ALT_COUNT: usize = 2;` | minimum multipath alt count | `use_site_path.rs:43` |
| `MAX_ALT_COUNT`\index{MAX\_ALT\_COUNT} | `pub const MAX_ALT_COUNT: usize = 9;` | maximum (3-bit field encoded as count − 2) | `use_site_path.rs:45` |

### V.1.3.19 `validate`\index{md\_codec::validate}

Decoder-side validations (SPEC §7; `validate.rs`).

| Item | Signature | Semantics | Source |
|---|---|---|---|
| `validate_placeholder_usage`\index{validate\_placeholder\_usage} | `fn validate_placeholder_usage(root: &Node, n: u8) -> Result<(), Error>` | every `@i ∈ 0..n` referenced; first occurrences ascending | `validate.rs:17` |
| `validate_multipath_consistency`\index{validate\_multipath\_consistency} | `fn validate_multipath_consistency(shared: &UseSitePath, overrides: &[(u8, UseSitePath)]) -> Result<(), Error>` | all multipath groups share alt-count | `validate.rs:117` |
| `validate_tap_script_tree`\index{validate\_tap\_script\_tree} | `fn validate_tap_script_tree(node: &Node) -> Result<(), Error>` | tap-tree leaves restricted (no `Wpkh\|Tr\|Wsh\|Sh\|Pkh\|Multi\|SortedMulti`) | `validate.rs:141` |
| `validate_explicit_origin_required`\index{validate\_explicit\_origin\_required} | `fn validate_explicit_origin_required(d: &Descriptor) -> Result<(), Error>` | when `canonical_origin(&d.tree).is_none()`, every `@N` needs an explicit non-empty origin | `validate.rs:182` |
| `validate_xpub_bytes`\index{validate\_xpub\_bytes} | `fn validate_xpub_bytes(d: &Descriptor) -> Result<(), Error>` | every `Pubkeys` entry's 33-byte compressed pubkey field parses as a valid secp256k1 point | `validate.rs:216` |

`encode_payload` runs the first three; `decode_payload` runs all five.

### V.1.3.20 `varint`\index{md\_codec::varint}

LP4-ext varint (SPEC §4.1; `varint.rs`).

| Item | Signature | Semantics | Source |
|---|---|---|---|
| `write_varint`\index{write\_varint} | `fn write_varint(writer: &mut BitWriter, value: u32) -> Result<(), Error>` | `[L: 4][payload: L]`; `L=15` triggers the extension `[L: 4][L_high: 4][payload_low: 14][payload_high: L_high]`. Single-extension max `2^29 − 1` | `varint.rs:15` |
| `read_varint`\index{read\_varint} | `fn read_varint(reader: &mut BitReader) -> Result<u32, Error>` | — | `varint.rs:45` |

## V.1.4 Error taxonomy

`pub enum Error` from `src/error.rs` — 43 variants (lines 19-392), `#[derive(Debug, Error, PartialEq, Eq)]`. Grouped by emit-site cluster; within each group, ordered by source line.

### Bitstream, header, varint

| Variant | Display | Emitted by | Source |
|---|---|---|---|
| `BitStreamTruncated { requested, available }`\index{Error::BitStreamTruncated} | requested N bits with only M available | `bitstream::BitReader::read_bits` | `bitstream.rs:129` |
| `WireVersionMismatch { got }`\index{Error::WireVersionMismatch} | wire version field ≠ 4 | `header::Header::read`, `chunk::ChunkHeader::read` | `header.rs:43`, `chunk.rs:63` |
| `MalformedHeader { detail }`\index{Error::MalformedHeader} | header malformed (non-version) | **declared but never constructed in v0.32 source** — see §V.1.7 | `error.rs:42-46` |
| `VarintOverflow { value }`\index{Error::VarintOverflow} | LP4-ext value > `2^29 − 1` | `varint::write_varint` | `varint.rs:31` |

### Path-decl + use-site

| Variant | Display | Emitted by | Source |
|---|---|---|---|
| `PathDepthExceeded { got, max }`\index{Error::PathDepthExceeded} | path depth > `MAX_PATH_COMPONENTS = 15` | `origin_path::OriginPath::write` | `origin_path.rs:56` |
| `KeyCountOutOfRange { n }`\index{Error::KeyCountOutOfRange} | `n` outside `1..=32` | `origin_path::PathDecl::write` | `origin_path.rs:112` |
| `DivergentPathCountMismatch { n, got }`\index{Error::DivergentPathCountMismatch} | divergent path count ≠ key count | `origin_path::PathDecl::write`, `canonicalize::expand_per_at_n` | `origin_path.rs:120`, `canonicalize.rs:427` |
| `AltCountOutOfRange { got }`\index{Error::AltCountOutOfRange} | multipath alt count ∉ `2..=9` | `use_site_path::UseSitePath::write` | `use_site_path.rs:83` |

### Tag + tree

| Variant | Display | Emitted by | Source |
|---|---|---|---|
| `TagOutOfRange { primary }`\index{Error::TagOutOfRange} | reserved/extension tag (`0x24..=0x3E`, or `0x3F + sub`) | `tag::Tag::read` | `tag.rs:161, 200` |
| `ThresholdOutOfRange { k }`\index{Error::ThresholdOutOfRange} | threshold `k` outside `1..=32` | `tree::write_node` | `tree.rs:93, 109` |
| `ChildCountOutOfRange { count }`\index{Error::ChildCountOutOfRange} | child count outside `1..=32` | `tree::write_node` | `tree.rs:96, 112` |
| `KGreaterThanN { k, n }`\index{Error::KGreaterThanN} | `k > n` in k-of-n | `tree` decode paths | `tree.rs:230, 242` |
| `DecodeRecursionDepthExceeded { depth, max }`\index{Error::DecodeRecursionDepthExceeded} | `read_node` recursion > `MAX_DECODE_DEPTH = 128` | `tree::read_node_with_depth` | `tree.rs:187` |

### TLV section

| Variant | Display | Emitted by | Source |
|---|---|---|---|
| `TlvOrderingViolation { prev, current }`\index{Error::TlvOrderingViolation} | TLV tag not ascending | `tlv::TlvSection::read` | `tlv.rs:235` |
| `PlaceholderIndexOutOfRange { idx, n }`\index{Error::PlaceholderIndexOutOfRange} | TLV placeholder idx ≥ n | `tlv` reads, `canonicalize`, `validate` | `tlv.rs:325`, `canonicalize.rs:256, 289`, `validate.rs:47, 73` |
| `OverrideOrderViolation { prev, current }`\index{Error::OverrideOrderViolation} | per-`@N` overrides not strictly ascending | `tlv::write`, `tlv::read` | `tlv.rs:110, 134, 158, 184, 329` |
| `EmptyTlvEntry { tag }`\index{Error::EmptyTlvEntry} | TLV entry has zero entries (encoder must omit) | `tlv::write`, `tlv::read` | `tlv.rs:101, 125, 151, 175, 249, 380` |
| `TlvLengthExceedsRemaining { length, remaining }`\index{Error::TlvLengthExceedsRemaining} | declared length > bits available | `tlv::read` | `tlv.rs:240` |

### Canonicality + placeholder usage

| Variant | Display | Emitted by | Source |
|---|---|---|---|
| `PlaceholderNotReferenced { idx, n }`\index{Error::PlaceholderNotReferenced} | placeholder `@i` not referenced | `canonicalize`, `validate::validate_placeholder_usage` | `canonicalize.rs:184`, `validate.rs:24` |
| `PlaceholderFirstOccurrenceOutOfOrder { expected_first, got_first }`\index{Error::PlaceholderFirstOccurrenceOutOfOrder} | first-occurrence ordering broken | `validate::validate_placeholder_usage` | `validate.rs:30` |
| `MultipathAltCountMismatch { expected, got }`\index{Error::MultipathAltCountMismatch} | multipath alt-count disagreement | `validate::validate_multipath_consistency` | `validate.rs:129` |
| `ForbiddenTapTreeLeaf { tag }`\index{Error::ForbiddenTapTreeLeaf} | forbidden tag at tap-script-tree leaf | `validate::walk_tap_tree_leaves` | `validate.rs:156` |
| `OperatorContextViolation { tag, context }`\index{Error::OperatorContextViolation} | operator in forbidden context | `decode::decode_payload` (TopLevel allow-list) | `decode.rs:40` |
| `NUMSSentinelConflict`\index{Error::NUMSSentinelConflict} | `is_nums=false` with `key_index` out of range | `validate::walk_for_placeholders`, `canonicalize::check_placeholder_bounds` | `validate.rs:96`, `canonicalize.rs:270` |
| `MissingExplicitOrigin { idx }`\index{Error::MissingExplicitOrigin} | non-canonical wrapper without explicit origin for `@idx` | `canonicalize::expand_per_at_n`, `validate::validate_explicit_origin_required` | `canonicalize.rs:454`, `validate.rs:203` |
| `InvalidPresenceByte { reserved_bits }`\index{Error::InvalidPresenceByte} | `WalletPolicyId` presence byte has non-zero reserved bits 2..7 | `identity::validate_presence_byte` | `identity.rs:256` |
| `InvalidXpubBytes { idx }`\index{Error::InvalidXpubBytes} | Pubkeys TLV's 33-byte pubkey is not a valid secp256k1 point | `validate::validate_xpub_bytes`, `derive::xpub_from_tlv_bytes` | `validate.rs:222`, `derive.rs:55` |

### Chunking

| Variant | Display | Emitted by | Source |
|---|---|---|---|
| `ChunkCountOutOfRange { count }`\index{Error::ChunkCountOutOfRange} | chunk count ∉ `1..=64` | `chunk::ChunkHeader::write` | `chunk.rs:31` |
| `ChunkIndexOutOfRange { index, count }`\index{Error::ChunkIndexOutOfRange} | chunk index ≥ count | `chunk::ChunkHeader::write` | `chunk.rs:34` |
| `ChunkSetIdOutOfRange { id }`\index{Error::ChunkSetIdOutOfRange} | chunk-set-id > 20 bits | `chunk::ChunkHeader::write` | `chunk.rs:40` |
| `ChunkHeaderChunkedFlagMissing`\index{Error::ChunkHeaderChunkedFlagMissing} | chunked-flag bit not set | `chunk::ChunkHeader::read` | `chunk.rs:67` |
| `ChunkCountExceedsMax { needed }`\index{Error::ChunkCountExceedsMax} | encoding needs > 64 chunks | `chunk::split` | `chunk.rs:244` |
| `ChunkSetEmpty`\index{Error::ChunkSetEmpty} | no strings provided to reassemble | `chunk::reassemble` | `chunk.rs:304` |
| `ChunkSetInconsistent`\index{Error::ChunkSetInconsistent} | chunks disagree on version / chunk-set-id / count | `chunk::reassemble` | `chunk.rs:341` |
| `ChunkSetIncomplete { got, expected }`\index{Error::ChunkSetIncomplete} | fewer chunks than expected | `chunk::reassemble` | `chunk.rs:345` |
| `ChunkIndexGap { expected, got }`\index{Error::ChunkIndexGap} | gap in chunk index sequence | `chunk::reassemble` | `chunk.rs:355` |
| `ChunkSetIdMismatch { expected, derived }`\index{Error::ChunkSetIdMismatch} | reassembled-then-derived id ≠ headers' id | `chunk::reassemble` | `chunk.rs:375` |

### Codex32 envelope

| Variant | Display | Emitted by | Source |
|---|---|---|---|
| `Codex32DecodeError(String)`\index{Error::Codex32DecodeError} | codex32 decode failure (HRP, alphabet, BCH) | `codex32::unwrap_string` | `codex32.rs:96, 109, 116, 123` |
| `Codex32EncodeError(String)`\index{Error::Codex32EncodeError} | codex32 encode failure | **declared but never constructed in v0.32 source** — see §V.1.7 | `error.rs:249-251` |

### Address derivation (feature `derive`)

| Variant | Display | Emitted by | Source |
|---|---|---|---|
| `MissingPubkey { idx }`\index{Error::MissingPubkey} | derivation lacks xpub for `@idx` | `to_miniscript::build_descriptor_public_key` | `to_miniscript.rs:73` |
| `ChainIndexOutOfRange { chain, alt_count }`\index{Error::ChainIndexOutOfRange} | `chain` param out of multipath range | `derive::derive_address`, `to_miniscript::use_site_to_derivation_path` | `derive.rs:105, 114`, `to_miniscript.rs:117` |
| `HardenedPublicDerivation`\index{Error::HardenedPublicDerivation} | use-site path requires hardened derivation (BIP-32 forbids) | `derive::derive_address`, `to_miniscript::use_site_to_derivation_path` | `derive.rs:100, 111`, `to_miniscript.rs:122` |
| `AddressDerivationFailed { detail }`\index{Error::AddressDerivationFailed} | miniscript-layer failure or converter mismatch | `derive::derive_address`, `to_miniscript::failed` helper | `derive.rs:123, 128`, `to_miniscript.rs:475` |

(Variant count = 43.)

## V.1.5 Integration patterns

### V.1.5.1 Encoder pipeline

Policy → AST → bytecode → wrapped card string. Caller assembles a `Descriptor`, hands it to `encode_md1_string`:

- Build `Descriptor { n, path_decl, use_site_path, tree, tlv }` (or obtain one from a higher layer such as `mnemonic-toolkit`).
- Call `encode_md1_string(&d)`. Internally:
  1. `encode_payload(&d)` clones `d`, runs `canonicalize_placeholder_indices`, then runs `validate_placeholder_usage`, `validate_multipath_consistency`, `validate_tap_script_tree`.
  2. Bit-packs header, path-decl, use-site-path, tree, TLVs via the `bitstream::BitWriter`.
  3. Calls `codex32::wrap_payload(bytes, total_bits)` to attach HRP `md`, separator `1`, and the 13-symbol BCH checksum.
- For payloads exceeding `SINGLE_STRING_PAYLOAD_BIT_LIMIT = 320` bits, use `split(&d)` instead — it produces N chunked-card strings, each with a 37-bit `ChunkHeader`.

Worked invocation:

```rust
use md_codec::{Descriptor, encode_md1_string, encode_payload, split};
use md_codec::chunk::SINGLE_STRING_PAYLOAD_BIT_LIMIT;
let card = encode_md1_string(&d)?;
println!("{}", card);
// Chunk when needed:
let (_bytes, bits) = encode_payload(&d)?;
if bits > SINGLE_STRING_PAYLOAD_BIT_LIMIT {
    let cards = split(&d)?;
    for (i, c) in cards.iter().enumerate() { println!("chunk {}: {}", i, c); }
}
```

### V.1.5.2 Decoder pipeline

Card string → bytes → AST → optional address.

- `decode_md1_string(s)` is the one-shot entry point: it calls `unwrap_string` then `decode_payload`.
- `decode_payload` runs all five `validate::*` checks plus the top-level wrapper allow-list `{Sh, Wsh, Wpkh, Pkh, Tr}` (v0.31; surfaces as `OperatorContextViolation { context: TopLevel }`).
- For chunked input, collect N card strings and call `reassemble(&[&str; N])`. Reassembly re-runs `decode_payload` after concatenating per-chunk fragment bits, and re-derives `Md1EncodingId` to verify the wire-carried `chunk_set_id` agrees.
- For address derivation, call `d.derive_address(chain, index, network)` (feature `derive`). The returned `Address<NetworkUnchecked>` must be `.assume_checked()`-ed or `.require_network(...)`-ed before use.

Worked invocation:

```rust
use bitcoin::Network;
use md_codec::{decode_md1_string, reassemble};

// Single string:
let d = decode_md1_string(card_str)?;

// Chunked:
let d = reassemble(&[chunk0, chunk1, chunk2])?;

// Address (feature `derive`):
let addr = d.derive_address(0, 7, Network::Bitcoin)?.assume_checked();
```

### V.1.5.3 Chunked reassembly

`split` produces a deterministic per-`Descriptor` set of chunk strings. Each chunk carries its own BCH checksum, so per-chunk damage is locally detectable. The `chunk_set_id` shared across chunks binds them to one parent `Md1EncodingId`:

- `derive_chunk_set_id(&id) -> u32` returns the top 20 bits (MSB-first) of the 128-bit id.
- `reassemble` rejects mixed-bundle inputs via `ChunkSetInconsistent` (header fields disagree) or `ChunkSetIdMismatch` (header id ≠ derived id from reassembled bytecode).

```rust
use md_codec::{derive_chunk_set_id, compute_md1_encoding_id};
let id = compute_md1_encoding_id(&d)?;
let stub = derive_chunk_set_id(&id);
// stub will appear in every chunk header for this encoding
```

## V.1.6 Versioning and MSRV

- Crate version: **0.32.0** (HEAD `df1ed24`).
- Rust edition: **2024** (inherited from workspace `Cargo.toml`).
- MSRV: **1.85** (`rust-version` inherited from workspace).
- License: MIT.
- Public semver promise: **none**. Pre-1.0 reference implementation; any 0.X bump may break. v0.32 removed `Error::UnsupportedDerivationShape`; future bumps may rename / drop variants or types as the wire format evolves. Cargo published as part of the `descriptor-mnemonic` workspace.

## V.1.7 Notes for advanced users

- **Two declared-but-unconstructed error variants.** `Error::MalformedHeader` (`error.rs:42`) and `Error::Codex32EncodeError` (`error.rs:249`) have no construction sites in `crates/md-codec/src/` at v0.32 HEAD. They are reserved / vestigial; callers should not write `match` arms that assume they are emitted in practice. Doc-comment drift candidates for retirement.
- **`Body::MultiKeys` carries `Vec<u8>` (NOT `Vec<Node>`).** Distinct from `Body::Variable` and `Body::Children`. Walkers that recurse on every body indiscriminately will mis-walk multi-family. See §II.1 §"Body shapes".
- **`Phrase::from_id_bytes` cannot fail.** Internal `bip39::Mnemonic::from_entropy(id).expect(...)` cannot panic on 128-bit entropy; the `Result<Self, Error>` signature is API-uniform shape only. Treat it as effectively infallible.
- **`Descriptor::derive_address` returns `Address<NetworkUnchecked>`.** Callers must `.assume_checked()` or `.require_network(network)` before publishing or comparing. This is the rust-bitcoin v0.32 idiom for separating untrusted parsing from trusted use.
- **`encode_payload` self-canonicalises; `expand_per_at_n` does not.** Encoders never need to canonicalise their input first; `expand_per_at_n` callers do (or must consume a decoder output, which is canonical by construction).
- **`compute_wallet_policy_id` also self-canonicalises** (`identity.rs:172-178`). Same convention as `encode_payload`.
- **`MAX_DECODE_DEPTH = 128`** is anti-DoS hardening, not spec-mandated. Numerical coincidence with BIP-341 `TAPROOT_CONTROL_MAX_NODE_COUNT` is incidental; the cap applies generically across all recursive tags.
- **`TlvSection::unknown`** round-trips unknown TLVs verbatim per D6 forward-compat. A v0.32 decoder that meets a v0.40-tag TLV preserves the `(tag, bytes, bit_len)` triple intact through re-encoding.
- **No foreign-type re-exports.** Public signatures reference `bitcoin::*` and `miniscript::*` directly. Consumers add those crates to their own `Cargo.toml`.

## Cross-references

- §II.1 — md1 wire format (the bit-level layout these APIs encode).
- §III.1 — descriptor → miniscript → address (the three-tier derivation walk that `derive_address` performs).
- §III.2 — shape coverage (every BIP-388-parseable shape via the v0.32 AST converter).
- §IV.1 — bundle anatomy (where `md-codec`'s output sits in a three-card bundle).
- Worked example: `cargo run --quiet --manifest-path docs/technical-manual/examples/Cargo.toml --example md-codec-api-roundtrip` — source at `docs/technical-manual/examples/examples/md-codec-api-roundtrip.rs`; transcript pair at `docs/technical-manual/transcripts/md-codec-api-roundtrip.{cmd,out}`.

<!-- cspell-additions: (none — every new term is taken from the existing manual or harvest doc-comments; the harvest's review-cycle gate guarantees vocabulary alignment) -->
