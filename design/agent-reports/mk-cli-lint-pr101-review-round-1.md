# PR #101 (docs/mk-cli-lint-followups, head ee6a6160) — adversarial review, round 1

## Verdict

**Not safe to merge as-is: 2 Important, both one-sentence wording fixes in
`mk-cli-pin-bump-checklist`.** The PR is clean otherwise. Its shape is right
(3 paths, 0 removed lines, `--check` clean). The two reports are byte-identical
to PR #100's head. Every path and line citation resolves on origin/master
(`4d703bb4`). It contains no secret and no email address. The defects are in the
checklist's advice: followed literally, step (3) turns two CI gates red and
leaves one pin stale with no gate to catch it, and step (2) leaves mutation R5
passing on a property it no longer tests. Neither changes any lint, chapter or CI
today, so merging does not break anything. The checklist exists to make the
pin-bump PR complete, though, and as written it is incomplete.

## What was checked (all executed)

1. **Diff shape.** `git diff --name-status origin/master...ee6a6160` returns
   `M design/FOLLOWUPS.md`, `A design/agent-reports/mk-cli-lint-consult-opus-1.md`
   and `A design/agent-reports/mk-cli-lint-pr100-review-round-1.md`, with
   3 files and 299 insertions. The merge-base is origin/master `4d703bb4`.
   `git diff --check` gives rc=0, and `grep -c '^-[^-]'` on the diff gives 0
   removed lines. The FOLLOWUPS hunk is a pure append at `@@ -6016,3 +6016,20 @@`,
   preceded by a blank line. The file ends in `\n` and has 0 CR bytes. Both
   entries use the newer house style of the neighbouring `mr-*` entries (5995–6018):
   a `### \`slug\` — … (tier: …; owning phase: …)` heading and `Surfaced` /
   `Where` / `What` / `Status` / `Tier` bullets. One extra `Fix options:`
   bullet has precedent in the file (`- **Fix …:**` at lines 34, 210 and 222).
2. **`scripts/followup-reconcile.sh`** on master and on the PR file: both rc=0.
   The only output difference is `104 open` → `105 open`, with resolved
   unchanged at 366.
3. **Path citations.** `git cat-file -e origin/master:<p>` succeeds for
   `docs/manual/tests/lint.sh`, `ci/doc-flag-lint.test.sh`,
   `.github/workflows/manual.yml`, `scripts/install.sh`,
   `docs/manual/src/40-cli-reference/44-mk-cli.md` and `docs/manual/Makefile`.
   - `manual.yml:110` reads
     `run: cargo install --git https://github.com/bg002h/mnemonic-key --tag mk-cli-v0.13.0 mk-cli`. ✓
   - `lint.sh:195` reads
     `PINNED[$bin]=$(eval $binv --version … | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1 …)`.
     A grep of lint.sh for `manual.yml`, `mk-cli-v` or a tag finds no match, so
     "the lint still has no check of its own" holds. ✓
   - In `44-mk-cli.md`, `grep -n unreleased` matches 16 (the paragraph) and the rows
     43, 44, 45, 46, 134, 183, 241, 412, 503 and 553, which is 10 rows. Every row
     carries `(unreleased: mk-cli after 0.13.0: <flag>)`. The flags are
     `--from-md1-set`, `--keys`, `--in`, `--out` and six more `--in`. The
     paragraph sits at 16–18 and line 12 is the `--tag mk-cli-v0.13.0` install line. ✓
   - In `ci/doc-flag-lint.test.sh`, R2 (`:76`) and R4 (`:84`) anchor on
     `(unreleased: mk-cli after 0.13.0: --in)`, and R6 (`:92`) and R7 (`:96`)
     anchor on `(… --keys)`. `edit_file` raises `mutation anchor not found` when
     the anchor is missing. ✓
   - The PR #100 report finding IDs exist as cited: C1 (line 56), I1 (81),
     M1 and M2 (98, 105) and N1 (115). "10 stale marker FAILs" is at report line 22. ✓
   - mnemonic-key `1b8c524` exists on `origin/main`, with the subject "mk-cli:
     0.14.0-dev -- a main build no longer passes for 0.13.0 (F-677)". ✓
