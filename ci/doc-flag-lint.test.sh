#!/usr/bin/env bash
# ci/doc-flag-lint.test.sh — mutation tests for the manual's flag-coverage lint
# (docs/manual/tests/lint.sh, step 4), both directions.
#
#   usage: MNEMONIC_BIN=… MD_BIN=… MS_BIN=… MK_BIN=… bash ci/doc-flag-lint.test.sh
#
# Use the CI-pinned binaries (the tags in scripts/install.sh, and mnemonic built
# from this tree): the lint's reverse direction and its `(unreleased: …)`
# markers are defined against the pinned release. Each mutation is applied to a
# COPY of docs/manual/src and must make the lint fail with the expected
# message inside step 4's output; the unmutated copy must pass step 4.
set -uo pipefail

: "${MNEMONIC_BIN:?}" "${MD_BIN:?}" "${MS_BIN:?}" "${MK_BIN:?}"
REPO="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
TESTS="$REPO/docs/manual/tests"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
pass=0; fail=0

run_lint() { # SRC -> lint output (exit status in $?); cwd = the book, as `make lint` runs it
  (cd "$REPO/docs/manual" && bash "$TMP/tests/lint.sh" SRC_DIR="$1" TESTS_DIR="$TMP/tests" \
    MNEMONIC_BIN="$MNEMONIC_BIN" MD_BIN="$MD_BIN" MS_BIN="$MS_BIN" MK_BIN="$MK_BIN" 2>&1)
}

# A fresh copy of src/ AND tests/ (so a mutation may edit cli-subcommands.list).
fresh() {
  rm -rf "$TMP/src" "$TMP/tests"
  cp -r "$REPO/docs/manual/src" "$TMP/src"; cp -r "$TESTS" "$TMP/tests"
  echo "$TMP/src"
}

mutate() { # LABEL PYTHON-EDIT EXPECTED-SUBSTRING
  local src out
  src=$(fresh)
  python3 - "$src" "$2" <<'EOF' || { echo "FAIL $1: mutation did not apply"; exit 1; }
import sys, re
src, edit = sys.argv[1], sys.argv[2]
ns = {"src": src, "re": re}
def edit_file(rel, old, new, count=1):
    p = f"{src}/{rel}" if not rel.startswith("/") else rel; s = open(p).read()
    if s.count(old) < 1:
        raise SystemExit(f"mutation anchor not found in {rel}: {old[:60]!r}")
    open(p, "w").write(s.replace(old, new, count))
ns["edit_file"] = edit_file
exec(edit, ns)
EOF
  out=$(step4 "$(run_lint "$src")"); rc=$?
  if grep -qF -- "$3" <<<"$out"; then
    pass=$((pass + 1)); echo "ok   $1"
  else
    fail=$((fail + 1)); echo "FAIL $1 (rc=$rc; expected: $3)"
    grep -F '[lint] FAIL' <<<"$out" | head -5 | sed 's/^/     /'
  fi
}

# Only step 4 is judged: in a copied src/ outside docs/manual, cspell and lychee
# lose their config and relative figure paths and report noise; flag-coverage
# does not depend on either.
step4() { sed -n '/=== 4\/6 flag-coverage ===/,/=== 5\/6/p' <<<"$1"; }

# baseline
out=$(run_lint "$(fresh)")
if ! grep -qF '[lint] FAIL' <<<"$(step4 "$out")" && grep -qF 'flag exemption(s)' <<<"$out"; then
  pass=$((pass + 1)); echo "ok   baseline: unmutated manual passes flag-coverage"
else fail=$((fail + 1)); echo "FAIL baseline: unmutated manual fails flag-coverage"; step4 "$out" | grep -F FAIL | head; fi

CMP='40-cli-reference/42-md.md'
MK='40-cli-reference/44-mk-cli.md'
MS='40-cli-reference/43-ms.md'

mutate "R1 reverse: a documented --no-such-flag row in md compose" \
  "edit_file('$CMP', '| \`--md-only\` |', '| \`--no-such-flag\` | a flag md compose does not have |\n| \`--md-only\` |')" \
  "flag --no-such-flag is documented for \`md compose\`"

mutate "R2 reverse: an unreleased marker naming the wrong version" \
  "edit_file('$MK', '(unreleased: mk-cli after 0.13.0: --in)', '(unreleased: mk-cli after 0.12.0: --in)')" \
  "does not name the pinned release"

mutate "R3 reverse: a stale marker on a flag the pinned binary defines" \
  "edit_file('$CMP', '| \`--md-only\` | compose even', '| \`--md-only\` | (unreleased: md-cli after 0.20.2: --md-only) compose even')" \
  "stale marker"

mutate "R4 reverse: an unreleased row with its marker removed" \
  "edit_file('$MK', ' (unreleased: mk-cli after 0.13.0: --in) |', ' |')" \
  "but the pinned binary does not define it"

mutate "R5 reverse (fix1 NEW-2): a fake flag added to a MARKED row's cell" \
  "edit_file('$MK', '| \`--keys <FILE>\` |', '| \`--keys <FILE>\`, \`--bogus-forever\` |')" \
  "flag --bogus-forever is documented for \`mk encode\`"

mutate "R6 reverse (fix1 NEW-2): a marker naming a flag its row does not document" \
  "edit_file('$MK', '(unreleased: mk-cli after 0.13.0: --keys)', '(unreleased: mk-cli after 0.13.0: --keys, --no-such-flag)')" \
  "names --no-such-flag, which that row does not document"

mutate "R7 reverse (fix1 NEW-2): the old row-wide marker form (no flag named)" \
  "edit_file('$MK', '(unreleased: mk-cli after 0.13.0: --keys)', '(unreleased: mk-cli after 0.13.0)')" \
  "names no flag"

mutate "C1 completeness: md compose dropped from cli-subcommands.list" \
  "edit_file(src + '/../tests/cli-subcommands.list', 'md compose\\n', '')" \
  "\`md compose\` is a subcommand of the pinned binary but is missing"

mutate "C2 completeness: a listed verb the binary does not have" \
  "edit_file(src + '/../tests/cli-subcommands.list', 'md compose\\n', 'md compose\\nmd no-such-verb\\n')" \
  "\`md no-such-verb\` is listed"

mutate "F1 forward: --kind removed from the ms hashlock section" \
  "import re as _r; p=f'{src}/$MS'; s=open(p).read(); a=s.index('## \`ms hashlock\`'); b=s.index('## \`ms vectors\`'); open(p,'w').write(s[:a]+s[a:b].replace('--kind','--kxnd')+s[b:])" \
  "flag --kind for \`ms hashlock\` is not documented in its section"

mutate "F2 forward: md compose section heading removed" \
  "edit_file('$CMP', '## \`md compose\` {#md-compose}', '## Composing {#md-compose}')" \
  "\`md compose\` has no section"

printf '\n%d passed, %d failed\n' "$pass" "$fail"
[ "$fail" -eq 0 ]
