#!/usr/bin/env bash
# tests/verify-examples.sh
#
# Re-run worked-example transcripts against locally-built CLIs and diff
# against the committed transcript. Drift = CI failure.
#
# Transcripts live in docs/manual/transcripts/<chapter-stem>.{cmd,out}.
# `<chapter>.cmd` is a single shell line invoking the CLI; `<chapter>.out`
# is the expected stdout. The script substitutes $MNEMONIC_BIN / $MD_BIN /
# $MS_BIN at runtime.
#
# Called from the Makefile as `make verify-examples`. v0.1 of the manual
# may have zero transcripts; the script must therefore exit 0 cleanly when
# none are present.

set -euo pipefail

for arg in "$@"; do
  case "$arg" in
    MNEMONIC_BIN=*) MNEMONIC_BIN="${arg#*=}" ;;
    MD_BIN=*)       MD_BIN="${arg#*=}" ;;
    MS_BIN=*)       MS_BIN="${arg#*=}" ;;
    TRANSCRIPTS=*)  TRANSCRIPTS="${arg#*=}" ;;
  esac
done

: "${TRANSCRIPTS:?TRANSCRIPTS is required}"

if [ ! -d "$TRANSCRIPTS" ]; then
  echo "[verify-examples] no transcript dir at $TRANSCRIPTS — vacuous pass"
  exit 0
fi

shopt -s nullglob
fail=0
count=0
for cmd_file in "$TRANSCRIPTS"/*.cmd; do
  count=$((count + 1))
  out_file="${cmd_file%.cmd}.out"
  if [ ! -f "$out_file" ]; then
    echo "[verify-examples] FAIL: $cmd_file has no matching .out" >&2
    fail=1
    continue
  fi
  cmd_line=$(sed -e "s#\$MNEMONIC_BIN#$MNEMONIC_BIN#g" \
                 -e "s#\$MD_BIN#$MD_BIN#g" \
                 -e "s#\$MS_BIN#$MS_BIN#g" \
                 "$cmd_file")
  actual=$(bash -c "$cmd_line" 2>&1 || true)
  expected=$(cat "$out_file")
  if [ "$actual" != "$expected" ]; then
    echo "[verify-examples] FAIL: $cmd_file output drifted" >&2
    diff <(printf '%s\n' "$expected") <(printf '%s\n' "$actual") || true
    fail=1
  fi
done

if [ "$count" -eq 0 ]; then
  echo "[verify-examples] no transcripts found — vacuous pass"
  exit 0
fi
if [ "$fail" -ne 0 ]; then
  echo "[verify-examples] FAILED ($count transcripts checked)" >&2
  exit 1
fi
echo "[verify-examples] OK ($count transcripts pass)"
