> Reviewer: opus architect — P1 per-phase R0 EXECUTION review · `mnemonic-toolkit` `feature/own-account-subset-search` @ `91298b11`

**Verdict: GREEN — 0 Critical, 0 Important.** (1 Minor, forward-looking.)

The funds-safety combinatorics core is correct. Every generator is provably bijective-onto-its-count, verified by my OWN independently-constructed oracles (different algorithm than the implementer's), the early_exit knob is funds-safe with a byte-invariant default path, overflow refuses rather than panics, and the committed diff is exactly the two declared files with no fmt churn. The gate advances to P2.

---

### Independent verification methodology
I did NOT trust the implementer's in-file oracle. I copied the four generators + five cardinality helpers VERBATIM from `permutation_search.rs@91298b11` into a standalone crate and re-implemented every oracle with a deliberately different construction — bitmask combination enumeration (vs the implementer's recursive choose) and std-style `next_permutation` (vs their recursive Heap-ish permute). I ran **1490 bijection/count test-groups** plus targeted edge/mutation/expect-safety probes. All pass. A shared-algorithm bug cannot hide behind matching oracles.

### 1. Own-anchored generator bijects EXACTLY `S_own` (the heart) — CONFIRMED
`own_anchored_unrank` (`permutation_search.rs:703-731`). My independent sweep over all `(k_own∈1..7, j∈1..k_own, m∈0..4)`:
- (a) For `r∈[0,S_own)` the generated set EQUALS my from-first-principles valid-assignment set, each exactly once (no dup via HashSet `|set|==|list|`, no miss/extra via set equality).
- (b) `count == C(k_own,j)·N!` (cross-checked against `s_own` AND my oracle count).
- (c) **NO cosigner-dropping** — every assignment contains all `m` cosigner indices `{k_own..k_own+m}` AND exactly `j` own indices `<k_own`. This is the I-1 danger direction (a scanned-but-uncounted placement = an unsized collision); my probe asserts it directly and it holds universally.
- (d) At `k_own==j` it collapses to plain `unrank_permutation(N)` rank-by-rank (v0.60.0 equivalence) — verified for all `(j∈1..5, m∈0..3)`.
- (e) Bonus: the `m==0` degenerate (no cosigners) is a clean `j`-perm bijection (no panic, no dup).

The construction is sound: `combo_rank = rank/N!` selects a `j`-subset of own via CNS-unrank; the `N` selected entries = `(j chosen own) ++ (all m cosigners)`; `perm_rank = rank%N!` orders them via `unrank_permutation`. Because cosigners are ALWAYS appended whole, dropping is structurally impossible — confirmed empirically, not just by argument.

### 2. `unrank_kperm` + `opt_in_unrank` biject their counts — CONFIRMED
- `unrank_kperm` (`:643-657`): bijective onto `P(pool,n)` injective placements for all `(pool∈1..8, n∈1..pool)`; lexicographic order matches the committed pin-test (`unrank_kperm(0..,4,2)`).
- `opt_in_unrank` (`:746-793`): bijective onto `S_opt` for all `(k_own∈1..5, m_sup∈1..5, n∈2..6)`, sorted AND non-sorted. The j-strata are disjoint (each assignment has exactly one own-slot-count `j`), so no cross-strata double-count — my set-equality check would have caught any collision and found none. Every assignment carries ≥1 own + ≥1 cosigner (the `j_min=1`/`≥1-cosigner` contract). The in-stratum rank decomposition `own_rank·(cos·N!) + cos_rank·N! + perm_rank` (`:771-774`) round-trips correctly.
- `s_opt` closed form cross-checked against my oracle count over `(k_own∈1..6, m_sup∈1..6, n∈2..7)` — exact match, sorted and non-sorted.

### 3. CNS `unrank_combination` correctness + load-bearing tests — CONFIRMED
`unrank_combination` (`:666-685`). Bijective `[0,C(k,r)) ↔ {r-subsets of k}` for all `(k∈0..16, r∈0..k)`; every output strictly ascending and in-range. **I reproduced the implementer's reported off-by-one**: parameterizing the second `c_choose` arg `remaining-1 → remaining` (off=+1) BREAKS the bijection (the set diverges from the reference / dup) on a case where the committed off=0 bijects — so the tests are genuinely load-bearing, not vacuous.

### 4. Cardinality overflow → REFUSE not panic — CONFIRMED
`c_choose(256,128)==None` (252-bit), `p_count(usize::MAX,40)==None`, `s_own(40,1,34,false)==None` (N=35 factorial leg), `s_own(256,128,0,false)==None` (c_choose leg), `s_opt(2,2,40,false)==None`. All `checked_mul`/`checked_add` with `?`/`None` propagation — no `unwrap`/panic anywhere in the cardinality helpers (`:543-630`). The generators' internal `.expect()` calls (`:651,:674,:714,:756,:764-767`) are safe because each computes a factor that DIVIDES the cardinality the caller already proved `Some(.)` — I probed the boundary domain (`k_own` up to 256, j/m/N at limits) and none fire. (Forward note → Minor.)