4. **Example path from the advice lens.**
   `ls -la /scratch/code/shibboleth/mk-tag-rv/root/bin/mk` shows that it exists
   and `--version` prints `mk 0.13.0`. `root/.crates.toml` reads
   `mk-cli 0.13.0 (git+…?tag=mk-cli-v0.13.0#0feaaaa9…)`, so it is the tagged
   build. With `MNEMONIC_BIN=mnemonic MD_BIN=md MS_BIN=ms MK_BIN=<that>`,
   `bash ci/doc-flag-lint.test.sh` printed `12 passed, 0 failed` with rc=0. The
   markers are correct today, as the entry says.
5. **Reports are verbatim.** PR #100's head `e95043d3` is fetchable, and the
   sha256 of each file matches between `e95043d3` and `ee6a6160`:
   - consult-opus-1: `90fb9a83…bfe34`, identical.
   - pr100-review-round-1: `4d7306c7…d6dd`, identical.

   The review ends with `1 / 1 / 2 / 1`. The consult ends with its provenance
   caveat and a trailing `\n`. It is a consult, so it has no counts line, which
   is not a defect.
6. **Secrets and email.** I grepped the diff for gmail, goss, email domains,
   `claude-<model>-<n>`, xprv/tprv, token, password, `ghp_`, 12-word runs, long
   ms1/mk1/md1 strings and long xpubs. The only hits are the word "Opus" in a
   filename and in prose. That is a model family name, not a model id. The email
   appears only in the git author metadata of the 3 commits, which is the
   configured identity and the same as other master commits. It is not in any
   file.

## Critical

None.

## Important

**I1 — design/FOLLOWUPS.md:6032–6033 (`mk-cli-pin-bump-checklist`, Where and
step 3): the pin-site list is incomplete, so following it literally turns two
gates red and leaves one pin stale with no gate to catch it.** Step (3) says to
move the pin at `manual.yml:110`, `scripts/install.sh` and `44-mk-cli.md:12`.
`git grep -n 'mk-cli-v0\.13\.0' origin/master` (outside `design/`) also returns
these sites:
- `.github/workflows/quickstart.yml:93` and
  `.github/workflows/technical-manual.yml:127`. Both are single-line
  `cargo install --git … --tag mk-cli-v0.13.0 mk-cli`. `sibling-pin-check.yml`
  scans every `.github/workflows/*.yml` and `docs/manual/src/**/*.md` line of
  that shape against install.sh's canonical tag. So bumping install.sh per
  step 3 gives `::error::sibling-pin-check: …quickstart.yml:93: pin
  'mk-cli-v0.13.0' … does not match`, and the same for technical-manual.yml:127.
- `.examples-build/Examples.md:110,142,143`. This is the `install.sh --list` and
  dry-run golden. `examples.yml` regenerates it whenever `scripts/install.sh`
  changes, and then runs `git diff --exit-code -- .examples-build/Examples.md`,
  so the `examples` job goes red.
- `.github/workflows/manual-gui.yml:200` holds `--tag mk-cli-v0.13.0` on a split
  line, which `sibling-pin-check` cannot see. It stays stale with nothing
  saying so. Existing entry `manual-gui-workflow-pins-md-cli-0-11-0-on-a-split-line`
  covers the same detector gap.
- `docs/manual-gui/pinned-upstream.toml:114` and four manual-gui chapters cite
  `mk-cli-v0.13.0`. These follow the GUI pin, which is a separate decision, but
  the checklist should say so rather than stay silent.

