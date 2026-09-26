#!/bin/sh
# install-signature.test.sh — offline harness for scripts/install.sh's minisign
# check of a release's SHA256SUMS* file (sign-all-releases, 2026-09-25).
#
# Uses THROWAWAY keypairs generated here (minisign -G -W), never the real
# release key: the installer under test is a COPY whose pinned MINISIGN_PUBKEY
# (and, per case, first_signed entry for mk) is rewritten by sed, and every
# rewrite is asserted to have applied, so a no-op substitution cannot make a
# case pass. A stub `curl` serves release files from a local fixture tree, as
# in install-verify.test.sh. Each case asserts the exit status, a message AND
# what landed in <root>/bin, so a refusal that still installs reads as a FAIL.
#
# Cases:
#   1  good signature                          -> verified, installed
#   2  tampered: asset AND its sums line swapped, old .minisig kept
#                                              -> refused (signature), nothing installed
#   3  sums signed by a different key          -> refused
#   4  missing .minisig, no first_signed entry -> installed with a note
#   5  missing .minisig, pin BEFORE first_signed -> installed with a note
#   6  missing .minisig, pin AT first_signed   -> refused
#   7  missing .minisig, pin AFTER first_signed -> refused
#   8  minisign absent                         -> said ONCE (two components), installed
#   9  minisign absent + --require-signature   -> refused before any download
#  10  missing .minisig + --require-signature  -> refused
#  11  good signature + --require-signature    -> installed
#
# Needs minisign on PATH (it generates the test keys). Usage:
#   sh scripts/install-signature.test.sh
set -eu

here=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
REAL_SH="$here/install.sh"
T=$(mktemp -d)
trap 'rm -rf "$T"' EXIT

