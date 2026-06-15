# mstring display grouping — P4 (mnemonic-toolkit) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:executing-plans. Steps use checkbox (`- [ ]`).

**Goal:** Bring the integration crate `mnemonic-toolkit` onto the standardized mstring display-grouping contract: collapse `chunk_5char`/`chunk_mk1`/`chunk_md1` into the P0 `display_grouping::render_grouped`; make `bundle` (+ `convert`/`ms-shares`) **print-once** with `--group-size`/`--separator`; add separator-stripping intake; regenerate the output-goldens + fix the input-fixture parsers; update both manuals; MINOR release.

**Source SHA (recon at write time):** toolkit branch `feature/mstring-display-grouping` (P0+continuity present; bin version 0.55.3). Already on the branch: `crates/mnemonic-toolkit/src/display_grouping.rs` (`render_grouped`/`strip_display_separators`/`is_display_separator` + `tests/display_grouping_conformance.rs`, P0). Vectors already at `design/display-grouping-vectors.tsv` (P0).

**Branch:** continue on the existing `feature/mstring-display-grouping` (toolkit). NO new branch.

**Spec:** `design/SPEC_mstring_display_grouping.md` (R0 GREEN). Implements §12 Phase 3 + the toolkit rows of §8/§9/§11.

---

## KEY FINDINGS / DECISIONS (recon `feature/mstring-display-grouping` HEAD)

1. **`display_grouping.rs` (P0) already has the 3 pure fns** but NOT `parse_separator` — ADD `pub fn parse_separator(s: &str) -> Result<char, String>` there (mirror P1/P2/P3).

2. **`chunk_*` collapse:** `format.rs:10-39` defines `chunk_5char`/`chunk_mk1`/`chunk_md1` (+ unit tests `:405-424`). DELETE all three + their unit tests. md1's `chunk_md1` currently routes through `md_codec::encode::render_codex32_grouped(s,5)` (hyphen) — after collapse the toolkit uses its OWN `display_grouping::render_grouped` (byte-identity guaranteed by the shared vectors; the toolkit no longer calls md_codec's render fn for display, spec §8).

3. **`bundle` is PRINT-TWICE today** (`cmd/bundle.rs::emit_unified`): unbroken (`:976` ms1, `:985`/`:997` mk1, `:1016` md1) THEN grouped (`:978` `chunk_5char`, `:989`/`:1001` `chunk_mk1`, `:1020` `chunk_md1`). Spec §6 = print-once: REMOVE the unbroken emits (+ their trailing blank lines), keep ONE grouped emit per card via `render_grouped(s, group_size, separator)` (default space/5, single line — md1 flips hyphen→space; ms1/mk1 lose wrap@10). `--json` branch untouched.

4. **`BundleArgs` gets `--group-size`/`--separator`.** `convert` (`--to ms1`/`--to mk1` arms) + `ms-shares` (`run_split` shares + `run_combine --to ms1`) ALSO get them per spec §9.1. `verify-bundle` + `repair` do NOT (repair emits unbroken; verify-bundle only `--json`).

5. **GOLDEN DUAL-ROLE (the crux):** `tests/vectors/v0_1/*.txt` (16 files: bip44/49/84/86 × main/test/sig/regtest) are BOTH:
   - **output-goldens** — `cli_bundle_full.rs:34` `assert_eq!(stdout, expected)` (loops templates×networks) + `cli_env_var_sentinel.rs:428` `assert_eq!(stdout, expected)` (bip84-mainnet). After print-once they MUST be regenerated.
   - **input-fixtures** — ~10 tests parse card strings out of them by filtering `starts_with("mk1") && !contains(' ') && !contains('-')` (the UNBROKEN line). Print-once REMOVES unbroken lines → those filters find nothing → break. Each such parser MUST change to take a grouped line + `strip_display_separators` it (single-line print-once means one chunk per line; the wrap@10 continuation lines are gone, which SIMPLIFIES the parse). Affected readers: `cli_argv_leakage.rs`, `cli_secret_in_argv_warning.rs`, `cli_verify_bundle_seedqr_slot.rs`, `cli_hrp_case_insensitive.rs`, `cli_json_envelopes.rs`, `cli_positional_hrp_autodetect.rs`, `cli_verify_bundle_forensics.rs`, `cli_verify_bundle_full.rs`, `cli_verify_bundle_watch_only.rs`, `cli_bundle_watch_only.rs` (each re-greped in Task 4).
   - **v0_2 output-golden:** `tests/vectors/v0_2/bip84-mainnet-0-false-true.txt` — `cli_self_check.rs:34` `assert_eq!`. Regenerate.

