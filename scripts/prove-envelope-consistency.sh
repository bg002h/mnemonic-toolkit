#!/usr/bin/env bash
# Proof that `bundle --import-json` and `export-wallet --from-import-json`
# refuse inconsistent import-wallet envelopes (FIX-V7 2.5, E1-E3) and still
# accept honest ones. Signet keys only.
#
# Usage: scripts/prove-envelope-consistency.sh
#   MNEMONIC_BIN=<path>  binary under test (default: cargo build --locked, target/debug/mnemonic)
#   PRE_BIN=<path>       optional pre-change binary: honest outputs must be byte-identical to it
#   WORK=<dir>           scratch dir (default: a new dir under /scratch/cache; never /tmp)
# Prints one PASS/FAIL line per case and a total; exits non-zero on any FAIL.
#
# Does not cover: the envelope fields the ruling leaves unread (account,
# multisig, master_fingerprint), or an envelope re-encoded consistently by a
# writer (no MAC).
set -u
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
export PATH="$HOME/.cargo/bin:$PATH"
if [ -z "${MNEMONIC_BIN:-}" ]; then
  (cd "$ROOT" && cargo build --locked -p mnemonic-toolkit >&2) || { echo "build failed" >&2; exit 2; }
  MNEMONIC_BIN="$ROOT/target/debug/mnemonic"
fi
BIN="$MNEMONIC_BIN"
WORK="${WORK:-$(mktemp -d -p /scratch/cache prove-envc.XXXXXX)}"
mkdir -p "$WORK"
FX="$ROOT/crates/mnemonic-toolkit/tests/fixtures/wallet_import"

K0="[73c5da0a/48'/1'/0'/2']tpubDFH9dgzveyD8zTbPUFuLrGmCydNvxehyNdUXKJAQN8x4aZ4j6UZqGfnqFrD4NqyaTVGKbvEW54tsvPTK2UoSbCC1PJY8iCNiwTL3RWZEheQ/<0;1>/*"
K1="[3f635a63/48'/1'/0'/2']tpubDFPtPArj4GzBEFHohegg1Xatrc1Fi9oSox5LzuSRX91miwQxuUrEpBxpvDRsmZYJKYFhgdK3UStsjC8JKXfUbMinjFqiEM4uNwzVaCaHpys/<0;1>/*"
K2="[b8688df1/48'/1'/0'/2']tpubDEfobrrtptRTbKf4gysDhoabneABDTAcdj3Vbn4XwPsLE2pmqpizSPRG6zHsbAMuiSgWmWPsYCLHTKTPpyrGJ5rAoTpKoQNZcxodiPf2tSJ/<0;1>/*"
K3="[28645006/48'/1'/0'/2']tpubDEwqCvJxKwKWX9xvRe48uofWJn1Y89Jn8UeH1Efrjb1UEVjUDy3URYTiqWaVCW7WdvHrL8XrSihHEhTwv5H3VDJoakjuCHiAnr6xcF2Xm4s/<0;1>/*"
P0="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
MS1_ZOO="ms10entrsqrlllllllllllllllllllllllllsnrx3qek9l5q60"
TAIL="so the envelope is not as import-wallet wrote it; re-create it with import-wallet from the same blob"
FORMATS="descriptor bitcoin-core bip388 sparrow coldcard electrum"

# import <template> <seed0:0|1> <outfile>: honest signet envelope from import-wallet.
import() {
  local d="$1" seed="$2" out="$3"
  d="${d//@0/$K0}"; d="${d//@1/$K1}"; d="${d//@2/$K2}"; d="${d//@3/$K3}"
  local extra=()
  [ "$seed" = 1 ] && extra=(--slot "@0.phrase=@env:ENVC_P0")
  printf '%s\n' "$d" | ENVC_P0="$P0" "$BIN" import-wallet --format descriptor --blob - \
    --network signet --json "${extra[@]}" >"$out" 2>/dev/null || { echo "setup failed: import $1" >&2; exit 2; }
}
# edit <in> <out> <python statements over b = v[0]["bundle"]> (v = whole array)
edit() {
  python3 -I - "$1" "$2" "$3" <<'PY'
import json, sys
v = json.load(open(sys.argv[1])); b = v[0]["bundle"]
exec(sys.argv[3])
json.dump(v, open(sys.argv[2], "w"))
PY
}
field() { python3 -I -c 'import json,sys; print(json.dumps(json.load(open(sys.argv[1]))[0]["bundle"][sys.argv[2]]))' "$1" "$2"; }

