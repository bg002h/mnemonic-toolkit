# Phase H — schema-4 cutover review — r1

**Date:** 2026-05-05
**Commits under review:** `67391b0`, `58baebe`, `cadfc4f` (delta `e8ca2e6..HEAD`)
**Reviewer:** feature-dev:code-reviewer (sonnet-4-6)
**Verdict:** BLOCK — 0C / 2I / 1L. Fixed in r2 commit; re-review pending.

## Important

**I-1 — `path_family: "bip87"` hardcoded in `emit_unified` for all multisig templates.** `cmd/bundle.rs:1616` sets `path_family: "bip87"` unconditionally; legacy `emit_multisig:872` correctly uses `args.multisig_path_family.unwrap_or_default().human_name()`. A `--template sh-wsh-sortedmulti --slot @0.phrase=...` invocation emits `"path_family": "bip87"` instead of `"bip48"`, breaking SPEC §5.6 cross-schema invariant for BIP-48 recovery tooling.

**I-2 — `synthesize_unified` has zero unit tests; plan H.1 required ≥8.** `synthesize_unified` is the sole synthesis entry for all five `BundleMode` variants. Four binary-level integration tests in `cli_unified_slot.rs` cover only the phrase→single-sig-full path. SingleSigWatchOnly, MultisigMultiSource, MultisigHybrid, MultisigWatchOnly shapes are unit-untested.

## Low (routed to FOLLOWUPS)

**L-1** — `origin_path: Some("")` vs `null` divergence between `emit_unified` and legacy `emit` for xpub-only single-sig slot without `--slot @0.path=`. Route to FOLLOWUPS as `unified-slot-xpub-missing-path-origin-path-null` at `v0.4.2-nice-to-have`.

## Verified

- H.1 Bundle/BundleJson migration completeness — no stale `Option<String>` on Bundle.ms1/BundleJson.ms1; all 5 producers correctly emit length-N MsField.
- H.1 mode_str consistency — all four emit sites use `bundle.any_secret_bearing()`.
- H.2 JSON test assertions — zero `schema_version.*"3"` matches in `.rs` files.
- H.5 resolve_slots phrase path — derivation chain byte-identical to `derive_full`.
- H.5 BIP-388 distinctness duplication safe; FOLLOWUP `cosigner-keyinfo-resolved-slot-merge` tracks merge.
- H.5 dispatch ordering correct (template-N checks gated by `if let Some(t) = args.template`).
- H.5 emit_unified single-sig — origin_path matches legacy emit for equivalent inputs.
- H.6 path_raw fallback `path.to_string()` correct at all CosignerKeyInfo construction sites.
- H.7 SPEC §4.11.b clarification unambiguous.

## Verdict

**BLOCK at r1; APPROVE after I-1 + I-2 fixes (see r2 commit).**

## r2 closure

**Delta:** commit `c1ffffd`. Verdict: APPROVE — 0C / 0I / 0L. Terminates Phase H iterative-review loop.

- I-1 closure: cmd/bundle.rs:1620 derives `path_family` from `args.multisig_path_family`; sh-wsh-* templates correctly emit `"bip48"`.
- I-2 closure: 7 new unit tests in synthesize.rs::tests cover all five BundleMode shape variants + threshold-out-of-range error path + BundleJson schema_version pin. Coverage adequate.
- L-1 closure: FOLLOWUPS entry `unified-slot-xpub-missing-path-origin-path-null` filed at `v0.4.2-nice-to-have`.
- No new findings.
