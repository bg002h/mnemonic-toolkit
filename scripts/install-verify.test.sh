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
# SIGNATURES. When minisign is on PATH, the stub curl answers a request for
# `<sums file>.minisig` by signing that fixture sums file with a THROWAWAY key
# generated here, and the installer under test is a copy whose signing_keys
# table trusts only that key (the rewrite is asserted). So every fixture is a
# correctly SIGNED release, and these cases keep passing once a component's
# first_signed version is filled in. INSTALL_VERIFY_FIRST_SIGNED=<version>
# fills mk's first_signed in the copy, to prove exactly that. Without
# minisign, the installer skips signatures and the stub serves none.
#
# Usage: sh scripts/install-verify.test.sh
set -eu

here=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
REAL_SH="$here/install.sh"
T=$(mktemp -d)
trap 'rm -rf "$T"' EXIT
INSTALL_SH="$T/install.sh"
if command -v minisign >/dev/null 2>&1; then
    minisign -G -W -p "$T/sig.pub" -s "$T/sig.key" >/dev/null 2>&1
    SIGPUB=$(sed -n 2p "$T/sig.pub")
    awk -v k="*|||$SIGPUB" '/^\*\|\|\|/ { print k; n++; next } { print } END { exit n != 1 }' \
        "$REAL_SH" > "$INSTALL_SH" || { echo "cannot rewrite signing_keys" >&2; exit 1; }
    grep -qxF "*|||$SIGPUB" "$INSTALL_SH" || { echo "signing_keys rewrite did not apply" >&2; exit 1; }
else
    cp "$REAL_SH" "$INSTALL_SH"
fi
if [ -n "${INSTALL_VERIFY_FIRST_SIGNED:-}" ]; then
    sed -i "s|^        mk)           echo \"\" ;;\$|        mk)           echo \"$INSTALL_VERIFY_FIRST_SIGNED\" ;;|" "$INSTALL_SH"
    grep -qx "        mk)           echo \"$INSTALL_VERIFY_FIRST_SIGNED\" ;;" "$INSTALL_SH" \
        || { echo "first_signed rewrite did not apply" >&2; exit 1; }
fi

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
out=""; url=""; prev=""; fail=""
for a in "\$@"; do
    case "\$a" in --*) ;; -*f*) fail=1 ;; esac
    [ "\$prev" = "-o" ] && out="\$a"; prev="\$a"; url="\$a"
done
echo "\$url" >> "$T/curl.log"
[ -n "\${CURL_STUB_SLEEP:-}" ] && sleep "\$CURL_STUB_SLEEP"
if [ -n "\${CURL_STUB_RMTMP:-}" ]; then
    tmproot=\$(dirname "\$(dirname "\$out")")
    case "\$tmproot" in /?*/?*) rm -rf "\$tmproot" ;; esac
    exit 22
fi
rel=\${url#https://github.com/bg002h/}
repo=\${rel%%/*}; rest=\${rel#*/releases/download/}
src="$T/fixture/\$repo/\$rest"
case "\$src" in
    *.minisig)
        # Sign the fixture sums file on demand (throwaway key), if it exists.
        if [ ! -f "\$src" ] && [ -f "$T/sig.key" ] && [ -f "\${src%.minisig}" ]; then
            minisign -S -s "$T/sig.key" -m "\${src%.minisig}" -x "\$out" >/dev/null 2>&1 </dev/null && exit 0
        fi ;;
esac
if [ ! -f "\$src" ]; then
    [ -n "\$fail" ] && exit 22
    echo "Not Found" > "\$out"; exit 0
fi
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
if [ "$rc" -eq 0 ] && grep -q -- "install .*--locked --git https://github.com/bg002h/mnemonic-key --tag $MK_TAG .*mk-cli" "$T/cargo.log" 2>/dev/null \
   && grep -q -- "--root $T/root " "$T/cargo.log" \
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
# ── fold 2 (review f675-677-toolkit-review.md) ─────────────────────────────

# rearchive <members...>: rebuild the asset from $T/pkg with the given
# members (paths relative to $T/pkg) and a matching sums line.
rearchive() {
    tar -czf "$REL/$ASSET" -C "$T/pkg" "$@"
    ( cd "$REL" && sha256sum "$ASSET" > SHA256SUMS.x86_64 )
}

