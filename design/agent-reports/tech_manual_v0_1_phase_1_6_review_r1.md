# Phase 1.6 final reviewer report — whole-cut review

| Field | Value |
|---|---|
| Phase | tech-manual-v0.1 Phase 1.6.2 (final whole-cut reviewer round, pre-tag) |
| Commit under review | post-`f9f0182` working tree + Phase 1.6.1 in-flight index additions |
| Reviewer | `feature-dev:code-reviewer` |
| Round | r1 (folded inline — no r2 dispatched) |
| Reviewer verdict | 0 Critical / 1 Important / 1 Low / 1 Nit |

## SPEC §7 v0.1 acceptance criteria — verdict

| Criterion | Target (v0.1) | Actual | Status |
|---|---|---|---|
| A4 — Glossary entries | ≥30 | 31 | PASS |
| A5 — Index entries | ≥100 | 110 | PASS |
| A6 — TOC | auto-generated, covers every Part/chapter | pandoc produces TOC; verified at lint + PDF rebuild | PASS |
| A8 — Worked-example transcripts verified | all green via `verify-examples.sh` | 6/6 green against HEAD `md`/`mk`/`ms`/`mnemonic` binaries | PASS |
| A10 — PDF length | ≥40pp (v0.1 soft floor) | 100pp | PASS |

A1 / A2 / A3 / A7 / A9 / A11 are out of scope at v0.1 (per IMPLEMENTATION_PLAN §1.6.2 framing).

## Findings (folded inline at fold-commit per `zero_followups_from_release_cycles`)

### Critical

None.

### Important — folded inline

- **I-1.** `MalformedPayloadPadding` variant fabricated for md1: listed in md1 troubleshooting table (`65-troubleshooting.md:20–22`) and referenced in §II.1 canonicality rule 5 (`21-md1-wire-format.md:231`), but the variant does **not exist** in `md-codec/src/error.rs`. The chunk-set / TLV-rollback prose at §II.1 traces back to v0.1's source where md-codec's actual behavior is: tolerates trailing ≤7 zero pad bits via TLV-rollback (`md-codec/src/tlv.rs:217`); non-zero pad bits surface as `MalformedHeader { detail }` or `BitStreamTruncated` depending on how the TLV parser interprets them. **Fold:** rewrote §II.1 canonicality rule 5 to accurately describe md-codec's behavior (no dedicated pad-bit-rejection variant; non-zero pad bits → `MalformedHeader` / `BitStreamTruncated`; zero pad bits tolerated by rollback-as-padding). Removed the md1 `MalformedPayloadPadding` row from the troubleshooting table; replaced with a paragraph explaining the md-codec behavior pattern. The mk1 row at `65-troubleshooting.md:37` is unaffected (mk-codec genuinely has `MalformedPayloadPadding` at `mk-codec/src/error.rs:69`). Confidence: 91 (reviewer).

### Low — folded inline

- **L-2.** Reviewer claim: `\index{BIP-39 mnemonic}` marker at §II.3 line 188; term appears earlier in §I.1. **Verification:** `grep -rn 'BIP-39 mnemonic' src/10-foundations/` returns zero matches. §I.1 uses "BIP-39 phrase" (different term). §II.3 line 165 + line 176 + line 188 are the only occurrences of "BIP-39 mnemonic" in the manual; line 188's "BIP-39 wordlist language" heading is the first *definitional* treatment of the entropy→mnemonic conversion. The current placement IS at first definitional use. Reviewer's L-2 is a misreading of the source; **no fold needed**. Confidence 82 (reviewer); rebuttal confidence 90 (verification).

### Nit — folded inline (deferred to existing FOLLOWUP)

- **N-3.** Bibliography BIP-93 entry lacks author attribution, inconsistent with other BIP entries. **Decision:** Phase 1.5's r1 already addressed this — couldn't verify canonical BIP-93 author list against local sources, dropped attribution rather than fabricate, filed `bibliography-bip-author-canonical-verification` FOLLOWUP (tier `tech-manual-v1.0-nice-to-have`). The reviewer's r1 (Phase 1.5) suggested "Russell O'Connor + Andrew Poelstra"; r1 (Phase 1.6) suggests "Leon Olsson Curr (Pearlwort Sneed), Andrew Poelstra" (a different attribution). Inconsistency across rounds vindicates the deferral. **Fold:** updated the bibliography prose to make the deliberate omission explicit, citing the FOLLOWUP. No new FOLLOWUP filed (existing one carries the resolution). Confidence 80 (reviewer); rebuttal confidence 90 (the codex32 paper authorship ≠ BIP-93 author header; deferral is correct).

## Cycle-exit verification (1.6.1)

- `cargo test --workspace --all-features` — 527 passed / 0 failed / 2 ignored.
- `make -C docs/technical-manual lint` — 6/6 green.
- `make -C docs/technical-manual pdf` — 100 pages; in SPEC §6 v0.1 bracket [40, 110].
- PDF reproducibility check: `rm -rf build && SOURCE_DATE_EPOCH=1746921600 make pdf` byte-identical across clean rebuilds.
- `tests/verify-examples.sh` (with the 4 HEAD release binaries) — 6/6 transcripts pass.

## Decision: no r2

Important I-1 is folded inline with explicit rewrite of the affected canonicality rule + table row + prose paragraph. Low L-2 is a reviewer misreading (current placement is correct). Nit N-3 is deferred to the existing FOLLOWUP (the Phase 1.5 r1 decision is documented and stands; the bibliography prose now makes the deliberate omission explicit). The tag-time `zero_followups_from_release_cycles` rule is respected — no NEW FOLLOWUPs filed at this commit; the one existing pre-tag FOLLOWUP (`bibliography-bip-author-canonical-verification`) was filed mid-cycle at Phase 1.5.

0C/0I achieved. Phase 1.6.2 closes. Ready for 1.6.4 CHANGELOG + 1.6.5 tag.
