# v0.36.1 — End-of-cycle architect review (opus) — MANDATORY pre-tag gate

**Date:** 2026-05-24
**Cycle:** v0.36.1 `silent-payment` `--passphrase`/`--passphrase-stdin` + `--change-address`
**Reviewer:** opus (feature-dev:code-reviewer), end-of-cycle (agentId a8ba01ea30a8fba5a)
**Scope:** whole-cycle diff `origin/master..HEAD` + live source.

## Critical
None.
## Important
None.
## Minor
- **M1 — `CANONICAL_FLAG_ROWS` argv-audit table stale (PRE-EXISTING, fails no test).** `tests/lint_argv_secret_flags.rs:47` self-describes as the canonical secret-argv enumeration but froze at v0.13.0 — already omits nostr `--secret` + silent-payment `--secret` + now `--passphrase`/`--passphrase-stdin`. The gate is a curated checklist (`assert_eq!(len, 28)` + per-row evidence) with NO clap-closure, so decay is silent. Enforced projections (`flag_is_secret` `secrets.rs:52-54` + runtime `secret_in_argv_warning` `silent_payment.rs:210`) ARE correct. Not introduced this cycle; does not block. → FILED FOLLOWUP `lint-argv-secret-flags-canonical-table-rebuild-from-clap`.

## Verification summary (all 8 gate items GREEN)
1. Code: passphrase threads both seed paths (`silent_payment.rs:116,150`, to_master `:144,:158`); xprv warn-and-ignore `:122-131`; empty-default byte-preserved (test `no_pass==BASE_SP`); dual-stdin guard hoisted `:183-188`; read_stdin_passphrase not trim; change-address distinct+additive `:238-243`, JSON warning iff change_address `:244-245`; borrow/move across arms clean; `--label 0` still refused `:176`. No new issue.
2. Version/release: Cargo.toml:3=0.36.1; Cargo.lock:694=0.36.1; install.sh:32 self-pin v0.36.1; CHANGELOG [0.36.1] accurate; SemVer PATCH correct.
3. FOLLOWUPs: both resolved (FOLLOWUPS.md:3097,3107).
4. Manual: 41-mnemonic.md:2079-2080,2084 document all 3 flags; flag-coverage passes; cli-subcommands.list unchanged; no gui-schema count change (stays 25).
5. Secret taxonomy: passphrase flags already in flag_is_secret; --change-address correctly non-secret; no projection gap.
6. GUI lockstep (separate Phase 3): toolkit gui-schema emits secret:true for passphrase flags automatically (gui_schema.rs:1170 per-arg flag_is_secret); GUI repo mirror ships next.
7. Tests: 2358 pass; new tests meaningful (non-tautological); clippy clean; manual lint 6/6.
8. Clean-tag blockers: none (no dbg!/todo!/FIXME/dead code; citations accurate; FOLLOWUPs filed).

VERDICT: GREEN (0C/0I)

## Controller note
GREEN → gate satisfied. The single Minor is pre-existing audit-table decay (FOLLOWUP filed); enforced secret projections correct. Cleared to tag/ship v0.36.1 + GUI lockstep.
