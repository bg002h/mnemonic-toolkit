#!/usr/bin/env bash
# Mutation tests for ci/repro/vendor-freshness.sh. SHARED FILE: byte-identical
# in the five repos that carry the gate, pinned by its sha256 in each.
#
# Each row copies the tracked tree (plus the working copies of the gate and the
# data file) into a scratch dir, applies ONE mutation, runs the gate there, and
# asserts the exit status AND a pattern naming the check that fired: a gate
# that reds for the wrong reason would otherwise read as a pass. The control
# row must be green, or every red is vacuous.
#
# Subjects, chosen from the tree so the file runs unchanged in every repo:
#   R = the first vendor dir, in sorted order, whose .cargo-checksum.json has a
#       non-null "package" and whose crate name occurs once in Cargo.lock;
#   G = the first row of the data file, in file order. With no row, the G rows
#       print "skip" and do not count as passed.
# Each subject's edited file is the first .rs entry, in sorted order, of its
# .cargo-checksum.json "files".
#
# Usage: bash ci/repro/vendor-freshness.test.sh. VF_TEST_VERBOSE=1 also prints
# the gate's output under each passing row. Needs bash, tar, python3,
# git, cargo and network access to each row's host (G rows run check 4).
# Exit 0 = every row that ran passed. NOT covered: check (3)'s PROVENANCE
# MISMATCH and unanchored-set lines (cargo or the data-file refusals red first
# on every mutation that reaches them); the refusals for a missing cargo or
# git, an absent or unreadable vendor manifest, a fetched HEAD that is not the
# rev, a package-name mismatch, a vendored dir outside check (1)'s resolution,
# cargo failing on upstream's manifest, the caller's GIT_* variables being
# dropped, and the "... and N more" cap; and a real network outage (G5 stands
# in: an unfetchable rev takes the same path).
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRATCH_PARENT="${VF_TEST_TMPDIR:-${TMPDIR:-/tmp}}"
WORK="$(mktemp -d -p "$SCRATCH_PARENT" vf-test.XXXXXX)"
trap 'rm -rf "$WORK"' EXIT
DATA=ci/repro/vendor-git-sources.txt

PRISTINE="$WORK/pristine"
mkdir -p "$PRISTINE"
( cd "$REPO_ROOT" && git ls-files -z ) \
  | ( cd "$REPO_ROOT" && tar --null -T - -cf - ) | tar -xf - -C "$PRISTINE"
for f in ci/repro/vendor-freshness.sh "$DATA"; do
  if [ -f "$REPO_ROOT/$f" ]; then cp "$REPO_ROOT/$f" "$PRISTINE/$f"; fi
done

