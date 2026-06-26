# Word-Card P4 — per-phase R0 review (round 1)

- **Phase:** P4 — integration pipeline (integrity tag + GEOM header + fixed-`U`
  ledger + stop-sign + full encode/decode). Plan §3, §4.1–4.5, §5, §7 P4.
- **Branch:** `feat/wc-p4-pipeline` @ `8d29112a` (parent `master@1175353c`, NOT merged).
- **Reviewer:** opus architect (independent adversarial; wrote own fuzz harnesses,
  ran ≥570k decode attempts, then discarded the harnesses — branch left clean).
- **Files:** `crates/wc-codec/src/pipeline.rs` (983), `src/lib.rs` (rewritten public
  API + `WcError`), `tests/pipeline.rs` (23 KATs), reorders in `regroup.rs`/`rs.rs`.

## VERDICT: GREEN — 0 Critical / 0 Important

The funds-safety floor (**never return a wrong payload**) is verified, independently
of the implementer's tests, across **>570,000 adversarial decode attempts with ZERO
wrong-payload escapes** — including the forced cross-codeword RS-miscorrection class,
thin-parity (m∈{0,1,2}) deletions, indels at every position, and tail-truncation with
stray markers. The integrity tag demonstrably fires (5,780/6,000 and 76,555/80,000
forced-toward-B splices REFUSED; 0 silently accepted as the wrong payload). All gates
pass clean. The diff is `wc-codec`-only. Nothing blocks merging P4 or starting P5.

Findings are limited to Minor/Nit (documentation tightness + one permissive test
branch). None gate the phase.

---

## Critical

**NONE.**

The never-wrong-payload guarantee — the whole feature's safety floor — was attacked
directly and held:

- **Forced cross-codeword miscorrection (the headline Critical-class probe).** Two
  payloads A,B of identical geometry; splice strictly-more-than-half of the differing
  interleave words from B into A's frame, forcing the received word closer to B's
  codeword (a genuine within-budget RS miscorrection away from the true A). Result
  over 6,000 + 80,000 runs (varied m, t): **0 wrong payloads; the splice NEVER came
  out as B** (220 + 3,445 returned exact A; the remaining 5,780 + 76,555 were REFUSED
  by the SHA-256 tag — direct proof the tag catches the cross-codeword landing).
  Reproduces the implementer's `miscorrection_forced_toward_wrong_codeword_refused`
  KAT and confirms its property at scale.
- The tag check in `rs_decode_and_check` (lib path `pipeline.rs:898–902`) recomputes
  `SHA-256` over the *returned* canonical payload and compares to the *recovered* tag
  bits; a self-consistent (payload, tag) miscorrection survives only at `≤ 2⁻ᵗ`
  (t≥33). There is no code path that returns a payload bypassing this check. The
  `NonZeroPad` regroup assertion is an additional independent net on the final symbol.

## Important

**NONE.**

Items I specifically tried to escalate to Important and could not:

- **Region-bounding soundness (item 8).** The indel-aware re-anchor off the tail
  stop-sign marker + creation-total `m` was probed with an indel at *every* position
  (header, ledger, both region boundaries, parity boundary, stop-sign) across 5,490
  cases: 0 panics, 0 wrong. A corrupted stop-sign *count* (marker intact) can mis-
  derive `m` → mis-slice interleave vs parity, but that only ever converts a
  recoverable card into a refusal (the tag/sync catch the mis-slice) — an
  availability cost, never a wrong payload. Not Important.
- **Single-deletion-via-tag (item 2).** Swept a single deletion at every data
  position × 50 seeds (3,300 positions): 3,006 recovered exactly, 294 refused
  (deleted-checkpoint / unlocalizable positions), **0 wrong**. The candidate-equal-
  payload "not ambiguous" rule (`pipeline.rs:820–824`) is sound: it refuses only when
  two candidates yield *different* tag-passing payloads (two valid pre-images at
  `≤ 2⁻ᵗ`); identical-payload candidates from different reinsert slots are correctly
  not treated as ambiguity.
