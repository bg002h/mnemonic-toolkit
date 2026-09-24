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
#
# REVERSE direction (2026-09-23, review I-4): every flag in the FIRST COLUMN of
# a flag-table row (`| `--flag ...` | ... |`) inside a verb's section must be
# defined by that verb's `--help` on the binary the lint runs (the CI pin), or
# be one of the binary's global options. A section shared by several verbs
# (`mnemonic seed-xor` for split + combine) accepts the union of their flags.
# Without this, a flag a release removes or renames stays documented and green.
#
# The ONE exemption is a row documenting a flag NEWER than the pinned release.
# It must carry the literal marker
#       (unreleased: <cli> after <X.Y.Z>: --flag[, --flag]...)
# e.g. `(unreleased: mk-cli after 0.13.0: --in)`, where <cli> is the binary's
# crate (mnemonic-toolkit / md-cli / ms-cli / mk-cli) and <X.Y.Z> is EXACTLY the
# version the pinned binary reports. The marker exempts exactly the flags it
# LISTS, in that row -- any other flag in the cell is linted normally (review
# fix1 NEW-2: a row-wide marker let a second, fictitious flag ride along).
# Every exemption is printed and counted. The marker FAILS when: it names no
# flag; its version is not the pin (the pin moved -- re-check the row); it
# names a flag the row does not document; or a listed flag is now defined by
# the pinned binary (the release shipped -- drop it from the marker).
step "4/6 flag-coverage"
LIST="$TESTS_DIR/cli-subcommands.list"
CLI_REF_DIR="$SRC_DIR/40-cli-reference"

