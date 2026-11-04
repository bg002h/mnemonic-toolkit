#!/usr/bin/env bash
# ci/miniscript-tripwire.test.sh -- the RED rows (a)-(f), plus row_g, for
# ci/miniscript-tripwire.py (architect miniscript-13-2, Q2). Offline: the index
# is a committed fixture (the real 13.x rows) plus synthetic rows built here,
# and the crates are built here. Needs bash, python3, tar, sha256sum, sed.
#
# Usage: bash ci/miniscript-tripwire.test.sh [row_a row_b ...]   (default: all)
#
# Rows (b) and (d) use "a 13.2.0 tree": the real miniscript-13.2.0.crate when
# MINISCRIPT_132_CRATE names it (its sha256 must equal the fixture's 13.2.0
# cksum, else the run aborts), otherwise a tree derived from vendor/miniscript
# with the four fork-only markers renamed. Both carry 0/4 markers.
set -eu
here=$(cd "$(dirname "$0")" && pwd)
repo=$(cd "$here/.." && pwd)
script=$here/miniscript-tripwire.py
fixture=$here/fixtures/miniscript-index-13x.jsonl
tmp=$(mktemp -d "${TMPDIR:-/tmp}/ms-tripwire-test.XXXXXX")
trap 'rm -rf "$tmp"' EXIT
fails=0

# ---- fixtures ---------------------------------------------------------------
control=$tmp/control/src                 # vendor/miniscript/src, 4/4 markers
mkdir -p "$tmp/control"
cp -r "$repo/vendor/miniscript/src" "$control"

mkcrate() {  # mkcrate <src-dir> <version> <out.crate>
  local stage; stage=$(mktemp -d "$tmp/stage.XXXXXX")
  mkdir -p "$stage/miniscript-$2"
  cp -r "$1" "$stage/miniscript-$2/src"
  tar -czf "$3" -C "$stage" "miniscript-$2"
}

