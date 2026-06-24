#!/usr/bin/env bash
# ci/repro/cc-validate.sh — the cc-under-musl validation GATE (P1 x86_64; P2 aarch64).
#
# THE UN-MEASURED RESIDUAL (brainstorm §1.2 / §5 / IMPLEMENTATION_PLAN P1/P2).
# The only C dependency is the vendored libsecp256k1 in secp256k1-sys, compiled
# by cc-rs which shells out to a musl C compiler. A C compiler can embed
# __DATE__ / __TIME__ and absolute OUT_DIR / -I paths into the object. The proven
# gnu double-build NEVER exercised any musl toolchain → libsecp determinism under
# musl is UN-measured, and the two musl legs use DIFFERENT compilers:
#   * x86_64 (P1)  — the container's apt `musl-gcc`        (BUILDER=cargo).
#   * aarch64 (P2) — `cross`'s DIGEST-PINNED (Cross.toml) bundled aarch64-musl
#                    toolchain, under QEMU                 (BUILDER=cross).
# The aarch64 compiler is a SEPARATE musl-gcc, so its determinism is RE-VALIDATED
# here independently — P1's x86_64 cc result does NOT transfer. This gate makes
# it rigorous, per §5 step 4 + step 5 + step 7:
#
#   (a) build the libsecp .o with SOURCE_DATE_EPOCH UNSET (or a DIFFERENT value)
#       → ASSERT the .o DIFFERS from the pinned-epoch .o. This PROVES the epoch
#       is load-bearing / honored by the musl toolchain. If it does NOT differ,
#       either the epoch is ignored (wrong cc / not passed through) OR — the
#       benign case — cc emits no __DATE__ at all; we DISTINGUISH these via the
#       residue grep in (c): no epoch-effect AND no __DATE__ residue ⇒ benign
#       (PASS with note); no epoch-effect WITH __DATE__ residue ⇒ epoch ignored
#       ⇒ HARD BLOCKER.
#   (b) build twice at the PINNED epoch → ASSERT the two .o MATCH.
#   (c) `readelf -p .comment` + grep the .o AND the final binary for __DATE__ /
#       __TIME__-shaped residue and host-path residue → ASSERT ZERO.
#   (d) (aarch64, R0-I2 — PRIMARY aarch64 evidence) ASSERT the .comment compiler
#       string contains CROSS_COMMENT_EXPECT, PROVING the digest-pinned cross
#       toolchain is in use and the Cross.toml passthrough took effect (cross A/B
#       can match while BOTH retain residue — the direct .comment + residue
#       assertions are the load-bearing aarch64 gate, not A/B-equality).
#
# If the musl toolchain proves NON-deterministic here (b fails, or c finds
# residue that the epoch/-ffile-prefix-map cannot neutralize), that is a GENUINE
# BLOCKER — this script REDs loudly; it does not paper over it.
#
# TDD. Authored BEFORE the env wiring. Against the un-remapped / no-epoch
# baseline it REDs (residue present in (c); epoch not load-bearing in (a)). For
# aarch64 the passthrough probe REDs with an empty Cross.toml passthrough list
# (residue in (c) / wrong .comment in (d)). With the recipe wired it GREENs.
#
# USAGE.  ci/repro/cc-validate.sh <build-root>
# where <build-root> is a real absolute dir holding the source tree, remapped to
# /build. The script does its own clean per-epoch rebuilds of secp256k1-sys.
#
# ENV (workflow-set): SOURCE_DATE_EPOCH (pinned), CARGO_HOME, TARGET, CRATE, BIN,
#   BUILDER (cargo|cross), CROSS_COMMENT_EXPECT (aarch64 .comment substring).

set -euo pipefail

ROOT="${1:?usage: cc-validate.sh <build-root>}"
TARGET="${TARGET:-x86_64-unknown-linux-musl}"
CRATE="${CRATE:-mnemonic-toolkit}"
BIN="${BIN:-mnemonic}"
: "${SOURCE_DATE_EPOCH:?SOURCE_DATE_EPOCH must be set (pinned epoch)}"

