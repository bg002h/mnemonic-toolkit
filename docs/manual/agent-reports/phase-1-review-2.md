# Phase 1 — feature-dev:code-architect review, round 2

**Date:** 2026-05-07
**Branch:** `manual/v0_1` (commit `19846e4`)
**Verdict:** Not converged. 0 critical / 1 important (introduced by round-1) / 0 nits.

## Critical

None.

## Important

### I-R2-1 — AUTHORING.md slug example teaches the wrong form

`AUTHORING.md:27` (inside the `:::primer` fenced-code example) links to:

```
[Appendix B](#appendix-b--bip-39-entropy-primer)
```

Pandoc's `auto_identifiers` algorithm removes punctuation except `-`, `_`, `.`, then collapses runs of spaces to a single `-`. The em-dash `—` is dropped; the spaces flanking it collapse to one hyphen. The correct pandoc slug for `# Appendix B — BIP-39 entropy primer` is `appendix-b-bip-39-entropy-primer` (single hyphen between `b` and `bip-39`), not the double-dash form in AUTHORING.md.

Verified empirically: `pandoc -f markdown -t html` on a fixture with that H1 emits `<h1 id="appendix-b-bip-39-entropy-primer">`.

All production cross-links already use the correct single-dash form:

- `src/00-frontmatter.md:30`
- `src/60-appendices/61-glossary.md:5,31`
- `src/10-foundations/13-concept-signposts.md:5,14`

Only the example in AUTHORING.md carries the wrong double-dash form.

**Origin:** The round-1 C2 resolution and N2 note both asserted "Pandoc's slugifier produces `appendix-b--bip-39-entropy-primer`" (em-dash + space → `--`). That claim is incorrect per pandoc's documented algorithm and per empirical test. The assertion had no effect on source files (which predated round-1 and were already right), but the newly-authored AUTHORING.md inherited the error.

**Risk:** Chapter authors who follow the AUTHORING.md example when writing cross-appendix anchor links will produce double-dash slugs that silently fail to navigate in both the concatenated markdown and PDF outputs.

**Fix:** `AUTHORING.md:27` — change `appendix-b--bip-39-entropy-primer` to `appendix-b-bip-39-entropy-primer`. One character removed.

## Round-1 findings confirmed resolved

| Finding | Status | Evidence |
|---|---|---|
| C1 — trapdoor only one direction | RESOLVED | `phase-1-lint-trapdoor.md` Step 2b present; transcript shows inverse diagnostic; `lint.sh:130–135` implements table-side loop |
| C2 — anchor slugs (false positive) | CONFIRMED FP | Stubs `62–65-*.md` carry correct H1 headings; production cross-links use correct single-dash slugs |
| I1 — no `\index{}` author-facing doc | RESOLVED | `AUTHORING.md` covers placement + mirror duty + DANGER policy + glossary discipline + voice/length |
| I2 — `card` entry omits fourth format | RESOLVED | "one of the three card codecs"; `bundle` clarifies toolkit is integration layer |
| I3 — `bundle` `policy_id_stub` asymmetry | RESOLVED | "carried on each mk1 card and is computable from each md1 card" |
| I4 — `mnemonic` entry missing `--slot` | RESOLVED | Names `--slot @N.<subkey>=<value>` uniform input shape |
| N1 — "checksumed" typo | RESOLVED | `codex32` entry reads "checksummed" |
| N2 — Appendix E slug verification | RESOLVED (claim wrong, files right) | Source uses correct single-dash slug; round-1's double-dash claim was false |
| N3 — `multi_a` BIP citation ambiguous | RESOLVED | "BIP-386 (`multi_a` descriptor) and exchanged via BIP-388 (wallet policy)" |

## Minor / nits

None.

## Convergence assessment

0 critical / 1 important / 0 nits remaining. The single-line fix to `AUTHORING.md:27` closes the round. No round-3 dispatch needed — self-evident correction; declare Phase 1 converged on commit inspection after the fix lands.
