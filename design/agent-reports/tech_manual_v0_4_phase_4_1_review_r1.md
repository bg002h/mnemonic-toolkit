# Phase 4.1 review — r1

Date: 2026-05-11
Reviewer: feature-dev:code-reviewer (r1)

## Summary

- Chapter (51-md-codec-api.md): 0C / 2I / 1L / 0N
- Index-table accretion (62-index-table.md): 0C / 0I / 0L / 0N
- Example crate (Cargo.toml + .rs): 0C / 0I / 0L / 0N
- Transcript pair (.cmd / .out): 0C / 0I / 0L / 0N
- Cross-cutting: 0C / 0I / 0L / 0N

Total: 0C / 2I / 1L / 0N

---

## Findings — chapter

### I-1 — §V.1.3.8 inline snippet imports `render_codex32_grouped` from the wrong path

**Location:** `docs/technical-manual/src/50-rust-api/51-md-codec-api.md:193`

**Evidence:** Chapter line 193 reads:

```rust
use md_codec::{Descriptor, encode_md1_string, render_codex32_grouped};
```

`render_codex32_grouped` is declared `pub fn` at `crates/md-codec/src/encode.rs:98`. The crate root re-export block at `crates/md-codec/src/lib.rs:42` is `pub use encode::{Descriptor, encode_md1_string, encode_payload}` — `render_codex32_grouped` is absent. A reader copying the snippet gets a compile error (`unresolved import md_codec::render_codex32_grouped`).

The crate-root re-export block shown in §V.1.3 (chapter lines 29-43) is itself correct; only the per-module snippet at line 193 is wrong.

**Fix:**

```rust
use md_codec::{Descriptor, encode_md1_string};
use md_codec::encode::render_codex32_grouped;
let card: String = encode_md1_string(&d)?;
let grouped = render_codex32_grouped(&card, 4);
```

---

### I-2 — §V.1.5.1 inline snippet imports `SINGLE_STRING_PAYLOAD_BIT_LIMIT` from the wrong path

**Location:** `docs/technical-manual/src/50-rust-api/51-md-codec-api.md:480`

**Evidence:** Chapter line 480 reads:

```rust
use md_codec::{Descriptor, encode_md1_string, split, SINGLE_STRING_PAYLOAD_BIT_LIMIT};
```

`SINGLE_STRING_PAYLOAD_BIT_LIMIT` is declared `pub const` at `crates/md-codec/src/chunk.rs:212`. The lib.rs re-exports from `chunk` (line 40) are `{ChunkHeader, derive_chunk_set_id, reassemble, split}` — the constant is absent. A reader copying the snippet gets a compile error.

The §V.1.3.4 chunk-module table at chapter line 122 correctly lists the constant under `md_codec::chunk`; only the §V.1.5.1 `use` statement is wrong. Line 484 uses the qualified `md_codec::encode_payload(...)`, which is fine because `encode_payload` IS re-exported at crate root.

**Fix:**

```rust
use md_codec::{Descriptor, encode_md1_string, encode_payload, split};
use md_codec::chunk::SINGLE_STRING_PAYLOAD_BIT_LIMIT;
```

---

### L-1 — Cross-references section contains a dangling `§V.1.8` forward pointer

**Location:** `docs/technical-manual/src/50-rust-api/51-md-codec-api.md:556`

Line 556 reads:

```
- §V.1.8 (deferred to Phase 4.1.2) — `cargo run --example md-codec-api-roundtrip` worked example file.
```

There is no §V.1.8 in the shipped chapter. The "deferred to Phase 4.1.2" label communicates intent but the cross-reference itself is non-navigable. The other four cross-references (§II.1, §III.1, §III.2, §IV.1) are valid (chapter headings confirmed).

Phase 4.1.2 has now shipped — the worked example transcript is at `docs/technical-manual/transcripts/md-codec-api-roundtrip.{cmd,out}` and the source at `docs/technical-manual/examples/examples/md-codec-api-roundtrip.rs`. The line should either be replaced by a real cross-reference to those artefacts, or removed.

---

## Findings — example crate

None. All four files verified:

