#!/usr/bin/env bash
# ci/apt-cache.test.sh -- scenario tests for ci/apt-cache.sh.
#
# Runs the script against stand-in `sudo` and `apt-get` programs (no apt, no
# root, no network). Covers the cache-key digest, the plan file, pruning of
# stale debs, and that a failing apt step fails the script. It does NOT cover
# real apt behaviour (hash re-verification of cached debs, _apt sandboxing);
# that is shown by the PR's own CI runs. Exit 0 = every scenario passed.
set -eu
here=$(cd "$(dirname "$0")" && pwd)
script=$here/apt-cache.sh
tmp=$(mktemp -d "${APT_CACHE_TEST_TMP:-${TMPDIR:-/tmp}}/apt-cache-test.XXXXXX")
trap 'rm -rf "$tmp"' EXIT
mkdir "$tmp/bin"

# sudo: run the command as-is.
cat > "$tmp/bin/sudo" <<'SH'
#!/usr/bin/env bash
exec "$@"
SH
# apt-get: logs its arguments to $APT_LOG. `update` exits $FAKE_UPDATE_RC.
# `install --print-uris` prints $FAKE_URIS (exit $FAKE_PLAN_RC). A real
# `install` exits $FAKE_INSTALL_RC, else creates one file per filename in
# $FAKE_URIS inside the Dir::Cache::archives directory it was given.
cat > "$tmp/bin/apt-get" <<'SH'
#!/usr/bin/env bash
printf '%s\n' "$*" >> "$APT_LOG"
case "$*" in
  *update*) exit "${FAKE_UPDATE_RC:-0}" ;;
  *--print-uris*)
    [ "${FAKE_PLAN_RC:-0}" -eq 0 ] || exit "$FAKE_PLAN_RC"
    cat "$FAKE_URIS"; exit 0 ;;
