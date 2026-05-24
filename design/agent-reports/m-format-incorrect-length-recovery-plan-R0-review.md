# R0 Architect Review — IMPLEMENTATION_PLAN_m_format_incorrect_length_recovery.md

**Round:** R0 (implementation-plan gate; mandatory pre-code)
**Reviewer:** feature-dev:code-reviewer (opus)
**Date:** 2026-05-24
**Reviewed against:** branch `m-format-incorrect-length-recovery` = `origin/master 925f5ed`; ms-codec 0.2.0, mk-codec 0.3.1; clap 4.6.1.
**Persisted verbatim before fold, per CLAUDE.md.**

## Verdict: RED — 0 Critical / 2 Important / 4 Minor

The plan is API-accurate and citation-clean to an unusually high degree — every one of the ~20 source anchors checked is ACCURATE (no DRIFTED/WRONG). The two Important findings are design-completeness gaps, not compile-blockers: (I1) the multi-group `run()` exit-aggregation is unspecified and will produce wrong/lossy behavior on heterogeneous `--ms1 X --mk1 Y` invocations; (I2) the dedup-by-recovered contract, as worded, risks a false-Ambiguous when P1 and P2 recover the identical string with differing metadata, and no test guards it.

---

## Critical
None. All reused signatures compile against source; all trigger/exit/oracle plumbing checks out.

---

## Important

### I1 — Multi-group `run()` exit aggregation is unspecified; mid-loop `return` silently skips remaining groups
**Where:** plan §2.4 vs `cmd/repair.rs::run` lines 102-122.

`resolve_groups` returns `Vec<(CardKind, Vec<String>)>` and D35 supports heterogeneous multi-group invocations — `mnemonic repair --ms1 X --mk1 Y` yields TWO groups (`repair.rs:269-281`). Today `run()` loops all groups, aggregates `total_repairs`, returns `Ok(0|5)` once at line 122.

The plan's `return Ok(4)` (ambiguous) and `return Err(...)` (unrecoverable) short-circuit mid-loop. Consequences not addressed:
- `--ms1 <ambiguous> --mk1 <valid>`: group 1 returns `Ok(4)` → group 2 never emitted (today both always emit).
- Cross-group exit precedence undefined (recovered-5 vs unrecoverable-2 vs ambiguous-4 vs already-valid-0).
- D9 ms1 advisory (`run()` 118-120, post-loop) is bypassed by an early `return`, even though the ambiguous branch emits ms1 candidates.

