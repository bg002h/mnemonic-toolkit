# SPEC R0 review — mk1-repair-set-level-reverify — round 2

**Verdict: NOT GREEN (0 Critical / 1 Important / 2 Minor)**
**Reviewer:** adversarial opus architect (read-only, cross-repo). Verified @ toolkit `13345d35`, mk `main@85bca69`.
**Dispatched:** 2026-07-07 (Cycle E, SPEC R0 loop round 2 — convergence on rev-2). Persisted verbatim per CLAUDE.md.

Round-1's C1/I1/I2/I3/M1-M4 all correctly folded (verified below). The tri-state closes C1 (partial-set per-plate repair preserved) + the funds fix (full-set miscorrection rejected) for the common single-card case. One new Important: §2's non-happy-path classification is under-specified in the funds-critical path (reject-condition breadth + multi-group aggregation), which a literal implementation could ship as a residual funds gap. Narrow + mechanical — not a redesign.

**NO-BUMP independently confirmed holds:** the discriminator uses only public API — `DecodedString::data()` (`bch.rs:604`), `StringLayerHeader::from_5bit_symbols` (`header.rs:120`, re-exported `mod.rs:39`), public `Chunked { chunk_set_id, total_chunks, chunk_index }` fields (`header.rs:45-53`). A caller parses `total_chunks`/`chunk_set_id` up front with no codec change.

## Round-1 convergence — all RESOLVED
- **C1** → §0.2 + §2 tri-state (rule 3 preserves per-plate, mk repair exit 5 + advisory / `mnemonic repair` exit-4 candidate); §4.2 pins both surfaces; discriminates count-vs-`total_chunks`, not the overloaded error string. ✅
- **I1** → §1 reachability math (min ≥2 chunks; single `Chunked` has the 4-byte hash → stronger; `SingleString` unreachable → no-weaker); numbers check out; §4.7 lock. ✅
- **I2** → §6 ADVANCE, all 5 refs enumerated (install.sh:41, manual.yml:79, quickstart.yml:77, technical-manual.yml:109, 44-mk-cli.md:12), sequenced release→pin, self-pin regardless, library-not-binary decoupling noted. ✅
- **I3** → §4.1 pinned known-miscorrection seed = hard funds proof; §4.8 Clopper-Pearson upper bound + seeded StdRng + fixed N + observed-≥1 + cite measured bound. ✅
- **M1-M4** → all resolved (exit 2 named + non_exhaustive + NO-BUMP; `crates/mk-cli/...` path; md1 no-bypass §4.6; GUI verify §6). ✅

## New finding — I-r2-1 (Important): §2 under-specifies non-happy-path classification in the funds path
**(a) Rule 2 too narrow.** §2 rejects only on `CrossChunkHashMismatch`/`ChunkSetIdMismatch`. A corrected FULL set can fail `mk_codec::decode` with OTHER errors — a substitution landing in the 8-symbol chunked-header region → `ChunkedHeaderMalformed`/`MixedHeaderTypes` (decode order `reassemble_from_chunks`→header-consistency→hash→`decode_bytecode`, `pipeline.rs:150-151`), or a 2⁻³²-hash-colliding miscorrection failing structurally in `decode_bytecode`. None are in rule 2's list, not partial (rule 3), not Ok (rule 1) → fall through; if the impl's unhandled default is "bless," a full-set miscorrection emits at exit 5. **Reject condition must be ANY full-set decode `Err`, not the two named variants** (variant only informs the message).

**(b) Multi-group/batch unspecified.** `mk repair` + `mnemonic repair --mk1` accept "one or more mk1 strings" (44-mk-cli.md:226; `read_mk1_strings` flat-collects; `resolve_groups` one Vec) — documented, reachable batch. §2 is singular ("read `total_chunks` from *a* header"). With multiple `chunk_set_id` groups, cross-group exit aggregation is undefined; if it inherits "exit 5 if any correction" while one group is a full-set miscorrection, the miscorrected chunks ship under success. No §4 multi-group test → ships untested.

**Fix (state the invariant, don't enumerate variants):**
> **Confident BLESS** (exit 5 / short-circuit, chunks presented as *recovered*, no unverified caveat) **iff `mk_codec::decode` on the exact supplied set returns `Ok`.** Any `Err` → **reject** if the group is complete-and-consistent, or **unverified-candidate** (loud advisory, no confident-success framing) if incomplete — **never a silent bless**. **Multi-group:** apply per-`chunk_set_id`; **reject dominates the invocation exit (reject > candidate > bless > clean)** and that group's chunks are NOT presented as recovered.

This closes (a)+(b) and the corrupted-`total_chunks` header case (misclassify-as-partial → candidate-with-advisory, never a clean confident success — bounded, user re-verifies at reassembly). Add §4 test: a batch {one full-set miscorrection group, one clean/partial group} must exit reject and must NOT emit the miscorrected group's chunks as recovered.

## Minors
- **M-r2-1** — §2 "supplied == total_chunks" → "**complete-and-consistent** group (indices `0..total-1` each present exactly once, consistent `total_chunks`/`chunk_set_id`)" so duplicate-index/gap/supplied>total route to reject-or-candidate, not an accidental "==count" bless.
- **M-r2-2** — §4.8's `observed-≥1` self-check flakes if `N` too small for ~10⁻⁴-10⁻⁵. Either size `N` so `E[hits]≫1` (N≈10⁶→~10-100 hits) OR make observed-≥1 a soft warning (the hard proof is §4.1's pinned seed, so §4.8 needn't gate on a hit). Also: §4.1's pinned seed can be invalidated by a future mk-codec BCH change — the test should fail with an explicit "re-pin the miscorrection seed" message, not a cryptic assertion.

## To GREEN
Fold I-r2-1 (rewrite §2 around BLESS-iff-decode-Ok + reject-dominant multi-group + add the batch §4 test) + the 2 Minors. Everything else converged. Mechanical §2/§4 tightening; round 3 closes fast.
