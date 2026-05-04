# CLAUDE.md — mnemonic-toolkit repo notes

This file is auto-loaded by Claude Code when starting a session in this repository.

## What this is

`mnemonic-toolkit` is the top-level integration crate of the **m-format star**:

- [`md-codec`](https://github.com/bg002h/descriptor-mnemonic) — wallet descriptors / policies (`md1`, HRP `md`).
- [`mk-codec`](https://github.com/bg002h/mnemonic-key) — xpubs (`mk1`, HRP `mk`).
- [`ms-codec`](https://github.com/bg002h/mnemonic-secret) — BIP-39 entropy (`ms1`, HRP `ms`).
- **mnemonic-toolkit** (this repo) — takes a seed phrase, emits the three cards as a coherent steel-engravable bundle.

The three sibling codecs ship independently; this toolkit consumes them as library deps (git deps until they hit crates.io in lockstep with v0.1).

## Cross-repo follow-ups

When toolkit work surfaces an action item that affects a sibling codec, mirror an entry in BOTH repos' `design/FOLLOWUPS.md` with cross-citing `Companion:` lines. When the action ships, both entries update in lockstep.

## Conventions

- Reference implementation in `crates/mnemonic-toolkit/`.
- Design artifacts in `design/`: `BRAINSTORM_*`, `SPEC_*`, `IMPLEMENTATION_PLAN_*`, `FOLLOWUPS.md`.
- Per-phase opus reviews persist to `design/agent-reports/`.
- Per-phase TDD: tests written before impl. Per-phase reviewer-loop until 0 critical / 0 important.
- Stage paths explicitly (no `git add -A`).
