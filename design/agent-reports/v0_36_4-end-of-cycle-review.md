# v0.36.4 — End-of-cycle architect review (opus) — MANDATORY pre-tag gate

**Date:** 2026-05-24
**Cycle:** v0.36.4 pin-staleness PATCH (config/CI/installer only)
**Reviewer:** opus (feature-dev:code-reviewer), end-of-cycle (agentId a58c2af6834b9cbc1)
**Scope:** whole-cycle diff origin/master..HEAD (12 files) + live source.

## Critical
None.
## Important
None.
## Minor (BOTH FOLDED — same pin-staleness class)
- **M1 (FOLDED):** `crates/mnemonic-toolkit/README.md:28` install example pinned `mnemonic-toolkit-v0.36.3` (a literal I added in v0.36.3; the guard checks the marker, not this prose literal → would decay every release). FOLDED: replaced with a non-decaying `mnemonic-toolkit-vX.Y.Z` + releases-page link.
- **M2 (FOLDED):** `docs/manual/src/20-quickstart/21-install.md:38` pinned `mk-cli-v0.2.0` (live v0.4.2; inconsistent with its untagged md/ms siblings). FOLDED: dropped the stale `--tag` to match siblings (unpinned latest). Manual lint 6/6 still GREEN.

## Verification summary (all 6 gate items GREEN)
1. Pin bumps correct+complete: manual.yml mk v0.4.2/md v0.6.1/ms v0.4.1; quickstart.yml:71 mk v0.4.2; install.sh:44 gui v0.21.1; install.sh:32 v0.36.4. All target tags REAL (no cargo-install break). No stale non-mnemonic pin left in any workflow (manual-gui.yml runtime-derives from pinned-upstream.toml, intentionally locked). install.sh siblings already-current, untouched.
2. README-marker lockstep: both markers → 0.36.4 (only 2 occurrences); guard passes (== CARGO_PKG_VERSION).
3. Version/release: Cargo.toml/lock 0.36.4; install.sh:32 v0.36.4; CHANGELOG [0.36.4] accurate; PATCH, no .rs/test-logic change, no GUI lockstep.
4. FOLLOWUPs: pin-staleness resolved (v0.36.4); export-wallet-from-import-json-template-format-reemit filed (+ multisig-ambiguity R0 note); prose-gate updated with coupling; canonical slug used.
5. Tests/lint: README guard GREEN; rust job runs suite+clippy (no .rs change); manual lint 6/6 GREEN post-M2-fold.
6. Clean-tag: post-fold, ZERO stale version literals remain in install paths/workflows (v0.36.3 / mk-cli-v0.2.0 / mnemonic-gui-v0.10.0 all gone).

VERDICT: GREEN (0C/0I)

## Controller note
GREEN → gate satisfied. Both end-of-cycle Minors (M1 crate-README literal, M2 manual quickstart pin) FOLDED — they were the same pin-staleness class this cycle closes; folding them makes the close complete (zero stale literals remain). Cleared to tag/ship v0.36.4 (toolkit-only; no GUI cycle).
