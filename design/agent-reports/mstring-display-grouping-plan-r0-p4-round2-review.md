# Plan-R0 (P4 toolkit) round 2 — architect review (verbatim)

> Persisted verbatim per CLAUDE.md. Dispatched via Agent (feature-dev:code-architect,
> Opus 4.8). **Verdict: NOT GREEN — 2 Critical / 2 Important** (round-1 4C/4I all
> RESOLVED; completeness sweep found a materially incomplete affected-test
> enumeration). Plan SHA at review: toolkit `f41cf82`. The exhaustive file:line
> lists below ARE the execution checklist for Task 2c/2d/3.

---

## Round-1 Fold Verification
- **C1** (ms-shares atomic emit+intake) — RESOLVED (Task 3 owns both; `cli_ms_shares.rs` listed).
- **C2** (`cli_convert_happy_paths.rs:154`) — RESOLVED (but same file has more pins, see new C2).
- **C3** (ms1/mk1/md1 filter fixes by type) — RESOLVED partially (recipe correct; file list incomplete — see new C1).
- **C4** (verify_bundle collection-top strip) — RESOLVED.
- **I1** (ms-shares ordering) — RESOLVED.
- **I2** (convert Mk1 strip before `split_whitespace`, `:1560` confirmed) — RESOLVED.
- **I3** (ms-shares `--json` isolation, `:283`/`:296-300` confirmed) — RESOLVED.
- **I4** (Task 7 grep + `.examples-build/gen.sh`) — RESOLVED.

## CRITICAL (new)

**C1 — Task 2d/2c file lists materially incomplete.** Live grep found 8+ more files with the `!contains(' ')`/`!contains('-')` fixture-parser filters that PANIC after print-once, PLUS `cli_argv_leakage.rs:248` is an output-golden (`assert_eq!(stdout, expected)` vs `v0_1/bip84-mainnet.txt`) not in Task 2c. **Exhaustive fix lists (USE THESE):**

*ms1 (`find(|l| l.starts_with("ms1") && !l.contains(' '))` → `find(starts_with("ms1"))` + `strip_display_separators`):* `cli_verify_bundle_full.rs:16`, `cli_verify_bundle_forensics.rs:19`, `cli_verify_bundle_seedqr_slot.rs:15`, `cli_secret_in_argv_warning.rs:157`+`:238`, `cli_env_var_sentinel.rs:33`, `cli_argv_leakage.rs:175`+`:264`, `cli_json_envelopes.rs:46`, `cli_verify_bundle_watch_only.rs:81`.

*mk1 (`filter(... && !contains(' ') && !contains('-'))` → `filter(starts_with("mk1"))` + per-string strip):* `cli_bundle_watch_only.rs:16`+`:74`, `cli_verify_bundle_full.rs:21`, `cli_verify_bundle_forensics.rs:24`+`:157`, `cli_verify_bundle_seedqr_slot.rs:21`, `cli_verify_bundle_watch_only.rs:21`/`:73`/`:220`, `cli_secret_in_argv_warning.rs:162`+`:243`, `cli_env_var_sentinel.rs:38`, `cli_argv_leakage.rs:180`+`:269`, `cli_positional_hrp_autodetect.rs:299`+`:365`, `cli_hrp_case_insensitive.rs:490`, `cli_json_envelopes.rs:51`.

*md1 (same pattern, `md1`):* `cli_verify_bundle_full.rs:26`, `cli_verify_bundle_forensics.rs:29`+`:162`, `cli_verify_bundle_seedqr_slot.rs:26`, `cli_verify_bundle_watch_only.rs:25`/`:77`/`:225`, `cli_secret_in_argv_warning.rs:167`+`:248`, `cli_env_var_sentinel.rs:43`, `cli_argv_leakage.rs:185`+`:274`, `cli_positional_hrp_autodetect.rs:304`+`:370`, `cli_hrp_case_insensitive.rs:495`, `cli_json_envelopes.rs:56`.

*Output-goldens (Task 2c regen — must run the binary + overwrite):* `cli_bundle_full.rs:34`, `cli_env_var_sentinel.rs:302-303`/`:393-394`/`:428`, `cli_self_check.rs:34` (v0_2), `cli_argv_leakage.rs:248-251`.

**C2 — 5 more convert/bundle exact-pin / length tests break (add to Task 3):** `cli_convert_happy_paths.rs:326` (`assert_eq!(stdout, "ms1: {TREZOR_24_ZERO_MS1_24WORD}\n")`), `cli_convert_round_trips.rs:69` (`assert_eq!(ms1, TREZOR_24…)`), `cli_mnem_emit_preserve.rs:409` (`assert_eq!(ms1_val, ENGLISH_MS1_GOLDEN)`), `cli_convert_language_advisory.rs:150` (`assert_eq!(ms1_val.len(), 51)` → 61 grouped), `cli_bundle_language_advisory.rs:65` (`assert_eq!(ms1_val.len(), MNEM_MS1_LEN_12WORD)`). Fix: `--group-size 0` on the invocation, OR `strip_display_separators(ms1_val)` before compare/len.

## IMPORTANT (new)
- **I1 — `cli_argv_leakage.rs` is triple-role:** `:172-216` fixture-parse (Task 2d), `:228-251` output-golden `assert_eq!` (Task 2c regen), `:258-` fixture-parse (Task 2d). Add to BOTH 2c and 2d.
- **I2 — `cli_env_var_sentinel.rs`** has output-golden `assert_eq!` at `:302-303`/`:393-394`/`:428` (Task 2c) AND its `bip84_mainnet_fixture()` helper `:33-43` has the broken filters (Task 2d). Add to BOTH.

## MINOR
- **m1** `cli_bundle_language_advisory.rs:60` uses a variant ms1 finder (`!starts_with("ms1 ") && !contains("(entropy")`) that SURVIVES print-once (find works) — only the `:65` length check fails. Fix the length assertion (strip first).
- **m2** `target/package/mnemonic-toolkit-0.38.0/` is a stale `cargo package` snapshot — not live test files; ignore.
- **m3** spot-checks confirmed: `ms_shares.rs:283`/`:296-300`, `convert.rs:1119`/`:1560`, `verify_bundle.rs:1407`, all correct.
- **m4** `cli_positional_hrp_autodetect.rs` has TWO test fns each with mk1+md1 filters (`:299/304` + `:365/370`).

## Required corrections (summary)
1. Task 2d: replace named lists with the exhaustive ms1/mk1/md1 lists above.
2. Task 2c: add `cli_argv_leakage.rs` + `cli_env_var_sentinel.rs` (multiple sites) to the output-golden regen list.
3. Task 3: add the 5 convert/bundle pins in C2.
