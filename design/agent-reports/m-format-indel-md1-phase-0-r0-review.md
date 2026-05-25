# R0 Architect Review — IMPLEMENTATION_PLAN_m_format_indel_md1.md (v0.37.2)

**Round:** R0 (plan-doc gate; mandatory pre-code). **Reviewer:** feature-dev:code-reviewer (opus). **Date:** 2026-05-24.
**Branch:** `md1-indel-recovery` (base `origin/master 950d5ef`). **Persisted verbatim per CLAUDE.md.**

## Verdict: GREEN (0 Critical / 0 Important)

The plan is sound. Every load-bearing fact verified against codec source. The mk1→md1 mirror is faithful, the reuse assumptions hold, release-prep/manual/FOLLOWUP coverage complete. 2 non-blocking doc Minors.

## Load-bearing facts — all CONFIRMED
1. **md-codec pub API:** `bch::MD_REGULAR_CONST = 0x0815c07747a3392e7` (bch.rs:17), `bch::polymod_run` (bch.rs:40), `bch::hrp_expand` (bch.rs:49), `bch_decode::decode_regular_errors -> Option<(Vec<usize>,Vec<Gf32>)>` (bch_decode.rs:403), `chunk::reassemble(&[&str]) -> Result<Descriptor,Error>` (chunk.rs:305; re-exported lib.rs:43). All pub.
2. **Shared codex32 regular params (the reuse assumption) — PROVABLY identical mk==md:** `GEN_REGULAR` (`[0x19dc500ce73fde210, 0x1bfae00def77fe529, 0x1fbd920fffe7bee52, 0x1739640bdeee3fdad, 0x07729a039cfc75f5a]`), `REGULAR_SHIFT=60` (md bch.rs:20 / mk bch.rs:191), `REGULAR_MASK=0x0fffffffffffffff` (md:21 / mk:194), AND **`POLYMOD_INIT=0x23181b3`** (md bch.rs:19 / mk bch.rs:185 — the 4th invariant, not enumerated in the plan but identical). ⇒ toolkit's `polymod_residue("md", vals, MD_REGULAR_TARGET, Regular)` computes md1's residue bit-for-bit correctly. R1 fallback (use md's own polymod) unnecessary.
3. **md is regular-only** — no GEN_LONG/MD_LONG_CONST/decode_long/BchCode::Long in md-codec → `target_residue(Md1, Long) => None → UnsupportedCodeVariant` correct.
4. **`reassemble` does NOT self-correct + checksum-gates:** `reassemble` (chunk.rs:305-389) calls `unwrap_string` per chunk, which hard `bch_verify_regular` checks (codex32.rs:115) — verify-or-reject, no correction. So a candidate must be residue-0 before reassembly accepts it (md1_chunk_solve guarantees this). Stronger than mk1 (mk_codec::decode self-corrects t≤4 unguarded; md does not). Cross-chunk checks: shared chunk_set_id/count/version, complete 0..count, no gaps (chunk.rs:339-367) + re-derived chunk_set_id integrity (chunk.rs:378-386).

## Mirror code — faithful
- `MD_REGULAR_TARGET` mirrors `MK_REGULAR_TARGET` (repair.rs:41). `target_residue` extension additive + exhaustive. **No regression:** the non-indel `repair_card` Md1 branch (repair.rs:777-794) does NOT call `target_residue` (it pre-gates + delegates to `repair_via_md_codec`); the new `Some` arm is consumed only by the new indel path + `repair_chunk_one`'s location call.
- `md1_chunk_solve` mirrors `mk1_chunk_solve` (repair.rs:901-927) with the correct divergence `BchCode::Long => return None`. Types match (`decode_regular_errors -> Option<(Vec<usize>,Vec<u8>)>`).
- `Md1IndelOracle` mirrors `Mk1IndelOracle` (repair.rs:933-948), swapping `mk_codec::decode` → `md_codec::chunk::reassemble`. `encode_chunk("md",…)` produces canonical wire form `unwrap_string` accepts.
- `recover_indel_card` Md1 arm replaces the BadInput refusal (repair.rs:1031-1033) with the mk1 pattern.

## Failing-chunk location cannot misfire
`repair_chunk_one(Md1,…)` (HRP-generic): valid chunk → residue 0 → Ok(None) → not flagged; too-long/short → Err (TooManyErrors / UnparseableInput / ReservedInvalidLength / UnsupportedCodeVariant for [96,108]) → flagged. Correct.

## Spurious-recovery — no new exposure
Dominant boundary is the per-chunk 65-bit BCH check in `unwrap_string` (~2⁻⁶⁵, same floor mk1 accepts). The 20-bit chunk_set_id is a secondary check + re-derived in reassemble. No weakening below mk1's accepted profile.

## Behavior-change + release-prep — present
- The existing `md1_indel_refusal_exit_1` cli_indel cell (cli_indel.rs:152-158) WILL fail post-change; Phase 2 Step 1 explicitly replaces it. `ins_data` helper exists; mk1 multichunk (cli_indel.rs:86-94) is the template.
- Engine HRP-agnostic: `data_part_bounds(input,"md")`=Some(3); `collect_prefix` builds "md1"+tail. ✓
- Manual: table row (41-mnemonic.md:2281) + prose (:2394) both to update. ✓ FOLLOWUP toolkit-only (no Companion) → single flip. ✓ SemVer PATCH + no GUI. ✓ v0.37.2 release-prep enumerated. ✓

## Minor (non-blocking — folded into Phase 1 Step 3; doc-only, no R1)
- **M1:** `indel.rs:60`/`:6` doc says `{"ms","mk"}` — add "md".
- **M2:** `recover_indel_card` doc (repair.rs:994-997) still says "md1 = refused … not yet supported" — update for the md1 recovery.

**Cleared for Phase 1.**
