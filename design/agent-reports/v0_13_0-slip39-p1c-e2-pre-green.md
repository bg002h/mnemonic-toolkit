# v0.13.0 P1c-E.2 RED pre-GREEN test-design review

**Phase:** v0.13.0 P1c-E.2 (driver impl + G1 + G2)
**Round:** Pre-GREEN, test-design review
**Reviewer:** Opus (`feature-dev:code-reviewer`)
**Commit under review:** `c257a6a`
**Date:** 2026-05-14

## Verdict

**1 Critical / 2 Important / 4 Nice-to-have — recommend FOLD CRITICAL before dispatching GREEN.**

The RED's overall shape (variant-mapping table, per-vector `#[test]` macro, G2 matrix, T==2 + T==1 split unit tests) is sound. The cross-checks in the T==2 split test (lagrange-interpolate + HMAC verify) correctly pin all three foot-guns called out by R0 C1. The negative-vector variant pinning is verified-correct for 14 of the 15 distinct fixture rows.

One Critical issue: **vector #40's pinned variant is structurally impossible given the toolkit's parser behavior** — the parser will refuse at step 3 (padding > 8) before the combine layer's `InvalidShareValueLength` check can run. This is a R0 I2 architect error that propagated into the plan and the RED. Two Important issues: G2 only ever combines the first `group_threshold` groups of each config, leaving non-zero-indexed groups never round-tripped at combine; and the share_idx remap convention for parse-time refusals deserves an explicit test pin.

