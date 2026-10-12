#!/usr/bin/env bash
# tests/root-shaped-test.sh — the self-test of tests/root-shaped.sh.
#
# Every root-shaped input below is GENERATED here (repeated or alternating
# draws, never a real root, and never written to the repo); each rule is
# tested at its threshold and one below it, and a page of the placeholders
# the `mr` chapter uses must pass. Exit 0 when every case behaves, else 1.
set -euo pipefail
here=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
SCAN="$here/root-shaped.sh"
dir=$(mktemp -d)
trap 'rm -rf "$dir"' EXIT
bad=0
n=0
rep() { local s=$1 n=$2 out='' i; for ((i = 0; i < n; i++)); do out+="$s"; done; printf '%s' "$out"; }
# expect NAME WANT(pass|W|C|D|M) TEXT
expect() {
  n=$((n + 1))
  local name=$1 want=$2 f="$dir/$1.md" rc=0 msg
  printf '%s\n' "$3" >"$f"
  msg=$(bash "$SCAN" "$f" 2>&1) || rc=$?
  if [ "$want" = pass ]; then
    [ "$rc" = 0 ] || { printf '[root-shaped-test] %s: want pass, got exit %s: %s\n' "$name" "$rc" "$msg" >&2; bad=1; }
  else
    [ "$rc" = 1 ] && grep -q -- ": $want: " <<<"$msg" \
      || { printf '[root-shaped-test] %s: want a %s refusal, got exit %s: %s\n' "$name" "$want" "$rc" "$msg" >&2; bad=1; }
  fi
}
# Placeholders: what the chapter writes where a root would go.
expect placeholders pass 'Type ROOT, an empty line, then PASS twice: `mr child --network testnet MR1_CARD`.
seed letters: A: abandon … avocado; B: avoid … cable
the root as `ms1…` or <24 words>; flips `HTHT…` (237 of them); rolls `1 6 2 …`
`ms10entrsqq9…` is the start of an ms1 string, never a whole one'
# W: 21 words pass, 22 fail; a numbered 24-word phrase over several lines fails;
# 22 four-letter prefixes (`abil`, which `mr` reads as `ability`) fail.
expect words-21 pass "$(rep 'abandon ' 21)"
expect words-22 W "$(rep 'abandon ' 22)"
expect words-24-numbered W "$(for i in $(seq 1 24); do printf '%d. ability\n' "$i"; done)"
expect words-upper W "$(rep 'Abandon ' 22)"
expect words-22-prefixes W "$(rep 'abil ' 22)"
# C: 236 flips pass, 237 fail; grouped over lines; mixed H/T with 1/0.
expect coins-236 pass "$(rep H 236)"
expect coins-237 C "$(rep H 237)"
expect coins-grouped C "$(for i in $(seq 1 47); do printf 'HTHTH '; [ $((i % 8)) = 0 ] && printf '\n'; done; printf 'HT')"
expect coins-mixed C "$(rep 'H1T0' 59)H"
# D: 91 rolls pass, 92 fail; grouped.
expect dice-91 pass "$(rep 3 91)"
expect dice-92 D "$(rep 3 92)"
expect dice-grouped D "$(rep '12345 6 ' 16)1234"
# M: ms1 + 71 characters passes, + 72 fails; grouped by 5 with spaces; upper case.
expect ms1-74 pass "ms1$(rep q 71)"
expect ms1-75 M "ms1$(rep q 72)"
expect ms1-grouped M "ms10e $(rep 'qqqqq ' 14)qq"
expect ms1-upper M "MS1$(rep Q 72)"
if [ "$bad" = 0 ]; then printf '[root-shaped-test] OK: %d cases\n' "$n"; fi
exit "$bad"
