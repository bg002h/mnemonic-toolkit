# Phase 4.0 harvest review — r1

Date: 2026-05-11
Reviewer: feature-dev:code-reviewer (r1)

## Summary

- md-codec: 1C / 0I / 1L / 0N
- mk-codec: 1C / 0I / 1L / 0N
- ms-codec: 0C / 0I / 0L / 0N
- mnemonic-toolkit: 0C / 1I / 0L / 0N
- Cross-cutting: 0C / 0I / 0L / 0N

Total: 2C / 1I / 2L / 0N

## Findings — md-codec

### Critical

#### C1 — Error variant count states "36"; actual count is 43

- **Location:** harvest line 259 (types section: `pub enum Error { ... } (error.rs:19-392) — 36 variants`) and line 531 (taxonomy preamble: `"pub enum Error from src/error.rs — 36 variants (line 19 to 392)"`).
- **Issue:** Two occurrences state "36 variants" for the `Error` enum. Actual count verified from `/scratch/code/shibboleth/descriptor-mnemonic/crates/md-codec/src/error.rs:20-392` is **43 variants**. The taxonomy table itself is complete and correct — all 43 variants appear. Only the stated count is wrong.
- **Evidence:** `src/error.rs` variant declarations (line: name): 23 `BitStreamTruncated`, 34 `WireVersionMismatch`, 43 `MalformedHeader`, 50 `PathDepthExceeded`, 59 `KeyCountOutOfRange`, 66 `DivergentPathCountMismatch`, 75 `AltCountOutOfRange`, 87 `TagOutOfRange`, 94 `ThresholdOutOfRange`, 101 `ChildCountOutOfRange`, 108 `KGreaterThanN`, 119 `TlvOrderingViolation`, 128 `PlaceholderIndexOutOfRange`, 137 `OverrideOrderViolation`, 146 `EmptyTlvEntry`, 153 `TlvLengthExceedsRemaining`, 162 `PlaceholderNotReferenced`, 173 `PlaceholderFirstOccurrenceOutOfOrder`, 182 `MultipathAltCountMismatch`, 191 `ForbiddenTapTreeLeaf`, 202 `OperatorContextViolation`, 211 `ChunkCountOutOfRange`, 218 `ChunkIndexOutOfRange`, 227 `ChunkSetIdOutOfRange`, 236 `ChunkHeaderChunkedFlagMissing`, 240 `ChunkCountExceedsMax`, 247 `Codex32DecodeError`, 251 `Codex32EncodeError`, 255 `ChunkSetEmpty`, 259 `ChunkSetInconsistent`, 263 `ChunkSetIncomplete`, 272 `ChunkIndexGap`, 281 `ChunkSetIdMismatch`, 290 `VarintOverflow`, 299 `MissingExplicitOrigin`, 312 `InvalidPresenceByte`, 322 `InvalidXpubBytes`, 332 `MissingPubkey`, 341 `ChainIndexOutOfRange`, 357 `HardenedPublicDerivation`, 365 `AddressDerivationFailed`, 377 `NUMSSentinelConflict`, 386 `DecodeRecursionDepthExceeded` = **43**.
- **Recommendation:** Replace "36 variants" with "43 variants" in both occurrences. The taxonomy table needs no changes.

### Low

#### L1 — Prose says "15 public modules" but table lists 20 rows

- **Location:** harvest line 42 (`"15 public modules + one private (mod bch)"`).
- **Issue:** The prose claims 15 public modules; the table immediately below has 20 rows. `/scratch/code/shibboleth/descriptor-mnemonic/crates/md-codec/src/lib.rs:17-37` declares exactly 20 `pub mod` statements (19 unconditional + 1 `#[cfg(feature = "derive")] pub mod to_miniscript`). The table is correct; the prose count is wrong.
- **Evidence:** `src/lib.rs` lines 17-37: bitstream, canonical_origin, canonicalize, chunk, codex32, decode, derive, encode, error, header, identity, origin_path, phrase, tag, tlv, to_miniscript (cfg-gated), tree, use_site_path, validate, varint = 20 total.
- **Recommendation:** Change "15 public modules" to "20 public modules (19 unconditional; `to_miniscript` requires `derive` feature)".

---

## Findings — mk-codec

### Critical

#### C1 — Error variant count states "21"; actual count is 22

