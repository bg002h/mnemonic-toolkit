# Plan-R0 round 2 — architect review (verbatim) — PLAN_older_timelock_advisory.md — **GREEN**

> Persisted verbatim per CLAUDE.md. Formal R0 gate on the IMPLEMENTATION PLAN, round 2. Dispatched via
> Agent tool (feature-dev:code-architect, inherited session default model Opus 4.8; body's self-attribution
> is the agent's own). **Verdict: GREEN (0 Critical, 0 Important; 2 cosmetic Minors).** The plan-R0 gate
> PASSES — implementation may proceed. The 2 cosmetic minors (m-5, m-6) were folded post-GREEN. Branch
> `older-timelock-advisory`, HEAD `34562c6`.

---

# Plan-R0 Round 2 — Formal Review

**Artifact:** `design/PLAN_older_timelock_advisory.md`, branch `older-timelock-advisory`, HEAD `34562c6`
**Date:** 2026-06-13
**Gate:** 0 Critical AND 0 Important required for GREEN.

## Round-1 Fold Verification — all correct

- **I-1:** Task 8 Step 3e present; enumerates `gate.rs:~1146/~1202/~1263`; Step 4 now runs `cargo test --bin mnemonic`. The "complete caller set" note matches grep exactly: `cmd/compare_cost.rs:98`, `cmd/build_descriptor.rs:500`+`:530`, `gate.rs:1146`/`:1202`/`:1263` — no other callers found across the entire repo (exhaustive grep). Fold correct + complete.
- **m-1:** Task 3 Step 2 note corrected (`older_advisories_node`, no `md_codec::Descriptor` literal). Correct.
- **m-2:** Task 10 Step 3b cites `descriptor_intake.rs:140`. Correct.
- **m-3:** `Tr` arm has the `&Arc<T>→&T` deref-coercion comment. Correct.
- **m-4:** `s512` renamed to `stray_blocks` (operand `0x0080_0064`, bit-23 stray → Blocks). Correct.

## API Verification (independent re-check)
- `run_compare_cost` current sig (`cost/mod.rs:123`) single `W`; adding `E` is the correct step.
- `Translated.segv0` (`cost/translate.rs:22`) = `Miniscript<DefiniteDescriptorKey, Segwitv0>`; `older_advisories_ms(&translated.segv0)` type-correct.
- `cmd::compare_cost::run` (`compare_cost.rs:67-98`) `<R,W>` no stderr; `main.rs:198` dispatch; the 3a→3e propagation covers all sites.
- Gate arm `gate.rs:257-296`: predicate `:264` matches `older_consensus_masked`; format strings `:280-293` match the characterization substrings; Task 2 refactor byte-identical by construction.
- Hook sites confirmed accurate: `import_wallet.rs:1285`, `bundle.rs:1662`/`1953`, `verify_bundle.rs:1028`, `restore.rs:1291`, `export_wallet.rs:452`/`721`, `descriptor_intake.rs:290`/`140`/`227`.

## Critical — None.
## Important — None.

## Minor (cosmetic, no execution risk)
- **m-5:** Task 3 Step 2 heading says "verify it fails" but expected is PASS (adapters written in Task 1). Copy-paste TDD-template artifact; the parenthetical clarifies. Reword the heading.
- **m-6:** Task 5 bundle Site 2 cites `emit_unified` at `~:1978`; `emit_unified` is reached after the `BundleMode` derivation block (`1955-1977`). The hook snippet places the advisory right after `synthesize_descriptor(...)` where `descriptor` is in scope — placement correct regardless of the exact `emit_unified` line; all `~:N` are approximate guides. No execution impact.

## Sequencing & Coverage
13-task order sound (module → gate → adapters → surfaces 4-10 → cross-surface → locksteps → verify); no forward dependency; each task TDD-ordered. Task 2 characterization runs BEFORE the refactor (baseline) then AFTER (byte-identity) — correct order. Spec coverage complete: §3.1→T1, §3.2→T1+T3, §3.3→T1+T10, §4 (7 surfaces)→T4-10, §5→T1, §6→T1-3+T4-10+T11, §7→T2+T12+T13. No gaps.

## Verdict
**GREEN** — 0 Critical, 0 Important, 2 cosmetic Minors. The plan-R0 gate passes. All five round-1 findings folded; complete `run_compare_cost` caller set grep-verified exhaustive; all 7 hook sites accurate; API usage compiles against miniscript `95fdd1c` + md-codec 0.35.3. **The plan is ready for implementation.**
