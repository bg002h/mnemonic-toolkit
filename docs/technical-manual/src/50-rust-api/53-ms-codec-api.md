# ms-codec Rust API

This chapter is the reference for the `ms-codec`\index{ms-codec} crate's public surface at v0.1.1\index{ms-codec v0.1.1} (HEAD `c31f336` in `bg002h/mnemonic-secret`). It enumerates every public module, function, type, constant, and error variant. The wire format these APIs encode/decode is §II.3; this chapter is the library API only — the `ms-cli` binary lives in a sibling crate and its surface is covered in the end-user manual, not Part V. For the normative wire spec, see `mnemonic-secret/design/SPEC_ms_v0_1.md` and the in-tree BIP draft, plus `mnemonic-secret/MIGRATION.md` for the v0.1 → v0.2 K-of-N share-encoding migration contract.

## V.3.1 Crate purpose

`ms-codec`\index{ms\_codec (crate)} is the reference encoder/decoder for the ms1 secret-card format (HRP `ms`). The crate's v0.1 scope is BIP-39 entropy only: 16, 20, 24, 28, or 32 raw bytes (bijective with BIP-39 word counts 12, 15, 18, 21, 24), wrapped as a single codex32 string with HRP `ms`, threshold position `0`, and share-index `s` (BIP-93's "the unshared secret"). Unlike `md-codec`\index{md-codec} and `mk-codec`\index{mk-codec} — both of which fork BIP-93's BCH primitives onto HRP-mixed target residues — `ms-codec` adopts BIP-93 codex32 **directly** via Andrew Poelstra's `rust-codex32`\index{rust-codex32} crate (`=0.1.0`, CC0). The fork-vs-direct asymmetry is deliberate: md1↔mk1's HRP-mixing isn't upstreamable to `rust-codex32`, but ms1's wire is literal BIP-93 plus an envelope, so the upstream crate fits without modification.

`ms-codec` wraps `rust-codex32` with three crate-private modules (`envelope`, `payload`, `tag`) plus public encoder/decoder/inspector entry points. The crate is library-only — `ms-cli` is a sibling binary crate out of Part V scope. v0.1 is single-string only (`threshold = 0`); K-of-N share encoding is **locked-in for v0.2+** but deferred — ms1 ships shares first across the m-format-star (BIP-93 already specifies the math; md1↔mk1 wait on their own forked-BCH share work). Pre-1.0 reference status; the v0.1 wire format is locked, but the Rust API may shift on any 0.X bump.

## V.3.2 Feature flags

**None.** The crate's `Cargo.toml` declares no `[features]` table; `grep -n '\[features\]'` on the manifest returns no match, and there are no `#[cfg(feature = ...)]` attributes anywhere under `src/`. `cargo doc --no-deps --all-features -p ms-codec` is equivalent to `cargo doc --no-deps -p ms-codec`.

The only crate-level conditional compilation is `#![cfg_attr(not(test), deny(missing_docs))]` at `crates/ms-codec/src/lib.rs:38` — a test-build relaxation of the missing-docs lint, not a feature gate. The empty feature surface is a deliberate v0.1 choice: there is no optional-derivation tier, no optional vector-generator binary, and no optional `serde` impl. The migration prefix-byte plumbing for v0.2 lives inside the crate-private `envelope` module (§V.3.7), not behind a flag.

```toml
ms-codec = "0.1"
```

## V.3.3 Public API by module

Seven public modules (`consts`, `decode`, `encode`, `error`, `inspect`, `payload`, `tag`) plus one crate-private module (`envelope`). Re-exports at the crate root pull the most commonly-used items into `ms_codec::`:

```rust
pub use decode::decode;
pub use encode::encode;
pub use error::{Error, Result};
pub use inspect::{inspect, InspectReport};
pub use payload::{Payload, PayloadKind};
pub use tag::Tag;
```

