#!/usr/bin/env bash
# ci/shared-drift.sh -- check that the five shared CI files on each other
# public Part A repo's default branch are byte-identical to this checkout's.
#
# Each repo checks its shared files only against its own ci/shared-ci.sha256,
# so a shared file edited together with its hash line in one repo passes
# there. This compares the repos with each other. Run from the repo root.
# Needs bash, sha256sum and an authenticated gh (GH_TOKEN in CI).
# Prints one line per repo and file, then a PASS or FAIL line.
# Exit 0 = every pair matches.
set -eu
owner=${SHARED_DRIFT_OWNER:-bg002h}
gh=${SHARED_DRIFT_GH:-gh}
repos=(descriptor-mnemonic mnemonic-engrave mnemonic-key mnemonic-secret mnemonic-transaction)
files=(ci/docs-only.sh ci/docs-only.test.sh ci/code-docs-check.sh ci/code-docs-check.test.sh ci/shared-ci.sha256)
tmp=$(mktemp "${TMPDIR:-/tmp}/shared-drift.XXXXXX")
trap 'rm -f "$tmp"' EXIT

sha() { # sha <file>: print the file's sha256
  local h
  h=$(sha256sum < "$1")
  printf '%s\n' "${h%% *}"
}

k=0
for repo in "${repos[@]}"; do
  for f in "${files[@]}"; do
    if [ ! -f "$f" ]; then
      echo "ERROR $repo $f: missing locally"; k=$((k + 1)); continue
    fi
    # No ref: the contents API then reads the repo's default branch.
    if ! "$gh" api -H 'Accept: application/vnd.github.raw' "repos/$owner/$repo/contents/$f" > "$tmp" < /dev/null; then
      echo "ERROR $repo $f: cannot fetch"; k=$((k + 1)); continue
    fi
    here_sha=$(sha "$f")
    there_sha=$(sha "$tmp")
    if [ "$there_sha" = "$here_sha" ]; then
      echo "ok    $repo $f"
    else
      echo "DRIFT $repo $f: $there_sha, mnemonic-toolkit has $here_sha"; k=$((k + 1))
    fi
  done
done

if [ "$k" -eq 0 ]; then
  echo "shared-drift: PASS (${#repos[@]} repos, ${#files[@]} files)"
  exit 0
fi
echo "shared-drift: FAIL: $k problem(s)"
exit 1
