**⚠ 1 Important (I-1: rows (b) and (d) scan zero files, so a loosened marker passes every row and an empty scan of a published crate reads green), 4 Minor, 6 Nit.**
# Review: toolkit PR 133 (T1 miniscript tripwire), head 8c7dc9b2

Reviewer: Sonnet 5.5 (independent of the author, a Sonnet). Review worktree
`/scratch/code/shibboleth/tk-t1-review`, detached at 8c7dc9b2 (base 02052a17 = origin/master).
Findings are appended as found; counts line last.

## Evidence so far (my runs, 2026-11-03)
- `bash ci/miniscript-tripwire.test.sh`, rows a-g as printed: 7/7 PASS (also with the real
  `miniscript-13.2.0.crate`, sha 4d79651c... = index cksum, via `MINISCRIPT_132_CRATE`).
- Live: `curl https://index.crates.io/mi/ni/miniscript` -> 90 rows; the script, run as the
  workflow runs it, prints the step-4 text, exit 0. With `--baseline 13.1.0` the live
  download path (urllib, static.crates.io, sha check, scan) works: step-7 notice, exit 0.
  Note the live index is not in version order (last row is 12.3.8, after 13.2.0).
- Plant harness: a copy of `ci/` under the scratchpad, one mutation each, full test script run.

## I-1 (Important): rows (b) and (d) scan zero files, so they cannot fail on a marker false positive

`ci/miniscript-tripwire.py:68,71` builds the archive prefix from the version under test
(`"miniscript-%s/src/" % version`). The test script builds its old tree as
`miniscript-13.2.0/` (`ci/miniscript-tripwire.test.sh:51` via `mkcrate "$tmp/old-src" 13.2.0`,
and the real `miniscript-13.2.0.crate` under `MINISCRIPT_132_CRATE`) but rows (b) and (d)
present it as 13.3.0 and 14.0.0 (`:94`, `:117`). The prefix never matches, so
`crate_texts` yields nothing and the scan sees 0 files, 0/4 markers. Measured:

    crate_texts(real 13.2.0 tarball, "13.3.0") -> 0 files    (also "14.0.0" -> 0 files)
    crate_texts(real 13.2.0 tarball, "13.2.0") -> 58 files, 0 markers

Failure, shown: plant the loosened M2 the architect names unusable (V11),
`^\s*SortedMulti\(` (it matches `ShInner`/`WshInner` at 13.2.0 `sh.rs:46`,
`segwitv0.rs:236`; 13.2.0 really does carry the line). Run in the default mode and with
`MINISCRIPT_132_CRATE` set to the real tarball: **all 7 rows PASS both times.** Rows (b)
and (d) exist to prove the real 13.2.0 tree scans 0/4; as built they would pass over
any content. The author's "GREEN 7 of 7, also against the real 13.2.0 tarball" is a
false PASS for those two rows (CLAUDE.md "Still blocking": a test that reports a false PASS).
The same prefix test is a production fail-open: a `.crate` with no `<dir>/src/*.rs`
(different layout) scans 0 files and, on the same major, prints the step-7 green notice
("lacks" all four) and exits 0. Measured with a sha-correct crate holding only `notsrc/a.rs`:
rc=0. The positive control covers only `vendor/miniscript`, never the downloaded crate.

Fix (run by me in a scratch copy; all 7 rows pass in both modes, and with the loosened
M2 planted rows (b) and (d) go FAIL in both modes; the zero-file crate gives rc=1):
- `ci/miniscript-tripwire.py`: take the top directory generically and refuse an empty scan.

      def crate_texts(data):
          out = []
          with tarfile.open(fileobj=io.BytesIO(data), mode="r:gz") as tf:
              for m in tf:
                  parts = m.name.split("/")
                  if m.isfile() and len(parts) >= 3 and parts[1] == "src" and m.name.endswith(".rs"):
                      out.append(tf.extractfile(m).read().decode("utf-8", errors="replace"))
          return out
      ...
          texts = crate_texts(data)
          if not texts:
              print("::error::miniscript-%s.crate has no src/*.rs files, so a scan of it "
                    "would mean nothing. Failing closed." % top)
              return 1
          found = scan(texts)

- `ci/miniscript-tripwire.test.sh`, derived 13.2.0 tree (the CI mode, since the workflow
  does not set `MINISCRIPT_132_CRATE`): add a decoy so a loosened M2 is caught in CI too,
  before `mkcrate "$tmp/old-src" 13.2.0 ...`:

      printf '    SortedMulti(SortedMultiVec<Pk, Legacy>),\n' > "$tmp/old-src/decoy.rs"