After folding the Critical issue (re-pin #40 to `InvalidPadding { share_idx: 0 }`), the RED is clear to dispatch GREEN.

---

## Findings

### Critical

#### C1 — Vector #40 cannot reach `InvalidShareValueLength`; parser refuses with `InvalidPadding` first (confidence: 95)

**Location:** `crates/mnemonic-toolkit/tests/lib_slip39_vectors.rs:117` — `40 => Shape(|e| matches!(e, InvalidShareValueLength { share_idx: 0, .. }))`

**Issue:** Vector #40's mnemonic is 21 words (counted from `tests/fixtures/slip39_vectors.json:357`):

> "fraction necklace academic academic award teammate mouse regular testify coding building member verdict purchase blind camera duration email prepare spirit quarter"

Per `parse_slip39_share` at `src/slip39/share.rs:208-218`:
- word count 21 ≥ MIN_MNEMONIC_LENGTH_WORDS (20) → step 2 passes
- `value_data_word_count = 21 - 7 = 14`
- `total_value_bits = 14 × 10 = 140`
- `padding_bits = 140 mod 16 = 12`
- `padding_bits > 8` → step 3 fires → `Err(InvalidPadding { share_idx: 0 })`

The combine layer's `InvalidShareValueLength` check (per plan §3.4 step 3) only runs after parse succeeds. For vector #40 parse cannot succeed, so the test will assert against an unreachable variant.

This mirrors python-shamir-mnemonic's behavior at `share.py`: the equivalent check `if padding_len > 8: raise MnemonicError("Invalid mnemonic length.")` raises at parse time. The fixture description "invalid master secret length" matches python's error message ("Invalid mnemonic LENGTH"), which the toolkit folds into `InvalidPadding`.

The R0 I2 architect report claim ("parses cleanly, has zero padding bits") is factually wrong — the architect did not check the actual word count.

**Fix:** Re-pin row 40 to `Exact(InvalidPadding { share_idx: 0 })`. Optionally retain the `InvalidShareValueLength` variant + the plan §3.4 step 3 combine-side check as defense-in-depth (no fixture vector exercises it now, but the variant covers a real combine-time class — for shares that DID parse cleanly but somehow carry an out-of-set value length, e.g. forged shares). Update plan §4.1 table row 40 and §8 if the variant is retained-but-unused.

**Optional follow-up:** if the variant is unused in vectors.json, file a FOLLOWUP noting `InvalidShareValueLength` has no fixture coverage and consider dropping it OR construct an internal forged-share unit test (see I2 below) to exercise the check.

---

### Important

#### I1 — G2 never exercises non-zero-indexed groups at combine (confidence: 85)

**Location:** `crates/mnemonic-toolkit/tests/lib_slip39_roundtrip.rs:145` — `for g_idx in 0..group_threshold as usize`

**Issue:** The trial loop picks shares only from groups `[0 .. group_threshold)`. For configs:
- `(1, [(2, 3), (3, 5)])` — only group 0's shares are tested. Group 1's encoding/split path is constructed by the driver but never round-tripped.
- `(2, [(3, 3), (3, 5), (2, 5)])` — groups 0 and 1 tested; group 2 never round-tripped.

A driver bug that mis-indexes group-level Shamir x-coordinates for groups ≥ group_threshold (e.g. off-by-one in the outer split loop's index) would silently slip past G2.

**Fix (one of):**
1. Add a second axis to the trial: for each config with `groups.len() > group_threshold`, run an additional trial that picks from a *rotated* group subset (e.g. groups `[1..group_threshold+1]` instead of `[0..group_threshold]`).
2. OR: pick `group_threshold` groups uniformly at random from `0..groups.len()` per trial (deterministic via the trial seed).

Option 2 is cheaper to code and gives broader coverage.

#### I2 — share_idx remap convention not pinned (confidence: 80)

**Location:** `crates/mnemonic-toolkit/tests/lib_slip39_vectors.rs:134-135` (the `collect::<Result<Vec<_>, _>>()` short-circuit path) + plan §3.4 step 3.

**Issue:** Plan §3.4 step 3 requires the combine driver to map per-share errors (`InvalidShareValueLength`) to use input-position `share_idx`, not the parser's 0. All G1 negative-vector parse failures happen at share_idx=0 (single-share vectors or first-share-broken), so the remap behavior IS untested by the fixture set.

After C1 is folded (vector #40 → InvalidPadding), no remaining G1 vector exercises this path.

**Fix:** Add a single targeted unit test in `#[cfg(test)] mod tests` of `src/slip39/mod.rs` (private access to `Share::from_parts`) that constructs `[valid_share, share_with_wrong_value_length]` and asserts the error has `share_idx: 1`. This pins the remap contract explicitly.

---

### Nice-to-have

#### N1 — DuplicateMemberIndex (#11, #30) can be tightened to Exact (confidence: 80)

**Location:** `lib_slip39_vectors.rs:111` — `11 | 30 => Shape(|e| matches!(e, DuplicateMemberIndex { .. }))`

Both shares of vectors #11 and #30 have share_params words `"academic always"` (verified by decoding: `academic=0`, `always=33` → `share_params_int=33` → `member_index=2, group_index=0, member_threshold=2, group_threshold=1, group_count=1`). Both shares carry `member_index=2` in `group_index=0`. The duplicate is therefore deterministic:

Pin both to `Exact(DuplicateMemberIndex { group_idx: 0, member_idx: 2 })`.

#### N2 — InsufficientShares for #5/#24 verified

I verified vector #5's share_params decode as: `group_threshold=1, group_count=1, member_threshold=2, member_index=2`. Single share to combine. Driver should report `group_idx=0, needed=2, got=1` (member-level insufficiency in group 0). The test pin matches. Just noting it's verified.

#### N3 — Plan §4.2 typo should be patched in the GREEN fold (confidence: 80)

**Location:** RED's `lib_slip39_roundtrip.rs:30-33` flags `(2, [(2, 3)])` as a plan typo and corrects to `(1, [(2, 3)])`.

The plan typo is real (`group_threshold=2` with one group violates the `1 ≤ gt ≤ groups.len()` invariant introduced in `slip39_split` validation). Fold a one-line patch in `design/PLAN_v0_13_0_p1c_e.md` §4.2 in the same commit as the test folds (or as a doc-only follow-up).

#### N4 — `InsufficientShares { .. }` for #14/#15/#16/#33/#34/#35 — fields can be derived but the group-level sentinel is ambiguous (confidence: 60)

For #14 (single share, gt=2, only 1 group present): `needed=2, got=1, group_idx=?` (sentinel — plan says "0 or smallest missing index"). For #16 (gt=2, group_idx={1,3} present, group 3 has 1 of 2 needed members): `needed=2, got=1, group_idx=3`.

These can be pinned to Exact once the driver picks a sentinel convention. Recommend the GREEN commit pick a convention (e.g. group-level insufficiency reports `group_idx=group_threshold` out-of-bounds or `group_idx=255`) and the post-GREEN R1 review tightens the test pins.

---

## Plan looks sound — verified clean

- **Negative variants pinned-correctly:** vectors #2, #3, #5, #6, #7, #8, #9, #10, #11, #12, #13, #14, #15, #16, #21, #22, #24, #25, #26, #27, #28, #29, #30, #31, #32, #33, #34, #35, #39 — all 29 verified via direct share_params decoding (using `slip39_english.txt` indices for the first 4 words) or by parse-flow inspection.
- **Vector #10 / #29 GroupThresholdExceedsCount mapping:** verified via decoding `acrobat acid` share_params → `group_threshold=2, group_count=1` for both vectors' first shares.
- **Vector #5 / #24 InsufficientShares mapping:** verified by decoding `academic always` share_params → single-group configuration where 1-of-2 member-threshold cannot be met from a single share.
- **T==2 unit test cross-check (split_secret_t2_n3...):** the test correctly pins all three R0 C1 foot-guns:
  - (1) random-share loop bound — caught by `shares.len() == 3` assertion (any loop overshoot adds a 4th share)
  - (2) digest||random concat order — caught by HMAC verify with `random_part = digest_payload[4..]`
  - (3) emit indices — caught by `*x == i as u8` for each share
  - The HMAC cross-check has a ~2^-32 false-pass probability if the impl transposes; acceptable per the threat model.
- **T==1 unit test (split_secret_t1_n5_replicates...):** correctly pins the no-digest replication path.
- **G2 matrix shape:** 5 × 4 × 2 × 5 = 200 trials confirmed; `trial_idx = trial_counter * 1009 + trial` is collision-free for trial counts < 1009.
- **G2 plan-typo correction:** `(1, [(2, 3)])` is the correct interpretation of "single group 2-of-3"; `(2, [(2, 3)])` violates `gt ≤ groups.len()`.
- **G1 positive-vector assertions:** hex-secret + xprv (Network::Bitcoin, mainnet) match python-shamir-mnemonic's `generate_vectors.py` derivation. Vector #41 is just a regular positive vector (its description note about "modular arithmetic errors" is upstream commentary, not a test signal).
- **G1 vector #39 InvalidPadding pin:** verified — 19 words < 20-word minimum → step 2 (`indices.len() < MIN_MNEMONIC_LENGTH_WORDS`) returns `InvalidPadding { share_idx: 0 }`. Test pin correct.
- **Compile-fail surface:** RED references `slip39_split`, `slip39_combine`, `GroupSpec`, `split_secret` — none exist. GREEN must add exactly these. `share.value()` accessor is YAGNI (driver internals can access the private field within the slip39 module; tests use `render_slip39_share`).
- **Public surface anchor test** (`group_spec_is_public_at_slip39_module_root`): pins `GroupSpec` is reachable via `mnemonic_toolkit::slip39::GroupSpec`. Good.
- **Memory hygiene:** combine returns `Zeroizing<Vec<u8>>`; test uses `recovered.as_slice()` which transparently derefs. ✓
- **G2 extensive test gating:** `#[ignore]`-attribute correctly applied per the `feedback_default_cargo_test_runs_sibling_dependent_tests` memory entry.

---

## Recommendation

**Fold C1 before dispatching GREEN.** The vector #40 test will fail at GREEN regardless of how correct the driver is, because the parser refuses the share before combine runs. Recommended approach:

1. Change `lib_slip39_vectors.rs:117` to `40 => Exact(InvalidPadding { share_idx: 0 })`. Update plan §4.1 table row 40 in the same commit. File a FOLLOWUP noting `InvalidShareValueLength` has no fixture coverage.
2. Retain `InvalidShareValueLength` AND construct a forged-share unit test (in `mod tests` of `src/slip39/mod.rs`) that synthesizes a Share with an out-of-set value length and passes it directly to `slip39_combine`. This keeps the variant's defense-in-depth value AND closes I2 (share_idx remap test) in a single test.

I1 (G2 group rotation) can fold pre-GREEN OR be promoted to post-GREEN R1 follow-up. N1 (tighten #11/#30 to Exact) is a one-line fix; recommend folding now. N3 (plan §4.2 typo) is a doc-only fix in the same commit.

Total fold scope: ~30-line diff across `lib_slip39_vectors.rs`, `lib_slip39_roundtrip.rs`, `src/slip39/mod.rs`, and `design/PLAN_v0_13_0_p1c_e.md`.

---

## References

- Plan: `design/PLAN_v0_13_0_p1c_e.md`
- R0 architect report: `design/agent-reports/v0_13_0-slip39-driver-plan-r0.md`
- RED commit: `c257a6a`
- RED files:
  - `crates/mnemonic-toolkit/tests/lib_slip39_vectors.rs`
  - `crates/mnemonic-toolkit/tests/lib_slip39_roundtrip.rs`
  - `crates/mnemonic-toolkit/src/slip39/mod.rs` (`#[cfg(test)] mod tests`)
- Fixture: `crates/mnemonic-toolkit/tests/fixtures/slip39_vectors.json`
- Parser ground truth: `crates/mnemonic-toolkit/src/slip39/share.rs:208-269`
- python-shamir-mnemonic @ 17fcce14: [`shamir.py::_split_secret`](https://github.com/trezor/python-shamir-mnemonic/blob/17fcce14/shamir_mnemonic/shamir.py), [`share.py`](https://github.com/trezor/python-shamir-mnemonic/blob/17fcce14/shamir_mnemonic/share.py), [`cipher.py`](https://github.com/trezor/python-shamir-mnemonic/blob/17fcce14/shamir_mnemonic/cipher.py)
