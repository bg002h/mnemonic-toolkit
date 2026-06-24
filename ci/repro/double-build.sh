#!/usr/bin/env bash
# ci/repro/double-build.sh — the core reproducibility GATE (P1, task #23).
#
# WHAT IT PROVES (brainstorm §5 / IMPLEMENTATION_PLAN P1).
#   Builds the target binary at TWO DISTINCT absolute source paths, both remapped
#   to the SAME logical /build via --remap-path-prefix, and ASSERTS the two
#   outputs are byte-identical:
#     (i)   the two `mnemonic` binaries,
#     (ii)  the two `.tar.gz` packaged tarballs (gzip-pinned form),
#     (iii) the two libsecp256k1 `.o` objects.
#
# WHY TWO DISTINCT PATHS IS LOAD-BEARING (R0-r3-I1). The remap from-side only
# VARIES — and the remap is only proven effective — when the two REAL paths
# differ. A same-path A/B would match trivially WITHOUT exercising the remap, so
# the gate could pass while --remap-path-prefix has silently regressed and the
# shipped binary has resumed leaking $HOME. Hence /build-a/src vs /build-b/src.
#
# THIS IS THE TEST (TDD). Authored + committed BEFORE the env wiring. Against the
# UN-remapped baseline it MUST RED (binaries differ at byte ~41 — the absolute
# build-path leak in .rodata). Once CARGO_BUILD_RUSTFLAGS carries the remap, it
# GREENs. CI runs it in the digest-pinned container at the fixed paths.
#
# USAGE.
#   ci/repro/double-build.sh <path-a> <path-b>
# where <path-a>/<path-b> are two DISTINCT absolute dirs into which the SAME
# source tree is materialized (the caller copies the checkout into each). The
# remap maps each real path to /build. The script is the single source of truth
# for the build command (the three-block --config [source] activation, the env
# pins) so the gate, the release re-home, and the verify doc all stay in sync.
#
# ENV (set by the workflow at the fixed container layout):
#   SOURCE_DATE_EPOCH   - pinned epoch (git show -s --format=%ct <SHA>)
#   CARGO_HOME          - /cargo (remapped)
#   TARGET              - e.g. x86_64-unknown-linux-musl
#   CRATE / BIN         - mnemonic-toolkit / mnemonic
#   ARCH                - x86_64 (artifact label)
#   VER                 - version label for the tarball name (default 0.0.0-repro)
# LC_ALL / TZ are inherited from the job env. The script SELF-CONSTRUCTS the
# remap (CARGO_BUILD_RUSTFLAGS) and CFLAGS per leg — it does NOT rely on the job
# setting CARGO_BUILD_RUSTFLAGS (the gate job deliberately does not; any caller
# value is appended after the per-leg remap, but normally there is none).
#
# This is WHY: each leg's remap from-side is the leg's OWN real path
# (/build-a/src, /build-b/src), prepended below so BOTH map to /build. That is
# the only place the from-side varies per leg — the load-bearing construction.

set -euo pipefail

PATH_A="${1:?usage: double-build.sh <path-a> <path-b>}"
PATH_B="${2:?usage: double-build.sh <path-a> <path-b>}"

TARGET="${TARGET:-x86_64-unknown-linux-musl}"
CRATE="${CRATE:-mnemonic-toolkit}"
BIN="${BIN:-mnemonic}"
ARCH="${ARCH:-x86_64}"
VER="${VER:-0.0.0-repro}"
: "${SOURCE_DATE_EPOCH:?SOURCE_DATE_EPOCH must be set (pinned epoch)}"

if [ "$PATH_A" = "$PATH_B" ]; then
  echo "::error::double-build requires TWO DISTINCT paths (got identical '$PATH_A') — a same-path A/B does not exercise --remap-path-prefix (R0-r3-I1)." >&2
  exit 2
fi

# The job-scoped three-block [source] activation (THE REFINEMENT; R0-r5-C1 — the
# git-fork block is MANDATORY: the two-block form FAILS --offline because the
# miniscript [patch.crates-io] git dep has its own git+…?rev=… source key not
# served by source.crates-io). Copy-EXACT; this is the authoritative recipe.
MINISCRIPT_REV="95fdd1c5773bd918c574d2225787973f63e16a66"
SRC_CONFIG=(
  --config 'source.crates-io.replace-with="vendored-sources"'
  --config "source.\"git+https://github.com/rust-bitcoin/rust-miniscript?rev=${MINISCRIPT_REV}\".git=\"https://github.com/rust-bitcoin/rust-miniscript\""
  --config "source.\"git+https://github.com/rust-bitcoin/rust-miniscript?rev=${MINISCRIPT_REV}\".rev=\"${MINISCRIPT_REV}\""
  --config "source.\"git+https://github.com/rust-bitcoin/rust-miniscript?rev=${MINISCRIPT_REV}\".replace-with=\"vendored-sources\""
  --config 'source.vendored-sources.directory="vendor"'
)

