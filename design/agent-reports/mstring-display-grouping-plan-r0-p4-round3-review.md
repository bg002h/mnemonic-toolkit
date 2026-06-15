# Plan-R0 (P4 toolkit) round 3 — architect review (verbatim) — GREEN

> Persisted verbatim per CLAUDE.md. Dispatched via Agent (feature-dev:code-architect,
> Opus 4.8). **Verdict: GREEN — 0 Critical / 0 Important.** Plan SHA at review:
> toolkit `7bf6ee7`. Independent live grep confirms the r2 exhaustive lists +
> "fix every RED" backstop jointly bound the surface; emit sites + release ritual
> confirmed.

## Round-2 fold verification
- C1 (exhaustive ms1/mk1/md1 filter lists) — RESOLVED; live grep of all `!contains(' ')`/`!contains('-')` + `starts_with("ms1"|"mk1"|"md1")` occurrences (47 across 12 files) all present in the r2 lists; no additional found.
- C2 (5 convert/bundle pins) — RESOLVED.
- I1 (`cli_argv_leakage.rs` triple-role) — RESOLVED (in both 2c + 2d).
- I2 (`cli_env_var_sentinel.rs` dual-role) — RESOLVED.

## Critical / Important
None.

## Minor
- **m1** Task 7 prose says `.examples-build/gen.sh` "6×0.55.3" but live grep finds 9. The file is correctly named + the executor greps for the version (doesn't trust the count) → no failure. Drop/fix the count. (FIXED in plan: count removed.)
- **m2** `cli_verify_bundle_watch_only.rs:81` ms1 finder IS in the r2 list (confirmation).
- **m3** `cli_convert_round_trips.rs:89-97` second `--to ms1`→`--from ms1=` round-trip validates Task 3 + Task 4 together; asserts on the recovered PHRASE (not the ms1), passes once both land. Per-commit-green ordering + "run full cargo test after each task" catches any inter-task RED.

## Final completeness assessment (independent grep)
(a) `!contains(' ')`/`!contains('-')` filters: 47 occ / 12 files — all in r2 C1 lists. (b) `assert_eq!(stdout,…)` bundle/convert pins: `cli_bundle_full.rs:34`, `cli_env_var_sentinel.rs:302-303/393-394/428`, `cli_self_check.rs:34`, `cli_argv_leakage.rs:248-251`, `cli_convert_happy_paths.rs:154/326` — all listed. (c) `.len()` checks on CLI-stdout ms1/mk1/md1: `cli_convert_language_advisory.rs:151`, `cli_bundle_language_advisory.rs:64-67`, `cli_mnem_emit_preserve.rs:409` — all in r2 C2 (other `.len()` use `--json`/lib values → safe). (d) v0_1/v0_2 consumers: all 12+ accounted for. **No gap. r2 lists + "fix every RED" backstop jointly sufficient.**

## Emit-site confirmation (spec §9.1)
- `convert.rs:1119` `writeln!("{}: {}",node,value)` single-emit for `--to ms1/mk1` → group `value` (text only). ✓
- `ms_shares.rs:296-300` text-branch share loop → group (NOT before `if args.json` `:283`). ✓
- `ms_shares.rs::run_combine:465` emits `--to ms1` → group (text). ✓
- `convert.rs:1560` `value.split_whitespace()` Mk1 arm → strip `value` before. ✓
- `mk1_to_xpub_decode:197` outputs xpub/fp/path (NOT ms1/mk1) → NOT broken. ✓

## Release ritual / ordering
Lockstep complete: README ×2, install.sh self-pin, manual.yml+quickstart.yml mk-cli `v0.8.0`→`v0.9.0` (`manual.yml:79` confirmed), `.examples-build/gen.sh` (all occ), fuzz separate-build, mlock.rs fmt-exempt revert. Per-commit-green ordering serializes the coupled tasks (Task 2 atomic, Task 3 atomic emit+ms-shares-intake). "FULL re-verify before tag" present.

## Verdict
**GREEN — 0 Critical / 0 Important.**
