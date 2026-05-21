# Cycle 5 kickoff — `mnemonic-toolkit-v0.29.1` (jade-seedqr)

**Created:** 2026-05-21, at the natural break after Wave 3 (Cycle 4 SemVer-minor cliff) shipped. Read this file first when resuming Cycle 5 work in a fresh session.

## Where we are

- **Last shipped cycle:** Cycle 4 (Wave 3) — `mnemonic-toolkit-v0.29.0` (post-amend `eebf798`) + paired `mnemonic-gui-v0.14.0` (`8f9e83b`) on 2026-05-21.
- **Cycles 1-4 in v0.28+ residual plan: ALL SHIPPED.**
- **Cycles 5-8 remain** — all multi-week parser cycles; plan-docs DEFERRED per brainstorm spec.

## Cycle 5 scope (from brainstorm spec)

Per `design/BRAINSTORM_v0_28_plus_residual_followups.md` §"Cycle 5 — `mnemonic-toolkit-v0.29.1` (jade-seedqr)":

- **Slug:** `wallet-import-jade-seedqr`
- **Shape:** add SeedQR parser to `wallet_import/jade.rs`; new fixture; chapter-45 prose; test cells.
- **CLI surface:** add `--format jade-seedqr` value OR fold into auto-detect under existing `--format jade`. **Decision deferred to cycle-start brainstorm.**
- **GUI lockstep:** mandatory IF new `--format` value added.
- **Effort:** multi-week.
- **SemVer:** PATCH (additive enum value if --format jade-seedqr; pure PATCH if auto-detect).

## Resume prompt

After `/clear`, issue this prompt:

```
Read design/CYCLE_5_KICKOFF.md and proceed with Cycle 5 (jade-seedqr) — brainstorm + plan-doc + execution end-to-end. Use the same discipline as Cycles 3 + 4 (P0 STRICT-GATE recon → plan-doc R0+R1 opus review → subagent-driven implementation → opus end-of-cycle review → split commits if mechanical work + version-bump are bundled → install-pin-check CI gate on tag push).
```

## Disciplines proven in Cycles 1-4 (preserve forward)

### P0 STRICT-GATE recon (Cycles 3 + 4)

Dispatch parallel Explore (read-only) agents BEFORE writing plan-doc body:
- Re-verify FOLLOWUPS line citations against current source (line numbers drift every merge).
- Confirm scope: FOLLOWUPS body framing can be stale relative to actual code state. Surface scope-drift findings as plan-doc inputs.
- For cross-repo cycles: include GUI repo state recon (pin lag, schema-mirror file, latest tag).
- Save dossier at `design/cycle-N-p0-recon.md`.

### Plan-doc reviewer-loop (CLAUDE.md mandate)

- Opus R0 review on plan-doc body BEFORE execution.
- Persist opus output verbatim at `design/agent-reports/v<N>-plan-doc-r0-review.md` BEFORE applying folds.
- Fold inline → R1 verify (sonnet for trivial fold-verify, opus for non-trivial).
- Repeat until 0C/0I.

### Opus end-of-cycle reviewer (Cycles 3-8 per brainstorm)

- Dispatch opus on full uncommitted working tree.
- Verify each slug + cross-cutting (alphabetical-by-variant; install-pin-check self-pin; GUI lockstep if applicable; test count math).
- Persist verbatim at `design/agent-reports/v<N>-phase-N-end-of-cycle-review.md`.

### Bisect-hygiene split commit (Cycle 4 R0-I3 precedent)

If the cycle bundles mechanical-only work (e.g., retroactive sort) with semantic changes:
- Commit 1: mechanical-only (pure reorder/refactor; no semantic change).
- Commit 2: semantic changes + version bump.
- Same tag on Commit 2.
- Sonnet diff-verify between commits.

### SHA self-reference under amend (3x recurrence: Cycles 2, 3, 4)

Accept the pattern: commit → sed-backfill `<PLACEHOLDER-COMMIT-SHA>` → `git commit --amend --no-edit` → SHA in FOLLOWUPS notes is pre-amend; post-amend HEAD differs. Tag is the durable anchor.

