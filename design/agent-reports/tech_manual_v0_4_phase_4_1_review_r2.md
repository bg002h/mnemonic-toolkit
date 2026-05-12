# Phase 4.1 review — r2

Date: 2026-05-11
Reviewer: feature-dev:code-reviewer (r2)

## Summary

- Chapter: 0C / 0I / 0L / 0N
- Cross-cutting: 0C / 0I / 0L / 0N

Total: 0C / 0I / 0L / 0N

## r1 fix-verification

**I-1 (`render_codex32_grouped` import):** CONFIRMED. Chapter lines 193-194 now read:

```rust
use md_codec::{Descriptor, encode_md1_string};
use md_codec::encode::render_codex32_grouped;
```

`render_codex32_grouped` is `pub fn` at `crates/md-codec/src/encode.rs:98` and absent from `lib.rs:39-52` re-exports. The module-qualified path is correct.

**I-2 (`SINGLE_STRING_PAYLOAD_BIT_LIMIT` import):** CONFIRMED. Chapter lines 481-482 now read:

```rust
use md_codec::{Descriptor, encode_md1_string, encode_payload, split};
use md_codec::chunk::SINGLE_STRING_PAYLOAD_BIT_LIMIT;
```

`SINGLE_STRING_PAYLOAD_BIT_LIMIT` is `pub const` at `chunk.rs:212` and absent from `lib.rs` `pub use chunk::{...}` re-exports. The body (lines 483-490) now uses plain `encode_payload(&d)?` (no `md_codec::` prefix) — correct since the import line covers it.

**L-1 (`§V.1.8` dangling reference):** CONFIRMED. Grep for `§V.1.8` and `deferred to Phase 4.1.2` returns no matches. The replacement at line 558 points to real navigable artefacts at `docs/technical-manual/transcripts/md-codec-api-roundtrip.{cmd,out}` and `docs/technical-manual/examples/examples/md-codec-api-roundtrip.rs`.

## Spot-check sweep (5 imports + 5 line-cites)

**Imports verified against HEAD `crates/md-codec/src/lib.rs:39-52`:**

1. Line 100-101: `canonicalize_placeholder_indices` (lib.rs:39 ✓), `decode_md1_string` (lib.rs:41 ✓), `expand_per_at_n` at `canonicalize.rs:420` ✓.
2. Line 125: `split`, `reassemble`, `Descriptor` — all three re-exported at lib.rs:40 + 42. ✓.
3. Line 141: `md_codec::codex32::{wrap_payload, unwrap_string}` — `pub mod codex32` at lib.rs:21; functions at `codex32.rs:67` + `92`. ✓.
4. Line 308: `md_codec::to_miniscript::to_miniscript_descriptor` — `#[cfg(feature="derive")] pub mod to_miniscript` at lib.rs:32-33; function at `to_miniscript.rs:54`. ✓.
5. Line 526: `derive_chunk_set_id` (lib.rs:40), `compute_md1_encoding_id` (lib.rs:45-48). ✓.

**Source-line cites verified:**

1. `decode_payload` at `decode.rs:15` ✓.
2. `decode_md1_string` at `decode.rs:79` ✓.
3. `validate_presence_byte` at `identity.rs:253` ✓.
4. `render_codex32_grouped` at `encode.rs:98` ✓.
5. `to_miniscript_descriptor` at `to_miniscript.rs:54` ✓.

## New findings

None.

## Verdict

- [x] 0 C / 0 I — Phase 4.1 ready to close (move to Phase 4.2)
- [ ] Findings present — iterate r3

All three r1 fixes correctly applied; spot-check sweep on a representative sample of unchanged chapter content found no additional issues. `make lint` 6/6 green is independently confirmed (run by the implementer between r1 and r2; the three fixes were prose-only edits that don't touch lint surface).
