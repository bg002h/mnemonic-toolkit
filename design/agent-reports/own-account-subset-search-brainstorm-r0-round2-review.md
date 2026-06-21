> Reviewer: opus architect (R0 round 2) · 2026-06-20 · brainstorm `design/BRAINSTORM_own_account_subset_search_2026-06-20.md` @ `e3d7b4eb` (branch `feature/own-account-subset-search`, fold commit `e3d7b4eb` "fold R0-r1 (3I)") · source-verified against the live tree at HEAD `e3d7b4eb` (= v0.60.0 src, master `4d5872ed` unchanged in the relevant files).

**Verdict: GREEN — 0 Critical, 0 Important.**

The fold closes all three round-1 Importants cleanly. I independently re-derived the load-bearing count by two methods (both equal `C(K_own, j)·N!`, both collapse to `N!` at no-over-supply), recomputed the I-2 prefix margin exactly (10 bytes, matching the brainstorm), and confirmed the §3a premise-violation table fails SAFE across all four modes. No fold-induced drift: the round-1 SOUND items did not regress, the two prescribed new open points were added without duplicating the original eight, the §4 manual caveat is present, and the live citations spot-checked accurate at HEAD. The one residual is a sub-Minor numeric-label imprecision in a "for reference" parenthetical that does not affect any safety conclusion. **This design may advance to SPEC.**

---

## Citation re-confirmation at HEAD `e3d7b4eb` (spot-check of load-bearing lines)

All accurate; the fold shifted nothing in src (v0.60.0 unchanged in restore.rs / permutation_search.rs):
- `complete_multisig_template` — `restore.rs:1416` ✓ (the shared core; `own_account_max` field at `:134`/`:1134`, ctx-wire at `:1370`).
- `--own-account-max` refuse gate — `restore.rs:1434` (`if ctx.own_account_max.is_some()`) ✓, with the I-1-gate comment `:1426-1432` confirming the "first n pool entries" mechanism the brainstorm lifts.
- under/over-supply gates — `:1626` (`pool.len() < n`) / `:1635` (`pool.len() > n`) ✓.
- `realized_s = perm_count_u128(n, n)` — `:1661-1662` ✓; the `None`→`bad("…candidate space overflow")` REFUSE path confirmed (no panic — the #28 M1 lesson, still honored).
- distinct-keys floor — `reject_duplicate_keys(&pool_key_blobs)` at `:1648`, blobs built from `c.key65` at `:1647`, BEFORE `realized_s` (`:1661`) ✓ — the I-2/I-3-load-bearing ordering ("floor fires before the search") holds.
- `validate_prefix_strength(prefix.len(), realized_s)` — `restore.rs:1700` ✓.
- `is_order_independent_shape` / `sorted_shape` — `:1676` and its use `:1739` (`if sorted_shape && !assignment.iter().enumerate().all(|(i,&v)| i==v)`) ✓ — confirms open-point 5's framing exactly (today's collapse skips every non-identity placement).
- `perm_count_u128` — `restore.rs:1882`, returns `None` on `pool < n` and on `checked_mul` overflow ✓.
- engine symbols — `SearchMode:247`, `reject_duplicate_keys:289`, `required_prefix_bytes:322`, `validate_prefix_strength:342`, `factorial:481`, `unrank_permutation:494`, `total_candidates:509`, `search:551` ✓.
- pinned test `prefix_ladder_own_account_max_subset_space` — `permutation_search.rs:740` (block `739-758`), helper `perm_count` at `:723`, computes `S = P((n−own)+K, n) = P(7+K, 11)` ✓. (The brainstorm cites `:739`/`:741` in two places — both inside the block; accurate.)

External-fact note: this is a self-contained combinatorics/search-engine change (no BIP-39 / NDEF / OTP / SDK external protocol facts). The only "authoritative source" to verify against is the toolkit's own pinned math and the four cited sources — all re-grepped above and below.

