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
# The reference commit is one whose tree already passed the required checks:
#   pull_request               PR_BASE_SHA, the base branch tip the PR targets
#   push to the default branch PUSH_BEFORE, the previous tip
#   push to any other branch   the current tip of origin/$DEFAULT_BRANCH
#   anything else              (tags, dispatch, schedule, workflow_call) full run
# A push to a non-default branch (ci/staging) is compared with the default
# branch, never with its own previous tip: that tip may never have passed.
# The diff is tree to tree (`git diff R HEAD`), so a depth-1 checkout is
# enough; only R itself is fetched, at depth 1.
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
  case $line in
    '+ '?*) allow[nallow]=${line#+ }; nallow=$((nallow + 1)) ;;
    '- '?*) deny[ndeny]=${line#- }; ndeny=$((ndeny + 1)) ;;
    '' | '#'*) ;;
    *) full "unparseable line in $paths_file: $line" ;;
  esac
done < "$paths_file"
[ "$nallow" -gt 0 ] || full "no + rules in $paths_file"

[ -n "${DEFAULT_BRANCH:-}" ] || full "DEFAULT_BRANCH is not set"
base=
case ${GITHUB_EVENT_NAME:-} in
  pull_request)
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

listing=$(git -c core.quotePath=false diff --no-renames --name-only -z "$base" HEAD | tr '\0' '\n'; echo "rc=${PIPESTATUS[0]}")
case $listing in
  *rc=0) listing=${listing%rc=0} ;;
  *) full "git diff $base HEAD failed" ;;
esac
[ -n "${listing%$'\n'}" ] || full "no file differs from $base"

n=0
while IFS= read -r f; do
  [ -n "$f" ] || continue
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
done <<<"$listing"

echo "docs_only=true" >> "$out"
echo "docs-only: all $n changed files are documentation (against $base); skipping the guarded steps"
exit 0