Fix wording for step (3): "(3) move every mk-cli pin: run `git grep -n
'mk-cli-v0\.13\.0'` outside `design/` and update each hit. Today these are
`scripts/install.sh:61`, `.github/workflows/manual.yml:110`, `quickstart.yml:93`,
`technical-manual.yml:127`, `manual-gui.yml:200` (a split line that
sibling-pin-check does not see) and `44-mk-cli.md:12`. Then regenerate
`.examples-build/Examples.md` with `bash .examples-build/gen.sh`, because the
`examples` job diffs it. The `docs/manual-gui` pins follow the GUI pin and are
not part of this PR." Mirror the paths in `Where:`.

**I2 — design/FOLLOWUPS.md:6033 (step 2): R5 is omitted. After the bump, R5
keeps passing while it no longer tests a marked row, so it reports a false
pass.** R5 (`ci/doc-flag-lint.test.sh:87-89`, "a fake flag added to a MARKED
row's cell") anchors on the `| \`--keys <FILE>\` |` row text rather than on the
marker. Its anchor therefore survives the marker removal in step (1), and its
expected substring is still produced. I measured this in a copy of
`docs/manual/src`: I removed all ten mk markers with `sed`, applied R5's edit and
ran `tests/lint.sh` with the mnemonic-key main `mk`, standing in for a pin that
defines the flags. Step 4 printed `[lint] FAIL: flag --bogus-forever is
documented for \`mk encode\` … (no '(unreleased: mk-cli after 0.14.0:
--bogus-forever)' marker naming it)` and `0 flag exemption(s)`. R5's expected
substring matches, so R5 reports `ok` on an unmarked row. That duplicates R1 and
no longer exercises the fix1 NEW-2 per-flag (not per-row) exemption that is its
whole point.

Fix wording for step (2): "(2) re-anchor R2, R4, R5, R6 and R7 on a marker still
live at the new pin, or inject a synthetic marked row into the copied `src/` as
R1 does for an unmarked row. R5 must then add its fake flag to that marked row,
because its current anchor (the `--keys` row text) survives marker removal and
it would keep passing on an unmarked row." Add R5 to the `Where:` list too.

## Minor

- M1 — FOLLOWUPS.md:6024 (`manual-lint-trusts-version-string`, What): "mk-cli
  closed the version hazard at source" overstates what happened. The silent half
  is closed, but the false FAILs remain, now with wrong advice. Running
  `tests/lint.sh` with today's mnemonic-key main `mk` (`mk 0.14.0-dev`, which
  `grep -oE` reads as `0.14.0`) prints 10 ×
  `marker '(unreleased: mk-cli after 0.13.0: …)' … does not name the pinned release; expected '(unreleased: mk-cli after 0.14.0: …)'`.
  That advice is wrong, since 0.14.0 is the release expected to ship these flags.
  The Makefile's default `MK_BIN` (`Makefile:49`, `cargo run` of the sibling
  `mnemonic-key` checkout) is exactly this build. Record the current symptom.
- M2 — FOLLOWUPS.md:6033 (step 5): the example `MK_BIN=/scratch/code/shibboleth/mk-tag-rv/root/bin/mk`
  exists today, but it is the 0.13.0 binary, which is wrong for the bump PR. It
  also lives in a scratch dir that the PR #100 review's Housekeeping asks to be
  deleted. `ci/doc-flag-lint.test.sh:14` also requires all four `*_BIN`, so
  setting only `MK_BIN` aborts on `MNEMONIC_BIN: parameter null or not set`.
  Give the recipe instead: `cargo install --root <dir> --git https://github.com/bg002h/mnemonic-key --tag mk-cli-v<new> mk-cli`,
  check `<dir>/.crates.toml` shows `?tag=`, then set all four `*_BIN`, with
  `MK_BIN=<dir>/bin/mk`.
- M3 — FOLLOWUPS.md:6033 (step 4): this step only covers the case where no
  marker remains. If any marker survives at the new pin, the paragraph at
  `44-mk-cli.md:16-17` still says `after 0.13.0` and "the released mk-cli
  0.13.0 this manual is verified against". The lint scans only table rows
  (`grep -E '^\|[[:space:]]*\`'`), so this prose goes stale with no error.
  Reword step 4 as: "update the version in the paragraph to the new pin, or drop
  it if no marker remains".

## Nit

- N1 — FOLLOWUPS.md:6026: Status `deferred-minor` occurs 0 times on master, and
  `followup-reconcile.sh` counts it as neither open nor resolved. RULES.md
  mandates the value, so this is a tooling gap rather than a PR defect.
- N2 — environment, not the PR: `~/.cargo/bin/mk` is still the path build that
  caused PR #100 (`~/.cargo/.crates.toml:17`
  `mk-cli 0.13.0 (path+file:///…/mnemonic-key/crates/mk-cli)`). `md`, `ms` and
  `mnemonic` on PATH are path builds too. Anyone who runs the lint with `MK_BIN=mk`
  reproduces the false premise.

0 / 2 / 3 / 2
