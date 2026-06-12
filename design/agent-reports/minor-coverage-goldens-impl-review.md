# Impl Review — minor coverage cluster (GAP 5) — self-review
Reviewer: orchestrator (Fable 5), 2026-06-12.

## Verdict: GREEN (0C/0I)

Two independent test-only commits (5a md-codec a3abdc8; 5b toolkit) implementing the R0-GREEN plan exactly. R0 round 1 was GREEN (0C/0I, 5 minors) and empirically resolved both 5b probes.

## 5a — md-codec goldens (committed a3abdc8)
- 6 cells pass (`self_test_wsh_multi_17_of_20`/`_17_of_17`, `_wsh_and_v_pk_after_800000`, `_tr_nums_and_v_{hash256,ripemd160,hash160}_pk`); each derived once via `p6_chain`, prefix-verified, pinned. **No render bug** (all 6 reparse fixed-points green — the multi 17..=20 window, which had ZERO render/address coverage before, renders correctly). Full md-codec suite GREEN; clippy `-D warnings` clean; `cargo fmt --all --check` clean. Folded M1 (stale comment → 32-key truth), M2 (`hash20` import), M4 (the optional 17-of-17 cap+1 cell included). NO-BUMP (test + comment).

## 5b — toolkit verify-bundle cells (this commit)
- 2 cells pass: `hashlock_wsh_and_v_sha256_round_trips_via_bundle_json` (bundle→verify-bundle round-trip — the genuinely-absent hashlock verify-bundle surface; mirrors the existing andor cell) and `verify_bundle_refuses_bip388_policy_json` (pins the current refusal — exit 2 + "mixes @N placeholders with inline keys" — documenting the intake asymmetry vs bundle/export-wallet; FOLLOWUP `verify-bundle-bip388-policy-intake` tracks the feature that flips it red-then-green). Both invocations are the R0-probe-verified forms (M3/M5 folded: the exact mixed-form classifier string pinned, not "miniscript parse error"). The timelock sub-goal was correctly dropped (already covered). clippy clean; fmt clean (only mlock.rs exempt-diff). NO-BUMP.

## Scope / lockstep
- 5a and 5b are INDEPENDENT (different repos; toolkit pins md-codec `=0.35.2` unchanged — no lockstep). No clap surface change → no manual/GUI/schema_mirror in either. No src change in either (md-codec renderer + toolkit verify-bundle are TESTED, not changed).

Cleared. NO-BUMP both repos.
