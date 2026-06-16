#!/usr/bin/env bash
set -eu
cd /scratch/code/shibboleth/mnemonic-toolkit
B=./target/debug/mnemonic
S1="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
S2="legal winner thank year wave sausage worth useful legal winner thank yellow"
S3="letter advice cage absurd amount doctor acoustic avoid letter advice cage above"

fp () { printf '%s' "$1" | $B convert --from phrase=- --to fingerprint --template bip84 --network mainnet 2>/dev/null | sed -n 's/^fingerprint: //p'; }
xp () { printf '%s' "$1" | $B convert --from phrase=- --to xpub --template bip84 --account "$2" --network mainnet 2>/dev/null | sed -n 's/^xpub: //p'; }

declare -A FP
FP[1]=$(fp "$S1"); FP[2]=$(fp "$S2"); FP[3]=$(fp "$S3")

# origin-annotated key: [fp/84'/0'/N']xpub
key () { local seedvar="$1" acct="$2" fpv="$3"; echo "[$fpv/84'/0'/$acct']$(xp "$seedvar" "$acct")"; }

{
  echo "K0=$(key "$S1" 0 "${FP[1]}")"
  echo "K1=$(key "$S1" 1 "${FP[1]}")"
  echo "K2=$(key "$S1" 2 "${FP[1]}")"
  echo "K3=$(key "$S1" 3 "${FP[1]}")"
  echo "K4=$(key "$S2" 0 "${FP[2]}")"
  echo "K5=$(key "$S2" 1 "${FP[2]}")"
  echo "K6=$(key "$S2" 2 "${FP[2]}")"
  echo "K7=$(key "$S2" 3 "${FP[2]}")"
  echo "K8=$(key "$S3" 0 "${FP[3]}")"
  echo "K9=$(key "$S3" 1 "${FP[3]}")"
  echo "K10=$(key "$S3" 2 "${FP[3]}")"
} > .examples-build/keys.env

echo "=== 11 distinct keys (origin-annotated) ==="; cat .examples-build/keys.env | sed 's/\(=\[[^]]*\]xpub......\).*/\1.../'
echo
echo "=== timelock interpretations ==="
python3 - <<'PY'
import datetime
print("after(1000000)   -> ABSOLUTE block height 1,000,000")
t=1893456000
print(f"after({t}) -> ABSOLUTE time (unix) =", datetime.datetime.utcfromtimestamp(t).strftime('%Y-%m-%d %H:%M UTC'))
print("older(65535)     -> RELATIVE blocks: 65,535 blocks (~%.0f days at 144/day)" % (65535/144))
v=(0x400000 | 61594)
secs=61594*512
print(f"older({v}) -> RELATIVE time: bit22 set + 61594 units x 512s = {secs}s = {secs/86400:.2f} days")
PY