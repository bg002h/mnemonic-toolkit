#!/usr/bin/env bash
# Generator for docs/Examples.pdf — emits Examples.md to stdout with EXACT,
# verbatim command input + output captured from the real `mnemonic` v0.75.0
# binary. No eliding: every command is run and its full combined output shown.
#
# Prose is ASCII-only (the body roman font lacks math glyphs); real command
# output keeps its exact unicode (Δ ± × → — etc.), which renders in the DejaVu
# Sans Mono code-block font. Render with:
#   pandoc Examples.md --include-in-header=preamble.tex --listings \
#          --pdf-engine=xelatex -f markdown-smart -o Examples.pdf
# (gen.sh writes preamble.tex; --listings + that preamble give line-wrap +
#  hook-arrow + the literate glyph map that forces output unicode through DejaVu.)
set -u

# Script-relative repo root (works from any checkout; CI's $GITHUB_WORKSPACE
# clone resolves correctly). Override with REPO=... if needed.
REPO="${REPO:-$(cd "$(dirname "$0")/.." && pwd)}"
BUILD="$REPO/.examples-build"
WORK="$BUILD/work"
# Prepend EXAMPLES_BIN_DIR (CI sets it to $GITHUB_WORKSPACE/target/debug) so the
# source-built binary wins over any stale cargo-installed one. PATH is resolved
# from the REAL $HOME first, THEN the environment is scrubbed below.
export PATH="${EXAMPLES_BIN_DIR:+$EXAMPLES_BIN_DIR:}$HOME/.cargo/bin:$PATH"

# Pin the output-visible environment so captured install.sh paths + tool output
# are a pure function of (repo tree, mnemonic binary) on any machine. install.sh
# derives every path it prints from exactly these three (MAN_DIR from
# XDG_DATA_HOME/HOME, install root from CARGO_INSTALL_ROOT/HOME); LC_ALL/LANG/TZ
# pin the python3/jq/sed output. (Set AFTER PATH resolution — order matters.)
unset XDG_DATA_HOME CARGO_INSTALL_ROOT
export HOME=/home/user
export LC_ALL=C LANG=C TZ=UTC

# Strict preflight: fail loud BEFORE emitting so a missing tool can NEVER be
# captured as "command not found" into the document body via run's eval … 2>&1.
# (bitcoind/bitcoin-cli are no longer required — §6.6 is a static capture.)
for _tool in jq python3 sed; do
  command -v "$_tool" >/dev/null 2>&1 || {
    echo "FATAL: required tool '$_tool' not found on PATH" >&2; exit 1; }
done

# Pin to the real, non-experimental shipped binary.
VER=$(mnemonic --version 2>/dev/null)
[ "$VER" = "mnemonic 0.75.0" ] || { echo "FATAL: expected mnemonic 0.75.0, got '$VER'" >&2; exit 1; }

rm -rf "$WORK"; mkdir -p "$WORK"; cd "$WORK" || exit 1

# ── Inputs (public BIP-39 test vectors + reused descriptor assets) ──────────
S0='abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about'
S1='legal winner thank year wave sausage worth useful legal winner thank yellow'
S2='letter advice cage absurd amount doctor acoustic avoid letter advice cage above'
cp "$BUILD/degrade2.desc" policy.desc
cp "$BUILD/tr2.desc"      taproot.desc
cp "$BUILD/tr4.desc"      taproot-4leaf.desc
# policy.json: the 11-key spec, hash normalised to the "opensessame" digest so
# the file is self-consistent if a reader inspects it (build-descriptor refuses
# on key-count before the hash matters, so this is cosmetic).
sed 's/68100fc148a239c4bbf3e6d517a5f4831a803f0603ca834cf790b6703b17bc9d/a84dce40975727c398023cfbd50d5db3b9662375521d0f1ac62dbd829b9a08ad/g' \
    "$BUILD/degrade2-spec.json" > policy.json

