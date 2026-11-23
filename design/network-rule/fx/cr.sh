#!/bin/sh
# usage: cr.sh NET COIN VER   (mr's --network, the seat keys' path coin type and version bytes)
# Runs `mr create` of an owner-heir card with --seeds (public BIP-39 test phrases; seeds.txt) and
# the two seats' key records, stdout and stderr passed through. Used by the TX-R RED rows.
cd "$(dirname "$0")" || exit 2
NET=$1; COIN=$2; VER=$3
K=$(sh keys.sh "$COIN" "$VER"); K0=$(echo "$K" | sed -n 1p); K1=$(echo "$K" | sed -n 2p)
H=250000; [ "$NET" = main ] && H=900000
exec mr create --template-preset owner-heir --network "$NET" --role "alice=creator:A" --role "bob=heir:H" --seeds - --accept-12-word 1 --accept-12-word 2 --height $H --time 1760000000 --preset none --no-other-cards-on-these-paths --keys "@0=$K0" "@1=$K1" < seeds.txt
