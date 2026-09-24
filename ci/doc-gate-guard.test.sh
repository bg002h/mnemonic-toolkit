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
PREFIX_REV=6ce7e464   # the guard the review found the counterexamples in
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

# relevant WF EVENT REF [BEFORE] [BASE_REF] [OLD] -> prints true|false|error
relevant() {
  local wf=$1 event=$2 ref=$3 before=${4:-} base_ref=${5:-} old=${6:-} cmd dir out ev
  dir="$W"
  if [ -n "$old" ]; then
    cmd=$(guard_line "$wf" "$PREFIX_REV") || { echo n/a; return; }
    rm -rf "$TMP/old"; mkdir -p "$TMP/old/ci"
    git -C "$W" show "$PREFIX_REV:ci/doc-gate-guard.sh" > "$TMP/old/ci/doc-gate-guard.sh" 2>/dev/null || { echo n/a; return; }
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
  if [ "$3" = n/a ]; then printf 'skip %s (pre-fix rev %s not in history)\n' "$1" "$PREFIX_REV"; return; fi
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
