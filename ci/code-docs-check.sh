#!/usr/bin/env bash
# ci/code-docs-check.sh -- does any code name a file ci/docs-only.sh treats
# as documentation?
#
# ci/docs-only.sh skips a job's build and test steps when every changed file
# is documentation (ci/docs-only-paths.txt). If a test, build script or CI
# step reads one of those files, a change to it would skip the very job that
# reads it. This check fails when a scanned file names a docs file, so that
# case is caught when the reference is written, not after it has hidden a
# failure.
#
# Docs files: tracked files that match a `+` rule and no `-` rule in
# ci/docs-only-paths.txt (the same rules ci/docs-only.sh applies).
# Scanned files: every other tracked *.rs, *.go, *.slint, *.sh, *.py, *.toml,
# *.yml, *.yaml and *.nix file, and every Makefile and justfile, except
# files under vendor/, files matching a line of ci/code-docs-exempt.txt (one
# ERE per line, `#` comments; the file is optional), and these shared files:
# ci/docs-only.sh, ci/docs-only.test.sh, ci/code-docs-check.sh and
# ci/code-docs-check.test.sh.
# A docs file is named when a scanned line contains, as a whole name (see
# named() below), its full path, or, for a .md file, its file name when no
# other tracked file has that name.
# Skipped lines: whole-line comments (`//` in .rs, .go and .slint files, `#`
# in the others); in a .toml file, a bare `readme = "..."` key (cargo reads
# it only to package or publish); and in a file under .github/workflows/, a
# one-line `paths: [...]` or `paths-ignore: [...]` trigger filter (a filter
# names paths, it reads none).
#
# Each hit is resolved in the repo by one of: a `- ` rule in
# ci/docs-only-paths.txt (the docs file then counts as code); rewording the
# line, but only when it names the file without reading it (a message, label
# or comment): a line that reads the file must never be reworded to hide the
# name; or listing the scanned file in ci/code-docs-exempt.txt with a
# comment saying why no step the skip can skip runs it.
#
# Exit 0 = no hit; 1 = hits or a configuration error. Linux, bash 4.
# usage (from anywhere in the repository): bash ci/code-docs-check.sh
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

rules=ci/docs-only-paths.txt
exempt=ci/code-docs-exempt.txt
die() { echo "code-docs check: $1"; exit 1; }

[ -r "$rules" ] || die "cannot read $rules"
allow=()
deny=()
while IFS= read -r line || [ -n "$line" ]; do
  line=${line%$'\r'}
  case $line in
    '+ '?*) allow+=(-e "${line#+ }") ;;
    '- '?*) deny+=(-e "${line#- }") ;;
    '' | '#'*) ;;
    *) die "unparseable line in $rules: $line" ;;
  esac
done < "$rules"
[ ${#allow[@]} -gt 0 ] || die "no + rules in $rules"
# Check every pattern here: a bad one inside the process substitutions below
# would only end the subshell, leaving an empty docs list and a false "ok".
for p in "${allow[@]}" "${deny[@]}"; do
  [ "$p" = -e ] && continue
  rc=0; grep -qE -- "$p" <<<"" || rc=$?
  [ "$rc" -le 1 ] || die "bad pattern in $rules: $p"
done
skip=()
if [ -e "$exempt" ]; then
  while IFS= read -r line || [ -n "$line" ]; do
    line=${line%$'\r'}
    case $line in
      '' | '#'*) ;;
      *)
        rc=0; grep -qE -- "$line" <<<"" || rc=$?
        [ "$rc" -le 1 ] || die "bad pattern in $exempt: $line"
        skip+=(-e "$line") ;;
    esac
  done < "$exempt"
fi

# filter <grep args...>: NUL-separated paths in, matching paths out. A bad
# pattern (grep exit 2) is an error; no match (exit 1) is not.
filter() {
  local rc=0
  grep -z "$@" || rc=$?
  [ "$rc" -le 1 ] || die "bad pattern in $rules"
}