# Build one leg at a real path that is remapped to /build. Emits, into $outdir,
# the built binary, the libsecp .o, and the gzip-pinned tarball.
build_leg() {
  local real_root="$1" outdir="$2"
  mkdir -p "$outdir"

  # The remap FROM-side is THIS leg's real path → /build (the only per-leg
  # variance). $CARGO_HOME (/cargo) is already remapped to itself (fixed).
  # Prepend to any inherited CARGO_BUILD_RUSTFLAGS the caller set.
  local leg_rustflags="--remap-path-prefix=${real_root}=/build --remap-path-prefix=${CARGO_HOME:-/cargo}=/cargo ${CARGO_BUILD_RUSTFLAGS:-}"
  local leg_cflags="-ffile-prefix-map=${real_root}=/build -ffile-prefix-map=${CARGO_HOME:-/cargo}=/cargo"

  (
    cd "$real_root"
    umask 022
    env \
      CARGO_BUILD_RUSTFLAGS="$leg_rustflags" \
      CFLAGS="$leg_cflags" \
      "CFLAGS_${TARGET//-/_}=$leg_cflags" \
      LC_ALL=C TZ=UTC \
      cargo build --locked --offline --release \
        --target "$TARGET" -p "$CRATE" --bin "$BIN" \
        "${SRC_CONFIG[@]}"
  )

  cp "$real_root/target/$TARGET/release/$BIN" "$outdir/$BIN"

  # Locate the libsecp256k1 object (isolates a C-frontier failure from Rust).
  local o
  o="$(find "$real_root/target/$TARGET/release/build" \
        -path '*secp256k1-sys-*/out/*.o' -print -quit 2>/dev/null || true)"
  if [ -n "$o" ]; then
    cp "$o" "$outdir/libsecp.o"
  else
    echo "::warning::no secp256k1-sys *.o found under $real_root (object-level check skipped)" >&2
  fi

  # gzip-pinned packaging (R0-r2-I1): deterministic tar members + gzip -n -9.
  tar --sort=name --owner=0 --group=0 --numeric-owner \
      --mtime="@$SOURCE_DATE_EPOCH" \
      -cf - -C "$real_root/target/$TARGET/release" "$BIN" \
    | gzip -n -9 > "$outdir/${BIN}-${VER}-${ARCH}-linux-musl.tar.gz"
}

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
OUT_A="$WORK/out-a"
OUT_B="$WORK/out-b"

echo "== leg A: $PATH_A → /build =="
build_leg "$PATH_A" "$OUT_A"
echo "== leg B: $PATH_B → /build =="
build_leg "$PATH_B" "$OUT_B"

fail=0

echo "== (i) binaries byte-identical =="
if cmp "$OUT_A/$BIN" "$OUT_B/$BIN"; then
  echo "  OK: $BIN identical across $PATH_A and $PATH_B"
else
  echo "::error::binaries DIFFER across the two paths — --remap-path-prefix not load-bearing / regressed." >&2
  fail=1
fi

echo "== (ii) tarballs byte-identical =="
if cmp "$OUT_A/${BIN}-${VER}-${ARCH}-linux-musl.tar.gz" \
       "$OUT_B/${BIN}-${VER}-${ARCH}-linux-musl.tar.gz"; then
  echo "  OK: .tar.gz identical"
else
  echo "::error::tarballs DIFFER — packaging (tar/gzip) non-deterministic." >&2
  fail=1
fi

echo "== (iii) libsecp .o byte-identical =="
if [ -f "$OUT_A/libsecp.o" ] && [ -f "$OUT_B/libsecp.o" ]; then
  if cmp "$OUT_A/libsecp.o" "$OUT_B/libsecp.o"; then
    echo "  OK: libsecp .o identical (C frontier deterministic)"
  else
    echo "::error::libsecp .o DIFFER — secp256k1-sys cc-under-musl non-deterministic (path/__DATE__ leak)." >&2
    fail=1
  fi
else
  echo "::warning::libsecp .o missing on one/both legs — object-level (iii) not asserted." >&2
fi

# Expose the canonical artifact + its hash to the caller (P4 will compare the
# published SHA256SUMS against this). Copy leg-A out to a stable location.
if [ -n "${REPRO_OUT_DIR:-}" ]; then
  mkdir -p "$REPRO_OUT_DIR"
  cp "$OUT_A/$BIN" "$REPRO_OUT_DIR/$BIN"
  cp "$OUT_A/${BIN}-${VER}-${ARCH}-linux-musl.tar.gz" "$REPRO_OUT_DIR/"
  # Standard `sha256sum` two-space output, basename only (matches the release
  # leg's SHA256SUMS format so a verifier can `sha256sum -c` either).
  ( cd "$REPRO_OUT_DIR" \
    && sha256sum "${BIN}-${VER}-${ARCH}-linux-musl.tar.gz" > "SHA256SUMS.${ARCH}" )
  echo "== canonical artifact + hash written to $REPRO_OUT_DIR =="
  cat "$REPRO_OUT_DIR/SHA256SUMS.${ARCH}"
fi

if [ "$fail" -ne 0 ]; then
  echo "::error::double-build GATE FAILED — build is NOT bit-for-bit reproducible across distinct paths." >&2
  exit 1
fi
echo "== double-build GATE PASSED: byte-identical across two distinct paths =="
