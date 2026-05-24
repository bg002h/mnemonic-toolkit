# R1 Architect Review (post-fold) — IMPLEMENTATION_PLAN_m_format_incorrect_length_recovery.md

**Round:** R1 (implementation-plan gate; mandatory pre-code; folds of R0's 2I/4M applied)
**Reviewer:** feature-dev:code-reviewer (opus)
**Date:** 2026-05-24
**Reviewed against:** `origin/master 925f5ed`; ms-codec 0.2.0, mk-codec 0.3.1; clap 4.6.1. All `:line` anchors re-grepped.
**Persisted verbatim per CLAUDE.md.**

## Verdict: GREEN — 0 Critical / 0 Important

All four Minor (M1-M4) and both Important (I1, I2) R0 findings are RESOLVED against source. The folds introduced no Critical or Important contradictions. Four non-blocking Minors noted (none gate Phase 0; folded inline into the plan after this review).

---

## Fold verification

**I1 (multi-group `run()` exit aggregation) — RESOLVED.**
- Real `run()` structure confirmed: loop over `groups` (`cmd/repair.rs:102`), `total_repairs`/`any_ms1` accumulators (`:97-98`), `repair_card?` `:103`, post-loop ms1 advisory `:118-120`, final `Ok(if total_repairs==0 {0} else {5})` `:122`. §2.4 `match` replacement faithful.
- Heterogeneous multi-group premise confirmed: `resolve_groups -> Vec<(CardKind, Vec<String>)>` (`repair.rs:255,269-281`).
- No-skip fully specified: Ambiguous sets `ambiguous_seen=true` + "NO return — continue"; only Unrecoverable short-circuits via `Err` (matches today's `?`).
- `is_indel_trigger` set `{TooManyErrors|PostCorrectionDecodeFailed|UnparseableInput|ReservedInvalidLength}` valid subset of `RepairError` (`repair.rs:388-430`); non-triggers pass through. Matches §1.7.
- Precedence `2>4>5>0` specified; exit 4 = human-review family (`error.rs:513`); IndelUnrecoverable→2 (`error.rs:507`); md1 BadInput→1 (`error.rs:464`); `kind.hrp()` `pub` (`repair.rs:58`).
- §4.3 #22a exercises multi-group no-skip + precedence.

**I2 (dedup keyed on `recovered`) — RESOLVED.** §2.1 `sort_by(recovered)` + `dedup_by(|a,b| a.recovered==b.recovered)`; `sort_by` present and necessary (`dedup_by` removes only consecutive runs). Keys on `recovered` only, not the derived 4-field `PartialEq`. `IndelOutcome::Ambiguous` doc = "≥2 DISTINCT `recovered`". Phase 3 1(c) + §4.1 test 4 now load-bearing (fail under derived `PartialEq`).

**M1 (mk decode self-corrects unguarded) — RESOLVED.** `mk-codec/.../string_layer/bch.rs:683-687`: `decode_string` calls `bch_correct_*` UNGUARDED. Confirms `mk1_chunk_solve` must ⊆-gate-solve first, then hand `decode` a clean chunk. Ordering sound.

**M2 (GUI `FlagKind::Number`) — RESOLVED.** `FlagKind::Number { min: i64, max: NumberMax }` `mnemonic-gui/src/schema/mod.rs:121`; `NumberMax::Static(i64)` `:167`; `REPAIR_FLAGS` `mnemonic.rs:1513-1561`. `Number{min:0,max:Static(4)}` valid.

**M3 (`mod indel;` slot) — RESOLVED.** `main.rs:15`=`mod friendly;`, `:16`=`mod language;`; `friendly < indel < language`. Slot correct.

**M4 (dual-codec t=4) — RESOLVED.** `ms-codec/.../bch_decode.rs:416` and `mk-codec/.../string_layer/bch_decode.rs:566` both `if deg == 0 || deg > 4`. Confirmed.

---

## Regression check (R0-verified claims re-confirmed)
parse_chunk `:511` / polymod_residue `:584` / encode_chunk `:595` / repair_chunk_one atomic `:700`; CardKind::target_residue `:72`; decode_*_errors imported `:31`, `-> Option<(Vec<usize>,Vec<u8>)>`, `^=` loop `:642-658`; apply_ms_corrections `:822`; CorrectionDetail.position `decode.rs:90`; decode_with_correction `:188`; mk_codec::decode `key_card.rs:114`; clap 4.6.1 `.range(0..=4)` valid; §1.7 trigger (`PostCorrectionDecodeFailed` `repair.rs:810-813`); auto-fire untouched (`:962-983`, `--max-indel 0` → zero iterations); pure-indel ⊆ + placeholder-collision; IndelUnrecoverable alphabetical (after HrpMismatch, before TooManyErrors); install.sh:32 pin; SemVer PATCH + lockstep/FOLLOWUP set complete; `VALID_STR_LENGTHS=[50,56,62,69,75]` `consts.rs:33`. All accurate.

## New-issue scan
§2.4 snippet references `is_indel_trigger`, `recover_indel_card`, `IndelOutcome::*`, `emit_recovered`, `emit_candidates`, `RepairError::IndelUnrecoverable`, `kind.hrp()`, `ambiguous_seen` — all defined/cited elsewhere and consistent. No contradiction introduced.

## Residual Minors (non-blocking — folded inline; no re-dispatch)
- **m1.** §2.4 `Ok(outcome)` arm should explicitly carry `any_ms1=true` for the ms1 passthrough (`cmd/repair.rs:105-107`); §4.4 test 23 guards it. → folded (explicit comment).
- **m2.** §2.1 sketch used `<= j`; Phase 3 code uses `== j` (correct). → §2.1 reconciled to `== j`.
- **m3.** `IndelJson.status "unrecoverable"` unreachable (Err path emits no JSON). → status reduced to `"unique"|"ambiguous"`.
- **m4.** mk-codec path shorthand → full `string_layer/` path. → fixed.

**Clean GREEN — the plan clears the mandatory 0C/0I gate and may proceed to Phase 0.**
