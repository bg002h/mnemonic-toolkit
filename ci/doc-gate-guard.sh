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
# RULE A' (SPEC-after-b.md §3.2; SPEC.md §2.1's rule A', with this gate's
# watched test in place of the docs-paths test). Against the merge-base, once
# a PR changes a watched path, every later push to it runs the heavy steps
# again, even a push that touches nothing watched. So on `pull_request` only,
# after the test above has found a watched change, the guard tries the
# previous PR head B (`github.event.before`, passed as PUSH_BEFORE). It writes
# relevant=false only when ALL of these hold, in this order:
#   1. GITHUB_BASE_REF is the default branch (other branches can be rewound
#      by anyone with push access);
#   2. CHECK_NAME (this job's check name) is set;
#   3. B is 40 or 64 characters, all 0-9a-f;
#   4. HEAD has exactly two parents (GitHub's test-merge commit); the first,
#      M, is the base this run merged;
#   5. B is present, or fetches (no --depth, so a full clone stays full);
#   6. the watched set derived from B's tree and HEAD's succeeds;
#   7. `git diff -z --no-renames B HEAD` succeeds and lists at least one file,
#      and no listed file is watched or matches --also;
#   8. the compare API reports M...B as `ahead` or `identical`: M is an
#      ancestor of B;
#   9. every latest CHECK_NAME check run on B is completed/success.
# The default branch only moves forward, so every base B's run merged was an
# ancestor of B too: that run tested B's own tree, and it passed. HEAD is the
# test merge, not the new PR head, so a base that gained a watched change
# since B makes condition 7 fail. When a condition fails, the answer is
# relevant=true with the reason "<the reason above> (A': <why>)", where <why>
# is the first failing condition. Every other event, and every path to
# relevant=false without A', is unchanged. Accepted limits (SPEC-after-b.md
# §6 item 1): an admin force-push that rewinds the default branch during an
# open PR, and a PR retargeted onto the default branch after B's run; the
# default branch's own push run after the merge still runs in full. Needs
# GH_TOKEN with `checks: read`; DOC_GATE_GH overrides the `gh` command (the
# tests use a stub).
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
GH="${DOC_GATE_GH:-gh}"

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

# changed_files A B -> the files that differ between A and B, one per line,
# unquoted; returns non-zero when git diff fails. -z, because without it git
# C-quotes a path holding a non-ASCII, quote or control character
# ("crates/\303\251.rs", quotes included), which then matches nothing
# watched: a silently skipped gate (ab-sec-quotepath). A path holding a
# newline still splits into lines (it is never quoted, only split).
changed_files() {
  git diff -z --no-renames --name-only "$1" "$2" 2>/dev/null | tr '\0' '\n'
}

# first_hit WATCHED CHANGED -> for the first CHANGED file that is watched or
# matches --also, prints why and returns 0; returns 1 when there is none. Both
# arguments are newline-separated lists. It never calls `decide` (which
# exits): rule A' calls it too, and must go on to its own answer.
# Here-strings throughout, never `printf | grep -q`: under pipefail, grep -q
# exiting on the first match can SIGPIPE the printf, fail the pipeline, and
# read as "nothing changed" -- a silently skipped gate, the one unsafe outcome.
first_hit() {
  local f w
  while IFS= read -r f; do
    [ -z "$f" ] && continue
    while IFS= read -r w; do
      [ -z "$w" ] && continue
      case "$w" in
        */) [ "${f#"$w"}" != "$f" ] && { printf '%s\n' "$f is under watched $w"; return 0; } ;;
        *)  [ "$f" = "$w" ] && { printf '%s\n' "$f is watched"; return 0; } ;;
      esac
    done <<<"$1"
    if [ -n "$also" ] && grep -qE -- "$also" <<<"$f"; then
      printf '%s\n' "$f matches --also"; return 0
    fi
  done <<<"$2"
  return 1
}

