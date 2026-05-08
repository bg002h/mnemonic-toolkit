# Phase 1 — code-quality review, round 1

**Date:** 2026-05-08
**Branch:** `quickstart/v0_1` at commit `45c830c`
**Reviewer:** feature-dev:code-reviewer (code-quality focus)
**Verdict:** SUGGESTIONS_ONLY

---

## Technical accuracy

All BIP-39, BIP-32, descriptor, BIP-388, and BIP-93 claims checked against `docs/manual/src/60-appendices/{62-bip39-primer,63-bip32-primer,64-descriptors-primer,65-bch-codex-primer}.md` and `README.md`. No factual errors found.

**S-1 (confidence 82): ch 12 BIP-39 simplification obscures checksum structure.** `12-bitcoin-in-30-seconds.md:12-14`: "you pick 128 or 256 random bits, append a short checksum." The word "append" implies the checksum trails the entropy, which is correct — but "slice the result into 11-bit chunks" loses the critical ordering: the checksum is computed from the entropy *before* appending. A newcomer reading this out of order might think you pick random bits and then arbitrarily append something. The BIP-39 primer (manual ch 62) states the sequence clearly: pick → SHA-256 → take first `bits/32` bits as checksum → concatenate → slice. Suggested fix: "you pick 128 or 256 random bits, compute a short checksum of them, append it, and slice the combined bitstream into 11-bit chunks."

**Mermaid labels:** ch 11 flowchart labels match README's description of `ms-codec`/`mk-codec`/`md-codec` surface. No label error.

**policy_id_stub terminology:** `13-the-three-cards.md:54-58` correctly names the term, accurately defines it as "a 4-byte hash derived from the wallet policy", and correctly names `mnemonic verify-bundle` as the verifier. README corroborates.

**md1 carries one bound xpub:** ch 13 line 19 consistent with `64-descriptors-primer.md:72`.

---

## Newcomer voice / jargon

**S-2 (confidence 80): "xpub" used before brief intro in ch 11.** `11-what-is-this.md:23`: "a **key card** (`mk1`) carrying a public key (an *xpub*)". The parenthetical introduces the term here; ch 12 is the first full explanation. The one-word parenthetical is enough for a first encounter and ch 12 immediately follows. No blocker — no action needed.

All other jargon (seed phrase, cosigner, BCH, BIP-32 path, master fingerprint) introduced before or at first use across all four files. Chain is clean.

---

## Prose quality

**S-3 (confidence 80): "synthesises" — British spelling, inconsistent with surrounding prose.** `11-what-is-this.md:29`: "synthesises". File is otherwise American spelling. Manual uses "synthesize" throughout. Change to "synthesizes".

**No throat-clearing or hedge words found.** All four chapters open directly on substance. Sentence lengths short to medium. No paragraph exceeds 6 lines. "Onward:" forward-pointer pattern consistent across all four files (00:55, 11:71, 12:62, 13:60).

ch 00 "reading order" section is terse and direct; no wordiness.

---

## Clarity / pedagogy

**S-4 (confidence 80): ch 13 "policy ID stub" paragraph needs one step of grounding.** `13-the-three-cards.md:53-58`: "a small fingerprint called the **policy ID stub**: a 4-byte hash derived from the wallet policy that every `mk1` and `md1` card in a coherent bundle carries identically." A newcomer who just learned in ch 12 that md1 = template + bound xpub and mk1 = xpub + origin may not see how both cards can carry "the same hash of the wallet policy" when mk1 doesn't appear to contain the full policy. Suggested fix adds an "encode time" framing: "…a 4-byte hash of the wallet policy that each card carries at encode time — so mixing cards from different wallets is caught immediately."

---

## Markdown / formatting

No heading-level inconsistencies found. Tables in ch 13 are well-formed. Line lengths within reason for 80-column soft limit. No bare URLs in prose. No issues.

---

## Summary

Four suggestions, no blocking issues. S-1 (BIP-39 checksum ordering) and S-4 (policy ID stub newcomer confusion) are the most pedagogically meaningful — each is a one-sentence fix. S-2 is a no-op. S-3 is cosmetic.

**Verdict: SUGGESTIONS_ONLY**
