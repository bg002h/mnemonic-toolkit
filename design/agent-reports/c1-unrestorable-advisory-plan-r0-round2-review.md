# C1 (unrestorable-shape advisory) plan-R0 round 2 — architect confirmation (verbatim)

> Reviewer: opus architect (general-purpose). Confirms the round-1 folds (I1, I2, M1/M2/M3) on
> `design/PLAN_C1_unrestorable_shape_advisory_2026-06-16.md` @ toolkit `342b5c1`. Verdict RED
> (0C/1I) — a single stale hook-count; folded immediately after (see footer) → converged 0C/0I.

---

**Verdict: RED (0C/1I)**

**Important — I2 fold introduced a stale hook-count contradiction (the C2-class defect).**
`PLAN_C1_…:192` (Execution step 3) read "Implement the module + **3 hooks**," but the I2 fold raised
the hook total to **4** everywhere else — Citations `:56` ("4th — R0-r1 I2"), Hook section `:129`
("**Hook (4 sites)**"), Non-vacuity `:160` ("revert the **4 hooks**"). An implementer executing step 3
literally would build only the 3 bundle hooks and drop the `import_wallet.rs:1291` hook that I2
mandates — silently re-opening the zero-warning import-wallet gap I2 exists to close. Fix: line 192 →
"module + 4 hooks." Re-dispatch after the fold.

Confirmation of the other 4 points (all correct):

1. **I1 (shape-1 predicate completeness) — CORRECT.** Verified against md-codec 0.36.0 source: exactly
   three SortedMulti acceptance arms — `new_wsh_sortedmulti` (:205), `new_sh_wsh_sortedmulti` (:231),
   `new_sh_sortedmulti` (:248, bare-P2SH). The plan's predicate lists (a)/(b)/(c) matching all three;
   the clean-negative TDD list and module-unit list both include `sh(sortedmulti(2,@0,@1))`. Toolkit
   emit-shape for (c) proven by `parse_descriptor.rs:1511 walk_sh_sortedmulti_root` (root `Tag::Sh`,
   `children[0].tag == Tag::SortedMulti`, no intervening Wsh). Algorithm descends past the recognized
   SortedMulti for all three sole-child shapes (false), fires only on combinator-leaf.
2. **I2 (scope) — CORRECT (apart from the line-192 count).** Both `bundle` + `import-wallet`, import-
   wallet rationale (`import_wallet.rs:1439 synthesize_descriptor` → `:1532 md1`, `p.descriptor:
   md_codec::Descriptor :1289`, hook `:1291`). Re-verified on disk: `:1290-1291` IS the older()
   `older_advisories_tree(&p.descriptor)` site; `:1439` synth; `:1532` `md1: bundle.md1`. Hook section
   + Citations list 4 sites; TDD cell 3 covers import-wallet bitcoin-core `/*h` parity; manual +
   FOLLOWUP + lockstep reflect both surfaces. export-wallet/inspect/repair/convert correctly excluded.
3. **M1/M2/M3 — all reflected.** M1: `pub(crate) fn tree_has_sortedmulti_in_combinator`. M2: shapes 2/3
   field reads covered at CLI layer, not module unit. M3: "at most one entry per shape."
4. **No design regression.** Mirrors `timelock_advisory.rs` (`emit_advisories :102`, `older_advisories_
   tree :187`, `older_advisories_node :193` `pub(crate)`); PATCH→v0.57.1; no schema_mirror/no
   ToolkitError; full lockstep incl. BOTH READMEs + fuzz/Cargo.lock; GAP-3 `.success()` guard intact.
   No "bundle ONLY"/"two positions" leftovers — the only residual was the line-192 "3 hooks."

---

## FOLD + CONVERGENCE (post-round-2, by implementer)

- **I2-residual fixed:** `PLAN_C1_…:192` → "Implement the module + **4 hooks (bundle ×3 + import-wallet
  ×1)**." Grep-verified no remaining stale total-count: all C1-total references read "4 hooks"/"4 sites"
  (`:56`, `:129`, `:160`, `:192`); the one "3 sites" at `:45` correctly describes the older() PRECEDENT's
  3 bundle sites (accurate — C1 adds the 4th import-wallet hook on top).
- The round-2 Important was a single stale count (documentation-faithfulness), the same C2-class
  defect. With it resolved and round-2's 4 substantive confirmations standing, **C1 converges to
  0 Critical / 0 Important.** Implementation unblocked.
