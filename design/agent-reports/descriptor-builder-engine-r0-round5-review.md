<!-- VERBATIM opus-architect R0 review, round 5, descriptor-builder engine (Release A, v0.50.0). Persisted per CLAUDE.md. SPEC + Phase-1 impl @ post-r4-fold; source b596d3f. Verdict: RED 0C/1I/0m — I1 overclaim survived in SPEC §7:103. -->

# R0 REVIEW (round 5) — descriptor-builder engine — **RED**

Preamble: A passing 20-green suite is necessary but NOT sufficient to clear I1 — a false comment passes the suite trivially; that is the false-green species. The three `ir.rs` comments are now honest, but the same overclaim survives in SPEC §7 line 103 (present-tense test-plan prose).

**RED. 0 Critical / 1 Important / 0 Minor.**

## CRITICAL
None.

## IMPORTANT

### I1 (still open) — SPEC §7 line 103 repeats the exact overclaim the `ir.rs` comments just dropped

`design/SPEC_descriptor_builder_engine.md:103`, §7 "Test plan":
> "**enum-coverage guard** (`node_kinds_cover_enum` — every `PolicyNode` variant's `kind()` ∈ `NODE_KINDS`, count-matched, **so a future variant cannot silently miss the `--spec-schema` grammar**; risk #2 freeze, R0-r3 I1)."

"so a future variant cannot silently miss the grammar" is the round-4 "a new variant cannot be silently absent from the schema" paraphrased. The "so" is a non-sequitur: the count-match + `kind()`-set equality catch *drift* between `samples` and `NODE_KINDS`, but a variant omitted from `samples` + `NODE_KINDS` + `NODE_GRAMMAR` together leaves all three at 16==16==16 and passes. So a future variant **can** silently miss the grammar — the bypass round-4 traced. Present-tense prose, no correcting bracket; a reader takes it as the current guarantee. Survived round 4 because round 4 enumerated the three `ir.rs` comments as the fix target and did not sweep §7.

**Fix (prose-only, mirrors the sanctioned `ir.rs` language):** reword §7:103 to the honest guarantee — cross-checks catch *drift* between the hand-lists; does NOT catch a variant jointly omitted from all three (needs a variant-enumerator macro); enum→list is author discipline. Drop "so a future variant cannot silently miss the `--spec-schema` grammar." No code/logic/architecture change.

## MINOR
None new. M(new)-1 (skeleton key-constants mutually-non-substring note) is **folded** at `mod.rs:32-34` — correct. M2 (citations) already correct.

## What passes

- **The three `ir.rs` I1 doc-comments are reworded correctly, no remaining overclaim:** `NODE_KINDS` doc (~:31-34, "catch a *drift* between these three hand-lists; they cannot by themselves force a brand-new variant into all three"), `all_variant_samples` doc (~:303-310, "a REMINDER that forces an author to visit this helper — but it does NOT force the new variant into the `samples` vec (E0004 is a property of the match arms…)"), `node_kinds_cover_enum` doc (~:357-363, "catches a sample/`NODE_KINDS`/grammar *drift*; it does NOT catch a variant omitted from all three together"). The two false claims ("forces a variant into the list", "cannot be silently absent") are gone.
- `ir.rs:208` ("Must stay in sync with NODE_KINDS") is imperative author-discipline framing, not a guarantee — fine.
- `mod.rs` `skeleton()` note (M(new)-1) folded and correct.
- **SPEC sweep otherwise clean.** Lines 23/33/38/55/72/79 use "forced by serde"/"serde enforces"/"rejects" about real serde/miniscript behavior — all true. Line 147 is superseded Round-3 change-log history ("…forcing it into NODE_KINDS"), explicitly corrected by the Round-4 entry at :151 — leave as-is (editing destroys the audit trail). Only :103 is a live present-tense overclaim. A paraphrase grep (`silently|ensures|guarant|enforces|impossible to|cannot|miss the|absent`) surfaced no further overclaim.
- **Test surface structurally sound, unchanged from round-4 code-green** (20 tests: 5 mod.rs fixtures + 13 ir.rs + 2 schema.rs). The I1 fix is comment-only; suite pass/clippy-clean preserved. (cargo not executed this round — no Bash tool; inferred from comment-only nature.)

---

**Not cleared. Phase 2 does NOT begin.** Fix = single prose reword at SPEC:103 (drop the "cannot silently miss" claim; state the drift-only guarantee). Leave :147 untouched (audit history). Re-dispatch round 6.
