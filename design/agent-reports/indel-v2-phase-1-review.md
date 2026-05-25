# indel-v2 Phase 1 Review — substitution accept gate + subst_count

**Round:** Phase 1 (per-phase gate). **Reviewer:** feature-dev:code-reviewer (opus). **Date:** 2026-05-24.
**Commit:** `29c7daf` (branch `indel-v2-cross-region-subst-fallback`). Files: `indel.rs`, `repair.rs`, `cmd/repair.rs`.
**Controller verification:** `cargo test -p mnemonic-toolkit --bins` → 821 passed / 2 ignored (818+3 new); clippy `-D warnings` clean; scope grep (max_subst/CrossRegion/substitution_seen) → none.

## Verdict: GREEN (0 Critical / 0 Important)

### Checks out
- **Gate equivalence at e_subst=0 (byte-identical default path):** Ms1 `off = corrections.filter(!allowed.contains).count(); off<=e_subst` (repair.rs:894-903); mk1/md1 `off = positions.filter(!allowed.contains).count(); if off>e_subst {None}` (repair.rs:935-949/980-994); residue==0 path returns `(encode_chunk,0)`. At e_subst=0: `off==0` ⟺ all corrections ∈ allowed ⟺ old `⊆`. Mk1/Md1 oracles thread e_subst + return `(chunk, off)`.
- **subst_count** set from oracle `sc` in 3 producers; test literals `=0`; `dedup_by_recovered` unchanged (recovered-only); `--json` does NOT yet carry subst_count (Phase 3).
- **Call-site completeness:** all `validate`/`recover_indel`/`recover_indel_card`/`*_chunk_solve`/`IndelCandidate{` sites updated; 2 mocks; `md1_chunk_solve` test `.map(|(s,_)| s).as_deref()` adaptation; 5 `recover_indel_card` test calls + cmd:142 pass `0`. Clean build ⇒ no miss.
- **No scope creep:** `recover_indel` keeps 3-producer structure; `IndelRegion::{Prefix,DataPart}` only; `indel_exit_code` 2-arg; `Unrecoverable→IndelUnrecoverable` (no fallback); no `--max-subst`.
- **Tests load-bearing:** ms1 drop+substitute pair (e1→Unique subst_count=1 / e0→Unrecoverable — the e0 rejection proves the substitution wasn't silently admitted); mk1 single-chunk indel+subst. Index ordering sound (drop at lower data idx than substitution → survives shift).
- **Regression:** all prior pure-indel sites pass e_subst=0 (byte-identical); 821=818+3.

### Minor (sub-threshold, no action)
A redundant e0 test (`indel_ms1_pure_indel_rejects_indel_plus_substitution` vs new `…rejected_at_e0`) — intentional paired-contract documentation.

Phase 1 clears the gate. Cleared to advance to Phase 2 (cross-region restructure).
