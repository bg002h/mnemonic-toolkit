# Post-implementation whole-diff review — verify-bundle exit-tier (pre-tag v0.97.0)

Reviewer: fable (independent). Date: 2026-08-03. Diff under review: `4024a119`.

**PERSISTENCE-ORDER VIOLATION (recorded, not hidden):** this report was written to
disk AFTER its fold was committed, violating the project MUST to persist verbatim
BEFORE the fold-and-commit step. It is the SECOND such violation in this cycle —
R0 round 2 flagged the identical failure for R0 round 1, and it recurred anyway.
Caught by the post-impl convergence check, which also found the amended commit
message had upgraded "three reviews persisted" to "four" without the fourth file
existing. See `verify-bundle-exit-tier-postimpl-convergence.md`.

## VERDICT
DO NOT SHIP — production code, docs, and behavior are fully conformant and the suite is green, but one Important execution gap remains: SPEC §6 T7's mandated cell pinning `Bip388VerifyDistinctness` → exit 4 on the verify path was never written, so one of the three exit-4 meanings v0.97.0's CHANGELOG and manual now advertise is unpinned at the CLI level (test-only fix, ~15 lines; behavior itself verified correct by live probe).

## Q1 SPEC CONFORMANCE
Walked SPEC §3's table against `git show 4024a119` and the live tree, site by site (old v0.96.0 line → live line in `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs`):

| SPEC site | Change specced | Implemented | Live line | Detail preserved |
|---|---|---|---|---|
| `:1371` file read | construct `DescriptorParse` | yes | `:1374` | byte-identical `--descriptor-file {}: {e}` (matches emit twin) |
| `:1445` lex | delete `.map_err` re-wrap | yes | `:1450` | native `DescriptorParse` message propagates (spec-sanctioned prefix loss, pinned by T1's negative assert) |
| `:1449` resolve | delete re-wrap | yes | `:1451` | native |
| `:1468` probe | delete re-wrap | yes | `:1468` | native |
| `:1514` slot-missing (dead) | construct `DescriptorParse` | yes | `:1517-1519` | byte-identical, with the specced BELIEVED-DEAD comment |
| `:1678` slot subkey | construct `DescriptorParse` | yes | `:1683-1685` | byte-identical |
| `:1722` final parse | keep at exit 4, defensive | yes | `:1737-1741`, unchanged + honest defensive comment | n/a |

Exactly one `DescriptorReparseFailed` construction site remains in `crates/` (`verify_bundle.rs:1738`); `error.rs` changed rustdoc only; no unauthorized site touched — the lookalike `bundle.rs` `--import-json` re-parse message is untouched. Exit map unchanged: `DescriptorParse => 2` (`:627`), `DescriptorReparseFailed => 4` (`:628`).

## Q2 BEHAVIOURAL REGRESSIONS
Full suite: exit 0, 211/211 result lines, zero failures. Live probes against a version-checked 0.97.0 binary:
- **Card mismatch → 4:** minted a real 2-of-2 bundle (emit exit 0), verified with a wrong-key md1 → exit 4, `result: mismatch`.
- **`result: partial` → 4:** all 9 cells of `cli_verify_bundle_partial.rs` green.
- **`Bip388VerifyDistinctness` → 4:** duplicate `@0.phrase == @1.phrase` → exit 4, `error: bundle violates BIP-388 distinct-key rule`. Sites `:1432`/`:1721` untouched.
- **Accept/reject boundary:** correct cards + descriptor → exit 0 `result: ok`; all six flipped classes still refuse (exit 2). No check lost — the three deleted wrappers became `?` propagation of the same error.

## Q3 TEST QUALITY
Mutation-proved five sites in a scratchpad worktree; every mutation restored the exact pre-change wrapper, COMPILED, and went RED with per-cell precision: M1 lex `:1450` → t1 + t6 + the flipped cycleA cell RED (with the restored prefix in stderr); M2 probe `:1468` → only t3; M3 subkey `:1683` → only t4; M4 file `:1374` → only t5; M5 resolve `:1451` → only t2.

`t7_no_collateral_result_tier_still_exits_4` is not vacuous — its exact command exits 4 with `result: mismatch` — but it asserts only `assert_ne!(Some(2))`, so a regression to 1/3/5 would pass.

## Q4 DOC ACCURACY
Non-negotiable holds: no user-facing doc presents "completed wallet failed re-parse" as a live exit-4 meaning. The two technical-manual rows keep the variant with the specced "believed unreachable, retained defensively" narrowing. CHANGELOG meaning set is the complete three-class set with the `$?`-migration note. The two manual paragraphs now agree with each other and the code; the old `:231` self-contradiction is resolved. Residual exit-4 mentions elsewhere are genuine result-tier meanings.

## CRITICAL
None.

## IMPORTANT
**I-1. SPEC §6 T7's second mandate — "Add a cell pinning `Bip388VerifyDistinctness` still exits 4, since §5 now advertises it" — was not implemented.** `cli_verify_bundle_exit_tier.rs` has 7 cells; none exercises distinctness (t7 uses distinct phrases). The only pin is the variant-level unit test (`error.rs:1368-1388`), which asserts the enum→4 mapping but would stay green if the verify sites were re-tiered to `Bip388Distinctness` (exit 2); `cli_bip388_distinctness.rs:4-9` explicitly defers CLI-level verify coverage to a phase that never landed. v0.97.0 newly ADVERTISES this meaning, so shipping it unpinned recreates the advertised-but-unpinned shape this cycle exists to close. Behavior correct today (live probe). Remedy: one cell. The commit message's "T7 pinning that the result tier is untouched" overstates what shipped.

## MINOR
1. `t7` asserts only `assert_ne!(code, Some(2))` while its name claims "still exits 4"; tighten to `assert_eq!(Some(4))`.
2. `cli_verify_bundle_entropy_slot.rs:6` module doc still names the catch-all as `DescriptorReparseFailed`, exit 4 — SPEC §4 listed it for lockstep update; only the cycleA one was done.
3. `cli_cycleA_phase2_funds_proof.rs:18` module doc still says the `@N` fork → exit 4, contradicting the flipped cell 200 lines below in the same file.
4. `41-mnemonic.md:202`'s `[VERIFY-ME tier](#exit-codes)` anchor resolves to the page's first "### Exit codes" heading (the import-wallet table), not a global tier definition. Cosmetic.

## PROOF OF WORK
(Full section in session transcript. 53 tool uses. Key: whole diff read across 20 files; repo-wide greps for construction sites and doc references; full suite 211/211; 7 live probes; 5 compiling mutations each RED-ing exactly its cell; release-ritual version sites verified.)
