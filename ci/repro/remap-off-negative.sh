#!/usr/bin/env bash
# ci/repro/remap-off-negative.sh — the remap-off NEGATIVE drift-gate probe (P4).
#
# WHAT IT PROVES (brainstorm §P4 "remap-off negative" / IMPLEMENTATION_PLAN P4).
#   The POSITIVE gate (double-build.sh) proves the two binaries are byte-IDENTICAL
#   *with* --remap-path-prefix. That is necessary but NOT sufficient: if the remap
#   ever silently became a no-op (the path leak it guards no longer present, or
#   the flag dropped), the positive gate would STILL pass — byte-identity with a
#   hollow remap is indistinguishable from byte-identity with a load-bearing one.
#   This negative probe closes that gap: it runs the SAME two-distinct-path build
#   with the remap DISABLED (empty CARGO_BUILD_RUSTFLAGS, no -ffile-prefix-map) and
#   asserts the protection is LOAD-BEARING. If the un-remapped build is ALSO
#   reproducible / leak-free, the remap is hollow → this gate REDs, alerting that
#   the path-leak protection has become unnecessary (and the positive gate's GREEN
#   is no longer evidence the remap works).
#
# ARCH-AWARE (BUILDER cargo vs cross) — the assertion differs per builder because
# the two builders expose the path leak differently:
#
#   * BUILDER=cargo (x86_64, direct in the digest-pinned container). The compiler
#     sees each leg's OWN real host path (/build-a/src vs /build-b/src). With the
#     remap OFF, that distinct absolute path leaks into .rodata (the file!()/
#     panic-Location literal) → the two binaries DIFFER (the empirically-measured
#     byte-~41 divergence, recon §1.1). ASSERTION: the two no-remap binaries MUST
#     DIFFER. (If they are byte-identical even WITHOUT the remap, the remap is not
#     load-bearing → RED.)
#
#   * BUILDER=cross (aarch64, under QEMU). cross bind-mounts BOTH host paths to its
#     FIXED internal /project (cross v0.2.5 src/docker/local.rs), so the two legs
#     collapse to the SAME in-container path → a no-remap A/B is byte-IDENTICAL
#     regardless of the remap (the path that would vary per-leg never does). So an
#     A/B-difference assertion is structurally impossible to satisfy under cross —
#     the SAME reason double-build.sh notes A/B is weak evidence for aarch64
#     (R0-I2). Instead the cross negative asserts the no-remap build LEAKS the
#     /project (and CARGO_HOME) host path into the binary — residue the remap +
#     -ffile-prefix-map would strip. ASSERTION: the no-remap binary MUST contain
#     /project (or $CARGO_HOME) path residue. (If a no-remap cross build has ZERO
#     path residue, the remap is not load-bearing → RED.)
#
# Either way the gate REDs when the remap has become a no-op, and GREENs when the
# remap is doing real work (un-remapped build is non-reproducible / leaks paths).
#
# COST. One extra no-remap build pair (cargo) or one no-remap build (cross). It
# reuses the same source trees the positive gate already materialized at <path-a>/
# <path-b>; the positive gate's own target/ is left untouched (this probe builds
# into the SAME target dirs but is run AFTER the positive gate has asserted, so a
# rebuild there does not perturb the positive result). Keep it cheap: cargo builds
# the bin at two paths; cross builds once.
#
# USAGE.  ci/repro/remap-off-negative.sh <path-a> <path-b>
# Same two distinct absolute source dirs the positive gate used. ENV mirrors
# double-build.sh (TARGET / CRATE / BIN / ARCH / SOURCE_DATE_EPOCH / BUILDER /
# CROSS_INTERNAL_SRC / MINISCRIPT_REV / CARGO_HOME). The remap is DELIBERATELY
# NOT constructed — that is the whole point.

set -euo pipefail

PATH_A="${1:?usage: remap-off-negative.sh <path-a> <path-b>}"
PATH_B="${2:?usage: remap-off-negative.sh <path-a> <path-b>}"

TARGET="${TARGET:-x86_64-unknown-linux-musl}"
CRATE="${CRATE:-mnemonic-toolkit}"
BIN="${BIN:-mnemonic}"
ARCH="${ARCH:-x86_64}"
: "${SOURCE_DATE_EPOCH:?SOURCE_DATE_EPOCH must be set (pinned epoch)}"

BUILDER="${BUILDER:-cargo}"
CROSS_INTERNAL_SRC="${CROSS_INTERNAL_SRC:-/project}"

