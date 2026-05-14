# v0.13.0 P1c-E — SLIP-39 split/combine driver — R0 (plan-review, pre-code)

**Phase:** v0.13.0 P1c-E (driver + G1 + G2)
**Round:** R0 plan-review, pre-code
**Reviewer:** Opus (`feature-dev:code-reviewer`, architect role)
**Date:** 2026-05-14
**Plan under review:** `design/PLAN_v0_13_0_p1c_e.md`

## Verdict

**1 Critical / 4 Important / 2 Nice-to-have — RECOMMENDATION: revise plan before coding.**

The driver shape, the two primitive scope expansions, and the test design are all fundamentally sound. The plan correctly identifies that `interpolate_at_zero` and `feistel`'s `salt_prefix` need to be extended, and the algorithm steps in §3.2-§3.5 mostly match the python reference. One Critical clarity gap (T==2 base-share shape + digest||random concat order). Four Important gaps in negative-vector variant mapping and cross-share consistency must be resolved before RED tests are written, because mis-pinning a variant in RED is the most expensive thing to discover at GREEN. After folding, code is cleared to start.

---

## Answers to §7 open questions

### Q1 — Should `slip39_split` accept `extendable: bool` directly?

**Answer: Yes, accept it as a parameter to the library function. Fix the CLI default to `false` for v0.13.0 and file a P2-deferral FOLLOWUP for the CLI flag.**