- **header-CRC choice (item 5).** The field-primitive `x¹¹+x²+1` as the CRC-11
  generator: empirically characterized — **single-word substitution: 0 misses /
  199,880** (the realistic engraving/misread error is ALWAYS caught, because a one-
  word change is a burst confined to one 11-bit lane ≤ the CRC degree); **all 1-bit
  flips: 0/88,000; all 2-bit flips: 0/1,892,000 (exhaustive over the 44-bit header)**;
  two-word random substitution misses at ~2⁻¹¹ (0.0005), as expected for any CRC-11.
  `(x+1)` is not a factor, so odd-weight ≥3 errors are not universally detected — but
  a standard CRC-11 would *not* improve the dominant single-word / 1–2-bit cases
  (already perfect) and a header-CRC miss cannot alone produce a wrong payload (it
  must *additionally* clear the `2⁻ᵗ` integrity tag). Acceptable; not a finding.
- **Truncation / false-positive (item 4).** 80,000 clean round-trips: **0 false
  truncations**, 0 mismatches. Deliberate-stop is correctly NOT flagged (the front
  ledger is read only at fixed positions; the stop-sign is validated only at the tail
  — the 2⁻¹¹/position scanning false-positive the implementer fixed is confirmed
  gone). Lost-tail (incl. tails that happen to carry the `0b1111` marker, 39,000
  runs) flags `truncated` or refuses — 0 wrong.
- **Append-only (item 3).** 5,000 random m1<m2 upgrade pairs: H0+GEOM byte-identical,
  interleave (K′ message body) byte-identical, parity prefix `[..m1]` byte-identical,
  both tiers decode the same payload. The ledger genuinely lives OUTSIDE the RS
  codeword (`pipeline.rs:26–40, 607–618`) — filling a slot never alters any
  RS/parity word.
- **No-panic (item 7).** Empty, all-`abandon`, all-stop-marker, 1,500-word random,
  and a doubled card (length > field cap) all return cleanly. `WcError` /
  `RegroupError` / `RsError` / `SyncError` are all alphabetical (verified
  programmatically); the regroup/rs reorder broke no caller (clippy + build clean).

## Minor / Nit

1. **(Nit) `miscorrection_forced_toward_wrong_codeword_refused` Ok-branch is weaker
   than reality.** The KAT accepts `d.payload == pa || d.payload == pb`. My 86,000-run
   reproduction shows the forced-toward-B splice *never* actually yields B (the tag
   refuses every cross-codeword landing; only exact-A or refuse occur). The `|| pb`
   makes the assertion looser than the implementation's true behavior. Harmless (the
   funds-safety property it guards — never a *third* payload — holds), but the test
   could be tightened to `== pa` for a sharper regression guard. P5+/no rush.
2. **(Nit) Header-CRC odd-weight detection — worth a one-line doc.** `CRC11_POLY`
   (`pipeline.rs:84`) already flags the field-primitive reuse; consider adding that
   single-word and all 1–2-bit header errors are *exhaustively* detected, odd-weight
   ≥3 at `2⁻¹¹`, and the integrity tag is the real backstop — so future readers don't
   re-litigate the (x+1)-factor question. Documentation only.
3. **(Nit) TDD provenance is unverifiable from the diff.** Impl + the 23 KATs landed
   in one squashed commit `8d29112a`, so test-before-impl can't be confirmed from
   history (consistent with the prompt's note that impl signatures preceded the full
   test file). Mitigating evidence the tests are NOT fitted-to-pass: the checked-in
   `pipeline.proptest-regressions` seed (`len=71,m=11,u=3,t=37,trim=3`) is a *real*
   past failure the property test caught and that was then fixed. No action.

---

## Suite results (gates — all GREEN)

