#!/usr/bin/env bash
# vendor/ freshness guard — the LEADING (PR-time) gate.
#
# REDs iff the committed `vendor/` tree cannot satisfy the current `Cargo.lock`
# under the reproducible build's `--offline --locked` source-replacement config.
# This is the v0.74.0 failure class — a dep bump (e.g. md-codec 0.39.0 -> 0.39.1)
# that updates Cargo.lock but forgets `cargo vendor vendor/`, so the release
# `repro` build (man-pages.yml, tag-triggered) can't resolve the bumped dep and
# publishes NO musl binary. That gate is LAGGING (fires only at the tag); this
# script makes the same failure surface on the PR.
#
# Cheap by design: `cargo metadata` does FULL-workspace, all-target resolution
# with NO compile / NO musl toolchain / NO Docker. With vendored-sources
# replacement active, resolution validates EVERY Cargo.lock entry against vendor/
# regardless of target cfg (R0 round-1 proved this — no musl-only false negative;
# see design/agent-reports/vendor-freshness-guard-r0-round-1.md).
#
# Spec: design/SPEC_vendor_freshness_ci_guard.md
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO_ROOT"

# Derive the miniscript fork rev from Cargo.lock (authoritative, comment-free) so
# the 3-block source config auto-tracks the [patch.crates-io] pin. Fail CLOSED on
# an empty match: a missing rev would silently drop the git-fork stanza and let
# resolution mis-resolve (false GREEN).
MINISCRIPT_REV="$(grep -oE 'rust-miniscript\?rev=[0-9a-f]{40}' Cargo.lock | head -1 | grep -oE '[0-9a-f]{40}' || true)"
if [ -z "$MINISCRIPT_REV" ]; then
  echo "::error::vendor-freshness: could not derive the miniscript fork rev from Cargo.lock" \
       "(expected a 'rust-miniscript?rev=<40-hex>' source line). Failing closed." >&2
  exit 1
fi

# 3-block source-replacement, mirroring ci/repro/double-build.sh: crates-io +
# the miniscript git-fork + vendored-sources -> the committed vendor/ tree.
SRC_CONFIG=(
  --config 'source.crates-io.replace-with="vendored-sources"'
  --config "source.\"git+https://github.com/rust-bitcoin/rust-miniscript?rev=${MINISCRIPT_REV}\".git=\"https://github.com/rust-bitcoin/rust-miniscript\""
  --config "source.\"git+https://github.com/rust-bitcoin/rust-miniscript?rev=${MINISCRIPT_REV}\".rev=\"${MINISCRIPT_REV}\""
  --config "source.\"git+https://github.com/rust-bitcoin/rust-miniscript?rev=${MINISCRIPT_REV}\".replace-with=\"vendored-sources\""
  --config 'source.vendored-sources.directory="vendor"'
)

echo "vendor-freshness: resolving Cargo.lock against committed vendor/ (offline, locked; miniscript rev ${MINISCRIPT_REV}) ..."
if cargo metadata --format-version 1 --locked --offline "${SRC_CONFIG[@]}" >/dev/null; then
  echo "vendor-freshness: OK — vendor/ satisfies Cargo.lock."
else
  echo "::error::vendor/ is out of sync with Cargo.lock — the --offline --locked reproducible build" \
       "cannot resolve a dependency from the committed vendor/ tree. Run 'cargo vendor vendor/' and" \
       "commit the result (see docs/verify-reproducibility.md). This is the v0.74.0 release-CI failure" \
       "class, now caught at PR time." >&2
  exit 1
fi