- Add a row_h: a sha-correct 13.3.0 crate with no `src/*.rs` expects exit 1 and the
  message above (the architect's Q2 has no such row; it is the production side of this finding).

## Plant results (Q1)
Harness: a copy of `ci/` with one mutation each, the full 7-row script run. "RED" = at least
one row FAILs. Harness scripts: `/tmp/claude-1000/-scratch-code-shibboleth-refugium-coordinator-coordinator/125b457d-0745-4589-8739-dbea56aad7bf/scratchpad/plant.sh`, `/tmp/claude-1000/-scratch-code-shibboleth-refugium-coordinator-coordinator/125b457d-0745-4589-8739-dbea56aad7bf/scratchpad/plant3.sh`.

| plant | rows FAIL (as submitted) |
|---|---|
| no-op control | none (7/7 PASS) |
| drop M1 from MARKERS | a b c d f g (control then finds 3/4) |
| drop M1 and make control and step 6 count `len(MARKERS)` | b c e g |
| `<=` to `<` in step 4 | a |
| remove `or new_major` (14.0.0 as a point release) | d |
| `new_major` uses `>=` | b c g |
| skip the vendor self-scan | e |
| skip the cksum check | f |
| scan the control dir instead of the crate | b d g |
| red at `>= 2` markers | g |
| **red at `>= 3` markers** | **none: survives** |
| **compare major.minor only (13.2.1 treated as not newer)** | **none: survives** |
| **highest = last index row** | **none: survives** |
| **highest = lexicographic max** | **none: survives** |
| **yanked rows not filtered** | **none: survives** |
| **pre-release rows not filtered** | **none: survives** |
| **M2 loosened to `^\s*SortedMulti\(` (V11 says unusable)** | **none: survives, in both test modes (I-1)** |

## M-1 (Minor): six mutants survive; the filters and the "all four" boundary have no row
`ci/miniscript-tripwire.py:115` (`live` filter), `:120` (`max(..., key=key)`), `:127` (`<=`),
`:142` (`len(found) == 4`). The rule's own words, "non-yanked, non-prerelease", "highest" and
"all four", are each unpinned (rows a-g cover 13.3.0 and 14.0.0 only, appended last). The live
index is not in version order (its last row is 12.3.8, after 13.2.0), and the 4-row fixture
hides that. Fails closed or loud when broken (a spurious red), and the path needs a later edit
plus a release of that shape, hence Minor. Fix, run in a scratch copy of the I-1 fix: all of
the six plants go RED with these rows, and 11/11 pass unmutated, in both test modes:

    row_i() {  # selection: highest by number (13.10.0 > 13.2.0), not last row, not yanked/pre-release
      local idx=$tmp/index-select.jsonl nsha; nsha=$(sha "$tmp/new.crate")
      cp "$fixture" "$idx"
      python3 -I -c '
    import json,sys
    sha=sys.argv[1]
    for v,y in (("13.10.0",False),("14.0.0",True),("14.1.0-rc.1",False),("12.3.8",False)):
        print(json.dumps({"name":"miniscript","vers":v,"deps":[],"cksum":sha,"features":{},"yanked":y}))' "$nsha" >> "$idx"
      run --index "$idx" --crate "$tmp/new.crate" --control-dir "$control"
      want_rc 1
      want_has "highest published  : 13.10.0"
      want_has "::error::miniscript 13.10.0 may let the [patch.crates-io] pin go: 4/4 fork-only APIs present"
      want_lacks "starts a new major line"
    }
    row_j() {  # a patch release carrying all four markers is above the baseline: exit 1
      run --index "$(index_with 13.2.1 "$(sha "$tmp/new.crate")")" --crate "$tmp/new.crate" --control-dir "$control"
      want_rc 1
      want_has "::error::miniscript 13.2.1 may let the [patch.crates-io] pin go: 4/4 fork-only APIs present"
    }
    row_k() {  # 3 of 4 markers, same major: stays green (pins the "all four" boundary)
      cp -r "$control" "$tmp/three-src"
      find "$tmp/three-src" -name '*.rs' -exec sed -i -E 's/pub fn into_definite\b/pub fn renamed_definite/' {} +
      mkcrate "$tmp/three-src" 13.3.0 "$tmp/three.crate"
      run --index "$(index_with 13.3.0 "$(sha "$tmp/three.crate")")" --crate "$tmp/three.crate" --control-dir "$control"
      want_rc 0
      want_has "lacks Descriptor::into_definite (#911); the [patch.crates-io] pin stays."
      want_lacks "::error::"
    }

(plus `row_h` from I-1 and the `rows=(...)` default list extended to h i j k). Scratch copy of
the fixed `ci/`: `/tmp/claude-1000/-scratch-code-shibboleth-refugium-coordinator-coordinator/125b457d-0745-4589-8739-dbea56aad7bf/scratchpad/plant2/fix1/ci/`.

## M-2 (Minor): the printed NEXT STEPS fail for a new major unless md-codec moves first
Printed text (`ci/miniscript-tripwire.py:148-158`, architect Q2 step 6 verbatim): step 1 tells the
reader to build each repo in its own scratch worktree with `[patch.crates-io]` deleted and "the
miniscript requirement" raised; step 3 says "If any fails: raise BASELINE to <V>".
Followed literally for a new major V: the toolkit takes md-codec by git rev
(`crates/mnemonic-toolkit/Cargo.toml:73`, cf35d61a), and md-codec's `miniscript` requirement is
the workspace's `13.0.0` (descriptor-mnemonic `Cargo.toml:18` at cf35d61a). With the patch deleted that md-codec resolves stock 13.2.0, has no
`Terminal::SortedMultiA`, and the toolkit build fails whatever V contains. Step 3 then says raise
BASELINE to V, which (if V carries all four) turns the tripwire permanently green on the very
event it watches (`key(top) <= key(baseline)` -> "No release newer"). The tripwire is red until a
human acts, and step 3 asks for a dated comment "naming what is still missing" (nothing is), so a
careful reader stops; Minor on the likelihood test (a new major, a literal reading, and a
skipped comment). The umbrella FOLLOWUP already has the right order (md-codec cycle first,
`design/FOLLOWUPS.md` Trigger/sequence 2 then 3). Fix wording for step 1 (text only; the
architect owns Q2, so this is a proposal): after "re-lock to <V>" add " (for a new major, do
descriptor-mnemonic first and build the other repos against its scratch worktree, since they take
md-codec by git rev)". No printed command changes; the test pins the printed lines (rows c and
d), so they change with it.

