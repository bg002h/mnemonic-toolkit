> Reviewer: opus architect (R0 round 1) · 2026-06-20 · brainstorm `design/BRAINSTORM_own_account_subset_search_2026-06-20.md` @ `02626a69` (branch `feature/own-account-subset-search`) · source-verified against master `4d5872ed` (v0.60.0).

**Verdict: RED — 0 Critical, 3 Important.**

The agreed two-tier user model (own-only-by-default / opt-in-bounded-unowned) is sound and safe-by-default, every code citation in §0 is accurate, and the bounding / SemVer / lockstep framing is correct. But the brainstorm's **central funds-safety combinatorics (D2's "own-anchored constrained" enumeration + its `realized_s`) directly contradict four canonical pinned sources** without reconciling them, and the *count itself is never derived* — which is exactly the load-bearing math an R0 must nail before SPEC. Two more funds-safety design questions (over-supply collision-freeness for the NEW subset axis; own-anchor mis-count failure modes) are under-specified. Each is closeable in a single fold.

---

## Citation audit (§0) — ALL CONFIRMED against `4d5872ed`

Every cited line is live and accurate:
- `restore.rs:1416` `pub(crate) fn complete_multisig_template` ✓ — the shared core, confirmed called by `run_multisig_template_completion` (`:1321`→`:1379`) AND `verify_bundle.rs::verify_multisig_template` (`:808`→`:874`) with the same `MultisigCompletionCtx`. Blast radius is exactly as stated: both surfaces gain the feature in lockstep.
- `restore.rs:1434` `--own-account-max` refuse gate ✓ (the P3a I-1 gate; comment `:1426-1432` confirms the "only n! placements of the FIRST n pool entries" mechanism).
- `restore.rs:1626` (`pool.len() < n`) / `:1635` (`pool.len() > n`) ✓.
- `restore.rs:1661` `realized_s = perm_count_u128(n, n)` ✓.
- `restore.rs:1882` `perm_count_u128(pool, n)` ✓ — and **confirms the #28 M1 lesson is already honored**: it returns `None` on `pool < n` (`:1883`) and on `checked_mul` overflow (`:1888`), and the call site `:1662` maps `None`→`bad("…candidate space overflow")` → REFUSE not panic. The brainstorm's "P(pool,N) u128 overflow must REFUSE" requirement is already met by the existing helper.
- `permutation_search.rs`: `search` (`:551`), `unrank_permutation` (`:494`), `factorial` (`:481`), `total_candidates` (`:509`), `required_prefix_bytes` (`:322`), `validate_prefix_strength` (`:342`), `SearchMode` (`:247`) ✓.

**Drift-correction (§0) is ACCURATE.** The gate did move from `run_multisig_template_completion` into the shared `complete_multisig_template` in P4 — confirmed by the comment at `restore.rs:1330-1332` ("now lives in the SHARED … core") and the gate's live location at `:1434` inside `complete_multisig_template`.

**Over-supply-unreachable mechanism — CONFIRMED.** `search(n, …)` (`:551`) enumerates `factorial(n)` permutations (`:564`); each shard calls `unrank_permutation(perm_rank, n)` (`:612`, `:643`, `:679`) which builds `elems = (0..n)` (`:496`) and permutes ONLY those. The evaluator does `pool[pi]` for `pi ∈ assignment`, so pool indices `≥ n` are never evaluated. There is **no existing `unrank_kperm` / pool-aware enumeration anywhere in the engine** (grep-confirmed: `kperm`/`k_perm`/`pool` absent from `permutation_search.rs` outside the test helper at `:723`). D2's `unrank_permutation(rank,n) → unrank_kperm(rank,pool,n)` is a genuine new build, correctly identified as such.

---

## I-1 (Important) — D2's "own-anchored constrained `realized_s`" CONTRADICTS four pinned canonical sources, and the constrained count is never derived

This is the funds-safety core and the reason for RED.

