#!/bin/sh
# usage: upx.sh FX_DIR SVER   (the fx directory; seat @0's plate version bytes)
# Mints the signet explicit-path card of `up.sh 1 043587cf signet`, then runs `mr export` on
# signet with seat @0's plate at m/48'/1'/0'/2' of version SVER and seat @1's tpub plate.
F=$1; SV=$2
A="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
B="legal winner thank year wave sausage worth useful legal winner thank yellow"
C=$(sh "$F/up.sh" 1 043587cf signet 2>/dev/null | grep -E '^  mr1' | head -1 | tr -d ' ')
p() { r=$(python3 -I "$F/b32.py" "$1" "48'/1'/0'/2'" "$2"); fp=$(printf '%s' "$r" | cut -c2-9)
  k=$(printf '%s' "$r" | sed -E 's/^\[[^]]*\]//')
  mk encode --xpub "$k" --origin-fingerprint "$fp" --origin-path "m/48'/1'/0'/2'" --policy-id-stub 00000000 2>/dev/null | grep -v '^$' | tr '\n' ' ' | sed 's/ $//'; }
exec mr export "$C" --network signet --keys "$(p "$A" "$SV")" "$(p "$B" 043587cf)" --height 250000 --time 1760000000
