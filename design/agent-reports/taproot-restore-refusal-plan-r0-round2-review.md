# R0 Review — taproot restore-refusal contracts (PLAN) — Round 2
Reviewer: Fable 5, 2026-06-12. Verified against mnemonic-toolkit origin/master 2f03eb0 (binary `target/debug/mnemonic` = 0.55.0).

## Verdict: GREEN (0C/0I)

## Findings (per fold: LANDED / NOT-LANDED + evidence)

**I1 — LANDED.** §1 now pins THREE bundle-reachable arms: general tr leaf + multi-leaf taptree (both → the `not a recognized multisig` arm) and the distinct-internal-key non-NUMS shape (`non-NUMS (cosigner) internal key` arm). Line citations re-verified at 2f03eb0: the `is_nums: false` match arm opens at restore.rs:685 with its message at **:689**, and the unrecognized-leaf message sits at **:710** — both as cited. §1 carries the corrected explanation for the round-1 false-negative (the earlier "bundle emits nothing" probe was tripped by the bundle-side BIP-388 distinct-key gate on `tr(K0,multi_a(2,K0,K1))`; a distinct K2 bundles fine), and only keypath-only `tr(NUMS)` (`tree: None`, blocked at bundle's origin-annotation gate) is deferred to the FOLLOWUP's T3 direct fixture. §2 adds the matching third cell `cosigner_internal_key_tr_bundles_but_restore_refuses_non_nums`. Exactly the fold requested.

**I2 — LANDED.** §2's closing paragraph now claims — correctly — that the 3-arm + faithful-backup set fully delivers `restore-general-and-multi-leaf-taproot-roundtrip`'s T1 scope-split (a): all 3 refusal arms pinned, and the multi-leaf wire round-trip delivered via cell 2's `.descriptor` exact-equality assertion on `tr(NUMS,{pk(K0),pk(K1)})` (probe-confirmed exact in round 1). The verify/address carve-out is restated with the round-1-agreed rationale (verify-bundle needs cosigner mk1 cards; restore refuses before address derivation). Registry and cycle now agree; round 1's option (a) taken, plus the §6 FOLLOWUP touch-up line.

**M1 — LANDED.** The false "modulo NUMS substitution" caveat is gone. Cell 1 (and cell 2) assert `.descriptor` == input **EXACTLY**, with the explicit note that the literal `NUMS` token is preserved verbatim on the wire (no substitution in either direction) — the exact wording round 1 prescribed for the literal-`NUMS` spelling choice. No dead normalizer invited.

**M2 — LANDED.** The "Keys" line now cites the real 3-cosigner trio at `cli_bundle_import_json.rs:312-314`, lifted as local consts; the nonexistent `XPUB4_*` reference is gone. Re-verified at 2f03eb0: lines 312/313/314 are exactly `[73c5da0a/87'/0'/0']xpub6DBji…`, `[b8688df1/87'/0'/0']xpub6Cbhr…`, `[28645006/87'/0'/0']xpub6DB7H…` — three distinct fingerprints, three distinct xpubs, all bracket-annotated `/<0;1>/*`, sufficient for every cell including the 3-key distinct-IK arm.

**M3 — LANDED.** §6 (and the §1 parenthetical) note the keypath-only `tr(<xpub>)` "multisig"-worded misleading message as a one-line touch-up on the `restore-general-and-multi-leaf-taproot-roundtrip` FOLLOWUP (T3), explicitly NOT reworded this cycle since the message is the contract being pinned. Correct resolution.

## Re-confirmations

- **Exit code:** `ToolkitError::ModeViolation { .. } => 2` at error.rs:**541** (re-grepped at 2f03eb0). Both refusal messages are `ModeViolation` constructions — exit-2 assertions correct for all three cells.
- **Substring pinning:** §5-Q2 pins `not a recognized multisig` and `non-NUMS (cosigner) internal key` as substrings, not full strings. Both substrings present verbatim in the source messages.
- **NO-BUMP / scope:** one new test file `tests/cli_restore_taproot_refusal.rs`; no `src/` change, no clap surface, no manual/GUI/schema_mirror lockstep, no new FOLLOWUPs needed (both already filed). No scope creep found — the plan's cell list is exactly the 3 arms + faithful-backup assertions; verify/address legs and the `tree: None` fixture stay deferred. No `tests/cli_restore_taproot_refusal.rs` exists yet (confirmed) — genuinely new.
- **Third-arm live re-probe (0.55.0 binary):** `tr([73c5da0a/87'/0'/0']K2…,multi_a(2,K0,K1))` with the trio → `bundle --descriptor … --network mainnet --json` exit **0**, 6-chunk md1, `.descriptor` == input **exactly**; `restore --network mainnet --md1 …×6` exit **2**, stderr contains `non-NUMS (cosigner) internal key` (full :689 message reproduced). Matches §1/§2 cell 3 precisely. Probe temp files deleted; no source edited.

## Residual

- (Minor, non-blocking) §3 mentions "the optional `md inspect` cell gates on `MD_BIN`", but §2's cell list contains no such cell — a dangling sentence from an earlier draft. Harmless either way: the implementer may add the optional MD_BIN-gated cell or skip it; neither affects the pinned contracts, scope, or NO-BUMP status. Suggest deleting the clause (or adding the optional cell) at implementation time; does not gate GREEN.
- Nothing else. Plan is internally consistent with the round-1 probe record and the live source at 2f03eb0.
