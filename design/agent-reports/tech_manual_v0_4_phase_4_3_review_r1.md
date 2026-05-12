# Phase 4.3 review — r1

Date: 2026-05-11
Reviewer: feature-dev:code-reviewer (r1)

## Summary

- Chapter (53-ms-codec-api.md): 1C / 1I / 0L / 0N
- Index-table accretion: 0C / 0I / 0L / 0N
- cspell additions: 0C / 0I / 0L / 0N
- Example crate: 0C / 0I / 0L / 0N
- Transcript pair: 0C / 0I / 0L / 0N
- Cross-cutting: 0C / 0I / 0L / 0N

Total: 1C / 1I / 0L / 0N

---

## Findings — chapter

### C-1 — §V.3.3.2 decode example string is wire-invalid: id and share-index fields are transposed

**Location:** `docs/technical-manual/src/50-rust-api/53-ms-codec-api.md:67`

**Evidence.** The chapter's `decode` module inline snippet:

```rust
let (tag, payload) = decode("ms10sentrqqqqqqqqqqqqqqqqqqqqqqqq...")?;
```

ms1 wire structure after the BIP-93 `1` separator (`envelope.rs:17-24`): `threshold (1 char)` + `id / type-tag (4 chars)` + `share-index (1 char)` + payload + checksum. For a valid v0.1 string: threshold=`0`, id=`entr`, share-index=`s` → prefix `ms10entrs`.

The chapter string `ms10sentrqqq...` has:
- threshold = `0` ✓
- id (4 chars) = `sent` ✗ (not in `RESERVED_TAG_TABLE`; not `entr`)
- share-index = `r` ✗ (not `s`)

`decode` would reject with `Error::ReservedTagNotEmittedInV01` (rule 7, since `sent` is not in the v0.1 accept set) or `Error::UnknownTag`. A reader copying this snippet verbatim gets a decode error, not `(Tag::ENTR, Payload::Entr)`. The correct prefix is confirmed by the shipped transcript `ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f` at `transcripts/ms-codec-api-roundtrip.out:1`.

**Fix.** Replace the placeholder string with the confirmed-correct transcript output:

```rust
let (tag, payload) = decode("ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f")?;
```

---

### I-1 — §V.3.5.2 decoder pipeline step 3 omits rule 5 from `discriminate`'s enforcement

**Location:** `docs/technical-manual/src/50-rust-api/53-ms-codec-api.md:218`

**Evidence.** Chapter line 218 reads:

> 3. Calls `envelope::discriminate` to enforce HRP, threshold, share-index, and reserved-prefix byte invariants (SPEC §4 rules 2–4, 8).

`envelope.rs:90-115` shows `discriminate` enforces:
- Rules 2, 3, 4 (HRP, threshold, share-index) at lines 94-110
- Rule 5 (TagInvalidAlphabet) at line 114 (`Tag::try_new(tag_str)?`)
- Rule 8 (reserved-prefix byte) after the tag check

The chapter's §V.3.4 error-taxonomy table already correctly lists `TagInvalidAlphabet` as emitted by `envelope::discriminate` at `envelope.rs:114`. The §V.3.5.2 step-3 description is inconsistent with the table — rule 5 is enforced inside `discriminate`, but the step prose omits it.

**Fix.** Change line 218 from "rules 2–4, 8" to "rules 2–5, 8" (and add `tag-alphabet` to the enumerated list):

```
3. Calls `envelope::discriminate` to enforce HRP, threshold, share-index, tag-alphabet, and reserved-prefix byte invariants (SPEC §4 rules 2–5, 8).
```

---

## Findings — index-table accretion

None. 29 new rows (384 → 413). Spot-checked 14 representative entries; all anchors resolve to `#ms-codec-rust-api` matching chapter H1.

## Findings — cspell additions

None. `bijective`, `rustdoc`, `upstreamable` all confirmed present in `.cspell.json`.

## Findings — example crate

None. `Cargo.toml` correctly extends with ms-codec git-tag pin; `examples/ms-codec-api-roundtrip.rs` uses only crate-root re-exports; output deterministic.

## Findings — transcript pair

None. `.cmd` and `.out` match expected format and content.

## Findings — cross-cutting

None.

- **SPEC §4.2.5 sub-sections.** All 6 required + bonus V.3.7 ✓.
- **Module coverage.** All 7 public modules (V.3.3.1–V.3.3.7); `envelope` correctly excluded ✓.
- **Error taxonomy — 10 variants.** All 10 verified against `error.rs:9-64` ✓.
- **`#[non_exhaustive]` audit.** Grep of `crates/ms-codec/src/**` yields exactly 4 occurrences: `Error` (`error.rs:8`), `PayloadKind` (`payload.rs:10`), `Payload` (`payload.rs:18`), `InspectReport` (`inspect.rs:12`). Chapter §V.3.7 lists all four; no false attributions ✓.
- **Import paths (Phase 4.1 pattern).** All 5 inline `use ms_codec::{...}` snippets resolve at crate root `lib.rs:50-55` ✓.
- **API signatures.** `encode(tag: Tag, payload: &Payload) -> Result<String>` at `encode.rs:16`; `decode(s: &str) -> Result<(Tag, Payload)>` at `decode.rs:19`. Chapter signatures match ✓.
- **Line cites (10 spot-checks).** `consts.rs:11`, `consts.rs:29,33`; `encode.rs:16`; `decode.rs:19`; `error.rs:115-120,122-126`; `inspect.rs:13,34`; `tag.rs:55,56` — all confirmed ✓.
- **§V.3.7 advanced notes vs harvest (6 items).** All present and accurate ✓.
- **§V.3.6 edition.** "edition 2021" — matches workspace ✓.
- **Cross-references.** §I.3, §II.3, §IV.3, §V.1, §V.2 all valid ✓.

---

## Verdict

- [ ] 0 C / 0 I — Phase 4.3 ready to close
- [x] Findings present — iterate r2

One Critical (decode example wire-string transposed) + one Important (decoder pipeline step omits rule 5). Both are local fixes in the chapter. All other deliverables clean — no `#[non_exhaustive]` misattributions (Phase 4.2 pattern), no wrong import paths (Phase 4.1 pattern), no dangling cross-refs.