# BUILDER selects the compile front-end (P2):
#   cargo  — x86_64 leg, direct in the digest-pinned container (musl-gcc, P1).
#   cross  — aarch64 leg, via `cross` under QEMU using cross's DIGEST-PINNED
#            (Cross.toml) bundled aarch64-musl toolchain — a DIFFERENT musl-gcc,
#            independently RE-VALIDATED here. `cross` forwards the `--config`
#            [source] overrides to the inner cargo and mounts the project dir
#            (so committed vendor/ is present); the determinism-bearing env
#            reaches the container ONLY via Cross.toml [build.env] passthrough.
BUILDER="${BUILDER:-cargo}"
# CROSS_COMMENT_EXPECT (aarch64, R0-I2) — a substring the `readelf -p .comment`
# of the cross-built .o MUST contain, PROVING the digest-pinned cross toolchain
# (Cross.toml) is the compiler in use (not some host musl-gcc). Empty ⇒ the
# .comment substring assertion is informational-only (x86_64 leg).
CROSS_COMMENT_EXPECT="${CROSS_COMMENT_EXPECT:-}"
# REMAP_SRC — the IN-COMPILER source root the remap maps to /build.
#   * cargo (x86_64): the compiler sees the real host path → REMAP_SRC=$ROOT.
#   * cross (aarch64): the project is bind-mounted at cross's FIXED internal path
#     /project (verified: cross v0.2.5 src/docker/local.rs `-v <host_root>:
#     /project`), so the remap from-side MUST be /project, NOT the host path —
#     otherwise /project survives un-stripped in .rodata/.o and (c) REDs.
# Auto-derived from BUILDER so the cross leg needs no extra workflow env (mirrors
# double-build.sh's CROSS_INTERNAL_SRC); an explicit REMAP_SRC override wins.
if [ -n "${REMAP_SRC:-}" ]; then
  :  # explicit override honored
elif [ "$BUILDER" = "cross" ]; then
  REMAP_SRC="${CROSS_INTERNAL_SRC:-/project}"
else
  REMAP_SRC="$ROOT"
fi

# PARAMETERIZED [source] activation (P3a) — see double-build.sh for the full
# rationale. MINISCRIPT_REV UNSET ⇒ toolkit rev (THREE-block, pre-P3a behavior);
# EMPTY ⇒ TWO-block (codec form); set ⇒ THREE-block keyed off that rev. Single-
# dash default fires ONLY when UNSET so a codec caller's explicit empty is honored.
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

# Remap literals for THIS root (always → /build), used for both rustc and cc.
# REMAP_SRC is the path the COMPILER sees: $ROOT for the cargo leg, /project for
# the cross leg (cross's fixed internal mount). CARGO_HOME inside the cross
# container is /cargo (cross sets it) — the same literal as the x86_64 leg.
RUSTFLAGS_REMAP="--remap-path-prefix=${REMAP_SRC}=/build --remap-path-prefix=${CARGO_HOME:-/cargo}=/cargo"
CFLAGS_REMAP="-ffile-prefix-map=${REMAP_SRC}=/build -ffile-prefix-map=${CARGO_HOME:-/cargo}=/cargo"

# Build secp256k1-sys (only) and copy out its first *.o. $1=epoch ("" = unset).
build_secp_o() {
  local epoch="$1" dest="$2"
  ( cd "$ROOT"
    rm -rf "target/$TARGET/release/build/secp256k1-sys-"* 2>/dev/null || true
    umask 022
    # GNU env: ALL `--unset`/option flags MUST precede any NAME=VALUE assignment
    # (env stops option parsing at the first NAME=VALUE). So the --unset for the
    # epoch-unset leg goes FIRST, before the assignment block.
    #
    # NOTE (P2 — why the env is EXPORTED for cross, not inlined): for BUILDER=cargo
    # the assignments are scoped to the single `env … cargo` invocation. For
    # BUILDER=cross the determinism-bearing vars must be present in the HOST
    # process env so `cross` can forward them via Cross.toml [build.env]
    # passthrough into the QEMU container — an inline `env VAR=… cross` DOES put
    # them in cross's own environment (cross is the child of `env`), so the same
    # `env "${envv[@]}" <builder>` form works for both: the vars are in the
    # builder process env, and cross reads its passthrough list from there.
    local -a envv=()
    if [ -z "$epoch" ]; then
      envv+=(--unset=SOURCE_DATE_EPOCH)
    fi
    envv+=(
      CARGO_BUILD_RUSTFLAGS="$RUSTFLAGS_REMAP"
      CFLAGS="$CFLAGS_REMAP"
      "CFLAGS_${TARGET//-/_}=$CFLAGS_REMAP"
      LC_ALL=C TZ=UTC
    )
    if [ -n "$epoch" ]; then
      envv+=("SOURCE_DATE_EPOCH=$epoch")
    fi
    env "${envv[@]}" \
      "$BUILDER" build --locked --offline --release \
        --target "$TARGET" -p "$CRATE" --bin "$BIN" \
        "${SRC_CONFIG[@]}"
  )
  local o
  o="$(find "$ROOT/target/$TARGET/release/build" \
        -path '*secp256k1-sys-*/out/*.o' -print -quit 2>/dev/null || true)"
  if [ -z "$o" ]; then
    echo "::error::cc-validate: no secp256k1-sys *.o produced — cannot validate the C frontier." >&2
    exit 1
  fi
  cp "$o" "$dest"
}

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

