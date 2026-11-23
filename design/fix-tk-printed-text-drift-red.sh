#!/usr/bin/env bash
# RED/GREEN probe for design/FIX-tk-printed-text-drift.md (bughunt-1119 bh-tk-I9, I10, I11).
# Usage: design/fix-tk-printed-text-drift-red.sh <path-to-mnemonic-binary>   (run from the repo root)
# Prints one block per row: "<row> <verdict> :: <evidence>". RED at the base, GREEN after the fix.
# Covers rows H1, H2, D1-D5, P1-P3, S1-S3, B1-B3, G1 of the FIX doc (section 5).
# Does NOT cover: the rendered PDF/HTML manual (it reads the markdown sources), the generated man pages
# (clap renders the same doc comments; not run here), a terminal stdin, the `xxd`/`qrencode` stages of the
# QR pipelines (cut off before running), any manual section other than the three named below.
# Public test material only (BIP-85 test-vector master, the all-zero "abandon ... about" phrase).
set -u
B=${1:?usage: $0 <mnemonic-binary>}
B=$(cd "$(dirname "$B")" && pwd)/$(basename "$B")
BINDIR=$(dirname "$B")
M41=docs/manual/src/40-cli-reference/41-mnemonic.md
M38=docs/manual/src/30-workflows/38-bip85-children.md
G46=docs/manual-gui/src/40-mnemonic/46-derive-child.md
LIST=docs/manual/tests/cli-subcommands.list
X=xprv9s21ZrQH143K2LBWUUQRFXhucrQqBpKdRRxNVq2zBqsx8HVqFk2uYo8kmbaLLHRdqtQpUm98uKfu3vca1LqdGhUtyoFnCNkfmXRyPXLjbKb
T=$(mktemp -d)
trap 'rm -r "$T"' EXIT
norm() { tr -s ' \t\n' ' ' | sed 's/^ //; s/ $//'; }
verdict() { if [ "$1" = 0 ]; then echo GREEN; else echo RED; fi; }

