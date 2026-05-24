# PER-PHASE REVIEW — v0.37.0 Phase 0+1 (implementation)

**Date:** 2026-05-24
**Reviewer:** feature-dev:code-reviewer (opus)
**Commits reviewed:** `3feb0b5` (template_from_descriptor + tests), `0807b21` (format_requires_template + EmitInputs wiring + test-matrix rewrite)
**Verdict:** GREEN (0 Critical / 0 Important) — 2 non-blocking Minors (both folded in `9a3fe03`)

Reviewed the diff against `master` for `wallet_export/mod.rs`, `cmd/export_wallet.rs`, `tests/cli_export_wallet_from_import_json.rs`; cross-read every emitter + the importer descriptor-rendering for multisig sources + the fixture map to verify the 34/6 truth table by hand.

## Verification results (all pass)
1. **`template_from_descriptor`** — `is_sorted` computed once, full token `sortedmulti(` checked (not `multi(` substring); `Sh(ShInner::{Wpkh,Wsh,Ms})` arms match miniscript-13 post-#915 shape; `Ms(_) → Err` message contains both substrings Cell 3 asserts; no panic/unwrap; Tr/Bare defensively unreachable.
2. **Injection point** — `derived_template` at `:680` after taproot refusal `:643-653` (Tr unreachable); `parsed_ms` `:627` + `threshold` `:673` in scope; only `template` + `threshold_user_supplied` changed (14 other fields untouched).
3. **`format_requires_template`** — exhaustive no-`_`; true-set = exactly the `template.ok_or_else`-refusers (sparrow `:104`, coldcard `:111`, jade `:36`, electrum `:52`, coldcard-multisig guard); false-set leaves bip388/green/bitcoin-core/bsms/specter on passthrough.
4. **Regression safety** — sole reader of `threshold_user_supplied` is `sparrow.rs:43` (fires only under `template.is_some() && is_multisig()`); change is observable only beneficially. Passthrough formats get `None` as before; the pre-existing `f9_from_import_json_bsms_l2_*` test (sh(multi) → bsms) still passes because bsms is in the false-set.
5. **Test fidelity** — p11c truth table matches real emitter behavior cell-by-cell (6 refuse = 3 singlesig × {coldcard-multisig, jade}; 34 succeed); p11d needles all load-bearing (electrum `"wallet_type": "standard"`, coldcard `p2wpkh`, coldcard-multisig/jade `Format: P2WSH`, sparrow `wpkh`/`wsh`); p11e guards the partition; jade-singlesig pattern matches `jade.rs:61` verbatim; multisig sources all render `wsh(sortedmulti(...))` → `WshSortedMulti`.
6. **Conventions** — no new ToolkitError variants; docstrings don't trip `doc_lazy_continuation`.

## Minor (folded in `9a3fe03`)
- **M1** — p11e is weaker than spec §5.5 (no byte-snapshot; would miss a bip388-into-inject-set bug producing different-but-nonempty output). **Fold:** added `format_requires_template_tests::partition_is_exact` — source-of-truth partition guard.
- **M2** — unit tests omitted `bare()→Err` (doubly-unreachable, hard to construct, cosmetic) and only exercised `sh(sortedmulti)` not `sh(multi)` for the P2shMulti Err path. **Fold:** added the `sh(multi)` Err case (the `bare()` case left as documented cosmetic).

## VERDICT
**GREEN (0C/0I).** Implementation faithfully realizes the R0-GREEN spec. The 2 Minors were test-strength observations (folded), not correctness defects.
