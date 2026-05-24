# Phase 4 Review — mk1 per-chunk recovery + reassembly oracle

**Round:** Phase 4 (per-phase gate). **Reviewer:** feature-dev:code-reviewer (opus). **Date:** 2026-05-24.
**Commit:** `7c439c3` (branch `m-format-incorrect-length-recovery`). File: `src/repair.rs` (only).
**Controller verification:** `cargo test -p mnemonic-toolkit --bins` → 810 passed / 2 ignored; 4 mk1 tests pass; clippy `-D warnings` clean.

## Verdict: GREEN (0 Critical / 0 Important)

### Fixture adaptation is CORRECT (structural necessity, not a coverage regression)
The brief's "single-chunk decode" assumption was wrong; the implementer's rewrite is verified against mk-codec source:
- `reassemble_from_chunks` returns `ChunkedHeaderMalformed("received N chunks, header declares total_chunks=T")` when `chunks.len() != total_chunks` (`mk-codec string_layer/chunk.rs:131-136`). `VALID_MK1_REG`==`MK1_CARD_C1` declares `total_chunks=2` → cannot decode alone.
- "No realistic mk1 fits one string" corroborated: compact xpub 73B > `SINGLE_STRING_LONG_BYTES=56` (`pipeline.rs:161-168`). Every realistic mk1 is chunked.
- The 2-chunk fixture genuinely reassembles (tests assert `decode(&[C0,C1]).is_ok()` precondition); cross-chunk SHA-256 hash check at `chunk.rs:189-199` is the disambiguation the oracle relies on. The rewrite tests the real supported path — coverage not weakened.

### Logic verified
- **`mk1_chunk_solve`** (repair.rs:886-912) mirrors `repair_chunk_one`'s core + the ⊆ gate (line 898). Types: `decode_*_errors -> Option<(Vec<usize>, Vec<Gf32>)>`, `Gf32=u8` (`mk-codec bch_decode.rs:94`), `corrected[p]^=m` typechecks. `target_residue(Mk1,_)` always `Some` (repair.rs:76-77). Defensive re-verify (908) + `p>=len` guard (903) present.
- **`Mk1IndelOracle::validate`** (922-933): ⊆-gated solve THEN substitute + `mk_codec::decode(&refs)` reassembly. The local ⊆ solve is necessary because `decode→decode_string→bch_correct_*` self-corrects t≤4 UNGUARDED (`bch.rs:683-687`); the oracle hands decode a clean chunk.
- **`recover_indel_card` mk1 arm** (953-972): failing chunks via `repair_chunk_one(...).is_err()`; `failing.len()!=1 → Unrecoverable`; else recover with `failing_index=f`. ms1 arm + md1 `BadInput` refusal unchanged.

### Tests non-degenerate
- Single-failing too-long/too-short: oracle carries both real chunks, `failing_index:1`, assert `recovered==MK1_CARD_C1`.
- Multichunk via `recover_indel_card`: corrupt only C1 (78-char regular ⇒ TooManyErrors Err); C0 intact (Ok(None)) ⇒ `failing==[1]`; reassembly oracle fires.
- Two-failing: C0+insert (109-char ⇒ UnparseableInput Err) + C1+insert (78-char ⇒ TooManyErrors Err) ⇒ `failing==[0,1]` len 2 ⇒ Unrecoverable for the right reason (not degenerate 0-failing).

### Scope/hygiene
Only `repair.rs` changed; cmd/repair.rs/main.rs/indel.rs ms1 path untouched; `recover_indel_card`'s `#[allow(dead_code)]` still warranted (non-test consumer in Phase 5). Clippy clean.

### Minor (informational, no action)
`mk1_chunk_solve` residue==0 branch always returns `encode_chunk(...)` (canonical) vs the plan's conditional `cand.to_string()` for empty-allowed — equivalent-or-better (canonical output; byte-identical for all-lowercase fixtures).

Phase 4 clears the gate. Cleared to advance to Phase 5 (CLI wiring).
