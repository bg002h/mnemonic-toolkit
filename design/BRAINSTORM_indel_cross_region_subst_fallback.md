# BRAINSTORM / SPEC — indel v2: cross-region + indel+substitution + HrpMismatch-fallback

**Slugs:** `m-format-indel-cross-region-split`, `m-format-indel-plus-substitution`, `m-format-indel-hrpmismatch-suggestion-fallback` (combined — they share the recovery machinery and must not break each other).
**Date:** 2026-05-24
**Base SHA:** `master` = `a6987f4` (post-v0.37.2). All `:line` anchors below grep-verified at this SHA; re-grep at plan-doc lift time.
**Status:** brainstorm APPROVED by user → feeds the implementation plan-doc, which passes the **mandatory opus architect R0 (0C/0I) before any code**.
**SemVer:** PATCH → toolkit `v0.37.3`. New flag NAME `--max-subst` ⇒ mandatory GUI `schema_mirror` + manual mirror lockstep (paired GUI PR). cross-region + fallback are behavior-only.
**Scope:** toolkit-only (error-decoder + placeholder). The `m-format-indel-erasure-decode-extend-to-8` FOLLOWUP stays open (it would do substitution+reach better via codec erasure decoding — a cross-repo cycle; this is its error-decoder approximation).

---

## §0 Why combine the three

All three extend `mnemonic repair --max-indel` (shipped v0.37.1/.2). They touch three different layers of the SAME machinery, and interact through one shared resource (the per-chunk BCH t=4 capacity), so doing them together with explicit integration guarantees is safer than sequentially (one could silently break another):
- **cross-region-split** → the candidate generator (`indel.rs`).
- **indel+substitution** → the accept gate (the per-kind oracles in `repair.rs`).
- **HrpMismatch-fallback** → the error path (`cmd/repair.rs::run`).

## §1 Current state (post-v0.37.2)