**The conflict.** D2 (`brainstorm:24`) and D4 (`brainstorm:30`) assert that own-only mode enumerates a space *constrained* to "exactly `j = N−M` own + all `M` cosigner" — explicitly "**not** the unconstrained `P(K+M, N)`" — and that `realized_s` becomes "the own-anchored count." But FOUR canonical sources all pin `realized_s = P(pool, N)` (the FULL k-permutation of the whole pool), NOT a smaller own-anchored count:

1. **The I-1 exec-review's "To turn GREEN" option 1** (`design/agent-reports/template-multisig-p3a-completion-exec-review.md:64`): "enumerate `P(pool, n)` injective placements … so the over-supplied own candidates AND the cosigners are all reachable and `realized_s = P(pool,n)` is the truly-enumerated space."
2. **The FOLLOWUP** (`design/FOLLOWUPS.md:48`): "enumerates the `P(pool, n)` injective k-permutations … size `realized_s = P(pool, n)` to the truly-enumerated space."
3. **The pinned engine test** `prefix_ladder_own_account_max_subset_space` (`permutation_search.rs:739-755`): computes `S = perm_count((n−own)+K, n) = P(7+K, 11)` and asserts the byte-ladder against THAT. This is live, passing, pinned.
4. **The #28 SPEC §6.1** (`design/SPEC_bundle_md1_template_multisig_2026-06-20.md:110`): "S = realized candidate count — `N!` for an explicit `--account` LIST, the larger `P((N−own)+K, N)` subset×permutation space for `--own-account-max K`."

The brainstorm cites none of these for its narrower count and does not acknowledge the conflict. This matters because **`realized_s` is the funds-safety sizing input** (D4): it sizes the `--expect-wallet-id` prefix strength (`validate_prefix_strength`, `restore.rs:1700`) and the cap ETA. The P3a I-1 lesson (`exec-review.md:45`) is precisely that `realized_s ≠ enumerated-space` is a SPEC §6.2/§7-floor-5 fidelity break.

**Which is right? The brainstorm's own-anchored count is a real refinement — but it is UNDER-derived and the conflict must be resolved deliberately, not silently.** The two are genuinely different:

- **Full `P(pool, N)`** counts every injective placement of `N` of the `pool = K + M` keys into the `N` ordered slots, INCLUDING placements that drop one of the `M` cosigners (using `< M` cosigners + `> j` own). Those placements are *funds-safety-inert* — they recompute to wallets the operator's stated premise ("I supplied all M cosigners") says cannot be the target — but they ARE scanned, so they ARE part of `realized_s` and they CAN spuriously collide on a short id-prefix.
- **Own-anchored** drops those placements up front. Correct derivation of the own-anchored count (let me do it, since the brainstorm doesn't):

  > Own-only premise: exactly `j = N − M` of the `K` own candidates are used, and ALL `M` cosigners are used. Enumeration = (a) choose which `j` own candidates: `C(K, j)`; (b) the chosen `j` own + the `M` cosigners = exactly `N` keys; assign these `N` keys to the `N` ordered slots: `N!`. The "which slots are own vs cosigner" the brainstorm mentions is NOT a separate factor — it is subsumed by the `N!` ordered assignment.
  >
  > **`S_own-anchored = C(K, j) · N!`** where `j = N − M`.
  >
  > Sanity at `K = j` (no over-supply): `C(j, j)·N! = 1·N! = N!` ✓ — matches D2's backward-compat claim and v0.60.0 exactly.
  > Worked example (the pinned `N=11, own slots j=4, M=7`, so a `pool` of `7 + K` where `K` here is the own-candidate COUNT, `K = j + extra`): at K=8 own candidates → `C(8,4)·11! = 70·39,916,800 ≈ 2.79e9`, vs full `P((11−4)+8, 11)`… **note the pinned test's variable naming differs** — its `K` is the EXTRA accounts beyond `j`, pool `= (n−own)+K = 7+K`. Disambiguating `K`'s meaning (extra-accounts vs total-own-candidates) is itself a SPEC must-fix; the brainstorm uses `K` = own-candidate range size (`accounts 0..K-1`), the test uses `K` = extra. **Whatever the convention, the two counts are not equal**, so the choice changes the prefix-strength floor.

  `S_own-anchored = C(K,j)·N!` is strictly `≤ P(pool,N)` (it forbids cosigner-dropping placements), so an own-anchored `realized_s` yields a SHORTER required prefix than the pinned ladder demands. **That is the safe direction ONLY IF the own-anchored enumeration genuinely never scans a cosigner-dropping placement** — otherwise a scanned-but-uncounted placement is a collision the prefix wasn't sized for (the I-1 hole, in the dangerous direction). So the SPEC MUST commit that the ENUMERATED set equals exactly the `C(K,j)·N!` counted set — every assignment using exactly `j` own + all `M` cosigners, each exactly once, and NOTHING else.

**The fix (single fold).** The brainstorm must, BEFORE SPEC:
1. State explicitly that it is REFINING the canonical `realized_s = P(pool, N)` down to the own-anchored `S_own = C(K, j)·N!` (with `j = N − M`), and that this supersedes the pinned `prefix_ladder_own_account_max_subset_space` test + the #28-SPEC §6.1 / FOLLOWUP / I-1-review sizing — call out that the existing pinned test must be UPDATED (its `P(7+K,11)` becomes the own-anchored count) and the #28 SPEC §6.1 + FOLLOWUP §"Fix" lines re-cited.
2. Carry the derivation `S_own = C(K, j)·N!` verbatim (and the opt-in mode's count — see I-2).
3. Commit the enumerate=count invariant (Floor: enumerated set ≡ counted set, brute-force-reference-tested per `brainstorm:43`) AND state that `realized_s` passed to `validate_prefix_strength`/`cap_decision` is exactly `S_own`, computable up-front (both `C(K,j)` and `N!` are closed-form → yes, computable before enumeration, satisfying D4's up-front-cap requirement).
4. Disambiguate the `K` convention (own-candidate-count vs extra-accounts) so the count is unambiguous.

Until the load-bearing count is on the page and the conflict with the four pinned sources is resolved, this is the I-1 class the FOLLOWUP itself warns about — vague combinatorics is a finding.

---

## I-2 (Important) — Over-supply introduces a NEW collision axis (distinct injective SUBSETS, not just distinct orderings); D3's address-search "collision-free" + id-search uniqueness claims are inherited from #28 but #28's proof does NOT cover it

D3 (`brainstorm:27`) reuses #28's two guarantees: (a) address-search is collision-free because "full scriptPubKey ⇒ a match is provably unique," and (b) id-search certifies uniqueness by full-scan. The #28 proof for (a) is at SPEC §6.1 `:111`: "for non-sorted shapes children serialize in stored slot order … → distinct permutations → distinct 256-bit P2WSH program → cryptographically unique match." **But that proof is about distinct ORDERINGS of a FIXED key set (the `N!` axis). Over-supply adds a SECOND axis the #28 proof never considered: two DISTINCT injective SUBSETS (different key sets) mapped to the slots.**

The question the brainstorm must answer: can two *different* key-subsets-with-ordering produce the SAME scriptPubKey (address-search) or SAME wallet-policy-id (id-search)?
- **Address-search:** the scriptPubKey is a hash of the actual N pubkeys in slot order. Two assignments with different key SETS → different pubkey multisets → different script → different hash. Collision requires a hash collision (cryptographically negligible) OR two of the supplied candidate keys being byte-identical — which the **distinct-keys floor** (`reject_duplicate_keys` on the WHOLE pool, `restore.rs:1648`, confirmed runs on `pool` = own + cosigners) already forbids. So (a) DOES extend — but only BECAUSE the distinct-keys floor runs on the full over-supplied pool. The brainstorm must STATE this dependency (the floor is what makes the new subset axis collision-free), not silently inherit it.
- **id-search:** `compute_wallet_policy_id` (`identity.rs:172`) hashes tree + per-@N origin-path + use-site + presence — confirmed it includes the per-slot origin/key data. Distinct subsets → distinct per-slot data → distinct id (modulo the prefix). The full-scan ambiguity certification (`search:638-647`, `≥2 matches → Ambiguous`) still holds over the LARGER own-anchored space — the engine counts matches across the whole enumerated set regardless of how it's enumerated. This extends correctly, BUT only if the enumerated space is the own-anchored one (I-1): if the enumerator scans cosigner-dropping placements that the count omits, a 2nd match there is real but the prefix wasn't sized for it.

**Distinct-keys floor edge the brainstorm must address (ties to own-anchor I-3):** what if an own candidate (own seed at account `a`) derives a key BYTE-IDENTICAL to a supplied cosigner card? Today with `pool.len()==n` that's a flat duplicate-reject. Under over-supply, the own pool spans `K` accounts — the probability one own-derived key equals a cosigner key is still negligible (different seeds), but if the operator supplied their OWN key as a "cosigner" card too (I-3 failure mode), the floor will reject — which is the SAFE outcome, but the brainstorm should confirm the floor fires BEFORE the search (it does, `:1646-1648`, before `realized_s` even — good) and say so.

**The fix (single fold).** Add to D3/Floors: (1) address-search collision-freeness over the over-supplied pool RESTS on the whole-pool distinct-keys floor — state the dependency; (2) the re-derived `ceil((log2(S_own)+32)/8)` margin at the worst K (do the arithmetic for the chosen `K` ceiling, e.g. K=32/64 per the pinned ladder's spirit, against `S_own` not `P(pool,N)`); (3) confirm id-search full-scan ambiguity certification is over the OWN-ANCHORED enumerated set, the same set `realized_s` counts (closes the I-1↔I-2 coupling).

---

## I-3 (Important) — The own-anchor prune's premises ("exactly `j=N−M` own, ALL `M` cosigners supplied") can be violated by a mis-counting operator; the brainstorm must enumerate the failure modes and prove fail-SAFE

D2's prune is only sound if the operator's two implicit assertions hold: (i) they own EXACTLY `j = N − M` of the N slots, and (ii) they supplied ALL `M` cosigners (no more, no fewer). The brainstorm asserts `j = N − M` is "DETERMINED" (`brainstorm:20`, `:38`) but never asks what happens when the premises are FALSE. Funds-safety requires every violation to fail SAFE (refuse / NO-MATCH), never silent-wrong. Enumerate:

1. **Operator under-supplies cosigners** (wallet has `M` cosigners, operator supplies `M' < M` thinking that's all): then `j_assumed = N − M' > j_true`. The own-anchored search forces `j_assumed` own slots + `M'` cosigners — it will NEVER try the real assignment (which needs `M` cosigner keys it doesn't have) → NO-MATCH → refuse. **Safe**, but the brainstorm should confirm the message points at "did you supply all cosigners?".
2. **Operator over-supplies cosigners in own-only (default) mode** (supplies `M' > M` cosigner cards): own-only mode requires cosigners EXACT, so `pool` has `M'` cosigner candidates. Either (a) the design still forces "all cosigner candidates used" → `j_assumed = N − M' < j_true`, real assignment unreachable → NO-MATCH (safe but confusing), or (b) the design must REFUSE up front ("own-only mode requires exact cosigners; use the opt-in flag for uncertain cosigners"). The brainstorm leaves this to the opt-in tier but doesn't say own-only REFUSES extra cosigners — it must (otherwise the prune silently mis-fires).
3. **Operator supplies their OWN key as a cosigner card** (double-counts a slot they own): the whole-pool distinct-keys floor (`:1648`) catches an exact byte-dup → refuse. **Safe**, but only if the own-derived key and the cosigner card are byte-identical; if the operator owns a slot via a DIFFERENT origin/account than the cosigner card encodes for the same pubkey — confirm the floor compares the 65-byte key (`pool_key_blobs`, `:1647`, confirmed `key65`) → same pubkey → caught regardless of origin. **Safe.** State it.
4. **Operator owns MORE than `j = N−M` slots** (e.g. a 2-of-3 where they hold 2 of the 3 keys): then `M_true < N − j_own_true`; supplying "all cosigners" means `M' = M_true` but they own 2 slots, so `j_true = 2 = N − M_true` — actually consistent IF they declare both own accounts. The risk is they supply own via `--own-account-max K` (one seed, K accounts) AND separately hold a second independent key — the own-only model assumes ONE own seed. The brainstorm must state own-only mode = single own seed (multiple ACCOUNTS of it), and a multi-SEED operator falls to explicit `--account`/`@N=` or the opt-in tier.

**The fix (single fold).** Add a "§ own-anchor premise-violation failure modes" table to D2/Floors: each of the above → the SAFE outcome (refuse/NO-MATCH) + the actionable message, and the explicit commitment that own-only mode (a) REFUSES extra cosigners (premise ii), (b) assumes a single own seed (multi-account), (c) relies on the whole-pool distinct-keys floor for the own-as-cosigner case. The boundary to the opt-in tier (which relaxes premise i/ii under an explicit bound) must be stated as: opt-in is the ONLY mode where cosigner count is uncertain.

---

## Findings by your scrutiny axes

**Combinatorics-correctness (axis 1):** the own-anchored count is the right *idea* but UN-derived and in conflict with four pinned sources — **I-1**. Corrected derivation `S_own = C(K, N−M)·N!`, collapsing to `N!` at no-over-supply, supplied above. The brute-force-reference-test floor (`brainstorm:43`) is the right verification discipline — keep it, and pin it against `S_own`.

**realized_s↔enumerated-space fidelity (axis 2):** the brainstorm commits to `realized_s = the enumerated count` (D4) — GOOD and exactly the I-1 lesson — but because the *count* is undefined (I-1), the commitment is currently unverifiable. Computable up-front: YES (`C(K,j)·N!` is closed-form). Resolve I-1 and this axis closes with it.

**Ambiguity/collision (axis 3):** id-search full-scan certification extends correctly; address-search collision-freeness extends correctly BUT rests on the whole-pool distinct-keys floor and a NEW subset axis the #28 proof didn't cover — **I-2**. Re-derive the margin against `S_own`.

**Own-anchor-safety (axis 4):** premise-violation failure modes unenumerated — **I-3**. All appear to fail safe via NO-MATCH + the distinct-keys floor, but the brainstorm must SAY so and commit own-only to refuse-extra-cosigners + single-own-seed.

**Bounding (axis 5):** GOOD. `perm_count_u128` already REFUSES on overflow (`:1882`, `None`→`bad`), satisfying the #28 M1 lesson — confirmed live, no panic. The brainstorm commits to an up-front hard `K`/pool ceiling (D1 `:21`, open-point 6 `:59`) DISTINCT from the time-cap — correct and necessary. Minor: open-point 6 says "refuse absurd K/pool before even calibrating the cap" — make the SPEC pin a concrete ceiling NUMBER (e.g. cap `S_own` at a value whose worst-case ETA is bounded), not just "absurd."

**Backward-compat / SemVer (axis 6):** SOUND. `S_own = N!` at `K=j` ⇒ the non-over-supplied path (`pool.len()==n`) is byte-identical to v0.60.0 — confirmed against `realized_s = perm_count_u128(n,n)` (`:1661`) and the `multi_account_own_resolves_both_slots` pin. Shared-core blast radius correctly scoped (both restore + verify-bundle via `complete_multisig_template`, confirmed). MINOR + lockstep claims correct: `--own-account-max` name unchanged (refuse→search is behavior-only, schema_mirror gates flag-NAMES not behavior per CLAUDE.md) → only the NEW opt-in flag needs schema_mirror + manual. One caveat to add to §4: the manual's `--own-account-max` row must change from "(refuses — deferred)" to its working description — a manual EDIT even though schema is untouched.

**Completeness (axis 7):** §5's eight open points are the right ones AND open-point 1 (the load-bearing combinatorics) is correctly flagged as load-bearing — but flagging it open is NOT sufficient for R0: the COUNT must be on the page now (I-1), because a SPEC built on a vague count inherits the vagueness. Missing funds-safety-relevant questions to add: (a) the own-as-cosigner distinct-keys interaction (folded into I-3); (b) own-only-refuses-extra-cosigners (I-3); (c) the over-supply × `sortedmulti` collapse — open-point list omits it: under over-supply, does the sorted-shape collapse (`is_order_independent_shape`, `restore.rs:1676`, `:1739`, which today restricts address-search to the identity placement) still correctly collapse when the SUBSET also varies? For sorted shapes, different subsets still produce different addresses (different key sets), so the collapse must restrict ORDER but still enumerate SUBSETS — confirm the sorted carve-out composes with subset-search (likely a real interaction: today's collapse assumes a fixed key set permuted; over-supply breaks that assumption). Add as an open point. (d) explicit `--cosigner @N=` under over-supply (open-point present implicitly via D5 but not its interaction with `--own-account-max` — can you pin some slots and search others? state in/out of scope).

---

## MINOR (non-blocking — fold opportunistically)

- **M-1 — `K` naming collision.** The brainstorm's `K` = own-account RANGE size (`accounts 0..K-1`, `brainstorm:20,33`); the pinned test's `K` = EXTRA accounts beyond `j` (`pool = 7+K`, `permutation_search.rs:741`); the #28 SPEC's `K` = same as the test. Pick ONE and state the relation to `pool`/`j` explicitly (folded into I-1 fix 4 but worth a standalone glossary line).
- **M-2 — open-point 5 (early-exit) is sound but under-stated.** Address-search early-exit-on-first-match is safe ONLY because address-search is collision-free (I-2); the brainstorm says so (D3) but should cross-link the early-exit permission to the I-2 collision-freeness proof so a SPEC author doesn't grant early-exit to id-search by analogy.
- **M-3 — `cosigner_family` first-pick heterogeneity** (P3a exec-review M-2, `:55`): over-supply doesn't worsen it, but the no-match message under a larger search will be more confusing for a heterogeneous-family wallet. Note `--origin` as the escape hatch in the SPEC's no-match UX.
- **M-4 — verify-bundle UX (open-point 7).** Confirm verify-bundle exposes the opt-in identically (it shares the core, so it gets it for free) — but verify-bundle's semantics are "does this bundle reproduce?", and an over-supplied search there is unusual (you already have the bundle). State whether the opt-in is meaningful at verify-bundle or should be restore-only at the CLI layer even though the core supports it.

---

## To turn GREEN (single fold closes each)

1. **I-1:** put the own-anchored count `S_own = C(K, N−M)·N!` (collapsing to `N!` at no-over-supply) ON THE PAGE; state it REFINES the canonical `realized_s = P(pool,N)` and explicitly supersedes the pinned `prefix_ladder_own_account_max_subset_space` test + #28-SPEC §6.1 + FOLLOWUP "Fix" + I-1-review option-1 (cite each); commit `realized_s = S_own` (up-front computable) + the enumerated≡counted brute-force floor; disambiguate `K`.
2. **I-2:** state address-search collision-freeness over the over-supplied pool depends on the whole-pool distinct-keys floor; re-derive the prefix margin against `S_own` at the K-ceiling; confirm id-search full-scan ambiguity is over the own-anchored enumerated set.
3. **I-3:** add the own-anchor premise-violation failure-mode table (under-/over-supply cosigners, own-as-cosigner, multi-own-seed) each → SAFE outcome + message; commit own-only = refuse-extra-cosigners + single-own-seed; opt-in = the only uncertain-cosigner mode.
4. Add the two missing open points (over-supply × sorted-shape collapse; `--own-account-max` × explicit `@N=` interaction) and the manual `--own-account-max` row edit to §4.

Re-dispatch this R0 after the fold. The model and citations are solid; only the load-bearing count + its two safety corollaries stand between this brainstorm and GREEN.
