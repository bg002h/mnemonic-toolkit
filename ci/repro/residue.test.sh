#!/usr/bin/env bash
# Tests for ci/repro/residue-lib.sh and ci/repro/remap-off-negative.sh (F-675).
#
# No compiler, no container: remap-off-negative.sh runs against STUB `cargo` and
# `cross` builders placed first on PATH, which write a fixture binary where the
# real build would. That isolates what these tests are about, the verdict logic,
# from the build.
#
# The "heavy residue" fixtures are the regression: 200k path matches, far more
# than a pipe buffers, which is what made the old `grep | head -1 | grep -q`
# shape SIGPIPE under pipefail. The OLD shape is run against the same fixture
# as a control, so this test shows the race is real on this machine rather
# than asserting it.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=ci/repro/residue-lib.sh
source "$HERE/residue-lib.sh"

WORK="$(mktemp -d -p "${RESIDUE_TEST_TMPDIR:-${TMPDIR:-/tmp}}" residue-test.XXXXXX)"
trap 'rm -rf "$WORK"' EXIT

pass=0; fail=0
ok()   { echo "ok   - $1"; pass=$((pass + 1)); }
bad()  { echo "FAIL - $1"; fail=$((fail + 1)); }

# ── fixtures ──────────────────────────────────────────────────────────────
heavy() {  # <file> <path-prefix>: 200k residue lines between binary noise
  python3 - "$1" "$2" <<'PY'
import sys
path, prefix = sys.argv[1], sys.argv[2].encode()
with open(path, "wb") as f:
    f.write(b"\x7fELF\x00\x01" * 1000)
    for i in range(200_000):
        f.write(prefix + b"/crates/mnemonic-toolkit/src/m%d.rs\x00" % i)
    f.write(b"\x00\xff" * 1000)
PY
}
clean() { python3 -c "import sys;open(sys.argv[1],'wb').write(b'\x7fELF\x00'*50000)" "$1"; }

heavy "$WORK/heavy-project.bin" /project
clean "$WORK/clean.bin"

# ── residue_hits unit cases ───────────────────────────────────────────────
n="$(residue_hits '/project' "$WORK/heavy-project.bin" | wc -l)"
[ "$n" -eq 200000 ] && ok "residue_hits finds all 200000 matches in heavy residue" \
                    || bad "residue_hits found $n of 200000"

h="$(residue_hits '/project' "$WORK/clean.bin")"
[ -z "$h" ] && ok "residue_hits prints nothing on a clean file" || bad "residue_hits on clean: '$h'"

printf 'Feb 29 2024\nJan  1 1980\n' > "$WORK/dates.txt"
h="$(residue_hits '[A-Z][a-z][a-z] [ 0-9][0-9] 20[0-9][0-9]' "$WORK/dates.txt" 'Jan  1 1980')"
[ "$h" = "Feb 29 2024" ] && ok "residue_hits applies the exclude pattern" || bad "exclude: got '$h'"
h="$(residue_hits '[A-Z][a-z][a-z] [ 0-9][0-9] 1980' "$WORK/dates.txt" 'Jan  1 1980')"
[ -z "$h" ] && ok "residue_hits: everything excluded reads as no residue" || bad "all-excluded: got '$h'"

rc=0; ( residue_hits '/project' "$WORK/does-not-exist" >/dev/null 2>&1 ) || rc=$?
[ "$rc" -eq 2 ] && ok "residue_hits returns 2 on a grep error (missing file)" || bad "missing file rc=$rc"

# Control: the OLD shape, same heavy fixture, same shell options. Count how
# often it reports "no residue" over 20 tries. This does not gate the test (a
# fast box may never lose the race); it records that the race exists here.
old_false=0
for _ in $(seq 20); do
  if ! bash -c 'set -euo pipefail; grep -aEo "/project" "$1" | head -1 | grep -q .' _ "$WORK/heavy-project.bin" 2>/dev/null; then
    old_false=$((old_false + 1))
  fi