mapfile -d '' all < <(git ls-files -z)
mapfile -d '' docs < <(printf '%s\0' "${all[@]}" | filter -E "${allow[@]}" |
  if [ ${#deny[@]} -gt 0 ]; then filter -vE "${deny[@]}"; else cat; fi)
declare -A is_doc=() names=()
for f in "${docs[@]}"; do is_doc[$f]=1; done
for f in "${all[@]}"; do b=${f##*/}; names[$b]=$((${names[$b]:-0} + 1)); done

code=()
for f in "${all[@]}"; do
  [ -z "${is_doc[$f]:-}" ] || continue
  case $f in
    vendor/* | ci/docs-only.sh | ci/docs-only.test.sh | ci/code-docs-check.sh | ci/code-docs-check.test.sh) continue ;;
    *.rs | *.go | *.slint | *.sh | *.py | *.toml | *.yml | *.yaml | *.nix | Makefile | */Makefile | justfile | */justfile) ;;
    *) continue ;;
  esac
  if [ ${#skip[@]} -gt 0 ] && printf '%s\0' "$f" | grep -qzE "${skip[@]}"; then continue; fi
  code+=("$f")
done

# One pattern per docs file: for a .md file whose name no other tracked file
# has, its file name (code cites prose docs by name); else its full path
# (code reads data files by path, and a bare name like out.txt is too
# common to mean the docs file).
declare -A doc_of=()
for d in "${docs[@]}"; do
  b=${d##*/}
  if [[ $b == *.md ]] && [ "${names[$b]}" -eq 1 ]; then doc_of[$b]=$d; else doc_of[$d]=$d; fi
done

# named <text> <pattern>: 0 when <pattern> occurs in <text> as a whole name:
# the character before it is not a letter, digit, `_`, `.` or `-`, and the
# character after it is not a letter, digit, `_` or `-` (so `x_out.txt`
# does not name `out.txt`, but `"out.txt"`, `/out.txt` and `out.txt.bak` do).
named() {
  local rest=$1 pre post l r
  while [[ $rest == *"$2"* ]]; do
    pre=${rest%%"$2"*}
    post=${rest#*"$2"}
    l=${pre: -1}
    r=${post:0:1}
    if [[ ! $l =~ [A-Za-z0-9_.-] && ! $r =~ [A-Za-z0-9_-] ]]; then return 0; fi
    rest=$post
  done
  return 1
}

hits=0
if [ ${#code[@]} -gt 0 ] && [ ${#doc_of[@]} -gt 0 ]; then
  pats=$(mktemp)
  trap 'rm -f "$pats"' EXIT
  printf '%s\n' "${!doc_of[@]}" > "$pats"
  # grep -Z prints each hit as <file> NUL <line>:<text> newline.
  while IFS= read -r -d '' file && IFS= read -r rest; do
    num=${rest%%:*}
    text=${rest#*:}
    case $file in
      *.rs | *.go | *.slint) [[ $text =~ ^[[:space:]]*// ]] && continue ;;
      *) [[ $text =~ ^[[:space:]]*# ]] && continue ;;
    esac
    if [[ $file == *.toml && $text =~ ^[[:space:]]*readme[[:space:]]*=[[:space:]]*\"[^\"]*\"[[:space:]]*$ ]]; then
      continue
    fi
    if [[ $file == .github/workflows/* && $text =~ ^[[:space:]]*paths(-ignore)?:[[:space:]]*\[.*\][[:space:]]*$ ]]; then
      continue
    fi
    for p in "${!doc_of[@]}"; do
      if named "$text" "$p"; then
        hits=$((hits + 1))
        echo "FAIL ${doc_of[$p]} is named at $file:$num: ${text:0:160}"
      fi
    done
  done < <(grep -nZF -f "$pats" -- "${code[@]}" /dev/null || true)
fi

if [ "$hits" -gt 0 ]; then
  echo "code-docs check: $hits docs file reference(s). Resolve each: protect the docs file with a '- ' rule in $rules; or, only when the line names the file without reading it (a message, label or comment), reword the line; or list the scanned file in $exempt with the reason no step the skip can skip runs it."
  exit 1
fi
echo "code-docs check: ok (${#code[@]} files scanned, ${#docs[@]} docs files)"
