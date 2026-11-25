#!/bin/sh
# usage: crs.sh NET SCOIN SVER [FLAG...]   (mr's --network; a sibling single-sig card's path coin
# type and its key's version bytes; any further arguments are passed to mr create)
# Runs `mr create` of an owner-heir card with --seeds (public BIP-39 test phrases; seeds.txt), the
# two seats' mainnet key records, and one --sibling md1 card (wpkh at m/84'/SCOIN'/0') with its
# --sibling-keys record (seed A at that path, version SVER). stdout and stderr passed through.
# Used by the mr create coin-type RED row (unit 8c).
cd "$(dirname "$0")" || exit 2
A="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
NET=$1; SCOIN=$2; SVER=$3; shift 3
K=$(sh keys.sh 0 0488b21e); K0=$(echo "$K" | sed -n 1p); K1=$(echo "$K" | sed -n 2p)
S=$(md encode 'wpkh(@0/<0;1>/*)' --path "m/84'/$SCOIN'/0'" 2>/dev/null | head -1)
SK=$(python3 -I b32.py "$A" "84'/$SCOIN'/0'" "$SVER")
H=250000; [ "$NET" = main ] && H=900000
exec mr create --template-preset owner-heir --network "$NET" --role "alice=creator:A" --role "bob=heir:H" --seeds - --accept-12-word 1 --accept-12-word 2 --height $H --time 1760000000 --preset none --sibling "$S" --sibling-keys "$SK" --keys "@0=$K0" "@1=$K1" "$@" < seeds.txt
