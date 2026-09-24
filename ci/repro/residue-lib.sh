# ci/repro/residue-lib.sh — sourced by cc-validate.sh and remap-off-negative.sh.
#
# residue_hits <ERE> <file> [exclude-ERE]
#   Prints every match of <ERE> in <file> (binary-safe, one per line), minus the
#   lines matching [exclude-ERE]. Prints nothing when there is no match, and
#   returns 0 either way. Returns 2 (and says so on stderr) only if grep itself
#   fails, e.g. on an unreadable file, so a caller under `set -e` stops rather
#   than reading an error as "no residue".
#
# WHY THIS EXISTS (F-675). Both scripts used to test presence with
#     grep -aEo "$RE" "$file" | head -1 | grep -q .
# under `set -o pipefail`. When the file holds more matches than the pipe
# buffers before `head` exits, the first grep takes SIGPIPE ("grep: write
# error: Broken pipe") and pipefail makes the whole pipeline FALSE, so the
# answer depends on how much residue there is and on scheduling:
#   * remap-off-negative.sh (cross): residue present, reported as "leaked ZERO
#     /project residue" (a false RED) — repro-drift runs 32002263285 (08-17)
#     and 35594533933 (09-21), while the same code passed on 08-31/09-07/09-14.
#   * cc-validate.sh: the same shape guards "zero residue", so heavy residue
#     there read as NO residue (a false GREEN).
# Capturing the matches first, with no early-exiting reader, removes the race.
residue_hits() {
  local re="$1" file="$2" exclude="${3:-}" out rc=0
  out="$(LC_ALL=C grep -aEo -- "$re" "$file")" || rc=$?
  if [ "$rc" -gt 1 ]; then
    echo "::error::residue scan: grep failed (exit $rc) on $file" >&2
    return 2
  fi
  if [ -n "$exclude" ] && [ -n "$out" ]; then
    out="$(LC_ALL=C grep -vE -- "$exclude" <<<"$out" || true)"
  fi
  if [ -n "$out" ]; then
    printf '%s\n' "$out"
  fi
  return 0
}
