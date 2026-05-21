# v0.31.2 plan-doc R0 review

**Reviewer:** opus
**Round:** R0
**Plan under review:** design/PLAN_mnemonic_toolkit_v0_31_2.md
**Date:** 2026-05-21
**Source SHA:** 7fa721d (master HEAD)

## Critical (C)

None.

## Important (I)

**I1 — Stale doc-comment / Steps 6+ documentation drift inside `sparrow.rs`.** Plan Task 1 Step 1 updates only the inline path-split comment block. The top-of-file rustdoc block at L52-56 (`Taproot multisig (`tr(NUMS, ...)`) descriptors … For P1B, the parse REFUSES descriptors whose `script` field contains `tr(` (taproot multisig path)`) is doubly stale (P1B taproot-multisig refusal was already lifted in Cycle 8, and now singlesig refusal is being lifted too). The Step 6 docstring on `fn parse` at L209-211 still references the refusal. Both surfaces should be updated in Task 1 Step 1; otherwise readers see contradictory narrative versus code.

**I2 — Phase 3 Step 3 "2nd taproot singlesig cell" is under-specified + half-rescinded.** Task 2 Step 3 says "Add a cell … skip for now; just one cell suffices" + then says "Optional: add a round-trip cell". The "skip for now" framing conflicts with the step's title. Recommend: required round-trip cell `taproot_singlesig_template_round_trip_via_from_import_json` that imports the Bip86 fixture, re-emits via `export-wallet --from-import-json`, asserts the re-emitted blob byte-equals the original.

**I3 — Fixture asymmetry.** `tests/fixtures/wallet_import/` carries `sparrow-singlesig-p2wpkh.json`, `sparrow-singlesig-p2sh-p2wpkh.json`, and `sparrow-tr-multi-a-nums-2of3.json` but NO `sparrow-singlesig-p2tr.json`. Phase 3 Step 1/2 keep inline-blob assertions; consistency with the other singlesig templates argues for a dedicated fixture file + a fixture-driven cell in the `tests` mod of `sparrow.rs`.

## Minor (M)

**M1 — Manual section title rename will break the existing anchor `#taproot-import-shipped-v0311` referenced at `45-foreign-formats.md:826`.** Update back-reference in lockstep.

**M2 — Plan-doc Cycle 8 reference precedent.** Embed a one-liner under "Cross-phase invariants" referencing the prior R0 report.

**M3 — FOLLOWUP closure entry sketch.** Suggest adding the SHA of the v0.31.2 release commit in the `Resolved by:` line.

## Verdict

**GREEN.** The plan is sound and ships a correct one-line semantic change. P0 recon empirically proved the substitution branch produces a descriptor (`tr([fp/86'/0'/0']xpub.../<0;1>/*)`) that the pipeline accepts cleanly. All three refusal-asserting tests are correctly enumerated. I1-I3 are tractable pre-execution folds.
