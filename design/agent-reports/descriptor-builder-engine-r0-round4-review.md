<!-- VERBATIM opus-architect R0 review, round 4, descriptor-builder engine (Release A, v0.50.0). Persisted per CLAUDE.md (architect had no Write tool). SPEC + Phase-1 impl @ post-r3-fold; source b596d3f, miniscript 95fdd1c. Verdict: RED 0C/1I/2m — I1 REOPENED (guard comments overclaim). -->

# R0 REVIEW (round 4) — descriptor-builder engine — **RED**

Preamble (architect's reasoning): The exhaustiveness check (E0004) is a compile-time property of match *arms*, structurally independent of `Vec` contents — settled Rust semantics. `main.rs:15` `#[allow(dead_code)]` masks an unused new variant, so the test is genuinely the only guard, and it has a hole.

The I1 fix is the *same species* of false-freeze-comment as the original I1. The chain:
- **enum → `kind()`**: exhaustive, compile-forced ✓ (real link)
- **enum → `all_variant_samples()` match**: exhaustive on arms, compile-forced to *visit* ✓ but does NOT force the `samples` *vec* to contain the new variant ✗ (the broken link)
- **`samples` → `NODE_KINDS`**: `node_kinds_cover_enum` asserts `produced == NODE_KINDS` — but `produced` derives from the hand-maintained `samples`, so if `samples` and `NODE_KINDS` drift *together* (both omit the variant), the assert passes
- **`NODE_KINDS` → `NODE_GRAMMAR`**: `grammar_matches_node_kinds_hand_list` asserts equality — both hand-lists, drift together passes

So a new variant that gets its forced `kind()`/`render()`/`children()`/match arms but is omitted from `samples` + `NODE_KINDS` + `NODE_GRAMMAR` compiles, deserializes, renders, and is silently absent from `--spec-schema`. **I1 is NOT closed.**

**Verdict: RED. 0 Critical / 1 Important / 2 Minor.**

## CRITICAL
None.

## IMPORTANT

### I1 (REOPENED — not closed) — the `node_kinds_cover_enum` guard does NOT force a new variant into `NODE_KINDS`/grammar; three doc-comments falsely assert it does

The round-3 fix replaced a *false comment about a nonexistent test* with a *real test carrying false comments about what it guarantees*. Same species, still open by the closure criterion ("a variant that compiles + deserializes + renders but is absent from `--spec-schema`" → if yes, I1 is not closed). It is yes.

**The broken link, traced exactly:** the compile-forcing in `all_variant_samples()` rests on the exhaustive `match n { ... }`. Rust's exhaustiveness check (E0004) is a type-level property of the match **arms**, structurally independent of the `samples` **vec** it iterates. Adding a variant `Foo` forces a new arm in `kind()`/`render()`/`children()` (real) and a new arm `| PolicyNode::Foo(_)` in the samples-helper match (real, but this only forces the author to **visit** the helper, NOT to add `PolicyNode::Foo(...)` to the `samples` vec — the match compiles with `samples` empty). Then `node_kinds_cover_enum`: `produced`(17) == `NODE_KINDS`(17) passes; `grammar_matches_node_kinds_hand_list`: `NODE_GRAMMAR`(17)==`NODE_KINDS`(17) passes; `Foo` compiles, deserializes, renders, silently absent from `--spec-schema`. Bypass confirmed.

Two aggravators: `main.rs:15` `#[allow(dead_code)]` suppresses the only other backstop (unused-variant warning under `-D warnings`) for all of Phase 1; and the doc-comments assert the airtight guarantee that does not exist (`ir.rs:298-301` "forcing the new variant into this list"; `:348-351` "a new variant cannot be silently absent from the schema"; `:29` overclaims "kept in sync by the kind match + test").

Harmless for the frozen v1 set today (17==17==17 hand-verified); severity is the false-green under CLAUDE.md anti-false-green + the invisibility-at-write-time of the future drift, exactly when risk #2 says the gate must hold.

**Fix (narrow, round-3-sanctioned route, no redesign):** reword `ir.rs:298-301`, `:348-351`, `:29` to the *actual* guarantee — the exhaustive match forces an author adding a variant to **visit** the helper; the `samples` list is hand-maintained and **bidirectionally** cross-checked against `NODE_KINDS` and `NODE_KINDS` against `NODE_GRAMMAR`; v1 is hand-verified 17==17==17. Drop "forcing the new variant into this list" and "cannot be silently absent from the schema." (A genuinely airtight enum→schema guard needs a derive/`strum::EnumIter`-class enumerator that can construct data-bearing variants — out of scope for a comment fix; the honest comment is the correct close, matching round-3's "name only the guards that exist.")

## MINOR

- **M(new)-1 — fixture render-skeleton goldens are exact and non-vacuous, but `skeleton()` masking is order-sensitive.** `mod.rs::skeleton` replaces `KEY_A..E` then strips `/<0;1>/*`. The five key constants being mutually non-substring makes it safe today; a one-line note ("key constants must be mutually non-substring") would harden it. Non-blocking.
- **M2 (carried) — citation tightening is correct.** `enumerate.rs:113-115`/`:119-120`; `derive_at_index` `descriptor/mod.rs:706`. No action.

## What passes

Everything except the I1 doc-comment honesty:
- **Render goldens real and exact** (incl. `v:pk`, `s:pk`, `andor`, `thresh(2,...)`, `or_b`); `round_trips_through_serde` closes the doc→render path; `mod.rs::fixtures_test` is a real golden (key-masked skeletons matching §6).
- **`deny_unknown_fields` cells genuinely reject and discriminate**: top-level unknown, struct-payload typo (`MultiSpec "x":1`), and the externally-tagged leaf sibling-key (`{"pk":"A","w":"v"}`) — the load-bearing single-key-rule case — all assert `Err(Json(_))`.
- **Wrong-arity cells reject** (`and_v` with 1/3, `andor` with 2) — serde `[T;N]` length check.
- **Version mismatch** returns typed `UnsupportedVersion(2)`, discriminated from Json.
- **`children()` is the right Phase-2 substrate** (deepest-first subtrees: `Wrap→[sub]`, combinators→arrays, `Thresh→subs`, leaves→[]); `SpecParseError` cleanly separates structural vs version — clean Phase-1→Phase-2 boundary.
- **5 fixtures encode the 5 §6 archetypes, all distinct-key** (tiered-recovery = `or_i(sortedmulti(2,A,B),and_v(v:older(4032),thresh(2,pk(C),s:pk(D),s:pk(E))))` verbatim; decaying-multisig carries the mandatory `v:`, uses `older`+`after` with no `HeightTimelockCombination`; hashlock pins a real 64-hex sha256) → will pass Phase-3 `sanity_check`. Correctly INPUT-only (goldens deferred to P3).
- **`grammar_matches_node_kinds_hand_list` + `schema_advertises_v1`** correctly gate the downstream link; the broken link is enum→`samples`, upstream of both.
- **SPEC edits introduce no new issue**; spot-checked citations hold (`pipeline.rs:28-30`, `enumerate.rs:113-120`, `descriptor/mod.rs:706`/`:946`, `analyzable.rs:225`/`:187-208`/`:139`/`:145`).

---

**Not cleared. Phase 2 does NOT begin.** Fix = three doc-comment rewrites in `ir.rs` (`:29`, `:298-301`, `:348-351`) to state the real guarantee and drop the two false claims — no code, no redesign. Re-dispatch round 5 to confirm closure.
