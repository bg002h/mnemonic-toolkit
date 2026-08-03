# Post-implementation convergence check — verify-bundle exit-tier (pre-tag v0.97.0)

Reviewer: fable (independent). Date: 2026-08-03. Diff under review: `7754cfac` (amends `4024a119`).

## VERDICT
DO NOT SHIP — the code-side fold is fully verified (t7b real and mutation-proved, all minors fixed, suite green), but the amended commit message claims "All four reviews persisted verbatim" while the post-implementation whole-diff review (the DO-NOT-SHIP one this fold answers) is nowhere in `design/agent-reports/` — a false audit-trail claim in the release commit plus a violation of the project's persist-before-fold MUST; remedy is a two-minute persistence + message fix, no code change.

## I-1 RESOLUTION
`t7b_bip388_distinctness_still_exits_4` exists at `cli_verify_bundle_exit_tier.rs:227-248`. It genuinely exercises distinctness: an `@N`-template descriptor with the same phrase in both slots passes the concrete-fork bypass and the intake gates, then hits `check_key_vector_distinctness` → the re-wrap at `verify_bundle.rs:1720-1722`. It asserts BOTH `assert_eq!(code, Some(4))` AND `stderr.contains("BIP-388 distinct-key rule")` — and that substring discriminates the variant ("distinct-key **rule**" appears only in `Bip388VerifyDistinctness`'s message, `error.rs:752`; the exit-2 twin says "distinct-key **violation**", `error.rs:749`).

Mutation: in a detached worktree at `7754cfac`, re-tiered `:1721` to `Bip388Distinctness { i: 0, j: 1 }` (exit 2). The mutation COMPILED (13.18s), t7b went RED (`left: Some(2), right: Some(4)`), the other 7 cells stayed green. Non-vacuous, confirmed. Baseline pre-mutation: full suite exit 0, 211 result blocks, zero failures.

## MINORS 1-3
- **MINOR-1 verified.** t7 now `assert_eq!(code, Some(4))` (`:211-215`).
- **MINOR-2 verified.** `cli_verify_bundle_entropy_slot.rs:6` names the catch-all as `DescriptorParse`, exit 2 since v0.97.0 — matches code (`verify_bundle.rs:1683`, `error.rs:627`).
- **MINOR-3 mostly verified, one residual.** The module doc is updated and agrees with the flipped cell. But the FIRST cell's assertion message at `:176-179` still reads "exit 4 is only the @N-template verify fork" — pre-existing v0.96.0 text, not fold-introduced, contradicting v0.97.0 inside the very file MINOR-3 claimed to make internally consistent. Failure-message-only.

## NEW COLLATERAL FROM THE FOLD
One item. The diff is exactly the four claimed fixes (3 test files, nothing else), but the rewritten commit message added **"All four reviews persisted verbatim" — unsupported**: only the three `r0-round-{1,2,3}` files exist; no file in `design/` contains "DO NOT SHIP"; prior cycles' postimpl reviews all have on-disk files, this cycle's did not. The pre-fold message ("All three reviews persisted") was accurate; the fold upgraded the count without landing the fourth file. The session transcript is exactly what the project rule says is insufficient.

The other rewritten claim — all three exit-4 meanings CLI-pinned — **is supported**: cards-mismatched by t7 (`eq Some(4)`), `result: partial` by `cli_verify_bundle_partial.rs`, distinctness by t7b (mutation-proved).

## CRITICAL
None.

## IMPORTANT
1. **Post-impl whole-diff review not persisted; release commit claims it is.** Fold committed before persisting, violating the persist-before-fold MUST — this cycle's own R0 round 2 flagged the identical violation for round 1. Remedy before tag: persist the review verbatim and make the message true. No code change; everything code-side is SHIP-clean.

## MINOR
1. `cli_cycleA_phase2_funds_proof.rs:176-179` — stale assertion message contradicts v0.97.0; pre-existing, fix opportunistically.
2. t7b pins the template-fork construction site (`:1721`); the concrete-fork site (`:1432`, `dup_xpub_path`) has no CLI-level pin. SPEC §6 T7 mandated "a cell" — satisfied as written; noting the uncovered twin.
3. MINOR-4 carried from the whole-diff review (manual `#exit-codes` anchor resolves to the wrong table on the same page): **agreed, can ship.** Cosmetic; the sentence carries its meaning in prose.

## PROOF OF WORK
(Full section in session transcript. 58 tool uses. Key: `git diff 4024a119 7754cfac` = 3 test files, 40/6; baseline suite 211 blocks zero failures; compiling mutation RED-ing exactly t7b; a mutant binary uplifted into the shared `target/debug` by the worktree build was detected via live probe + embedded-path grep, purged, and rebuilt before re-verification; persistence sweep — `ls -t design/agent-reports/`, repo-wide grep for "DO NOT SHIP"/"whole-diff"/"post-impl", `find -newermt 2026-08-03` — found only the three r0-round files.)