# 14. M2: mktemp fails -> refused before ANY rm/mkdir, nothing installed.
#     rm and mkdir are wrapped to log every call; the log must stay empty.
fresh_release
mkdir -p "$T/m2"
for tool in rm mkdir; do
    real=$(command -v "$tool")
    printf '#!/bin/sh\necho "%s $*" >> "%s/m2.log"\nexec %s "$@"\n' "$tool" "$T" "$real" > "$T/m2/$tool"
done
printf '#!/bin/sh\nexit 1\n' > "$T/m2/mktemp"
chmod +x "$T/m2/rm" "$T/m2/mkdir" "$T/m2/mktemp"
rm -rf "$T/root"; rm -f "$T/m2.log"; rc=0
out=$(PATH="$T/m2:$T/stub:$PATH" MNEMONIC_INSTALL_PLATFORM=linux-x86_64-gnu \
      sh "$INSTALL_SH" --only mk --no-man --root "$T/root" 2>&1) || rc=$?
if [ "$rc" -ne 0 ] && ! installed && [ ! -e "$T/m2.log" ] \
   && printf '%s' "$out" | grep -q 'cannot create a temporary directory'; then
    ok "mktemp fails: refused before any rm/mkdir, nothing installed"
else bad "mktemp failure (rc=$rc, rm/mkdir calls: $(cat "$T/m2.log" 2>/dev/null | tr '\n' ';')): $out"; fi
# mktemp that "succeeds" but prints a RELATIVE path, or an absolute path it
# never created: both must be refused before any rm/mkdir (review fold-2 Minor).
for lie in 'relative/evil-path' "$T/never-created"; do
    printf '#!/bin/sh\necho "%s"\nexit 0\n' "$lie" > "$T/m2/mktemp"; chmod +x "$T/m2/mktemp"
    rm -rf "$T/root"; rm -f "$T/m2.log"; rc=0
    out=$(cd "$T" && PATH="$T/m2:$T/stub:$PATH" MNEMONIC_INSTALL_PLATFORM=linux-x86_64-gnu \
          sh "$INSTALL_SH" --only mk --no-man --root "$T/root" 2>&1) || rc=$?
    if [ "$rc" -ne 0 ] && ! installed && [ ! -e "$T/m2.log" ] && [ ! -e "$T/relative" ] \
       && printf '%s' "$out" | grep -q 'cannot create a temporary directory'; then
        ok "mktemp prints '$lie' (exit 0): refused before any rm/mkdir"
    else bad "mktemp lie '$lie' (rc=$rc, rm/mkdir: $(tr '\n' ';' < "$T/m2.log" 2>/dev/null)): $out"; fi
done
# install_binary's own guard: the temp root disappears between components
# (the curl stub deletes it during ms's download). mk must not be fetched into
# a recreated directory; it is refused.
fresh_release
rm -rf "$T/root"; rm -f "$T/curl.log"; rc=0
out=$(PATH="$T/stub:$PATH" MNEMONIC_INSTALL_PLATFORM=linux-x86_64-gnu CURL_STUB_RMTMP=1 \
      sh "$INSTALL_SH" --only ms,mk --no-man --root "$T/root" 2>&1) || rc=$?
if [ "$rc" -ne 0 ] && ! installed && printf '%s' "$out" | grep -q 'temporary directory .* is gone' \
   && ! grep -q 'mnemonic-key' "$T/curl.log" 2>/dev/null; then
    ok "temp root deleted mid-run: the next component is refused, not refetched"
else bad "temp root gone (rc=$rc, downloads: $(tr '\n' ' ' < "$T/curl.log" 2>/dev/null)): $out"; fi
# ... and the realistic trigger: a TMPDIR that does not exist, real mktemp.
rm -f "$T/root/bin/mk"; rc=0
out=$(TMPDIR="$T/no-such-dir" PATH="$T/stub:$PATH" MNEMONIC_INSTALL_PLATFORM=linux-x86_64-gnu \
      sh "$INSTALL_SH" --only mk --no-man --root "$T/root" 2>&1) || rc=$?
if [ "$rc" -ne 0 ] && ! installed && printf '%s' "$out" | grep -q 'cannot create a temporary directory'; then
    ok "TMPDIR that does not exist: refused, nothing installed"
