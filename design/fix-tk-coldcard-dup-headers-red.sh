#!/usr/bin/env bash
# RED/GREEN probe for design/FIX-tk-coldcard-multisig-duplicate-headers.md (bughunt-1119 bh-tk-I1).
# Usage: design/fix-tk-coldcard-dup-headers-red.sh <path-to-mnemonic-binary>
# Run from the repository root (reads two committed fixtures by relative path).
# Prints one line per row:
#   "<row> rc=<exit> out=<stdout bytes> quorum=<sortedmulti(K> wrap=<wsh(|sh(wsh(> fp0=<first origin fp> paths=<origin paths, comma-joined> name=<name> err=<last stderr line>"
# Covers: rows R1-R18 of the FIX doc (coldcard-multisig text via --format coldcard-multisig,
# auto-sniff and the jade multisig_file wrapper; repeated Name/Policy/Format/XFP headers in both
# orders; a case-variant repeated key; the per-cosigner Derivation shape; the original fixture; a lowercase derivation key; the
# Coldcard render_export path-change shape).
# Does NOT cover: library callers of parse_text other than the CLI, the bsms/sparrow/specter
# parsers, Windows, or the cosigner-count predicate (bug-hunt I1 tail, separate line).
# Public xpubs from committed test fixtures only; no seed is used.
set -u
B=${1:?usage: $0 <mnemonic-binary>}
FX=crates/mnemonic-toolkit/tests/export_wallet/coldcard_multisig_2of3_wsh.txt
JADE=crates/mnemonic-toolkit/tests/fixtures/wallet_import/jade-multisig-2of3-p2wsh.json
T=$(mktemp -d)
trap 'rm -r "$T"' EXIT

X1=xpub6Buxw9MmbkJr4iAw8SACNci2hQNuPCMwt9P7HkK62ZQAW9UcJaQ2bc6ARD892TToQQ9Rp6AHujHxBLXqAsvn5fRnLfnhKSRfz8qtaoyKUYx
X2=xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6
X3=xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX

report() { # report <name> <rc>
  local name=$1 rc=$2
  local bytes quorum wrap fp0 nm paths
  bytes=$(wc -c <"$T/out")
  quorum=$(grep -o 'sortedmulti([0-9]*' "$T/out" | head -n1)
  wrap=$(grep -o '"descriptor": "[a-z]*([a-z]*(' "$T/out" | head -n1 | sed 's/"descriptor": "//')
  fp0=$(grep -o '\[[0-9a-f]\{8\}' "$T/out" | head -n1 | tr -d '[')
  nm=$(grep -o '"name": "[^"]*"' "$T/out" | head -n1 | sed 's/"name": //')
  paths=$(grep -o '"descriptor": "[^"]*"' "$T/out" | head -n1 | grep -o "\[[0-9a-f]*/[^]]*\]" | sed 's/^\[[0-9a-f]*\///; s/\]$//' | paste -sd, -)
  printf '%s rc=%s out=%s quorum=%s wrap=%s fp0=%s paths=%s name=%s err=%s\n' \
    "$name" "$rc" "$bytes" "$quorum" "$wrap" "$fp0" "$paths" "$nm" "$(tail -n1 "$T/err")"
}

text() { # text <name> <blob> [--autosniff]
  local name=$1 blob=$2 rc
  if [ "${3:-}" = --autosniff ]; then
    printf '%s' "$blob" | "$B" import-wallet --blob - --json >"$T/out" 2>"$T/err"; rc=$?
  else
    printf '%s' "$blob" | "$B" import-wallet --format coldcard-multisig --blob - --json >"$T/out" 2>"$T/err"; rc=$?
  fi
  report "$name" "$rc"
}

jade() { # jade <name> <inner-text>
  local name=$1 inner=$2 rc
  jq --arg f "$inner" '.multisig_file = $f' "$JADE" >"$T/jade.json"
  "$B" import-wallet --format jade --blob "$T/jade.json" --json >"$T/out" 2>"$T/err"; rc=$?
  report "$name" "$rc"
}