DIFF_EPOCH=315532800   # 1980-01-01 UTC — a deliberately DIFFERENT, fixed value.
# ⚠ COUPLING: the residue scan below whitelists the rendered form of THIS date
# ('Jan  1 1980') so the diff-epoch leg's own __DATE__ cannot false-trip (c).
# If DIFF_EPOCH changes, update the `grep -vE 'Jan  1 1980'` allowlist to match.

# NOTE (expected outcome): for THIS binary the .o very likely does NOT change
# with the epoch — libsecp256k1 embeds no __DATE__/__TIME__ and -ffile-prefix-map
# already strips paths, so there is no live timestamp for the epoch to pin. That
# is the BENIGN branch, and it is the EXPECTED result here, NOT a failure: the
# load-bearing assertions are (b) determinism at the pinned epoch + (c) zero
# residue. Probe (a) exists only to DISTINGUISH "epoch irrelevant (benign)" from
# "epoch ignored despite a __DATE__ present (BLOCKER)" — disambiguated at the
# end against the (c) residue result.
echo "== (a) epoch load-bearing probe (different epoch ⇒ .o may DIFFER) =="
build_secp_o "$SOURCE_DATE_EPOCH" "$WORK/o.pinned1"
build_secp_o "$DIFF_EPOCH"        "$WORK/o.diff"

epoch_loadbearing=0
if cmp -s "$WORK/o.pinned1" "$WORK/o.diff"; then
  echo "  NOTE: .o did NOT change when SOURCE_DATE_EPOCH changed (expected if no __DATE__ is embedded; benign — confirmed against (c) below)."
else
  echo "  OK: .o changed with the epoch — SOURCE_DATE_EPOCH is honored by musl-gcc."
  epoch_loadbearing=1
fi

echo "== (b) two builds at the PINNED epoch ⇒ .o must MATCH =="
build_secp_o "$SOURCE_DATE_EPOCH" "$WORK/o.pinned2"
fail=0
if cmp "$WORK/o.pinned1" "$WORK/o.pinned2"; then
  echo "  OK: two pinned-epoch .o are byte-identical (C frontier deterministic)."
else
  echo "::error::cc-validate (b) FAILED — two pinned-epoch libsecp .o DIFFER. musl-gcc is NON-deterministic at the pinned epoch. GENUINE BLOCKER." >&2
  fail=1
fi

echo "== (c) residue grep (.o + final binary) — expect ZERO =="
BINARY="$ROOT/target/$TARGET/release/$BIN"
# __DATE__ ("Mmm DD YYYY") + __TIME__ ("HH:MM:SS") shapes; host-path residue.
DATE_RE='[A-Z][a-z][a-z] [ 0-9][0-9] 20[0-9][0-9]'
TIME_RE='[0-2][0-9]:[0-5][0-9]:[0-5][0-9]'
# Scan for BOTH the host real path ($ROOT) AND cross's internal /project mount —
# the aarch64 leak source is /project, not the host path. (REMAP_SRC is omitted:
# it is ALWAYS already a member here — $ROOT on the cargo leg, /project on the
# cross leg — so listing it would only duplicate an existing alternative.)
PATHS_RE="${ROOT}|/project|/build-a|/build-b|/home/|${CARGO_HOME:-/cargo}/registry"

