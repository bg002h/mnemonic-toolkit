#!/usr/bin/env bash
# RED/GREEN probe for design/FIX_sec_multipath_duplicate_refusal.md (security sweep unit
# multipath-duplicate-refusal; BIP-389 L50 "Duplicate NUMs within a tuple are not allowed.").
# Usage: design/fix-sec-multipath-duplicate-refusal-red.sh <path-to-mnemonic-binary>
# Prints one line per row: "<row> rc=<exit> out=<stdout bytes> err=<first stderr line>".
# Covers: the CLI rows R1-R10 and G1-G4 of the FIX (bundle concrete and @N forms, a 3-value and a
# leading-zero tuple, a second-key tuple, import-wallet --format descriptor, verify-bundle, the
# refusal order against the no-wildcard and residue refusals, gui-schema --classify-descriptor,
# and the unchanged guards).
# Does NOT cover: the other import-wallet formats (same parse_descriptor call, own prefix),
# export-wallet --from-import-json, md1 decode (restore,
# inspect), md-cli, or the out-of-scope surfaces listed in the FIX section 7.
# Signet public keys from committed test fixtures only; no seed is used.
set -u
B=${1:?usage: $0 <mnemonic-binary>}
T=$(mktemp -d)
trap 'rm -r "$T"' EXIT

T84=tpubDC8msFGeGuwnKG9Upg7DM2b4DaRqg3CUZa5g8v2SRQ6K4NSkxUgd7HsL2XVWbVm39yBA4LAxysQAm397zwQSQoQgewGiYZqrA9DsP4zbQ1M
TA=tpubDFH9dgzveyD8zTbPUFuLrGmCydNvxehyNdUXKJAQN8x4aZ4j6UZqGfnqFrD4NqyaTVGKbvEW54tsvPTK2UoSbCC1PJY8iCNiwTL3RWZEheQ
TC=tpubDEwqCvJxKwKWX9xvRe48uofWJn1Y89Jn8UeH1Efrjb1UEVjUDy3URYTiqWaVCW7WdvHrL8XrSihHEhTwv5H3VDJoakjuCHiAnr6xcF2Xm4s

row() { # row <name> <args...>
  local name=$1 rc
  shift
  "$B" "$@" >"$T/out" 2>"$T/err"
  rc=$?
  printf '%s rc=%s out=%s err=%s\n' "$name" "$rc" "$(wc -c <"$T/out")" "$(head -n1 "$T/err")"
}

single() { printf 'wpkh([73c5da0a/84h/1h/0h]%s%s)' "$T84" "$1"; }

row R1 bundle --network signet --descriptor "$(single '/<0;0>/*')" --slot "@0.xpub=$T84"
row R2 bundle --network signet --descriptor 'wpkh([73c5da0a/84h/1h/0h]@0/<1;1>/*)' --slot "@0.xpub=$T84"
row R3 bundle --network signet --descriptor "$(single '/<0;1;0>/*')" --slot "@0.xpub=$T84"
row R4 bundle --network signet --descriptor "$(single '/<1;01>/*')" --slot "@0.xpub=$T84"
row R5 bundle --network signet --descriptor "wsh(sortedmulti(2,[73c5da0a/48h/1h/0h/2h]$TA/<0;1>/*,[28645006/48h/1h/0h/2h]$TC/<2;2>/*))" --slot "@0.xpub=$TA" --slot "@1.xpub=$TC"
printf '%s\n' "$(single '/<0;0>/*')" >"$T/w.txt"
row R6 import-wallet --network signet --format descriptor --blob "$T/w.txt"
row R7 verify-bundle --network signet --mk1 '' --md1 '' --descriptor "$(single '/<0;0>/*')" --slot "@0.xpub=$T84"
row R8 bundle --network signet --descriptor "$(single '/<0;0>')" --slot "@0.xpub=$T84"
row R9 bundle --network signet --descriptor "$(single '/<0;0>/0/*')" --slot "@0.xpub=$T84"
row R10 gui-schema --classify-descriptor 'wpkh(@0/<0;0>/*)'
row G1 bundle --network signet --descriptor "$(single '/<0;1>/*')" --slot "@0.xpub=$T84"
row G2 bundle --network signet --descriptor "$(single '/<0;1;2>/*')" --slot "@0.xpub=$T84"
row G3 bundle --network signet --descriptor "$(single "/<0;0'>/*")" --slot "@0.xpub=$T84"
row G4 bundle --network signet --descriptor "$(single '/<0;x;0>/*')" --slot "@0.xpub=$T84"