---

## I-1 (round-1 Important) — CLOSED

The fold puts the derived own-anchored count on the page (D2, `brainstorm:26-33`) and resolves every sub-requirement:

**(a) Derivation CORRECT — independently re-derived, two methods.** I re-derived from scratch (numeric check across six `(N,M,K_own)` cases):
- Method A (choose-then-assign): `C(K_own, j)·N!`.
- Method B (place-directly): `P(N,M)·P(K_own,j) = N!/j! · K_own!/(K_own−j)!`.
- These are algebraically equal (`P(N,M)=N!/j!` since `M=N−j`; `P(K_own,j)·1/j! · N!/... ` reduces to `C(K_own,j)·N!`). Numeric confirmation: e.g. `N=11,M=7,K_own=32` → both = `1,435,408,128,000`; `N=5,M=3,K_own=5` → both `1200`; `N=4,M=2,K_own=4` → both `144`. **Equal in every case.** The brainstorm's cross-check at `:28` is the same Method B and is arithmetically sound.

**(b) Collapses to `N!` at `K_own = j` — CONFIRMED.** `C(j,j)·N! = 1·N! = N!`. Numeric: `N=11,M=7,j=4,K_own=4` → `39,916,800 = 11!` ✓; `N=5,M=3` → `120` ✓; `N=4,M=2` → `24` ✓. This is byte-identical to v0.60.0's `perm_count_u128(n,n)` at `restore.rs:1661` (the brainstorm states this at `:29`) — backward-compat is exact.

**(c) Refines `realized_s = P(pool,N)` and supersedes the four pinned sources — CONFIRMED, all four re-verified live:**
- pinned test `prefix_ladder_own_account_max_subset_space` (`permutation_search.rs:739` — verified body computes `P(7+K,11)`; comment to UPDATE it to `S_own` is at `brainstorm:31`, with the test-`K`-vs-`K_own` relation stated).
- FOLLOWUP "Fix" line (`FOLLOWUPS.md:48` — verified: "size `realized_s = P(pool, n)` to the truly-enumerated space"; the brainstorm correctly flags this as superseded).
- #28 SPEC §6.1 (`SPEC_bundle_md1_template_multisig_2026-06-20.md:110` — verified: "S = realized candidate count … the larger `P((N−own)+K, N)` … for `--own-account-max K`"; superseded).
- P3a I-1-review option-1 (`template-multisig-p3a-completion-exec-review.md:64` — verified: "enumerate `P(pool, n)` … `realized_s = P(pool,n)`"; superseded).
The brainstorm states it is STRICTLY `≤ P(pool,N)` because it forbids cosigner-dropping placements (`brainstorm:30`) — correct: `P(35,11) ≈ 1.67e16` vs `S_own ≈ 1.44e12` for the worked case. The supersession is explicit, not silent.

