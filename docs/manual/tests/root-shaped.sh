#!/usr/bin/env bash
# tests/root-shaped.sh FILE...
#
# Refuses ROOT-SHAPED text in each FILE (refugium spec §5c *Root mode* and
# §7 `mr root` / `mr child`: every form in which a root, or the draws that
# make one, can be typed or printed). Exit 1, one `[root-shaped] FAIL:` line
# per finding; exit 0 when no FILE holds any. The forms, and the rule for each:
#
#   W  a run of 22 or more consecutive BIP-39 English words: a root's 24
#      words (`mr root` prints them; `mr child` and `create --root -` read
#      them) and the 22 pill draws of `mr root --from pills`. A word is a
#      maximal run of ASCII letters, compared lower-cased with
#      tests/bip39-english.txt; it counts when it is in the list or, being
#      exactly four letters, is the first four letters of a list word (`mr`
#      reads a word's unique four-letter prefix as the word, so
#      `aban abil ...` is a root as typed); any other character (space,
#      newline, punctuation, digit, markdown) separates words and never ends
#      a run; only a word that does not count ends it.
#   C  237 or more coin flips: characters H, T, 1, 0 (`mr root --from
#      coins` reads exactly 237, H/T or 1/0, whitespace ignored). Spaces,
#      tabs, carriage returns and line ends inside the run are skipped;
#      any other character ends it.
#   D  92 or more dice rolls: digits 1 to 6 (`mr root --from dice` reads at
#      least 92, whitespace ignored), whitespace skipped as for C.
#   M  a root's ms1 string: `ms1` followed by 72 or more bech32 data
#      characters (`qpzry9x8gf2tvdw0s3jn54khce6mua7l`), either case, after
#      deleting spaces, tabs and `-` within the line (a 32-byte ms1 string
#      is 75 characters, `ms1` included; `mr child` and `create --root -`
#      read it, `create` prints it).
#
# NOT covered: a form split by a character other than those skipped (a
# coin run inside a markdown table cell, `|` between flips; a word run is
# never split this way); an ms1 string broken across lines; a root given
# as hex entropy (no `mr` verb reads one); a root's words in a language
# other than English (`mr` reads English only).
set -euo pipefail
here=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
LIST="$here/bip39-english.txt"
WANT_SHA=2f5eed53a4727b4bf8880d8f3f199efc90e58503646d9ff8eff3a2ed3b24dbda
got=$(sha256sum "$LIST" | cut -d' ' -f1)
if [ "$got" != "$WANT_SHA" ]; then
  printf '[root-shaped] FAIL: %s is not the BIP-39 English list (sha256 %s)\n' "$LIST" "$got" >&2
  exit 1
fi
fail=0
for f in "$@"; do
  out=$(awk -v listfile="$LIST" -v name="$f" '
    BEGIN {
      while ((getline w < listfile) > 0) { ok[w] = 1; pfx[substr(w, 1, 4)] = 1 }
      wr = 0; cr = 0; dr = 0; wrep = 0; crep = 0; drep = 0
    }
    {
      line = $0
      # W: words
      n = split(tolower(line), t, /[^a-z]+/)
      for (i = 1; i <= n; i++) {
        if (t[i] == "") continue
        if ((t[i] in ok) || (length(t[i]) == 4 && (t[i] in pfx))) {
          if (wr == 0) { ws = NR; wrep = 0 }
          wr++
          if (wr >= 22 && !wrep) { printf "%s:%d: W: a run of 22 or more BIP-39 English words (a root'\''s 24 words or 22 pill draws)\n", name, ws; wrep = 1 }
        } else wr = 0
      }
      # C and D: characters
      L = length(line)
      for (i = 1; i <= L; i++) {
        c = substr(line, i, 1)
        if (c == " " || c == "\t" || c == "\r") continue
        if (index("HT10", c)) {
          if (cr == 0) { cs = NR; crep = 0 }
          cr++
          if (cr >= 237 && !crep) { printf "%s:%d: C: 237 or more coin flips (H, T, 1, 0)\n", name, cs; crep = 1 }
        } else cr = 0
        if (index("123456", c)) {
          if (dr == 0) { ds = NR; drep = 0 }
          dr++
          if (dr >= 92 && !drep) { printf "%s:%d: D: 92 or more dice rolls (1 to 6)\n", name, ds; drep = 1 }
        } else dr = 0
      }
      # M: ms1 strings
      m = tolower(line); gsub(/[ \t-]/, "", m)
      if (m ~ /ms1[qpzry9x8gf2tvdw0s3jn54khce6mua7l][qpzry9x8gf2tvdw0s3jn54khce6mua7l]*/) {
        while (match(m, /ms1[qpzry9x8gf2tvdw0s3jn54khce6mua7l]+/)) {
          if (RLENGTH >= 75) { printf "%s:%d: M: an ms1 string of a 32-byte secret (a root'\''s ms1 string)\n", name, NR; break }
          m = substr(m, RSTART + RLENGTH)
        }
      }
    }
  ' "$f")
  if [ -n "$out" ]; then
    while IFS= read -r l; do printf '[root-shaped] FAIL: %s\n' "$l" >&2; done <<<"$out"
    fail=1
  fi
done
exit "$fail"
