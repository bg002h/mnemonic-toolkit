#!/usr/bin/env bash
# RED/GREEN probe for design/FIX-tk-env-dash-stdin-order.md (bughunt-1119 bh-tk-I5).
# Usage: design/fix-tk-env-dash-stdin-order-red.sh <path-to-mnemonic-binary>
# Prints one line per row: "<row> rc=<exit> out=<first stdout line, 40 chars> err=<last stderr line>".
# Every row sets E='-' and names it through `@env:E` on a secret-bearing value; stdin carries a valid
# input. At the base the command reads stdin; after the fix it treats `-` as the literal value.
# Covers: rows R1-R10, S1-S9, A1-A3, C1-C6 of the FIX doc. Does NOT cover: the in-crate unit rows (section 6),
# a terminal stdin (every row pipes stdin), Windows.
# Public test phrase and signet cards only.
set -u
B=${1:?usage: $0 <mnemonic-binary>}
export E='-'
P='abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about'
export G="$P"
T=$(mktemp -d)
trap 'rm -r "$T"' EXIT

row() { # row <name> <stdin> args...
  local name=$1 input=$2; shift 2
  printf '%s\n' "$input" | "$B" "$@" >"$T/out" 2>"$T/err"
  local rc=$?
  printf '%s rc=%s out=%s err=%s\n' "$name" "$rc" "$(head -n1 "$T/out" | cut -c1-40)" "$(tail -n1 "$T/err")"
}

arow() { # arow <name> args... : stdin empty; reports whether stderr carries the argv advisory
  local name=$1; shift
  "$B" "$@" </dev/null >"$T/out" 2>"$T/err"
  local rc=$?
  printf '%s rc=%s argv-advisory=%s\n' "$name" "$rc" "$(grep -c 'secret material on argv' "$T/err")"
}

# Shares of $P (seed-xor --deterministic-from-master; ms-shares 2-of-3; slip39 with
# MNEMONIC_SLIP39_TEST_RNG=11...11, --group-threshold 1 --group 3,2).
X='throw enter benefit planet indoor happy cheap exhibit rule aisle then average'
M1=ms126mq5qqwxr08thfjy39dcehlnxfwf5fxhs8whrj2rsryask
M2=ms126mq5pq4fsz7v0tkwxe9d60npft4tltf0383kdy72303gps
L1='else garden academic acid crowd spew exchange thunder medal station galaxy cubic military cinema evaluate database saver petition unwrap racism'
L2='else garden academic agency away oasis decorate swimming quarter parking roster sister energy crucial amount cluster segment estate prune spirit'
# Bundle cards of $P from `bundle --template bip84 --network signet` (as fix-tk-stdout-write-errors-red.sh).
MS=ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f
K1=mk1qpg397pqqsq5gjlze9eutks2zvzrtp70pm6trteu3ssr0mjvzcsa5rf53k63zcmsnf3z6rfg8rw7dkzpn3glvvquvgpms6mzamgctgratycf
K2=mk1qpg397pp3c8mu0myvvm7myaups8nhpplea7jtz09ajyyw48xgqsz02ysk3l6fyxg6m66tmvhzh70a
D1=md1fkwsupqpqpm6j8pqqxqqks883w6pg25zpu3ssr0mjvzcsa5rf5sfn48qvmn2cutt
D2=md1fkwsupqtd4z93hpxnz95xjswxaumvyr8z37ccpccsrhz8ql03lv04y7x3v7nh0ze
D3=md1fkwsupq3rrxlke80qvpuacg07005jcne0v3pr4fejqyqn63y95qcngq49s9nrnck

# R: an @env: value of `-` must not read stdin (wrong source, exit 0 at the base).
row R1-convert "$P" convert --from phrase=@env:E --to entropy
row R2-derive-child "$P" derive-child --from phrase=@env:E --application hex --length 16 --index 0
row R3-bundle-slot "$P" bundle --template bip84 --network signet --slot @0.phrase=@env:E
row R4-verify-bundle-slot "$P" verify-bundle --allow-argv-secret --template bip84 --network signet \
  --ms1 "$MS" --mk1 "$K1" "$K2" --md1 "$D1" "$D2" "$D3" --slot @0.phrase=@env:E
row R5-seed-xor-split "$P" seed-xor split --from phrase=@env:E --shares 2 --deterministic-from-master
row R6-seed-xor-combine "$X" seed-xor combine --allow-argv-secret --shares 2 --share phrase=@env:E --share "phrase=$X"
row R7-ms-shares-split "$P" ms-shares split --from phrase=@env:E --threshold 2 --shares 3
row R8-ms-shares-combine "$M1" ms-shares combine --allow-argv-secret --share @env:E --share "$M2"
row R9-slip39-split "$P" slip39 split --from phrase=@env:E --group-threshold 1 --group 3,2
row R10-slip39-combine "$L1" slip39 combine --allow-argv-secret --share @env:E --share "$L2"

# S: the single-stdin guards must judge the value as written (wrong refusal at the base).
row S1-derive-child-pp-stdin "" derive-child --from phrase=@env:E --passphrase-stdin --application hex --length 16 --index 0
row S2-convert-pp-stdin "" convert --from phrase=@env:E --passphrase-stdin --to entropy
row S3-bundle-slot-pp-stdin "" bundle --template bip84 --network signet --slot @0.phrase=@env:E --passphrase-stdin
row S4-seed-xor-combine-two-env "$X" seed-xor combine --shares 2 --share phrase=@env:E --share phrase=@env:E
row S5-slip39-split-pp-stdin "" slip39 split --from phrase=@env:E --passphrase-stdin --group-threshold 1 --group 3,2
row S6-slip39-combine-two-env "$L1" slip39 combine --share @env:E --share @env:E
row S7-ms-shares-combine-two-env "$M1" ms-shares combine --share @env:E --share @env:E
row S8-bundle-literal-and-env-slot "$P" bundle --template wsh-sortedmulti --threshold 1 --network signet --slot @0.phrase=- --slot @1.phrase=@env:E
row S9-verify-bundle-slot-pp-stdin "" verify-bundle --allow-argv-secret --template bip84 --network signet \
  --ms1 "$MS" --mk1 "$K1" "$K2" --md1 "$D1" "$D2" "$D3" --slot @0.phrase=@env:E --passphrase-stdin

# A: the split argv advisory must judge the value as written (G holds the valid phrase).
arow A1-seed-xor-split-env-source seed-xor split --from phrase=@env:G --shares 2 --deterministic-from-master
arow A2-ms-shares-split-env-source ms-shares split --from phrase=@env:G --threshold 2 --shares 3
arow A3-slip39-split-env-source slip39 split --from phrase=@env:G --group-threshold 1 --group 3,2

# C: controls that pass at the base and after the fix.
row C1-convert-literal-dash "$P" convert --from phrase=- --to entropy
row C2-convert-env-phrase "" convert --from phrase=@env:G --to entropy
row C3-bundle-literal-dash "$P" bundle --template bip84 --network signet --slot @0.phrase=-
row C4-stdin-text-not-env-resolved '@env:G' convert --from phrase=- --to entropy
row C5-bundle-stdin-text-not-env-resolved '@env:G' bundle --template bip84 --network signet --slot @0.phrase=-
arow C6-ms-shares-split-inline-keeps-advisory ms-shares split --allow-argv-secret --from "phrase=$P" --threshold 2 --shares 3
