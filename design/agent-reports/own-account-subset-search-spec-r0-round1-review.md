> Reviewer: opus architect (R0 round 1) · 2026-06-20 · SPEC `design/SPEC_own_account_subset_search_2026-06-20.md` @ HEAD `34030a54` (branch `feature/own-account-subset-search`) · source-verified against base `82e58674` (v0.60.0). Brainstorm + its 2 reviews (R0-GREEN, 2 rounds) read in full.

**Verdict: RED — 0 Critical, 5 Important.**

The load-bearing combinatorics are SOUND: I independently re-derived the own-anchored count `S_own = C(K_own,j)·N!` two ways and confirmed the §4.1 composed-rank generator (combo-rank → CNS-unrank j-subset, perm-rank → `unrank_permutation(N)` over the N selected keys) is a genuine bijection onto exactly the `S_own` set with no cosigner-dropping placements — the I-1 danger direction is structurally closed. Every code citation is live and accurate at the SPEC's base SHA, INCLUDING the §10.2-flagged-unverified sorted-collapse site (located + read). The opt-in count `Σ_j C(K_own,j)·C(M_sup,N−j)·N!` is correct and double-count-free. Backward-compat at `K_own=j` is byte-exact.

But five Important findings stand between the SPEC and a build-ready funds-safe contract. None is a count/bijection error — the math passed. They are CONTRACT gaps where the SPEC asserts a behavior the live source contradicts, or defers a funds-safety decision to "confirm at impl" that an R0 must nail now:

1. **The sorted-shape composition (§3/§4.1) mis-describes the live MECHANISM** — today's collapse is an EVALUATOR filter over a full `n!` enumeration, not an enumeration restriction; the SPEC's "enumerate `C(K_own,j)` subsets, identity-order each" is the right TARGET but the wrong description of how it composes with the engine, and the actual composition has a subset-axis subtlety the SPEC must pin.
2. **The "existing 25 multisig-template tests stay GREEN" backward-compat claim (§7) is FALSE** for two pinned refuse-asserting tests that this SPEC's core feature deliberately FLIPS.
3. **verify-bundle parity (§2/§7-P4/§9) under-scopes NEW clap surface** — `--own-account-max` does not exist on verify-bundle today (its `--account` is a scalar `u32`, not a list; `own_account_max` is hardcoded `None`), so exposing it is a NEW flag NAME there that the schema_mirror gate WILL catch — contradicting §9's "only `--search-cosigner-subset` needs the schema mirror."
4. **The `--account`⊕`--own-account-max` mutual-exclusion (§2) cannot be implemented as specified** — `--account` has clap `default_value = "0"`, so it is ALWAYS present; a naive "both present → BadInput" check refuses `--own-account-max` used alone.
5. **The address-search early-exit funds-safety gate (§4.4) is deferred to a §10 "confirm at impl" open item** — but it is a behavior-CHANGE to the shared engine that must NOT leak to the v0.60.0 exact path or to id-search; the gate is funds-safety-load-bearing and must be a SPEC contract, not a plan guess.

Each is closeable in a single fold. Details + concrete fixes below.

---

## Citation audit (base `82e58674`) — ALL CONFIRMED, including the §10.2-flagged-unverified site

Every line the SPEC cites is live and accurate at the base SHA:

| SPEC cite | Confirmed at `82e58674` |
|---|---|
| `complete_multisig_template` `restore.rs:1416` | ✓ `pub(crate) fn complete_multisig_template<E: Write>` |
| `--own-account-max` refuse gate `:1434` | ✓ `if ctx.own_account_max.is_some() {` → `bad("…not supported yet…")` |
| under/over-supply gates `:1626`/`:1635` | ✓ `pool.len() < n` / `pool.len() > n` |
| `realized_s` `:1661` | ✓ `perm_count_u128(n, n)` → `bad("…candidate space overflow")` on `None` (`:1662`) |
| `reject_duplicate_keys` whole-pool `:1648` | ✓ on `pool_key_blobs` (`c.key65`, `:1647`), BEFORE `realized_s` |
| `perm_count_u128` `:1882` | ✓ `None` on `pool < n` (`:1883`) + `checked_mul` overflow (`:1888`) |
| `validate_prefix_strength(prefix.len(), realized_s)` `:1700` | ✓ |
| engine `search` `:551` | ✓; `unrank_permutation` `:494` | ✓; `total_candidates` `:509` | ✓ |
| `validate_prefix_strength` `:342` | ✓; `required_prefix_bytes` `:322` | ✓ |
| **`is_order_independent_shape` (§10.2 flagged UNVERIFIED)** | ✓ **`synthesize.rs:335`** `pub(crate) fn is_order_independent_shape(tree: &Node) -> bool` (SortedMulti/SortedMultiA → true, recurses through single-child + Tr) — the SPEC's grep block omitted the FILE; it lives in `synthesize.rs`, not `restore.rs`. |
| **the sorted address-search collapse site (§10.2, brainstorm cited `:1676`/`:1739`)** | ✓ `restore.rs:1676` `let sorted_shape = crate::synthesize::is_order_independent_shape(&d.tree);` and `:1739` `if sorted_shape && !assignment.iter().enumerate().all(\|(i,&v)\| i==v) { return false; }` — INSIDE the address-search evaluator closure. |