(`crates/ms-codec/src/lib.rs:50-55`.) The crate root re-exports **no** `codex32::*` symbol. The only path by which `rust-codex32` types reach this crate's public surface is `Error::Codex32(codex32::Error)` plus the `From<codex32::Error> for Error` impl (§V.3.4, §V.3.7). Consumers needing `codex32::Codex32String` itself add `codex32 = "=0.1.0"` separately — but they should not need to: the crate's `inspect` + `decode` entry points cover every diagnostic and round-trip use case the wrapper is meant to expose.

The crate-private `mod envelope;` (`src/lib.rs:48`) is the **v0.2-migration seam**: it is the only module that contacts `rust-codex32`, and every wire-invariant check (HRP, threshold, share-index, reserved-prefix byte) runs there. Its items (`WireFields`, `extract_wire_fields`, `discriminate`, `package`) are `pub(crate)`, not part of the public surface, and are intentionally undocumented in this chapter. See §V.3.7 for the v0.1 → v0.2 contract.

### V.3.3.1 `consts`\index{ms\_codec::consts}

Crate-wide constants for the ms1 wire format (`crates/ms-codec/src/consts.rs`).

| Item | Signature | Semantics | Source |
|---|---|---|---|
| `HRP`\index{HRP (ms1)} | `pub const HRP: &str = "ms"` | ms1 HRP (BIP-93 codex32 HRP) | `consts.rs:11` |
| `SEPARATOR`\index{SEPARATOR (bech32)} | `pub const SEPARATOR: char = '1'` | BIP-93 separator character | `consts.rs:14` |
| `RESERVED_PREFIX`\index{RESERVED\_PREFIX (ms1)} | `pub const RESERVED_PREFIX: u8 = 0x00` | v0.1 reserved-prefix byte; becomes the v0.2 type discriminator | `consts.rs:17` |
| `THRESHOLD_V01`\index{THRESHOLD\_V01} | `pub const THRESHOLD_V01: u8 = b'0'` | v0.1 emit-side threshold value (ASCII `'0'`) | `consts.rs:20` |
| `SHARE_INDEX_V01`\index{SHARE\_INDEX\_V01} | `pub const SHARE_INDEX_V01: u8 = b's'` | v0.1 emit-side share-index value (ASCII `'s'` per BIP-93 "the unshared secret") | `consts.rs:23` |
| `CHECKSUM_LEN_SHORT`\index{CHECKSUM\_LEN\_SHORT} | `pub const CHECKSUM_LEN_SHORT: usize = 13` | short codex32 checksum length in characters | `consts.rs:26` |
| `VALID_ENTR_LENGTHS`\index{VALID\_ENTR\_LENGTHS} | `pub const VALID_ENTR_LENGTHS: &[usize] = &[16, 20, 24, 28, 32]` | allowed v0.1 entr entropy byte lengths (bijective with BIP-39 word counts {12,15,18,21,24}) | `consts.rs:29` |
| `VALID_STR_LENGTHS`\index{VALID\_STR\_LENGTHS} | `pub const VALID_STR_LENGTHS: &[usize] = &[50, 56, 62, 69, 75]` | allowed total ms1 string lengths (HRP + sep + threshold + id + share + payload + cksum) | `consts.rs:33` |
| `TAG_ENTR`\index{TAG\_ENTR} | `pub const TAG_ENTR: [u8; 4] = *b"entr"` | 4-byte type tag — v0.1 emit (also accept) | `consts.rs:36` |
| `RESERVED_NOT_EMITTED_V01`\index{RESERVED\_NOT\_EMITTED\_V01} | `pub const RESERVED_NOT_EMITTED_V01: &[[u8; 4]] = &[*b"seed", *b"xprv", *b"mnem", *b"prvk"]` | 4-byte type tags reserved-not-emitted in v0.1 (decoder + encoder reject) | `consts.rs:39` |

(No functions, types, or traits in this module.)

### V.3.3.2 `decode`\index{ms\_codec::decode}

Top-level decoder (`crates/ms-codec/src/decode.rs`).

