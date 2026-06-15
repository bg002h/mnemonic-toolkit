# CONTINUITY — mstring display-grouping cycle (resume guide)

**Last updated:** 2026-06-15. **Toolkit branch:** `feature/mstring-display-grouping` @ `0281886` (clean). Siblings unchanged: descriptor-mnemonic `main@eb9f368`, mnemonic-secret `master@b616530`, mnemonic-key `main@21786dc`, mnemonic-gui `master@c5e3434`.

## What this cycle is
Standardize mstring (`ms1`/`mk1`/`md1`) display output across all four CLIs (`mnemonic`/`md`/`ms`/`mk`): a uniform `--group-size <u16>` (default 5, `0`=unbroken) + `--separator` (set `{space,hyphen,comma}`, keyword+literal, default `space`); **print-once**, default **space/5 single line**; `--json` + verify-bundle forensics stay **unbroken**; `repair` output stays unbroken (no flags). Intake strips ALL whitespace + `-` + `,` so grouped/unbroken/any-separator forms re-ingest. `ms split`→shares one-per-line (labels→stderr); **`ms combine` gains `-`→stdin**. Drift control = **copy-with-checksum conformance vectors** (canonical TSV in toolkit; sibling copies + `.sha256`; CI `sha256sum -c`) — NO new crate. SemVer **MINOR per crate**. "chunk"=wire-splitting (reserved); "group"=this feature.

## Authoritative docs (read these first on resume)
- Spec (R0 GREEN ×3): `design/SPEC_mstring_display_grouping.md`
- P0 plan (plan-R0 GREEN): `design/IMPLEMENTATION_PLAN_mstring_grouping_p0_foundation.md`
- P1 plan (plan-R0 r1 folded, **re-dispatch pending**): `design/IMPLEMENTATION_PLAN_mstring_grouping_p1_md.md` — note its top **"R0-r1 corrections (MUST APPLY)"** block.
- Reviews (verbatim): `design/agent-reports/mstring-display-grouping-{r0-round1,2,3, plan-r0-p0-round1,2, plan-r0-p1-round1}-review.md`
- Memory: `project_mstring_display_grouping_cycle.md`

## Done
- **P0 foundation SHIPPED** on the branch (reversible; no release): `design/display-grouping-vectors.tsv` (22 vectors); `crates/mnemonic-toolkit/src/display_grouping.rs` (`render_grouped`/`strip_display_separators`/`is_display_separator` + 10 unit tests; `pub mod display_grouping;` added to `lib.rs` UNCONDITIONALLY — NOT format.rs, which is `#[cfg(fuzzing)]`-gated); `crates/mnemonic-toolkit/tests/display_grouping_conformance.rs`. Full toolkit suite green; fmt-clean (only mlock.rs diffs, g6-exempt).

## RESUME HERE — exact next actions
1. **P1 re-plan-R0:** dispatch `feature-dev:code-architect` to re-review `design/IMPLEMENTATION_PLAN_mstring_grouping_p1_md.md` (verify the R0-r1 corrections block resolves C1-C3/I2-I4; new-drift sweep). Persist verbatim → `design/agent-reports/mstring-display-grouping-plan-r0-p1-round2-review.md`. Loop to **0C/0I**.
2. **Execute P1 inline** (in descriptor-mnemonic; create branch `feature/mstring-display-grouping` there). Follow the P1 plan with its corrections block. Order: Task1(md-codec fns) → Task2(vectors+checksum+CI) → **Task4(intake strip, 6 surfaces) → Task3(encode flags + fix smoke.rs/help_examples.rs/cli_repair.rs via `--group-size 0`)** → Task5(suite+fmt) → Task6(FOLLOWUP companion) → Task7(version bump md-codec 0.36.0 + md-cli 0.7.0, pin `=0.36.0`).
3. **PAUSE before release:** do NOT `git tag`/`git push`/`cargo publish` without explicit user authorization (md-codec must publish before md-cli's `=0.36.0` pin). Report code-complete + green, request authz.
4. Then **P2** (mnemonic-secret/ms-cli), **P3** (mnemonic-key/mk-cli), **P4** (toolkit pin-bumps + wire emit/intake + DELETE `chunk_5char`/`chunk_mk1`/`chunk_md1` + golden regen `tests/vectors/v0_1`+`v0_2` + manual + technical-manual remove dead `chunk_*` rows + Examples.pdf regen), **P5** (mnemonic-gui schema_mirror). Each: own plan → plan-R0 GREEN → TDD execute → pause before release.

## Hard constraints / gotchas (do not relearn the hard way)
- **R0 gate is mandatory** at every level (spec, each plan, each phase): loop architect to 0C/0I, persist verbatim to `design/agent-reports/` BEFORE folding. It has caught real bugs in EVERY round this cycle.
- **NEVER `cargo fmt` mlock.rs** (g6 permanent exemption). Toolkit fmt gate = `cargo +1.95.0 fmt --all -- --check`; tolerate ONLY mlock.rs diffs. md/ms/mk fmt on stable (no exemption). Run the pinned fmt before any toolkit push.
- **md-cli is bin-only** (no lib.rs) → both pure fns live in **md-codec** (lib) for conformance-test reachability. Same `--lib`-runs-zero-tests trap applies in toolkit (use a real lib module; `--lib` only runs lib tests).
- **md-codec keeps `render_codex32_grouped`** as a thin wrapper (`render_grouped(s,n,'-')`) — public API + technical-manual entry; do not rename/remove.
- **Six md1-intake surfaces** in md-cli: decode/bytecode/verify/inspect/address/repair (shared `strip_md1_inputs`); address strips inside `build_descriptor`.
- **Release = outward-facing**: confirm with user before any tag/publish (per phase). Stage paths explicitly (no `git add -A`).
- Commit trailer: `Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`.
- Lockstep (P4/P5): GUI `schema_mirror` (flags + separator keyword dropdown, paired PR), toolkit `docs/manual/src/40-cli-reference` (all 4 CLIs, common-flags section), sibling `FOLLOWUPS.md` companions (`display-grouping-render-strip-v1`).

## Kickoff prompt for the next session
> Resume the mstring display-grouping cycle. Read `design/CONTINUITY_mstring_grouping.md` (and the spec + P1 plan it points to), then continue P1: re-dispatch the P1 plan-R0 to GREEN (0C/0I, persist verbatim), then execute P1 inline in descriptor-mnemonic per the plan's corrections block, pausing for my authorization before the md-codec/md-cli tag+publish.
