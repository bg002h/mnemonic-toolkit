# v0.26.0 / mnemonic-gui-v0.11.0 three-way merge plan

**Canonical record per `coordinator-runbook-into-design-dir` FOLLOWUP** (closed in v0.27.0; see `design/FOLLOWUPS.md`). Promoted from project-root scratch `.v0_26_0-merge-plan.md` into `design/` as the multi-instance coordination playbook for future cycles. `CLAUDE.md` cross-cites this file under Conventions.

**Author:** Claude (compare-cost instance) — drafted, reviewed, and folded by Claude; **executed by the user as merge coordinator**.
**Status:** R3 (folds opus R0+R1 reviewer-loop + R2 architect re-evaluation; switched to integration-branch model; stripped Claude-coordinator role).
**Created:** 2026-05-18

## Topology: integration-branch model (R2 architect recommendation)

**Master stays at v0.25.1 throughout the cycle.** Cycle work happens on dedicated integration branches:

- **`release/v0.26.0`** in `bg002h/mnemonic-toolkit` — created from current master (`7c1f874`). All three toolkit feature PRs are retargeted from `master` → `release/v0.26.0` and merged into it sequentially with rebases between. A final integration PR (`release/v0.26.0 → master`) is the single squash-merge to master; the tag fires on that merge commit.
- **`release/v0.11.0`** in `bg002h/mnemonic-gui` — same shape; created after the toolkit tag fires.

This isolates conflict-resolution from master, eliminates the "half-baked master" hazard between phases, and gives a single dry-run-CI moment on the final integration PR. Adds one coordination step (the integration branch) and one PR (the integration PR) per repo.

## Coordinator role: human, not Claude (R2 fold)

