#!/usr/bin/env bash
# followup-reconcile.sh — a LEADING control against stale FOLLOWUPS.md status.
#
# Root-cause it guards (2026-06-16 retro): work ships faster than its tracking.
# `toolkit-descriptor-fuzz-target` stayed `Status: open` for days after the
# fuzz target was committed + green in CI — the shipping commit never flipped
# the status, so a later session re-scoped already-shipped work. This sweep
# is a periodic audit: it flags every `Status: open` FOLLOWUP whose body cites
# a NEW-deliverable artifact (a CI workflow, a fuzz target, or a test file)
# that ALREADY EXISTS on disk — the exact "shipped-but-unflipped" signature.
#
# It is a TRIAGE aid, not a gate: a flagged entry may legitimately be open
# (the cited file is the bug site, not the deliverable). Eyeball each flag.
#
# Usage: scripts/followup-reconcile.sh [path/to/FOLLOWUPS.md]
# Exit 0 always (advisory). Run it before picking up "open" work, and flip a
# FOLLOWUP's Status IN THE SAME COMMIT that ships its deliverable.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FOLLOWUPS="${1:-$REPO_ROOT/design/FOLLOWUPS.md}"

[ -f "$FOLLOWUPS" ] || { echo "no FOLLOWUPS.md at $FOLLOWUPS" >&2; exit 0; }

# Artifact globs that are almost always NEW deliverables (low false-positive).
artifact_exists() {
  case "$1" in
    .github/workflows/*.yml|fuzz/fuzz_targets/*.rs|fuzz/fuzz_targets/*) [ -e "$REPO_ROOT/$1" ] ;;
    tests/*.rs|crates/*/tests/*.rs)                                     [ -e "$REPO_ROOT/$1" ] ;;
    *) return 1 ;;
  esac
}

open_count=0
resolved_count=0
flagged=0
slug=""
status=""
declare -a body=()

flush() {
  [ -n "$slug" ] || return 0
  case "$status" in
    open) open_count=$((open_count + 1)) ;;
    resolved|done) resolved_count=$((resolved_count + 1)); slug=""; status=""; body=(); return 0 ;;
    *) slug=""; status=""; body=(); return 0 ;;
  esac
  # OPEN entry: scan its body for cited artifact paths that exist.
  local hits=""
  for tok in $(printf '%s\n' "${body[@]}" | grep -oE '`[^`]+`' | tr -d '`'); do
    if artifact_exists "$tok"; then hits+=" $tok"; fi
  done
  if [ -n "$hits" ]; then
    flagged=$((flagged + 1))
    printf '  [VERIFY] open FOLLOWUP `%s` cites existing artifact(s):%s\n' "$slug" "$hits"
  fi
  slug=""; status=""; body=()
}

echo "== FOLLOWUP status reconciliation: $FOLLOWUPS =="
while IFS= read -r line; do
  if [[ "$line" =~ ^###[[:space:]]+\`([^\`]+)\` ]]; then
    flush
    slug="${BASH_REMATCH[1]}"
    continue
  fi
  if [ -n "$slug" ]; then
    body+=("$line")
    if [[ "$line" =~ \*\*Status:\*\*[[:space:]]*\`?([a-zA-Z]+)\`? ]]; then
      status="${BASH_REMATCH[1]}"
    fi
  fi
done < "$FOLLOWUPS"
flush

echo "-- $open_count open, $resolved_count resolved/done; $flagged flagged for verification --"
[ "$flagged" -eq 0 ] && echo "  (no open entry cites an existing new-deliverable artifact — clean)"
echo "Reminder: flip Status -> resolved IN THE SAME COMMIT that ships the deliverable."
exit 0