- **Location:** harvest line 93 (types section) and taxonomy preamble at harvest line 324 (`"21 variants on Error (all in src/error.rs:20; #[non_exhaustive])"`).
- **Issue:** Two occurrences state "21 variants". Actual count from `/scratch/code/shibboleth/mnemonic-key/crates/mk-codec/src/error.rs:20-162` is **22 variants**. The taxonomy table is complete (all 22 variants listed); `CardPayloadTooLarge { bytecode_len, max_supported }` at `error.rs:156` is the 22nd variant.
- **Evidence:** `error.rs` variant list: `InvalidHrp` (24), `MixedCase` (31), `InvalidStringLength` (40), `InvalidChar` (48), `BchUncorrectable` (58), `UnsupportedCardType` (63), `MalformedPayloadPadding` (69), `ChunkSetIdMismatch` (74), `ChunkedHeaderMalformed` (80), `MixedHeaderTypes` (92), `CrossChunkHashMismatch` (97), `UnsupportedVersion` (102), `ReservedBitsSet` (107), `InvalidPolicyIdStubCount` (111), `InvalidPathIndicator` (118), `PathTooDeep` (123), `InvalidPathComponent` (128), `InvalidXpubVersion` (132), `InvalidXpubPublicKey` (138), `UnexpectedEnd` (142), `TrailingBytes` (146), `CardPayloadTooLarge` (156) = **22**.
- **Recommendation:** Replace "21 variants" with "22 variants" in both occurrences. The taxonomy table needs no changes.

### Low

#### L1 — `DecodedString` doc-comments reference `"md1"` — stale copy-from-md-codec; not flagged in Notes

