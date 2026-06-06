# R0 Architect Re-Review (round 4, FINAL) — SPEC_xpub_search_passphrase_candidates_file.md

**Reviewer:** opus `feature-dev:code-reviewer`. **Date:** 2026-06-05. **Branch:** `xpub-search-passphrase-candidates-file` (master `5ebd4a9`).
**Verdict:** **0 Critical / 0 Important — GREEN.** R0 hard gate satisfied; implementation may begin. (2 non-blocking Minors.)

> Persisted verbatim per CLAUDE.md. Convergence: r1 (0C/3I) → r2 (0C/1I) → r3 (0C/1I) → r4 (0C/0I GREEN).

---

## Item closure
**1. I-r3 (memory hygiene) fold — CONFIRMED complete.** Candidate-line `Zeroizing<String>` (§3:46) matches `passphrase_of_xpub.rs:260`; `derive_master_seed -> Zeroizing<[u8;64]>` (`derive_slot.rs`) covers the seed. `ZEROIZE_ROWS` is the right mechanism (`lint_zeroize_discipline.rs:45-47` "add a row AND wrap"; per-row anchor test `:262-288`; row-count bound `18..=35` accommodates one more). **No other unwrapped owned secret** — §3:39 confirms streaming line-by-line (no slurp); only the per-iteration candidate line + the winning `matched_passphrase` allocate secrets, both addressed.
**2. No new drift.** §7/§8 coherent; lint runs under Phase-2 workspace test. (Minor M-r4a: the curated-row lint is lagging, not a leading gate — wording softened.)
**3. Whole-SPEC coherence — CONFIRMED.** I1 dispatch (after `resolve_seed :252`, before inline resolve `:260`), I2 new variant (`XpubSearchPassphraseCandidatesExhausted` sorts after `XpubSearchNoMatch :328`; exit 4; the hardcoded "paths searched; widen --max-account" Display at `:785` correctly judged wrong for a scan; json-no-match-then-error live `:365-384`), I3 secret:false/Path, I-A variant name, M-2 ArgGroup (replaces live pairwise `:80-81/:89-90`), I-r3 zeroize — all compose, no contradiction, no open TODO. `matched_passphrase` typed `Option<String>` (not Zeroizing) is correct + necessary (`PassphraseOfXpubResult` is `#[derive(Serialize)]`; a Zeroizing field can't derive Serialize) — the "held Zeroizing until serialized" describes a local assigned at emit time. `searched_count` field name maps live (`:187/:196`).

## Critical / Important
None / None.

## Minor (non-blocking)
- **M-r4a (§3:54 wording):** "turning the convention into an enforced gate" overstates `lint_zeroize_discipline.rs` (curated-row; a forgotten row won't fail — lagging like `schema_mirror`). Soften to "recording the new site in the curated discipline list" + add "add the `passphrase_search.rs` ZEROIZE_ROWS row" as an explicit §8 Phase-2 deliverable.
- **M-r4b (§4):** `matched_passphrase` Zeroizing-until-serialized is inherently partial (deliberately serialized to `--json`) — SPEC already flags it; no change.

## VERDICT: 0 Critical / 0 Important — GREEN. Implementation may begin.

---

## Fold note (applied after persisting)
- **M-r4a — FOLDED:** §3 wording softened; §8 Phase-2 gains an explicit "add the `passphrase_search.rs` ZEROIZE_ROWS row" deliverable.
- M-r4b — no action (already flagged partial).
- GREEN ⇒ no re-dispatch.
