# PLAN — Phase 0 test-hardening (B1–B4), 2026-06-16

> Consolidated R0-gate plan for the 4 Tier-1 test-hardening items from
> `design/PLAN_remaining_open_items_tiered_2026-06-16.md`. All test-only / NO-BUMP,
> across 3 repos (independent → parallel impl, separate commits). Source SHAs:
> toolkit `65affc6`, descriptor-mnemonic `origin/main` (re-verify), mnemonic-secret
> `origin/master` (re-verify). Each item is implemented TDD (the test IS the deliverable);
> non-vacuity = the test goes RED if the property it guards breaks. Per-item impl review
> to 0C/0I after.

---

## B1 — STRESS-A taproot leg (toolkit, `tests/prop_backup_restore_roundtrip.rs`)

**Gap:** the property harness is wrapper-`wsh`-only (`build_policy` emits `{"wrapper":"wsh"}` at `:251`; `negative_property_…` at `:556` covers only wsh `@N` shapes). The toolkit-UNIQUE `tr(NUMS,{multi_a|sortedmulti_a})` reconstruction (v0.49.1/v0.55.x, route-around md-codec) is covered only by fixed goldens in `cli_restore_multisig.rs`, never property-tested.

**Add (positive tr leg):** a proptest cell that GENERATES concrete `tr(<NUMS_HEX>,{multi_a|sortedmulti_a}(k, K0..Kn))` descriptor STRINGS directly (NOT via `build_descriptor`, whose `WrapperKind` is wsh-only by design) from the existing origin-annotated `KEYS` pool, with randomized k (1..=n), n (2..=5 within KEYS), and sorted-vs-unsorted variant. Enter the EXISTING pipeline: `bundle_md1(desc)` → `restore(md1)` → assert the 3 oracles already in the file:
- **O1 structural** — `normalize(reconstructed) == normalize(original)` (the existing `normalize` erases xpub bodies + strips checksum; tr/multi_a/sortedmulti_a parse on the workspace `95fdd1c` fork).
- **O2 fixed-point** — re-bundle the reconstruction reproduces the md1 (the existing helper pattern).
- **O3 address** — `derive_receive(original, N)` == restore's reported `first_addresses` (independent rust-miniscript derivation; the existing `derive_receive` helper).

**Negative cell — SKIP (R0-r1 I-1): already comprehensively covered.** The `@`-in-both /
non-NUMS-tr loud-refusal is fully tested in `cli_restore_taproot.rs`: `build_at_in_both_descriptor`
(`:347`, builds the md1 DIRECTLY via `md_codec` tree types + `chunk::split` — bundle rejects
`@`-intake) drives `at_in_both_tr_refuses_structurally` (`:447`, n=3 2-of-3 MultiA — the real
structural RED), `at_in_both_tr_2of2_refuses_structurally` (`:472`), and
`at_in_both_sortedmulti_a_refuses_structurally` (`:489`). The 2-of-2-is-coincidental-k>n nuance is
already documented at `cli_restore_taproot.rs:440-442`. A new proptest negative cell would be
redundant → do NOT add one. **B1 = the positive tr-leg proptest only.**

**Non-vacuity:** revert v0.55.x's tr reconstruction (or the route-around) → O1/O3 fail + shrink to a minimal tr shape. Generator must cover BOTH multi_a and sortedmulti_a (assert a coverage counter, like the existing `generator_covers_all_fragments`).

**Tier:** test-only NO-BUMP. **File the missing FOLLOWUP slug** `stress-a-taproot-leg` (resolved on ship). ~120–180 LOC.

## B2 — `arm_dup_if` de-stub (toolkit, `parse_descriptor.rs` test mod)

**Gap:** `arm_dup_if` (`:2420`) is `#[ignore]` with an empty body + a DISPROVEN reason ("DupIf descriptor-unreachable" — `wsh(or_i(pk(X),dv:older(144)))` parses on pinned miniscript 13.0.0). De-ignoring alone = vacuous (empty body).

**Implement:** mirror the sibling `arm_non_zero` (`:2426`) EXACTLY — remove `#[ignore]`, write:
```rust
fn arm_dup_if() {
    // dv:X is Terminal::DupIf(Terminal::Verify(X)). or_i lets DupIf appear.
    let s = "wsh(or_i(pk(@0/<0;1>/*),dv:older(144)))";
    let inner = wsh_inner(s);
    let n = find_tag(&inner, Tag::DupIf).expect("DupIf");
    assert!(matches!(n.body, Body::Children(_)));
}
```
(`Terminal::DupIf(i) => walk_one_child(Tag::DupIf, i, km)` at `:668` is the walker arm under test.)

