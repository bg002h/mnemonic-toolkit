#!/usr/bin/env bash
# tests/lint.sh
#
# Single linter entry point for the GUI manual. Calls in sequence:
#   1. markdownlint-cli2 (style)
#   2. cspell (spelling)
#   3. lychee --offline (link integrity)
#   4. gui-schema-coverage (every SubcommandSchema / FlagSchema /
#      Dropdown variant / NodeValueComposite node / TaggedOrIndexed
#      tag in mnemonic-gui source has a matching `id="..."` anchor
#      in the rendered HTML; bidirectional — schema-shaped HTML
#      anchors must also have a schema entry. Per SPEC §2.1 G1.)
#   5. outline-coverage (every subcommand section with >=2 flags has
#      an `### Outline {#<sub>-outline}` heading with N bullets; every
#      enumerated flag with >=2 variants has a `#### Outline
#      {#<flag>-outline}` with V bullets. Per SPEC §2.1 G2 / §2.3.)
#   6. glossary-coverage (every defined term has a glossary entry)
#   7. index bidirectional (\index{X} markers ↔ 99-index-table.md entries)
#   8. gui-form-xref (every transcripts/gui/*.gui stem has exactly one
#      `{#gui-form-<stem>}` gallery anchor in src/75-gui-forms/ AND
#      exactly one `](#gui-form-<stem>)` cross-link in the subcommand
#      chapters; no orphan gui-form-* token. Per SPEC §6.)
#
# Called from the Makefile as `make lint`. Args (NAME=value):
#   SRC_DIR                 — absolute path to src/
#   BUILD_DIR               — absolute path to build/
#   TESTS_DIR               — absolute path to tests/
#   MANUAL_GUI_UPSTREAM_ROOT — absolute path to mnemonic-gui repo checkout
#                              at the pinned tag (per SPEC §2.5).
#   TRANSCRIPTS_GUI         — absolute path to transcripts/gui/ (the
#                              canonical *.gui stem list; threaded from
#                              the Makefile, used by gui-form-xref).
#   MNEMONIC_BIN, MD_BIN, MS_BIN, MK_BIN — CLI invocation strings
#                              (unused by gui-schema-coverage; reserved
#                              for future GUI-side worked-example phases).

set -euo pipefail

for arg in "$@"; do
  case "$arg" in
    SRC_DIR=*)                  SRC_DIR="${arg#*=}" ;;
    BUILD_DIR=*)                BUILD_DIR="${arg#*=}" ;;
    TESTS_DIR=*)                TESTS_DIR="${arg#*=}" ;;
    MANUAL_GUI_UPSTREAM_ROOT=*) MANUAL_GUI_UPSTREAM_ROOT="${arg#*=}" ;;
    TRANSCRIPTS_GUI=*)          TRANSCRIPTS_GUI="${arg#*=}" ;;
    MNEMONIC_BIN=*)             MNEMONIC_BIN="${arg#*=}" ;;
    MD_BIN=*)                   MD_BIN="${arg#*=}" ;;
    MS_BIN=*)                   MS_BIN="${arg#*=}" ;;
    MK_BIN=*)                   MK_BIN="${arg#*=}" ;;
  esac
done

: "${SRC_DIR:?SRC_DIR is required}"
: "${BUILD_DIR:?BUILD_DIR is required}"
: "${TESTS_DIR:?TESTS_DIR is required}"
: "${MANUAL_GUI_UPSTREAM_ROOT:?MANUAL_GUI_UPSTREAM_ROOT is required}"

fail=0
step() { printf '\n[lint] === %s ===\n' "$1"; }
warn() { printf '[lint] WARN: %s\n' "$1" >&2; }
err()  { printf '[lint] FAIL: %s\n' "$1" >&2; fail=1; }

# 1. markdownlint
step "1/8 markdownlint"
if command -v markdownlint-cli2 >/dev/null; then
  markdownlint-cli2 "$SRC_DIR/**/*.md" || err "markdownlint reported issues"
else
  warn "markdownlint-cli2 not on PATH; skipping"
fi

# 2. cspell
step "2/8 cspell"
if command -v cspell >/dev/null; then
  # `--no-must-find-files` keeps cspell from exiting 1 when src/ is
  # empty (the baseline state at P1; SPEC §2.1 G3 says all three
  # markdown phases must pass-clean on an empty manual).
  cspell --no-progress --no-must-find-files "$SRC_DIR/**/*.md" \
    || err "cspell reported issues"
else
  warn "cspell not on PATH; skipping"
fi

