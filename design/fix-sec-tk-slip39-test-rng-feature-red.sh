#!/usr/bin/env bash
# RED/GREEN rows for design/FIX_sec-tk-slip39-test-rng-feature.md (secret row
# tk-slip39-test-rng-env-in-production). Run against a `mnemonic` binary:
#   design/fix-sec-tk-slip39-test-rng-feature-red.sh <mnemonic-binary>
# One line per row: rc, share-line count, the count of the test-RNG warning
# line on stderr, and the first stderr line starting with "error:". The
# phrase is the public BIP-39 test vector (abandon x11 about); it guards no
# funds.
# NOT covered: the CI workflow edits (read the diff).
set -u
B=${1:?usage: $0 <mnemonic-binary>}
T=$(mktemp -d "${TMPDIR:-/tmp}/slip39-rng-red.XXXXXX")
trap 'rm -f "$T"/*; rmdir "$T"' EXIT
P='abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about'
S=1111111111111111111111111111111111111111111111111111111111111111
SPLIT=(slip39 split --group-threshold 1 --group 3,2)

show() { # show <name> <rc> : summarise $T/out and $T/err
  local shares warn stem
  shares=$(grep -c ' ' "$T/out")
  warn=$(grep -c 'deterministic and INSECURE' "$T/err")
  stem=$(grep -m1 '^error:' "$T/err")
  printf '%s rc=%s shares=%s insecure-warning=%s error=%s\n' "$1" "$2" "$shares" "$warn" "${stem:-<none>}"
}

# R1: the test seed makes split reproducible: two runs, same shares.
printf '%s' "$P" | env -u MNEMONIC_SLIP39_TEST_IDENTIFIER MNEMONIC_SLIP39_TEST_RNG=$S "$B" "${SPLIT[@]}" --from phrase=- >"$T/a" 2>"$T/err"; rc=$?
printf '%s' "$P" | env -u MNEMONIC_SLIP39_TEST_IDENTIFIER MNEMONIC_SLIP39_TEST_RNG=$S "$B" "${SPLIT[@]}" --from phrase=- >"$T/out" 2>/dev/null
show R1-test-rng-seed "$rc"
if [ -s "$T/a" ] && cmp -s "$T/a" "$T/out"; then echo "R1-test-rng-seed reproducible=yes"; else echo "R1-test-rng-seed reproducible=no"; fi

# R2: the identifier variable alone is honoured.
printf '%s' "$P" | env -u MNEMONIC_SLIP39_TEST_RNG MNEMONIC_SLIP39_TEST_IDENTIFIER=12345 "$B" "${SPLIT[@]}" --from phrase=- >"$T/out" 2>"$T/err"
show R2-identifier-only $?

# R3: an empty value counts as set.
printf '%s' "$P" | env -u MNEMONIC_SLIP39_TEST_IDENTIFIER MNEMONIC_SLIP39_TEST_RNG= "$B" "${SPLIT[@]}" --from phrase=- >"$T/out" 2>"$T/err"
show R3-empty-value $?

# R4: a non-UTF-8 value counts as set.
printf '%s' "$P" | env -u MNEMONIC_SLIP39_TEST_IDENTIFIER $'MNEMONIC_SLIP39_TEST_RNG=\xff' "$B" "${SPLIT[@]}" --from phrase=- >"$T/out" 2>"$T/err"
show R4-non-utf8-value $?

# R5: the refusal comes before stdin is read (left = bytes still unread).
left=$(printf '%s' "$P" | { env -u MNEMONIC_SLIP39_TEST_IDENTIFIER MNEMONIC_SLIP39_TEST_RNG=$S "$B" "${SPLIT[@]}" --from phrase=- >"$T/out" 2>"$T/err"; echo "rc=$?" >"$T/rc"; wc -c | tr -d ' '; })
show R5-before-stdin "$(cut -d= -f2 "$T/rc")"
echo "R5-before-stdin left=$left"

# R6: the refusal comes before an @env: source is resolved (MNX_RED_UNSET is unset).
env -u MNX_RED_UNSET -u MNEMONIC_SLIP39_TEST_IDENTIFIER MNEMONIC_SLIP39_TEST_RNG=$S "$B" "${SPLIT[@]}" --from phrase=@env:MNX_RED_UNSET </dev/null >"$T/out" 2>"$T/err"
show R6-before-env-source $?

# R7: the refusal comes before the argv checks (--from xpub= is refused by row 17).
env -u MNEMONIC_SLIP39_TEST_IDENTIFIER MNEMONIC_SLIP39_TEST_RNG=$S "$B" "${SPLIT[@]}" --from xpub=x </dev/null >"$T/out" 2>"$T/err"
show R7-before-argv-checks $?

# C1 (control, unchanged): combine ignores the variables.
env -u MNEMONIC_SLIP39_TEST_IDENTIFIER MNEMONIC_SLIP39_TEST_RNG=$S "$B" slip39 combine --share - </dev/null >"$T/out" 2>"$T/err"
show C1-combine-ignores $?

# C2 (control, unchanged): with both variables unset, two splits differ.
printf '%s' "$P" | env -u MNEMONIC_SLIP39_TEST_RNG -u MNEMONIC_SLIP39_TEST_IDENTIFIER "$B" "${SPLIT[@]}" --from phrase=- >"$T/a" 2>/dev/null; rc=$?
printf '%s' "$P" | env -u MNEMONIC_SLIP39_TEST_RNG -u MNEMONIC_SLIP39_TEST_IDENTIFIER "$B" "${SPLIT[@]}" --from phrase=- >"$T/out" 2>"$T/err"
show C2-unset-os-rng "$rc"
if cmp -s "$T/a" "$T/out"; then echo "C2-unset-os-rng reproducible=yes"; else echo "C2-unset-os-rng reproducible=no"; fi
