#!/usr/bin/env bash
# ci/repro/double-build.sh — the core reproducibility GATE (P1 x86_64; P2 aarch64).
#
# WHAT IT PROVES (brainstorm §5 / IMPLEMENTATION_PLAN P1/P2).
#   Builds the target binary at TWO DISTINCT absolute source paths, both remapped
#   to the SAME logical /build via --remap-path-prefix, and ASSERTS the two
#   outputs are byte-identical:
#     (i)   the two `mnemonic` binaries,
#     (ii)  the two `.tar.gz` packaged tarballs (gzip-pinned form),
#     (iii) the two libsecp256k1 `.o` objects.
#   BUILDER=cargo (x86_64, direct in the container) or BUILDER=cross (aarch64,
#   under QEMU with cross's digest-pinned toolchain — Cross.toml).
#
# WHY TWO DISTINCT PATHS IS LOAD-BEARING for x86_64 (R0-r3-I1). The remap
# from-side only VARIES — and the remap is only proven effective — when the two
# REAL paths differ. A same-path A/B would match trivially WITHOUT exercising the
# remap, so the gate could pass while --remap-path-prefix has silently regressed
# and the shipped binary has resumed leaking $HOME. Hence /build-a/src vs
# /build-b/src.
#
# CAVEAT for aarch64/cross (R0-I2). `cross` bind-mounts BOTH host paths to its
# FIXED internal /project (cross v0.2.5 src/docker/local.rs `-v <host_root>:
# /project`), so the two legs collapse to the same in-container path and this
# A/B is TRIVIALLY satisfied — WEAK evidence. The aarch64 load-bearing gate is
# the DIRECT residue/.comment assertions in cc-validate.sh (passthrough took
# effect; the pinned cross toolchain is in use; zero host-path/__DATE__ residue).
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

# BUILDER selects the compile front-end (P2): `cargo` (x86_64, direct in the
# digest-pinned container) or `cross` (aarch64, under QEMU with cross's
# digest-pinned bundled toolchain — Cross.toml). `cross` forwards the --config
# [source] overrides to the inner cargo and mounts the project (committed vendor/
# present); the determinism-bearing env reaches it ONLY via the Cross.toml
# [build.env] passthrough list. The TWO-DISTINCT-PATHS shape STILL runs under
# cross, but note (R0-I2): cross bind-mounts BOTH host paths to its FIXED
# internal /project (verified: cross v0.2.5 src/docker/local.rs), so the two
# legs collapse to the same in-container path → A/B-equality is TRIVIALLY
# satisfied and is WEAK evidence for aarch64; the load-bearing aarch64 gate is
# the direct residue/.comment assertions in cc-validate.sh, not this A/B.
BUILDER="${BUILDER:-cargo}"
# CROSS_INTERNAL_SRC — the FIXED path the cross container bind-mounts the project
# to (cross v0.2.5: `-v <host_root>:/project`). For BUILDER=cross the remap
# from-side MUST be this internal path, NOT the per-leg host path, because that
# is what the in-container compiler actually sees. Empty for the cargo leg ⇒ the
# per-leg host real path is used (the load-bearing variance for x86_64).
CROSS_INTERNAL_SRC="${CROSS_INTERNAL_SRC:-/project}"

if [ "$PATH_A" = "$PATH_B" ]; then
  echo "::error::double-build requires TWO DISTINCT paths (got identical '$PATH_A') — a same-path A/B does not exercise --remap-path-prefix (R0-r3-I1)." >&2
  exit 2
fi

# The job-scoped [source] activation (THE REFINEMENT; R0-r5-C1). PARAMETERIZED by
# MINISCRIPT_REV (P3a) so the SAME script serves the toolkit (THREE-block — the
# git-fork block is MANDATORY because the miniscript [patch.crates-io] git dep has
# its own git+…?rev=… source key not served by source.crates-io, so the two-block
# form FAILS --offline) AND the codec CLIs (md/ms/mk), which do NOT depend on the
# miniscript fork and need only the TWO-block form (crates-io + vendored-sources).
#
#   MINISCRIPT_REV  UNSET  ⇒ default to the toolkit rev (THREE-block — preserves
#                            the exact pre-P3a standalone behavior, byte-identical).
#   MINISCRIPT_REV  EMPTY  ⇒ TWO-block (no git-fork stanza) — the codec form.
#   MINISCRIPT_REV  set    ⇒ THREE-block keyed off that rev.
# (Single-dash `${VAR-default}` defaults ONLY when UNSET, so an explicit empty
# value from a codec caller is honored as the two-block selector.)
MINISCRIPT_REV="${MINISCRIPT_REV-95fdd1c5773bd918c574d2225787973f63e16a66}"
SRC_CONFIG=(
  --config 'source.crates-io.replace-with="vendored-sources"'
)
if [ -n "$MINISCRIPT_REV" ]; then
  SRC_CONFIG+=(
    --config "source.\"git+https://github.com/rust-bitcoin/rust-miniscript?rev=${MINISCRIPT_REV}\".git=\"https://github.com/rust-bitcoin/rust-miniscript\""
    --config "source.\"git+https://github.com/rust-bitcoin/rust-miniscript?rev=${MINISCRIPT_REV}\".rev=\"${MINISCRIPT_REV}\""
    --config "source.\"git+https://github.com/rust-bitcoin/rust-miniscript?rev=${MINISCRIPT_REV}\".replace-with=\"vendored-sources\""
  )
fi
SRC_CONFIG+=(
  --config 'source.vendored-sources.directory="vendor"'
)

# Build one leg at a real path that is remapped to /build. Emits, into $outdir,
# the built binary, the libsecp .o, and the gzip-pinned tarball.
build_leg() {
  local real_root="$1" outdir="$2"
  mkdir -p "$outdir"

  # The remap FROM-side is the path the COMPILER sees → /build.
  #   * cargo (x86_64): THIS leg's REAL host path ($real_root) — the per-leg
  #     variance that makes --remap-path-prefix load-bearing.
  #   * cross (aarch64): cross's FIXED internal mount ($CROSS_INTERNAL_SRC,
  #     /project) — both legs share it, so the remap does NOT vary per leg here;
  #     that is why aarch64's load-bearing proof is the residue/.comment gate,
  #     not this A/B (R0-I2).
  # $CARGO_HOME (/cargo) is already remapped to itself (fixed; cross sets /cargo).
  # Prepend to any inherited CARGO_BUILD_RUSTFLAGS the caller set.
  local remap_src="$real_root"
  if [ "$BUILDER" = "cross" ]; then
    remap_src="$CROSS_INTERNAL_SRC"
  fi
  local leg_rustflags="--remap-path-prefix=${remap_src}=/build --remap-path-prefix=${CARGO_HOME:-/cargo}=/cargo ${CARGO_BUILD_RUSTFLAGS:-}"
  local leg_cflags="-ffile-prefix-map=${remap_src}=/build -ffile-prefix-map=${CARGO_HOME:-/cargo}=/cargo"

  (
    cd "$real_root"
    umask 022
    # For cross, these NAME=VALUE assignments land in the `cross` process env,
    # which cross forwards into the QEMU container per Cross.toml passthrough.
    env \
      CARGO_BUILD_RUSTFLAGS="$leg_rustflags" \
      CFLAGS="$leg_cflags" \
      "CFLAGS_${TARGET//-/_}=$leg_cflags" \
      LC_ALL=C TZ=UTC \
      "$BUILDER" build --locked --offline --release \
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
