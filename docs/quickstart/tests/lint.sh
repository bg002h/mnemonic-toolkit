#!/usr/bin/env bash
# Trimmed lint for docs/quickstart/. Skips manual-only checks:
# glossary-coverage (no glossary), flag-coverage (no CLI ref part),
# index-bidirectional (no \index{} markers).
set -euo pipefail

for arg in "$@"; do
  case "$arg" in
    SRC_DIR=*)   SRC_DIR="${arg#*=}" ;;
    TESTS_DIR=*) TESTS_DIR="${arg#*=}" ;;
  esac
done

: "${SRC_DIR:?SRC_DIR is required}"
: "${TESTS_DIR:?TESTS_DIR is required}"

QUICKSTART_DIR="$(dirname "$TESTS_DIR")"
fail=0

step() { printf '\n[lint] === %s ===\n' "$1"; }
warn() { printf '[lint] WARN: %s\n' "$1" >&2; }
err()  { printf '[lint] FAIL: %s\n' "$1" >&2; fail=1; }

step "1/3 markdownlint"
if command -v markdownlint-cli2 >/dev/null; then
  ( cd "$QUICKSTART_DIR" && markdownlint-cli2 "src/**/*.md" "!build/**" "!tests/fixtures/**" ) || err "markdownlint reported issues"
else
  warn "markdownlint-cli2 not on PATH; skipping"
fi

step "2/3 cspell"
if command -v cspell >/dev/null; then
  ( cd "$QUICKSTART_DIR" && cspell --no-progress --no-summary "src/**/*.md" ) || err "cspell reported issues"
else
  warn "cspell not on PATH; skipping"
fi

step "3/3 lychee"
if command -v lychee >/dev/null; then
  lychee --offline --no-progress "$SRC_DIR" || err "lychee reported issues"
else
  warn "lychee not on PATH; skipping"
fi

if [ "$fail" -ne 0 ]; then
  printf '\n[lint] FAILED\n' >&2
  exit 1
fi
printf '\n[lint] OK\n'
