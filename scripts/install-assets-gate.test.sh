#!/bin/sh
# install-assets-gate.test.sh — offline cases for install-assets.test.sh's
# signed-pin gate (review I1), which otherwise only runs against live
# releases, where no signed release exists yet to exercise it.
#
# The rule is extracted from install-assets.test.sh itself (signed_pin_gate)
# and version_ge from install.sh, so this tests the real code, not a copy.
# first_signed is replaced per case by a stub.
#
# Usage: sh scripts/install-assets-gate.test.sh [install-assets.test.sh]
set -eu

here=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
ASSETS=${1:-$here/install-assets.test.sh}
fail=0
ok()  { printf '  ok   %s\n' "$1"; }
bad() { printf '  FAIL %s\n' "$1"; fail=1; }

eval "$(sed -n '/^version_ge() {/,/^}/p' "$here/install.sh")"
eval "$(sed -n '/^signed_pin_gate() {/,/^}/p' "$ASSETS")"
if ! command -v version_ge >/dev/null 2>&1 || ! command -v signed_pin_gate >/dev/null 2>&1; then
    echo "cannot extract version_ge / signed_pin_gate" >&2; exit 1
fi

FIRST=""
first_signed() { echo "$FIRST"; }

# expect <label> <first_signed> <pin> <minisig yes|no> <ok|bad>
expect() {
    FIRST=$2
    v=$(signed_pin_gate mk "$3" "$4")
    case "$v" in
        "$5 "*) ok "$1 -> $5" ;;
        *) bad "$1: expected $5, got: $v" ;;
    esac
}

echo "[install-assets-gate.test] $ASSETS"
expect "signed release, first_signed empty (the stripped-signature hole)" ""       0.14.0 yes bad
expect "signed release, first_signed ABOVE the pin"                       0.15.0   0.14.0 yes bad
expect "signed release, first_signed == pin"                              0.14.0   0.14.0 yes ok
expect "signed release, first_signed below the pin"                       0.13.5   0.14.0 yes ok
expect "unsigned release, first_signed empty (today)"                     ""       0.13.0 no  ok
expect "unsigned release, first_signed above the pin (older pin)"         0.14.0   0.13.0 no  ok
expect "unsigned release, first_signed == pin (signature lost)"           0.13.0   0.13.0 no  bad
expect "unsigned release, first_signed below the pin"                     0.12.0   0.13.0 no  bad

if [ "$fail" -eq 0 ]; then
    echo "[install-assets-gate.test] OK"
else
    echo "[install-assets-gate.test] FAILED" >&2
    exit 1
fi
