# R0 round 1 — architect review (verbatim) — SPEC_taproot_depth_ge2_experimental.md

> Persisted verbatim per CLAUDE.md (before the fold). Dispatched via Agent tool
> (feature-dev:code-architect, inherited session default model Opus 4.8). **Verdict:
> YELLOW** (0 Critical, 2 Important, 4 Minor). Source SHA `8da9008`, spec commit
> `5fc7e9e`. EXPERIMENTAL never-merge POC — release/lockstep findings excluded by design.

---

## Pre-verdict working notes (verbatim)

The spec doesn't call out updating these doc comments, but they'll be stale after the lift. This is a minor (doc comment staleness, not a correctness issue).

[`subtree_contains_sortedmulti_a` recursion check] at depth-≥2, `inner` IS a TapTree containing children. The `subtree_contains_sortedmulti_a` function at `:801-812` handles `Body::Children(c)` by recursing `c.iter().any(subtree_contains_sortedmulti_a)`. Since `TapTree` nodes have `Body::Children(c)`, this correctly recursively checks all descendants for `SortedMultiA`. Confirmed correct.

[`ensure_taptree_wellformed` recursive impl] verifies every TapTree node (at any depth) has exactly 2 children. Leaves terminate the recursion. This matches md-codec's `tree_to_taptree` structure exactly. The toolkit's `ensure_taptree_wellformed` only checks `TapTree`-tagged nodes, not the miniscript leaf nodes; miniscript leaf well-formedness is checked by `to_miniscript_descriptor` downstream. So the recursive well-formedness check is correctly scoped.

---

**R0 REVIEW — SPEC_taproot_depth_ge2_experimental.md — Round 1**

Source SHA verified: branch `experimental/taproot-depth-ge2` built from `d0201d5` (1 behind `8da9008`). All citations re-grepped against current source.

### CRITICAL
None.

### IMPORTANT

**I1.** The spec's §6 "retained funds-safety nets" item 4 states the `@-in-both` guard is "untouched; still applies to the Template multisig path." Accurate, but the spec does NOT note that at depth-≥2 the guard is structurally unreachable — a depth-≥2 tree always has `inner.tag == TapTree` (not MultiA/SortedMultiA), which sends it through the `_` arm of `classify_taproot_restore` (`:740`) and never reaches `refuse_at_in_both`. The spec should confirm this is by design and correct: the general arm reconstructs via `faithful_multisig_descriptor` → `to_miniscript_descriptor`, which reads the actual tree, so the Template-path's Cosigner-exclusion blind spot does not apply. The existing code comment at `:775-776` already says "General-arm leaves never reach this helper (they reconstruct via the route-around, which reads the ACTUAL tree)" — but the spec omits this justification, creating a false-alarm risk during review. **Fix:** add the justification to §6 item 4.

**I2.** The advisory emission point is underspecified. `classify_taproot_restore` and `ensure_taptree_wellformed` are pure functions with no access to `stderr`; `stderr` lives in `run_multisig`. The spec does not specify WHERE in `run_multisig` the advisory fires or how the "has depth-≥2" fact reaches the caller. **Fix:** specify a standalone `fn taproot_is_deep(inner: &Node) -> bool` (one-level check: any TapTree child is itself a TapTree — enough to detect depth-≥2 without a full re-walk), called in `run_multisig` after `classify_taproot_restore` returns `GeneralFaithful`, emitting the advisory where `stderr` is in scope (the existing `older()` advisory emit at `:1373-1374` is the natural home). Do NOT add a third `TaprootRestore` enum variant.

### MINOR

**m1.** Stale doc comments will remain after the lift: `TaprootRestore::GeneralFaithful` (`:668` "depth-1 two-leaf"); `classify_taproot_restore` doc (`:686-695` "depth ≥2 … refuses"); test module comment (`:16-19` "depth ≥2 … lift on the #953 release"). Add to the implementer checklist.

**m2.** The `ensure_taptree_depth_le_one` → `ensure_taptree_wellformed` rename also requires updating the call site (`:748`) and the fn's own doc (`:814-818`). Note explicitly.

**m3.** After bumping the rev in both `Cargo.toml:29` + `fuzz/Cargo.toml:28` and refreshing both lock files, run `cargo test --workspace` (root) AND `cargo build` (fuzz/) BEFORE writing restore.rs code — the lock-file refresh is a prerequisite for a meaningful suite. The spec's §7 says "full suite green" but not the ordering: rev bump + lock refresh FIRST.

**m4.** The advisory is non-blocking (exit 0); the test assertion is `.success().stderr(predicate::str::contains("EXPERIMENTAL"))`, NOT the exit-2 refusal pattern the old cells use. Minor clarity gap.

### Funds-safety verdict (the crux, answered)
The Display-fidelity guard at `:1365` IS genuinely sound and sufficient: classify (no depth refusal) → `faithful_multisig_descriptor` → md_codec `to_miniscript` → miniscript Display (#953-fixed) → parse→print fidelity check → address derivation. No depth-≥2 shape reconstructs to a wrong-but-parse-stable descriptor that the guard would miss. The structural depth cap was a conservative superset; removing it with the guard retained is safe. export-wallet (no explicit depth gate; miniscript-Display-dependent), `parse_descriptor::walk_tap_tree` (already multi-leaf), and bundle (already engraves depth-≥2) need no change — confirmed.

### VERDICT: YELLOW
Two Important (I1, I2) block GREEN; neither is a correctness flaw in the funds-safety argument. Both have clear fixes. Fold and re-submit for round 2.
