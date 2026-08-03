#!/bin/sh
# install-msrv-guard.test.sh — regression harness for the GUI rustc-MSRV guard
# in scripts/install.sh (wave3def L5).
#
# WHY THIS EXISTS. The L5 spec waived a committed test ("a self-contained bash
# assertion in the PR description is acceptable"), so the guard shipped with
# ZERO regression coverage: a 2026-08-03 post-implementation audit deleted the
# entire guard block and every check in the repo stayed green. install.sh gains
# man-page and pin-bump churn every cycle, so a silent drop is a live risk —
# rustc 1.85-1.87 users would regress to the raw exit-101 mid-loop failure the
# guard exists to prevent.
#
# Asserts:
#   (a) `sh -n install.sh` parses clean;
#   (b) the guard block is PRESENT (constant + comparison + skip);
#   (c) BEHAVIOUR, on a stubbed old rustc: warns, skips the GUI, exits 0, and
#       still plans the 4 CLIs — the whole point of the guard;
#   (d) BEHAVIOUR, on a stubbed new rustc: no skip warning.
# (c)/(d) are the load-bearing ones — (b) alone would pass if the block were
# gutted, the signature-anchor false-PASS shape.
#
# Usage: sh scripts/install-msrv-guard.test.sh
set -eu

here=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
INSTALL_SH="$here/install.sh"

fail=0
ok()   { printf '  ok   %s\n' "$1"; }
bad()  { printf '  FAIL %s\n' "$1"; fail=1; }

printf '[install-msrv-guard.test] %s\n' "$INSTALL_SH"

# (a) parses clean
if sh -n "$INSTALL_SH" 2>/dev/null; then ok "sh -n parses clean"; else bad "sh -n failed"; fi

# (b) guard present
if grep -q 'GUI_MIN_RUSTC_MINOR=' "$INSTALL_SH" \
   && grep -q 'rustc_minor" -lt "\$GUI_MIN_RUSTC_MINOR' "$INSTALL_SH"; then
  ok "guard constant + comparison present"
else
  bad "GUI_MIN_RUSTC_MINOR constant or its comparison is MISSING from install.sh"
fi

# stub a rustc of a chosen version on PATH, run install.sh --dry-run-less plan
run_with_rustc() {
  _ver="$1"; shift
  _tmp=$(mktemp -d)
  cat > "$_tmp/rustc" <<STUB
#!/bin/sh
echo "rustc $_ver (stubbed)"
STUB
  chmod +x "$_tmp/rustc"
  # cargo stub: record invocations instead of installing anything.
  cat > "$_tmp/cargo" <<STUB
#!/bin/sh
echo "CARGO-INVOKED \$*"
STUB
  chmod +x "$_tmp/cargo"
  PATH="$_tmp:$PATH" sh "$INSTALL_SH" "$@" 2>&1 || true
  rm -rf "$_tmp"
}

# (c) old rustc -> warn + skip GUI + exit 0 + 4 CLIs still planned
out_old=$(run_with_rustc "1.85.0" --only mnemonic-gui)
if printf '%s' "$out_old" | grep -q 'mnemonic-gui needs rustc'; then
  ok "old rustc: emits the MSRV warning"
else
  bad "old rustc: NO MSRV warning — guard not firing"
fi
if printf '%s' "$out_old" | grep -q 'CARGO-INVOKED.*mnemonic-gui'; then
  bad "old rustc: GUI was installed anyway — guard not skipping"
else
  ok "old rustc: GUI skipped"
fi

# (d) new rustc -> no skip warning
out_new=$(run_with_rustc "1.99.0" --only mnemonic-gui)
if printf '%s' "$out_new" | grep -q 'mnemonic-gui needs rustc'; then
  bad "new rustc: MSRV warning fired spuriously — guard over-triggers"
else
  ok "new rustc: no spurious MSRV warning"
fi

if [ "$fail" -eq 0 ]; then
  printf '[install-msrv-guard.test] OK\n'
else
  printf '[install-msrv-guard.test] FAILED\n' >&2
  exit 1
fi
