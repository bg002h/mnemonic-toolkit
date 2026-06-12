# Impl Review — taproot restore-refusal contracts (GAP 1-T1) — self-review
Reviewer: orchestrator (Fable 5), 2026-06-12. Working tree at origin/master 2f03eb0 + this change.

## Verdict: GREEN (0C/0I)

A pure test-only cycle (`tests/cli_restore_taproot_refusal.rs`, 3 cells) implementing the R0-GREEN (round 2) plan exactly.

- **3 cells pass** (`cargo test --test cli_restore_taproot_refusal`: 3 passed): general tr leaf + multi-leaf taptree both → exit 2 + `not a recognized multisig`; distinct-cosigner-IK `tr(K2,multi_a(2,K0,K1))` → exit 2 + `non-NUMS (cosigner) internal key`. Each first asserts `bundle` emits a non-empty card and (cells 1–2) that `.descriptor` round-trips EXACTLY (the faithful-backup leg; literal `NUMS` preserved on the wire — R0-M1).
- **R0 GREEN round 2 (0C/0I)** — round 1 (0C/2I) found the is_nums:false arm IS bundle-reachable (my earlier false-negative was a duplicate-key BIP-388 gate); folded to 3 arms + the multi-leaf wire-round-trip assertion, fully delivering the FOLLOWUP's T1 scope. R0 re-probed the 3rd arm live. All 5 folds (I1/I2/M1/M2/M3) verified landed.
- **Scope/NO-BUMP** — one new test file; no `src/` change → no clap surface, no manual/GUI/schema_mirror; the refusal arms (restore.rs:689/:710, `ModeViolation => 2` exit) are PINNED as the current contract (a future restore-walker change must update these cells). Keys = the real `cli_bundle_import_json.rs:312-314` trio (R0-M2). clippy `-D warnings` clean; `cargo +1.95.0 fmt --all --check` = only mlock.rs differs (standing exemption) → my file fmt-clean.
- **FOLLOWUP** — `restore-general-and-multi-leaf-taproot-roundtrip` updated: T1 SHIPPED; T3 remainder = faithful reconstruction + the `tree:None` keypath-only fixture arm + the misleading keypath-only-`tr(<xpub>)` message reword.
- **No silent mis-reconstruction found** — R0 probed 6 taproot shapes; every non-(multi_a/sortedmulti_a ∧ NUMS) shape refuses loudly. The "wrong outcome" class does not exist on this surface.

Cleared to commit. NO-BUMP.