The pinned test `prefix_ladder_own_account_max_subset_space` (`permutation_search.rs:740`, body `739-758`) computes `S = P((11−4)+K, 11) = P(7+K, 11)` — confirming the SPEC's claim it must be UPDATED to `S_own`, and confirming the `K`-convention split (test-`K` = EXTRA accounts; SPEC `K_own` = TOTAL own candidates, `K_own = j + extra`). The four superseded sources (#28-SPEC §6.1 `:110`, FOLLOWUP `:48`, the pinned test, the P3a I-1-review option-1) are all re-verified live and the SPEC's supersession is explicit, not silent.

**External-fact note:** this is a self-contained combinatorics/search-engine change — no BIP-39/NDEF/OTP/SDK external protocol facts. The only authoritative sources are the toolkit's own pinned math + the four cited sources, all re-grepped here.

---

## Generator correctness (§4.1) — the load-bearing item — CORRECT (no finding)

I independently verified the own-anchored composed-rank generator is a bijection onto exactly the `S_own` set.

**The construction (§4.1):** `rank → (combo_rank ∈ [0, C(K_own,j)), perm_rank ∈ [0, N!))` via `combo_rank = rank / N!`, `perm_rank = rank % N!`. `combo_rank` → a j-subset of the K_own own indices (combinatorial-number-system unrank); the "N selected keys" = (those j own) ++ (all M cosigners); `perm_rank` → an ordering of those N keys into the N slots via `unrank_permutation(perm_rank, N)`.

**Bijection proof (independent):**
- **Domain size:** `rank ∈ [0, C(K_own,j)·N!)`. The `(combo_rank, perm_rank)` split is the standard mixed-radix decode — bijective onto `[0,C(K_own,j)) × [0,N!)`. ✓
- **`combo_rank` → j-subset:** CNS-unrank is a bijection `[0,C(K_own,j)) ↔ {j-subsets of K_own}`. Each `combo_rank` yields exactly one distinct j-subset; all `C(K_own,j)` subsets are hit. ✓
- **The "N selected keys" set is well-defined and has exactly N members:** j chosen own (distinct, from the subset) + M cosigners (distinct, all of them), and `j + M = (N−M) + M = N`. The distinct-keys floor (`:1648`) guarantees no own key byte-equals a cosigner key, so the N selected keys are genuinely N distinct keys (not N−1 with a collision). ✓ — and this is precisely why §5's "distinct-keys floor is now LOAD-BEARING" is correct: without it the "N selected keys" could be < N distinct, breaking the count.
- **`perm_rank` → ordering:** `unrank_permutation(perm_rank, N)` is a bijection `[0,N!) ↔ S_N` (the live Lehmer decode, `:494`). Each yields one distinct slot-assignment of the N selected keys. ✓
- **Composition covers exactly C(K_own,j)·N! distinct assignments, no cosigner-dropping:** every enumerated assignment uses exactly the M cosigners (ALL of them, by construction) + exactly j own — it can NEVER drop a cosigner (the cosigners are not subject to the combo-choice; they are unconditionally in the selected set). So no placement using `<M` cosigners is ever scanned. ✓ This is the I-1 danger direction (a scanned-but-uncounted cosigner-dropping placement) — **structurally impossible by this construction.** Two distinct ranks → distinct `(subset, ordering)` → distinct assignment (different subset ⇒ different key SET; same subset + different ordering ⇒ different slot map). No assignment is hit twice. ✓

`realized_s == enumerated count` is therefore structurally guaranteed: the generator's image is exactly the `S_own` set, `|image| = C(K_own,j)·N!`, and the §7 brute-force-reference test pins it. **The §3/§5 FLOOR holds for own-only.** No finding here — this is the part an R0 most needed to nail, and the SPEC nailed it.

