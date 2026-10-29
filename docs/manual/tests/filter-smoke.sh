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

# Assertion: a CodeBlock line longer than wrap-long-code's MAX_BLOCK_LINE (1000
# characters) is split at whitespace. fvextra boxes a whole logical line before
# breaking it, and a line wider than TeX's maximum dimension (about 16383pt, so
# about 2,700 characters at \footnotesize DejaVu Sans Mono) aborts the PDF build
# with "Dimension too large". The chunker above only touches non-space runs of
# 40+ characters, so a long line of short words needs this rule.
# Case 1: one 2,500-character line of words separated by single
# spaces comes out as lines of at most 1000 characters, with the same words in
# the same order (each break replaces exactly one space, so rejoining the lines
# with spaces rebuilds the input).
LONG_IN="$WORK/longline.in"
LONG_MD="$WORK/longline.md"
LONG_TEX="$WORK/longline.tex"
LONG_OUT="$WORK/longline.out"
: > "$LONG_IN"
for i in $(seq 1 499); do printf 'w%03d ' "$i" >> "$LONG_IN"; done
printf 'w0500' >> "$LONG_IN"
if [ "$(wc -c < "$LONG_IN")" -ne 2500 ]; then
  echo "[filter-smoke] FAIL: long-line fixture is not 2500 characters ($(wc -c < "$LONG_IN"))" >&2
  exit 1
fi
{ printf '```text\n'; cat "$LONG_IN"; printf '\n```\n'; } > "$LONG_MD"
"$PANDOC" --from markdown --to latex --no-highlight \
  --lua-filter "$FILTERS/wrap-long-code.lua" \
  --output "$LONG_TEX" "$LONG_MD"
sed -n '/\\begin{verbatim}/,/\\end{verbatim}/p' "$LONG_TEX" | sed '1d;$d' > "$LONG_OUT"
LONG_MAX="$(awk '{ if (length($0) > m) m = length($0) } END { print m + 0 }' "$LONG_OUT")"
LONG_LINES="$(wc -l < "$LONG_OUT")"
if [ "$LONG_MAX" -gt 1000 ]; then
  echo "[filter-smoke] FAIL: a 2500-character code line was not split (longest output line: $LONG_MAX characters)" >&2
  exit 1
fi
if [ "$LONG_LINES" -lt 3 ]; then
  echo "[filter-smoke] FAIL: a 2500-character code line should become at least 3 lines (got $LONG_LINES)" >&2
  exit 1
fi
if [ "$(tr '\n' ' ' < "$LONG_OUT" | sed 's/ $//')" != "$(cat "$LONG_IN")" ]; then
  echo "[filter-smoke] FAIL: split long code line does not rejoin to the same words in the same order" >&2
  exit 1
fi
# Case 2: the limit counts characters, not bytes. One line of 169 words of 5
# characters ("é" is two bytes) plus a last word of 4 characters is 849
# characters and 1,019 bytes: over 1000 bytes, under 1000 characters, so it must
# come out as one unchanged line.
MB_IN="$WORK/multibyte.in"
MB_MD="$WORK/multibyte.md"
MB_TEX="$WORK/multibyte.tex"
MB_OUT="$WORK/multibyte.out"
: > "$MB_IN"
for i in $(seq 1 169); do printf 'é%03d ' "$i" >> "$MB_IN"; done
printf 'é170' >> "$MB_IN"
MB_BYTES="$(wc -c < "$MB_IN")"
if [ "$MB_BYTES" -le 1000 ]; then
  echo "[filter-smoke] FAIL: multibyte fixture is not over 1000 bytes ($MB_BYTES)" >&2
  exit 1
fi
{ printf '```text\n'; cat "$MB_IN"; printf '\n```\n'; } > "$MB_MD"
"$PANDOC" --from markdown --to latex --no-highlight \
  --lua-filter "$FILTERS/wrap-long-code.lua" \
  --output "$MB_TEX" "$MB_MD"
sed -n '/\\begin{verbatim}/,/\\end{verbatim}/p' "$MB_TEX" | sed '1d;$d' > "$MB_OUT"
if [ "$(wc -l < "$MB_OUT")" -ne 1 ] || [ "$(cat "$MB_OUT")" != "$(cat "$MB_IN")" ]; then
  echo "[filter-smoke] FAIL: an 849-character ($MB_BYTES-byte) code line was changed; the limit must count characters" >&2
  exit 1
fi
echo "[filter-smoke] LaTeX OK (code-block lines over 1000 characters split at whitespace)"

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