else bad "bad TMPDIR (rc=$rc): $out"; fi

# 15. I1: glibc floors decide binary vs source, before any download.
floor_plan() {  # floor_plan <component> <glibc> -> "binary" | "source"
    MNEMONIC_INSTALL_PLATFORM=linux-x86_64-gnu MNEMONIC_INSTALL_GLIBC="$2" \
        sh "$INSTALL_SH" --dry-run --no-man --only "$1" 2>&1 \
        | sed -n "s/^install  $1 (\([a-z]*\).*/\1/p" | sed 's/release/binary/'
}
r="$(floor_plan md 2.33) $(floor_plan md 2.34) $(floor_plan mnemonic-gui 2.17) $(floor_plan mnemonic-gui 2.18) $(floor_plan mk 2.17)"
if [ "$r" = "source binary source binary binary" ]; then
    ok "glibc floors: md 2.33->source 2.34->binary; gui 2.17->source 2.18->binary; mk static"
else bad "glibc floors: got '$r'"; fi
n=$(MNEMONIC_INSTALL_PLATFORM=linux-x86_64-gnu MNEMONIC_INSTALL_GLIBC=2.33 sh "$INSTALL_SH" --dry-run --no-man --only md 2>&1)
if printf '%s' "$n" | grep -q 'md binary needs glibc >= 2.34; this host has 2.33'; then
    ok "glibc floor: the note names the floor and the host's glibc"
else bad "glibc floor note: $n"; fi
# the host glibc comes from getconf when not overridden
mkdir -p "$T/gc"; printf '#!/bin/sh\necho "glibc 2.33"\n' > "$T/gc/getconf"; chmod +x "$T/gc/getconf"
g=$(PATH="$T/gc:$PATH" MNEMONIC_INSTALL_PLATFORM=linux-x86_64-gnu sh "$INSTALL_SH" --dry-run --no-man --only md 2>&1)
if printf '%s' "$g" | grep -q 'install  md (source'; then
    ok "glibc floor: host glibc read from getconf GNU_LIBC_VERSION"
else bad "getconf glibc: $g"; fi

# 16. I1: the refusal names a recipe, and that recipe works after a run that
#     already installed the binary (cargo refuses an untracked file without
#     --force, so the installer must pass it).
rerelease 'echo "mk 0.0.1"'
run_case recipe linux-x86_64-gnu
if printf '%s' "$out" | grep -q -- 're-run with: --from-source --only mk'; then
    ok "refusal prints the recipe: --from-source --only mk"
else bad "refusal recipe: $out"; fi
fresh_release
run_case first linux-x86_64-gnu                      # binary install, rc 0
rc=0
out=$(PATH="$T/stub:$PATH" MNEMONIC_INSTALL_PLATFORM=linux-x86_64-gnu \
      sh "$INSTALL_SH" --only mk --no-man --root "$T/root" --from-source 2>&1) || rc=$?
if [ "$rc" -eq 0 ] && grep -q -- '--force mk-cli' "$T/cargo.log" 2>/dev/null \
   && printf '%s' "$out" | grep -q 'which no cargo record in .* claims (--force)'; then
    ok "--from-source over an installer-copied binary: cargo gets --force"
else bad "--from-source after a binary install (rc=$rc): $out / $(cat "$T/cargo.log" 2>/dev/null)"; fi
rm -f "$T/cargo.log"
printf '[v1]\n"mk-cli 0.13.0 (git+https://github.com/bg002h/mnemonic-key?tag=%s#0feaaaa9)" = ["mk"]\n' "$MK_TAG" > "$T/root/.crates.toml"
out=$(PATH="$T/stub:$PATH" MNEMONIC_INSTALL_PLATFORM=linux-x86_64-gnu \
      sh "$INSTALL_SH" --only mk --no-man --root "$T/root" --from-source 2>&1) || true
if ! grep -q -- '--force' "$T/cargo.log" 2>/dev/null; then
    ok "--from-source over a cargo-tracked binary: no --force"
else bad "tracked binary was forced: $(cat "$T/cargo.log")"; fi

