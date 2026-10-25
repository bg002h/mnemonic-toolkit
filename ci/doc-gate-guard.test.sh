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
#
# S10 covers rule A' (SPEC-after-b.md §3.2-§3.3): a PR push that changes no
# watched path since the previous PR head skips. GitHub's test-merge commits
# are built with `git merge --no-ff`, and a stub `gh` ($TMP/gh) answers the
# compare and check-runs calls from the FAKE_* variables. S11 covers the doc
# gates watching rust-toolchain.toml (§3.1). Both run the guard before A'
# (PREAP_REV) as their control.
set -uo pipefail

REPO="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
TMP="${DOC_GATE_TEST_TMP:-$(mktemp -d)}"
rm -rf "$TMP/origin.git" "$TMP/work" "$TMP/old"
mkdir -p "$TMP"
PREFIX_REV=6ce7e464   # the guard the first review found I-1..I-3 in
FIX1_REV=aafd9d27     # the round-1 guard the re-review found NEW-1 in
PREAP_REV=101f9c75    # the guard before rule A' and rust-toolchain.toml
WFS=(manual quickstart technical-manual manual-gui)
WFS5=(examples "${WFS[@]}")

pass=0; fail=0
ok()  { pass=$((pass + 1)); printf 'ok   %s\n' "$1"; }
bad() { fail=$((fail + 1)); printf 'FAIL %s\n' "$1"; }

git init --quiet --bare "$TMP/origin.git"
git --git-dir="$TMP/origin.git" fetch --quiet --no-tags --update-shallow "$REPO" "HEAD:refs/heads/master" || { echo "cannot seed origin"; exit 1; }
git clone --quiet --no-checkout "$TMP/origin.git" "$TMP/work"
W="$TMP/work"
g() { git -C "$W" -c user.name=t -c user.email=t@t -c commit.gpgsign=false "$@"; }
g sparse-checkout set --cone docs ci .github .examples-build scripts >/dev/null
g checkout --quiet master
MASTER=$(g rev-parse HEAD)

# guard_line WF [REV] -> the workflow's guard command (at REV if given)
guard_line() {
  local src
  if [ -n "${2:-}" ]; then src=$(git -C "$W" show "$2:.github/workflows/$1.yml" 2>/dev/null) || return 1
  else src=$(cat "$W/.github/workflows/$1.yml"); fi
  grep -oE 'bash ci/doc-gate-guard\.sh .*$' <<<"$src" | head -1
}

