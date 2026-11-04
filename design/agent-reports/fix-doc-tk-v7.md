# FIX-doc author report: toolkit V7 (bundle card recovery text)

FIX document: `design/FIX_v7_bundle_recovery.md` (branch `fix/v7-bundle-recovery`, base `ac3ff70a`; doc head `f22a4b8f`).
Evidence: `design/agent-reports/fix-doc-tk-v7-evidence/` (row names, nextest filter, hunks `fix-v7-docs.patch`, `examples.diff`, `fix-v7-changelog.patch`).

## What the doc holds

- Section 2 gives the spec delta with exact texts for rulings 1-3:
  - The plain predicate: the only child of wsh/sh/sh(wsh) is multi, sortedmulti, multi_a, sortedmulti_a or a pure-key thresh; or tr has a NUMS internal key and one such leaf.
  - The general-policy card has no Threshold line and prints the exact general Recovery text.
  - JSON `multisig.threshold` is `null` for a general policy.
  - import-wallet prints `threshold=none`.
- Section 3: the production change (what, not how).
- Section 4: 30 RED rows by exact name.
  - Rows 1-12 are triage rows (`d19256e8`).
  - Rows 13-30 are new (`283132e7`). They cover import-wallet, pure-key thresh, JSON null, the tr/NUMS shapes, `--import-json --threshold`, and a `format.rs` unit row.
  - Measured on base: `30 tests run: 6 passed, 24 failed`.
  - Measured with the patch: `30 tests run: 30 passed`.
  - The RED run must show `Compiling mnemonic-toolkit`. A stale nextest binary gave a false `30 passed` on base once on this branch; the doc records the guard.
- Section 5: the tests and transcripts that change.
- Section 6: 8 lockstep files, with hunks in the evidence dir, and the build steps in order.
- Section 7: the full toolkit CI gate, every item measured on the patched tree.
  - It includes the manual job, the doc-flag-lint mutation self-test, transcripts and Examples regeneration, the GUI book lint and the mutation gate.
  - It ends with the FIX rows by exact name, expected matched count 30.
- Section 9 maps rulings 1, 2, 3 and 7 to the rows.

## Findings

None Critical or Important. Ruling 3 departs from the scratch predicate and from restore (pure-key `thresh` stays "any k of n"); this is recorded in the doc as ruled, not filed.

## STOPs (product points for the architect; the builder waits on all three)

1. **STOP-1, GUI tutorial items of ruling 7.** These are `tut-j3-12` and `tut-j4-16` (stderr transcripts, PNGs, prose) and the j3/j4 restore-feed bundle-json transcripts.
   - `manual-gui.yml:75-78` byte-compares them with the pinned `mnemonic-gui-v0.63.0` corpus, so this PR cannot change them.
   - Two points need a ruling:
     - (1) whether to defer them to the GUI re-pin, via the proposed FOLLOWUPS entry F1 `manual-gui-tutorial-v7-general-recovery`;
     - (2) an interim prose note: (i) none, or (ii) a note with wording the ruling gives.
   - Until then the GUI book keeps printing the C-graded false claims `any 3 of 11` (j3) and `any 3 of 12` (j4).
2. **STOP-2, `bundle --import-json --threshold`.** Today the flag sets k (a plain 2-of-3 envelope with `--threshold 1` prints `1 of 3`). The options are:
   - (a) refuse with `ModeViolation` at exit 2, mode and text to be ruled;
   - (b) ignore the flag.
   Row 24 accepts both until the ruling.
3. **STOP-3, CHANGELOG.** The options are:
   - (a) no change, the same as FIX-V5;
   - (b) apply `fix-v7-changelog.patch`.

## Counts note

18 rows are new; 13 of them (rows 13-24 and 30) fail on base. 11 of the 12 triage rows fail on base.

Counts: RED rows 30 (new 18, fail-on-base 24); lockstep files 8; STOPs 3
