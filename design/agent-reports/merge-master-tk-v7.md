# Merge origin/master (V5, PR 134 as 47baf6f2) into build/v7-bundle-recovery (PR 135)

Merge commit: 8eefb84190b135dfaca18745a87069193534c260 (parent 1: 37ac77ac).

## Conflicted files

Only `design/FOLLOWUPS.md` (append-vs-append at the end of the file). Resolved by keeping
both sides in full: V7's entries (`manual-gui-tutorial-v7-general-recovery`, the v7-s2
entries incl. N-3) first, then V5's `fix-v5-r1-minors` and the V5 s1/s2 minors block. No
product choice involved. Auto-merged without conflict: `.examples-build/Examples.md`,
`docs/manual/src/40-cli-reference/41-mnemonic.md`, others.

## Gate (outputs in /scratch/cache/tk-v7-merge/)

- `cargo fmt --all -- --check`: exit 0.
- `cargo clippy --locked --all-targets -- -D warnings`: exit 0.
- `cargo nextest run --locked --workspace`: `4251 tests run: 4251 passed, 38 skipped`.
- FIX section 4 command as printed: `30 tests run: 30 passed, 4147 skipped`.
- `make audit` in docs/manual (run because 41-mnemonic.md was touched by both sides):
  exit 2 in `lint`: 10 FAILs, all `44-mk-cli.md` markers `(unreleased: mk-cli after 0.13.0 ...)`
  vs pinned 0.14.0. `44-mk-cli.md` and `Cargo.lock` are byte-identical to origin/master
  (`git diff origin/master HEAD` empty), so this is not caused by the merge; it is
  inherited from master and is a separate owed item (not filed by me).

## CI (PR 135, head 8eefb841)

At last look: 18 pass, 12 pending, 2 skipping (release jobs), 0 fail. The runners were
slow; `gh pr checks --watch` exited early and the later polling loops were killed, so the
final counts were not observed. The push of this report re-triggers CI.
