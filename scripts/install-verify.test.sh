#!/bin/sh
# install-verify.test.sh — offline harness for scripts/install.sh's binary path
# (F-676): download -> sha256 verify against the release's SHA256SUMS* -> install,
# and the cargo fallback.
#
# A stub `curl` first on PATH serves https://github.com/<owner>/<repo>/releases/
# download/<tag>/<file> from a local fixture tree (404 = exit 22, as curl -f),
# and a stub `cargo` records its argv. Nothing touches the network. Each case
# asserts the exit status, a message, AND what landed in <root>/bin, so a
# refusal that still installs something reads as a failure.
#
# Usage: sh scripts/install-verify.test.sh
set -eu

here=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
INSTALL_SH="$here/install.sh"
T=$(mktemp -d)
trap 'rm -rf "$T"' EXIT

fail=0
ok()  { printf '  ok   %s\n' "$1"; }
bad() { printf '  FAIL %s\n' "$1"; fail=1; }

# The pinned mk release, from the installer's own table.
MK_TAG=$(sed -n 's/.*"mk-cli|https:\/\/github.com\/bg002h\/mnemonic-key|\([^|]*\)|.*/\1/p' "$INSTALL_SH")
MK_VER=${MK_TAG##*-v}
[ -n "$MK_TAG" ] || { echo "cannot read the mk pin from install.sh" >&2; exit 1; }
ASSET="mk-$MK_VER-x86_64-linux-musl.tar.gz"
REL="$T/fixture/mnemonic-key/$MK_TAG"

# ── stubs ─────────────────────────────────────────────────────────────────
mkdir -p "$T/stub"
cat > "$T/stub/curl" <<EOF
#!/bin/sh
out=""; url=""; prev=""
for a in "\$@"; do [ "\$prev" = "-o" ] && out="\$a"; prev="\$a"; url="\$a"; done
echo "\$url" >> "$T/curl.log"
rel=\${url#https://github.com/bg002h/}
repo=\${rel%%/*}; rest=\${rel#*/releases/download/}
src="$T/fixture/\$repo/\$rest"
[ -f "\$src" ] || exit 22
cp "\$src" "\$out"
EOF
cat > "$T/stub/cargo" <<EOF
#!/bin/sh
echo "CARGO \$*" >> "$T/cargo.log"
EOF
chmod +x "$T/stub/curl" "$T/stub/cargo"

# fresh_release: a good archive (./mk prints a version) + a matching sums file.
fresh_release() {
    rm -rf "$T/fixture" "$T/pkg"; mkdir -p "$REL" "$T/pkg"
    printf '#!/bin/sh\necho "mk %s"\n' "$MK_VER" > "$T/pkg/mk"
    chmod +x "$T/pkg/mk"
    printf 'license\n' > "$T/pkg/LICENSE"
    tar -czf "$REL/$ASSET" -C "$T/pkg" mk LICENSE
    ( cd "$REL" && sha256sum "$ASSET" > SHA256SUMS.x86_64 )
}

# run_case <label> <platform> [install.sh args...] -> sets rc, out; fresh root.
run_case() {
    _label=$1; _plat=$2; shift 2
    rm -rf "$T/root" "$T/curl.log" "$T/cargo.log"
    rc=0
    out=$(PATH="$T/stub:$PATH" MNEMONIC_INSTALL_PLATFORM="$_plat" \
          sh "$INSTALL_SH" --only mk --no-man --root "$T/root" "$@" 2>&1) || rc=$?
}
installed() { [ -x "$T/root/bin/mk" ]; }

echo "[install-verify.test] $INSTALL_SH (mk pin $MK_TAG)"

# 1. good download installs, and the installed file is the archive's binary.
fresh_release
run_case good linux-x86_64-gnu
if [ "$rc" -eq 0 ] && installed && [ "$("$T/root/bin/mk")" = "mk $MK_VER" ] \
   && printf '%s' "$out" | grep -q 'verified sha256'; then
    ok "good archive: verified, installed, runs"
else bad "good archive (rc=$rc): $out"; fi
if [ ! -e "$T/cargo.log" ]; then ok "good archive: cargo never invoked"; else bad "good archive invoked cargo"; fi

# 2. one flipped byte in the archive -> refused, nothing installed.
python3 - "$REL/$ASSET" <<'PY'
import sys; p = sys.argv[1]; b = bytearray(open(p, "rb").read()); b[len(b) // 2] ^= 1; open(p, "wb").write(b)
PY
run_case tampered linux-x86_64-gnu
if [ "$rc" -ne 0 ] && ! installed && printf '%s' "$out" | grep -q 'REFUSED .*sha256 mismatch'; then
    ok "tampered archive (one byte flipped): refused, nothing installed"
else bad "tampered archive (rc=$rc, installed=$(installed && echo yes || echo no)): $out"; fi

# 3. sums file present but does not list the asset -> refused.
fresh_release
echo "0000000000000000000000000000000000000000000000000000000000000000  other.tar.gz" > "$REL/SHA256SUMS.x86_64"
run_case unlisted linux-x86_64-gnu
if [ "$rc" -ne 0 ] && ! installed && printf '%s' "$out" | grep -q 'REFUSED .*no checksum'; then
    ok "asset missing from SHA256SUMS: refused, nothing installed"
else bad "asset missing from SHA256SUMS (rc=$rc): $out"; fi

# 4. no sums file at all -> refused (not silently trusted, not a cargo fallback).
fresh_release
rm -f "$REL"/SHA256SUMS*
run_case nosums linux-x86_64-gnu
if [ "$rc" -ne 0 ] && ! installed && [ ! -e "$T/cargo.log" ] \
   && printf '%s' "$out" | grep -q 'REFUSED .*no checksum'; then
    ok "no SHA256SUMS published: refused, no cargo fallback"
else bad "no SHA256SUMS (rc=$rc): $out"; fi

# 5. binary-mode sums line (`<hex> *<name>`) in a later candidate file is honored.
fresh_release
( cd "$REL" && rm SHA256SUMS.x86_64 && printf '%s *%s\n' "$(sha256sum "$ASSET" | cut -d' ' -f1)" "$ASSET" > SHA256SUMS.portable )
run_case starform linux-x86_64-gnu
if [ "$rc" -eq 0 ] && installed && printf '%s' "$out" | grep -q 'SHA256SUMS.portable'; then
    ok "'<hex> *<name>' line in SHA256SUMS.portable accepted"
else bad "star-form sums (rc=$rc): $out"; fi

# 6. the release asset is missing (404) -> FAILED, nothing installed.
fresh_release
rm -f "$REL/$ASSET"
run_case missing linux-x86_64-gnu
if [ "$rc" -ne 0 ] && ! installed && printf '%s' "$out" | grep -q 'download failed'; then
    ok "asset 404: download failed, nothing installed"
else bad "asset 404 (rc=$rc): $out"; fi

# 7. a platform with no binary -> cargo from the pinned tag, said on stderr.
fresh_release
run_case freebsd freebsd-x86_64
if [ "$rc" -eq 0 ] && grep -q -- "install --locked --git https://github.com/bg002h/mnemonic-key --tag $MK_TAG .*--root $T/root .*mk-cli" "$T/cargo.log" 2>/dev/null \
   && printf '%s' "$out" | grep -q 'publishes no mk binary for platform'; then
    ok "no binary for freebsd-x86_64: cargo install --git --tag $MK_TAG, with a note"
else bad "freebsd fallback (rc=$rc): $out / $(cat "$T/cargo.log" 2>/dev/null)"; fi
if [ ! -e "$T/curl.log" ]; then ok "freebsd fallback: nothing downloaded"; else bad "freebsd fallback downloaded: $(cat "$T/curl.log")"; fi

# 8. --from-source forces cargo even where a binary exists; no note.
run_case fromsource linux-x86_64-gnu --from-source
if [ "$rc" -eq 0 ] && grep -q -- "--tag $MK_TAG" "$T/cargo.log" 2>/dev/null && [ ! -e "$T/curl.log" ] \
   && ! printf '%s' "$out" | grep -q 'publishes no'; then
    ok "--from-source: cargo at the pinned tag, no download"
else bad "--from-source (rc=$rc): $out"; fi

# 9. crates.io is gone: no cargo invocation without --git anywhere.
run_case cratesio freebsd-x86_64
if grep -v -- '--git' "$T/cargo.log" 2>/dev/null | grep -q .; then
    bad "a cargo install without --git (crates.io) was issued: $(cat "$T/cargo.log")"
else ok "no crates.io install path"; fi

# rerelease <mk script body>: a correctly checksummed archive whose ./mk is
# the given script, so only the run-check can refuse it.
rerelease() {
    fresh_release
    printf '#!/bin/sh\n%s\n' "$1" > "$T/pkg/mk"
    tar -czf "$REL/$ASSET" -C "$T/pkg" mk LICENSE
    ( cd "$REL" && sha256sum "$ASSET" > SHA256SUMS.x86_64 )
}

# 10. checksum matches but the binary reports another version -> refused.
rerelease 'echo "mk 0.0.1"'
run_case wrongver linux-x86_64-gnu
if [ "$rc" -ne 0 ] && ! installed && printf '%s' "$out" | grep -q "REFUSED .*did not print 'mk $MK_VER'"; then
    ok "binary reporting the wrong version: refused, nothing installed"
else bad "wrong version (rc=$rc, installed=$(installed && echo yes || echo no)): $out"; fi

# 11. checksum matches but the binary cannot run here (e.g. too-old glibc).
rerelease 'echo "mk: /lib/libc.so.6: version GLIBC_2.34 not found" >&2; exit 1'
run_case wontrun linux-x86_64-gnu
if [ "$rc" -ne 0 ] && ! installed && printf '%s' "$out" | grep -q "it printed: (exit 1) mk: .*GLIBC_2.34"; then
    ok "binary that does not run on this host: refused, nothing installed"
else bad "unrunnable binary (rc=$rc, installed=$(installed && echo yes || echo no)): $out"; fi

# 12. libc detection: glibc wins even if ldd would say musl; musl otherwise.
detect() {  # detect <getconf exit status> -> prints the detected platform
    mkdir -p "$T/det"
    printf '#!/bin/sh\ncase "$1" in -s) echo Linux ;; -m) echo x86_64 ;; esac\n' > "$T/det/uname"
    printf '#!/bin/sh\nexit %s\n' "$1" > "$T/det/getconf"
    printf '#!/bin/sh\necho "musl libc (x86_64)"\n' > "$T/det/ldd"
    chmod +x "$T/det/uname" "$T/det/getconf" "$T/det/ldd"
    PATH="$T/det:$PATH" MNEMONIC_INSTALL_PLATFORM='' sh "$INSTALL_SH" --list | sed -n 's/^platform: //p'
}
g=$(detect 0); m=$(detect 1)
if [ "$g" = linux-x86_64-gnu ] && [ "$m" = linux-x86_64-musl ]; then
    ok "libc detection: getconf GNU_LIBC_VERSION -> gnu; otherwise ldd musl -> musl"
else bad "libc detection: glibc host -> '$g', musl host -> '$m'"; fi

if [ "$fail" -eq 0 ]; then
    echo "[install-verify.test] OK"
else
    echo "[install-verify.test] FAILED" >&2
    exit 1
fi