(Cross-check the SPEC's worked margin: `K_own=32, j=4, M=7, N=11` → `C(32,4)=35,960`, `·11! = 1,435,408,128,000`, `log2 = 40.385`, `ceil((40.385+32)/8) = ceil(9.048) = 10` bytes. ✓ Recomputed exactly — matches §5.)

---

## I-1 (Important) — sorted-shape composition (§3/§4.1) mis-describes the live MECHANISM; the subset-axis composition is under-pinned

§3 states for an order-independent shape: "the search restricts ORDER to the identity placement (as v0.60.0 does) BUT still enumerates SUBSETS. `S_own_sorted = C(K_own, j)` … enumerate the `C(K_own,j)` subsets, identity order each." §4.1 echoes: "Sorted shape: drop the `perm_rank` factor — identity order only, space `C(K_own,j)`." The MATH is correct (distinct subsets ⇒ distinct sorted-pubkey multiset ⇒ distinct scriptPubKey, leaning on the distinct-keys floor — confirmed; there is NO hazard of two subsets colliding on the same sorted address). But the SPEC describes a mechanism that **does not match how the live code collapses**, and the real composition has a subtlety the SPEC must pin.

**What the live code actually does (`restore.rs:1739`, confirmed):** v0.60.0 does NOT restrict the ENUMERATION for sorted shapes. The engine `search(n, …)` ALWAYS enumerates the full `[0, n!·outer)` rank space (it has no knowledge of `sorted_shape`); the collapse is an **evaluator-closure filter**: `if sorted_shape && assignment != identity { return false; }`. So today, for a sorted wallet, the engine scans all `n!` orderings and the evaluator rejects all but the identity permutation — the "collapse to 1" is a *match-side* filter, not an enumeration-side restriction.

**Why this matters for the SPEC contract:** the SPEC's `S_own_sorted = C(K_own,j)` is the count of DISTINCT WALLETS, but it is NOT (under the live mechanism) the size of the ENUMERATED rank space. If the plan author reads §4.1's "drop the perm_rank factor — identity order only, space `C(K_own,j)`" literally and sizes the engine's rank space to `C(K_own,j)`, that is one valid design (enumerate subsets only, force identity order). But the §3 FLOOR is "realized_s == enumerated count" — and `realized_s` feeds `validate_prefix_strength`. Under the SPEC there are now TWO defensible designs and they size the prefix DIFFERENTLY:

- **(A) Enumeration-side:** the own-anchored generator drops `perm_rank` for sorted shapes ⇒ rank space = `C(K_own,j)` ⇒ `realized_s = C(K_own,j)`. Tighter prefix.
- **(B) Evaluator-side (the live v0.60.0 mechanism, extended):** keep the full `C(K_own,j)·N!` enumeration, evaluator-filter non-identity orderings ⇒ enumerated count = `C(K_own,j)·N!` but realized DISTINCT-MATCH count = `C(K_own,j)`. Now `realized_s` (for prefix sizing — what set could collide?) should be `C(K_own,j)` (distinct wallets), but the SCANNED count (for the cap ETA) is `C(K_own,j)·N!`. These two numbers DIVERGE for sorted shapes, and the SPEC's single `realized_s` symbol conflates them.

The SPEC §3 asserts "`S_own_sorted = C(K_own, j)` (no `·N!`)" and "the realized space + the address-search collapse must compose" — but it does not state WHICH mechanism, and the prefix-sizing-vs-cap-ETA divergence under the evaluator-side mechanism is unaddressed. For a NON-sorted shape the two coincide (`realized_s = C(K_own,j)·N!` is both the scanned count and the colliding-set size); the divergence is sorted-specific and funds-safety-relevant (under-sizing the prefix is the danger; over-scanning is only a perf/cap concern).

**The fix (single fold).** §3/§4.1 must:
1. State the chosen mechanism explicitly. Recommended: **(A) enumeration-side** — the own-anchored generator emits `C(K_own,j)` identity-ordered subsets for sorted shapes (drop `perm_rank`), so the enumerated count, the colliding-set size, and the cap-ETA scan count all equal `C(K_own,j)` (the three collapse cleanly, the FLOOR holds verbatim, and the evaluator's `sorted_shape && !identity → false` filter becomes redundant for the subset path — keep it as defense-in-depth or note it's subsumed). This avoids the prefix-vs-ETA divergence and is the cleaner extension.
2. If instead (B) is chosen (reuse the live evaluator filter unchanged + full enumeration), the SPEC MUST split the symbol: `realized_s_for_prefix = C(K_own,j)` (distinct-wallet collision set) vs `scan_count_for_cap = C(K_own,j)·N!`, and state that `validate_prefix_strength` takes the former. Leaving one `realized_s` symbol for both is the funds-safety ambiguity.
3. Either way, state the composition note the R0-r2 review flagged as a SPEC carry: today's `assignment == identity` skip would discard every non-first SUBSET if naively reused, because under subset-search the "identity" placement of subset #2 is a *different* assignment than the identity of subset #1 — the collapse must restrict ORDER WITHIN a subset, not collapse ACROSS subsets. The SPEC says this in prose ("restrict ORDER but STILL enumerate SUBSETS") but must tie it to the concrete `assignment == identity` predicate so the plan author does not reuse `:1739` verbatim and silently drop all-but-the-first subset.

This is Important (not Critical) because the MATH is right and the safe direction is identifiable — but a SPEC that leaves the mechanism + the sorted prefix-sizing symbol ambiguous hands the plan author a funds-safety guess (which `S` sizes the prefix for a sorted over-supplied wallet?).

---

## I-2 (Important) — §7's "the existing 25 multisig-template tests stay GREEN" is FALSE; two pinned refuse-asserting tests FLIP under this SPEC's core feature

§7-P2 and §5 (axis-5) commit "the existing 25 multisig-template tests stay GREEN" + "the `multi_account_own_resolves_both_slots` pin stay[s] GREEN" as the backward-compat anchor. The `multi_account_own_resolves_both_slots` pin (`cli_restore_md1_template_multisig.rs:635`) DOES stay GREEN (it uses an exact pool `--account 0,1` — confirmed it's the byte-identical exact path). But TWO existing pinned tests in the same file assert the EXACT behavior this SPEC removes, and CANNOT stay GREEN:

1. **`own_account_max_flag_refuses_with_actionable_message` (`:677`)** — asserts `--own-account-max 3` REFUSES with a message naming `--account` and NOT containing "no match." This SPEC's entire P2 deliverable is to make `--own-account-max` SEARCH instead of refuse. This test MUST be REWRITTEN to assert the new search behavior (or deleted + replaced). It does not "stay GREEN."
2. **`pool_larger_than_slots_refuses_with_actionable_message` (`:715`)** — `--account 0` + cosigner-B + extra-outsider-C ⇒ pool 3 > n 2 ⇒ refuses. Under the SPEC §5a this is now the OWN-ONLY over-supplied-cosigners case (`M' > M`), which §5a says "**REFUSE up front** ('own-only needs exact cosigners; use `--search-cosigner-subset`')". So the REFUSAL outcome is preserved — but it now flows through a DIFFERENT gate with a DIFFERENT message, and the test's current assertion (`low.contains("--account") || "more keys" || "over-supply" || "exactly"`) may or may not match the new message. The test's wording assertion likely needs updating; at minimum the SPEC must acknowledge the refusal PATH changes.

There are 27 `#[test]` fns in `cli_restore_md1_template_multisig.rs` (the SPEC's "25" is itself imprecise — possibly excluding these 2 refuse tests, but then it cannot claim they "stay GREEN" while also re-enabling the flag). The funds-safety risk is small here (it's a test-maintenance contract gap, not a runtime hole), but for an R0 the backward-compat axis must be HONEST: this SPEC FLIPS at least one pinned test's asserted behavior by design, and the SPEC currently claims the opposite.

**The fix (single fold).** §7-P2 + §5 axis-5: replace "the existing 25 … stay GREEN" with a precise split: (a) the EXACT-POOL tests (incl. `multi_account_own_resolves_both_slots`, the explicit-`@N=` tests, all `pool.len()==n` completions) stay byte-identical GREEN — that IS the backward-compat anchor; (b) `own_account_max_flag_refuses_with_actionable_message` is REWRITTEN to assert the new search-completes behavior (the SPEC should name it as a converted test, RED-first per the §7 TDD plan); (c) `pool_larger_than_slots_refuses_…` is updated to assert the new own-only-refuses-extra-cosigners message (refusal preserved, message changed). State the count precisely after audit.

---

## I-3 (Important) — verify-bundle parity (§2/§7-P4/§9) under-scopes NEW clap surface; the §9 schema_mirror scoping is WRONG for verify-bundle

§2 says "verify-bundle: the same flags are exposed (shared core)." §7-P4 commits "verify-bundle parity." §9 scopes the lockstep as: "only the NEW `--search-cosigner-subset` flag needs the schema mirror … on restore + verify-bundle; `--own-account-max` NAME is unchanged (refuse→search is behavior-only)." The last clause is TRUE for restore but FALSE for verify-bundle, and the "shared core ⇒ exposed for free" framing hides real CLI work:

**Live verify-bundle surface (`verify_bundle.rs`, confirmed):**
- `--own-account-max` **does NOT exist** on verify-bundle. The ctx field is hardcoded `own_account_max: None` (`:865`). There is no `#[arg(long = "own-account-max")]` in the verify-bundle args struct.
- `--account` is a SCALAR: `#[arg(long, default_value = "0")] pub account: u32` (`:63`), wired as `own_accounts: vec![args.account]` (`:862`). It is NOT a `Vec<u32>` list (restore's is `Vec<u32>`, `:106`). So even the OWN-only-via-`--account`-list path is unavailable on verify-bundle today.
- `--cosigner`, `--search-address`, `--search-addr-min/max`, `--search-chain`, `--expect-wallet-id`, `--accept-search-time` DO exist on verify-bundle — so the search machinery is reachable; only the own-account-range surface is missing.

**Consequences the SPEC must address:**
1. Exposing `--own-account-max` on verify-bundle is a **NEW flag NAME on the verify-bundle subcommand** — the `schema_mirror` gate (clap flag-NAME parity, per CLAUDE.md) WILL fire on it at the next GUI pin bump. §9's "only `--search-cosigner-subset` needs the schema mirror" is wrong: BOTH `--own-account-max` AND `--search-cosigner-subset` are new NAMES on verify-bundle. (On restore, `--own-account-max` already exists ⇒ no schema delta there — §9 is right for restore, wrong for verify-bundle.)
2. The SPEC must decide whether verify-bundle's `--account u32` becomes a list `Vec<u32>` (to support own-only-via-account-list parity) or whether verify-bundle own-only is `--own-account-max`-only. If `--account` changes arity (scalar → list), that is a behavior change to an existing flag (an empty/multi list) — state in/out of scope. (Recommended: add `--own-account-max` to verify-bundle but keep `--account` scalar there, since verify-bundle's "you already hold the bundle" semantics make the over-supplied-list case marginal — §2 already concedes verify-bundle over-supply is "unusual.")
3. §9 must list the FULL schema_mirror delta on verify-bundle (`--own-account-max` + `--search-cosigner-subset`) and the manual edits for both subcommands, or the lagging schema-mirror gate (CLAUDE.md: it fires only on the next GUI pin bump) silently accumulates the miss.

**The fix (single fold).** §2: state explicitly that `--own-account-max` is NEW clap surface on verify-bundle (it exists only on restore today; verify-bundle hardcodes `None`), and pin whether `--account` stays scalar on verify-bundle. §9: correct the schema_mirror scope to "`--own-account-max` + `--search-cosigner-subset` are both new NAMES on the verify-bundle subcommand ⇒ both need the schema mirror there; on restore only `--search-cosigner-subset` is new" + the manual rows for both subcommands.

---

## I-4 (Important) — the `--account` ⊕ `--own-account-max` mutual-exclusion (§2) cannot be implemented as written: `--account` has clap `default_value = "0"` and is ALWAYS present

§2 specifies: "Mutually exclusive with `--account` … supply EITHER an explicit `--account <list>` OR `--own-account-max K_own`. Both → `BadInput` ('use --account OR --own-account-max, not both')." This contract is unimplementable as a presence check because `--account` is `#[arg(long, value_delimiter = ',', default_value = "0")] pub account: Vec<u32>` (`restore.rs:106`) — clap ALWAYS populates it (to `[0]` when the user omits it). So:
- `args.account` is `[0]` whether the user typed `--account 0` or typed nothing.
- A naive "if `!account.is_empty()` && `own_account_max.is_some()` → BadInput" refuses `--own-account-max 5` used ALONE (because `account == [0]` from the default) — **breaking the feature's primary invocation.**

This is a funds-safety-adjacent gate (the SPEC's §5/§2 premise gating depends on cleanly distinguishing the two own-supply modes), and getting it wrong fails CLOSED (refuses the valid case) — annoying, not dangerous — but it makes the SPEC's stated contract literally unbuildable, which an R0 must catch.

**The fix (single fold).** §2 must specify the DETECTION mechanism, not just the rule. Two correct options:
1. **clap `conflicts_with` (preferred, codebase precedent at `restore.rs:86`):** `#[arg(long = "own-account-max", conflicts_with = "account")]`. clap's `conflicts_with` correctly ignores a `default_value` — it fires only when `--account` was EXPLICITLY supplied (value source = CommandLine), so `--own-account-max 5` alone passes, and `--account 0,1 --own-account-max 5` errors at clap-parse time with clap's own message. This also moves the refusal to parse-time (cleaner than a `bad()` in the core).
2. If a custom message is required, use `ArgMatches::value_source(\"account\") == Some(ValueSource::CommandLine)` to distinguish supplied-vs-default before the `bad()`.

State which, and note that the naive `is_empty()`/`is_some()` check is WRONG (it refuses the headline case). Also confirm the same default-vs-supplied distinction for any other gate that keys off "`--account` was given" (e.g. the §5a own-only-vs-explicit-account branching).

---

## I-5 (Important) — the address-search early-exit funds-safety gate (§4.4) is deferred to a §10 "confirm at impl" open item; it is a behavior-CHANGE to the shared engine and must be a SPEC contract, not a plan guess

§4.4 grants address-search **early-exit-on-first-match** ("the perf win that makes large pools tractable") and justifies it via collision-freeness + the distinct-keys floor. The justification is CORRECT for the subset axis: two distinct subsets → distinct key SET → distinct scriptPubKey (via the floor), so an address match is provably unique ⇒ early-exit is safe. No finding on the *safety of early-exit in principle*. The finding is that §4.4 then says "Confirm against the current engine's behavior (if it currently full-scans address-search, this is a deliberate, correctness-preserving change …)" and §10.5 DEFERS the entire gating decision: "Whether address-search early-exit changes the v0.60.0 EXACT-path behavior (must be gated to the over-supply path so the exact path is byte-unchanged) — confirm at impl." That deferral is the problem.

**Verified facts the SPEC should have stated (not deferred):**
1. **The live engine `search` (`:551`) DOES full-scan address-search today** — confirmed. It does NOT early-exit on first match; it scans the whole `[0, n!·outer)` space and short-circuits ONLY at the SECOND match (ambiguity detection: `global_matches.fetch_add` → `≥2 → stop`). This holds for BOTH `SearchMode::Id` AND `SearchMode::Address` — the doc-comment at `:535-548` is explicit ("does NOT stop at the first match … favor full-scan for ambiguity DETECTION"). So §4.4's "if it currently full-scans" is not an "if" — it DOES, and the SPEC can state it as fact.
2. **Adding first-match early-exit is therefore a real behavior CHANGE to the shared engine**, and the engine is shared by the v0.60.0 EXACT path (`pool.len()==n`, which routes through the SAME `search`). If early-exit is added unconditionally, the v0.60.0 exact-path address-search changes from full-scan to first-match-exit — which §7/§5 axis-5 requires be BYTE-UNCHANGED. So the gate ("over-supply path only") is funds-safety-load-bearing, not a perf detail.
3. **There is NO existing per-mode early-exit knob in `search`** — the early-exit/full-scan choice is hardcoded by the `≥2` ambiguity logic. Adding a first-match-exit mode means a NEW parameter or a NEW `SearchMode`/flag on the engine. The SPEC defers HOW this is gated, but the gate is exactly where a funds-safety regression would hide (leak early-exit to id-search → miss a 2nd-match ambiguity → silently pick a wrong wallet; leak to the exact path → change v0.60.0 bytes).

**Why this is Important, not a plan open-item:** §10 open-items are acceptable for IMPL micro-detail, but the contract "address-search early-exit is enabled ONLY on the over-supply path AND ONLY for address-search AND NEVER for prefix-id (which stays full-scan for ambiguity) AND the v0.60.0 exact path is byte-unchanged" is the FUNDS-SAFETY CONTRACT, and the SPEC currently states the first three but DEFERS the fourth (exact-path byte-invariance) to "confirm at impl." The whole point of §4.4's "gate behind the over-supply path so the v0.60.0 exact path is byte-unchanged" is a contract; demoting it to a §10 "confirm at impl" is the gap. A plan author who reads §10.5 as "maybe early-exit changes the exact path, decide later" could ship an engine change that silently alters v0.60.0 — the exact regression the post-impl adversarial review exists to catch, but which the SPEC should forbid up front.

**The fix (single fold).** Promote the gate from §10.5 into the §4.4 CONTRACT, stated as a hard invariant: "address-search first-match early-exit is enabled IFF (over-supply path: `realized_s != n!`) AND mode == Address; the v0.60.0 exact path (`pool.len()==n`) and ALL id-search/prefix-id paths retain the unchanged full-scan-with-2nd-match-ambiguity behavior — byte-identical to v0.60.0. The §7 TDD pins a v0.60.0-exact-path address-search outcome byte-for-byte before/after." Name the engine surface (a per-call `early_exit: bool` or a `SearchMode::Address { early_exit }` variant) so the plan author does not improvise the gating. Delete §10.5 (now contract) — or reduce it to "the engine-API SHAPE of the early-exit knob" (a true impl-detail) once the INVARIANT is in §4.4.

---

## Findings by your scrutiny axes

**1. Own-anchored generator bijective onto S_own (load-bearing):** CORRECT — independently verified bijection, no cosigner-dropping, `realized_s == enumerated` structurally guaranteed (above). The strongest part of the SPEC. **No finding.**

**2. Sorted-shape composition:** math CORRECT (distinct subsets ⇒ distinct sorted addresses, via the distinct-keys floor — no two-subset-collision hazard), but the SPEC mis-describes the live evaluator-filter mechanism and leaves the sorted prefix-sizing-vs-cap-ETA symbol ambiguous — **I-1.**

**3. Opt-in enumeration (§4.3) realized_s fidelity:** the sum `Σ_{j∈[j_min,j_max]} C(K_own,j)·C(M_sup,N−j)·N!` is CORRECT and double-count-free — partitioning by `j` (own-slot-count) makes the j-strata disjoint (each assignment has exactly one j), and within a stratum `C(K_own,j)·C(M_sup,N−j)·N!` is choose-own × choose-cosigner × order-all-N (the same bijection structure as own-only, generalized). `j_min=1`/`j_max=min(K_own, N−1)` is sound (≥1 own via `--from`, ≥1 cosigner). The §6 hard-ceiling refuses before this DoS's. **NOT hand-waved at the COUNT level** — but it IS a Σ without a concrete UNRANK/generator (§4.3 gives the count, not the "stratified unrank" that enumerates it bijectively: e.g. partition `[0, S_opt)` by cumulative j-strata, then within stratum-j compose `(own-combo-rank, cosigner-combo-rank, perm-rank)`). For OWN-ONLY the SPEC gave the generator (§4.1); for OPT-IN it gave only the count. Per the R0-r2 ruling this is acceptable to DEFER to the plan IF the safe-bounding (enable + ceiling + cap) is committed — which it is (§4.3 + §6). I rate this **a sub-Important borderline, folded into a note** rather than a standalone finding: the count is build-ready; the unrank is the same I-1-class obligation the §7-P3 TDD already pins ("opt-in completes to the golden" + the ceiling refuse). **Recommend the SPEC add one sentence** that the opt-in unrank is the stratified composition of §4.1's primitives (CNS-unrank own-combo + CNS-unrank cosigner-combo + `unrank_permutation`), with the same brute-force-reference floor as §4.1 — so it is not an *unbounded* plan guess. Not RED-blocking on its own (the bound is safe), but fold it with I-1.

**4. Early-exit funds-safe:** safe in principle (verified), but the exact-path byte-invariance gate is deferred to §10.5 instead of being the §4.4 contract, over a shared engine that today full-scans both modes — **I-5.**

**5. Backward-compat / SemVer:** the `S_own = N!` at `K_own=j` byte-identity is exact (✓). SemVer MINOR + md/mk NO-BUMP correct. BUT the "25 tests stay GREEN" claim is false for the 2 refuse-tests (**I-2**), and the verify-bundle schema-mirror scope is wrong (**I-3**).

**6. Mutual-exclusions + premise gates:** §5a's own-only premise-violation table is complete and fails SAFE (carried verbatim from the GREEN brainstorm §3a — re-verified each row terminates in refuse/NO-MATCH). The `@N=`⊕subset-search exclusion and own-only-refuses-extra-cosigners are well-specified. BUT the `--account`⊕`--own-account-max` exclusion is unimplementable as a presence check (**I-4**). One additional combination to pin (Minor, below): `--own-account-max` + `--search-address` + `--expect-wallet-id` both supplied — the SPEC should state which search mode wins (today id vs addr are mutually-decided by `id_search`/`addr_search` flags at `:1665-1666`; confirm the over-supply path inherits that precedence and doesn't run both).

**7. Bounding (§6):** `K_own ≤ 256` is defensible (a sane account ceiling). `S_MAX = 1e15` is defensible: at the #28 ~170M cand/min benchmark, `1e15 / 170e6 ≈ 5.88e6 min ≈ 4083 days` — clearly refuse, as the SPEC says. The ceiling-before-calibration ordering is correct in intent (§6: "refuse (before cap calibration, distinct from the time-cap)"). `u128` overflow → `bad` (not panic) is already live (`perm_count_u128`). One GAP to pin (folded into I-1/§6): the SPEC must state the ceiling is checked on the OWN-ANCHORED `realized_s` (`S_own`/`S_opt`), computed via overflow-checked `checked_mul` on `C(K_own,j)·N!` — NOT on the looser `P(pool,N)`. And the combo-count `C(K_own,j)` itself must be overflow-checked (it can be large for K_own near 256; e.g. `C(256,128)` overflows u128 — so the combo-count needs its own `checked_mul`/refuse, which §4.1's "the combo-count all REFUSE on u128 overflow → None → bad" asserts but the SPEC should pin the helper). **Minor/folded, not standalone.**

**8. Completeness:** the funds-safety CORE (count, bijection, distinct-keys floor, ambiguity full-scan for id, premise-violation table) is on the page and correct. The five Importants are all CONTRACT gaps (mechanism description, test-flip honesty, verify-bundle surface, mutex implementability, early-exit gate promotion) — none is a hidden math hole. The §10 open items: #1 (re-grep), #3 (`--own-slots` pin — genuinely plan-level, the inferred range is safe), #4 (opt-in sum × ceiling — fold the one-sentence unrank note per axis-3), #6 (no-match UX) are acceptable plan-level. #2 (sorted-collapse location) is RESOLVED by this review (confirmed `synthesize.rs:335` + `restore.rs:1676`/`:1739`) — update §10.2 to cite the confirmed site. #5 (early-exit gate) must be PROMOTED out of §10 into §4.4 (I-5).

---

## MINOR (non-blocking — fold opportunistically with the Importants)

- **m-1 — §10.2 is now resolved.** The `is_order_independent_shape` site is confirmed at `synthesize.rs:335` (not `restore.rs` — the SPEC's §-grep block omitted the file) and the collapse at `restore.rs:1676`/`:1739`. Replace §10.2's "verify" with the confirmed citation.
- **m-2 — the §7-P3 opt-in TDD should pin the stratified-unrank bijection** (the same brute-force-reference floor as own-only, per axis-3), not just "completes to the golden + ceiling refuse." A golden-match test does not pin the enumerate≡count floor for the opt-in strata; add the exhaustive-small-N reference test for `S_opt` too.
- **m-3 — combo-count overflow.** §4.1 says "`factorial`/`perm_count`/the combo-count all REFUSE on u128 overflow." Pin the combo-count helper (CNS / `C(K_own,j)`) as overflow-checked — `C(256,128)` overflows u128, and `K_own ≤ 256` permits it. State the helper returns `None` → `bad`, same as `perm_count_u128`.
- **m-4 — id+addr both supplied under over-supply.** Confirm the over-supply path inherits the v0.60.0 `id_search`/`addr_search` precedence (`restore.rs:1665-1666`) and never runs both modes in one search; state it in §2/§4.4.

---

## To turn GREEN (single fold closes each)

1. **I-1:** §3/§4.1 — state the sorted-shape collapse MECHANISM (recommend enumeration-side: generator emits `C(K_own,j)` identity-ordered subsets, all three counts collapse to `C(K_own,j)`); if evaluator-side is kept, SPLIT `realized_s_for_prefix = C(K_own,j)` vs `scan_count_for_cap = C(K_own,j)·N!` and route `validate_prefix_strength` to the former; tie the "restrict order within a subset, not across subsets" prose to the concrete `assignment == identity` predicate so `:1739` isn't reused verbatim.
2. **I-2:** §7-P2/§5 — replace "the existing 25 stay GREEN" with: exact-pool tests (incl. `multi_account_own_resolves_both_slots`) stay byte-identical GREEN; `own_account_max_flag_refuses_…` is REWRITTEN (refuse→search, RED-first); `pool_larger_than_slots_refuses_…` is updated to the new own-only-refuses-extra-cosigners message. State the precise count post-audit (27 `#[test]` in the file).
3. **I-3:** §2/§9 — state `--own-account-max` is NEW clap surface on verify-bundle (hardcoded `None` today, `--account` scalar there); decide verify-bundle's `--account` arity; correct §9 schema_mirror scope to BOTH `--own-account-max` + `--search-cosigner-subset` on verify-bundle (+ both manual rows).
4. **I-4:** §2 — specify the mutex via clap `conflicts_with = "account"` (or `value_source` check), NOT an `is_some()`/`is_empty()` presence check (which refuses `--own-account-max` used alone, since `--account` defaults to `[0]`). Cite the `restore.rs:86` `conflicts_with` precedent.
5. **I-5:** §4.4 — promote the early-exit gate from §10.5 into the contract: address-search first-match early-exit IFF over-supply path AND mode==Address; the v0.60.0 exact path + ALL id/prefix paths retain unchanged full-scan-with-2nd-match-ambiguity, byte-identical; name the engine knob (`early_exit: bool` / `SearchMode::Address { early_exit }`); §7 pins a before/after byte-identical exact-path address-search outcome.
6. Fold the Minors (m-1 §10.2 resolved-citation; m-2 opt-in reference test; m-3 combo-count overflow helper; m-4 id+addr precedence) opportunistically.

Re-dispatch this R0 after the fold. The load-bearing combinatorics + the own-anchored bijection + the opt-in count are SOLID and verified — the five Importants are all build-readiness contract gaps (not math holes), each a single mechanical fold. Once the sorted mechanism is pinned, the test-flip is honest, the verify-bundle surface + schema scope is corrected, the mutex is implementable, and the early-exit gate is a contract not a deferral — this SPEC is GREEN and advances to the plan-doc.