# A 13.2.0 tree: no fork-only markers.
if [ -n "${MINISCRIPT_132_CRATE:-}" ]; then
  want=$(python3 -I -c '
import json,sys
for l in open(sys.argv[1]):
    r=json.loads(l)
    if r["vers"]=="13.2.0": print(r["cksum"])' "$fixture")
  have=$(sha256sum "$MINISCRIPT_132_CRATE" | cut -d' ' -f1)
  [ "$want" = "$have" ] || { echo "MINISCRIPT_132_CRATE sha $have != fixture cksum $want" >&2; exit 2; }
  cp "$MINISCRIPT_132_CRATE" "$tmp/old.crate"
else
  cp -r "$control" "$tmp/old-src"
  find "$tmp/old-src" -name '*.rs' -exec sed -i -E \
    -e 's/^(\s*)SortedMultiA\(Threshold</\1RenamedA(Threshold</' \
    -e 's/^(\s*)SortedMulti\(Threshold</\1Renamed(Threshold</' \
    -e 's/pub fn derive_at_index\b/pub fn renamed_derive/' \
    -e 's/pub fn into_definite\b/pub fn renamed_definite/' {} +
  mkcrate "$tmp/old-src" 13.2.0 "$tmp/old.crate"
fi
mkcrate "$control" 13.3.0 "$tmp/new.crate"       # carries all four markers

sha() { sha256sum "$1" | cut -d' ' -f1; }

# index_with <version> <cksum> -> path of the fixture plus one synthetic row
index_with() {
  local out=$tmp/index-$1-$2.jsonl
  cp "$fixture" "$out"
  python3 -I -c '
import json,sys
print(json.dumps({"name":"miniscript","vers":sys.argv[1],"deps":[],"cksum":sys.argv[2],"features":{},"yanked":False}))' \
    "$1" "$2" >> "$out"
  echo "$out"
}

# run <args...>: sets $out and $rc.
run() {
  set +e
  out=$(python3 -I "$script" --baseline 13.2.0 "$@" 2>&1)
  rc=$?
  set -e
}

# ---- assertions -------------------------------------------------------------
fail() { echo "  FAIL: $*"; echo "$out" | sed 's/^/    | /'; fails=$((fails+1)); }
want_rc() { [ "$rc" -eq "$1" ] || fail "exit $rc, want $1"; }
want_has() { printf '%s\n' "$out" | grep -qF -- "$1" || fail "output lacks: $1"; }
want_lacks() { ! printf '%s\n' "$out" | grep -qF -- "$1" || fail "output has: $1"; }

# ---- rows -------------------------------------------------------------------
row_a() {  # live index, nothing newer than 13.2.0: step 4, exit 0
  run --index "$fixture" --control-dir "$control"
  want_rc 0
  want_has "baseline           : 13.2.0"
  want_has "highest published  : 13.2.0"
  want_has "pinned rev markers : 4/4 (vendor/miniscript)"
  want_has "No release newer than the baseline. The [patch.crates-io] pin stays."
  want_lacks "::error::"
}

row_b() {  # 13.3.0 with the 13.2.0 tree: step 7, all four missing, exit 0
  run --index "$(index_with 13.3.0 "$(sha "$tmp/old.crate")")" --crate "$tmp/old.crate" --control-dir "$control"
  want_rc 0
  want_has "highest published  : 13.3.0"
  want_has "::notice::miniscript 13.3.0 is newer than the baseline 13.2.0 but lacks Terminal::SortedMultiA (#910), Terminal::SortedMulti (#915), Descriptor::derive_at_index (#911), Descriptor::into_definite (#911); the [patch.crates-io] pin stays."
  want_lacks "::error::"
  want_lacks "NEXT STEPS:"
}

row_c() {  # 13.3.0 carrying the vendored tree: step 6, 4/4, exit 1
  run --index "$(index_with 13.3.0 "$(sha "$tmp/new.crate")")" --crate "$tmp/new.crate" --control-dir "$control"
  want_rc 1
  want_has "baseline           : 13.2.0"
  want_has "highest published  : 13.3.0"
  want_has "pinned rev markers : 4/4 (vendor/miniscript)"
  want_has "::error::miniscript 13.3.0 may let the [patch.crates-io] pin go: 4/4 fork-only APIs present (Terminal::SortedMultiA (#910), Terminal::SortedMulti (#915), Descriptor::derive_at_index (#911), Descriptor::into_definite (#911))"
  want_lacks "starts a new major line"
  want_has "NEXT STEPS:"
  want_has "  1. In a scratch worktree of every repo that carries the pin (mnemonic-toolkit and its fuzz/, descriptor-mnemonic, mnemonic-refugium and its scripts/factcheck, refugium-wallet, mnemonic-gui), delete [patch.crates-io], raise the miniscript requirement if 13.3.0 is a new major, re-lock to 13.3.0 and build (cargo check --workspace --all-targets --keep-going, as the 2026-11-03 recon did)."
  want_has "  2. If all build: work FOLLOWUP taproot-coverage-cycle-on-miniscript-gt-13-1-0, and re-derive mr-codec cost.rs's planner compensations against 13.3.0."
  want_has "  3. If any fails: raise BASELINE to 13.3.0 with a dated comment naming what is still missing."
}

row_d() {  # 14.0.0 with the 13.2.0 tree: step 6, 0/4, new major line, exit 1
  run --index "$(index_with 14.0.0 "$(sha "$tmp/old.crate")")" --crate "$tmp/old.crate" --control-dir "$control"
  want_rc 1
  want_has "highest published  : 14.0.0"
  want_has "::error::miniscript 14.0.0 may let the [patch.crates-io] pin go: 0/4 fork-only APIs present (); 14.0.0 starts a new major line"
  want_has "NEXT STEPS:"
}

row_e() {  # control dir lacking M1's line: step 2, exit 1, nothing else printed
  cp -r "$control" "$tmp/broken-src"
  find "$tmp/broken-src" -name '*.rs' -exec sed -i -E '/^\s*SortedMultiA\(Threshold</d' {} +
  run --index "$fixture" --control-dir "$tmp/broken-src"
  want_rc 1
  want_has "::error::marker scan found 3/4 fork-only APIs in vendor/miniscript (the pinned rev); the scan or the vendor tree is broken, so a green would mean nothing. Failing closed."
  want_lacks "No release newer"
}

row_f() {  # wrong cksum on the newest row: step 5, exit 1
  run --index "$(index_with 13.3.0 "$(printf '0%.0s' $(seq 64))")" --crate "$tmp/new.crate" --control-dir "$control"
  want_rc 1
  want_has "::error::could not verify miniscript-13.3.0.crate against the crates.io index checksum. Failing closed."
  want_lacks "NEXT STEPS:"
  want_lacks "::notice::"
}

row_g() {  # not in the architect's six: 2/4 markers, same major, stays green
  cp -r "$control" "$tmp/half-src"
  find "$tmp/half-src" -name '*.rs' -exec sed -i -E \
    -e 's/^(\s*)SortedMultiA\(Threshold</\1RenamedA(Threshold</' \
    -e 's/^(\s*)SortedMulti\(Threshold</\1Renamed(Threshold</' {} +
  mkcrate "$tmp/half-src" 13.3.0 "$tmp/half.crate"
  run --index "$(index_with 13.3.0 "$(sha "$tmp/half.crate")")" --crate "$tmp/half.crate" --control-dir "$control"
  want_rc 0
  want_has "::notice::miniscript 13.3.0 is newer than the baseline 13.2.0 but lacks Terminal::SortedMultiA (#910), Terminal::SortedMulti (#915); the [patch.crates-io] pin stays."
  want_lacks "::error::"
}

rows=("$@")
[ ${#rows[@]} -gt 0 ] || rows=(row_a row_b row_c row_d row_e row_f row_g)
for r in "${rows[@]}"; do
  before=$fails
  "$r"
  if [ "$fails" -eq "$before" ]; then echo "PASS $r"; else echo "FAIL $r"; fi
done
[ "$fails" -eq 0 ] || { echo "$fails assertion(s) failed"; exit 1; }
echo "all ${#rows[@]} rows passed"
