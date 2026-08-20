# Final whole-branch review (verbatim) — experimental depth-≥2 taproot POC

> Persisted per CLAUDE.md. Dispatched via Agent (feature-dev:code-reviewer, Opus 4.8). Full code diff
> `d0201d5..f3ec461`. **Verdict: SHIP-READY AS A POC — 0 Critical / 0 Important / 0 Minor** (one
> sub-threshold observation noted as a non-bug). EXPERIMENTAL/never-merge — not tagged, not merged.

---

## Strengths
- **Funds-safety chain sound.** Every depth-≥2 path → `GeneralFaithful` → `faithful_multisig_descriptor` → `to_miniscript_descriptor` (ff4732e #953-fixed Display) → `translate_pk` → `to_string()` → parsed back → `parsed.to_string() != descriptor` fidelity guard (`restore.rs:1381`) BEFORE any address derivation/output. If ff4732e still mis-renders a shape, the guard catches it. No silently-wrong reconstruction reachable.
- **Gate sequencing correct.** `_ =>` arm calls `subtree_contains_sortedmulti_a` then `ensure_taptree_wellformed`; depth-≥2 always has inner tag `TapTree` → falls to `_ =>`; `refuse_at_in_both` (Template-only) structurally unreachable at depth-≥2 (correct).
- **`taproot_is_deep` correct** (false for Template/keypath-only/non-Tr/depth-1; true only depth-≥2). Advisory fires iff a depth-≥2 shape reconstructed, before `import_payload` (so it precedes any `--format` output).
- **`ensure_taptree_wellformed` correct** (terminates at leaves; validates exactly-2-children at every TapTree; recurses both children; no panic/unwrap; depth bounded).
- **Internal consistency clean:** no surviving inaccurate `ensure_taptree_depth_le_one` / "depth-≥2 refused" in `crates/`+`Cargo.toml`; the 3 `95fdd1c` refs in `crates/` are accurate context (master stays on 95fdd1c; md-codec two-miniscripts split); `fuzz/Cargo.toml` pin matches root; both lockfiles resolve `ff4732e`; EXPERIMENTAL.md + Cargo banner + clap help (`:67-68`) + inline comments (`:1277`,`:1281`) all accurate.
- **Never-merge signals loud + sufficient** (EXPERIMENTAL.md + Cargo banner ×2 + runtime advisory; no tags; master untouched).
- **Test coverage genuine:** 3 depth cells (left-heavy, right-spine, depth-3 4-leaf) assert success + golden desc + golden addr + `stderr(contains("EXPERIMENTAL"))`; K3 distinct (account 1); depth-3 exercises a 3rd recursion level; pre-existing refusal cells unchanged + green.

## Issues
**Critical — None. Important — None. Minor — None above threshold.**
(Sub-threshold, conf ~55, recorded as a NON-bug: the `_ =>` arm doesn't pre-check the subtree root for `sortedmulti_a` before `ensure_taptree_wellformed` — but a root `SortedMultiA` is caught by the `SortedMultiA =>` arm above, and `subtree_contains_sortedmulti_a` recurses all descendants. Not a bug.)

## Per-question answers
**(a) Silently-wrong path:** None. Fidelity guard (`:1381`) fires before any address/output; Template path Display-stable + unreachable at depth-≥2; @-in-both guard Template-only (single-leaf).
**(b) Internal consistency:** the whole diff agrees — ff4732e pin, depth-≥2 reconstructs w/ advisory, never-merge. The 3 surviving `95fdd1c` refs are intentional accurate context.
**(c) Blockers to "done":** None. POC builds, tests prove reconstruction, never-merge markers adequate, no release actions required.

## Assessment: SHIP-READY AS A POC.
