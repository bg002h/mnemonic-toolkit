#!/bin/sh
# usage: up.sh COIN VERBYTES NET   (the md1's path coin type, the keys' version bytes, mr's --network)
# Runs `mr upgrade` of a two-seat md1 at m/48'/COIN'/0'/2' with keys of VERBYTES (public BIP-39
# test phrases; seeds.txt), stdout and stderr passed through. Used by the unit 4 and 8c RED rows.
cd "$(dirname "$0")" || exit 2
A="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
B="legal winner thank year wave sausage worth useful legal winner thank yellow"
COIN=$1; V=$2; NET=$3
M=$(md encode 'wsh(or_d(pk(@0/<0;1>/*),and_v(v:pkh(@1/<0;1>/*),after(1260000))))' --path "m/48'/$COIN'/0'/2'" 2>/dev/null | head -1)
KA=$(python3 -I b32.py "$A" "48'/$COIN'/0'/2'" "$V"); KB=$(python3 -I b32.py "$B" "48'/$COIN'/0'/2'" "$V")
D="wsh(or_d(pk(${KA}/<0;1>/*),and_v(v:pkh(${KB}/<0;1>/*),after(1260000))))"
H=250000; [ "$NET" = main ] && H=900000
exec mr upgrade "$M" --network "$NET" --role "alice=creator:A" --role "bob=heir:H" --seeds - --accept-12-word 1 --accept-12-word 2 --height $H --time 1760000000 --preset none --delta-blocks 52560 --no-other-cards-on-these-paths --witness-descriptor "$D" --keys "@0=$KA" "@1=$KB" < seeds.txt
