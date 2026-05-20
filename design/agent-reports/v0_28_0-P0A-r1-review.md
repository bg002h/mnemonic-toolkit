# Phase P0A — architect R1 review

**Reviewer:** Opus 4.7 via feature-dev:code-architect
**Branch:** `v0.28.0/p0a-spec-scaffolding`
**Commit under review:** `87cb7e6` (R1 fold commit; preceded by `aa3a537` P0A scope + `12c248f` cycle-followups)
**Source SHA verified against:** working-tree at `87cb7e6`
**Previous review:** [`v0_28_0-P0A-r0-review.md`](v0_28_0-P0A-r0-review.md) (YELLOW, 4 Important)

---

## Fold verification (R0 → R1)

| R0 finding | R1 fold applied? | Verified |
|---|---|---|
| **I1** Electrum wallet_type set incomplete | YES — §11.6 now carries Electrum-4.x-post-upgrade scoping footnote citing `electrum/wallet_db.py::_convert_wallet_type`; 4-value set retained; FOLLOWUP `wallet-import-electrum-pre-4x-legacy-types` named (conditional filing) | SPEC §11.6 |
| **I2** `bitcoin_core.rs:62` → `:74` citation | PARTIALLY — citation updated to `:74` in both §6.1.1 and §12, BUT the const has now drifted to `:81` post-R1-doc-comment-expansion (see N1) | NO — drift recurred |
| **I3** `label` removal | YES — removed from VENDOR_MARKER_KEYS in source + SPEC | Consistent |
| **I4** Jade `register_multisig` removal | YES — removed; `multisig_file` retained; §11.5 wording clarified | Consistent |

## Critical (correctness-blocking)

**None.**

## Important (would block P0A merge)

**N1 — Recurring off-by-N: `bitcoin_core.rs:74` citation has drifted to `:81` after R1's doc-comment expansion.** R0→R1 I2 fold correctly updated SPEC's citation from `:62` to `:74` to point at the const declaration. However, R1 I3 + I4 folds expanded the doc-comment above the const (7 lines of R0-fold-rationale text). This pushed the const declaration down from line 74 to line **81**.

Drift sites:
- SPEC §6.1.1 line 94: `wallet_import/bitcoin_core.rs:74` — should be `:81`
- SPEC §6.1.1 line 94 parenthetical: "lines `:59-72` are the doc-comment" — should be `:59-80`
- SPEC §12 line 542: `bitcoin_core.rs:74` — should be `:81`

Verified by source grep at SHA `87cb7e6`:
```
81:const VENDOR_MARKER_KEYS: &[&str] = &[
```

**Fold:** update SPEC §6.1.1 + §12 citations from `:74` to `:81` (and `:59-72` to `:59-80`).

**N2 — §A summary table row still claims "expanded with 10 new format markers" but actual is 8.** SPEC §A line 27 says "10 new format markers". After R1 I3 + I4 removed `label` and `register_multisig`, actual addition count is **8** (`seed_version`, `wallet_type`, `policyType`, `defaultPolicy`, `keystores`, `devices`, `blockheight`, `multisig_file`). The detailed §6.1.1 body correctly says "expands from 5 to 13 entries" (5+8=13), but the summary table contradicts.

**Fold:** change SPEC §A line 27 from "10 new format markers" to "8 new format markers".

## Minor

- M1: I1 FOLLOWUP filing is conditional — defensible; no fold required.
- M2: Doc-comment lines 68-72 verb-tense slightly awkward — cosmetic only.
- M3: SPEC §11.5 vs source doc-comment framing on `get_registered_multisig` — both consistent; no fold.
- M4: Specter false-positive re-verified — `blockheight` + `devices` remain load-bearing; no risk.
- M5: SPEC §6.1.1 internally consistent (13 entries listed, "5 to 13" claim matches).

## Scope-creep audit

No new scope additions in R1. Pure fold round.

## Overall verdict

**YELLOW.**

R0 folds for I1, I3, I4 applied correctly. But two findings introduced BY the R1 folds require a second R-round:
- **N1 (Important)** — citation drift recurrence: I2 fold updated `:62` → `:74`, but I3+I4 fold expansions pushed const down to `:81`. Drift in §6.1.1 + §12 + §A.
- **N2 (Important)** — §A summary "10 new format markers" contradicts §6.1.1 body's "5 to 13" (actual: 8 new).

Both are simple SPEC text edits. Recommend one R2 round.

### R2 fold recommendations

1. **N1 fold:** update SPEC §6.1.1 line 94 from `bitcoin_core.rs:74` to `bitcoin_core.rs:81` AND `:59-72` to `:59-80`. Update SPEC §12 line 542 identically.

2. **N2 fold:** update SPEC §A line 27 from "10 new format markers" to "8 new format markers".

After R2, expect 0C/0I and GREEN for P0A merge. Risk of new R2 drift is low (edits don't shift line numbers in source or SPEC).

---

**Sources:**
- Working-tree `crates/mnemonic-toolkit/src/wallet_import/bitcoin_core.rs` at commit `87cb7e6`
- Working-tree `design/SPEC_wallet_import_v0_28_0.md` at commit `87cb7e6`
- Previous round: `design/agent-reports/v0_28_0-P0A-r0-review.md`
