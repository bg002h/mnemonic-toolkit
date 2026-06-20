# P1 — Permutation-search engine — mandatory per-phase R0 EXECUTION review

**Scope:** implemented diff on `feature/bundle-md1-template-multisig` (`29bbfb53`, 1 commit off `master`).
`src/permutation_search.rs` (new, 1125 lines incl. 28 tests) + `lib.rs` mount. mlock.rs untouched.
**Plan:** `IMPLEMENTATION_PLAN_bundle_md1_template_multisig_2026-06-20.md` §2. **SPEC:** `…SPEC…` §6 + §7 floors 2/5.
**Reviewer:** opus architect, adversarial, source-verified + tests/clippy run + independent oracle probes.

**Verdict: GREEN — 0 Critical, 0 Important.**

---

## Verified correct

**Tests + clippy (run by reviewer).**
- `cargo test -p mnemonic-toolkit --lib permutation_search` → **28 passed; 0 failed**.
- `cargo test -p mnemonic-toolkit --lib` (full regression sweep) → **156 passed; 0 failed; 3 ignored**.
- `cargo clippy -p mnemonic-toolkit --lib --tests -- -D warnings` → **clean** (Finished, 0 warnings).

**1. Unique-vs-Ambiguous (the load-bearing property) — CORRECT.** `search:601-612`. A match is pushed to the thread-`local` Vec, THEN `global_matches.fetch_add`; only at global count ≥2 does the thread set `stop` and break. `local` is unconditionally flushed to the shared Mutex Vec after the loop (`:614-616`) on every exit path (exhaustion or break). The final decision reads `found.len()` from the Mutex Vec AFTER `thread::scope` joins (`:621-632`) — a full happens-before barrier — so `Relaxed` on the counter/flag is sound (the counter only gates the early-stop optimization; correctness rides on the joined Vec length). `Unique` (`:624`) is returned ONLY for `found.len()==1`, i.e. after a full scan proved no 2nd match. No early-terminate on first match.
  - **(a) Unique only after no-2nd-match:** yes — full scan unless ≥2 short-circuit. `:622-632`.
  - **(b) Race that misses a 2nd match → false Unique:** disproven. Reviewer ran a faithful port under 16,000 adversarial 2-match runs (n=7, 20 threads; rank pairs at shard boundaries, adjacent ranks, the 1024-poll boundary, far ends) → **0 false-Unique**. A skipped 2nd match implies count already ≥2 elsewhere → `Ambiguous` regardless. `stop` only truncates *unevaluated* candidates.
  - **(c) Determinism oracle non-vacuous:** reviewer injected a deliberate early-terminate-on-first-match bug into the same harness; over a 2-match input the buggy variant returned `Unique([0,1,2,3,4,5])` while the reference returned `Ambiguous` — **the oracle distinguishes them**. Tests `engine_two_targets_is_ambiguous:828`, `parallel_matches_reference_none_and_ambiguous:876` are therefore non-vacuous and WOULD fail an early-terminate regression. `search_reference:639-688` is a genuine full-scan reference (stops only at its own 2nd match, semantically identical outcome).

**2. Parallel permutation coverage — EXACTLY ONCE, no gap/overlap.** `search:569-619`. `chunk = total.div_ceil(nthreads)`; shard `t = [t*chunk, min((t+1)*chunk, total))`; `if start >= total { break }` truncates surplus shards. Reviewer exhaustively verified the union == `[0,total)` exactly once for every `total ∈ [1,130) × nthreads ∈ [1,25)` incl. non-divisible last-shard and `nthreads > total` → **0 failures**. `unrank_permutation:485-496` (Lehmer/factorial-number-system) is a bijection `[0,n!) → S_n`: test `unrank_covers_all_permutations_bijectively:910` (n=5, all 120 distinct + valid) passes and is non-vacuous. `idx/perms`, `idx%perms` (u128) correct; `(idx/perms) as u64` lossless for legal range (outer_k < outer_count ≤ 8.59e9 < u64::MAX, reviewer-checked).

**3. `required_prefix_bytes(S) = ceil((log2(S)+32)/8)` — EXACT.** `:313-328`. The impl's `128 - (S-1).leading_zeros()` == `ceil(log2 S)` for S>1 (reviewer verified at boundaries 2,3,4,5,7,8,9,2^20,2^20+1); S≤1 clamped to 0 bits. Reviewer independently recomputed the full ladder: `11!`→8, `K=8`→9, `K=16`→10, `K=32`→11, `K=64`→13 — all match (tests `prefix_ladder_*:712,720`). `S=0/1`→4, `S=2`→5 (`prefix_floor_small_spaces:738`). `validate_prefix_strength:333` rejects short / accepts ≥required (`:748`). No overflow (u128 throughout; `div_ceil`).

**4. `reject_duplicate_keys` (floor 2) — CORRECT.** `:280-289`. All-pairs O(n²), returns the FIRST colliding pair `(a,b)` with `a<b`. Generic `T: PartialEq` (raw 65-byte blobs in P3). No missed pair. Tests `:766,779`. Correctly does not over-reject same-`@N` multi-leaf reuse (that is one supplied key, SPEC §7 floor 2).

