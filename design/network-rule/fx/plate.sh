#!/bin/sh
# usage: plate.sh COIN VER SEAT -> prints the mk1 plate (one line, strings space-separated)
# of seat SEAT (0 or 1) keyed at m/270028'/COIN'/SEAT' with version bytes VER, from the
# public BIP-39 test phrases in keys.sh. Used by the mr export (D35) RED rows.
cd "$(dirname "$0")" || exit 2
r=$(sh keys.sh "$1" "$2" | sed -n "$(( $3 + 1 ))p")
fp=$(printf '%s' "$r" | cut -c2-9)
op=$(printf '%s' "$r" | sed -E 's/^\[[0-9a-f]{8}\/([^]]*)\].*/\1/')
k=$(printf '%s' "$r" | sed -E 's/^\[[^]]*\]//')
mk encode --xpub "$k" --origin-fingerprint "$fp" --origin-path "m/$op" --policy-id-stub 00000000 2>/dev/null | grep -v '^$' | tr '\n' ' ' | sed 's/ $//'
