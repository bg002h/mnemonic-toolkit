# Phase 2 — feature-dev:code-reviewer review, round 1

**Date:** 2026-05-07
**Branch:** `manual/v0_1` (commit `e6c8191`)
**Verdict:** Not converged. 0 critical / 1 important / 3 nits.

## Critical

None.

## Important

### I1 — Multisig signpost conflates wallet threshold (supported) with codex32 secret-share splitting (ms-codec v0.2 feature)

**File:** `src/10-foundations/13-concept-signposts.md` lines 43–44.
**Confidence:** 100.

Current text:
```
K=N (every cosigner signs) is the toolkit's v0.1
default; K<N (threshold) is planned for v0.2.
```

Factually incorrect. The toolkit v0.8 supports K < N wallet multisig: `--threshold K` accepts any value 1..=N. The headline workflow chapter is titled "Multi-source 2-of-3 multisig" (K=2, N=3). The "planned for v0.2" note belongs to *codex32 K-of-N secret-share splitting* (ms-codec v0.2), a distinct feature.

The same error is present in `src/60-appendices/61-glossary.md` `## threshold (K-of-N)`:
```
A multisig parameter: any K of the N cosigners can sign. Toolkit v0.1
ships threshold = N (single-string); K < N is planned for v0.2.
```

**Fix — `13-concept-signposts.md` lines 43–44:** Replace "K=N (every cosigner signs) is the toolkit's v0.1 default; K<N (threshold) is planned for v0.2." with "K is set via `--threshold K`; any value 1..=N is valid."

**Fix — `61-glossary.md` `## threshold (K-of-N)`:** Replace with: "A multisig parameter: any K of the N cosigners can sign. Set via `--threshold K` (any value 1..=N). Note: K-of-N *secret-share splitting* (splitting the ms1 card itself into N shares) is a separate feature planned for ms-codec v0.2."

## Minor / nits

### N1 — Mermaid `BIP-39 seed` node label inconsistent with edge label

**File:** `src/10-foundations/11-welcome.md` line 21.

Edge `ms1 -. "BIP-39 entropy" .-> seed[(BIP-39 seed)]` labels the edge "BIP-39 entropy" but names the target node "BIP-39 seed". In BIP-39 terminology these differ (entropy = pre-PBKDF2 bytes; seed = 64-byte PBKDF2 output). The prose table calls it entropy. Suggested rename: `phrase[(seed phrase)]`.

### N2 — `\index{}` markers inside H2 heading lines (hold-to-observe)

**Files:** `src/10-foundations/13-concept-signposts.md` lines 7, 16, 27, 40, 53.

Section-level index markers are placed inside `##` headings. Pandoc's LaTeX emission may need `\protect\index{}` for headings (moving arguments). PDF currently builds clean per Phase 0 verification, so no action required unless build regresses.

### N3 — `multisig` indexed but no glossary entry

**Files:** `src/60-appendices/69-index-table.md` line 25; `src/60-appendices/61-glossary.md` (no `## multisig` entry).

`multisig` is indexed but not in the glossary. AUTHORING.md requires entries for "every acronym or m-format-specific term"; multisig is general Bitcoin, not m-format-specific. Adding a one-sentence entry would close the gap. Low priority for v0.1.

## Convergence assessment

0 critical / 1 important / 3 nits. Not converged. After applying I1 fixes (two-line edits in two files), round-2 confirmation should be a 1-paragraph spot-check. N1, N3 are deferrable; N2 is hold-to-observe.
