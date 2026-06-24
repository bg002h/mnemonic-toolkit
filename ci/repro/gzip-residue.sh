#!/usr/bin/env bash
# ci/repro/gzip-residue.sh — gzip-header determinism GATE (P1, task #23).
#
# WHY (brainstorm §3.13 / §5 step 6 / R0-r2-I1 + R0-r4-m4). The published
# SHA256SUMS.<arch> hashes a .tar.GZ, not a .tar. gzip embeds a 4-byte mtime
# (header offset 4-7) and a 1-byte OS field (offset 9). A non-`-n` build bakes
# the mtime; a divergent gzip build can vary the OS byte. Either ships a
# non-provenance tarball hash even with a byte-identical inner binary. So:
#   - ASSERT the gzip header MTIME field (offset 4-7) is ZERO  (proves -n).
#   - ASSERT the OS byte (offset 9) equals the PINNED value (default 03 = Unix),
#     primarily pinned by the container digest; this is the cheap residue check.
#
# gzip magic/method (offset 0-2 = 1f 8b 08) is also asserted as a sanity guard.
#
# USAGE.  ci/repro/gzip-residue.sh <file.tar.gz> [expected-os-byte-hex]
#   expected-os-byte-hex default = 03 (Unix; the observed value in the recon).

set -euo pipefail

F="${1:?usage: gzip-residue.sh <file.tar.gz> [expected-os-byte-hex]}"
EXPECT_OS="${2:-03}"

if [ ! -f "$F" ]; then
  echo "::error::gzip-residue: $F not found" >&2
  exit 1
fi

# Read the first 10 header bytes as space-separated 2-digit hex.
read -r -a hdr < <(od -An -tx1 -N10 "$F" | tr -s ' ' | sed 's/^ //')

if [ "${#hdr[@]}" -lt 10 ]; then
  echo "::error::gzip-residue: $F shorter than a 10-byte gzip header" >&2
  exit 1
fi

magic0="${hdr[0]}"; magic1="${hdr[1]}"; method="${hdr[2]}"
mtime="${hdr[4]}${hdr[5]}${hdr[6]}${hdr[7]}"
osbyte="${hdr[9]}"

fail=0

echo "== gzip magic/method =="
if [ "$magic0" = "1f" ] && [ "$magic1" = "8b" ] && [ "$method" = "08" ]; then
  echo "  OK: 1f 8b 08 (gzip, deflate)"
else
  echo "::error::not a gzip-deflate stream (magic ${magic0} ${magic1}, method ${method})" >&2
  fail=1
fi

echo "== gzip MTIME field (offset 4-7) must be 00000000 =="
if [ "$mtime" = "00000000" ]; then
  echo "  OK: mtime field zero (gzip -n honored)"
else
  echo "::error::gzip mtime field is ${mtime}, not zero — gzip -n not applied; tarball hash is non-reproducible." >&2
  fail=1
fi

echo "== gzip OS byte (offset 9) must equal pinned ${EXPECT_OS} =="
if [ "$osbyte" = "$EXPECT_OS" ]; then
  echo "  OK: OS byte ${osbyte}"
else
  echo "::error::gzip OS byte is ${osbyte}, expected ${EXPECT_OS} — divergent gzip build (container-digest drift)." >&2
  fail=1
fi

# Belt-and-suspenders: gzip -l also reports the stored timestamp.
if command -v gzip >/dev/null 2>&1; then
  echo "== gzip -l (informational) =="
  gzip -l "$F" 2>/dev/null || true
fi

if [ "$fail" -ne 0 ]; then
  echo "::error::gzip-residue GATE FAILED." >&2
  exit 1
fi
echo "== gzip-residue GATE PASSED =="