done
echo "info - OLD 'grep | head -1 | grep -q' shape reported NO residue in $old_false/20 runs on heavy residue"

# ── remap-off-negative.sh end to end, stub builders ───────────────────────
STUBS="$WORK/stubs"; mkdir -p "$STUBS"
# The stub writes target/<TARGET>/release/<BIN> in the leg's cwd from the
# fixture STUB_FIXTURE_A / STUB_FIXTURE_B, chosen by which leg dir it is in.
for b in cargo cross; do
  cat > "$STUBS/$b" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
case "$PWD" in
  */leg-a) src="$STUB_FIXTURE_A" ;;
  */leg-b) src="$STUB_FIXTURE_B" ;;
  *) echo "stub builder: unexpected cwd $PWD" >&2; exit 3 ;;
esac
mkdir -p "target/$TARGET/release"
cp "$src" "target/$TARGET/release/$BIN"
EOF
  chmod +x "$STUBS/$b"
done
mkdir -p "$WORK/leg-a" "$WORK/leg-b"

heavy "$WORK/heavy-build-a.bin" /build-a/src
heavy "$WORK/heavy-build-b.bin" /build-b/src
python3 -c "import sys;open(sys.argv[1],'wb').write(b'\x7fELF'*1000+b'nondeterminism-1')" "$WORK/nopath-1.bin"
python3 -c "import sys;open(sys.argv[1],'wb').write(b'\x7fELF'*1000+b'nondeterminism-2')" "$WORK/nopath-2.bin"

# neg_case <name> <want: pass|fail> <regex> <BUILDER> <fixture-a> <fixture-b>
neg_case() {
  local name="$1" want="$2" re="$3" builder="$4" fa="$5" fb="$6" out rc=0
  export TARGET=aarch64-unknown-linux-musl BIN=mnemonic
  out="$(PATH="$STUBS:$PATH" BUILDER="$builder" STUB_FIXTURE_A="$fa" STUB_FIXTURE_B="$fb" \
         SOURCE_DATE_EPOCH=1 CARGO_HOME=/cargo MINISCRIPT_REV="" \
         bash "$HERE/remap-off-negative.sh" "$WORK/leg-a" "$WORK/leg-b" 2>&1)" || rc=$?
  if { [ "$want" = pass ] && [ "$rc" -eq 0 ]; } || { [ "$want" = fail ] && [ "$rc" -ne 0 ]; }; then
    if grep -qE "$re" <<<"$out"; then ok "$name (exit $rc)"; return; fi
  fi
  bad "$name (exit $rc; wanted $want and /$re/)"
  sed 's/^/       | /' <<<"$out" | tail -8
}

# cross: residue present, and plenty of it -> PASS, every time.
for i in 1 2 3 4 5; do
  neg_case "cross, heavy /project residue -> PASS (run $i/5)" pass \
    'remap-off NEGATIVE PASSED' cross "$WORK/heavy-project.bin" "$WORK/heavy-project.bin"
done
# cross: the remap would be hollow -> FAIL. The gate CAN fail.
neg_case "cross, zero residue -> FAIL" fail 'leaked ZERO' cross "$WORK/clean.bin" "$WORK/clean.bin"
# cargo: legs differ by their own build paths -> PASS.
neg_case "cargo, legs leak /build-a and /build-b -> PASS" pass \
  'remap-off NEGATIVE PASSED' cargo "$WORK/heavy-build-a.bin" "$WORK/heavy-build-b.bin"
# cargo: identical without the remap -> FAIL (hollow remap).
neg_case "cargo, identical legs -> FAIL" fail 'byte-IDENTICAL' cargo "$WORK/clean.bin" "$WORK/clean.bin"
# cargo: legs differ, but not by a build path -> FAIL (m1).
neg_case "cargo, differ without a path leak -> FAIL" fail 'NEITHER leg leaks' cargo "$WORK/nopath-1.bin" "$WORK/nopath-2.bin"

echo "residue.test: $pass passed, $fail failed"
[ "$fail" -eq 0 ]