# relevant WF EVENT REF [BEFORE] [BASE_REF] [OLD-REV] -> prints true|false|n/a
# OLD-REV runs the guard (and its path deriver) as they were at that commit,
# with that commit's workflow arguments: the control that proves a scenario
# reproduces the defect it guards.
relevant() {
  local wf=$1 event=$2 ref=$3 before=${4:-} base_ref=${5:-} old=${6:-} cmd dir out ev
  dir="$W"
  [ "$old" = old ] && old=$PREFIX_REV
  if [ -n "$old" ]; then
    cmd=$(guard_line "$wf" "$old") || { echo n/a; return; }
    rm -rf "$TMP/old"; mkdir -p "$TMP/old/ci"
    git -C "$W" show "$old:ci/doc-gate-guard.sh" > "$TMP/old/ci/doc-gate-guard.sh" 2>/dev/null || { echo n/a; return; }
    git -C "$W" show "$old:ci/doc-gate-paths.py" > "$TMP/old/ci/doc-gate-paths.py" 2>/dev/null || true
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
  if [ "$3" = n/a ]; then printf 'skip %s (pre-fix rev not in history)\n' "$1"; return; fi
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

# ---- S4 (fix1 NEW-1): the SHARED file itself is deleted ----
# Every book that reaches docs/manual/tests/verify-examples.sh (manual owns it;
# quickstart, technical-manual and manual-gui symlink to it) must run.
branch_from_master s4
g rm --quiet docs/manual/tests/verify-examples.sh
g commit --quiet -m "s4: delete the shared verify-examples.sh"
for wf in "${WFS[@]}"; do
  expect "S4 $wf, the shared verify-examples.sh deleted" true "$(relevant "$wf" push refs/heads/ci/staging 0000000000000000000000000000000000000000)"
done
for wf in quickstart technical-manual; do
  expect "S4-control $wf with the round-1 guard (the defect)" false "$(relevant "$wf" push refs/heads/ci/staging 0000000000000000000000000000000000000000 '' "$FIX1_REV")"
done

# ---- S5 (fix1 NEW-1): a book's symlink is RETARGETED ----
branch_from_master s5
ln -sfn ../../manual/tests/lint.sh "$W/docs/technical-manual/tests/verify-examples.sh"
g add docs/technical-manual/tests/verify-examples.sh
g commit --quiet -m "s5: retarget technical-manual's verify-examples.sh symlink"
expect "S5 technical-manual, its verify-examples.sh symlink retargeted" true "$(relevant technical-manual push refs/heads/ci/staging 0000000000000000000000000000000000000000)"
# ...and a directory symlink repointed in the same commit as a change to its
# OLD target (the symlink change alone already makes it relevant; this pins
# that the combination is not mis-handled).
branch_from_master s5b
ln -sfn ../manual/tests "$W/docs/quickstart/transcripts"
echo '# touched' >> "$W/docs/manual/transcripts/22-first-bundle.cmd"
g add docs/quickstart/transcripts docs/manual/transcripts/22-first-bundle.cmd
g commit --quiet -m "s5b: retarget quickstart transcripts + touch the old target"
expect "S5b quickstart, transcripts symlink retargeted" true "$(relevant quickstart push refs/heads/ci/staging 0000000000000000000000000000000000000000)"

# ---- S6 (fix1 NEW-1): the symlink itself is deleted ----
for b in quickstart technical-manual manual-gui; do
  branch_from_master "s6-$b"
  g rm --quiet "docs/$b/tests/verify-examples.sh"
  g commit --quiet -m "s6: delete docs/$b/tests/verify-examples.sh (the symlink)"
  expect "S6 $b, its verify-examples.sh symlink deleted" true "$(relevant "$b" push refs/heads/ci/staging 0000000000000000000000000000000000000000)"
done

# ---- S7: derivation reads the BASE tree too (a dependency only the base shows) ----
branch_from_master s7
g rm --quiet docs/manual/.cspell.json
g commit --quiet -m "s7: delete a config quickstart imports by relative reference"
expect "S7 quickstart, its imported ../manual/.cspell.json deleted" true "$(relevant quickstart push refs/heads/ci/staging 0000000000000000000000000000000000000000)"
expect "S7-control quickstart with the round-1 guard (the defect)" false "$(relevant quickstart push refs/heads/ci/staging 0000000000000000000000000000000000000000 '' "$FIX1_REV")"

# ---- S8 (fix1 self-check): a repo-rooted tool only the Makefile names ----
# manual-gui's `make lint` runs $(TOOLKIT_ROOT)/docs/tools/render-mermaid-cache.py
# (figures-cache-verify); neither its old paths: filter nor its round-1 --also
# watched it.
branch_from_master s8
mkdir -p "$W/docs/tools"
echo '# touched' >> "$W/docs/tools/render-mermaid-cache.py"
g add --sparse docs/tools/render-mermaid-cache.py
g commit --quiet -m "s8: touch only the mermaid cache tool"
for wf in "${WFS[@]}"; do
  expect "S8 $wf, only docs/tools/render-mermaid-cache.py changed" true "$(relevant "$wf" push refs/heads/ci/staging 0000000000000000000000000000000000000000)"
done
expect "S8-control manual-gui with the round-1 guard (the defect)" false "$(relevant manual-gui push refs/heads/ci/staging 0000000000000000000000000000000000000000 '' "$FIX1_REV")"

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

# ---- S9 (F-677): examples.yml runs on EVERY push; the guard carries its scope ----
# The hole: examples.yml filtered `push:` by paths, so a docs-only commit on
# ci/staging never started it and the REQUIRED `examples` context never
# appeared. Structural half: no `paths:` on any trigger, and every heavy step
# is gated on the guard alone.
if python3 - "$W/.github/workflows/examples.yml" <<'PYEOF'
import sys, yaml
w = yaml.safe_load(open(sys.argv[1]))
on = w.get("on", w.get(True))
bad = [ev for ev, cfg in on.items() if isinstance(cfg, dict) and ("paths" in cfg or "paths-ignore" in cfg)]
steps = w["jobs"]["examples"]["steps"]
gi = [i for i, st in enumerate(steps) if st.get("id") == "guard"]
assert gi and "if" not in steps[gi[0]], "guard step missing or conditional"
heavy = [st.get("name") for st in steps[gi[0] + 1:]
         if not str(st.get("if", "")).startswith("startsWith(github.ref, 'refs/tags/examples-v')")
         and st.get("if") != "steps.guard.outputs.relevant == 'true'"]
if bad or heavy:
    print("paths filters on:", bad, "; ungated steps:", heavy); sys.exit(1)
PYEOF
then ok "S9 examples.yml: no paths filter on any trigger; heavy steps gated only on the guard"
else bad "S9 examples.yml trigger/step shape"; fi
if git -C "$W" show 15443535:.github/workflows/examples.yml 2>/dev/null | grep -qE "^    paths:"; then
  ok "S9-control the pre-fix examples.yml (15443535) filtered push by paths (the defect)"
else printf 'skip S9-control (pre-fix rev not in history)\n'; fi

branch_from_master s9-docs
echo '# touched' >> "$W/docs/manual/src/40-cli-reference/42-md.md"
g commit --quiet -am "s9: docs-only"
expect "S9 examples, ci/staging push, docs-only (context reports, heavy steps skip)" false "$(relevant examples push refs/heads/ci/staging 0000000000000000000000000000000000000000)"
expect "S9 examples, PR, docs-only" false "$(relevant examples pull_request refs/pull/9/merge '' master)"
# Every path the old push filter or the old PR regex gated must still gate.
for f in .examples-build/Examples.md docs/Examples.pdf scripts/install.sh crates/x.txt \
         Cargo.lock Cargo.toml rust-toolchain.toml .gitattributes .github/workflows/examples.yml ci/doc-gate-guard.sh ci/doc-gate-paths.py; do
  branch_from_master "s9-$(tr '/.' '__' <<<"$f")"
  mkdir -p "$W/$(dirname "$f")"
  echo '# touched' >> "$W/$f"
  g add --sparse -f "$f"
  g commit --quiet -m "s9: touch $f"
  expect "S9 examples, ci/staging push, only $f" true "$(relevant examples push refs/heads/ci/staging 0000000000000000000000000000000000000000)"
  expect "S9 examples, master push (before=parent), only $f" true "$(relevant examples push refs/heads/master "$(g rev-parse HEAD~1)")"
done
expect "S9 examples, examples-v* tag push" true "$(relevant examples push refs/tags/examples-v9.9.9)"
expect "S9 examples, workflow_dispatch" true "$(relevant examples workflow_dispatch refs/heads/master)"

# ---- S10 (SPEC-after-b.md §3.3): rule A', the previous PR head ----
# Stub gh: logs its arguments, one per line, to $TMP/ghargs; prints $FAKE_RUNS
# when an argument contains /check-runs, else $FAKE_COMPARE; exits ${FAKE_RC:-0}.
{
  printf '#!/usr/bin/env bash\n'
  printf 'log=%q\n' "$TMP/ghargs"
  cat <<'EOF'
printf '%s\n' "$@" >> "$log"
reply=${FAKE_COMPARE:-}
for a in "$@"; do
  case "$a" in */check-runs*) reply=${FAKE_RUNS:-} ;; esac
done
[ -z "$reply" ] || printf '%s\n' "$reply"
exit "${FAKE_RC:-0}"
EOF
} > "$TMP/gh"
chmod +x "$TMP/gh"
: > "$TMP/ghargs"

