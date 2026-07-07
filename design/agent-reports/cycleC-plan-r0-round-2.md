# PLAN R0 review — bip388-double-star-shorthand-support — round 2

**Verdict: NOT GREEN (0 Critical / 1 Important / 0 Minor)**
**Reviewer:** opus architect, source basis `0964462d` + live-binary verification.
**Dispatched:** 2026-07-06 (Cycle C, IMPLEMENTATION_PLAN R0 loop round 2). Persisted verbatim per CLAUDE.md.

Round-1 I1 (BSMS canonicalize soft-gap), export-wallet IN, gui-schema reclassification, and M1-M3 all correctly folded + independently re-verified. Round-2 completeness sweep found NO further missed user-`/**` surface. Single new blocker: **compare-cost scoped IN on a false premise** — it rejects multipath `/<0;1>/*`, so the expander delivers no usable `/**` acceptance and §7.11's compare-cost cell is un-satisfiable as written.

## Important

### I3 — compare-cost IN scoping based on a false premise; §7.11 "accepts" cell un-satisfiable
Empirically verified (live binary):
```
compare-cost --descriptor "wsh(multi(1,[…]xpub…/<0;1>/*))"  → error: multipath key cannot be a DerivedDescriptorKey
compare-cost --descriptor "wsh(multi(1,[…]xpub…/0/*))"       → Feerate: … (accepts)
```
`translate_descriptor` (cost/strip.rs:26-27) calls `derive_at_index(0)` directly with NO multipath split (no `into_single_descriptors`/`is_multipath` in cost/), and miniscript rejects `derive_at_index` on a multipath key. So compare-cost does NOT accept `/<0;1>/*` at all — a pre-existing limitation orthogonal to `/**`. Expanding `/**`→`/<0;1>/*` at cost/strip.rs:21 changes reject→different-reject; §7.11's "accepts == /<0;1>/*" cell is un-satisfiable (`/<0;1>/*` exits non-zero). SPEC §0 item 5's "HARD-reject `/**` … scoped IN so accepting is consistent" is factually wrong for compare-cost.

Contrast: **export-wallet is correct** — `export-wallet --descriptor "…xpub/<0;1>/*"` ACCEPTS (emits `…/<0;1>/*#csum`), so expanding at export_wallet.rs:517 genuinely delivers acceptance; §7.11 export-wallet cell valid. (export-wallet's `is_at_n_form` reject@508 means only CONCRETE `/**` reaches :517 — AtN `@0/**` stays rejected by design; §7.11 correctly uses a concrete xpub.)

**Fix — pick one:**
- **(A) Scope compare-cost OUT** (revert). File the pre-existing multipath gap as a separate FOLLOWUP. Keep export-wallet IN. Cleaner.
- **(B) Keep compare-cost IN, reframe** §7.11 as strict EQUIVALENCE (`/**` output/exit byte-identical to `/<0;1>/*` — both the multipath-reject), NOT "accepts"; correct SPEC §0 item 5 to state compare-cost still rejects `/**` (via the pre-existing multipath limitation) but now IDENTICALLY to `/<0;1>/*`. Preserves the §6 equivalence invariant honestly (error-consistency only).
- NOT (C) make compare-cost split multipath — separate feature, out of scope.

## Verified resolved (round-1 folds + completeness)
- **I1 → RESOLVED.** `recanonicalize_descriptor` (roundtrip.rs:231, before from_str@241) in the IN set. Independently confirmed line 241 is the ONLY production `from_str` in roundtrip.rs (34/222/233 doc-comments, 1631 test), reached only by `canonicalize_bsms`(96) + `canonicalize_bitcoin_core`(170). The "audit sibling canonicalize_*" will correctly come up empty (coldcard/electrum/sparrow/specter/jade do no raw-body miniscript parse) — IN set complete for that family.
- **I2 export-wallet → RESOLVED & correct** (accepts multipath). **gui-schema:1319 → correct** (chokepoint-covered). OUT rationale split accurate.
- **M1/M2/M3 → folded.**
- **Final user-`/**` sweep → no further missed site.** Full `--descriptor` clap-field enumeration = `bundle`, `verify-bundle`, `export-wallet`, `compare-cost`, `xpub-search`, `gui-schema --classify-descriptor`, `import-wallet` — all IN. Other `descriptor: String` structs are serde OUTPUT rows (`word_card.rs:511` display-only, `nostr.rs:120` built, `restore.rs:287` reconstructed) — OUT-correct.
- **Scope coherence:** SPEC §0 rev-4 IN items 1-7 + scope-principle match the plan IN set (modulo compare-cost correction).
- **New-drift:** export-wallet/compare-cost expanders no-op on non-`/**` (borrowed Cow) → zero non-`/**` change. bsms.rs:300/roundtrip.rs:231 route `/**`→`/<0;1>/*` into already-multipath-capable paths → safe. Only compare-cost carries I3.

**To GREEN:** apply I3 (A or B), update SPEC §0 item 5 + plan §7.11 in lockstep, re-dispatch a scoped convergence check on the compare-cost decision. Everything else GREEN.
