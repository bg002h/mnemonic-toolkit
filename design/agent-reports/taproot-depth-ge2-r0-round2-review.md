# R0 round 2 — architect review (verbatim) — SPEC_taproot_depth_ge2_experimental.md

> Persisted verbatim per CLAUDE.md. Dispatched via Agent tool (feature-dev:code-architect, Opus 4.8).
> **Verdict: GREEN (0 Critical, 0 Important, 1 Minor).** Source SHA `8da9008`, spec commit post-r1-fold.
> The mandatory R0 gate is MET. The 1 Minor (m-r2-1, `taproot_is_deep` body clarity) folded into §4.

---

## R0 Round 2 — Review

**Source verified:** `experimental/taproot-depth-ge2` branch, built from `d0201d5`. All citations re-grepped against the actual branch source.

### CRITICAL — None.
### IMPORTANT — None.

### MINOR

**m-r2-1.** The `taproot_is_deep` predicate takes `&d.tree` (the `Tag::Tr` root node) but the description says it probes "the `Tag::Tr` node's tree child" — a minor off-by-one in the spec's description. The predicate must first extract `inner` (the `tree: Some(inner)` child) from the `Tr` body before checking whether that child is a `TapTree` containing another `TapTree`. Resolved by the pattern already established in `classify_taproot_restore` (`match &tree.body { Body::Tr { tree: Some(inner), .. } => ... }`). **Fix:** add one sentence to §4: the predicate's body matches `Body::Tr { tree: Some(inner), .. }` on its argument, then checks `inner.tag == Tag::TapTree && inner has a TapTree child`. Clarity, not correctness.

### Fold verification (I1, I2, m1–m4)
- **I1 — LANDED AND CORRECT.** §6 item 4 now justifies: depth-≥2 always has `inner.tag == TapTree` → `_` arm at `:740` → never reaches `refuse_at_in_both`; general arm reconstructs via `faithful_multisig_descriptor` reading the actual tree. Source confirms `refuse_at_in_both` is only called in the `MultiA`/`SortedMultiA` arms (`:727`,`:734`). No blind spot.
- **I2 — LANDED AND CORRECT.** §4 specifies `fn taproot_is_deep(tree: &Node) -> bool` called at `:1373-1374` inside `if is_taproot || template_opt.is_none()`, guarded `if is_taproot && taproot_is_deep(&d.tree)`. Feasibility verified: `d` bound at `:1228`, `d.tree` is `Node`, `stderr` is a `run_multisig` param, the emit point is AFTER the fidelity guard at `:1365`. No new enum variant needed (`is_taproot` already set at `:1281`).
- **m1 — LANDED.** Three stale doc locations listed (`:668`, `:686-695`, test `:16-19`).
- **m2 — LANDED.** Rename targets: call site `:748` + fn doc `:814-818`.
- **m3 — LANDED.** Rev bump + lock refresh + `cargo test --workspace`/fuzz build BEFORE restore.rs.
- **m4 — LANDED.** Advisory test `.success().stderr(contains("EXPERIMENTAL"))`.

### Load-bearing checks
1. **Funds-safety after folds:** unweakened. Display-fidelity guard at `:1365` unchanged; `subtree_contains_sortedmulti_a` at `:741` unchanged; malformed-tree refusal preserved inside `ensure_taptree_wellformed`. The fidelity guard fires strictly LATER than the removed structural cap, so removing the cap cannot create a silently-wrong path the cap blocked but the guard misses. Sound.
2. **`taproot_is_deep` placement:** fires exactly when `is_taproot` AND depth-≥2 (a `TapTree` inside a `TapTree`), which implies `classify_taproot_restore` returned `GeneralFaithful`. Template arms have `inner.tag == MultiA/SortedMultiA` → predicate returns false → advisory does NOT fire for depth-1 Template. No false positives, no missed fires. (Note: Template-taproot arms DO enter the `if is_taproot || template_opt.is_none()` block since `is_taproot` is true, but the predicate correctly returns false for them.)
3. **`d.tree` vs `inner`:** `d.tree` is the `Tag::Tr` root; the predicate extracts `inner` internally (one level beyond `ensure_taptree_wellformed(inner)`). Consistent; the m-r2-1 sentence eliminates the ambiguity.
4. **Blocking ambiguity:** none — the `classify_taproot_restore` pattern is a direct template.

### VERDICT: GREEN
0 Critical / 0 Important. All r1 folds landed correctly against source; the funds-safety argument is sound; the I2 wiring is feasible and fires at exactly the right point. Implementation may proceed.