**(d) Enumerated≡counted FLOOR committed + `realized_s = S_own` computable up-front — CONFIRMED.** Stated twice, in D2 (`brainstorm:33`) and the Floors (`:52`): the enumerator must scan EXACTLY the `S_own` set (no cosigner-dropping placements), proven by an exhaustive small-N brute-force-reference test (engine's enumerated set ≡ independently-generated valid set) plus a `realized_s == enumerated count` assertion. `S_own` is closed-form ⇒ computable before enumeration for the cap ETA + `validate_prefix_strength` (`:33`). The SAFE-direction argument is correct and on the page (`:33`, `:52`): under-sizing the prefix is the danger, and the floor closes it — a scanned-but-uncounted placement would be "a collision the prefix wasn't sized for"; the brainstorm names this as "the dangerous direction" and links it to the floor. The `realized_s = S_own` direction (sizing to the SMALLER count) is therefore SAFE *iff* the enumerator is bijective onto the counted set — exactly the link the review demanded.

**(e) `K` disambiguated — CONFIRMED.** `brainstorm:24` defines `K_own` = total own-candidate count (the `--account` list length or `--own-account-max K_own` ⇒ accounts `0..K_own−1`) and explicitly contrasts the pinned test's / #28-SPEC's `K` = EXTRA accounts beyond `j` (pool `= (N−own)+K`), with the stated relation `K_own = j + extra`. The notation block at `:24` carries `N`/`M`/`j`/`K_own` glossary-style and states `K_own ≥ j` (= `K_own + M ≥ N`).

I-1 is **CLOSED**.

## I-2 (round-1 Important) — CLOSED

**(a) Address-search collision-freeness depends on the whole-pool distinct-keys floor — STATED.** Floors `brainstorm:51`: over-supply adds distinct injective SUBSETS (not just orderings); two distinct subsets have distinct key SETS ⇒ distinct scriptPubKey "ONLY because no two pool keys are byte-identical (`pool_key_blobs` compares the 65-byte `key65`…)". This is the dependency the review asked be made explicit, and it cites `restore.rs:1648` (the floor) — verified live, runs on the full `pool` before `realized_s`.

**(b) Prefix-strength margin re-derived against `S_own` — ARITHMETIC CONFIRMED EXACT.** D4 `brainstorm:39`: `N=11, j=4, M=7, K_own=32` → `C(32,4)=35,960`, `·11! ≈ 1.435e12`, `log2 ≈ 40.4`, `ceil((40.4+32)/8) = ceil(9.05) = 10` bytes. I recomputed independently: `C(32,4)=35960` ✓, `S_own = 1,435,408,128,000` ✓, `log2 = 40.3846` ✓, `ceil((40.3846+32)/8) = ceil(9.048) = 10` ✓. The brainstorm's claim that the looser `P(pool,N)` ladder over-sizes is correct: `P(32,11)` and `P(35,11)` both demand 11 bytes vs `S_own`'s 10 — `S_own` is the faithful, tighter sizing, and still safe (the 32 padding bits give collision-prob `≤ ~2e-10` independent of S). (See sub-Minor m-1 below for a numeric-label nit in the parenthetical that does not change this.)

**(c) id-search full-scan ambiguity over the own-anchored enumerated set — STATED.** Floors `brainstorm:53`: "id-search full-scan ambiguity certification is over the OWN-ANCHORED enumerated set (the same set `realized_s` counts) — closes the I-1↔I-2 coupling." D3 (`:35-36`) preserves the unchanged ≥2-match → Ambiguous → refuse gate. Correct: the engine counts matches across whatever set it enumerates (`search:638-647` increments `global_matches` per match), so certification is sound iff the enumerated set is the own-anchored one (the I-1 floor) — the coupling is named and closed.

I-2 is **CLOSED**.

## I-3 (round-1 Important) — CLOSED

The §3a failure-mode table (`brainstorm:58-65`) covers all four round-1 modes, each resolving SAFE:
1. **Under-supply cosigners** (`M' < M`) → NO-MATCH → refuse (own-anchored search forces `j_assumed > j_true`; real assignment needs cosigner keys it lacks → unreachable). Actionable message named. ✓
2. **Over-supply cosigners in own-only** (`M' > M`) → **REFUSE up front** with "own-only mode needs exact cosigner cards; use `<opt-in flag>`…" — this is the commitment the review demanded (own-only REFUSES extra cosigners, else the prune silently mis-fires). ✓
3. **Own key supplied ALSO as a cosigner card** → refuse via distinct-keys floor (`key65` byte-dup caught before search, regardless of origin/account). ✓
4. **Owns >`j` slots via MULTIPLE independent seeds** → out of own-only scope; own-only = ONE own seed (multi-account); multi-SEED operator uses explicit `--account`/`@N=` or the opt-in tier. Scope stated. ✓

The opt-in tier is named as the ONLY uncertain-cosigner mode (D1 `:21`, the §3a refuse-message, and the §3a row-4 scope line). Every violation resolves to refuse/NO-MATCH — none fails to silent-wrong. I-3 is **CLOSED**.

---

## Fold-drift check

**No new drift.** The fold (`02626a69 → e3d7b4eb`, +37/−17 on one file) is purely additive to the design content:
- **Two new open points present, no dupes.** §5 has exactly ten numbered points (1–10), all distinct. The two prescribed additions are there: #5 (over-supply × `sortedmulti` sorted-shape collapse, axis-7c) and #7 (`--own-account-max` × explicit `--cosigner @N=`, axis-7d). The original eight were renumbered/absorbed cleanly; round-1's worry that the fold "risked duplicating open points" did NOT materialize — I read all ten and there is no repeat.
- **§4 manual `--own-account-max` row-edit caveat present** (`brainstorm:69`): "Manual EDIT still required even where schema is untouched … the `--own-account-max` manual row must change from its v0.60.0 '(reserved/refused — deferred)' wording to its working subset-search description." Matches the axis-6 caveat.
- **Round-1 SOUND items did NOT regress.** Bounding/overflow-refuse intact (Floors `:54` cites `perm_count_u128`/`factorial` refuse-on-overflow, `restore.rs:1882`/`permutation_search.rs:481`, "never panic — #28 M1, confirmed live" — I re-verified the `None`→`bad` path). SemVer MINOR + md/mk NO-BUMP unchanged (`:68`). Shared-core blast radius (both restore + verify-bundle via `complete_multisig_template`) preserved (`:44`, `:10`). `S_own = N!` backward-compat preserved (`:29`). Lockstep scoping (`--own-account-max` name unchanged → schema_mirror needs no change; only the NEW opt-in flag mirrored) preserved (`:69`).

---

## Adversarial sweep (remaining funds-safety gaps the fold may have exposed or left)

- **Opt-in tier count left as an open point — ACCEPTABLE.** The fold derives `S_own` for own-only and leaves the opt-in (unowned/cosigner-search) count to the SPEC: D4 `:39` says "opt-in ⇒ the bounded count for that mode (I-2/D6)"; open-point 3 `:74` defers the exact bound form; D6 `:48` defers `j`-inference vs declaration. This is NOT asserted vaguely-as-derived — it is explicitly flagged as the bounded mode that grows toward `P(pool,N)` and is gated default-off + count-capped + ceiling-refused (D1 `:21`, open-point 6 `:77`). A brainstorm legitimately defers an opt-in-mode count to the SPEC provided the SAFE bounding mechanism (explicit enable + count cap + hard ceiling + time-cap) is committed, which it is. No funds-safety gap: the prefix-strength sizing for opt-in will be sized to *its* realized count by the same `validate_prefix_strength` machinery, and the enumerated≡counted floor (`:52`) is stated generally ("the constrained enumeration"), not own-only-specifically.
- **Brute-force-reference test concrete enough as a SPEC anchor — YES.** It is specified twice with a falsifiable shape: "engine's enumerated set ≡ the independently-generated valid-assignment set" + "`realized_s == enumerated count` assertion" over "exhaustive small-N" (`:33`, `:52`, open-point 1 `:72`). The valid set is precisely defined ("every assignment using exactly `j` own + all `M` cosigners, each once, NOTHING else"), which is directly machine-enumerable for small N as the SPEC's reference oracle. That is a concrete anchor, not a hand-wave.
- **Sharding × non-`n!` rank space — COMPOSES CLEANLY (verified against `search` body).** I read `permutation_search.rs:551-655`. The parallel `search` shards a *contiguous* index space `[0, total)` with `total = perms × outer`, splitting via `div_ceil`, then per-index decodes `outer_k = idx / perms`, `perm_rank = idx % perms`, and unranks `unrank_permutation(perm_rank, n)`. The sharding logic is **agnostic to the internal structure of the rank space** — it requires only (i) the cardinality `perms` and (ii) a bijection from `[0, perms)` onto the enumerated set. The own-anchored generalization replaces `perms = n!` with `perms = S_own = C(K_own,j)·N!` and `unrank_permutation` with `unrank_kperm` (bijective onto the `S_own` set, the I-1 floor). `total = S_own × outer`, the `div_ceil`/`idx%perms`/`idx/perms` shard arithmetic carries over verbatim, the ≥2-match Ambiguous/stop logic is structure-independent, and `search_reference` (`:655`, the single-threaded oracle the parallel path must agree with) provides the determinism check. The non-`n!` rank space shards correctly — this is in fact the cleanest possible extension point, because the sharding never assumes factorial structure. (One SPEC-level note, NOT a finding: `factorial(n)` overflow-refuse at `:564` must be replaced by an `S_own` overflow-refuse — but the brainstorm already commits the up-front `S_own` ceiling at open-point 6 and the `perm_count_u128`-style overflow→refuse backstop, so the safety property is preserved by design intent. SPEC detail, correctly deferred.)
- **No premise-violation fails to silent-wrong.** Re-checked all four §3a rows independently — each terminates in refuse or NO-MATCH; the dangerous "silently complete the wrong wallet" outcome is structurally unreachable because (i) the own-anchored search only places exactly-`j`-own + all-`M`-cosigners, so a wrong premise makes the TRUE assignment unreachable (NO-MATCH, not a wrong match), and (ii) the address/id match predicate is collision-free / full-scan-certified over the enumerated set. No fail-unsafe path.

---

## Sub-Minor (NON-blocking; do NOT re-loop — fold at SPEC-time or ignore)

- **m-1 — D4 reference parenthetical numeric label.** `brainstorm:39` writes "the looser `P(7+25)` the pinned ladder would demand." With `K_own=32` total own candidates and `own=j=4`, the pinned-test convention is `pool = (N−own)+K_extra = 7 + (32−4) = 7+28 = 35`, i.e. `P(35,11)`, not `P(7+25,11)=P(32,11)`. Both `P(32,11)` (11 bytes) and `P(35,11)` (11 bytes) give the same conclusion ("larger ⇒ over-sizes vs `S_own`'s 10 bytes"), so the comparison's POINT is correct and no safety claim depends on the exact pool number. This is a "for reference" aside mixing the `K_own`-total and `K_extra` conventions in one expression. Tidy the parenthetical to `P(35,11)` (or drop the literal) when the SPEC re-derives the ladder. **Not an Important** — it does not touch the load-bearing `S_own` sizing (which is exact at 10 bytes) and the qualitative claim holds under either reading.

---

## Closing verdict

**GREEN — 0 Critical, 0 Important.** I-1, I-2, and I-3 are all CLOSED with on-the-page derivations, exact arithmetic (re-verified two ways for the count, exactly for the margin), and a fail-SAFE premise-violation table. The fold introduced no drift, no duplicate open points, and no contradiction; the round-1 SOUND items are intact; the live citations are accurate at HEAD `e3d7b4eb`; and the non-`n!` rank space shards cleanly through the existing engine. The opt-in-tier count is appropriately deferred-with-bounds (not asserted), the brute-force-reference test is a concrete SPEC anchor, and the single sub-Minor is a reference-parenthetical label that affects no safety conclusion.

**The design may advance to the SPEC.** The SPEC author's load-bearing carries are: (1) the `unrank_kperm` bijection onto `S_own` + its brute-force-reference oracle (open-point 1); (2) replacing the engine's `factorial(n)` cardinality/overflow-refuse with `S_own`'s; (3) the sorted-shape-collapse × subset-axis composition (open-point 5 — the collapse must restrict ORDER but still enumerate SUBSETS, since today's `assignment == identity` skip would otherwise discard every non-first subset); (4) the opt-in count + bound form. All four are correctly scoped as SPEC-level, not brainstorm-level, gaps.
