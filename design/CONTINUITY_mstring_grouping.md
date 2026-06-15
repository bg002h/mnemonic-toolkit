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
- **P0 foundation SHIPPED** on the toolkit branch (reversible; no release): `design/display-grouping-vectors.tsv` (22 vectors); `crates/mnemonic-toolkit/src/display_grouping.rs` (`render_grouped`/`strip_display_separators`/`is_display_separator` + 10 unit tests; `pub mod display_grouping;` added to `lib.rs` UNCONDITIONALLY — NOT format.rs, which is `#[cfg(fuzzing)]`-gated); `crates/mnemonic-toolkit/tests/display_grouping_conformance.rs`. Full toolkit suite green; fmt-clean (only mlock.rs diffs, g6-exempt).
- **P1 plan-R0 GREEN** (round 3, 0C/0I): reviews `…plan-r0-p1-round{1,2,3}-review.md`. r2 caught a Task-3 `git add` omission (smoke.rs+cli_repair.rs); folded.
- **P1 CODE-COMPLETE on `descriptor-mnemonic@feature/mstring-display-grouping`** (7 commits `4259893..a3f9d8f`, branched off `main@eb9f368`; **NOT pushed/tagged/published — awaiting authz**). All gates green: `cargo test --workspace` (+`--all-features`), `--doc`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all --check` (md uses stable; no mlock exemption). What shipped on that branch: md-codec `render_grouped`/`strip_display_separators`/`is_display_separator` (+ `render_codex32_grouped` now a hyphen wrapper); copy-with-checksum vectors (`design/display-grouping-vectors.tsv`+`.sha256`, CI `sha256sum -c` in fmt job) + `crates/md-codec/tests/display_grouping_conformance.rs`; `cmd::strip_md1_inputs` + intake strip on ALL six md1 surfaces (decode/bytecode/verify/inspect/address[in `build_descriptor`]/repair[stdin+positional]); `md encode --group-size`/`--separator` (default space/5 print-once; `--json`+`repair` unbroken; smoke.rs/after_long_help/cli_repair `encode_chunked` got `--group-size 0`); FOLLOWUP `display-grouping-render-strip-v1`; **version bump md-codec 0.35.3→0.36.0 + md-cli 0.6.2→0.7.0 (pin `=0.36.0`) + CHANGELOG** committed. **NOTE: md-codec's codex32 decode ALREADY tolerates whitespace+hyphen (codex32.rs:134 "D11"); net-new md-cli strip coverage = comma + the repair decode path → conformance tests use COMMA-grouped fixtures to genuinely exercise the strip.**

## RESUME HERE — exact next actions
1. **AWAITING USER AUTHZ to release P1:** publish `md-codec 0.36.0` to crates.io FIRST (md-cli pins `=0.36.0`), then `md-cli 0.7.0`; then `git push` the branch + tags. md CI: tag pushes — confirm the rust workflow + any publish flow; md has NO mlock exemption (stable fmt). Until authz, leave the branch local. (Whether to PR→merge `main` first vs tag is the user's call.)
2. Then **P2** (mnemonic-secret/ms-cli), **P3** (mnemonic-key/mk-cli), **P4** (toolkit pin-bumps + wire emit/intake + DELETE `chunk_5char`/`chunk_mk1`/`chunk_md1` + golden regen `tests/vectors/v0_1`+`v0_2` + manual + technical-manual remove dead `chunk_*` rows + Examples.pdf regen + file the toolkit-side `display-grouping-render-strip-v1` companion), **P5** (mnemonic-gui schema_mirror). Each: own plan → plan-R0 GREEN → TDD execute → pause before release.

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
