# Plan-R0 (P3 mk) round 1 — architect review (verbatim)

> Persisted verbatim per CLAUDE.md BEFORE the fold. Dispatched via Agent
> (feature-dev:code-architect). **Verdict: NOT GREEN — 0 Critical / 1 Important.**
> Plan SHA at review: toolkit `3bf0ed1`; mk repo `21786dc`. The agent's output
> churned through several self-corrected false starts (labeled REVISED/REAL/FINAL);
> the SUBSTANTIVE converged finding is a single Important. Controller-verified
> against live `cli_slip132.rs` before folding (see summary).

## Controller summary of the substantive finding
**I1 (Important) — `cli_slip132.rs::run_encode_decode` (`:58-84`) is a missing breaking test.** It invokes `mk encode` via the CLI (`:62`), captures `stdout.lines()` (`:79`), and passes them DIRECTLY to `mk_codec::decode` (`:82`) — bypassing the CLI intake strip. After Task 3 (grouped emit), `mk_codec::decode` receives space-grouped strings → errors. Same defect pattern as `round_trip.rs::from_md1_derivation`. Used by `encode_accepts_zpub_with_matching_path` (calls it twice). **Fix:** add `"--group-size", "0"` to `run_encode_decode`'s `mk encode` args; add `cli_slip132.rs` to Task 3's file list + the suite-sweep list.

**Controller verification of the REST of `cli_slip132.rs` (the agent did not fully classify all 7 `mk encode` calls):** `make_card()` (`:30`) returns grouped chunks to `mk verify` (CLI, `:245-248` / `:274-277`) → `mk verify`'s `read_mk1_strings` strips them under intake-first ordering → SAFE (no fix). The other 5 `mk encode` CLI calls (`:114`, `:149`, `:180`, `:210`, `:298`) assert ONLY on exit-code + stderr (never decode stdout) → SAFE. So `run_encode_decode` is the ONLY breaking site in the file.

**Minor (m1):** spec §5 says `--separator bogus` → exit 2; mk-cli's clap-error catch-all (`main.rs:72`) routes ALL clap parse errors to exit 64. The plan/test correctly use 64. Spec text is generic/stale; no plan action.

All cited file/line numbers VERIFIED correct (read_mk1_strings :84/:93/:101; the 6 caller sites; encode emit :93-95; main.rs :68-72; no mlock.rs; fmt-1.95.0 job; CARGO_MANIFEST_DIR path). `gui_schema.rs` confirmed assertion-based (subset `find()` checks; new flags → `"text"` kind; safe). mk-codec confirmed to tolerate NO separators. INTAKE-first ordering confirmed sound. Architecture deviation (mk-cli-local `format.rs`, bin-crate conformance) ratified.

---

## Verbatim agent output

## Plan-R0 Architect Review — P3 mk — Round 1

**Target SHA verified against:** `mnemonic-key` `main` `21786dc` (live grep)

### Verdict
**NOT GREEN — 0 Critical / 1 Important** (missing breaking test: `cli_slip132.rs::run_encode_decode`).

### Important
The plan lists one breaking test (`round_trip.rs::from_md1_derivation`). The full `tests/` sweep reveals ONE additional breaking site not on the list: **`cli_slip132.rs::run_encode_decode`** (used by `encode_accepts_zpub_with_matching_path`): invokes `mk encode` via CLI, captures stdout lines, passes directly to `mk_codec::decode`. After Task 3 (grouped emit), decode receives grouped strings → errors. Same defect pattern as `from_md1_derivation`; not fixed in the plan. **Fix:** Add `cli_slip132.rs` to Task 3's file list; add `"--group-size", "0"` to the `mk encode` invocation inside `run_encode_decode()` (`:60-75`). Also amend the suite-sweep list (plan line 35) to explicitly include `cli_slip132.rs`.

All other test files verified clean:
- `cli_address.rs` — builds cards via `mk_codec::encode` (not CLI). Safe.
- `cli_derive.rs` — `child_xpub_roundtrips_through_encode` calls `mk encode` but only asserts `contains("mk1")` on grouped output → still passes. Safe.
- `cli_repair.rs` — chunk-equality asserts use `mk_codec`-derived strings vs unbroken repair output. Safe.
- `cli_output_class.rs` — checks advisory lines on stderr / `status.success()`. Safe.
- `gui_schema.rs` — auto-generated from clap; subset assertions; new flags cause no failures. Safe.
- `version_help_exit_codes.rs` — no mk1 strings. Safe.
- `round_trip.rs` `encode_decode_round_trip` / `verify_content_mismatch_exits_4` / `vectors_subcommand_no_path_dep` — use `mk_codec::encode` / no grouping. Safe.

### Verified citations
read_mk1_strings `:84`/`.trim()` `:93`/`out.push(a.clone())` `:101` ✓; callers verify.rs:48, repair.rs:59, decode.rs:25, derive.rs:42, inspect.rs:29, address.rs:74 ✓; encode emit loop `:93-95` ✓; main.rs clap→64 `:68-72` ✓; NO `mlock.rs` in mk-cli ✓; `fmt (pinned 1.95.0)` job present with plain checkout ✓; `CARGO_MANIFEST_DIR/../../design` → repo-root `design/` ✓. INTAKE-first ordering keeps every intermediate commit green ✓. Type-change in Task 4 Step 3 (`s` becomes `String`, `out.push(s)`) accurate ✓.

### Minor
- spec §5 says `--separator bogus` → exit 2; mk-cli routes clap errors → 64; the plan/test correctly use 64 (spec text generic). 
- `fmt` job is the right home for the checksum step (plain checkout). 
- adding fields to `EncodeArgs` does not break `is_json_mode` (still reads `a.json`). 

### Required fixes before execution
1. (Important) Add `cli_slip132.rs` to Task 3's file list + add `--group-size 0` to `run_encode_decode()`'s `mk encode` invocation.
2. Amend the plan's suite-sweep list to include `cli_slip132.rs`.

**Verdict: NOT GREEN — 0 Critical / 1 Important.**
