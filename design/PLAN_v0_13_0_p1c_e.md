# P1c-E execution plan (v0.13.0 SLIP-39 split/combine driver)

**Phase:** v0.13.0 P1c-E (driver + G1 + G2)
**Status:** Plan-mode artifact, R0 architect-reviewed at `design/agent-reports/v0_13_0-slip39-driver-plan-r0.md` (1C/4I/2N — all findings folded into this revision).
**Date:** 2026-05-14
**Preconditions:** P1c-D LOCKed at `b20025b`; P0/P1a/P1b/P1c-A/B/C/D all shipped + pushed to origin.

## §1 Goal

Land the SLIP-0039 split/combine driver in `src/slip39/mod.rs`, the G1 vectors harness in `tests/lib_slip39_vectors.rs`, and the G2 round-trip property test in `tests/lib_slip39_roundtrip.rs`. Total ~800–1100 LOC across 5+ files. Closes the library-side of the v0.13.0 cycle (P2 CLI + P3 manual + PE rollup follow).

## §2 Scope expansions of P1a + P1b + P1c-D

The full SLIP-39 algorithm surfaces three limitations in the already-LOCKed primitives + share parser:

### §2.1 Lagrange — `interpolate_at_zero` is x=0-only

SLIP-0039 stores the master secret at `SECRET_INDEX = 255` and the digest payload at `DIGEST_INDEX = 254`. Recovery must interpolate at x=255 (secret) and x=254 (digest), neither of which is x=0.

**Fix:** Add `lagrange::interpolate_at(points: &[(u8, u8)], x: u8) -> u8` and `lagrange::interpolate_secret_at(points: &[(u8, &[u8])], x: u8) -> Vec<u8>` (general-purpose).

**DELETE** the existing `interpolate_at_zero` and `interpolate_secret_at_zero` (no consumers post-P1c-E). Update the 7 P1a math tests at `lib_slip39_math.rs:186-261` to call `interpolate_at(..., 0)` and `interpolate_secret_at(..., 0)` respectively. Cleaner API; no legacy-wrapper bloat (per R0 N1).

Python ground truth: `_interpolate(shares, x)` at `shamir.py:78` takes x as a parameter; the secret-recovery call is `_interpolate(shares, SECRET_INDEX)`.

### §2.2 Feistel — salt prefix wrong for ext=1

SLIP-0039 §"Encryption of the master secret": *"If ext = 1, then salt_prefix is an empty string. If ext = 0, then salt_prefix = 'shamir' || id, where the random identifier value id is encoded as two bytes in big-endian byte order."*

Current `feistel::encrypt/decrypt` always include `b"shamir" || identifier` regardless of ext.

**Fix:** Add `extendable: bool` as a 5th parameter to both `encrypt` and `decrypt`. When true, `build_salt_prefix` returns empty. **No legacy wrapper** — fully replace (per R0 Q3). Update all 19 P1b test sites mechanically to append `, false` to existing calls; ADD new tests covering `extendable=true` round-trip + cross-axis non-interoperability (encrypt with ext=0 → decrypt with ext=1 yields garbage, not master).

R0 Q3 constraint: P1c-E.1 GREEN commit's diff must be ZERO-TOLERANCE for any P1b test changing its assertion direction. Only `feistel::encrypt(a, b, c, d)` → `feistel::encrypt(a, b, c, d, false)`. Anything else is regression.

### §2.3 Share parser — `group_count >= group_threshold` not enforced (R0 I3)

Python `Share.from_mnemonic` raises at parse time when `group_count < group_threshold` (verified `share.py:216-219` of python-shamir-mnemonic). Toolkit's `parse_slip39_share` does NOT enforce this. Vectors #10 / #29 hit this case.

**Fix:** Add the parse-time check to `parse_slip39_share` (P1c-D scope expansion, lands in P1c-E.1). Add new `Slip39Error::GroupThresholdExceedsCount { share_idx, threshold, count }` variant. Update `tests/lib_slip39_share.rs` with a new negative test pinning this refusal.

This is a P1c-D LOCK re-entry — but cleanly, since the new check only refuses additional inputs (no behavior change for previously-accepted inputs).

## §3 Driver design (`src/slip39/mod.rs`)

### §3.1 Public surface (per SPEC §2.1 + R0 Q1)

