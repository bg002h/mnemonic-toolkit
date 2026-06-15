# R0 round 2 — architect review (verbatim) — SPEC_mstring_display_grouping.md

> Persisted verbatim per CLAUDE.md BEFORE the fold. Dispatched via Agent
> (feature-dev:code-architect, Opus 4.8). **Verdict: NOT GREEN — 1 Critical /
> 3 Important / 3 Minor.** Spec SHA at review: toolkit `f09e7b2`.

---

## R0 Round 2 — Architect Review: SPEC_mstring_display_grouping.md

### Fold Verification Results

**C1/C2 — `ms split` print-once with labels→stderr.** Structurally correct. `split.rs:147-163` shows the current two-part emit; §6 collapses it (shares one-per-line on stdout, labels→stderr). §10 heuristic removal is safe once collapsed. BUT: `ms combine -` (stdin) in `combine.rs:50-52` does `std::mem::take(&mut args.shares)` from clap positionals — there is NO `-`→stdin resolution in `combine.rs::run`; `read_input` (which handles `-`) is NOT called. So `ms split | ms combine -` does not work today and the SPEC's round-trip claim assumes stdin handling that doesn't exist. (Promoted to C1 below.)

**C3 — `ms combine` positional intake.** Site `combine.rs:38-39` confirmed. But stripping positionals only fixes `ms combine "ms10... q q q"`; it does NOT enable `ms split | ms combine -` unless `-`→stdin-multiline is also added.

### Findings

**Critical**

**C1 — `ms combine -` stdin pipe is not handled in `ms-cli/src/cmd/combine.rs`, and the SPEC does not require adding it.** `combine.rs:36-52`: `shares: Vec<String>` is a raw clap positional, no `-`-to-stdin expansion; `run()` does `std::mem::take(&mut args.shares)` and passes raw positionals to `ms_codec::combine_shares`. The SPEC §6/§10/§15-1 all assert `ms split | ms combine -` round-trips after the heuristic removal. It does not today, and after Phase 1 (heuristic removed, split→one-per-line stdout) the pipe fails entirely (`-` passed literally → BCH error) — worse than before. Fix: §9.2 + Phase 1 must explicitly add `-`→stdin multiline share intake to `ms combine` (parallel to mk-cli `read_mk1_strings`, `mod.rs:84-110`). Without this the C1/C2 fold is textually present but semantically wrong.

**Important**

**I1 — technical-manual dead-entry gap.** `docs/technical-manual/src/50-rust-api/54-mnemonic-toolkit-api.md:50-51` documents `chunk_md1`/`chunk_5char` as pub fns. §8 DELETES them; §11 only says ADD a `render_grouped` entry, not REMOVE the dead `chunk_*` entries. The manual lint won't catch a Rust-API-doc deletion. Fix: §11 must instruct removing the `chunk_5char`/`chunk_mk1`/`chunk_md1` rows from `54-mnemonic-toolkit-api.md` in Phase 3.

**I2 — Phase-1 test-rewrite assignment.** `mnemonic-secret/crates/ms-cli/tests/encode_canonical_12_word.rs:17` asserts `contains("\n\n")` (print-twice). Phase 1 removes print-twice → test goes RED. §13 mentions the rewrite generically; §12 Phase 1 says only "update unit tests." Fix: name `encode_canonical_12_word.rs` (+ `encode_canonical_24_word.rs`) explicitly in Phase 1.

**I3 — toolkit `ms_shares` annotations.** `ms_shares.rs:296-300`: `run_split` ALREADY emits one share per line (no labels on stdout; advisory already on stderr `:305-310`). So §6 "labels→stderr" is already satisfied for the toolkit surface — the Phase 3 change is purely additive (wrap with `render_grouped`), not a restructure; SPEC language implies a `ms-cli split`-style restructure and could mislead. Also §9.1's `run_combine --to ms1` annotation "per §6 split rule" is DANGLING — §6 defines no combine rule; it should read "apply `render_grouped` to ms1 output." Fix both annotations.

**Minor**

**m1 — differential harness has no disk artifacts.** `cli_cross_tool_differential.rs:34-39` is `#[ignore]` and compares two LIVE binaries; verdicts are compile-time constants, no golden files. It decodes format-independent IDs (`wallet_policy_id`, template id via `md inspect --json`), so a separator change won't false-positive. §12/§13 "regenerate differential-harness artifacts" implies files that don't exist. Fix: clarify the differential harness needs no regen; only fuzz corpus SEEDS should be re-examined for embedded formatted strings.

**m2 — TSV strip-row `group_size`.** §8 defines `separator=none` and `<empty>` sentinels but is silent on what `group_size` holds for `op=strip` rows (strip ignores it). Fix: state `group_size=0` for strip rows.

**m3 — (same surface as I3)** §9.1's combine annotation mislabel + run_split redundancy; covered by I3 fix.

### Internal Consistency Checks
§3.2 strip vs §4 safety: consistent (separator set ⊂ stripped set, none in alphabet). §6 repair-unbroken vs §9.1 (excluded) vs §12 Phase 3 ("NOT repair"): consistent. `--json`/`VerifyCheck` unbroken: `format.rs:170-171` are plain `Option<String>`, no formatting — consistent. CLAUDE.md: no new ToolkitError variant (exit-2 clap-layer) → alphabetical rule untriggered; citation-at-write-time stated; no `git add -A`. Wrapper approach (I6 fold) means no atomic cross-phase break — Phase 3 `chunk_md1` deletion compiles cleanly at any point. All satisfied.

### Verdict
NOT GREEN — 1 Critical / 3 Important / 3 Minor. C1 blocks: the central `ms split | ms combine -` round-trip claim requires `-`→stdin in `ms combine`, which does not exist and is not instructed. I1 (manual dead entries), I2 (Phase-1 test naming), I3 (toolkit annotation fixes) are targeted SPEC edits. Folds otherwise landed correctly.