6. **6 INTAKE sites (spec §9.2):** `slot_ms1.rs` (ms1 slot decode), `cmd/verify_bundle.rs` (`--ms1`/`--mk1`/`--md1` flag values; NOT `--bundle-json` which is canonical), `cmd/repair.rs` (resolve groups), `cmd/convert.rs` (`--from ms1=`/`--from mk1=`), `cmd/ms_shares.rs` (combine shares). Strip via `display_grouping::strip_display_separators` before each decode. (Re-grep exact lines in Task 5 — the spec cites decayed numbers.)

7. **`VerifyCheck.expected`/`.actual`** (`format.rs:~164-182`) stay UNBROKEN (forensic; never grouped). `--json` everywhere stays unbroken.

8. **Release ritual (heaviest in the constellation — from v0.55.x lessons):** rustfmt **1.95.0** gate + **mlock.rs g6-EXEMPT** (NEVER `cargo fmt` mlock.rs; run `cargo +1.95.0 fmt --all` THEN `git checkout crates/mnemonic-toolkit/src/mlock.rs` to revert any mlock reformat; the fmt gate tolerates ONLY mlock.rs diffs). README version-markers (×2 per v0.55.2), `scripts/install.sh` self-pin, `.github/workflows/manual.yml` + `quickstart.yml` **mk-cli sibling-pin → bump to `mk-cli-v0.9.0`** (P3 shipped it). Fuzz crate = SEPARATE workspace (`fuzz/`) — re-run `cargo build` there after the bump. Rust workflow does NOT run on tags → run FULL suite + fuzz build BEFORE tagging. Toolkit ships **git-tag (master + tag), NOT crates.io** historically.

9. **md-codec pin:** `Cargo.toml:36` is `md-codec = "0.35"`. The toolkit no longer needs md-codec's render fn (collapsed to local). Bumping to `"0.36"` keeps constellation lockstep (additive, low-risk) but is OPTIONAL for the feature — **plan-R0 to decide: bump to "0.36" for lockstep, or leave "0.35" since the display dependency is removed.** (Recommendation: bump to "0.36" + verify build, to keep the constellation on the published md-codec.)

---

## R0-r1 corrections (MUST APPLY — override the task bodies where they conflict)

Plan-R0 round 1 = NOT GREEN (4C/4I; review `design/agent-reports/mstring-display-grouping-plan-r0-p4-round1-review.md`). Verified against live source:

