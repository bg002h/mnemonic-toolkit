# Fold report: whole-diff V7 s1 and s2 (mnemonic-toolkit, PR #135)

Branch `build/v7-bundle-recovery`, base head fd283d6f. Brief:
`coordinator/briefs/fold-tk-v7-whole-diff.md`. Gate output: `/scratch/cache/tk-v7-fold/`.

## Findings folded

| Finding | Applied | RED on fd283d6f | Test |
|---|---|---|---|
| s1 I-1 (J4 GUI note, `docs/manual-gui/tutorial/50-j4-taproot-twin.md`) | yes, exact text, no reflow | n/a (prose) | none (prose; gated by make lint, see below) |
| s1 I-2 (`export_wallet.rs` takes K from envelope md1) | yes, exact code | yes: `v7_export_wallet_takes_k_from_envelope_descriptor` FAIL (null.json, sparrow refusal, `cli_bundle_recovery_threshold_text.rs:744`) | `v7_export_wallet_takes_k_from_envelope_descriptor` |
| s2 I-1 (template-mode guard rows 31/32) | yes, exact rows, rustfmt reflow | no, by design: both PASS on head | `v7_template_mode_card_keeps_flag_threshold_and_recovery_lines`, `v7_template_mode_json_threshold_is_the_flag` |

RED run (`red-s1i2-s2i1.txt`): 3 tests run, 2 passed, 1 failed. GREEN (`green-new-rows.txt`): 3 passed.

## Gate

Each run captured once to `/scratch/cache/tk-v7-fold/`.

- `cargo fmt --all -- --check`: exit 0 (`fmt.txt` empty). See the reflow note below.
- `cargo clippy --locked --all-targets -- -D warnings`: exit 0, 0 `warning`/`error` lines (`clippy.txt`).
- `cargo nextest run --locked --workspace`: `4218 tests run: 4218 passed, 38 skipped`, exit 0 (`nextest-workspace.txt`).
- FIX doc section 4 RED command, as printed: `30 tests run: 30 passed, 4114 skipped` (`fix-doc-section4-red.txt`).
- `make lint` in `docs/manual-gui`: exit 2 (`make-lint.txt`). Passing: markdownlint (`Summary: 0 error(s)`), cspell (`Files checked: 104, Issues found: 0`), and the other tutorial, xref, pin and transcript checks. Failing: `verify-figures-gui` (census: pinned corpus has 127 PNGs, expected 66; snapshots missing from `figures/gui`) and `verify-tutorial-figures` (pinned clone disagrees with its manifest). Neither reads the changed tutorial text. I did not verify these against the base commit, so they are not confirmed as pre-existing. They look environmental: the inputs are the figure corpus and the pinned mnemonic-gui clone.

## Deviations and open points

1. **fmt reflow.** `cargo fmt --check` failed on three line wraps in the new rows. I ran `cargo fmt --all`, which changed only `cli_bundle_recovery_threshold_text.rs`. The content is unchanged. The fmt check then passed.
2. **s1 I-2 amendment not applied.** The I-2 fix text also amends FIX 2.3's export-wallet bullet and the SPEC fold's last sentence to say that `export-wallet --from-import-json` takes K from the envelope's descriptor and ignores the envelope's `threshold`. The brief says "no other edits", so I did not apply it. Controller decision needed: apply it as a doc-only commit, or drop it.
3. **Printed filter does not include the new rows.** `design/agent-reports/fix-doc-tk-v7-evidence/fix-rows-filter.txt` lists only the 30 existing rows. The printed command runs 30 and passes. The 3 new rows were run separately by exact name.
4. **Mutation gate not run.** The fold changes production Rust (`export_wallet.rs`). The brief's gate does not include `coordinator/scripts/mutation-gate.sh`, which RULES.md requires for such a change. It has not been run on this fold.
5. **Deferred items filed.** Six `deferred-minor` lines were added to `design/FOLLOWUPS.md` in commit 79bfc25d: s1 M-1, N-1, N-2 and s2 N-1, N-2, N-3. Each is owned by "next toolkit cleanup cycle", the phase name used by the existing `v7-fix-review-minors` entry. That owning phase is my choice; the controller should confirm it.

## Commits

- `432b55ac` fold: whole-diff V7 s1 I-1, s1 I-2, s2 I-1 (gate counts in the message)
- `79bfc25d` FOLLOWUPS: file V7 whole-diff deferred-minor items
- this commit: this report, alone

0 Critical / 0 Important are new in this fold. The findings it closes are in the two whole-diff reports.

C / I / M / N (this report's own findings): 0 / 0 / 0 / 0