| Item | Signature | Semantics | Source |
|---|---|---|---|
| `decode`\index{decode (ms-codec)} | `fn decode(s: &str) -> Result<(Tag, Payload)>` | decode a v0.1 ms1 string into `(Tag, Payload)`. Applies SPEC §4 validity rules 1–10 in order: string-length check, upstream BIP-93 parse + checksum, wire-invariant checks via `envelope::discriminate`, reserved-not-emitted-tag rejection, accept-set membership (currently `{entr}` only), and `Payload::validate()` for byte-length match | `decode.rs:19` |

```rust
use ms_codec::{decode, Payload, Tag};
let (tag, payload) = decode("ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f")?;
assert_eq!(tag, Tag::ENTR);
if let Payload::Entr(bytes) = payload {
    assert_eq!(bytes.len(), 16);  // 12-word BIP-39 entropy
}
```

### V.3.3.3 `encode`\index{ms\_codec::encode}

Top-level encoder (`crates/ms-codec/src/encode.rs`).

| Item | Signature | Semantics | Source |
|---|---|---|---|
| `encode`\index{encode (ms-codec)} | `fn encode(tag: Tag, payload: &Payload) -> Result<String>` | encode a `(Tag, Payload)` as a v0.1 ms1 string. Per SPEC §3.5 + §3.5.1: rejects reserved-not-emitted tags symmetrically with the decoder, then validates the payload, then delegates to `envelope::package` | `encode.rs:16` |

```rust
use ms_codec::{encode, Payload, Tag};
let entropy = vec![0xAAu8; 16];
let s = encode(Tag::ENTR, &Payload::Entr(entropy))?;
assert_eq!(s.len(), 50);   // 12-word entr ⇒ 50-char ms1 string
```

### V.3.3.4 `error`\index{ms\_codec::error}

Error taxonomy (10 variants; full table in §V.3.4). Two public types and three trait impls:

| Item | Signature | Semantics | Source |
|---|---|---|---|
| `Error`\index{Error (ms-codec)} | `pub enum Error { ... }` (`#[derive(Debug)]`, `#[non_exhaustive]`) | 10 variants | `error.rs:9` |
| `Result<T>`\index{Result (ms-codec)} | `pub type Result<T> = std::result::Result<T, Error>` | crate alias | `error.rs:129` |
| `impl fmt::Display for Error` | — | one-line message per variant | `error.rs:66-113` |
| `impl std::error::Error for Error` | — | `source()` always returns `None` (see §V.3.7) | `error.rs:115-120` |
| `impl From<codex32::Error> for Error` | `fn from(e: codex32::Error) -> Self` | produces `Error::Codex32(e)`; lets `?` lift upstream errors at every codex32 call site | `error.rs:122-126` |

### V.3.3.5 `inspect`\index{ms\_codec::inspect}

Structural inspection — strictly less strict than `decode` (`crates/ms-codec/src/inspect.rs`).

| Item | Signature | Semantics | Source |
|---|---|---|---|
| `InspectReport`\index{InspectReport} | `pub struct InspectReport { pub hrp: String, pub threshold: u8, pub tag: Tag, pub share_index: char, pub prefix_byte: u8, pub payload_bytes: Vec<u8>, pub checksum_valid: bool }` (`#[derive(Debug, Clone)]`, `#[non_exhaustive]`) | structural dump of a parsed ms1 string | `inspect.rs:13` |
| `inspect`\index{inspect} | `fn inspect(s: &str) -> Result<InspectReport>` | inspect an ms1 string. Returns a report even for strings that would fail decoder validity rules (wrong threshold, reserved-not-emitted tag, non-zero prefix byte); caller examines fields to diagnose. Still requires a valid BIP-93 parse | `inspect.rs:34` |

`InspectReport` fields, all public (`inspect.rs:15-27`):

- `hrp: String` — expected `"ms"` in v0.1.
- `threshold: u8` — expected `0` in v0.1; stored as digit value (after `- b'0'`), not ASCII.
- `tag: Tag` — the parsed type tag (id field).
- `share_index: char` — expected `'s'` in v0.1.
- `prefix_byte: u8` — `0x00` in v0.1 (reserved); becomes the type discriminator in v0.2+.
- `payload_bytes: Vec<u8>` — payload bytes after the prefix byte.
- `checksum_valid: bool` — BCH verification result. `true` if the upstream codex32 parser accepted.