# check_name WF -> the CHECK_NAME: in the env: of WF's guard step (the one
# step whose run: is `bash ci/doc-gate-guard.sh …`); nothing unless exactly one.
check_name() {
  python3 - "$W/.github/workflows/$1.yml" <<'PYEOF'
import sys, yaml
w = yaml.safe_load(open(sys.argv[1]))
steps = [st for job in w["jobs"].values() for st in job.get("steps", [])
         if str(st.get("run", "")).startswith("bash ci/doc-gate-guard.sh ")]
if len(steps) == 1:
    print((steps[0].get("env") or {}).get("CHECK_NAME", ""))
PYEOF
}

# relevant_ap WF EVENT BEFORE CHECK [OLD] -> prints true|false|n/a
# Runs the guard as `relevant` does (the workflow's own guard line), on the
# current HEAD, for refs/pull/10/merge (refs/heads/ci/staging for a push),
# with PUSH_BEFORE=BEFORE, CHECK_NAME=CHECK, GITHUB_REPOSITORY=o/r and the
# stub gh, and GITHUB_BASE_REF=AP_BASE_REF (default master; never the
# runner's own GITHUB_BASE_REF, which is the real PR's base). FAKE_COMPARE
# (ahead), FAKE_RUNS (completed/success) and FAKE_RC (0) come from the
# caller's environment. OLD (any non-empty value) runs the guard, its path deriver and
# the workflow's arguments as they were at PREAP_REV.
relevant_ap() {
  local wf=$1 event=$2 before=$3 check=$4 old=${5:-} cmd ref out ev
  if [ -n "$old" ]; then
    cmd=$(guard_line "$wf" "$PREAP_REV") || { echo n/a; return; }
    rm -rf "$TMP/old"; mkdir -p "$TMP/old/ci"
    git -C "$W" show "$PREAP_REV:ci/doc-gate-guard.sh" > "$TMP/old/ci/doc-gate-guard.sh" 2>/dev/null || { echo n/a; return; }
    git -C "$W" show "$PREAP_REV:ci/doc-gate-paths.py" > "$TMP/old/ci/doc-gate-paths.py" 2>/dev/null || true
    cmd=${cmd/bash ci\/doc-gate-guard.sh/bash $TMP/old/ci/doc-gate-guard.sh}
  else
    cmd=$(guard_line "$wf")
  fi
  ref=refs/pull/10/merge
  [ "$event" = push ] && ref=refs/heads/ci/staging
  out="$TMP/out.$$"; : > "$out"; ev="$TMP/ev.json"
  printf '{"before":"%s"}' "$before" > "$ev"
  ( cd "$W" && env GITHUB_EVENT_NAME="$event" GITHUB_REF="$ref" GITHUB_BASE_REF="${AP_BASE_REF:-master}" \
      GITHUB_EVENT_PATH="$ev" GITHUB_OUTPUT="$out" PUSH_BEFORE="$before" CHECK_NAME="$check" \
      GITHUB_REPOSITORY=o/r DOC_GATE_GH="$TMP/gh" FAKE_COMPARE="${FAKE_COMPARE-ahead}" \
      FAKE_RUNS="${FAKE_RUNS-completed/success}" FAKE_RC="${FAKE_RC-0}" bash -c "$cmd" >/dev/null 2>&1 )
  grep -oE '^relevant=(true|false)$' "$out" | tail -1 | cut -d= -f2 || true
  rm -f "$out"
}

