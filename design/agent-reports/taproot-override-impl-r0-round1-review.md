# #26 impl execution review — taproot tr(NUMS,multi_a) use-site override restore (opus architect, verbatim)

> Mandatory per-phase R0 over the IMPLEMENTED diff (`feature/restore-taproot-use-site-override`, commits `0b0f807`+`e5698be` off `master`==`09ab04c`). Reviewer ran the suite + clippy + decoded fixtures + ran restore on the live fixture end-to-end. **Verdict: GREEN — 0 Critical, 0 Important.** 2 non-gating Minors. Cleared for P2.6 (version 0.59.1 + manual prose + ship).

---

**Verdict: GREEN — 0 Critical, 0 Important.**

## Verified correct

**The single predicate `restorable_taproot_override_card` (`restore.rs:1497-1519`) — all four conjuncts correct.** `taproot_override_card(d)` → `!has_hardened_use_site(d)` → `match &d.tree.body { Body::Tr { is_nums: true, tree: Some(inner), .. } => inner.tag == Tag::MultiA, _ => false }`. The NUMS/leaf destructure is BYTE-IDENTICAL to `classify_taproot_restore`'s NUMS arm (`:1089-1093`) — cannot diverge (R0 Min-B). `tree:None`/non-NUMS/non-`Tr` → `_ => false`. Admits ONLY `MultiA` (rejects `SortedMultiA`). No unwrap/panic on card input.

**§3 parity invariant — structurally closed on ONE expression.** Guard (`:1687`): `taproot_override_card(&d) && !restorable_taproot_override_card(&d)`. Classify-reroute (`:1716`): `is_taproot && restorable_taproot_override_card(&d)` → `(None, Some(Nums))`. Advisory (`unrestorable_advisory.rs:114-116`): `taproot_override_card(desc) && !restorable_taproot_override_card(desc)`. Guard runs BEFORE classify (`:1687` precedes `:1730`); hardened guard (`:1679`) precedes both. guard-admits ⟺ classify-reroutes ⟺ advisory-silent, partitioned exactly on the predicate. sortedmulti_a/non-NUMS/hardened all fail R → refuse + advise. No silent-wrong-address hole.

**Classify-reroute correctness.** Admitted card → `template_opt=None` (`:1728`) → faithful arm → `to_miniscript_descriptor_multipath` (per-`@N`), NOT the `Template` string-builder. Non-override `tr(multi_a)` skips the reroute → `Template` fast path UNCHANGED. `refuse_at_in_both` (`:1167`) fires only for `Cosigner` (non-NUMS) trunks; an admitted card is `is_nums:true` (immune), a non-NUMS @-in-both card fails R → falls to classify → refuses there as before. Crux preserved.

**Funds-safety golden (P2.5) — non-vacuous, NOT self-referential; verified end-to-end.** `cli_restore_multisig_general.rs:556` asserts the reconstructed string keeps `<0;1>/*` (@0) AND `<2;3>/*` (@1, divergent, NOT collapsed). Reviewer ran restore on the live fixture: output `tr(50929b74…,multi_a(2,[73c5da0a]/<0;1>/*,[b8688df1]/<2;3>/*))`, ADDR0 == const `DIVERGENT_TR_MULTI_A_CHAIN0_IDX0_GOLDEN` (`:687`); internal key = BIP-341 NUMS H-point. Anti-vacuity `divergent_taproot_golden_differs_from_baseline_and_anchors` (`:746`): `assert_ne!(divergent_addr, baseline_addr)` where baseline is the `<2;3>→<0;1>` hand-edit → a silent suffix-collapse FAILS regardless of the const's provenance. `@1` genuinely divergent. Neither test `#[ignore]`; both pass. The (A) bitcoind differential `tr-nums-multi_a-2of3-divergent` row ran vs Core v27.0: 55 receive-checks byte-identical.

**Advisory hardened∩taproot co-fire (Min-2).** `cli_unrestorable_shape_advisory.rs:208` asserts restore `.failure()` AND `stderr.contains(ADVISORY_PREFIX)` (≥1, not "exactly one"); passes.

**Non-regression.** #25 wsh/sh non-taproot override restore unchanged. Non-override taproot stays `Template`. The `prop_backup_restore_roundtrip.rs:633` flip is CORRECT: `tr(NUMS,multi_a(2,@0/<0;1>/*,@1/<2;3>/*))` removed from the refusal list (now genuinely restorable — proven by the positive test + live decode), 3 genuinely-refusing shapes added; `.failure()` assertion intact. Negative fixtures genuinely refuse (independent CLI check: sortedmulti_a → exit 2 "a sortedmulti_a tap leaf…"; non-NUMS → exit 2 "non-NUMS internal/trunk").

**Regression sweep.** `cargo test -p mnemonic-toolkit`: 3146 passed, 0 failed, 13 ignored / 178 suites, all ok. `cargo clippy -p mnemonic-toolkit --tests`: clean. `mlock.rs` NOT in the diff. No over-broad match (reroute gated `is_taproot && restorable_…`; `_ => false` predicate arm conservative).

## CRITICAL
None.
## IMPORTANT
None.
## MINOR
- **M-1 (cosmetic).** `restore.rs:1716` `is_taproot && restorable_…` — `is_taproot` is logically redundant (the predicate's first conjunct implies `Tag::Tr`). Harmless early-out, keeps the ladder symmetric. No change.
- **M-2 (test-hygiene).** The golden const is pinned to restore's current output via `assert_eq!(divergent_addr, const)`; NOT vacuous — the load-bearing guarantee is the independent `assert_ne!(divergent_addr, baseline_addr)`. Same pattern as #25's wsh golden. No action.

## To turn GREEN
Already GREEN. Ships gate-clean. P2.1-P2.5 faithfully implemented per the GREEN plan; funds-safety oracle non-vacuous + verified end-to-end. (P2.6 ship deferred as scoped.)
