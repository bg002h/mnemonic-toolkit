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
#   $EXAMPLES_DIR — absolute path to a cargo-example crate root (technical
#                   manual only: its api-roundtrip transcripts run
#                   `cargo run --manifest-path $EXAMPLES_DIR/Cargo.toml …`;
#                   each .cmd executes in a fresh mktemp -d cwd, so a relative
#                   `examples/Cargo.toml` would not resolve. Books with no
#                   cargo-example transcripts leave it unset → it substitutes
#                   to the empty string and is never referenced.)
#
# Each .cmd runs in a fresh `mktemp -d` cwd so recipe side-effects
# (intermediate `> envelope.json`, etc.) don't leak across transcripts.
#
# The cli-help/ snapshot dir was REMOVED (FOLLOWUP
# `cli-help-golden-broad-staleness-not-gated`) — those were stale,
# unrendered v0.8.0-era `--help` captures; live `--help` is authoritative
# and flag-NAME parity is gated by the flag-coverage lint. The
# `-not -path '*/cli-help/*'` predicate below is kept as a guard: if a
# `cli-help/` dir is ever re-introduced with `.cmd` files but not wired as
# a real `.cmd`→`.out` transcript pair, this keeps the runner from
# mis-discovering them.
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
    EXAMPLES_DIR=*)  EXAMPLES_DIR="${arg#*=}" ;;
    TRANSCRIPTS=*)   TRANSCRIPTS="${arg#*=}" ;;
  esac
done

: "${TRANSCRIPTS:?TRANSCRIPTS is required}"
# Manual-prose-execution-gate hardening (Piece 2 of FOLLOWUP
# `manual-prose-command-execution-gate`): require explicit binary paths. The
# pre-hardening defaults (`:= true`) made an unset env var silently resolve to
# `/bin/true`, so any md/ms/mk-using transcript would vacuously pass in a
# misconfigured CI or unparameterized direct invocation. The Makefile
# (`docs/manual/Makefile:42-45`) defaults all four via `?=` to cargo-run
# invocations, so `make audit` still works; direct script invocation now
# fails fast with a clear message instead of silently passing.
: "${MNEMONIC_BIN:?MNEMONIC_BIN is required (path to mnemonic binary)}"
: "${MD_BIN:?MD_BIN is required (path to md binary)}"
: "${MS_BIN:?MS_BIN is required (path to ms binary)}"
: "${MK_BIN:?MK_BIN is required (path to mk binary)}"
: "${FIXTURES_DIR:=}"
# Optional: only the technical-manual's cargo-example transcripts reference it.
# Defaults to empty so books without such transcripts substitute a no-op.
: "${EXAMPLES_DIR:=}"

if [ ! -d "$TRANSCRIPTS" ]; then
  echo "[verify-examples] no transcript dir at $TRANSCRIPTS — vacuous pass"
  exit 0
fi

# Recursive .cmd discovery. The `-not -path '*/cli-help/*'` predicate is a
# re-introduction guard — the cli-help/ snapshot dir was removed (see the
# header comment + FOLLOWUP `cli-help-golden-broad-staleness-not-gated`).
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

  # Guard: transcripts MUST invoke the CLIs via $MNEMONIC_BIN/$MD_BIN/$MS_BIN/
  # $MK_BIN — never a bare `mnemonic`/`md`/`ms`/`mk`. A bare name resolves off
  # $PATH locally (so it passes a dev box where the bin is installed) but FAILS
  # in CI, where only the env-var paths are provided (`mnemonic: command not
  # found`). Catch it here, loudly, instead of one CI cycle later.
  bare=$(grep -nE '(^|[|;&]|\$\()[[:space:]]*(mnemonic|md|ms|mk)[[:space:]]+[a-z-]' "$cmd_file" \
           | grep -vE '\$(MNEMONIC|MD|MS|MK)_BIN|/(mnemonic|md|ms|mk)' || true)
  if [ -n "$bare" ]; then
    echo "[verify-examples] FAIL: $cmd_file invokes a BARE binary (use \$MNEMONIC_BIN/\$MD_BIN/\$MS_BIN/\$MK_BIN; bare names pass locally on \$PATH but fail in CI):" >&2
    echo "$bare" | sed 's/^/    /' >&2
    fail=1
    continue
  fi

  cmd_line=$(sed -e "s#\$MNEMONIC_BIN#$MNEMONIC_BIN#g" \
                 -e "s#\$MD_BIN#$MD_BIN#g" \
                 -e "s#\$MS_BIN#$MS_BIN#g" \
                 -e "s#\$MK_BIN#$MK_BIN#g" \
                 -e "s#\$FIXTURES_DIR#$FIXTURES_DIR#g" \
                 -e "s#\$EXAMPLES_DIR#$EXAMPLES_DIR#g" \
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
