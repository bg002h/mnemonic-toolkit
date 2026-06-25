# Word-Card P1 — per-phase R0 review (round 1)

- **Phase:** P1 — `wc-codec` value-engine foundation (field + BIP-39 symbol map + 8↔11 regroup + stripe padding).
- **Branch:** `feat/wc-p1-scaffold` @ `a847b10c` (parent `master@84be2d82` — reconciled: master advanced to `84be2d82` after the session-start `60af98dd` snapshot, and `84be2d82` is an ancestor of current master; the branch is a clean single-commit child).
- **Reviewer:** opus architect, adversarial. Math re-derived independently (not from the implementer's report).
- **Date:** 2026-06-25.
- **Gate:** 0 Critical / 0 Important required to merge P1 and start P2.

---

## Verdict

**GREEN — 0 Critical / 0 Important.**

P1 is a clean, faithful, well-tested foundation. The frozen field constants are
mathematically correct and independently verified; the 8↔11 regroup is bit-exact, lossless,
and panic-free on its error paths; the wordmap is a true single-source-of-truth over `bip39`
English; padding is correct. The full package suite is GREEN (28 tests), clippy `-D warnings`
is clean, `cargo fmt --check` is clean, the workspace builds at root, and the diff is exactly
the 12 expected files plus a two-line workspace-membership edit — **no `cargo fmt --all`
collateral** (mlock.rs untouched, no other crate reformatted), and the toolkit crate does
**not** yet depend on `wc-codec` (per plan §7: not until P6). Three Minor/Nit items below,
all genuinely cosmetic — none blocks the gate. **P1 may merge; P2 may start.**

---

## Critical

None.

---

## Important

None.

---

## Minor / Nit

### N1 — `RegroupError` variants are not alphabetical (`regroup.rs:19,26,34`)
Declaration order is `NotEnoughBits` (19) → `SymbolOutOfRange` (26) → `NonZeroPad` (34).
Alphabetical would be `NonZeroPad`, `NotEnoughBits`, `SymbolOutOfRange`. The `Display` arm
order (`regroup.rs:40,47,50`) matches the declaration, so it is at least *internally
consistent*. Severity is **Nit, not Important**: the CLAUDE.md alphabetical convention is
written for `enum ToolkitError` and its exhaustive match blocks (the merge-conflict-magnet
case); plan §6.1 mandates alphabetical specifically for the **public `WcError`** that P4/P6
introduce, and is silent on this crate-local helper enum. Variants are scoped sensibly
(all three are genuine `symbols_to_bits` decode-failure modes, with structured fields that
aid diagnosis). **Concrete fold (optional, recommend for future-proofing):** reorder the enum
declaration and the `Display` match to `NonZeroPad`, `NotEnoughBits`, `SymbolOutOfRange` so it
sets the house style before `WcError` lands and absorbs `RegroupError`. Defer-acceptable.

### N2 — `sha2` and `proptest` deps declared a phase early (`crates/wc-codec/Cargo.toml:16,19`)
Confirmed by grep: **neither `sha2`/`Sha256` nor `proptest` is referenced anywhere in P1
`src/` or `tests/`** (`sha2` is for the P4 integrity tag; `proptest` for later property
tests). The doc-comments honestly say so ("used from P4 onward"). cargo/clippy do not warn,
and **`cargo machete`/`udeps` are not run in this repo's CI** (verified: no `machete`/`udeps`
reference under `.github/`), so there is no automated gate this trips today. `cargo-machete` is
not installed locally to demonstrate the flag. Severity **Nit**: declaring a phase early is a
defensible convenience (avoids a Cargo.toml churn-commit in P4) and the pins are deliberately
matched to the toolkit's. **Concrete fold (optional):** either (a) move `sha2` to P4 and
`proptest` to the first phase that uses a property test, or (b) leave as-is and add a one-line
`# unused until P4 (integrity tag) / Pn (proptests)` rationale — the existing comment on
`sha2` already does this; mirror it on `proptest`. No action required for GREEN.