esac
[ "${FAKE_INSTALL_RC:-0}" -eq 0 ] || exit "$FAKE_INSTALL_RC"
dir=
for a in "$@"; do case "$a" in Dir::Cache::archives=*) dir=${a#*=} ;; esac; done
[ -n "$dir" ] || { echo "no archives dir" >&2; exit 2; }
mkdir -p "$dir/partial"; : > "$dir/lock"
while read -r _ name _; do : > "$dir/$name"; done < "$FAKE_URIS"
SH
chmod +x "$tmp/bin/sudo" "$tmp/bin/apt-get"
export PATH="$tmp/bin:$PATH" APT_LOG="$tmp/apt.log" FAKE_URIS="$tmp/uris.txt"

fails=0
ok()  { printf 'ok   %s\n' "$1"; }
bad() { printf 'FAIL %s\n' "$1"; fails=$((fails + 1)); }
check() { if eval "$2"; then ok "$1"; else bad "$1 :: $2"; fi; }

U1="'http://a.example/pool/pandoc_3.1_amd64.deb' pandoc_3.1_amd64.deb 100 SHA512:aaa"
U2="'http://a.example/pool/texlive-base_2.0%3a1_all.deb' texlive-base_2.0%3a1_all.deb 200 SHA512:bbb"
printf '%s\n%s\n' "$U1" "$U2" > "$FAKE_URIS"

plan() { # plan <dir> <pkgs...> -> stdout of the script
  bash "$script" plan "$@"
}

# --- plan: digest and pkgsum lines, plan file beside the cache dir
d=$tmp/c1
out=$(plan "$d" pandoc make)
digest1=$(printf '%s\n' "$out" | sed -n 's/^digest=//p')
pkgsum1=$(printf '%s\n' "$out" | sed -n 's/^pkgsum=//p')
check "plan prints a 64-hex digest" '[[ $digest1 =~ ^[0-9a-f]{64}$ ]]'
check "plan prints a pkgsum" '[[ $pkgsum1 =~ ^[0-9a-f]{12}$ ]]'
check "plan runs apt-get update first" 'head -1 "$APT_LOG" | grep -q update'
check "plan writes the plan file beside the dir" '[ -s "$d.plan" ]'
check "plan file lists both deb filenames" 'grep -q "^pandoc_3.1_amd64.deb 100 SHA512:aaa$" "$d.plan" && grep -q "^texlive-base_2.0%3a1_all.deb 200 " "$d.plan"'
check "plan resolves with --no-install-recommends and the packages" 'grep -q -- "--print-uris.*--no-install-recommends.* pandoc make" "$APT_LOG" || grep -q -- "--no-install-recommends.*--print-uris.* pandoc make" "$APT_LOG"'

# --- digest is order- and mirror-insensitive, hash-sensitive
printf '%s\n%s\n' "$U2" "$U1" > "$FAKE_URIS"
out=$(plan "$tmp/c2" make pandoc)
check "digest ignores URI order" '[ "$(printf "%s\n" "$out" | sed -n "s/^digest=//p")" = "$digest1" ]'
check "pkgsum ignores package order" '[ "$(printf "%s\n" "$out" | sed -n "s/^pkgsum=//p")" = "$pkgsum1" ]'
printf '%s\n%s\n' "${U1/a.example/b.example}" "$U2" > "$FAKE_URIS"
out=$(plan "$tmp/c3" pandoc make)
check "digest ignores the mirror host" '[ "$(printf "%s\n" "$out" | sed -n "s/^digest=//p")" = "$digest1" ]'
printf '%s\n%s\n' "${U1/aaa/zzz}" "$U2" > "$FAKE_URIS"
out=$(plan "$tmp/c4" pandoc make)
check "digest changes when a deb hash changes" '[ "$(printf "%s\n" "$out" | sed -n "s/^digest=//p")" != "$digest1" ]'
out=$(plan "$tmp/c5" pandoc)
check "pkgsum changes with the package set" '[ "$(printf "%s\n" "$out" | sed -n "s/^pkgsum=//p")" != "$pkgsum1" ]'

# --- plan fails loudly
printf '%s\n%s\n' "$U1" "$U2" > "$FAKE_URIS"
check "plan fails when apt-get update fails" '! FAKE_UPDATE_RC=100 plan "$tmp/c6" pandoc >/dev/null 2>&1'
check "plan fails when --print-uris fails" '! FAKE_PLAN_RC=100 plan "$tmp/c7" pandoc >/dev/null 2>&1'
check "plan fails with no packages" '! plan "$tmp/c8" >/dev/null 2>&1'

# --- install: prune, clean lock dirs, pass the same packages
d=$tmp/c9
plan "$d" pandoc make >/dev/null
mkdir -p "$d/partial"; : > "$d/lock"; : > "$d/stale_1.0_all.deb"; : > "$d/pandoc_3.1_amd64.deb"
: > "$APT_LOG"
bash "$script" install "$d" pandoc make
check "install passes the archives dir to apt-get" 'grep -q "Dir::Cache::archives=$d " "$APT_LOG"'
check "install uses -y --no-install-recommends and the packages" 'grep -q -- "-y.* --no-install-recommends.* pandoc make\$\|--no-install-recommends.* -y.* pandoc make\$" "$APT_LOG"'
check "install removes a stale deb" '[ ! -e "$d/stale_1.0_all.deb" ]'
check "install keeps the planned debs" '[ -e "$d/pandoc_3.1_amd64.deb" ] && [ -e "$d/texlive-base_2.0%3a1_all.deb" ]'
check "install removes partial/ and lock" '[ ! -e "$d/partial" ] && [ ! -e "$d/lock" ]'

# --- install fails loudly
d=$tmp/c10
plan "$d" pandoc >/dev/null
check "install fails when apt-get install fails" '! FAKE_INSTALL_RC=100 bash "$script" install "$d" pandoc >/dev/null 2>&1'
check "install fails without a plan file" '! bash "$script" install "$tmp/c11" pandoc >/dev/null 2>&1'
check "unknown mode fails" '! bash "$script" frobnicate "$tmp/c12" pandoc >/dev/null 2>&1'

if [ "$fails" -ne 0 ]; then printf '%d scenario(s) failed\n' "$fails"; exit 1; fi
printf 'all scenarios passed\n'