# 3. lychee
step "3/8 lychee"
if command -v lychee >/dev/null; then
  lychee --offline --no-progress "$SRC_DIR" || err "lychee reported issues"
else
  warn "lychee not on PATH; skipping"
fi

# 4. gui-schema-coverage
step "4/8 gui-schema-coverage"
CHECKER="$TESTS_DIR/check_gui_schema_coverage.py"
HTML="$BUILD_DIR/m-format-gui-manual.html"
if [ ! -d "$MANUAL_GUI_UPSTREAM_ROOT" ]; then
  err "MANUAL_GUI_UPSTREAM_ROOT not a directory: $MANUAL_GUI_UPSTREAM_ROOT (set the env var or override on the command line; SPEC §2.5)"
elif [ ! -x "$CHECKER" ] && [ ! -f "$CHECKER" ]; then
  err "$CHECKER missing"
else
  python3 "$CHECKER" \
      --upstream-root "$MANUAL_GUI_UPSTREAM_ROOT" \
      --html "$HTML" \
    || err "gui-schema-coverage reported anchor parity errors"
fi

# 5. outline-coverage
step "5/8 outline-coverage"
OUTLINE_CHECKER="$TESTS_DIR/check_outline_coverage.py"
if [ ! -d "$MANUAL_GUI_UPSTREAM_ROOT" ]; then
  err "MANUAL_GUI_UPSTREAM_ROOT not a directory (see phase 4 above)"
elif [ ! -f "$OUTLINE_CHECKER" ]; then
  err "$OUTLINE_CHECKER missing"
else
  python3 "$OUTLINE_CHECKER" \
      --upstream-root "$MANUAL_GUI_UPSTREAM_ROOT" \
      --src-dir "$SRC_DIR" \
    || err "outline-coverage reported missing or mismatched outlines"
fi

# 6. glossary-coverage
step "6/8 glossary-coverage"
# GUI manual appendices live under 90-appendices/ per SPEC §1.4 (the
# numbering deviates from the CLI manual's 60-appendices/ scheme so
# the two manuals never share an anchor namespace). Token list will
# grow as P2 content writes land; kept minimal at P1.
GLOSSARY="$SRC_DIR/90-appendices/91-glossary.md"
if [ -f "$GLOSSARY" ]; then
  for term in "m-format constellation" "ms1" "mk1" "md1" "card" "bundle" "slot" "SubcommandSchema" "FlagSchema" "NodeValueComposite"; do
    if ! grep -qiF "$term" "$GLOSSARY"; then
      err "glossary missing entry for term: $term"
    fi
  done
else
  warn "$GLOSSARY missing; skipping glossary-coverage"
fi

# 7. index bidirectional
step "7/8 index bidirectional"
INDEX_TABLE="$SRC_DIR/90-appendices/99-index-table.md"
if [ -f "$INDEX_TABLE" ]; then
  # Every \index{TERM} in src/ must be in 69-index-table.md, and vice versa.
  # The index table file itself is excluded from the source-side scan
  # (it is the destination, not a source of authored markers, and its
  # prose may legitimately reference \index{} as documentation).
  # Strip LaTeX escape backslashes (e.g. \_ in \index{policy\_id\_stub}) so
  # the comparison is by semantic term, not by escape form.
  src_terms=$(grep -rohE --exclude='99-index-table.md' '\\index\{[^}]*\}' "$SRC_DIR" | sed -E 's/^\\index\{([^}]*)\}$/\1/' | sed -E 's/\\_/_/g' | sort -u || true)
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

# 8. gui-form-xref
step "8/8 gui-form-xref"
XREF_CHECKER="$TESTS_DIR/check_gui_form_xref.py"
if [ ! -d "${TRANSCRIPTS_GUI:-}" ]; then
  err "TRANSCRIPTS_GUI not a directory: ${TRANSCRIPTS_GUI:-<unset>} (pass TRANSCRIPTS_GUI=...; the Makefile lint: target threads it from TRANSCRIPTS_GUI := \$(TRANSCRIPTS)/gui)"
elif [ ! -f "$XREF_CHECKER" ]; then
  err "$XREF_CHECKER missing"
else
  python3 "$XREF_CHECKER" \
      --transcripts-gui "$TRANSCRIPTS_GUI" \
      --src-dir "$SRC_DIR" \
    || err "gui-form-xref reported missing/extra/orphan cross-references"
fi

if [ "$fail" -ne 0 ]; then
  printf '\n[lint] FAILED\n' >&2
  exit 1
fi
printf '\n[lint] OK\n'