PASS=0; FAIL=0
report() { # ok name detail
  if [ "$1" = 1 ]; then PASS=$((PASS+1)); echo "PASS  $2"; else FAIL=$((FAIL+1)); echo "FAIL  $2  -- $3"; fi
}

# run_refused <name> <label> <env> <network> <want-stderr-line> <cmd: bundle|export>
refused() {
  local name="$1" env="$2" net="$3" want="$4" cmd="$5" bad="" fmt rc
  if [ "$cmd" = bundle ]; then
    "$BIN" bundle --network "$net" --import-json "$env" >"$WORK/o" 2>"$WORK/e"; rc=$?
    [ $rc = 1 ] || bad="exit $rc"
    [ -s "$WORK/o" ] && bad="$bad stdout not empty"
    grep -qxF "$want" "$WORK/e" || bad="$bad missing line"
  else
    for fmt in $FORMATS; do
      "$BIN" export-wallet --from-import-json "$env" --format "$fmt" >"$WORK/o" 2>"$WORK/e"; rc=$?
      [ $rc = 1 ] || bad="$bad [$fmt exit $rc]"
      [ -s "$WORK/o" ] && bad="$bad [$fmt stdout not empty]"
      grep -qxF "$want" "$WORK/e" || bad="$bad [$fmt missing line]"
    done
  fi
  if [ -z "$bad" ]; then report 1 "$name ($cmd refuses)"; else report 0 "$name ($cmd refuses)" "$bad"; fi
}
# both_refused <name> <env> <net> <E-text after "<L>: ">; an E3 text is bundle-only
both_refused() {
  refused "$1" "$2" "$3" "error: --import-json: $4" bundle
  refused "$1" "$2" "$3" "error: --from-import-json: $4" export
}

# accepted <name> <env> <net>: both commands exit 0 with stdout; byte-identical to PRE_BIN if given.
accepted() {
  local name="$1" env="$2" net="$3" fmts="${4:-$FORMATS}" bad="" fmt rc
  "$BIN" bundle --network "$net" --import-json "$env" >"$WORK/o" 2>"$WORK/e"; rc=$?
  [ $rc = 0 ] && [ -s "$WORK/o" ] || bad="bundle exit $rc"
  if [ -n "${PRE_BIN:-}" ]; then
    "$PRE_BIN" bundle --network "$net" --import-json "$env" >"$WORK/po" 2>/dev/null
    cmp -s "$WORK/o" "$WORK/po" || bad="$bad bundle output differs from PRE_BIN"
  fi
  for fmt in $fmts; do
    "$BIN" export-wallet --from-import-json "$env" --format "$fmt" >"$WORK/o" 2>"$WORK/e"; rc=$?
    [ $rc = 0 ] && [ -s "$WORK/o" ] || bad="$bad [$fmt exit $rc]"
    if [ -n "${PRE_BIN:-}" ]; then
      "$PRE_BIN" export-wallet --from-import-json "$env" --format "$fmt" >"$WORK/po" 2>/dev/null
      cmp -s "$WORK/o" "$WORK/po" || bad="$bad [$fmt output differs from PRE_BIN]"
    fi
  done
  if [ -z "$bad" ]; then report 1 "$name (both accept)"; else report 0 "$name (both accept)" "$bad"; fi
}

E1="envelope.bundle.md1 does not match envelope.bundle.descriptor, $TAIL"
e2key() { echo "cosigner $1 of envelope.bundle.mk1 (its key, fingerprint or origin path) does not match key @$1 of envelope.bundle.descriptor, $TAIL"; }

PLAIN='wsh(sortedmulti(2,@0,@1,@2))'
import "$PLAIN" 0 "$WORK/honest.json"
import "$PLAIN" 1 "$WORK/honest-seed.json"
import 'wsh(sortedmulti(3,@0,@1,@2))' 0 "$WORK/three.json"
import 'wsh(sortedmulti(1,@0,@1,@2))' 0 "$WORK/one.json"
import 'wsh(sortedmulti(2,@0,@1,@3))' 0 "$WORK/other.json"

# Honest envelopes
accepted "honest plain 2-of-3" "$WORK/honest.json" signet
accepted "honest 2-of-3 with seed card" "$WORK/honest-seed.json" signet
accepted "v0.27.0 envelope as released" "$FX/envelope_v0_27_0_as_released.json" mainnet "descriptor bitcoin-core bip388"  # legacy bare P2SH: no sparrow/coldcard/electrum template

