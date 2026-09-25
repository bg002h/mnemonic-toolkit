#!/bin/sh
# install-assets.test.sh — every platform mapping in scripts/install.sh names a
# real asset of the PINNED release, and that release's SHA256SUMS* lists it
# (F-676). NEEDS NETWORK (plain HTTPS to github.com; no API, no token).
#
# For each platform the installer knows, it asks `install.sh --list` which
# asset each component would download (so this tests the real mapping code,
# not a copy), then:
#   * the asset URL must resolve (HTTP 200 after redirects);
#   * exactly one of the release's SHA256SUMS* files must list it by name.
# The platforms/components WITHOUT a binary are asserted explicitly too, so a
# mapping that silently returns "none" (a typo in a case arm) fails here.
#
# Run it whenever a pin in component_info moves: the repos name their assets
# differently and a release can change its naming.
#
# Usage: sh scripts/install-assets.test.sh
set -eu

here=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
INSTALL_SH="$here/install.sh"
T=$(mktemp -d)
trap 'rm -rf "$T"' EXIT

fail=0
ok()  { printf '  ok   %s\n' "$1"; }
bad() { printf '  FAIL %s\n' "$1"; fail=1; }

PLATFORMS="linux-x86_64-gnu linux-x86_64-musl linux-aarch64-gnu linux-aarch64-musl macos-x86_64 macos-aarch64 windows-x86_64 freebsd-x86_64"
# component@platform pairs that have NO binary in the pinned releases.
EXPECT_NONE="md@linux-x86_64-musl mnemonic@freebsd-x86_64 md@freebsd-x86_64 ms@freebsd-x86_64 mk@freebsd-x86_64 mnemonic-gui@freebsd-x86_64"
SUMS_FILES=$(sed -n 's/^SUMS_FILES="\(.*\)"$/\1/p' "$INSTALL_SH")
[ -n "$SUMS_FILES" ] || { echo "cannot read SUMS_FILES from install.sh" >&2; exit 1; }

# repo URL + tag per component, from the table itself.
base_of() {
    sed -n "s/.*echo \"[a-z-]*|\(https:[^|]*\)|\([^|]*\)|$1|.*/\1\/releases\/download\/\2/p" "$INSTALL_SH"
}

echo "[install-assets.test] $INSTALL_SH"
checked=0
for p in $PLATFORMS; do
    MNEMONIC_INSTALL_PLATFORM="$p" sh "$INSTALL_SH" --list > "$T/list"
    for n in mnemonic md ms mk mnemonic-gui; do
        asset=$(awk -v n="$n" '$1 == n { print $NF }' "$T/list")
        case " $EXPECT_NONE " in
            *" $n@$p "*)
                if [ "$asset" = "source)" ]; then ok "$n@$p: no binary (expected; builds from source)"
                else bad "$n@$p: expected no binary, installer maps it to '$asset'"; fi
                continue ;;
        esac
        if [ "$asset" = "source)" ] || [ -z "$asset" ]; then
            bad "$n@$p: installer has no binary mapping, but one is expected"
            continue
        fi
        base=$(base_of "$n")
        [ -n "$base" ] || { bad "$n: cannot read its repo/tag from install.sh"; continue; }
        if ! curl -fsSLI --proto '=https' -o /dev/null "$base/$asset"; then
            bad "$n@$p: $base/$asset does not resolve"
            continue
        fi
        hits=""
        for s in $SUMS_FILES; do
            key=$(echo "$base/$s" | tr '/:' '__')
            if [ ! -e "$T/$key" ] && [ ! -e "$T/$key.404" ]; then
                curl -fsSL --proto '=https' -o "$T/$key" "$base/$s" 2>/dev/null || { rm -f "$T/$key"; : > "$T/$key.404"; }
            fi
            [ -e "$T/$key" ] || continue
            if awk -v a="$asset" '$2 == a || $2 == "*" a { f = 1 } END { exit !f }' "$T/$key"; then
                hits="$hits $s"
            fi
        done
        set -- $hits
        if [ $# -eq 1 ]; then
            ok "$n@$p: $asset (listed in $1)"
            checked=$((checked + 1))
        else
            bad "$n@$p: $asset listed in $# SHA256SUMS files (want exactly 1):${hits:- none}"
        fi
    done
done

# The count is the guard against a loop that checked nothing.
want=$(( $(echo "$PLATFORMS" | wc -w) * 5 - $(echo "$EXPECT_NONE" | wc -w) ))
if [ "$checked" -eq "$want" ]; then ok "$checked of $want mapped assets verified"
else bad "only $checked of $want mapped assets verified"; fi

if [ "$fail" -eq 0 ]; then
    echo "[install-assets.test] OK"
else
    echo "[install-assets.test] FAILED" >&2
    exit 1
fi