- `Cargo.toml`: `[workspace]` isolates the crate from the toolkit workspace; `edition = "2024"` matches workspace; git tag `md-codec-v0.32.0` correct; `publish = false` present.
- `examples/md-codec-api-roundtrip.rs`: all `use` paths verified against HEAD. Module-qualified imports (`md_codec::origin_path::*`, `md_codec::tree::*`, `md_codec::use_site_path::*`, `md_codec::tlv::*`) access public module items. Crate-root imports (`md_codec::{Descriptor, Tag, decode_md1_string, encode_md1_string}`) all present in lib.rs re-exports. No `pub(crate)` items accessed.
- `Cargo.lock` present (determinism requirement met).
- `.gitignore` contains `/target`.

---

## Findings — transcript pair

None. `.cmd` contains `cargo run --quiet --manifest-path examples/Cargo.toml --example md-codec-api-roundtrip` (no `$BIN` substitution required; correct format for `verify-examples.sh`). `.out` matches two-line expected output.

---

## Findings — cross-cutting

None. Checked items:

- **SPEC §4.2.5 compliance.** All six required sub-sections present: V.1.1 (purpose), V.1.2 (feature flags), V.1.3 (public API by module), V.1.4 (error taxonomy), V.1.5 (integration patterns), V.1.6 (versioning + MSRV). V.1.7 (advanced notes) is a bonus.
- **Module coverage.** V.1.3.1–V.1.3.20 present; `derive` (body-gated) and `to_miniscript` (whole-module-gated) correctly distinguished.
- **Error taxonomy — 43 variants.** All variants from `error.rs:19-392` appear in §V.1.4. Spot-checked: `VarintOverflow` (varint.rs:31 ✓), `DecodeRecursionDepthExceeded` (tree.rs:187 ✓), `OperatorContextViolation` (decode.rs:40 ✓), `MalformedHeader` and `Codex32EncodeError` correctly flagged as declared-but-unconstructed (✓). Count `(Variant count = 43.)` at line 462 correct.
- **Cross-references.** §II.1 ("md1 Wire Format" ✓), §III.1 ("Descriptor to Miniscript to Address" ✓), §III.2 ("Shape Coverage" ✓), §IV.1 ("Bundle Anatomy" ✓). All four target chapters exist with matching H1 titles.
- **Harvest-flagged §V.1.7 notes.** All five required notes present and accurate (`MalformedHeader`+`Codex32EncodeError` never-constructed ✓; `Body::MultiKeys` carries `Vec<u8>` ✓; `Phrase::from_id_bytes` effectively infallible ✓; `Descriptor::derive_address` returns `Address<NetworkUnchecked>` ✓; `MAX_DECODE_DEPTH=128` is anti-DoS ✓).
- **Feature flags.** `Cargo.toml:21-23` (`default = ["derive"]` / `derive = ["dep:miniscript"]`) matches §V.1.2 table exactly.
- **Line-number spot-checks (10).** `encode.rs:65,98,114`; `tree.rs:167`; `lib.rs:39-52`; `chunk.rs:212`; `identity.rs:15-16,20,25,30`; `error.rs:19-392`; `varint.rs:31` — all confirmed correct in HEAD.
- **Index-table.** 302 total rows. New §V.1 entries alphabetically interleaved. Chapter H1 `# md-codec Rust API` matches anchor `#md-codec-rust-api`.
- **cspell additions.** `thiserror` and `usize` present in `.cspell.json`.

---

## Verdict

- [ ] 0 C / 0 I — Phase 4.1 ready to close (move to Phase 4.2)
- [x] Findings present — iterate r2

Two Important: chapter inline-snippet imports use crate-root paths for two symbols that aren't re-exported (`render_codex32_grouped` at line 193, `SINGLE_STRING_PAYLOAD_BIT_LIMIT` at line 480). Both need module-qualified `use` paths. One Low: dangling `§V.1.8` forward reference (chapter line 556) — should be replaced with a real reference to the now-shipped worked-example transcript or removed.

Runnable example (`examples/md-codec-api-roundtrip.rs`) is correct and unaffected by these findings — the bugs are only in the chapter's inline prose snippets.
