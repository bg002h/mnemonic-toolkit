# End-of-Cycle Review — indel-v2 (v0.37.3)

**Round:** end-of-cycle R0 (final gate before tag). **Reviewer:** feature-dev:code-reviewer (opus). **Date:** 2026-05-24.
**Scope:** branch `indel-v2-cross-region-subst-fallback` full diff vs `origin/master`; Phase-5 commit `1ab90da`.
**Controller verification (pre-fold):** version 0.37.3 consistent (Cargo.toml/Cargo.lock/both READMEs/install.sh); 3 FOLLOWUPs resolved; full suite 128 ok / 0 failed; clippy clean; manual lint green.

## Verdict: RED — 0 Critical / 2 Important / 2 Minor → FOLDED → (R1 GREEN-confirm pending)

Implementation functionally sound (the three compose orthogonally; version/CHANGELOG/FOLLOWUP surfaces consistent). Two completeness gaps held the gate; both folded in `63f82d6`.

## Important (folded)
- **I1 — missing "all three at once" integration cell** (plan §3 item 4; the user's core concern). Pairwise coverage existed (cross-region E=0 `repair.rs:2065`; data indel+subst N=1,E=1 `repair.rs:1971`+mk1 `:2147`), but NO test combining prefix indel + data indel + data substitution. Composition is structurally correct (region-agnostic gate + two-level superset search), so verification-artifact gap, not functional defect. **FOLDED (`63f82d6`):** engine unit test `indel_ms1_all_three_cross_region_plus_substitution` (substitute data[4] 'r'→'p', drop data[1] 'e', strip 'm' → `recover_indel("ms",2,1)` → Unique, recovered==VALID_MS1, region==CrossRegion, subst_count==1, indel_count==2 — Unique on first try) + a CLI cell (exit 4 + stdout VALID_MS1 + verify WARNING).
- **I2 — manual `--json` `region` enum omits `"cross-region"`** (`41-mnemonic.md:2509` said "data-part or prefix"; `region_str` emits "cross-region" at `cmd/repair.rs:348`). Headline feature undocumented; the JSON surface has no drift gate (manual lint = flag-NAMES only). **FOLDED (`63f82d6`):** region enum + a cross-region prose sentence in the `--max-indel` subsection.

## Minor
- **M1** over-budget→Unrecoverable cell — covered indirectly by the e0-rejection tests; belt-and-suspenders, no action.
- **M2** manual repair flag table says `--ms1/--mk1/--md1` "mutually exclusive" — PRE-EXISTING inaccuracy (source says "May be combined per D35"; `multi_group_both_emit_exit_5` proves it); out of scope → file a docs FOLLOWUP.

## What checks out (verified)
- Versions 0.37.3 × 5 surfaces. CHANGELOG [0.37.3] accurate (3 extensions + candidate-list/exit-4 + HrpMismatch-reversal note); SemVer PATCH correct. 3 FOLLOWUPs resolved; erasure→8 + asymmetric-delete stay open. Stale `--max-indel` "ms1/mk1 only" comment de-staled.
- **All-three coherence traced sound:** prefix-drop+data-drop+data-subst, N=2/E=1 → HrpMismatch trigger → two-level search reaches j_prefix=1×j_data=1, gate off≤1 accepts → Unique subst_count=1 region CrossRegion → exit 4 + advisory.
- **Non-breaking:** E=0 byte-identical (substitution_seen never set; cross-region identical at N=1); widening never drops the true recovery (superset + dedup-on-recovered); substitution-bearing → exit-4 verify; fallback fires only on genuine Unrecoverable (recoverable prefix recovers first).
- No HRP regression; `indel_exit_code` 3-arg everywhere; scope clean (no GUI/codec/tag in branch; `--max-subst` schema_mirror correctly deferred to post-tag GUI v0.21.3).

## Remaining post-tag (correctly deferred): tag+push (clean tree first); paired GUI v0.21.3 (`--max-subst` → REPAIR_FLAGS Number{0..=4} + pin bump).

---

## End-of-cycle R1 (GREEN-confirm) — fold `63f82d6`

**Round:** end-of-cycle R1. **Reviewer:** feature-dev:code-reviewer (opus). **Date:** 2026-05-24.
**Scope:** verify the I1+I2 fold (`63f82d6`) resolves both Importants + introduces no regression; R0-confirmed-sound surfaces re-checked unperturbed.

### Verdict: GREEN — 0 Critical / 0 Important — ship-ready

- **I1 RESOLVED** — `indel_ms1_all_three_cross_region_plus_substitution` (`repair.rs:2095`) is load-bearing, not vacuous. Corruption sound: subst `data[4]` full-idx7 'r'→'p' (non-'q', real BCH-solve), drop `data[1]` full-idx4 'e' (drop-idx 4 < subst-idx 7 ⇒ 'p' survives shift to idx6), strip leading 'm'. Asserted triple `(CrossRegion, indel_count=2, subst_count=1)` is only satisfiable by exercising all three: `region==CrossRegion` requires `j_prefix>0 && j_data>0` (`indel.rs:95-110`); no cheaper degenerate path yields it. CLI cell `ms1_all_three_cross_region_plus_substitution_exit_4` (`tests/cli_indel.rs:376`) mirrors the corruption → exit 4 + stdout VALID_MS1 + verify WARNING (`cmd/repair.rs:222`).
- **I2 RESOLVED** — manual `--json` `region` enum (`41-mnemonic.md:2511`) now lists `"data-part"`/`"prefix"`/`"cross-region"`; `--max-indel` prose (`:2392-2393`) notes the cross-region split. Accurate vs `region_str` (`cmd/repair.rs:348`).
- **No regression** — `63f82d6` is tests + manual ONLY; engine APIs the new tests call all pre-exist (`indel.rs:78/25/51/38`). Versions 0.37.3 ×5, CHANGELOG, 3 resolved FOLLOWUPs, non-breaking guarantees, exit invariant — all unperturbed.
- **Minors carried:** M1 no action; M2 (manual repair-table "mutually exclusive" — PRE-EXISTING; source "May be combined per D35", proven by `multi_group_both_emit_exit_5`) → file a docs FOLLOWUP, not a ship blocker.

Gate cleared (0C/0I). Cleared to ship v0.37.3.
