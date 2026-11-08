#!/usr/bin/env bash
# design/spec-vendor-freshness/gate.sh -- build gate for
# design/SPEC_vendor_freshness_shared.md: runs the reference prototype
# (proto/) in the five repos at the SHAs the spec records, and checks every
# expected-output block the spec prints against what actually ran.
#
# usage: bash design/spec-vendor-freshness/gate.sh <checkouts> <logs>
#   <checkouts>/<repo> is a clean checkout (a worktree is fine) of each of
#   mnemonic-toolkit, mnemonic-refugium, mnemonic-key, mnemonic-secret and
#   descriptor-mnemonic at the SHA below. The gate installs the prototype into
#   each, runs it, and restores each checkout before it exits.
# Needs bash, python3, git, rsync, cargo (rustup; the repos pin 1.85.0) and
# network access to github.com. Exit 0 = every check passed.
#
# Checks, each printed as PASS/FAIL <id>:
#   C1  each checkout is at its recorded SHA and clean
#   C2  each spec data-file block equals proto/data/<repo>.txt, and its rows
#       name exactly the git urls that repo's Cargo.lock pins
#   C3  the gate's whole output in each repo equals the spec's green block
#   C4  the mutation test's row lines in each repo equal the spec's test block
#   C5  every refusal block equals the gate's own lines (`::error::` and
#       `  - `) under that row in the verbose test run named in its label
#   C6  RED: with each repo's CURRENT script and the new test + data file, the
#       test exits non-zero, and the rows where that script gave the wrong exit
#       (<row>:<exit>) include the spec's red block
#   C7  mutants of the prototype in mnemonic-toolkit: each mutant's FAIL rows
#       include the spec's mutant block
#   C8  with each public repo's docs-only rule for docs/verify-reproducibility.md
#       removed as the spec says, ci/code-docs-check.sh exits 0 (and in mr,
#       which keeps its file unchanged)
#   C9  the gate is shown failing: a spec green block with one byte changed
#       must FAIL C3's comparison
#   C10 the spec's proto-sha256 block equals `sha256sum vendor-freshness.sh
#       vendor-freshness.test.sh` run in proto/
#
# NOT covered: the workflow (.github/) edits, the ci/shared-ci.sha256,
# ci/shared-vendor.sha256 and ci/shared-drift.sh edits (the pinning is OPEN in
# the spec), the catalog and FOLLOWUPS entries, a real network outage (G5 stands in), mr's self-hosted runner
# having rsync, and whether green CI on the per-repo PRs matches this machine.
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SPEC="$HERE/../SPEC_vendor_freshness_shared.md"
PROTO="$HERE/proto"
CO="$(cd "${1:?usage: gate.sh <checkouts> <logs>}" && pwd)"
LOG="${2:?usage: gate.sh <checkouts> <logs>}"
mkdir -p "$LOG"; LOG="$(cd "$LOG" && pwd)"
export PATH="$HOME/.cargo/bin:$PATH"
export VF_TEST_TMPDIR="$LOG/tmp"; mkdir -p "$VF_TEST_TMPDIR"

declare -A SHA=(
  [mnemonic-toolkit]=3753b5c7bf90bcdb5354dbd18a7612cde087406a
  [mnemonic-refugium]=08b6ccb7e4c1ad8bc5f96065593ab77b66eda91a
  [mnemonic-key]=613f8e3482e6f2c6a0ba2e639ad40142d62492be
  [mnemonic-secret]=7a62298732e5bbf0b7d35d223659eaae5318537a
  [descriptor-mnemonic]=0d96d2c89272be2b50df644987205b292c3afb33
)
REPOS=(mnemonic-toolkit mnemonic-refugium mnemonic-key mnemonic-secret descriptor-mnemonic)
PUBLIC=(mnemonic-toolkit mnemonic-key mnemonic-secret descriptor-mnemonic)
k=0
pass() { echo "PASS $1"; }
failc() { echo "FAIL $1"; k=$((k + 1)); }

