# Phase 1 — spec-compliance review, round 1

**Date:** 2026-05-08
**Branch:** `quickstart/v0_1` at commit `3a689e2`
**Reviewer:** feature-dev:code-reviewer (spec-compliance focus)
**Verdict:** SPEC_COMPLIANT

---

## Task 1.1 — `00-frontmatter.md`

All plan-required elements present: H1 "About this Quick Start", audience paragraph framing ("you've heard of Bitcoin self-custody"), prerequisites (Linux/macOS terminal, shell comfort, ~1 hour), "What you'll have at the end" covering Parts II/III/IV, reading-order note (~90 minutes, top-to-bottom), manual pointer. Forward-pointer "Onward to the foundations." present at line 55. PASS.

## Task 1.2 — `11-what-is-this.md`

H1 "What you're building" matches plan Body column. All 5 required sections present: problem paragraph, answer paragraph, mermaid block, "What this guide covers", forward-pointer. Mermaid block opens with ` ```mermaid` at line 33 — Q4 satisfied. Newcomer-tuned labels ("produces", "the random bits", "a public key", "the spending rule") are appropriate per D3 adaptation allowance; they do not misrepresent the toolkit's actual surface. Forward-pointer at line 71 points to ch 12. PASS.

## Task 1.3 — `12-bitcoin-in-30-seconds.md`

All 5 required sections present: "Seed phrase", "Extended public key (xpub)", "Wallet descriptor", "BIP, what's a BIP?", forward-pointer. Technical accuracy checked against `62-bip39-primer.md`, `63-bip32-primer.md`, `64-descriptors-primer.md`:

- BIP-39: 11-bit chunks indexing a 2048-word list — correct. "128 or 256 random bits" is a simplification of the full set (128/160/192/224/256) but accurate for the two dominant cases; newcomer voice permits this.
- BIP-32: master key → child tree; xpub enables watch-only and cosigner sharing — correct.
- Descriptor: `wpkh([fingerprint/84h/0h/0h]xpub6Cat.../<0;1>/*)` example shape with explanation — correct and consistent with `64-descriptors-primer.md` §anatomy.
- BIP-388: template/bound-key split — correct.
- BIP aside correctly names BIP-93 ("codex32") as the error-correcting alphabet.

Forward-pointer at line 63 points to ch 13. PASS.

## Task 1.4 — `13-the-three-cards.md`

H1 "The three cards: ms1, mk1, md1" matches plan Body column. All required sections present: one-card-per-concept bullet list (ms1/mk1/md1 with BIP-concept mapping), 3-row "What each card answers" table (lifted/adapted from manual ch 11), "Why three cards instead of one" with `policy_id_stub` inline one-sentence primer. Forward-pointer "Onward: install the toolkit..." at line 60–61. PASS.

## cspell.json

Only the `words` array modified (adds `"custodied"`). `import` key and `ignorePaths`/`ignoreRegExpList` are structural entries from the scaffold, not new manual word-list mutations. Compliant with plan Task 1.5 Step 1 restriction. PASS.

## Cross-cutting checks

| Check | Result |
|---|---|
| Forward-pointer chain 00→11→12→13→Part II | Complete (lines 55, 71, 63, 60) |
| Part I file count: 3 numbered chapters + frontmatter | Confirmed (11/12/13 + 00) |
| Q1 newcomer voice (zero Bitcoin prerequisites assumed) | Consistent across all 4 files; all technical terms introduced inline before use |
| Q4 mermaid block in ch 11 source | ` ```mermaid` at line 33 |
| D1 zero-hands-on-Bitcoin assumption | Maintained |
| §5 Part I chapter coverage | Exactly 11, 12, 13 plus 00 |

No spec gaps found. The authored content satisfies Tasks 1.1–1.4 and all applicable §2/§3/§5/§7 criteria for Phase 1.
