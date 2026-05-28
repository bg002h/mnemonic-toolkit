# R1 re-review — SPEC_wallet_cross_format_convergence_tests.md (verbatim, post-fold)

Reviewer: feature-dev:code-reviewer (opus). R0 was RED 0C/2I/3M (`wallet-convergence-R0-review.md`); folded; this is the re-dispatch.

## VERDICT: GREEN — 0 Critical / 0 Important
All four R0 folds (I1, I2, M1, M3) correctly applied; no new drift. Every jade-related claim verified against current source. Cleared to implement (mandatory R0 gate satisfied: RED 0C/2I → fold → R1 GREEN 0C/0I).

## Fold verification
- **I1 (jade wrap) — CORRECT.** Export (`wallet_export/jade.rs:34-64`) = bare Coldcard text, requires multisig template, refuses singlesig/taproot. Import (`wallet_import/jade.rs:101-153`) extracts top-level `multisig_file` string (`:119-128`) and feeds it byte-for-byte to `parse_coldcard_multisig_text` (`:131`); `multisig_name`/`id` preserved-but-unused, NOT required/validated; sniff only needs non-empty `multisig_file` (`:95-98`). The `{"multisig_file":"<text>"}` wrap works exactly as the spec claims. jade correctly in C2/C3 7-format set.
- **I2 (H4 jade B-leg) — CORRECT/constructible.** `export-wallet --format jade` requires multisig template (`format_requires_template` `cmd/export_wallet.rs:54` Jade=>true), refuses singlesig/taproot, `--threshold` exists. H4 drives `--format jade --template wsh-sortedmulti --threshold 2` + wrap.
- **M1 (C1 coldcard) — CONSISTENT + clarified.** Importer builds origin bracket from top-level `xfp` (true master fp) (`wallet_import/coldcard.rs:232-242,296-303`); `bipNN.xfp` parent fp is informational, NOT used for the bracket. So coldcard CONVERGES on master fp — tolerate-and-record is a safety net, not an expected divergence. (Spec wording updated accordingly.)
- **M3 (soft descriptor sub-check) — CORRECT.** C2/C3 descriptor-difference probe now soft (record, don't hard-fail) with re-canonicalization rationale.

## No new drift
C2/C3 7-format set all export+import capable (electrum multisig export `wallet_export/electrum.rs:70-71`, requires `--threshold`, refuses taproot; coldcard-multisig text writes `cs.fingerprint`=true master fp `coldcard.rs:360-361`). C4 order-preserving set + coldcard probe, C-neg, H1/H2/H3/H5/H6, KeyMaterial extraction untouched. Internal consistency holds.

## Residual Minor (non-blocking)
1. M1 framing (applied): coldcard EXPECTS convergence; tolerate-and-record is a safety net.
2. Source SHA: reviewer read the stale session-start gitStatus (`e7b0157`); actual `git rev-parse HEAD` = `9a88a46` (confirmed). Spec SHA is correct — false alarm.
3. Cosmetic line-number drift (a few citations off by a few lines / missing `cmd/` prefix); all content verified correct against current source.

None gate GREEN.
