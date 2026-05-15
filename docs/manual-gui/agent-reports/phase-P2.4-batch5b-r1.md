# Phase P2.4 sub-batch 5b — R1 opus architect-reviewer

**Date:** 2026-05-15
**Branch:** `manual-gui-v1`
**Scope:** Verify R0 folds (4C/1I/2n) for `42-bundle.md` + GUI FOLLOWUP filing.

**Verdict:** **LOCK 0C / 0I / 0N / 0n.** All 7 folds byte-verified.

## Per-fold verification

| Fold | Status | Evidence |
|---|---|---|
| **C-1** JSON envelope | PASS | `42-bundle.md:322-337` shows `"schema_version": "4"` with all 14 fields in canonical order matching `format.rs:120-145` `BundleJson`. |
| **C-2** SlotSubkey 8 variants | PASS | `42-bundle.md:436` enumerates 8 subkeys including `master_xpub`; historical-7 parser-refusal-message lag correctly footnoted. |
| **C-3** Refusals table | PASS | All 7 mode-violation rows + 1 clap-conflict row are byte-exact mirrors of `cmd/bundle.rs:91-101` `mode_text::*` constants. |
| **C-4** `--account` body refusal | PASS | Line 310 has byte-exact `DESCRIPTOR_WITH_NONZERO_ACCOUNT` on a single line. |
| **I-1** Worked-example invocation | PASS | Step 3 instructs clearing `--multisig-path-family`; cites FOLLOWUP `gui-bundle-multisig-flags-conditional`; Preview line omits the flag. GUI FOLLOWUP filed at `mnemonic-gui/FOLLOWUPS.md:170` under `## Deferred to v0.3+`. |
| **n-3** Threshold range | PASS | Line 418 "Allowed range 1 to 16 inclusive" matches `BUNDLE_FLAGS:228` `Number { min: 1, max: 16 }`. |
| **n-4** Danger admonition link | PASS | Top admonition has markdown link to §14 Defense 2. |

## Lint + build state

- Phase 4 schema-coverage RED at **403 missing** (unchanged; folds were prose-only).
- Phase 5 outline-coverage RED at **52 missing** (unchanged).
- Phases 1-3 GREEN.
- PDF 64 pages.

## Cross-repo lockstep

GUI FOLLOWUP `gui-bundle-multisig-flags-conditional` filed under `## Deferred to v0.3+` (tier `v0.4`); cites the manual chapter's worked-example workaround. No toolkit-side companion needed (this is a GUI conditional-visibility enhancement, not a bidirectional manual lockstep concern).

**LOCK — proceed to commit.**
