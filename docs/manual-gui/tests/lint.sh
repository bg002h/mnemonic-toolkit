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
#   9. verify-figures-gui (every figures/gui/<stem>.png is BYTE-identical
#      to the pinned mnemonic-gui checkout's
#      tests/snapshots/forms/<stem>.png; census both directions — an
#      extra manual figure (orphan baseline) fails, an upstream snapshot
#      missing from the manual fails; fail-closed. The visual analogue
#      of verify-examples-gui. Per the visual-screenshot-track SPEC §6.)
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
#   FIGURES_GUI             — absolute path to figures/gui/ (the committed
#                              screenshot corpus; threaded from the
#                              Makefile, used by verify-figures-gui).
#   EXPECTED_GUI_RENDER_COUNT — the pinned GUI schema's subcommand total
#                              (census key for verify-figures-gui; bumps
#                              in lockstep with the GUI pin).
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
    FIGURES_GUI=*)              FIGURES_GUI="${arg#*=}" ;;
    EXPECTED_GUI_RENDER_COUNT=*) EXPECTED_GUI_RENDER_COUNT="${arg#*=}" ;;
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
step "1/9 markdownlint"
if command -v markdownlint-cli2 >/dev/null; then
  markdownlint-cli2 "$SRC_DIR/**/*.md" || err "markdownlint reported issues"
else
  warn "markdownlint-cli2 not on PATH; skipping"
fi

# 2. cspell
step "2/9 cspell"
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
step "3/9 lychee"
if command -v lychee >/dev/null; then
  lychee --offline --no-progress "$SRC_DIR" || err "lychee reported issues"
else
  warn "lychee not on PATH; skipping"
fi

# 4. gui-schema-coverage
step "4/9 gui-schema-coverage"
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
step "5/9 outline-coverage"
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
step "6/9 glossary-coverage"
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
step "7/9 index bidirectional"
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
step "8/9 gui-form-xref"
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

# 9. verify-figures-gui
step "9/9 verify-figures-gui"
# Byte-compares the committed screenshot corpus (figures/gui/<stem>.png)
# against the PINNED mnemonic-gui checkout's tests/snapshots/forms/ —
# the egui_kittest snapshot corpus the GUI repo's `snapshots` CI job
# arbitrates. Census runs BOTH directions, fail-closed:
#   - a manual figure with no upstream snapshot = ORPHAN BASELINE (fail);
#   - an upstream snapshot missing from the manual = COVERAGE GAP (fail);
#   - any byte drift on a shared stem = STALE FIGURE (fail).
# Reuses the same pinned clone the schema/outline phases read (lint.sh
# already hard-requires MANUAL_GUI_UPSTREAM_ROOT); no rasterizer needed.
SNAP_DIR="$MANUAL_GUI_UPSTREAM_ROOT/tests/snapshots/forms"
if [ ! -d "${FIGURES_GUI:-}" ]; then
  err "FIGURES_GUI not a directory: ${FIGURES_GUI:-<unset>} (pass FIGURES_GUI=...; the Makefile lint: target threads it from FIGURES_GUI := \$(FIGURES_DIR)/gui)"
elif [ ! -d "$SNAP_DIR" ]; then
  err "pinned snapshot corpus missing: $SNAP_DIR (MANUAL_GUI_UPSTREAM_ROOT must be a mnemonic-gui checkout at a pinned tag >= v0.54.0)"
else
  expected_figs="${EXPECTED_GUI_RENDER_COUNT:-61}"
  fig_stems=$(find "$FIGURES_GUI" -maxdepth 1 -type f -name '*.png' | sed 's|.*/||; s/\.png$//' | sort)
  snap_stems=$(find "$SNAP_DIR" -maxdepth 1 -type f -name '*.png' | sed 's|.*/||; s/\.png$//' | sort)
  fig_count=$(printf '%s\n' "$fig_stems" | grep -c . || true)
  snap_count=$(printf '%s\n' "$snap_stems" | grep -c . || true)
  figs_fail=0
  if [ "$fig_count" -ne "$expected_figs" ]; then
    err "verify-figures-gui census: figures/gui has $fig_count PNGs, expected $expected_figs"
    figs_fail=1
  fi
  if [ "$snap_count" -ne "$expected_figs" ]; then
    err "verify-figures-gui census: pinned corpus has $snap_count PNGs, expected $expected_figs"
    figs_fail=1
  fi
  only_fig=$(comm -23 <(printf '%s\n' "$fig_stems") <(printf '%s\n' "$snap_stems") | grep . || true)
  only_snap=$(comm -13 <(printf '%s\n' "$fig_stems") <(printf '%s\n' "$snap_stems") | grep . || true)
  if [ -n "$only_fig" ]; then
    err "verify-figures-gui: manual figure(s) with NO pinned upstream snapshot (orphan baseline): $(echo $only_fig)"
    figs_fail=1
  fi
  if [ -n "$only_snap" ]; then
    err "verify-figures-gui: pinned upstream snapshot(s) MISSING from figures/gui: $(echo $only_snap)"
    figs_fail=1
  fi
  drift=""
  while IFS= read -r stem; do
    [ -z "$stem" ] && continue
    if ! cmp -s "$FIGURES_GUI/$stem.png" "$SNAP_DIR/$stem.png"; then
      drift="$drift $stem"
    fi
  done < <(comm -12 <(printf '%s\n' "$fig_stems") <(printf '%s\n' "$snap_stems"))
  if [ -n "$drift" ]; then
    err "verify-figures-gui: byte drift vs the pinned corpus for stem(s):$drift"
    figs_fail=1
  fi
  if [ "$figs_fail" -eq 0 ]; then
    printf '[lint] verify-figures-gui: OK (%s/%s figures byte-identical to the pinned corpus; census clean both directions)\n' "$fig_count" "$expected_figs"
  fi
fi

if [ "$fail" -ne 0 ]; then
  printf '\n[lint] FAILED\n' >&2
  exit 1
fi
printf '\n[lint] OK\n'