# --- extractor shared by S1/B1: logical commands of ```sh blocks in a line range.
# Joins a line onto the previous one when the previous ends with `\` or the line starts with `|`.
# Skips comment lines, but a "# → <value>" line right after a command is that command's expectation
# (value = text after "# → " up to the first run of 2+ spaces). Output records: <cmd>\x1f<expect>\x1e
extract() { # extract <file> <start-regex> <end-regex>
  awk -v s="$2" -v e="$3" '
    function flush() { if (cur != "") { printf "%s\037%s\036", cur, exp_; } cur=""; exp_="" }
    $0 ~ s && !on { on=1; next }
    on && $0 ~ e { flush(); on=0 }
    !on { next }
    /^```sh/ { insh=1; cur=""; exp_=""; next }
    /^```/ { if (insh) flush(); insh=0; next }
    !insh { next }
    {
      t=$0; sub(/^[ \t]+/, "", t)
      if (t ~ /^# → /) { v=substr(t, length("# → ")+1); sub(/  .*/, "", v); if (cur!="") exp_=v; next }
      if (t ~ /^#/) next
      if (t == "") { flush(); next }
      if (cur != "" && (cur ~ /\\$/ || t ~ /^\|/)) { cur = cur "\n" $0 } else { flush(); cur=$0 }
    }
    END { flush() }' "$1"
}

run_section() { # run_section <row> <file> <start> <end> <needle> <skip-regex>
  local row=$1 n=0 bad=0 rec cmd exp_ out err rc
  extract "$2" "$3" "$4" > "$T/recs"
  while IFS= read -r -d $'\036' rec; do
    cmd=${rec%%$'\037'*}; exp_=${rec#*$'\037'}
    case $cmd in *"$5"*) ;; *) continue ;; esac
    if printf '%s' "$cmd" | grep -qE -- "$6"; then continue; fi
    # cut the external QR stages (`| xxd`, `| qrencode`) and any trailing backslash
    cmd=$(printf '%s' "$cmd" | awk 'BEGIN{RS="\0"} { i=index($0,"| xxd"); j=index($0,"| qrencode");
      k=(i&&j)?((i<j)?i:j):(i?i:j); if(k) $0=substr($0,1,k-1); sub(/[ \t\n\\]+$/, ""); printf "%s", $0 }')
    n=$((n+1))
    out=$(PATH="$BINDIR:$PATH" bash -o pipefail -c "$cmd" 2>"$T/err"); rc=$?
    err=$(grep -v '^warning: stdout carries' "$T/err" | head -n1)
    local first=${out%%$'\n'*}
    if [ "$rc" != 0 ] || [ -z "$out" ] || { [ -n "$exp_" ] && [ "$first" != "$exp_" ]; }; then bad=$((bad+1)); fi
    printf '  [%s] rc=%s out=%.48s%s err=%.110s\n' "$(printf '%s' "$cmd" | norm | cut -c1-90)" "$rc" "$first" \
      "$([ -n "$exp_" ] && printf ' expect=%.48s' "$exp_")" "$err"
  done < "$T/recs"
  echo "  runnable=$n failing=$bad"
  [ "$bad" = 0 ] && [ "$n" = "$7" ]
}

inline_lines() { # inline_lines <file> <start> <end> <regex>: sh-block lines with a secret given inline
  awk -v s="$2" -v e="$3" '$0 ~ s && !on {on=1; next} on && $0 ~ e {on=0} !on {next}
    /^```sh/ {insh=1; next} /^```/ {insh=0; next} insh {print FNR": "$0}' "$1" | grep -E -- "$4"
}

app_entry() { # app_entry <token>: the normalized --application help entry of <token>
  "$B" derive-child --help 2>/dev/null | awk -v tok="$1" '
    { t=$0; sub(/^[ \t]+/, "", t) }
    t ~ /^--application / { inapp=1; next }
    inapp && t ~ /^--/ { inapp=0 }
    !inapp { next }
    t ~ /^(bip39|hd-seed|xprv|hex|password-base64|password-base85|dice|rsa|rsa-gpg)  / { split(t, a, " "); on=(a[1]==tok) }
    t == "" || t ~ /^--/ { on=0 }
    on { print t }' | norm
}

section() { awk -v s="$2" 'index($0,s)==1 {on=1; print; next} on && /^### / {exit} on {print}' "$1" | norm; }

echo "== I9: compare-cost --help example"
ex=$("$B" compare-cost --help | norm | grep -oP 'e\.g\., `\K[^`]*' | head -n1)
"$B" compare-cost --miniscript "$ex" >"$T/o" 2>"$T/e"; rc=$?
[ "$rc" = 0 ]; echo "H1 $(verdict $?) :: example=[$ex] rc=$rc out=$(head -n1 "$T/o") err=$(head -n1 "$T/e" | cut -c1-120)"

: > "$T/eg"
grep -E '^mnemonic( |$)' "$LIST" | while read -r _ sub; do
  # shellcheck disable=SC2086
  "$B" $sub --help 2>/dev/null | norm | grep -oP 'e\.g\.,? `\K[^`]*' | sed "s|^|${sub:-<root>}: |" >> "$T/eg"
done
cat > "$T/eg.want" <<'EOF'
compare-cost: or_b(pk(A),s:pk(B))
convert: --to mk1
export-wallet: bip84-0
gen-man: mnemonic-seed-xor-split.1
restore: --account 0,1,2,3
restore: 2h
restore: m/84'/0'/7'
verify-bundle: 2h
verify-bundle: m/84'/0'/7'
EOF
LC_ALL=C sort "$T/eg" > "$T/eg.got"; LC_ALL=C sort -o "$T/eg.want" "$T/eg.want"
diff "$T/eg.want" "$T/eg.got" > "$T/eg.diff"; rc=$?
echo "H2 $(verdict $rc) :: $(wc -l < "$T/eg.got") e.g. spans; diff want/got: $(tr '\n' ' ' < "$T/eg.diff")"

echo "== I10: derive-child --help application entries"
chk() { # chk <row> <token> <want>
  local got; got=$(app_entry "$2")
  [ "$got" = "$3" ]; echo "$1 $(verdict $?) :: [$got]"
}
chk D1 hd-seed "hd-seed BIP-85 application 2' — derives a private key, printed as a compressed WIF, at m/83696968'/2'/<index>' (\`--length 0\`)"
chk D2 xprv "xprv BIP-85 application 32' — derives a child xprv at m/83696968'/32'/<index>' (\`--length 0\`)"
chk D3 dice "dice BIP-85 application 89101' — derives \`--length\` rolls (1..=2^32-1) of a \`--dice-sides\`-sided die"
chk D4 bip39 "bip39 BIP-85 application 39' — derives a child BIP-39 mnemonic of \`--length\` words (12 / 15 / 18 / 21 / 24)"
chk D5 rsa-gpg "rsa-gpg BIP-85 application 828365' (RSA-GPG key scheme); REFUSED at runtime"

echo "== I10 controls (behaviour is right at the base; must stay so)"
o=$(printf '%s' "$X" | "$B" derive-child --from xprv=- --application hd-seed --length 0 --index 0 2>/dev/null)
[ "$o" = Kzyv4uF39d4Jrw2W7UryTHwZr1zQVNk4dAFyqE6BuMrMh1Za7uhp ]; echo "P1 $(verdict $?) :: hd-seed idx0 = $o"
o=$(printf '%s' "$X" | "$B" derive-child --from xprv=- --application xprv --length 0 --index 0 2>/dev/null)
[ "$o" = xprv9s21ZrQH143K2srSbCSg4m4kLvPMzcWydgmKEnMmoZUurYuBuYG46c6P71UGXMzmriLzCCBvKQWBUv3vPB3m1SATMhp3uEjXHJ42jFg7myX ]
echo "P2 $(verdict $?) :: xprv idx0 = $o"
r=$(printf '%s' "$X" | "$B" derive-child --from xprv=- --application dice --dice-sides 6 --length 100 --index 0 2>/dev/null | tr ',' '\n' | grep -c .)
w15=$(printf '%s' "$X" | "$B" derive-child --from xprv=- --application bip39 --length 15 --index 0 2>/dev/null | wc -w)
w21=$(printf '%s' "$X" | "$B" derive-child --from xprv=- --application bip39 --length 21 --index 0 2>/dev/null | wc -w)
[ "$r" = 100 ] && [ "$w15" = 15 ] && [ "$w21" = 21 ]; echo "P3 $(verdict $?) :: dice rolls=$r bip39 words=$w15,$w21"

echo "== I11: manual seedqr section (41-mnemonic.md, '## \`mnemonic seedqr\`' to the next '## ')"
S_START='^## `mnemonic seedqr`'; S_END='^## '
run_section S1 "$M41" "$S_START" "$S_END" 'mnemonic seedqr' '--json-out' 8; echo "S1 $(verdict $?) (want runnable=8 failing=0)"
inline_lines "$M41" "$S_START" "$S_END" '--from (phrase|seedqr)=[^-]|--digits [^-]' > "$T/s2"
[ ! -s "$T/s2" ]; echo "S2 $(verdict $?) :: inline lines: $(cut -c1-80 "$T/s2" | tr '\n' '|')"
sec=$(awk '/^## `mnemonic seedqr`/ {on=1; next} on && /^## / {exit} on' "$M41" | norm)
case $sec in *'to avoid the advisory'*|*'argv-leakage advisory'*) s3=1 ;; *'(#argv-refusal)'*) s3=0 ;; *) s3=1 ;; esac
echo "S3 $(verdict $s3) :: $(printf '%s' "$sec" | grep -oE 'Both subsubcommands [^.]*\.' | head -n1)"

