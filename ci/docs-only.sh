#!/usr/bin/env bash
# ci/docs-only.sh -- is this run's change documentation only?
#
# Writes `docs_only=true` to $GITHUB_OUTPUT only when it can PROVE that every
# file differing from an already-checked reference commit is documentation, as
# ci/docs-only-paths.txt defines it. Any doubt writes `docs_only=false`. The
# steps it guards carry `if: steps.docs.outputs.docs_only != 'true'`, so they
# also run when this script writes nothing at all. A crash fails the step,
# which fails the job.
#
# The reference commit R:
#   pull_request into the default branch   PR_BASE_SHA, the tip it targets;
#                                          failing that, PUSH_BEFORE, the
#                                          previous PR head (rule A', below)
#   push to the default branch             PUSH_BEFORE, the previous tip
#   push to any other branch               the current tip of origin/$DEFAULT_BRANCH
#   anything else (a PR into another branch, tags, dispatch, schedule): full run
# A push to a non-default branch (ci/staging) is compared with the default
# branch, never with its own previous tip: that tip may never have passed.
# The diff is tree to tree (`git diff R HEAD`), so a depth-1 checkout is
# enough; only R itself is fetched, at depth 1.
#
# Rule A' (a docs-only push to a PR that already changes code): when the
# PR_BASE_SHA rule fails, R is the previous PR head (github.event.before,
# passed as PUSH_BEFORE). HEAD is the PR's test-merge commit, so a base that
# gained code since R makes the diff non-docs and the job runs. A' also needs
# HEAD to be a two-parent merge, and the GitHub compare API to report
# <HEAD's first parent>...R as `ahead` or `identical`: the base this run
# merged is an ancestor of R. (Not PR_BASE_SHA: on a push to a PR it can be
# the base the PR was opened on.) Then, when R's run was for this PR into the
# default branch, every base it tested was an ancestor of R too, so it tested
# R's tree, and HEAD differs from that only in documentation. Two cases are
# not covered: an admin force push that rewinds the default branch during an
# open PR, and a PR retargeted onto the default branch after R's run (that
# run tested another base). Both are accepted limits (SPEC §5); the default
# branch's own push run after the merge still runs fully. Needs
# `contents: read` for the compare.
#
# R is not assumed to be good. Before writing `true`, the guard asks the
# GitHub API for this job's own check run on R (CHECK_NAME, the job's
# rendered name) and requires every latest run of that name to be completed
# with conclusion success. A red, pending or missing run, or any API error,
# means a full run. (With branch protection `strict: false`, a PR merged on a
# moved base puts an untested tree at the default tip; this check is what
# stops a docs-only change from reporting green on top of it.) A reference
# that was itself docs-skipped green is fine: its tree differs from a tested
# one only in documentation. Needs GH_TOKEN with `checks: read`;
# DOCS_ONLY_GH overrides the `gh` command (the tests use a stub).
#
# Any changed path that contains a newline forces a full run.
#
# ci/docs-only-paths.txt: one rule per line. `+ <ERE>` marks a path as
# documentation, `- <ERE>` protects a path even when a `+` rule matches it,
# `#` starts a comment, blank lines are ignored, anything else is an error.
# A changed file is documentation iff it matches at least one `+` rule and no
# `-` rule. Patterns are tested against the repo-relative path with grep -E.
#
# Runs on bash 3.2 (macOS) and Git Bash (Windows): no `set -u`, no
# associative arrays, no `mapfile`.

out=${GITHUB_OUTPUT:-/dev/stdout}
gh_cmd=${DOCS_ONLY_GH:-gh}
paths_file=${DOCS_ONLY_PATHS:-ci/docs-only-paths.txt}
zero=0000000000000000000000000000000000000000

full() {
  echo "docs_only=false" >> "$out"
  echo "docs-only: full run ($1)"
  exit 0
}

skip() {
  echo "docs_only=true" >> "$out"
  echo "docs-only: $1; skipping the guarded steps"
  exit 0
}

# match <ERE> <path>: 0 on a match, 1 on none; a bad pattern forces a full run.
match() {
  grep -qE -- "$1" <<<"$2"
  rc=$?
  [ "$rc" -le 1 ] || full "bad pattern in $paths_file: $1"
  return "$rc"
}

[ -r "$paths_file" ] || full "no readable $paths_file"
nallow=0
ndeny=0
while IFS= read -r line || [ -n "$line" ]; do
  line=${line%$'\r'}
  case $line in
    '+ '?*) allow[nallow]=${line#+ }; nallow=$((nallow + 1)) ;;
    '- '?*) deny[ndeny]=${line#- }; ndeny=$((ndeny + 1)) ;;
    '' | '#'*) ;;
    *) full "unparseable line in $paths_file: $line" ;;
  esac
done < "$paths_file"
[ "$nallow" -gt 0 ] || full "no + rules in $paths_file"

[ -n "${DEFAULT_BRANCH:-}" ] || full "DEFAULT_BRANCH is not set"
[ -n "${CHECK_NAME:-}" ] || full "CHECK_NAME is not set"

list=$(mktemp "${RUNNER_TEMP:-${TMPDIR:-/tmp}}/docs-only.XXXXXX") || full "mktemp failed"
trap 'rm -f "$list"' EXIT

