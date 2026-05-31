# Plan-doc R2 confirmation (after plan-R1 one-token fold) — descriptor-form symmetry (A1)

**Date:** 2026-05-31 · **Reviewer:** opus architect · **Repo SHA:** `ea8ba88` · **Target:** `design/IMPLEMENTATION_PLAN_descriptor_form_symmetry.md`
**Verdict: GREEN (0C/0I).** Plan-doc R0 gate SATISFIED — proceed to subagent-driven implementation.

> Closes the plan reviewer-loop: R0 RED 2C/4I/3m → R1 RED 1C/0I/2m → R2 GREEN.

## Confirmed
- **C1 fold correct end-to-end:** call site (plan:353) `bundle_run_concrete_descriptor(args, body, …)` (no `&`) ↔ fn param `args: &BundleArgs` ↔ `run`'s `args: &BundleArgs` (bundle.rs:171) ↔ interior `emit_unified(args,…)` / `self_check_bundle(&bundle, args)` (bundle.rs:698/:1804). All ref-correct.
- **Borrow/move clean:** `classify_descriptor_form(&body)` borrow ends before `body` moves into the call; `descriptor_body_no_csum(&body,…)` returns a `&str` tied to `body` which outlives it.
- **No drift:** cosmetic guard-span edit (:235-284/:286) is prose-only; confirms (not alters) I3's fork-after-guards placement.
- **R1 GREEN items stay GREEN:** C2 (JSON wire-shape + P3b mirror of cli_verify_bundle_multi_cosigner_mk1.rs:114-161), I1 (imports), I2 (verify_emit_from_expected = :867+:871-902), I3 (fork at :285 after guards), I4 (4 version sites). None touched by the C1 fold.

**PLAN R2 GREEN.** Controller proceeds to implementation.
