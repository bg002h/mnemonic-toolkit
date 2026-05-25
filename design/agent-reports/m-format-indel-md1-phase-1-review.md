# md1 Phase 1 Review — per-chunk recovery (mirror mk1)

**Round:** Phase 1 (per-phase gate). **Reviewer:** feature-dev:code-reviewer (opus). **Date:** 2026-05-24.
**Commit:** `957480c` (branch `md1-indel-recovery`). Files: `repair.rs`, `indel.rs`.
**Controller verification:** `cargo test -p mnemonic-toolkit --bins` → 818 passed / 2 ignored; 5 md1 tests pass; clippy `-D warnings` clean.

## Verdict: GREEN (0 Critical / 0 Important / 0 Minor)

Faithful mk1→md1 mirror. Verified against the **pinned md-codec v0.34.0** (Cargo.lock:643-646, checksum 4628f625…; docs.rs confirms `bch::MD_REGULAR_CONST = 0x0815c07747a3392e7` + `chunk::reassemble(&[&str])` pub in 0.34.0 — not just the local 0.35.0 checkout).

### Checks out
- `MD_REGULAR_TARGET = md_codec::bch::MD_REGULAR_CONST` (repair.rs:46), mirrors MK targets.
- `target_residue` `(Md1,Regular)=>Some`, `(Md1,Long)|(Ms1,_)=>None`; exhaustive; doc updated. **No regression:** non-indel `repair_card` Md1 branch delegates to `repair_via_md_codec`, never calls `target_residue`.
- `md1_chunk_solve` (repair.rs:950-976) line-by-line mirror of `mk1_chunk_solve` (HRP "md", Md1 target, `BchCode::Long => return None`, ⊆-gate, `^=`+bounds guard, defensive re-verify, `encode_chunk("md",…)`).
- `Md1IndelOracle` (repair.rs:1004-1019) mirrors `Mk1IndelOracle`, oracle = `md_codec::chunk::reassemble(&refs)`. reassemble verify-or-rejects (no self-correction) → ⊆ guarantee preserved (stronger than mk1's self-correcting decode); residue-0 solved chunk passes.
- `recover_indel_card` Md1 arm (repair.rs:1104-1124) replaces the BadInput refusal with the Mk1-arm shape (`repair_chunk_one(Md1,i,c).is_err()` location; ≠1 → Unrecoverable; else recover). Valid chunks → `Ok(None)` → not miscounted; locator cannot misfire.
- R0 doc-Minors folded: indel.rs:6/:60 include "md"; recover_indel_card doc no longer says md1 refused.
- Tests real: reassembly precondition; too-long/too-short → Unique (recovered==MD1_C1); two-failing (both C0+C1 corrupted, genuinely failing==[0,1]) → Unrecoverable; ⊆-rejection (substitution w/ empty allowed → None) + residue-0 round-trip.
- Scope/hygiene: only repair.rs+indel.rs; Phase-2 surfaces (manual "md1 not yet supported", FOLLOWUP open, Cargo 0.37.1) untouched; no flag-name change → GUI unaffected; clippy clean, no new allows. Shared-generator reuse empirically validated (residue-0 on real md1 cards only holds if mk's computation matches md's).

Cleared to advance to Phase 2.
