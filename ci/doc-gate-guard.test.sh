#!/usr/bin/env bash
# ci/doc-gate-guard.test.sh — scripted scenarios for ci/doc-gate-guard.sh.
#
#   usage: bash ci/doc-gate-guard.test.sh           (from anywhere in the repo)
#   env:   DOC_GATE_TEST_TMP=<dir>  scratch location (default: mktemp -d)
#
# Builds a throwaway "origin" (a bare repo holding this checkout's HEAD as
# master) and a sparse clone of it, makes commits there, and runs the guard
# EXACTLY as each workflow invokes it -- the `run: bash ci/doc-gate-guard.sh …`
# line is read out of .github/workflows/<wf>.yml, so the test exercises the
# real arguments, not a copy. Nothing touches the real remote.
#
# Each review counterexample (I-1, I-2, I-3 in
# mnemonic-engrave/design/agent-reports/toolkit-docs-green-review.md) is a
# scenario that must answer relevant=true. Negative scenarios prove the guard
# is not trivially "always true". Where the pre-fix guard (commit 6ce7e464) is
# in history, each counterexample is ALSO run against it and must answer
# relevant=false there: that is the proof the scenario reproduces the defect
# rather than passing by construction.
set -uo pipefail

REPO="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
TMP="${DOC_GATE_TEST_TMP:-$(mktemp -d)}"
rm -rf "$TMP/origin.git" "$TMP/work" "$TMP/old"
mkdir -p "$TMP"
PREFIX_REV=6ce7e464   # the guard the first review found I-1..I-3 in
FIX1_REV=aafd9d27     # the round-1 guard the re-review found NEW-1 in
WFS=(manual quickstart technical-manual manual-gui)

pass=0; fail=0
ok()  { pass=$((pass + 1)); printf 'ok   %s\n' "$1"; }
bad() { fail=$((fail + 1)); printf 'FAIL %s\n' "$1"; }

git init --quiet --bare "$TMP/origin.git"
git --git-dir="$TMP/origin.git" fetch --quiet --no-tags --update-shallow "$REPO" "HEAD:refs/heads/master" || { echo "cannot seed origin"; exit 1; }
git clone --quiet --no-checkout "$TMP/origin.git" "$TMP/work"
W="$TMP/work"
g() { git -C "$W" -c user.name=t -c user.email=t@t -c commit.gpgsign=false "$@"; }
g sparse-checkout set --cone docs ci .github >/dev/null
g checkout --quiet master
MASTER=$(g rev-parse HEAD)

# guard_line WF [REV] -> the workflow's guard command (at REV if given)
guard_line() {
  local src
  if [ -n "${2:-}" ]; then src=$(git -C "$W" show "$2:.github/workflows/$1.yml" 2>/dev/null) || return 1
  else src=$(cat "$W/.github/workflows/$1.yml"); fi
  grep -oE 'bash ci/doc-gate-guard\.sh .*$' <<<"$src" | head -1
}

# relevant WF EVENT REF [BEFORE] [BASE_REF] [OLD-REV] -> prints true|false|n/a
# OLD-REV runs the guard (and its path deriver) as they were at that commit,
# with that commit's workflow arguments: the control that proves a scenario
# reproduces the defect it guards.
relevant() {
  local wf=$1 event=$2 ref=$3 before=${4:-} base_ref=${5:-} old=${6:-} cmd dir out ev
  dir="$W"
  [ "$old" = old ] && old=$PREFIX_REV
  if [ -n "$old" ]; then
    cmd=$(guard_line "$wf" "$old") || { echo n/a; return; }
    rm -rf "$TMP/old"; mkdir -p "$TMP/old/ci"
    git -C "$W" show "$old:ci/doc-gate-guard.sh" > "$TMP/old/ci/doc-gate-guard.sh" 2>/dev/null || { echo n/a; return; }
    git -C "$W" show "$old:ci/doc-gate-paths.py" > "$TMP/old/ci/doc-gate-paths.py" 2>/dev/null || true
    cmd=${cmd/bash ci\/doc-gate-guard.sh/bash $TMP/old/ci/doc-gate-guard.sh}
  else
    cmd=$(guard_line "$wf")
  fi
  out="$TMP/out.$$"; : > "$out"; ev="$TMP/ev.json"
  printf '{"before":"%s"}' "$before" > "$ev"
  ( cd "$dir" && env GITHUB_EVENT_NAME="$event" GITHUB_REF="$ref" GITHUB_BASE_REF="$base_ref" \
      GITHUB_EVENT_PATH="$ev" GITHUB_OUTPUT="$out" bash -c "$cmd" >/dev/null 2>&1 )
  grep -oE '^relevant=(true|false)$' "$out" | tail -1 | cut -d= -f2 || true
  rm -f "$out"
}

