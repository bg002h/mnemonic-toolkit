# Plan-R0 (P4 toolkit) round 1 — architect review (verbatim)

> Persisted verbatim per CLAUDE.md BEFORE the fold. Dispatched via Agent
> (feature-dev:code-architect, Opus 4.8). **Verdict: NOT GREEN — 4 Critical /
> 4 Important.** Plan SHA at review: toolkit `457bfa7` (branch HEAD).
> High-quality review verified against live source with specific line numbers.

---

## Verdict: NOT GREEN — 4 Critical / 4 Important

### CRITICAL

**C1 — `cli_ms_shares.rs` round-trip tests break + missing from affected list.** `parse_shares` (`:54-56`) collects `stdout.lines()`; after `run_split` emits `render_grouped(share,5,' ')`, the grouped share lines are passed DIRECTLY as `--share <grouped>` to combine (`:69`,`:102`,`:115`). Combine must strip before `ms_codec::combine_shares` — so split-emit + combine-intake CANNOT be separate commits (Task 3 then Task 4 strands RED). `cli_ms_shares.rs` is NOT in any affected-test list. **Fix:** merge the ms-shares split-emit + combine-intake into ONE atomic commit (or intake-before-emit ordering for ms-shares, like P1/P3); add `cli_ms_shares.rs` to the affected list.

**C2 — `cli_convert_happy_paths.rs:154` exact-pin breaks.** `assert_eq!(stdout, format!("ms1: {TREZOR_12_ZERO_MS1}\n"))` (unbroken). `convert --to ms1` emits via `convert.rs:1119` `writeln!(stdout,"{}: {}",node,value)`; grouping `value` → `ms1: ms10e ntrsq…` → breaks. NOT in Task 3's fix list. **Fix:** add `cli_convert_happy_paths.rs:154` to Task 3 (`--group-size 0` on that invocation, or assert grouped). Also check `mk1_to_xpub_decode:197`.

**C3 — ms1 extraction filter is a DIFFERENT pattern than mk1; Task 2d only addresses mk1.** ms1 parsers use `find(|l| l.starts_with("ms1") && !l.contains(' '))` (NO hyphen check) at `cli_verify_bundle_full.rs:16`, `cli_verify_bundle_forensics.rs:19`, `cli_verify_bundle_seedqr_slot.rs:15`, `cli_secret_in_argv_warning.rs:157`. Print-once removes the unbroken line → `find`→None→`.expect()` PANICS. Task 2d's fix recipe is written only as `starts_with("mk1")`. **Fix:** Task 2d must give the ms1 AND mk1 AND md1 filter fixes EXPLICITLY (each: drop the `!contains(' ')`/`!contains('-')` guard, take the grouped line, `strip_display_separators`).

**C4 — verify_bundle strip must bind at top-of-function (collection level).** `verify_bundle.rs:1407` does a raw `supplied_ms1 == expected_ms1` string-equality (forensic match). If strip is applied only immediately before `ms_codec::decode` (`:1399`) but `args.ms1` stays grouped for the `:1407` compare, a correct grouped card ALWAYS mismatches. **Fix:** Task 4 must strip the whole `args.ms1`/`args.mk1`/`args.md1` collections at the TOP of the run fn so the stripped (unbroken) values propagate to decode AND the equality check AND the forensic expected/actual fields.

### IMPORTANT

**I1 — ms-shares ordering (compounds C1):** split-emit + combine-intake must be one atomic commit (or intake-first for ms-shares, matching P1/P3). The plan's 3→4 split strands `cli_ms_shares.rs` round-trips RED.

**I2 — convert `--from mk1=` Mk1 arm uses `split_whitespace()` (`convert.rs:1560`).** Strip must precede that tokenize, not just the `mk_codec::decode`. A default space/5 single-chunk mk1 via `--from mk1=<grouped>` would mis-tokenize. **Fix:** Task 4 strips `value` before `value.split_whitespace()` for the Mk1 arm.

**I3 — ms-shares split `--json` isolation.** `render_grouped` must be applied ONLY in the text branch (`ms_shares.rs:296-300` loop), NOT before the `if args.json` branch (`:283`) — else `--json` shares get grouped (violates §6). Plan is silent (P2 was explicit for ms-cli). **Fix:** state the json/text isolation + add a `split --json` unbroken-invariant test.

**I4 — Task 7 missing steps.** (a) add `grep render_codex32_grouped crates/` to confirm no remaining display call sites after the collapse (the kept md-codec wrapper means non-display callers still work, but verify). (b) `.examples-build/gen.sh` has 6×`v0.55.3` — a lockstep version-pin site not in the release ritual.

### MINOR
- **m1** `.examples-build/gen.sh` 6×version pins (see I4b).
- **m2** verify-bundle `--auto-repair` short-circuit (`:1429-1436`) → repair output unbroken (§6); correctly covered, note it.
- **m3** toolkit-side `cli_gui_schema.rs` is assertion-based (subcommand list/version, not exhaustive flag sets) → new flags don't break it. Confirmed.
- **m4** `cli_gui_schema_v3/v4/v5_extensions.rs` are additive-field tests → safe. Confirmed.
- **m5** fuzz workspace `Cargo.lock` self-heals via `cd fuzz && cargo build`.
- **m6** README version-markers ×2 = README:13 + install.sh:32 confirmed.

### Notes verified
- Bundle print-twice→once at `emit_unified` (unbroken `:976`/`:985`/`:997`/`:1016` + grouped `:978`/`:989`/`:1001`/`:1020`) confirmed; md1 currently HYPHEN-grouped (default space/5 is a visible change). `--json` branch separate. `BundleArgs` is the flag home.
- 6 intake sites confirmed; `--bundle-json` correctly excluded.
- md-codec pin `Cargo.toml:36 = "0.35"` does NOT auto-pick 0.36.0; bump to `"0.36"` for lockstep is additive/safe (the kept `render_codex32_grouped` wrapper means any non-display caller still builds).
- Task 2 (bundle+goldens atomic) coupling confirmed correct.