# LaTeX preamble (real .tex, NOT YAML header-includes -- the literal unicode +
# \char in the literate map break YAML quoting). Fed to pandoc via
# --include-in-header. `literate` forces output unicode through the DejaVu code
# font (extendedchars alone renders Δ/± but, oddly, drops U+2265). Render:
#   pandoc Examples.md --include-in-header=preamble.tex --listings \
#          --pdf-engine=xelatex -f markdown-smart -o Examples.pdf
cat > "$BUILD/preamble.tex" <<'TEX'
% U+2265/U+2264 leak out of listings into text mode in this document (a
% listings/xelatex edge case), where the roman body font lacks them and they
% drop. Make them active (xetex idiom, no package) and render them in the mono
% font so they match surrounding code output. Other output glyphs render via
% listings' literate map below.
\catcode"2265=\active \protected\def^^^^2265{{\ttfamily\symbol{"2265}}}
\catcode"2264=\active \protected\def^^^^2264{{\ttfamily\symbol{"2264}}}
\usepackage{listings}
\usepackage{xcolor}
\lstset{
  breaklines=true,breakatwhitespace=false,extendedchars=true,
  columns=fullflexible,keepspaces=true,
  basicstyle=\ttfamily\scriptsize,frame=single,framesep=4pt,xleftmargin=2pt,
  postbreak=\mbox{\textcolor{gray}{$\hookrightarrow$}\space},
  literate=%
    {Δ}{{\char"0394}}1 {±}{{\char"00B1}}1 {×}{{\char"00D7}}1
    {→}{{\char"2192}}1 {…}{{\char"2026}}1 {—}{{\char"2014}}1
}
TEX

# ── Emit helpers ───────────────────────────────────────────────────────────
# run: show `$ <cmd>` then its full combined output (executed in THIS shell so
#      variables / created files persist across calls). Single-quote the arg to
#      defer $( ) / $VAR expansion to run time so display == executed.
# The output is captured (not streamed) and re-emitted with an explicit trailing
# newline so the closing ``` ALWAYS lands on its own line. Streaming broke the
# fence when a command's stdout lacked a final newline (e.g. `cat <file>` on a
# descriptor file with no trailing newline ran the descriptor straight into the
# ``` on one line -> pandoc dropped the fence -> all following blocks rendered
# as overrunning prose). $(...) still runs in this shell's cwd so `> file`
# redirects persist; only persistent VAR assignments would be lost (run never
# sets any).
run() { printf '\n```\n$ %s\n' "$1"; printf '%s\n```\n' "$(eval "$1" 2>&1)"; }
# show: print a code block of command(s) WITHOUT executing (for the curl|sh
#       installer and the bitcoin-cli steps that need a live node).
show() { printf '\n```\n'; for c in "$@"; do printf '$ %s\n' "$c"; done; printf '```\n'; }

# ════════════════════════════════════════════════════════════════════════════
cat <<'MD'
---
title: "m-format constellation -- Command-line Examples"
subtitle: "mnemonic-toolkit v0.75.0 -- worked examples (Linux), exact verbatim I/O"
date: "2026-07-05"
geometry: margin=1.8cm
fontsize: 10pt
colorlinks: true
toc: true
toc-depth: 2
monofont: "DejaVu Sans Mono"
---

\newpage

# About these examples

This document shows real, copy-pasteable command lines for the **m-format
constellation** -- a steel-engravable Bitcoin backup system built around four
CLIs (`mnemonic`, `md`, `ms`, `mk`). Every command below was executed against
`mnemonic` **v0.75.0** on Linux and **both the command and its full output are
reproduced verbatim** -- no abbreviations, no ellipses, no elided keys or
addresses. Long lines wrap with a grey hook-arrow continuation marker in the
left margin.

> **All seed phrases here are public BIP-39 TEST VECTORS** (`abandon abandon
> ... about`, etc.). They are world-known and hold no funds. **Never type a real
> seed phrase onto a networked machine, and never reuse these test phrases for
> real money.** The toolkit is alpha software -- use only with disposable
> amounts or on testnet until it has been independently audited.

## Seed input from a file

Yes -- the toolkit reads a seed phrase from a file. There is no `--phrase-file`
flag; instead every secret slot accepts the value `-`, meaning **"read this
secret from stdin"**, which you point at a file with the shell's `< file`
redirect:

MD
show "mnemonic bundle --template bip84 --network mainnet --slot @0.phrase=- < seed.txt"
cat <<'MD'

This is the **secure** idiom: the phrase travels on stdin, so it never appears
in `argv` / `/proc/$PID/cmdline` or your shell history (unlike the inline form
`--slot @0.phrase='<words>'`, which the toolkit flags as secret-on-argv). The
reader strips a single trailing newline, so an ordinary one-line text file
works.

## Multiple files for multiple seeds

Only **one** stdin secret is allowed per invocation, so `< file` reads exactly
one seed. To use several seed files there are two patterns:

1. **One file per invocation (secure -- recommended).** Process each seed file
   in its own command and combine only the resulting public xpubs. No machine
   ever holds more than one seed, and nothing secret reaches `argv`. This is the
   per-device 2-of-3 multisig flow in section 3.
2. **Several files in one command (convenient -- less safe).** Read each file
   with command substitution, `--slot "@N.phrase=$(cat seedN.txt)"`. This works
   for any number of seeds, but each substituted phrase lands on `argv` (the
   toolkit prints a secret-on-argv warning). Only one slot may use the `=-`
   stdin form; a second `=-` is rejected. Shown in section 3.4.

> **Convention in this document:** whenever a command reads a file (a seed, a
> descriptor, a policy JSON, or an `md1` chunk list), the file's contents are
> printed with `cat` immediately beforehand, so every input is visible.

The three engraved cards:

| Card | What it carries |
|------|-----------------|
| **ms1** | BIP-39 entropy (recovers the seed) |
| **mk1** | xpub + origin (master fingerprint + BIP path) |
| **md1** | wallet policy (descriptor template + bound xpubs) |

Throughout, `$` is the shell prompt; everything after it is what you type.

\newpage

# 1. Install the constellation on Linux

The in-repo installer builds each component with `cargo install --locked` into
`~/.cargo/bin` (no `sudo`, no system files touched). It needs `cargo`, `git`,
and a C toolchain; the CLIs require `rustc >= 1.85`.

Install all four CLIs (this compiles from source, so the build log is
machine-specific and not reproduced here):
MD
show 'sh -c "$(curl -fsSL https://raw.githubusercontent.com/bg002h/mnemonic-toolkit/master/scripts/install.sh)" -- --no-gui'
cat <<'MD'

The installer carries the current version pins, so it never goes stale. Useful
flags: `--only <c>`, `--exclude <c>`, `--no-gui`, `--from-git`, `--force`,
`--dry-run`, `--list`. The pin table (`--list`) and a dry run are deterministic
(`$REPO` = your clone root):
MD
run 'sh "$REPO/scripts/install.sh" --list'
run 'sh "$REPO/scripts/install.sh" --no-gui --dry-run'
cat <<'MD'

Verify the install and list every subcommand:
MD
run 'mnemonic --version'
run 'mnemonic --help'

cat <<'MD'

\newpage

# 2. Single-sig card set from a seed phrase (file input)

Create a native-segwit (BIP-84, `m/84'/0'/0'`) single-sig 3-card bundle from one
seed phrase held in a file. Write the phrase to `seed0.txt` (here a public test
vector) and feed it on stdin:
MD
run "printf '%s\n' '$S0' > seed0.txt"
run 'cat seed0.txt'
cat <<'MD'

(`--template` choices for single-sig: `bip44`, `bip49`, `bip84`, `bip86`.) Run
the bundle. stdout carries the three cards to engrave; stderr carries the
human-readable engraving panel and the secret-material warning:
MD
run 'mnemonic bundle --template bip84 --network mainnet --slot @0.phrase=- < seed0.txt'
cat <<'MD'

Each card is printed once, grouped into 5-character blocks
(`ms10e ntrsq qqqqq ...`) -- exactly the form you punch or engrave. Add
`--no-engraving-card` to suppress the stderr panel when piping into other tools.

\newpage

# 3. Conventional 2-of-3 multisig from 3 seed phrases (per-device, file input)

A real multisig never lets one machine see more than one seed. Each cosigner
derives **only their public xpub** from their own seed file (on their own,
ideally air-gapped, device); the coordinator then combines the three **public**
keys into a watch-only `wsh(sortedmulti(...))`. No secret ever leaves its file.

Put each cosigner's seed in its own file:
MD
run "printf '%s\n' '$S0' > seed0.txt"
run "printf '%s\n' '$S1' > seed1.txt"
run "printf '%s\n' '$S2' > seed2.txt"
cat <<'MD'

On each device, derive that cosigner's BIP-87 multisig fingerprint and account
xpub (`--template wsh-sortedmulti` implies the `m/87'/0'/0'` path) from the seed
file. Cosigner @0:
MD
run 'cat seed0.txt'
run 'mnemonic convert --from phrase=- --to fingerprint --template wsh-sortedmulti --network mainnet < seed0.txt'
run 'mnemonic convert --from phrase=- --to xpub        --template wsh-sortedmulti --network mainnet < seed0.txt'
cat <<'MD'

Cosigner @1:
MD
run 'cat seed1.txt'
run 'mnemonic convert --from phrase=- --to fingerprint --template wsh-sortedmulti --network mainnet < seed1.txt'
run 'mnemonic convert --from phrase=- --to xpub        --template wsh-sortedmulti --network mainnet < seed1.txt'
cat <<'MD'

Cosigner @2:
MD
run 'cat seed2.txt'
run 'mnemonic convert --from phrase=- --to fingerprint --template wsh-sortedmulti --network mainnet < seed2.txt'
run 'mnemonic convert --from phrase=- --to xpub        --template wsh-sortedmulti --network mainnet < seed2.txt'

# Assemble the descriptor from the captured public keys (no secrets involved).
FP0=$(mnemonic convert --from phrase=- --to fingerprint --template wsh-sortedmulti --network mainnet < seed0.txt 2>/dev/null | sed -n 's/^fingerprint: //p')
XP0=$(mnemonic convert --from phrase=- --to xpub        --template wsh-sortedmulti --network mainnet < seed0.txt 2>/dev/null | sed -n 's/^xpub: //p')
FP1=$(mnemonic convert --from phrase=- --to fingerprint --template wsh-sortedmulti --network mainnet < seed1.txt 2>/dev/null | sed -n 's/^fingerprint: //p')
XP1=$(mnemonic convert --from phrase=- --to xpub        --template wsh-sortedmulti --network mainnet < seed1.txt 2>/dev/null | sed -n 's/^xpub: //p')
FP2=$(mnemonic convert --from phrase=- --to fingerprint --template wsh-sortedmulti --network mainnet < seed2.txt 2>/dev/null | sed -n 's/^fingerprint: //p')
XP2=$(mnemonic convert --from phrase=- --to xpub        --template wsh-sortedmulti --network mainnet < seed2.txt 2>/dev/null | sed -n 's/^xpub: //p')
K0="[$FP0/87'/0'/0']$XP0"
K1="[$FP1/87'/0'/0']$XP1"
K2="[$FP2/87'/0'/0']$XP2"
printf 'wsh(sortedmulti(2,%s/<0;1>/*,%s/<0;1>/*,%s/<0;1>/*))\n' "$K0" "$K1" "$K2" > multisig.desc

cat <<'MD'

Wrap each as an origin-annotated descriptor key `[fingerprint/87'/0'/0']xpub`
and combine into a 2-of-3 sorted-multisig descriptor (`/<0;1>/*` = the
external/change multipath). The assembled descriptor file:
MD
run 'cat multisig.desc'
cat <<'MD'

Validate and canonicalise it (this also computes the BIP-380 checksum):
MD
run 'mnemonic export-wallet --descriptor "$(cat multisig.desc)" --format descriptor --network mainnet'
cat <<'MD'

The first receive address (here via the BSMS / BIP-129 record, which also
carries the `/0/*,/1/*` derivation):
MD
run 'mnemonic export-wallet --descriptor "$(cat multisig.desc)" --format bsms --network mainnet'
cat <<'MD'

Engrave the shared watch-only card set from the public descriptor (the md1
policy card is shared by all cosigners; each cosigner additionally backs up
their own seed as a single-sig ms1 set per section 2). With only public xpubs
supplied, the ms1 cards are empty placeholders:
MD
run 'mnemonic bundle --descriptor-file multisig.desc --network mainnet'

cat <<'MD'

## 3.4 Building from all seeds on one machine (multiple files, one command)

If instead you hold all the seeds yourself, you can build the whole bundle in a
single command, reading each seed from its own file with command substitution.
This is less safe -- each substituted phrase lands on `argv`, so the toolkit
prints a secret-on-argv warning for every one -- but it needs no per-device
coordination. The three seed files (shown again for reference):
MD
run 'cat seed0.txt'
run 'cat seed1.txt'
run 'cat seed2.txt'
cat <<'MD'

Because seeds (not just xpubs) are supplied, this emits the **full secret card
set** -- one `ms1` per cosigner -- not the watch-only placeholders of 3.3:
MD
run 'mnemonic bundle --template wsh-sortedmulti --threshold 2 --network mainnet --slot "@0.phrase=$(cat seed0.txt)" --slot "@1.phrase=$(cat seed1.txt)" --slot "@2.phrase=$(cat seed2.txt)"'
cat <<'MD'

Only one secret may arrive on stdin, so you cannot replace more than one
substitution with the `=-` file-redirect form -- a second `=-` is rejected:
MD
run 'mnemonic bundle --template wsh-sortedmulti --threshold 2 --network mainnet --slot @0.phrase=- --slot @1.phrase=- --slot "@2.phrase=$(cat seed2.txt)" < seed0.txt'

cat <<'MD'

\newpage

# 4. Card set -> Bitcoin Core wallet descriptor (and how to import)

`mnemonic restore --md1 <chunks>` reconstructs the watch-only wallet from the
**shared md1 card alone** -- no seeds needed. First produce that card from the
section-3 wallet (descriptor file shown again) and pull out its md1 chunks:
MD
run 'cat multisig.desc'
run 'mnemonic bundle --descriptor-file multisig.desc --network mainnet --json | jq -r ".md1[]" > multisig.md1'
run 'cat multisig.md1'
cat <<'MD'

Restore reconstructs the wallet from exactly those chunks. The default form
prints the descriptor and first address (note the address matches section 3 --
same wallet -- while the descriptor *string* differs because the md1 card stores
each key as a depth-0 master xpub, an equivalent serialisation):
MD
run 'mnemonic restore --network mainnet $(sed "s/^/--md1 /" multisig.md1)'
cat <<'MD'

Add `--format bitcoin-core` for a ready-to-import `importdescriptors` request
array (external `.../0/*` + change `.../1/*`):
MD
run 'mnemonic restore --network mainnet $(sed "s/^/--md1 /" multisig.md1) --format bitcoin-core'
cat <<'MD'

Import into Bitcoin Core: save the array, create a blank descriptor wallet, and
load it (these run against your own node, so their output is not shown here):
MD
show 'mnemonic restore --network mainnet $(sed "s/^/--md1 /" multisig.md1) --format bitcoin-core > wallet.json' \
     'bitcoin-cli -named createwallet wallet_name="multisig-2of3" disable_private_keys=true blank=true descriptors=true' \
     'bitcoin-cli -rpcwallet="multisig-2of3" importdescriptors "$(cat wallet.json)"' \
     'bitcoin-cli -rpcwallet="multisig-2of3" getnewaddress'
cat <<'MD'

Tips: `--timestamp now` skips the rescan for a fresh wallet (default `0` rescans
from genesis); `--range 0,4999` widens the gap limit; `--bitcoin-core-version 24`
targets older Core. `restore` also emits `--format descriptor` (the bare
`wsh(...)#checksum`) for other wallets.

\newpage

# 5. Custom degrading-miniscript wallet -- the pathological example (distinct keys per tier) + watch-only export

A four-tier vault -- our **pathological example** wallet. **Each tier uses its own distinct key set (no key reuse)**,
deliberately mixing all four Bitcoin timelock kinds:

| Tier | Spend condition | Timelock kind |
|---|---|---|
| 1 | **3-of-3** (K0,K1,K2) **+ secret word** | absolute height -- `after(1000000)` |
| 2 | **2-of-3** (K3,K4,K5) **+ secret word** | absolute time -- `after(1893456000)` |
| 3 | **both** K6 and K7 | relative blocks -- `older(65535)` |
| 4 | **any 1 of** K8,K9,K10 | relative time -- `older(4255898)` |

That is **11 distinct keys** (3+3+2+3). Absolute locks (`after`) count from the
chain's height/clock; relative locks (`older`) count from each coin's own
confirmation. Encodings:

- `after(1000000)` -- absolute **block height** 1,000,000 (BIP-65; values below
  500,000,000 are heights).
- `after(1893456000)` -- absolute **Unix time** = 2030-01-01 00:00 UTC (values
  at/above 500,000,000 are timestamps).
- `older(65535)` -- relative **blocks**: 65,535 blocks (~455 days). This is the
  largest safe relative-block lock; `older(65536)` would be BIP-68
  consensus-masked to zero, and the toolkit warns if it sees one.
- `older(4255898)` -- relative **time**: BIP-68's time flag (bit 22) set, plus
  61,594 units x 512 s ~= 365 days.

## 5.1 The secret word (a hashlock shared by tiers 1 and 2)

Reusing a *hash* across tiers is fine -- it is not a key. The secret word is
`opensessame`; the descriptor commits to `H = sha256(sha256(word))` and a spend
reveals the 32-byte preimage `X = sha256(word)`:
MD
run 'python3 -c "import hashlib; w=b'"'"'opensessame'"'"'; X=hashlib.sha256(w).digest(); print('"'"'preimage X ='"'"', X.hex()); print('"'"'hash H     ='"'"', hashlib.sha256(X).hexdigest())"'

cat <<'MD'

## 5.2 The guided builder caps complexity -- use the raw descriptor path

`mnemonic build-descriptor` runs a satisfiability + cost preview that it
**bounds** for funds-safety. An 11-key, 4-branch policy exceeds that envelope, so
the guided builder refuses and points you at the raw `--descriptor` path. The
policy-tree spec it reads:
MD
run 'cat policy.json'
cat <<'MD'

Running the guided builder on it:
MD
run 'mnemonic build-descriptor --spec policy.json --network mainnet'
cat <<'MD'

(For a policy *within* the envelope -- fewer keys -- `build-descriptor --spec`
validates and emits it for you.) For arbitrarily complex policies you hand the
miniscript descriptor straight to `export-wallet` / `bundle`. The hand-written
descriptor file:
MD
run 'cat policy.desc'
cat <<'MD'

Validate and canonicalise it (this adds the BIP-380 checksum). The full
canonical descriptor, with every xpub in full:
MD
run 'mnemonic export-wallet --descriptor "$(cat policy.desc)" --format descriptor --network mainnet'
cat <<'MD'

First receive address (Mainnet), via the BSMS record:
MD
run 'mnemonic export-wallet --descriptor "$(cat policy.desc)" --format bsms --network mainnet'
cat <<'MD'

## 5.3 Engrave the card set

Because every key is **distinct**, this is a valid BIP-388 wallet policy, so --
unlike a key-reusing policy -- `bundle` will engrave it. With only public xpubs
supplied, the result is watch-only (the ms1 cards are empty placeholders). The
descriptor file it reads:
MD
run 'cat policy.desc'
cat <<'MD'

The watch-only card set:
MD
run 'mnemonic bundle --descriptor-file policy.desc --network mainnet'
cat <<'MD'

## 5.4 Restore round-trip -- the card set reconstructs the same first address

This 11-key, 4-branch policy is about as complex as a BIP-388 wallet gets, but
the md1 cards carry it faithfully: take the md1 chunks and `restore` to read back
the descriptor and its first address. The reconstructed first address is
**identical** to the canonical descriptor's (section 5.2) -- proof the card set
round-trips this whole policy without loss, every `after`/`older`/`sha256` lock
and `multi(...)` threshold across all four `or_i` branches preserved.

Note the contrast with the section 6 appendix: that depth->=2 *taproot* tree is
the one shape the **shipped** `mnemonic` refuses to restore (it needs the
`experimental/taproot-depth-ge2` branch). This degrading-multisig policy is fully
supported on the real shipped binary -- the round-trip below uses it, no
experimental branch required.
MD
run 'mnemonic bundle --descriptor-file policy.desc --network mainnet --json | jq -r ".md1[]" > policy.md1'
run 'mnemonic restore --network mainnet $(sed "s/^/--md1 /" policy.md1)'
cat <<'MD'

(As in section 6.3, `restore` re-serialises each key as a depth-0 `xpub661My...`
-- a different descriptor string, identical addresses. Compare the `first recv:`
line above with the BSMS first address in 5.2: byte-for-byte the same.)

## 5.5 Watch-only export for Nunchuk / Core / Sparrow

There is no dedicated `nunchuk` emitter, but Nunchuk imports miniscript wallets
from a **descriptor** or a **BSMS (BIP-129)** record -- both shown above in 5.2
(`--format descriptor` for *Add Wallet -> Import -> descriptor*, and
`--format bsms` for the multisig import format, which also Bitcoin Core and
Sparrow accept). After import, fund the address; each spend path opens only when
its lock matures, and the hashlock tiers additionally require revealing the
secret word's preimage `X`.

\newpage

# 6. Taproot version of the degrading wallet (the pathological example)

`wsh(...)` reveals the whole policy on every spend. **Taproot** gives a
cooperative **key-path** spend (cheap, private, looks like single-sig) and
splits the fallbacks into **script-tree leaves**, so a spend reveals only the
leaf it uses. We keep the same four timelock/hash/multisig tiers as fallbacks
and add a distinct cooperative internal key `Kint` (`[.../84'/0'/4']`). Taproot
multisig uses `multi_a`, not `multi`. **12 distinct keys** in total.

## 6.1 Why depth-1 (the one-tier-per-leaf limit on master)

The tidiest layout is one tier per leaf (4 leaves), but that is a **depth-2**
taptree, and the shipped rust-miniscript pin mis-formats depth->=2 taptrees (the
PR-#953 bug). The toolkit refuses such a descriptor up front rather than emit a
malformed one. The four-leaf (depth-2) descriptor file:
MD
run 'cat taproot-4leaf.desc'
cat <<'MD'

Asking the toolkit to export it:
MD
run 'mnemonic export-wallet --descriptor "$(cat taproot-4leaf.desc)" --format descriptor --network mainnet'
cat <<'MD'

So we use a **depth-1** tree (2 leaves) and pack two tiers per leaf with `or_i`:
Leaf A = tier 1 or tier 2 (the absolute-timelock + secret-word tiers); Leaf B =
tier 3 or tier 4 (the relative-timelock tiers). (A rust-miniscript release
> 13.1.0 containing #953 reopens deep trees -- tracked in FOLLOWUP
`taproot-coverage-cycle-on-miniscript-gt-13-1-0`.)

## 6.2 Build + validate

The hand-written depth-1 `tr(...)` descriptor file:
MD
run 'cat taproot.desc'
cat <<'MD'

Validate and canonicalise it. The full canonical descriptor, every xpub in full:
MD
run 'mnemonic export-wallet --descriptor "$(cat taproot.desc)" --format descriptor --network mainnet'
cat <<'MD'

`Kint` (`[73c5da0a/84'/0'/4']`) is the key-path; the two `or_i(...)` blocks are
the two script leaves; `after(...)` are the absolute (height/time) locks and
`older(...)` the relative (blocks/time) locks -- the same four kinds as section 5.

## 6.3 Engrave + first address

Every key is distinct, so it engraves (watch-only). Take the md1 chunks and
restore to read the first address (this round-trip also proves the **real
internal key** at the trunk reconstructs -- the non-NUMS internal-key feature,
shipped in v0.55.3). The
descriptor file (shown again):
MD
run 'cat taproot.desc'
run 'mnemonic bundle --descriptor-file taproot.desc --network mainnet --json | jq -r ".md1[]" > taproot.md1'
run 'cat taproot.md1'
run 'mnemonic restore --network mainnet $(sed "s/^/--md1 /" taproot.md1)'
cat <<'MD'

(Restore re-serialises each key as a depth-0 `xpub661My...` -- a different
descriptor string, identical addresses; that is how the md1 card stores keys.)

## 6.4 Export for wallets (Nunchuk / Core / Sparrow)

`descriptor` and `bitcoin-core` both work for taproot. The descriptor file
(shown again):
MD
run 'cat taproot.desc'
cat <<'MD'

Bitcoin Core `importdescriptors` payload:
MD
run 'mnemonic export-wallet --descriptor "$(cat taproot.desc)" --format bitcoin-core --network mainnet'
cat <<'MD'

But **BSMS / BIP-129 is not available for taproot** (BIP-129 predates BIP-386) --
the toolkit says so and points you elsewhere:
MD
run 'mnemonic export-wallet --descriptor "$(cat taproot.desc)" --format bsms --network mainnet'
cat <<'MD'

## 6.5 Adding a condition: which depth? (a spending-cost comparison)

Suppose you want a fifth spend path: a **new key Knew plus the preimage of a
RIPEMD-160 hashlock** (secret word "please"). Should it be **folded** into an
existing leaf (tree stays depth-1) or given its **own leaf** (forcing depth-2)?
Decide on spending cost, not aesthetics. A taproot script-path witness costs:
(satisfaction) + the **revealed leaf script** + a **control block** of
`33 + 32*depth` bytes. Folding many conditions into one leaf bloats *every*
spend through it (you reveal the unused branches too); a deeper leaf adds one
32-byte hash per level. A witness byte weighs 1 WU = 0.25 vB, so **one extra
depth level = +8 vB**.

The new hashlock (same two-step scheme): preimage `X = sha256("please")`,
descriptor hash `Hp = ripemd160(X)`:
MD
run 'python3 -c "import hashlib; X=hashlib.sha256(b'"'"'please'"'"').digest(); print('"'"'preimage X ='"'"', X.hex()); print('"'"'hash Hp    ='"'"', hashlib.new('"'"'ripemd160'"'"', X).hexdigest())"'
cat <<'MD'

`mnemonic compare-cost` reports per-condition witness vbytes (key-agnostic --
abstract labels A,B,... are auto-dummy-keyed). **Folded** -- the new tier joins
Leaf B, so spending it reveals all of tiers 3+4+5:
MD
run 'mnemonic compare-cost --miniscript "or_i(and_v(v:older(65535),multi(2,A,B)),or_i(and_v(v:older(4255898),multi(1,C,D,E)),and_v(v:pk(F),ripemd160(06d05e2f02fb90ddf98d8cd95d806ba12b27aff4))))"'
cat <<'MD'

**Dedicated** -- the new tier is its own leaf, revealing only itself:
MD
run 'mnemonic compare-cost --miniscript "and_v(v:pk(F),ripemd160(06d05e2f02fb90ddf98d8cd95d806ba12b27aff4))"'
cat <<'MD'

`compare-cost` models each input as a single leaf at the tree root (depth-0,
33-byte control block); add the real Merkle depth (+8 vB per level):

| Placement | revealed script | tr vB (depth-0) | real depth | + control block | real tr |
|---|---|---:|---|---:|---:|
| Folded into Leaf B | whole `or_i(t3,t4,t5)` | 139 | 1 (still 2 leaves) | +8 | ~147 vB |
| Dedicated leaf | just t5 | 90 | 2 (now 3 leaves) | +16 | ~106 vB |

**A dedicated leaf is ~41 vB (~28%) cheaper** to spend the new condition: not
revealing tiers 3+4 (~49 vB) beats the +8 vB for the extra depth level. It is
better still in practice because folding also makes *t3 and t4* spends reveal
t5's script, and a 3-leaf tree can place the **most-used** condition in the
shallow slot and bury cold paths deeper.

**Rule of thumb:** in taproot, one-condition-per-leaf almost always wins -- each
sibling script you avoid revealing is worth far more than the 8 vB/level of
depth -- and you order leaves hot-shallow / cold-deep.

**The catch:** cost says depth-2, but master cannot build depth->=2 yet (the
PR-#953 taptree-Display bug, 6.1), so today the only buildable option is the
folded depth-1 leaf at the ~28% premium. That premium is the concrete,
quantified motive for landing the upstream fix.
MD

# ── 6.6 Taproot multisig (NUMS), cross-checked against Bitcoin Core ──────────
NUMS=50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0
printf 'tr(%s,sortedmulti_a(2,%s/<0;1>/*,%s/<0;1>/*,%s/<0;1>/*))\n' \
  "$NUMS" "$K0" "$K1" "$K2" > taproot-multi.desc
cat <<'MD'

\newpage

## 6.6 Taproot multisig (NUMS), cross-checked against Bitcoin Core

The taproot wallets above are deliberately complex. The *simplest* taproot
**multisig** is a single `sortedmulti_a` 2-of-3 leaf under the BIP-341 **NUMS**
("nothing-up-my-sleeve") internal key -- an **unspendable** key-path, so the
only way to spend is the sorted multisig script. It round-trips on the
**shipped** binary, and Bitcoin Core derives the **identical** first address.
(The toolkit's own `tests/bitcoind_differential.rs` gate proves exactly this --
`bundle` -> `restore` -> `first_addresses` vs Core `deriveaddresses` -- for the
taproot-multisig corpus against a pinned Bitcoin Core v27.0.) Reuse the three
section-3 cosigners:
MD
run 'cat taproot-multi.desc'
cat <<'MD'

Validate and canonicalise it (the NUMS hex is the BIP-341 unspendable H-point):
MD
run 'mnemonic export-wallet --descriptor "$(cat taproot-multi.desc)" --format descriptor --network mainnet'
cat <<'MD'

Engrave the watch-only card set and read the first address back from the md1
chunks alone:
MD
run 'mnemonic bundle --descriptor-file taproot-multi.desc --network mainnet --json | jq -r ".md1[]" > taproot-multi.md1'
run 'mnemonic restore --network mainnet $(sed "s/^/--md1 /" taproot-multi.md1)'
cat <<'MD'

`restore` reports a `bc1p...` Taproot address. Confirm it against Bitcoin
Core's **independent C++** derivation: `deriveaddresses` on the receive
(`.../0/*`) descriptor (split from the `<0;1>` multipath, which Core rejects):

*(STATIC CAPTURE -- recorded from Bitcoin Core v27.0. `deriveaddresses` is a
deterministic descriptor-to-address function of the fixed descriptor above;
this line is NOT regenerated by `gen.sh` and needs no node in CI.)*

```
$ bitcoin-cli -chain=main deriveaddresses "tr(50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0,sortedmulti_a(2,[73c5da0a/87'/0'/0']xpub661MyMwAqRbcFrooZ2966EcDmVX5MoFXZhuJqXTudvJzwBTBfPQSc5JzX52fvS18oqSdEJXJ4kTGRJ76wPWDUSNJsY5JsgVBQoD6KrbdCLL/0/*,[b8688df1/87'/0'/0']xpub661MyMwAqRbcEnFgxHRLx7i1fnjcBPgc71qy8mVkbGXYukNGMK2XFRbAaCLYEJDUufNoBxTNa68i5MYhqmrEkfhjzgHCUEcvJBhXS5bk4RW/0/*,[28645006/87'/0'/0']xpub661MyMwAqRbcEdy4jr5EtEhQBctfscE6a99DGLr2cW4HnnmBsXDoe3odGzRiw3hcRM5wfKcQmb7s5FjdGrR6SrExXmeopaoY9Lk7tQusDjN/0/*))#mk8vdqmt" "[0,0]"
["bc1p550zvnachy40z6hh8llka93mkm0c3635samp264ck6rfd0dcdc8s00n8c8"]
```

Byte-for-byte the same `bc1p...` that `restore` reported -- the toolkit's own
derivation (which v0.49.1 routes *around* the codec for taproot) agrees with
Bitcoin Core. `tr(NUMS,sortedmulti_a)` is a shape **only** the toolkit can
render -- its pinned rust-miniscript fork has `sortedmulti_a`, the codec's
crates.io build does not -- so this end-to-end check is the only place an
external oracle sees it.
MD
cat <<'MD'

\newpage

# Appendix A -- the public test seeds used

| Slot | BIP-39 test phrase | Fingerprint (BIP-87) |
|---|---|---|
MD
printf '| @0 | `%s` | `%s` |\n' "$S0" "$FP0"
printf '| @1 | `%s` | `%s` |\n' "$S1" "$FP1"
printf '| @2 | `%s` | `%s` |\n' "$S2" "$FP2"
cat <<'MD'

These are world-known BIP-39 vectors with no funds. Sections 5-6 derive their
keys from these same three seeds at distinct `m/84'/0'/N'` accounts. Generated
with `mnemonic` v0.75.0 on Linux. See the in-repo manual (`docs/manual/`) for
the authoritative per-flag reference.

\newpage

# Appendix B -- EXPERIMENTAL: depth->=2 taproot reconstruction (NOT FOR REAL FUNDS)

> **EXPERIMENTAL -- DO NOT USE FOR REAL FUNDS.** Everything in this appendix
> uses `mnemonic-depth2`, a proof-of-concept binary built from the never-merged
> branch `experimental/taproot-depth-ge2`. It pins an **unreleased**
> rust-miniscript commit (the PR-#953 merge, in no crates.io release) to lift
> the depth->=2 taptree cap. It is **not** part of any shipped release, is not
> on the install path, and must never be used to secure real funds. The shipped
> `mnemonic` v0.75.0 documented everywhere else in this guide deliberately
> **refuses** depth->=2 (sections 6.1 and below); that refusal is the supported
> behavior until a rust-miniscript release > 13.1.0 ships #953.

Master refuses to *restore* a depth->=2 taptree because the shipped miniscript
pin mis-formats nested taptrees. The POC bumps that pin and lifts the cap. Build
it under a **distinct name** so it never overwrites your real `mnemonic`. The
twin below was built from the 0.55.3-era experimental branch; the static
reconstruction capture further down is from that build:
MD
show 'git clone https://github.com/bg002h/mnemonic-toolkit && cd mnemonic-toolkit' \
     'git checkout experimental/taproot-depth-ge2' \
     'cargo build --release --manifest-path crates/mnemonic-toolkit/Cargo.toml' \
     'cp target/release/mnemonic ~/.cargo/bin/mnemonic-depth2   # DISTINCT name'

cat <<'MD'

Recall the depth-2 four-leaf descriptor from section 6.1 (one tier per leaf):
MD
run 'cat taproot-4leaf.desc'
cat <<'MD'

The engraved card is a faithful backup even on master -- `bundle` accepts it;
only *reading it back* is gated. Make the card and pull its md1 chunks with the
shipped binary:
MD
run 'mnemonic bundle --descriptor-file taproot-4leaf.desc --network mainnet --json | jq -r ".md1[]" > depth2.md1'
run 'cat depth2.md1'
cat <<'MD'

The shipped `mnemonic` refuses to restore it (the supported behavior -- the card
stays a faithful backup):
MD
run 'mnemonic restore --network mainnet $(sed "s/^/--md1 /" depth2.md1)'
cat <<'MD'

The experimental `mnemonic-depth2` reconstructs the very same card -- depth-2
taptree and all -- and prints a loud EXPERIMENTAL advisory on stderr before the
reconstructed descriptor and first address:

*(STATIC CAPTURE -- recorded 2026-06-15 from the experimental `mnemonic-depth2`
build at v0.55.3. This block is NOT regenerated by `gen.sh` and is not
reproducible with a released binary or in CI; build the branch as shown above
to reproduce it. Everything else in this document is live-captured and CI-gated.)*

```
$ mnemonic-depth2 restore --network mainnet $(sed "s/^/--md1 /" depth2.md1)
EXPERIMENTAL: depth-≥2 taproot reconstruction relies on an UNRELEASED rust-miniscript commit (#953, in no crates.io release) — proof-of-concept only; do NOT use for real funds and do NOT merge. Rebuild when miniscript > 13.1.0 ships.
miniscript policy restore (12 cosigners)
CONFIRM: verify each cosigner fingerprint against your records before importing.
  descriptor: tr([73c5da0a/84'/0'/4']xpub661MyMwAqRbcEyUKSqsBgaz1Lob8pCa1rM1SJ8CEzGCYyP9LisxZ2m1goDqj137XvHdY2nNkctqiE1ixaAFqYHf91CFpFpKicVb7TzvrGsE/<0;1>/*,{{and_v(v:after(1000000),and_v(v:sha256(a84dce40975727c398023cfbd50d5db3b9662375521d0f1ac62dbd829b9a08ad),multi_a(3,[73c5da0a/84'/0'/0']xpub661MyMwAqRbcFHMVYpCiBTXd2Caj7vZhNFHJSgE59Aue2yYkXSrz5q9GaQ4rRjJVhHZTsCiHWSzgMS5beaaTHWVmhpGC7SMdqMXHRXZi8as/<0;1>/*,[73c5da0a/84'/0'/1']xpub661MyMwAqRbcGaxoYcLaxHHXZqEgSRQmN2P5ung8MJ8MNE535mLuhq7zjnrMKyA5eX6ehicVbU1FFPU39LGXbY8PmLPLQxVRQmPFa3Q7spa/<0;1>/*,[73c5da0a/84'/0'/2']xpub661MyMwAqRbcGuXAHBK3oquZS1HJiz2fVZ2idNcK4GLGTXJyGZkPK7fviN6euv5GzY18JD3WBG3SoLat23TLAVjhQMxDVMAqymQNhg3RFT8/<0;1>/*))),and_v(v:after(1893456000),and_v(v:sha256(a84dce40975727c398023cfbd50d5db3b9662375521d0f1ac62dbd829b9a08ad),multi_a(2,[73c5da0a/84'/0'/3']xpub661MyMwAqRbcGHLCZcLjg25oG8wSyqSE5XNM9uMks6vrpH4pRDC8UmAynovThuKraidMeEKJ2FcqBw1eF76aeu1vrGtLJXUiJXr4r9N1TZQ/<0;1>/*,[b8688df1/84'/0'/0']xpub661MyMwAqRbcGowNgeNcLS8CgL2vnZybpJqtkbCmSQMdq2qzcDWqq3CXXg7x5BqvcNCSaNUw6nisoN7JFK2j3HfxV57nNm2RLKo2UzHgbs6/<0;1>/*,[b8688df1/84'/0'/1']xpub661MyMwAqRbcEv3U8uuxavsQA8LNNYwcNge8rT7SaMS5S8KiEwxoP72TQ8ARYjczPTtVQz6CxcaBTEE3XchmYvSiHcVbC9h17CmyfG7sVq9/<0;1>/*)))},{and_v(v:older(65535),multi_a(2,[b8688df1/84'/0'/2']xpub661MyMwAqRbcG7Xht9EwgNucA47Rmgg8Bn5bNmFdJMkotHQDXirpogQHkVNRcwAy6KwGnUYMUNBFCNaRq4WnsqWW2VNUDdW6ymHXfVpk4c3/<0;1>/*,[b8688df1/84'/0'/3']xpub661MyMwAqRbcEbqBvNkLuDtudGA2PHAbtWUuHKe3CKjZCaLjxLGSG8SJpwBCnXsj8xPGXaV9ZWL3j9ktbed8y1aeNVK95HrkgHfHGBXM5Eh/<0;1>/*)),and_v(v:older(4255898),multi_a(1,[28645006/84'/0'/0']xpub661MyMwAqRbcEdBofBaGbgnse74WRuyEbXRSmzq8jzthzutDnXTV2yNQPzgs3ubwuNp7yrSHnECoA5xHgnoEDH4HSGWqLtYdi6nWVZCfXPk/<0;1>/*,[28645006/84'/0'/1']xpub661MyMwAqRbcH2WNMbtz4pZ8wDtpxndYo6E4r5o8pXedve17srma1LCEjM8WcpVk67xsc36KpBNtYUdqo5dpcFMzRfzSZSa4C5DRty4eDNF/<0;1>/*,[28645006/84'/0'/2']xpub661MyMwAqRbcGqcAAnB9mhvQsdUx2fKasUoXT2gMpt2tFz94wRfAkhuLhZUJkjQ5pgnd9Ny9EwrgcHbAASVnQShCbfhnGsKAk2k6yGoWXAv/<0;1>/*))}})#5trrgdg0
  first recv: bc1p6yc7kzttzsafprr6hwsaefuyqxvee4j48zdrqt4kl9ers68mhcestwvn66
  cosigner @0: 73c5da0a [84'/0'/4']  from md1 (not independently verified)
  cosigner @1: 73c5da0a [84'/0'/0']  from md1 (not independently verified)
  cosigner @2: 73c5da0a [84'/0'/1']  from md1 (not independently verified)
  cosigner @3: 73c5da0a [84'/0'/2']  from md1 (not independently verified)
  cosigner @4: 73c5da0a [84'/0'/3']  from md1 (not independently verified)
  cosigner @5: b8688df1 [84'/0'/0']  from md1 (not independently verified)
  cosigner @6: b8688df1 [84'/0'/1']  from md1 (not independently verified)
  cosigner @7: b8688df1 [84'/0'/2']  from md1 (not independently verified)
  cosigner @8: b8688df1 [84'/0'/3']  from md1 (not independently verified)
  cosigner @9: 28645006 [84'/0'/0']  from md1 (not independently verified)
  cosigner @10: 28645006 [84'/0'/1']  from md1 (not independently verified)
  cosigner @11: 28645006 [84'/0'/2']  from md1 (not independently verified)
UNVERIFIED: no --from/--cosigner cross-check supplied; verify each cosigner fingerprint above against your records before importing
note: stdout is watch-only — public keys only, cannot spend
```

The reconstructed descriptor is the genuine depth-2 shape `tr(Kint,{{A,B},{C,D}})`
(four leaves, one tier each) -- the layout section 6.5 showed is cheaper to spend
but that master cannot yet build. When a rust-miniscript release > 13.1.0 ships
#953, this is rebuilt as a supported feature and this appendix retires.
MD
