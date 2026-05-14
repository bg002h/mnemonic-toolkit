# Phase 3a Re-scope Proposal v3-fold Review (R0)

**Reviewer:** Opus 4.7 (1M context), `feature-dev:code-reviewer`
**Date:** 2026-05-13
**Proposal reviewed:** `/home/bcg/.claude/plans/2026-05-13-cycle-b-phase-3a-rescope-proposal.md`
**Prior round:** v3 R0 at `design/agent-reports/v0_9_B-phase-3a-rescope-r0-v3.md` (1 Critical / 2 Important; declined v3)
**Verdict:** **LOCK** — 0 Critical / 0 Important — proceed to Task 1

**Method:** Source-ground-truth verification for every claim per `feedback_r0_must_read_source_off_by_n`.

## v3 finding fold verification

### v3 C-1 (ResolvedSlot field skeleton)

**Verified against** `crates/mnemonic-toolkit/src/synthesize.rs:578-592`:

| Field | Type | Order | Match |
|---|---|---|---|
| `xpub` | `Xpub` | 1 | ✓ |
| `fingerprint` | `Fingerprint` | 2 | ✓ |
| `path` | `DerivationPath` | 3 | ✓ |
| `path_raw` | `String` | 4 | ✓ |
| `entropy` | `Option<Vec<u8>>` | 5 | ✓ |
| `master_xpub` | `Option<Xpub>` | 6 | ✓ |

§3.3 skeleton matches source exactly. `_entropy_pin: Option<Arc<PinnedPageRange>>` correctly declared LAST so on Drop, `entropy` drops first then `_entropy_pin`. **Fold correct.**

Cascade check (Task 2.1 helper): `dummy_resolved_slot_with_entropy` ctor body uses correct field names matching the source. **Fold correct.**

### v3 I-1 (`bundle.rs:417` arm label)

**Verified against** `cmd/bundle.rs:340-498`:

| Site | Arm | `entropy` value in source | §3.3 table claim | Match |
|---|---|---|---|---|
| `:348` | Phrase | `Some(entropy)` | `Some(entropy)` | ✓ |
| `:417` | Xpub (watch-only) | `entropy: None` | `None` (watch-only) | ✓ |
| `:449` | Entropy | `Some(entropy_bytes)` | `Some(entropy_bytes)` | ✓ |
| `:491` | Wif (watch-only) | `entropy: None` | `None` (watch-only) | ✓ |
| `:1065` | cosigner-bridging | conditional `if i == 0` | `Some(...)` | mostly ✓ (i=0 only; else-arm pins None) |
| `synthesize.rs:1184` | test fixture | conditional on `entropy_indices` | `Some(test_vec)` | ✓ in always-pin variant |

The `:1065` cosigner-bridging row is conditional in source but the proposal's table presents the populated-pin case. Acceptable narrative shorthand — the implementing engineer will pattern-match the existing `if i == 0` conditional. **Fold correct.**

### v3 I-2 (SPEC §6 G4.a rewrite)

**Verified against** `design/SPEC_secret_memory_hygiene_v0_9_B.md:201` current G4.a text. Task 1.2 REPLACE clause:
- Drops "OPPOSITE order" claim (preserves drop ORDER) ✓
- States Site 2 has NO scrub under Path B-lite ✓
- Distinguishes Site 3 (Cycle A's `impl Drop` zeroize) from Site 2 (no zeroize) ✓
- Cites the deferred FOLLOWUP `resolved-slot-derived-account-zeroizing-field` ✓

**Fold correct.**

### v3 V3-7 (FOLLOWUP status string)

**Verified against** `design/FOLLOWUPS.md:54`. Actual current Status string:

> `scheduled for closure in v0.9.0 Cycle B Phase 3a`

Task 1.3 "change FROM" clause uses this exact string verbatim. **Fold correct.**

## New-issue scan (below report threshold)

- **§1.1 vs §3.3 narrative tightness:** §1.1:40 states all 6 ctor sites populate `Some(Arc::new(...))` while §3.3's table correctly shows 2 watch-only sites use `None`. Minor imprecision; §3.3 is source-of-truth and unambiguous. Confidence ~50, below report threshold.
- **R-5 leftover phrasing:** "8 (5 named + 3 vec-iteration) Site 1 pins per cmd handler" describes bundle.rs only; convert.rs and derive_child.rs have 2-3 each per §3.1. Cosmetic-only. Below threshold.
- **`dummy_xpub()` helper:** Confirmed via grep that no such helper currently exists in `synthesize.rs`. Proposal correctly states "to be added at GREEN time" with a fallback suggestion to reuse the existing `unified_fixture(1, &[0])` at `:1170`. Honest framing. No issue.
- **Imports for test code:** `Xpub`, `Fingerprint`, `DerivationPath` all imported at `synthesize.rs:12`; available to `mod path_b_lite_pin_tests` via `use super::*`. ✓

## Verdict

All v3 findings (C-1, I-1, I-2, V3-7) correctly addressed by the v3-fold. No new high-confidence (≥80) issues introduced. Source ground truth verified for every load-bearing claim.

**LOCK** (0 Critical / 0 Important — proceed to Task 1).

**Files relevant to this review:**
- `/home/bcg/.claude/plans/2026-05-13-cycle-b-phase-3a-rescope-proposal.md`
- `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/synthesize.rs` (lines 578-592, 1170-1198)
- `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/cmd/bundle.rs` (lines 340-498, 1062-1073)
- `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/derive.rs` (lines 1-58)
- `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/derive_slot.rs` (line 77)
- `/scratch/code/shibboleth/mnemonic-toolkit/design/FOLLOWUPS.md` (line 54)
- `/scratch/code/shibboleth/mnemonic-toolkit/design/SPEC_secret_memory_hygiene_v0_9_B.md` (line 201)
