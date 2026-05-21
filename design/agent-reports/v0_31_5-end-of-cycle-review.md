# v0.31.5 end-of-cycle architect review

**Reviewer:** opus (feature-dev:code-reviewer)
**Round:** end-of-cycle
**Cycle:** Cycle 12 (seedqr-15-18-21-word-counts)
**Date:** 2026-05-21
**Pre-tag SHA:** `14745bb` (Phase 2-4 combined; Phase 5 uncommitted)

## Verdict

**GREEN.** All 7 verification items PASS.

## Verifications

1. **Source gates (`seedqr.rs`)** — both gates use `matches!(_, 48 | 60 | 72 | 84 | 96)` (L70) and `matches!(_, 12 | 15 | 18 | 21 | 24)` (L116). Error texts at L42, L48 reflect the widened sets.
2. **Test coverage** — 9 lib happy-path cells (decode/encode/round-trip × 15/18/21 at `seedqr.rs:207-252`) + 1 boundary refusal `encode_rejects_22_word_count` at L340 (drops obsolete 18-word refusal with explanatory comment) + 4 CLI cells (3 accepts + 1 JSON-envelope with `word_count: 15` field assertion).
3. **Canonical vectors** — cross-verified against the BIP-39 English wordlist (`raw.githubusercontent.com/bitcoin/bips/master/bip-0039/english.txt`). 0-based indices: `address`=27, `agent`=39, `admit`=29. Digit-strings `0027`/`0039`/`0029` correct.
4. **Manual mirror** — `41-mnemonic.md:1614,1629,1734,1738` all updated.
5. **Cargo.toml / install.sh / CHANGELOG** — all at `0.31.5`; CHANGELOG entry comprehensive.
6. **SemVer PATCH** — pure behavior-expansion; correct.
7. **Test totals** — 2162 (+10 vs v0.31.4 baseline 2152). Math checks.

## Cleared for tag.