### N3 — `pad_payload_to` panics on `target < input.len()` (`pad.rs:21-25`)
The task asked whether this should return a `Result`. **Assessment: panic is acceptable for
this frozen internal helper — Nit, not a finding.** Rationale: the contract (pad to the
*array-wide max*, which is by construction ≥ every member length) makes `target < len` a
caller programming error, not a runtime/recoverable condition; the panic message is precise
(`pad.rs:23`), there is no untrusted-input path into it (the value-engine never feeds
attacker-controlled lengths here — the array max is computed from the codec's own payloads),
and it is explicitly KAT-locked as a panic (`tests/pad.rs:38-42`, `#[should_panic]`). This
mirrors `slice` indexing / `Vec::resize` semantics. **Recommendation:** keep the panic; if P5
ever exposes `pad_payload_to` to a length derived from decoded/untrusted header geometry,
revisit then (a `debug_assert!` + saturating behavior, or a `Result`, would be the move). No
change for P1.

---

## Suite results

Run in the worktree `/scratch/.../agent-a7d27e936aae75a5a`:

- `cargo test -p wc-codec` — **28 passed, 0 failed, 0 ignored** across 4 integration test
  files + lib + doctests:
  - `tests/field.rs`: **10/10** (`frozen_constants`, `alpha_is_primitive`, `alpha_full_orbit`,
    `inverse_all_nonzero`, `mul_identity_and_zero`, `mul_commutative_sample`,
    `add_is_xor_and_self_inverse`, `distributivity_sample`, `pow_matches_iterated_mul`,
    `results_stay_in_range`).
  - `tests/regroup.rs`: **8/8** (KAT `[0xB5,0x2A]→[1449,640]`, byte-aligned round-trip 0..40,
    bit-precise non-multiples, non-zero-pad rejection, empty, single-partial-symbol,
    not-enough-bits, out-of-range).
  - `tests/wordmap.rs`: **5/5** (count, all-2048 round-trip, equals-bip39-English, rejects
    OOR/non-words/capitalized, known anchors abandon/zoo at 0/2047).
  - `tests/pad.rs`: **5/5** (incl. `#[should_panic]` for target<input).
  - lib unit / doctests: 0 (none authored — expected at P1).
- `cargo clippy -p wc-codec --all-targets -- -D warnings` — **clean (Finished, no warnings)**.
- `cargo fmt -p wc-codec --check` — **clean (exit 0)**.
- `cargo build` at workspace root — **clean (Finished)**.
- Diff integrity: `git diff master..feat/wc-p1-scaffold` touches exactly the 12 `wc-codec`
  files + `Cargo.toml` (one-line `members += "crates/wc-codec"`) + `Cargo.lock` (one additive
  `wc-codec` node: bip39/proptest/sha2). **`crates/mnemonic-toolkit/src/mlock.rs` NOT in the
  diff; no other crate reformatted.** Toolkit `Cargo.toml` has **no** `wc-codec` dep (correct
  for P1). `wc-codec` builds standalone (`cargo build -p wc-codec`) — the plan §2
  extractability claim holds (BIP-39 `English` is in `bip39`'s default features, so no reliance
  on the toolkit's `all-languages` feature unification).
- Src hygiene scan: **no** `panic!`/`unwrap`/`expect`/`todo!`/`unimplemented!`/`dbg!`/
  `TODO`/`FIXME` in `crates/wc-codec/src/` (the sole `assert!` is the documented `pad`
  contract). No commented-out code.

---

## Field-math independent check

All re-derived from scratch in Python (independent carry-less multiply with reduce-on-bit-11,
plus a second *naive* full-product-then-reduce multiplier as a cross-oracle — NOT the
implementer's algorithm). Scripts: `scratchpad/r0_p1_verify.py`,
`scratchpad/r0_p1_regroup_adversarial.py`.

1. **Primitivity of `0x805` / order of `α=x=0x002`.** Brute-forced `ord(α)` by iterated
   multiply — first return to `1` is at **exactly 2047** (= 2¹¹−1), so `0x805` is **primitive**
   (not merely irreducible) and `α=x` is a generator. The KAT's logic is sound:
   `2047 = 23·89`, proper divisors are `{1,23,89}`, so `α^2047=1 ∧ α^23≠1 ∧ α^89≠1` ⇒ order is
   exactly 2047 (`field.rs:14-16` / `tests/field.rs:21-26`). Implementer's `0x802` (=`x¹¹+x+1`)
   mutation claim **sane**: my brute force finds `x` never returns to 1 within 2048 steps under
   `0x802` (it is not primitive), so `alpha_full_orbit`/`alpha_is_primitive` genuinely go RED —
   the KATs are **non-vacuous**.
2. **Full orbit.** `α^0..α^2046` independently hits all 2047 non-zero elements exactly once;
   `0` never appears; wraps to `1` at 2047. Matches `alpha_full_orbit`.
3. **`mul` reduction (Russian-peasant, reduce-on-bit-10-carry by `^=0x805`).** **20,000 random
   products** agree with the independent naive poly-mul-then-reduce oracle. Spot: `mul(2,1024)`
   = `x·x¹⁰` = `x¹¹` ≡ `x²+1` = `0b101` = **5** ✓. `mul(a,0)=0` for **all** 2048 `a` ✓. The
   `& ELEM_MASK` after reduce and the `a & 0x0400` (bit-10) carry detection are correct — the
   shift carries bit-10 into the (virtual) bit-11 `x¹¹` term, which `^=0x805` folds to `x²+1`.
4. **`inv` (Fermat `a^2046`).** `field.rs:109` computes `pow(a, MULTIPLICATIVE_ORDER-1)` =
   `pow(a, 2046)`; `2047-1=2046` is safe u16 (no underflow). Verified `a·inv(a)=1` for **all
   2047** non-zero elements; `inv(0)=None`. Matches `inverse_all_nonzero`.
5. **Distributivity / identity / range.** `a·(b+c)=ab+ac` holds on the sample; `1` is the
   identity; results stay in 11 bits. Confirmed.

**Regroup (independent re-implementation, exhaustive probe):**

6. **MSB-first KAT.** `[0xB5,0x2A]` = bits `1011010100101010`; top 11 = `10110101001` =
   **1449**; next 5 = `01010`, low-padded `00000` (+1 final pad bit) → `01010000000` =
   **640**. Matches `tests/regroup.rs:13` by hand. Partial KAT `0xA0/3bits` → `101` low-padded
   → `0b101_0000_0000` = **1280** ✓.
7. **Losslessness incl. non-byte/non-11 boundaries (md1 case).** Exhaustive round-trip over
   **every `total_bits` in 0..512** against a 64-byte buffer: **0 mismatches**; byte-aligned
   lengths recover the exact source prefix; the re-encode invariant holds at all
   non-multiples. No off-by-one in the final partial symbol.
8. **Adversarial edge — `(1u16<<take) as u8` mask when `take==8`.** This *looks* like a bug
   (`256 as u8 == 0`), but `wrapping_sub(1)` then yields `0xFF` — the **correct** 8-bit mask.
   Verified by re-deriving the exact Rust truncation/wrap semantics. Benign.
9. **Decode rejection without panic.** `symbols_to_bits` rejects non-zero trailing pad
   (verified at **all 181** non-multiple-of-11 lengths by flipping the last symbol's low pad
   bit), too-few-bits (`NotEnoughBits`), and out-of-range symbols (`SymbolOutOfRange`) — each
   returns an `Err`, **never panics**. `debug_assert!` in `bits_to_symbols` (`regroup.rs:73`)
   is a release no-op and guards only a documented caller-contract (`total_bits ≤ 8·len`), not
   a decode path.

**wordmap:** `symbol_to_word`/`word_to_symbol` are thin wrappers over
`bip39::Language::English.{word_list,find_word}`, so symbol == index by construction;
`tests/wordmap.rs` asserts the all-2048 round-trip AND position-for-position equality against
the canonical list, plus anchors (0=`abandon`, 2047=`zoo`), OOR rejection, and
case-sensitivity. Correct.

---

### Bottom line

**GREEN (0C/0I).** The three Nits (N1 enum ordering, N2 early deps, N3 pad panic) are
cosmetic and explicitly deferrable; I recommend folding N1 opportunistically when `WcError`
lands in P4 (so `RegroupError` is already in house style) and mirroring N2's `sha2` rationale
comment onto `proptest`, but **neither gates this phase**. P1 clears the per-phase R0 gate —
merge `feat/wc-p1-scaffold` and proceed to P2 (systematic evaluation-form RS).