if [ "$PATH_A" = "$PATH_B" ]; then
  echo "::error::remap-off-negative requires TWO DISTINCT paths (got identical '$PATH_A')." >&2
  exit 2
fi

# The job-scoped [source] activation — IDENTICAL construction to double-build.sh /
# cc-validate.sh (so the negative build resolves the committed vendor/ offline the
# same way the positive build does; only the REMAP is dropped). MINISCRIPT_REV
# UNSET ⇒ toolkit rev (THREE-block); EMPTY ⇒ TWO-block (codec). Single-dash default
# fires ONLY when UNSET so a codec caller's explicit empty is honored.
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

# Build one leg with the remap DELIBERATELY DISABLED. No --remap-path-prefix, no
# -ffile-prefix-map — the un-remapped baseline the positive gate compares against.
# CARGO_BUILD_RUSTFLAGS is forced EMPTY for this probe (the brainstorm's literal
# "empty CARGO_BUILD_RUSTFLAGS for that probe"). Everything ELSE (epoch, locale,
# offline vendor resolution) is held identical so the ONLY removed variable is the
# remap — otherwise an unrelated non-determinism could mask the result.
build_leg_no_remap() {
  local real_root="$1" outdir="$2"
  mkdir -p "$outdir"
  (
    cd "$real_root"
    umask 022
    env \
      CARGO_BUILD_RUSTFLAGS="" \
      LC_ALL=C TZ=UTC \
      "$BUILDER" build --locked --offline --release \
        --target "$TARGET" -p "$CRATE" --bin "$BIN" \
        "${SRC_CONFIG[@]}"
  )
  cp "$real_root/target/$TARGET/release/$BIN" "$outdir/$BIN"
}

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

if [ "$BUILDER" = "cross" ]; then
  # ── aarch64/cross: A/B collapses to /project → assert PATH RESIDUE instead ────
  echo "== remap-off NEGATIVE (cross/aarch64): no-remap build MUST leak /project residue =="
  build_leg_no_remap "$PATH_A" "$WORK/out-a"
  BINARY="$WORK/out-a/$BIN"
  # The remap (and -ffile-prefix-map) would have stripped /project (cross's fixed
  # internal mount) and $CARGO_HOME. With the remap OFF, that residue MUST survive
  # in .rodata — proving the remap is load-bearing. Zero residue here means the
  # remap is unnecessary → RED.
  RESIDUE_RE="${CROSS_INTERNAL_SRC}|${CARGO_HOME:-/cargo}/registry"
  if grep -aEo "$RESIDUE_RE" "$BINARY" | head -1 | grep -q .; then
    echo "  OK: no-remap cross build leaks the expected host-path residue:"
    grep -aEo "$RESIDUE_RE" "$BINARY" | sort -u | head -5
    echo "== remap-off NEGATIVE PASSED: the remap/-ffile-prefix-map IS load-bearing (it strips this /project leak). =="
    exit 0
  fi
  echo "::error::remap-off NEGATIVE FAILED — a no-remap cross build leaked ZERO ${CROSS_INTERNAL_SRC}/\$CARGO_HOME path residue. The --remap-path-prefix protection is HOLLOW (a no-op): either the leak source is gone or the remap is not doing real work. The positive gate's byte-identity is no longer evidence the remap works (R0-r3-I1 / brainstorm §P4 remap-off negative)." >&2
  exit 1
fi

# ── x86_64/cargo: each leg sees its OWN real path → assert the two binaries DIFFER ─
echo "== remap-off NEGATIVE (cargo/x86_64): no-remap binaries at two distinct paths MUST DIFFER =="
build_leg_no_remap "$PATH_A" "$WORK/out-a"
build_leg_no_remap "$PATH_B" "$WORK/out-b"

if cmp -s "$WORK/out-a/$BIN" "$WORK/out-b/$BIN"; then
  echo "::error::remap-off NEGATIVE FAILED — two no-remap builds at DISTINCT paths ($PATH_A vs $PATH_B) are byte-IDENTICAL. The --remap-path-prefix protection is HOLLOW (a no-op): the binary is reproducible across distinct paths EVEN WITHOUT the remap, so the positive gate's byte-identity is no longer evidence the remap works (R0-r3-I1 / brainstorm §P4 remap-off negative)." >&2
  exit 1
fi
echo "  OK: the two no-remap binaries DIFFER (the absolute build-path leak returns when the remap is off)."
echo "     cmp first-difference offset:"
cmp "$WORK/out-a/$BIN" "$WORK/out-b/$BIN" || true
echo "== remap-off NEGATIVE PASSED: --remap-path-prefix IS load-bearing (it collapses this path divergence). =="
