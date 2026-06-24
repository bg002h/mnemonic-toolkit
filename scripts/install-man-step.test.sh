#!/bin/sh
# install-man-step.test.sh — P3 TDD harness for the v0.73.0 man-page install
# step in scripts/install.sh.
#
# Asserts (per SPEC §8 P3):
#   (a) `sh -n install.sh` parses clean (+ shellcheck if available);
#   (b) the real `gen-man --out` invocation is `||`-guarded (NEVER bare) so a
#       non-zero gen-man is non-fatal under `set -eu` (I-1);
#   (c) install-layer canary (C-1): generating pages from the freshly-built
#       mnemonic binary yields ZERO `*-help*.1` shadow pages.
#
# Usage: sh scripts/install-man-step.test.sh [MNEMONIC_BIN]
#   MNEMONIC_BIN defaults to target/debug/mnemonic relative to repo root.
set -eu

here=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo=$(CDPATH= cd -- "$here/.." && pwd)
INSTALL_SH="$here/install.sh"
MNEMONIC_BIN="${1:-$repo/target/debug/mnemonic}"

fail=0
ok()  { printf '  ok   %s\n' "$1"; }
bad() { printf '  FAIL %s\n' "$1" >&2; fail=1; }

echo "[install-man-step.test] (a) parse / shellcheck"
if sh -n "$INSTALL_SH"; then ok "sh -n install.sh parses"; else bad "sh -n install.sh"; fi
if command -v shellcheck >/dev/null 2>&1; then
    if shellcheck "$INSTALL_SH"; then ok "shellcheck clean"; else bad "shellcheck"; fi
else
    echo "  skip shellcheck (not on PATH)"
fi

echo "[install-man-step.test] (b) gen-man invocation is ||-guarded"
# The single REAL (non-dry-run) invocation must be followed by `|| echo
# "warning: ...`. A bare `gen-man` under set -eu would abort the install.
if grep -qE '"\$bin" gen-man --out "\$MAN_DIR" 2>/dev/null[[:space:]]*\\$' "$INSTALL_SH" \
   && grep -qE '\|\|[[:space:]]+echo "warning: man pages skipped' "$INSTALL_SH"; then
    ok "real gen-man invocation is || echo-guarded"
else
    bad "gen-man invocation not ||-guarded (would abort under set -eu)"
fi

echo "[install-man-step.test] (c) install-layer canary — zero *-help*.1 pages"
if [ -x "$MNEMONIC_BIN" ]; then
    tmp=$(mktemp -d)
    "$MNEMONIC_BIN" gen-man --out "$tmp" >/dev/null 2>&1
    total=$(find "$tmp" -name '*.1' | wc -l | tr -d ' ')
    helps=$(find "$tmp" -name '*help*.1' | wc -l | tr -d ' ')
    rm -rf "$tmp"
    if [ "$total" -gt 0 ] && [ "$helps" -eq 0 ]; then
        ok "$total pages, 0 *-help*.1 shadow pages"
    else
        bad "expected >0 pages and 0 *-help*.1 (got total=$total help=$helps)"
    fi
else
    echo "  skip canary ($MNEMONIC_BIN not built; run: cargo build -p mnemonic-toolkit --bin mnemonic)"
fi

if [ "$fail" -ne 0 ]; then
    echo "[install-man-step.test] FAILED" >&2
    exit 1
fi
echo "[install-man-step.test] OK"
