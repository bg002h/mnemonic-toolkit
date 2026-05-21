# v0.30.0 brainstorm R1 review

**Reviewer:** opus
**Round:** R1 (verify R0 fold)
**Spec under review:** design/BRAINSTORM_v0_30_0_seedqr.md
**R0 review:** design/agent-reports/v0_30_0-brainstorm-r0-review.md
**Date:** 2026-05-21

**Tooling note:** my available toolset (Read/Grep/Glob/WebFetch/WebSearch) does NOT include Write or Edit. The orchestrator must persist this review verbatim to `design/agent-reports/v0_30_0-brainstorm-r1-review.md` before fold-and-commit, per CLAUDE.md "persist verbatim BEFORE the fold-and-commit step."

## R0 fold verification

| R0 finding | Claimed fold | Verified? | Note |
|---|---|---|---|
| C1 | library-local `SeedqrError` enum + `map_seedqr_error` CLI boundary; Phase 3 removed | yes | L44 (decision lock), L87 (lib module description), L92 ("No new `ToolkitError` variants"), L115-145 (`SeedqrError` enum + mapper code blocks), L218 (cross-cutting reaffirmation), L239-241 (Phase 3 explicit "REMOVED"); zero `ToolkitError::Seedqr*` variant references remain anywhere. |
| C2 | reclassified to MINOR v0.30.0 + paired GUI v0.15.0; filename renamed | yes | Title L1, L26-35 reclassification block, L149-153 SemVer+lockstep section, L215-216 install.sh + Cargo.toml bump; remaining `v0.29.1` / `v0.14.1` / `PATCH` references appear only in legitimate historical-framing context (predecessor brainstorm cite, reclassification table, fold-summary table). |
| I1 | chapter-45 citation L620-626 | yes | L169 + L249 + L325 all cite `L620-626`; verified against `45-foreign-formats.md:620` = `### Deferral — SeedQR` header (correct anchor); also includes explicit "L607-608 `jade_specific_fields` reservation sentence stays as-is" clarification. |
| I2 | OMIT `--language` flag; English-locked rationale explicit | yes | L45 decision lock with rationale; L64 CLI surface explicit "No `--language` flag — SeedQR is English-locked per spec". |
| I3 | `schema_version: "1"` first field | yes | L46 decision lock; L70 JSON envelope shape places `schema_version` as first field; L81 cites precedent. |
| I4 | new FOLLOWUP `seedqr-digits-from-input-unification` at cycle close | yes | L164 entry 4 in newly-filed FOLLOWUPs list with full rationale. |
| I5 | `kind` → `operation` rename | yes | L46 decision lock; L71 JSON shape uses `operation: "decode"`; L79-82 semantics; no `kind:` field remains in any envelope spec. |
| I6 | dossier path `design/cycle-5-p0-recon.md` confirmed | yes | L9 + L226 cite `design/cycle-5-p0-recon.md` matching `cycle-N-p0-recon.md` precedent. |
| I7 | A4 task added to Phase 0 | partial | A4 IS added at L229 + L179 cross-ref, but L229 says "DEFERRED to Phase 5 prelude" — stale (post-renumbering, manual writing is Phase 4). See I-new-1 below. |
| I8 | GUI v0.15.0 cascaded | yes | L35, L153, L261, L324 all cite `v0.15.0`. |
| M1 | cell count widened to 30-60 | yes | L207 "Approximate cell count: **30-60 cells**" with rationale. |
| M2 | predecessor anchor — no change | yes | unchanged (was already correct). |
| M3 | `pub mod seedqr;` in `lib.rs` locked | yes | L47 decision lock; L91 architecture section. |
| M4 | phase-numbering — no change (single-slug cycle) | yes | unchanged. |
| M5 | stdin loosened to `--digits=-` OR `--digits -` | yes | L48 decision lock; L62 CLI surface "Stdin signaled by `--digits=-` or `--digits -`"; L200 test for alt-form. |

## New Critical (introduced by fold)

NONE.

## New Important (introduced by fold)

### I-new-1 — Stale "Phase 5 prelude" reference at L229 contradicts post-renumbering layout

**Citation:** L229 ("**A4 (added per R0 I7):** … **DEFERRED to Phase 5 prelude** (manual writing) …").

**Conflict:** the C1 fold removed old Phase 3 ("ToolkitError variant additions") and renumbered: old Phase 5 (manual) → new Phase 4. This is reflected correctly at L247 ("Phase 4 (was 5) — Manual chapter"), L249 ("Phase 4 prelude: complete A4"), L277 ("4 (manual + A4 prelude)"), and L323 fold-summary ("deferred to Phase 4 prelude"). But L229 inside Phase 0 still says "Phase 5 prelude". Post-renumbering Phase 5 is now "Cycle close (commit + tag + push + GH Release)" — NOT manual writing.

**Impact:** plan-doc author reading Phase 0's A4 deferral target will look at the new Phase 5 (cycle close) and find no manual-writing prelude there. Self-contradiction within the same brainstorm.

**Fix:** L229 — change "DEFERRED to Phase 5 prelude" to "DEFERRED to Phase 4 prelude" (matching L249/L277/L323). One-token edit.

## Verdict

**YELLOW** — 0 Critical / 1 Important (I-new-1: single-token stale "Phase 5 prelude" reference at L229 contradicting the renumbering applied elsewhere). All 15 R0 findings (2C+8I+5M) are otherwise correctly folded with no regressions. The one stray reference is a low-cost mechanical fix; after that single-token edit the brainstorm is GREEN and ready to commit.
