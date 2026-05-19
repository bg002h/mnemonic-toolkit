# v0.27.0 Phase 4 HOLISTIC Architect Review

**Date:** 2026-05-18
**Reviewer:** opus-4-7 (feature-dev:code-reviewer holistic dispatch)
**Phase:** v0.27.0 Phase 4 — full envelope (`8ac6847`) in context of Phases 1-3 SHIPPED + Phases 5-6 REMAINING
**Verdict:** GREEN — 0 Critical / 1 Important / 3 Minor

## Summary

Phase 4 is internally clean (per-phase R0 GREEN 0C/0I) and composes correctly with Phases 1/2/3 deliverables. The holistic dimension surfaces one Important finding (Phase 5 deserialization tripwire — `BundleJson` is Serialize-only with `&'static str` fields, so Phase 5's parser MUST go via `serde_json::Value` workaround like `verify_bundle.rs:980-1010` does, or define a parallel `BundleJsonOwned`). The 4-line emit vs 2/6-line parse asymmetry from Phase 3 is acknowledged via FOLLOWUP `bsms-bip129-full-cutover` and structurally cannot block Phase 5's headline cross-format cell. The Phase 4 `roundtrip.status="blocked_no_emitter"` for BSMS is an accepted scope cut, documented at `import_wallet.rs:396-399`.

## Findings

### Important — I1: Phase 5 deserialization tripwire on `BundleJson` (confidence 85)

**Where:** `crates/mnemonic-toolkit/src/format.rs:119` — `#[derive(Debug, Serialize)]` only (no `Deserialize`); fields `schema_version`, `mode`, `network`, `template: Option<&'static str>` are `&'static str`. `MkField` at format.rs:64-69 is `#[serde(untagged)]` Serialize-only. `MultisigInfo.template` at format.rs:105 is also `&'static str`.

**Why this matters for Phase 5:** Plan-doc §4.5 says the shared helper `wallet_import/json_envelope.rs` parses an envelope element "into a typed struct `ImportJsonEnvelope` (with `#[serde(deserialize_with = ...)]` for `bundle: BundleJson`)". But `BundleJson` cannot be deserialized as-is — `serde` cannot deserialize into `&'static str`. Phase 5's parser must either (a) define a parallel `BundleJsonView` mirror struct with `String` fields, or (b) traverse via `serde_json::Value` like `verify_bundle.rs:980-1010` does for ms1/mk1/md1.

**Concrete fix suggestion:** During Phase 5 R0 dispatch, explicitly include this in scope. The cleanest pattern is option (a): a `wallet_import::json_envelope::BundleJsonView` deserialization mirror struct with `String` fields. The existing precedent at `verify_bundle.rs:980-1010` uses option (b) and works fine; given Phase 5 needs the full envelope shape (not just ms1/mk1/md1), option (a) scales better. Update plan-doc §4.5 to specify which.

**Not a Phase 4 defect:** Phase 4's emission is correct; the tripwire only manifests at Phase 5 implementation. Surface now so it doesn't surprise the Phase 5 implementer.

### Minor — M1: Plan-doc §3.7.1 stale field count (confidence 90, but cosmetic)

**Where:** plan-doc `design/PLAN_v0_27_0_bsms_round_trip_and_wallet_import_handoff.md` §3.7.1 row count vs. live `crates/mnemonic-toolkit/src/wallet_export/mod.rs:336-382`.

**Issue:** Plan-doc calls §3.7.1 the "16-field EmitInputs contract"; live struct has **17 fields** (Phase 3 added `bsms_form: BsmsForm`). User-side kickoff already flagged this. The §4.5 Phase 5 R0 explicit scope says "enumerate all 16 fields against the live `cmd::export_wallet::run` construction site". Update the count to 17 (and add a `bsms_form` row to §3.7.1's table) at start of Phase 5 to prevent R0 dispatch from carrying stale guidance.

### Minor — M2: `multisig.template` literal `"descriptor"` instead of mapped (confidence 35)

**Where:** `cmd/import_wallet.rs:476` — `template: "descriptor"`.

**Issue:** §3.2.1 row `multisig` says "template = `WalletScriptType::as_static_str()` (or equivalent Display impl)". Phase 4 chose literal `"descriptor"` matching the precedent at `cmd/bundle.rs:663` (`template.unwrap_or("descriptor")`). No internal consumer reads this field; Phase 5's `script_type` derivation goes via descriptor parse + `script_type_from_descriptor` per §3.7.1. Phase 4 R0 (M1) evaluated this at confidence 35 and folded into the GREEN verdict. I concur — cosmetic deviation, not load-bearing. Possibly file a v0.27-cycle-close FOLLOWUP `bundle-json-multisig-template-static-str-mapping` for v0.28+ cleanup if the field becomes load-bearing for any consumer.

### Minor — M3: 4-line emit vs 2/6-line parse asymmetry within v0.27.0 (confidence 80, but acknowledged)

**Where:** Phase 3 emits 4-line BSMS canonical Round-2 by default; Phase 4 import-wallet parser at `wallet_import/bsms.rs:95` only accepts 2 or 6. `canonicalize_bsms` at `roundtrip.rs:82-90` also rejects 4-line.

**Status:** Explicitly acknowledged at `wallet_export/bsms.rs:28-30` doc comment and tracked via FOLLOWUP `bsms-bip129-full-cutover` (v0.28+). Phase 4's `roundtrip.status="blocked_no_emitter"` for BSMS is structurally correct: even if Phase 4 wired the import-side roundtrip block to call Phase 3's emitter, the parser couldn't re-ingest the 4-line emit. The 2-line round-trip path WAS exercised in Phase 3's cell `2-line→import round-trip`. NOT a defect; surfacing for full-cycle traceability.

## Holistic dimension verifications

**Cross-phase coherence.** Phase 1's `InspectEnvelope` is a sibling envelope, no coupling with Phase 4's `bundle:` field. Phase 3's BSMS emitter is NOT wired to Phase 4's roundtrip block (documented). `bsms_round1_verifications` correctly at outer envelope level, NOT inside `bundle`.

**Phase 5/6 readiness.** mk1 decode contract (§3.6.1) directly exercised in Cell 3. 17-field EmitInputs (§3.7.1): see M1. Deserialization tripwire (I1) is the load-bearing pre-Phase-5 awareness item.

**mk-codec depth/child_number quirk.** Cell 3 uses `(parent_fp, chain_code, public_key)` tuple correctly; Cell 7 uses real derivations. No silent failure masks.

**CHANGELOG accumulation deferred to Phase 6.** Phase 6 sweep must enumerate prior phase commits explicitly. Risk surface: the audit dependency. Mitigated by Phase 6 commit-shape brief listing each phase → CHANGELOG section.

**Tests-quality.** 8 cells per plan; coverage is right.

**Memory-flagged patterns.**
- `[[feedback-r0-must-read-source-off-by-n]]`: Met.
- `[[feedback-verify-bundle-round-trip-per-phase-r0-scope]]`: Met (Cell 7).
- `[[feedback-synthesize_unified-is-cli-hotpath]]`: Phase 4 uses `synthesize_descriptor` (correct — descriptor-mode); the memory note is about CLI `bundle` subcommand's template-mode hotpath, not wallet-import.

## Top 3 follow-up actions before Phase 5

1. **Update plan-doc §3.7.1 from "16-field" to "17-field"** and add the `bsms_form` row. Trivial pre-Phase-5 doc fold (M1).
2. **In Phase 5 R0 brief, explicitly include the BundleJson deserialization tripwire (I1).** Spec the chosen workaround: parallel `BundleJsonView` (deserialize-friendly mirror with `String` fields) OR `serde_json::Value` traversal like `verify_bundle.rs:980-1010`. Plan-doc §4.5's `#[serde(deserialize_with = ...)]` hint is incomplete.
3. **Phase 6 CHANGELOG sweep checklist:** enumerate Phase 1 (`e908309`), Phase 2 (`149b341`), Phase 3 (`4a2b6e7`), Phase 4 (`8ac6847`) commit hashes in the Phase 6 commit-shape brief, with one bullet per phase indicating which CHANGELOG section it contributes to. Mitigates the per-phase deferral pattern's audit risk.

## No Critical findings

No issues require pre-Phase-5 folding beyond the Important I1 awareness item. Phase 4 commit `8ac6847` is sound to keep on `release/v0.27.0`.
