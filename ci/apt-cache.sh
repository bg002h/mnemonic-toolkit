#!/usr/bin/env bash
# ci/apt-cache.sh -- cache the .deb files of an apt install between CI runs.
#
#   apt-cache.sh plan    <dir> <pkg>...   apt-get update, resolve the exact set
#                                         of debs, print `digest=` and `pkgsum=`
#                                         lines (for $GITHUB_OUTPUT) and write
#                                         the set to <dir>.plan
#   apt-cache.sh install <dir> <pkg>...   apt-get install with <dir> as the apt
#                                         archives directory, then drop debs not
#                                         in the plan and make <dir> cacheable
#
# The digest covers each deb's filename, size and hash from the signed apt
# index, not the mirror it comes from, so one key names one exact set of
# bytes. apt re-checks every cached deb against that index before using it
# and re-downloads on a mismatch, so a stale or tampered cache cannot change
# what is installed. Any apt failure fails the script (no retry, no fallback).
# Used by .github/actions/apt-cache; tested by ci/apt-cache.test.sh.
set -euo pipefail

mode=${1:-}
dir=${2:-}
[ -n "$dir" ] || { echo "usage: apt-cache.sh plan|install <dir> <pkg>..." >&2; exit 2; }
shift 2
[ "$#" -gt 0 ] || { echo "apt-cache.sh: no packages given" >&2; exit 2; }
pkgs=("$@")
plan_file=$dir.plan

case "$mode" in
  plan)
    sudo apt-get update
    # An empty archives dir makes --print-uris list every deb that is not
    # already installed, whatever an earlier run left behind.
    scratch=$(mktemp -d)
    trap 'rm -r -f "$scratch"' EXIT
    uris=$(sudo apt-get -qq -o "Dir::Cache::archives=$scratch" install --print-uris \
             -y --no-install-recommends "${pkgs[@]}")
    mkdir -p "$(dirname "$plan_file")"
    # Lines look like:  'URL' filename size HASH:value
    printf '%s\n' "$uris" | { grep -E "^'" || true; } \
      | awk '{print $2, $3, $4}' | LC_ALL=C sort > "$plan_file.tmp"
    mv "$plan_file.tmp" "$plan_file"
    digest=$(sha256sum < "$plan_file" | cut -d' ' -f1)
    pkgsum=$(printf '%s\n' "${pkgs[@]}" | LC_ALL=C sort | sha256sum | cut -c1-12)
    printf 'digest=%s\npkgsum=%s\n' "$digest" "$pkgsum"
    ;;
  install)
    [ -f "$plan_file" ] || { echo "apt-cache.sh: no plan file $plan_file (run plan first)" >&2; exit 1; }
    mkdir -p "$dir"
    sudo apt-get -o "Dir::Cache::archives=$dir" -o APT::Keep-Downloaded-Packages=true install -y --no-install-recommends "${pkgs[@]}"
    # Keep only the debs the plan names, so a partial restore from an older
    # key never grows the cache with debs nothing will use again.
    keep=$(awk '{print $1}' "$plan_file")
    for f in "$dir"/*.deb; do
      [ -e "$f" ] || continue
      printf '%s\n' "$keep" | grep -qxF -- "$(basename "$f")" || rm -f -- "$f"
    done
    # apt leaves root/_apt-owned partial/ and lock behind; the cache step
    # must be able to read the directory as the runner user.
    sudo rm -r -f "$dir/partial" "$dir/lock"
    sudo chown -R "$(id -u):$(id -g)" "$dir"
    ;;
  *)
    echo "apt-cache.sh: unknown mode '$mode'" >&2
    exit 2
    ;;
esac
