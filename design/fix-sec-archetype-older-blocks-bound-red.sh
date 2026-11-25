#!/usr/bin/env bash
# RED/GREEN rows for design/FIX_sec_archetype_older_blocks_bound.md (security
# sweep unit archetype-older-blocks-bound). Run against a `mnemonic` binary:
#   design/fix-sec-archetype-older-blocks-bound-red.sh <mnemonic-binary>
# One line per row: rc, the older(...) operands in the emitted descriptor
# (stdout), then each stderr diagnostic line (the lines starting "  [").
# Rows ending in "-json" print the first --json diagnostic's kind, node_path
# and flag instead. Keys are the public BIP-32 test-vector xpubs the
# descriptor-builder fixtures use (tests/cli_build_descriptor.rs K1..K5);
# they guard no funds.
# NOT covered: the --spec path (unchanged by the FIX), the manual and SPEC
# prose (read the diff), the unit tests inside archetype.rs (nextest).
set -u
B=${1:?usage: $0 <mnemonic-binary>}
T=$(mktemp -d "${TMPDIR:-/tmp}/older-bound-red.XXXXXX")
trap 'rm -f "$T"/*; rmdir "$T"' EXIT
K1='[11111111/48h/0h/0h/2h]xpub661MyMwAqRbcEZVB4dScxMAdx6d4nFc9nvyvH3v4gJL378CSRZiYmhRoP7mBy6gSPSCYk6SzXPTf3ND1cZAceL7SfJ1Z3GC8vBgp2epUt13'
K2='[22222222/48h/0h/0h/2h]xpub661MyMwAqRbcFtXgS5sYJABqqG9YLmC4Q1Rdap9gSE8NqtwybGhePY2gZ29ESFjqJoCu1Rupje8YtGqsefD265TMg7usUDFdp6W1EGMcet8'
K3='[33333333/48h/0h/0h/2h]xpub661MyMwAqRbcFW31YEwpkMuc5THy2PSt5bDMsktWQcFF8syAmRUapSCGu8ED9W6oDMSgv6Zz8idoc4a6mr8BDzTJY47LJhkJ8UB7WEGuduB'
K4='[44444444/48h/0h/0h/2h]xpub661MyMwAqRbcGczjuMoRm6dXaLDEhW1u34gKenbeYqAix21mdUKJyuyu5F1rzYGVxyL6tmgBUAEPrEz92mBXjByMRiJdba9wpnN37RLLAXa'
K5='[55555555/48h/0h/0h/2h]xpub68Gmy5EdvgibQVfPdqkBBCHxA5htiqg55crXYuXoQRKfDBFA1WEjWgP6LHhwBZeNK1VTsfTFUHCdrfp1bgwQ9xv5ski8PX9rL2dZXvgGDnw'
H=926a54995ca48600920a19bf7bc502ca5f2f7d07e6f804c4f00ebf0325084dbc

HG=(build-descriptor --archetype hashlock-gated --key "$K1" --hash "$H" --recovery-key "$K2")
STI=(build-descriptor --archetype simple-timelocked-inheritance --key "$K1" --recovery-key "$K2")
KOFN=(build-descriptor --archetype kofn-recovery --key "$K1" --key "$K2" --key "$K3" --threshold 2 --recovery-key "$K4")
TR=(build-descriptor --archetype tiered-recovery --key "$K1" --key "$K2" --threshold 2 --recovery-key "$K3" --recovery-key "$K4" --recovery-threshold 2)
DM=(build-descriptor --archetype decaying-multisig --key "$K1" --key "$K2" --threshold 2 --recovery-key "$K3" --recovery-key "$K4" --recovery-threshold 2 --final-key "$K5" --after 4000000)

row() { # row <name> <args...>
  local name=$1; shift
  "$B" "$@" </dev/null >"$T/out" 2>"$T/err"; local rc=$?
  local ops
  ops=$(grep -o 'older([0-9]*)' "$T/out" | sort -u | tr '\n' ' ')
  printf '%s rc=%s older=[%s]\n' "$name" "$rc" "${ops% }"
  grep '^  \[' "$T/err" | sed "s/^/    $name /"
}

jrow() { # jrow <name> <args...> (adds --json)
  local name=$1; shift
  "$B" "$@" --json </dev/null >"$T/out" 2>"$T/err"; local rc=$?
  local d
  d=$(python3 -I -c 'import json,sys
v=json.load(open(sys.argv[1]))
ds=v.get("diagnostics") or []
print("n=%d" % len(ds) + ("" if not ds else " kind=%s node_path=%s flag=%s" % (ds[0].get("kind"), ds[0].get("node_path"), ds[0].get("flag","<absent>"))))' "$T/out" 2>/dev/null)
  printf '%s rc=%s %s\n' "$name" "$rc" "${d:-<no json>}"
}

# The 512-second-unit window 0x400001..=0x40FFFF, one row per archetype.
row R1-hg-4200000 "${HG[@]}" --older 4200000
row R2-sti-4194305 "${STI[@]}" --older 4194305
row R3-kofn-4259839 "${KOFN[@]}" --older 4259839
row R4-tr-4200000 "${TR[@]}" --older 4200000
row R5-dm-both-512s "${DM[@]}" --older 4194305 --recovery-older 4194306
row R6-dm-cross-unit "${DM[@]}" --older 145 --recovery-older 4194305
jrow R7-hg-4200000-json "${HG[@]}" --older 4200000
# Outside 1..=65535 and outside the window (the gate refuses these today).
row R8-kofn-0 "${KOFN[@]}" --older 0
row R9-kofn-65536 "${KOFN[@]}" --older 65536
row R10-kofn-4194304 "${KOFN[@]}" --older 4194304
row R11-kofn-105120 "${KOFN[@]}" --older 105120
jrow R12-kofn-105120-json "${KOFN[@]}" --older 105120
row R13-dm-recovery-70000 "${DM[@]}" --older 1000 --recovery-older 70000
# Controls (unchanged by the FIX).
row C1-sti-65535 "${STI[@]}" --older 65535
row C2-hg-1 "${HG[@]}" --older 1
row C3-dm-1000-2000 "${DM[@]}" --older 1000 --recovery-older 2000
row C4-dm-2000-1000 "${DM[@]}" --older 2000 --recovery-older 1000
row C5-kofn-inapplicable-recovery-older "${KOFN[@]}" --older 144 --recovery-older 4200000
