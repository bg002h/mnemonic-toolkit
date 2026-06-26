# Word-Card P5 — per-phase R0 review (round 1)

**Phase:** P5 — cross-plate RAID layer + `has-raid=1` header extension to `wc-codec`.
**Branch:** `feat/wc-p5-raid` @ `58209936` (parent `master@795e33b8`, NOT merged).
**Reviewer:** opus architect, adversarial, independent fuzzing.
**Date:** 2026-06-25.

---

## Verdict: **GREEN (0 Critical / 0 Important)**

P5 is correct, funds-safe, and ready to merge. The load-bearing MDS recovery
property holds independently for the full surfaced range — including the NEW-I2
regression target `n=15, r=2` (every pair) and `n=32, r=2` (every pair) — and the
never-wrong-reconstruction property held across 10000+ adversarial corruption
trials with **zero** silently-wrong outputs. Suite GREEN at `PROPTEST_CASES=4000`
(99 tests, 0 failures); clippy `-D warnings` clean; `fmt --check` clean; diff is
`wc-codec`-only (no `mlock.rs`, no cross-crate, no `fmt --all`).

The 2 flagged items resolve as: **`r≥n` refusal = Minor** (a safe, plan-consistent
limitation, not a defect); **parity-index-0 placeholder = no collision** (grouping
dispatches on role, never on index, for parity plates). Nothing rises to
Important.

---

## Critical

None.

## Important

None.

## Minor / Nit

