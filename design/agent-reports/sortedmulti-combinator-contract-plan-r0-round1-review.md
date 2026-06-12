# R0 Review — sortedmulti-in-combinator contract (PLAN) — Round 1
Reviewer: Fable 5, 2026-06-12. Verified against mnemonic-toolkit origin/master f0587ab; companions descriptor-mnemonic a3abdc8, miniscript-fork 95fdd1c, crates.io miniscript-13.0.0.

## Verdict: RED (0C/2I)

Both technical pillars verified correct (the 2-key shape is a clean sole-child-class representative — probed, NOT a key-reuse refusal; the FOLLOWUP root-cause correction is factually right — wire round-trips, refusal is the renderer/pin split). The 2 Importants are scope/durability defects with cheap NO-BUMP remedies.

## Critical
None.
## Important
- **I1 — the deferral leaves `design/FOLLOWUPS.md:85` falsely claiming GAP-3 implemented the advisory.** :85's re-scope text says the advisory was "RE-SCOPED into the `bundle-unrestorable-shape-advisory` umbrella implemented by the GAP-3 cycle" + Tier "advisory re-scoped to the GAP-3 cycle." If GAP-3 ships WITHOUT the advisory, the registry's only pointer claims this cycle did it → functionally a silent drop. **Fix:** add a 3rd doc edit amending :85 (advisory OPEN; deferred OUT of GAP-3, which shipped the contract-pin only) — or promote `bundle-unrestorable-shape-advisory` to a standalone deferred entry enumerating the 3 shapes. Still NO-BUMP.
- **I2 — `.failure()`-only is insufficient for the chosen shape; pin the `sole child` substring.** `wsh(or_d(pk(@1),sortedmulti(2,@0,@1)))` has @1 reused in both pk + sortedmulti (a 2nd refusal candidate). The sole-child arm fires TODAY (probed; identical message to the no-reuse 3-key shape), so it's correctly targeted now — but when the recon's deferred faithful nested-sortedmulti fix lands, a reuse-confounded shape may KEEP refusing (repeated-pubkey sanity) and the cell stays green, masking the transition where this pin must flip red. The recon itself (:65) recommended asserting `sole child` + `faithful backup`. **Fix:** a sibling `#[test]` cell asserting `.failure()` + stderr contains `"sole child"` (+ optionally `"faithful backup"`). Probe-verified stderr: `error: --md1 → descriptor: address derivation failed: Tag::SortedMulti must be the sole child of wsh/sh; cannot appear as a miniscript leaf. The engraved card remains a faithful backup.` (md-codec `to_miniscript.rs:419` → toolkit `restore.rs:926`). Test-only; NO-BUMP.

## Minor
- M1: shape style cosmetic (bare @0/@1 vs `/<0;1>/*` — both probe-refuse identically; both trigger origin-defaulting).
- M2: sweep the harness comment `prop_backup_restore_roundtrip.rs:91-96` (it repeats the wire-vs-renderer-ambiguous framing the FOLLOWUP correction fixes) → add "renderer-level (miniscript-pin split), not wire-level".
- M3: `self_test_bad_sortedmulti_under_combinator` moved to `descriptor-mnemonic .../proptest_to_miniscript.rs:663-676` @ a3abdc8 (recon cited :380-393 @ 422b049) — use live lines + SHA in the FOLLOWUP rewrite.

## Verified
1. Contract-pin shape CONFIRMED clean (probe: bundle exit 0, restore exit 1, sole-child message — not key-reuse; matches the no-reuse 3-key shape). 2. FOLLOWUP correction RIGHT: md-codec P7 proves wire round-trips; crates.io miniscript-13.0.0 has 0 `Terminal::SortedMulti`, fork 95fdd1c has it (`decode.rs:157`). 3. Deferral reasonable but I1. 4. NO-BUMP confirmed (test + docs, no clap surface).

## Probe results
`wsh(or_d(pk(@1),sortedmulti(2,@0,@1)))`: bundle exit 0 (4 chunks), restore exit 1, stderr = sole-child message above. No-reuse 3-key `wsh(or_d(sortedmulti(2,@0,@1),and_v(v:pk(@2),older(144))))`: identical refusal arm. miniscript-13.0.0 grep `Terminal::SortedMulti` = 0; fork = present.

**Re-dispatch after folding I1 + I2.**