# 17. exactly one binary: two copies that both print the right version.
fresh_release
mkdir -p "$T/pkg/sub"; cp "$T/pkg/mk" "$T/pkg/sub/mk"
rearchive mk sub/mk LICENSE
run_case twobins linux-x86_64-gnu
if [ "$rc" -ne 0 ] && ! installed && printf '%s' "$out" | grep -q "expected exactly one 'mk'"; then
    ok "two 'mk' in the archive: refused, nothing installed"
else bad "two binaries (rc=$rc): $out"; fi

# 18. the version match is exact, not a prefix.
rerelease "echo \"mk $MK_VER-rc1\""
run_case rc1 linux-x86_64-gnu
if [ "$rc" -ne 0 ] && ! installed && printf '%s' "$out" | grep -q "did not print 'mk $MK_VER'"; then
    ok "binary printing 'mk $MK_VER-rc1': refused (exact match)"
else bad "prefix version (rc=$rc): $out"; fi

# 19. an asset that matches its checksum but does not unpack.
fresh_release
head -c 4096 /dev/urandom > "$REL/$ASSET"
( cd "$REL" && sha256sum "$ASSET" > SHA256SUMS.x86_64 )
run_case badtar linux-x86_64-gnu
if [ "$rc" -ne 0 ] && ! installed && printf '%s' "$out" | grep -q "cannot unpack $ASSET"; then
    ok "checksummed asset that is not an archive: 'cannot unpack', nothing installed"
else bad "unpack failure (rc=$rc): $out"; fi

# 20. Darwin detection, both arches.
darwin() {
    mkdir -p "$T/dw"
    printf '#!/bin/sh\ncase "$1" in -s) echo Darwin ;; -m) echo %s ;; esac\n' "$1" > "$T/dw/uname"
    chmod +x "$T/dw/uname"
    PATH="$T/dw:$PATH" MNEMONIC_INSTALL_PLATFORM='' sh "$INSTALL_SH" --list | sed -n 's/^platform: //p'
}
d="$(darwin arm64) $(darwin x86_64)"
if [ "$d" = "macos-aarch64 macos-x86_64" ]; then ok "Darwin detection: arm64 -> macos-aarch64, x86_64 -> macos-x86_64"
else bad "Darwin detection: got '$d'"; fi

# 21. SIGTERM mid-download stops the run (the draft's trap cleaned up and
#     carried on to the next component).
fresh_release
rm -rf "$T/root"; rm -f "$T/curl.log"
( PATH="$T/stub:$PATH" MNEMONIC_INSTALL_PLATFORM=linux-x86_64-gnu CURL_STUB_SLEEP=2 \
    exec sh "$INSTALL_SH" --only ms,mk --no-man --root "$T/root" > "$T/term.out" 2>&1 ) &
tpid=$!
sleep 1; kill -TERM "$tpid" 2>/dev/null; rc=0; wait "$tpid" || rc=$?
if [ "$rc" -eq 143 ] && ! grep -q 'mnemonic-key' "$T/curl.log" 2>/dev/null && ! installed; then
    ok "SIGTERM during a download: exit 143, the next component is never started"
else bad "SIGTERM (rc=$rc, downloads: $(cat "$T/curl.log" 2>/dev/null | tr '\n' ' ')): $(cat "$T/term.out")"; fi

