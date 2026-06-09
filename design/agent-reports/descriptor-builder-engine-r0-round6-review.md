<!-- VERBATIM opus-architect R0 review, round 6, descriptor-builder engine (Release A, v0.50.0). Persisted per CLAUDE.md. SPEC + Phase-1 impl @ post-r5-fold; source b596d3f. Verdict: RED 0C/1I — I2 (4th-location instance of the enum-coverage overclaim, schema.rs:81-84). -->

# R0 REVIEW (round 6) — descriptor-builder engine — **RED**

**RED. 0 Critical / 1 Important / 0 Minor.** SPEC §7:103 (the round-5 I1 target) is now correctly honest — that instance IS closed. But the exhaustive final-sweep surfaced the **same overclaim species in a fourth, never-swept location**: `schema.rs:80-84`. Round 4 scoped its fix to the three `ir.rs` comments; round 5 was SPEC-focused; the `grammar_matches_node_kinds_hand_list` test comment was never swept. A passing 20-green suite is necessary but not sufficient — a false comment passes trivially.

## CRITICAL
None.

## IMPORTANT

### I2 (new) — `schema.rs:80-84` repeats the enum-coverage overclaim (fourth location of the same species)

`crates/mnemonic-toolkit/src/descriptor_builder/schema.rs:81-84`, in `grammar_matches_node_kinds_hand_list`:
> "// The enum→NODE_KINDS direction is guarded separately by `ir::tests::node_kinds_cover_enum`, completing the enum == NODE_KINDS == grammar chain (risk #2 freeze)."

Both clauses are the sanctioned-as-false overclaim:
1. **"the enum→NODE_KINDS direction is guarded by `node_kinds_cover_enum`"** — false. That test asserts `all_variant_samples().kind()`-set == `NODE_KINDS` — the **samples→NODE_KINDS** direction, NOT enum→NODE_KINDS. The enum→samples link is the unguarded author-discipline gap rounds 4-5 established (the exhaustive match forces a *visit*, not a vec insertion; E0004 is a property of match arms).
2. **"completing the enum == NODE_KINDS == grammar chain (risk #2 freeze)"** — asserts a complete/airtight freeze, no caveat. Contradicts the sanctioned honest language (`ir.rs:357` "**one link** of the freeze… does NOT catch a variant omitted from all three"; SPEC:103 "risk #2 **partial** freeze"). Load-bearing word-swap: `ir.rs` writes "the hand-maintained `all_variant_samples()` tag set == `NODE_KINDS`"; `schema.rs:84` writes "**enum** == NODE_KINDS". The samples→enum swap plus "completing" is the overclaim in miniature.

Present-tense source comment, no correcting bracket → live overclaim = Important.

**Fix (prose-only, mirrors `ir.rs:357-363`/SPEC:103):** reword to the honest split — `node_kinds_cover_enum` cross-checks the hand-maintained `all_variant_samples()` tag set against `NODE_KINDS`; this test cross-checks `NODE_GRAMMAR` == `NODE_KINDS`; together they catch *drift* across the three hand-lists; the enum→samples link is author discipline → a partial/drift-only freeze, not airtight (airtight needs a variant-enumerator macro). Drop "the enum→NODE_KINDS direction is guarded" and "completing the … chain."

## MINOR
None new.

## What passes

- **The round-5 I1 IS closed.** SPEC §7:103 now reads the honest drift-only guarantee, correctly attributes the two checks to the two distinct tests, no "cannot silently miss"/"forces a variant".
- **The three `ir.rs` comments remain honest** (`NODE_KINDS` :31-34, `all_variant_samples` :303-310, `node_kinds_cover_enum` :357-363 — "drift"/"author discipline"/"one link"/"does NOT catch"). `ir.rs:208`/`:309` accurate (says samples, not enum).
- **SPEC change-log history left correctly as audit trail** (line 147 round-3, superseded by :151/:153). Lines 6/98/100/114/125 "freeze the schema" = versioned-contract/fixture-freeze concept (not enum-coverage airtightness) — true. Line 107 "complete for the 5 fixtures" = fixture-scoped — true. Lines 23/33/38/55/72/79 serde/miniscript behavior — true.
- **`mod.rs:20`** "freezes the schema against reality" = fixture purpose — benign.
- **Code state structurally unchanged since round-4 code-green** (5 mod.rs fixtures + 13 ir.rs + 2 schema.rs = 20). I2 fix is comment-only.

---

**Not cleared. Phase 2 does NOT begin.** Round-5 I1 (§7:103) closed; I2 (`schema.rs:81-84`) open. Fix = single prose reword. Leave SPEC:147 (corrected history). Re-dispatch round 7 — this is the fourth round the species resurfaced; the next fold MUST do a final unified grep across all four files for `freeze|complet|chain|link|direction|guard|enum.*==.*grammar` to confirm no fifth location before re-dispatch.