```rust
pub struct GroupSpec { pub member_count: u8, pub member_threshold: u8 }

pub fn slip39_split(
    master_secret: &[u8],
    passphrase: &[u8],
    group_threshold: u8,
    groups: &[GroupSpec],
    iteration_exponent: u8,
    extendable: bool,                  // R0 Q1: library exposes; CLI hardcodes false for v0.13.0
    identifier: Option<u16>,
    rng: &mut (impl rand_core::CryptoRng + rand_core::RngCore),
) -> Result<Vec<Vec<Share>>, Slip39Error>;

pub fn slip39_combine(
    shares: &[Share],
    passphrase: &[u8],
) -> Result<zeroize::Zeroizing<Vec<u8>>, Slip39Error>;
```

R0 Q1 note: SPEC §2.1's `slip39_split` signature already implies `extendable` is required for the library surface. CLI defers to `false` in v0.13.0 P2, file `slip39-cli-extendable-flag` FOLLOWUP for the v0.14 CLI flag.

### §3.2 Split algorithm

1. Validate inputs:
   - master_secret length ∈ {16, 20, 24, 28, 32}. Refuse with `BadEntropyByteLength`.
   - iteration_exponent ∈ 0..=15. Refuse with `BadIterationExponent`.
   - 1 ≤ group_threshold ≤ groups.len() ≤ 16. Refuse with `BadGroupThreshold`.
   - For each `GroupSpec`: 1 ≤ member_threshold ≤ member_count ≤ 16. Refuse with `BadGroupSpec`.
   - Per python `split_ems`: forbid `member_threshold == 1 && member_count > 1` (use 1-of-1 sharing instead). Refuse with `BadGroupSpec`.
2. Derive identifier from RNG if `None`; mask to 15 bits via `& 0x7FFF`.
3. Encrypt master via `feistel::encrypt(master_secret, passphrase, iter_exp, identifier, extendable)` → EMS.
4. Group-level split: `split_secret(group_threshold, groups.len(), &ems)` → `Vec<(u8, Vec<u8>)>` of `(group_index, group_share_value)`.
5. For each group: `split_secret(group.member_threshold, group.member_count, &group_share_value)` → member shares.
6. Wrap each (group_index, group_share, member_index, member_share, ..., metadata) into a `Share` via `Share::from_parts`.

### §3.3 `split_secret` helper (R0 C1 — T==2 base-share shape clarified)

```rust
fn split_secret(threshold: u8, share_count: u8, secret: &[u8], rng: &mut impl Rng) -> Vec<(u8, Vec<u8>)>;
```

Algorithm per python `_split_secret`:

- **If threshold == 1**: return `[(0, secret), (1, secret), ..., (N-1, secret)]` (replication). NO digest computed.
- **Else (threshold >= 2)**:
  - Generate `T - 2` random shares at indices `0..T-2`. **For T == 2, this is 0 random shares — the random-share loop iterates 0 times.**
  - Compute `R = RANDOM_BYTES(len(secret) - 4)` (`len(secret) - 4` bytes).
  - Compute `digest = HMAC-SHA256(key=R, msg=secret)[0..4]` (4 bytes).
  - **Digest payload byte order**: `digest_payload = digest (4 bytes) || R (n-4 bytes)`. NOT the inverse. Verified against python `RawShare(DIGEST_INDEX, digest + random_part)`.
  - Base shares = `[random_0..T-2, (254, digest_payload), (255, secret)]`. For T == 2, base shares are exactly `[(254, digest_payload), (255, secret)]`.
  - For `i in T-2..N`: `share[i] = (i, interpolate_secret_at(base_shares, i))`.

**R0 C1 dedicated unit test** (lands in P1c-E.2 RED): pin `_split_secret(threshold=2, share_count=3, secret=[fixed 16 bytes])` against a known digest payload + known interpolated share bytes from a hand-computed reference. This catches both the random-share-loop bound foot-gun AND the concat-order foot-gun before the G1 harness runs.

### §3.4 Combine algorithm

