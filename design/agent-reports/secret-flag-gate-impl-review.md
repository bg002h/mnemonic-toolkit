# Implementation review — v0.53.1 secret-flag gate + classifications (2026-06-10)

Reviewer: Fable 5 implementation-review agent (post-impl, pre-commit). Plan @ design/PLAN_secret_flag_gate_non_circular.md (R0 GREEN r2). Verdict: GREEN (0 Critical / 0 Important / 3 Minor). Review verbatim below. Minor 1 folded post-review (distinct-name floor); Minors 2-3 noted, no action.

---

## Critical

None.

## Important

None.

## Minor

1. **Cell 2's non-vacuity floor counts instances but its prose says "toggles" / cites distinct names** — `crates/mnemonic-toolkit/tests/cli_gui_schema_v5_extensions.rs:418-428`: `toggles_seen` increments per *instance* (25 in the live schema, verified empirically), while the comment cites "8 distinct toggle names / 25 instances" and the failure message says "expected at least 8 `--X-stdin` boolean toggles". The floor of 8 against an instance count of 25 is much slacker than the census it documents (a surface collapsed to one toggle name across 9 subcommands would still pass). The plan's ">=8 toggle non-vacuity floor" is satisfied under either reading, so this is a wording/strength nit, not a conformance gap. Concrete fix (optional, next touch): collect distinct names into a `BTreeSet` and assert `>= 8` on its len, matching the message.
2. **The 3 new `Route` rows add one rustfmt-divergent site** (`tests/lint_argv_secret_flags.rs:111` region — long single-line struct literals). This deliberately matches the file's existing convention (all 33 pre-existing rows are single-line and the file already carried 7 rustfmt-divergent sites at HEAD), HEAD itself fails `cargo fmt --check` repo-wide (1447 sites), and `rust.yml` has **no fmt gate** (verified by grep). No action needed; noting for the record.
3. **CHANGELOG compression**: "all three were red on exactly the three names above" is true *collectively* (the union of red names is exactly `--phrase`/`--phrase-stdin`/`--ms1-stdin`, empirically confirmed) but per-cell the red sets differ (Cell 1: `--phrase` only; Cell 2: `--ms1-stdin` only; Cell 3: all three). The precise matrix lives in the plan §D2, so no edit required.

## Verdict

**GREEN (0 Critical / 0 Important).**

Evidence per review charter:

1. **Plan conformance — full.** D1: 3 names added to `matches!` at `src/secrets.rs:67-69` in the planned near-alphabetical slots, membership-rationale doc entries `:31-37`, unit-test rows `:91-93`; total = 14 names matching Cell 3's frozen literal. D2: Cell 1 (`:330-380`) never calls `flag_is_secret`, excludes by kind not name (verified the live schema's kind vocabulary is exactly `{number, text, dropdown, boolean, path}` — the exclusion strings are real), 12-needle net matches the plan list incl. the speculative-`priv` comment (R0-r1 M-3); `EXEMPT` is empty with the rationale-comment requirement (`:329-331`). Cell 2 (`:382-441`) prints `(subcommand, toggle, base)` triples, asserts base-flag existence (orphan branch `:411-416`), >=8 floor present. Cell 3 (`:443-481`) is a hard-coded 14-name const, not derived from the predicate. The `:284` cell renamed `secret_bit_plumbing_matches_predicate` with the honest "NOT a completeness gate" comment. D3: 3 Route rows with `pub phrase_stdin`/`fn phrase_stdin` needles — verified present at exactly the plan-cited lines (`path_of_xpub.rs:45,124`, `passphrase_of_xpub.rs:63,155`, `account_of_descriptor.rs:46,136`) and that bare `phrase_stdin` would indeed be satisfied by the files' `passphrase_stdin` anchors; boundary prose rewritten with the honest residual (`lint_argv_secret_flags.rs:30-41`). D4: CHANGELOG `[0.53.1]` present; version 0.53.1 in `Cargo.toml`, `Cargo.lock`, both README markers; FOLLOWUPS promotes/resolves `vacuous-secret-flag-gate` + folds the `[obs]` line, files `gui-secret-mirror-phrase-ms1-stdin`; GUI `FOLLOWUPS.md` adds `Companion:` lines to the two existing entries only (no third entry — git status confirms `M FOLLOWUPS.md` is the GUI repo's sole modification). Bonus accuracy check: all 9 GUI line citations (`--phrase` :2280/:2442/:2712, `--phrase-stdin` :2291/:2453/:2723, `--ms1-stdin` :2312/:2474/:2744) verified correct at GUI master `036776b`.
2. **TDD integrity — empirically verified.** I reverted only the 3 `matches!` arms in a scratch edit and ran the test file: **Cell 1 RED on `--phrase` ×3, Cell 2 RED on `--ms1-stdin` ×3 only, Cell 3 RED on the exact 3-name set diff, plumbing cell GREEN** — the corrected RED matrix from R0-r1 I-1, including the subtle `--phrase-stdin`-is-green-in-Cell-2 prediction. The plumbing cell staying green with the fix reverted also re-demonstrates the original tautology. `secrets.rs` was restored byte-identical (sha256 `727a2252…` matches before/after; backup deleted; worktree removed; final `git status` identical to the starting state).
3. **Test quality.** The only `flag_is_secret` *call* in the test file is the plumbing cell's `:298`; §7b mentions it solely in comments/messages. Cells run via `run_gui_schema()` → `Command::cargo_bin("mnemonic")` (`:33`) — proven live by the revert experiment (a source-only change flipped the results). Failure messages name the offending `(subcommand, flag, kind)` / triple and tell the maintainer what to edit.
4. **No collateral.** Diff touches exactly the 9 declared files; `Cargo.lock` delta is the version line only; no flag names changed (schema_mirror/manual claims hold — confirmed `flag_is_secret`'s sole non-test consumer remains `gui_schema.rs:1196`); old test name survives only in a `target/package` build artifact. CHANGELOG claims each verified against the diff (8/25 toggle census confirmed live; "no runtime behavior change" confirmed via the single-consumer grep).
5. **Final state green:** `cargo test --workspace` — 156 test-result lines, all `ok`, zero failures; `cargo clippy --workspace --all-targets -- -D warnings` clean (matches the CI invocation).