1. **(Minor — flagged item #7) `r ≥ n` is refused, so `[4,2]` / `n=2,r=2` is
   rejected even though `[n+r,n]` MDS is valid for `r=n`.** `raid_encode` enforces
   `(r as usize) >= n ⇒ InvalidParams` (`raid.rs:229`). This is a *safe*
   limitation — refusing a valid-but-unsurfaced config never risks funds; it only
   narrows the accepted set. It is also plan-consistent: the plan surfaces
   `r ∈ {1,2}` for cosigner arrays where `r < n` is the natural regime (you keep
   strictly fewer recovery plates than data plates), and the task KAT (`raid.rs`
   test `n_and_r_out_of_range_errors_no_panic`, lines 548-554) explicitly asserts
   `n=2,r=2` is rejected as "degenerate." **Recommendation:** leave as-is for P5;
   if a 2-of-2 "either-plate-recovers-the-other" product use ever surfaces,
   relaxing to `r ≤ n` is a localized future change (the solve already handles
   1-unknown via `P₁`; the 2-unknown branch is only reachable when 2 data plates
   are missing, which `r<n=2` already forbids). No action required to ship P5.

2. **(Nit — flagged item #8) Parity wire-`index=0` placeholder: confirmed no
   collision.** Parity plates store `wire_index=0` (`raid.rs:274,281`) and the
   reconstruct grouping (`raid.rs:380-406`) dispatches on `d.role`: only
   `RAID_ROLE_DATA` consults `wire_index` to populate `present_data[idx]`;
   `RAID_ROLE_PARITY_A/B` route to the `p1`/`p2` slots regardless of index. A
   parity plate therefore can never be mistaken for data-index-0. `parse_header`
   reinforces this: it constrains `index < n` *only* for `role==DATA`
   (`pipeline.rs` raid-parse block) and leaves the parity index unconstrained (its
   identity is the role). The public API correctly re-derives the *logical* index
   (`ParityA⇒n`, `ParityB⇒n+1`) in `raid_meta`. No fix needed; the doc comments
   already explain the dual meaning clearly.

3. **(Nit — informational, not a P5 defect) 22-bit array-id is a matcher, not a
   cryptographic separator.** Two distinct seeds that collide on the top-22-bits of
   `SHA-256` AND share `(n, W)` would not be separated by the grouping check
   (probability ≈ 2⁻²² per pair, by design). Even then the present plates' own
   integrity tags remain individually valid (they are genuine cards from the
   colliding array), so only the *reconstructed* missing plate would be wrong — the
   same inherited bound discussed under safety below. This is the documented
   design parameter of a 22-bit plate-matching aid (plan §3/§4.2), not a
   P5-introduced flaw. No action.

---

## Suite results (`PROPTEST_CASES=4000`, clean checkout, harness removed)

```
unittests src/lib.rs ........ 0 passed
tests/field.rs .............. 10 passed
tests/pad.rs ................ 5 passed
tests/pipeline.rs ........... 23 passed   (154.8s)
tests/raid.rs ............... 13 passed   (92.7s)
tests/regroup.rs ............ 8 passed
tests/rs.rs ................. 12 passed
tests/sync.rs ............... 23 passed
tests/wordmap.rs ............ 5 passed
doc-tests ................... 0 passed
TOTAL: 99 passed, 0 failed, 0 ignored. No warnings.
```

- `cargo clippy -p wc-codec --all-targets -- -D warnings` → **clean** (forced
  rebuild, not a cache hit).
- `cargo fmt -p wc-codec --check` → **clean**.
- `git diff --name-only master..feat/wc-p5-raid` → exactly
  `lib.rs`, `pipeline.rs`, `raid.rs`, `tests/raid.rs`. No `mlock.rs`, no
  cross-crate, no `fmt --all` churn.
- Worktree left clean (`git status --porcelain` empty, on `feat/wc-p5-raid`).

---

## Independent MDS + safety fuzz (my own harness, re-derived, then discarded)

Harness used an independent splitmix64 PRNG (distinct from the implementer's
xorshift), re-derived the recovery contract from the public API only, and was
removed before the final clean suite run. All passed.

### 1. Recover-any-≤r (load-bearing MDS)

- **Exhaustive subset sweep** `n∈{2,3,5,8}, r∈{1,2}` (r<n), 3 seeds each: for every
  subset of ≤r plate positions removed, **exact** recovery of all n
  `(payload_bytes, payload_bits)`. Varied payload lengths 60–80 bytes (length-prefix
  + array-wide zero-pad trimming exercised). PASS.
- **NEW-I2 — `n=15, r=2`, every pair** (all C(17,2)=136 pairs, data+parity mixed):
  **exact** recovery. Proves the full 5-bit `index-in-array` α-exponent keeps r=2
  MDS for n>8. PASS. (A single failure here would have been Critical.)
- **`n=15, r=2`, every single plate** removed: exact. PASS.
- **`n=32, r=2`, every pair** (all C(34,2)=561 pairs; top exponents 29/30/31
  exercised — the largest the 5-bit field allows): **exact**. PASS. Confirms MDS at
  the maximum surfaced `n` and the highest α-exponents.

The MDS math checks out on inspection too: `missing` is built `(0..n).filter(...)`
so it is ascending (`j<k`); the 2-unknown determinant `det = α^j + α^k` is nonzero
because `α` has order 2047 ≫ 32 so `α^j ≠ α^k` for distinct `j,k ∈ [0,31]`;
`field::inv(det)` is therefore always `Some`. The 1-unknown branch prefers `P₁`
(XOR) and falls back to `P₂` with `α⁻ʲ`. Field arithmetic (`add`/`mul`/`pow`/`inv`)
is the frozen, KAT-locked GF(2¹¹).

### 2. Never-wrong-RECONSTRUCTION (funds-safety)

- **`indep_never_wrong_under_corruption`** — 6000 trials: random `n∈[2,6]`,
  `r∈{1,2}`, random parity budgets, optionally drop a data plate, then flip 0–4
  words (any magnitude, any offset incl. header) in a present plate. Result:
  **1937 Ok (every one byte-exact for all n), 4063 refused, 0 WRONG.**
- **`indep_corrupt_parity_missing_data`** — 4000 trials: drop data plate 0, then
  heavily corrupt a parity plate (1–19 word flips) while both parity present.
  Result: **0 wrong** xpubs (correct-or-refuse).

**Safety chain (verified by reasoning + the fuzz):** every present plate is decoded
via P4 `decode`, which is tag-gated — within-budget corruption is RS-corrected and
the SHA-256 integrity tag re-verifies; beyond-budget or tag-mismatch ⇒
`Uncorrectable`/`IntegrityMismatch`, propagated by `?` ⇒ **refuse**. So each present
plate is *correct-or-refused* at ≤2⁻ᵗ. The reconstructed (missing) stripe is an
*exact* GF linear combination of the present stripes; if every input stripe is
tag-verified-correct, the output is mathematically exact — there is NO additional
wrong-reconstruction path beyond the inherited per-present-plate ≤2⁻ᵗ
miscorrection bound. `stripe_to_payload` then re-reads the length-prefix and
`symbols_to_bits` enforces `NonZeroPad`, adding a structural sanity check on the
reconstructed stripe. The reconstructed xpub is **not** independently
cross-checkable (the array-id covers the seed, not the payloads), so it rests on
the inherited tag bound — which is the same bound a solo card already carries and
is the documented, accepted property. No new wrong-path introduced. PASS.

### 3. P₁ append-only

- **`indep_p1_append_only_broad`** — `n∈{3,4,8,15,32}`: the r=1 ParityA plate is
  byte-for-byte identical to the r=2 ParityA plate (words equal), and every DATA
  plate is identical across r too (data stripes don't depend on r). PASS. P₂ is a
  pure additional stripe; r is append-only across the RAID dimension.

### 4. Privacy

- **`indep_privacy_underdetermined`** — `n∈{3,4,5,8}`: holding ParityA + (n−2) data
  plates (2 missing > r=1) ⇒ reconstruct **refuses**. Structurally, with `P₁` and
  the present data known, the two missing stripes satisfy a single column equation
  `x_j + x_k = s` with 2¹¹ consistent assignments per column — underdetermined, no
  unique recovery. PASS (the implementer's KAT 5 also exhibits two distinct
  consistent assignments directly).

### 5. No silent array-mixing

- **`indep_no_array_mixing`** — 500 trials mixing most of array A + one foreign B
  plate (distinct array-id): **always errors** (`RaidArrayMismatch`), never blends.
  Grouping requires all plates to share `(array_id, n, width)`.
- **`indep_duplicate_index_errors`** — feeding data plate 0 twice ⇒
  `RaidArrayMismatch`. Mismatched n / inconsistent width ⇒ error (same grouping
  guard). PASS.

### 6. Solo path UNCHANGED (regression — `pipeline.rs` heavily edited)

- The full P4 pipeline suite (`tests/pipeline.rs`, 23 tests) is GREEN at 4000
  cases, including the never-wrong-payload props and cold-decode-from-words. The
  header refactor (hard-coded `5 → header_word_count(has_raid)`,
  `ledger_start = geom.header_words`, `header_offset = geom.header_words`) is
  derived consistently in encode, decode, and `rs_decode_and_check`. The header-CRC
  now covers `H0‖H1‖array-id‖GEOM-A..C` (raid) and `H0‖GEOM-A..C` (solo) via the
  shared `build_geom(prefix, …)`; `parse_header` recomputes over the same positional
  prefix. `indep_garbage_no_panic` also confirms a valid SOLO card fed to
  `raid_reconstruct` ⇒ `RaidArrayMismatch` (a solo card has `raid==None`). PASS.

### 7/8. (covered above under Minor/Nit 1 and 2.)

### 9. Panic-safety / WcError

- **`indep_garbage_no_panic`** — short/garbage/non-word lists, all-`zoo`, a solo
  card ⇒ `WcError`, no panic. Implementer KAT 11 covers `n<2`, `n>32`, `r>2`,
  `r=0`, `r≥n`, empty set. `encode_inner` + `parse_header` both range-check the
  RAID fields defensively (the comment "a hostile list must never panic" is backed
  by `(2..=32).contains(n)`, `role ≤ 2`, `index < 32`, `array_id ≤ 0x3F_FFFF`, and
  `role==DATA ⇒ index<n`). `array_id_from_seed` is structurally ≤ 0x3F_FFFF (top 22
  bits). **New variants `RaidArrayMismatch`, `RaidUnrecoverable` are correctly
  alphabetical** in `enum WcError` and its `Display` match
  (`…InvalidParams < RaidArrayMismatch < RaidUnrecoverable < Regroup…`). PASS.

### 10. Gates

All green — see Suite results above.

---

## Bottom line

P5 meets the gate: **0 Critical / 0 Important.** The MDS recovery and
funds-safety (correct-or-refuse, never silently wrong) properties hold under
independent exhaustive + randomized adversarial testing, including the n=15/n=32
r=2 full-exponent cases that the NEW-I2 fold was specifically meant to fix. The two
flagged items are a safe, plan-consistent limitation (Minor) and a non-issue
(Nit). **Cleared to merge `feat/wc-p5-raid` and proceed to P6.**