expect() { # LABEL WANT GOT   (GOT=n/a: a control whose pre-fix rev is absent)
  if [ "$3" = n/a ]; then printf 'skip %s (pre-fix rev not in history)\n' "$1"; return; fi
  if [ "$2" = "$3" ]; then ok "$1 -> $3"; else bad "$1: want $2, got ${3:-<none>}"; fi
}

branch_from_master() { g checkout --quiet -B "$1" "$MASTER"; }

# ---- S0: the derived watched sets pin the known cross-book dependencies ----
paths() { (cd "$W" && python3 ci/doc-gate-paths.py "$@"); }
tm=$(paths docs/technical-manual); qs=$(paths docs/quickstart); gui=$(paths docs/manual-gui)
grep -qxF docs/manual/tests/verify-examples.sh <<<"$tm" && ok "S0 technical-manual watches manual verify-examples.sh" || bad "S0 technical-manual misses verify-examples.sh"
grep -qxF docs/manual/pandoc/filters/include-transcript.lua <<<"$tm" && ok "S0 technical-manual watches include-transcript.lua" || bad "S0 technical-manual misses include-transcript.lua"
grep -qxF docs/manual/transcripts/ <<<"$qs" && ok "S0 quickstart watches manual transcripts/" || bad "S0 quickstart misses manual transcripts/"
grep -qE '^docs/manual/tests/(verify-examples\.sh)?$' <<<"$gui" && ok "S0 manual-gui watches manual verify-examples.sh" || bad "S0 manual-gui misses verify-examples.sh"
# every symlink inside every book resolves into that book's derived set
for b in manual quickstart technical-manual manual-gui; do
  set_b=$(paths "docs/$b")
  while IFS= read -r l; do
    [ -z "$l" ] && continue
    t=$(cd "$W" && realpath --relative-to=. "$l")
    hit=0
    while IFS= read -r w; do
      case "$w" in */) [ "${t#"$w"}" != "$t" ] || [ "$t/" = "$w" ] && hit=1 ;; *) [ "$t" = "$w" ] && hit=1 ;; esac
    done <<<"$set_b"
    [ $hit = 1 ] && ok "S0 docs/$b symlink $l -> $t is watched" || bad "S0 docs/$b symlink $l -> $t NOT watched"
  done < <(cd "$W" && find "docs/$b" -type l -not -path '*/build/*')
done

# ---- S1 (I-1): only the shared verify-examples.sh changes ----
branch_from_master s1
echo '# touched' >> "$W/docs/manual/tests/verify-examples.sh"
g commit --quiet -am "s1: shared verify-examples.sh only"
for wf in "${WFS[@]}"; do
  expect "S1 $wf, ci/staging push, only docs/manual/tests/verify-examples.sh" true "$(relevant "$wf" push refs/heads/ci/staging 0000000000000000000000000000000000000000)"
  expect "S1 $wf, PR, only docs/manual/tests/verify-examples.sh" true "$(relevant "$wf" pull_request refs/pull/1/merge '' master)"
done
for wf in technical-manual manual-gui; do
  expect "S1-control $wf with the pre-fix guard (the defect)" false "$(relevant "$wf" push refs/heads/ci/staging 0000000000000000000000000000000000000000 '' old)"
done