- **(C1/I1) ms-shares split-emit + combine-intake are COUPLED → ONE atomic task.** `cli_ms_shares.rs::parse_shares` (`:54-56`) feeds `run_split` stdout lines directly as `--share <grouped>` to combine (`:69`,`:102`,`:115`). Move the `ms-shares run_split` grouping AND the `ms-shares combine` intake-strip into a SINGLE task/commit (Task 3 below now owns BOTH ms-shares emit + ms-shares-combine intake). Add `cli_ms_shares.rs` to the affected-test list.
- **(C2) `cli_convert_happy_paths.rs:154`** `assert_eq!(stdout, format!("ms1: {TREZOR_12_ZERO_MS1}\n"))` breaks when `convert --to ms1` groups (emit is `convert.rs:1119` `writeln!("{}: {}",node,value)`). Add `--group-size 0` to that invocation OR assert the grouped value. Add the file to Task 3's fix list. Also re-check `mk1_to_xpub_decode:197`.
- **(C3) ms1/mk1/md1 input-fixture filters are DIFFERENT patterns — Task 2d MUST give each explicitly.** ms1 filter = `find(|l| l.starts_with("ms1") && !l.contains(' '))` (NO hyphen guard) at `cli_verify_bundle_full.rs:16`, `cli_verify_bundle_forensics.rs:19`, `cli_verify_bundle_seedqr_slot.rs:15`, `cli_secret_in_argv_warning.rs:157`. mk1 filter = `…starts_with("mk1") && !contains(' ') && !contains('-')`. After print-once the unbroken line is GONE → `find`→None→`.expect()` PANIC. **Fix for EACH (ms1, mk1, md1):** drop the `!contains(' ')`/`!contains('-')` guard, take the (now single-line) grouped line, and `display_grouping::strip_display_separators` it before decode.
- **(C4) `verify_bundle.rs` strip at COLLECTION TOP-of-function, not just pre-decode.** `:1407` does raw `supplied_ms1 == expected_ms1`. Strip the whole `args.ms1`/`args.mk1`/`args.md1` vecs at the top so the stripped (unbroken) values reach decode AND the `:1407` equality AND the forensic `expected`/`actual` fields.
- **(I2) `convert --from mk1=` Mk1 arm:** strip `value` BEFORE `value.split_whitespace()` (`convert.rs:1560`), not just before `mk_codec::decode`.
- **(I3) ms-shares `run_split` `--json` isolation:** apply `render_grouped` ONLY in the text branch (`ms_shares.rs:296-300` loop), NOT before the `if args.json` (`:283`). Add a `split --json` unbroken-invariant test.
- **(I4) Task 7 additions:** (a) `grep -rn render_codex32_grouped crates/` to confirm no remaining DISPLAY call site after the collapse (the kept md-codec wrapper keeps non-display callers building). (b) `.examples-build/gen.sh` has 6×`0.55.3` — add it to the lockstep version-pin sites.

## Execution order (per-commit green)
1 (parse_separator) → 2 (collapse `chunk_*` → `render_grouped`, bundle print-once + flags, regen 16 v0_1 + 1 v0_2 output-goldens, fix the ms1/mk1/md1 input-fixture parsers — ALL ATOMIC, coupled) → 3 (convert `--to ms1/mk1` flags + the **ms-shares run_split emit AND ms-shares-combine intake together** [C1/I1 coupling] + fix `cli_convert_happy_paths.rs:154` + `cli_ms_shares.rs`) → 4 (the OTHER intake strip sites: `slot_ms1`, `verify_bundle --ms1/--mk1/--md1` [collection-top strip, C4], `repair`, `convert --from ms1=/mk1=` [strip before split_whitespace, I2]) → 5 (full suite + fmt-1.95.0 + clippy + fuzz build) → 6 (manuals + FOLLOWUP companion) → 7 (version bump + release ritual + tag).

> **Why bundle+goldens are ONE task (Task 2):** the print-once emit change and the 16+1 golden regen + ~10 input-parser fixes are mutually dependent — splitting them strands a RED commit. Do them together; the commit is green only once all are consistent.

---