```rust
use ms_codec::inspect;
let rep = inspect(suspect_str)?;
if rep.threshold != 0 {
    eprintln!("threshold = {} (expected 0)", rep.threshold);
}
if rep.prefix_byte != 0x00 {
    eprintln!("reserved-prefix byte = 0x{:02x} (expected 0x00)", rep.prefix_byte);
}
```

### V.3.3.6 `payload`\index{ms\_codec::payload}

`Payload` + `PayloadKind` types (`crates/ms-codec/src/payload.rs`).

| Item | Signature | Semantics | Source |
|---|---|---|---|
| `PayloadKind`\index{PayloadKind} | `pub enum PayloadKind { Entr }` (`#[derive(Debug, Clone, Copy, PartialEq, Eq)]`, `#[non_exhaustive]`) | v0.1 payload kind. Future kinds (Mnem, Seed, Xprv) arrive in v0.2+ | `payload.rs:11` |
| `Payload`\index{Payload (ms-codec)} | `pub enum Payload { Entr(Vec<u8>) }` (`#[derive(Debug, Clone, PartialEq, Eq)]`, `#[non_exhaustive]`) | v0.1 payload | `payload.rs:19` |
| `Payload::validate`\index{Payload::validate} | `fn validate(&self) -> Result<()>` | validate intrinsic structure (byte length for Entr). Encoder MUST call before emitting; decoder calls after extracting payload bytes following the reserved-prefix byte | `payload.rs:36` |
| `Payload::kind`\index{Payload::kind} | `fn kind(&self) -> PayloadKind` | the `PayloadKind` discriminant | `payload.rs:52` |
| `Payload::as_bytes`\index{Payload::as\_bytes} | `fn as_bytes(&self) -> &[u8]` | borrow the inner byte slice | `payload.rs:59` |

The `Entr(Vec<u8>)` variant doc-comment includes a caller-responsibility caveat (`payload.rs:29`): ms-codec validates byte length only; it does **not** check the statistical quality of the bytes. Callers feeding entropy from non-OS-CSPRNG sources are responsible for randomness assurance.

### V.3.3.7 `tag`\index{ms\_codec::tag}

`Tag` type — 4-byte type discriminator (`crates/ms-codec/src/tag.rs`).

| Item | Signature | Semantics | Source |
|---|---|---|---|
| `Tag`\index{Tag (ms-codec)} | `pub struct Tag([u8; 4])` (`#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]`) | 4-byte type tag. Field is private to enforce validated construction via `try_new` (alphabet-checked) or `from_raw_bytes` (tooling-only, unvalidated). Tuple field is **not** public | `tag.rs:12` |
| `Tag::ENTR`\index{Tag::ENTR} | `pub const ENTR: Tag = Tag(TAG_ENTR)` | the v0.1 emit-tag for BIP-39 entropy | `tag.rs:16` |
| `Tag::from_raw_bytes`\index{Tag::from\_raw\_bytes} | `fn from_raw_bytes(b: [u8; 4]) -> Self` | construct from raw 4 bytes **without** alphabet validation. Reserved for tooling (e.g. `inspect()`) that needs to surface whatever bytes appeared on the wire, alphabet violators included. Encoder + decoder paths MUST use `try_new` | `tag.rs:22` |
| `Tag::try_new`\index{Tag::try\_new} | `fn try_new(s: &str) -> Result<Self>` | construct from a 4-character string slice. Returns `Error::TagInvalidAlphabet` if any character is outside the codex32 alphabet (`b"qpzry9x8gf2tvdw0s3jn54khce6mua7l"`, BIP-173 lowercase bech32 charset) | `tag.rs:28` |
| `Tag::as_bytes`\index{Tag::as\_bytes} | `fn as_bytes(&self) -> &[u8; 4]` | borrow the underlying 4 bytes | `tag.rs:49` |
| `Tag::as_str`\index{Tag::as\_str} | `fn as_str(&self) -> &str` | view the tag as a string slice. Always succeeds for `try_new`-constructed tags (codex32 alphabet is ASCII); for `from_raw_bytes`-constructed tags containing non-UTF-8 bytes, returns the literal sentinel `"\<non-utf8\>"` | `tag.rs:56` |