# a_prime -> rule A' (header), conditions 1-9 in order. When all hold it calls
# `decide false`, which exits. Otherwise it sets $why to the first failing
# condition's text and returns 1.
a_prime() {
  local before="${PUSH_BEFORE:-}" hdr line n=0 merged='' w2 c2 h status runs r
  # 1. A pull request into the default branch only.
  [ "${GITHUB_BASE_REF:-}" = "$default" ] \
    || { why="pull_request into ${GITHUB_BASE_REF:-}, not $default"; return 1; }
  # 2. The check name to look up on B.
  [ -n "${CHECK_NAME:-}" ] || { why="CHECK_NAME is not set"; return 1; }
  # 3. B is a full commit SHA (SHA-1 or SHA-256), lower-case hex.
  case "$before" in
    '') why="no previous PR head"; return 1 ;;
    *[!0-9a-f]*) why="previous PR head '$before' is not a commit SHA"; return 1 ;;
  esac
  { [ "${#before}" -eq 40 ] || [ "${#before}" -eq 64 ]; } \
    || { why="previous PR head '$before' is not a commit SHA"; return 1; }
  # 4. HEAD is the two-parent test-merge commit; M is its first parent. Its
  # own object names its parents even in a shallow clone. Only the header is
  # read: it ends at the first blank line, before the message.
  hdr=$(git cat-file commit HEAD 2>/dev/null) || hdr=''
  while IFS= read -r line; do
    [ -z "$line" ] && break
    case "$line" in
      "parent "*) n=$((n + 1)); [ "$n" -eq 1 ] && merged=${line#parent } ;;
    esac
  done <<<"$hdr"
  [ "$n" -eq 2 ] || { why="HEAD has $n parent(s), not a two-parent test-merge commit"; return 1; }
  # 5. B is present or fetches (no --depth: a full clone stays full).
  git cat-file -e "${before}^{commit}" 2>/dev/null \
    || git fetch --no-tags --quiet origin "$before" 2>/dev/null \
    || { why="cannot fetch $before"; return 1; }
  # 6. The watched set, derived from B's tree and HEAD's.
  w2=$(python3 "$here/doc-gate-paths.py" --rev "$before" --rev HEAD "${books[@]}") \
    || { why="could not derive the watched set at $before"; return 1; }
  # 7. Something differs from B, and nothing that differs is watched.
  c2=$(changed_files "$before" HEAD) \
    || { why="git diff $before HEAD failed"; return 1; }
  [ -n "$c2" ] || { why="no file differs from $before"; return 1; }
  if h=$(first_hit "$w2" "$c2"); then why="$h since $before"; return 1; fi
  # 8. The merged base M is an ancestor of B.
  status=$("$GH" api "repos/${GITHUB_REPOSITORY:-}/compare/$merged...$before" --jq .status 2>/dev/null) \
    || { why="cannot compare $merged...$before"; return 1; }
  case "$status" in
    ahead | identical) ;;
    *) why="$merged...$before is '$status', not ahead or identical"; return 1 ;;
  esac
  # 9. Every latest CHECK_NAME check run on B is completed/success.
  runs=$("$GH" api -X GET "repos/${GITHUB_REPOSITORY:-}/commits/$before/check-runs" \
           -f check_name="$CHECK_NAME" -f filter=latest \
           --jq '.check_runs[] | "\(.status)/\(.conclusion)"' 2>/dev/null) \
    || { why="cannot read check runs for $before"; return 1; }
  [ -n "$runs" ] || { why="no '$CHECK_NAME' check run on $before"; return 1; }
  while IFS= read -r r; do
    [ "$r" = completed/success ] \
      || { why="'$CHECK_NAME' on $before is $r, not completed/success"; return 1; }
  done <<<"$runs"
  decide false "A': no watched path changed since the previous PR head $before, the merged base $merged is an ancestor of it, and '$CHECK_NAME' passed on it"
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

changed=$(changed_files "$base" HEAD) \
  || decide true "git diff $base HEAD failed"
printf 'changed files (vs %s):\n%s\n' "$base" "${changed:-<none>}"

hit=$(first_hit "$watched" "$changed") || decide false "no watched path changed"
if [ "$event" = pull_request ]; then
  why=''
  a_prime || decide true "$hit (A': $why)"
fi
decide true "$hit"