# 22. every platform mapping, exactly (a swap that still passes its checksum,
#     e.g. macOS arm64 given the amd64 build, or the GUI glibc/musl pair,
#     must fail here). Expected names were checked with `file` against the
#     real assets on 2026-09-24. Versions come from the pin table.
pin_ver() { sed -n "s/.*|https:\/\/github.com\/bg002h\/$1|[^|]*-v\([0-9.]*\)|.*/\1/p" "$INSTALL_SH"; }
TV=$(pin_ver mnemonic-toolkit); DV=$(pin_ver descriptor-mnemonic); SV=$(pin_ver mnemonic-secret)
KV=$(pin_ver mnemonic-key); GV=$(pin_ver mnemonic-gui)
expect_map() {
    case "$1" in
        linux-x86_64-gnu)   echo "mnemonic-$TV-x86_64-linux-musl.tar.gz md-$DV-linux-amd64.tar.gz ms-$SV-x86_64-linux-musl.tar.gz mk-$KV-x86_64-linux-musl.tar.gz mnemonic-gui-v$GV-x86_64-linux.tar.gz" ;;
        linux-x86_64-musl)  echo "mnemonic-$TV-x86_64-linux-musl.tar.gz source ms-$SV-x86_64-linux-musl.tar.gz mk-$KV-x86_64-linux-musl.tar.gz source" ;;
        linux-aarch64-gnu)  echo "mnemonic-$TV-aarch64-linux-musl.tar.gz md-$DV-aarch64-linux-musl.tar.gz ms-$SV-aarch64-linux-musl.tar.gz mk-$KV-aarch64-linux-musl.tar.gz mnemonic-gui-v$GV-aarch64-linux.tar.gz" ;;
        linux-aarch64-musl) echo "mnemonic-$TV-aarch64-linux-musl.tar.gz md-$DV-aarch64-linux-musl.tar.gz ms-$SV-aarch64-linux-musl.tar.gz mk-$KV-aarch64-linux-musl.tar.gz source" ;;
        macos-x86_64)       echo "mnemonic-$TV-macos-amd64.tar.gz md-$DV-macos-amd64.tar.gz ms-$SV-macos-amd64.tar.gz mk-$KV-macos-amd64.tar.gz mnemonic-gui-v$GV-x86_64-macos.tar.gz" ;;
        macos-aarch64)      echo "mnemonic-$TV-macos-arm64.tar.gz md-$DV-macos-arm64.tar.gz ms-$SV-macos-arm64.tar.gz mk-$KV-macos-arm64.tar.gz mnemonic-gui-v$GV-aarch64-macos.tar.gz" ;;
        windows-x86_64)     echo "mnemonic-$TV-windows-amd64.zip md-$DV-windows-amd64.zip ms-$SV-windows-amd64.zip mk-$KV-windows-amd64.zip mnemonic-gui-v$GV-x86_64-windows.zip" ;;
        freebsd-x86_64)     echo "source source source source source" ;;
    esac
}
mapbad=0
for p in linux-x86_64-gnu linux-x86_64-musl linux-aarch64-gnu linux-aarch64-musl macos-x86_64 macos-aarch64 windows-x86_64 freebsd-x86_64; do
    got=$(MNEMONIC_INSTALL_PLATFORM=$p MNEMONIC_INSTALL_GLIBC=99.0 sh "$INSTALL_SH" --list \
          | awk 'NR > 3 { print ($NF == "source)" ? "source" : $NF) }' | tr '\n' ' ' | sed 's/ $//')
    want=$(expect_map "$p")
    [ "$got" = "$want" ] || { bad "mapping $p: got '$got', want '$want'"; mapbad=1; }
done
[ "$mapbad" -eq 0 ] && ok "every platform mapping matches the measured table (8 platforms x 5)"

# 23. the run check applies to Windows CLI binaries too (only the Windows
#     GUI is exempt).
fresh_release
WZIP="mk-$MK_VER-windows-amd64.zip"
printf '#!/bin/sh\necho "mk 0.0.1"\n' > "$T/pkg/mk.exe"
python3 - "$REL/$WZIP" "$T/pkg/mk.exe" <<'PY'
import sys, zipfile
z = zipfile.ZipFile(sys.argv[1], "w"); z.write(sys.argv[2], "mk.exe"); z.close()
PY
( cd "$REL" && sha256sum "$WZIP" > SHA256SUMS.portable )
rm -rf "$T/root"; rc=0
out=$(PATH="$T/stub:$PATH" MNEMONIC_INSTALL_PLATFORM=windows-x86_64 \
      sh "$INSTALL_SH" --only mk --no-man --root "$T/root" 2>&1) || rc=$?
if [ "$rc" -ne 0 ] && [ ! -e "$T/root/bin/mk.exe" ] && printf '%s' "$out" | grep -q "REFUSED $WZIP: 'mk.exe --version'"; then
    ok "Windows CLI binary with the wrong version: refused (run check not skipped)"
else bad "windows run check (rc=$rc): $out"; fi

# 24. sums matching is by exact name: a decoy line whose name merely contains
#     the asset name (listed first, wrong digest) must not be used.
fresh_release
( cd "$REL" && { echo "0000000000000000000000000000000000000000000000000000000000000000  old/$ASSET"; sha256sum "$ASSET"; } > SHA256SUMS.x86_64 )
run_case decoy linux-x86_64-gnu
if [ "$rc" -eq 0 ] && installed; then ok "sums decoy 'old/<asset>' ignored; exact entry used"
else bad "sums decoy (rc=$rc): $out"; fi

