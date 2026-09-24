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
#   MNEMONIC_BIN, MD_BIN, MS_BIN, MK_BIN — CLI invocation strings.

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

# 4. flag-coverage
#
# SECTION-scoped (2026-09-23). Every `<bin> <sub>` in cli-subcommands.list
# must have a SECTION in its chapter — a heading carrying `` `<bin> <sub>` ``
# (or, for a nested verb such as `mnemonic seed-xor split`, the parent's
# `` `<bin> <parent>` `` heading) — and every flag in `<bin> <sub> --help`
# must appear INSIDE that section, i.e. between the heading and the next
# heading of the same or a higher level.
#
# Why not the whole chapter any more: the chapter-wide grep this replaced
# passed a flag documented under ANY verb of the same binary, so `ms decode
# --in` counted as documented because `ms hashlock --in` was, and a verb with
# no section at all passed as long as its flags appeared elsewhere. Measured
# on the day of the change: 108 flag/verb pairs were undocumented in their own
# section while the chapter-wide check was green, and `md compose` /
# `md shape-key` had no section at all.
#
# Exemptions, derived from the binary rather than listed by hand:
#   * `--help` / `--version` (every verb has them);
#   * the binary's GLOBAL options — the flags `<bin> --help` itself lists —
#     which must still appear somewhere in the chapter (the old rule), since
#     they are documented once, not per verb.
# An EMPTY `--help` output is a FAIL, not a skip: it means the binary was not
# invoked (e.g. `*_BIN=true`), and a gate that checked nothing must not pass.
step "4/6 flag-coverage"
LIST="$TESTS_DIR/cli-subcommands.list"
CLI_REF_DIR="$SRC_DIR/40-cli-reference"

# section_of CHAPTER HEADING-TOKEN -> prints the section body (heading line
# excluded); empty output + exit 1 when no such heading exists.
section_of() {
  awk -v tok="$2" '
    function level(l) { match(l, /^#+/); return RLENGTH }
    /^```/ { fence = !fence }
    !fence && /^#+ / {
      if (inside && level($0) <= lvl) { exit }
      if (!inside && index($0, "`" tok "`") > 0) { inside = 1; lvl = level($0); found = 1; next }
    }
    inside { print }
    END { if (!found) exit 1 }
  ' "$1"
}

if [ ! -f "$LIST" ]; then
  err "$LIST missing"
else
  declare -A GLOBALS=()
  while IFS= read -r line; do
    case "$line" in '' | '#'*) continue ;; esac
    bin="${line%% *}"; sub="${line#* }"
    case "$bin" in
      mnemonic)   binv="$MNEMONIC_BIN" ; chapter="$CLI_REF_DIR/41-mnemonic.md" ;;
      md)         binv="$MD_BIN"       ; chapter="$CLI_REF_DIR/42-md.md" ;;
      ms)         binv="$MS_BIN"       ; chapter="$CLI_REF_DIR/43-ms.md" ;;
      mk|mk-cli)  binv="$MK_BIN"       ; chapter="$CLI_REF_DIR/44-mk-cli.md" ;;
      *) err "unknown binary in cli-subcommands.list: $bin"; continue ;;
    esac
    if [ ! -f "$chapter" ]; then
      err "chapter $chapter missing for $bin $sub"
      continue
    fi
    if [ -z "${GLOBALS[$bin]+x}" ]; then
      # shellcheck disable=SC2086
      GLOBALS[$bin]=$(eval $binv --help 2>&1 | sed -n '/^Options:/,$p' | grep -oE -- '^ +(-[a-zA-Z], )?--[a-z][a-z0-9-]+' | grep -oE -- '--[a-z][a-z0-9-]+' | sort -u || true)
    fi
    # shellcheck disable=SC2086
    help=$(eval $binv $sub --help 2>&1 || true)
    if [ -z "$help" ]; then
      err "\`$bin $sub --help\` printed nothing (is ${bin}'s *_BIN a real binary?)"
      continue
    fi
    if ! section=$(section_of "$chapter" "$bin $sub"); then
      parent="${sub%% *}"
      if [ "$parent" = "$sub" ] || ! section=$(section_of "$chapter" "$bin $parent"); then
        err "\`$bin $sub\` has no section in $(basename "$chapter") (no heading carrying \`$bin $sub\`)"
        continue
      fi
    fi
    # DEFINED options only — the lines clap indents under Options:, not every
    # `--word` in the help prose (a verb's help that mentions another verb's
    # `--json` in passing does not give this verb a `--json`).
    flags=$(printf '%s\n' "$help" | grep -oE -- '^ +(-[a-zA-Z], )?--[a-z][a-z0-9-]+' | grep -oE -- '--[a-z][a-z0-9-]+' | sort -u || true)
    while read -r flag; do
      [ -z "$flag" ] && continue
      case "$flag" in --help | --version) continue ;; esac
      # Here-strings, not `printf | grep -q`: under pipefail, grep -q exiting on
      # the first match SIGPIPEs the printf and the pipeline reports FAILURE,
      # so a documented flag would read as missing (measured on the first run).
      if grep -qxF -- "$flag" <<<"${GLOBALS[$bin]}"; then
        grep -qE -- "${flag}([^a-z0-9-]|\$)" "$chapter" \
          || err "global flag $flag of \`$bin\` is not documented anywhere in $(basename "$chapter")"
        continue
      fi
      # `--` end-of-options marker keeps grep from reading the flag as its own option.
      # Whole-flag match: `--unspendable` must not be satisfied by
      # `--unspendable-key` (the chapter-wide substring grep this replaced
      # passed `md compose --unspendable` on exactly that). Flags are
      # [a-z0-9-] only, so they are safe to splice into an ERE.
      if ! grep -qE -- "${flag}([^a-z0-9-]|\$)" <<<"$section"; then
        err "flag $flag for \`$bin $sub\` is not documented in its section of $(basename "$chapter")"
      fi
    done <<<"$flags"
  done <"$LIST"
fi

# 5. glossary-coverage
step "5/6 glossary-coverage"
GLOSSARY="$SRC_DIR/60-appendices/61-glossary.md"
if [ -f "$GLOSSARY" ]; then
  # Token list — keep deliberately small; expand by curating, not by regex.
  for term in "m-format constellation" "ms1" "mk1" "md1" "card" "bundle" "slot" "policy_id_stub" "codex32" "BCH" "BIP-388"; do
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
  # Strip LaTeX escape backslashes (e.g. \_ in \index{policy\_id\_stub}) so
  # the comparison is by semantic term, not by escape form.
  src_terms=$(grep -rohE --exclude='69-index-table.md' '\\index\{[^}]*\}' "$SRC_DIR" | sed -E 's/^\\index\{([^}]*)\}$/\1/' | sed -E 's/\\_/_/g' | sort -u || true)
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
