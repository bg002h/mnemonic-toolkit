#!/usr/bin/env bash
# RED/GREEN probe R2 for design/FIX_sec_repro_doc_miniscript_rev.md (r11 F-371).
# Usage (from anywhere): bash design/fix-sec-repro-doc-miniscript-rev-red.sh
# Takes the `--config 'source...'` lines printed in the section 9.3 rebuild
# command of docs/verify-reproducibility.md (the two-space-indented ones) and
# runs `cargo metadata --locked --offline` with exactly those lines and an
# EMPTY CARGO_HOME (no git checkout cache), so exit 0 means the printed
# stanzas alone resolve every git source in Cargo.lock against vendor/.
# Prints the --config lines, cargo's error if any, then `exit=<rc>`.
# Does NOT cover: the section 4 docker command (its stanzas are checked as
# text by tests/repro_doc_pins.rs), compiling, or byte reproducibility.
set -uo pipefail
cd "$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
export PATH="$HOME/.cargo/bin:$PATH"
home=$(mktemp -d)
mapfile -t A < <(sed -n "s/^  --config '\(.*\)'.*$/--config=\1/p" docs/verify-reproducibility.md)
printf '%s\n' "${A[@]}"
CARGO_HOME="$home" cargo metadata --locked --offline --format-version 1 "${A[@]}" >/dev/null
rc=$?
echo "exit=$rc"
rmdir "$home" 2>/dev/null || true
exit "$rc"
