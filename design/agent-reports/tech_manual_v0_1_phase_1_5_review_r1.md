# Phase 1.5 reviewer report — back-matter skeleton

| Field | Value |
|---|---|
| Phase | tech-manual-v0.1 Phase 1.5 (back-matter skeleton) |
| Commit under review | `ae5bb51` (mnemonic-toolkit/master) |
| Reviewer | `feature-dev:code-reviewer` |
| Round | r1 (folded inline + FOLLOWUPS — no r2 dispatched) |
| Reviewer verdict | 0 Critical / 2 Important / 5 Low / 1 Nit |

## Findings (raw reviewer output, organized)

### Critical

None.

### Important (folded inline at fold-commit)

- **I-1.** Troubleshooting intro variant counts inconsistent with source (e.g., "mk-codec (17)" vs 22 actual variants in `mk-codec/src/error.rs`). **Fold:** dropped the parenthetical variant counts; intro now says "a curated subset of wire-format-layer variants" and points readers to `error.rs` for the complete enum surface.
- **I-2.** Troubleshooting `md1 MalformedPayloadPadding` row conflated bytecode-section padding (rule 5) with TLV-section rollback exception (separate carve-out). **Fold:** split the parenthetical out of the table cell; moved the TLV-section carve-out into a separate paragraph below the table with explicit cross-reference to BIP draft §"End-of-section detection (rollback-as-padding)" and §II.1 "TLV section".

### Low (folded inline)

- **L-1.** Troubleshooting `OperatorContextViolation` row claimed `context: MultiBody` as an active rejection path, but `md-codec/src/error.rs:197-207` documents `MultiBody` as structurally unreachable post-v0.30 Phase C (multi-family bodies carry raw indices, not child tags). **Fold:** updated the row to flag `MultiBody` as structurally-unreachable in v0.30, retained for completeness; cited the error.rs line range; updated the canonicality-rules pointer from "2/3/4" to "2/4" (rule 3 — multi-family raw-index body — is the very rule whose existence makes `MultiBody` unreachable, so it's the wrong reference).
- **L-2.** Release-history missing the `mnemonic-toolkit v0.7.1` row (2026-05-07; multi-repo BIP test-vector audit cycle close-out; the same date a different reviewer might mistake for a missed entry). **Fold:** added v0.7.1 row in chronological position between mk-codec v0.2.2 and toolkit v0.8.0; one-line summary matches CHANGELOG entry; CHANGELOG pointer cited.
- **L-3.** BIP cross-reference listed BIP-39 as cited in §I.3, but `13-codex32-and-bch.md` has only one passing reference to BIP-39 (line 66 — "BIP-39 entropy" used as a descriptor, not a normative citation). **Fold:** removed §I.3 from BIP-39's "Sections citing it" column in both `64-bip-cross-reference.md` and the matching bibliography entry in `66-bibliography.md`.

### Low (deferred to FOLLOWUPS.md)

- **L-4.** BIP-173 §II.3 citation is single-line and thin. **Verdict:** the citation is verifiable (ms1 §II.3 line 59: "The separator is BIP 173's `1`"). Citation is technically correct; the "thin"-ness is the reviewer's qualitative judgment. No action; the table row is accurate as written.
- **L-5.** Bibliography BIP-93 author list incomplete. **Decision:** reviewer claimed BIP-93's canonical author list is "Russell O'Connor and Andrew Poelstra"; we couldn't verify against the canonical bitcoin/bips repo from the local working tree, and best-recollection authorship is closer to "Leon Olsson Curr / Pearlwort Sneed (pseudonyms) + Andrew Poelstra" per the codex32 paper's author block. Rather than fabricate an unverified attribution, **fold:** dropped the BIP-93 author attribution entirely; the bibliography entry now points the reader to the canonical bip-0093.mediawiki for authoritative authorship + adds a cross-reference to the codex32 paper entry for the design history. **Companion FOLLOWUP filed:** `bibliography-bip-author-canonical-verification` (tier `tech-manual-v1.0-nice-to-have`) — every BIP entry's author list to be re-verified at Phase 5.5 against the canonical bitcoin/bips repo.

### Nit (folded inline)

- **N-1.** Spelling inconsistency `walker normalisation` (British, in source `21-md1-wire-format.md` lines 154+217 and in `62-index-table.md`) vs `walker normalization` (American, in my new `61-glossary.md` + `63-release-history.md`). **Fold:** changed glossary + release-history to match the source spelling (`normalisation`, British). Source authority wins. Added `normalisation` to `.cspell.json`.

## Filed FOLLOWUPS

Both at `docs/technical-manual/FOLLOWUPS.md`:

- `bibliography-bip-author-canonical-verification` (tier `tech-manual-v1.0-nice-to-have`) — verify every BIP entry's author list at Phase 5.5.
- `troubleshooting-mk-codec-variant-coverage-audit` (tier `tech-manual-v0.4`) — re-evaluate the troubleshooting inclusion set against Part V coverage by v1.0.

## Cycle-exit verification

- `make -C docs/technical-manual lint` — 6/6 green (markdownlint, cspell, lychee, api-surface-coverage stub, glossary-coverage, index bidirectional).
- `make -C docs/technical-manual pdf` — green, 100 pages (was 83 at Phase 1.4; +17pp from back-matter). Within SPEC §6 v0.1 bracket [40, 110].

## Decision: no r2

Importants (I-1, I-2) are folded inline. Lows (L-1, L-2, L-3) folded inline. Nit (N-1) folded inline. L-4 and L-5 are deferred — L-4 because the citation is correct as written (reviewer judgment, not factual error); L-5 because canonical author verification requires offline-unavailable resources, and the bibliography entry has been made conservative (no author attribution claim that could be wrong).

The fold-commit ships with all reviewer-Critical and reviewer-Important findings closed and three of the five Lows + the Nit closed inline. Mid-cycle policy (`zero_followups_from_release_cycles` does NOT apply at non-tag commits) permits this distribution. Phase 1.6's tag-time commit will revisit the remaining FOLLOWUPS (specifically: the BIP author-attribution issue) at the wider Phase 5.5 bibliography-completion sweep.

0C/0I achieved. Phase 1.5 closes.