Rationale: the library is the durable surface; gating an essential SLIP-0039 axis at the library layer creates a future API break. The python ref is `extendable=True` by default since 2024 but accepts it as an argument explicitly. The G1 test vectors include 4 `extendable=true` vectors (#42-45) for the *combine* path — those work without `slip39_split` exposing the flag. The G2 round-trip property test in §4.2 explicitly contemplates both axes, so the library MUST accept the flag.

The CLI flag is the only thing that can be deferred without a SemVer break. Default the CLI to `false` (matches Trezor Model T's legacy default and the majority of the G1 vector set) and file `slip39-cli-extendable-flag` FOLLOWUP for v0.14.

### Q2 — Disposition of ambiguous-description vectors (#5, #10, #24, #29, #40)

**Answer: Pin specific variants. None are truly ambiguous once you read the share content.**

- **#5 / #24** ("Basic sharing 2-of-3" — single share): This is one share of a 2-of-3 set (matches the first share of #4 / #23 byte-for-byte). It's an *insufficient shares at group level* refusal. Map to `InsufficientShares { group_idx: 0, needed: 2, got: 1 }`. (Trezor's vectors.json is poorly labeled here.)

- **#10 / #29** ("greater group threshold than group counts"): Python raises `MnemonicError` at PARSE time when `group_count < group_threshold` (verified in `share.py:216-219` of python-shamir-mnemonic). The toolkit's current `parse_slip39_share` does NOT enforce this. **Recommended:** add the check at parse time matching python, and add a new variant `Slip39Error::GroupThresholdExceedsCount { share_idx, threshold, count }`. This is a parse-layer scope expansion of the LOCKed P1c-D surface that must be folded explicitly; the plan currently silently inherits the gap.

- **#40** ("invalid master secret length"): single share that *parses successfully* but whose value-byte length is not in `{16, 20, 24, 28, 32}`. This is a combine-time refusal because the parse layer (correctly) does not enforce master-secret length per-share. Map to a NEW variant `Slip39Error::InvalidShareValueLength { share_idx, got }`. Folding into a generic `InvalidPadding` is WRONG because vector #40 has zero padding bits (it gets past padding to test a deeper check); lumping it into InvalidPadding would mask real padding-bit bugs.

### Q3 — Feistel: preserve no-arg legacy variant, or fully replace?

**Answer: Fully replace. Add `extendable: bool` as the 5th parameter to `encrypt`/`decrypt`. Do NOT preserve a legacy wrapper.**

Rationale: `feistel::encrypt`/`decrypt` are pub-but-internal (`pub mod feistel` in `mod.rs` but no caller outside `src/slip39/`). The 19 P1b tests are the only external consumers. Adding a `false` argument at each test site is a 19-line mechanical edit. A legacy wrapper preserves an API that no one uses and would surface as dead code in clippy. The P1b LOCK was against the primitive contract, not the public API — extending the primitive is acceptable.

However: the P1c-E.1 RED commit must include EVERY P1b test site updated explicitly, and the P1c-E.1 GREEN commit's diff must be ZERO-TOLERANCE for any P1b test changing its assertion direction (only `feistel::encrypt(a, b, c, d)` → `feistel::encrypt(a, b, c, d, false)`). Anything else is regression.

### Q4 — Is G6 hygiene correctly deferred to P1c-E.3?

**Answer: Yes, defer is correct. But add a precondition to P1c-E.2's verification gate.**

Rationale: SPEC §4 G6 lists "split's output buffer must pin once across all shares; combine's recovered secret must pin once" as a Cycle B requirement. Layering hygiene on top of a correct algorithm is the right phase order (matches v0.10.0 cycle B's pattern where the algorithm landed first, then mlock layered on). The plan correctly notes that pinning is OOS for the FIRST RED+GREEN pass.

**Augmentation:** P1c-E.2 LOCK must declare `lint_zeroize_discipline.rs`'s loose-bound row count (`18..=35`) acceptable WITHOUT the new pins so the lint doesn't false-alarm at green. The plan does not say this.

### Q5 — xprv derivation for G1 positive vectors?

**Answer: Required at G1 LOCK; the `bitcoin` crate is already a dep (verified in Cargo.toml: `bitcoin = "0.32"`).**

Rationale: SPEC §4 G1 says explicitly "must recover the expected hex secret + match the expected BIP-32 xprv". Hex-secret-only is the algorithm-correctness gate; xprv match is the *encoding-pathway* gate (proves BIP-32 master-from-seed derivation runs correctly on the recovered bytes). `bitcoin = "0.32"` is already in `[dependencies]` so no new dep weight.

---

## Findings

### Critical

#### C1 — `recover_secret` digest-verification path skipped when `threshold == 2`

**Location:** Plan §3.5 ("recover_secret helper") and §3.3

**Issue:** The plan reads:

> Generate (T-2) random shares at indices 0..T-2.
> Compute D = HMAC-SHA256(R, secret)[0..4] || R where R is `len(secret)-4` random bytes.
> Base shares = [random_0..T-2, (254, D), (255, secret)].

For T=2, `random_0..T-2` is the empty list, so base shares are `[(254, D), (255, secret)]` — exactly 2 base shares for a degree-1 polynomial. Correct, but the plan does not assert this and the implementer could easily flip a `<` to `<=` on the random-share loop. Similarly the §3.5 combine-side recovery for `threshold == 2` is implicit: it MUST run the digest verification using the same `interpolate_at(shares, 254)` + `interpolate_at(shares, 255)` machinery as `threshold == 3+`. The plan §3.5 correctly does this, but the §3.3 split-side reciprocal is *not* spelled out.

**Fix:** Add explicit text to §3.3:

> For T == 2, the random-share loop iterates 0 times (T-2 = 0), so base shares are exactly `[(DIGEST_INDEX, digest_payload), (SECRET_INDEX, secret)]`. The N emitted shares at indices 0..N are computed via `interpolate_at(base_shares, i)` for i in 0..N. The digest IS computed and IS used for verification at combine time when T == 2 (unlike T == 1, where the digest is skipped entirely).

Also pin the digest-payload byte ordering as `digest (4 bytes) || random_part (n-4 bytes)` — verified against python `RawShare(DIGEST_INDEX, digest + random_part)`. A transposed concatenation is impossible to find from a single failing G1 vector if all you have is "digest mismatch".

**Confidence: 85.** A unit test pinning `_split_secret(T=2, N=3, secret=[fixed])` against known digest bytes would catch the random-share-loop bound and the concat-order foot-guns before the G1 harness runs.

---

### Important

#### I1 — Cross-share consistency check missing share-value-length; add `ShareValueLengthMismatch` and `ExtendableMismatch`

**Location:** Plan §3.4 step 3

**Issue:** The plan enumerates the consistency-tuple as `(identifier, extendable, iter_exp, group_threshold, group_count)` but the SLIP-0039 spec also requires:

> All shares MUST have the same ... iteration exponent, group threshold, group count and **length**.

Share-value length consistency is not enumerated. Python checks it implicitly when interpolation runs and panics on mismatched lengths. The toolkit's `lagrange::interpolate_secret_at_zero` panics on length mismatch (verified at `lagrange.rs:65-75`). A panic on user-supplied input is a contract violation — the combine entry point MUST refuse cleanly with a dedicated error variant.

Additionally, while the current plan's tuple DOES include `extendable`, there is no `Slip39Error::ExtendableMismatch` variant in `error.rs`. Mixing extendable=true and extendable=false shares is a hard refusal class that needs its own variant (NOT folded into IdentifierMismatch — they're orthogonal).

**Fix:**

1. Add `Slip39Error::ShareValueLengthMismatch` (parser succeeds per-share, but values across shares differ in length).
2. Add `Slip39Error::ExtendableMismatch` (ext bit divergence across shares).
3. Update §3.4 step 3's consistency-tuple enumeration to: `(identifier, extendable, iter_exp, group_threshold, group_count, value_length)` — 6 invariants, not 5.

**Confidence: 90.**

---

#### I2 — Negative-vector variant mapping for #40 must be pinned at RED, not TBD at GREEN

**Location:** Plan §4.1 table — "Mnemonic with invalid master secret length" row

**Issue:** Vector #40 is a single share that parses cleanly (passes RS1024, has zero padding bits) but whose recovered value-bytes length is not in `{16, 20, 24, 28, 32}`. The plan says "TBD — likely a new variant or InvalidPadding".

**Fix:** Add `Slip39Error::InvalidShareValueLength { share_idx, got }`. The check fires immediately after parse in `slip39_combine` (before consistency cross-share checks), per-share, validating `value.len() ∈ {16, 20, 24, 28, 32}`. Pin vector #40 → `InvalidShareValueLength { share_idx: 0, got: <actual> }`.

**Confidence: 85.**

---

#### I3 — Vectors #10 / #29 disposition requires a P1c-D scope-expansion folded INTO P1c-E.1

**Location:** Plan §4.1 row "greater group threshold than group counts" + §2.1/§2.2 scope-expansion narrative

**Issue:** Python enforces `group_count >= group_threshold` at parse time in `Share.from_mnemonic` (verified `share.py:216-219`). Toolkit's `parse_slip39_share` does NOT enforce this. The plan claims only 2 primitive scope expansions (lagrange + feistel) but this is a third silent expansion of the LOCKed P1c-D surface.

**Fix:** Add the parse-time check to `parse_slip39_share` (P1c-D scope expansion, lands in P1c-E.1). Add `Slip39Error::GroupThresholdExceedsCount { share_idx, threshold, count }`. Pin vectors #10/#29 to this variant. Matches python; fails fast at parse-time is the right semantic for invalid metadata.

**Confidence: 90.**

---

#### I4 — G2 should test both `extendable` axes; trim cartesian product deliberately

**Location:** Plan §4.2

**Issue:** The plan posits `5 × 4 × 2 × 50 = 2000+` round-trip ops. At iter_exp=0 that's ~40M PBKDF2-SHA-256 iters, tens of seconds on commodity x86 and closer to a minute on weak CI runners. The plan mentions `#[ignore]` if this balloons — but ignored tests do not run in default `cargo test`, which is where they catch regressions.

Per `feedback_default_cargo_test_runs_sibling_dependent_tests.md` memory entry: `#[ignore]`-gated tests need a dedicated CI job opting in via `-- --include-ignored`.

**Fix:** Trim deliberately:

- Default `cargo test`: 5 entropy sizes × 4 group configs × 2 ext-axes × 5 trials = 200 trials.
- `#[ignore]`-gated `extensive_roundtrip_test`: 5 × 4 × 2 × 50 = 2000 trials, opted into via dedicated CI job.

**Confidence: 80.**

---

### Nice-to-have

#### N1 — `interpolate_at_zero` wrapper retention is API surface bloat

**Location:** Plan §2.1

**Issue:** The plan says "the existing `_at_zero` variants become wrappers calling `_at(..., 0)`". After P1c-E.1 ships, there is no caller of `interpolate_at_zero` (driver uses `interpolate_at(shares, 254)` and `interpolate_at(shares, 255)`).

**Fix:** Delete `interpolate_at_zero` and `interpolate_secret_at_zero` once `interpolate_at`/`interpolate_secret_at` ship; update P1a tests in-place. Equivalent net LOC; cleaner API.

---

#### N2 — Pre-GREEN test-design review dispatch should explicitly name the new variants to be pinned

**Location:** Plan §5

**Issue:** The pre-GREEN test-design review (which proved itself at P1c-D) is the right pattern. But the dispatch should explicitly hand the reviewer the variant-mapping table for the 30 negative vectors (post the I2/I3 folds above), so the reviewer can verify the RED tests pin the right variant before GREEN.

**Fix:** Update §4.1's table with the post-fold dispositions for #5/#10/#24/#29/#40, then dispatch with "verify this exact mapping" as the reviewer's primary task.

---

## Plan looks sound — verified clean

The following parts of the plan were verified clean and need NO changes:

- **Public surface §3.1** matches SPEC §2.1 verbatim. Parameter ordering, lifetime/ownership shape (Zeroizing wrapper on combine return), and RNG generic bound all correct.
- **§3.2 split algorithm step ordering** matches python `generate_mnemonics` + `_split_secret`. Group-level split runs first on EMS, then per-group member-level split. Correct.
- **§3.4 step 4 (per-group member-threshold uniformity + duplicate-member-index detection)** matches python `recover_ems` enforcement. Correct.
- **§3.5 recover_secret threshold-1 special case** matches python: `return shares[0].1.clone()` when T==1 (skip digest). Correct (subject to C1's clarification).
- **§3.6 memory hygiene** correctly identifies that recovered master, all intermediate share values, and random buffers must be Zeroizing-wrapped; correctly defers the mlock pinning to P1c-E.3.
- **§5 phase split** (P1c-E.1 primitives → P1c-E.2 driver) is the right granularity. Two RED+GREEN pairs over two sub-phases avoids a 1500-LOC blast radius commit. Pre-GREEN test-design review is the right pattern.
- **§6 risk areas** correctly identifies the digest 4-byte || R structure, the group/member nesting depth, the ext=1 dependency, and the threshold-1 special case.
- **G1 harness shape §4.1** (one #[test] per vector via macro/paste) is the right shape — matches the v0.11.0 + v0.12.0 precedents and gives per-vector failure granularity.

---

## Recommendation

**Revise plan before coding.** Fold:

1. **C1**: explicit text on T==2 base-share shape + digest||random concat order; add a `_split_secret` unit test as part of P1c-E.2 RED.
2. **I1**: add `ShareValueLengthMismatch` + `ExtendableMismatch` variants; update §3.4 consistency-tuple to 6 invariants.
3. **I2**: pin vector #40 → new `InvalidShareValueLength` variant.
4. **I3**: add `GroupThresholdExceedsCount` parse-time refusal; pin vectors #10/#29.
5. **I4**: trim G2 default test count to ~200 trials; gate the 2000-trial run with `#[ignore]` + dedicated CI job per the memory entry.

After folding, dispatch P1c-E.1 RED (lagrange + feistel primitive extensions + the new error variants + the new parse-time check on share). Once green, dispatch a pre-GREEN test-design review for P1c-E.2 with the explicit variant-mapping table to verify before driver-impl code starts.

---

## References

- Plan: `design/PLAN_v0_13_0_p1c_e.md`
- SPEC: `design/SPEC_slip39_v0_13_0.md` §2, §4
- [SLIP-0039 spec](https://github.com/satoshilabs/slips/blob/master/slip-0039.md)
- [python-shamir-mnemonic @ 17fcce14](https://github.com/trezor/python-shamir-mnemonic/tree/17fcce14)
- Vendored fixture: `crates/mnemonic-toolkit/tests/fixtures/slip39_vectors.json`