fail=0
ok()  { printf '  ok   %s\n' "$1"; }
bad() { printf '  FAIL %s\n' "$1"; fail=1; }
# has <text>: the last run's output contains <text> (no pipe: F-695).
has() { case "$out" in *"$1"*) return 0 ;; esac; return 1; }
count() { _c=0; _rest="$out"; while :; do case "$_rest" in *"$1"*) _c=$((_c + 1)); _rest=${_rest#*"$1"} ;; *) break ;; esac; done; echo "$_c"; }

command -v minisign >/dev/null 2>&1 || { echo "install-signature.test: minisign is required (apt-get install minisign)" >&2; exit 1; }

# ── throwaway keys ───────────────────────────────────────────────────────
minisign -G -W -p "$T/test.pub" -s "$T/test.key" >/dev/null 2>&1
minisign -G -W -p "$T/other.pub" -s "$T/other.key" >/dev/null 2>&1
TESTPUB=$(sed -n 2p "$T/test.pub")
REALPUB=$(sed -n 's/^MINISIGN_PUBKEY="\(.*\)"$/\1/p' "$REAL_SH")
[ -n "$TESTPUB" ] && [ -n "$REALPUB" ] && [ "$TESTPUB" != "$REALPUB" ] \
    || { echo "cannot set up keys (test=$TESTPUB real=$REALPUB)" >&2; exit 1; }

# installer_copy <mk first_signed version, or empty>: $T/install.sh with the
# test key pinned. Both rewrites are asserted.
installer_copy() {
    sed -e "s|^MINISIGN_PUBKEY=\".*\"\$|MINISIGN_PUBKEY=\"$TESTPUB\"|" \
        -e "s|^        mk)           echo \"\" ;;\$|        mk)           echo \"$1\" ;;|" \
        "$REAL_SH" > "$T/install.sh"
    grep -qx "MINISIGN_PUBKEY=\"$TESTPUB\"" "$T/install.sh" \
        || { echo "pubkey rewrite did not apply" >&2; exit 1; }
    grep -qx "        mk)           echo \"$1\" ;;" "$T/install.sh" \
        || { echo "first_signed rewrite did not apply" >&2; exit 1; }
}

# ── fixture releases (mk and ms) ─────────────────────────────────────────
pin() { sed -n "s/.*\"$1|https:\/\/github.com\/bg002h\/$2|\([^|]*\)|.*/\1/p" "$REAL_SH"; }
MK_TAG=$(pin mk-cli mnemonic-key); MK_VER=${MK_TAG##*-v}
MS_TAG=$(pin ms-cli mnemonic-secret); MS_VER=${MS_TAG##*-v}
[ -n "$MK_TAG" ] && [ -n "$MS_TAG" ] || { echo "cannot read the mk/ms pins" >&2; exit 1; }
MK_REL="$T/fixture/mnemonic-key/$MK_TAG"; MK_ASSET="mk-$MK_VER-x86_64-linux-musl.tar.gz"
MS_REL="$T/fixture/mnemonic-secret/$MS_TAG"; MS_ASSET="ms-$MS_VER-x86_64-linux-musl.tar.gz"

# release <dir> <bin> <ver> <asset> <signing key or "none">
release() {
    mkdir -p "$1" "$T/pkg-$2"
    printf '#!/bin/sh\necho "%s %s"\n' "$2" "$3" > "$T/pkg-$2/$2"
    chmod +x "$T/pkg-$2/$2"
    tar -czf "$1/$4" -C "$T/pkg-$2" "$2"
    ( cd "$1" && sha256sum "$4" > SHA256SUMS.x86_64 )
    rm -f "$1/SHA256SUMS.x86_64.minisig"
    [ "$5" = none ] || minisign -S -s "$5" -m "$1/SHA256SUMS.x86_64" >/dev/null 2>&1 </dev/null
}
fresh() {  # fresh <signing key or none>: both releases, same signing state
    rm -rf "$T/fixture"
    release "$MK_REL" mk "$MK_VER" "$MK_ASSET" "$1"
    release "$MS_REL" ms "$MS_VER" "$MS_ASSET" "$1"
}

mkdir -p "$T/stub"
cat > "$T/stub/curl" <<EOF
#!/bin/sh
out=""; url=""; prev=""; fail=""
for a in "\$@"; do
    case "\$a" in --*) ;; -*f*) fail=1 ;; esac
    [ "\$prev" = "-o" ] && out="\$a"; prev="\$a"; url="\$a"
done
echo "\$url" >> "$T/curl.log"
rel=\${url#https://github.com/bg002h/}
repo=\${rel%%/*}; rest=\${rel#*/releases/download/}
src="$T/fixture/\$repo/\$rest"
if [ ! -f "\$src" ]; then
    [ -n "\$fail" ] && exit 22
    echo "Not Found" > "\$out"; exit 0
fi
cp "\$src" "\$out"
EOF
chmod +x "$T/stub/curl"

# A PATH with every tool the installer uses EXCEPT minisign, for cases 8-9.
mkdir -p "$T/nomsig"
for t in sh awk sed grep cut tr head wc sort mktemp rm mkdir cp mv chmod find tar gzip \
         sha256sum dirname cat ls uname getconf ldd env sleep printf; do
    p=$(command -v "$t" 2>/dev/null) || continue
    case "$p" in /*) ln -sf "$p" "$T/nomsig/$t" ;; esac
done

# run <only-list> <path-mode: with|without> [args...] -> rc, out; fresh root
run() {
    _only=$1; _mode=$2; shift 2
    rm -rf "$T/root" "$T/curl.log"
    if [ "$_mode" = with ]; then _p="$T/stub:$PATH"; else _p="$T/stub:$T/nomsig"; fi
    rc=0
    out=$(PATH="$_p" MNEMONIC_INSTALL_PLATFORM=linux-x86_64-gnu \
          sh "$T/install.sh" --only "$_only" --no-man --root "$T/root" "$@" 2>&1) || rc=$?
}
installed() { [ -x "$T/root/bin/$1" ]; }

echo "[install-signature.test] $REAL_SH (mk $MK_TAG, ms $MS_TAG)"
if PATH="$T/stub:$T/nomsig" command -v minisign >/dev/null 2>&1; then
    echo "the minisign-free PATH still finds minisign; cases 8-9 would be vacuous" >&2; exit 1
fi

# 1. good signature
installer_copy ""
fresh "$T/test.key"
run mk with
if [ "$rc" -eq 0 ] && installed mk && has "verified signature on SHA256SUMS.x86_64" && has "verified sha256"; then
    ok "1 good signature: verified, then sha256 verified, installed"
else bad "1 good signature (rc=$rc): $out"; fi

# 2. an attacker swaps the asset AND rewrites its sums line to match; the old
#    signature no longer covers the file. Must be refused on the SIGNATURE.
printf '#!/bin/sh\necho "mk %s"\n# evil\n' "$MK_VER" > "$T/pkg-mk/mk"
tar -czf "$MK_REL/$MK_ASSET" -C "$T/pkg-mk" mk
( cd "$MK_REL" && sha256sum "$MK_ASSET" > SHA256SUMS.x86_64 )
run mk with
if [ "$rc" -ne 0 ] && ! installed mk && has "REFUSED $MK_ASSET: SHA256SUMS.x86_64.minisig does not verify"; then
    ok "2 tampered sums file (asset + digest swapped): refused on the signature, nothing installed"
else bad "2 tampered sums (rc=$rc, installed=$(installed mk && echo yes || echo no)): $out"; fi

# 3. signed, but by a key that is not the pinned one
fresh "$T/other.key"
run mk with
if [ "$rc" -ne 0 ] && ! installed mk && has "does not verify against the pinned"; then
    ok "3 signature by another key: refused, nothing installed"
else bad "3 other key (rc=$rc): $out"; fi

# 4. missing signature, no first_signed entry yet -> allowed with a note
fresh none
run mk with
if [ "$rc" -eq 0 ] && installed mk && has "note: SHA256SUMS.x86_64 is not signed" && has "verified sha256"; then
    ok "4 missing signature, no first-signed pin: installed with a note"
else bad "4 missing, no pin (rc=$rc): $out"; fi

# 5-7. missing signature against a first_signed pin
installer_copy "999.0.0"
run mk with
if [ "$rc" -eq 0 ] && installed mk && has "note: SHA256SUMS.x86_64 is not signed"; then
    ok "5 missing signature, pin $MK_VER BEFORE first-signed 999.0.0: installed with a note"
else bad "5 missing, before pin (rc=$rc): $out"; fi

installer_copy "$MK_VER"
run mk with
if [ "$rc" -ne 0 ] && ! installed mk && has "releases are signed from $MK_VER on"; then
    ok "6 missing signature, pin AT first-signed $MK_VER: refused, nothing installed"
else bad "6 missing, at pin (rc=$rc): $out"; fi

installer_copy "0.0.1"
run mk with
if [ "$rc" -ne 0 ] && ! installed mk && has "releases are signed from 0.0.1 on"; then
    ok "7 missing signature, pin AFTER first-signed 0.0.1: refused, nothing installed"
else bad "7 missing, after pin (rc=$rc): $out"; fi

# 8. minisign absent: said once for two binary installs, both installed
installer_copy ""
fresh "$T/test.key"
run mk,ms without
n=$(count "minisign is not installed")
if [ "$rc" -eq 0 ] && installed mk && installed ms && [ "$n" -eq 1 ] \
   && ! has "verified signature"; then
    ok "8 minisign absent: said once (2 components), sha256 only, both installed"
else bad "8 minisign absent (rc=$rc, said $n times): $out"; fi

# 9. minisign absent + --require-signature: refused before anything downloads
run mk without --require-signature
if [ "$rc" -ne 0 ] && ! installed mk && [ ! -e "$T/curl.log" ] \
   && has "--require-signature was given, and"; then
    ok "9 minisign absent + --require-signature: refused, nothing downloaded"
else bad "9 absent + require (rc=$rc, curl.log=$(cat "$T/curl.log" 2>/dev/null)): $out"; fi

# 10. missing signature + --require-signature (no first_signed pin)
fresh none
run mk with --require-signature
if [ "$rc" -ne 0 ] && ! installed mk && has "and --require-signature was given"; then
    ok "10 missing signature + --require-signature: refused"
else bad "10 missing + require (rc=$rc): $out"; fi

# 11. good signature + --require-signature
fresh "$T/test.key"
run mk with --require-signature
if [ "$rc" -eq 0 ] && installed mk && has "verified signature on SHA256SUMS.x86_64"; then
    ok "11 good signature + --require-signature: installed"
else bad "11 good + require (rc=$rc): $out"; fi

# The shipped installer pins the real key, not a test one.
if [ "$REALPUB" = "RWQPmgBXsuw5yi8W0SfDr8KF+IqY/Z5U2p724emSODS1UPfJBP3agbKW" ]; then
    ok "install.sh pins the constellation release key"
else bad "install.sh pins an unexpected key: $REALPUB"; fi

if [ "$fail" -eq 0 ]; then
    echo "[install-signature.test] OK"
else
    echo "[install-signature.test] FAILED" >&2
    exit 1
fi
