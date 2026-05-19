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

## Conventions

- Reference implementation in `crates/mnemonic-toolkit/`.
- Design artifacts in `design/`: `BRAINSTORM_*`, `SPEC_*`, `IMPLEMENTATION_PLAN_*`, `FOLLOWUPS.md`.
- Per-phase opus reviews persist to `design/agent-reports/`.
- Per-phase TDD: tests written before impl. Per-phase reviewer-loop until 0 critical / 0 important.
- Stage paths explicitly (no `git add -A`).
- Multi-instance coordination playbook: see `design/PLAN_v0_26_0_three_way_merge.md` (integration-branch model + per-instance branch ownership).