block() { # block <label>: print the spec's fenced block after <!-- vf:<label> -->
  awk -v want="<!-- vf:$1 -->" '
    $0 == want { armed = 1; next }
    armed == 1 && /^```/ { armed = 2; next }
    armed == 2 && /^```/ { exit }
    armed == 2 { print }' "$SPEC"
}
has_block() { grep -qxF "<!-- vf:$1 -->" "$SPEC"; }

restore() { # restore <repo>: the checkout back to its SHA, tracked and new files
  local d="$CO/$1"
  git -C "$d" reset -q
  git -C "$d" checkout -q -- .
  rm -f "$d/ci/repro/vendor-git-sources.txt"
  if ! git -C "$d" ls-files --error-unmatch ci/repro/vendor-freshness.test.sh >/dev/null 2>&1; then
    rm -f "$d/ci/repro/vendor-freshness.test.sh"
  fi
}
install_proto() { # install_proto <repo> [gate script]: prototype + data file in
  local d="$CO/$1"
  cp "${2:-$PROTO/vendor-freshness.sh}" "$d/ci/repro/vendor-freshness.sh"
  cp "$PROTO/vendor-freshness.test.sh" "$d/ci/repro/vendor-freshness.test.sh"
  cp "$PROTO/data/$1.txt" "$d/ci/repro/vendor-git-sources.txt"
}
trap 'for r in "${REPOS[@]}"; do restore "$r" 2>/dev/null || true; done' EXIT

# ── C1 ──────────────────────────────────────────────────────────────────────
for r in "${REPOS[@]}"; do
  h="$(git -C "$CO/$r" rev-parse HEAD)"
  if [ "$h" = "${SHA[$r]}" ] && [ -z "$(git -C "$CO/$r" status --porcelain)" ]; then pass "C1 $r at ${h:0:8}, clean"
  else failc "C1 $r at $h (want ${SHA[$r]}) or dirty"; fi
done
[ "$k" -eq 0 ] || { echo "gate: FAIL ($k)"; exit 1; }

