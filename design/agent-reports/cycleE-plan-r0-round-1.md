# PLAN R0 review — mk1-repair-set-level-reverify — round 1

**Verdict: NOT GREEN (0 Critical / 2 Important / 3 Minor)**
**Reviewer:** adversarial opus architect (read-only, cross-repo). Verified @ toolkit `01ebde6b`, mk `main@85bca69`.
**Dispatched:** 2026-07-07 (Cycle E, plan-R0 loop round 1). Persisted verbatim per CLAUDE.md.

The plan faithfully decomposes the R0-GREEN SPEC — phase ordering right, no repair site missed, §4 cells all homed, release sequencing correct, classifier-written-twice sound (public-API chain re-confirmed sufficient in both crates → NO-BUMP holds). Two Important gaps before implementation: the tri-state's plumbing through `repair_card`'s return type + the three callers is unspecified, and the pinned funds-anchor seed form is non-reproducible.

## Coordinator questions — answered
- **Q1 phases/ordering:** correct. P0 (toolkit) before P1 (mk-cli) right despite reverse release order (toolkit links mk_codec library unchanged; P1 pinned seed reuses P0's constant). All 3 `repair_card` consumers covered. But P0 layering under-specified → PI-1.
- **Q2 classifier twice:** correct, no codec coupling. Both crates build the classifier from existing public API (`DecodedString::data()` bch.rs:604; `from_5bit_symbols` header.rs:120 re-exported mod.rs:39; `Chunked{...}` header.rs:45-53; `mk_codec::decode` key_card.rs:158). mk-cli already imports `DecodedString` (repair.rs:25). NO-BUMP genuinely holds. Residual: duplicated funds-logic no anti-drift guard → PM-1.
- **Q3 pinned-seed feasibility:** feasible (~10⁵-10⁶ samples finds one at the ~10⁻⁴-10⁻⁵ rate; "aliases to valid-≠-original" ≈ "fails decode" barring 2⁻³² SHA collision). But the pinned FORM is brittle → PI-2; needs a search cap → PM-2.
- **Q4 §4 cells→phases:** all mapped. §4.5b/§4.6/§4.7/§4.8 correctly homed in toolkit; §4.2 split correctly (mnemonic-repair exit-4 P0, mk-repair exit-5 P1). TDD tests-first + per-phase FULL `cargo test -p` present.
- **Q5 release/vendor:** correct. mk-cli tag→crates.io first, then toolkit v0.80.0 (5 pin refs + self-pin). Pin-with-release satisfied (P1 changes mk-cli). Vendor conditional right; clarify → PM-3.
- **Q6 guard-rails:** G1-G6 good but miss guards for PI-1 (tri-state representation) + PI-2 (seed reproducibility) + PM-1 (existing-test audit).

## PI-1 (Important) — tri-state propagation through `repair_card` return type + 3 callers unspecified
`repair_card` sig is `Result<RepairOutcome, RepairError>` (repair.rs:760) — BINARY, can't carry 3 verdicts:
- **Bless vs Candidate collide in `Ok`.** Both "corrections applied" but Bless→exit 5, Candidate→exit 4+advisory. `RepairOutcome` (repair.rs:437-441) has no verdict discriminant → indistinguishable to callers.
- **`try_repair_and_short_circuit` (repair.rs:1340) would short-circuit a Candidate.** It does `match repair_card {Ok(o)=>emit+RepairShortCircuit{5}, Err(_)=>fall through}`. SPEC §2 requires auto-repair to NOT bless a Candidate → Candidate must be no-short-circuit (like Reject). If Candidate returns as `Ok`, auto-repair short-circuits exit 5 (WRONG — undermines the funds property).
- **Batch aggregation location unstated.** `mnemonic repair --mk1 a0 a1 b0 b1` reaches `repair_card(Mk1,[all])` as ONE call (resolve_groups groups by KIND, not chunk_set_id). The csid sub-grouping + dominant fold must happen inside `repair_card` or cmd/repair.rs — plan doesn't say which; `repair_card` returns one `RepairOutcome` for the whole input.
- **Ripple to ms1/md1 arms + existing matchers** — a new discriminant needs a value for ms1/md1 + touches every `match` on the return.

**Fix (specify before P0):** (a) return-shape change — add a verdict discriminant to `RepairOutcome` (e.g. `set_verify: SetVerify{Blessed, Unverified}`) or return a `RepairVerdict`; (b) `repair_card` computes csid-grouping + dominant fold + surfaces the aggregate verdict (recommended — shared engine), raw corrected chunks still available for the Candidate advisory; (c) explicit 3-consumer map, esp. `try_repair_and_short_circuit`: **Bless→short-circuit(5); Reject→no short-circuit; Candidate→no short-circuit**; (d) default verdict for ms1/md1 arms = Blessed (they already return only on decode success).

## PI-2 (Important) — pinned funds-anchor seed omits `chunk_set_id` → non-reproducible / vacuous
Plan pins `(payload_seed, chunk_index, positions, from→to)` (plan:49). But `encode()` draws a RANDOM `chunk_set_id` (pipeline.rs:45-47, `fresh_chunk_set_id` via getrandom). The csid sits in the 8-symbol chunked header = part of the SAME BCH codeword as the fragment → a different csid changes the codeword, syndrome, and `bch_correct` behavior. Re-encoding under a fresh csid each run → the pinned positional substitution may NOT reproduce the miscorrection → the funds-anchor flakes OR silently regenerates a cleanly-decoding set that asserts nothing (vacuous — SPEC I3 forbids).

**Fix:** (a) pin `chunk_set_id` + encode via `encode_with_chunk_set_id(card, PINNED_CSID)` (pipeline.rs:67, exists), OR (b) cleaner/robust — pin the fully-resolved corrupted chunk STRINGS directly (`const CORRUPTED_SET: [&str; N]`) so the test needs NO re-encode. Update G3 to require deterministic reproduction independent of the OS RNG.

## Minors
- **PM-1** — pre-flag the existing-test audit: full-set miscorrection 5→2, single-chunk `mnemonic repair --mk1` **5→4** (today exits 5 via `indel_exit_code` w/ no indel; `cli_repair.rs`/`cli_auto_repair.rs` carry 13+12 mk1 refs). Most (full-card ≤4 bless; >4-per-chunk `BchUncorrectable` before reassembly) unaffected, but single-chunk-expects-5 flips. Per-phase FULL `cargo test -p` catches these; naming avoids mis-triage. Manual's executed repair goldens are ms1-based (`41-repair-ms1.*`) — NO `mnemonic repair --mk1` golden → P2 mk-repair-only scope adequate; state explicitly. Pinned seed is DUPLICATED into P1's mk-cli test (copy the same constant — same mk_codec aliases identically).
- **PM-2** — `find_mk1_miscorrection_seed` needs an explicit iteration cap (~10⁷ ample) that fails with a "rate lower than assumed — escalate" message rather than hanging.
- **PM-3** — default = DO NOT move the toolkit's `mk-codec` git-dep pin (mk_codec no source change → no re-vendor). The install.sh mk-cli pin advance is a `cargo install` RECOMMENDATION line (5 TEXT refs), NOT a Cargo dependency → zero Cargo.lock/vendor impact. Make "leave it" the default in G6.

## To GREEN
Fold PI-1 (verdict return-shape + 3-caller map, esp. `try_repair_and_short_circuit` Candidate=no-short-circuit, + batch fold location) + PI-2 (deterministic pinned seed) + PM-1/2/3. Localized additions, not a redecomposition; round 2 closes fast.