echo "== I10/I11: manual chapter 38 (38-bip85-children.md, whole file)"
run_section B1 "$M38" '^# Deterministic child secrets' '^XXXX-never' 'mnemonic derive-child' '<' 5; echo "B1 $(verdict $?) (want runnable=5 failing=0)"
inline_lines "$M38" '^# Deterministic child secrets' '^XXXX-never' '--from (phrase|xprv)=[^-]' > "$T/b2"
[ ! -s "$T/b2" ]; echo "B2 $(verdict $?) :: inline lines: $(cut -c1-70 "$T/b2" | tr '\n' '|')"
b3=0
for want in \
  '| `bip39` | new BIP-39 phrase (10 BIP-85-coded languages) | `12`, `15`, `18`, `21`, `24` |' \
  '| `hd-seed` | private key as a compressed WIF | `0` (sentinel) |' \
  '| `dice` | deterministic dice rolls | `1..=2^32-1` |' \
  '  (12 / 15 / 18 / 21 / 24 words; 10 BIP-85 languages) for an air-gapped'; do
  grep -qxF -- "$want" "$M38" || { b3=1; echo "  missing: $want"; }
done
for gone in 'HD-seed bytes' '64-byte HD-seed' '9 BIP-85' 'land in v0.9' '1..=10000'; do
  grep -qF -- "$gone" "$M38" && { b3=1; echo "  still present: $gone"; }
done
echo "B3 $(verdict $b3)"

echo "== I10: GUI manual 46-derive-child.md"
hd=$(section "$G46" '### `hd-seed` {#mnemonic-derive-child-application-hd-seed}'); xp=$(section "$G46" '### `xprv` {#mnemonic-derive-child-application-xprv}')
want_hd="### \`hd-seed\` {#mnemonic-derive-child-application-hd-seed} Emit a child private key as a compressed WIF (BIP-85 application 2', path \`m/83696968'/2'/<index>'\`). \`--length\` must be 0. \`--network\` selects the mainnet (\`K…\` / \`L…\`) or test-network (\`c…\`) WIF prefix."
want_xp="### \`xprv\` {#mnemonic-derive-child-application-xprv} Emit a child xprv at depth 0 (BIP-85 application 32', path \`m/83696968'/32'/<index>'\`). \`--length\` must be 0. \`--network\` selects the prefix."
[ "$hd" = "$want_hd" ] && [ "$xp" = "$want_xp" ]; echo "G1 $(verdict $?) :: hd-seed=[$hd] xprv=[$xp]"
