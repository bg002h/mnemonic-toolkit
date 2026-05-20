# v0.28.0 cycle — P15 end-of-cycle architect review

**Reviewer:** Opus 4.7 via feature-dev:code-architect (end-of-cycle pass)
**Branch:** `release/v0.28.0` @ `9005715` (22 commits ahead of master)
**Test surface:** 1986/0 pass; `clippy --all-targets -- -D warnings` clean
**Date:** 2026-05-20

---

## Cycle changeset summary

- **22 commits** on `release/v0.28.0` (`9005715` HEAD).
- **6 new wallet-import parsers** shipped (sparrow / specter / coldcard / coldcard-multisig / electrum / jade).
- **BSMS BIP-129-canonical 4-line Round-2 parser** (P7A) + 6-line DEPRECATION NOTICE (P7B/C).
- **Single-leaf `tr(IK, {M})` compare-cost input** (P12).
- **Cross-format conversion matrix** (P11): 74 cells.
- **Fixtures expansion:** 7 BSMS + 8 Bitcoin Core.
- **SPEC docs added:** SPEC_wallet_import_v0_28_0 + SPEC_compare_cost_v0_28_0.
- **Manual chapters added:** 6 parser sections + 8 cross-format recipes + compare-cost tr() worked example.
- **CLI surface change:** `import-wallet --format` 2 → 8 values.
- **Test count:** 1581 (v0.27.2) → 1986 (+405 cells, +25.6%).
- **Agent reports persisted:** 37 R-files across 12 phases.
- **clippy:** clean.

## Critical (block tag): NONE.

## Important (must fold before tag)

### I1. `compare_cost.rs:22-25` doc-comment stale

`crates/mnemonic-toolkit/src/cmd/compare_cost.rs:22-25` current text says: "Multi-leaf tr and single-leaf tr inputs are refused (the latter via a future FOLLOWUP)". P12 closed that gap. Doc-comment propagates to `mnemonic compare-cost --help` AND `gui-schema` JSON description — user-facing drift.

**Fold:** rewrite the doc-comment to match manual chapter 41-mnemonic.md:2485 which is already correct ("full descriptor — `wsh(M)`, `sh(wsh(M))`, or single-leaf `tr(IK, {M})` (v0.28.0)").

### I2. mnemonic-gui schema-mirror paired PR not filed

`mnemonic-gui/src/schema/mnemonic.rs` `import-wallet --format` dropdown still has only `["bsms", "bitcoin-core"]`. v0.28.0 toolkit ships 8 values. Per CLAUDE.md GUI schema-mirror lockstep invariant: any `--format` value addition needs a paired mnemonic-gui PR.

**Fold options:**
1. **Preferred:** author paired mnemonic-gui PR now bumping IMPORT_WALLET_FORMAT_VALUES to 8-value set.
2. **Acceptable with documentation:** file FOLLOWUP `gui-schema-mirror-v0_28_0-import-wallet-format-expansion` documenting the lagging-indicator gap.

This is the exact anti-pattern v0.27.2's `gui-schema-mirror-lockstep-discipline` FOLLOWUP codified against.

## Minor

- M1: CHANGELOG "8 sources × N destinations" narrative; cosmetic.
- M2: CHANGELOG cell-count "24 + 42 + adjuncts = 74" math ambiguity; cosmetic.
- M3: Manual `45-foreign-formats.md` jade heading anchor uses `{#jade-multisig}` not `{#wallet-import-jade}`; cosmetic.
- M4: Duplicate stub at FOLLOWUPS.md:2477 retained per R1-I5 — verified correct.
- M5: I1 cross-link.
- M6: `wallet_import/sniff.rs:23-26` doc-comment vestigial "placeholder false slots flip on as their per-parser P{N}A sub-phase lands" language post-cycle; all 8 slots wired.

## Verification table

| Check | Pass/Fail | Notes |
|---|---|---|
| Cargo.toml version | PASS | `0.28.0` at `crates/mnemonic-toolkit/Cargo.toml:3` |
| CHANGELOG completeness | PASS-with-Minor | All shipped work documented; 2 Minor narrative quibbles (M1/M2) |
| FOLLOWUPS.md state | PASS | 9 resolved + 2 sub-deliverable updates (kept open) + 9 new filed + duplicate stub kept open synced |
| Manual chapters | PASS | 6 new parser sections + 8 recipes + compare-cost tr() worked example all present |
| Spot-checks (5-10 hunks) | PASS | sniff dispatch + BSMS 4-line + cost/strip.rs tr-handling + sparrow doc-header + PossibleValuesParser + first-address WARNING semantic per W1-end I1 fold |
| mnemonic-gui schema-mirror impact | FAIL (I2) | 8-value `--format` dropdown lockstep PR not filed; must address per CLAUDE.md |

## Tag readiness verdict

**YELLOW — fold needed before tag.**

Fold I1 (10-line doc-comment fix) + I2 (mnemonic-gui paired PR OR FOLLOWUP entry per CLAUDE.md). Everything else GREEN.

## Suggested next actions

1. Fold I1 — rewrite `cmd/compare_cost.rs:22-25` doc-comment.
2. Address I2 — paired mnemonic-gui PR OR explicit FOLLOWUP entry.
3. Optionally fold M6 (sniff.rs:23-26 vestigial language) into the same I1 commit.
4. Post-tag: push release/v0.28.0 → master via PR; tag mnemonic-toolkit-v0.28.0; install.sh pin bump; merge paired mnemonic-gui PR; bump mnemonic-gui pinned-upstream.toml + ship mnemonic-gui release.
5. Verify cycle-followups tracker post-P14A triage.
