#!/usr/bin/env bash
# tests/filter-smoke.sh
#
# Phase 0 verification: render the filter-smoke.md fixture via both
# pipelines (md + pdf) and assert each produces the expected artifact.
#
# Called from the Makefile as:
#   make filter-smoke
#
# Args (NAME=value pairs from Makefile):
#   MANUAL_DIR  — absolute path to docs/manual/
#   PANDOC, XELATEX, MAKEINDEX — tool paths
#
# Exit 0 if both render paths produce the expected artifacts; non-zero
# with a diagnostic on failure.

set -euo pipefail

# Parse NAME=value args from the Makefile invocation.
for arg in "$@"; do
  case "$arg" in
    MANUAL_DIR=*)  MANUAL_DIR="${arg#*=}" ;;
    PANDOC=*)      PANDOC="${arg#*=}" ;;
    XELATEX=*)     XELATEX="${arg#*=}" ;;
    MAKEINDEX=*)   MAKEINDEX="${arg#*=}" ;;
  esac
done

: "${MANUAL_DIR:?MANUAL_DIR is required}"
: "${PANDOC:=pandoc}"
: "${XELATEX:=xelatex}"
: "${MAKEINDEX:=makeindex}"

FIXTURE="$MANUAL_DIR/tests/fixtures/filter-smoke.md"
FILTERS="$MANUAL_DIR/pandoc/filters"
PREAMBLE="$MANUAL_DIR/pandoc/preamble.tex"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

echo "[filter-smoke] using fixture: $FIXTURE"
echo "[filter-smoke] work dir:      $WORK"

# --- markdown render path ---------------------------------------------------
echo "[filter-smoke] render path 1/2: make md"
"$PANDOC" \
  --from markdown --to gfm \
  --lua-filter "$FILTERS/strip-latex-from-md.lua" \
  --lua-filter "$FILTERS/primer-box.lua" \
  --output "$WORK/smoke.md" \
  "$FIXTURE"

# Assertions: no \index, no \begin{primerbox}, but the prefix string is present.
if grep -q '\\index' "$WORK/smoke.md"; then
  echo "[filter-smoke] FAIL: \\index{} leaked into markdown output" >&2
  exit 1
fi
if grep -q 'primerbox' "$WORK/smoke.md"; then
  echo "[filter-smoke] FAIL: primerbox env leaked into markdown output" >&2
  exit 1
fi
if ! grep -q 'Background' "$WORK/smoke.md"; then
  echo "[filter-smoke] FAIL: 'Background.' prefix missing in markdown output" >&2
  exit 1
fi
echo "[filter-smoke] markdown OK"

# --- PDF render path --------------------------------------------------------
echo "[filter-smoke] render path 2/2: make pdf"
"$PANDOC" \
  --from markdown --to latex --standalone \
  --lua-filter "$FILTERS/primer-box.lua" \
  -H "$PREAMBLE" \
  --output "$WORK/smoke.tex" \
  "$FIXTURE"

# Assertion: primerbox env emitted to LaTeX.
if ! grep -q 'begin{primerbox}' "$WORK/smoke.tex"; then
  echo "[filter-smoke] FAIL: \\begin{primerbox} missing from LaTeX output" >&2
  exit 1
fi
# Assertion: \index entry preserved in LaTeX.
if ! grep -q 'index{m-format star}' "$WORK/smoke.tex"; then
  echo "[filter-smoke] FAIL: \\index{m-format star} missing from LaTeX output" >&2
  exit 1
fi
echo "[filter-smoke] LaTeX OK"

# Compile to PDF + run makeindex to verify the index machinery wires up.
( cd "$WORK" \
  && "$XELATEX" -interaction=nonstopmode -halt-on-error smoke.tex >/dev/null \
  && "$MAKEINDEX" smoke.idx \
  && "$XELATEX" -interaction=nonstopmode -halt-on-error smoke.tex >/dev/null \
)
if [ ! -s "$WORK/smoke.pdf" ]; then
  echo "[filter-smoke] FAIL: PDF output empty or missing" >&2
  exit 1
fi
if [ ! -s "$WORK/smoke.ind" ]; then
  echo "[filter-smoke] FAIL: index (.ind) file empty or missing" >&2
  exit 1
fi
echo "[filter-smoke] PDF + index OK"

echo "[filter-smoke] all checks passed."