- `recover_indel(input, hrp, max_indel, oracle)` (`indel.rs:61`) loops `for j in 1..=max_indel { collect_prefix; collect_data_delete; collect_data_insert }` — producers run **independently per j** (single-region; a candidate's edits are all-prefix OR all-data). `dedup_by_recovered` then maps count→`Unique/Ambiguous/Unrecoverable`.
- The per-kind oracle accept gate is **pure-indel**: `corrections ⊆ placeholders` (`repair.rs:889` Ms1; `:922` mk1_chunk_solve; `:962` md1_chunk_solve).
- `IndelCandidate` (`indel.rs:30`) = `{ recovered, indel_count, region, direction }`.
- `cmd/repair.rs::run` maps `Unique→Ok(5)`, `Ambiguous→Ok(4)`, `Unrecoverable→Err(RepairError::IndelUnrecoverable)→exit 2`, already-valid→0, via `indel_exit_code(ambiguous_seen, total_repairs)` (`repair.rs:1055`). `is_indel_trigger` includes `HrpMismatch` (so prefix indels engage); excludes `EmptyInput|UnsupportedCodeVariant|IndelUnrecoverable` (`repair.rs:1047`).

## §2 Unified budget model

Two orthogonal flags + one hard physical constraint:
- **`--max-indel N`** — insert/delete edits (prefix + data), `0..=4`, default 0 (unchanged).
- **`--max-subst E`** — substitution errors (wrong-but-in-place chars), **`0..=4`, default 0** (NEW).
- **Hard constraint (the BCH ceiling):** per chunk the decoder corrects ≤ **t=4** total. In the recovery path the 4 are shared between **placeholders** (for too-short data indels — each is 1 BCH correction) and **substitutions**. So per candidate: `placeholders + substitutions ≤ 4`. `E` is therefore effectively clamped to `4 − placeholders` (full 4 only for pure too-long/prefix recovery where placeholders=0). `E=0` reproduces v0.37.2 pure-indel behavior byte-for-byte.

Note: a **correct-length** card with substitutions is handled by plain `repair` (BCH) already, no flags. `--max-subst` only enters on a wrong-length (indel) input that ALSO has substitutions.

## §3 cross-region (candidate generator) — `m-format-indel-cross-region-split`

Restructure `recover_indel` into a **two-level search**: for each prefix-region edit count `j_prefix ∈ 0..=min(N, prefix-window)`, produce a candidate `xx1`-prefix + residual data string, then run the data-region producers (delete/placeholder-insert) with budget `N − j_prefix`, validating the **assembled full string** via the oracle. `j_prefix=0` = today's data-only path; `j_data=0` = today's prefix-only path; both > 0 = the new cross-region case. This **subsumes** the current independent single-region producers (no separate path). Flagless (just makes the search complete within N).
Combinatorial cost: the prefix is 3 chars (window ~`2N+1`, tiny), so cross-region multiplies the data search by a small constant — feasible at the same N as v1.

## §4 substitution (accept gate) — `m-format-indel-plus-substitution`

Relax the per-kind oracle gate from `corrections ⊆ placeholders` to **`|corrections \ placeholders| ≤ E`** (the count of corrections at non-placeholder positions ≤ the substitution budget). The decoder already corrects up to t=4; this only changes whether a substitution-bearing result is ACCEPTED vs rejected. `IndelCandidate` gains **`subst_count: usize`** = `|corrections \ placeholders|` (0 for pure indel), so the output/exit layer can flag substitution-bearing results. The candidate set (decode count) is UNCHANGED by E — E is an accept-window relaxation, not more candidates.

## §5 Output: candidate-list + verify advisory

The printed list is provably tiny (~1 entry; the 65-bit checksum admits only ~`32⁻¹³` spurious accepts per candidate — see §9), so emit-ALL is cheap and human-scannable.
- **Emit all accepted candidates** (dedup by `recovered`), as today.
- **Verify advisory:** if **any** emitted candidate has `subst_count ≥ 1`, print a prominent stderr advisory: *"these are candidate recoveries, not confirmed corrections — derive an address from each and verify it controls your funds before trusting any; some may be false positives."* (Plus the existing ms1 secret-on-stdout advisory.)
- **`--json`:** per-candidate `subst_count`; top-level `confident: false` when any candidate is substitution-bearing (else true). Wire-shape NOT schema_mirror-gated → GUI self-updates (paired-PR).

## §6 Exit-code contract (architect-reviewed)

Reuses the existing families — **no new exit code**. Preserves the invariant **5 = safe to use as-is; 4 = verify it.**

| Outcome | Exit |
|---|---|
| input already valid (no recovery) | **0** |
| unique, **pure-indel** (subst_count=0) | **5** (trustworthy, unchanged) |
| **any candidate used ≥1 substitution** (1+ candidates) | **4** (verify-me) |
| ambiguous (≥2 distinct), pure-indel | **4** (unchanged) |
| unrecoverable within budget | **2** (`RepairError::IndelUnrecoverable`) |

Rationale: a substitution recovery is "a correction was applied **but it's a guess, not confident**" — exactly the exit-4 "no single trustworthy answer / verify yourself" family (`BundleMismatch` `error.rs:474`, `XpubSearchNoMatch` `:513`, the existing ambiguous-indel case). Forcing it to 5 would break the "5 ⇒ trust it" invariant every consumer relies on. **Wiring:** track `substitution_seen` alongside `ambiguous_seen` in `run()`; extend the helper to `indel_exit_code(ambiguous_seen, substitution_seen, total_repairs)` (one auditable place) → `if ambiguous_seen || substitution_seen { 4 } else if total_repairs==0 { 0 } else { 5 }`. Default path (`E=0`) never sets `substitution_seen` → outcome set `{0,5,4-ambiguous,2}` byte-identical to v0.37.2. Substitution-unique vs ambiguous differ only in advisory text + `--json`, not a third exit code.

## §7 HrpMismatch-fallback (error path) — `m-format-indel-hrpmismatch-suggestion-fallback`

Strictly downstream of all recovery: `recover_indel_card` runs the full search (including cross-region prefix recovery) first. Only on `Unrecoverable` does `run()` check the **originating** `repair_card` error — if it was `RepairError::HrpMismatch`, return that original error (carrying its Levenshtein-1 "did you mean 'mk'?" suggestion via its `Display`) instead of `IndelUnrecoverable`. A recoverable prefix typo recovers (no fallback); a genuine wrong-HRP fails the search → fallback → suggestion. Requires `run()` to preserve the original `e` (already in scope at the match arm) to test its variant.

## §8 Non-breaking guarantees (the integration contract)

**Structural safety net:** the search is exhaustive within budget, so the TRUE recovery is always a candidate. cross-region only *adds* candidates; substitution only *widens each candidate's accept window*. With dedup + emit-all, for any input that IS recoverable within budget, widening can only yield Unique or Ambiguous — never a silent wrong-Unique. Substitution-bearing results are exit-4 + advisory (never silently trusted). The residual "wrong single candidate on an *unrecoverable* input" is, by design, a labeled verify-me candidate (exit 4 + advisory), not a confident answer.

Pairwise:
1. **cross-region × substitution:** only shared resource is the per-chunk t=4, enforced as `placeholders + subst ≤ 4` per candidate. Both widen; both land in the emit-all list; FP stays bounded (§9).
2. **substitution × fallback:** a genuine wrong-HRP stays Unrecoverable (FP negligible) ⇒ fallback still fires; substitution can't suppress the suggestion in practice.
3. **cross-region × fallback:** fallback is downstream of ALL recovery (prefix included) ⇒ a recoverable prefix indel recovers before the fallback is consulted.

## §9 False-positive / table-size analysis

Two distinct quantities:
- **Decode attempts (compute, N-driven, E-independent):** ~`2·C(L,j)` per j (L≤108): ~220 (N=1), ~10⁴ (N=2), ~3×10⁵ (N=3), ~8×10⁶ (N=4). Runtime: instant → minutes at N=4. E does NOT change this (accept-gate only).
- **Accepted/printed table (tiny):** per-candidate spurious-accept ≈ `ball(d)/32¹³` (d = placeholders+E ≤ 4): d=1 ~8e-17 … d=4 ~7e-8. Aggregate = attempts × per-candidate: even at the extreme (N=4, E=4) expected spurious ≈ **~0.56** — i.e. the printed list is ~1 (the true recovery) ± a rare stray. Each extra correction-unit `d` erodes the floor ~×3000 (the `C(93,E)·31^E` free-position search ball for substitutions). The danger line (~1e-6 aggregate, confident-wrong-single-answer) is why the *confident* model would cap E=1 — but the **candidate-list + verify model (§5/§6) tolerates the wider window** because the rare stray is a labeled verify-me candidate the user checks, not a trusted answer.

## §10 CLI surface / SemVer / lockstep

- New flag `--max-subst <E>` on `repair`, `value_parser` range `0..=4`, default 0. (`--max-indel` unchanged.)
- **SemVer PATCH** (additive, default-off).
- **Mandatory lockstep:** new flag NAME ⇒ GUI `schema_mirror` (`mnemonic-gui/src/schema/mnemonic.rs` repair entry, `FlagKind::Number{min:0, max:NumberMax::Static(4)}`) + manual mirror (`docs/manual/src/40-cli-reference/41-mnemonic.md` flag row + prose + exit-4 row refinement: "…or a candidate required ≥1 substitution — verify before trusting"). Paired post-tag GUI PR (`v0.21.3`). The `--json` wire-shape (`subst_count`, `confident`) is NOT schema_mirror-gated → GUI self-updates.
- Engine is HRP-agnostic — ms1/mk1/md1 all benefit.

## §11 Integration test matrix

1. cross-region: 1 prefix indel + 1 data indel (N=2, E=0) → Unique, exit 5.
2. indel+subst: 1 data indel + 1 subst (N=1, E=1) → exit 4 + verify advisory; recovered correct.
3. **all three at once:** prefix indel + data indel + data subst within N=2,E=1 → recovered, exit 4.
4. **regression:** `--max-subst 0` at various N → byte-identical to v0.37.2 (pure-indel; exit 5 unique / 4 ambiguous / 2 unrecoverable).
5. substitution-bearing unique → exit **4** (not 5); pure-indel unique → exit 5.
6. over-budget (`placeholders + subst > 4`) → Unrecoverable.
7. genuine wrong-HRP + N=1,E=1 → HrpMismatch fallback suggestion (not IndelUnrecoverable).
8. widening → Ambiguous-not-wrong-Unique (construct a 2-distinct-candidate case via mock oracle).
9. `--json` carries `subst_count` + `confident:false` on a substitution recovery.
10. `--max-subst 5` rejected by clap.

## §12 Scope / deferred

- **In:** cross-region, indel+subst (`--max-subst`, candidate-list), HrpMismatch-fallback. Resolves all three FOLLOWUPs at ship.
- **Stays open:** `m-format-indel-erasure-decode-extend-to-8` (codec `decode_with_erasures` ⇒ `2e+s≤8` reach + lower FP; cross-repo). `m-format-indel-asymmetric-delete-budget` (larger too-long budget). This cycle is the toolkit-only error-decoder version.

## §13 Open items for R0

1. `subst_count` plumbing: the oracle's `validate` currently returns `Option<String>` (the recovered string only). To populate `subst_count`, it must also return the substitution count (or the engine computes it). Plan-doc to pick: change `validate` to return `Option<(String, usize)>`, or have the engine diff. Affects all four oracles (Ms1/Mk1/Md1 + the two `chunk_solve` helpers).
2. Exact `indel_exit_code` signature change + its test (`repair.rs:2088`).
3. The two-level cross-region search restructure of `recover_indel` (subsuming the current producers) — confirm `dedup_by_recovered` + Unique/Ambiguous mapping still hold; confirm `subst_count` doesn't enter dedup (dedup on `recovered` only).
4. The verify-advisory wording + where it fires (run() after emit, gated on any `subst_count ≥ 1`).
5. md is regular-only — cross-region + subst for md1 stays within its single regular code (no long).
6. Re-grep all `:line` anchors at plan-doc write time.
