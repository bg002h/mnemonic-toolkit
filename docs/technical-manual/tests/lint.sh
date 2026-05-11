#!/usr/bin/env bash
# tests/lint.sh
#
# Single linter entry point for the m-format constellation TECHNICAL manual.
# Companion to docs/manual/tests/lint.sh (end-user manual) — same checks
# minus the CLI flag-coverage mirror (the technical manual's Part V is a
# library API surface, not a CLI surface; CLI-flag mirroring lives in the
# end-user manual exclusively).
#
# Sequence:
#   1. markdownlint-cli2 (style)
#   2. cspell (spelling)
#   3. lychee --offline (link integrity)
#   4. api-surface-coverage (hint helper; warning-only; Part V Rust API mirror)
#   5. glossary-coverage (every defined term has a glossary entry)
#   6. index bidirectional (\index{X} markers ↔ 62-index-table.md entries)
#
# Called from the Makefile as `make lint`. Args (NAME=value):
#   SRC_DIR       — absolute path to src/
#   TESTS_DIR     — absolute path to tests/
#   MNEMONIC_BIN, MD_BIN, MS_BIN, MK_BIN — CLI invocation strings (unused
#                   at v0.1; reserved for Phase 4.5 api-surface-coverage
#                   hint helper).

set -euo pipefail

for arg in "$@"; do
  case "$arg" in
    SRC_DIR=*)      SRC_DIR="${arg#*=}" ;;
    TESTS_DIR=*)    TESTS_DIR="${arg#*=}" ;;
    MNEMONIC_BIN=*) MNEMONIC_BIN="${arg#*=}" ;;
    MD_BIN=*)       MD_BIN="${arg#*=}" ;;
    MS_BIN=*)       MS_BIN="${arg#*=}" ;;
    MK_BIN=*)       MK_BIN="${arg#*=}" ;;
  esac
done

: "${SRC_DIR:?SRC_DIR is required}"
: "${TESTS_DIR:?TESTS_DIR is required}"

fail=0
step() { printf '\n[lint] === %s ===\n' "$1"; }
warn() { printf '[lint] WARN: %s\n' "$1" >&2; }
err()  { printf '[lint] FAIL: %s\n' "$1" >&2; fail=1; }

# 1. markdownlint
step "1/6 markdownlint"
if command -v markdownlint-cli2 >/dev/null; then
  markdownlint-cli2 "$SRC_DIR/**/*.md" || err "markdownlint reported issues"
else
  warn "markdownlint-cli2 not on PATH; skipping"
fi

# 2. cspell
step "2/6 cspell"
if command -v cspell >/dev/null; then
  cspell --no-progress "$SRC_DIR/**/*.md" || err "cspell reported issues"
else
  warn "cspell not on PATH; skipping"
fi

# 3. lychee
step "3/6 lychee"
if command -v lychee >/dev/null; then
  lychee --offline --no-progress "$SRC_DIR" || err "lychee reported issues"
else
  warn "lychee not on PATH; skipping"
fi

# 4. api-surface-coverage (Part V Rust API mirror hint)
step "4/6 api-surface-coverage"
HELPER="$TESTS_DIR/api-surface-coverage.sh"
if [ -x "$HELPER" ]; then
  bash "$HELPER" \
    SRC_DIR="$SRC_DIR" \
    MD_BIN="${MD_BIN:-}" \
    MK_BIN="${MK_BIN:-}" \
    MS_BIN="${MS_BIN:-}" \
    MNEMONIC_BIN="${MNEMONIC_BIN:-}" \
    || warn "api-surface-coverage reported gaps (warning only; see SPEC §4.4)"
else
  warn "$HELPER not present or not executable; skipping (populated at Phase 4.5)"
fi

# 5. glossary-coverage
step "5/6 glossary-coverage"
GLOSSARY="$SRC_DIR/60-back-matter/61-glossary.md"
if [ -f "$GLOSSARY" ]; then
  # Technical-manual seed terms. Expand by curating per-cut; the user-
  # facing manual's glossary is the place for end-user-facing terms.
  for term in "m-format constellation" "ms1" "mk1" "md1" "wire format" "codex32" "BCH" "BIP-388" "miniscript" "is_nums" "NUMS"; do
    if ! grep -qiF "$term" "$GLOSSARY"; then
      err "glossary missing entry for term: $term"
    fi
  done
else
  warn "$GLOSSARY missing; skipping glossary-coverage"
fi

# 6. index bidirectional
step "6/6 index bidirectional"
INDEX_TABLE="$SRC_DIR/60-back-matter/62-index-table.md"
if [ -f "$INDEX_TABLE" ]; then
  # Every \index{TERM} in src/ must be in 62-index-table.md, and vice
  # versa. The index table file itself is excluded from the source-side
  # scan (it is the destination, not a source of authored markers, and
  # its prose may legitimately reference \index{} as documentation).
  # Strip LaTeX escape backslashes (e.g. \_ in \index{policy\_id\_stub})
  # so the comparison is by semantic term, not by escape form.
  src_terms=$(grep -rohE --exclude='62-index-table.md' '\\index\{[^}]*\}' "$SRC_DIR" | sed -E 's/^\\index\{([^}]*)\}$/\1/' | sed -E 's/\\_/_/g' | sort -u || true)
  tbl_terms=$(grep -oE '^\| `[^`]+`' "$INDEX_TABLE" | sed -E 's/^\| `([^`]*)`$/\1/' | sort -u || true)
  while read -r t; do
    [ -z "$t" ] && continue
    if ! grep -qxF "$t" <(printf '%s\n' "$tbl_terms"); then
      err "src \\index{$t} missing from $INDEX_TABLE"
    fi
  done <<<"$src_terms"
  while read -r t; do
    [ -z "$t" ] && continue
    if ! grep -qxF "$t" <(printf '%s\n' "$src_terms"); then
      err "$INDEX_TABLE term '$t' has no matching \\index{} marker in src/"
    fi
  done <<<"$tbl_terms"
else
  warn "$INDEX_TABLE missing; skipping index bidirectional check"
fi

if [ "$fail" -ne 0 ]; then
  printf '\n[lint] FAILED\n' >&2
  exit 1
fi
printf '\n[lint] OK\n'
