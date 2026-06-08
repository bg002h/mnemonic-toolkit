# R0 Architect Gate — Round 2 — SPEC_trmultia_nums_internal_key.md

> Round 1 = 0C/2I/3M; all folded. Reviewer had Read/Grep across both repos. Source basis: HEAD `b642fbe`.

## Verdict: GREEN (0C / 0I / 0M-new)

All Round-1 folds landed correctly + consistently; the I1/I2 reframes are complete with no residual old framing; every line citation re-verified against source. SPEC is implementation-ready.

## Critical / Important
None. The claim that could have re-opened I1 — "the live execution tests keep passing, `is_nums`-independent" — was verified empirically: `cli_restore_multisig.rs::tr_multisig_refused_exit2` (`:240-251`) asserts only `code(2)` + stderr `"taproot"`/`"restore-multisig-taproot-reconstruction"` (refusal fires at `restore.rs:777` Tr pre-gate, internal-key-independent); `cli_tr_bip48_advisory.rs` ×4 assert only success + the path-derived `ADVISORY` substring, never a descriptor/policy_id/prefix. Both genuinely `is_nums`-independent.

## Minor (non-blocking, cosmetic)
- Item 4 cites `build_nums_internal_key` as `:161-165` (the call-site branch it describes; definition is `:183`) — correct for the branch.
- `restore.rs:777` bare basename — one `restore.rs` exists (`src/cmd/`), unambiguous.

## Fold confirmation
- **Citation fix:** CONFIRMED. `template.rs:209` = `is_nums: false,` (TrMultiA/TrSortedMultiA arm); `:208` = comment; `:150` = the **`CliTemplate::Bip86`** single-sig tr arm (`:141-154`) — the CAUTION correctly forbids touching it; line-anchored `209s/` is right; `:446` = `assert!(!is_nums,…)` flip target.
- **I1 (Item 3+4):** CONFIRMED + COMPLETE. Orphan claim empirically true (no harness reads `vectors/v0_2/tr-*-0-false-false.txt`; `cli_bundle_full.rs:17` reads only `v0_1/`, `cli_self_check.rs:13` only `bip84-`). Reframe decisive; Item 4 mandatory/gating; its literal `50929b74…803ac0` == md-codec `NUMS_H_POINT_X_ONLY_HEX` (`to_miniscript.rs:33-34`) exactly.
- **I2 (Item 6):** CONFIRMED + COMPLETE. Manual already documents NUMS (`33-taproot-multi.md:13-14,:50,:54-58,:83-85`, self-check `:70-81`/`:84`). Positive-verification-only; no residual "if prose says @0 update it".
- **M1:** CONFIRMED. Empirical wire delta + whole-bundle (md1 AND mk1) propagated to Disposition, Item 3 §2, §7.
- **M2:** CONFIRMED. §7.5 schema_mirror rationale (flag-NAME parity + value-enums only; wire change touches neither).
- **M3:** CONFIRMED. Item 5 names `restore.rs:777` (`if d.tree.tag == md_codec::Tag::Tr {`) + mandates the `Companion:`-cross-cited md-codec SortedMultiA FOLLOWUP.

## Round-1 substance intact
`is_nums:true` valid (`validate.rs:94` gates NUMSSentinelConflict on `!*is_nums`); tr-multi-a renders (`to_miniscript.rs:161-162,394-398`) / tr-sortedmulti-a errors (`:406-410`); no backward-compat break; MINOR `0.47.4→0.48.0` correct; restore re-scope accurate (both FOLLOWUP entries exist).

## Implementation-readiness: YES
Item 3 forces decisive disposal of orphaned goldens + Item 4 supplies the only real `is_nums` pin; the CAUTION + line-anchored-edit prevents the `:150` Bip86 mis-edit.