residue=0
scan() {
  local label="$1" file="$2"
  [ -f "$file" ] || { echo "  ($label: $file absent — skipped)"; return; }
  if grep -aEo "$DATE_RE" "$file" | grep -vE 'Jan  1 1980' | head -1 | grep -q .; then
    echo "::error::$label: __DATE__-shaped residue present" >&2
    grep -aEo "$DATE_RE" "$file" | head -3 >&2
    residue=1
  fi
  if grep -aEo "$TIME_RE" "$file" | head -1 | grep -q .; then
    echo "::warning::$label: __TIME__-shaped token present (may be a false positive — verify)" >&2
    grep -aEo "$TIME_RE" "$file" | head -3 >&2
  fi
  if grep -aEo "$PATHS_RE" "$file" | head -1 | grep -q .; then
    echo "::error::$label: host-path residue present (real build path leaked — -ffile-prefix-map/remap gap)" >&2
    grep -aEo "$PATHS_RE" "$file" | head -3 >&2
    residue=1
  fi
}
echo "  -- readelf -p .comment of the .o --"
COMMENT="$(readelf -p .comment "$WORK/o.pinned1" 2>/dev/null || true)"
if [ -n "$COMMENT" ]; then
  echo "$COMMENT"
else
  echo "  (no .comment section)"
fi
scan ".o" "$WORK/o.pinned1"
scan "binary" "$BINARY"

# (d) PASSTHROUGH / COMPILER-STRING assertion (R0-I2 — PRIMARY aarch64 evidence).
# For the aarch64 cross leg, A/B-equality alone is WEAK evidence: both legs share
# the cross container's INTERNAL paths, so they can match while BOTH retaining a
# host path or a __DATE__ that the passthrough silently failed to neutralize. So
# assert DIRECTLY that the digest-pinned cross toolchain (Cross.toml) is the
# compiler in use: the .comment string MUST contain CROSS_COMMENT_EXPECT. If the
# passthrough list were empty (the RED-first TDD probe), the build would either
# leak residue (caught by (c)) or — for the compiler-identity check — the .comment
# would not carry the expected pinned-toolchain string. Empty CROSS_COMMENT_EXPECT
# (x86_64 leg) ⇒ informational only.
if [ -n "$CROSS_COMMENT_EXPECT" ]; then
  echo "== (d) .comment compiler-string assertion (aarch64; expect substring '$CROSS_COMMENT_EXPECT') =="
  if printf '%s' "$COMMENT" | grep -aqF "$CROSS_COMMENT_EXPECT"; then
    echo "  OK: .comment carries the expected pinned cross-toolchain string."
  else
    echo "::error::cc-validate (d) FAILED — .comment does NOT contain '$CROSS_COMMENT_EXPECT'; the digest-pinned cross toolchain (Cross.toml) is NOT the compiler in use (passthrough gap or wrong image). PRIMARY aarch64 gate." >&2
    fail=1
  fi
fi

if [ "$residue" -ne 0 ]; then
  echo "::error::cc-validate (c) FAILED — __DATE__/__TIME__ or host-path residue present after the recipe. Not reproducible." >&2
  fail=1
else
  echo "  OK: zero __DATE__/host-path residue in .o and binary."
fi

# Disambiguate (a): no epoch-effect is a BLOCKER only if __DATE__ residue exists.
if [ "$epoch_loadbearing" -eq 0 ]; then
  if [ "$residue" -ne 0 ]; then
    echo "::error::cc-validate (a) FAILED — epoch NOT load-bearing AND __DATE__/path residue present ⇒ SOURCE_DATE_EPOCH is being IGNORED. GENUINE BLOCKER." >&2
    fail=1
  else
    echo "  NOTE (a benign): epoch produced no .o delta AND no __DATE__ residue ⇒ musl-gcc embeds no timestamp here; epoch is correctly a no-op. PASS."
  fi
fi

if [ "$fail" -ne 0 ]; then
  echo "::error::cc-validate GATE FAILED." >&2
  exit 1
fi
echo "== cc-validate GATE PASSED: musl-gcc deterministic, epoch honored-or-irrelevant, zero residue =="
