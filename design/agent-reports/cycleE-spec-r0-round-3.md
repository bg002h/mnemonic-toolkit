# SPEC R0 review — mk1-repair-set-level-reverify — round 3

**Verdict: GREEN (0 Critical / 0 Important)** — 2 Minor editorial nits noted for cleanup (non-blocking).
**Reviewer:** adversarial opus architect (read-only, cross-repo). Verified @ toolkit `0afc40f2`, mk `main@85bca69`.
**Dispatched:** 2026-07-07 (Cycle E, SPEC R0 loop round 3 — convergence on rev-3). Persisted verbatim per CLAUDE.md.

The design has converged. The BLESS-iff-decode-Ok invariant closes the last funds-path gap, the multi-group aggregation is now reject-dominant and tested, and every round-1/round-2 finding is folded without introducing substantive drift.

## Convergence verification
**I-r2-1a — reject on ANY full-set `Err` — ✅ CLOSED.** §2 states the invariant plainly: "confident BLESS … IFF `mk_codec::decode` on the EXACT supplied corrected set returns `Ok`. Any `Err` is NEVER a silent bless." Rule 2 rejects a complete-and-consistent group on ANY `decode` `Err` — enumerating the previously-uncovered cases (header-region `ChunkedHeaderMalformed`/`MixedHeaderTypes`, structural `decode_bytecode` failure from a hash-colliding miscorrection) as illustrations, not an allowlist. Re-walked the branch space:
- full set + genuine ≤4 → decode Ok → bless ✓
- full set + hash-mismatch miscorrection → Err → reject ✓
- full set + header-region miscorrection corrupting parsed `total_chunks` → consistency check fails → candidate-with-advisory (never a clean bless), per §2 "Header-corruption note" ✓
- supplied > total (dup/gap) → not complete-and-consistent → candidate ✓
No path reaches a confident bless without `decode == Ok`. The funds property holds airtight.

**I-r2-1b — multi-group reject-dominant aggregation — ✅ CLOSED.** §2 + §3: per-`chunk_set_id` group, invocation exit = dominant (reject > candidate > bless > clean), a rejected group's chunks not presented as recovered. §4.5b pins the {miscorrection group, clean/partial group} → exit-reject + not-emitted case. A batch success can no longer carry a miscorrection.

**M-r2-1 — ✅** "complete-and-consistent group" (indices `0..total-1` each once, consistent `total`/`id`) replaces bare "== count".
**M-r2-2 — ✅** §4.8 N-sizing (E[hits]≫1 or soft-warn, `--ignored`/env gate if slow); §4.1 explicit "re-pin the seed" failure message on a future BCH change.
**NO-BUMP / implementability — ✅ re-confirmed.** §2 cites the exact public-API chain verified: `DecodedString::data()` (`bch.rs:604`, pub) → `StringLayerHeader::from_5bit_symbols` (`header.rs:120`, pub, re-exported `mod.rs:39`) → public `Chunked{chunk_set_id,total_chunks,chunk_index}` (`header.rs:45-53`). No new codec API. mk-codec/md-codec/ms-codec NO-BUMP holds.

All round-1 findings (C1 tri-state, I1 reachability, I2 5-ref sibling-pin advance, I3 non-vacuous harness, M1-M4) remain correctly folded from rev-2; no regression.

## Minor nits (do not block GREEN; cleaned at implementation-open)
- **M-r3-1 (editorial):** §2 duplicated the "Residual partial-set exposure is bounded:" paragraph verbatim — a copy-paste artifact from the rev-3 edit; delete one. **[FOLDED: dup removed.]**
- **M-r3-2 (internal-consistency):** §7.2 "the re-verify only rejects on the cross-chunk hash, which a real repair passes" predates the rev-3 "reject on ANY `Err`" rule. Intent correct (no false-reject of a genuine full-set ≤4 correction — it yields `decode == Ok` → bless), but reword to "…rejects on any full-set decode failure; a genuine ≤4 correction decodes `Ok` and is blessed, never rejected." **[FOLDED: reworded.]**

## Gate result
0 Critical / 0 Important. **The SPEC passes R0.** Persist this round-3 review, fold the two Minor nits (mechanical, done in the implementation-open commit), and implementation may proceed per CLAUDE.md phase-3 (single implementer, TDD, worktree; §4 tests before src).