# test_merge NAME BASE HEAD -> branch NAME = GitHub's test-merge commit of HEAD
# into BASE: first parent BASE, second parent HEAD.
test_merge() {
  g checkout --quiet -B "$1" "$2"
  g merge --quiet --no-ff --no-edit "$3" >/dev/null
}

for wf in "${WFS5[@]}"; do
  ck=$(check_name "$wf")
  if [ "$ck" = "$wf" ]; then ok "S10a $wf: its guard step's CHECK_NAME is $wf"
  else bad "S10a $wf: its guard step's CHECK_NAME is '${ck}', not $wf"; fi
done

# P1: the PR's first head, watched or --also in all five gates.
g checkout --quiet -B ap-p1 "$MASTER"
mkdir -p "$W/crates"
echo '# touched' >> "$W/crates/x.txt"
echo '# touched' >> "$W/scripts/install.sh"
g add --sparse -f crates/x.txt scripts/install.sh
g commit --quiet -m "ap P1: crates/x.txt + scripts/install.sh"
P1=$(g rev-parse HEAD)
# P2: watched by none. P2w: watched in all five. P2t: rust-toolchain.toml only.
g checkout --quiet -B ap-p2 "$P1"
mkdir -p "$W/design"
echo 'a note' > "$W/design/a-prime-note.md"
g add --sparse -f design/a-prime-note.md
g commit --quiet -m "ap P2: design/a-prime-note.md"
P2=$(g rev-parse HEAD)
g checkout --quiet -B ap-p2w "$P1"
echo '# touched' >> "$W/docs/manual/tests/verify-examples.sh"
echo '# touched again' >> "$W/scripts/install.sh"
g add --sparse -f docs/manual/tests/verify-examples.sh scripts/install.sh
g commit --quiet -m "ap P2w: verify-examples.sh + scripts/install.sh"
P2w=$(g rev-parse HEAD)
g checkout --quiet -B ap-p2t "$P1"
echo '# touched' >> "$W/rust-toolchain.toml"
g add --sparse -f rust-toolchain.toml
g commit --quiet -m "ap P2t: rust-toolchain.toml"
P2t=$(g rev-parse HEAD)
# MASTER2: the base gains a watched change.
g checkout --quiet -B ap-master2 "$MASTER"
echo '# touched' >> "$W/docs/manual/src/40-cli-reference/42-md.md"
g add --sparse -f docs/manual/src/40-cli-reference/42-md.md
g commit --quiet -m "ap MASTER2: the base gains a docs/manual change"
MASTER2=$(g rev-parse HEAD)
test_merge M "$MASTER" "$P2"
test_merge Mw "$MASTER" "$P2w"
test_merge Mt "$MASTER" "$P2t"
test_merge M2 "$MASTER2" "$P2"
test_merge M0 "$MASTER" "$P1"

