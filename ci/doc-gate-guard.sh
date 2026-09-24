#!/usr/bin/env bash
# ci/doc-gate-guard.sh — decide whether a documentation gate has work to do.
#
#   usage: bash ci/doc-gate-guard.sh '<ERE over repo-relative paths>'
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
# gated path changed. A skipped-because-irrelevant run is a fast green.
# This is examples.yml's always-run PR pattern, extended to `push`: a doc
# gate must also report on the `ci/staging` push that earns a commit its
# contexts before it lands on master, including a code-only commit.
#
# FAIL-SAFE: the guard may only ever err toward RUNNING the gate. Every
# situation in which it cannot establish a base to diff against — a tag, a
# manual dispatch, a new branch whose base cannot be fetched, a force-push
# whose `before` is gone, any git error — yields relevant=true. The diff is
# two-dot (base tree vs HEAD tree), which over-includes changes on the base
# side and so can run the gate unnecessarily but never miss a change on the
# head side.
set -uo pipefail

re="${1:?usage: doc-gate-guard.sh '<ERE of gated paths>'}"
out="${GITHUB_OUTPUT:-/dev/stdout}"
event="${GITHUB_EVENT_NAME:-}"
ref="${GITHUB_REF:-}"

decide() { # $1 = true|false, $2 = reason
  echo "doc-gate-guard: relevant=$1 ($2)"
  echo "relevant=$1" >> "$out"
  exit 0
}

zero='0000000000000000000000000000000000000000'
base=''
case "$event" in
  pull_request)
    git fetch --no-tags --depth=1 origin "${GITHUB_BASE_REF:?}" 2>/dev/null \
      && base=$(git rev-parse FETCH_HEAD 2>/dev/null) \
      || decide true "could not fetch the PR base ${GITHUB_BASE_REF:-?}"
    ;;
  push)
    case "$ref" in refs/tags/*) decide true "tag push $ref" ;; esac
    before=''
    if [ -n "${GITHUB_EVENT_PATH:-}" ] && [ -f "$GITHUB_EVENT_PATH" ]; then
      before=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1])).get("before") or "")' \
                 "$GITHUB_EVENT_PATH" 2>/dev/null || true)
    fi
    if [ -n "$before" ] && [ "$before" != "$zero" ] \
       && { git cat-file -e "${before}^{commit}" 2>/dev/null \
            || git fetch --no-tags --depth=1 origin "$before" 2>/dev/null; }; then
      base="$before"
    else
      # A new branch (ci/staging is created fresh for every push window) or an
      # unfetchable `before`: compare against the default branch the commit is
      # about to land on.
      default="${DEFAULT_BRANCH:-master}"
      git fetch --no-tags --depth=1 origin "$default" 2>/dev/null \
        && base=$(git rev-parse FETCH_HEAD 2>/dev/null) \
        || decide true "no usable push base (before=${before:-none}, $default unfetchable)"
    fi
    ;;
  *)
    decide true "event '${event:-none}' always runs the gate"
    ;;
esac

changed=$(git diff --name-only "$base" HEAD 2>/dev/null) \
  || decide true "git diff $base HEAD failed"
printf 'changed files (vs %s):\n%s\n' "$base" "${changed:-<none>}"
# A here-string, never `printf | grep -q`: under pipefail, grep -q exiting on
# the first match can SIGPIPE the printf, fail the pipeline, and read as "no
# gated path changed" -- a silently skipped gate, the one unsafe outcome.
if grep -qE -- "$re" <<<"$changed"; then
  decide true "a gated path changed"
fi
decide false "no gated path changed"
