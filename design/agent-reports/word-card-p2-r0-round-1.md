# Word-Card P2 — per-phase R0 review (round 1)

**Phase:** P2 — systematic evaluation-form Reed–Solomon engine (`wc-codec`).
**Branch:** `feat/wc-p2-rs` @ `62dee745` (parent `master@a782fb1a`, NOT merged).
**Reviewer:** opus architect (independent adversarial R0).
**Date:** 2026-06-25.
**Scope reviewed:** `crates/wc-codec/src/rs.rs`, `src/poly.rs`, `src/lib.rs`,
`tests/rs.rs` (the entire P2 diff; 4 files, +944/−5).
**Contract:** Plan §3 (frozen RS construction), §4.1, §7 P2
(`design/IMPLEMENTATION_PLAN_word_card_encoding.md`).

---

## Verdict

**GREEN — 0 Critical / 0 Important.**

The decoder is correct. I did not trust the implementer's tests; I wrote and ran my
own independent randomized + exhaustive verification (≈275k decode trials plus an
independent cross-language Lagrange oracle) and the implementation meets the RS
distance guarantee exactly: **every** within-budget `(t,s)` split decodes exactly,
**zero** silent miscorrections within budget, **zero** panics on any input I threw at
it, and beyond budget it refuses ~99.8% of the time (returning to the caller for the
P4 tag to catch the rare residual, exactly as the plan documents). Systematic
placement, `βⱼ=α^j` from the frozen `α=0x002`, and parity `= P(β_{k+i})` are byte-for-
byte confirmed against an independent oracle. The phase clears the gate. P2 may merge
and P3 may start. Two Nits below are non-blocking.

---

## Critical

None.

---

## Important

None.

---

## Minor / Nit

- **N1 (Nit) — `RsError` variant ordering: `Uncorrectable` and `Underdetermined` are
  swapped vs strict alphabetical.** `rs.rs:64,72` order them `Underdetermined` then
  `Uncorrectable`. Alphabetically `Uncorrectable` < `Underdetermined` (3rd char `c`<`d`),
  so the strict-alphabetical convention (`CLAUDE.md`) wants them swapped. The `Display`
  match (`rs.rs:97–106`) uses the same swapped order, so it is *internally consistent* —
  this is purely a one-line cosmetic. Impact is nil for merge mechanics (a fresh enum,
  not interleaved with an unsorted legacy block). Recommend a one-line swap of the two
  variants + their two `Display` arms when convenient; not a gate.

- **N2 (Nit) — `divmod` tolerates a zero divisor only via `debug_assert!`; in
  release a zero divisor would mis-execute (line 118 `divisor.degree().unwrap()`
  panics if the degree-guard at 115 is bypassed).** I traced **all three** `divmod`
  call sites (`poly.rs:235` inside the `partial_gcd` loop whose guard ensures
  `r_cur.degree() ≥ deg_stop` so `r_cur` is non-zero; `rs.rs:272` guarded by the
  explicit `v.is_zero()` check at `rs.rs:269`) — a zero divisor is **unreachable** in
  the present call graph, so this is not a live defect. Flagging only because future
  phases (P3 sync may reuse `poly`) could add a caller; a cheap hardening would be to
  make `divmod` return `(zero, self)` (or the existing early-return shape) on a zero
  divisor instead of relying on `debug_assert`. Optional.

Neither Nit blocks the gate. No other findings.

---

## Suite results

Run in the live worktree `…/.claude/worktrees/agent-a5b059e61a2fb81ea`.

- `cargo test -p wc-codec` — **GREEN, 40/40** across all targets:
  - `tests/field.rs` 10, `tests/pad.rs` 5, `tests/regroup.rs` 8, **`tests/rs.rs` 12**
    (incl. 4 proptests: `correct_floor_m_over_2_errors` 400 cases,
    `recover_up_to_m_erasures` 300, `mixed_errors_and_erasures` 400,
    `beyond_budget_never_panics` 300), `tests/wordmap.rs` 5; unit 0; doc 0.
  - 0 failed, 0 ignored.
- `cargo clippy -p wc-codec --all-targets -- -D warnings` — **clean** (no warnings).
- `cargo fmt -p wc-codec --check` — **clean** (exit 0).
- **Diff scope** — `git diff master..HEAD --name-only` = exactly the 4 intended files
  (`src/lib.rs`, `src/poly.rs`, `src/rs.rs`, `tests/rs.rs`). **No `mlock.rs` / other-crate
  reformat, no `cargo fmt --all` collateral.** P1 `field.rs` / `regroup.rs` / `pad.rs` /
  `wordmap.rs` are **untouched** — the field is reused, not reimplemented (`rs.rs` 4×
  `field::`, `poly.rs` 13× `field::`; `Poly::add`/`Poly::mul` are polynomial-level ops
  that delegate to `field::`). `lib.rs` change is the expected `mod poly` (private) +
  `pub mod rs` + doc update.

---

## Independent decoder verification (reviewer-authored, then discarded)

I did **not** rely on `tests/rs.rs`. I wrote a separate test file
(`tests/zz_r0_independent.rs`) and two `examples/` harnesses with my own
splitmix64 PRNG, ran them in the worktree, then **deleted them** — the branch is
left clean (verified `git status --short` empty; only the 4 P2 files present).

