# End-of-Cycle Review — md1 indel recovery (v0.37.2)

**Round:** end-of-cycle (final 0C/0I gate before tag). **Reviewer:** feature-dev:code-reviewer (opus). **Date:** 2026-05-24.
**Scope:** branch `md1-indel-recovery` full diff vs `origin/master`; Phase-1 `957480c`, Phase-2 `7a695ee`.
**Controller verification:** version 0.37.2 consistent (Cargo.toml/Cargo.lock/both READMEs/install.sh); full default `cargo test -p mnemonic-toolkit` green (128 ok-results, 0 failed); clippy `-D warnings` clean; manual lint GREEN; FOLLOWUP resolved.

## Verdict: GREEN (0 Critical / 0 Important / 0 Minor) — ship-ready

### Phase-2
1. Behavior-change test: old `md1_indel_refusal_exit_1` GONE; replaced by `md1_multichunk_one_corrupted_recovers_exit_5` (cli_indel.rs:149-162; 3-chunk fixture, exit 5 + recovered MD1_C1). No dangling refusal assertion.
2. Manual: `--max-indel` row + prose now cover md1 ("recovers per-chunk like mk1, with cross-chunk reassembly validation"); no "md1 not yet supported" straggler. Lint green.
3. FOLLOWUP `m-format-indel-md1-chunked` → resolved + Resolution (v0.37.2) note; toolkit-only (no Companion).
4. Version 0.37.2 across all 5 surfaces.
5. CHANGELOG [0.37.2] accurate; SemVer PATCH correct.

### Integration
- Coherence sound: `--md1` chunk fails → trigger → `recover_indel_card(Md1)` locates failing chunk (`repair_chunk_one(Md1,…).is_err()`, cannot misfire — valid chunk → Ok(None)) → `recover_indel(…,"md",…,&Md1IndelOracle)` → `md1_chunk_solve` ⊆-gated → `md_codec::chunk::reassemble` (no self-correction → ⊆ preserved) → exit 5.
- No HRP regression: ms1/mk1 untouched; non-indel `repair_card(Md1)` still delegates to `repair_via_md_codec`, never calls `target_residue`.
- Scope clean: no GUI/codec files; no tag in branch; `--max-indel` flag unchanged → GUI schema_mirror correctly NOT updated (the row shipped with v0.21.1→v0.21.2 in the v0.37.1 cycle). No loose ends (doc folds applied; the lone repair.rs TODO is pre-existing mk1 parity-smoke, unrelated).

### Ship-readiness
Faithful mk1→md1 mirror; Phase-1 validated at the library level, Phase-2 closed test/manual/release-prep with no drift. Ready to tag `mnemonic-toolkit-v0.37.2`.
**Remaining post-tag: NONE beyond tag + push** (no GUI PR this cycle). Pre-tag: clean `git status --porcelain` before checkout→ff→tag→push; Cargo.lock staged with the bump (done).
