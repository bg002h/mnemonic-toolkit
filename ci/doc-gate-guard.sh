#!/usr/bin/env bash
# ci/doc-gate-guard.sh — decide whether a documentation gate has work to do.
#
#   usage: bash ci/doc-gate-guard.sh --book <docs/dir> [--book …] [--also '<ERE>']
#   writes `relevant=true|false` to $GITHUB_OUTPUT (stdout when unset)
#
# WHY IT EXISTS. The doc workflows (manual, quickstart, technical-manual,
# manual-gui) used to filter their `push:` and `pull_request:` triggers with
# `paths:`. A path-filtered workflow cannot be a REQUIRED status check: on a
# commit that touches none of its paths the workflow never starts, its context
# never appears, and GitHub waits for it forever ("Expected — waiting for
# status"). scripts/push-via-staging.sh then stops with "required context(s)
# NEVER RAN". So the doc gates were never required, and they went red on
# master unnoticed.
#
# The workflows now trigger on EVERY push and pull request, so their context
# always reports; this guard runs first and the heavy steps run only when a
# watched path changed. A skipped-because-irrelevant run is a fast green.
#
# WHAT IS WATCHED (review I-1). Each --book directory, PLUS everything the
# book reaches outside itself, DERIVED by ci/doc-gate-paths.py from the
# git trees of BOTH the diff base and HEAD (union): lexical symlink targets
# and `../` references in the book's Makefiles, scripts, filters and configs,
# to a fixed point -- so deleting or retargeting a shared file is still seen. E.g.
# technical-manual watches docs/manual/tests/verify-examples.sh (its
# tests/verify-examples.sh is a symlink to it). --also adds what no file
# spells as a relative path: crates/, Cargo.*, the mermaid cache tool, the
# workflow file, this guard.
#
# WHAT IT DIFFS AGAINST (review I-3). The base must be a commit that already
# PASSED this gate, or the diff can hide an ungated change underneath:
#   * push to the default branch (master/main): the event's `before` — the
#     previous tip of the protected branch;
#   * push to ANY other branch (ci/staging, topics): the merge-base of HEAD and
#     origin/<default>, never `before`. A ci/staging left behind by an
#     interrupted push window may hold an unchecked commit; a README-only
#     commit pushed on top of it must still see that commit's changes;
#   * pull_request: the merge-base of HEAD and origin/<base>.
# The diff runs with --no-renames (review I-2): a rename then prints BOTH the
# old and the new path, so moving a watched file out of scope is seen.
#
# FAIL-SAFE: the guard may only ever err toward RUNNING the gate. A tag, a
# manual dispatch, an unfetchable base, a missing merge-base history, or any
# git/python error all yield relevant=true.
#
# `[skip ci]` (review N-2): GitHub does not start ANY workflow for a commit
# whose message carries [skip ci] / [ci skip] / [no ci] / [skip actions] /
# [actions skip], so this guard never runs and the required doc contexts never
# appear for that SHA. That is fail-closed: scripts/push-via-staging.sh stops
# with "required context(s) NEVER RAN", and such a commit cannot be pushed to
# master through staging. Do not use [skip ci] on commits bound for master.
set -uo pipefail

books=()
also=''
while [ $# -gt 0 ]; do
  case "$1" in
    --book) books+=("${2:?--book needs a directory}"); shift 2 ;;
    --also) also="${2:?--also needs an ERE}"; shift 2 ;;
    *) echo "doc-gate-guard: unknown argument $1" >&2; exit 64 ;;
  esac
done
[ "${#books[@]}" -gt 0 ] || { echo "doc-gate-guard: at least one --book is required" >&2; exit 64; }

out="${GITHUB_OUTPUT:-/dev/stdout}"
event="${GITHUB_EVENT_NAME:-}"
ref="${GITHUB_REF:-}"
default="${DEFAULT_BRANCH:-master}"
here="$(cd "$(dirname "$0")" && pwd)"

