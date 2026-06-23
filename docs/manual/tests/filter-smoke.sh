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
#   TRANSCRIPTS_DIR — absolute path the include-transcript.lua filter reads
#                     the whole-include fixture (include-whole-sample.out) from;
#                     points at this fixtures dir, NOT the manual transcripts.
#
# Exit 0 if both render paths produce the expected artifacts; non-zero
# with a diagnostic on failure.

set -euo pipefail

# Parse NAME=value args from the Makefile invocation.
for arg in "$@"; do
  case "$arg" in
    MANUAL_DIR=*)       MANUAL_DIR="${arg#*=}" ;;
    PANDOC=*)           PANDOC="${arg#*=}" ;;
    XELATEX=*)          XELATEX="${arg#*=}" ;;
    MAKEINDEX=*)        MAKEINDEX="${arg#*=}" ;;
    TRANSCRIPTS_DIR=*)  TRANSCRIPTS_DIR="${arg#*=}" ;;
  esac
done

: "${MANUAL_DIR:?MANUAL_DIR is required}"
: "${PANDOC:=pandoc}"
: "${XELATEX:=xelatex}"
: "${MAKEINDEX:=makeindex}"
# include-transcript.lua reads its includes from here. Default to this
# fixtures dir so a direct invocation without the Makefile still resolves the
# whole-include fixture.
: "${TRANSCRIPTS_DIR:=$MANUAL_DIR/tests/fixtures}"
export TRANSCRIPTS_DIR

FIXTURE="$MANUAL_DIR/tests/fixtures/filter-smoke.md"
FILTERS="$MANUAL_DIR/pandoc/filters"
PREAMBLE="$MANUAL_DIR/pandoc/preamble.tex"
METADATA="$MANUAL_DIR/pandoc/metadata.yaml"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

echo "[filter-smoke] using fixture: $FIXTURE"
echo "[filter-smoke] transcripts:   $TRANSCRIPTS_DIR"
echo "[filter-smoke] work dir:      $WORK"

# --- markdown render path ---------------------------------------------------
# include-transcript.lua is PREPENDED (first) so the whole-include fence's
# body is materialised from include-whole-sample.out before strip-latex sees it.
echo "[filter-smoke] render path 1/2: make md"
"$PANDOC" \
  --from markdown --to gfm \
  --lua-filter "$FILTERS/include-transcript.lua" \
  --lua-filter "$FILTERS/strip-latex-from-md.lua" \
  --lua-filter "$FILTERS/primer-box.lua" \
  --output "$WORK/smoke.md" \
  "$FIXTURE"

# Assertions: the literal `\index{m-format constellation}` marker (the only "real"
# marker in the fixture) must be stripped; the LaTeX env names must not
# appear; the primer-box prefix string must be present. We match on the
# specific marker rather than `\\index` because the fixture's prose
# legitimately contains the literal `` `\index{}` `` inside an inline
# code span as documentation.
if grep -qF '\index{m-format constellation}' "$WORK/smoke.md"; then
  echo "[filter-smoke] FAIL: real \\index{m-format constellation} marker leaked into markdown output" >&2
  exit 1
fi
if grep -q 'begin{primerbox}\|begin{dangerbox}' "$WORK/smoke.md"; then
  echo "[filter-smoke] FAIL: primerbox/dangerbox env leaked into markdown output" >&2
  exit 1
fi
if ! grep -q 'Background' "$WORK/smoke.md"; then
  echo "[filter-smoke] FAIL: 'Background.' prefix missing in markdown output" >&2
  exit 1
fi
# Assertion: the whole-include fence resolved — its body must contain the
# fixture xpub and must NOT carry the PLACEHOLDER text.
if ! grep -q 'xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4' "$WORK/smoke.md"; then
  echo "[filter-smoke] FAIL: whole-include body (xpub) missing from markdown output" >&2
  exit 1
fi
if grep -q 'PLACEHOLDER' "$WORK/smoke.md"; then
  echo "[filter-smoke] FAIL: include-transcript did not replace the PLACEHOLDER body" >&2
  exit 1
fi
echo "[filter-smoke] markdown OK"

# --- PDF render path --------------------------------------------------------
# The real PDF chain is include-transcript -> primer-box -> wrap-long-code
# (Makefile PDF_FILTER_ARGS). Load all three here so the smoke exercises the
# include->wrap COMPOSITION: the whole-include xpub6... run (>64 chars) must be
# chunked by wrap-long-code in the LaTeX writer.
echo "[filter-smoke] render path 2/2: make pdf"
"$PANDOC" \
  --from markdown --to latex --standalone \
  --lua-filter "$FILTERS/include-transcript.lua" \
  --lua-filter "$FILTERS/primer-box.lua" \
  --lua-filter "$FILTERS/wrap-long-code.lua" \
  --metadata-file="$METADATA" \
  -H "$PREAMBLE" \
  --output "$WORK/smoke.tex" \
  "$FIXTURE"

# Assertion: primerbox env emitted to LaTeX.
if ! grep -q 'begin{primerbox}' "$WORK/smoke.tex"; then
  echo "[filter-smoke] FAIL: \\begin{primerbox} missing from LaTeX output" >&2
  exit 1
fi
# Assertion: \index entry preserved in LaTeX.
if ! grep -q 'index{m-format constellation}' "$WORK/smoke.tex"; then
  echo "[filter-smoke] FAIL: \\index{m-format constellation} missing from LaTeX output" >&2
  exit 1
fi
# Assertion: include->wrap composition. wrap-long-code chunks a >=40-char
# non-space run in a CodeBlock by inserting a newline every 64 chars. The
# included xpub6... run is 111 chars, so the LaTeX CodeBlock must NOT contain
# the full unbroken 111-char run — it must be split. We assert the original
# unbroken full run is NOT present contiguously, while the first 64-char chunk
# IS — proving the wrap fired on the INCLUDED body.
if grep -qF 'xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V' "$WORK/smoke.tex"; then
  echo "[filter-smoke] FAIL: included xpub6... run was NOT wrapped by wrap-long-code (full 111-char run present unbroken)" >&2
  exit 1
fi
# The first 64-char chunk (wrap-long-code BLOCK_CHUNK_SIZE) must survive as a
# contiguous prefix — proves the body IS the included xpub, merely chunked.
if ! grep -qF 'xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3Xyuv' "$WORK/smoke.tex"; then
  echo "[filter-smoke] FAIL: first 64-char chunk of the included xpub missing from LaTeX (include did not resolve)" >&2
  exit 1
fi
echo "[filter-smoke] LaTeX OK (include->wrap composition fired)"

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
