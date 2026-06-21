# SPEC — `restore`/`verify-bundle` multisig-template own-account subset-search (+ opt-in bounded unowned)

**Date:** 2026-06-20 · **Base SHA:** mnemonic-toolkit `82e58674` (branch `feature/own-account-subset-search`, atop master v0.60.0). **Brainstorm (R0-GREEN, 2 rounds):** `design/BRAINSTORM_own_account_subset_search_2026-06-20.md` + `design/agent-reports/own-account-subset-search-brainstorm-r0-round{1,2}-review.md`. **FOLLOWUP:** `template-multisig-own-account-range-subset-search`.
**SemVer:** toolkit **MINOR** `0.60.0 → 0.61.0` (re-enables `--own-account-max` behavior refuse→search + one NEW opt-in flag, both on `restore` + `verify-bundle`). md-codec/mk-codec **NO-BUMP**. GUI MINOR paired.

## 1. Summary
Lift the #28-phase-2 P3a-deferred over-supply gate. Today `complete_multisig_template` (the shared restore/verify-bundle core, `restore.rs:1416`) requires the supplied keys to EXACTLY fill the N slots (`pool.len() == n`, gates `:1626`/`:1635`) and enumerates `n!` orderings via `unrank_permutation` over `(0..n)` (`permutation_search.rs:494`). This SPEC adds an **own-anchored subset-search**: the operator over-supplies OWN-account candidates (they don't recall the wallet's account index) and the engine resolves the unique `@N`→key assignment over the enlarged pool. Two tiers (brainstorm D1): **own-only by default** (cosigners exact); **opt-in + bounded unowned/cosigner search**.