decide() { # $1 = true|false, $2 = reason
  echo "doc-gate-guard: relevant=$1 ($2)"
  echo "relevant=$1" >> "$out"
  exit 0
}

# merge_base_with BRANCH -> prints the merge-base of HEAD and origin/BRANCH,
# fetching that branch first; returns non-zero on failure.
merge_base_with() {
  git fetch --no-tags --quiet origin "+refs/heads/$1:refs/remotes/origin/$1" 2>/dev/null || return 1
  git merge-base HEAD "refs/remotes/origin/$1" 2>/dev/null && return 0
  # A shallow clone may lack the merge-base history: deepen it rather than guess.
  [ "$(git rev-parse --is-shallow-repository 2>/dev/null)" = true ] || return 1
  git fetch --no-tags --quiet --unshallow origin 2>/dev/null || return 1
  git merge-base HEAD "refs/remotes/origin/$1" 2>/dev/null
}

zero='0000000000000000000000000000000000000000'
base=''
case "$event" in
  pull_request)
    base=$(merge_base_with "${GITHUB_BASE_REF:?}") \
      || decide true "no merge-base with the PR base ${GITHUB_BASE_REF:-?}"
    ;;
  push)
    case "$ref" in
      refs/tags/*) decide true "tag push $ref" ;;
      "refs/heads/$default" | refs/heads/main | refs/heads/master)
        before=''
        if [ -n "${GITHUB_EVENT_PATH:-}" ] && [ -f "$GITHUB_EVENT_PATH" ]; then
          before=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1])).get("before") or "")' \
                     "$GITHUB_EVENT_PATH" 2>/dev/null || true)
        fi
        { [ -n "$before" ] && [ "$before" != "$zero" ]; } \
          || decide true "default-branch push without a previous tip"
        git cat-file -e "${before}^{commit}" 2>/dev/null \
          || git fetch --no-tags --quiet --depth=1 origin "$before" 2>/dev/null \
          || decide true "previous tip $before is unfetchable"
        base="$before"
        ;;
      *)
        base=$(merge_base_with "$default") \
          || decide true "no merge-base with origin/$default"
        ;;
    esac
    ;;
  *)
    decide true "event '${event:-none}' always runs the gate"
    ;;
esac

# The watched set is derived from BOTH trees and unioned (review fix1 NEW-1):
# a file a book reaches only by symlink or reference can be DELETED or
# retargeted in HEAD, and then only the base tree still shows the dependency.
watched=$(python3 "$here/doc-gate-paths.py" --rev "$base" --rev HEAD "${books[@]}") \
  || decide true "could not derive the watched set"
printf 'watched (derived from %s at %s and HEAD):\n%s\n' "${books[*]}" "$base" "$watched"
[ -n "$also" ] && printf 'also: %s\n' "$also"

changed=$(git diff --no-renames --name-only "$base" HEAD 2>/dev/null) \
  || decide true "git diff $base HEAD failed"
printf 'changed files (vs %s):\n%s\n' "$base" "${changed:-<none>}"

# Here-strings throughout, never `printf | grep -q`: under pipefail, grep -q
# exiting on the first match can SIGPIPE the printf, fail the pipeline, and
# read as "nothing changed" -- a silently skipped gate, the one unsafe outcome.
while IFS= read -r f; do
  [ -z "$f" ] && continue
  while IFS= read -r w; do
    [ -z "$w" ] && continue
    case "$w" in
      */) [ "${f#"$w"}" != "$f" ] && decide true "$f is under watched $w" ;;
      *)  [ "$f" = "$w" ] && decide true "$f is watched" ;;
    esac
  done <<<"$watched"
  if [ -n "$also" ] && grep -qE -- "$also" <<<"$f"; then
    decide true "$f matches --also"
  fi
done <<<"$changed"
decide false "no watched path changed"
