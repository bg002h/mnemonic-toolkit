#!/usr/bin/env bash
# doc-gate.sh: build gate for design/FIX_v5_wsh_cost.md.
#
#   bash doc-gate.sh           check mode (exit 0 = every check passed)
#   bash doc-gate.sh --embed   rewrite each appendix fence from its evidence file, then run check mode
#
# Checks:
#   1. The appendix markers are exactly A-G (`<!-- evidence: FILE -->`, in order), and each marker's
#      `~~~~` fence holds FILE byte for byte.
#   2. row-alt.txt is the non-comment lines of row-names.txt joined with `|`.
#   3. The section 3 table's row numbers are 1..N and its row names equal row-names.txt, in order.
#   4. Appendices A-E apply (git apply --cached --check, on a temporary index) to ac3ff70a and to 12e013bc,
#      each alone and all five in order; Appendix F applies alone to ac3ff70a.
#
# Does NOT check: whether any figure is correct, agreement with Bitcoin Core, that the patched tree builds
# or passes its tests, the GUI byte gate, the manual job, runtime output, or the prose of the document.
# The runs the document quotes (sections 3 and 7) cover those.
set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
root=$(git -C "$here" rev-parse --show-toplevel)
doc="$root/design/FIX_v5_wsh_cost.md"
bases=(ac3ff70a 12e013bc)
build_patches=(fix-v5a-only.patch fix-e9-production.patch fix-v5a-docs.patch transcript-diffs.txt examples.diff)

if [[ ${1:-} == --embed ]]; then
  python3 -I - "$doc" "$here" <<'PY'
import re, sys, pathlib
doc, ev = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])
lines = doc.read_text().split("\n")
out, i = [], 0
while i < len(lines):
    m = re.fullmatch(r"<!-- evidence: (\S+) -->", lines[i])
    out.append(lines[i]); i += 1
    if not m:
        continue
    assert lines[i].startswith("~~~~"), f"marker {m.group(1)} not followed by a fence"
    out.append(lines[i]); i += 1
    while lines[i] != "~~~~":
        i += 1
    body = (ev / m.group(1)).read_text()
    assert body.endswith("\n") and "\n~~~~" not in "\n" + body, m.group(1)
    out.extend(body[:-1].split("\n"))
    out.append("~~~~"); i += 1
doc.write_text("\n".join(out))
PY
  echo "embedded"
fi

fail=0
python3 -I - "$doc" "$here" <<'PY' || fail=1
import re, sys, pathlib
doc, ev = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])
text = doc.read_text()
lines = text.split("\n")
bad = 0
want = ["fix-v5a-only.patch", "fix-e9-production.patch", "fix-v5a-docs.patch", "transcript-diffs.txt",
        "examples.diff", "fix-v5-production.patch", "sh-wsh-pk.txt"]
got = []
for n, l in enumerate(lines):
    m = re.fullmatch(r"<!-- evidence: (\S+) -->", l)
    if not m:
        continue
    f = m.group(1); got.append(f)
    j = n + 2
    while lines[j] != "~~~~":
        j += 1
    fence = "\n".join(lines[n + 2:j]) + "\n"
    if fence != (ev / f).read_text():
        print(f"FAIL fence != file: {f}"); bad = 1
    else:
        print(f"ok   fence == file: {f}")
if got != want:
    print(f"FAIL appendix markers {got} != {want}"); bad = 1
names = [l for l in (ev / "row-names.txt").read_text().split("\n") if l and not l.startswith("#")]
alt = (ev / "row-alt.txt").read_text().rstrip("\n")
if alt != "|".join(names):
    print("FAIL row-alt.txt != row-names.txt joined with |"); bad = 1
else:
    print(f"ok   row-alt.txt == row-names.txt ({len(names)} names)")
sec = text[text.index("\n## 3. RED rows"):text.index("\n### 3.1 ")]
rows = re.findall(r"^\| (\d+) \| [^|]* \| `([^`]+)` \|", sec, re.M)
nums = [int(r[0]) for r in rows]
if nums != list(range(1, len(rows) + 1)) or [r[1] for r in rows] != names:
    print(f"FAIL section 3 table ({len(rows)} rows) != row-names.txt ({len(names)})"); bad = 1
else:
    print(f"ok   section 3 table == row-names.txt ({len(rows)} rows)")
sys.exit(bad)
PY

idx="$(git -C "$root" rev-parse --absolute-git-dir)/doc-gate-index.$$"
trap 'rm -f "$idx"' EXIT
apply_check() {  # base, patch...: apply each in order to a fresh index of base
  local base=$1; shift
  GIT_INDEX_FILE=$idx git -C "$root" read-tree "$base"
  local p done=""
  for p in "$@"; do
    if ! GIT_INDEX_FILE=$idx git -C "$root" apply --cached "$here/$p" 2>/dev/null; then
      echo "FAIL $p does not apply to $base (after:${done:- nothing})"; return 1
    fi
    done="$done $p"
  done
}
for b in "${bases[@]}"; do
  for p in "${build_patches[@]}"; do
    if apply_check "$b" "$p"; then echo "ok   $p applies alone to $b"; else fail=1; fi
  done
  if apply_check "$b" "${build_patches[@]}"; then echo "ok   A-E apply in order to $b"; else fail=1; fi
done
if apply_check ac3ff70a fix-v5-production.patch; then echo "ok   F applies alone to ac3ff70a"; else fail=1; fi

echo "not covered: figure correctness, Core agreement, build/tests of the patched tree, GUI byte gate, manual job, runtime output, prose"
if (( fail )); then echo "doc-gate: FAIL"; exit 1; fi
echo "doc-gate: PASS"
