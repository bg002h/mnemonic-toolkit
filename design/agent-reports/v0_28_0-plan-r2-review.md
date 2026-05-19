## Appendix C — Architect R2 review (verbatim; persistence-debt-noted)

**Persistence note:** plan mode prevented direct write to `design/agent-reports/v0_28_0-plan-r2-review.md`. The orchestrating session MUST copy this appendix verbatim to that path immediately upon ExitPlanMode, before Phase P0A execution.

---

```
# v0.28.0 plan-doc — architect R2 review

**Reviewer:** Opus 4.7 (via feature-dev:code-architect)
**Plan-doc state:** post-R0-fold + post-R1-fold
**Source SHA reviewed against:** fcf9e6d

## R1 fold verification

All 21 findings folded correctly:
- R1-C1 (ImportProvenance Bsms→BitcoinCore reorder) ✓
- R1-C2 (P0B.1 truth-table test clarification) ✓
- R1-I1 (P11A executable start-gate) ✓
- R1-I2 (P0C arms before Some(other) at :239-243) ✓
- R1-I3 (§S.4 5-row xfp truth table + WARNING template + xfp_header_disagreed field) ✓
- R1-I4 (P12A two-assertion pair: prefix + parity-invariance) ✓
- R1-I5 (P14A keep dup stub OPEN; Keep-a-Changelog CHANGELOG) ✓
- R1-I6 (derive_first_address at derive_address.rs:26) ✓
- R1-I7 (build_tr_descriptor at cost/translate.rs:101) ✓
- R1-M1 through R1-M7 all folded ✓

## R2 new findings

### R2 Critical: NONE
### R2 Important: NONE
### R2 Minor: 1

**R2-M1.** §S.11 P12B prose "Existing build_tr_descriptor at cost/mod.rs:148 reused" is ambiguous (definition vs call site). Cosmetic only; canonical citation in §B.2 #11 + §S.11 already establishes the distinction. Folded inline.

## 27-citation source verification

All 27 file:line references re-verified against origin/master @ fcf9e6d.
None drifted. None wrong.

## Overall R2 verdict

GREEN. ExitPlanMode recommended.

Mandatory post-ExitPlanMode actions:
1. Copy Appendix A → design/agent-reports/v0_28_0-plan-r0-review.md
2. Copy Appendix B → design/agent-reports/v0_28_0-plan-r1-review.md
3. Copy this R2 review → design/agent-reports/v0_28_0-plan-r2-review.md

Then begin execution at Phase P0A.
```

---

---

