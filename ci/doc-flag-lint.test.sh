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
#
# The mr cases (G0, G1, G3) run the lint with MR_BIN set to a STUB `mr` built
# below, because mr is private and this repository's CI cannot build it. The
# real mr is linted in mnemonic-refugium's CI (job `manual-mr`).
set -uo pipefail

: "${MNEMONIC_BIN:?}" "${MD_BIN:?}" "${MS_BIN:?}" "${MK_BIN:?}"
REPO="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
TESTS="$REPO/docs/manual/tests"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
pass=0; fail=0
MR_STUB=''   # empty: the lint skips the mr lines. The G cases below set it to the stub.

run_lint() { # SRC -> lint output (exit status in $?); cwd = the book, as `make lint` runs it
  (cd "$REPO/docs/manual" && bash "$TMP/tests/lint.sh" SRC_DIR="$1" TESTS_DIR="$TMP/tests" \
    MNEMONIC_BIN="$MNEMONIC_BIN" MD_BIN="$MD_BIN" MS_BIN="$MS_BIN" MK_BIN="$MK_BIN" MR_BIN="$MR_STUB" 2>&1)
}

# A fresh copy of src/ AND tests/ (so a mutation may edit cli-subcommands.list).
fresh() {
  rm -rf "$TMP/src" "$TMP/tests"
  cp -r "$REPO/docs/manual/src" "$TMP/src"; cp -r "$TESTS" "$TMP/tests"
  [ -z "$MR_STUB" ] || make_mr_stub "$TMP/mrstub"
  echo "$TMP/src"
}

# make_mr_stub DIR -> a stub `mr` at DIR/mr. Its verbs are the mr lines of
# cli-subcommands.list; each verb defines exactly the flags in the first column
# of that verb's section in the PRISTINE (unmutated) 45-mr.md, so the unmutated
# lint passes step 4 against it. `mr --help` ends with an after_help whose
# wrapped lines begin with a flag (`   --letters YOUR_LETTERS MR1_CARD`, ...),
# as the real mr's procedure list does (review r2 of mr PR 111 measured eight
# such names in the real help). A mutation edits DIR/data/<verb>.flags (the
# binary drops a flag) or the chapter copy (the chapter gains a row).
make_mr_stub() {
  local d=$1
  rm -rf "$d"; mkdir -p "$d/data"
  python3 -I - "$d" "$REPO/docs/manual/src/40-cli-reference/45-mr.md" "$TESTS/cli-subcommands.list" <<'EOF' || { echo "FAIL: could not build the mr stub"; exit 1; }
import re, sys
d, chapter, lst = sys.argv[1:4]
verbs = [l.split(None, 1)[1].strip() for l in open(lst) if re.match(r"mr( |$)", l)]
lines = open(chapter).read().split("\n")

def section(tok):  # mirrors lint.sh section_of: the first heading carrying `tok`, to the next heading of level <= its own
    fence = inside = False
    lvl = 0
    body = []
    for l in lines:
        if l.startswith("```"):
            fence = not fence
        if not fence and re.match(r"#+ ", l):
            n = len(re.match(r"#+", l).group(0))
            if inside and n <= lvl:
                break
            if not inside and ("`" + tok + "`") in l:
                inside, lvl = True, n
                continue
        if inside:
            body.append(l)
    return body

for v in verbs:
    flags = {"--network"}
    for l in section("mr " + v):
        if re.match(r"\|\s*`", l):
            flags.update(re.findall(r"--[a-z][a-z0-9-]+", l.split("|")[1]))
    open(f"{d}/data/{v}.flags", "w").write("".join(f + "\n" for f in sorted(flags)))

commands = "".join(f"  {v:<15} A stub verb\n" for v in verbs)
open(f"{d}/data/root.help", "w").write(f"""The `mr` stub

Usage: mr [OPTIONS] <COMMAND>

Commands:
{commands}  help            Print this message or the help of the given subcommand(s)

Options:
      --network <NETWORK>  Network
  -h, --help               Print help
  -V, --version            Print version

If a seat's key is in doubt

   --for-spend` take it with a warning. Never mint a plate with the
     --origin-fingerprint FP --origin-path "PATH" --policy-id-stub
     --key @i=XPUB_i --fingerprint @i=FP_i ... --network NETWORK`,
   --private`; dash keeps none), or use a live system with no persistent
   --root -`, the root check (`mr child --check`)). A wrong passphrase
   --from coins|dice|pills`, LETTERS the letters the root makes (step 1).
   --letters YOUR_LETTERS MR1_CARD`, the root plate's ms1 string typed at
   --check --network NET MR1_CARD` with it: it prints "this root makes
""")
EOF
  cat >"$d/mr" <<'EOF'
#!/usr/bin/env bash
d="$(dirname "$0")/data"
case "${1:-}" in
  -V | --version) echo "mr 0.1.0-dev (stub)" ;;
  "" | -h | --help) cat "$d/root.help" ;;
  *)
    [ -f "$d/$1.flags" ] || { echo "error: unrecognized subcommand '$1'" >&2; exit 2; }
    printf 'A stub verb\n\nUsage: mr %s [OPTIONS]\n\nOptions:\n' "$1"
    while IFS= read -r f; do printf '      %s <X>  A stub flag\n' "$f"; done <"$d/$1.flags"
    printf '  -h, --help  Print help\n'
    ;;
