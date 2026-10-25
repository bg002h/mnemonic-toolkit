#!/usr/bin/env bash
# ci/shared-drift.test.sh -- scenario tests for ci/shared-drift.sh.
#
# Serves this checkout's own five shared files, for every repo, through a
# stand-in for `gh`, then changes that copy one row at a time. Needs only
# bash and sha256sum. Never calls GitHub. Exit 0 = every row passed.
set -eu
here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/.." && pwd)
tmp=$(mktemp -d "${TMPDIR:-/tmp}/shared-drift-test.XXXXXX")
trap 'rm -rf "$tmp"' EXIT
repos="descriptor-mnemonic mnemonic-engrave mnemonic-key mnemonic-secret mnemonic-transaction"
files="ci/docs-only.sh ci/docs-only.test.sh ci/code-docs-check.sh ci/code-docs-check.test.sh ci/shared-ci.sha256"

# A stand-in for `gh`: reads the last argument
# (repos/<owner>/<repo>/contents/<file>), appends every argument to
# $GH_ARGS one per line, then prints $REMOTE/<repo>/<file> and exits 0, or
# exits 1 when that file is absent.
cat > "$tmp/gh" <<'EOF'
#!/usr/bin/env bash
for last; do :; done
printf '%s\n' "$@" >> "$GH_ARGS"
p=${last#repos/*/}
repo=${p%%/*}
file=${p#*/contents/}
[ -f "$REMOTE/$repo/$file" ] || exit 1
cat "$REMOTE/$repo/$file"
EOF
chmod +x "$tmp/gh"

fresh() { # fresh: rebuild $REMOTE from the checkout's five files, per repo
  rm -rf "$tmp/remote"
  for r in $repos; do
    for f in $files; do
      mkdir -p "$(dirname "$tmp/remote/$r/$f")"
      cp "$root/$f" "$tmp/remote/$r/$f"
    done
  done
}

run() { # run <dir>: run ci/shared-drift.sh from <dir>; sets $out and $rc
  : > "$tmp/ghargs"
  set +e
  out=$(cd "$1" && env -u SHARED_DRIFT_OWNER SHARED_DRIFT_GH="$tmp/gh" \
    GH_ARGS="$tmp/ghargs" REMOTE="$tmp/remote" bash ci/shared-drift.sh 2>&1)
  rc=$?
  set -e
}

has_line() { # has_line <text>: some line of $out is exactly <text>
  grep -qxF -- "$1" <<<"$out"
}
has_prefix() { # has_prefix <text>: some line of $out starts with <text>
  local l
  while IFS= read -r l; do
    [ "${l#"$1"}" = "$l" ] || return 0
  done <<<"$out"
  return 1
}

pass=0 fail=0
ok() { pass=$((pass + 1)); echo "ok   $1 ($(tail -1 <<<"$out"))"; }
bad() { fail=$((fail + 1)); echo "FAIL $1: exit $rc:"; printf '%s\n' "$out"; }

# D1: every remote file matches.
fresh; run "$root"
n=$(grep -c '^ok    ' <<<"$out" || true)
if [ "$rc" -eq 0 ] && [ "$n" -eq 25 ] && has_line 'shared-drift: PASS (5 repos, 5 files)'; then
  ok "D1 no change: 25 ok lines and PASS"
else bad "D1 no change ($n ok lines)"; fi
cp "$tmp/ghargs" "$tmp/ghargs.d1"

# D2: one remote file has one byte more.
fresh; printf 'x' >> "$tmp/remote/mnemonic-key/ci/docs-only.sh"; run "$root"
if [ "$rc" -eq 1 ] && has_prefix 'DRIFT mnemonic-key ci/docs-only.sh: ' &&
   has_line 'shared-drift: FAIL: 1 problem(s)'; then
  ok "D2 one byte appended remotely: DRIFT, 1 problem"
else bad "D2 one byte appended remotely"; fi

# D3: gh cannot fetch one remote file.
fresh; rm "$tmp/remote/mnemonic-engrave/ci/code-docs-check.sh"; run "$root"
if [ "$rc" -eq 1 ] && has_line 'ERROR mnemonic-engrave ci/code-docs-check.sh: cannot fetch'; then
  ok "D3 remote file absent: cannot fetch"
else bad "D3 remote file absent"; fi

# D4: one remote file is empty.
fresh; : > "$tmp/remote/descriptor-mnemonic/ci/shared-ci.sha256"; run "$root"
if [ "$rc" -eq 1 ] && has_prefix 'DRIFT descriptor-mnemonic ci/shared-ci.sha256: '; then
  ok "D4 remote file empty: DRIFT"
else bad "D4 remote file empty"; fi

# D5: run from a directory holding the check and the four shared files
# other than ci/docs-only.test.sh.
fresh; mkdir -p "$tmp/d5/ci"
cp "$root/ci/shared-drift.sh" "$tmp/d5/ci/"
for f in $files; do
  [ "$f" = ci/docs-only.test.sh ] || cp "$root/$f" "$tmp/d5/$f"
done
run "$tmp/d5"
if [ "$rc" -eq 1 ] && has_line 'ERROR mnemonic-secret ci/docs-only.test.sh: missing locally'; then
  ok "D5 local file missing: missing locally"
else bad "D5 local file missing"; fi

# D6: the arguments gh got in D1.
d6=0
for a in api -H 'Accept: application/vnd.github.raw' \
  repos/bg002h/mnemonic-transaction/contents/ci/shared-ci.sha256; do
  grep -qxF -- "$a" "$tmp/ghargs.d1" || { d6=1; echo "FAIL D6: gh never got the argument '$a'"; }
done
if [ "$d6" -eq 0 ]; then pass=$((pass + 1)); echo "ok   D6 gh got api, -H, the raw Accept header and the contents path"
else fail=$((fail + 1)); echo "gh arguments in D1:"; cat "$tmp/ghargs.d1"; fi

# D7: sha256sum fails. The check must not print PASS.
fresh; mkdir -p "$tmp/bin"
printf '#!/bin/sh\nexit 1\n' > "$tmp/bin/sha256sum"; chmod +x "$tmp/bin/sha256sum"
opath=$PATH; PATH="$tmp/bin:$PATH"; run "$root"; PATH=$opath
if [ "$rc" -eq 1 ] && ! has_prefix 'shared-drift: PASS'; then
  ok "D7 sha256sum fails: exit 1, no PASS"
else bad "D7 sha256sum fails"; fi

echo "shared-drift.test.sh: $pass passed, $fail failed"
[ "$fail" -eq 0 ]
