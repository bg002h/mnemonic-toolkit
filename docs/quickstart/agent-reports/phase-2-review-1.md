# Phase 2 — spec-compliance review, round 1

**Date:** 2026-05-08
**Branch:** `quickstart/v0_1` at commit `8660014`
**Reviewer:** feature-dev:code-reviewer (spec-compliance focus)
**Verdict:** SPEC_GAPS

---

## Findings

### Critical

**C-1: DANGER box absent from ch23, ch24, ch26 (spec §5:126 + Q6).** Spec §5 states: "DANGER box once per chapter that uses the canonical seed; subsequent re-uses cross-reference." Q6: "DANGER box on every chapter that uses the canonical seed — manually audited."

- `23-bundle.md` — canonical seed phrase appears at line 12 (bundle command). No `:::danger` box and no cross-reference to ch22's DANGER box.
- `24-verify.md` — canonical seed phrase appears at line 17 (verify-bundle command). No `:::danger` box and no cross-reference.
- `26-recover.md` — canonical seed strings appear at lines 21, 43, 66 (three convert/decode commands). No `:::danger` box and no cross-reference.

Fix: add either a `:::danger` block or a one-line cross-reference sentence (e.g., "Remember: this is the public test phrase — see ch 22.") at the first use of the canonical seed in each of ch23, ch24, ch26.

---

## Passing checks

- **H1 titles** (Tasks 2.1-2.6): all match plan Body column.
- **Forward-pointer chain** (spec §5, Q9): 21→22→23→24→25→26→Part III/IV. All six chapters end with explicit forward-pointer.
- **Section structure**: All task-level sections present in each chapter.
- **ch22 DANGER box syntax**: `:::danger` fenced div at lines 40-49. Matches `primer-box.lua` `is_danger` path. Body text newcomer-voiced (explains "public," "swept," "within seconds"). Satisfies Task 2.2 step 2, Q5, N-2.
- **ch25 mermaid block**: `flowchart TD` present at lines 9-23. Covers entropy → bundle → verify → stamp → re-decode → geographic separation per Task 2.5 step 1.
- **Transcript drift (Q3 cross-check)**:
  - ch23 bundle command matches `22-first-bundle.cmd` exactly.
  - ch24 verify command matches `23-verify.cmd` exactly (all card strings; two `--mk1`; three `--md1`).
  - ch26 step 1 matches `24-recover.cmd`; step 2 matches `24-recover-mk1.cmd`; step 3 matches `24-recover-md1.cmd`.
- **ch26 API form**: positional `md decode` (Task 2.6 step 1). No broken pre-v0.8 `--mk1`/`--md1` flags on `md decode`.
- **ch24 verify-bundle flags**: `--network`, `--template`, `--slot @0.phrase=`, `--ms1`, `--mk1`, `--md1` — all validated against `mnemonic-verify-bundle.txt`.
- **D1 (zero Bitcoin background assumed)**: All six chapters introduce terms inline.
- **Spec §5 chapter count**: exactly 6 chapters in `20-singlesig/` (21-26).

---

## Required before SPEC_COMPLIANT

Resolve C-1 only (one gap, three files). Add a DANGER box or explicit cross-reference sentence in `23-bundle.md`, `24-verify.md`, and `26-recover.md` at the first appearance of the canonical seed.