# ---- S2 (I-2): a watched file is git-mv'd OUT of scope ----
branch_from_master s2
mkdir -p "$W/attic"
g mv --sparse docs/manual/transcripts/22-first-bundle.cmd attic/22-first-bundle.cmd
g commit --quiet -m "s2: move a transcript out of docs/"
expect "S2 manual, rename out of docs/manual/" true "$(relevant manual push refs/heads/ci/staging 0000000000000000000000000000000000000000)"
expect "S2 quickstart, rename out of docs/manual/transcripts/" true "$(relevant quickstart push refs/heads/ci/staging 0000000000000000000000000000000000000000)"
expect "S2-control manual with the pre-fix guard (the defect)" false "$(relevant manual push refs/heads/ci/staging 0000000000000000000000000000000000000000 '' old)"

# ---- S3 (I-3): an unchecked commit left on ci/staging, README-only on top ----
branch_from_master s3
printf '\n| `--no-such-flag` | a doc-breaking row |\n' >> "$W/docs/manual/src/40-cli-reference/42-md.md"
g commit --quiet -am "s3 C: doc-breaking, pushed to ci/staging, window interrupted"
C=$(g rev-parse HEAD)
git --git-dir="$TMP/origin.git" fetch --quiet --no-tags --update-shallow "$W" "$C:refs/heads/ci/staging"
echo x > "$W/README-s3.txt"; g add README-s3.txt
g commit --quiet -m "s3 D: README-only, pushed on top (before = C)"
for wf in manual; do
  expect "S3 $wf, ci/staging push D with before=C (C unchecked)" true "$(relevant "$wf" push refs/heads/ci/staging "$C")"
done
expect "S3-control manual with the pre-fix guard (the defect)" false "$(relevant manual push refs/heads/ci/staging "$C" '' old)"

# ---- S4 (fix1 NEW-1): the SHARED file itself is deleted ----
# Every book that reaches docs/manual/tests/verify-examples.sh (manual owns it;
# quickstart, technical-manual and manual-gui symlink to it) must run.
branch_from_master s4
g rm --quiet docs/manual/tests/verify-examples.sh
g commit --quiet -m "s4: delete the shared verify-examples.sh"
for wf in "${WFS[@]}"; do
  expect "S4 $wf, the shared verify-examples.sh deleted" true "$(relevant "$wf" push refs/heads/ci/staging 0000000000000000000000000000000000000000)"
done
for wf in quickstart technical-manual; do
  expect "S4-control $wf with the round-1 guard (the defect)" false "$(relevant "$wf" push refs/heads/ci/staging 0000000000000000000000000000000000000000 '' "$FIX1_REV")"
done

# ---- S5 (fix1 NEW-1): a book's symlink is RETARGETED ----
branch_from_master s5
ln -sfn ../../manual/tests/lint.sh "$W/docs/technical-manual/tests/verify-examples.sh"
g add docs/technical-manual/tests/verify-examples.sh
g commit --quiet -m "s5: retarget technical-manual's verify-examples.sh symlink"
expect "S5 technical-manual, its verify-examples.sh symlink retargeted" true "$(relevant technical-manual push refs/heads/ci/staging 0000000000000000000000000000000000000000)"
# ...and a directory symlink repointed in the same commit as a change to its
# OLD target (the symlink change alone already makes it relevant; this pins
# that the combination is not mis-handled).
branch_from_master s5b
ln -sfn ../manual/tests "$W/docs/quickstart/transcripts"
echo '# touched' >> "$W/docs/manual/transcripts/22-first-bundle.cmd"
g add docs/quickstart/transcripts docs/manual/transcripts/22-first-bundle.cmd
g commit --quiet -m "s5b: retarget quickstart transcripts + touch the old target"
expect "S5b quickstart, transcripts symlink retargeted" true "$(relevant quickstart push refs/heads/ci/staging 0000000000000000000000000000000000000000)"

