# R0 Architect Review (round 2, convergence) — `SPEC_synthesize_descriptor_dedup.md`

**Reviewer:** opus `feature-dev:code-reviewer` (mandatory pre-implementation R0 gate). **Date:** 2026-06-06.
**Branch:** `synthesize-descriptor-dedup`. **Verdict:** **0 Critical / 0 Important** (+ 1 Minor M2). **GREEN.**

> Persisted verbatim per CLAUDE.md. Both round-1 folds (I1, M1) verified against source. M2 (new, documentary) = the characterization cell must use FROZEN literals, not a live `synthesize_descriptor` compare (vacuous post-delegation). Folded inline; no new round.

---

## VERDICT: 0 Critical / 0 Important / 1 Minor (M2) — GREEN, implementation may proceed.

### Item 1 (THE GATE) — characterization cell constructible + drives the right fn/branch: CLEAN
- **Constants exist:** `TREZOR_12_ZERO` + `BIP39_TEST_2` at `synthesize.rs:968-971` (also in integration crates, e.g. `cli_verify_bundle_multi_cosigner_mk1.rs:21-23`).
- **Invocation drives `synthesize_unified` Multi branch:** `bundle.rs:394-407` routes `MultisigMultiSource`/`MultisigWatchOnly`/`MultisigHybrid` + the single-sig modes through `synthesize_unified` (`:399`). A 2-of-2 `wsh-sortedmulti` with two distinct `@N.phrase=` slots = `MultisigMultiSource` → `synthesize_unified` n=2 → the `Multi` branch (`:869-889`) — exactly what §2 refactors. (Direct unit call `synthesize_unified(slots, WshSortedMulti, 2, …)` lands in the same branch.)
- The existing `synthesize_unified_multisig_*` cells (`:1637-1693`) assert only `.len()`/`.starts_with`/`any_secret_bearing`, and `unified_fixture` shares ONE seed (identical fingerprints) → cannot catch per-cosigner csi/ordering drift. The distinct-phrase choice fixes this.

### Item 2 — non-vacuous guard: CLEAN
`Multi` branch `csi = derive_mk1_chunk_set_id(&s.xpub.fingerprint().to_bytes())` (`:882-883`); two distinct phrases → distinct fingerprints → distinct csi → `mk1[0] != mk1[1]`. Byte-exact assert on `mk1[0]+mk1[1]` goes RED on any per-cosigner ordering swap / csi-seed change / stub change.

### Items 3/4 — no new drift; adversarial: CLEAN
- M1 fold (§3) accurate — statement-order immaterial (independent Bundle fields, identical iteration order).
- n==1 path guarded: `synthesize_unified`'s `Single` branch (`:854-868`) byte-identical to `synthesize_descriptor`'s (`:249-263`); the frozen 16-cell golden (`cli_bundle_full.rs:14-37`) drives n==1 end-to-end via the CLI and stays RED-sensitive post-edit (runs the post-edit binary). Both n==1 and n>1 paths through `synthesize_unified` guarded after the fold; the only previously-unguarded path was n>1, now closed by I1.

### MINOR — M2 (new, folded)
The I1 cell must assert against **FROZEN byte literals** captured from the pre-edit binary (or a fixture file), NOT `assert_eq!(synthesize_unified(…), synthesize_descriptor(…))` — post-delegation both sides are the same function → `assert_eq!(x,x)`, vacuous (the unit-test form of the co-moving trap). Fold one sentence into §4/§6. Minor, non-gating.

---

**GREEN — 0 Critical / 0 Important. Implementation may proceed** (fold M2's one-sentence clarification while writing the cell).
