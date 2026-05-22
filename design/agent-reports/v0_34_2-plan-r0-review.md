# v0.34.2 nostr --import — plan-doc opus R0 review (verbatim)

**Date:** 2026-05-22
**Reviewer:** opus `feature-dev:code-reviewer` (agent `a56feb224f9ac5b30`)
**Target:** `design/IMPLEMENTATION_PLAN_v0_34_2_nostr_import_watchonly.md` (commit `dc98639`) vs spec + source `1d6436d`.
**Verdict:** **GREEN** — 0 Critical, 0 Important, 4 Minor.

## Verified (all six checks pass against real source)
1. **Shared-helper path/visibility OK.** `crate::wallet_export::import_array_single` resolves from `cmd/nostr.rs` (17 existing `crate::wallet_export::` refs; re-export alongside `pub(crate) use bitcoin_core::BitcoinCoreEmitter` `mod.rs:32`). `to_json` (`pub(super)` `mod.rs:147`) reachable in `bitcoin_core.rs` (existing fn calls it `:71,:81`). `TimestampArg: Copy` (`mod.rs:140`).
2. **parse_timestamp reuse OK.** `pub(crate) fn` + fully-qualified `value_parser` path has live precedent (`nostr.rs:40` `crate::cmd::convert::parse_script_type_arg`). `TimestampArgValue` ALREADY derives `Clone` (`export_wallet.rs:201`) → plan's "add Clone if asked" is moot. `default_value="0"` → `TimestampArg::Unix(0)` (passes the `n>=0` guard).
3. **run insertion OK.** `rows: Vec<OutputRow>` exists pubkey `:96-104` + secret `:153-161`; `OutputRow.descriptor` `:60`. `import_recipe.clone()` (json branch) + `&import_recipe` (text branch) are mutually exclusive → no move issue.
4. **Test parse OK.** `split("importdescriptors '").nth(1)…split("'\n")` is robust (compact JSON has no `'`).
5. **Spec coverage/SemVer/lockstep OK.** Watch-only only; 3 FOLLOWUPs; PATCH v0.34.2; GUI schema_mirror + manual lockstep; no secret-projection; install.sh self-pin bump.
6. **No-regression OK.** New fn + new re-export; ranged path untouched.

## Critical / Important — None

## Minor (folded post-review)
- **M1 (conf 85):** Plan line 7 / §3 say "generalize the existing emitter" but Task 1 actually adds a SEPARATE `import_array_single` (correct, lower-risk). Reword to "add a sibling non-ranged builder; do NOT refactor the ranged path" (prevents an over-eager refactor breaking the byte-exact v0.7 bitcoin-core fixture).
- **M2 (conf 80):** Task 2 Step 6 "AFTER the per-row loop" places `import:` BEFORE `wif:` in the secret path (`:184`). Cosmetic (tests use `contains`). Fix: secret path → after the `wif:` line; pubkey path → after the loop (import last in both).
- **M3 (conf 85):** Task 4 says `--import` "(text/dropdown)". Per `gui_schema.rs:34-45`, a custom `value_parser` collapses to `kind:"text"` (like `--script-type`). The GUI hand-schema MUST register `--import` + `--timestamp` as `text` or `schema_mirror` fails on the pin bump. Resolve to `text`.
- **M4 (conf 80, informational):** `args.import == Some(ImportMode::ReadOnly)` needs `ImportMode: PartialEq` — the plan's enum derives it. No change.

## Verdict: GREEN — cleared to implement
M1 + M3 folded to prevent execution mistakes (ranged-path refactor; GUI schema kind). M2 cosmetic fold. M4 no-op.
