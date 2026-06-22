# R0 REVIEW (Round 1) — `PLAN_sibling_pin_check_green.md`

**Reviewer:** opus architect. **Pinned SHA:** `7cd3ccf1` (HEAD, v0.70.1). **Date:** 2026-06-22.
**Persisted verbatim per project discipline.**

## VERDICT (as delivered by reviewer): 0 Critical / 1 Important / 2 Minor

> **Orchestrator note (ground-truth reconciliation, appended at persist time):** The reviewer's **I1 is itself factually mistaken** — it claimed the `manual`/`technical-manual` workflow is live-RED citing run `27967547684`, which is a **pre-session stale run**. Current `gh run list` on HEAD `7cd3ccf1` shows **`technical-manual` = success**. The `verify_bundle.rs:2861` enriched string is on a code path NOT exercised by `41-inheritance.cmd` (its transcript hits a shorter `match expected (multiset)` emit), so there is no CI-breaking transcript drift. HOWEVER, the reviewer's *instinct* ("§5.4's all-green criterion is unsatisfiable; a gate is red") was correct for a DIFFERENT gate: **`changelog-check` is live-RED on HEAD** (run `27989458427`) because the v0.70.1 tag was pushed without a `## mnemonic-toolkit [0.70.1]` CHANGELOG section (a release-ritual miss in the v0.70.1 ship, separate from this pin chore). The fold therefore corrects §5.4/§6 to the TRUE CI state and the CHANGELOG miss is fixed as a separate release fixup (re-point the tag). The substantive engineering verdict (0 Critical; Option A correct; NO-BUMP; funds-wire-neutral by corpus structure) stands and is independently confirmed.

---

## Verification results (reviewer's 6 load-bearing points)
All 6 CONFIRMED by the reviewer:
1. Drift = exactly the two md-cli sites (`cross-tool-differential.yml:46`, `manual.yml:86`) at v0.6.2 vs install.sh:35 canonical v0.7.1; mk/ms aligned.
2. `cross-tool-differential.yml:39-42` documents the baseline as tracking `install.sh:35`, moved deliberately → **align (Option A), not exclude**; `sibling-pin-check.yml` has no per-line exclusion (only whole-file self-skip :84) → exclude would need net-new gate logic.
3. md-cli **source is byte-identical** v0.6.2→v0.7.1 (`git diff --stat md-cli/` empty); only delta = Cargo.toml md-codec pin `=0.37.0`. Zero new CLI flag.
4. Flag-coverage is one-directional (binary flag ⟹ must be in manual; `lint.sh:84-94`); md adds zero flags → no manual cascade; 42-md.md needs no edit.
5. Leaving ms at v0.7.0 keeps the frozen g6 mlock byte-anchor untouched (`rust.yml:40-49`) → full de-stale correctly scoped out.
6. NO-BUMP correct (CI-only).

**Funds-neutrality (decisive, corpus-structural):** the differential (`cli_cross_tool_differential.rs:12-13,148-153`) compares `wallet_policy_id` + `wallet_descriptor_template_id` (**encode-side ids, not derived addresses**); every multi-key corpus row uses an identical `/<0;1>/*` suffix on all keys (no divergent-per-cosigner-suffix rows), so the #25/md-codec-0.37.0 fix has **no behavioral trigger** in this corpus on either side → Match holds by construction.

---

## Findings

### IMPORTANT
**I1 — [PARTIALLY RETRACTED — see orchestrator note]** Reviewer claimed `technical-manual`/`manual` is live-RED (a `41-inheritance.cmd` verify-examples transcript drift from `verify_bundle.rs:2861` enrichment, commit 4ed3bbe9 vs fixture `41-inheritance.out:29`) and that §5.4's "technical-manual green" criterion is unsatisfiable. **Ground truth:** technical-manual is GREEN on HEAD (reviewer cited a stale pre-session run). The real RED gate is **`changelog-check`** (v0.70.1 CHANGELOG miss). **Fold:** correct §5.4/§6 to the true CI state; fix the CHANGELOG miss as a separate v0.70.1 release fixup (add the `[0.70.1]` section + re-point the tag); do NOT chase the non-manifesting transcript drift.

### MINOR
**M1** — §3 wire-neutrality should lead with the corpus-structural proof (ids-not-addresses; no divergent-suffix rows) rather than the weaker "toolkit already carries the fix" argument; the local differential run becomes a confirmation, not the sole guarantor.
**M2** — make the `cross-tool-differential.yml:43-44` comment-refresh NON-optional: when bumping `:46` to v0.7.1, update the stale "md-codec 0.35.0→0.35.1" skew note to cite `=0.37.0` + its corpus-structural wire-neutrality.

---

## Rulings
- **Option A (align) vs exclude → Option A CORRECT** (documented design intent; exclude needs net-new gate logic = strictly worse).
- **Local differential-Match run SUFFICIENT** for the funds-derivation risk (corpus is structurally insensitive to the #25 fix; Match guaranteed by construction; local run is cheap confirmation). A divergent-suffix regression row would be a *separate* test-coverage cycle (touches the `assert_eq!(entries.len(), 17)` pin) — out of scope.

**Reviewer's gate line:** NOT GREEN pending I1 fold. **Post-reconciliation:** the engineering is 0C/0I; I1 reduces to a prose correction (CI-state) + a separate CHANGELOG fixup. Fold M1/M2 + correct §5.4/§6, then proceed.
