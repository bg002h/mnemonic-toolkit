#!/usr/bin/env bash
# tests/lint.sh
#
# Single linter entry point for the manual. Calls in sequence:
#   1. markdownlint-cli2 (style)
#   2. cspell (spelling)
#   3. lychee --offline (link integrity)
#   4. flag-coverage  (every CLI flag in cli-subcommands.list is documented)
#   5. glossary-coverage (every defined term has a glossary entry)
#   6. index bidirectional (\index{X} markers ↔ 69-index-table.md entries)
#
# Called from the Makefile as `make lint`. Args (NAME=value):
#   SRC_DIR       — absolute path to src/
#   TESTS_DIR     — absolute path to tests/
#   MNEMONIC_BIN, MD_BIN, MS_BIN — CLI invocation strings.

set -euo pipefail

for arg in "$@"; do
  case "$arg" in
    SRC_DIR=*)      SRC_DIR="${arg#*=}" ;;
    TESTS_DIR=*)    TESTS_DIR="${arg#*=}" ;;
    MNEMONIC_BIN=*) MNEMONIC_BIN="${arg#*=}" ;;
    MD_BIN=*)       MD_BIN="${arg#*=}" ;;
    MS_BIN=*)       MS_BIN="${arg#*=}" ;;
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

# 4. flag-coverage
step "4/6 flag-coverage"
LIST="$TESTS_DIR/cli-subcommands.list"
CLI_REF_DIR="$SRC_DIR/40-cli-reference"
if [ ! -f "$LIST" ]; then
  err "$LIST missing"
else
  while IFS= read -r line; do
    case "$line" in '' | '#'*) continue ;; esac
    bin="${line%% *}"; sub="${line#* }"
    case "$bin" in
      mnemonic) cmd="$MNEMONIC_BIN $sub --help" ; chapter="$CLI_REF_DIR/41-mnemonic.md" ;;
      md)       cmd="$MD_BIN $sub --help"       ; chapter="$CLI_REF_DIR/42-md.md" ;;
      ms)       cmd="$MS_BIN $sub --help"       ; chapter="$CLI_REF_DIR/43-ms.md" ;;
      *) err "unknown binary in cli-subcommands.list: $bin"; continue ;;
    esac
    if [ ! -f "$chapter" ]; then
      warn "chapter $chapter missing; skipping flag-coverage for $bin $sub"
      continue
    fi
    # shellcheck disable=SC2086
    flags=$(eval $cmd 2>&1 | grep -oE -- '--[a-z][a-z0-9-]+' | sort -u || true)
    if [ -z "$flags" ]; then
      warn "no flags parsed from \`$cmd\`; skipping"
      continue
    fi
    while read -r flag; do
      # `--` end-of-options marker prevents grep from interpreting the
      # flag string itself as an option to grep (which causes grep to
      # spam its --help output and exit non-zero).
      if ! grep -qF -- "$flag" "$chapter"; then
        err "flag $flag for \`$bin $sub\` is not documented in $(basename "$chapter")"
      fi
    done <<<"$flags"
  done <"$LIST"
fi

# 5. glossary-coverage
step "5/6 glossary-coverage"
GLOSSARY="$SRC_DIR/60-appendices/61-glossary.md"
if [ -f "$GLOSSARY" ]; then
  # Token list — keep deliberately small; expand by curating, not by regex.
  for term in "m-format star" "ms1" "mk1" "md1" "card" "bundle" "slot" "policy_id_stub" "codex32" "BCH" "BIP-388"; do
    if ! grep -qiF "$term" "$GLOSSARY"; then
      err "glossary missing entry for term: $term"
    fi
  done
else
  warn "$GLOSSARY missing; skipping glossary-coverage"
fi

# 6. index bidirectional
step "6/6 index bidirectional"
INDEX_TABLE="$SRC_DIR/60-appendices/69-index-table.md"
if [ -f "$INDEX_TABLE" ]; then
  # Every \index{TERM} in src/ must be in 69-index-table.md, and vice versa.
  # The index table file itself is excluded from the source-side scan
  # (it is the destination, not a source of authored markers, and its
  # prose may legitimately reference \index{} as documentation).
  src_terms=$(grep -rohE --exclude='69-index-table.md' '\\index\{[^}]*\}' "$SRC_DIR" | sed -E 's/^\\index\{([^}]*)\}$/\1/' | sort -u || true)
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
