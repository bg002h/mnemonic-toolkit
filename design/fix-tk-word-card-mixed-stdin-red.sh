#!/usr/bin/env bash
# RED/GREEN probe for design/FIX-tk-word-card-mixed-stdin.md (bughunt-1119 bh-tk-I7).
# Usage: design/fix-tk-word-card-mixed-stdin-red.sh <path-to-mnemonic-binary>
# Prints one line per row:
#   "<row> rc=<exit> out=<stdout bytes> xpub=<recovered xpub or empty> err=<last stderr line>"
# Covers: rows R1-R9 of the FIX doc (`word-card --decode` positional <WORD> list mixed with `-`
# in either order, `-` twice, a full card as positionals plus `-`, text and --json forms, and
# the unchanged controls: lone `-`, word-per-arg, one quoted whole-card arg, lone `-` on empty stdin).
# Does NOT cover: encode mode (positionals are documented as ignored there), `--from -`
# (collect_from_sources), `--decode-plate`, Windows, or a stdin read error.
# Public mk1 xpub card (the committed cli_word_card.rs MK1 constant); no seed is used.
set -u
B=${1:?usage: $0 <mnemonic-binary>}
MK1="mk1qprsqhpqqsq3cqtsleeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8qp3yxg3dpe854wq4 mk1qprsqhpp0f30mtxzd65mvwcur9usdatwuqvq6z70r9nwrgk6xn6l8gy6nwa2n977sw6zh34rma0nh"
T=$(mktemp -d)
trap 'rm -r "$T"' EXIT

CARD=$("$B" word-card --from "$MK1" --parity-words 4 | sed -n 2p)
[ -n "$CARD" ] || { echo "setup: encode failed" >&2; exit 2; }
read -r -a W <<<"$CARD"

row() { # row <name> <stdin-text> <args...>
  local name=$1 input=$2 rc
  shift 2
  printf '%s' "$input" | "$B" word-card --decode "$@" >"$T/out" 2>"$T/err"; rc=$?
  printf '%s rc=%s out=%s xpub=%s err=%s\n' "$name" "$rc" "$(wc -c <"$T/out")" \
    "$(grep -o 'xpub[1-9A-HJ-NP-Za-km-z]\{100,\}' "$T/out" | head -n1)" "$(tail -n1 "$T/err")"
}

row R1-junk-then-dash-json   "$CARD" --json nonsense -
row R2-junk-then-dash-text   "$CARD" nonsense -
row R3-dash-then-junk        "$CARD" - nonsense
row R4-dash-twice            "$CARD" - -
row R5-full-card-args-plus-dash "$CARD" "${W[@]}" -
row R6-lone-dash             "$CARD" -
row R7-word-per-arg          ""      "${W[@]}"
row R8-one-quoted-arg        ""      "$CARD"
row R9-lone-dash-empty-stdin ""      -
