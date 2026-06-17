# Plan-R0 (Phase-0 test-hardening B1–B4) round 1 — architect review (verbatim)

> Persisted verbatim per CLAUDE.md BEFORE the fold. **Verdict: NOT GREEN — 0 Critical / 2 Important.**
> Both Importants are premise-invalidations, not soundness errors. B1 positive leg + B2 + B4 SOUND.
> **I-2 headline: B3 (ms-codec themes 1/2/3) is ALREADY SHIPPED** in `bch_all_lengths.rs`. Plan SHA:
> toolkit `65affc6`. Architect verified across 3 repos with exact line cites.

---

## Verdict: NOT GREEN — 0C / 2I

### B1 — STRESS-A taproot leg — SOUND (positive leg); negative-cell citation WRONG (I-1)
- wsh-only today CONFIRMED (`prop_backup_restore_roundtrip.rs:251` wrapper=wsh; KEYS `:28-34`; helpers bundle_md1 `:294`/restore `:320`/derive_receive `:383`/normalize `:356`; build_descriptor wsh-only `:265`; negative cell `:556`).
- Feeding concrete `tr(<NUMS_HEX>,…)` to `bundle_md1` (bypass build_descriptor) SOUND + proven: `cli_restore_taproot.rs:46-74` does exactly this; raw NUMS hex recognized at `parse_descriptor.rs:469` (`== NUMS_H_POINT_X_ONLY_HEX`); restore reconstructs + reports first_addresses (`cli_restore_multisig.rs:257/:281`); normalize/derive_receive handle multi_a/sortedmulti_a on the `95fdd1c` fork (route-around per `derive_address.rs:72-75`).
- KEYS pool usable for tr multi_a (5 distinct-fp keys, k≤n≤5; walk_tr accepts NUMS-internal tr w/ multi_a/sortedmulti_a leaf). Must `--network mainnet` (helpers already do).
- **I-1:** the plan cites `cli_restore_multisig.rs::at_in_both_*` — **no such test there.** @-in-both coverage is in `cli_restore_taproot.rs`: `build_at_in_both_descriptor` (`:347`, builds md1 DIRECT via md_codec tree types + `chunk::split`) + `at_in_both_tr_refuses_structurally` (`:447`, n=3 the real RED), `_2of2_` (`:472`), `_sortedmulti_a_` (`:489`). So md_codec-direct build IS feasible AND the negative coverage is already comprehensive at n≥3. The plan's negative cell would be REDUNDANT → SKIP (unconditionally, not "if heavy"). 2-of-2-coincidental-k>n nuance already at `cli_restore_taproot.rs:440-442`. **Fix:** correct the citation + make SKIP unconditional. Positive leg unchanged.
- Non-vacuity SOUND (revert v0.55.x tr reconstruction → O1/O3 fail). Coverage-counter (both multi_a + sortedmulti_a) is the anti-vacuity guard. NO-BUMP holds.

### B2 — `arm_dup_if` de-stub — SOUND (0 findings)
- `arm_dup_if` `#[ignore]` empty stub `:2420-2423`; sibling `arm_non_zero` `:2426-2432`; helpers `wsh_inner` `:2304`, `find_tag` `:2313`, `H20` `:2301`; `walk_one_child` `:705` → `Body::Children(vec![…])` so the `matches!` assert is correct.
- `wsh(or_i(pk(@0/<0;1>/*),dv:older(144)))` type-checks + produces `Terminal::DupIf` (traced pinned `correctness.rs`: older→B/Zero; v:→V/Zero; d:v:→cast_dupif needs V+Zero, both met → B/OneNonZero; or_i(c:pk_k B, DupIf B) valid). Walker arm under test `:668`.
- Non-vacuity SOUND (drop `:668` → `.expect("DupIf")` panics). Bin-crate placement correct. NO-BUMP holds.

### B3 — ms-codec themes 1/2/3 — **I-2: PREMISE INVALIDATED — ALREADY SHIPPED**
The plan's premise "md+mk shipped all three; ms is the gap" is FALSE. `crates/ms-codec/tests/bch_all_lengths.rs` (committed) implements all three, labeled:
- **Theme 1** `corrects_1_to_4_errors_every_length` (`:104-128`): 1-4 errors → recovery + position-set match, all 5 lengths. (Also `mnem.rs::mnem_decode_with_correction_recovers_from_corruption` `:94`.)
- **Theme 2** `five_to_eight_errors_never_return_original_every_length` (`:130-164`): 80 trials × 5-8 errors × 5 lengths, `assert_ne!` original never silently returned. Multi-length already.
- **Theme 3** `raw_wrong_length_fails_closed_every_length` (`:166-198`): insert/delete → `Err(TooManyErrors|UnexpectedStringLength)`, via public `decode_with_correction` (no reassemble/split needed — ms's length-rule-9 gate `decode.rs:44` + re-decode do it). Inline comment cites toolkit `Ms1IndelOracle`.
- `decode_with_correction` at `decode.rs:237` (exported `lib.rs:53`). The re-verify polymod branch is at **`decode.rs:280-288`** (plan's `:231-238` is wrong — that's the doc/sig). `repair.rs::Ms1IndelOracle` is at `:919` (plan's `:884` wrong). `bch_all_lengths.rs` header cites `design/BUG_decode_with_correction_length_divergence.md` + the v0.2.1 fix — post-dates the plan's premise.
- **Fix (I-2):** RESCOPE or DROP B3. Re-survey `bch_all_lengths.rs` + `mnem.rs` + `parity_smoke.rs` + `uppercase_envelope.rs`; either (a) target a proven-residual gap (a DETERMINISTIC test forcing the `:280-288` re-verify branch specifically — a 5+-error pattern yielding a degree-≤4 locator with 4 valid roots whose re-encode fails polymod; and/or a proptest form), or (b) resolve B3 as already-satisfied (no new test, file the slug as satisfied). As written it re-implements shipped tests.

### B4 — md-codec bitcoind corpus breadth — SOUND (0 findings)
- `Shape{label,desc}` `:103`, `corpus()` `:112`, 10 shapes confirmed. `pkk(idx)` `:89` → `Tag::PkK` auto-normalized to `Check(pk_k)` B-type at `to_miniscript.rs:290-298`. All proposed tags exist + render: Multi `:402`, Sha256 `:437`, After `:429`, OrD `:380`, AndOr `:361`; `Miniscript::from_ast` `:478` type-checks. Per-shape: (1) plain multi mirror row 5 SOUND; (2) and_v(v:pk,sha256) mirror row 9 SOUND; (3) and_v(v:pk,after(800000)) <500M block-height SOUND; (4) or_d(c:pk_k,and_v(v:pk,older)) SOUND; (5) andor optional SOUND. NO SortedMultiA (correctly excluded; `to_miniscript.rs:423-428` hard-errors). Non-vacuity = the differential oracle. NO-BUMP holds.

### Required folds
1. **I-1 (B1):** fix negative-cell citation → `cli_restore_taproot.rs::at_in_both_*` (`:447/:472/:489`) + `build_at_in_both_descriptor` (`:347`); SKIP unconditional.
2. **I-2 (B3):** rescope/drop — re-survey `bch_all_lengths.rs` (themes at `:105/:135/:175`); correct dead-branch cite (`decode.rs:280-288`) + `repair.rs:919`; either target a proven residual or resolve already-satisfied.

Re-dispatch after folding.
