#!/usr/bin/env bash
# tests/anchor-check.sh
#
# Intra-doc anchor-dangler gate against build/m-format-manual.html.
#
# Why HTML and not the GFM intermediate: pandoc's `--to gfm` emitter
# STRIPS explicit `{#id}` heading anchors (a `## Heading {#my-id}` line
# becomes `## Heading` in the output, with the only anchor-target being
# pandoc's auto-derived slug from the heading text). 15 of the 20 src/
# explicit `{#id}` anchors are lost this way; lychee sees them as
# "missing fragment" errors when run against `build/m-format-manual.md`.
# pandoc's `--to html` emitter preserves them as `<h2 id="my-id">`,
# which lychee reads natively. See `design/SPEC_manual_anchor_dangler_
# cleanup.md` for the full architectural rationale.
#
# Baseline-snapshot enforcement:
# Pre-cycle the manual has a residual of ~known author-side dangler
# slugs (heading-rename-without-link-update, slug-guess mismatches like
# singular-vs-plural, version-suffix-period mangles). The baseline file
# `tests/anchor-dangler-baseline.txt` captures every dangling slug at
# the time of capture; this script asserts the CURRENT dangler set
# matches the baseline EXACTLY:
#
#   - NEW dangler (not in baseline) → exit 1 + `::error::` with the
#     offending slug. Blocks the PR.
#   - OLD dangler (in baseline) that no longer dangles → exit 1 + a
#     different `::error::` instructing the author to ratchet the
#     baseline file down (i.e. delete the now-fixed slug from the
#     baseline txt and commit that as part of the same PR). This makes
#     the baseline a strict one-way ratchet enforced as a CI block —
#     warnings would be voluntary; CLAUDE.md fix-the-class discipline
#     demands enforcement.
#
# Variables (KEY=VALUE):
#   BUILD_HTML — path to build/m-format-manual.html (the lychee target)
#   BASELINE   — path to tests/anchor-dangler-baseline.txt
#
# Called from the Makefile as `make anchor-check`. Mirrors the
# verify-examples.sh KEY=VALUE arg-parsing convention.

set -euo pipefail

for arg in "$@"; do
  case "$arg" in
    BUILD_HTML=*) BUILD_HTML="${arg#*=}" ;;
    BASELINE=*)   BASELINE="${arg#*=}"   ;;
  esac
done

: "${BUILD_HTML:?BUILD_HTML is required (path to build/m-format-manual.html)}"
: "${BASELINE:?BASELINE is required (path to tests/anchor-dangler-baseline.txt)}"

if [ ! -f "$BUILD_HTML" ]; then
  echo "::error::anchor-check: BUILD_HTML $BUILD_HTML missing — run \`make html\` first" >&2
  exit 1
fi
if [ ! -f "$BASELINE" ]; then
  echo "::error::anchor-check: BASELINE $BASELINE missing" >&2
  exit 1
fi

# Extract the current dangler set via the same pipeline used for
# baseline capture (single-source-of-truth for the slug-extraction
# rule — any divergence between capture and check would cause
# spurious drift).
tmpdir=$(mktemp -d)
trap 'rm -rf "$tmpdir"' EXIT

lychee --offline --include-fragments --no-progress "$BUILD_HTML" 2>&1 \
  | grep '^\[ERROR\]' \
  | grep -oE '#[^ ]+' \
  | sed 's/^#//' \
  | sort -u > "$tmpdir/current.txt" || true

sort -u "$BASELINE" > "$tmpdir/baseline.txt"

# Diff classes:
#   `comm -23 a b` — lines in a, not in b → NEW danglers (in current, not in baseline).
#   `comm -13 a b` — lines in b, not in a → OLD danglers (in baseline, not in current; fixed).
new_danglers=$(comm -23 "$tmpdir/current.txt" "$tmpdir/baseline.txt")
shrunk_baseline=$(comm -13 "$tmpdir/current.txt" "$tmpdir/baseline.txt")

fail=0

if [ -n "$new_danglers" ]; then
  while IFS= read -r slug; do
    echo "::error::anchor-check: new dangler '#$slug' (not in baseline). Re-run \`make html\` then \`lychee --offline --include-fragments build/m-format-manual.html\` to reproduce; fix the offending link in src/ or update the target heading."
  done <<<"$new_danglers"
  fail=1
fi

if [ -n "$shrunk_baseline" ]; then
  while IFS= read -r slug; do
    echo "::error::anchor-check: baseline shrunk — slug '#$slug' no longer dangles; ratchet docs/manual/tests/anchor-dangler-baseline.txt to remove it (delete this line + commit in the same PR that fixed the dangler)."
  done <<<"$shrunk_baseline"
  fail=1
fi

if [ "$fail" -ne 0 ]; then
  exit 1
fi

echo "OK anchor-check: $(wc -l < "$tmpdir/current.txt") danglers in current run match baseline (no new, no shrunk)"
