# Phase 4.2 review — r1

Date: 2026-05-11
Reviewer: feature-dev:code-reviewer (r1)

## Summary

- Chapter (52-mk-codec-api.md): 0C / 2I / 0L / 0N
- Index-table accretion: 0C / 0I / 0L / 0N
- cspell additions: 0C / 0I / 0L / 0N
- Example crate (Cargo.toml + .rs): 0C / 0I / 0L / 0N
- Transcript pair: 0C / 0I / 0L / 0N
- Cross-cutting: 0C / 0I / 0L / 0N

Total: 0C / 2I / 0L / 0N

---

## Findings — chapter

### I-1 — §V.2.3.11 and §V.2.7: `BchCode` falsely claimed `#[non_exhaustive]`

**Location:** `docs/technical-manual/src/50-rust-api/52-mk-codec-api.md`, lines 203 and 368

**Evidence.** HEAD source at `crates/mk-codec/src/string_layer/bch.rs:26-32`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BchCode {
    Regular,
    Long,
}
```

`#[non_exhaustive]` is absent. Comprehensive grep across `src/`:

```
error.rs:18   (Error)
key_card.rs:22  (KeyCard)
chunk.rs:25   (ChunkFragment)
header.rs:33  (StringLayerHeader)
bch.rs:362    (CorrectionResult)
bch.rs:568    (DecodedString)
```

Six total. `BchCode` at `bch.rs:27` is not among them.

**Impact.** Readers writing exhaustive `match` on `BchCode` without `_ =>` arm are correct; the chapter falsely indicates a catch-all is needed.

**Fix.** Line 203 — drop the false `(#[non_exhaustive])` claim from the Signature cell. Line 368 — combined with I-2, rewrite the §V.2.7 sentence to:

```
`KeyCard`, `Error`, `StringLayerHeader`, `CorrectionResult`, `DecodedString`, `ChunkFragment` ARE marked `#[non_exhaustive]`. `BchCode`, `CaseStatus`, `BytecodeHeader`, and `XpubCompact` are NOT.
```

---

### I-2 — §V.2.3.11 and §V.2.7: `CaseStatus` falsely claimed `#[non_exhaustive]`

**Location:** `docs/technical-manual/src/50-rust-api/52-mk-codec-api.md`, lines 204 and 368

**Evidence.** HEAD source at `crates/mk-codec/src/string_layer/bch.rs:154-162`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaseStatus {
    Lower,
    Upper,
    Mixed,
}
```

`#[non_exhaustive]` is absent. Same structural finding as I-1.

**Impact.** Same as I-1.

**Fix.** Line 204 — drop the false `(#[non_exhaustive])` claim from the Signature cell. Line 368 fix is shared with I-1 (one combined edit).

---

## Findings — index-table accretion

None. Alphabetical interleaving spot-checked for 12 representative new rows; all anchors resolve to `#mk-codec-rust-api`.

## Findings — cspell additions

None. Three new words verified against chapter usage: `CHUNKABLE` (line 259), `getrandom` (lines 177, 321, 370), `shibbolethnumskey` (line 49).

## Findings — example crate

None. All four files verified. `[workspace]` isolation preserved; git-tag pin correct; `[[example]]` entry added; all imports resolve at HEAD; `encode_with_chunk_set_id` correctly chosen for determinism; `chunk_set_id = 144470` fits in 20 bits.

## Findings — transcript pair

None. `.cmd` matches expected format; `.out` 4 lines consistent with BIP-84 single-stub card encoded with `chunk_set_id = 144470`.

## Findings — cross-cutting

None. Checked items:

- **SPEC §4.2.5 sub-sections.** All six required (V.2.1–V.2.6) plus bonus V.2.7 present. ✓
- **Module coverage.** All 13 public modules covered in §V.2.3.1–§V.2.3.13. `bch_decode` (`pub(crate)`) and `pipeline` (private) correctly excluded. ✓
- **Error taxonomy — 22 variants.** Counted independently: 11 string-layer + 11 bytecode-layer. Spot-checked 8 against `error.rs`: all confirmed. ✓
- **Import-path correctness — all 5 inline `use` snippets verified** (Phase 4.1 I-1/I-2 pattern check):
  1. §V.2.3.3 line 87: `use mk_codec::{KeyCard, encode, decode}` — all three in `lib.rs:50` ✓
  2. §V.2.3.9 line 161: `use mk_codec::bytecode::{XpubCompact, reconstruct_xpub}` — both in `bytecode/mod.rs:31` ✓
  3. §V.2.3.11 line 229: `use mk_codec::string_layer::{bytes_to_5bit, encode_5bit_to_string, decode_string}` — all three in `string_layer/mod.rs:30-34` ✓
  4. §V.2.3.13 line 265: `use mk_codec::bytecode::encode_bytecode` + `use mk_codec::string_layer::{split_into_chunks, reassemble_from_chunks}` ✓
  5. §V.2.5.1 line 325: `use mk_codec::{encode, encode_with_chunk_set_id, KeyCard}` ✓

  No Phase-4.1-pattern wrong-path errors found.

- **Line-number spot-checks (10).** `consts.rs:9,15,50`; `key_card.rs:79,99`; `bytecode/header.rs:30`; `bytecode/path.rs:38`; `bytecode/xpub_compact.rs:85`; `string_layer/pipeline.rs:47`; `string_layer/chunk.rs:21` — all confirmed in HEAD. ✓

- **§V.2.7 advanced-user notes vs harvest.** All flagged notes present (BCH fork, path-dict standalone, non-exhaustive policy [I-1/I-2 cover the enumeration error within it], `reconstruct_xpub` panic, CSPRNG panic, `corrected_char_at` panic, `KeyCard::new` permissive, `Error` non-exhaustive, `bech32` unused, 3 cargo-doc warnings, stale `"md1"` at bch.rs:575+603). ✓

- **Cross-references.** §I.3 → `13-codex32-and-bch.md` ✓; §II.2 → `22-mk1-wire-format.md` ✓; §V.1 → `51-md-codec-api.md` ✓; worked-example artefacts exist. No dangling refs analogous to Phase 4.1 L-1. ✓

- **§V.1 style alignment.** Subsection numbering, citation format, `\index{}` convention, code-block/table conventions all consistent with §V.1. ✓

- **Feature-flag table.** `gen-vectors` entry matches `Cargo.toml:13-18`. ✓

- **Versioning §V.2.6.** v0.2.2, HEAD `e8782fd`, edition 2024, MSRV 1.85, MIT — match harvest. ✓

---

## Verdict

- [ ] 0 C / 0 I — Phase 4.2 ready to close (move to Phase 4.3)
- [x] Findings present — iterate r2

Two Important findings. Both are factual misattributions of `#[non_exhaustive]` to types that are NOT marked. Three sites total: §V.2.3.11 row for `BchCode` (line 203), row for `CaseStatus` (line 204), and §V.2.7 enumeration (line 368). All other aspects pass — no import errors, error count correct, 10/10 line cites accurate, cross-refs valid, example deterministic.
