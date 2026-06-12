# PLAN — sortedmulti-in-combinator: pin the refusal contract + correct the FOLLOWUP (GAP 3)

**Date:** 2026-06-12 · **Repo:** mnemonic-toolkit · **SemVer:** NO-BUMP (test + docs)
**Source SHA:** `origin/master` = `f0587ab`. **Recon:** `cycle-prep-recon-sortedmulti-in-combinator-contract.md`. **FOLLOWUP:** `bundle-accepts-sortedmulti-in-combinator-restore-cannot` (+ the re-scoped `bundle-engraves-unrestorable-pk-keyed-cards` umbrella).

## 1. Problem (probe-grounded)

`bundle --descriptor 'wsh(or_d(pk(@1),sortedmulti(2,@0,@1)))'` exits 0 and emits a faithful md1 card with NO warning; `restore --md1` then refuses cleanly (exit 1, "must be the sole child of wsh/sh"). The recon classified this as **silent-ENGRAVE but loud-SAFE-restore** (robustness gap, NOT a funds bug — the wire round-trips byte-exact, so the card is recoverable; restore never mis-reconstructs). The existing STRESS-A negative property `negative_property_unreconstructable_shapes_refuse_loudly` (`tests/prop_backup_restore_roundtrip.rs:551`) covers per-key use-site override + hardened wildcard but NOT sortedmulti-in-combinator — STRESS-A EXCLUDES the shape (`allow_sorted` only at top level) rather than pinning the refusal.

Two recon corrections to apply:
- The FOLLOWUP's root-cause sentence is **structurally wrong**: it says the md1 WIRE can't represent nested sortedmulti. FALSE — md-codec's P7 proves the nested shape wire-round-trips byte-exact (`tree.rs` encodes `Body::MultiKeys` for `Tag::SortedMulti` at ANY position). The refusal lives in the RENDERER (`to_miniscript.rs`): md-codec pins crates.io miniscript 13.0.0 (no `Terminal::SortedMulti`), while the toolkit pins git `95fdd1c` (HAS it). The FOLLOWUP's option (b) "extend md-codec to encode" is mis-framed; the gap is the renderer/two-miniscripts split.

## 2. The fix (test + docs, NO-BUMP)

1. **Pin the refusal contract** — add `wsh(or_d(pk(@1),sortedmulti(2,@0,@1)))` to the `negative_property_unreconstructable_shapes_refuse_loudly` loop's shape list (`prop_backup_restore_roundtrip.rs:556`). PROBE-VERIFIED: bundle emits, restore exit 1 (refuses loudly). Update the loop's doc-comment to name sortedmulti-in-combinator + cite `bundle-accepts-sortedmulti-in-combinator-restore-cannot`. This pins the end-to-end "engrave-but-loud-safe-refuse" contract so a future change can't silently turn it into a wrong reconstruction.
2. **Correct the FOLLOWUP root-cause** — fix `bundle-accepts-sortedmulti-in-combinator-restore-cannot`'s root-cause sentence (the md1 wire is NOT the limit; the renderer/miniscript-pin split is — cite md-codec crates.io 13.0.0 vs toolkit git `95fdd1c`). Note md-codec already pins its half (`self_test_bad_sortedmulti_under_combinator` P7). Cite the two-miniscripts split (v0.49.1 precedent).

## 3. DEFERRED (NOT this cycle): the bundle-time unrestorable-shape advisory
The recon's "non-blocking bundle-time advisory" (warn at engrave time that the card is faithful but not yet mechanically restorable) is a real PATCH UX feature requiring shape-detection in the bundle path + an R0 design pass (where to detect, the 3 shapes: sortedmulti-in-combinator + use-site-override + hardened-wildcard). It is tracked by the re-scoped `bundle-engraves-unrestorable-pk-keyed-cards` → `bundle-unrestorable-shape-advisory` umbrella (hygiene pass). This cycle PINS the safety contract (proving it's loud-safe today); the advisory is the deferred UX follow-up. Honest scope split — documented, not silently dropped.

## 4. Verification
`cargo test -p mnemonic-toolkit --test prop_backup_restore_roundtrip -- negative_property_unreconstructable_shapes_refuse_loudly` (the new shape bundles + refuses). clippy clean; fmt only-mlock-exempt. NO-BUMP (one test shape + 2 doc edits; no src change).

## 5. R0 questions
1. Is the 2-key `wsh(or_d(pk(@1),sortedmulti(2,@0,@1)))` shape the right contract-pin (fits the existing 2-slot loop; probe-confirmed bundle+refuse), or should it be the recon's 3-key `wsh(or_d(sortedmulti(2,A,B),and_v(v:pk(C),older)))` (needs restructuring the loop to 3 slots)? (Lean: the 2-key shape — same refusal arm, fits the loop, zero restructure.)
2. Deferring the advisory to the umbrella FOLLOWUP vs doing it now — confirm the safety contract-pin + the FOLLOWUP correction is a complete, honest GAP-3 cycle (the advisory is UX, the contract proves loud-safe). (Lean: defer — keeps this NO-BUMP + tractable; the advisory deserves its own R0.)
3. Pin a stable substring of the restore refusal (`sole child`?) or just assert `.failure()` (the existing loop asserts only `.failure()`)? (Lean: match the existing loop — assert `.failure()`; the message-pin is the advisory cycle's concern.)
