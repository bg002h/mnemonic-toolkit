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

# A stand-in for `gh`: records its arguments, prints $FAKE_RUNS (one
# "status/conclusion" line per check run) and exits $FAKE_RC.
printf '%s\n' '#!/usr/bin/env bash' 'printf "%s\n" "$@" > "$GH_ARGS"' \
  '[ "${FAKE_RC:-0}" -eq 0 ] || exit "$FAKE_RC"' 'printf "%s\n" "$FAKE_RUNS"' > "$tmp/gh"
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
  : > "$tmp/out"
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

echo "docs-only.test.sh: $pass passed, $fail failed"
[ "$fail" -eq 0 ]
