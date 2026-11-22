#!/usr/bin/env bash
# RED/GREEN probe for design/FIX-tk-stdout-write-errors.md.
# Usage: design/fix-tk-stdout-write-errors-red.sh <path-to-mnemonic-binary>
# Prints one line per row: "<row> rc=<exit> stderr=<last stderr line or empty>".
# Covers: the rows R1-R18 and R23 of the FIX doc (stdout to /dev/full, a regular file under `ulimit -f 0`, a closed pipe,
# a read-only stdout, --help/--version). Does NOT cover: --out/--json-out file
# writes, stderr write errors, Windows, or a writer that fails only at flush
# (the in-crate unit test owns that row).
# Signet / public test phrase only.
set -u
B=${1:?usage: $0 <mnemonic-binary>}
P='abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about'
T=$(mktemp -d)
trap 'rm -r "$T"' EXIT

row() { # row <name> <stdin> <redirect-kind> args...
  local name=$1 input=$2 kind=$3; shift 3
  local rc
  case $kind in
    full) printf '%s\n' "$input" | "$B" "$@" >/dev/full 2>"$T/err"; rc=$? ;;
    pipe) rc=$( { printf '%s\n' "$input" | "$B" "$@" 2>"$T/err"; echo "rc=$?" >"$T/rc"; } | head -c 1 >/dev/null; sleep 0.2; cat "$T/rc" ); rc=${rc#rc=} ;;
    ro)   printf '%s\n' "$input" | "$B" "$@" 1</dev/null 2>"$T/err"; rc=$? ;;
  esac
  printf '%s rc=%s stderr=%s\n' "$name" "$rc" "$(tail -n1 "$T/err")"
}

row R1-derive-child-full "$P" full derive-child --from phrase=- --application hex --length 16 --index 0
row R2-convert-text-full "$P" full convert --from phrase=- --to entropy
row R3-convert-json-full "$P" full convert --from phrase=- --to entropy --json
row R4-bundle-full "$P" full bundle --template bip84 --network signet --slot @0.phrase=-
row R5-gui-schema-full "" full gui-schema
row R6-addresses-full "$P" full addresses --from phrase=- --address-type p2wpkh --network signet --count 3
row R7-addresses-closed-pipe "$P" pipe addresses --from phrase=- --address-type p2wpkh --network signet --count 2000
row R8-gui-schema-closed-pipe "" pipe gui-schema
row R9-help-full "" full --help
row R10-version-full "" full --version
row R11-gui-schema-read-only "" ro gui-schema
row R12-convert-read-only "$P" ro convert --from phrase=- --to entropy

# Codes other than 0 (bundle cards from `bundle --template bip84 --network signet`
# on the public phrase above; one character flipped for the repair rows).
MS=ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f
K1=mk1qpg397pqqsq5gjlze9eutks2zvzrtp70pm6trteu3ssr0mjvzcsa5rf53k63zcmsnf3z6rfg8rw7dkzpn3glvvquvgpms6mzamgctgratycf
K1BAD=mk1qpg397pqqsq5gjlze9eutks2zvzrtp70pm6trteu3ssr0mjvzcsa5rf53k63zcmsnf3z6rfg8rw7dkzpn3glvvquvgpms6mzamgctgratycq
K2=mk1qpg397pp3c8mu0myvvm7myaups8nhpplea7jtz09ajyyw48xgqsz02ysk3l6fyxg6m66tmvhzh70a
D1=md1fkwsupqpqpm6j8pqqxqqks883w6pg25zpu3ssr0mjvzcsa5rf5sfn48qvmn2cutt
D2=md1fkwsupqtd4z93hpxnz95xjswxaumvyr8z37ccpccsrhz8ql03lv04y7x3v7nh0ze
D3=md1fkwsupq3rrxlke80qvpuacg07005jcne0v3pr4fejqyqn63y95qcngq49s9nrnck
Z='zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong'
VB=(verify-bundle --allow-argv-secret --template bip84 --network signet --ms1 "$MS" --mk1 "$K1" "$K2" --md1 "$D1" "$D2" "$D3" --slot @0.phrase=-)
row R13-verify-bundle-ok-full "$P" full "${VB[@]}"
row R14-verify-bundle-mismatch4-full "$Z" full "${VB[@]}"
row R15-repair-applied5-full "" full repair --mk1 "$K1BAD" "$K2"
row R16-repair-unverified4-full "${MS%f}q" full repair --ms1 -
row R23-inspect-partial4-full "" full inspect --md1 md1yppqqxp3cg2x3r70ckk4kjaf
# The consultant's case: a regular file, RLIMIT_FSIZE 0, SIGXFSZ ignored. Stderr goes to a pipe
# (a regular file would hit the same limit and lose the contract line).
{ printf '%s\n' "$P" | sh -c 'trap "" XFSZ; ulimit -f 0; exec "$0" "$@"' "$B" \
  derive-child --from phrase=- --application hex --length 16 --index 0 >"$T/out"; echo $? >"$T/rc"; } 2>&1 | cat >"$T/err"
printf '%s rc=%s bytes=%s stderr=%s\n' R17-derive-child-fsize0 "$(cat "$T/rc")" "$(wc -c <"$T/out")" "$(tail -n1 "$T/err")"
# Guard: a stdout closed outright (std reopens fd 1 on /dev/null at start-up).
"$B" gui-schema >&- 2>"$T/err"
printf '%s rc=%s stderr=%s\n' R18-gui-schema-closed-fd "$?" "$(tail -n1 "$T/err")"
