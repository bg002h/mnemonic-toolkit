#!/usr/bin/env bash
# Mutation tests for ci/repro/vendor-freshness.sh (F-675).
#
# Each case copies the tracked tree (plus the working copy of the gate) into a
# scratch dir, applies ONE mutation, runs the gate there, and asserts both the
# exit status AND the check that fired. Asserting the message matters: a gate
# that REDs for the wrong reason would otherwise read as a pass here.
#
# The control case (no mutation) must be GREEN, or every RED below is vacuous.
#
# Case "md-codec removed" is the F-675 regression: before the fix, check (1)
# passed on a developer box because ~/.cargo/git held the md-codec rev, and no
# [source] stanza redirected that git source to vendor/.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRATCH_PARENT="${VF_TEST_TMPDIR:-${TMPDIR:-/tmp}}"
WORK="$(mktemp -d -p "$SCRATCH_PARENT" vf-test.XXXXXX)"
trap 'rm -rf "$WORK"' EXIT

# One pristine copy; each case clones it with hard links where it only reads.
PRISTINE="$WORK/pristine"
mkdir -p "$PRISTINE"
( cd "$REPO_ROOT" && git ls-files -z ) \
  | ( cd "$REPO_ROOT" && rsync -a --from0 --files-from=- ./ "$PRISTINE/" )
# The gate under test is the working copy, not HEAD's.
cp "$REPO_ROOT/ci/repro/vendor-freshness.sh" "$PRISTINE/ci/repro/vendor-freshness.sh"

MD_REV="$(sed -nE 's/^source = "git\+https:\/\/github.com\/bg002h\/descriptor-mnemonic\?rev=([0-9a-f]{40})#.*/\1/p' "$PRISTINE/Cargo.lock")"
[ -n "$MD_REV" ] || { echo "FAIL: cannot find md-codec's git rev in Cargo.lock" >&2; exit 1; }
OTHER_REV="0000000000000000000000000000000000000001"

pass=0; fail=0
# run_case <name> <expected exit: 0|nonzero> <regex the output must match> <mutation...>
run_case() {
  local name="$1" want="$2" re="$3"; shift 3
  local dir="$WORK/case"
  rm -rf "$dir"
  cp -a "$PRISTINE" "$dir"
  ( cd "$dir" && "$@" )
  local out rc=0
  out="$(cd "$dir" && bash ci/repro/vendor-freshness.sh 2>&1)" || rc=$?
  local ok=1
  if [ "$want" = 0 ] && [ "$rc" -ne 0 ]; then ok=0; fi
  if [ "$want" != 0 ] && [ "$rc" -eq 0 ]; then ok=0; fi
  if ! grep -qE "$re" <<<"$out"; then ok=0; fi
  if [ "$ok" = 1 ]; then
    echo "ok   - $name (exit $rc)"
    pass=$((pass + 1))
  else
    echo "FAIL - $name (exit $rc; wanted $want; output must match /$re/)"
    sed 's/^/       | /' <<<"$out"
    fail=$((fail + 1))
  fi
}

noop() { :; }
rm_md_codec() { rm -rf vendor/md-codec; }
# Stale: the pin moved to a new md-codec release, vendor/ still holds 0.47.0.
bump_md_version() {
  python3 - "$MD_REV" "$OTHER_REV" <<'PY'
import re, sys
old, new = sys.argv[1], sys.argv[2]
s = open("Cargo.lock").read()
blocks = s.split("[[package]]")
for i, b in enumerate(blocks):
    if re.search(r'^name = "md-codec"$', b, re.M):
        b = re.sub(r'^version = ".*"$', 'version = "0.99.0"', b, flags=re.M)
        blocks[i] = b.replace(old, new)
open("Cargo.lock", "w").write("[[package]]".join(blocks))
PY
  sed -i "s/$MD_REV/$OTHER_REV/" crates/mnemonic-toolkit/Cargo.toml
}
# Pin moved, same version, not re-grounded.
move_md_rev() {
  sed -i "s/$MD_REV/$OTHER_REV/g" Cargo.lock
  sed -i "s/$MD_REV/$OTHER_REV/" crates/mnemonic-toolkit/Cargo.toml
}
# A hand edit: bytes change, the manifest does not.
edit_md_file() { echo '// tampered' >> vendor/md-codec/src/lib.rs; }
# The F-354 shape: bytes change AND the manifest is rewritten to match them, so
# the tree is self-consistent. Only the grounded digest can see this.
revendor_md_wrong() {
  echo '// vendored from the wrong rev' >> vendor/md-codec/src/lib.rs
  python3 - <<'PY'
import hashlib, json
p = "vendor/md-codec/.cargo-checksum.json"
d = json.load(open(p))
d["files"]["src/lib.rs"] = hashlib.sha256(open("vendor/md-codec/src/lib.rs", "rb").read()).hexdigest()
json.dump(d, open(p, "w"))
PY
}
# A git source nobody grounded.
new_git_source() {
  sed -i 's#https://github.com/bg002h/descriptor-mnemonic?rev=#https://github.com/example/elsewhere?rev=#' Cargo.lock
}

run_case "control: unmodified tree is GREEN"        0        '\(4/4\) OK'                          noop
run_case "md-codec removed -> check 1 REDs"         nonzero  'vendor/ is out of sync with Cargo.lock' rm_md_codec
run_case "md-codec stale (pin bumped) -> check 1"   nonzero  'vendor/ is out of sync with Cargo.lock' bump_md_version
run_case "md-codec pin moved -> check 4"            nonzero  'the md-codec pin MOVED'              move_md_rev
run_case "md-codec file edited -> check 2"          nonzero  'lib.rs: CONTENT MISMATCH'            edit_md_file
run_case "md-codec self-consistent re-vendor -> 4"  nonzero  'md-codec/.cargo-checksum.json: GIT-SOURCE PROVENANCE MISMATCH' revendor_md_wrong
run_case "ungrounded git source -> fails closed"    nonzero  'has no row in GROUNDED_GIT_SOURCES'  new_git_source

# Hermetic CARGO_HOME, observed directly (review M3/VF-H): the caller's
# CARGO_HOME holds a malformed config.toml. Check (1) passes only because the
# gate resolves under its own empty home; drop that and cargo fails to parse it.
POISON_HOME="$WORK/poisoned-cargo-home"
mkdir -p "$POISON_HOME"
printf 'this is [not valid toml\n' > "$POISON_HOME/config.toml"
CARGO_HOME="$POISON_HOME" run_case "hermetic CARGO_HOME: caller's config.toml never read" 0 '\(4/4\) OK' noop

echo "vendor-freshness.test: $pass passed, $fail failed"
[ "$fail" -eq 0 ]
