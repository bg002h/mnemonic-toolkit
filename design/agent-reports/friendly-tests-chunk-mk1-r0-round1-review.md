# R0 Architect Review — SPEC_friendly_tests_and_chunk_mk1.md — Round 1

> Reviewer had Read/Grep/Bash; parent persists. Source basis: HEAD `8665d91`.

## Verdict: NOT GREEN — 0 Critical / 2 Important / 5 Minor

The cycle is sound and low-risk; disposition (no-bump/no-tag) is right. Two Important defects must fold: the §3 verification command runs ZERO new tests (false-green on the whole deliverable), and the exhaustiveness taxonomy the implementer is told to encode into a comment + FOLLOWUPS is wrong about `bip39::Error`. The "constructibility feasibility risk" is NOT real — `#[non_exhaustive]` restricts external *matching* (forces `_`), not *construction* of existing variants; shipped green tests already build `mk_codec::Error::PathTooDeep(11)` etc.

## Critical
None.

## Important

**I1 — §3.1 verification command runs ZERO new tests (`--lib` on a bin-only module).**
`friendly` is declared only in the **binary** crate (`src/main.rs:16` `mod friendly;`); NOT in `src/lib.rs`. So `cargo test -p mnemonic-toolkit --lib friendly` compiles the lib, finds no `friendly`, runs **0 tests** — green proving nothing on a cycle whose deliverable is "these tests run."
**Fix:** §3.1 → `cargo test -p mnemonic-toolkit --bin mnemonic friendly` (or no target filter). Remove the "lib or bin" / "(or the bin target)" hedges. `format.rs` is likewise bin-only (`mod format;` in main.rs) — its byte-identical test runs under the same bin target.

**I2 — SPEC miscategorizes `bip39::Error` as `#[non_exhaustive]`.**
SPEC line 38 names "the four `#[non_exhaustive]` mappers (bip39/bitcoin/ms_codec/mk_codec)"; line 24 calls md_codec "the only exhaustive match." Both wrong. `bip39::Error` (bip39-2.2.2/src/lib.rs:116-117) has **no** `#[non_exhaustive]` — closed 5-variant enum, which is why `friendly_bip39` matches all 5 with **no `_` wildcard**. The `friendly.rs` module-doc (lines 4-6) is ALSO wrong (lists bip39 among non_exhaustive).
Correct taxonomy: **closed (all arms testable, no wildcard):** `md_codec` (44) + `bip39` (5). **`#[non_exhaustive]` (wildcard `_`):** `bitcoin::bip32::Error`, `ms_codec`, `mk_codec` — **three**, not four.
**Fix:** correct lines 24 + 38; have the implementer fix friendly.rs module-doc (lines 4-6) in the same edit.

## Minor

**M1 — `bip39::Error::AmbiguousLanguages` is NOT constructible from the test crate; drop that row.** Payload `AmbiguousLanguages([bool; MAX_NB_LANGUAGES])` (lib.rs:94) is a tuple struct with a private field + no public constructor. The other 3 bip39 arms (`BadEntropyBitCount`, `BadWordCount`, `InvalidChecksum`) are constructible. Net bip39 new coverage: 3, not 4.

**M2 — `bundle.rs::emit` does not exist; the fn is `emit_unified` (bundle.rs:778).** All three chunk sites (951/962/974) are inside `emit_unified`. Fix the SPEC citation AND the FOLLOWUPS.md entry (which carries the same stale `emit` at :1022/1025/1026) when flipping to resolved.

**M3 — Drop the vestigial `#[allow(dead_code)]` on `chunk_mk1` (format.rs:32).** Post-swap `chunk_mk1` is live (2 sites) → the allow masks a future real dead-code regression. Add to §1 scope; build-warning-clean is the RED-equivalent.

**M4 — Update the stale `chunk_mk1` doc comment (format.rs:28-31).** "Reserved: mk1 currently uses `chunk_5char` directly" is false post-swap. Reword: mk1 now routes through `chunk_mk1`; the single future swap point stays the body (`:33`).

**M5 (non-blocking opinion) — assertion meaningfulness.** `assert(!contains("unhandled"))` is load-bearing ONLY for the 3 non_exhaustive mappers (where a future variant silently falls to the `_` "unhandled" arm). For md_codec + bip39 (closed) it's vacuous (no `_` arm); there the substantive assertions are no-Debug-variant-name-leak + tag-present. Note this in the table comment so the assertions aren't cargo-culted. Highest-value subset: the 44 md_codec arms (only place a message-quality regression hides) + the wildcard-prone mk_codec/ms_codec arms.

## Verified-correct
- **Chunk sites:** ms1 `:951` (stays), `MkField::Single` `:962` + `MkField::Multi` `:974` (swap). Import `bundle.rs:7` `use crate::format::{chunk_5char, chunk_md1, BundleJson, CosignerEntry, MkField, MultisigInfo}` — add `chunk_mk1`, keep `chunk_5char` (still used :951). Enclosing fn = `emit_unified` (:778).
- **Byte-identical (#4):** `format.rs:33` `chunk_mk1` is `{ chunk_5char(s) }`; referenced nowhere else in `src/` (genuinely dead today). Swap → binary byte-identical.
- **Baseline:** 12 `#[test]` covering ~15 arms; `friendly_md_codec` 0 tests.
- **Arm counts:** md_codec 44 (exhaustive) ✓; mk_codec 22 `+ _` ✓; ms_codec 19 `+ _` (SPEC "~20" close) ✓; bip39 5 (closed) ✓; bitcoin 3 (closed; `Bip32` arm wraps non_exhaustive `bitcoin::bip32::Error`) ✓.
- **Constructibility — RESOLVED, no blocker:** all 44 md_codec variants pub w/ pub fields (md-codec's own tests build `OperatorContextViolation{tag:Tag::Multi, context:ContextKind::MultiBody}`); `Tag`/`ContextKind` pub enums; struct-field variants plain usize/u32. ms_codec/mk_codec variants buildable from the toolkit crate (existing tests prove it; `XpubOriginPathMismatch` needs `ChildNumber`, already imported in the existing test). `bitcoin::bip32::Error` `Bip32` arm: pick a unit variant (`CannotDeriveFromHardenedKey`/`MaximumDepthExceeded`), avoid wrapper variants. Only blocked: `bip39 AmbiguousLanguages` (M1).
- **md_codec exhaustiveness self-protection:** accurate (new variant → toolkit compile error). So testing its 44 arms is message-quality, not fallthrough-catching — same for bip39 (per I2).
- **Disposition:** no-bump/no-tag correct (tests bin-internal `#[cfg(test)]`; chunk swap byte-identical); no schema_mirror/manual/GUI lockstep. Locksteps: none.

## Required folds before GREEN
1. (I1) §3.1 → bin target, remove lib/bin hedges.
2. (I2) lines 24+38 → bip39 closed; non_exhaustive set is THREE; + implementer fixes friendly.rs module-doc lines 4-6.
3. (M1–M4) drop AmbiguousLanguages row; `emit`→`emit_unified` (SPEC + FOLLOWUPS); remove `#[allow]` format.rs:32; update comment format.rs:28-31.
Re-dispatch R0 after folding.
