# BRAINSTORM — `restore --md1` multisig-template own-account subset-search (+ opt-in bounded unowned search)

**Date:** 2026-06-20 · **FOLLOWUP:** `template-multisig-own-account-range-subset-search` (`design/FOLLOWUPS.md`). **Base SHA (grep-verify at SPEC):** mnemonic-toolkit `4d5872ed` (master, v0.60.0). **Predecessor:** #28 phase 2 (multisig/general template completion, v0.60.0) — this lifts the P3a-deferred over-supply gate.

## 0. Citation-drift correction (vs the FOLLOWUP, which was filed pre-P4)
The FOLLOWUP cites the `--own-account-max` gate + every-slot gate in `restore.rs::run_multisig_template_completion`. **P4 (the verify-bundle shared-core refactor) MOVED them into the shared `complete_multisig_template` core.** Live (v0.60.0 `4d5872ed`):
- `--own-account-max` refuse gate: `cmd/restore.rs:1434` (`if ctx.own_account_max.is_some()`).
- under/over-supply gates: `:1626` (`pool.len() < n`), `:1635` (`pool.len() > n`).
- `realized_s`: `:1661` (`perm_count_u128(n, n)` = `n!`).
- the completion core entry: `pub(crate) fn complete_multisig_template` (`:1416`), shared by restore (`run_multisig_template_completion:1321`) AND verify-bundle (`verify_bundle.rs::verify_multisig_template`) — **so BOTH surfaces gain this feature in lockstep.**
- engine: `permutation_search.rs::search` (`:551`), private `unrank_permutation` (`:494`), `factorial` (`:481`), `total_candidates` (`:509`), `required_prefix_bytes` (`:322`), `validate_prefix_strength` (`:342`), `SearchMode` (`:247`). `perm_count_u128(pool,n)` (the `P(pool,n)` math) already exists at `restore.rs:1882`.