- `cargo build -p wc-codec` — clean.
- `cargo test -p wc-codec` (full package) — **86 pass / 0 fail** (10 field + 5 pad +
  23 pipeline + 8 regroup + 12 rs + 23 sync + 5 wordmap; 0 lib-unit + 0 doc).
- `cargo test -p wc-codec --test pipeline` @ `PROPTEST_CASES=4000` — **23 pass / 0
  fail** (164.9s).
- `cargo clippy -p wc-codec --all-targets -- -D warnings` — clean (0 warnings).
- `cargo fmt -p wc-codec --check` — clean.
- Diff scope: **`wc-codec`-only** (lib.rs, pipeline.rs, regroup.rs, rs.rs,
  pipeline.rs tests, proptest-regressions). No `mlock.rs`, no cross-crate, no
  `fmt --all` collateral. Enum reorders (regroup/rs) confined to wc-codec, callers
  compile.

## Independent safety-fuzz (harnesses written by the reviewer, then discarded)

All harnesses used a deterministic xorshift PRNG, ran in `--release`, and assert
**WRONG == 0** (a single wrong-payload decode would be a Critical). Branch left clean.

| Harness | Cases | Result (WRONG = wrong payload returned) |
|---|---|---|
| never-wrong-payload (random kind/payload/bits/m/U/t × 10 corruption recipes: sub/del/ins, single+multi, in/over budget, bursts, mixed, truncation) | 200,000 | 87,112 correct · 112,888 refuse/err · **0 WRONG** |
| never-wrong-payload, 2nd master seed, heavier mixed corruption | 200,000 | 16,869 correct · 183,131 refuse · **0 WRONG** |
| **forced cross-codeword miscorrection** (splice >½ of B's differing interleave words into A) | 6,000 | 220 exact-A · **0 became B** · 5,780 **tag-REFUSED** · **0 WRONG** |
| **forced miscorrection at exactly ⌊m/2⌋+1**, varied m/t | 80,000 | 3,445 exact-A · **0 became B** · 76,555 **tag-REFUSED** · **0 WRONG** |
| single-deletion sweep (every data position × 50 seeds) | 3,300 | 3,006 recovered · 294 refuse · **0 WRONG** |
| thin-parity deletion (m∈{0,1,2}) | 3,000 | 1,546 recovered · 1,454 refuse · **0 WRONG** |
| indel at EVERY position (header/ledger/boundaries/parity/stop) | 5,490 | 0 panic · **0 WRONG** |
| tail-truncation w/ stray markers | 39,000 | flag/refuse · **0 WRONG** |
| clean round-trip (false-truncation hunt) | 80,000 | 80,000 exact · **0 false-truncation** · 0 mismatch |
| append-only invariant (random m1<m2, U) | 5,000 | K′ + parity-prefix byte-identical, both decode equal |
| cold-decode U∈{1,3}, K=1..400 | — | exact |
| degenerate/malformed (empty, all-marker, 1500-word random, doubled-over-cap) | — | no panic |
| **TOTAL adversarial decode attempts** | **>570,000** | **0 wrong-payload escapes** |

**The miscorrection class specifically:** the integrity tag is observed firing on
**82,335** forced cross-codeword landings (5,780 + 76,555 REFUSED) and let **0**
through as the wrong payload — the net works exactly as the plan's funds-safety
argument (`≤ 2⁻ᵗ` residual) requires.

---

### Bottom line

P4 is **GREEN (0C/0I)**. The integration pipeline correctly composes P1–P3 behind a
non-linear SHA-256 integrity tag that provably prevents any wrong payload from
decoding, the ledger-outside-the-RS-codeword decision preserves append-only, the
positional GEOM cold-decode is deterministic across `U` fills, truncation is signalled
without false positives, and no malformed input panics. The three Nits are
documentation/test-tightness only and do not gate. **Merge P4; proceed to P5.**