1. Reject empty `shares` input → new `Slip39Error::EmptyShares` variant.
2. Parse each share via `parse_slip39_share`. Map `share_idx` from 0 (parser's convention) to the input position via `map_err` reindexing.
3. **Per-share value-length sanity** (R0 I2): for each parsed share, validate `value.len() ∈ {16, 20, 24, 28, 32}`. Refuse with new `Slip39Error::InvalidShareValueLength { share_idx, got }`. This catches vector #40.
4. **Cross-share consistency** (R0 I1 — 6 invariants, not 4): collect `(identifier, extendable, iter_exp, group_threshold, group_count, value_len)` for each share. If any of the six fields disagrees across shares:
   - identifier diverges → `IdentifierMismatch`
   - extendable diverges → new `Slip39Error::ExtendableMismatch`
   - iter_exp diverges → `IterationExponentMismatch`
   - group_threshold diverges → `GroupThresholdMismatch`
   - group_count diverges → `GroupCountMismatch`
   - value_len diverges → new `Slip39Error::ShareValueLengthMismatch`
5. Group shares by `group_index`. For each group:
   - All shares must have the same `member_threshold` → else `MemberThresholdMismatch`.
   - Member indices must be pairwise distinct → else `DuplicateMemberIndex`.
   - Number of shares < member_threshold → `InsufficientShares { group_idx, needed, got }`.
   - Number of shares > member_threshold → also `InsufficientShares` (the algorithm strictly needs == threshold).
6. Group-level threshold: number of distinct `group_index` values < `group_threshold` → `InsufficientShares` (group-level; `group_idx` field encodes a sentinel like 0 or the smallest missing index).
7. For each group: `recover_secret(member_threshold, [(member_index, share_value), ...])` → group share value.
8. Group-level recover: `recover_secret(group_threshold, [(group_index, group_share_value), ...])` → EMS.
9. `feistel::decrypt(&ems, passphrase, iter_exp, identifier, extendable)` → master_secret.
10. Wrap in `Zeroizing<Vec<u8>>` for return.

### §3.5 `recover_secret` helper

```rust
fn recover_secret(threshold: u8, shares: &[(u8, Vec<u8>)]) -> Result<Vec<u8>, Slip39Error>;
```

Per python `_recover_secret`:

- **If threshold == 1**: return `shares[0].1.clone()`. NO digest verification.
- **Else (threshold >= 2)** — applies uniformly to T == 2 and T >= 3:
  - secret = `interpolate_secret_at(shares, 255)`.
  - digest_share = `interpolate_secret_at(shares, 254)`.
  - digest = `digest_share[0..4]` (first 4 bytes); random_part = `digest_share[4..]` (remaining n-4 bytes).
  - If `HMAC-SHA256(key=random_part, msg=secret)[0..4] != digest` → `DigestVerificationFailed`.
  - Return secret.

### §3.6 Memory hygiene

- All intermediate share values wrapped in `Zeroizing<Vec<u8>>`.
- Recovered master returned as `Zeroizing<Vec<u8>>`.
- Random share values for split use a `Zeroizing<Vec<u8>>` buffer.
- `interpolate_secret_at` returns `Vec<u8>` — caller wraps in Zeroizing at the boundary.
- **G6 mlock pinning + lint rows DEFERRED to P1c-E.3** (per R0 Q4). P1c-E.2 LOCK criteria explicitly permit current `lint_zeroize_discipline.rs` loose-bound row count without the new pins.

## §4 Test design

### §4.1 G1 vectors harness (`tests/lib_slip39_vectors.rs`, ~400-500 LOC)

Loads the vendored `slip39_vectors.json` via `include_str!` + `serde_json::from_str`. Iterates 45 entries. For each:

```rust
struct Vector {
    description: String,
    mnemonics: Vec<String>,
    hex_secret: String,    // empty for negative
    expected_xprv: String, // empty for negative
}

fn negative_expected(description: &str) -> Slip39Error;
```

Test layout: one `#[test]` per vector (using a macro or `paste!` for 45 named tests). Each test calls `slip39_combine(parsed_shares, b"TREZOR")` (Trezor's standard test passphrase, NOT b"") and asserts:
- Positive: `hex::encode(combine_result?) == hex_secret`, then derive BIP-32 master xprv via `bitcoin::bip32::Xpriv::new_master(network, &recovered)` and assert `xprv.to_string() == expected_xprv`. R0 Q5: `bitcoin = "0.32"` already in Cargo.toml.
- Negative: `combine_result.unwrap_err() == negative_expected(description)`.

**Negative-variant mapping (post-R0 folding — verified at pre-GREEN test-design review per R0 N2):**

| Vector # | Description | Expected Slip39Error |
|---|---|---|
| 2, 21 | Mnemonic with invalid checksum | `InvalidChecksum { share_idx: 0 }` |
| 3, 22 | Mnemonic with invalid padding | `InvalidPadding { share_idx: 0 }` |
| 5, 24 | Basic sharing 2-of-3 (1 share of 2-needed) | `InsufficientShares { group_idx: 0, needed: 2, got: 1 }` |
| 6, 25 | Mnemonics with different identifiers | `IdentifierMismatch` |
| 7, 26 | Mnemonics with different iteration exponents | `IterationExponentMismatch` |
| 8, 27 | Mnemonics with mismatching group thresholds | `GroupThresholdMismatch` |
| 9, 28 | Mnemonics with mismatching group counts | `GroupCountMismatch` |
| 10, 29 | Mnemonics with greater group threshold than group counts | `GroupThresholdExceedsCount { share_idx, threshold, count }` (NEW; parse-time refusal) |
| 11, 30 | Mnemonics with duplicate member indices | `DuplicateMemberIndex { group_idx, member_idx }` |
| 12, 31 | Mnemonics with mismatching member thresholds | `MemberThresholdMismatch` |
| 13, 32 | Mnemonics giving an invalid digest | `DigestVerificationFailed` |
| 14, 15, 33, 34 | Insufficient number of groups | `InsufficientShares { group_idx: <sentinel>, needed, got }` |
| 16, 35 | Threshold groups but insufficient members | `InsufficientShares { group_idx, needed, got }` (member-level) |
| 39 | Mnemonic with insufficient length | `InvalidPadding { share_idx: 0 }` (per P1c-D fold) |
| 40 | Mnemonic with invalid master secret length | `InvalidPadding { share_idx: 0 }` (pre-GREEN C1 re-pin: vector is 21 words → `padding_bits = 140 % 16 = 12 > 8` → parser refuses at step 3 BEFORE the combine-layer `InvalidShareValueLength` check can run; the variant is retained as defense-in-depth and exercised by a synthetic forged-share test in `src/slip39/mod.rs::tests::combine_invalid_share_value_length_remaps_share_idx_to_input_position`) |

Plus #5 / #24 and the "Basic sharing 2-of-3" duplicate description: dispatch by inspecting `mnemonics.len()`. If `mnemonics.len() == 1` and `mnemonics[0]` matches a share known to be from the 2-of-3 set → InsufficientShares.

R0 N2 fold: the pre-GREEN test-design review for P1c-E.2 takes this exact table as its primary verification task.

### §4.2 G2 round-trip property test (`tests/lib_slip39_roundtrip.rs`, ~200 LOC) — R0 I4 fold

**Default `cargo test` shape — ~200 trials (target ≤ 5 seconds at iter_exp=0):**
- 5 entropy sizes: {16, 20, 24, 28, 32}
- 4 group configs (notation: `(group_threshold, [(member_threshold, member_count), ...])`):
  - `(1, [(1, 1)])` — 1-of-1 trivial
  - `(1, [(2, 3)])` — single group 2-of-3 (pre-GREEN N3 typo fix: was `(2, [(2, 3)])` which violates `group_threshold ≤ groups.len()`)
  - `(1, [(2, 3), (3, 5)])` — 1-of-2 groups (either group reconstructs)
  - `(2, [(3, 3), (3, 5), (2, 5)])` — 2-of-3 groups, varied member configs
- 2 ext-axes: {false, true}
- 5 trials per shape: 5 × 4 × 2 × 5 = 200 trials.

For each trial: split → flatten shares → shuffle → take `≥ threshold` per group → combine → assert byte-equal master.

**Extensive `#[ignore]`-gated test** (R0 I4 + memory `feedback_default_cargo_test_runs_sibling_dependent_tests`): 5 × 4 × 2 × 50 = 2000 trials. Run via a dedicated CI job opting in with `cargo test -- --include-ignored`. Required for cycle PE LOCK, optional for P1c-E.2 LOCK.

Deterministic seeding via `rand_chacha::ChaCha20Rng::seed_from_u64(SEED + per_shape_offset)`.

## §5 Phase split

Four commit pairs over two sub-phases:

| Sub-phase | RED commit | GREEN commit |
|---|---|---|
| P1c-E.1 primitives + share-parser fix | `test(slip39): v0.13.0 P1c-E.1 RED — primitive extensions + 4 new error variants + share parse-time group-threshold-vs-count refusal` | `feat(slip39): v0.13.0 P1c-E.1 GREEN — lagrange interpolate_at + feistel extendable + share GroupThresholdExceedsCount` |
| P1c-E.2 driver | `test(slip39): v0.13.0 P1c-E.2 RED — slip39_split + slip39_combine + GroupSpec + G1 + G2 + _split_secret T==2 unit test` | `feat(slip39): v0.13.0 P1c-E.2 GREEN — driver impl` |

Plus reviewer-report commits:
- P1c-E.1 post-GREEN R1 review (matches the LOCK round 1 pattern from P1c-D).
- P1c-E.2 pre-GREEN test-design review (R0-style; with explicit variant-mapping verification per R0 N2).
- P1c-E.2 post-GREEN R1 review.

P1c-E.3 G6 hygiene pass (mlock pinning + lint rows) is a separate session after P1c-E.2 LOCKs.

## §6 Risk areas

1. **Digest verification** (combine §3.5): the `(4-byte HMAC || (n-4)-byte R)` structure is unusual; bug-prone in offset bookkeeping. Mitigation: §3.3's R0 C1 unit test pins the order before G1 runs.
2. **Group/member nesting in split** (§3.2): two levels of Shamir, easy to mix up the indices or pass the wrong threshold to the inner loop. Mitigation: §4.2 G2 round-trip exercises this end-to-end.
3. **Ext=1 vectors (#42–45)**: depend on §2.2 feistel fix landing first; P1c-E.1 must complete before P1c-E.2.
4. **G1 negative-variant mapping** (§4.1 table): R0 I3 + I2 + N2 folds settle the previously-ambiguous rows; pre-GREEN test-design review on P1c-E.2 RED verifies the table.
5. **Threshold-1 special case**: split + combine both special-case `T == 1` per python ref; ensure the code path SKIPS the digest path (the digest isn't computed for T == 1). The R0 C1 unit test pins the T==2 boundary; a separate T==1 test pins the no-digest path.

## §7 Open questions — RESOLVED at R0

R0 settled all five open questions (see `design/agent-reports/v0_13_0-slip39-driver-plan-r0.md`). Decisions baked into this revision:

1. `slip39_split` accepts `extendable: bool` directly. CLI hardcodes `false` for v0.13.0; `slip39-cli-extendable-flag` FOLLOWUP for v0.14.
2. All 5 ambiguous vectors pinned (table §4.1).
3. Feistel fully replaces — no legacy wrapper.
4. G6 hygiene deferred to P1c-E.3 (separate session); P1c-E.2 LOCK criteria explicitly accept current lint-row count.
5. xprv derivation REQUIRED at G1 LOCK; `bitcoin = "0.32"` already in Cargo.toml.

## §8 New `Slip39Error` variants (R0 I1 + I2 + I3 + §3.4 step 1)

P1c-E.1 introduces 5 new variants to `slip39::error::Slip39Error`:

```rust
/// Empty share list passed to `slip39_combine`. (R0 §3.4 step 1.)
EmptyShares,

/// Per-share value-length validation at combine entry: not in {16, 20, 24, 28, 32}. (R0 I2; vector #40.)
InvalidShareValueLength { share_idx: usize, got: usize },

/// Cross-share value-length divergence at combine. (R0 I1.)
ShareValueLengthMismatch,

/// Cross-share extendable-bit divergence at combine. (R0 I1.)
ExtendableMismatch,

/// Parse-time refusal: `group_count < group_threshold` on a single share. (R0 I3; vectors #10, #29.)
GroupThresholdExceedsCount { share_idx: usize, threshold: u8, count: u8 },
```

This grows `Slip39Error` from 16 variants (post-P1c-D) to 21. **SPEC §2.5 mirror update is pending** — covers 5 new refusal classes; current SPEC §2.5 has 18 rows. Defer the SPEC §2.5 row additions to a paired commit at P1c-E.1 LOCK, OR roll into PE rollup. **Open decision for the user to make at the next check-in.**

## §9 Verification gates

**P1c-E.1 LOCK criteria:**
- 23 P1a math tests (existing) + new `interpolate_at` tests (≥ 5 new, covering x=0, x=255, x=254, and non-power-of-2 x values) all pass.
- 19 P1b feistel tests (existing, now with `, false` appended) + new ext=true round-trip tests (≥ 3) all pass.
- 20 P1c-A error tests (existing) + new variant constructibility tests (≥ 5) all pass.
- 11 P1c-B + 15 P1c-C + 10 P1c-D tests (existing) — no regressions.
- New parse-time `GroupThresholdExceedsCount` test in `lib_slip39_share.rs` passes.
- Clippy `--all-targets -- -D warnings` clean.

**P1c-E.2 LOCK criteria:**
- All 45 G1 vectors pass (15 positive byte-equal + xprv match; 30 negative with correct variant per §4.1 table).
- G2 default property tests pass (200 trials).
- `_split_secret(T=2, N=3, ...)` unit test passes.
- `lint_zeroize_discipline.rs` loose-bound check passes WITHOUT new pin rows (per R0 Q4 augmentation).
- Full project `cargo test --tests` clean.
- Clippy clean.

**Out of P1c-E scope:**
- G6 hygiene pass at P1c-E.3.
- G2 extensive 2000-trial run (CI-gated, opt-in).
- P2 CLI surface.
- P3 manual chapter.
- Tag `mnemonic-toolkit-v0.13.0`.