## 1. Problem
v0.60.0 requires the supplied keys to EXACTLY fill the N slots (`pool.len() == n`); the engine enumerates `n!` orderings via `unrank_permutation` over the first `n` pool entries. `--own-account-max K` (the operator over-supplying own-account candidates because they don't recall which account index the wallet uses) is wired but **refuses** — the engine can't select a *subset* of an over-sized pool, and any pool index `≥ n` is never evaluated (the P3a I-1 finding). This makes a documented recovery feature non-functional for the very-common "which account was it?" case.

## 2. Decisions (the agreed model — to be R0'd)

### D1 — Own-only subset-search by DEFAULT; opt-in + BOUNDED unowned (cosigner) search.
Two tiers, safe-by-default:
- **Default (own-only):** only the OWN seed may be over-supplied. The operator supplies own-account candidates via the existing `--account <list>` OR re-enabled `--own-account-max K` (own seed derived at accounts `0..K-1` → K own candidates), and **ALL `M` cosigner mk1 cards EXACTLY** (no extra). The own-slot-count `j = N − M` is then DETERMINED. The search selects which `j` of the `K` own candidates fill the `j` own slots, and assigns all `N` keys to the `N` slots (positions of own-vs-cosigner slots are themselves unknown). Bounded: `K` is a small account range; `M` exact ⇒ the space stays modest.
- **Opt-in (unowned/cosigner search):** a new flag ENABLES over-supplying/uncertain cosigner candidates too ("I'm not sure which of these mk1 cards belong / how many cosigners there are"). This makes `j` uncertain and grows the space toward `P(pool, N)`. It MUST be explicitly enabled AND BOUNDED (a count cap on extra unowned candidates, on top of the adaptive time-cap), so it can never silently blow up. Default-off.

### D2 — Enumeration = subset-select k-permutation, OWN-ANCHORED.
The engine gains a subset-select enumeration (generalize `unrank_permutation(rank, n)` → `unrank_kperm(rank, pool, n)`: the injective placement of `n` of `pool` keys into `n` ordered slots; count `P(pool, n) = pool!/(pool−n)!`). **Own-anchored prune (the key bound):** `--from` ⇒ the operator owns `≥1` slot; in own-only mode `exactly j = N−M` own candidates are used and all `M` cosigners are used — so the enumeration is CONSTRAINED to assignments using exactly `j` own + all `M` cosigner (not the unconstrained `P(K+M, N)`). This both shrinks the space and keeps the cap from biting in the common case. (For `K=j`, no over-supply, the count collapses to `N!` — exact backward-compat with v0.60.0.)

### D3 — Ambiguity full-scan PRESERVED for prefix id-search; safe early-exit for address-search.
The funds-safety gate is unchanged: **prefix id-search MUST scan the full realized space to certify uniqueness** (≥2 matching assignments → `Ambiguous` → refuse, never silently pick one). Ordering ("own first") does NOT shortcut this — it's the time-cap that bounds prefix id-search. **Address-search is collision-free** (full scriptPubKey ⇒ a match is provably unique), so own-anchored ordering + early-exit on first match is SAFE there and makes the common case fast. (Optional: a *full* — non-prefix — wallet-id is also collision-safe for early-exit; SPEC to decide.)

### D4 — Prefix-strength sizing to the REALIZED space; address-search recommended for large pools.
`realized_s` becomes the ACTUAL enumerated count (own-only: the own-anchored count; opt-in: the bounded `P(pool,N)`) — NOT `n!`. `validate_prefix_strength` then sizes the required `--expect-wallet-id` prefix to this larger space (bigger pool ⇒ longer prefix needed to keep collision-prob `≤ ~2e-10`, per the #28 `ceil((log2(S)+32)/8)` ladder). A short prefix over a large pool REFUSES (point the operator at a longer id or address-search). Address-search stays collision-free regardless of pool size ⇒ the recommended mode for large pools.

### D5 — Flags.
- **Re-enable `--own-account-max K`** (remove the `:1434` refuse): own seed at accounts `0..K-1` → K own candidates (own-only mode). Coexists with `--account <list>` (explicit own accounts); SPEC to define interaction (likely: `--account` = explicit list, `--own-account-max` = range `0..K`; mutually exclusive OR union — R0 to decide).
- **NEW opt-in flag for unowned search** (name TBD — e.g. `--search-cosigner-subset` / `--allow-extra-cosigners`) + its **bound** (e.g. `--max-extra-cosigners K2`, or the flag takes the bound as its value). Default-off. SPEC/R0 to fix the exact name + bound form.
- All on BOTH `restore` and `verify-bundle` (shared `complete_multisig_template`).

### D6 — Unknown own-slot-count handling.
- Own-only mode: `j = N − M` is DETERMINED (cosigners exact) → no declaration needed.
- Opt-in mode: `j` uncertain (extra cosigners) → the search tries the valid `j` range, bounded; OR the operator may pin `j`/own-count. SPEC to decide the inference vs declaration (the prune is tighter if `j` is pinned).

## 3. Funds-safety floors (carried from #28 + new)
- **Distinct-keys** (`reject_duplicate_keys`) on the full pool — unchanged.
- **Own-anchor correctness:** the constrained enumeration MUST cover EXACTLY the valid assignments (every assignment using `j` own + all `M` cosigners, no more/less) — an enumeration bug that skips a valid assignment ⇒ a legitimate wallet NO-MATCHES (the I-1 class); one that mis-counts ⇒ `realized_s` ≠ enumerated ⇒ prefix-strength under/over-sized. The k-perm unranking + the own-anchor constraint are the R0-load-bearing combinatorics (off-by-one = funds-safety bug) → exhaustive small-N enumeration tests (compare the engine's enumerated set against a brute-force reference).
- **Ambiguity** (≥2) + **no-match** → refuse (D3).
- **Adaptive time-cap** (the ~1hr ceiling + `--accept-search-time`) — the universal blow-up bound; the bounded opt-in count-cap is an ADDITIONAL up-front bound.
- **Prefix-strength sized to `realized_s`** (D4).
- **Per-slot origins BUILT fresh from supplied keys; carried origin never loaded (C1)** — unchanged from #28.

## 4. SemVer + locksteps
- **toolkit MINOR** (re-enables `--own-account-max` + a NEW opt-in flag = additive clap surface + behavior). md-codec/mk-codec NO-BUMP (pure toolkit search-engine change).
- **GUI `schema_mirror`** (`mnemonic-gui/src/schema/mnemonic.rs`) + **manual** (`docs/manual/src/40-cli-reference/41-mnemonic.md`) lockstep for the new flag(s) on restore + verify-bundle (paired GUI MINOR). `--own-account-max` is already in the v0.60.0 schema (its behavior changes from refuse→search, but the flag name is unchanged — so schema_mirror is unaffected by the re-enable; only the NEW opt-in flag needs mirroring).

## 5. Open points for the SPEC / R0
1. The exact `unrank_kperm` algorithm + the own-anchored-constraint enumeration (the load-bearing combinatorics) — and its brute-force-reference test design.
2. `--account` (list) vs re-enabled `--own-account-max` (range) interaction (mutually exclusive / union / precedence).
3. The opt-in flag's exact NAME + BOUND form (count of extra cosigner candidates; default-off; refuse if the bound × pool exceeds a hard ceiling even before the time-cap).
4. Opt-in `j`-inference vs declaration (D6).
5. Early-exit policy: address-search (safe) vs full-id (safe?) vs prefix-id (full-scan only) — and the own-first ORDERING within the enumerator (so address-search finds the real wallet early).
6. Worst-case-size guardrails: a hard upper bound on `K` / `pool` (refuse absurd inputs before even calibrating the cap), distinct from the time-cap.
7. Whether `verify-bundle` exposes the same opt-in (it shares the core — likely yes, but confirm the verify UX).
8. Re-grep all citations vs the SPEC base SHA.
