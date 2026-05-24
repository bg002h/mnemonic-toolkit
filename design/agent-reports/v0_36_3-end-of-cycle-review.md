# v0.36.3 — End-of-cycle architect review (opus) — MANDATORY pre-tag gate

**Date:** 2026-05-24
**Cycle:** v0.36.3 documentation refresh (README + manual hygiene)
**Reviewer:** opus (feature-dev:code-reviewer), end-of-cycle (agentId ab5ef76ceb0a71167)
**Scope:** whole-cycle diff origin/master..HEAD (14 files) + live source.

## Critical
None.
## Important
None.
## Minor
- **M1 (FOLDED → FOLLOWUP):** `install.sh:44` pins `mnemonic-gui-v0.10.0` (live GUI v0.21.1). Pre-existing, not touched by this docs-cycle (only the `mnemonic` self-pin @:32 bumped); not gated by install-pin-check. Broadened the `manual-yml-and-install-sh-sibling-gui-pin-staleness` FOLLOWUP to cover the GUI pin (higher-impact than the manual.yml sibling drift). Out of scope for this PATCH.

## Verification summary (all 8 gate items GREEN)
- **G1 (both READMEs):** both carry `<!-- toolkit-version: 0.36.3 -->` + v0.36.x status; grep for stale v0.8.0/v0.13.0 = zero (crate README's only `v0.36.3` is the current build-tag); guard test loops both paths, ties to CARGO_PKG_VERSION, non-tautological (was RED pre-marker); install via install.sh (no stale pin); 20-subcommand inventory matches main.rs:63-104 Command variants; intra-README links resolve.
- **G2:** cli-subcommands.list:29-31 adds electrum-decrypt + seedqr encode/decode (correct flag-coverage form).
- **G3/C2:** intro lists all 20 ("Twenty"); all 20 slugs resolve (15 auto + 5 explicit incl xpub-search `{#mnemonic-xpub-search}` @:2557, which would otherwise dangle).
- **G4:** "mirrors v0.13.0"→version-agnostic; "snapshot as of v0.1's tag"→gone; residual v0.13.0 mentions are legit historical citations.
- **Version/release:** Cargo.toml/lock=0.36.3; install.sh:32 v0.36.3; CHANGELOG [0.36.3] docs-only PATCH; NO crate-logic change → NO GUI lockstep.
- **FOLLOWUPs:** manual-prose-command-execution-gate + (broadened) manual-yml-and-install-sh-sibling-gui-pin-staleness filed.
- **Tests/lint:** suite 2361 GREEN (incl new guard); clippy clean; manual lint 6/6 (new intro links + cli-list entries pass). R0 (RED 2C/2I→R1 GREEN) caught 2nd stale README + dangling anchor + phase-order trap pre-impl.
- **Clean-tag:** no leftover stale string, no dangling anchor, guard path correct, no debug.

VERDICT: GREEN (0C/0I)

## Controller note
GREEN → gate satisfied. M1 folded into the broadened pin-staleness FOLLOWUP (the live GUI pin in install.sh is the higher-impact residual — worth a quick follow-on bump, but out of this docs-cycle's R0-approved scope). Cleared to tag/ship v0.36.3 (toolkit-only; no GUI cycle).