- **Location:** Source at `/scratch/code/shibboleth/mnemonic-key/crates/mk-codec/src/string_layer/bch.rs:575` and `bch.rs:603` (reviewer caught :575; a second occurrence at :603 also exists, verified by harvest implementer).
- **Issue:** Both doc-comments contain `"md1"` (`"chars after \"md1\""`). The `"md1"` is a copy-from-md-codec artifact — mk1 strings use HRP `"mk"` (prefix `"mk1"`). The harvest's Notes section correctly identifies the related line-610 `crate::Correction::corrected` unresolved-link as a copy-from-md-codec issue, but does not flag the two `"md1"` strings in the visible doc-comments. A chapter author quoting either doc-comment verbatim would assert the wrong string format.
- **Evidence:** `bch.rs:575` reads `(chars after "md1")`; `bch.rs:603` reads `data part (chars after the "md1" HRP+separator)`; `consts.rs:9` declares `pub const HRP: &str = "mk"`. The prefix is `"mk1"`, not `"md1"`.
- **Recommendation:** Add a Notes item flagging both occurrences. (Source fix is outside harvest scope; this becomes a `cross-repo` FOLLOWUP candidate in mk1's tracker, mid-cycle.)

---

## Findings — ms-codec

No issues at any severity. Full verification performed:

- All 10 `Error` variants enumerated and confirmed against `src/error.rs:9-64`.
- "No `[features]` table" claim correct — confirmed by direct Cargo.toml inspection.
- 7 public modules + 1 private `mod envelope` correctly enumerated per `src/lib.rs:40-48`.
- Representative line citations verified: `decode` at `decode.rs:19`; `encode` at `encode.rs:16`; `Tag::try_new` at `tag.rs:28`; `Tag::as_str` at `tag.rs:56`; `InspectReport` at `inspect.rs:13`; `inspect` at `inspect.rs:34` — all confirmed.
- `std::error::Error::source()` always-returns-`None` observation confirmed at `error.rs:115-120`.
- rust-codex32 delegation scope and public-surface isolation are accurately described: no `pub use codex32::*` re-export; only `codex32::Error` reaches the public surface via `Error::Codex32` payload and `From<codex32::Error> for Error` at `error.rs:122`.
- The `<non-utf8>` HTML-tag warning at `tag.rs:55` is confirmed real. `#[non_exhaustive]` discipline verified uniformly across `Error`, `Payload`, `PayloadKind`, `InspectReport`.

---

## Findings — mnemonic-toolkit

### Important

#### I1 — Notes item 4 attributes typed DerivationPath equality to a non-existent `synthesize::check_key_vector_distinctness` function

- **Location:** harvest "Notes for chapter author (Phase 4.4)" item 4, sentence: `"synthesize::check_key_vector_distinctness and parse_descriptor::check_key_vector_distinctness (the pub API at parse_descriptor.rs:1104) compare typed DerivationPath equality (cs[i].path == cs[j].path)"`.
- **Issue:** The note asserts that `synthesize::check_key_vector_distinctness` exists. No such function is declared in `src/synthesize.rs`. The only `pub` function by that name is `parse_descriptor::check_key_vector_distinctness` at `parse_descriptor.rs:1104`. A chapter author who searches for `synthesize::check_key_vector_distinctness` will not find it and may conclude the note is fabricated. The note also implies both functions use typed `DerivationPath ==` — but only the `parse_descriptor` variant does (`parse_descriptor.rs:1108: cs[i].path == cs[j].path`). The CLI bundle path's distinctness check (a `pub(crate)` helper in `cmd::bundle` — out of harvest scope) uses raw-string equality, but that function has a different name.
- **Evidence:** `parse_descriptor.rs:1104` — `pub fn check_key_vector_distinctness(binding: &DescriptorBinding) -> Result<(), ToolkitError>`. Source inspection of `src/synthesize.rs` shows no function of that name (the harvest's own synthesize section enumerates all `pub fn` items: `xpub_to_65`, `build_descriptor`, `synthesize_full`, `synthesize_watch_only`, `synthesize_descriptor`, `synthesize_multisig_full`, `synthesize_multisig_watch_only`, `synthesize_unified` — `check_key_vector_distinctness` is absent).
- **Recommendation:** Revise Notes item 4: the sole `pub` function enforcing BIP-388 distinct-key semantics is `parse_descriptor::check_key_vector_distinctness` at `parse_descriptor.rs:1104`, which compares typed `DerivationPath ==` (folds `h ↔ '`). The `cmd::bundle`-internal mirror `check_resolved_slots_distinctness` (pub-crate, out of scope) uses raw-string equality and has the doc-comment lag documented at `error.rs:68-71`. Remove the `synthesize::check_key_vector_distinctness` attribution.

---

## Findings — cross-cutting

No critical or important cross-cutting issues. Consistency observations (all satisfactory):

- All four harvests uniformly mark CLI binary targets (`md-cli`, `mk-cli`, `ms-cli`, `cmd::*`) as out of scope and not enumerated as public-surface items.
- Feature flag enumeration format is consistent and correctly scoped across all harvests.
- Line-citation spot-check (≥5 per crate): all checked citations resolve at ±0 lines in HEAD source. md-codec `encode_payload` at `encode.rs:65`, `expand_per_at_n` at `canonicalize.rs:420`, `re_emit_bits` at `bitstream.rs:220`, `ExpandedKey` at `canonicalize.rs:337` confirmed. mk-codec `BytecodeHeader::parse` at `header.rs:40`, `STANDARD_PATHS` at `path.rs:38`, `KeyCard::new` at `key_card.rs:79` confirmed. mnemonic-toolkit `ParsedKey` at `parse_descriptor.rs:673`, `check_key_vector_distinctness` at `parse_descriptor.rs:1104`, `DescriptorBinding` at `parse_descriptor.rs:790` confirmed.
- Front-matter fields (version/MSRV/HEAD-commit): all four harvests correctly reflect HEAD state. The mnemonic-toolkit's stated dependency on md-codec at git tag `md-codec-v0.16.1` is correctly distinguished from md-codec's own v0.32.0 HEAD — the separation into independent harvests handles this correctly.
- The "binary-only" finding for mnemonic-toolkit (no `[lib]` target) is accurate: `Cargo.toml:15-18` contains only `[[bin]] name = "mnemonic" path = "src/main.rs"` with no `[lib]` section.
- The stale `format.rs:114` doc-comment flag (mnemonic-toolkit Notes item 2) is correctly identified: `format.rs:113-118` reads `"v0.2: schema_version \"2\""` while every `BundleJson` construction site at HEAD uses `schema_version: "4"` (confirmed at `synthesize.rs:1295-1296`).
- The v0.3 Phase 3.2 doc-comment lag at `error.rs:68-71` (`Bip388Distinctness` doc says "raw-string equality") is correctly noted as persisting at HEAD in Notes item 4, modulo the function-attribution error flagged as I1 above.

---

## Verdict

- [ ] 0 C / 0 I — Phase 4.0 ready to close (move to Phase 4.1)
- [x] Findings present — iterate r2

2C (md-codec error-variant count stated as 36, actual 43; mk-codec error-variant count stated as 21, actual 22). Both are mechanical count corrections — the taxonomy tables themselves are complete and no variant is missing or miscited. 1I (mnemonic-toolkit Notes item 4 cites a non-existent `synthesize::check_key_vector_distinctness` function). 2L (md-codec prose "15 modules" vs 20-row table; mk-codec stale `"md1"` strings in `DecodedString` doc-comments at `bch.rs:575` and `bch.rs:603` not flagged in harvest Notes).

After the implementer applies the C1×2 count fixes and the I1 Notes attribution correction (and folds the two Lows), request r2.
