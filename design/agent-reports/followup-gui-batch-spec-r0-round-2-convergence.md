# Convergence R0 — SPEC_followup_gui_batch.md (round 2) — Fable, adversarial. VERDICT: GREEN (0C/0I)

C1/I1/I2 + M1/M2/M5 all folded + re-verified vs live source + binaries.
- C1: choices-only feasible with ZERO src/ change (`json_flag_choices(cli_name,…)` + `run_gui_schema_json` already per-CLI, all 4 `gui-schema-capable`). md/ms/mk `gui-schema`=v1, 0 default_value, non-null choices on 16 flags; day-one choices drift = 0 (all 16 dropdowns match value+order). One-sided defaults guard vacuous-not-broken (serde-default + version≥1 parse → Some(map-of-None) → passes; self-arms at v5, catching the 7-omission F6 class). 3-category STOP-clause + cross-repo FOLLOWUP (7 backfills enumerated) present.
- I1: h?-ONLY suffix-group mutation (conditional.rs:110/112/114, 2nd `(?:/\d+'?h?)*` after `@\d+`); whole-tree grep = exactly ONE suffix-origin fixture (L12 apostrophe) + zero h-suffix → h?-only isolates. Fixture Expect=canonical re-confirmed live at pinned 0.75.0.
- I2/M1/M2/M5: (cli,subcommand,flag) keying (collisions verified); per-CLI resolver in tests/ (mirror schema_mirror.rs:47); fixture-count 25→26; gui-schema (no --schema).
Minor nits (no re-fold): FOLLOWUP filing locations (GUI = repo-root FOLLOWUPS.md; companions in siblings' design/FOLLOWUPS.md); trailer sub-counts; local binaries patch-newer than pins (CI re-captures via *_BIN=pinned).

**GUI batch implementation may begin.**