No `From`, `Display`, `AsRef`, or non-derived `Hash` impls. No `Default`, `PartialOrd`, or `Ord`.

```rust
use ms_codec::Tag;
let t = Tag::try_new("entr")?;
assert_eq!(t, Tag::ENTR);
assert_eq!(t.as_str(), "entr");
let bad = Tag::try_new("ENTR");  // upper-case violates codex32 alphabet
assert!(bad.is_err());
```

## V.3.4 Error taxonomy

`pub enum Error` from `crates/ms-codec/src/error.rs` — 10 variants (lines 9-64), `#[derive(Debug)]`, `#[non_exhaustive]`. Order matches the source. Every variant corresponds to one SPEC §4 validity rule (or the encoder-symmetry §3.5.1 mirror); the rule mapping is preserved verbatim from doc-comments.

| Variant | Display | Emitted by | Source |
|---|---|---|---|
| `Codex32(codex32::Error)`\index{Error::Codex32} | `codex32 parse error: {0:?}` | `decode::decode` (via `?` on `Codex32String::from_string`, `decode.rs:30`); `inspect::inspect` (`inspect.rs:36`); `envelope::package` (`envelope.rs:149`); auto-lifted by `From<codex32::Error> for Error` (`error.rs:122`) | `error.rs:11` |
| `WrongHrp { got: String }`\index{Error::WrongHrp} | `wrong HRP: got {got:?}, expected "ms"` (SPEC §4 rule 2) | `envelope::discriminate` (`envelope.rs:96`); `envelope::extract_wire_fields` no-separator fallback (`envelope.rs:62`) | `error.rs:13` |
| `ThresholdNotZero { got: u8 }`\index{Error::ThresholdNotZero} | `threshold not 0 (got '{got as char}'); v0.1 is single-string only` (SPEC §4 rule 3) | `envelope::discriminate` (`envelope.rs:101`) | `error.rs:18` |
| `ShareIndexNotSecret { got: char }`\index{Error::ShareIndexNotSecret} | `share-index not 's' (got '{got}'); BIP-93 requires 's' for threshold=0` (SPEC §4 rule 4) | `envelope::discriminate` (`envelope.rs:106`) | `error.rs:23` |
| `TagInvalidAlphabet { got: [u8; 4] }`\index{Error::TagInvalidAlphabet} | `tag bytes not in codex32 alphabet: {got:?}` (SPEC §4 rule 5) | `Tag::try_new` (`tag.rs:34, 39`); `envelope::discriminate` (`envelope.rs:114`) | `error.rs:28` |
| `UnknownTag { got: [u8; 4] }`\index{Error::UnknownTag} | `unknown tag {got:?}; not a member of RESERVED_TAG_TABLE` (SPEC §4 rule 6) | `decode::decode` (`decode.rs:51`) | `error.rs:33` |
| `ReservedTagNotEmittedInV01 { got: [u8; 4] }`\index{Error::ReservedTagNotEmittedInV01} | `tag {got:?} reserved-not-emitted in v0.1; deferred to v0.2+` (SPEC §4 rule 7, §3.5.1 encoder symmetry) | `decode::decode` (`decode.rs:37`); `encode::encode` (`encode.rs:19`) | `error.rs:39` |
| `ReservedPrefixViolation { got: u8 }`\index{Error::ReservedPrefixViolation} | `reserved-prefix byte was 0x{got:02x}, expected 0x00` (SPEC §4 rule 8) | `envelope::discriminate` (`envelope.rs:126`) | `error.rs:44` |
| `UnexpectedStringLength { got: usize, allowed: &'static [usize] }`\index{Error::UnexpectedStringLength} | `string length {got} outside v0.1 set {allowed:?}` (SPEC §4 rule 9) | `decode::decode` (`decode.rs:22`); `envelope::extract_wire_fields` defensive (`envelope.rs:67`) | `error.rs:49` |
| `PayloadLengthMismatch { tag: [u8; 4], expected: &'static [usize], got: usize }`\index{Error::PayloadLengthMismatch} | `tag {tag:?} payload length {got} not in expected set {expected:?}` (SPEC §3.5, §4 rule 10) | `Payload::validate` (`payload.rs:40`); transitively by `encode::encode` and `decode::decode` via the `payload.validate()?` call sites | `error.rs:56` |