# R and G, and each one's edited file.
read -r R R_NAME R_RS < <(cd "$PRISTINE" && python3 -I - <<'PY'
import glob, json, os, re
lock = open("Cargo.lock").read()
for cj in sorted(glob.glob("vendor/*/.cargo-checksum.json")):
    m = json.load(open(cj))
    if m.get("package") is None:
        continue
    d = os.path.dirname(cj)
    name = re.search(r'^name = "(.*)"$', open(os.path.join(d, "Cargo.toml")).read(), re.M).group(1)
    if len(re.findall(rf'^name = "{re.escape(name)}"$', lock, re.M)) != 1:
        continue
    rs = sorted(f for f in m["files"] if f.endswith(".rs"))
    if rs:
        print(os.path.basename(d), name, rs[0])
        break
PY
)
[ -n "${R:-}" ] || { echo "FAIL: no registry subject R in vendor/" >&2; exit 1; }
G_ROW="$(grep -vE '^(#|$)' "$PRISTINE/$DATA" | head -1 || true)"
G=""; G_URL=""; G_REV=""; G_RS=""; G_NAME=""; G_NEWV=""
if [ -n "$G_ROW" ]; then
  IFS='|' read -r G G_URL _ <<<"$G_ROW"
  G_REV="$(sed -nE "s#^source = \"git\+${G_URL}\?rev=([0-9a-f]{40})\#.*#\1#p" "$PRISTINE/Cargo.lock" | head -1)"
  G_NAME="$(sed -nE 's/^name = "(.*)"$/\1/p' "$PRISTINE/vendor/$G/Cargo.toml" | head -1)"
  # G9's version: semver-compatible with any ^ requirement (patch + 1000), so (1) still resolves.
  G_NEWV="$(sed -nE 's/^version = "([0-9]+)\.([0-9]+)\.([0-9]+).*"$/\1 \2 \3/p' "$PRISTINE/vendor/$G/Cargo.toml" | head -1 | { read -r a b c; echo "$a.$b.$((c + 1000))"; })"
  G_RS="$(python3 -I -c 'import json,sys; print(sorted(f for f in json.load(open(sys.argv[1]))["files"] if f.endswith(".rs"))[0])' "$PRISTINE/vendor/$G/.cargo-checksum.json")"
  [ -n "$G_REV" ] && [ -n "$G_NAME" ] && [[ "$G_NEWV" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo "FAIL: cannot find row $G's rev or name" >&2; exit 1; }
fi
GN="${G:-<none>}"
echo "subjects: R=$R ($R_NAME, $R_RS) G=${G:-none}${G:+ ($G_NAME, $G_RS, ${G_REV:0:8})}"
OTHER_REV="0000000000000000000000000000000000000001"

pass=0; fail=0; skip=0
# run_case <name> <expected exit: 0|1> <ERE the output must match> <mutation...>
run_case() {
  local name="$1" want="$2" re="$3"; shift 3
  local dir="$WORK/case"
  rm -rf "$dir"
  cp -a "$PRISTINE" "$dir"
  ( cd "$dir" && "$@" )
  local out rc=0
  out="$(cd "$dir" && env -u VF_SOURCES bash ci/repro/vendor-freshness.sh ${CASE_ARGS:-} 2>&1)" || rc=$?
  if [ "$rc" = "$want" ] && grep -qE "$re" <<<"$out"; then
    echo "ok   - $name (exit $rc)"
    if [ -n "${VF_TEST_VERBOSE:-}" ]; then sed 's/^/       | /' <<<"$out"; fi
    pass=$((pass + 1))
  else
    echo "FAIL - $name (exit $rc; wanted $want; output must match /$re/)"
    sed 's/^/       | /' <<<"$out"
    fail=$((fail + 1))
  fi
}
g_case() { # g_case <args of run_case>: a G row; skipped when the data file has no row
  if [ -z "$G" ]; then echo "skip - $1 (no row in $DATA)"; skip=$((skip + 1)); return; fi
  run_case "$@"
}

noop() { :; }
rm_dir() { rm -rf "vendor/$1"; }
edit_file() { echo '// tampered' >> "vendor/$1/$2"; }
bump_version() { # bump_version <crate name> [<version>]: its Cargo.lock entry claims <version> (default 0.99.0)
  python3 -I - "$1" "${2:-0.99.0}" <<'PY'
import re, sys
s = open("Cargo.lock").read()
blocks = s.split("[[package]]")
for i, b in enumerate(blocks):
    if re.search(rf'^name = "{re.escape(sys.argv[1])}"$', b, re.M):
        blocks[i] = re.sub(r'^version = ".*"$', f'version = "{sys.argv[2]}"', b, count=1, flags=re.M)
open("Cargo.lock", "w").write("[[package]]".join(blocks))
PY
}
append_lock() { printf '\n[[package]]\nname = "vf-ghost"\nversion = "0.0.0"\nsource = "%s"\n' "$1" >> Cargo.lock; }
ghost_git() { append_lock "git+https://github.com/example/elsewhere?rev=${OTHER_REV}#${OTHER_REV}"; }
ghost_branch() { append_lock "git+https://github.com/example/elsewhere?branch=main#${OTHER_REV}"; }
stale_row() { echo 'ghost|https://github.com/example/ghost|.' >> "$DATA"; }
bad_row() { echo 'not a row' >> "$DATA"; }
no_data() { rm "$DATA"; }
# The F-354 shape: a byte changes AND the manifest is rewritten to match.
revendor_wrong() {
  edit_file "$G" "$G_RS"
  python3 -I - "vendor/$G" "$G_RS" <<'PY'
import hashlib, json, sys
d, rel = sys.argv[1], sys.argv[2]
p = d + "/.cargo-checksum.json"
m = json.load(open(p))
m["files"][rel] = hashlib.sha256(open(d + "/" + rel, "rb").read()).hexdigest()
json.dump(m, open(p, "w"))
PY
}
move_rev() { # Cargo.lock and every manifest that names the rev move to OTHER_REV
  grep -rlZ --include=Cargo.lock --include=Cargo.toml --exclude-dir=vendor --exclude-dir=target "$G_REV" . \
    | xargs -0 sed -i "s/$G_REV/$OTHER_REV/g"
}
drop_row() { grep -vF "$G_ROW" "$DATA" > "$DATA.new" || true; mv "$DATA.new" "$DATA"; }
wrong_sub() { sed -i "s#^${G}|\([^|]*\)|.*#${G}|\1|no-such-dir#" "$DATA"; }
plant_file() { echo '// unlisted' > "vendor/$G/src/vf_planted.rs"; }
dup_row() { stale_row; stale_row; }
two_revs() {
  append_lock "git+https://github.com/example/elsewhere?rev=${OTHER_REV}#${OTHER_REV}"
  append_lock "git+https://github.com/example/elsewhere?rev=${OTHER_REV%1}2#${OTHER_REV%1}2"
}
# Another version vendored self-consistently: lock, Cargo.toml and manifest agree.
revendor_version() {
  bump_version "$G_NAME" "$G_NEWV"
  sed -i "0,/^version = \".*\"\$/s//version = \"$G_NEWV\"/" "vendor/$G/Cargo.toml"
  python3 -I - "vendor/$G" <<'PY'
import hashlib, json, sys
d = sys.argv[1]
p = d + "/.cargo-checksum.json"
m = json.load(open(p))
m["files"]["Cargo.toml"] = hashlib.sha256(open(d + "/Cargo.toml", "rb").read()).hexdigest()
json.dump(m, open(p, "w"))
PY
}
# A Cargo.toml-only change, manifest digest rewritten: a feature added, so name,
# version and every other file still equal upstream's (I-1).
revendor_feature() {
  python3 -I - "vendor/$G" <<'PY'
import hashlib, json, re, sys
d = sys.argv[1]
t = open(d + "/Cargo.toml", encoding="utf-8").read()
if re.search(r"^\[features\]$", t, re.M):
    t = re.sub(r"^\[features\]$", "[features]\nvf-planted = []", t, count=1, flags=re.M)
else:
    t += "\n[features]\nvf-planted = []\n"
open(d + "/Cargo.toml", "w", encoding="utf-8").write(t)
p = d + "/.cargo-checksum.json"
m = json.load(open(p))
m["files"]["Cargo.toml"] = hashlib.sha256(open(d + "/Cargo.toml", "rb").read()).hexdigest()
json.dump(m, open(p, "w"))
PY
}

# S2: src dir replaced by a symlink to an edited copy outside the crate, its src/ entries dropped from the manifest.
symlink_src() {
  python3 -I - "vendor/$G" <<'PY'
import json, os, shutil, sys
d = sys.argv[1]
shutil.copytree(d + "/src", "vf-evil-src")
with open("vf-evil-src/lib.rs", "a") as f: f.write("// tampered\n")
shutil.rmtree(d + "/src")
os.symlink("../../vf-evil-src", d + "/src")
p = d + "/.cargo-checksum.json"
m = json.load(open(p))
m["files"] = {k: v for k, v in m["files"].items() if not k.startswith("src/")}
json.dump(m, open(p, "w"))
PY
}

OOS='vendor/ is out of sync with Cargo.lock'
run_case "A1 control: the unmodified tree is green"            0 '\(4/4\) OK'                      noop
POISON_HOME="$WORK/poisoned-cargo-home"
mkdir -p "$POISON_HOME"
printf 'this is [not valid toml\n' > "$POISON_HOME/config.toml"
CARGO_HOME="$POISON_HOME" \
run_case "A2 hermetic CARGO_HOME: the caller's config.toml is never read" 0 '\(4/4\) OK'      noop
run_case "A3 registry crate $R removed -> check 1"             1 "$OOS"                            rm_dir "$R"
run_case "A4 registry crate $R stale (lock bumped) -> check 1" 1 "$OOS"                            bump_version "$R_NAME"
run_case "A5 registry crate $R file edited -> check 2"         1 "$R/$R_RS: CONTENT MISMATCH"      edit_file "$R" "$R_RS"
run_case "A6 git source with no row -> refused"                1 "has no row in $DATA"             ghost_git
run_case "A7 git source pinned by branch -> refused"           1 'not pinned by a 40-hex rev'      ghost_branch
run_case "A8 row Cargo.lock does not pin -> refused"           1 'which Cargo.lock does not pin from git' stale_row
run_case "A9 malformed data-file row -> refused"               1 "$DATA:[0-9]+: malformed row 'not a row'" bad_row
run_case "A10 data file missing -> refused"                    1 "data file $DATA not found"       no_data
CASE_ARGS=extra \
run_case "A11 an argument -> refused"                          1 'takes no arguments; got: extra'  noop
run_case "A12 two rows for one vendor dir -> refused"          1 "$DATA:[0-9]+: vendor dir 'ghost' already has a row" dup_row
run_case "A13 one git url at two revs -> refused"              1 'pins https://github.com/example/elsewhere at more than one rev' two_revs
g_case   "G1 git source $GN removed -> check 1 (F-675)"         1 "$OOS"                            rm_dir "$G"
g_case   "G2 git source $GN stale (lock bumped) -> check 1"     1 "$OOS"                            bump_version "$G_NAME"
g_case   "G3 git source $GN file edited -> check 2"             1 "$G/$G_RS: CONTENT MISMATCH"      edit_file "$G" "$G_RS"
g_case   "G4 $GN self-consistent wrong-rev byte -> check 4 (F-354)" 1 "vendor/$G: $G_RS: differs from upstream at ${G_REV:0:8}" revendor_wrong
g_case   "G5 $GN pin moved to an unfetchable rev -> check 4"    1 "vendor/$G: cannot fetch .* at $OTHER_REV" move_rev
g_case   "G6 $GN row dropped -> refused"                        1 "has no row in $DATA"             drop_row
g_case   "G7 $GN crate dir wrong -> check 4"                    1 "vendor/$G: no-such-dir is not a crate directory in" wrong_sub
g_case   "G8 $GN unlisted file planted -> check 4"              1 "vendor/$G: file set differs from .cargo-checksum.json: src/vf_planted.rs" plant_file
g_case   "G9 $GN another version vendored self-consistently -> check 4" 1 "vendor/$G: Cargo.toml: version $G_NEWV is not upstream's" revendor_version
g_case   "G10 $GN Cargo.toml-only change (a feature added) -> check 4" 1 "vendor/$G: Cargo.toml: 'features' differs from upstream at ${G_REV:0:8}" revendor_feature
g_case   "G11 $GN src dir symlinked to an edited copy, entries dropped -> refused" 1 "vendor/$G/src: SYMLINK in vendor/" symlink_src

echo "vendor-freshness.test: $pass passed, $fail failed, $skip skipped"
[ "$fail" -eq 0 ]