# section_of CHAPTER HEADING-TOKEN -> prints `@@START <line>` (the heading's
# line number, which keys the section) and then the section body (heading line
# excluded); empty output + exit 1 when no such heading exists.
section_of() {
  awk -v tok="$2" '
    function level(l) { match(l, /^#+/); return RLENGTH }
    /^```/ { fence = !fence }
    !fence && /^#+ / {
      if (inside && level($0) <= lvl) { exit }
      if (!inside && index($0, "`" tok "`") > 0) { inside = 1; lvl = level($0); found = 1; print "@@START " NR; next }
    }
    inside { print }
    END { if (!found) exit 1 }
  ' "$1"
}

declare -A SEC_FLAGS=() SEC_BODY=() SEC_BIN=() SEC_LABEL=() PINNED=()
crate_of() { case "$1" in mnemonic) echo mnemonic-toolkit ;; md) echo md-cli ;; ms) echo ms-cli ;; mk) echo mk-cli ;; esac; }

# leaves BIN-NAME BIN-INVOCATION -> every leaf subcommand the binary exposes,
# one "<name> <sub>[ <subsub>]" per line (clap `Commands:` blocks, `help` skipped).
leaves() {
  local name=$1 binv=$2 c s subs
  # shellcheck disable=SC2086
  for c in $(eval $binv --help 2>/dev/null | sed -n '/^Commands:/,/^$/p' | awk 'NR>1 && NF{print $1}' | grep -vx help); do
    # shellcheck disable=SC2086
    subs=$(eval $binv $c --help 2>/dev/null | sed -n '/^Commands:/,/^$/p' | awk 'NR>1 && NF{print $1}' | grep -vx help || true)
    if [ -n "$subs" ]; then for s in $subs; do echo "$name $c $s"; done; else echo "$name $c"; fi
  done
}

if [ ! -f "$LIST" ]; then
  err "$LIST missing"
else
  # COMPLETENESS (fix1 self-check): the list is the lint's scope, so a verb
  # missing from it is a verb nobody checks -- exactly how `md compose` went
  # undocumented (F-647). Every leaf subcommand each pinned binary exposes must
  # be listed, and every listed verb must exist.
  listed_verbs=$(grep -vE '^[[:space:]]*(#|$)' "$LIST" | sort -u)
  exposed_verbs=$( { leaves mnemonic "$MNEMONIC_BIN"; leaves md "$MD_BIN"; leaves ms "$MS_BIN"; leaves mk "$MK_BIN"; } | sort -u)
  if [ -z "$exposed_verbs" ]; then
    err "no subcommands enumerated from the binaries (are the *_BIN real binaries?)"
  else
    while IFS= read -r v; do
      [ -n "$v" ] && ! grep -qxF -- "$v" <<<"$listed_verbs" && err "\`$v\` is a subcommand of the pinned binary but is missing from $(basename "$LIST")"
    done <<<"$exposed_verbs"
    while IFS= read -r v; do
      [ -n "$v" ] && ! grep -qxF -- "$v" <<<"$exposed_verbs" && err "\`$v\` is listed in $(basename "$LIST") but the pinned binary has no such subcommand"
    done <<<"$listed_verbs"
  fi
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
      # shellcheck disable=SC2086
      PINNED[$bin]=$(eval $binv --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1 || true)
    fi
    # shellcheck disable=SC2086
    help=$(eval $binv $sub --help 2>&1 || true)
    if [ -z "$help" ]; then
      err "\`$bin $sub --help\` printed nothing (is ${bin}'s *_BIN a real binary?)"
      continue
    fi
    # Lookup order, narrowest first: `<bin> <sub>`; for a nested verb, a
    # subsection headed `<sub>` (e.g. "### `seed-xor split` flags"); only then
    # the parent's `<bin> <parent>` section. A section shared by sibling verbs
    # accepts the union of their flags in the reverse check, so the narrower
    # section is always preferred; the shared sections left are printed below.
    parent="${sub%% *}"
    if ! section=$(section_of "$chapter" "$bin $sub") \
       && { [ "$parent" = "$sub" ] || ! section=$(section_of "$chapter" "$sub"); }; then
      if [ "$parent" = "$sub" ] || ! section=$(section_of "$chapter" "$bin $parent"); then
        err "\`$bin $sub\` has no section in $(basename "$chapter") (no heading carrying \`$bin $sub\`)"
        continue
      fi
    fi
    key="$chapter:$(head -1 <<<"$section" | cut -d' ' -f2)"
    section=$(tail -n +2 <<<"$section")
    # DEFINED options only — the lines clap indents under Options:, not every
    # `--word` in the help prose (a verb's help that mentions another verb's
    # `--json` in passing does not give this verb a `--json`).
    flags=$(printf '%s\n' "$help" | grep -oE -- '^ +(-[a-zA-Z], )?--[a-z][a-z0-9-]+' | grep -oE -- '--[a-z][a-z0-9-]+' | sort -u || true)
    SEC_FLAGS[$key]+="$flags"$'\n'
    SEC_BODY[$key]="$section"
    SEC_BIN[$key]="$bin"
    SEC_LABEL[$key]+="${SEC_LABEL[$key]:+, }$bin $sub"
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

  # Reverse direction: documented (first-column) flags must exist.
  # The exemption is per FLAG, never per row (review fix1 NEW-2): the marker
  # lists the exact flags it exempts, `(unreleased: <crate> after <X.Y.Z>:
  # --a, --b)`, and every OTHER flag in the cell is linted normally.
  exempt=0
  mre='\(unreleased: [a-z-]+ after [0-9]+\.[0-9]+\.[0-9]+(: --[a-z][a-z0-9-]+(, --[a-z][a-z0-9-]+)*)?\)'
  for key in "${!SEC_BODY[@]}"; do
    bin="${SEC_BIN[$key]}"; chap=$(basename "${key%%:*}")
    label="${SEC_LABEL[$key]}"
    known=$(printf '%s\n%s\n--help\n--version\n' "${SEC_FLAGS[$key]}" "${GLOBALS[$bin]}")
    want_head="(unreleased: $(crate_of "$bin") after ${PINNED[$bin]}:"
    while IFS= read -r row; do
      cell=$(cut -d'|' -f2 <<<"$row")
      cell_flags=$(grep -oE -- '--[a-z][a-z0-9-]+' <<<"$cell" | sort -u || true)
      markers=$(grep -oE "$mre" <<<"$row" || true)
      listed=''
      while IFS= read -r marker; do
        [ -z "$marker" ] && continue
        case "$marker" in
          *": --"*) ;;
          *) err "marker '$marker' in $chap ($label) names no flag; write '${want_head} --flag)'"; continue ;;
        esac
        if [ "${marker%%: --*}" != "${want_head%:}" ]; then
          err "marker '$marker' in $chap ($label) does not name the pinned release; expected '${want_head} …)'"
          continue
        fi
        names=$(sed -E 's/^.* after [0-9.]+: //; s/\)$//' <<<"$marker" | tr ',' '\n' | sed 's/^ *//')
        while IFS= read -r n; do
          [ -z "$n" ] && continue
          grep -qxF -- "$n" <<<"$cell_flags" \
            || err "marker '$marker' in $chap ($label) names $n, which that row does not document"
          listed+="$n"$'\n'
        done <<<"$names"
      done <<<"$markers"
      while IFS= read -r flag; do
        [ -z "$flag" ] && continue
        in_marker=0; grep -qxF -- "$flag" <<<"$listed" && in_marker=1
        if grep -qxF -- "$flag" <<<"$known"; then
          [ $in_marker = 1 ] && err "stale marker in $chap ($label): $flag is defined by the pinned $(crate_of "$bin") ${PINNED[$bin]}; remove it from the marker"
          continue
        fi
        if [ $in_marker = 1 ]; then
          printf '[lint] exempt: %s in %s (%s), unreleased after %s %s\n' "$flag" "$chap" "$label" "$(crate_of "$bin")" "${PINNED[$bin]}"
          exempt=$((exempt + 1))
          continue
        fi
        err "flag $flag is documented for \`$label\` in $chap but the pinned binary does not define it (no '${want_head} $flag)' marker naming it)"
      done <<<"$cell_flags"
    done < <(grep -E '^\|[[:space:]]*`' <<<"${SEC_BODY[$key]}" || true)
  done
  for key in "${!SEC_LABEL[@]}"; do
    case "${SEC_LABEL[$key]}" in *", "*) printf '[lint] shared section (union of flags): %s\n' "${SEC_LABEL[$key]}" ;; esac
  done
  printf '[lint] flag-coverage: %d flag exemption(s) for unreleased flags\n' "$exempt"
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