## M-3 (Minor): FOLLOWUP `fuzz-and-repro-miniscript-pin-realign` misstates what 95fdd1c lacks, and omits two stale defaults
`design/FOLLOWUPS.md` (new entry, "What"): "Both carry #910 and #915, but `95fdd1c5` lacks
#911/#929/#953". Checked in the cargo git db for rust-miniscript: `git log 95fdd1c` contains the
merges of #910, #911, #915 and #929 (7cd03d0, ddc42c7, b2f21d2, 78254b2); only #953 (merged
2026-05-25, after 95fdd1c of 2026-05-02) is absent. Marker counts at 95fdd1c are 1/1/1/1.
`gh pr diff 911` adds `pub fn derive_at_index` and `pub fn into_definite`. A later worker who
reads "lacks #911" will conclude the fuzz crate cannot call `derive_at_index`; it can. Fix:
"`95fdd1c5` lacks only #953 (the taptree Display fix); it carries #910, #911, #915 and #929".
Same entry, "What"/"Action": it names only the `ci/repro/*.sh` defaults
(`cc-validate.sh:94`, `double-build.sh:109`, `remap-off-negative.sh:82`) but
`.github/workflows/reproducible-musl-build.yml:123` and `:209` (the `miniscript_rev` input
defaults) and the comment `man-pages.yml:139` ("also the default; passed explicitly for
clarity") are stale too. Both callers pass the rev explicitly (`man-pages.yml:140`, `repro-drift.yml`
from `Cargo.lock`), so CI is unaffected, as the entry says; a manual dispatch with the input
omitted gets the old rev and fails `--offline` (loud). Fix: add those three places to the Action.

## M-4 (Minor): the workflow's existing "DELETE this workflow" advice no longer covers the files this PR adds
`.github/workflows/miniscript-fork-tripwire.yml:74` (unchanged text) says to DELETE this
workflow if the fork was dropped. This PR adds `ci/miniscript-tripwire.py`, its test and the
fixture, which only this workflow runs; deleting the workflow alone leaves them orphaned, and the
test script reads `vendor/miniscript`, which the same drop removes. Fix: ", together with
`ci/miniscript-tripwire.py`, `ci/miniscript-tripwire.test.sh` and `ci/fixtures/miniscript-index-13x.jsonl`".