# ── C2 ──────────────────────────────────────────────────────────────────────
for r in "${REPOS[@]}"; do
  if ! has_block "data-$r"; then failc "C2 $r: no spec block vf:data-$r"; continue; fi
  if ! diff <(block "data-$r") "$PROTO/data/$r.txt" > "$LOG/c2-$r.diff"; then
    failc "C2 $r: spec block differs from proto/data/$r.txt ($LOG/c2-$r.diff)"; continue; fi
  lock_urls="$(sed -nE 's/^source = "git\+([^?"#]+)\?.*/\1/p' "$CO/$r/Cargo.lock" | sort -u | tr '\n' ' ')"
  row_urls="$( { grep -vE '^(#|$)' "$PROTO/data/$r.txt" || true; } | cut -d'|' -f2 | sort -u | tr '\n' ' ')"
  if [ "$lock_urls" = "$row_urls" ]; then pass "C2 $r: data rows = Cargo.lock git urls (${row_urls:-none})"
  else failc "C2 $r: lock urls [$lock_urls] != rows [$row_urls]"; fi
done

# ── C3, C4, C5 ──────────────────────────────────────────────────────────────
for r in "${REPOS[@]}"; do install_proto "$r"; done
for r in "${REPOS[@]}"; do
  ( cd "$CO/$r" && rc=0; bash ci/repro/vendor-freshness.sh > "$LOG/green-$r.raw" 2>&1 || rc=$?; echo "exit $rc" >> "$LOG/green-$r.raw" ) &
done; wait
for r in "${REPOS[@]}"; do
  grep -vE '^ +Blocking waiting for file lock' "$LOG/green-$r.raw" > "$LOG/green-$r.txt" || true
  if diff <(block "green-$r") "$LOG/green-$r.txt" > "$LOG/c3-$r.diff"; then pass "C3 $r green output = spec"
  else failc "C3 $r green output differs ($LOG/c3-$r.diff)"; fi
done
for r in "${REPOS[@]}"; do
  ( cd "$CO/$r" && rc=0; VF_TEST_VERBOSE=1 bash ci/repro/vendor-freshness.test.sh > "$LOG/test-$r.raw" 2>&1 || rc=$?; echo "exit $rc" >> "$LOG/test-$r.raw" ) &
done; wait
for r in "${REPOS[@]}"; do
  grep -E '^(subjects|ok   - |skip - |FAIL - |vendor-freshness.test:|exit )' "$LOG/test-$r.raw" > "$LOG/test-$r.txt" || true
  if diff <(block "test-$r") "$LOG/test-$r.txt" > "$LOG/c4-$r.diff"; then pass "C4 $r test rows = spec"
  else failc "C4 $r test rows differ ($LOG/c4-$r.diff)"; fi
done
# Refusal blocks: label refusal-<repo>-<row id>.
mapfile -t RLABELS < <(sed -nE 's/^<!-- vf:(refusal-[a-z-]+-[AG][0-9]+) -->$/\1/p' "$SPEC")
[ "${#RLABELS[@]}" -gt 0 ] || failc "C5 no refusal blocks in the spec"
for lab in "${RLABELS[@]}"; do
  id="${lab##*-}"; rr="${lab#refusal-}"; rr="${rr%-*}"
  awk -v id="$id" '
    /^(ok   - |FAIL - |skip - )/ { on = (index($0, "- " id " ") == 6) ; next }
    /^vendor-freshness.test:/ { on = 0 }
    on && /^       \| / { print substr($0, 10) }' "$LOG/test-$rr.raw" \
    | grep -E '^(::error::|  - |      )' > "$LOG/c5-$lab.txt" || true
  if diff <(block "$lab") "$LOG/c5-$lab.txt" > "$LOG/c5-$lab.diff" && [ -s "$LOG/c5-$lab.txt" ]; then pass "C5 $lab"
  else failc "C5 $lab differs or empty ($LOG/c5-$lab.diff)"; fi
done

# ── C8 (needs the prototype installed) ──────────────────────────────────────
for r in "${PUBLIC[@]}"; do
  f="$CO/$r/ci/docs-only-paths.txt"
  python3 -I - "$f" <<'PY'
import sys
p = sys.argv[1]
lines = open(p).read().split("\n")
i = lines.index("- ^docs/verify-reproducibility\\.md$")
assert lines[i - 1] == "# Named in ci/repro/vendor-freshness.sh (ci/code-docs-check.sh).", lines[i - 1]
del lines[i - 1:i + 1]
open(p, "w").write("\n".join(lines))
PY
done
for r in "${REPOS[@]}"; do
  git -C "$CO/$r" add -N ci/repro/vendor-freshness.test.sh ci/repro/vendor-git-sources.txt
  if ( cd "$CO/$r" && bash ci/code-docs-check.sh > "$LOG/c8-$r.txt" 2>&1 ); then pass "C8 $r code-docs-check exit 0"
  else failc "C8 $r code-docs-check failed ($LOG/c8-$r.txt)"; fi
done

# ── C6 RED: the current scripts ─────────────────────────────────────────────
for r in "${REPOS[@]}"; do
  restore "$r"; install_proto "$r" <(git -C "$CO/$r" show "${SHA[$r]}:ci/repro/vendor-freshness.sh")
done
for r in "${REPOS[@]}"; do
  ( cd "$CO/$r" && rc=0; bash ci/repro/vendor-freshness.test.sh > "$LOG/red-$r.raw" 2>&1 || rc=$?; echo "exit $rc" >> "$LOG/red-$r.raw" ) &
done; wait
for r in "${REPOS[@]}"; do
  # <id>:<exit> for every row whose exit was wrong (the defect passed, or a green tree failed)
  got="$(sed -nE 's/^FAIL - ([AG][0-9]+) .*\(exit ([01]); wanted ([01]);.*/\1 \2 \3/p' "$LOG/red-$r.raw" | awk '$2 != $3 { printf "%s:%s ", $1, $2 }')"
  want="$(block "red-$r")"
  miss=""; for id in $want; do grep -qw -- "$id" <<<"$got" || miss+="$id "; done
  if [ -n "$want" ] && [ -z "$miss" ] && grep -qx 'exit 1' "$LOG/red-$r.raw"; then pass "C6 $r current script REDs; wrong-exit rows $got"
  else failc "C6 $r: want FAIL rows [$want], got [$got], missing [$miss]"; fi
done
for r in "${REPOS[@]}"; do restore "$r"; done

# ── C7 mutants (mnemonic-toolkit) ───────────────────────────────────────────
mk_mutant() { # mk_mutant <id> <out>: the prototype with one defect
  python3 -I - "$PROTO/vendor-freshness.sh" "$1" "$2" <<'PY'
import sys
src, mid, out = sys.argv[1], sys.argv[2], sys.argv[3]
s = open(src).read()
edits = {
  # M1: resolve under the caller's CARGO_HOME (no hermetic home).
  "M1": ('env = dict(os.environ, CARGO_HOME=os.path.abspath(home))', 'env = dict(os.environ)'),
  # M2: no check (4) at all: green after (3).
  "M2": ('    if not rows:\n        print(', '    if True:\n        print('),
  # M3: check (4) without the file-set comparison.
  "M3": ('        if on_disk != listed:', '        if False:'),
  # M4: check (4) compares the version only, never the bytes.
  "M4": ('            elif open(upstream, "rb").read() != open(os.path.join(vend, rel), "rb").read():', '            elif False:'),
  # M5: check (4) without the Cargo.toml version comparison.
  "M5": ('        if version_of(vt) != version_of(ut) or version_of(ut) == "none":', '        if False:'),
}
a, b = edits[mid]
assert s.count(a) == 1, (mid, s.count(a))
open(out, "w").write(s.replace(a, b))
PY
}
for m in M1 M2 M3 M4 M5; do
  restore mnemonic-toolkit
  mk_mutant "$m" "$LOG/mutant-$m.sh"
  install_proto mnemonic-toolkit "$LOG/mutant-$m.sh"
  ( cd "$CO/mnemonic-toolkit" && rc=0; bash ci/repro/vendor-freshness.test.sh > "$LOG/mut-$m.raw" 2>&1 || rc=$?; echo "exit $rc" >> "$LOG/mut-$m.raw" ) || true
  got="$(sed -nE 's/^FAIL - ([AG][0-9]+) .*/\1/p' "$LOG/mut-$m.raw" | tr '\n' ' ')"
  want="$(block "mutant-$m")"
  miss=""; for id in $want; do grep -qw "$id" <<<"$got" || miss+="$id "; done
  if [ -n "$want" ] && [ -z "$miss" ]; then pass "C7 $m caught: FAIL rows $got"
  else failc "C7 $m: want FAIL rows [$want], got [$got], missing [$miss]"; fi
done
restore mnemonic-toolkit

# ── C10 the recorded hashes ─────────────────────────────────────────────────
if diff <(block proto-sha256) <(cd "$PROTO" && sha256sum vendor-freshness.sh vendor-freshness.test.sh) > "$LOG/c10.diff"; then
  pass "C10 proto sha256 = spec"
else failc "C10 proto sha256 differs ($LOG/c10.diff)"; fi

# ── C9 the comparison can fail ──────────────────────────────────────────────
if diff <(block green-mnemonic-key | sed '1s/^vendor-freshness/vendor_freshness/') "$LOG/green-mnemonic-key.txt" >/dev/null; then
  failc "C9 a changed spec block still compared equal"
else pass "C9 a one-byte change to a spec block FAILs the C3 comparison"; fi

if [ "$k" -eq 0 ]; then echo "gate: PASS"; exit 0; fi
echo "gate: FAIL ($k)"; exit 1
