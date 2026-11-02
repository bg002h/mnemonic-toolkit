**⚠ Critical and Important confirmed by RED rows on unmodified code (C1: seed --path ignored; I1: xprv --path ignored).**

# FIX document report: tk `convert --path`

FIX doc: `design/FIX_convert_path.md`. RED file: `crates/mnemonic-toolkit/tests/cli_convert_path.rs`. Commit `77a59eab` on `triage/convert-path` (base `6ccfdebc`), not pushed. No production code is committed.

## Findings

- **C1 (Critical), CONFIRMED.** On phrase, seedqr and entropy sources, `--to xpub/xprv` ignore `--path`. With `--template` they print the template key and exit 0; without it they refuse with `--template is required…`. Rows: the `r1_*` rows, `convert_template_xpub_path_not_silently_ignored`, `r2_path_fingerprint_is_master` and `r5_bare_m_is_master`.
- **I1 (Important), CONFIRMED.** `--from xprv=<tprv> --to xpub --path m/0` exits 0 and prints the parent xpub. Row: `r4_xprv_to_xpub_path_refused`. The other R4 rows show that `--path` is silently accepted on every non-reading edge.
- **M1 (Minor), folded (R5).** `--help` lists named and hex path forms that `DerivationPath::from_str` refuses. The help rows and the `r5_named/hex` rows fail today.

## Evidence

- RED on unmodified code: 27 run, 2 passed (both guard rows), 25 failed. Capture: `/scratch/cache/convert-path/red-final.txt`, sha256 `cdc96503…2611`.
- GREEN with the reference patch `/scratch/cache/convert-path/fix-reference.patch` (sha256 `835f1326…cc43`, applies at HEAD): 27/27.
- Workspace on the patched tree: 4171/4171 passed, 38 skipped. clippy `-D warnings` and fmt are clean.
- Tests that change: none.
- Spec anchors S1-S8 each match exactly once.
- cargo is 1.85.0, matching the pin.

## STOPs (architect)

1. R3 vs R4 order when both apply. The proposal is R4 first.
2. R3/R4 vs the `--xpub-prefix` refusal (5.a). The proposal is R3/R4 first.
3. R3 vs the R5 parse error. The proposal is R3 first, which is the natural order because the path is parsed later.

The RED rows avoid all three overlaps.

## Environmental notes

The `docs/manual` lint, verify-examples and anchor-check fail locally for reasons outside this change: the sibling binaries are not at their pins, and the mk worktree is missing. In the FIX gate these are CI items.

Follow-ups F1 (`convert-path-fix`, owed-to-build) and F2 (GUI manual omits the R3/R4 refusals, deferred-minor) are written in FIX section 9 for the builder to append.

Counts: Critical 1, Important 1, Minor 1 (folded), Nit 0; STOPs 3.
