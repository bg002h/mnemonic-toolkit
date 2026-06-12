# Impl Review — sortedmulti-in-combinator contract (GAP 3) — self-review
Reviewer: orchestrator (Fable 5), 2026-06-12.

## Verdict: GREEN (0C/0I)

Test + docs cycle (NO-BUMP). R0 round 1 was RED (0C/2I); both Importants folded with the reviewer's EXACT specified remedies and verified.

## R0 folds landed
- **I1 (advisory not silently dropped):** FOLLOWUPS.md:85 amended — the `bundle-unrestorable-shape-advisory` umbrella is now marked **STILL OPEN/deferred** ("the GAP-3 cycle pinned the contract only; the advisory is a deferred PATCH UX feature needing its own R0"), removing the false "implemented by the GAP-3 cycle" claim. The registry now tracks the deferred advisory correctly.
- **I2 (substring pin, not `.failure()`-only):** a dedicated cell `sortedmulti_in_combinator_bundles_but_restore_refuses_loudly` asserts `.failure()` + stderr contains `"sole child"` AND `"faithful backup"` — the reviewer's exact probe-verified message (md-codec `to_miniscript.rs:419` → toolkit `restore.rs:926`). The chosen shape `wsh(or_d(pk(@1),sortedmulti(2,@0,@1)))` reuses @1, so a bare `.failure()` could go green for the wrong reason when the deferred faithful fix lands; the substring forces a conscious re-pin at that transition. Cell PASSES.
- **M2:** the harness comment (`prop_backup_restore_roundtrip.rs:91`) corrected — renderer-level (miniscript-pin split), not wire-level.
- **M3:** the FOLLOWUP `:53` root-cause sentence corrected (the md1 wire round-trips byte-exact — md-codec P7; the refusal is the renderer / crates.io-13.0.0-vs-fork-95fdd1c split), with live cites (a3abdc8). Option (b) "extend md-codec to ENCODE" re-framed (encoding already works; the gap is the renderer).

## Verification
- `cargo test --test prop_backup_restore_roundtrip -- sortedmulti_in_combinator negative_property`: 2 passed. The new cell pins bundle-emits + restore-refuses-loudly with the sole-child substring (probe-confirmed: bundle exit 0, restore exit 1).
- clippy `-D warnings` clean; `cargo +1.95.0 fmt --all --check` = only mlock.rs differs (standing exemption) → touched file fmt-clean.
- **Scope/NO-BUMP:** one test cell + one import + doc edits (test file comment + 2 FOLLOWUPS entries). No src change → no clap surface, no manual/GUI/schema_mirror. The bundle-time advisory (PATCH feature) is explicitly DEFERRED to the `bundle-unrestorable-shape-advisory` umbrella (I1).

Cleared. NO-BUMP.
