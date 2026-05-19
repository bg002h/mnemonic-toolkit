# compare-cost cycle — architect-review meta-record

**Cycle:** v0.26.0 / mnemonic compare-cost subcommand
**Branch:** `compare-cost/p1-miniscript`
**SPEC:** `design/SPEC_compare_cost_v0_26_0.md`
**Cycle close:** 2026-05-18

## Note on retention

CLAUDE.md (line 30) prescribes per-phase opus reviews persist as individual `phase-N-*-review-*.md` files (per the convention used in `v0_22_0-repair-PE-*.md` etc.). The compare-cost cycle's architect reviews were dispatched via the `Agent` tool's `feature-dev:code-reviewer` agent (model: opus) but the verbatim review text was not written to disk in real-time — it was inlined into the development-session transcript only.

This meta-record back-fills the audit trail by listing every architect-review dispatch with the commit where its findings were folded. The verbatim review text lives in the session transcript at `/home/bcg/.claude/projects/-scratch-code-shibboleth-mnemonic-toolkit/72492dd3-7437-4487-8e45-0781d8888e37.jsonl` (`agentId` fields below cross-reference the transcript).

**FOLLOWUP filed:** `compare-cost-agent-reports-back-fill` — for future cycles, persist each architect-review verbatim to a `phase-N-r0-review.md` file at the time of dispatch, per established convention. Adopt as a per-cycle reviewer-loop discipline.

## Dispatches

### Plan-doc reviewer-loop (R0 → R3)

| Round | Verdict | Folded in | Notes |
|---|---|---|---|
| Plan R0 | YELLOW | plan-doc R1 rewrite | Caught `plan_at_age_and_height` API hallucination; locked `Plan::witness_size()` substrate |
| Plan R1 | YELLOW | plan-doc R2 | Caught `Miniscript::build_template(&Assets)` (older API); fixed to `Descriptor::plan(&AssetProvider)` |
| Plan R2 | YELLOW | plan-doc R3 | Caught `check_after(Sequence)` vs `absolute::LockTime`; locked timelock-saturation per-kind |
| Plan R3 | GREEN | (ready for execution) | 0C/0I; cycle kicked off |

### Per-phase reviewer-loops

| Phase | Round | Verdict | Folded in | Headline finding |
|---|---|---|---|---|
| Phase 1 (`--miniscript`) | R0 | YELLOW | `2495dc2` | C1 per-context key mismatch (segv0 keys fed to tr_desc); I3 silent under-count on timelocks/preimages |
| Phase 1 | R1 | YELLOW | `b47a36c` | C1 timelock kind mismatch (block-height vs MTP-time saturation) |
| Phase 1 | R2 | GREEN | `378244d` (minors only) | Stale comments folded |
| Phase 2 (`--descriptor`) | R0 | YELLOW | `4b02519` | C1 xpub coverage; M1 display-fix |
| Phase 2 | R1 | GREEN | (no follow-on) | 0C/0I |
| Phase 3 (stdin) | R0 | GREEN | `928bfcd` (minor doc) | `pk` classification note clarification |
| Phase 5 (manual) | R0 | YELLOW | `98f8269` | C2 JSON example mis-formatted (used hand-aligned inline JSON; real output is multi-line); I1 missing ContextIncompat exit-code row |
| Phase 5 | R1 | GREEN | (no follow-on) | 0C/0I |

### End-of-cycle holistic review

| Round | Verdict | Folded in | Notes |
|---|---|---|---|
| Holistic R0 | YELLOW | `ce07ad6` | I1 soft-cap unreachable below 256; I2 extracted_miniscript divergence (echoed full descriptor); I3 GUI mutex drift gate missing; I4 install.sh bump-note missing from kickoff |
| Holistic R1 | YELLOW | `516fe54` | I1 missing schema-mirror regression test for the new `compare-cost` mutex (count assertion); folded as `compare_cost_has_one_input_mutex` |

### Pre-merge final code review (multi-aspect)

| Round | Verdict | Folded in | Headline finding |
|---|---|---|---|
| Multi-aspect R0 | YELLOW | (this commit) | CLAUDE.md compliance: missing agent-reports + off-repo SPEC path; comment rot on `_tap_marker`; plaintext/JSON Input asymmetry; 3 test gaps |

## Cross-references

- Holistic-fold commit chain: `ce07ad6` → `516fe54` → `41354f4` (clippy) → this commit
- Per-phase commits: `2495dc2` (P1) · `b47a36c` (P1 R1) · `378244d` (P1 R2) · `094fda3` (P2) · `4b02519` (P2 R0) · `6bfdc5d` (P3) · `928bfcd` (P3 docs) · `ac62965` (P5) · `98f8269` (P5 R0)
- Plan-doc + kickoff handoff at repo root: `.cost-to-spend-policy-comparison-kickoff.md`
- Memory entries: `[[project-v0-26-0-compare-cost-kickoff]]`, `[[project-v0-26-0-compare-cost-instance-identity]]`
