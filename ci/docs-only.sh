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
#   pull_request into the default branch   PR_BASE_SHA, the tip it targets
#   push to the default branch             PUSH_BEFORE, the previous tip
#   push to any other branch               the current tip of origin/$DEFAULT_BRANCH
#   anything else (a PR into another branch, tags, dispatch, schedule): full run
# A push to a non-default branch (ci/staging) is compared with the default
# branch, never with its own previous tip: that tip may never have passed.
# The diff is tree to tree (`git diff R HEAD`), so a depth-1 checkout is
# enough; only R itself is fetched, at depth 1.
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
base=
case ${GITHUB_EVENT_NAME:-} in
  pull_request)
    [ "${GITHUB_BASE_REF:-}" = "$DEFAULT_BRANCH" ] || full "pull_request into ${GITHUB_BASE_REF:-an unset base}, not $DEFAULT_BRANCH"
    [ -n "${PR_BASE_SHA:-}" ] || full "PR_BASE_SHA is not set"
    fetch=$PR_BASE_SHA ;;
  push)
    case ${GITHUB_REF:-} in
      "refs/heads/$DEFAULT_BRANCH")
        if [ -z "${PUSH_BEFORE:-}" ] || [ "$PUSH_BEFORE" = "$zero" ]; then
          full "push with no previous tip"
        fi
        fetch=$PUSH_BEFORE ;;
      refs/heads/?*) fetch="refs/heads/$DEFAULT_BRANCH" ;;
      *) full "push to ${GITHUB_REF:-an unset ref}" ;;
    esac ;;
  *) full "event ${GITHUB_EVENT_NAME:-unset}" ;;
esac

git fetch --no-tags --depth=1 origin "$fetch" > /dev/null 2>&1 || full "cannot fetch $fetch"
base=$(git rev-parse --verify -q 'FETCH_HEAD^{commit}') || full "fetched $fetch but FETCH_HEAD is not a commit"

list=$(mktemp "${RUNNER_TEMP:-${TMPDIR:-/tmp}}/docs-only.XXXXXX") || full "mktemp failed"
trap 'rm -f "$list"' EXIT
git -c core.quotePath=false diff --no-renames --name-only -z "$base" HEAD > "$list" || full "git diff $base HEAD failed"
[ -s "$list" ] || full "no file differs from $base"

n=0
while IFS= read -r -d '' f; do
  case $f in *$'\n'*) full "a changed path contains a newline" ;; esac
  n=$((n + 1))
  ok=
  i=0
  while [ "$i" -lt "$nallow" ]; do
    if match "${allow[i]}" "$f"; then ok=1; break; fi
    i=$((i + 1))
  done
  [ -n "$ok" ] || full "not documentation: $f"
  i=0
  while [ "$i" -lt "$ndeny" ]; do
    if match "${deny[i]}" "$f"; then full "protected: $f"; fi
    i=$((i + 1))
  done
done < "$list"
[ "$n" -gt 0 ] || full "no file differs from $base"

# Every changed file is documentation. Now prove R itself passed this job.
runs=$("$gh_cmd" api -X GET "repos/${GITHUB_REPOSITORY:-}/commits/$base/check-runs" \
  -f check_name="$CHECK_NAME" -f filter=latest \
  --jq '.check_runs[] | "\(.status)/\(.conclusion)"' 2> /dev/null) || full "cannot read check runs for $base"
[ -n "$runs" ] || full "no '$CHECK_NAME' check run on $base"
while IFS= read -r r; do
  [ "$r" = "completed/success" ] || full "'$CHECK_NAME' on $base is $r, not completed/success"
done <<<"$runs"

echo "docs_only=true" >> "$out"
echo "docs-only: all $n changed files are documentation, and '$CHECK_NAME' passed on $base; skipping the guarded steps"
exit 0
