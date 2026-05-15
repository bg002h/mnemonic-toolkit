# Phase P2.4 sub-batch 5e (Track M — slip39 + seed-xor families) — R0 opus architect-reviewer

**Date:** 2026-05-15
**Branch:** `manual-gui-v1`
**Scope:** 5e final sub-batch of batch 5 — `48-seed-xor-split.md` (NEW, ~150 LOC, 5 flags); `49-seed-xor-combine.md` (NEW, ~150 LOC, 4 flags); `4a-slip39-split.md` (NEW, ~250 LOC, 8 flags); `4b-slip39-combine.md` (NEW, ~210 LOC, 6 flags). Plus single-variant patches in `47-final-word.md`, `48-seed-xor-split.md`, `49-seed-xor-combine.md` (3 anchor additions for 1-variant `--from`/`--share`).

**Verdict:** **ITERATE 6C / 1I / 0N / 0n.**

Outline structure, per-flag sections, multi-variant outlines (slip39-split `--from`, slip39-combine `--to`, all 4 `--language` instances), conditional-visibility prose, and worked examples all verified clean. The single-variant fold patches are source-faithful in all three target files. **But six byte-exactness drifts** in advisory and refusal stems for the slip39 + seed-xor pairs (R0 caught the cycle's recurring pattern again).

## Critical

### C-1 — slip39-split degenerate `--group 1,1` refusal stem fabricated

`4a-slip39-split.md:104, 220` invented `slip39 split: group <i> = 1,1 is degenerate (zero-information share)`. Source at `crates/mnemonic-toolkit/src/cmd/slip39.rs:320-322` + mapped variant `:654-656`: `slip39 split: 1-of-1 group offers no recovery benefit; use --group N,T with N >= 2 (toolkit policy); got group <g_idx>=1,1`. Replaced with byte-exact mirror.

### C-2 — slip39-split G9 iteration-exponent advisory fabricated

`4a-slip39-split.md:230` wrote a paraphrased "≈ <T> on this machine" advisory. Source at `cmd/slip39.rs:480-484` is much more specific (cites Trezor's E=1 default, gives PBKDF2 iteration count formula, specific hardware threshold). Replaced with byte-exact mirror.

### C-3 — slip39-combine insufficient-shares refusal fabricated

`4b-slip39-combine.md:188` invented `not enough shares to recover master`. Source at `cmd/slip39.rs:678-683` has two byte-exact variants (group-level sentinel vs per-group). Replaced with both forms.

### C-4 — slip39-combine TTY combine advisory fabricated

`4b-slip39-combine.md:197` invented `warning: reconstructed master is secret material; do not paste this output into untrusted tools`. Source at `cmd/slip39.rs:595-598` is `warning: reconstructed secret material on stdout — verify the recovered wallet's expected derived address before trusting`. Replaced.

### C-5 — seed-xor-combine TTY combine advisory fabricated

`49-seed-xor-combine.md:163` invented similar wording. Source at `cmd/seed_xor.rs:317-321` is `warning: combined phrase is secret material — Seed XOR has no authentication tag; verify the recovered wallet's expected derived address before trusting; if a share was substituted with a wrong-but-valid one, the result will validate but derive the wrong wallet`. Replaced.

### C-6 — seed-xor-split TTY share-secrecy advisory fabricated

`48-seed-xor-split.md:171` invented short wording. Source at `cmd/seed_xor.rs:193-198` is the much longer K-of-N advisory describing the per-share independence + substitution-undetectability property. Replaced.

## Important

### I-1 — seed-xor-split missing the `--deterministic-from-master` + 15/21-word toolkit-only advisory

`cmd/seed_xor.rs:183-190` emits a conditional advisory when `--deterministic-from-master` is set AND the master word count is 15 or 21 (the toolkit supports these but Coldcard does not — resulting shares won't round-trip a Coldcard device). The `--deterministic-from-master` flag is documented in detail in the chapter but the advisory class was missing from the table. Added.

## Lint state (post-fold)

- Phase 4 schema-coverage RED at **127 missing** (was 201 → -74 = 4 sub + 23 flags + 47 variants for 5e + 3 single-variant fold patches). No orphans.
- Phase 5 outline-coverage RED at **18 missing** (was 28 → -10 = 4 subcommand-outlines + 6 flag-outlines = `slip39-split-from` + `slip39-split-language` + `slip39-combine-to` + `slip39-combine-language` + `seed-xor-split-language` + `seed-xor-combine-language`).
- Phases 1-3 GREEN.
- HTML 25 H1 chapters (was 21 → +4).
- PDF 131 pages (was 109 → +22).

After folds, R1 should LOCK. Batch 5 complete after this commit.