# 25. N5: the same name listed twice with different digests -> refused.
fresh_release
( cd "$REL" && { sha256sum "$ASSET"; echo "1111111111111111111111111111111111111111111111111111111111111111  $ASSET"; } > SHA256SUMS.x86_64 )
run_case dup linux-x86_64-gnu
if [ "$rc" -ne 0 ] && ! installed && printf '%s' "$out" | grep -q 'lists it with more than one digest'; then
    ok "asset listed with two different digests: refused"
else bad "duplicate digests (rc=$rc): $out"; fi

# 26. N1: a directory where the binary goes -> refused, directory untouched.
fresh_release
rm -rf "$T/root"; mkdir -p "$T/root/bin/mk"; rc=0
out=$(PATH="$T/stub:$PATH" MNEMONIC_INSTALL_PLATFORM=linux-x86_64-gnu \
      sh "$INSTALL_SH" --only mk --no-man --root "$T/root" 2>&1) || rc=$?
if [ "$rc" -ne 0 ] && [ -z "$(ls -A "$T/root/bin/mk")" ] && printf '%s' "$out" | grep -q 'is a directory; not replacing it'; then
    ok "directory at <root>/bin/mk: refused, left empty"
else bad "directory target (rc=$rc, contents: $(ls -A "$T/root/bin/mk")): $out"; fi

# 27. N3: --root "" is an error, not a silent ~/.cargo.
rc=0; out=$(sh "$INSTALL_SH" --only mk --no-man --dry-run --root "" 2>&1) || rc=$?
rc2=0; out2=$(sh "$INSTALL_SH" --only mk --no-man --dry-run --root= 2>&1) || rc2=$?
if [ "$rc" -eq 2 ] && [ "$rc2" -eq 2 ] && printf '%s' "$out" | grep -q 'non-empty'; then
    ok "--root \"\" and --root= are refused (exit 2)"
else bad "--root empty (rc=$rc/$rc2): $out / $out2"; fi

# 28. M5: warn when <root>/bin is not on PATH, and only then.
fresh_release
run_case pathwarn linux-x86_64-gnu
w1=$(printf '%s' "$out" | grep -c 'is not on your PATH' || true)
rm -rf "$T/root"; rc=0
out=$(PATH="$T/root/bin:$T/stub:$PATH" MNEMONIC_INSTALL_PLATFORM=linux-x86_64-gnu \
      sh "$INSTALL_SH" --only mk --no-man --root "$T/root" 2>&1) || rc=$?
w2=$(printf '%s' "$out" | grep -c 'is not on your PATH' || true)
if [ "$w1" -eq 1 ] && [ "$w2" -eq 0 ]; then ok "PATH warning: printed when <root>/bin is off PATH, not when on it"
else bad "PATH warning: off-PATH=$w1 on-PATH=$w2"; fi