## 2. Flags (clap surface — on BOTH `restore` and `verify-bundle`, shared core)
- **`--own-account-max <K_own>`** (re-enable; remove the refuse at `restore.rs:1434`). Own seed (`--from`) derived at accounts `0..K_own−1` ⇒ `K_own` own candidates. **Mutually exclusive with `--account`** (open-point 2 resolved): supply EITHER an explicit `--account <list>` (the v0.60.0 path, own candidates = the listed accounts) OR `--own-account-max K_own` (the range). Both → `BadInput` ("use --account OR --own-account-max, not both"). `K_own` hard-ceilinged (see §6).
- **`--search-cosigner-subset`** (NEW, opt-in boolean, default OFF) (open-point 3 resolved). When OFF (default = own-only): the `--cosigner` cards must be EXACT (`pool` cosigner count == the wallet's `M`); an over-supplied cosigner pool REFUSES (§5 premise table). When ON: the operator MAY over-supply `--cosigner` cards (uncertain which belong / how many cosigners); the search selects the correct cosigner subset too. Bounded by the §6 hard ceiling + the existing adaptive time-cap + `--accept-search-time`. Default-off ⇒ the safe, bounded common case.
- **Unchanged / reused:** `--from`, `--account`, `--cosigner` (unassigned mk1s), `--expect-wallet-id`, `--search-address` + `--search-addr-min/max` + `--search-chain`, `--accept-search-time`, `--origin`. Explicit `--cosigner @N=` assignment is **mutually exclusive** with subset-search (`--own-account-max` / `--search-cosigner-subset`) (open-point 7 resolved): mixing → `BadInput` ("explicit @N= assignment cannot combine with subset-search").
- **verify-bundle:** the same flags are exposed (shared core; open-point 8). Over-supplied search at verify-bundle is unusual (you hold the bundle) but supported for parity; the manual notes it.

## 3. The own-anchored count (the load-bearing math — from the brainstorm D2, R0-GREEN)
`N` slots; `M` cosigner cards (own-only: exact, all used); `j = N − M` own-slot-count (own-only: determined); `K_own` own candidates. **Own-only enumerated count:**
> **`S_own = C(K_own, j) · N!`**, `j = N − M`. (Choose `j` of `K_own` own: `C(K_own,j)`; assign the `j` own + `M` cosigners to `N` slots: `N!`.) Collapses to `N!` at `K_own = j` (byte-identical to v0.60.0 `perm_count_u128(n,n)`, `restore.rs:1661`).

**Sorted-shape composition (open-point 5 resolved):** for an order-INDEPENDENT shape (`sortedmulti`/`sortedmulti_a`, `is_order_independent_shape`), all `N!` orderings of a fixed key SET are the same wallet, so the search restricts ORDER to the identity placement (as v0.60.0 does) BUT still enumerates SUBSETS. **`S_own_sorted = C(K_own, j)`** (no `·N!`). The realized space + the address-search collapse must compose: enumerate the `C(K_own,j)` subsets, identity order each.

**Opt-in count** (`--search-cosigner-subset` ON): the cosigner count is uncertain — the pool is `K_own` own + `M_sup` supplied cosigner candidates, and the search ranges over valid `(j, cosigner-subset)`. `S_opt ≤ P(K_own + M_sup, N)` (the uniform bound); the engine sizes `realized_s` to the ACTUAL enumerated count for the chosen enumeration (SPEC §4.3) and §6 hard-ceilings it. (The opt-in exact enumeration formula is pinned in §4.3.)

**`realized_s` = the ACTUAL enumerated count** (`S_own` / `S_own_sorted` / `S_opt`) — NEVER `n!`, NEVER the looser `P(pool,N)` for own-only. Closed-form ⇒ computable up-front for `validate_prefix_strength` (`:1700`) + the cap ETA. **FLOOR — enumerated ≡ counted:** the generator MUST enumerate EXACTLY the counted set (bijection), verified by a brute-force-reference test (§7). A scanned-but-uncounted placement = a collision the prefix wasn't sized for (the dangerous direction).

## 4. Engine changes (`permutation_search.rs`)
### 4.1 Subset-select enumeration
Add a pool-aware enumerator generalizing `unrank_permutation(rank, n)` (`:494`, which builds `elems=(0..n)`):
- **`unrank_kperm(rank, pool, n) -> Vec<usize>`** — the `rank`-th injective k-permutation (lexicographic): place `n` of `pool` items into `n` ordered slots, returning the `pool`-indices. Count `P(pool, n)`. O(n·pool) Lehmer-style unrank. Used by the OPT-IN/uniform path.
- **The OWN-ANCHORED generator** — own-only is NOT a plain `unrank_kperm` over the whole pool (that includes cosigner-dropping placements `S_own` excludes). It is a composed rank over the `S_own = C(K_own,j)·N!` space: `rank → (combo_rank ∈ [0,C(K_own,j)), perm_rank ∈ [0,N!))`; `combo_rank` unranks to the `j`-subset of the `K_own` own indices (combinatorial-number-system unrank); `perm_rank` unranks (via the existing `unrank_permutation(perm_rank, N)`) to the ordering of the `N` selected keys (`j` chosen own + all `M` cosigners) into the `N` slots. **Bijective over `S_own` by construction** — every assignment using exactly `j` own + all `M` cosigners, each once, nothing else. (Sorted shape: drop the `perm_rank` factor — identity order only, space `C(K_own,j)`.)
- **`total_candidates` / cardinality:** extend (or add `total_candidates_subset(mode, S)`) so the search drives `S` candidates where `S = realized_s` (the §3 count), not `factorial(n)`. `factorial`/`perm_count`/the combo-count all REFUSE on `u128` overflow → `None` → `bad(...)` (the #28 M1 lesson; `perm_count_u128:1882` already does this).

### 4.2 Sharding (R0-r2 confirmed structure-agnostic)
`search` (`:551`) shards the rank space `[0, S)` across `min(20, ncpu)` threads. The shard logic needs only `S` (cardinality) + a bijective `unrank` — both provided. The own-anchored composed rank shards identically (contiguous rank ranges). No sharding redesign.

### 4.3 Opt-in exact enumeration
`--search-cosigner-subset` ON: the search ranges over `(own-subset, cosigner-subset, ordering)`. To keep `realized_s == enumerated`, the SPEC fixes the enumeration as: for the supplied `K_own` own + `M_sup` cosigner candidates filling `N` slots with `j` own + `(N−j)` cosigner for each valid `j ∈ [j_min, j_max]` — enumerate `Σ_j C(K_own,j)·C(M_sup, N−j)·N!`. `realized_s` = that sum (closed-form). `j_min/j_max`: default `j_min=1` (own ≥1 via `--from`), `j_max=min(K_own, N−1)` (≥1 cosigner) — OR the operator pins `j` (open-point 3 / D6: a `--own-slots <j>` could pin it; SPEC default = infer the range, R0 to confirm whether to add the pin flag or defer). The §6 hard ceiling refuses if this sum is too large.

### 4.4 Early-exit policy (open-point 4 resolved)
- **prefix id-search:** FULL-SCAN (ambiguity certification — ≥2 matches → `Ambiguous` → refuse). Own-first ORDERING does not shortcut this; the time-cap bounds it.
- **address-search:** collision-free (full scriptPubKey; the whole-pool distinct-keys floor `:1648` guarantees distinct subsets ⇒ distinct programs, §I-2) ⇒ a match is provably UNIQUE ⇒ **early-exit on first match is SAFE and ENABLED** (the perf win that makes large pools tractable). Confirm against the current engine's behavior (if it currently full-scans address-search, this is a deliberate, correctness-preserving change for the subset-search path; gate behind the over-supply path so the v0.60.0 exact path is byte-unchanged).
- **full (non-prefix) wallet-id:** collision-safe ⇒ early-exit permitted (SPEC may keep full-scan for simplicity; decide at impl — not funds-safety-load-bearing either way since a full id is unique).
- **Own-first ordering:** the enumerator yields own-anchored / low-account-index assignments first (the common case) so address-search hits the real wallet early. Ordering is a perf heuristic, NOT a correctness mechanism (the cross-link to §4.4 prefix-id full-scan is mandatory — R0-r1 M-2).

## 5. Funds-safety floors (from brainstorm §3/§3a — R0-GREEN)
- **Distinct-keys** (`reject_duplicate_keys` on the WHOLE pool, `:1648`, before `realized_s`/search) — now LOAD-BEARING for the new subset collision axis (distinct subsets ⇒ distinct key SETS ⇒ distinct scriptPubKey/id, ONLY because no two pool keys are byte-identical; `key65` compare catches an own-derived key == a cosigner card regardless of origin).
- **Enumerated ≡ counted** (§3 FLOOR) — the generator bijects the `realized_s` set; brute-force-reference tested (§7).
- **Ambiguity (≥2) + no-match → refuse**, over the own-anchored enumerated set.
- **Prefix-strength sized to `realized_s`** (= `S_own`/`S_own_sorted`/`S_opt`), `ceil((log2(S)+32)/8)` ladder. Worked: `K_own=32, j=4, M=7, N=11` → `S_own≈1.435e12` → 10-byte prefix.
- **Hard ceiling + time-cap** (§6).
- **Per-slot origins built fresh; carried origin never loaded (C1)** — unchanged from #28.
- **§5a premise-violation failure modes (own-only) — all fail SAFE:**
  | Violation | Outcome |
  |---|---|
  | Under-supplies cosigners (`M'<M`) | NO-MATCH → refuse (message: "supply ALL cosigner cards?") |
  | Over-supplies cosigners in own-only (`M'>M`) | **REFUSE up front** ("own-only needs exact cosigners; use `--search-cosigner-subset`") |
  | Own key supplied ALSO as a cosigner card | refuse (distinct-keys floor, before search) |
  | Owns >`j` slots via MULTIPLE seeds | out of own-only scope (own-only = ONE seed, multi-account); use `--account`/`@N=`/opt-in |

## 6. Bounding (open-point 6 resolved)
- **Hard `K_own` ceiling:** `K_own ≤ 256` (a sane account-range; larger → `BadInput`).
- **Hard `realized_s` ceiling:** refuse (before cap calibration, distinct from the time-cap) if `realized_s` exceeds a concrete `S_MAX` whose worst-case ETA at the calibrated per-candidate cost is bounded — pin `S_MAX` such that even at `--accept-search-time` the search is finite/practical; e.g. `S_MAX = 1e15` (≈ the #28 benchmark's ~170M cand/min ⇒ ~4000 days — clearly refuse). The operator must narrow inputs. `u128` overflow → `bad` (backstop, already live).
- **Time-cap:** the existing adaptive ~1hr ceiling + `--accept-search-time` (calibrated per-machine) — unchanged.

## 7. TDD plan (per-phase, RED-first)
- **Engine (P1):** `unrank_kperm` bijection (exhaustive small-(pool,n): the enumerated set == the brute-force injective-placement set, each once); the OWN-ANCHORED composed-rank generator bijection over `S_own` (exhaustive small-(K_own, j, M): == the independently-generated valid-assignment set; `count == C(K_own,j)·N!`); sorted-shape variant (`C(K_own,j)`, identity order); overflow → refuse; the cardinality helpers.
- **restore (P2):** re-enable `--own-account-max` completes a wallet whose own account is NOT 0 (e.g. own at account 3, `--own-account-max 5`) to the independent golden; `--account`⊕`--own-account-max` mutual-exclusion refuse; the §5a premise-violation refusals (each fail-safe); backward-compat (exact pool == v0.60.0, byte-identical addresses + the existing 25 multisig-template tests stay GREEN); the worked 10-byte prefix sizing; address-search early-exit finds a non-zero-own-account wallet.
- **opt-in (P3):** `--search-cosigner-subset` completes with an over-supplied cosigner pool to the golden; the hard-ceiling refuse; the `@N=` mutual-exclusion refuse.
- **verify-bundle (P4):** parity (verify == restore == golden) for an over-supplied own-account completion.
- **differential (P5):** property/corpus — randomized own-only subset-search completes to the independent rust-miniscript golden; anti-vacuity (a wrong subset ≠).

## 8. Phase map (each = TDD + per-phase R0 to 0C/0I)
P1 engine (`unrank_kperm` + own-anchored generator + cardinality, brute-force-reference tested) → P2 restore own-only (re-enable + premise gates + backward-compat) → P3 opt-in (`--search-cosigner-subset` + bound) → P4 verify-bundle parity → P5 differential/property → P6 GUI schema_mirror (the NEW `--search-cosigner-subset` flag; `--own-account-max` name unchanged) + manual (the `--own-account-max` row refuse→search + a "subset-search" section) + version 0.61.0 + whole-diff ship review + ship + FOLLOWUP flip.

## 9. SemVer + locksteps
- toolkit **MINOR 0.61.0**. md-codec/mk-codec NO-BUMP.
- **GUI `schema_mirror`** + **manual** lockstep (P6): only the NEW `--search-cosigner-subset` flag needs the schema mirror (flag-NAME) on restore + verify-bundle (+ paired GUI MINOR); `--own-account-max` NAME is unchanged (refuse→search is behavior-only). The manual `--own-account-max` row STILL needs an edit (refuse→search description) even though schema is untouched.

## 10. Open items for the plan-doc / execution
1. Re-grep all citations vs the plan base SHA (decay).
2. Confirm `is_order_independent_shape` location + the address-search sorted collapse site (brainstorm cited `restore.rs:1676`/`:1739` — re-grep; not in the §-grep above, verify).
3. Decide `--own-slots <j>` pin flag (opt-in `j`-pin) vs inferring `j_min..j_max` (§4.3) — default infer; add the pin only if R0 deems the inferred range too loose.
4. The exact opt-in `realized_s` sum + its hard-ceiling interaction (§4.3/§6).
5. Whether address-search early-exit changes the v0.60.0 EXACT-path behavior (must be gated to the over-supply path so the exact path is byte-unchanged) — confirm at impl.
6. No-match UX: name `--origin` as the escape hatch for heterogeneous cosigner families (R0-r1 M-3).
