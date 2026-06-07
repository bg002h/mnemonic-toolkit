# Per-Phase Review — technical-manual-symbol-pin-lint — Phase 2 (migration)

> Persisted verbatim from the opus `feature-dev:code-reviewer` agent
> (`agentId: a72146fa26d40db98`). Toolkit-only checkout; sibling anchors'
> existence covered by the siblings-present `make lint` GREEN run + decisions-JSON
> evidence + opportunistic raw.githubusercontent ground-truth at `0599c23`.

## VERDICT: 0 Critical / 0 Important / 3 Minor — **GREEN**

### Critical / Important
None.

### Minor

**M1 — Convention exemplar `tree.rs::Node::write` names a non-existent method (vacuous G2 pass).** `src/10-foundations/14-conventions-and-notation.md:64` reworded prose uses `crates/md-codec/src/tree.rs::Node::write` as the "Methods use `Type::method`" example. Ground-truth `descriptor-mnemonic@0599c23:crates/md-codec/src/tree.rs` has NO `impl Node`/`write` method (write logic is free fn `write_node`). G2 passes vacuously: segments `Node` (enum) + `write` (bare word in `.write(w)` calls) each resolve, though the pair doesn't exist. The exact vacuous-pass class the multi-segment design targets, surviving on coincidental co-occurrence. Fix: use a real method, e.g. `crates/md-codec/src/tlv.rs::TlvSection::write` (already the exemplar in AUTHORING.md:189) or `chunk.rs::ChunkHeader::write`.

**M2 — Undeduped duplicate anchors in a comma-list expansion.** `src/50-rust-api/52-mk-codec-api.md:285` renders `bch.rs::bch_correct_regular, bch.rs::bch_correct_regular, bch.rs::bch_correct_long, bch.rs::bch_correct_long` (old `bch.rs:424, 440, 478, 493`: 424+440 in bch_correct_regular, 478+493 in bch_correct_long). Each anchor correct; SPEC §1 says comma-lists dedup. Cosmetic. Fix: collapse to `bch.rs::bch_correct_regular, bch.rs::bch_correct_long`.

**M3 — (out-of-scope observation) Stale prose claim about a source doc-comment.** `src/40-bundle-formation/42-anti-collision-invariants.md:119` and `:150` assert the `Bip388Distinctness` variant doc-comment "still carries the v0.4-era `(xpub, derivation_path_string)` raw-string equality" as "the lone residual lag." Current `crates/mnemonic-toolkit/src/error.rs:13-16` is the TYPED `(xpub.to_string(), path)` framing — resynced by the v0.47.4+ hygiene cycle (Cycle A this session). The anchor `error.rs::ToolkitError::Bip388Distinctness` is correct (intent-match); only the prose claim is now false. Predates/untouched by this migration; SPEC §7 excludes prose-claim revalidation. Merits a FOLLOWUP.

### Gate-as-built vs SPEC §2
- G1 (line-ref ban): whole-file incl. fences + `<!-- lint-allow-lineref -->` hatch; 0 `.rs:N` and 0 bare-`:N` in src/. OK.
- G2 (multi-segment existence): `grep -wF` per `::` segment. Correct. Known residual (by design, M1 is the one instance): a fictional `T::method` whose `T` and `method` co-occur passes vacuously — documented "existence ≠ correctness" residual.
- Collision rule: path-suffix-in-≥2-repos AND non-authoritative → require qualification; authoritative bare basename trusted. Implemented symbol-ref-check.py:146-151. Matches narrowed R3-I1.
- Graceful sibling-absent skip: `skip:<repo>` → g2_skipped++, end WARN; why local lint is GREEN with absent siblings. Matches §2.
- Gate-internal polish (not a defect): `default_repo()` (symbol-ref-check.py:80-85) is dead code (resolve() uses authoritative_repo() directly). Harmless; future tidy.

### SemVer / lockstep
No-bump/no-tag correct (docs + docs-lint helper, binary byte-identical; precedents a83dc75/dd7c228/3d9d38e). No CLI/GUI/end-user-manual/sibling surface → no lockstep.

### Verified clean (sample — all OK except M1)
ch42 `bundle.rs::build_unified_card` (was :707/:724 chunk_set_id fmt @1059/1078) OK; `verify_bundle.rs::emit_multisig_checks`@1533, `::MappingFailure`@1527, `::emit_md1_checks`@2026 OK; `check_resolved_slots_distinctness`@429, `ToolkitError::message`@611, `::exit_code`@488 OK; ch31/51 `to_miniscript.rs::failed`/`::to_miniscript_descriptor`/`::build_descriptor_public_key` ground-truthed OK; ch41 mermaid `synthesize_unified` label-only OK; ch53 `consts.rs::tests::valid_str_lengths...` tests:: form OK; AUTHORING/lint.sh six→seven + false-CI fix coherent. ch14:64 `Node::write` WRONG (M1).

### Bottom line
GREEN (0C/0I). Anchors name correct definition sites + match prose intent; gate wired blocking, implements G1/G2 + narrowed collision + graceful skip; six→seven/false-CI coherent. 3 Minors are polish (fix M1 before ship — trivial + in the format-defining file; M2 cosmetic; M3 → FOLLOWUP). None blocks the phase gate.
