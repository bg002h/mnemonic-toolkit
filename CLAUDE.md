# CLAUDE.md — mnemonic-toolkit repo notes

This file is auto-loaded by Claude Code when starting a session in this repository.

## What this is

`mnemonic-toolkit` is the top-level integration crate of the **m-format constellation**:

- [`md-codec`](https://github.com/bg002h/descriptor-mnemonic) — wallet descriptors / policies (`md1`, HRP `md`); CLI `md`.
- [`mk-codec`](https://github.com/bg002h/mnemonic-key) — xpubs (`mk1`, HRP `mk`); CLI `mk` (since v0.2).
- [`ms-codec`](https://github.com/bg002h/mnemonic-secret) — BIP-39 entropy (`ms1`, HRP `ms`); CLI `ms`.
- **mnemonic-toolkit** (this repo) — takes a seed phrase, emits the three cards as a coherent steel-engravable bundle; CLI `mnemonic`.

The three sibling codecs ship independently; this toolkit consumes them as library deps (git deps until they hit crates.io in lockstep with v0.1).

## Cross-repo follow-ups

When toolkit work surfaces an action item that affects a sibling codec, mirror an entry in BOTH repos' `design/FOLLOWUPS.md` with cross-citing `Companion:` lines. When the action ships, both entries update in lockstep.

## Manual coverage

The end-user manual lives at `docs/manual/` in this repo and is the single source of truth for the m-format constellation end-user surface (`mnemonic` / `md` / `ms` / `mk` CLIs). Tagged builds attach a PDF asset to the GitHub release (CI workflow at `.github/workflows/manual.yml`).

Mirror invariant: any flag/API addition or removal in this repo's CLI surface — or in the sibling-codec CLIs (`descriptor-mnemonic/md-cli`, `mnemonic-secret/ms-cli`, `mnemonic-key/mk-cli`) — must update the manual under `docs/manual/src/40-cli-reference/` in lockstep with the implementing PR. The bidirectional flag-coverage check lives at `docs/manual/tests/lint.sh` and is invoked via `make -C docs/manual lint MNEMONIC_BIN=... MD_BIN=... MS_BIN=... MK_BIN=...`; CI calls this from `.github/workflows/manual.yml`. The manual chapters mirror clap-derive's `--help` output for all four CLIs. See `design/FOLLOWUPS.md` entry `manual-cli-surface-mirror` for the canonical record; sibling repos carry companion entries.

## GUI schema-mirror coverage

The downstream consumer `mnemonic-gui` maintains a clap-flag schema mirror at `mnemonic-gui/src/schema/mnemonic.rs` (subcommand-by-subcommand flag listings + dropdown value enums), enforced by the `schema_mirror` integration test which runs `mnemonic gui-schema` against the pinned toolkit binary and compares against the hand-maintained schema.

Mirror invariant: **any flag/option/subcommand/dropdown-value addition, removal, or rename in `mnemonic-toolkit`'s clap-derived CLI surface MUST update `mnemonic-gui/src/schema/mnemonic.rs` in lockstep with the implementing PR** (same PR if cross-repo authoring is feasible; otherwise a paired sibling PR on mnemonic-gui).

The `schema_mirror` test fires on `mnemonic-gui` pin bumps via the toolkit binary it consumes — so missing the lockstep update does not surface immediately; it accumulates silently until the next GUI pin bump catches the cumulative delta. The drift gate is therefore a **lagging indicator**, not a leading one. The leading discipline is the paired-PR rule.

Historical case study (v0.27.0 + v0.27.1): neither cycle paired its toolkit CLI additions with a GUI schema-mirror update. v0.11.1's pin bump v0.26.0 → v0.27.2 fired the drift gate against 8 accumulated missing flags (`bundle --import-json`, `--import-json-index`; `export-wallet --bsms-form`, `--from-import-json`, `--from-import-json-index`, `bsms` format, `BSMS_FORMS` enum; `import-wallet --bsms-round1`, `--bsms-verify-strict`). The Phase 3 of v0.27.2 had to backfill all 8 in one go.

See `design/FOLLOWUPS.md` entry `gui-schema-mirror-lockstep-discipline` for the canonical record. Companion convention lives in `mnemonic-gui/CLAUDE.md`.

## Conventions

- **MANDATORY pre-implementation R0 gate — NO code before GREEN (0C/0I).** Every brainstorm spec and implementation plan-doc MUST pass an opus architect **R0 review BEFORE any implementation begins**. R0 is mandatory — never skipped, never deferred "until after a first cut." Implementation (writing code, dispatching implementer subagents) MUST NOT start until the reviewer-loop has **converged to 0 Critical / 0 Important** — fold findings → persist the review verbatim to `design/agent-reports/` → re-dispatch → repeat until GREEN. **Proceeding past ANY gate (start coding, advance to the next phase, tag, ship) with an open Critical or Important finding is prohibited.** This is a hard gate at every level: brainstorm spec, plan-doc, and per-phase execution. (See the two reviewer-loop bullets below for the per-fold / per-phase mechanics this gate builds on.)
- Reference implementation in `crates/mnemonic-toolkit/`.
- Design artifacts in `design/`: `BRAINSTORM_*`, `SPEC_*`, `IMPLEMENTATION_PLAN_*`, `FOLLOWUPS.md`.
- Per-phase opus reviews persist to `design/agent-reports/`.
- Per-phase TDD: tests written before impl. Per-phase reviewer-loop until 0 critical / 0 important.
- **Plan-doc + spec citations are grep-verified at write time.** `FOLLOWUPS.md` entries cite source line numbers but those are snapshots from when the entry was filed — they decay every merge. When lifting a citation from `FOLLOWUPS.md` into a plan-doc, brainstorm spec, or SPEC body, re-grep against current `origin/master` source (`git show origin/master:<path> | grep -n <pattern>`) and use the live line numbers. Document the source SHA in the spec for future readers.
- **Reviewer-loop continues after every fold.** "Per-phase reviewer-loop until 0 critical / 0 important" applies to plan-docs and brainstorm specs too, not just per-phase execution. After folding architect findings, re-dispatch the architect. Stopping after R0 → fold → done is insufficient because folds themselves can introduce drift.
- **New `enum ToolkitError` variants + new exhaustive `match self { ... }` blocks use alphabetical-by-variant-name ordering.** Drift across concurrent feature PRs (9+ new variants in v0.26.0 cycle) is otherwise a guaranteed merge-conflict generator; alphabetical order makes resolution mechanical. Pre-v0.27.2 variants in `error.rs::ToolkitError` + its `Display` / `exit_code` / `kind` match blocks are not yet sorted — retroactive sort tracked as `error-rs-retroactive-alphabetical-sort` in `design/FOLLOWUPS.md`.
- **Per-phase architect-review agent outputs persist verbatim to `design/agent-reports/<cycle>-phase-N-<round>-review.md` BEFORE the fold-and-commit step.** Transcript-only review text is unrecoverable from outside the session. Future cycles MUST persist the full review-agent output (Critical / Important / Minor sections + file/line citations) before applying folds, so the audit trail survives session boundaries. Compare-cost cycle (v0.26.0 C2-C5) reviews were lost via this gap.
- Stage paths explicitly (no `git add -A`).
- Multi-instance coordination playbook: see `design/PLAN_v0_26_0_three_way_merge.md` (integration-branch model + per-instance branch ownership).
