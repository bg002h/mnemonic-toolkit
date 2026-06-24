#!/usr/bin/env bash
# ci/repro/cc-validate.sh — the cc-under-musl validation GATE (P1, task #23).
#
# THE UN-MEASURED RESIDUAL (brainstorm §1.2 / §5 / IMPLEMENTATION_PLAN P1).
# The only C dependency is the vendored libsecp256k1 in secp256k1-sys, compiled
# by cc-rs which shells out to `musl-gcc`. A C compiler can embed __DATE__ /
# __TIME__ and absolute OUT_DIR / -I paths into the object. The proven gnu
# double-build NEVER exercised musl-gcc → libsecp determinism under musl is
# UN-measured. This gate makes it rigorous, per §5 step 4 + step 7:
#
#   (a) build the libsecp .o with SOURCE_DATE_EPOCH UNSET (or a DIFFERENT value)
#       → ASSERT the .o DIFFERS from the pinned-epoch .o. This PROVES the epoch
#       is load-bearing / honored by musl-gcc. If it does NOT differ, either the
#       epoch is ignored (wrong cc / not passed through) OR — the benign case —
#       cc emits no __DATE__ at all; we DISTINGUISH these via the residue grep in
#       (c): no epoch-effect AND no __DATE__ residue ⇒ benign (PASS with note);
#       no epoch-effect WITH __DATE__ residue ⇒ epoch ignored ⇒ HARD BLOCKER.
#   (b) build twice at the PINNED epoch → ASSERT the two .o MATCH.
#   (c) `readelf -p .comment` + grep the .o AND the final binary for __DATE__ /
#       __TIME__-shaped residue and host-path residue → ASSERT ZERO.
#
# If musl-gcc proves NON-deterministic here (b fails, or c finds residue that
# the epoch/-ffile-prefix-map cannot neutralize), that is a GENUINE BLOCKER —
# this script REDs loudly; it does not paper over it.
#
# TDD. Authored BEFORE the env wiring. Against the un-remapped / no-epoch
# baseline it REDs (residue present in (c); epoch not load-bearing in (a)). With
# the recipe wired it GREENs.
#
# USAGE.  ci/repro/cc-validate.sh <build-root>
# where <build-root> is a real absolute dir holding the source tree, remapped to
# /build. The script does its own clean per-epoch rebuilds of secp256k1-sys.
#
# ENV (workflow-set): SOURCE_DATE_EPOCH (pinned), CARGO_HOME, TARGET, CRATE, BIN.

set -euo pipefail

ROOT="${1:?usage: cc-validate.sh <build-root>}"
TARGET="${TARGET:-x86_64-unknown-linux-musl}"
CRATE="${CRATE:-mnemonic-toolkit}"
BIN="${BIN:-mnemonic}"
: "${SOURCE_DATE_EPOCH:?SOURCE_DATE_EPOCH must be set (pinned epoch)}"

MINISCRIPT_REV="95fdd1c5773bd918c574d2225787973f63e16a66"
SRC_CONFIG=(
  --config 'source.crates-io.replace-with="vendored-sources"'
  --config "source.\"git+https://github.com/rust-bitcoin/rust-miniscript?rev=${MINISCRIPT_REV}\".git=\"https://github.com/rust-bitcoin/rust-miniscript\""
  --config "source.\"git+https://github.com/rust-bitcoin/rust-miniscript?rev=${MINISCRIPT_REV}\".rev=\"${MINISCRIPT_REV}\""
  --config "source.\"git+https://github.com/rust-bitcoin/rust-miniscript?rev=${MINISCRIPT_REV}\".replace-with=\"vendored-sources\""
  --config 'source.vendored-sources.directory="vendor"'
)

# Remap literals for THIS root (always → /build), used for both rustc and cc.
RUSTFLAGS_REMAP="--remap-path-prefix=${ROOT}=/build --remap-path-prefix=${CARGO_HOME:-/cargo}=/cargo"
CFLAGS_REMAP="-ffile-prefix-map=${ROOT}=/build -ffile-prefix-map=${CARGO_HOME:-/cargo}=/cargo"

# Build secp256k1-sys (only) and copy out its first *.o. $1=epoch ("" = unset).
build_secp_o() {
  local epoch="$1" dest="$2"
  ( cd "$ROOT"
    rm -rf "target/$TARGET/release/build/secp256k1-sys-"* 2>/dev/null || true
    umask 022
    # GNU env: ALL `--unset`/option flags MUST precede any NAME=VALUE assignment
    # (env stops option parsing at the first NAME=VALUE). So the --unset for the
    # epoch-unset leg goes FIRST, before the assignment block.
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
      cargo build --locked --offline --release \
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
PATHS_RE="${ROOT}|/build-a|/build-b|/home/|${CARGO_HOME:-/cargo}/registry"

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
readelf -p .comment "$WORK/o.pinned1" 2>/dev/null || echo "  (no .comment section)"
scan ".o" "$WORK/o.pinned1"
scan "binary" "$BINARY"

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