(Variant count = 10.) The `From<codex32::Error> for Error` impl at `error.rs:122` is the **only** path by which `rust-codex32`'s own error type bleeds into the public API surface: `Error::Codex32` carries it as a payload-by-value. See §V.3.7.

## V.3.5 Integration patterns

### V.3.5.1 Encoder pipeline

`(Tag, Payload)` → wire-invariant + payload-length validation → BIP-93 codex32 string.

- Build a `Payload` (currently only `Payload::Entr(Vec<u8>)`) and pair it with a `Tag` (currently only `Tag::ENTR`, since `RESERVED_NOT_EMITTED_V01` rejects `seed`, `xprv`, `mnem`, `prvk` at the encoder boundary per SPEC §3.5.1).
- Call `encode(tag, &payload)`. Internally:
  1. Reject reserved-not-emitted tags (§3.5.1 encoder symmetry; produces `ReservedTagNotEmittedInV01`).
  2. Run `payload.validate()` (rejects out-of-set entr lengths with `PayloadLengthMismatch`).
  3. Delegate to `envelope::package` (crate-private), which prepends the `0x00` reserved-prefix byte and feeds the result to `codex32::Codex32String::from_seed` with HRP `"ms"`, threshold `'0'`, share-index `'s'`, and the 4-byte tag as the id field.
- No chunking layer exists at v0.1 — the longest emittable string is 75 characters (32-byte entr), well inside BIP-93's short-code length bracket.

Worked invocation:

```rust
use ms_codec::{encode, Payload, Tag};
let entropy = vec![0xAAu8; 32];    // 24-word BIP-39 entropy
let s = encode(Tag::ENTR, &Payload::Entr(entropy))?;
assert_eq!(s.len(), 75);           // 24-word entr ⇒ 75-char ms1 string
println!("{}", s);
```

### V.3.5.2 Decoder pipeline

ms1 card string → BIP-93 parse + checksum → wire-invariant check → tag-table membership → payload-length validation → `(Tag, Payload)`.

- `decode(s)` is the single entry point. It:
  1. Checks total string length is in `VALID_STR_LENGTHS` (rejects with `UnexpectedStringLength`).
  2. Hands off to `codex32::Codex32String::from_string` for parse + BCH checksum (any failure surfaces as `Error::Codex32` via the `From` impl).
  3. Calls `envelope::discriminate` to enforce HRP, threshold, share-index, tag-alphabet, and reserved-prefix byte invariants (SPEC §4 rules 2–5, 8).
  4. Looks up the tag against `RESERVED_NOT_EMITTED_V01` (rejects with `ReservedTagNotEmittedInV01`) and against the v0.1 accept set `{entr}` (rejects with `UnknownTag` for any structurally-valid tag outside the accept set).
  5. Constructs `Payload::Entr(bytes)` from the bytes after the prefix byte and calls `payload.validate()` (catches `PayloadLengthMismatch`).
- For diagnostic-only use, prefer `inspect(s)` — it returns an `InspectReport` even when `decode` would reject (it still requires a valid BIP-93 parse, but skips wire-invariant + tag-table + length checks).

Worked invocation:

