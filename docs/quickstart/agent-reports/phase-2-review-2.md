# Phase 2 — code-quality review, round 1

**Date:** 2026-05-08
**Branch:** `quickstart/v0_1` at commit `fdea9ee`
**Reviewer:** feature-dev:code-reviewer (code-quality focus)
**Verdict:** MUST_FIX

---

## Critical

### C-1: Cross-chapter `#anchor` links are brittle (confidence: 85)

7 cross-chapter links across `23-bundle.md` (×2: lines 32, 102) and `26-recover.md` (×5: lines 3, 10, 90, 97, 101) use bare `#anchor` form. Lychee passes today (pandoc-to-single-PDF concatenation makes intra-document anchors resolve), so these are not currently broken — but the form is brittle if the build ever switches to per-chapter rendering (mdBook, Hugo, etc.). Convert to relative file paths for robustness and consistency with Phase 1 (which uses text-only "Onward:" prose without hyperlinks).

### C-2: ch25 flowchart and prose contradict each other on re-decode timing (confidence: 82)

`25-stamp.md` flowchart (lines 9-23) sequences: stamp ms1 → stamp mk1 → stamp md1 → "Re-decode each plate via mnemonic verify-bundle" (one step, after all three).

Prose (lines 39-45) says: "After each plate is stamped... run `mnemonic verify-bundle` with the just-stamped strings" — implying per-plate verification.

`mnemonic verify-bundle` requires all three cards as inputs; running it after only one plate has no clean affordance. Align flowchart and prose on "verify after all three plates are stamped, with the steel-read strings."

---

## Important

### I-1: ch22 line 3 — throat-clearing opener (confidence: 72)

`22-generate-entropy.md` line 3: "This chapter is short because the imperative is short:" — meta-observation gains nothing for a newcomer. Cut.

---

## Passing checks

**Technical accuracy — all clean:**
- ch21 install command (`cargo install --locked --git ...`) matches `mnemonic-toolkit` README.
- ch22 "ten BIP-39 wordlists" — CLI help `--language` lists exactly 10. Correct.
- ch22 entropy lengths (12/15/18/21/24-word) — standard BIP-39 values. Correct.
- ch23 `--template bip84` = `m/84'/0'/0'` single-sig native SegWit — confirmed against CLI help and transcript origin path.
- ch23 `--slot @0.phrase=` grammar — matches CLI help `@N.<subkey>=<value>`. Correct.
- ch23 output transcript walkthrough — byte-identical match to `transcripts/22-first-bundle.out`.
- ch24 verify-bundle command — matches `transcripts/23-verify.cmd` exactly.
- ch24 check-name interpretation (`*_decode` = BCH + parse, `*_match` = semantic content) — accurate.
- ch26 Steps 1/2/3 commands — verified against transcript files in spec review.

**Newcomer voice:** Consistent across all 6 chapters. Jargon (xpub, BCH, BIP-39, policy_id_stub) introduced inline or cross-referenced to ch13. The DANGER box in ch22 and Reminder blockquotes in ch23/24/26 frame the test-vector hazard clearly.

**Forward-pointer chain:** ch21→22→23→24→25→26→Part III/IV. All six "Onward:" closers present.

**Markdown formatting:** Heading levels consistent. Tables well-formed. Code blocks use `sh` / `text` / `mermaid` correctly throughout.

---

## Required before APPROVED

C-1 (7 brittle anchor links across 2 files) and C-2 (flowchart/prose contradiction in ch25). I-1 is a suggestion.
