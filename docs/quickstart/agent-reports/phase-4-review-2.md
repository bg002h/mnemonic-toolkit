# Phase 4 — code-quality review, round 1

**Date:** 2026-05-08
**Branch:** `quickstart/v0_1` at commit `91fa5f9`
**Reviewer:** feature-dev:code-reviewer (code-quality focus)
**Verdict:** SUGGESTIONS_ONLY

## Technical accuracy — all pass

- **ch 41 commands.** `mnemonic convert --from phrase=… --to xpub --template bip84 --network mainnet` and `mnemonic bundle --slot @0.xpub=…` verified against `mnemonic-convert.txt` and `mnemonic-bundle.txt`. Table descriptor `wpkh(@0/<0;1>/*)` correct for bip84.
- **ch 42 mermaid + prose.** Coordinator receives xpubs only; seeds stay in cosigner subgraphs. Output "4 cards: 3 mk1 + 1 md1, no ms1" correct.
- **ch 42 `--privacy-preserving`** confirmed in `mnemonic-bundle.txt` line 52.
- **ch 42 Step 3 ms1 derivation.** `mnemonic convert --from phrase=… --to ms1` grammar correct.
- **ch 52 item #2 (BCH error position).** Single-error correction is correct; "re-stamp that one character" is the right action.
- **ch 52 item #3 (Bitcoin Core / Sparrow / Specter).** Sparrow / Specter framed as deferral stubs — confirmed by manual `37-wallet-export.md`. `--bitcoin-core-version` default of 25 confirmed.
- **ch 51 forward-pointers.** All 21 linked manual chapter files verified on-disk; zero dead links.

## Suggestions

**S1 — ch 52 item #3 Fix block omits subcommand name (confidence 82).** Lines 54-58. Fix block lists three format bullets without naming `mnemonic export-wallet` as the command they apply to. Items #2 and #4 name `mnemonic convert` explicitly. Suggested: prefix the bullet list with "Re-run `mnemonic export-wallet` with the format matched to your software:".

**S2 — ch 42 Step 3 heading + body both say "separately" (confidence 80).** Lines 94-98. The heading "each cosigner separately derives their own ms1" and the first body sentence "each cosigner separately derives their *own* ms1" repeat the same adverb. Drop "separately" from the body sentence.

## Newcomer voice / pedagogy

ch 41: two-step seed-off-host shape explained before the commands. "Reminder" callout correctly flags test-vector risk.

ch 42 mermaid: classDef colour scheme (red=secret, blue=public, yellow=toolkit) makes the privacy boundary self-evident.

ch 51: reads as a curated guide, not a directory listing. Each bullet has a one-line description of chapter content.

ch 52 "When in doubt" footer: naming `verify-bundle` as the primary diagnostic first is the right priority for a newcomer.

## Markdown / structure

Heading levels consistent (H1 title, H2 sections, no skipped levels). Code blocks use `sh`, `text`, `mermaid` correctly. Tables well-formed. No unclosed fences.

## Summary

No technical errors. S1 + S2 are minor prose polish; neither blocks correctness or comprehension.
