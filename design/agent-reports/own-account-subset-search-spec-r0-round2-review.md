> Reviewer: opus architect (R0 round 2) · 2026-06-20 · SPEC `design/SPEC_own_account_subset_search_2026-06-20.md` @ HEAD `2f5d0882` (fold commit, branch `feature/own-account-subset-search`) · source-verified against base `82e58674` (v0.60.0). Round-1 review (`own-account-subset-search-spec-r0-round1-review.md`, RED 0C/5I) + brainstorm + its 2 R0-GREEN reviews read in full. Note: the cited source files live under `crates/mnemonic-toolkit/src/cmd/` (`restore.rs`, `verify_bundle.rs`) and `crates/mnemonic-toolkit/src/` (`permutation_search.rs`, `synthesize.rs`); the SPEC's bare-basename `restore.rs:NNNN` convention is unambiguous and every line was re-grepped at HEAD.

**Verdict: GREEN — 0 Critical, 0 Important.**

The fold closes all five round-1 Importants with no drift, lands all four Minors plus the opt-in-stratified-generator note, and introduces no new finding. Every load-bearing source contradiction the round-1 review flagged is now corrected against live code, the four count/bijection/early-exit/distinct-keys funds-safety contracts are pinned on the page, and the five "behavior the source contradicts" gaps are resolved. The R0-SOUND items (own-anchored bijection §4.1, opt-in count §4.3, backward-compat at `K_own=j`, bounding §6) did not regress. **This SPEC advances to the plan-doc.**

---

## Citation re-audit at HEAD `2f5d0882` — ALL CONFIRMED

Every line the post-fold SPEC cites is live and accurate at HEAD (re-grepped, not lifted from round-1):

