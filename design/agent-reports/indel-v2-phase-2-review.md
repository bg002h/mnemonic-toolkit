# indel-v2 Phase 2 Review — cross-region two-level search

**Round:** Phase 2 (per-phase gate). **Reviewer:** feature-dev:code-reviewer (opus). **Date:** 2026-05-24.
**Commit:** `ca80783` (branch `indel-v2-cross-region-subst-fallback`). Files: `indel.rs`, `cmd/repair.rs`, `repair.rs` (test).
**Controller verification:** `cargo test -p mnemonic-toolkit --bins` → 822 passed / 2 ignored; 26 indel_* tests green; clippy clean; `collect_*` deleted; no max_subst/substitution_seen in src.

## Verdict: GREEN (0 Critical / 0 Important)

Reviewer reconstructed the OLD `collect_*` bodies (from the v0.37.1 plan + GREEN phase-3 review) and diffed each against the new helpers.

### Checks out
- **`prefix_restorations` reproduces `collect_prefix` exactly** (indel.rs:139-169): j_prefix=0 entry iff `data_part_bounds.is_some()`; j_prefix∈1..=max enumerates `p∈(3-j)..=(3+j).min(len)`, `levenshtein(&chars[..p],k)==j` exact, `Inserted if p<3 else Deleted`. No off-by-one.
- **`data_variants` reproduces `collect_data_delete`+`collect_data_insert` exactly** (indel.rs:213-260): j_data=0 → as-is `(k+data,∅,_)`; delete (`data.len()>j` guard, `combinations(len,j)`, ∅, Deleted); insert (`slots=len+j`, `combinations(slots,j)`, placeholders, allowed=combo, Inserted). Identical.
- **Budget/region/direction** (indel.rs:88-110): outer j_prefix, inner `j_data∈0..=(max-j_prefix)`, `(0,0)` skip; region CrossRegion iff both>0; direction data_dir if j_data>0 else pfx_dir; indel_count=j_prefix+j_data; `(false,false)=>unreachable!()` genuinely dead.
- **Subsumption provably non-regressing:** at max_indel=1 the new candidate set is byte-identical (j_prefix=1⇒j_data=0; j_prefix=0⇒j_data∈{0,1}; no cross pair reachable). At N≥2 the old set is a strict subset + only the intended cross pairs added. dedup-on-`recovered` ⇒ no spurious distinct candidate (a cross pair can only surface a genuinely-different valid codeword = real ambiguity, not a fidelity bug).
- `IndelRegion::CrossRegion` + the sole exhaustive match `region_str` (cmd/repair.rs:308-313) gains `=> "cross-region"`. dedup unchanged.
- **No scope creep:** no max_subst/substitution_seen/fallback; indel_exit_code still 2-arg; cmd:142 passes e_subst=0; `Unrecoverable→IndelUnrecoverable`. 3 old `collect_*` fully deleted.
- **Cross-region test** genuine: strip 'm' (prefix no longer intact → no j_prefix=0 path) + drop a data char, N=2 → only the (j_prefix=1)×(j_data=1) path reaches VALID_MS1; asserts CrossRegion + indel_count=2.

### Minor (informational, no action)
`prefix_restorations` takes both `hrp` and `k=format!("{hrp}1")` — mildly redundant, harmless (avoids per-iteration format).

Phase 2 clears the gate. Cleared to advance to Phase 3 (CLI surface).