## Task 1: `display_grouping::parse_separator`
- [ ] Add `pub fn parse_separator(s) -> Result<char, String>` to `display_grouping.rs` (keyword|literal → char; mirror P1) + a unit test. `cargo test -p mnemonic-toolkit --bin mnemonic display_grouping::` (or `--lib` — confirm the module's test home). Commit.

## Task 2: Collapse `chunk_*` + `bundle` print-once + flags + regen/fix goldens (ATOMIC)
- [ ] **2a. Flags:** add `--group-size`/`--separator` to `BundleArgs` (`value_parser = crate::display_grouping::parse_separator`); thread to `emit_unified`.
- [ ] **2b. Collapse + print-once:** in `emit_unified`, REMOVE the unbroken emits (`:976`/`:985`/`:997`/`:1016`) + their now-redundant blank lines; replace each `chunk_5char`/`chunk_mk1`/`chunk_md1` call with `crate::display_grouping::render_grouped(s, group_size, separator)`. DELETE `chunk_5char`/`chunk_mk1`/`chunk_md1` from `format.rs` + their unit tests (`:405-424`); drop the `use ...chunk_*` import in `bundle.rs:8`.
- [ ] **2c. Regenerate output-goldens:** re-run the EXACT `mnemonic bundle` invocations that `cli_bundle_full.rs` + `cli_env_var_sentinel.rs` + `cli_self_check.rs` use, capturing stdout → overwrite the 16 `v0_1/*.txt` + the 1 `v0_2/*.txt`. (Use the binary built from this branch. Verify each regenerated golden is print-once single-line grouped.)
- [ ] **2d. Fix input-fixture parsers (C3 — ms1/mk1/md1 are DIFFERENT filters, fix EACH explicitly):** for each `vectors/v0_1` reader (re-grep), the unbroken line is GONE after print-once → the `!contains(' ')`-style filters return None and `.expect()` PANICS. Fix per card type:
  - **ms1:** `find(|l| l.starts_with("ms1") && !l.contains(' '))` → `find(|l| l.starts_with("ms1"))` then `strip_display_separators(line)`. Files: `cli_verify_bundle_full.rs:16`, `cli_verify_bundle_forensics.rs:19`, `cli_verify_bundle_seedqr_slot.rs:15`, `cli_secret_in_argv_warning.rs:157` (+ any other re-greped).
  - **mk1:** `…starts_with("mk1") && !contains(' ') && !contains('-')` → `starts_with("mk1")` + `strip_display_separators` per line (one chunk per line now — wrap@10 gone). Files: `cli_bundle_watch_only.rs:14-17` (+ others).
  - **md1:** same shape (`starts_with("md1")` + strip).
  Re-grep ALL `vectors/v0_1` consumers and apply the matching fix; confirm the list is complete (the architect flagged the ms1 variant the round-1 plan missed).
- [ ] **2e.** `cargo test -p mnemonic-toolkit` → GREEN (bundle/self-check/env-sentinel/all parsers). Commit (one atomic commit).

## Task 3: `convert` + `ms-shares` flags + ms-shares-combine intake (ATOMIC — C1/I1/C2/I2/I3)
- [ ] **convert emit:** add `--group-size`/`--separator` to convert's `--to ms1`/`--to mk1` arms; group the `value` in `convert.rs:1119` emit (text only). Fix `cli_convert_happy_paths.rs:154` (`--group-size 0` or assert grouped); re-check `:197`.
- [ ] **ms-shares (atomic emit+intake):** add `--group-size`/`--separator` to `MsSharesSplitArgs`; in `run_split` apply `render_grouped` ONLY in the text branch (`ms_shares.rs:296-300`), NOT before the `if args.json` (`:283`) [I3]. In the SAME commit, add `strip_display_separators` to the ms-shares **combine** share intake (so `ms-shares split | combine` round-trips — `cli_ms_shares.rs:59/93` feed split's grouped lines to combine) [C1/I1]. Also group `run_combine --to ms1` output (text branch).
- [ ] **Tests:** default-grouped + `--group-size 0` CLI tests for convert/ms-shares; `split --json` unbroken-invariant test; confirm `cli_ms_shares.rs` round-trips GREEN. Commit.

## Task 4: INTAKE strip (the OTHER 5 sites — ms-shares-combine done in Task 3)
- [ ] Apply `display_grouping::strip_display_separators` at: `slot_ms1.rs` (before ms1 decode); **`verify_bundle.rs` — strip the WHOLE `args.ms1`/`args.mk1`/`args.md1` collections at the TOP of the run fn [C4], so the stripped values reach decode + the `:1407` `supplied_ms1 == expected_ms1` equality + the forensic expected/actual fields** (NOT `--bundle-json`); `repair.rs` resolve-groups; `convert.rs` `--from ms1=`/`--from mk1=` — for the Mk1 arm strip `value` BEFORE `value.split_whitespace()` (`:1560`) [I2]. Add comma-grouped intake tests (comma = net-new for ms1/mk1 since ms/mk-codec strip nothing; md1 already tolerates ws+hyphen via md-codec D11 → use comma). Commit.

## Task 5: Full suite + fmt(1.95.0, mlock-exempt) + clippy + fuzz
- [ ] `cargo test --workspace` GREEN. `cargo +1.95.0 fmt --all` THEN `git checkout crates/mnemonic-toolkit/src/mlock.rs`; `cargo +1.95.0 fmt --all --check` tolerating ONLY mlock.rs. `cargo clippy --workspace --all-targets -- -D warnings`. `cd fuzz && cargo build` (separate workspace). Commit fixups.

## Task 6: Manuals + FOLLOWUP companion
- [ ] `docs/manual/src/40-cli-reference/`: document `--group-size`/`--separator` once in a common output-grouping section + per-CLI cross-refs (all 4 CLIs); run `make -C docs/manual lint ...` (bidirectional flag coverage). `docs/technical-manual`: REMOVE the `chunk_5char`/`chunk_md1` rows from `54-mnemonic-toolkit-api.md` + ADD `render_grouped`; `51-md-codec-api.md` `render_codex32_grouped` stays (wrapper). Run the technical-manual lint. File the toolkit-side `display-grouping-render-strip-v1` FOLLOWUP (siblings already point here). Commit.

## Task 7: Version bump + RELEASE ritual (autonomous — authorized)
- [ ] **Pre-bump sweep (I4a):** `grep -rn render_codex32_grouped crates/` — confirm NO remaining DISPLAY call site after the Task-2 collapse (the kept md-codec wrapper keeps any non-display caller building; if a display caller remains, route it through `display_grouping::render_grouped`).
- [ ] Bump `crates/mnemonic-toolkit/Cargo.toml:3` (MINOR, e.g. 0.55.3 → 0.56.0). md-codec pin "0.35"→"0.36" (plan-R0 ratified: additive/safe for lockstep). Update CHANGELOG. **Lockstep sites:** README version-marker (`README.md:13`) + `scripts/install.sh:32` self-pin; `manual.yml`+`quickstart.yml` mk-cli pin → `mk-cli-v0.9.0`; **`.examples-build/gen.sh` (6×`0.55.3`) [I4b]**. `cargo update`/build.
- [ ] FULL re-verify: `cargo test --workspace` + `cargo +1.95.0 fmt --all --check` (mlock-tolerant) + clippy + `cd fuzz && cargo build`.
- [ ] Commit. ff-merge `feature/mstring-display-grouping` → `master`, push. Tag `mnemonic-toolkit-v0.56.0` (git-tag; NOT crates.io). Push tag. Verify CI green on master.
- [ ] Examples.pdf: regenerate if the build tooling is present (`.github/workflows/manual.yml` / a make target); else file a FOLLOWUP (separators changed every card).

---

## Self-Review / Open items for plan-R0
(1) ratify the bundle print-once form removal (unbroken emits deleted) + the golden DUAL-role strategy (regen output-goldens + fix input-parsers) — is the enumeration of the ~10 input-parsers COMPLETE? (2) confirm convert/ms-shares emit sites + that they need flags per spec §9.1. (3) confirm the 6 intake sites + exact lines. (4) md-codec pin bump 0.35→0.36: necessary or optional? (5) the heaviest release ritual — is the lockstep list (README ×2, install.sh, manual.yml+quickstart.yml mk-cli→v0.9.0, fuzz workspace, mlock fmt-exempt) complete? (6) Examples.pdf regen feasibility.
