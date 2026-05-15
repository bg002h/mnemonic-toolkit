# Phase P2.4 sub-batch 5e — R2 opus architect-reviewer (LOCK)

**Date:** 2026-05-15
**Branch:** `manual-gui-v1`
**Scope:** Verify R1 fold (I-R1-1: `4a-slip39-split.md` body prose corrected to match source's E-iteration-exponent characterization).

**Verdict:** **LOCK 0C / 0I / 0N / 0n.** Closes batch 5 (40-mnemonic chapter; 11 files; 5 sub-batches; 14 reviewer rounds total).

## I-R1-1 fold verification (PASS)

`4a-slip39-split.md:117-124` now reads byte-source-faithful against `crates/mnemonic-toolkit/src/cmd/slip39.rs:480-484`:
- E=5 → "sub-second to multi-second split+combine performance" (matches stderr template)
- E ≥ 10 may exceed 30s on weak hardware (corrected from prior "E=5 ≈ 30 seconds")
- Cites Trezor's E=1 (20000 iters) default
- Notes SLIP-0039 spec gives no recommended values
- Cross-references Advisories table (line 235, byte-exact)

## R0 C-1..C-6 + I-1 spot-check (no regressions)

All 7 byte-exact mirrors held intact through the prose-only I-R1-1 fold:
- C-1: `1-of-1 group offers no recovery benefit` — matches `cmd/slip39.rs:320-322`
- C-2: G9 advisory — byte-exact
- C-3: `insufficient shares for group` two-variant — matches `:678-683`
- C-4: `reconstructed secret material on stdout — verify the recovered wallet's expected derived address` — matches `:595-598`
- C-5: `Seed XOR has no authentication tag` long form — matches `cmd/seed_xor.rs:317-321`
- C-6: K-of-N seed-xor stdout-on-TTY long form — matches `cmd/seed_xor.rs:193-198`
- I-1: `--deterministic-from-master` 15/21-word advisory — matches `:183-190`

## Lint + build state

- Phase 4 schema-coverage RED at **127 missing** (no anchor change from prose-only fold).
- Phase 5 outline-coverage RED at **18 missing**.
- Phases 1-3 GREEN.
- HTML 25 H1 chapters; PDF 131 pages.

## Cycle close

Batch 5 (40-mnemonic chapter) closes here:
- 11 files (1 overview + 10 subcommand chapters)
- ~5500 LOC
- 5 sub-batches: 5a (overview + final-word), 5b (bundle alone), 5c (verify-bundle + convert), 5d (export-wallet + derive-child), 5e (slip39 + seed-xor families)
- 14 reviewer rounds total (5a R0+R1, 5b R0+R1, 5c R0+R1, 5d R0+R1, 5e R0+R1+R2 = 11 + previously the 3 batch-1/2/3 = 14 cumulative for the cycle's pattern)

**Batch 5 LOCKED. Next: batch 6 (50-md chapter).**