### 5. `early_exit` knob — funds-safe + byte-invariant default — CONFIRMED, caveat reasoning VALID
- Default path byte-unchanged: the only behavioral edits to `search` are (i) `stop_at = if early_exit {1} else {2}` (was hardcoded `>= 2`; evaluates to 2 when false), and (ii) a new `if early_exit {…}` first-match branch BEFORE the unchanged `match found.len()` classifier. With `early_exit=false`, `stop_at==2` and the classifier is byte-identical to the parent (`git show 91298b11^` diff confirms only the hardcoded `2 → stop_at` substitution + the new false-gated branch).
- Sole production caller `restore.rs:2015` passes `false` (the +5/-1 diff is PURELY threading `false` through the one `ps::search` call — I grepped the whole tree: exactly ONE `ps::search` call site exists, and verify_bundle reaches the engine via the shared `complete_multisig_template`, inheriting `false`). No behavior change.
- **Mutation testing (the anchor is genuinely load-bearing).** In a throwaway worktree I applied both outcome-changing mutations: (a) classify any non-zero `found.len()` as Unique → anchor test `early_exit_false_reproduces_v060_full_scan_outcomes` goes **RED** (expects Ambiguous, gets Unique). (b) leak early-exit into default (`if true` + `stop_at=1`) → **RED** (the 64-iteration many-match ambiguity loop catches it). Both outcome flips are caught.
- **The implementer's honest caveat is correct.** I verified empirically: `stop_at=1` ALONE (keeping the `if early_exit` final-classifier gate intact, flag still false) PASSES the anchor — because on a many-match space the racing threads accumulate ≥2 hits into `found` before the global counter trips the stop flag, so `match found.len()` still classifies Ambiguous. `stop_at` only bounds over-scan (perf), not the observable outcome (funds-safety). The anchor guards the CLASSIFIER, which is exactly the right invariant. Prefix-id NEVER gets `early_exit=true` (P2's obligation, documented `:838-840`).

### 6. No regression + clean build — CONFIRMED
- `cargo test -p mnemonic-toolkit`: **3447 passed / 0 failed** (exact expected count). Lib target 182 passed; `permutation_search.rs` has 48 `#[test]` fns.
- `cargo clippy -p mnemonic-toolkit --tests --bins --lib -- -D warnings`: clean (exit 0).
- `git diff --name-only 91298b11^..91298b11` = exactly `permutation_search.rs` + `cmd/restore.rs`. No `Cargo.toml`/`Cargo.lock`/`mlock.rs`, no new dep.
- fmt: the two P1 files have NO P1-introduced fmt churn. The `restore.rs:398/:829` rustfmt-diff PRE-EXISTS in `91298b11^` (the known repo rustfmt-version divergence / g6 exemption, "never cargo fmt the toolkit"); `permutation_search.rs` is fmt-clean. The P1 restore.rs hunk is solely `@@ -2009,7 +2009,10 @@`.

### 7. Pool-index convention consistency — CONFIRMED
OWN-FIRST (own `0..k_own`, cosigners `k_own..k_own+m`) is documented in the module header (`:518-526`) and used consistently by all three generators — own subset first, then cosigners appended at `k_own + c`. My oracles assume the same layout and bijected against the implementations, so any convention mix would have shown as a set mismatch. None found. P2 builds its `key65`/origin pool in this order.

---

### Minor (forward-looking, P2/P3 — NOT a P1 defect)
- **M-1 — `unrank_kperm`'s `pub` + internal `.expect()` is an unguarded entrypoint.** `unrank_kperm` (`:650`) calls `p_count(...).expect(...)`; it is `pub` and doc'd "used by the OPT-IN/uniform path," but the actual opt-in generator is `opt_in_unrank` (internally guarded by `s_opt`). Within the realized multisig domain (`N ≤ ~15`, `pool ≤ 256+M`) `p_count` never overflows, so this cannot fire in practice. But if a future P2/P3 caller invokes `unrank_kperm` directly on an unbounded `pool`/`n` without a preceding `p_count`-`None` guard, it could panic. Fix (optional, P2/P3): either gate every `unrank_kperm` call behind a `p_count(...).is_some()` check (the §6 ceiling already bounds this) or note the precondition in the doc-comment. No action required for P1 GREEN.

---

GREEN (0C/0I). Every generator is provably bijective-onto-its-count under independent enumeration; no cosigner-dropping in own-anchored; no cross-strata double-count in opt-in; the CNS unrank is bijective with load-bearing tests; overflow refuses without panic; the early_exit knob is byte-invariant on the default/id/prefix-id paths and cannot leak (verified by mutation). **The gate advances to P2.**