BASE=$(cat "$FX"; printf 'x')
BASE=${BASE%x}
# Per-cosigner shape whose first cosigner is a bare xpub: its origin fingerprint comes from the
# top-level XFP: header (coldcard_multisig.rs L272-L278 and L381 at the base).
bare() { # bare <xfp-lines>
  printf 'Name: bare-xfp\nPolicy: 2 of 3\nFormat: P2WSH\n%sDerivation: m/48'"'"'/0'"'"'/0'"'"'/2'"'"'\n%s\nDerivation: m/48'"'"'/0'"'"'/1'"'"'/2'"'"'\n28645006: %s\nDerivation: m/48'"'"'/0'"'"'/2'"'"'/2'"'"'\nB8688DF1: %s\n' "$1" "$X1" "$X2" "$X3"
}
# Two Derivation: lines with no cosigner line between them (third and fourth lines).
TWODER=$(sed -n 1,3p "$FX"; printf 'Derivation: m/48'"'"'/0'"'"'/9'"'"'/2'"'"'\n'; sed -n '4,$p' "$FX")
PERCOS=$(printf 'Name: per-cosigner\nPolicy: 2-of-3\nFormat: P2WSH\nDerivation: m/48'"'"'/0'"'"'/0'"'"'/2'"'"'\n5436D724: %s\nDerivation: m/48'"'"'/0'"'"'/1'"'"'/2'"'"'\n28645006: %s\nDerivation: m/48'"'"'/0'"'"'/2'"'"'/2'"'"'\nB8688DF1: %s\n' "$X1" "$X2" "$X3")

text R1-policy-conflict-appended            "${BASE}Policy: 1 of 3
"
text R2-policy-conflict-prepended           "Policy: 1 of 3
${BASE}"
text R3-policy-conflict-autosniff           "${BASE}Policy: 1 of 3
" --autosniff
jade R4-policy-conflict-jade                "${BASE}Policy: 1 of 3
"
text R5-format-conflict-appended            "${BASE}Format: P2SH-P2WSH
"
text R6-format-conflict-prepended           "Format: P2SH-P2WSH
${BASE}"
text R7-xfp-conflict                        "$(bare 'XFP: 0000AAAA
XFP: 0000BBBB
')"
text R8-name-conflict                       "${BASE}Name: other-wallet
"
text R9-policy-identical-repeat             "${BASE}Policy: 2 of 3
"
text R10-policy-repeat-other-spelling       "${BASE}Policy: 2-of-3
"
text R11-policy-lowercase-key-repeat        "${BASE}policy: 1 of 3
"
text R12-control-original-fixture           "${BASE}"
text R13-control-per-cosigner-derivation    "${PERCOS}"
text R14-control-xfp-single                 "$(bare 'XFP: 0000AAAA
')"
text R15-derivation-twice-before-cosigners  "${TWODER}
"
TWODERLC=$(sed -n 1,3p "$FX"; printf 'derivation: m/48'"'"'/0'"'"'/9'"'"'/2'"'"'\n'; sed -n '4,$p' "$FX")
text R16-derivation-lowercase-after-shared  "${TWODERLC}
"
text R17-derivation-lowercase-per-cosigner  "$(printf '%s\n' "$PERCOS" | sed '6s/^Derivation:/derivation:/')
"
# Coldcard render_export shape: Derivation only where the path changes (paths A, B, B).
RENDER=$(printf 'Name: render-export\nPolicy: 2 of 3\nFormat: P2WSH\nDerivation: m/48'"'"'/0'"'"'/0'"'"'/2'"'"'\n5436D724: %s\nDerivation: m/48'"'"'/0'"'"'/1'"'"'/2'"'"'\n28645006: %s\nB8688DF1: %s\n' "$X1" "$X2" "$X3")
text R18-render-export-path-change-carries-forward "${RENDER}
"
