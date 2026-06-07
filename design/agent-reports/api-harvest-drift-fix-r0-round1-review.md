# R0 Architect Review — api-harvest-drift-fix — Round 1

> Persisted verbatim from the opus `feature-dev:code-architect` agent
> (`agentId: abe44dc82cb37c725`). Had Read/Glob/Grep; verified against source.

---

## VERDICT: 1 Critical / 2 Important (+ 4 Minor) — NOT GREEN

### Critical
**C1: §1b "chapter drift only at :56 + :67" and §4 grep are FALSE — the fix as specified leaves substantial stale content.** After the SPEC's stated changes, these survive uncorrected in the RENDERED chapter `src/50-rust-api/54-mnemonic-toolkit-api.md`:

| Chapter line | Current (stale) | Correct |
|---|---|---|
| `:62` | `ResolvedSlot … path_raw … synthesize.rs:569` | `:642`; **`path_raw` field DELETED in v0.37.9** |
| `:63` | `is_secret_bearing … :579` | `:690` |
| `:64` | `CosignerKeyInfo … :190` | `:219` |
| `:65` | `xpub_to_65 … :69` | `:98` |
| `:66` | `build_descriptor … :80` | `:109` (SPEC fixes this only in the transcript) |

Worst: `path_raw` (`pub path_raw: String`, deleted v0.37.9 `project_path_raw_unification_cycle`) is documented as an ACTIVE field of `ResolvedSlot` at chapter `:62`. The actual struct (`synthesize.rs:642`) has no `path_raw` (adds `master_xpub: Option<Xpub>` at `:659`). Fixing synthesize_descriptor's arg-list while leaving a deleted field in the same table is the exact falsehood this cycle exists to eliminate.

### Important
**I1: THREE other rendered chapters carry stale `synthesize.rs:593` / `path_raw` / stale-range refs** — the §4 grep (`:593` bare form) misses the full-path form, and §3 doesn't scope them. All render into the PDF:
- `src/40-bundle-formation/41-bundle-anatomy.md:5` `synthesize.rs:593`; `:55` mermaid node `synthesize_unified (synthesize.rs:593)`; `:87` `synthesize.rs:568-582` + mentions `path_raw` as an active field in prose; `:201` `synthesize.rs:593-725` (correct range `:745-827`).
- `src/60-back-matter/61-glossary.md:53` `crates/mnemonic-toolkit/src/synthesize.rs:593`.
- `src/40-bundle-formation/42-anti-collision-invariants.md:40` `synthesize.rs:69-74` (`xpub_to_65` now `:98-102`).

**I2: refine the dead_code-grouping prose** (chapter `:56` + table `:68`). Since v0.47.1 `synthesize_unified` delegates to `synthesize_descriptor` (`synthesize.rs:826`); the `#[allow(dead_code)]` at `:218` is vestigial. Lumping `synthesize_descriptor` with the dead variants is misleading. Lift it to its own row as the live delegation target. File a code-side FOLLOWUP for the vestigial `#[allow(dead_code)]` at `synthesize.rs:218`. ("the CLI no longer calls them DIRECTLY" remains literally true.)

### Minor
- **M1:** transcript stale entries BEYOND the 7 synthesize fns: `:257` `xpub_to_65 :69`→`:98`; `:273` `ResolvedSlot :569`→`:642`; `:277` `path_raw: String` (deleted); `:278` `entropy: Option<Vec<u8>>`→`Option<zeroize::Zeroizing<Vec<u8>>>`; `:279` `is_secret_bearing :579`→`:690`; `:280` `CosignerKeyInfo :190`→`:219`. Fixing 7 of 14 in one table is inconsistent.
- **M2:** §4 grep structurally inadequate; replace with `grep -rn 'synthesize\.rs:[0-9]' docs/technical-manual/` enumerate-and-verify-all.
- **M3:** chapter struct refs off-by-2: `Bundle :20`(`:60`)→`:22`; `any_secret_bearing :33`(`:61`)→`:35`.
- **M4:** transcript `synthesize.rs:1296` dead pointer (lines 133 + 427) — `BundleJson schema_version:"4"` construction is in `cmd/bundle.rs`, not `synthesize.rs` (content-moved, not line-shifted).

### Verified Clean
1. All 7 synthesize signatures + line numbers the SPEC proposes (`:109/:142/:181/:229/:344/:489/:745`) — verified correct; synthesize_descriptor IS 4-arg `(descriptor, cosigners, privacy_preserving, run_language)` (SPEC right, FOLLOWUP wrong).
2. `synthesize_unified` delegates to `synthesize_descriptor` at `:826`; `#[allow(dead_code)]` at `:218` vestigial — verified.
3. Transcript is unrendered scaffolding (Makefile renders only `src/**`; no include) — confirmed.
4. `api-surface-coverage.sh` doesn't gate the transcript (exits 0; checks 7 JSON-envelope names in `src/50-rust-api/*`) — confirmed.
5. Sibling transcripts (`api-harvest-{md,mk,ms}-codec.md`) don't cite toolkit `synthesize_*` — confirmed.
6. No-bump/no-tag for `docs/technical-manual/` — confirmed (separate cadence, no CI).
7. FIX (not delete) the transcript — correct (maintained reference).

### Required folds
- **C1:** expand §2b to fix ALL stale rows in the chapter synthesize table (`:62-:66`, incl. deleted `path_raw` + stale `entropy` type); remove the false §1b sentence; fix §4 grep.
- **I1:** add §2c covering the 4 rendered non-API chapters (`41-bundle-anatomy.md`, `61-glossary.md`, `42-anti-collision-invariants.md`) OR explicitly FOLLOWUP-defer them with each file named + correct the false §4 claim.
- **I2:** refine the dead_code prose (`:56` + `:68`); file the code-side `#[allow(dead_code)]` FOLLOWUP.
- C1+I1 are scope/completeness gaps, not direction errors; the SPEC's factual corrections are sound.