| SPEC cite | Confirmed at `2f5d0882` |
|---|---|
| `complete_multisig_template` `restore.rs:1416` | ✓ `pub(crate) fn complete_multisig_template<E: Write>` |
| `--own-account-max` refuse gate `:1434` | ✓ `if ctx.own_account_max.is_some() {` → `bad("…not supported yet…")` |
| under/over-supply gates `:1626`/`:1635` | ✓ `pool.len() < n` / `pool.len() > n` |
| `realized_s` `:1661` | ✓ `perm_count_u128(n, n)` at `:1662`, `bad("…candidate space overflow")` on `None` |
| `reject_duplicate_keys` whole-pool `:1648` | ✓ on `pool_key_blobs`, BEFORE `realized_s` |
| `perm_count_u128` `:1882` | ✓ `fn perm_count_u128(pool, n) -> Option<u128>` (the `:1882` cite is exact — round-1's table mis-located it; I-1/m-3 math unaffected) |
| `validate_prefix_strength(…, realized_s)` `:1700` | ✓ (sig `:342`; `required_prefix_bytes` `:322`) |
| engine `search` `:551` | ✓; `unrank_permutation` `:494` ✓ (`fn unrank_permutation(rank, n)` builds `elems=(0..n)`); `total_candidates` `:509` ✓ |
| `is_order_independent_shape` `synthesize.rs:335` (m-1) | ✓ `pub(crate) fn is_order_independent_shape(tree: &md_codec::tree::Node) -> bool` — `SortedMulti\|SortedMultiA → true`, recurses single-child + `Tr` |
| sorted evaluator-filter `restore.rs:1676`/`:1739` | ✓ `:1676` `let sorted_shape = crate::synthesize::is_order_independent_shape(&d.tree);`; `:1739` `if sorted_shape && !assignment.iter().enumerate().all(\|(i,&v)\| i==v)` |
| `id_search`/`addr_search` precedence `:1665`/`:1666` | ✓ `let id_search = ctx.expect_wallet_id.is_some();` / `let addr_search = ctx.search_address.is_some();`; consumed `:1696` (`if id_search`) / `:1721` (`else if addr_search`) |
| `restore.rs:86` `conflicts_with` precedent | ✓ `#[arg(long, conflicts_with = "passphrase")] pub passphrase_stdin: bool` (the only `conflicts_with` in restore.rs — a genuine, live precedent for the IDIOM, on `passphrase`) |
| `--account` `restore.rs:106` | ✓ `#[arg(long, value_delimiter = ',', default_value = "0")] pub account: Vec<u32>` |
| `verify_bundle.rs:865` `own_account_max: None` | ✓ hardcoded `None` in the ctx; `--account` is SCALAR `pub account: u32` (`:64`), wired `own_accounts: vec![args.account]` (`:862`); NO `#[arg(long="own-account-max")]` exists on verify-bundle |
| test fns `:635`/`:677`/`:715` | ✓ `multi_account_own_resolves_both_slots` `:635`, `own_account_max_flag_refuses_with_actionable_message` `:677`, `pool_larger_than_slots_refuses_with_actionable_message` `:715`; file has exactly **27** `#[test]` fns |

**External-fact note:** self-contained combinatorics/search-engine change — no BIP-39/NDEF/OTP/SDK external protocol facts. Authoritative sources are the toolkit's own pinned math + the cited code, all re-grepped here. The worked margin recomputes exactly: `C(32,4)=35,960`, `·11!=1,435,408,128,000`, `log2≈40.385`, `ceil((40.385+32)/8)=10` bytes (§5 ✓). `C(256,128)` is 252 bits > u128 (m-3 overflow is real ✓). `S_MAX=1e15` at 170M cand/min ≈ 4085 days → clearly refuse (§6 ✓).

---

## Per-Important closure status

### I-1 (sorted-shape mechanism) — CLOSED

The fold adopts the **enumeration-side** mechanism throughout (§3 "ENUMERATION-SIDE", §4.1 "drop the `perm_rank` factor", §4.3 sorted "drop the `·N!` per stratum").

- **(a) Internally consistent + math right.** §3 now states `S_own_sorted = C(K_own,j)` is "simultaneously the enumerated count, the cap-ETA scan count, AND the colliding-set size (the three COLLAPSE — no prefix-vs-ETA divergence; the `realized_s` symbol is unambiguous)." This is exactly the round-1-recommended option-(A) that eliminates the prefix-sizing-vs-cap-ETA divergence. The distinct-subset ⇒ distinct-sorted-address floor is correct: I re-verified `is_order_independent_shape` returns true for SortedMulti/SortedMultiA, and `reject_duplicate_keys` operates over the whole pool's `key65` blobs (`reject_duplicate_keys<T: PartialEq>`, `permutation_search.rs:289`), so distinct subsets ⇒ distinct key SET ⇒ distinct sorted-pubkey multiset ⇒ distinct scriptPubKey. No two-subset address collision. ✓
- **(b) The verbatim-reuse warning landed.** §3: "**that filter is WRONG to reuse verbatim** — 'identity' is per-subset, so a verbatim `assignment==identity` skip would discard every non-first SUBSET." This is the exact round-1 I-1(3) hazard, tied to the concrete `:1739` predicate. ✓
- **(c) v0.60.0 EXACT path keeps the evaluator-filter byte-unchanged.** §3 closes: "(The v0.60.0 EXACT path — `pool.len()==n`, no over-supply — keeps its evaluator-filter mechanism byte-unchanged; only the subset path uses the enumeration-side generator.)" ✓
- **No contradiction with §4.4 or §3's non-sorted count.** Non-sorted: `realized_s = C(K_own,j)·N!` (scanned = colliding count, coincide). Sorted: all three collapse to `C(K_own,j)`. §4.4 early-exit keys off `realized_s ≠ n!` (over-supply) — orthogonal to the sorted/non-sorted axis, no conflict.

### I-2 (test-flip honesty) — CLOSED

§7-P2 now reads honestly and I source-verified each test:

- **(a)** Names the byte-identical-GREEN exact-pool tests incl. `multi_account_own_resolves_both_slots (cli_restore_md1_template_multisig.rs:635)` (confirmed at `:635`, an exact-pool `--account 0,1` path) "the explicit-`@N=` tests, all `pool.len()==n` completions, AND a NEW before/after byte-identical v0.60.0 EXACT-path address-search outcome (the I-5 early-exit-gate regression guard)." ✓
- **(b)** Commits `own_account_max_flag_refuses_with_actionable_message (:677)` to be **REWRITTEN (RED-first)** — "this pinned test FLIPS by design (the SPEC's core feature)." I read the test body at HEAD: it asserts `--own-account-max 3` `.failure()` with a message naming `--account` and NOT containing "no match" — exactly the refuse-behavior the P2 feature converts to search-completes. The flip is real; the SPEC is now honest. ✓
- **(c)** `pool_larger_than_slots_refuses_with_actionable_message (:715)` **UPDATED** message. Body confirmed: `--account 0` + cosigner-B + extra-outsider-C → pool 3 > n 2; refusal preserved, gate/message changed to the §5a own-only-extra-cosigners path. ✓
- **"27 #[test]" stated** (§7-P2 "(The file has 27 `#[test]` fns; (a) stay green, (b)+(c) change.)") — `grep -c '#\[test\]'` = **27** exactly. ✓ The imprecise round-1 "25" is gone.

### I-3 (verify-bundle surface + schema scope) — CLOSED

- §2 now states `--own-account-max` is "**NEW clap surface**" on verify-bundle, "today `own_account_max` is hardcoded `None` at `verify_bundle.rs:865`; no such `#[arg]` exists ⇒ schema_mirror-gated there (§9)." Source-verified: `own_account_max: None` at `:865`, no `--own-account-max` arg in the file. ✓
- `--account` "**stays SCALAR `u32`** on verify-bundle (NOT widened to a list)" (§2, §9). Source-verified: `pub account: u32` at `:64`. ✓ The not-widening decision is justified ("you hold the bundle"; widening = extra behavior change for marginal value).
- §9 scopes schema_mirror to BOTH new names on verify-bundle + `--search-cosigner-subset` on restore + both manual rows: "**verify-bundle:** BOTH `--own-account-max` AND `--search-cosigner-subset` are NEW flag NAMES there … ⇒ schema-mirror BOTH + add both manual rows." Plus the restore side correctly limited to `--search-cosigner-subset` only (`--own-account-max` name pre-exists on restore) and its manual-row edit. The lagging-gate caveat is preserved. ✓

### I-4 (mutex implementability) — CLOSED

§2 specifies clap **`conflicts_with = "account"`** (NOT a presence check): "implemented via clap **`conflicts_with = "account"`** … NOT a runtime presence check: `--account` has `default_value="0"`/`[0]`, `restore.rs:106`, so it is ALWAYS populated; clap `conflicts_with` correctly ignores the default and fires only when `--account` is EXPLICITLY supplied, so `--own-account-max 5` ALONE passes; precedent `restore.rs:86`." Source-verified: `--account` is `Vec<u32>` `default_value="0"` (`:106`) and `conflicts_with = "passphrase"` is the live precedent (`:86`). The §7 regression guard is committed: "AND `--own-account-max 5` ALONE must PASS — the I-4 regression guard" (§7-P2). ✓

### I-5 (early-exit contract) — CLOSED

The gate is PROMOTED into §4.4 as a hard invariant: "**address-search first-match early-exit is enabled IFF (over-supply path: `realized_s ≠ n!`) AND `mode == Address`.** The v0.60.0 EXACT path (`pool.len()==n`) AND all id-search/prefix-id paths RETAIN the unchanged full-scan-with-2nd-match-ambiguity behavior, **byte-identical to v0.60.0**." The knob is named (`early_exit: bool` or `SearchMode::Address { early_exit }`), the §7 before/after byte-regression guard is pinned ("§7 pins a v0.60.0 EXACT-path address-search outcome byte-for-byte before/after"), and prefix-id is explicitly forbidden early-exit. §10.5 is correctly reduced to API-shape-only: "The early-exit engine-knob SHAPE … — the INVARIANT is now a §4.4 contract; only the API shape is plan-level." ✓

Source-verified the SPEC's premise: the live `search` (`:551`) full-scans BOTH modes — the doc `:530-548` states it "does NOT stop at the first match"; it short-circuits only at the 2nd match (`global_matches.fetch_add(1, …)` at `:623`, `if prior + 1 >= 2` at `:624`). The address-ordering note (`:540-546`) confirms a low-index target is found while high indices are unscanned but the engine still completes — so first-match early-exit IS a real behavior change. The SPEC's "this is a behavior change to gate" is accurate. ✓

---

## Minors + opt-in note — all landed

- **m-1 (§10.2 resolved citation `synthesize.rs:335`):** §10.2 now reads "RESOLVED (R0-r1 m-1): `is_order_independent_shape` lives at **`synthesize.rs:335`**; … evaluator-filter is **`restore.rs:1676`** … + **`:1739`**." All three confirmed live. ✓
- **m-2 (opt-in stratified brute-force test §7-P3):** §7 engine line + §7-P3 both pin "the opt-in STRATIFIED generator bijection over `S_opt` (exhaustive small case == the independently-generated valid set; `count == Σ_j C(K_own,j)·C(M_sup,N−j)·N!`; m-2)" and "the STRATIFIED-generator brute-force-reference bijection test (m-2)." ✓
- **m-3 (`c_choose` overflow helper §4.1):** §4.1 adds "NEW overflow-checked **combination helper `c_choose(K_own, j) -> Option<u128>`** … `C(256,128)` … the helper `checked_mul`s like `perm_count_u128` and returns `None` → `bad`. `S_own = c_choose(K_own,j)? .checked_mul(factorial(N)?)?` — overflow at any step refuses." `C(256,128)` is 252 bits — the overflow is real and the helper is necessary. ✓
- **m-4 (id+addr precedence §2):** §2 "Mode precedence (R0-r1 m-4): under the over-supply path the search MODE inherits the v0.60.0 `id_search`/`addr_search` selection (`restore.rs:1665-1666`) … never BOTH in one search." Confirmed live at `:1665-1666`. ✓
- **Opt-in stratified generator (§4.3, round-1 axis-3):** the fold added the stratified composition tying it to §4.1 primitives: "within stratum-`j` compose `(own-combo-rank via CNS-unrank, cosigner-combo-rank via CNS-unrank, perm-rank via unrank_permutation(N))` — bijective onto the stratum by the same argument as §4.1. Brute-force-reference-tested per §7-P3 (m-2). The §6 hard ceiling refuses if `S_opt` is too large." This is the build-ready-bounded composition round-1 asked for — not an unbounded plan guess. ✓

---

## R0-SOUND items did NOT regress

- **Own-anchored bijection (§4.1):** unchanged structure — `combo_rank` (CNS-unrank j-subset) × `perm_rank` (`unrank_permutation(N)`), no cosigner-dropping, `realized_s == enumerated` structurally guaranteed. The §3 FLOOR ("enumerated ≡ counted, brute-force-reference tested") is intact. ✓
- **Opt-in count (§4.3):** `Σ_j C(K_own,j)·C(M_sup,N−j)·N!`, `j_min=1`, `j_max=min(K_own,N−1)`, disjoint-by-j ⇒ double-count-free — unchanged and correct. ✓
- **Backward-compat at `K_own=j`:** §3 still pins "Collapses to `N!` at `K_own = j` (byte-identical to v0.60.0 `perm_count_u128(n,n)`, `restore.rs:1661`)." ✓
- **Bounding (§6):** `K_own ≤ 256`, `S_MAX = 1e15`, ceiling-before-cap-calibration, u128-overflow backstop — unchanged and correct. ✓

---

## Fold-drift check — no fold-induced contradiction

I adversarially probed each round-1-flagged tension:

- **§3 sorted enumeration-side vs §4.4 early-exit:** no conflict. §3's three-way collapse for sorted is independent of §4.4's `realized_s ≠ n!` early-exit predicate (over-supply, not sortedness). For a sorted over-supplied wallet, `realized_s = C(K_own,j) ≠ n!`, so address-search early-exit applies — and it's safe because distinct subsets ⇒ distinct sorted address (the §5 floor). Consistent. ✓
- **`c_choose` overflow vs `S_MAX` ceiling ordering:** consistent. §4.1 computes `S_own = c_choose(K_own,j)?.checked_mul(factorial(N)?)?` — overflow → `None` → `bad` (refuse). §6 checks `realized_s > S_MAX` "before cap calibration" with "`u128` overflow → `bad` (backstop, already live)." The overflow path is a strict-superset backstop to the `S_MAX` ceiling (anything overflowing u128 ≫ 1e15), so ordering is immaterial — both refuse. No contradiction. ✓
- **verify-bundle `--account`-scalar vs own-only-via-`--account`-list path:** consistent. §2 keeps `--account` scalar on verify-bundle AND exposes `--own-account-max` there, so verify-bundle own-over-supply is `--own-account-max`-only (the scalar `--account` is the single-own-account path). No path requires a verify-bundle `--account` list. The asymmetry with restore (`Vec<u32>`) is explicitly stated and justified. ✓
- **§4.4 early-exit gate vs shared-engine byte-invariance:** the gate `IFF realized_s ≠ n! AND mode==Address` cleanly excludes the exact path (`pool.len()==n` ⇒ `realized_s == n!`) and all id/prefix paths. No leak. ✓

---

## Adversarial — remaining funds-safety-contract gaps

None. The four funds-safety contracts are on the page and source-consistent:

1. **Enumerated ≡ counted (bijection):** §3 FLOOR + §4.1/§4.3 generators + §7 brute-force-reference tests (own-only, sorted, opt-in). The dangerous direction (scanned-but-uncounted placement) is structurally closed by the own-anchored construction (no cosigner-dropping) and brute-force-pinned.
2. **Distinct-keys floor LOAD-BEARING (§5):** whole-pool `reject_duplicate_keys` before search; guarantees distinct subsets ⇒ distinct key SETS ⇒ distinct scriptPubKey/id (and the sorted-multiset variant). Verified live.
3. **Prefix-strength sized to `realized_s`** (= `S_own`/`S_own_sorted`/`S_opt`, NEVER the looser `P(pool,N)`), with the sorted symbol now unambiguous (I-1 enumeration-side collapse).
4. **Ambiguity full-scan for id/prefix-id (§4.4):** prefix-id NEVER gets early-exit (would miss a 2nd-match ambiguity → silent-wrong-wallet); the live `search` 2nd-match short-circuit is retained byte-identical for those paths.

The §5a premise-violation table fails SAFE on every row (under-supply → NO-MATCH refuse; over-supply cosigners in own-only → refuse up front; own-as-cosigner → distinct-keys refuse; multi-seed → out-of-scope redirect). The `@N=`⊕subset-search and `--account`⊕`--own-account-max` exclusions are well-specified and implementable.

The remaining §10 open items (#1 re-grep at plan base, #3 `--own-slots` pin — inferred range is safe, #4 opt-in rank→stratum offset arithmetic, #6 no-match UX `--origin` hatch) are legitimate IMPL micro-detail correctly deferred to the plan — none is a funds-safety-CONTRACT gap. A SPEC may defer the concrete rank→offset index layout to the plan when the count + the composition + the brute-force floor are pinned (they are).

---

## Closing verdict

**GREEN — 0 Critical, 0 Important.** All five round-1 Importants are CLOSED with source-verified corrections; all four Minors + the opt-in-stratified-generator note landed; the R0-SOUND combinatorics, bijection, backward-compat, and bounding did not regress; the fold introduced no new contradiction. The counts, the own-anchored bijection, the funds-safety floors, the early-exit contract, and the five source-contradictions are all nailed. **This SPEC is build-ready and advances to the plan-doc.** No new scope invented; no rubber-stamp — the verdict rests on re-grepped HEAD line numbers, two recomputed combinatorial margins, and a read of the live `search` full-scan/2nd-match-short-circuit logic.
