#!/usr/bin/env bash
# ci/docs-only.test.sh -- scenario tests for ci/docs-only.sh.
#
# Builds a scratch origin (file://, so shallow fetches by SHA behave as they
# do against GitHub) and a depth-1 clone, then runs the guard under each
# event shape and checks the single `docs_only=` line it writes. Needs only
# git and bash. Never touches the real remote. Exit 0 = every scenario passed.
set -eu
here=$(cd "$(dirname "$0")" && pwd)
guard=$here/docs-only.sh
tmp=$(mktemp -d "${DOCS_ONLY_TEST_TMP:-${TMPDIR:-/tmp}}/docs-only-test.XXXXXX")
trap 'rm -rf "$tmp"' EXIT
export GIT_CONFIG_GLOBAL=$tmp/gitconfig GIT_CONFIG_NOSYSTEM=1
git config --global user.email test@example.invalid
git config --global user.name docs-only-test
git config --global init.defaultBranch main
git config --global protocol.file.allow always

cat > "$tmp/paths.txt" <<'EOF'
# test rules
+ ^design/
+ ^[^/]+\.md$
- ^design/protected\.tsv$
EOF

# A stand-in for `gh`: appends its arguments to $GH_ARGS and exits $FAKE_RC.
# A compare call prints $FAKE_COMPARE (default "ahead") or exits
# $FAKE_CMP_RC. A check-runs call for commit $FAKE_PREV prints
# $FAKE_RUNS_PREV (default "completed/success"); any other call prints
# $FAKE_RUNS (one "status/conclusion" line per check run).
cat > "$tmp/gh" <<'EOF'
#!/usr/bin/env bash
printf '%s\n' "$@" >> "$GH_ARGS"
[ "${FAKE_RC:-0}" -eq 0 ] || exit "$FAKE_RC"
case "$*" in
  *compare/*)
    [ "${FAKE_CMP_RC:-0}" -eq 0 ] || exit "$FAKE_CMP_RC"
    printf '%s\n' "${FAKE_COMPARE-ahead}" ;;
  *"commits/${FAKE_PREV:-none}/"*) printf '%s\n' "${FAKE_RUNS_PREV-completed/success}" ;;
  *) printf '%s\n' "$FAKE_RUNS" ;;
esac
EOF
chmod +x "$tmp/gh"

git init -q --bare "$tmp/origin.git"
git -C "$tmp/origin.git" config uploadpack.allowAnySHA1InWant true
git init -q "$tmp/w"
cd "$tmp/w"
git remote add origin "file://$tmp/origin.git"
mkdir -p design src vendor/x
echo a > src/lib.rs; echo a > design/notes.md; echo a > README.md
echo a > design/protected.tsv; echo a > vendor/x/README.md
git add -A; git commit -qm c0; c0=$(git rev-parse HEAD)
git push -q origin HEAD:main

commit() { # commit <message> <file>... : append a line to each file, commit
  m=$1; shift
  for f in "$@"; do mkdir -p "$(dirname "$f")"; echo "$m" >> "$f"; done
  git add -A; git commit -qm "$m"; git rev-parse HEAD
}

pass=0 fail=0
check() { # check <name> <want true|false> <env...>
  name=$1 want=$2; shift 2
  rm -rf "$tmp/c"
  git clone -q --depth=1 --no-local --branch "$branch" "file://$tmp/origin.git" "$tmp/c" 2> /dev/null
  [ -z "${detach:-}" ] || git -C "$tmp/c" -c advice.detachedHead=false checkout -q "$detach"
  : > "$tmp/out"; : > "$tmp/ghargs"
  (cd "$tmp/c" && env -i PATH="$PATH" HOME="$tmp" GIT_CONFIG_GLOBAL="$GIT_CONFIG_GLOBAL" GIT_CONFIG_NOSYSTEM=1 \
     GITHUB_OUTPUT="$tmp/out" DOCS_ONLY_PATHS="${paths:-$tmp/paths.txt}" \
     DOCS_ONLY_GH="$tmp/gh" GH_ARGS="$tmp/ghargs" GITHUB_REPOSITORY=o/r CHECK_NAME=test \
     FAKE_RUNS=completed/success "$@" bash "$guard" > "$tmp/log" 2>&1) || {
    echo "FAIL $name: guard exited non-zero"; cat "$tmp/log"; fail=$((fail + 1)); return; }
  got=$(cat "$tmp/out")
  if [ "$got" = "docs_only=$want" ]; then pass=$((pass + 1)); echo "ok   $name ($(tail -1 "$tmp/log"))"
  else fail=$((fail + 1)); echo "FAIL $name: wrote '$got', want docs_only=$want ($(tail -1 "$tmp/log"))"; fi
}

PUSH='GITHUB_EVENT_NAME=push DEFAULT_BRANCH=main'
PR='GITHUB_EVENT_NAME=pull_request DEFAULT_BRANCH=main GITHUB_BASE_REF=main'

# --- pushes to the default branch ------------------------------------------
c1=$(commit docs1 design/notes.md README.md); git push -q origin HEAD:main
branch=main detach= check "push main, docs only vs previous tip" true $PUSH GITHUB_REF=refs/heads/main PUSH_BEFORE=$c0
c2=$(commit code1 src/lib.rs); git push -q origin HEAD:main
branch=main detach= check "push main, code vs previous tip" false $PUSH GITHUB_REF=refs/heads/main PUSH_BEFORE=$c1
branch=main detach= check "push main, zero before (new branch)" false $PUSH GITHUB_REF=refs/heads/main PUSH_BEFORE=0000000000000000000000000000000000000000
branch=main detach= check "push main, empty before" false $PUSH GITHUB_REF=refs/heads/main PUSH_BEFORE=
branch=main detach= check "push main, unfetchable before" false $PUSH GITHUB_REF=refs/heads/main PUSH_BEFORE=1234567890123456789012345678901234567890
branch=main detach= check "push main, same tree as before (empty diff)" false $PUSH GITHUB_REF=refs/heads/main PUSH_BEFORE=$c2

# --- pushes to another branch (ci/staging) ----------------------------------
s1=$(commit docs2 design/notes.md); git push -q origin HEAD:refs/heads/ci/staging
branch=ci/staging detach= check "push ci/staging, main + docs" true $PUSH GITHUB_REF=refs/heads/ci/staging PUSH_BEFORE=$c2
git reset -q --hard "$c2"; s2=$(commit code2 src/lib.rs); git push -q -f origin HEAD:refs/heads/ci/staging
branch=ci/staging detach= check "push ci/staging, main + code" false $PUSH GITHUB_REF=refs/heads/ci/staging PUSH_BEFORE=$s1
git reset -q --hard "$s2"; s3=$(commit docs3 design/notes.md); git push -q origin HEAD:refs/heads/ci/staging
branch=ci/staging detach= check "push ci/staging, docs on top of an unchecked code tip (before is ignored)" false $PUSH GITHUB_REF=refs/heads/ci/staging PUSH_BEFORE=$s2
git reset -q --hard "$c1"; s4=$(commit docs4 design/notes.md); git push -q -f origin HEAD:refs/heads/ci/staging
branch=ci/staging detach= check "push ci/staging, docs on an OLD main (main gained code since)" false $PUSH GITHUB_REF=refs/heads/ci/staging PUSH_BEFORE=$s3
branch=ci/staging detach= check "push ci/staging, DEFAULT_BRANCH unset" false GITHUB_EVENT_NAME=push GITHUB_REF=refs/heads/ci/staging
branch=ci/staging detach= check "push ci/staging, DEFAULT_BRANCH names no branch" false GITHUB_EVENT_NAME=push DEFAULT_BRANCH=nope GITHUB_REF=refs/heads/ci/staging

# --- pull requests (HEAD is the PR's test-merge commit) ---------------------
git reset -q --hard "$c2"
p1=$(commit docs5 design/new\ dir/with\ space.md); git push -q -f origin HEAD:refs/heads/pr-docs
branch=pr-docs detach= check "pull_request, docs only (path with spaces)" true $PR PR_BASE_SHA=$c2
git reset -q --hard "$c2"; p2=$(commit mixed design/notes.md src/lib.rs); git push -q -f origin HEAD:refs/heads/pr-mixed
branch=pr-mixed detach= check "pull_request, docs + code" false $PR PR_BASE_SHA=$c2
git reset -q --hard "$c2"; p3=$(commit prot design/protected.tsv); git push -q -f origin HEAD:refs/heads/pr-prot
branch=pr-prot detach= check "pull_request, protected path under design/" false $PR PR_BASE_SHA=$c2
git reset -q --hard "$c2"; p4=$(commit vend vendor/x/README.md); git push -q -f origin HEAD:refs/heads/pr-vend
branch=pr-vend detach= check "pull_request, nested .md (vendor/) is not docs" false $PR PR_BASE_SHA=$c2
git reset -q --hard "$c2"; p5=$(commit wf .github/workflows/ci.yml); git push -q -f origin HEAD:refs/heads/pr-wf
branch=pr-wf detach= check "pull_request, workflow change is code" false $PR PR_BASE_SHA=$c2
branch=pr-docs detach= check "pull_request, PR_BASE_SHA unset" false $PR

# --- other events and bad configuration -------------------------------------
branch=main detach= check "tag push" false $PUSH GITHUB_REF=refs/tags/v1 PUSH_BEFORE=$c1
branch=main detach= check "workflow_dispatch" false GITHUB_EVENT_NAME=workflow_dispatch DEFAULT_BRANCH=main GITHUB_REF=refs/heads/main
branch=main detach= check "schedule" false GITHUB_EVENT_NAME=schedule DEFAULT_BRANCH=main GITHUB_REF=refs/heads/main
branch=main detach= check "no event name" false DEFAULT_BRANCH=main
paths=$tmp/missing.txt branch=pr-docs detach= check "paths file missing" false $PR PR_BASE_SHA=$c2
printf '+ ^design/\nnonsense\n' > "$tmp/bad1.txt"
paths=$tmp/bad1.txt branch=pr-docs detach= check "unparseable rule line" false $PR PR_BASE_SHA=$c2
printf '# only a deny\n- ^x$\n' > "$tmp/bad2.txt"
paths=$tmp/bad2.txt branch=pr-docs detach= check "no + rules" false $PR PR_BASE_SHA=$c2
printf '+ ^design/(\n' > "$tmp/bad3.txt"
paths=$tmp/bad3.txt branch=pr-docs detach= check "bad + pattern" false $PR PR_BASE_SHA=$c2
printf '+ ^design/\n- ^design/(\n' > "$tmp/bad4.txt"
paths=$tmp/bad4.txt branch=pr-docs detach= check "bad - pattern" false $PR PR_BASE_SHA=$c2
printf '+ ^design/' > "$tmp/nonl.txt"
paths=$tmp/nonl.txt branch=pr-docs detach= check "rules file without final newline" true $PR PR_BASE_SHA=$c2

# --- the reference commit's own check run -----------------------------------
D="$PR PR_BASE_SHA=$c2"
branch=pr-docs detach= check "reference check run queried for R and CHECK_NAME" true $D
grep -qx "repos/o/r/commits/$c2/check-runs" "$tmp/ghargs" && grep -qx "check_name=test" "$tmp/ghargs" &&
  grep -qx "filter=latest" "$tmp/ghargs" && pass=$((pass + 1)) && echo "ok   gh asked about R's latest 'test' run" ||
  { fail=$((fail + 1)); echo "FAIL gh arguments:"; cat "$tmp/ghargs"; }
branch=pr-docs detach= check "reference run failed" false $D FAKE_RUNS=completed/failure
branch=pr-docs detach= check "reference run pending" false $D FAKE_RUNS=in_progress/null
branch=pr-docs detach= check "reference run skipped" false $D FAKE_RUNS=completed/skipped
branch=pr-docs detach= check "reference run missing" false $D FAKE_RUNS=
branch=pr-docs detach= check "check-runs API error" false $D FAKE_RC=1
branch=pr-docs detach= check "two same-name runs, one failed" false $D FAKE_RUNS=$'completed/success\ncompleted/failure'
branch=pr-docs detach= check "two same-name runs, both passed" true $D FAKE_RUNS=$'completed/success\ncompleted/success'
branch=pr-docs detach= check "CHECK_NAME unset" false $D CHECK_NAME=
branch=pr-docs detach= check "pull_request into another base" false GITHUB_EVENT_NAME=pull_request DEFAULT_BRANCH=main GITHUB_BASE_REF=release PR_BASE_SHA=$c2
git reset -q --hard "$c2"; p6=$(commit nl "design/a"$'\n'"b.md"); git push -q -f origin HEAD:refs/heads/pr-nl
branch=pr-nl detach= check "changed path contains a newline" false $D
printf '+ ^design/\r\n- ^design/protected\\.tsv$\r\n' > "$tmp/crlf.txt"
paths=$tmp/crlf.txt branch=pr-docs detach= check "CRLF rules file" true $D
paths=$tmp/crlf.txt branch=pr-prot detach= check "CRLF rules file, deny rule still applies" false $D

# --- (A') a docs-only push to a PR that already holds code -----------------
# PR_BASE_SHA's rule fails (the PR changes code), so the guard compares HEAD
# with the previous PR head (PUSH_BEFORE, github.event.before).
args_have() { # args_have <row name> <line>... : each line is one gh argument
  local n=$1 l; shift
  for l in "$@"; do
    grep -qxF -- "$l" "$tmp/ghargs" || { fail=$((fail + 1)); echo "FAIL $n: gh was not given '$l':"; cat "$tmp/ghargs"; return; }
  done
  pass=$((pass + 1)); echo "ok   $n"
}
args_lack() { # args_lack <row name> <pattern> : no gh argument contains it
  if grep -qF -- "$2" "$tmp/ghargs"; then fail=$((fail + 1)); echo "FAIL $1: gh was given '$2':"; cat "$tmp/ghargs"
  else pass=$((pass + 1)); echo "ok   $1"; fi
}
# HEAD is the PR's test-merge commit, as on GitHub: base first, PR head second.
tmerge() { # tmerge <base> <pr head> <branch>: push merge(base, head) as <branch>
  git checkout -q --detach "$1"
  git merge -q --no-ff --no-edit "$2" > /dev/null
  git push -q -f origin "HEAD:refs/heads/$3"
  tm=$(git rev-parse HEAD)
  git checkout -q --detach "$2"
}
git reset -q --hard "$c2"
a1=$(commit acode src/lib.rs); a2=$(commit adocs design/notes.md README.md); tmerge "$c2" "$a2" pr-a
A="$PR PR_BASE_SHA=$c2 FAKE_PREV=$a1"
branch=pr-a detach= check "A': code then docs, previous head passed" true $A PUSH_BEFORE=$a1
args_have "A': gh asked about the previous head's run and compared the merged base...previous head" \
  "repos/o/r/compare/$c2...$a1" "repos/o/r/commits/$a1/check-runs" "check_name=test" "filter=latest"
# github.event.pull_request.base.sha can be stale (the base the PR was opened
# on): A' must compare the base this run merged, HEAD's first parent.
branch=pr-a detach= check "A': a stale PR_BASE_SHA is not used for the compare" true $PR PR_BASE_SHA=$c0 FAKE_PREV=$a1 PUSH_BEFORE=$a1
args_have "A': the compare names HEAD's first parent, not PR_BASE_SHA" "repos/o/r/compare/$c2...$a1"
args_lack "A': the stale PR_BASE_SHA is not compared" "compare/$c0..."
branch=pr-a detach= check "A': previous head's run failed" false $A PUSH_BEFORE=$a1 FAKE_RUNS_PREV=completed/failure
branch=pr-a detach= check "A': previous head's run cancelled" false $A PUSH_BEFORE=$a1 FAKE_RUNS_PREV=completed/cancelled
branch=pr-a detach= check "A': previous head's run pending" false $A PUSH_BEFORE=$a1 FAKE_RUNS_PREV=in_progress/null
branch=pr-a detach= check "A': previous head has no run" false $A PUSH_BEFORE=$a1 FAKE_RUNS_PREV=
branch=pr-a detach= check "A': previous head, one of two same-name runs failed" false $A PUSH_BEFORE=$a1 FAKE_RUNS_PREV=$'completed/success\ncompleted/failure'
branch=pr-a detach= check "A': merged base is not an ancestor of the previous head (compare behind)" false $A PUSH_BEFORE=$a1 FAKE_COMPARE=behind
branch=pr-a detach= check "A': merged base and previous head diverged" false $A PUSH_BEFORE=$a1 FAKE_COMPARE=diverged
branch=pr-a detach= check "A': compare says identical" true $A PUSH_BEFORE=$a1 FAKE_COMPARE=identical
branch=pr-a detach= check "A': compare prints nothing" false $A PUSH_BEFORE=$a1 FAKE_COMPARE=
branch=pr-a detach= check "A': compare API error" false $A PUSH_BEFORE=$a1 FAKE_CMP_RC=1
branch=pr-a detach= check "A': no PUSH_BEFORE (opened, reopened)" false $A
branch=pr-a detach= check "A': PUSH_BEFORE all zeros" false $A PUSH_BEFORE=0000000000000000000000000000000000000000
# Fetchable branch names that are not SHAs: one fails only the hex check (40
# characters), one only the length check (all hex).
nonhex=prev-a1-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
git push -q origin "$a1:refs/heads/$nonhex" "$a1:refs/heads/abc123"
branch=pr-a detach= check "A': PUSH_BEFORE a 40-character branch name, not hex" false $A PUSH_BEFORE=$nonhex FAKE_PREV=$a1
branch=pr-a detach= check "A': PUSH_BEFORE a short all-hex branch name" false $A PUSH_BEFORE=abc123 FAKE_PREV=$a1
branch=pr-a detach= check "A': PUSH_BEFORE unfetchable" false $A PUSH_BEFORE=1234567890123456789012345678901234567890
# HEAD must be a two-parent test-merge commit; a PR head checked out
# directly (one parent) gives no merged base.
git push -q -f origin "$a2:refs/heads/pr-a-tip"
branch=pr-a-tip detach= check "A': HEAD is not a test-merge commit" false $A PUSH_BEFORE=$a1
git checkout -q --detach "$a2"
a3=$(commit acode2 src/lib.rs); tmerge "$c2" "$a3" pr-a
branch=pr-a detach= check "A': the new push changes code" false $A PUSH_BEFORE=$a2 FAKE_PREV=$a2
branch=pr-a detach= check "A': code since an older previous head" false $A PUSH_BEFORE=$a1
git checkout -q --detach "$a2"; a4=$(commit aprot design/protected.tsv); tmerge "$c2" "$a4" pr-a
branch=pr-a detach= check "A': the new push changes a protected path" false $A PUSH_BEFORE=$a2 FAKE_PREV=$a2
# The base gained code after the previous head: the test merge holds it.
git checkout -q --detach "$c2"; m1=$(commit mcode src/main.rs); git push -q origin HEAD:main
git checkout -q --detach "$a2"; a5=$(commit adocs2 design/notes.md); tmerge "$m1" "$a5" pr-a
branch=pr-a detach= check "A': test-merge HEAD holds code the base gained since the previous head" false $A PUSH_BEFORE=$a2 FAKE_PREV=$a2
# Main added code X before the previous head's run and reverted it since:
# the merged base (main after the revert) is not an ancestor of the previous
# head, so the compare (here answered as GitHub would) refuses.
git checkout -q --detach "$m1"; git revert --no-edit "$m1" > /dev/null; mx=$(git rev-parse HEAD); git push -q origin HEAD:main
tmerge "$mx" "$a5" pr-a
branch=pr-a detach= check "A': main added and reverted code since the previous head's run" false $A PR_BASE_SHA=$c2 PUSH_BEFORE=$a2 FAKE_PREV=$a2 FAKE_COMPARE=diverged
args_have "A': that compare names the merged base" "repos/o/r/compare/$mx...$a2"
git checkout -q --detach "$c2"
d1=$(commit ddocs1 design/notes.md); d2=$(commit ddocs2 README.md); tmerge "$c2" "$d2" pr-d
branch=pr-d detach= check "A': docs-only PR, base rule wins first" true $PR PR_BASE_SHA=$c2 PUSH_BEFORE=$d1 FAKE_PREV=$d1
args_lack "A': base rule wins, no compare call" "compare/"
branch=pr-d detach= check "A': docs-only PR, base's run failed, previous head passed" true $PR PR_BASE_SHA=$c2 PUSH_BEFORE=$d1 FAKE_PREV=$d1 FAKE_RUNS=completed/failure
branch=pr-a detach= check "A': not used for a push event" false $PUSH GITHUB_REF=refs/heads/pr-a PUSH_BEFORE=$a2 FAKE_PREV=$a2

echo "docs-only.test.sh: $pass passed, $fail failed"
[ "$fail" -eq 0 ]
