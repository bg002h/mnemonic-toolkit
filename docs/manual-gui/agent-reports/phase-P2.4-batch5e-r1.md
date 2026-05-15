# Phase P2.4 sub-batch 5e — R1 opus architect-reviewer

**Date:** 2026-05-15
**Branch:** `manual-gui-v1`
**Scope:** Verify R0 folds (6C/1I/0n) for the 4 5e chapters.

**Verdict:** **ITERATE 0C / 1I / 0N / 0n.**

## R0 fold verification (PASS)

All 6 R0 Critical folds + 1 Important fold byte-verified:

| Fold | Status | Source |
|---|---|---|
| **C-1** slip39-split degenerate `1,1` | PASS | byte-exact per `cmd/slip39.rs:320-322` |
| **C-2** slip39-split G9 advisory | PASS | byte-exact per `cmd/slip39.rs:480-484` |
| **C-3** slip39-combine insufficient shares | PASS | both variants per `cmd/slip39.rs:678-683` |
| **C-4** slip39-combine TTY advisory | PASS | byte-exact per `cmd/slip39.rs:595-598` |
| **C-5** seed-xor-combine TTY advisory | PASS | byte-exact per `cmd/seed_xor.rs:317-321` |
| **C-6** seed-xor-split share-secrecy advisory | PASS | byte-exact per `cmd/seed_xor.rs:193-198` |
| **I-1** seed-xor-split deterministic-from-master + 15/21-word | PASS | byte-exact per `cmd/seed_xor.rs:183-190` |

## New finding (Important)

### I-R1-1 — `4a-slip39-split.md:117-120` body prose contradicts source on E=5 timing

The body prose claimed "at E=5 the share-decoding takes ≈ 30 seconds on commodity hardware; at E=15 it takes hours." Source (`cmd/slip39.rs:480-484`) characterizes E=5 as "sub-second to multi-second" and only "E ≥ 10 may exceed 30s on weak hardware". Iteration math confirms: E=5 = 320,000 PBKDF2 iterations (sub-second on modern hardware); E=10 = 10.24M (~30s on weak hardware).

The chapter's own Advisories-table entry (line 235, post-fold) is byte-correct against source — only the body prose contradicts it.

**Fix needed:** rewrite L117-120 to mirror source's characterization.

## Lint state

- Phase 4 schema-coverage RED at **127 missing** (unchanged; folds were prose-only).
- Phase 5 outline-coverage RED at **18 missing** (unchanged).
- Phases 1-3 GREEN.
- HTML 25 H1 chapters; PDF 131 pages.

After I-R1-1 fold, R2 should LOCK.