The user is the merge coordinator. Each phase is labeled `[user]`; specific branch-handling work is annotated `[Claude-compare-cost assists]` (this instance, PR #23/#5), `[Claude-xpub-search assists]` (instance-2, PR #24/#6), `[Claude-wallet-import assists]` (instance-3, PR #25/#7). Claude instances DO NOT push to branches owned by other instances; the user dispatches branch-specific instructions to each Claude separately.

(Instance-identity table in memory: `[[project-v0-26-0-compare-cost-instance-identity]]`.)

## Scope

Coordinate the merge + release of three parallel feature cycles all targeting `mnemonic-toolkit-v0.26.0` + `mnemonic-gui-v0.11.0`:

| Toolkit PR | Branch | Feature | Files | Delta | CI |
|---|---|---|---|---|---|
| [#23](https://github.com/bg002h/mnemonic-toolkit/pull/23) | `compare-cost/p1-miniscript` | compare-cost (wsh-vs-tr per-spending-condition cost) | 17 | +2567/−3 | 9/9 GREEN |
| [#24](https://github.com/bg002h/mnemonic-toolkit/pull/24) | `worktree-xpub-search-brainstorm` | xpub-search umbrella (4 modes) + P2PKH gap-fix in convert.rs | 31 | +6515/−34 | 9/9 GREEN |
| [#25](https://github.com/bg002h/mnemonic-toolkit/pull/25) | `worktree-wallet-import-export-multiformat-brainstorm` | wallet-import (BSMS+Core) + @env:VAR sentinel cross-cut | 64 | +10635/−31 | **lint + test FAIL** |

| GUI PR | Branch | Feature | Files | Toolkit pin |
|---|---|---|---|---|
| [#5](https://github.com/bg002h/mnemonic-gui/pull/5) | `compare-cost/p4-gui` | compare-cost panel | 5 | master pin (v0.25.1; deferred) |
| [#6](https://github.com/bg002h/mnemonic-gui/pull/6) | `worktree-xpub-search-v0-11-0` | 4 xpub-search subcommand schemas | 8 | rev-pin `02090eb` (#24 tip) |
| [#7](https://github.com/bg002h/mnemonic-gui/pull/7) | `feat/import-wallet-v0_11_0` | import-wallet subcommand schema | 3 | **v0.24.0 (stale)** — DRAFT |

## Review findings folded into this plan

Per opus `feature-dev:code-reviewer` (model: opus) R0 review, 2026-05-18:

### Critical (must address)

**C1 — `@env:VAR` semantic gap in xpub-search secret-bearing flags.** PR #25's resolver is wired to 6 enumerated surfaces (convert's `--passphrase`/`--bip38-passphrase`, `--ms1`, `--share`, `--slot @N.phrase=`, `--slot @N.ms1=`). PR #24's xpub-search adds NEW secret-bearing flags (`--passphrase` on `passphrase-of-xpub`; `--phrase` on all four modes) that are NOT in #25's list. After both merge cleanly (no git conflict — different files), the new flags silently treat `@env:MY_PP` as a literal passphrase string and produce a no-match.

**Resolution:** filed as **post-merge step P1** (before tag fires). Wire `resolve_env_var_sentinel` into all 5 xpub-search secret-bearing flag surfaces. Adds ~30 LOC + ~5 test cells. Folded into the rebase of whichever of #23/#24 lands after #25.

**C2 — Subcommand-count off-by-one.** Master has **12** user-facing subcommands (not 13 as I'd assumed). Branch deltas: #23 +1 → 13; #24 +4 → 16; #25 +1 → 13. Final union count after all three merge = **12 + 4 + 1 + 1 = 18** (not 19). Each PR's `cli_gui_schema.rs` test renames the function (`_thirteen_subcommands` / `_sixteen_subcommands`); the final rebase must rename to `_eighteen_subcommands` and assemble the alphabetically-sorted vec from the union.

### Important

**I1 — `gui_schema.rs::build_subcommand_conditional_rules` dispatcher arm drop risk.** Three-way merge could silently drop one arm if two PRs insert at the identical line. Mitigation: after each rebase, run `grep -c '=> .*_conditional_rules()' crates/mnemonic-toolkit/src/cmd/gui_schema.rs` and verify the count grows monotonically (master baseline + N expected per branch).

**I2 — `error.rs` variant ordering canonicalization.** 9+ new variants across three PRs (compare-cost: 1; xpub-search: 1 `XpubSearchNoMatch`; wallet-import: 7 `EnvVarMissing` + 6 wallet-import variants). Each PR's `impl Display` + `exit_code()` matches the same `match self` blocks. **Canonical ordering rule for rebases:** alphabetical by variant name within each block (variant declaration, Display arm, exit_code arm). Adopt before rebases start so all three converge to the same order.

**I3 — GUI #7's v0.24.0 pin crosses v0.25.0 TTY-gate behavior change.** v0.25.0 extended the single-gate-at-entry TTY rule to `convert` + `inspect` subcommands (piped users must set `MNEMONIC_FORCE_TTY=1`). #7's 8 kittest cells were written against v0.24.0; subprocess invocations may need spawn-env adjustment.

**Resolution:** filed as **GUI-bump step G1.5**. Audit #7's 8 kittest cells for `mnemonic convert/inspect` invocations via piped stdin without `MNEMONIC_FORCE_TTY=1`; add the env-var per `[[project-v0-9-0-mnemonic-gui-shipped]]` precedent.

### Minor

**M1 — Install.sh ordering is load-bearing.** #24 first is mandatory (not convenience): if #25 landed first, master would have `Cargo.toml=0.26.0` + `install.sh=v0.25.1`, so `curl … | bash` installs the v0.25.1 binary against a 0.26.0-shaped GUI schema.

**M2 — CHANGELOG v0.26.0 block is a guaranteed manual conflict** (both #24 and #25 write the `## [0.26.0]` header at the top); not auto-mergeable.

## Recipe — toolkit merge phases (T0–T5)

### Phase T0 — preflight (no merges yet)

- T0.1 [user]: dispatch `[Claude-wallet-import]` to fix PR #25's CI (lint + test failures must clear before T2 can fire).
- T0.2 [user]: decide C1 disposition. Two options:
  - (a) Dispatch `[Claude-wallet-import]` to fold the `@env:VAR` xpub-search wiring into PR #25 directly (recipe at §T0.2.a below). Cleanest; one PR fewer.
  - (b) Ships as a separate follow-on PR T2.5 in this cycle.
- T0.3 [user]: pick (a) or (b). **If (a): strike T2.5 (task #21) — C1 closes inside T2.**

#### T0.2.a — instance-3 wiring brief (option (a) only; per P-I1 fold)

Wire `crate::env_sentinel::resolve_env_var_sentinel` (the same helper PR #25 wires into convert/bundle/etc.) into all xpub-search secret-bearing flag surfaces. Read the existing pattern at `crates/mnemonic-toolkit/src/cmd/convert.rs:154` (PR #25's `parse_from_input` rewrite) and mirror at:

| File | Field | Notes |
|---|---|---|
| `crates/mnemonic-toolkit/src/cmd/xpub_search/path_of_xpub.rs` | `--phrase` (Source::Phrase) and `--passphrase` | Plumb through `seed_intake.rs::Source` and the passphrase line |
| `crates/mnemonic-toolkit/src/cmd/xpub_search/account_of_descriptor.rs` | `--phrase` and `--passphrase` | Reuses `resolve_seed` from seed_intake |
| `crates/mnemonic-toolkit/src/cmd/xpub_search/passphrase_of_xpub.rs` | `--phrase` and `--passphrase` (mandatory) | Mandatory mutex + resolve before `derive_master_seed` |
| (address-of-xpub: no seed material — no wiring needed) | — | — |

Pattern (mirror convert.rs precisely; keep clap-derive boilerplate unchanged, route the string through the resolver before consuming):

```rust
let resolved_passphrase: Zeroizing<String> = match args.passphrase.as_deref() {
    Some(raw) => resolve_env_var_sentinel(raw)
        .map_err(|e| ToolkitError::XpubSearch(XpubSearchError::Other(e.to_string())))?
        .into(),
    None => Zeroizing::new(String::new()),
};
```

Add 1 test cell per wired flag exercising the `@env:` sentinel + the missing-var exit-1 case (4 modes × 2 flags = ~8 cells; reuse `tests/cli_env_var_sentinel.rs` precedent from PR #25).

### Phase T1.0 — create integration branch (NEW; R2 fold)

- T1.0.1 [user]: `git checkout master && git pull && git checkout -b release/v0.26.0 && git push -u origin release/v0.26.0`. Master remains at v0.25.1 (`7c1f874`).
- T1.0.2 [user]: optionally enable branch protection on `release/v0.26.0` (status-check-required, no force-push). Not load-bearing for the merge but tightens the dry-run-CI moment at T3.5.

### Phase T1 — merge PR #24 into `release/v0.26.0`

- T1.1 [user]: confirm #24's CI is GREEN against current master.
- T1.2 [user]: retarget PR #24's base from `master` → `release/v0.26.0` via `gh pr edit 24 --base release/v0.26.0 --repo bg002h/mnemonic-toolkit`. CI re-runs against the new base.
- T1.3 [user]: squash-merge #24 → `release/v0.26.0`. Integration branch becomes v0.26.0-shaped (Cargo.toml=0.26.0, install.sh=v0.26.0, CHANGELOG v0.26.0 block, 10 FOLLOWUPs).
- T1.4: **master is unchanged** — still at v0.25.1.

### Phase T2 — rebase + merge PR #25 into `release/v0.26.0`

- T2.1 [user] (blocked on T0.1) [Claude-wallet-import assists]: rebase `worktree-wallet-import-export-multiformat-brainstorm` onto current `release/v0.26.0`.
- T2.2 [user] [Claude-wallet-import assists]: conflict resolution per file:
  - `Cargo.toml` — drop own version bump (already 0.26.0); keep `similar = "2"` dep add.
  - `Cargo.lock` — **accept the integration branch's lockfile** via `git checkout --theirs Cargo.lock && cargo check --workspace`. **Do NOT `cargo update -w`** (P-I2 fold).
  - `CHANGELOG.md` — M2 manual fold: merge wallet-import `### Added` bullets into the existing v0.26.0 block in **chronological PR-merge order** (xpub-search bullets first, wallet-import second; P-M1 fold). Drop the duplicate `## [0.26.0]` header.
  - `error.rs` — I2 canonical ordering: alphabetical variant names; Display + exit_code arms in same order.
  - `cli_gui_schema.rs` — C2 fold: rename test fn to `_seventeen_subcommands` (12 master + 4 #24 + 1 #25 = 17); add `import-wallet` to the alphabetically-sorted vec.
  - `cmd/gui_schema.rs` — I1 verification: `grep -c '=> .*_conditional_rules()' crates/mnemonic-toolkit/src/cmd/gui_schema.rs` returns baseline + 4 + 1.
  - `cmd/convert.rs` — git auto-merges (non-overlapping hunks); spot-check combined `ScriptType` + `@env:VAR` coherence.
- T2.3 [user] [Claude-wallet-import assists]: re-run `cargo test --workspace` + `cargo clippy --workspace --all-targets -- -D warnings` + `make -C docs/manual lint` locally on the rebased branch.
- T2.4 [user] [Claude-wallet-import assists]: retarget PR #25's base to `release/v0.26.0` (`gh pr edit 25 --base release/v0.26.0`); force-push the rebased branch; wait for CI green.
- T2.5 [user]: squash-merge #25 → `release/v0.26.0`.

### Phase T2.5 — (conditional) C1 post-merge wiring PR

- **Only if T0.3 chose option (b):** [user] [Claude-wallet-import assists]: open a new follow-on PR (base = `release/v0.26.0`) wiring `resolve_env_var_sentinel` into xpub-search's `--phrase` (4 modes) + `--passphrase` (1 mode). ~30 LOC + ~5 test cells per §T0.2.a brief. Merge into `release/v0.26.0` before T3.5 dry-run.

### Phase T3 — rebase + merge PR #23 into `release/v0.26.0`

- T3.1 [user] [Claude-compare-cost assists]: rebase `compare-cost/p1-miniscript` onto current `release/v0.26.0`.
- T3.2 [user] [Claude-compare-cost assists]: conflict resolution per file:
  - `Cargo.toml` — no version bump needed (already 0.26.0).
  - `Cargo.lock` — `git checkout --theirs Cargo.lock && cargo check --workspace` (P-I2 fold).
  - `CHANGELOG.md` — append compare-cost `### Added` bullet THIRD in v0.26.0 block (PR-merge order).
  - `error.rs` — I2 alphabetical insert of `CompareCost` variant; Display + exit_code arms.
  - `cli_gui_schema.rs` — C2 fold: rename test fn to `_eighteen_subcommands`; add `compare-cost` to vec (alphabetically positioned just before `convert`).
  - `cmd/gui_schema.rs` — I1 verification: add `compare_cost_conditional_rules()` dispatcher arm; verify final count is baseline + 4 + 1 + 1 = 6 new arms.
  - `main.rs` — alphabetical insert of `Command::CompareCost` variant + dispatch arm.
  - `docs/manual/src/40-cli-reference/41-mnemonic.md` — append `## mnemonic compare-cost` chapter after the last xpub-search section.
  - `docs/manual/tests/cli-subcommands.list` — append `mnemonic compare-cost` (alphabetically).
  - `docs/manual/.cspell.json` — additive merge of new words.
- T3.3 [user] [Claude-compare-cost assists]: run cargo test + clippy + manual lint locally on the rebased branch.
- T3.4 [user] [Claude-compare-cost assists]: retarget PR #23's base to `release/v0.26.0`; force-push; wait for CI green; squash-merge into `release/v0.26.0`.

### Phase T3.5 — integration PR `release/v0.26.0 → master` (NEW; R2 fold)

- T3.5.1 [user]: `gh pr create --base master --head release/v0.26.0 --title "release(toolkit): mnemonic-toolkit v0.26.0 — three-feature lockstep (xpub-search + wallet-import + compare-cost)" --body <consolidated release notes>`. This is the single integration PR; its CI run is the dry-run of exactly what gets tagged.
- T3.5.2 [user]: verify all CI checks GREEN on the integration PR (full `cargo test --workspace`, clippy, manual lint, install-pin-check, 18-subcommand test).
- T3.5.3 [user]: **squash-merge** the integration PR into master. The resulting squash commit IS the v0.26.0 release commit.

### Phase T4 — tag `mnemonic-toolkit-v0.26.0`

- T4.1 [user]: verify master HEAD = the squash-merge commit produced by T3.5.3. Capture SHA: `MERGE_SHA=$(gh pr view <integration-PR-#> --repo bg002h/mnemonic-toolkit --json mergeCommit --jq '.mergeCommit.oid')`. Confirm `git log -1 --format=%H master` equals `$MERGE_SHA`.
- T4.2 [user]: `git tag mnemonic-toolkit-v0.26.0 $MERGE_SHA && git push origin mnemonic-toolkit-v0.26.0`. (P-M3 fold: tag the merge commit explicitly, not a re-detected HEAD.)
- T4.3 [user]: create GitHub Release pointing at the tag, body = consolidated CHANGELOG section.
- T4.4 [user] (optional): delete `release/v0.26.0` branch post-tag (`git push origin --delete release/v0.26.0`).

### Phase T5 — file post-cycle FOLLOWUPs

Not all anticipated FOLLOWUPs were already filed (each cycle filed its own subset; combined cycle FOLLOWUPs may add new ones):
- `gui-schema-arm-drop-detector` — formalize the `grep -c` count assertion into a `tests/cli_gui_schema_arm_count.rs` regression test (I1 codification).
- `error-rs-canonical-ordering-doc` — record the alphabetical-by-variant-name rule in `CLAUDE.md` or `design/CONVENTIONS.md` (I2 codification).
- `coordinator-runbook-into-design-dir` — promote this plan-doc to `design/PLAN_v0_26_0_three_way_merge.md` for future-cycle reference.

## Recipe — mnemonic-gui merge phases (G1–G4)

**Blocked on T4 firing** — every GUI PR's schema-mirror gate runs against the tagged toolkit binary.

**Sequential G1→G2→G3 is chosen for review clarity, not technical necessity.** The three GUI PRs touch disjoint `src/schema/mnemonic.rs` entries (different subcommand schemas) and disjoint `tests/` files; they could merge in any order with rebase-on-master at each merge. Sequential keeps the CHANGELOG consolidation simpler since each merge inherits the previous PR's CHANGELOG block. (P-M4 fold.)

### Phase G0 — preflight

- G0.1: GUI v0.11.0 will own a unified CHANGELOG block similar to the toolkit; consolidate the three GUI PRs' entries.

### Phase G1.0 — create GUI integration branch (NEW; R2 fold)

- G1.0.1 [user]: in mnemonic-gui: `git checkout master && git pull && git checkout -b release/v0.11.0 && git push -u origin release/v0.11.0`. Master remains at v0.10.0.

### Phase G1 — unblock + merge PR #7 (DRAFT) into `release/v0.11.0`

- G1.1 [user] [Claude-wallet-import assists]: bump `pinned-upstream.toml:22` v0.24.0 → v0.26.0.
- G1.2 [user] [Claude-wallet-import assists]: bump `Cargo.toml:42` toolkit dep `tag` v0.24.0 → v0.26.0.
- G1.3 [user] [Claude-wallet-import assists]: bump `Cargo.toml:3` GUI version 0.10.0 → 0.11.0.
- G1.4 [user] [Claude-wallet-import assists]: add CHANGELOG `[0.11.0]` entry for import-wallet.
- **G1.5 (I3 + P-I3 fold) [user] [Claude-wallet-import assists]: audit the 8 `tests/kittest_import_wallet_form.rs` cells** for two v0.24.0→v0.26.0 behavior changes:
  - (a) v0.25.0 TTY-gate extension to `convert`+`inspect`: piped-stdin invocations must set `MNEMONIC_FORCE_TTY=1` in spawn env (`[[project-v0-9-0-mnemonic-gui-shipped]]`).
  - (b) v0.25.1 empty-`ms1` watch-only sentinel: multi-cosigner `verify-bundle`/`repair` cells with `ms1[idx]==""` now emit a stderr NOTICE per cosigner (`[[project-v0-25-1-patch-shipped]]`). Update or move to substring-match.
- G1.6 [user]: flip Draft → Ready.
- G1.7 [user]: retarget PR #7's base to `release/v0.11.0` (`gh pr edit 7 --base release/v0.11.0 --repo bg002h/mnemonic-gui`); squash-merge into `release/v0.11.0`.

### Phase G2 — flip PR #6's rev-pin → tag-pin, merge into `release/v0.11.0`

- G2.1 [user] [Claude-xpub-search assists]: edit `Cargo.toml:42` from `rev = "02090eb"` to `tag = "mnemonic-toolkit-v0.26.0"`.
- G2.2 [user] [Claude-xpub-search assists]: bump `pinned-upstream.toml` to v0.26.0.
- G2.3 [user] [Claude-xpub-search assists]: rebase onto current `release/v0.11.0`; resolve `src/schema/mnemonic.rs` (additive 4 entries) + `CHANGELOG.md` (append xpub-search bullets to existing v0.11.0 block).
- G2.4 [user] [Claude-xpub-search assists]: retarget base to `release/v0.11.0`; force-push; wait for schema-mirror gate green; squash-merge.

### Phase G3 — flip PR #5 toolkit pin, merge into `release/v0.11.0`

- G3.1 [user] [Claude-compare-cost assists]: edit `Cargo.toml` toolkit dep to `tag = "mnemonic-toolkit-v0.26.0"`.
- G3.2 [user] [Claude-compare-cost assists]: bump `pinned-upstream.toml` to v0.26.0.
- G3.3 [user] [Claude-compare-cost assists]: rebase onto current `release/v0.11.0`; resolve `src/schema/mnemonic.rs` (additive `compare-cost` entry) + `CHANGELOG.md` (append compare-cost bullets).
- G3.4 [user] [Claude-compare-cost assists]: retarget base to `release/v0.11.0`; force-push; wait for schema-mirror gate green; squash-merge.

### Phase G3.5 — integration PR `release/v0.11.0 → master` (NEW; R2 fold)

- G3.5.1 [user]: `gh pr create --base master --head release/v0.11.0 --title "release(gui): mnemonic-gui v0.11.0 — toolkit v0.26.0 three-feature lockstep" --body <consolidated release notes>`.
- G3.5.2 [user]: verify all CI checks GREEN on the integration PR.
- G3.5.3 [user]: squash-merge the integration PR into master.

### Phase G4 — tag `mnemonic-gui-v0.11.0`

- G4.1 [user]: capture `MERGE_SHA=$(gh pr view <gui-integration-PR-#> --repo bg002h/mnemonic-gui --json mergeCommit --jq '.mergeCommit.oid')`.
- G4.2 [user]: `git tag mnemonic-gui-v0.11.0 $MERGE_SHA && git push origin mnemonic-gui-v0.11.0`. (P-M3 fold.)
- G4.3 [user]: GitHub Release.
- G4.4 [user] (optional): delete `release/v0.11.0` branch post-tag.

## Per-file conflict-resolution cheat-sheet (toolkit)

| File | Resolver | Rule |
|---|---|---|
| `Cargo.toml` | manual | Version stays 0.26.0; preserve all dep additions (similar="2" from #25); the only difference is the version line which agrees in all post-T1 cases |
| `Cargo.lock` | accept theirs | `git checkout --theirs Cargo.lock && cargo check --workspace` (per P-I2 fold; **do NOT `cargo update -w`** — that drifts 50+ transitive deps) |
| `CHANGELOG.md` | manual (M2) | Single `## [0.26.0]` header; consolidate `### Added` bullets in **chronological PR-merge order** (xpub-search #24 first / wallet-import #25 second / compare-cost #23 third) so release-notes narrative tracks merge sequence and Cargo.toml lineage; each cycle's deeper changes (Changed/Deprecated/Removed) flow under existing block. (P-M1 fold: rejected "alphabetical by feature name" as bikeshed-tier; PR-order is reader-friendly and matches `git log master --oneline`.) |
| `src/error.rs` | manual (I2) | Variants in `match self`-touched enums: alphabetical by name; Display + exit_code arms in same order |
| `tests/cli_gui_schema.rs` | manual (C2) | Rename fn to `_{N}_subcommands` matching final count; alphabetical vec union |
| `src/cmd/gui_schema.rs` | manual (I1) | Dispatcher arms order: keep insertion clusters; verify `grep -c` count post-rebase |
| `src/main.rs` | manual | Alphabetical `Command::Xxx` variants; dispatch arms in same order |
| `src/cmd/mod.rs` | git auto | `pub mod xxx;` alphabetical |
| `docs/manual/src/40-cli-reference/41-mnemonic.md` | manual | Append chapters alphabetically by subcommand name (compare-cost, import-wallet, xpub-search-{...}) |
| `docs/manual/tests/cli-subcommands.list` | manual | Alphabetical |
| `docs/manual/.cspell.json` | manual | Alphabetical word merge inside the words array |
| `scripts/install.sh` | none after T1 | #24 already bumps; subsequent PRs don't need to touch |
| `cmd/convert.rs` | git auto | #24 + #25 hunks non-overlapping; spot-check coherence post-merge |

## Risk matrix

| Risk | Likelihood | Blast | Mitigation |
|---|---|---|---|
| I1: dispatcher arm drop | medium | Schema-mirror CI red post-tag | `grep -c` assertion per rebase; codified test as FOLLOWUP |
| I2: variant ordering drift | high | One rebase's hunk loses to another's | Canonical alphabetical rule documented; verify per rebase |
| C1: xpub-search @env:VAR gap | high (semantic, not detectable by CI) | Users supply `@env:MY_PP` as literal text | Post-merge wiring PR before tag |
| C2: subcommand count miscount | medium | Test rename mismatch → CI red | Plan documents 12→13→17→18 progression explicitly |
| #25 CI still red | high | Cannot merge | instance-3 fix required for T2 |
| M1: install.sh stale on out-of-order merge | high | curl install installs old binary | T1 (PR #24 first) is load-bearing |
| I3: GUI #7 v0.24.0 pin spawn-env drift | medium | kittest cells flake post-pin-bump | G1.5 audit step |

## Open user decisions

1. **T0.3:** Where does C1 fix live? (a) folded into #25 by `[Claude-wallet-import]`, OR (b) follow-on PR before T3.5 integration PR.
2. **#25 CI red ownership:** dispatch `[Claude-wallet-import]` for the fix (recommended — they wrote the code) OR fix it yourself.
3. **GUI #7 unblocking ownership:** dispatch `[Claude-wallet-import]` for G1.1–G1.6 (recommended) OR fix it yourself.
4. **Tag timing:** atomic (T4 right after T3.5) OR with cooling-off period for CI to bake on `release/v0.26.0` between phases? Recommended: atomic — the integration-PR CI run at T3.5.2 IS the cooling-off-period equivalent.
5. **Integration branch protection:** enable branch-protection-with-status-checks on `release/v0.26.0` (and `release/v0.11.0`)? Recommended: yes, to force the dry-run-CI moment at T3.5/G3.5.

## Status field for resumption

If interrupted, the doc state can be picked up by looking at:
- whether `release/v0.26.0` branch exists (`git ls-remote --heads origin release/v0.26.0`)
- which PRs are merged into `release/v0.26.0` (`gh pr list --state merged --repo bg002h/mnemonic-toolkit --search "is:merged base:release/v0.26.0"`)
- whether the integration PR (`release/v0.26.0 → master`) has been opened/merged
- whether `mnemonic-toolkit-v0.26.0` tag exists (`git ls-remote --tags origin "mnemonic-toolkit-v0.26.0"`)
- whether #25's CI on `release/v0.26.0` base is green
- mirror checks for GUI: `release/v0.11.0`, `mnemonic-gui-v0.11.0` tag
