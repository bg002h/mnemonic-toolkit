# Phase 3 — code-quality review, round 1

**Date:** 2026-05-08
**Branch:** `quickstart/v0_1` at commit `fadc627`
**Reviewer:** feature-dev:code-reviewer (code-quality focus)
**Verdict:** MUST_FIX (one technical error; one suggestion)

---

## Critical

### C-1: ch 33 line 46 + line 53 — "any cosigner's seed" claim is wrong for mk1

**Confidence: 90.** The recovery-table row for "One blue mk1 plate" reads: "Re-derive from any cosigner's seed via `mnemonic convert --from phrase=… --to mk1`; the mk1 carries no secret." Each mk1 card encodes one specific cosigner's xpub + origin. To re-derive cosigner N's mk1, you need cosigner N's seed, not any arbitrary cosigner's seed. The introductory framing at line 45-46 carries the same confusion: "Public material (mk1, md1) is derivable from any cosigner's seed plus the wallet policy" — true for md1 (since the policy template + xpubs is symmetric and the public xpubs are known) but false for mk1 (each mk1 is bound to one specific cosigner's xpub).

Using the wrong seed produces the wrong mk1 silently — `verify-bundle` would catch it later, but a newcomer reading the table might not realize any seed won't do.

Fix: change "any cosigner's seed" → "that cosigner's own seed" in the table cell, and rephrase the introductory paragraph to call out the mk1↔seed binding.

---

## Suggestions

### S-1: ch 31 line 24 — parenthetical "(geographic separation of `ms1`/`mk1`/`md1`)" slightly off-register

**Confidence: 50** — not blocking; flagged for writer awareness.

The sentence attributes *geographic separation* to the stamping ceremony. The ceremony chapter (`25-stamp.md`) is about encode/verify discipline; geographic separation is a storage decision the ceremony enables but doesn't itself enforce. Could be tightened to "durability and separation" or dropped.

---

## Passing checks

**ch 31 framing.** "K=N loses recovery property" and "1-of-N defeats multisig" are technically correct at the right abstraction level for a newcomer. `1 ≤ K ≤ N ≤ 16` cap matches CLI help. No expert drift.

**ch 32 command flags.** All four flags (`--template wsh-sortedmulti`, `--threshold 2`, `--slot @N.phrase=…`, `--self-check`) appear verbatim in `mnemonic-bundle.txt`. Descriptor expansion `wsh(sortedmulti(2,@0,@1,@2))` is consistent. BIP-67 attribution accurate.

**ch 32 DANGER box.** Body is clear, newcomer-appropriate. Forward pointer to `22-generate-entropy.md` correct. No throat-clearing.

**ch 32 mermaid.** Input→output flow unambiguous. Node labels match prose counts (3 ms1, 3 mk1, 1 md1). `classDef` color coding aligns with red/blue/green plate language in ch 33.

**ch 33 recovery table — remaining rows.** "Two cosigners' ms1s + md1 readable → spendable", "all three ms1s lost → bricked", "Only one cosigner's ms1 + md1 → watch-only only" all match `35-recovery-paths.md` exactly.

**Markdown.** H1/H2 throughout; tables well-formed; code blocks use `sh`/`mermaid`/`text` correctly; relative cross-chapter links follow established pattern.

**Pedagogical flow.** ch 31→32→33 builds concept→procedure→consequence without gaps. Per-cosigner plate-set table in ch 33 grounds "five plates" before recovery table demands subset-reasoning. "Onward:" pattern correct at chapter boundaries.

---

## Required before APPROVED

C-1 only. S-1 is optional polish.