```rust
use ms_codec::{decode, inspect, Payload, Tag};

// Strict decode:
let (tag, payload) = decode(card_str)?;
assert_eq!(tag, Tag::ENTR);
let bytes: &[u8] = match &payload { Payload::Entr(b) => b, _ => unreachable!() };

// Loose inspect (no rule 2/3/4/8/9/10 enforcement):
let rep = inspect(suspect_str)?;
println!("tag = {}, threshold = {}, prefix = 0x{:02x}, checksum_valid = {}",
         rep.tag.as_str(), rep.threshold, rep.prefix_byte, rep.checksum_valid);
```

### V.3.5.3 No chunked reassembly

ms1 v0.1 is **single-string only** (`threshold = 0`, `share-index = 's'` hard-coded). There is no `split` / `reassemble` API analogous to `mk-codec::chunk` or `md-codec::chunk`. Every emittable ms1 string fits in BIP-93's short-code length bracket (max 75 characters at 32-byte entr); chunking is unnecessary and would conflict with BIP-93's threshold-bookkeeping convention.

K-of-N share encoding is **locked-in for v0.2+** but deferred — ms1 will ship shares first across the m-format-star (BIP-93 already specifies the share-encoding math), but no v0.1 caller can construct one. The crate-private `envelope` module documents the v0.1 → v0.2 prefix-byte invariant that gates the migration (see §V.3.7); v0.1 emits `0x00` and v0.1 decoders reject anything else with `ReservedPrefixViolation`.

### V.3.5.4 v0.2-migration seam

External callers do not see the `envelope` module: it is `mod envelope;` (no `pub`) at `src/lib.rs:48`. Its items (`WireFields`, `extract_wire_fields`, `discriminate`, `package`) are `pub(crate)`. Every contact with `rust-codex32` happens inside this module — no other source file imports `codex32::*`. Consequence: when `rust-codex32` ships its post-v0.1.0 API (share-construction, broader type discriminants), the upgrade is contained to one module plus the `Error::Codex32` payload type. Downstream code is insulated from upstream churn except via the single error wrapping path. Cross-references: §II.3 (the prefix-byte and grouping invariants) and §IV.3 (the v0.1 → v0.2 migration locked across all three formats).

## V.3.6 Versioning and MSRV

- Crate version: **0.1.1** (HEAD `c31f336`).
- Rust edition: **2021** (inherited from workspace `Cargo.toml`). **Distinct from `md-codec` and `mk-codec`**, which both use edition 2024 — ms-codec held edition 2021 in v0.1.
- MSRV: **1.85** (`rust-version` inherited from workspace).
- License: **MIT**.
- `rust-codex32`: pinned `=0.1.0` (exact, in `[workspace.dependencies]`); no minor-range bump until ms-codec v0.2 lands. The exact-pin reflects the v0.2 share-encoding migration: callers must not silently upgrade to a `rust-codex32` minor that changes the wire layer.
- Public semver promise: **none**. Pre-1.0 reference implementation; any 0.X bump may break. The v0.1 wire format is locked, but the Rust API may shift on any 0.X bump. The v0.2 migration is wire-additive (the `0x00` reserved-prefix byte becomes a type discriminator); v0.1 decoders that strictly enforce `ReservedPrefixViolation` will reject v0.2 strings (by design).

## V.3.7 Notes for advanced users