esac
EOF
  chmod +x "$d/mr"
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
# The step total ("4/7") is not part of the match: lint.sh renumbers when a step
# is added, and an empty slice here makes every mutation look rejected-by-nothing.
step4() { sed -n '/=== 4\/[0-9][0-9]* flag-coverage ===/,/=== 5\/[0-9]/p' <<<"$1"; }

# baseline
out=$(run_lint "$(fresh)")
if [ -z "$(step4 "$out")" ]; then
  fail=$((fail + 1)); echo "FAIL baseline: step 4 (flag-coverage) header not found in the lint output; the mutation cases would judge an empty slice"
elif ! grep -qF '[lint] FAIL' <<<"$(step4 "$out")" && grep -qF 'flag exemption(s)' <<<"$out"; then
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
  "edit_file('$CMP', '| \`--md-only\` | compose even', '| \`--md-only\` | (unreleased: md-cli after 0.20.3: --md-only) compose even')" \
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

# --- mr cases: the lint's `mr` global set must be the root help's Options block
# only. GLOBALS once read `mr --help` to the end, so every wrapped after_help
# line that begins with a flag (the stub has eight, like the real help) was
# taken as a global option, and a verb's table row naming one was never checked
# against that verb (review r2 of mr PR 111, I-1; controls g1b and g3).
MR='40-cli-reference/45-mr.md'
MR_STUB="$TMP/mrstub/mr"

out=$(run_lint "$(fresh)")
if [ -z "$(step4 "$out")" ]; then
  fail=$((fail + 1)); echo "FAIL G0: step 4 (flag-coverage) header not found in the lint output"
elif grep -qF 'MR_BIN unset' <<<"$out"; then
  fail=$((fail + 1)); echo "FAIL G0: the lint skipped the mr lines (MR_BIN unset)"
elif grep -qF '[lint] FAIL' <<<"$(step4 "$out")"; then
  fail=$((fail + 1)); echo "FAIL G0: the unmutated manual fails flag-coverage against the mr stub"; step4 "$out" | grep -F FAIL | head
else pass=$((pass + 1)); echo "ok   G0: unmutated manual passes flag-coverage against the mr stub"; fi

mutate "G1 reverse (g1b): mr child drops --letters, which the chapter's child table keeps" \
  "edit_file(src + '/../mrstub/data/child.flags', '--letters\n', '')" \
  "flag --letters is documented for \`mr child\`"

mutate "G3 reverse (g3): the chapter's mr child table gains --root, which child never defines" \
  "edit_file('$MR', '| \`--check\` | Check', '| \`--root <ROOT>\` | A flag mr child does not have |\n| \`--check\` | Check')" \
  "flag --root is documented for \`mr child\`"

printf '\n%d passed, %d failed\n' "$pass" "$fail"
[ "$fail" -eq 0 ]