**1. Within-budget exhaustive `(t,s)` split sweep — ALWAYS exact.**
For 14 `(k,m)` shapes (incl. canonical `(58,8)`, `(160,30)`, `(200,47)`, and edges
`(1,1)`,`(2,2)`), I swept **every** legal split `w = 2t+s` for `w ∈ 1..=m`, every
`t ∈ 0..=⌊w/2⌋`, 6 trials each (3,864 trials). Errors = random wrong symbols;
erasures = random garbage flagged known-bad. **Result: every trial decoded EXACTLY,
zero refusals, zero miscorrections.** This is the load-bearing guarantee — confirmed.

**2. No-silent-miscorrection census — 40,000 random within-budget trials.**
Random `k∈1..60`, `m∈0..40`, random within-budget `(t,s)`. **Zero miscorrections,
zero within-budget refusals.** Confirms the implementation actually achieves the RS
distance (a subtle Gao/puncture bug would surface here as a within-budget wrong-Ok).

**3. Tightest-boundary stress — 200,000 trials at `2t+s = m` exactly.**
A separate `--release` harness, random `t` split, `s = m−2t`. **All 200,000 exact,
zero refusals, zero miscorrections** at the hardest boundary (zero slack).

**4. Beyond budget `2t+s = m+1` — 30,000 trials.** **Never panicked.**
Census: `Err` 29,935 (99.78%) / `Ok-but-wrong` 48 (0.16%) / `Ok-and-right-by-luck`
17 (0.06%). The decoder refuses the overwhelming majority and only rarely returns a
wrong-but-valid codeword — precisely the residual the plan says is caught by the P4
integrity tag, **not** a "returns wrong-data-as-Ok within the correctable region"
defect. Healthy.

**5. `m+1` erasures (all-known-bad over-budget) — 5,000 trials.** Always
`Err(Uncorrectable)`, never a wrong Ok, never a panic. (Matches the early
`erasures.len() > m ⇒ Uncorrectable` short-circuit at `rs.rs:230`.)

**6. Append-only prefix — 2,000 random `(k, m_small ≤ m_big)`.**
`rs_parity(data,m_big)[..m_small] == rs_parity(data,m_small)` held universally,
incl. `m_small = 0`. This is a **real** RS prefix-extensibility property (β-sequence
is a fixed prefix), not a test artifact.

**7. §3 conformance vs an independent oracle.**
   - I wrote a naive O(n²) Lagrange interpolant + evaluator in a *separate* module
     using only the frozen `α=0x002` / `0x805` reduction, and compared its parity to
     `rs_parity` over 400 random `(k,m)` — **identical every time.**
   - **Cross-language check (Python):** an independent Python GF(2¹¹) + Lagrange
     oracle gives `rs_parity([5,100,2000], 2) = [1386, 1864]`; the Rust crate returns
     **exactly `[1386, 1864]`.** Also re-derived `α^2047=1, α^23=34≠1, α^89=322≠1`
     in Python (primitivity ⇒ order exactly 2047).
   - **Systematic:** `rs_codeword(data,m)[..k] == data` held in every case; codeword
     position `j` carries `P(βⱼ)`. Single-error correction (`t=1`, `m=2`) verified
     for the hand case at **every** position.

**8. Panic-safety fuzz — all return `Err`, never panic:** `k=0`/empty data (→ all-zero
parity, by the degree-`<0` zero-poly convention), `m=0` (+ `m=0` with an injected
error), both-empty, duplicate erasures `[1,1]`, unsorted `[3,1]`, out-of-range
`[len]` and `[usize::MAX]`, `parity_len = usize::MAX` and `5000` (→ `LengthExceedsField`),
`data_len > codeword.len()` and `= 10_000` (→ `Err`), symbol `2048`/`9999`/`5000`
in data/codeword (→ `SymbolOutOfRange`), oversize codeword `len = 2048` (→
`LengthExceedsField`), all positions erased `s = n`, and `s = 7 > m` (early refuse).

**9. `n_used == k` full-erasure-no-error edge — 3,000 trials.** Erase exactly `m`
positions, no errors ⇒ surviving points `= k` exactly, the interpolant **is** the
answer. **All exact.** The `deg_stop = (n_used+k)/2` integer-floor threshold and the
`n_used < k ⇒ Uncorrectable` guard (`rs.rs:247`) behave correctly at this boundary.

**Static panic-reachability audit.** I traced every `expect`/`unwrap` in `src`:
`poly.rs:119` (`inv(divisor.leading())`), `:190` (`master.coeffs.last()`), `:203`
(`inv(den))`), `:118` (`divisor.degree().unwrap()`). All `Poly` values flow through
`from_coeffs`/`constant`/`zero` (the only struct literals, both trimming), so any
non-empty `Poly` has a non-zero leading coeff ⇒ `inv(leading)` is `Some`; both
`interpolate` callers (`rs.rs:157,261`) pass distinct `βⱼ` (distinct `j < 2047`) ⇒
`den ≠ 0` and `master` non-empty. **Every `expect`/`unwrap` is unreachable given the
crate-internal call graph.** (The `divmod` zero-divisor case is N2 above — unreachable
today, hardening optional.)

---

### Bottom line

P2 is correct, well-scoped, clippy/fmt-clean, field-reusing (not -reimplementing),
and the cryptographic core (the decoder) is independently verified to meet the RS
distance guarantee with no silent miscorrection and no panic. **GREEN, 0C/0I.**
The two Nits (variant order, optional `divmod` hardening) are non-blocking and may be
folded opportunistically or in P3. The gate to merge P2 and start P3 is cleared.
