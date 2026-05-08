# Phase 3 — spec-compliance review, round 1

**Date:** 2026-05-08
**Branch:** `quickstart/v0_1` at commit `f340d92`
**Reviewer:** feature-dev:code-reviewer (spec-compliance focus)
**Verdict:** SPEC_COMPLIANT (after controller verification of one finding)

---

## Passing checks

- **H1s** match plan Body column: ch 31 "Why multisig", ch 32 "Producing a 2-of-3 bundle", ch 33 "Stamping and recovering a 2-of-3 wallet".
- **Section structure** per Tasks 3.1, 3.2, 3.3: all required bullets present (single-point compromise, 2-of-3 mechanics, air-gapped vs coordinated, K=N rationale; mermaid + command + per-flag explain + 7-card output table; per-cosigner plate set + recovery quick-table).
- **Forward-pointer chain** 31→32→33→Part IV intact via relative paths.
- **DANGER box** `:::danger` present in ch 32:40, body re-authored newcomer voice with "public BIP-39 test vectors" framing and ch 22 cross-reference.
- **Mermaid block** ` ```mermaid flowchart LR` at ch 32:10-31 (3 cosigners → toolkit → 7-card output).
- **ch 33 cross-reference** to ch 32's DANGER box at line 13-14: "Reminder. Examples below still reference the public BIP-39 test phrases used in [chapter 32's bundle](32-bundle.md)…"
- **Bundle command flags** (ch 32) all valid against `cli-help/mnemonic-bundle.txt`: `--network`, `--template wsh-sortedmulti`, `--threshold 2`, three `--slot @N.phrase=…`, `--self-check`. Phrases are BIP-39 canonical test vectors #1, #2, #3.
- **Recovery quick-table** (ch 33) cells cross-referenced to manual `35-recovery-paths.md`; all match.
- **Cross-chapter links** all use relative paths (`../20-singlesig/22-generate-entropy.md`, `32-bundle.md`, `../40-watch-only/41-singlesig-watch-only.md`); no bare `#anchor` form.

## Reviewer flagged finding (controller-validated as not a bug)

The reviewer flagged ch 32 line 113 ("six `--mk1` (two per cosigner × 3), four `--md1`") as a factual error, claiming a 2-of-3 bundle has 3 mk1 cards (so `--mk1` ×3) and 1 md1 card (so `--md1` ×1).

**Controller verification:** Reviewer's interpretation is wrong. The flag count tracks emitted strings, not cards. Per `docs/manual/src/30-workflows/32-multisig-2of3.md:36`:

```
--mk1 <mk1-cosigner-0-line-1> --mk1 <mk1-cosigner-0-line-2>
--mk1 <mk1-cosigner-1-line-1> --mk1 <mk1-cosigner-1-line-2>
--mk1 <mk1-cosigner-2-line-1> --mk1 <mk1-cosigner-2-line-2>
--md1 <md1-line-1> --md1 <md1-line-2> --md1 <md1-line-3> --md1 <md1-line-4>
```

That is 3 mk1 cards × 2 strings = 6 `--mk1` repetitions, and 1 md1 card × 4 strings = 4 `--md1` repetitions. Implementer's original numbers are correct.

**Action taken:** Clarified the prose at ch 32:111-118 to explicitly disambiguate "string count" from "card count" so the same misreading is harder to make. No factual fix needed.

## Verdict

`SPEC_COMPLIANT`. All structural requirements met. The one flagged item was a reviewer-side reading error, validated against the manual's authoritative invocation example.