ckm=$(check_name manual)
zero40=0000000000000000000000000000000000000000
g checkout --quiet M
for wf in "${WFS5[@]}"; do
  ck=$(check_name "$wf")
  expect "S10 AP1 $wf, PR push, nothing watched changed since the previous head P1" false "$(relevant_ap "$wf" pull_request "$P1" "$ck")"
  expect "S10 AP1c $wf, as AP1 with the guard before A' (always ran)" true "$(relevant_ap "$wf" pull_request "$P1" "$ck" old)"
done
expect "S10 AP2 manual, '$ckm' failed on P1" true "$(FAKE_RUNS=completed/failure relevant_ap manual pull_request "$P1" "$ckm")"
expect "S10 AP3 manual, no '$ckm' check run on P1" true "$(FAKE_RUNS='' relevant_ap manual pull_request "$P1" "$ckm")"
expect "S10 AP4 manual, one of two latest runs still in progress" true "$(FAKE_RUNS=$'completed/success\nin_progress/null' relevant_ap manual pull_request "$P1" "$ckm")"
expect "S10 AP5 manual, merged base...P1 diverged" true "$(FAKE_COMPARE=diverged relevant_ap manual pull_request "$P1" "$ckm")"
expect "S10 AP6 manual, merged base...P1 behind" true "$(FAKE_COMPARE=behind relevant_ap manual pull_request "$P1" "$ckm")"
expect "S10 AP7 manual, gh fails" true "$(FAKE_RC=1 relevant_ap manual pull_request "$P1" "$ckm")"
expect "S10 AP8 manual, no previous head" true "$(relevant_ap manual pull_request '' "$ckm")"
expect "S10 AP9 manual, previous head is P1 in upper case" true "$(relevant_ap manual pull_request "$(tr a-f A-F <<<"$P1")" "$ckm")"
expect "S10 AP10 manual, previous head is 40 zeros" true "$(relevant_ap manual pull_request "$zero40" "$ckm")"
expect "S10 AP11 manual, CHECK_NAME empty" true "$(relevant_ap manual pull_request "$P1" '')"
g checkout --quiet Mw
for wf in "${WFS5[@]}"; do
  expect "S10 AP12 $wf, a watched path changed since P1" true "$(relevant_ap "$wf" pull_request "$P1" "$(check_name "$wf")")"
done
g checkout --quiet Mt
for wf in "${WFS5[@]}"; do
  expect "S10 AP13 $wf, rust-toolchain.toml changed since P1" true "$(relevant_ap "$wf" pull_request "$P1" "$(check_name "$wf")")"