# ---- S6 (fix1 NEW-1): the symlink itself is deleted ----
for b in quickstart technical-manual manual-gui; do
  branch_from_master "s6-$b"
  g rm --quiet "docs/$b/tests/verify-examples.sh"
  g commit --quiet -m "s6: delete docs/$b/tests/verify-examples.sh (the symlink)"
  expect "S6 $b, its verify-examples.sh symlink deleted" true "$(relevant "$b" push refs/heads/ci/staging 0000000000000000000000000000000000000000)"
done

# ---- S7: derivation reads the BASE tree too (a dependency only the base shows) ----
branch_from_master s7
g rm --quiet docs/manual/.cspell.json
g commit --quiet -m "s7: delete a config quickstart imports by relative reference"
expect "S7 quickstart, its imported ../manual/.cspell.json deleted" true "$(relevant quickstart push refs/heads/ci/staging 0000000000000000000000000000000000000000)"
expect "S7-control quickstart with the round-1 guard (the defect)" false "$(relevant quickstart push refs/heads/ci/staging 0000000000000000000000000000000000000000 '' "$FIX1_REV")"

# ---- S8 (fix1 self-check): a repo-rooted tool only the Makefile names ----
# manual-gui's `make lint` runs $(TOOLKIT_ROOT)/docs/tools/render-mermaid-cache.py
# (figures-cache-verify); neither its old paths: filter nor its round-1 --also
# watched it.
branch_from_master s8
mkdir -p "$W/docs/tools"
echo '# touched' >> "$W/docs/tools/render-mermaid-cache.py"
g add --sparse docs/tools/render-mermaid-cache.py
g commit --quiet -m "s8: touch only the mermaid cache tool"
for wf in "${WFS[@]}"; do
  expect "S8 $wf, only docs/tools/render-mermaid-cache.py changed" true "$(relevant "$wf" push refs/heads/ci/staging 0000000000000000000000000000000000000000)"
done
expect "S8-control manual-gui with the round-1 guard (the defect)" false "$(relevant manual-gui push refs/heads/ci/staging 0000000000000000000000000000000000000000 '' "$FIX1_REV")"

# ---- negatives: the guard is not trivially true ----
branch_from_master n1
echo x > "$W/README-n1.txt"; g add README-n1.txt; g commit --quiet -m "n1: README only"
for wf in "${WFS[@]}"; do
  expect "N1 $wf, ci/staging push, README-only from master" false "$(relevant "$wf" push refs/heads/ci/staging 0000000000000000000000000000000000000000)"
  expect "N1 $wf, PR, README-only" false "$(relevant "$wf" pull_request refs/pull/2/merge '' master)"
done
# default-branch push diffs against `before` (the previous protected tip)
expect "N2 manual, master push, before=parent, README-only" false "$(relevant manual push refs/heads/master "$(g rev-parse HEAD~1)")"
expect "P1 manual, master push, before=master~1 of s3 C (docs change)" true "$(g checkout --quiet s3; relevant manual push refs/heads/master "$(g rev-parse HEAD~2)")"
g checkout --quiet n1
expect "P2 technical-manual, master push, README-only but the code crates/ touched" true "$(mkdir -p "$W/crates"; echo x > "$W/crates/x.txt"; g add --sparse -f crates/x.txt; g commit --quiet -m crates; relevant technical-manual push refs/heads/master "$(g rev-parse HEAD~1)")"
# fail-safe answers
expect "F1 tag push" true "$(relevant manual push refs/tags/manual-v9.9.9)"
expect "F2 workflow_dispatch" true "$(relevant manual workflow_dispatch refs/heads/master)"
expect "F3 master push with no previous tip" true "$(relevant manual push refs/heads/master 0000000000000000000000000000000000000000)"
expect "F4 master push with an unfetchable previous tip" true "$(relevant manual push refs/heads/master deadbeefdeadbeefdeadbeefdeadbeefdeadbeef)"
expect "F5 PR against a base that does not exist" true "$(relevant manual pull_request refs/pull/3/merge '' no-such-branch)"

printf '\n%d passed, %d failed\n' "$pass" "$fail"
[ "$fail" -eq 0 ]
