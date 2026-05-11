#!/usr/bin/env bash
# tests/api-surface-coverage.sh
#
# STUB — Phase 1.0.3 placeholder. Populated at Phase 4.5 when Part V
# (Rust API reference) ships. SPEC §4.4 defines this as a hint helper,
# NOT a lint gate: it grep-checks that every public symbol in each of
# the four crates (md-codec / mk-codec / ms-codec / mnemonic-toolkit)
# appears as a Markdown heading or code-block reference in the relevant
# Part V chapter, and emits a warning row per missing symbol.
#
# Exits 0 on warnings (warning-only); the v1.0 architect sign-off at
# Phase 5.6.2 is the actual gate.
#
# Called from lint.sh step 4/6. Phase 1 returns 0 with a "stub" notice.

set -euo pipefail

for arg in "$@"; do
  case "$arg" in
    SRC_DIR=*)      SRC_DIR="${arg#*=}" ;;
    MD_BIN=*)       MD_BIN="${arg#*=}" ;;
    MK_BIN=*)       MK_BIN="${arg#*=}" ;;
    MS_BIN=*)       MS_BIN="${arg#*=}" ;;
    MNEMONIC_BIN=*) MNEMONIC_BIN="${arg#*=}" ;;
  esac
done

printf '[api-surface-coverage] STUB — Part V populated at Phase 4.5; no checks run.\n'
exit 0