done
g checkout --quiet ap-p2
expect "S10 AP14 manual, HEAD P2 has one parent" true "$(relevant_ap manual pull_request "$P1" "$ckm")"
g checkout --quiet M
expect "S10 AP15 manual, push to ci/staging (A' is PR-only)" true "$(relevant_ap manual push "$P1" "$ckm")"
g checkout --quiet M2
expect "S10 AP16 manual, the merged base gained a watched change (stub still says ahead)" true "$(relevant_ap manual pull_request "$P1" "$ckm")"
g checkout --quiet M0
expect "S10 AP17 manual, no file differs from P1" true "$(relevant_ap manual pull_request "$P1" "$ckm")"
g checkout --quiet M
: > "$TMP/ghargs"
relevant_ap manual pull_request "$P1" "$ckm" >/dev/null
missing=''
for l in "repos/o/r/commits/$P1/check-runs" check_name=manual filter=latest "repos/o/r/compare/$MASTER...$P1"; do
  grep -qxF -- "$l" "$TMP/ghargs" || missing="$missing [$l]"
done
if [ -z "$missing" ]; then ok "S10 AP18 manual, gh asked for P1's '$ckm' check runs and compared MASTER...P1"
else bad "S10 AP18 manual, gh arguments missing:$missing"; fi
git --git-dir="$TMP/origin.git" update-ref refs/heads/side "$MASTER"
expect "S10 AP19 manual, PR into side, not master" true "$(AP_BASE_REF=side relevant_ap manual pull_request "$P1" "$ckm")"

# ---- S11 (SPEC-after-b.md §3.1): only rust-toolchain.toml changes ----
branch_from_master s11
echo '# touched' >> "$W/rust-toolchain.toml"
g add --sparse -f rust-toolchain.toml
g commit --quiet -m "s11: rust-toolchain.toml only"
for wf in "${WFS5[@]}"; do
  expect "S11 $wf, PR, only rust-toolchain.toml" true "$(relevant "$wf" pull_request refs/pull/11/merge '' master)"
  expect "S11 $wf, ci/staging push, only rust-toolchain.toml" true "$(relevant "$wf" push refs/heads/ci/staging "$zero40")"
  expect "S11-control $wf, PR, with the guard before §3.1 (the defect)" false "$(relevant "$wf" pull_request refs/pull/11/merge '' master "$PREAP_REV")"
done

# ---- S12 (ab-sec-quotepath): a changed path git would C-quote ----
# Without -z, git diff prints "crates/\303\251.rs" (quotes included), which
# matches no watched path or --also: a silently skipped gate. manual-gui does
# not watch crates/, so it is not in these rows.
WFS_CRATES=(examples manual quickstart technical-manual)
branch_from_master s12
mkdir -p "$W/crates"
echo '# touched' > "$W/crates/é.rs"
g add --sparse -f "crates/é.rs"
g commit --quiet -m "s12: crates/é.rs only"
for wf in "${WFS_CRATES[@]}"; do
  expect "S12 $wf, PR, only crates/é.rs" true "$(relevant "$wf" pull_request refs/pull/12/merge '' master)"
  expect "S12-control $wf, PR, with the guard before -z (the defect)" false "$(relevant "$wf" pull_request refs/pull/12/merge '' master "$PREAP_REV")"
done
# A': since the previous head P1, only crates/é2.rs changed.
g checkout --quiet -B ap-p2q "$P1"
mkdir -p "$W/crates"
echo '# touched' > "$W/crates/é2.rs"
g add --sparse -f "crates/é2.rs"
g commit --quiet -m "ap P2q: crates/é2.rs only"
P2q=$(g rev-parse HEAD)
[ "$P2q" != "$P1" ] || bad "S12 AP setup: the crates/é2.rs commit was not made"
test_merge Mq "$MASTER" "$P2q"
for wf in "${WFS_CRATES[@]}"; do
  expect "S12 AP $wf, only crates/é2.rs changed since P1" true "$(relevant_ap "$wf" pull_request "$P1" "$(check_name "$wf")")"
done

printf '\n%d passed, %d failed\n' "$pass" "$fail"
[ "$fail" -eq 0 ]