# docs_since <ref>: fetch <ref> at depth 1 and return 0 when at least one file
# differs between it and HEAD and every such file is documentation. Sets
# $base (the fetched commit) and $n (the file count). Otherwise returns 1 and
# says why in $why. A path containing a newline, or a bad pattern, ends the
# run (full) at once.
docs_since() {
  why=
  base=
  n=0
  git fetch --no-tags --depth=1 origin "$1" > /dev/null 2>&1 || { why="cannot fetch $1"; return 1; }
  base=$(git rev-parse --verify -q 'FETCH_HEAD^{commit}') || { why="fetched $1 but FETCH_HEAD is not a commit"; return 1; }
  git -c core.quotePath=false diff --no-renames --name-only -z "$base" HEAD > "$list" ||
    { why="git diff $base HEAD failed"; return 1; }
  [ -s "$list" ] || { why="no file differs from $base"; return 1; }
  while IFS= read -r -d '' f; do
    case $f in *$'\n'*) full "a changed path contains a newline" ;; esac
    n=$((n + 1))
    ok=
    i=0
    while [ "$i" -lt "$nallow" ]; do
      if match "${allow[i]}" "$f"; then ok=1; break; fi
      i=$((i + 1))
    done
    [ -n "$ok" ] || { why="not documentation: $f"; return 1; }
    i=0
    while [ "$i" -lt "$ndeny" ]; do
      if match "${deny[i]}" "$f"; then why="protected: $f"; return 1; fi
      i=$((i + 1))
    done
  done < "$list"
  [ "$n" -gt 0 ] || { why="no file differs from $base"; return 1; }
  return 0
}

# passed <sha>: 0 when every latest CHECK_NAME check run on <sha> is
# completed with conclusion success; otherwise 1, with the reason in $why.
passed() {
  runs=$("$gh_cmd" api -X GET "repos/${GITHUB_REPOSITORY:-}/commits/$1/check-runs" \
    -f check_name="$CHECK_NAME" -f filter=latest \
    --jq '.check_runs[] | "\(.status)/\(.conclusion)"' 2> /dev/null) ||
    { why="cannot read check runs for $1"; return 1; }
  [ -n "$runs" ] || { why="no '$CHECK_NAME' check run on $1"; return 1; }
  while IFS= read -r r; do
    [ "$r" = "completed/success" ] || { why="'$CHECK_NAME' on $1 is $r, not completed/success"; return 1; }
  done <<<"$runs"
  return 0
}

case ${GITHUB_EVENT_NAME:-} in
  pull_request)
    [ "${GITHUB_BASE_REF:-}" = "$DEFAULT_BRANCH" ] || full "pull_request into ${GITHUB_BASE_REF:-an unset base}, not $DEFAULT_BRANCH"
    [ -n "${PR_BASE_SHA:-}" ] || full "PR_BASE_SHA is not set"
    if docs_since "$PR_BASE_SHA" && passed "$base"; then
      skip "all $n changed files are documentation, and '$CHECK_NAME' passed on $base"
    fi
    why_base=$why
    # Rule A': the previous PR head.
    prev=${PUSH_BEFORE:-}
    [ -n "$prev" ] || full "base: $why_base; previous PR head: none"
    case $prev in
      *[!0-9a-f]*) full "base: $why_base; previous PR head: '$prev' is not a commit SHA" ;;
    esac
    [ "${#prev}" -eq 40 ] || [ "${#prev}" -eq 64 ] ||
      full "base: $why_base; previous PR head: '$prev' is not a commit SHA"
    # The base this run merged is HEAD's first parent: HEAD is the test-merge
    # commit. (github.event.pull_request.base.sha can be the base the PR was
    # opened on, not the one merged now.) Its object holds its parents' ids
    # even in a depth-1 clone.
    parents=$(git cat-file -p HEAD 2> /dev/null | sed -n 's/^parent //p')
    set -- $parents
    [ $# -eq 2 ] || full "base: $why_base; previous PR head: HEAD has $# parent(s), not a two-parent test-merge commit"
    merged=$1
    # An all-zeros SHA cannot be fetched, so docs_since refuses it.
    docs_since "$prev" || full "base: $why_base; previous PR head: $why"
    status=$("$gh_cmd" api "repos/${GITHUB_REPOSITORY:-}/compare/$merged...$base" \
      --jq .status 2> /dev/null) ||
      full "base: $why_base; previous PR head: cannot compare $merged...$base"
    case $status in
      ahead | identical) ;;
      *) full "base: $why_base; previous PR head: $merged...$base is '$status', not ahead or identical" ;;
    esac
    passed "$base" || full "base: $why_base; previous PR head: $why"
    skip "all $n files changed since the previous PR head $base are documentation, the merged base $merged is an ancestor of it, and '$CHECK_NAME' passed on it"
    ;;
  push)
    case ${GITHUB_REF:-} in
      "refs/heads/$DEFAULT_BRANCH")
        if [ -z "${PUSH_BEFORE:-}" ] || [ "$PUSH_BEFORE" = "$zero" ]; then
          full "push with no previous tip"
        fi
        fetch=$PUSH_BEFORE ;;
      refs/heads/?*) fetch="refs/heads/$DEFAULT_BRANCH" ;;
      *) full "push to ${GITHUB_REF:-an unset ref}" ;;
    esac
    docs_since "$fetch" || full "$why"
    passed "$base" || full "$why"
    skip "all $n changed files are documentation, and '$CHECK_NAME' passed on $base"
    ;;
  *) full "event ${GITHUB_EVENT_NAME:-unset}" ;;
esac