**5. Adaptive cap — CORRECT and un-bypassable.** `cap_decision:379-406`. Estimate = `per_candidate × total_candidates` over the FULL realized space (caller passes realized S — test `cap_estimate_…:1089` feeds `13!`). `<30s`→silent; `≤1h`→progress; `>1h`→refuse unless `accept_search_time ≥ estimate` (`:396`); below-estimate override → `AcceptSearchTimeTooLow` (`:397`); no override → `SearchTimeExceedsCeiling` (`:401`). The override gate compares against the realized estimate, so it cannot be bypassed with a token value. `checked_mul_u64:415-425` saturates to `Duration::MAX` on overflow (→ stays above ceiling → refuses) — fail-safe. Tests `:1027-1118` cover all four branches + a calibrated synthetic-slow path.

**6. Ascending-address-index-OUTER — CORRECT.** Flatten is `idx = outer*n! + perm_rank` (`search:594-595`, `search_reference:658-659`), index slowest. Reviewer confirmed all `n!` perms of address-index 0 precede any of index 1 (addr=3 candidates at idx∈[18,24), addr=18 at idx∈[108,114) for n=3) → a low-index target is reached first. `AddressRange::flatten:223-231` is index-OUTER/chain-INNER, monotonic non-decreasing idx (test `address_range_flatten_is_ascending_index_outer:963`). `address_search_finds_low_index_match:929` asserts the exact `(perm, address_index)` → non-vacuous. Note the engine still completes/short-circuits-at-2 to honor uniqueness; ordering is a find-first-fast property, not a stop-at-first.

**7. Regression sweep — clean.** No `.unwrap()`/`panic!` on evaluator-controlled values: the only non-test panics are `factorial(n).expect` / `checked_mul(...).expect` (`:490,550,555,647,652`) on engine-internal `n`/`total` (see MINOR-1). `matches.lock().unwrap()` / `into_inner().unwrap()` (`:615,621`) only panic on a poisoned lock, i.e. a panic already in flight inside a thread — and the evaluators here cannot be made to panic by P1 inputs; a thread panic would propagate out of `thread::scope` (not silently swallowed). `factorial:472` and `total_candidates:500` correctly return `Option` (checked_mul) for caller pre-checks.

**8. Plan fidelity — faithful.** Module API matches plan §2: `SearchMode::{Id,Address}` (the plan's `MatchPredicate::{WalletPolicyId,Address}` is realized as the injected `CandidateEvaluator` predicate + `SearchMode` carrying the address range — a cleaner seam that keeps P1 free of md-codec, exactly the standalone-testability the plan demands); `std::thread` parallel `min(20,ncpu)` (`search_threads:462`, no rayon); realized-S sizing (`required_prefix_bytes`); adaptive cap (`cap_decision`); ascending-address-outer; `reject_duplicate_keys` + strong-prefix primitives; all unit-tested in isolation with synthetic evaluators (no completion code). The lib-only mount is documented for P3: `lib.rs:88-99` states the bin reaches the engine via `mnemonic_toolkit::permutation_search::*` (the same external-self path used for `mnemonic_toolkit::mlock::*`), and explains the rationale (avoid a dead-code bin copy tripping `-D warnings` before P3). Bench-commit decision (plan §2 TDD / §9) is deferred — acceptable at P1 (the cap cost-model is exercised by the synthetic-slow calibration test instead).

---

## CRITICAL
None.

## IMPORTANT
None.

## MINOR
- **M1 — `search()`/`search_reference()` panic via `.expect()` on factorial/total overflow for unrealistic `n`; no upper-`n` guard.** `:550,555,647,652`. `factorial(n).expect` panics for `n≥35`; `perms.checked_mul(outer).expect` can overflow u128 for `n≈34` + a wide address range. The module docstring (`:470-471`) itself says "callers must handle it rather than panic," yet `search()` is a caller that `.expect()`s instead of returning a `SearchError`. **Not Critical:** realistic multisig N ≤ 20 (consensus `OP_CHECKMULTISIG` 20-key limit; `multi`/`sortedmulti` cap at 20); `20!`=2.4e18 and `20! × (u32 range × 2 chains)` do NOT overflow u128 (reviewer-verified) — so the panic is unreachable on realistic input, and the review's Critical bar ("overflow/panic on realistic input") is not met. Defense-in-depth suggestion for P3 wiring (not a P1 blocker): either add an explicit `n` bound check returning a `SearchError::TooManySlots`, or have `search()` consume `factorial(n)?` / `total_candidates(n,mode)?` and surface `None`→error rather than `.expect()`. Track via FOLLOWUP if not folded into P3's distinct-keys/floor pre-checks.
- **M2 — `outer_count`/`flatten` use `u64` while `search` mixes `u128`.** Internally consistent and lossless for the legal range (reviewer-checked), but the `(idx<<1)|chain` address encoding is an opaque engine↔evaluator contract (`:228-231`) that P3/P4 must decode identically. Documented (`:223-229`) but un-pinned by a cross-module test until P3 wires the real evaluator — call this out in the P3 R0.

---

## To turn GREEN
Already GREEN. M1/M2 are non-blocking; recommend M1 be addressed (bound or `?`-propagate) when P3 wires the engine so the funds-safety-core public entrypoint cannot panic even on a malformed/hostile slot count, and M2's address-index encoding be pinned by the P3 real-evaluator round-trip test.