**Fix:** specify aggregation. Do NOT `return` mid-loop for Ambiguous; process every group, emit each in-loop (fire per-group ms1 advisory inline OR rely on the post-loop site by not early-returning), compute one final exit by precedence `unrecoverable(2) > ambiguous(4) > recovered(5) > already-valid(0)`. (Hard-failure/unrecoverable as `Err` short-circuit is consistent with today's `?` behavior; the new divergence is the Ambiguous mid-loop return.) Add a multi-group integration test to §4.3.

### I2 — `dedup_by_recovered` must key on `recovered` only; derived `PartialEq` over all 4 fields yields false Ambiguous; no test covers P1∩P2 collision
**Where:** plan §2.1 line 61 (`PartialEq` over `recovered,indel_count,region,direction`), §3 Phase 1 ("sort by `recovered`, dedup"), §4.1 test 4.

P1 (prefix) and P2 (data) can both recover the SAME string with different `region`/`direction`. Under derived `PartialEq` a `Vec::dedup()` leaves both → `len()==2` → false `Ambiguous` → exit 4 on a unique recovery.

**Fix:** (a) `hits.sort_by(|a,b| a.recovered.cmp(&b.recovered)); hits.dedup_by(|a,b| a.recovered == b.recovered);` — keyed on `recovered` ONLY. (b) Tighten test 4 / Phase-3 Step 1(c) to use two hits with identical `recovered` but mismatched `region`/`direction` and assert collapse to `Unique` (otherwise the test vacuously passes).

---

## Minor

### M1 — `mk_codec::decode(&[&str])` self-corrects (t≤4); plan's "requires already-valid codewords" rationale is wrong (ordering still correct)
**Where:** plan §2.2 lines 132-133, §5 R3.
`decode` → `pipeline.rs:118` → `decode_string` (`bch.rs:650`) → `bch_correct_*` (`bch.rs:683-687`) runs full BM/Chien/Forney up to t=4. So the pre-solve-then-decode ordering is correct/necessary because `decode`'s built-in correction is UNGUARDED (would silently apply ≤4 substitutions, defeating the ⊆ rule) — not because decode requires clean input. `mk1_chunk_solve` enforces ⊆; handing decode the already-solved chunk means it sees 0 corrections. Fix the rationale wording; confirm against `bch.rs:683-687` in Phase 4.

### M2 — GUI `FlagKind` for `--max-indel` should be `Number{min:0,max:Static(4)}`, not `Text`
**Where:** plan §2.5 / Phase 6 Step 1.
schema_mirror compares flag-NAME sets only, so any kind passes the gate, but `gui-schema` emits `("number", None)` for a `value_parser!(u8)` flag. For correct GUI rendering the mirror should use `FlagKind::Number { min:0, max: NumberMax::Static(4) }` (`mnemonic-gui/src/schema/mod.rs:121`). State the kind in the plan.

### M3 — `mod indel;` insertion point (alphabetical) unspecified
`main.rs` declares modules alphabetically; `mod indel;` belongs between `mod friendly;` (line 15) and `mod language;` (line 16).

### M4 — t=4 ceiling citation is single-codec
Plan §0/§2.1 cites `ms-codec bch_decode.rs:416`. The mk1 path enforces t=4 in mk-codec's copy (`mk-codec/bch_decode.rs:566`). Both ACCURATE; note the dual location for the implementer.

---

## API / citation audit table

| # | Claim (plan) | Source | Status |
|---|---|---|---|
| 1 | `parse_chunk(chunk, chunk_index, kind) -> Result<(Vec<u8>, BchCode), RepairError>` `:511` | `repair.rs:511-515,578` | ACCURATE |
| 2 | `polymod_residue(hrp:&str, data:&[u8], target:u128, code:BchCode) -> u128` `:584` | `repair.rs:584` | ACCURATE |
| 3 | `encode_chunk(hrp:&str, data:&[u8]) -> String` `:595` | `repair.rs:595` | ACCURATE |
| 4 | `decode_regular_errors`/`decode_long_errors` → `Option<(Vec<usize>, Vec<u8>)>`, `^=` valid | `bch_decode.rs:520-548,94`; `repair.rs:642-659` | ACCURATE |
| 5 | `CardKind::target_residue(self, code) -> Option<u128>` `:72`, private | `repair.rs:72-81` | ACCURATE |
| 6 | `apply_ms_corrections(...) -> (String, Vec<(usize,char,char)>)` `:822`; `CorrectionDetail.position` | `repair.rs:822`; `ms decode.rs:90` (doc :88) | ACCURATE |
| 7 | `ms_codec::decode_with_correction(&str) -> Result<(Tag,Payload,Vec<CorrectionDetail>)>` `:188` | `ms decode.rs:188` | ACCURATE |
| 8 | `mk_codec::decode(&[&str]) -> Result<KeyCard>` `:114`; pre-solve-then-decode | `key_card.rs:114`; `pipeline.rs:118`; `bch.rs:650,683` | ACCURATE (rationale fix M1) |
| 9 | clap `default_value_t=0, value_parser!(u8).range(0..=4)` | Cargo.lock clap 4.6.1 | ACCURATE |
| 10a | `run` `:89-123`, `repair_card?` `:103`, exit `:122` | `cmd/repair.rs:89,103,122` | ACCURATE |
| 10b | advisory `:118-120` | `cmd/repair.rs:118-120` | ACCURATE |
| 10c | args `:36-72`, `--json` `:59`, json structs `:153-204` | `cmd/repair.rs` | ACCURATE |
| 10d | `RepairError` `:388-430`; `IndelUnrecoverable` slot after `HrpMismatch` | `repair.rs:388-430` | ACCURATE |
| 10e | `Repair(_) => 2` `:507`; `BadInput => 1` | `error.rs:507,464` | ACCURATE |
| 10f | GUI `REPAIR_FLAGS` `:1513-1561` | `mnemonic-gui schema/mnemonic.rs` | ACCURATE |
| 10g | `mod repair;` main.rs `:21` | `main.rs:21` | ACCURATE (M3) |
| 10h | `install.sh:32` pin | `scripts/install.sh:32` | ACCURATE |
| — | spec anchors (bch_decode.rs:416; consts.rs:33/42; repair.rs:559/406/414/690-708/700/982; inspect.rs:195) | verified | ACCURATE |
| — | README markers (both, `readme_version_current.rs`); GUI 0.21.1→0.21.2; toolkit 0.37.0→0.37.1 | verified | ACCURATE |

---

## Design-fidelity confirmations (hold up)
- **Trigger predicate (§1.7) sound both directions.** Too-short ms1 survives the length band, BCH-runs, `decode` rule-9 rejects (`ms decode.rs:29`) → `PostCorrectionDecodeFailed` via `Err(other)` (`repair.rs:810-813`) — in the trigger set. Substitution never produces false "already valid" (rule-9 keys on `s.len()`, unchanged by substitution).
- **Auto-fire untouched.** `try_repair_and_short_circuit` (`repair.rs:962-983`) calls `repair_card` with no `max_indel`; engine lives only in `run()`. `--max-indel 0` ⇒ `for j in 1..=0` zero iterations ⇒ no-op. Confirmed.
- **Pure-indel ⊆ rule + placeholder-collision** handled in both oracles. Confirmed.
- **mk1 per-chunk** + reassembly + >1-failing ⇒ Unrecoverable matches D8 atomicity. Sound.
- **`IndelUnrecoverable` alphabetical** placement; enum stays unsorted. Confirmed.
- **SemVer PATCH** for additive default-off flag matches precedent; the 4 FOLLOWUPs + flip + version + Cargo.lock + READMEs + install.sh + GUI/manual lockstep all present.
- **md1 refusal as `BadInput`/exit 1** acceptable.

## Required folds before Phase 0 (re-dispatch after fold)
1. I1 — multi-group exit-aggregation contract + multi-group test.
2. I2 — dedup keyed on `recovered`; load-bearing dedup test.
3. M1-M4 inline.
