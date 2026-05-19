## Appendix D — Architect R3 review (verbatim; persistence-debt-noted)

**Persistence note:** plan mode prevented direct write to `design/agent-reports/v0_28_0-plan-r3-review.md`. The orchestrating session MUST copy this appendix verbatim to that path before any Phase P0A execution.

R3 was scoped to the new §"Parallel execution plan" section only; the body (Phases P0-P15 + SPEC) remained GREEN per R2.

---

```
# v0.28.0 plan-doc — architect R3 review (parallel-execution section)

## R3 Critical (new — both folded in R4)

### R3-C1. Instance E (jade) has source-code dependency on Instance D (coldcard-multisig); Wave 1 is NOT 8-way parallel
[Folded R4: Wave-1 table now lists E with explicit "P5A may run parallel with D; P5B/C BLOCKED until D's P4B merges" dependency. New §"Cross-instance dependency: D → E" subsection. Recommended option (a): hard-gate E on D's P4B merge.]

### R3-C2. `sniff_format` 2-parser→8-parser dispatch-shape rewrite has NO assigned owner
[Folded R4: new sub-phase P0D added to Wave 0 ("`sniff_format` dispatch-shape rewrite") that refactors 2×2 truth-table → consult-all-then-count + pre-stubs 6 `false` placeholder bools. Per-parser P{N}A flips ONE bool. Conflict table sniff.rs row updated to reflect new pattern. Wall-clock estimate updated (Wave 0 now 5 sub-phases not 4).]

## R3 Important (new — all folded)
- R3-I1: 6th hazard for `Agent isolation: "worktree"` stale-branch-reuse (claude-code#51596) added.
- R3-I2: Instance G split into G1/G2/G3 (BSMS parser / BSMS taproot / BSMS fixtures).
- R3-I3: Instance H split into H (Core fixtures) + I (compare-cost-tr).
- R3-I4: Wave-1 conflict table `sniff.rs` row split into two entries (variant additions + sniff_format body).
- R3-I5: P15 GUI schema-mirror-delta checklist added; explicit anti-anti-pattern mitigation.

## R3 Minor (all folded)
- R3-M1: 21-day total recomputed → 19 days post-splits.
- R3-M2: Instance G "3 PRs" reconciled with G1+G2+G3 split.
- R3-M3: Instance H "2 PRs" reconciled with H+I split.
- R3-M4: 8-vs-10 instance count reconciled (max theoretical = 10; practical sweet-spot = 2-3).
- R3-M5: PR-count total recomputed → ~14 PRs total.
- R3-M6: ASCII diagram lane count reconciled with 10-way fan-out (left as-is; ellipsis-with-n=10 note added).
- R3-M7: Single-orchestrator-execution-only `--admin`-bypass assumption documented.
- R3-M8: `Agent.isolation: "worktree"` parameter verified at the agent tool description level + claude-code#51596 bug citation.

## Overall R3 verdict

YELLOW (parallel-execution section); body remains GREEN per R2. Recommended: apply R4 folds for R3-C1 + R3-C2, then re-dispatch R4 review. R3-Important folds are quality-of-execution improvements; all applied.
```

---

---