**Non-vacuity:** if the `DupIf` walker arm were dropped, `find_tag(Tag::DupIf)` → None → `.expect()` panics. Confirm the descriptor parses + walks at write time (run the test). Bin-crate test (`cargo test --bin mnemonic`, NOT `--lib`).

**Tier:** test-only NO-BUMP. Resolves `toolkit-arm-dup-if-ignored-stub` (open). 1 test.

## B3 — ms-codec themes 1/2/3 — ALREADY SATISFIED (R0-r1 I-2), no new test

**Premise invalidated:** the codec-test-hardening recon's "ms is the gap" is FALSE. `crates/ms-codec/tests/bch_all_lengths.rs` (committed, the v0.2.1 BCH fix-lock hardening) already implements all three themes, labeled:
- **Theme 1** `corrects_1_to_4_errors_every_length` (`:104-128`) — 1-4 errors → recovery + reported positions match, all 5 entropy lengths. (Also `mnem.rs::mnem_decode_with_correction_recovers_from_corruption`.)
- **Theme 2** `five_to_eight_errors_never_return_original_every_length` (`:130-164`) — 80 trials × 5-8 errors × 5 lengths, `assert_ne!` the original is never silently returned.
- **Theme 3** `raw_wrong_length_fails_closed_every_length` (`:166-198`) — insert/delete → `Err(TooManyErrors|UnexpectedStringLength)` via public `decode_with_correction` (the indel reject-contract the toolkit `repair.rs::Ms1IndelOracle` `:919` relies on).

**Disposition:** B3 = **already satisfied**. No new ms-codec test. On wrap-up, file `ms-codec-test-hardening-themes` (or note in the codec-test-hardening recon) as ALREADY-SATISFIED with these three cites. The one theoretical residual — a DETERMINISTIC test forcing the defensive re-verify branch at `decode.rs:280-288` (a 5+-error pattern yielding a degree-≤4 locator with 4 valid roots whose re-encode fails polymod) — is **deferred / low-value**: the PROPERTY that branch guards (5-8 errors never silently return original) is already asserted by the theme-2 sweep, and constructing the specific input deterministically is hard for negligible added assurance. No code in this Phase.

## B4 — md-codec bitcoind corpus breadth (descriptor-mnemonic, `crates/md-codec/tests/bitcoind_differential.rs`)

**Gap:** the corpus is 10 shapes (`corpus()` `:112`, `Shape{label,desc}` `:103`); confirmed-absent: plain (unsorted) `multi`, all hashlocks, `after`, `or_d`/`andor`. Slug `bitcoind-differential-corpus-breadth` (open).

**Add 4–6 `Shape` rows** (each ~30–45 LOC of md-codec `Descriptor`/TLV construction mirroring an existing row of the same family):
- `wsh(multi(2,…))` plain unsorted — mirror the `wsh(sortedmulti …)` row, `Tag::Multi` not `SortedMulti`.
- `wsh(and_v(v:pk(…),sha256(<h>)))` — a hashlock (mirror the `and_v(v:pk,older)` row, swap `older`→`sha256`).
- `wsh(and_v(v:pk(…),after(800000)))` — absolute timelock (swap `older`→`after`).
- `wsh(or_d(pk(…),and_v(v:pk(…),older(144))))` — an `or_d` combinator.
- (optional) `wsh(andor(pk,pk,older))`.
- **NO `SortedMultiA`** (md-codec's crates.io miniscript 13.0.0 can't render it — A1 upstream block); each new shape must be in the `to_miniscript`-renderable ∩ Core-v27-sane intersection. The test's `#[ignore]`+env gate is unchanged.

**Non-vacuity:** the differential itself is the oracle (md-codec vs Core byte-equality); a new shape that md-codec mis-derives → the existing FUNDS-CRITICAL assertion fails. Each new row needs one local derivability check (the test already runs against a live node in CI).

**Tier:** test-only NO-BUMP. Resolves `bitcoind-differential-corpus-breadth`. ~150–250 LOC. Runs in md-codec's existing daily-cron bitcoind workflow (no workflow change).

---

## Execution

Independent across 3 repos → implement in any order / parallel; separate per-repo commits.
Per-repo: TDD (write the failing/characterization test), confirm non-vacuity, run the repo's
gates (`cargo test` + `clippy -D warnings` + the repo's fmt rule — md/ms/mk stable fmt, toolkit
1.95.0 + mlock exempt), per-phase impl review to 0C/0I (persist to `design/agent-reports/`),
commit (NO-BUMP), push. B3/B4 are sibling repos (no crates.io publish needed — test-only).
Sequence suggestion: B2 (trivial) → B4 (mechanical) → B1 (headline) → B3 (heaviest; theme-2 may
spawn a real-bug PATCH).
