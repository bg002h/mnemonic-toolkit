# ms-codec API surface harvest

| Field | Value |
|---|---|
| Crate | ms-codec |
| Version | 0.1.1 |
| Source root | /scratch/code/shibboleth/mnemonic-secret/crates/ms-codec |
| HEAD commit | c31f336954439714554863a2bdeb95ca3a3a68de |
| Rust edition | 2021 (inherited from `[workspace.package]`) |
| MSRV | 1.85 (inherited from `[workspace.package]`) |
| rust-codex32 version | `=0.1.0` (exact pin, via `[workspace.dependencies]`) |

## Feature flags

The `ms-codec` crate declares **no `[features]` table** in its `Cargo.toml` (verified `grep -n '\[features\]' Cargo.toml` returned no match). The crate has no feature-gated public surface; `cargo doc --no-deps --all-features -p ms-codec` is equivalent to `cargo doc --no-deps -p ms-codec`. No `#[cfg(feature = ...)]` annotations appear anywhere in `src/`.

The only crate-level conditional compilation is `#![cfg_attr(not(test), deny(missing_docs))]` at `src/lib.rs:38` — a test-build relaxation of the missing-docs lint, not a feature gate.

## Dependencies (public-facing only — types/traits re-exported or appearing in public signatures)

- `codex32` v`=0.1.0` (Andrew Poelstra's `rust-codex32`, CC0). Appears in the public surface via:
  - `Error::Codex32(codex32::Error)` variant (`src/error.rs:11`) — wraps upstream parse/checksum errors. (`codex32::Error` is exposed as a payload-by-value in this variant.)
  - `From<codex32::Error> for Error` impl (`src/error.rs:122-126`).

No other dependencies (workspace `Cargo.toml` declares only `codex32 = "=0.1.0"` in `[workspace.dependencies]`; the crate's `[dependencies]` is `codex32 = { workspace = true }` plus dev-only `proptest`/`bip39`/`serde`/`serde_json`).

## Public modules (top-level)

Declared at `src/lib.rs:40-46`:

- `pub mod consts;` — wire-format constants.
- `pub mod decode;` — public decoder.
- `pub mod encode;` — public encoder.
- `pub mod error;` — error taxonomy + `Result` alias.
- `pub mod inspect;` — structural inspection (less strict than `decode`).
- `pub mod payload;` — `Payload` + `PayloadKind` types.
- `pub mod tag;` — `Tag` type.

One crate-private module:

- `mod envelope;` (`src/lib.rs:48`) — "the v0.2-migration seam"; only module that contacts `rust-codex32`. Items are `pub(crate)`, not part of the public surface.

## Public surface by module

### `ms_codec` (crate root, `src/lib.rs`)

#### Re-exports (`pub use`)

- `pub use decode::decode;` (`src/lib.rs:50`)
- `pub use encode::encode;` (`src/lib.rs:51`)
- `pub use error::{Error, Result};` (`src/lib.rs:52`)
- `pub use inspect::{inspect, InspectReport};` (`src/lib.rs:53`)
- `pub use payload::{Payload, PayloadKind};` (`src/lib.rs:54`)
- `pub use tag::Tag;` (`src/lib.rs:55`)

The crate root does **not** re-export any `codex32::*` symbol. The only path by which upstream rust-codex32 types appear in this crate's public surface is `Error::Codex32(codex32::Error)` and the `From<codex32::Error> for Error` impl in `src/error.rs`.

### `ms_codec::consts` (`src/consts.rs`)

#### Constants

- `pub const HRP: &str = "ms";` (`src/consts.rs:11`) — "HRP for ms1 strings (BIP-93 codex32 HRP)."
- `pub const SEPARATOR: char = '1';` (`src/consts.rs:14`) — "BIP-93 separator character."
- `pub const RESERVED_PREFIX: u8 = 0x00;` (`src/consts.rs:17`) — "v0.1 reserved-prefix byte (becomes the v0.2 type discriminator)."
- `pub const THRESHOLD_V01: u8 = b'0';` (`src/consts.rs:20`) — "v0.1 emit-side threshold value (ASCII)."
- `pub const SHARE_INDEX_V01: u8 = b's';` (`src/consts.rs:23`) — "v0.1 emit-side share-index value (ASCII; \"s\" denotes the unshared secret per BIP-93)."
- `pub const CHECKSUM_LEN_SHORT: usize = 13;` (`src/consts.rs:26`) — "Short codex32 checksum length in characters."
- `pub const VALID_ENTR_LENGTHS: &[usize] = &[16, 20, 24, 28, 32];` (`src/consts.rs:29`) — "Allowed v0.1 entr entropy byte lengths (bijective with BIP-39 word counts {12,15,18,21,24})."
- `pub const VALID_STR_LENGTHS: &[usize] = &[50, 56, 62, 69, 75];` (`src/consts.rs:33`) — "Allowed v0.1 total ms1 string lengths (HRP+sep+threshold+id+share+payload+cksum)."
- `pub const TAG_ENTR: [u8; 4] = *b"entr";` (`src/consts.rs:36`) — "4-byte type tag — v0.1 emit (also accept)."
- `pub const RESERVED_NOT_EMITTED_V01: &[[u8; 4]] = &[*b"seed", *b"xprv", *b"mnem", *b"prvk"];` (`src/consts.rs:39`) — "4-byte type tags reserved-not-emitted in v0.1 (decoder rejects)."

(No functions, types, or traits in this module.)

### `ms_codec::decode` (`src/decode.rs`)

#### Functions

- `pub fn decode(s: &str) -> Result<(Tag, Payload)>` (`src/decode.rs:19`) — "Decode a v0.1 ms1 string into `(Tag, Payload)`." Applies SPEC §4 validity rules 1–10 in order: string-length check, upstream BIP-93 parse + checksum, wire-invariant checks via `envelope::discriminate`, reserved-not-emitted-tag check, accept-set membership (currently `{entr}`), and `Payload::validate()` for byte-length match.

### `ms_codec::encode` (`src/encode.rs`)

#### Functions

- `pub fn encode(tag: Tag, payload: &Payload) -> Result<String>` (`src/encode.rs:16`) — "Encode a `(Tag, Payload)` as a v0.1 ms1 string." Per SPEC §3.5 + §3.5.1: rejects reserved-not-emitted tags symmetrically with the decoder, then validates the payload, then delegates to `envelope::package`.

### `ms_codec::error` (`src/error.rs`)

#### Types

- `pub enum Error` (`src/error.rs:9`) — "ms-codec error type." `#[derive(Debug)]`, `#[non_exhaustive]`. Variants enumerated in the Error taxonomy table below.

#### Trait impls (public, on `Error`)

- `impl fmt::Display for Error` (`src/error.rs:66-113`).
- `impl std::error::Error for Error` (`src/error.rs:115-120`) — `source()` always returns `None` ("codex32::Error doesn't impl std::error::Error in v0.1.0; chain stops here").
- `impl From<codex32::Error> for Error` (`src/error.rs:122-126`) — produces `Error::Codex32(e)`.

#### Type aliases

- `pub type Result<T> = std::result::Result<T, Error>;` (`src/error.rs:129`) — "Result alias for ms-codec."

### `ms_codec::inspect` (`src/inspect.rs`)

#### Types

- `pub struct InspectReport` (`src/inspect.rs:13`) — "Structural dump of a parsed ms1 string." `#[derive(Debug, Clone)]`, `#[non_exhaustive]`. Fields, all public:
  - `pub hrp: String` (`:15`) — "Expected \"ms\" in v0.1."
  - `pub threshold: u8` (`:17`) — "Expected 0 in v0.1." (Stored as digit value after `- b'0'`, not ASCII.)
  - `pub tag: Tag` (`:19`) — "The parsed type tag (id field)."
  - `pub share_index: char` (`:21`) — "Expected 's' in v0.1."
  - `pub prefix_byte: u8` (`:23`) — "0x00 in v0.1 (reserved); becomes type discriminator in v0.2+."
  - `pub payload_bytes: Vec<u8>` (`:25`) — "Payload bytes after the prefix byte."
  - `pub checksum_valid: bool` (`:27`) — "BCH verification result. True if the upstream codex32 parser accepted."

#### Functions

- `pub fn inspect(s: &str) -> Result<InspectReport>` (`src/inspect.rs:34`) — "Inspect an ms1 string. Less strict than `decode()`: returns a report even for strings that would fail decoder validity rules (e.g., wrong threshold, reserved-not-emitted tag, non-zero prefix byte) — caller can examine the fields to diagnose what's wrong. Still requires a valid BIP-93 parse."

### `ms_codec::payload` (`src/payload.rs`)

#### Types

- `pub enum PayloadKind` (`src/payload.rs:11`) — "v0.1 payload kind. Future kinds (Mnem, Seed, Xprv) will arrive in v0.2+." `#[derive(Debug, Clone, Copy, PartialEq, Eq)]`, `#[non_exhaustive]`. Variants:
  - `Entr` (`:13`) — "BIP-39 entropy (16/20/24/28/32 B)."
- `pub enum Payload` (`src/payload.rs:19`) — "v0.1 payload." `#[derive(Debug, Clone, PartialEq, Eq)]`, `#[non_exhaustive]`. Variants:
  - `Entr(Vec<u8>)` (`:29`) — "BIP-39 entropy. Length MUST be in {16, 20, 24, 28, 32} bytes (bijective with BIP-39 word counts {12, 15, 18, 21, 24})." Doc-comment includes caller-responsibility caveat: ms-codec does not check statistical quality of bytes.

#### Methods on `Payload`

- `pub fn validate(&self) -> Result<()>` (`src/payload.rs:36`) — "Validate the payload's intrinsic structure (byte length for Entr). Encoder MUST call this before emitting; decoder calls it after extracting the payload bytes following the reserved-prefix byte."
- `pub fn kind(&self) -> PayloadKind` (`src/payload.rs:52`) — "The PayloadKind discriminant."
- `pub fn as_bytes(&self) -> &[u8]` (`src/payload.rs:59`) — "Borrow the inner byte slice."

### `ms_codec::tag` (`src/tag.rs`)

#### Types

- `pub struct Tag([u8; 4]);` (`src/tag.rs:12`) — "4-byte type tag. Field is private to enforce validated construction via `try_new` (alphabet-checked) or `from_raw_bytes` (tooling-only, unvalidated)." `#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]`. Tuple field is **not** public.

#### Associated constants on `Tag`

- `pub const ENTR: Tag = Tag(TAG_ENTR);` (`src/tag.rs:16`) — "The v0.1 emit-tag for BIP-39 entropy."

#### Methods on `Tag`

- `pub fn from_raw_bytes(b: [u8; 4]) -> Self` (`src/tag.rs:22`) — "Construct a Tag from raw 4-byte input WITHOUT alphabet validation. Reserved for tooling (e.g., `inspect()`) that needs to surface whatever bytes were observed on the wire, including alphabet violators. Encoder + decoder paths MUST go through `try_new` instead."
- `pub fn try_new(s: &str) -> Result<Self>` (`src/tag.rs:28`) — "Construct a Tag from a 4-character string slice. Returns `Error::TagInvalidAlphabet` if any character is outside the codex32 alphabet." (Internal alphabet: `b"qpzry9x8gf2tvdw0s3jn54khce6mua7l"`, BIP-173 lowercase bech32 charset; the constant itself is module-private.)
- `pub fn as_bytes(&self) -> &[u8; 4]` (`src/tag.rs:49`) — "Borrow the underlying 4 bytes."
- `pub fn as_str(&self) -> &str` (`src/tag.rs:56`) — "View the tag as a string slice. Always succeeds for `try_new`-constructed tags (codex32 alphabet is ASCII); for `from_raw_bytes`-constructed tags containing non-UTF-8 bytes, returns \"<non-utf8>\"."

(No `From`/`Display`/`AsRef`/`Hash`-besides-derive trait impls. No `Default`, no `PartialOrd`/`Ord`.)

## Error taxonomy

All variants live in `pub enum Error` at `src/error.rs:9` (`#[non_exhaustive]`).

| Variant | Doc-comment | Emitted by (functions) |
|---|---|---|
| `Codex32(codex32::Error)` (`:11`) | "Upstream codex32 parse / checksum failure (delegated from rust-codex32)." | `decode::decode` (via `?` on `Codex32String::from_string`, `src/decode.rs:30`); `inspect::inspect` (via `?` on `Codex32String::from_string`, `src/inspect.rs:36`); `envelope::package` (via `?` on `Codex32String::from_seed`, `src/envelope.rs:149`); generally produced via `From<codex32::Error> for Error` (`src/error.rs:122`). |
| `WrongHrp { got: String }` (`:13`) | "HRP was not \"ms\" (SPEC §4 rule 2)." | `envelope::discriminate` (`src/envelope.rs:96`); `envelope::extract_wire_fields` returns it as a "no separator" fallback (`src/envelope.rs:62`). |
| `ThresholdNotZero { got: u8 }` (`:18`) | "Threshold was not 0 (SPEC §4 rule 3)." | `envelope::discriminate` (`src/envelope.rs:101`). |
| `ShareIndexNotSecret { got: char }` (`:23`) | "Share-index was not 's' — BIP-93 requires 's' for threshold=0 (SPEC §4 rule 4)." | `envelope::discriminate` (`src/envelope.rs:106`). |
| `TagInvalidAlphabet { got: [u8; 4] }` (`:28`) | "Tag bytes were not in the codex32 alphabet (SPEC §4 rule 5)." | `Tag::try_new` (`src/tag.rs:34, 39`); `envelope::discriminate` (`src/envelope.rs:114`, via `map_err` on the `from_utf8` of the id bytes). |
| `UnknownTag { got: [u8; 4] }` (`:33`) | "Tag was structurally valid but not in RESERVED_TAG_TABLE (SPEC §4 rule 6)." | `decode::decode` (`src/decode.rs:51`). |
| `ReservedTagNotEmittedInV01 { got: [u8; 4] }` (`:39`) | "Tag was in RESERVED_TAG_TABLE but reserved-not-emitted in v0.1 (SPEC §4 rule 7, SPEC §3.5.1 encoder symmetry)." | `decode::decode` (`src/decode.rs:37`); `encode::encode` (`src/encode.rs:19`). |
| `ReservedPrefixViolation { got: u8 }` (`:44`) | "Reserved-prefix byte was not 0x00 (SPEC §4 rule 8)." | `envelope::discriminate` (`src/envelope.rs:126`). |
| `UnexpectedStringLength { got: usize, allowed: &'static [usize] }` (`:49`) | "Total string length was outside the v0.1 emittable set (SPEC §4 rule 9)." | `decode::decode` (`src/decode.rs:22`); `envelope::extract_wire_fields` (`src/envelope.rs:67`, defensive). |
| `PayloadLengthMismatch { tag: [u8; 4], expected: &'static [usize], got: usize }` (`:56`) | "Payload byte length did not match the tag's spec (SPEC §3.5, §4 rule 10)." | `Payload::validate` (`src/payload.rs:40`); transitively by `encode::encode` and `decode::decode` via the `payload.validate()?` call sites. |

## Feature-gated items

None. The crate has no `[features]` table and no `#[cfg(feature = ...)]` items.

## Notes for chapter author (Phase 4.3)

- **ms-codec is library-only at this layer.** `crates/ms-cli` is a separate workspace member (binary name `ms`, depends on `ms-codec = "=0.1.1"`). It is **out of scope** per the harvest brief and intentionally not enumerated above.
- **rust-codex32 delegation is narrow and crate-private.** All upstream contact happens inside `mod envelope` (declared `mod envelope;` — non-`pub` — at `src/lib.rs:48`). The public surface contains no `pub use codex32::*` re-export. The only direct rust-codex32 type that leaks into the public API is `codex32::Error`, which appears solely as the payload of `Error::Codex32` and through the `From<codex32::Error> for Error` impl. The chapter draft does **not** need to document `codex32::*` symbols transitively — `rust-codex32` is an internal implementation detail except for that single error wrapping.
- **No v0.3-lesson false claims found.** The v0.3 lesson flagged that `rust-codex32` v0.1.0 does not expose `Codex32String::shares` (only `interpolate_at` for reconstruction). The ms-codec public surface makes no share-generation claim and contains no API named `shares`, `share`, `split`, or similar; v0.1 of ms1 is explicitly single-string only (threshold=0, share-index='s' hard-coded in `consts.rs` and enforced by `envelope::discriminate`). The K-of-N share-encoding work is "deferred to v0.2+" per `src/payload.rs:7-8` and `src/error.rs:18` ("threshold not 0 … v0.1 is single-string only"). Nothing in the harvested surface needs correction on this axis.
- **Crate-private "v0.2-migration seam".** `mod envelope` is documented in `src/envelope.rs:1-27` as "THE v0.2-MIGRATION SEAM. This is the only module that contacts `rust-codex32`." It declares `pub(crate)` items (`WireFields`, `extract_wire_fields`, `discriminate`, `package`). These are **not** part of the public API; the chapter should mention that the crate isolates v0.2 wire-format evolution to this single module but should not enumerate its items as public surface.
- **`#[non_exhaustive]` discipline.** `Error`, `Payload`, `PayloadKind`, and `InspectReport` are all `#[non_exhaustive]`. Downstream pattern-matchers must use a wildcard arm; downstream constructors of `Payload`/`PayloadKind`/`InspectReport` cannot brace-initialize from outside the crate. (Match arms on `Error` outside the crate likewise need `_`.)
- **Versioning conventions; pre-1.0 semver promises.** Crate is v0.1.1, pre-1.0. No declared semver-stability commitment is visible in `Cargo.toml` or `src/lib.rs` doc-comments. The SPEC pointer at `src/lib.rs:9-11` references `design/SPEC_ms_v0_1.md` and `MIGRATION.md` (v0.1 → v0.2 K-of-N share-encoding migration contract); the chapter author should consult those docs (out of harvest scope) for cross-version compatibility commitments.
- **Doc-comment lint warning (informational, not a surface item).** `cargo doc --no-deps --all-features -p ms-codec` emits one warning: `rustdoc::invalid_html_tags` on `src/tag.rs:55` for the literal `<non-utf8>` in the `as_str` doc-comment (rustdoc reads it as an HTML tag). The function behavior is correct; the doc-comment renders with the angle-bracket literal stripped. Not a public-surface issue, but the chapter draft may want to quote `as_str`'s sentinel as `\"<non-utf8>\"` (escaped) to avoid the same problem.
- **Crate-level lint posture.** `#![cfg_attr(not(test), deny(missing_docs))]` (`src/lib.rs:38`) — every public item is documented in non-test builds, which is why every entry above has a doc-comment first line. The harvest can be trusted as complete in that sense: any undocumented public item would fail to compile.