## Nits
- N-1: workflow `on.push.paths` (`:50-55`) omits `vendor/miniscript/**`, so the positive control does not run on a PR that changes the vendored tree (it runs weekly); `vendor-freshness` covers the content.
- N-2: row (d) prints `0/4 fork-only APIs present (); 14.0.0 starts a new major line` (empty parens); it is the Q2 template and the test pins it. Cosmetic.
- N-3: manual `41-mnemonic.md:1542` says "since the 2026-08-20 miniscript pin"; the manual names versions elsewhere ("Since v0.55.3"). The first release containing 5f88071c is `mnemonic-toolkit-v0.98.0` (`git merge-base --is-ancestor 5f88071c mnemonic-toolkit-v0.98.0` true; v0.97.1 false). "since v0.98.0" lets a holder check their build.
- N-4: manual `41-mnemonic.md:194-198` says "a tracked follow-up" without naming it; the refusal text names `restore-md1-taproot-use-site-override-arm`.
- N-5: root `Cargo.toml:13` opens "carried for fragments and APIs that no crates.io release has" and then lists #953, which 13.2.0 has; the bullet explains, the lead sentence contradicts. Reword to "mostly fragments and APIs".
- N-6: the `resolved` status of `upstream-miniscript-taptree-depth2-display-asymmetry` does not mention that the stale `fuzz/` pin (95fdd1c, no #953) still shares the Display bug; add "(the `fuzz/` copy still shares it until `fuzz-and-repro-miniscript-pin-realign`)".

## Answers to the brief's questions
1. Script vs Q2: the printed texts, step order, exit codes and the 7 rows match Q2 exactly
   (compared line by line; the `NEXT STEPS` block and the three header lines are as Q2 prints them).
   Rows (a)-(g) as printed: 7/7 PASS (also in CI, run 41756373110, and live from this machine:
   the step-4 text with the 90-row index; the download path with `--baseline 13.1.0` gives the
   step-7 notice for the real 13.2.0 crate). Plants: the architect's four (drop a marker check,
   mis-compare the baseline, 14.0.0 as a point release, skip the vendor scan) all go RED.
   Defects: **I-1** (rows b and d scan nothing, so a loosened marker passes; an empty scan is a
   green) and M-1 (six surviving mutants).
2. Workflow: schedule `0 6 * * 1` unchanged; fail-closed on curl error (`--fail`, `set -e`), empty
   file (`test -s`), unparseable or empty index (script exit 1), failed download (3 tries, exit 1),
   cksum mismatch (exit 1). Printed texts match Q2. The `tripwire` job on the PR head passes.
   The only gap is I-1's empty-scan green and M-4's stale delete advice.
3. Tracker, Cargo.toml, fuzz and manual text, against sources: #910 SortedMultiA, #911
   derive_at_index and into_definite, #915 SortedMulti into Terminal, #929 generic plan are merged
   to master 2026-04-03..25 and are in ff4732e (merge ancestry) but not in 13.2.0 (13.2.0
   `plan` is on `Descriptor<DefiniteDescriptorKey>` only; 0/0/0/0 markers). #953 merged 2026-05-25,
   backported as #1039 (13.2.0 CHANGELOG: "fix unbalanced Taproot tree formatting"). 13.2.0 was
   published 2026-10-17 (crates.io API; its CHANGELOG header says October 15). 5f88071c (2026-08-20)
   lifted the depth gate; the tests `left_heavy_3leaf_tr_restores_depth2`,
   `right_spine_3leaf_tr_also_restores_depth2` and `restore_help_says_depth2_taproot_restores` exist
   (`cli_restore_taproot.rs:262,293,320`). `mr` `cost.rs` lines 241, 265, 478, 10-19 are as cited
   (`git show 5ed5b66f`). `fuzz/Cargo.toml` parses (`tomllib`) and `cargo metadata` succeeds in a
   scratch `git archive` copy (it re-locks: md-codec 0.42.0 -> 0.47.0 git, toolkit 0.97.0 ->
   0.105.1, which confirms the entry's "stale beyond the pin"); the review worktree's
   `fuzz/Cargo.lock` was not touched. Wrong: M-3 (what 95fdd1c lacks). md PR 45 agrees on the
   Q1 text, the 13.2.0 facts and the #910/#915/#911/#929 set (its own Minor M1 notes #929 has an
   `into_plan` marker; the toolkit tripwire does not watch #929, as Q2 decided).
4. `resolved` for `upstream-miniscript-taptree-depth2-display-asymmetry`: correct. The pin
   `ff4732e5` carries #953 (ancestry checked), the gate was lifted by 5f88071c, no crate code cites the
   slug as a live refusal (`grep` finds none), "Actions on close" are done, and `resolved` follows
   the file's status convention (many entries). Caveat N-6 only.

## Round bookkeeping
First review of this PR (not a re-review), so no fix-commit attribution. Author: Sonnet-class
(per the brief). Reviewer: Sonnet 5.5.

**Status: 1 Important (I-1), 4 Minor, 6 Nit. I-1 blocks until folded.**
0 C / 1 I / 4 M / 6 N
