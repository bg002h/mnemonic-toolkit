# v0.31.5 plan-doc R0 review

**Reviewer:** opus (feature-dev:code-reviewer)
**Round:** R0
**Plan under review:** `design/PLAN_mnemonic_toolkit_v0_31_5.md`
**Date:** 2026-05-21
**Source SHA:** `92cbdc0` (master HEAD)

## Verdict

**YELLOW.** 0 Critical / 3 Important / 0 Minor. All tractable as plan-doc folds.

## Important (I)

**I1 — Risk-register claim about "OLD error text" is factually wrong.** `cli_seedqr.rs:278-289` + `:325-333` use `predicates::str::contains("seedqr: encode: invalid word count")` — a prefix substring that survives the new parenthetical update. NO assertion update needed. Drop the entry from the risk register.

**I2 — Line citation L292/L303/L314 verified correct.** Plan's L292 (`encode_rejects_15`) + L303 (`encode_rejects_18`) + L314 (`encode_rejects_21`) match source. No action.

**I3 — Test surface adjustments:**
- **Drop duplicate boundary cells (I3a):** existing lib cells at `seedqr.rs:186-219` AND existing CLI cells at `cli_seedqr.rs:110-154` already cover 47/49/95/97 decode-digit-count boundaries. The plan's "NEW refusal cells: `decode_rejects_47_digit_count` + `decode_rejects_49_digit_count` + `decode_rejects_97_digit_count`" duplicate existing coverage. Drop.
- **Add JSON-envelope CLI cell (I3b):** existing 12-word test surface has JSON-envelope variants (`decode_json_mode_*`, `encode_json_mode_*`); the new word counts need at least one parallel cell (e.g., `encode_json_mode_15_word`) to confirm `word_count` field emits correctly for non-`{12,24}` values.

## Verifications passed

- Source line citations L40/L43/L62/L107: all match source verbatim.
- Decode digit math 60/72/84 = 15/18/21 × 4: correct.
- SemVer PATCH: behavior-expansion only; correct.
- Canonical Trezor zero-entropy vectors (`agent` / `agree` / `ahead`): not cross-validated against external sources, but plan's self-check via encode round-trip is adequate.

## Recommended folds before Phase 2

1. Drop the assertion-string-update item from §"Risk register" (I1).
2. Drop the 3 duplicate decode boundary cells (I3a).
3. Add at least one 15/18/21 JSON-envelope CLI cell (I3b).