# 29. M6: no cargo and a component that must build from source -> the error
#     names --exclude for exactly those components.
mkdir -p "$T/nocargo"
for tool in sh cut sed tr grep awk getconf uname ldd ls head wc sort find cat [ test echo printf; do
    real=$(command -v "$tool") && ln -sf "$real" "$T/nocargo/$tool"
done
rc=0; out=$(PATH="$T/nocargo" MNEMONIC_INSTALL_PLATFORM=linux-x86_64-musl \
    "$T/nocargo/sh" "$INSTALL_SH" --no-man --root "$T/root" 2>&1) || rc=$?
if [ "$rc" -eq 1 ] && printf '%s' "$out" | grep -q 're-run with --exclude md,mnemonic-gui'; then
    ok "no cargo on x86_64 musl: error suggests --exclude md,mnemonic-gui"
else bad "no-cargo message (rc=$rc): $out"; fi

# 30. --root with a space reaches cargo as ONE argument (the source path used
#     to word-split it). The stub cargo records one argument per line.
printf '#!/bin/sh\nfor a in "$@"; do printf "%%s\\n" "$a"; done > "%s/cargo.argv"\n' "$T" > "$T/stub/cargo-argv"
mkdir -p "$T/argv"; cp "$T/stub/cargo-argv" "$T/argv/cargo"; chmod +x "$T/argv/cargo"
rm -f "$T/cargo.argv"
PATH="$T/argv:$T/stub:$PATH" MNEMONIC_INSTALL_PLATFORM=freebsd-x86_64 \
    sh "$INSTALL_SH" --only mk --no-man --root "$T/sp ace" >/dev/null 2>&1 || true
if grep -qx -- "--root" "$T/cargo.argv" 2>/dev/null && grep -qx -- "$T/sp ace" "$T/cargo.argv"; then
    ok "--root with a space: passed to cargo as one argument"
else bad "--root with a space: cargo argv was: $(tr '\n' '|' < "$T/cargo.argv" 2>/dev/null)"; fi
# ... and with no --root, cargo still gets the root this script inspects
# ($CARGO_INSTALL_ROOT here), not whatever cargo's own config would pick.
rm -f "$T/cargo.argv"
CARGO_INSTALL_ROOT="$T/cir" PATH="$T/argv:$T/stub:$PATH" MNEMONIC_INSTALL_PLATFORM=freebsd-x86_64 \
    sh "$INSTALL_SH" --only mk --no-man >/dev/null 2>&1 || true
if grep -qx -- "--root" "$T/cargo.argv" 2>/dev/null && grep -qx -- "$T/cir" "$T/cargo.argv"; then
    ok "no --root: cargo is given --root \$CARGO_INSTALL_ROOT explicitly"
else bad "implicit root: cargo argv was: $(tr '\n' '|' < "$T/cargo.argv" 2>/dev/null)"; fi

# 31-36. NEW-1 (fold-2 re-review): --from-source must never auto-force over a
#     binary that cargo tracks under ANOTHER package. The .crates.toml below is
#     a real cargo-written file (paths shortened): several packages, one of
#     them with a multi-line bin array, as cargo writes it.
src_case() {  # src_case <crates.toml content or -> [extra install.sh args...]
    _toml=$1; shift
    rm -rf "$T/root"; mkdir -p "$T/root/bin"; rm -f "$T/cargo.log"
    printf '#!/bin/sh\necho "I am someone else"\n' > "$T/root/bin/mk"; chmod +x "$T/root/bin/mk"
    [ "$_toml" = - ] || printf '%s\n' "$_toml" > "$T/root/.crates.toml"
    rc=0
    out=$(PATH="$T/stub:$PATH" MNEMONIC_INSTALL_PLATFORM=linux-x86_64-gnu \
          sh "$INSTALL_SH" --only mk --no-man --root "$T/root" --from-source "$@" 2>&1) || rc=$?
}
mk_untouched() { [ "$("$T/root/bin/mk")" = "I am someone else" ]; }
REAL_TOML='[v1]
"fake-mk 0.1.0 (path+file:///home/user/src/fake-mk)" = ["mk"]
"ms-cli 0.19.0 (git+https://github.com/bg002h/mnemonic-secret?tag=ms-cli-v0.19.0#4f1e99eb05eefac7fd9d8eb9087ee1f4c6aaea07)" = ["ms"]
"multi 2.3.4 (path+file:///home/user/src/multi)" = [
    "alpha",
    "beta",
]'

# 31. another package owns mk -> refused, cargo never run, file untouched, owner named.
src_case "$REAL_TOML"
if [ "$rc" -ne 0 ] && [ ! -e "$T/cargo.log" ] && mk_untouched \
   && printf '%s' "$out" | grep -q "belongs to cargo package 'fake-mk 0.1.0 (path+file:///home/user/src/fake-mk)', not mk-cli" \
   && printf '%s' "$out" | grep -q 'cargo uninstall --root .* fake-mk'; then
    ok "mk owned by another cargo package (fake-mk): refused, owner named, cargo not run"
else bad "other owner (rc=$rc, cargo: $(cat "$T/cargo.log" 2>/dev/null)): $out"; fi

# 32. same, owner inside a multi-line array (as cargo writes several bins).
MULTI_TOML='[v1]
"ms-cli 0.19.0 (git+https://github.com/bg002h/mnemonic-secret?tag=ms-cli-v0.19.0#4f1e99eb)" = ["ms"]
"tools 1.0.0 (registry+https://github.com/rust-lang/crates.io-index)" = [
    "alpha",
    "mk",
]'
src_case "$MULTI_TOML"
if [ "$rc" -ne 0 ] && [ ! -e "$T/cargo.log" ] && mk_untouched \
   && printf '%s' "$out" | grep -q "belongs to cargo package 'tools 1.0.0"; then
    ok "mk owned via a multi-line bin array: refused"
else bad "multi-line owner (rc=$rc): $out"; fi

# 33. a user-supplied --force is the one override.
src_case "$REAL_TOML" --force
if [ "$rc" -eq 0 ] && grep -q -- '--force mk-cli' "$T/cargo.log" 2>/dev/null \
   && printf '%s' "$out" | grep -q "force given: replacing .*owned by cargo package 'fake-mk"; then
    ok "user --force overrides another package's mk, and says whose"
else bad "user --force (rc=$rc): $out / $(cat "$T/cargo.log" 2>/dev/null)"; fi

# 34. cargo tracks mk as mk-cli itself -> no --force (cargo upgrades or skips).
src_case '[v1]
"mk-cli 0.12.0 (git+https://github.com/bg002h/mnemonic-key?tag=mk-cli-v0.12.0#b0abc867)" = ["mk"]
"multi 2.3.4 (path+file:///home/user/src/multi)" = [
    "alpha",
    "beta",
]'
if [ "$rc" -eq 0 ] && [ -e "$T/cargo.log" ] && ! grep -q -- '--force' "$T/cargo.log"; then
    ok "mk tracked as mk-cli: cargo run without --force"
else bad "own package (rc=$rc): $out / $(cat "$T/cargo.log" 2>/dev/null)"; fi

# 35. records present but unreadable with certainty -> never force.
for bad_toml in 'not toml at all' '"fake-mk 0.1.0 (x)" = ["mk"]' '[v1]
"half = ["mk"' '[v1]
"x 1 (y)" = [
    "mk",'; do
    src_case "$bad_toml"
    if [ -e "$T/cargo.log" ] && ! grep -q -- '--force' "$T/cargo.log" \
       && printf '%s' "$out" | grep -q 'could not be read with certainty; not forcing'; then :
    else bad "malformed .crates.toml '$(printf '%s' "$bad_toml" | head -n1)': forced or no note: $out / $(cat "$T/cargo.log" 2>/dev/null)"; fi
done
ok "malformed .crates.toml (4 shapes): cargo run without --force"
#     .crates2.json without .crates.toml, and .crates.toml/.crates2.json disagreeing.
src_case -
printf '{"installs":{"fake-mk 0.1.0 (path+file:///x)":{"bins":["mk"]}}}\n' > "$T/root/.crates2.json"
rm -f "$T/cargo.log"; rc=0
out=$(PATH="$T/stub:$PATH" MNEMONIC_INSTALL_PLATFORM=linux-x86_64-gnu \
      sh "$INSTALL_SH" --only mk --no-man --root "$T/root" --from-source 2>&1) || rc=$?
j1=$(grep -c -- '--force' "$T/cargo.log" 2>/dev/null || true)
printf '[v1]\n"multi 2.3.4 (path+file:///x)" = ["alpha"]\n' > "$T/root/.crates.toml"
rm -f "$T/cargo.log"
out=$(PATH="$T/stub:$PATH" MNEMONIC_INSTALL_PLATFORM=linux-x86_64-gnu \
      sh "$INSTALL_SH" --only mk --no-man --root "$T/root" --from-source 2>&1) || rc=$?
j2=$(grep -c -- '--force' "$T/cargo.log" 2>/dev/null || true)
if [ "$j1" = 0 ] && [ "$j2" = 0 ]; then
    ok ".crates2.json alone, or naming mk when .crates.toml does not: no --force"
else bad ".crates2.json cases forced (json-only=$j1, disagree=$j2)"; fi

# 36. no cargo record at all (the binary-install case): --force, with a note
#     that no longer claims "cargo does not track" when that is unknown.
src_case -
if [ "$rc" -eq 0 ] && grep -q -- '--force mk-cli' "$T/cargo.log" 2>/dev/null \
   && printf '%s' "$out" | grep -q 'which no cargo record in .* claims (--force)'; then
    ok "no cargo records: the installer-copied binary is replaced (--force)"
else bad "no records (rc=$rc): $out"; fi

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
