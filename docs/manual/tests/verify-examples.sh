#!/usr/bin/env bash
# tests/verify-examples.sh
#
# Re-run worked-example transcripts against locally-built CLIs and diff
# against the committed transcript. Drift = CI failure.
#
# Transcript layouts:
#   docs/manual/transcripts/<stem>.{cmd,out}          (pair format; 2>&1 merge)
#   docs/manual/transcripts/<subdir>/<stem>.{cmd,out,err}  (triple format)
#
# The .cmd file is a shell script body invoking the CLIs; .out is the
# expected stdout (or stdout+stderr in pair mode); .err (triple format
# only) is the expected stderr.
#
# Substitutions applied to .cmd before execution:
#   $MNEMONIC_BIN — path to the mnemonic binary
#   $MD_BIN       — path to the md binary
#   $MS_BIN       — path to the ms binary
#   $MK_BIN       — path to the mk binary
#   $FIXTURES_DIR — directory holding cross-format-recipes fixtures
#
# Each .cmd runs in a fresh `mktemp -d` cwd so recipe side-effects
# (intermediate `> envelope.json`, etc.) don't leak across transcripts.
#
# The cli-help/ subdir is excluded — it holds `--help` text snapshots,
# not transcripts (§2.1 C2 fold).
#
# Called from the Makefile as `make verify-examples`. v0.1 of the manual
# may have zero transcripts; the script must therefore exit 0 cleanly when
# none are present.

set -euo pipefail

for arg in "$@"; do
  case "$arg" in
    MNEMONIC_BIN=*)  MNEMONIC_BIN="${arg#*=}" ;;
    MD_BIN=*)        MD_BIN="${arg#*=}" ;;
    MS_BIN=*)        MS_BIN="${arg#*=}" ;;
    MK_BIN=*)        MK_BIN="${arg#*=}" ;;
    FIXTURES_DIR=*)  FIXTURES_DIR="${arg#*=}" ;;
    TRANSCRIPTS=*)   TRANSCRIPTS="${arg#*=}" ;;
  esac
done

: "${TRANSCRIPTS:?TRANSCRIPTS is required}"
: "${MNEMONIC_BIN:=true}"
: "${MD_BIN:=true}"
: "${MS_BIN:=true}"
: "${MK_BIN:=true}"
: "${FIXTURES_DIR:=}"

if [ ! -d "$TRANSCRIPTS" ]; then
  echo "[verify-examples] no transcript dir at $TRANSCRIPTS — vacuous pass"
  exit 0
fi

# Recursive .cmd discovery; exclude cli-help/ subdir (--help snapshots,
# not transcripts).
mapfile -t cmd_files < <(find "$TRANSCRIPTS" -type f -name '*.cmd' -not -path '*/cli-help/*' | sort)

fail=0
count=0
for cmd_file in "${cmd_files[@]}"; do
  count=$((count + 1))
  out_file="${cmd_file%.cmd}.out"
  err_file="${cmd_file%.cmd}.err"

  if [ ! -f "$out_file" ]; then
    echo "[verify-examples] FAIL: $cmd_file has no matching .out" >&2
    fail=1
    continue
  fi

  cmd_line=$(sed -e "s#\$MNEMONIC_BIN#$MNEMONIC_BIN#g" \
                 -e "s#\$MD_BIN#$MD_BIN#g" \
                 -e "s#\$MS_BIN#$MS_BIN#g" \
                 -e "s#\$MK_BIN#$MK_BIN#g" \
                 -e "s#\$FIXTURES_DIR#$FIXTURES_DIR#g" \
                 "$cmd_file")

  # Per-cmd tmpdir cwd so recipe side-effects don't leak across transcripts.
  tmpdir=$(mktemp -d)
  trap 'rm -rf "$tmpdir"' EXIT

  if [ -f "$err_file" ]; then
    # Triple format: stdout + stderr captured separately.
    actual_out=$(cd "$tmpdir" && bash -c "$cmd_line" 2>"$tmpdir/_stderr" || true)
    actual_err=$(cat "$tmpdir/_stderr" 2>/dev/null || true)
    expected_out=$(cat "$out_file")
    expected_err=$(cat "$err_file")

    stream_failed=0
    if [ "$actual_out" != "$expected_out" ]; then
      echo "[verify-examples] FAIL: $cmd_file STDOUT drifted" >&2
      echo "--- expected stdout ---" >&2
      diff <(printf '%s\n' "$expected_out") <(printf '%s\n' "$actual_out") >&2 || true
      stream_failed=1
    fi
    if [ "$actual_err" != "$expected_err" ]; then
      echo "[verify-examples] FAIL: $cmd_file STDERR drifted" >&2
      echo "--- expected stderr ---" >&2
      diff <(printf '%s\n' "$expected_err") <(printf '%s\n' "$actual_err") >&2 || true
      stream_failed=1
    fi
    [ "$stream_failed" -eq 1 ] && fail=1
  else
    # Pair format: 2>&1 merge.
    actual=$(cd "$tmpdir" && bash -c "$cmd_line" 2>&1 || true)
    expected=$(cat "$out_file")
    if [ "$actual" != "$expected" ]; then
      echo "[verify-examples] FAIL: $cmd_file output drifted" >&2
      diff <(printf '%s\n' "$expected") <(printf '%s\n' "$actual") >&2 || true
      fail=1
    fi
  fi

  rm -rf "$tmpdir"
  trap - EXIT
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
