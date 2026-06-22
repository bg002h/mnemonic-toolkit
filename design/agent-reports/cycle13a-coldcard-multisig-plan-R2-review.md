# R0 REVIEW — cycle-13 Lane A PLAN-DOC (coldcard-multisig fidelity H11+H14) — Round 2

Verified against `origin/master = 9b2a8ae3`.

## VERDICT: NOT GREEN — 0 Critical / 1 Important / 0 Minor

The R1 I-1 fold is factually correct and well-executed for the in-module unit fixtures, but INCOMPLETE at the CLI integration-test layer — two `tests/cli_import_wallet_coldcard_multisig.rs` tests break under the new H14 matrix and are nowhere in the plan.

## Axis 1 — I-1 fold completeness (priority)

**Decoded facts CONFIRMED:** `XPUB_A`/`XPUB_B` depth 4, `XPUB_C` depth 3 (no depth-0 multisig const exists → `XPUB_D0_*`/`FP_D0_*` prerequisite genuinely needed). Truth-table loop `for (i, raw)` `:336` fully resolves (and can refuse) BEFORE the coin-type heterogeneity check `:438-444`. `tpub_a` in `:1418` is depth 4 (both cosigners need supplied XFP).

**The 4 named in-module breaks — each verified with correct post-change outcome:** `:990` (warn→silent, re-point depth-0); `:1042` (silent→REFUSE, SPLIT into depth-0-silent + depth>0-refuse — the `:1052` `.unwrap()` would panic without the split); `:1265` (warn→silent, re-point); `:1418` (refuses before the coin-type assertion → supply per-line XFPs). Regression-guard "stays GREEN" set verified complete.

**THE FINDING (I-1, blocks GREEN): break set incomplete across the file boundary.** `tests/cli_import_wallet_coldcard_multisig.rs` has two CLI tests that are end-to-end twins of the broken unit fixtures and flip warn→silent:
- **`coldcard_ms_xfp_header_divergence_warns_byte_exact_template`** (`:166`) — `XFP: DEADBEEF` + bare depth-4 xpub → under H14-c (depth>0 + supplied) goes SILENT → 4 `stderr.contains` warning asserts FAIL. CLI twin of unit `:990`.
- **`coldcard_ms_per_cosigner_xfp_divergence_warns_per_cosigner`** (`:209`) — `CAFEBABE: <depth-4-xpub>` → SILENT under H14-c → 3 asserts FAIL. CLI twin of unit `:1265`.

The plan's §P1 H14-h enumerates only `coldcard_multisig.rs` fixtures and tells the implementer "do NOT rely on a catch-all token scan" — but the whole-suite gate is the only thing that would catch these two → surprise RED mid-TDD (the exact hazard axis-1 exists to close). Rest of the CLI surface verified CLEAN: jade CLI tests use Row-1; `coldcard_multisig_mainnet_xpub_on_cointype1_rejects` supplies matching per-line XFPs (no regress); `cli_wallet_cross_format_convergence.rs` asserts only exit-0 + key-material (no stderr-silence assert); format-mismatch/p0c-dispatch refuse before the truth table. **Complete cross-file break set = 4 named unit fixtures + these 2 CLI tests.**

**Required:** §P1 H14-h must add these two CLI tests with the correct outcome (re-point both blobs to a depth-0 `XPUB_D0_*` so they stay meaningful Row-2/H14-d warn cases — asserts KEPT not deleted); §6 whole-diff checklist + RED #11's regression note reference the CLI layer. Mechanically identical to the `:990`/`:1265` re-point.

## Axis 2 — Minor folds (M-1/M-2/M-3): CONFIRMED
- M-1: §P4 (`:196`) mandates #15/#16 divergent blobs carry per-line `<XFP_master>:` (or depth-0 xpubs) → RED stays canonicalizer-attributable.
- M-2: §P4 (`:197`) lists the full `:1397/:1410/:1428/:1445/:1485` cluster; all five feed homogeneous supplied==computed → stay GREEN.
- M-3: §P2 #13b (`:164`) per-line `<XFP_master>` == computed `xpub.fingerprint()` of the depth-0 xpubs → no incidental Row-2 warning.

## Axis 3 — No new drift: CONFIRMED
Fold additive/surgical. P1 matrix, P2 Q1 arm, P3 H11 sorted-slot pairing, P4 canonicalizer, phase ordering (P1→P2→P3→P4, RED-first), ~16-test inventory, §0 scope/SemVer all unchanged + correct. Round-2 nits (`computed_fp :359-360`, `cs.fingerprint :366`) remain folded.

## Disposition
NOT GREEN. Fold the CLI-layer break-set gap (add the 2 `cli_import_wallet_coldcard_multisig.rs` tests to §P1 H14-h with the depth-0 re-point), persist, re-dispatch plan-R0. Core design, phase ordering, pairing rule, protocol grounding, in-module reconciliation all sound — only cross-file break-set completeness needs one more pass before TDD.
