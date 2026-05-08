# Phase 1 — feature-dev:code-architect review, round 1

**Date:** 2026-05-07
**Branch:** `manual/v0_1` (commit `e83f8d4`, prior to round-1 fixes which landed in `19846e4`)
**Verdict:** Phase 1 close but not converged. 2 critical + 4 important + 3 nits. All resolved in commit `19846e4`. Round 2 confirmation pending.

## Critical

### C1 — Trapdoor tests only one direction of the bidirectional check

`phase-1-lint-trapdoor.md` Step 2 removes a row from `69-index-table.md` while preserving the `\index{}` source marker — exercises "source term missing from table." The inverse direction — row present in the table but `\index{}` marker absent from source — is never deliberately triggered. The script implements both loops (lint.sh lines 122–133), but only one is proven non-vacuous.

**Remedy applied:** Added Step 2b to the trapdoor that removes only the source-side `\index{m-format constellation}` marker from `00-frontmatter.md` (preserving the table row). Confirmed lint FAILS with the inverse diagnostic `... 69-index-table.md term 'm-format constellation' has no matching \index{} marker in src/`. Restored. Both loops now proven non-vacuous.

### C2 — Anchor slugs for appendices B–E (architect missed Phase 0 stubs exist)

The architect flagged that `00-frontmatter.md` and `61-glossary.md` link to `#appendix-b--bip-39-entropy-primer` etc. but the actual appendix files don't exist. Re-verification: Phase 0 commit `1f3d2af` already created stub files for `62-bip39-primer.md` through `65-bch-codex-primer.md`, each with the correct H1 heading (`# Appendix B — BIP-39 entropy primer`). Pandoc's slugifier produces `appendix-b--bip-39-entropy-primer` from that heading (em-dash + space → `--`). Anchors will resolve correctly in the concatenated md and PDF outputs.

**Remedy:** None needed. Mark RESOLVED on inspection (architect oversight on Phase 0 stub presence).

## Important

### I1 — `\index{}` convention has no author-facing documentation

The only description of how to place `\index{}` markers is in lint script comments and the trapdoor report. Future Phase 2–6 chapter authors won't know the placement, formatting, or mirror duty.

**Remedy applied:** Authored `docs/manual/AUTHORING.md` covering: heading levels (H1 = chapter), two-track `:::primer` / `:::danger` convention, inline `\index{TERM}` placement (immediately after first definitional use, same line, no newline) and `69-index-table.md` mirror duty, canonical-test-seed DANGER policy, glossary discipline, mermaid blocks, voice/length, pre-commit lint expectations.

### I2 — Glossary entry `card` omits the fourth format

`card` entry: "A single engravable string emitted by one of the four codecs: ms1, mk1, or md1." Three listed, "four" claimed.

**Remedy applied:** Changed to "one of the three card codecs"; added explanatory clause in `bundle` entry that `mnemonic-toolkit` is the integration *layer*, not a fourth card codec.

### I3 — `bundle` entry says `policy_id_stub` carried on mk1 only

Glossary `bundle` entry omitted "computable from each md1 card" — meaningful asymmetry that authors writing the recovery workflow (Phase 3) would draw on incorrectly.

**Remedy applied:** Mirrored the README phrasing verbatim: "carried on each mk1 card and is computable from each md1 card."

### I4 — `mnemonic` entry doesn't mention `--slot` uniform input shape

Will require Phase 2-3 authors to re-explain `--slot @N.<subkey>=<value>` from scratch.

**Remedy applied:** Added one sentence to `mnemonic` glossary entry naming the uniform input shape.

## Minor / nits

### N1 — `codex32` entry says "checksumed" (typo)

**Remedy applied:** Fixed to "checksummed".

### N2 — Appendix E heading slug verification

Architect flagged "verify the heading in `65-bch-codex-primer.md` (once written) actually produces that exact slug." Stub heading is `# Appendix E — codex32 / BCH / m-codec error correction` — pandoc slug = `appendix-e--codex32--bch--m-codec-error-correction`. Glossary link uses `appendix-e--codex32--bch--m-codec-error-correction`. Match confirmed.

**Remedy:** None needed. RESOLVED on inspection.

### N3 — `multi_a` entry says "post-BIP-386 / BIP-388" (ambiguous)

**Remedy applied:** Clarified to "BIP-386 (`multi_a` descriptor) and exchanged via BIP-388 (wallet policy)".

## Convergence assessment

Phase 1 round-1 fixes shipped in commit `19846e4`. All 2 critical and 4 important findings have concrete remedies in the commit. **Round 2 confirmation dispatch is recommended** as the resume session's first action — cheap (~30K tokens), verifies the trapdoor's inverse-direction step actually exercises the new path, and sanity-checks the AUTHORING.md conventions are author-actionable.
