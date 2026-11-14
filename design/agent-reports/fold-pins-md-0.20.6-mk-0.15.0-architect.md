Fold report: toolkit#162 architect rulings S1-S7 + owned FOLLOWUPS. Author: Claude Haiku 5.5 (model id claude-haiku-5-5). Brief: /scratch/code/shibboleth/refugium-coordinator/coordinator/briefs/fold-toolkit-pins-md-mk-architect.md. Base head 98215eb7, branch pins/md-0.20.6-mk-0.15.0. Binaries MK=/scratch/cache/toolkit-pins-md-mk-logs/rel/bin/mk (mk 0.15.0).

## A.1 s4-regen
## A.2 pair transcripts
## A.2 done: 3 pair transcripts rc=0 (stems found in expected dirs)
## A.3 cp doc-flag-lint
## A.4 fix.py
## A done: s4 rc0, pairs rc0, cp rc0, fix.py 'OK 20 edits applied'
## B applied via script /scratch/cache/toolkit-pins-md-mk-logs/fold/apply-B.py (texts extracted from brief lines 18-26)
## C applied via /scratch/cache/toolkit-pins-md-mk-logs/fold/apply-C.py (C1 replaced backticked open only; exit-table status keeps trailing ', Important, blocks the F1 toolkit PR.' -> QUESTION Q1)
## D applied via /scratch/cache/toolkit-pins-md-mk-logs/fold/apply-D.py (bullet wording authored from the brief's list; PR-body text below)
## E gates (fold-gates.sh, output /scratch/cache/toolkit-pins-md-mk-logs/fold/gates-run.txt)
```
manual audit rc=0
qs lint rc=0
qs verify rc=0
tm lint rc=0
tm verify rc=0
docflag rc=0
gui html rc=0
gui lint rc=0
install-signature.test.sh rc=0
install-assets.test.sh rc=0
```

## F. Files changed by A-D (git status --short, nothing unexpected)
Changed: CHANGELOG.md, ci/doc-flag-lint.test.sh, design/FOLLOWUPS.md, docs/manual-gui/src/80-troubleshooting/82-binary-and-launch.md, docs/manual/src/20-quickstart/24-recover.md, docs/manual/src/40-cli-reference/42-md.md, 43-ms.md, 44-mk-cli.md, 8 docs/manual/transcripts/44-mk-*.cmd, 44-mk-inspect-text.out, 44-mk-repair-json.out, 44-mk-repair-text.out, qs-26-recover-mk1.out, docs/quickstart/src/20-singlesig/26-recover.md, docs/technical-manual/src/20-wire-formats/22-mk1-wire-format.md, 2 docs/technical-manual/transcripts/*.out. This report is not committed (coordinator commits it).

## Questions for the coordinator (STOP-class, not decided by this agent)
- Q1 (C1, mk-manual-exit-table-0-14): the entry's status line was `- Status: \`open\`, Important, blocks the F1 toolkit PR.\``. C1 replaces the backticked value only, so it now reads `closed, by the fold commit ...`, Important, blocks the F1 toolkit PR. The trailing qualifier now contradicts the closed status. Brief does not say whether to drop it. Left as the literal replacement; please say which to keep.
- Q2 (F, trailers): brief says exactly two trailers (Co-Authored-By, Model-Id). Standing rules and the attribution reminder also require a Claude-Session line. Commit carries all three: Co-Authored-By, Model-Id, Claude-Session.
- Note: D bullet wording was written by this agent from the brief's list (the brief gives content, not wording). Review the CHANGELOG diff.

## PR-body addition (text only; PR not edited)
- The 44-mk worked-example card is re-minted with mk 0.15.0; the 44-mk transcripts are regenerated.
- mk transcripts regenerated with mk 0.15.0 (chunk_set_id warning on the bundle and test-vector cards).
- doc-flag-lint marker cases re-based on a planted row.
- 43-ms.md --separator rows match ms 0.20.4 (space only).
- Manual *When stdout fails* texts for md (42-md.md) and mk (44-mk-cli.md).
- mk exit-code table re-pinned to mk 0.15.0.

Counts: Critical 0, Important 0, Minor 1 (Q1), Nit 0
Fold commit: 1ecc4160352ec688cf0e5c5158401e0d84c17fb6 (short 1ecc4160), 24 files, Model-Id claude-haiku-5-5. Report left uncommitted.