- **Narrow `rust-codex32` delegation.** Only `codex32::Error` leaks into the public surface, as the payload of `Error::Codex32` plus the `From<codex32::Error> for Error` impl at `error.rs:122`. There is no `pub use codex32::*` re-export anywhere; the crate-private `envelope` module isolates upstream contact. External callers cannot accidentally couple to `rust-codex32`'s API surface — only to its error type via ms-codec's wrapper. When `rust-codex32` ships its share-construction API, the upgrade lives behind `envelope`.
- **No false share-generation claim.** ms-codec v0.1 exposes no `shares`, `share`, `split`, or analogous threshold-share API, because `rust-codex32 v0.1.0` itself exposes only `interpolate_at` for reconstruction and no `Codex32String::shares` constructor. The crate enforces `threshold = 0` and `share-index = 's'` on every emit and reject path (via `consts::THRESHOLD_V01`, `consts::SHARE_INDEX_V01`, and `envelope::discriminate`). Threshold encoding is deferred to ms-codec v0.2. Chapter readers should not infer a share API from `rust-codex32`'s presence in the dep graph.
- **Uniform `#[non_exhaustive]` discipline.** `Error`, `Payload`, `PayloadKind`, and `InspectReport` are **all** marked `#[non_exhaustive]`. Unlike `mk-codec` (where `BchCode`, `CaseStatus`, `BytecodeHeader`, and `XpubCompact` are not marked — see §V.2.7), ms-codec is uniformly non-exhaustive across every public type that participates in matching or construction. External callers must include `_ => ...` arms throughout, and cannot brace-initialize `Payload` / `PayloadKind` / `InspectReport` from outside the crate.
- **`Error::source()` always returns `None`.** The `std::error::Error` impl at `error.rs:115-120` explicitly stops the error chain: `codex32::Error` does not itself implement `std::error::Error` in `rust-codex32 v0.1.0`, so ms-codec cannot forward into it. Callers chaining errors should not expect a meaningful source from any ms-codec `Error`; the `Error::Codex32` payload carries the underlying `codex32::Error` directly via `Debug` (in the `Display` impl: `"codex32 parse error: {:?}"`), not via the `source()` trait method.
- **`cargo doc` warning at `tag.rs:55`** (`rustdoc::invalid_html_tags`). The `Tag::as_str` doc-comment contains the literal `<non-utf8>` sentinel value, which rustdoc parses as an HTML tag. The function behaviour is unchanged; the rendered doc strips the angle brackets. Informational, not blocking. Callers quoting the sentinel in their own docs should escape it as `\<non-utf8\>` (as this chapter does in §V.3.3.7).
- **v0.2-migration seam (`mod envelope`) is the only `rust-codex32` contact point.** The doc-comment at `crates/ms-codec/src/envelope.rs:1-27` reads "THE v0.2-MIGRATION SEAM. This is the only module that contacts `rust-codex32`." `pub(crate)` items (`WireFields`, `extract_wire_fields`, `discriminate`, `package`) are not part of the public API. v0.1 callers never see the seam; v0.2 will repurpose the `0x00` reserved-prefix byte as a type discriminator and add share-encoding entry points alongside `encode`/`decode`. The v0.1 → v0.2 prefix-byte invariant is locked: v0.1 emits `0x00`, v0.1 decoders reject anything else.
- **`Payload::Entr` does not check randomness quality.** The `validate()` method enforces byte length only (`payload.rs:40`). Callers seeding `Entr` from non-OS-CSPRNG sources (dice, paper, stretched passphrases) are responsible for entropy quality. ms-codec's role is faithful wire-format round-tripping, not entropy assurance.
- **Crate-level lint posture is strict.** `#![cfg_attr(not(test), deny(missing_docs))]` (`src/lib.rs:38`) ensures every public item carries a doc-comment in non-test builds. Any undocumented public item would fail to compile in release builds — the API surface in this chapter is therefore exhaustively documented in-source.

## Cross-references

- §I.3 — codex32 and BCH (BIP-93 foundations directly adopted here, not forked).
- §II.3 — ms1 wire format (the prefix-byte / chunk-grouping invariants these APIs encode/decode).
- §IV.3 — future shares (the v0.1 → v0.2-shares migration locked across md1 / mk1 / ms1; ms1 ships first).
- §V.1 — md-codec (sibling: forked BCH on HRP-mixed residues; contrast with ms-codec's direct `rust-codex32` adoption).
- §V.2 — mk-codec (sibling: forked BCH; same contrast).
- Worked example: `cargo run --quiet --manifest-path docs/technical-manual/examples/Cargo.toml --example ms-codec-api-roundtrip` — source at `docs/technical-manual/examples/examples/ms-codec-api-roundtrip.rs`; transcript pair at `docs/technical-manual/transcripts/ms-codec-api-roundtrip.{cmd,out}`.

<!-- cspell-additions: (none — every new term is taken from the existing manual or harvest doc-comments; the harvest's review-cycle gate guarantees vocabulary alignment) -->