# Tampered variants
D3=$(field "$WORK/three.json" descriptor); D1=$(field "$WORK/one.json" descriptor)
edit "$WORK/honest.json" "$WORK/t-3of3.json" "b['descriptor'] = json.loads('''$D3''')"
both_refused "descriptor edited to 3-of-3, md1 kept" "$WORK/t-3of3.json" signet "$E1"
edit "$WORK/honest.json" "$WORK/t-1of3.json" "b['descriptor'] = json.loads('''$D1''')"
both_refused "descriptor edited to 1-of-3, md1 kept" "$WORK/t-1of3.json" signet "$E1"

M=$(field "$WORK/other.json" md1)
edit "$WORK/honest.json" "$WORK/t-md1.json" "b['md1'] = json.loads('''$M''')"
both_refused "md1 of another wallet" "$WORK/t-md1.json" signet "$E1"

C=$(field "$WORK/other.json" mk1)
edit "$WORK/honest.json" "$WORK/t-swap.json" "b['mk1'][2] = json.loads('''$C''')[2]"
both_refused "mk1[2] swapped from another wallet" "$WORK/t-swap.json" signet "$(e2key 2)"

edit "$WORK/honest.json" "$WORK/t-path.json" "b['origin_path'] = \"m/48'/1'/7'/2'\""
both_refused "origin path edited" "$WORK/t-path.json" signet "$(e2key 0)"

# Card-only origin-path mismatch: mk1[0] is a card of the same xpub and fingerprint
# whose origin path is m/48'/1'/7'/2'; descriptor, md1 and origin_path stay at 0'.
K0P7="${K0/48\'\/1\'\/0\'\/2\'/48\'/1\'/7\'/2\'}"
[ "$K0P7" != "$K0" ] || { echo "setup failed: K0P7" >&2; exit 2; }
import "${PLAIN/@0/$K0P7}" 0 "$WORK/cardpath.json"
CP=$(field "$WORK/cardpath.json" mk1)
edit "$WORK/honest.json" "$WORK/t-cardpath.json" "b['mk1'][0] = json.loads('''$CP''')[0]"
both_refused "mk1[0] card path only differs" "$WORK/t-cardpath.json" signet "$(e2key 0)"

edit "$WORK/honest.json" "$WORK/t-drop.json" "b['mk1'] = b['mk1'][:2]; b['ms1'] = b['ms1'][:2]"
both_refused "one mk1 card dropped" "$WORK/t-drop.json" signet \
  "envelope.bundle.mk1 has 2 cosigners but envelope.bundle.descriptor has 3 keys, $TAIL"

edit "$WORK/honest-seed.json" "$WORK/t-ms1.json" "b['ms1'][0] = '$MS1_ZOO'"
refused "foreign ms1 for cosigner 0" "$WORK/t-ms1.json" signet \
  "error: --import-json: envelope.bundle.ms1[0] is not the seed of cosigner 0 of envelope.bundle.mk1, $TAIL" bundle
# export-wallet never reads ms1 (E3 is bundle-only): the same file must still export.
accepted_export_only() {
  local bad="" fmt rc
  for fmt in $FORMATS; do
    "$BIN" export-wallet --from-import-json "$1" --format "$fmt" >"$WORK/o" 2>/dev/null; rc=$?
    [ $rc = 0 ] && [ -s "$WORK/o" ] || bad="$bad [$fmt exit $rc]"
  done
  if [ -z "$bad" ]; then report 1 "$2 (export accepts: ms1 is not read)"; else report 0 "$2 (export accepts: ms1 is not read)" "$bad"; fi
}
accepted_export_only "$WORK/t-ms1.json" "foreign ms1 for cosigner 0"

D="wpkh([73c5da0a/84'/1'/0']tpubDC8msFGeGuwnKG9Upg7DM2b4DaRqg3CUZa5g8v2SRQ6K4NSkxUgd7HsL2XVWbVm39yBA4LAxysQAm397zwQSQoQgewGiYZqrA9DsP4zbQ1M/<0;1>/*)"
"$BIN" bundle --network signet --descriptor "$D" --md1-form template --json >"$WORK/tpl.json" 2>/dev/null || { echo "setup failed: template bundle" >&2; exit 2; }
python3 -I -c 'import json,sys; json.dump([{"schema_version":"1","source_format":"descriptor","bundle":json.load(open(sys.argv[1]))}], open(sys.argv[2],"w"))' "$WORK/tpl.json" "$WORK/t-wrapped.json"
both_refused "hand-wrapped bundle --md1-form template --json" "$WORK/t-wrapped.json" signet "$E1"

echo "TOTAL: $PASS passed, $FAIL failed ($((PASS+FAIL)) cases) [bin: $BIN]"
[ "$FAIL" = 0 ]