### Cross-repo lockstep ordering (Cycle 4 precedent)

Toolkit tag MUST land first. GUI dep can't resolve to a not-yet-existent tag. Order:
1. Toolkit commit (+ amend SHA backfill) + tag + push.
2. Toolkit install-pin-check CI GREEN.
3. Toolkit GH Release.
4. GUI pin bump (`pinned-upstream.toml` + `Cargo.toml`) + `cargo update`.
5. GUI `schema_mirror` test verify (use explicit `MNEMONIC_BIN=<full path>` since `$PATH` often resolves stale).
6. GUI CHANGELOG + Cargo.toml version bump.
7. GUI commit + tag + push.
8. GUI GH Release.
9. Closure-verification: GUI CI `schema_mirror` gate GREEN on tag.

### Stale `$PATH` binary gotcha (Cycle 4 surprise)

`mnemonic-gui/tests/schema_mirror.rs` resolves `mnemonic` binary via `MNEMONIC_BIN` env var or `$PATH`. The latter picks up the installed binary which is often stale. **ALWAYS use explicit `MNEMONIC_BIN=/scratch/code/shibboleth/mnemonic-toolkit/target/debug/mnemonic`** when running schema_mirror locally. CI environments install pinned binary correctly.

### `schema_mirror` gate scope (Cycle 4 R0-I1)

GUI's `schema_mirror` integration test enforces **clap flag-name parity** between hand-maintained `SubcommandSchema` and `gui-schema` JSON. It does **NOT** gate JSON wire-shape from `--json` output. If a cycle changes JSON wire-shape only (no clap surface change), the schema-mirror file needs zero edits. Verify with `gui-schema` JSON byte-diff before assuming the mirror needs updates.

## Memory entries to consult

- `project_v0_29_0_cycle_shipped` — Cycle 4 full context (most recent cycle).
- `project_v0_28_7_cycle_shipped` — Cycle 3.
- `project_v0_28_plus_wave_1_shipped` — Cycles 1 + 2.
- `feedback_a0_recon_check_gui_schema_json` — toolkit `--help` vs `gui-schema` JSON divergence.
- `feedback_no_parallelism_for_code_generation` — Parts A + B (worktree-isolation invariant; parallel-safe only for read-only Explore).
- `feedback_opus_primary_review_agent` — opus is the primary review agent for substantive cycles.
- `feedback_architect_must_run_prose_commands` — for manual chapters / recipes: source-faithful prose can still ship broken if commands fail.
- `feedback_r0_must_read_source_off_by_n` — every R0 should grep against source ground truth.

## Repo state at session-end

- mnemonic-toolkit master HEAD: `eebf798` (post-amend Cycle 4 commit; pre-amend was `49cb211`).
- mnemonic-gui master HEAD: `8f9e83b` (Cycle 4 paired GUI commit).
- Both tags pushed to origin.
- Both GH Releases live.
- Toolkit install-pin-check CI on `mnemonic-toolkit-v0.29.0`: PASS.
- GUI build + schema_mirror CI on `mnemonic-gui-v0.14.0`: in_progress at session end — verify post-`/clear` with `gh run list -R bg002h/mnemonic-gui --limit 3 --json status,conclusion,headBranch,name`.

## Cycle 5 brainstorm starter questions (for the first AskUserQuestion in the new session)

These came up during Cycle 4 planning but were out-of-scope:

1. **CLI surface decision:** new `--format jade-seedqr` value vs fold into auto-detect under existing `--format jade`?
   - New value: clearer user intent; mandates GUI lockstep.
   - Auto-detect: pure PATCH semver; no GUI lockstep; but ambiguous if input is malformed.
2. **SeedQR variant scope:** standard SeedQR (numeric BIP-39 indices) only? Or also CompactSeedQR (binary entropy)?
3. **Fixture sources:** does the implementer have access to a Jade device for empirical fixture capture? Or synthesize from spec?
4. **Chapter-45 prose:** new section under "Foreign-format wallet imports" → "Jade" chapter, or extend existing Jade section?

These should drive the cycle-start brainstorm spec before plan-doc body is written.
