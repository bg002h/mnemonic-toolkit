---
title: "m-format constellation -- Command-line Examples"
subtitle: "mnemonic-toolkit v0.97.0 -- worked examples (Linux), exact verbatim I/O"
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
`mnemonic` **v0.97.0** on Linux and **both the command and its full output are
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


```
$ mnemonic bundle --template bip84 --network mainnet --slot @0.phrase=- < seed.txt
```

This is the **secure** idiom: the phrase travels on stdin, so it never appears
in `argv` / `/proc/$PID/cmdline` or your shell history. The inline form
`--slot @0.phrase='<words>'` is **refused** at exit 2 before the command line
is parsed. The reader strips a single trailing newline, so an ordinary one-line
text file works.

## Multiple files for multiple seeds

Only **one** stdin secret is allowed per invocation, so `< file` reads exactly
one seed. To use several seed files there are two patterns:

1. **One file per invocation (secure -- recommended).** Process each seed file
   in its own command and combine only the resulting public xpubs. No machine
   ever holds more than one seed, and nothing secret reaches `argv`. This is the
   per-device 2-of-3 multisig flow in section 3.
2. **Several files in one command, via the environment.** Only one slot may
   use the `=-` stdin form; a second `=-` is rejected. For the rest, export each
   seed and point the slot at it with `--slot '@N.phrase=@env:VAR'`. The
   environment is not `argv`: it is absent from `ps` and from your shell
   history. Command substitution -- `--slot "@N.phrase=$(cat seedN.txt)"` --
   puts every phrase on `argv` and is now **refused**. Shown in section 3.4.

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

```
$ sh -c "$(curl -fsSL https://raw.githubusercontent.com/bg002h/mnemonic-toolkit/master/scripts/install.sh)" -- --no-gui
```

The installer carries the current version pins, so it never goes stale. Useful
flags: `--only <c>`, `--exclude <c>`, `--no-gui`, `--from-git`, `--force`,
`--dry-run`, `--list`. The pin table (`--list`) and a dry run are deterministic
(`$REPO` = your clone root):

```
$ sh "$REPO/scripts/install.sh" --list
COMPONENT       CARGO_PACKAGE        DEFAULT      FEATURES       GIT_TAG
---------       -------------        -------      --------       -------
mnemonic        mnemonic-toolkit     git (only)   (none)         mnemonic-toolkit-v0.97.0
md              md-cli               crates.io    cli-compiler   descriptor-mnemonic-md-cli-v0.14.0
ms              ms-cli               crates.io    (none)         ms-cli-v0.16.0
mk              mk-cli               crates.io    (none)         mk-cli-v0.12.0
mnemonic-gui    mnemonic-gui         git (only)   (none)         mnemonic-gui-v0.59.0
```

```
$ sh "$REPO/scripts/install.sh" --no-gui --dry-run
m-format constellation installer
install root: /home/user/.cargo/bin
source: crates.io (default; mnemonic-toolkit stays on git+tag)

install  mnemonic (git: mnemonic-toolkit-v0.97.0)
  [dry-run] cargo install --locked --git https://github.com/bg002h/mnemonic-toolkit --tag mnemonic-toolkit-v0.97.0   mnemonic-toolkit
  [dry-run] mkdir -p "/home/user/.local/share/man/man1" && "/home/user/.cargo/bin/mnemonic" gen-man --out "/home/user/.local/share/man/man1"
install  md (crates.io: md-cli)
  [dry-run] cargo install --locked --features cli-compiler  md-cli
  [dry-run] mkdir -p "/home/user/.local/share/man/man1" && "/home/user/.cargo/bin/md" gen-man --out "/home/user/.local/share/man/man1"
install  ms (crates.io: ms-cli)
  [dry-run] cargo install --locked   ms-cli
  [dry-run] mkdir -p "/home/user/.local/share/man/man1" && "/home/user/.cargo/bin/ms" gen-man --out "/home/user/.local/share/man/man1"
install  mk (crates.io: mk-cli)
  [dry-run] cargo install --locked   mk-cli
  [dry-run] mkdir -p "/home/user/.local/share/man/man1" && "/home/user/.cargo/bin/mk" gen-man --out "/home/user/.local/share/man/man1"
skip     mnemonic-gui

4 installed.

verify:
    mnemonic --version       md --version
    ms --version             mk --version
    mnemonic-gui --version

man pages installed to /home/user/.local/share/man/man1;
if "man <cli>" does not find them, run: man -M "/home/user/.local/share/man/man1" <cli>
```

Verify the install and list every subcommand:

```
$ mnemonic --version
mnemonic 0.97.0
```

```
$ mnemonic --help
engraving-bundle CLI for the m-format star (ms1 + mk1 + md1)

Usage: mnemonic [OPTIONS] <COMMAND>

Commands:
  bundle            emit a 3-card engraving bundle from a phrase or xpub
  verify-bundle     round-trip-check an engraved bundle
  convert           convert between seed/key formats (BIP-39 / BIP-32 / WIF / ms1 / mk1)
  addresses         list a wallet's receive/change addresses (batch, read-only)
  decode-address    decode a Bitcoin address → network(s) / script type / witness version / scriptPubKey
  export-wallet     emit watch-only wallet artifacts (Bitcoin Core importdescriptors, BIP-388 wallet_policy)
  import-wallet     import a third-party wallet blob into an m-format bundle (v0.26.0 Phase 2: BSMS Round-2 only)
  derive-child      derive deterministic child entropy / keys from a master xprv (BIP-85)
  electrum-decrypt  decrypt an Electrum field-encrypted secret (seed phrase / xprv) with a password
  final-word        emit the set of BIP-39 last words that yield a valid checksum for an N-1 partial phrase
  seed-xor          split a BIP-39 phrase into N XOR shares OR combine N shares back into a phrase
  seedqr            encode/decode SeedQR (BIP-39 mnemonic ↔ numeric digit-string QR payload)
  nostr             Wrap an existing nostr key (npub/nsec) as Bitcoin addresses/descriptors/WIF
  silent-payment    Derive a BIP-352 silent-payment receiver address (base + labeled) from a seed
  slip39            split a master secret into SLIP-39 K-of-N shares OR combine shares back (Trezor-compatible)
  ms-shares         split a secret into BIP-93 codex32 K-of-N (ms1) shares OR combine shares back
  gen-man           emit roff man pages for the whole CLI tree into a directory (clap-faithful)
  gui-schema        emit SPEC §7 GUI-overlay flag-surface schema JSON (companion to `mnemonic-gui` v0.2)
  repair            BCH error-correct a corrupted m-format card (ms1 / mk1 / md1)
  inspect           describe the contents of an m-format card (ms1 / mk1 / md1)
  compare-cost      compare wsh-vs-tr per-spending-condition cost for a miniscript or descriptor
  xpub-search       search for a target (xpub, descriptor, address, or passphrase) under a seed or xpub
  verify-message    verify a Bitcoin message signature (legacy P2PKH signmessage + BIP-322 segwit/taproot)
  restore           emit a watch-only restore document (single-sig) from a seed + optional passphrase
  build-descriptor  build a validated wsh(...) descriptor + BIP-388 policy from a JSON policy-tree spec
  word-card         encode an mk1/md1 card as an engravable BIP-39 Word Card (+ optional RAID), or --decode one back
  help              Print this message or the help of the given subcommand(s)

Options:
      --no-auto-repair     v0.22.0 — skip auto-fire repair on decode failures; preserve pre-v0.22 exit policy. Global flag. Honored by `convert`, `inspect`, and (v0.22.1+) `verify-bundle`. For `verify-bundle`, auto-fire is additionally gated on `std::io::stdout().is_terminal()` to preserve the legacy VerifyCheck-row behavior when output is piped or captured (per v0.22.1 D18 — TTY-conditional default). Standalone `repair` ignores this flag (the whole point of that subcommand IS repair). Under `--json` calling contexts the auto-fire emits a structured JSON envelope on stdout (per v0.22.1 D20) instead of text-form
      --allow-argv-secret  SPEC_constellation_cli_uniformity 6d — proceed even though secret material is on argv. Use it only where argv is safe (a single-user air-gapped box, an amnesic Tails session). The DECISION is not made here: `argv_guard::inspect` reads this flag out of raw `std::env::args()` before `Cli::try_parse()` runs; this declaration exists so clap ACCEPTS it and so `--help` and `gui-schema` show it. Greppable, so a reviewer can find every place a script opted in
  -h, --help               Print help
  -V, --version            Print version

RECOVERING A FORGOTTEN BIP-39 PASSPHRASE:
  If you have your seed words (entropy) but not the BIP-39 passphrase
  (the optional "25th word"): if you have a LIST of likely passphrases,
  `mnemonic xpub-search passphrase-of-xpub --passphrase-candidates-file
  <file> --target-xpub <a known xpub>` tests each candidate against a
  value you already know. To GENERATE or mutate a keyspace (wordlists,
  masks, typo models), `mnemonic` does not — an external open-source tool
  does: btcrecover searches passphrase candidates and confirms each by
  deriving an address / xpub / master-fingerprint at common default paths
  and matching a value you already know.
    btcrecover (maintained):  https://github.com/3rdIteration/btcrecover
    original:                 https://github.com/gurnec/btcrecover
  Pointer current as of 2026-05-25. Run untrusted recovery tools
  offline, on an air-gapped machine.
```

\newpage

# 2. Single-sig card set from a seed phrase (file input)

Create a native-segwit (BIP-84, `m/84'/0'/0'`) single-sig 3-card bundle from one
seed phrase held in a file. Write the phrase to `seed0.txt` (here a public test
vector) and feed it on stdin:

```
$ printf '%s\n' 'abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about' > seed0.txt

```

```
$ cat seed0.txt
abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about
```

(`--template` choices for single-sig: `bip44`, `bip49`, `bip84`, `bip86`.) Run
the bundle. stdout carries the three cards to engrave; stderr carries the
human-readable engraving panel and the secret-material warning:

```
$ mnemonic bundle --template bip84 --network mainnet --slot @0.phrase=- < seed0.txt
# ms1 (entropy, BCH-checksummed)
ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f

# mk1 (xpub + origin)
mk1qprsqhpqqsq3cqtsleeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8qp3yxg3dpe854wq4
mk1qprsqhpp0f30mtxzd65mvwcur9usdatwuqvq6z70r9nwrgk6xn6l8gy6nwa2n977sw6zh34rma0nh

# md1 (wallet policy)
md1fgdxlpqpqpm6jzzqqvqpdqw0za5zs4gyy55aq4vsmnhy4s6wyaypu34c7raqu8np
md1fgdxlpqf2zcgefcpupmel75q5435j7seugaj5jr7qyur6vt76es5cdeyrq7zdy0d
md1fgdxlpq3xa2dk8vwpj7gx74hwqxqdp083jehp5tdrfa0n5zdfkqcdlrvnh5r62jn

# === Wallet bundle: bip84, mainnet ===
# ms1: 1c017
# mk1: 1c017
# fingerprint: 73c5da0a
# origin path: m/84'/0'/0'
# Template: bip84
# md1: 1c01
warning: stdout carries private key material (can spend) — redirect or encrypt (e.g. '> file.txt' or '| age -e ...')
```

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

```
$ printf '%s\n' 'abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about' > seed0.txt

```

```
$ printf '%s\n' 'legal winner thank year wave sausage worth useful legal winner thank yellow' > seed1.txt

```

```
$ printf '%s\n' 'letter advice cage absurd amount doctor acoustic avoid letter advice cage above' > seed2.txt

```

On each device, derive that cosigner's BIP-87 multisig fingerprint and account
xpub (`--template wsh-sortedmulti` implies the `m/87'/0'/0'` path) from the seed
file. Cosigner @0:

```
$ cat seed0.txt
abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about
```

```
$ mnemonic convert --from phrase=- --to fingerprint --template wsh-sortedmulti --network mainnet < seed0.txt
fingerprint: 73c5da0a
```

```
$ mnemonic convert --from phrase=- --to xpub        --template wsh-sortedmulti --network mainnet < seed0.txt
xpub: xpub6DBjiYnc4ewKti13Q1L35bqdodw5z3VGJnf516B3icHrEGEUcCuCG5GVQDZtH8Xmsyt3Fs9YDNwLaqjUbbRidwXZ6sxufZcr4VqqzrXvicM
note: stdout is watch-only — public keys only, cannot spend
```

Cosigner @1:

```
$ cat seed1.txt
legal winner thank year wave sausage worth useful legal winner thank yellow
```

```
$ mnemonic convert --from phrase=- --to fingerprint --template wsh-sortedmulti --network mainnet < seed1.txt
fingerprint: b8688df1
```

```
$ mnemonic convert --from phrase=- --to xpub        --template wsh-sortedmulti --network mainnet < seed1.txt
xpub: xpub6CbhrPzY2z7NcCGCGjLAJLq8iRyjUfwmdXQs66MxTVUReKqb9DpLnVJ5D1qpatZjUuPGTyxf5TYU1vA34YFE9FHB4TvfYmokYLVsyEFZFt9
note: stdout is watch-only — public keys only, cannot spend
```

Cosigner @2:

```
$ cat seed2.txt
letter advice cage absurd amount doctor acoustic avoid letter advice cage above
```

```
$ mnemonic convert --from phrase=- --to fingerprint --template wsh-sortedmulti --network mainnet < seed2.txt
fingerprint: 28645006
```

```
$ mnemonic convert --from phrase=- --to xpub        --template wsh-sortedmulti --network mainnet < seed2.txt
xpub: xpub6DB7HNqw6CZojxN85NuFTPWZhi2FagSnexPS1rv3nYQhngkmdHgb7iebYvTFmFKKDA3ozf5yezDsCH6cXAw3WZijviSZtZC2hjHn2uazz4z
note: stdout is watch-only — public keys only, cannot spend
```

Wrap each as an origin-annotated descriptor key `[fingerprint/87'/0'/0']xpub`
and combine into a 2-of-3 sorted-multisig descriptor (`/<0;1>/*` = the
external/change multipath). The assembled descriptor file:

```
$ cat multisig.desc
wsh(sortedmulti(2,[73c5da0a/87'/0'/0']xpub6DBjiYnc4ewKti13Q1L35bqdodw5z3VGJnf516B3icHrEGEUcCuCG5GVQDZtH8Xmsyt3Fs9YDNwLaqjUbbRidwXZ6sxufZcr4VqqzrXvicM/<0;1>/*,[b8688df1/87'/0'/0']xpub6CbhrPzY2z7NcCGCGjLAJLq8iRyjUfwmdXQs66MxTVUReKqb9DpLnVJ5D1qpatZjUuPGTyxf5TYU1vA34YFE9FHB4TvfYmokYLVsyEFZFt9/<0;1>/*,[28645006/87'/0'/0']xpub6DB7HNqw6CZojxN85NuFTPWZhi2FagSnexPS1rv3nYQhngkmdHgb7iebYvTFmFKKDA3ozf5yezDsCH6cXAw3WZijviSZtZC2hjHn2uazz4z/<0;1>/*))
```

Validate and canonicalise it (this also computes the BIP-380 checksum):

```
$ mnemonic export-wallet --descriptor "$(cat multisig.desc)" --format descriptor --network mainnet
wsh(sortedmulti(2,[73c5da0a/87'/0'/0']xpub6DBjiYnc4ewKti13Q1L35bqdodw5z3VGJnf516B3icHrEGEUcCuCG5GVQDZtH8Xmsyt3Fs9YDNwLaqjUbbRidwXZ6sxufZcr4VqqzrXvicM/<0;1>/*,[b8688df1/87'/0'/0']xpub6CbhrPzY2z7NcCGCGjLAJLq8iRyjUfwmdXQs66MxTVUReKqb9DpLnVJ5D1qpatZjUuPGTyxf5TYU1vA34YFE9FHB4TvfYmokYLVsyEFZFt9/<0;1>/*,[28645006/87'/0'/0']xpub6DB7HNqw6CZojxN85NuFTPWZhi2FagSnexPS1rv3nYQhngkmdHgb7iebYvTFmFKKDA3ozf5yezDsCH6cXAw3WZijviSZtZC2hjHn2uazz4z/<0;1>/*))#4wup4at0
note: stdout is watch-only — public keys only, cannot spend
```

The first receive address (here via the BSMS / BIP-129 record, which also
carries the `/0/*,/1/*` derivation):

```
$ mnemonic export-wallet --descriptor "$(cat multisig.desc)" --format bsms --network mainnet
BSMS 1.0
wsh(sortedmulti(2,[73c5da0a/87'/0'/0']xpub6DBjiYnc4ewKti13Q1L35bqdodw5z3VGJnf516B3icHrEGEUcCuCG5GVQDZtH8Xmsyt3Fs9YDNwLaqjUbbRidwXZ6sxufZcr4VqqzrXvicM/<0;1>/*,[b8688df1/87'/0'/0']xpub6CbhrPzY2z7NcCGCGjLAJLq8iRyjUfwmdXQs66MxTVUReKqb9DpLnVJ5D1qpatZjUuPGTyxf5TYU1vA34YFE9FHB4TvfYmokYLVsyEFZFt9/<0;1>/*,[28645006/87'/0'/0']xpub6DB7HNqw6CZojxN85NuFTPWZhi2FagSnexPS1rv3nYQhngkmdHgb7iebYvTFmFKKDA3ozf5yezDsCH6cXAw3WZijviSZtZC2hjHn2uazz4z/<0;1>/*))#4wup4at0
/0/*,/1/*
bc1qkssenl2m6t3aynza394sr9m86vt6md2v76kj52jun2xlwrdeaa4q84qtpl
note: stdout is watch-only — public keys only, cannot spend
```

Engrave the shared watch-only card set from the public descriptor (the md1
policy card is shared by all cosigners; each cosigner additionally backs up
their own seed as a single-sig ms1 set per section 2). With only public xpubs
supplied, the ms1 cards are empty placeholders:

```
$ mnemonic bundle --descriptor-file multisig.desc --network mainnet
# ms1 (omitted — descriptor watch-only mode)

# mk1[0] (cosigner 0 xpub + origin)
mk1qperpupqqspu3s7denyv8nwverpumnrnchdq5pcy3zepa59349dcgs5n5stvpvk3v8x4eqsdngy2jl4wddt5ac2ptv4fya76tpreapyrdfqr
mk1qperpuppnq8kp6xcpqphr7svxwkxx5ag99s9zfyml9v7tcqrexmpdj7jgqgmny8rr0z7vlj7eqzv2486xkkcftrzq8

# mk1[1] (cosigner 1 xpub + origin)
mk1qperpapqqspu3s7denyv8nwverpumn9cdzxlzpcy3zepaqxafl630m4as45q6fz4ltsntlues3e3gylcu6jsa6jdz69hy0whcpg2f28j5awu
mk1qperpapp02p3qtffjqpxtuj95jzaevqs7jqje40324vxlfw0txswawpxte3zmhzp6lrpj3ga2lw65h2dd24cuerkpa

# mk1[2] (cosigner 2 xpub + origin)
mk1qperp7pqqspu3s7denyv8nwverpumnpgv3gqvpcy3zepanecw97sn9g25uukrgxca47at8as54xhd0rkgx57cs78mc8fk6507cs8tnugnpnf
mk1qperp7ppr4e59ef7u5p038j900ye439hsphez506fra5k3eac5kg0n7jhn3kpzpna6n0lygdlpfeq4a75lc2mrrz7f

# md1 (multisig wallet policy)
md1f5przzspq3m67zzqqvzrs3pstucw0za5znwrg3hcc5xg5qxzhs7yyg2f6g9kqktgkrn2spp6tlfzprv6ye
md1f5przzswgyrv6pz5hatnt2a8wzs2m92ffsrmqarvqsqm3lgxr8trr2w5zjcz3yjdljk0qf2g9tn3u5jtsz
md1f5przzsj7qq7fkctvh5jqzxuepccmchn8u30m4as45q6fz4ltsntlues3e3gylcu6jsasrwqtqpe4mvjxf
md1f5przzsafx3dzmj02p3qtffjqpxtuj95jzaevqs7jqje40324vxlfw0txswawpxte3zmspu35hx7lrg2t6
md1f5przz3r3qa03segkpx2s4feevxsd3mta6k0mpf2dw678vsdfa3pu0hswnvwhxsh98mjsqpsz8xkjh9l6g
md1f5przz3gzlz0y277fntzt0qr0j9gl5j8mfdrnm3fvsl8a908rvzyr8m4xl7gs9e2t6dyvjgv4j

# === Wallet bundle: descriptor, mainnet ===
# Threshold: 2 of 3
# Cosigners:
#   @0: (no ms1; watch-only),mk1:c8c3c (73c5da0a @ m/87'/0'/0')
#   @1: (no ms1; watch-only),mk1:c8c3d (b8688df1 @ m/87'/0'/0')
#   @2: (no ms1; watch-only),mk1:c8c3e (28645006 @ m/87'/0'/0')
# Template: descriptor
# md1: c8c3
# Recovery: any 2 of 3 signing keys + md1 (template card).
note: stdout is watch-only — public keys only, cannot spend
```

## 3.4 Building from all seeds on one machine (multiple files, one command)

If instead you hold all the seeds yourself, you can build the whole bundle in a
single command, reading each seed from its own file into an environment
variable. It needs no per-device coordination, and the seeds stay off `argv`:
command substitution would put all three there, which the toolkit refuses at
exit 2 before parsing. The three seed files (shown again for reference):

```
$ cat seed0.txt
abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about
```

```
$ cat seed1.txt
legal winner thank year wave sausage worth useful legal winner thank yellow
```

```
$ cat seed2.txt
letter advice cage absurd amount doctor acoustic avoid letter advice cage above
```

Because seeds (not just xpubs) are supplied, this emits the **full secret card
set** -- one `ms1` per cosigner -- not the watch-only placeholders of 3.3:

```
$ export SEED0="$(cat seed0.txt)" SEED1="$(cat seed1.txt)" SEED2="$(cat seed2.txt)"

```

```
$ mnemonic bundle --template wsh-sortedmulti --threshold 2 --network mainnet --slot "@0.phrase=@env:SEED0" --slot "@1.phrase=@env:SEED1" --slot "@2.phrase=@env:SEED2"
# ms1[0] (entropy, BCH-checksummed)
ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f

# ms1[1] (entropy, BCH-checksummed)
ms10entrsqplh7lml0alh7lml0alh7lml0als5cclar2zmksh6

# ms1[2] (entropy, BCH-checksummed)
ms10entrsqzqgpqyqszqgpqyqszqgpqyqszqqlfm7mep84hunu

# mk1[0] (cosigner 0 xpub + origin)
mk1qperpupqqspu3s7denyv8nwverpumnrnchdq5pcy3zepa59349dcgs5n5stvpvk3v8x4eqsdngy2jl4wddt5ac2ptv4fya76tpreapyrdfqr
mk1qperpuppnq8kp6xcpqphr7svxwkxx5ag99s9zfyml9v7tcqrexmpdj7jgqgmny8rr0z7vlj7eqzv2486xkkcftrzq8

# mk1[1] (cosigner 1 xpub + origin)
mk1qperpapqqspu3s7denyv8nwverpumn9cdzxlzpcy3zepaqxafl630m4as45q6fz4ltsntlues3e3gylcu6jsa6jdz69hy0whcpg2f28j5awu
mk1qperpapp02p3qtffjqpxtuj95jzaevqs7jqje40324vxlfw0txswawpxte3zmhzp6lrpj3ga2lw65h2dd24cuerkpa

# mk1[2] (cosigner 2 xpub + origin)
mk1qperp7pqqspu3s7denyv8nwverpumnpgv3gqvpcy3zepanecw97sn9g25uukrgxca47at8as54xhd0rkgx57cs78mc8fk6507cs8tnugnpnf
mk1qperp7ppr4e59ef7u5p038j900ye439hsphez506fra5k3eac5kg0n7jhn3kpzpna6n0lygdlpfeq4a75lc2mrrz7f

# md1 (multisig wallet policy)
md1f5przzspq3m67zzqqvzrs3pstucw0za5znwrg3hcc5xg5qxzhs7yyg2f6g9kqktgkrn2spp6tlfzprv6ye
md1f5przzswgyrv6pz5hatnt2a8wzs2m92ffsrmqarvqsqm3lgxr8trr2w5zjcz3yjdljk0qf2g9tn3u5jtsz
md1f5przzsj7qq7fkctvh5jqzxuepccmchn8u30m4as45q6fz4ltsntlues3e3gylcu6jsasrwqtqpe4mvjxf
md1f5przzsafx3dzmj02p3qtffjqpxtuj95jzaevqs7jqje40324vxlfw0txswawpxte3zmspu35hx7lrg2t6
md1f5przz3r3qa03segkpx2s4feevxsd3mta6k0mpf2dw678vsdfa3pu0hswnvwhxsh98mjsqpsz8xkjh9l6g
md1f5przz3gzlz0y277fntzt0qr0j9gl5j8mfdrnm3fvsl8a908rvzyr8m4xl7gs9e2t6dyvjgv4j

# === Wallet bundle: wsh-sortedmulti, mainnet ===
# Threshold: 2 of 3
# Cosigners:
#   @0: ms1:c8c3c,mk1:c8c3c (73c5da0a @ m/87'/0'/0')
#   @1: ms1:c8c3d,mk1:c8c3d (b8688df1 @ m/87'/0'/0')
#   @2: ms1:c8c3e,mk1:c8c3e (28645006 @ m/87'/0'/0')
# Template: wsh-sortedmulti
# md1: c8c3
# Recovery: any 2 of 3 signing keys + md1 (template card).
warning: stdout carries private key material (can spend) — redirect or encrypt (e.g. '> file.txt' or '| age -e ...')
```

Only one secret may arrive on stdin, so you cannot replace more than one
substitution with the `=-` file-redirect form -- a second `=-` is rejected:

```
$ mnemonic bundle --template wsh-sortedmulti --threshold 2 --network mainnet --slot @0.phrase=- --slot @1.phrase=- --slot "@2.phrase=@env:SEED2" < seed0.txt
error: at most one --slot @N.<secret>=- per invocation (single stdin per invocation)
```

\newpage

# 4. Card set -> Bitcoin Core wallet descriptor (and how to import)

`mnemonic restore --md1 <chunks>` reconstructs the watch-only wallet from the
**shared md1 card alone** -- no seeds needed. First produce that card from the
section-3 wallet (descriptor file shown again) and pull out its md1 chunks:

```
$ cat multisig.desc
wsh(sortedmulti(2,[73c5da0a/87'/0'/0']xpub6DBjiYnc4ewKti13Q1L35bqdodw5z3VGJnf516B3icHrEGEUcCuCG5GVQDZtH8Xmsyt3Fs9YDNwLaqjUbbRidwXZ6sxufZcr4VqqzrXvicM/<0;1>/*,[b8688df1/87'/0'/0']xpub6CbhrPzY2z7NcCGCGjLAJLq8iRyjUfwmdXQs66MxTVUReKqb9DpLnVJ5D1qpatZjUuPGTyxf5TYU1vA34YFE9FHB4TvfYmokYLVsyEFZFt9/<0;1>/*,[28645006/87'/0'/0']xpub6DB7HNqw6CZojxN85NuFTPWZhi2FagSnexPS1rv3nYQhngkmdHgb7iebYvTFmFKKDA3ozf5yezDsCH6cXAw3WZijviSZtZC2hjHn2uazz4z/<0;1>/*))
```

```
$ mnemonic bundle --descriptor-file multisig.desc --network mainnet --json | jq -r ".md1[]" > multisig.md1
note: stdout is watch-only — public keys only, cannot spend
```

```
$ cat multisig.md1
md1f5przzspq3m67zzqqvzrs3pstucw0za5znwrg3hcc5xg5qxzhs7yyg2f6g9kqktgkrn2spp6tlfzprv6ye
md1f5przzswgyrv6pz5hatnt2a8wzs2m92ffsrmqarvqsqm3lgxr8trr2w5zjcz3yjdljk0qf2g9tn3u5jtsz
md1f5przzsj7qq7fkctvh5jqzxuepccmchn8u30m4as45q6fz4ltsntlues3e3gylcu6jsasrwqtqpe4mvjxf
md1f5przzsafx3dzmj02p3qtffjqpxtuj95jzaevqs7jqje40324vxlfw0txswawpxte3zmspu35hx7lrg2t6
md1f5przz3r3qa03segkpx2s4feevxsd3mta6k0mpf2dw678vsdfa3pu0hswnvwhxsh98mjsqpsz8xkjh9l6g
md1f5przz3gzlz0y277fntzt0qr0j9gl5j8mfdrnm3fvsl8a908rvzyr8m4xl7gs9e2t6dyvjgv4j
```

Restore reconstructs the wallet from exactly those chunks. The default form
prints the descriptor and first address (note the address matches section 3 --
same wallet -- while the descriptor *string* differs because the md1 card stores
each key as a depth-0 master xpub, an equivalent serialisation):

```
$ mnemonic restore --network mainnet $(sed "s/^/--md1 /" multisig.md1)
2-of-3 multisig restore
CONFIRM: verify each cosigner fingerprint against your records before importing.
  descriptor: wsh(sortedmulti(2,[73c5da0a/87'/0'/0']xpub661MyMwAqRbcFrooZ2966EcDmVX5MoFXZhuJqXTudvJzwBTBfPQSc5JzX52fvS18oqSdEJXJ4kTGRJ76wPWDUSNJsY5JsgVBQoD6KrbdCLL/<0;1>/*,[b8688df1/87'/0'/0']xpub661MyMwAqRbcEnFgxHRLx7i1fnjcBPgc71qy8mVkbGXYukNGMK2XFRbAaCLYEJDUufNoBxTNa68i5MYhqmrEkfhjzgHCUEcvJBhXS5bk4RW/<0;1>/*,[28645006/87'/0'/0']xpub661MyMwAqRbcEdy4jr5EtEhQBctfscE6a99DGLr2cW4HnnmBsXDoe3odGzRiw3hcRM5wfKcQmb7s5FjdGrR6SrExXmeopaoY9Lk7tQusDjN/<0;1>/*))#yjp7hj7w
  first recv: bc1qkssenl2m6t3aynza394sr9m86vt6md2v76kj52jun2xlwrdeaa4q84qtpl
  cosigner @0: 73c5da0a [87'/0'/0']  from md1 (not independently verified)
  cosigner @1: b8688df1 [87'/0'/0']  from md1 (not independently verified)
  cosigner @2: 28645006 [87'/0'/0']  from md1 (not independently verified)
UNVERIFIED: no --from/--cosigner cross-check supplied; verify each cosigner fingerprint above against your records before importing
note: stdout is watch-only — public keys only, cannot spend
```

Add `--format bitcoin-core` for a ready-to-import `importdescriptors` request
array (external `.../0/*` + change `.../1/*`):

```
$ mnemonic restore --network mainnet $(sed "s/^/--md1 /" multisig.md1) --format bitcoin-core
[
  {
    "active": true,
    "desc": "wsh(sortedmulti(2,[73c5da0a/87'/0'/0']xpub661MyMwAqRbcFrooZ2966EcDmVX5MoFXZhuJqXTudvJzwBTBfPQSc5JzX52fvS18oqSdEJXJ4kTGRJ76wPWDUSNJsY5JsgVBQoD6KrbdCLL/0/*,[b8688df1/87'/0'/0']xpub661MyMwAqRbcEnFgxHRLx7i1fnjcBPgc71qy8mVkbGXYukNGMK2XFRbAaCLYEJDUufNoBxTNa68i5MYhqmrEkfhjzgHCUEcvJBhXS5bk4RW/0/*,[28645006/87'/0'/0']xpub661MyMwAqRbcEdy4jr5EtEhQBctfscE6a99DGLr2cW4HnnmBsXDoe3odGzRiw3hcRM5wfKcQmb7s5FjdGrR6SrExXmeopaoY9Lk7tQusDjN/0/*))#y65a0dtg",
    "internal": false,
    "range": [
      0,
      999
    ],
    "timestamp": 0
  },
  {
    "active": true,
    "desc": "wsh(sortedmulti(2,[73c5da0a/87'/0'/0']xpub661MyMwAqRbcFrooZ2966EcDmVX5MoFXZhuJqXTudvJzwBTBfPQSc5JzX52fvS18oqSdEJXJ4kTGRJ76wPWDUSNJsY5JsgVBQoD6KrbdCLL/1/*,[b8688df1/87'/0'/0']xpub661MyMwAqRbcEnFgxHRLx7i1fnjcBPgc71qy8mVkbGXYukNGMK2XFRbAaCLYEJDUufNoBxTNa68i5MYhqmrEkfhjzgHCUEcvJBhXS5bk4RW/1/*,[28645006/87'/0'/0']xpub661MyMwAqRbcEdy4jr5EtEhQBctfscE6a99DGLr2cW4HnnmBsXDoe3odGzRiw3hcRM5wfKcQmb7s5FjdGrR6SrExXmeopaoY9Lk7tQusDjN/1/*))#k0gfvz2t",
    "internal": true,
    "range": [
      0,
      999
    ],
    "timestamp": 0
  }
]
2-of-3 multisig restore
CONFIRM: verify each cosigner fingerprint against your records before importing the payload above.
  descriptor: wsh(sortedmulti(2,[73c5da0a/87'/0'/0']xpub661MyMwAqRbcFrooZ2966EcDmVX5MoFXZhuJqXTudvJzwBTBfPQSc5JzX52fvS18oqSdEJXJ4kTGRJ76wPWDUSNJsY5JsgVBQoD6KrbdCLL/<0;1>/*,[b8688df1/87'/0'/0']xpub661MyMwAqRbcEnFgxHRLx7i1fnjcBPgc71qy8mVkbGXYukNGMK2XFRbAaCLYEJDUufNoBxTNa68i5MYhqmrEkfhjzgHCUEcvJBhXS5bk4RW/<0;1>/*,[28645006/87'/0'/0']xpub661MyMwAqRbcEdy4jr5EtEhQBctfscE6a99DGLr2cW4HnnmBsXDoe3odGzRiw3hcRM5wfKcQmb7s5FjdGrR6SrExXmeopaoY9Lk7tQusDjN/<0;1>/*))#yjp7hj7w
  first recv: bc1qkssenl2m6t3aynza394sr9m86vt6md2v76kj52jun2xlwrdeaa4q84qtpl
  cosigner @0: 73c5da0a [87'/0'/0']  from md1 (not independently verified)
  cosigner @1: b8688df1 [87'/0'/0']  from md1 (not independently verified)
  cosigner @2: 28645006 [87'/0'/0']  from md1 (not independently verified)
UNVERIFIED: no --from/--cosigner cross-check supplied; verify each cosigner fingerprint above against your records before importing
note: stdout is watch-only — public keys only, cannot spend
```

Import into Bitcoin Core: save the array, create a blank descriptor wallet, and
load it (these run against your own node, so their output is not shown here):

```
$ mnemonic restore --network mainnet $(sed "s/^/--md1 /" multisig.md1) --format bitcoin-core > wallet.json
$ bitcoin-cli -named createwallet wallet_name="multisig-2of3" disable_private_keys=true blank=true descriptors=true
$ bitcoin-cli -rpcwallet="multisig-2of3" importdescriptors "$(cat wallet.json)"
$ bitcoin-cli -rpcwallet="multisig-2of3" getnewaddress
```

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

```
$ python3 -c "import hashlib; w=b'opensessame'; X=hashlib.sha256(w).digest(); print('preimage X =', X.hex()); print('hash H     =', hashlib.sha256(X).hexdigest())"
preimage X = cd6f70a4440de96063f8dbd7a4a3bbcff6b993af6c83e08d87b359e0760ca9c6
hash H     = a84dce40975727c398023cfbd50d5db3b9662375521d0f1ac62dbd829b9a08ad
```

## 5.2 The guided builder caps complexity -- use the raw descriptor path

`mnemonic build-descriptor` runs a satisfiability + cost preview that it
**bounds** for funds-safety. An 11-key, 4-branch policy exceeds that envelope, so
the guided builder refuses and points you at the raw `--descriptor` path. The
policy-tree spec it reads:

```
$ cat policy.json
{
  "schema_version": 1,
  "wrapper": "wsh",
  "root": {
    "or_i": [
      {
        "and_v": [
          {
            "wrap": {
              "w": "v",
              "sub": {
                "after": 1000000
              }
            }
          },
          {
            "and_v": [
              {
                "wrap": {
                  "w": "v",
                  "sub": {
                    "sha256": "a84dce40975727c398023cfbd50d5db3b9662375521d0f1ac62dbd829b9a08ad"
                  }
                }
              },
              {
                "multi": {
                  "k": 3,
                  "keys": [
                    "[73c5da0a/84'/0'/0']xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V",
                    "[73c5da0a/84'/0'/1']xpub6CatWdiZiodmYVtWLtEQsAg1H9ooS1bmsJUBwQ83FE1Fyk386FWcyicJgEZv3quZSJKA5dh5Lo2PbubMGxCfZtRthV6ST2qquL9w3HSzcUn",
                    "[73c5da0a/84'/0'/2']xpub6CatWdiZiodmbNGqcQxxjGN165QxTU4PwNNi9WrijYgYf7VxcmuFxosRw3foczLgRDbjDjJbqZhPCTfkcaWmL9BuSw98ybKKJtcHgWeryy6"
                  ]
                }
              }
            ]
          }
        ]
      },
      {
        "or_i": [
          {
            "and_v": [
              {
                "wrap": {
                  "w": "v",
                  "sub": {
                    "after": 1893456000
                  }
                }
              },
              {
                "and_v": [
                  {
                    "wrap": {
                      "w": "v",
                      "sub": {
                        "sha256": "a84dce40975727c398023cfbd50d5db3b9662375521d0f1ac62dbd829b9a08ad"
                      }
                    }
                  },
                  {
                    "multi": {
                      "k": 2,
                      "keys": [
                        "[73c5da0a/84'/0'/3']xpub6CatWdiZiodmdHurRokjbycCrxddTDJgTsyEAaQfKjkWbwUi79LAWG5gHjMCQB7BeJc47MkubXuZdf45JZHK1qcr1GZ5EwREUDVDLVdPkEC",
                        "[b8688df1/84'/0'/0']xpub6DNfJehqF1LUs9kwaqDu12Ajpz9psYVtbGhTykQo1CYdkkqV2vAyR4DiWXSTTDujWHzVy1AtV6ENGKWgwbLWqa4wXMZR4ZmdpRjQBG5EgTV",
                        "[b8688df1/84'/0'/1']xpub6DNfJehqF1LUsoh1a4XLG3yuFY1oCRJd6Bkba97xvW6SwvY71Nc6LvkqfDLWuyCivJ4eMDxgFtiTxnazgp8rYVzQSKR3L8EGRs9asBarqpe"
                      ]
                    }
                  }
                ]
              }
            ]
          },
          {
            "or_i": [
              {
                "and_v": [
                  {
                    "wrap": {
                      "w": "v",
                      "sub": {
                        "older": 65535
                      }
                    }
                  },
                  {
                    "multi": {
                      "k": 2,
                      "keys": [
                        "[b8688df1/84'/0'/2']xpub6DNfJehqF1LUwZ1DrFcQN365CDMPLSsrqpSwaz4eRrtYz9qWdsVH9JqsHpQ6yNGdG4VXGKbLgcxKkXtwTB5B4iCuHcmjomqR8z6NNxNpU51",
                        "[b8688df1/84'/0'/3']xpub6DNfJehqF1LUxb8gRSstbR9LcAxWgwD4V678z5FZ7BLftWzwQCt3yZb5eW4U8AVJwbKcZ2iegjjvKv2xggJpMBRkk3CE7bz7g4uMV7Qp3TU"
                      ]
                    }
                  }
                ]
              },
              {
                "and_v": [
                  {
                    "wrap": {
                      "w": "v",
                      "sub": {
                        "older": 4255898
                      }
                    }
                  },
                  {
                    "multi": {
                      "k": 1,
                      "keys": [
                        "[28645006/84'/0'/0']xpub6DBbzvudcQg2nS4tHSkrm7FGkXL6arWxEYoZ1g1GWbaNdWRobcJmxH8KQdjuzTZJtwDveibGd83eGzdGCR6NjSqpU8p2Xx4wWQK7iGBycAm",
                        "[28645006/84'/0'/1']xpub6DBbzvudcQg2sPDRWpqfEn5VzPiwrd1zNes4aHmmNUog9Jmc2fc2JSfM2E39YZy2iakmpWRpa3rXhzGNmd4GKiJEZxmTCftQpwBGd9ihVks",
                        "[28645006/84'/0'/2']xpub6DBbzvudcQg2uk9BrxsuxCWjsYsbfPYkPahQfmTVABfJ4j8TRxUnRd5eGgEXfgPJ63xcuTV9uny7pQBFbu3XKCn9rNxcNGRaDT9BD1gkBGZ"
                      ]
                    }
                  }
                ]
              }
            ]
          }
        ]
      }
    ]
  }
}
```

Running the guided builder on it:

```
$ mnemonic build-descriptor --spec policy.json --network mainnet
build-descriptor: refused — 1 diagnostic(s):
  [over_envelope] root: policy exceeds the always-previewable envelope (2^(11 keys + 2 hashes) × 9 timelock-states > cap 4096); use the raw `--descriptor` path for arbitrarily complex policies
```

(For a policy *within* the envelope -- fewer keys -- `build-descriptor --spec`
validates and emits it for you.) For arbitrarily complex policies you hand the
miniscript descriptor straight to `export-wallet` / `bundle`. The hand-written
descriptor file:

```
$ cat policy.desc
wsh(or_i(and_v(v:after(1000000),and_v(v:sha256(a84dce40975727c398023cfbd50d5db3b9662375521d0f1ac62dbd829b9a08ad),multi(3,[73c5da0a/84'/0'/0']xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V/<0;1>/*,[73c5da0a/84'/0'/1']xpub6CatWdiZiodmYVtWLtEQsAg1H9ooS1bmsJUBwQ83FE1Fyk386FWcyicJgEZv3quZSJKA5dh5Lo2PbubMGxCfZtRthV6ST2qquL9w3HSzcUn/<0;1>/*,[73c5da0a/84'/0'/2']xpub6CatWdiZiodmbNGqcQxxjGN165QxTU4PwNNi9WrijYgYf7VxcmuFxosRw3foczLgRDbjDjJbqZhPCTfkcaWmL9BuSw98ybKKJtcHgWeryy6/<0;1>/*))),or_i(and_v(v:after(1893456000),and_v(v:sha256(a84dce40975727c398023cfbd50d5db3b9662375521d0f1ac62dbd829b9a08ad),multi(2,[73c5da0a/84'/0'/3']xpub6CatWdiZiodmdHurRokjbycCrxddTDJgTsyEAaQfKjkWbwUi79LAWG5gHjMCQB7BeJc47MkubXuZdf45JZHK1qcr1GZ5EwREUDVDLVdPkEC/<0;1>/*,[b8688df1/84'/0'/0']xpub6DNfJehqF1LUs9kwaqDu12Ajpz9psYVtbGhTykQo1CYdkkqV2vAyR4DiWXSTTDujWHzVy1AtV6ENGKWgwbLWqa4wXMZR4ZmdpRjQBG5EgTV/<0;1>/*,[b8688df1/84'/0'/1']xpub6DNfJehqF1LUsoh1a4XLG3yuFY1oCRJd6Bkba97xvW6SwvY71Nc6LvkqfDLWuyCivJ4eMDxgFtiTxnazgp8rYVzQSKR3L8EGRs9asBarqpe/<0;1>/*))),or_i(and_v(v:older(65535),multi(2,[b8688df1/84'/0'/2']xpub6DNfJehqF1LUwZ1DrFcQN365CDMPLSsrqpSwaz4eRrtYz9qWdsVH9JqsHpQ6yNGdG4VXGKbLgcxKkXtwTB5B4iCuHcmjomqR8z6NNxNpU51/<0;1>/*,[b8688df1/84'/0'/3']xpub6DNfJehqF1LUxb8gRSstbR9LcAxWgwD4V678z5FZ7BLftWzwQCt3yZb5eW4U8AVJwbKcZ2iegjjvKv2xggJpMBRkk3CE7bz7g4uMV7Qp3TU/<0;1>/*)),and_v(v:older(4255898),multi(1,[28645006/84'/0'/0']xpub6DBbzvudcQg2nS4tHSkrm7FGkXL6arWxEYoZ1g1GWbaNdWRobcJmxH8KQdjuzTZJtwDveibGd83eGzdGCR6NjSqpU8p2Xx4wWQK7iGBycAm/<0;1>/*,[28645006/84'/0'/1']xpub6DBbzvudcQg2sPDRWpqfEn5VzPiwrd1zNes4aHmmNUog9Jmc2fc2JSfM2E39YZy2iakmpWRpa3rXhzGNmd4GKiJEZxmTCftQpwBGd9ihVks/<0;1>/*,[28645006/84'/0'/2']xpub6DBbzvudcQg2uk9BrxsuxCWjsYsbfPYkPahQfmTVABfJ4j8TRxUnRd5eGgEXfgPJ63xcuTV9uny7pQBFbu3XKCn9rNxcNGRaDT9BD1gkBGZ/<0;1>/*))))))#4ld0crxa
```

Validate and canonicalise it (this adds the BIP-380 checksum). The full
canonical descriptor, with every xpub in full:

```
$ mnemonic export-wallet --descriptor "$(cat policy.desc)" --format descriptor --network mainnet
wsh(or_i(and_v(v:after(1000000),and_v(v:sha256(a84dce40975727c398023cfbd50d5db3b9662375521d0f1ac62dbd829b9a08ad),multi(3,[73c5da0a/84'/0'/0']xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V/<0;1>/*,[73c5da0a/84'/0'/1']xpub6CatWdiZiodmYVtWLtEQsAg1H9ooS1bmsJUBwQ83FE1Fyk386FWcyicJgEZv3quZSJKA5dh5Lo2PbubMGxCfZtRthV6ST2qquL9w3HSzcUn/<0;1>/*,[73c5da0a/84'/0'/2']xpub6CatWdiZiodmbNGqcQxxjGN165QxTU4PwNNi9WrijYgYf7VxcmuFxosRw3foczLgRDbjDjJbqZhPCTfkcaWmL9BuSw98ybKKJtcHgWeryy6/<0;1>/*))),or_i(and_v(v:after(1893456000),and_v(v:sha256(a84dce40975727c398023cfbd50d5db3b9662375521d0f1ac62dbd829b9a08ad),multi(2,[73c5da0a/84'/0'/3']xpub6CatWdiZiodmdHurRokjbycCrxddTDJgTsyEAaQfKjkWbwUi79LAWG5gHjMCQB7BeJc47MkubXuZdf45JZHK1qcr1GZ5EwREUDVDLVdPkEC/<0;1>/*,[b8688df1/84'/0'/0']xpub6DNfJehqF1LUs9kwaqDu12Ajpz9psYVtbGhTykQo1CYdkkqV2vAyR4DiWXSTTDujWHzVy1AtV6ENGKWgwbLWqa4wXMZR4ZmdpRjQBG5EgTV/<0;1>/*,[b8688df1/84'/0'/1']xpub6DNfJehqF1LUsoh1a4XLG3yuFY1oCRJd6Bkba97xvW6SwvY71Nc6LvkqfDLWuyCivJ4eMDxgFtiTxnazgp8rYVzQSKR3L8EGRs9asBarqpe/<0;1>/*))),or_i(and_v(v:older(65535),multi(2,[b8688df1/84'/0'/2']xpub6DNfJehqF1LUwZ1DrFcQN365CDMPLSsrqpSwaz4eRrtYz9qWdsVH9JqsHpQ6yNGdG4VXGKbLgcxKkXtwTB5B4iCuHcmjomqR8z6NNxNpU51/<0;1>/*,[b8688df1/84'/0'/3']xpub6DNfJehqF1LUxb8gRSstbR9LcAxWgwD4V678z5FZ7BLftWzwQCt3yZb5eW4U8AVJwbKcZ2iegjjvKv2xggJpMBRkk3CE7bz7g4uMV7Qp3TU/<0;1>/*)),and_v(v:older(4255898),multi(1,[28645006/84'/0'/0']xpub6DBbzvudcQg2nS4tHSkrm7FGkXL6arWxEYoZ1g1GWbaNdWRobcJmxH8KQdjuzTZJtwDveibGd83eGzdGCR6NjSqpU8p2Xx4wWQK7iGBycAm/<0;1>/*,[28645006/84'/0'/1']xpub6DBbzvudcQg2sPDRWpqfEn5VzPiwrd1zNes4aHmmNUog9Jmc2fc2JSfM2E39YZy2iakmpWRpa3rXhzGNmd4GKiJEZxmTCftQpwBGd9ihVks/<0;1>/*,[28645006/84'/0'/2']xpub6DBbzvudcQg2uk9BrxsuxCWjsYsbfPYkPahQfmTVABfJ4j8TRxUnRd5eGgEXfgPJ63xcuTV9uny7pQBFbu3XKCn9rNxcNGRaDT9BD1gkBGZ/<0;1>/*))))))#4ld0crxa
note: stdout is watch-only — public keys only, cannot spend
```

First receive address (Mainnet), via the BSMS record:

```
$ mnemonic export-wallet --descriptor "$(cat policy.desc)" --format bsms --network mainnet
BSMS 1.0
wsh(or_i(and_v(v:after(1000000),and_v(v:sha256(a84dce40975727c398023cfbd50d5db3b9662375521d0f1ac62dbd829b9a08ad),multi(3,[73c5da0a/84'/0'/0']xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V/<0;1>/*,[73c5da0a/84'/0'/1']xpub6CatWdiZiodmYVtWLtEQsAg1H9ooS1bmsJUBwQ83FE1Fyk386FWcyicJgEZv3quZSJKA5dh5Lo2PbubMGxCfZtRthV6ST2qquL9w3HSzcUn/<0;1>/*,[73c5da0a/84'/0'/2']xpub6CatWdiZiodmbNGqcQxxjGN165QxTU4PwNNi9WrijYgYf7VxcmuFxosRw3foczLgRDbjDjJbqZhPCTfkcaWmL9BuSw98ybKKJtcHgWeryy6/<0;1>/*))),or_i(and_v(v:after(1893456000),and_v(v:sha256(a84dce40975727c398023cfbd50d5db3b9662375521d0f1ac62dbd829b9a08ad),multi(2,[73c5da0a/84'/0'/3']xpub6CatWdiZiodmdHurRokjbycCrxddTDJgTsyEAaQfKjkWbwUi79LAWG5gHjMCQB7BeJc47MkubXuZdf45JZHK1qcr1GZ5EwREUDVDLVdPkEC/<0;1>/*,[b8688df1/84'/0'/0']xpub6DNfJehqF1LUs9kwaqDu12Ajpz9psYVtbGhTykQo1CYdkkqV2vAyR4DiWXSTTDujWHzVy1AtV6ENGKWgwbLWqa4wXMZR4ZmdpRjQBG5EgTV/<0;1>/*,[b8688df1/84'/0'/1']xpub6DNfJehqF1LUsoh1a4XLG3yuFY1oCRJd6Bkba97xvW6SwvY71Nc6LvkqfDLWuyCivJ4eMDxgFtiTxnazgp8rYVzQSKR3L8EGRs9asBarqpe/<0;1>/*))),or_i(and_v(v:older(65535),multi(2,[b8688df1/84'/0'/2']xpub6DNfJehqF1LUwZ1DrFcQN365CDMPLSsrqpSwaz4eRrtYz9qWdsVH9JqsHpQ6yNGdG4VXGKbLgcxKkXtwTB5B4iCuHcmjomqR8z6NNxNpU51/<0;1>/*,[b8688df1/84'/0'/3']xpub6DNfJehqF1LUxb8gRSstbR9LcAxWgwD4V678z5FZ7BLftWzwQCt3yZb5eW4U8AVJwbKcZ2iegjjvKv2xggJpMBRkk3CE7bz7g4uMV7Qp3TU/<0;1>/*)),and_v(v:older(4255898),multi(1,[28645006/84'/0'/0']xpub6DBbzvudcQg2nS4tHSkrm7FGkXL6arWxEYoZ1g1GWbaNdWRobcJmxH8KQdjuzTZJtwDveibGd83eGzdGCR6NjSqpU8p2Xx4wWQK7iGBycAm/<0;1>/*,[28645006/84'/0'/1']xpub6DBbzvudcQg2sPDRWpqfEn5VzPiwrd1zNes4aHmmNUog9Jmc2fc2JSfM2E39YZy2iakmpWRpa3rXhzGNmd4GKiJEZxmTCftQpwBGd9ihVks/<0;1>/*,[28645006/84'/0'/2']xpub6DBbzvudcQg2uk9BrxsuxCWjsYsbfPYkPahQfmTVABfJ4j8TRxUnRd5eGgEXfgPJ63xcuTV9uny7pQBFbu3XKCn9rNxcNGRaDT9BD1gkBGZ/<0;1>/*))))))#4ld0crxa
/0/*,/1/*
bc1q4g7564xxd9hj68hqwu5e558cqafhsklerkr0asfzqp6puq74veesrp6qss
note: stdout is watch-only — public keys only, cannot spend
```

## 5.3 Engrave the card set

Because every key is **distinct**, this is a valid BIP-388 wallet policy, so --
unlike a key-reusing policy -- `bundle` will engrave it. With only public xpubs
supplied, the result is watch-only (the ms1 cards are empty placeholders). The
descriptor file it reads:

```
$ cat policy.desc
wsh(or_i(and_v(v:after(1000000),and_v(v:sha256(a84dce40975727c398023cfbd50d5db3b9662375521d0f1ac62dbd829b9a08ad),multi(3,[73c5da0a/84'/0'/0']xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V/<0;1>/*,[73c5da0a/84'/0'/1']xpub6CatWdiZiodmYVtWLtEQsAg1H9ooS1bmsJUBwQ83FE1Fyk386FWcyicJgEZv3quZSJKA5dh5Lo2PbubMGxCfZtRthV6ST2qquL9w3HSzcUn/<0;1>/*,[73c5da0a/84'/0'/2']xpub6CatWdiZiodmbNGqcQxxjGN165QxTU4PwNNi9WrijYgYf7VxcmuFxosRw3foczLgRDbjDjJbqZhPCTfkcaWmL9BuSw98ybKKJtcHgWeryy6/<0;1>/*))),or_i(and_v(v:after(1893456000),and_v(v:sha256(a84dce40975727c398023cfbd50d5db3b9662375521d0f1ac62dbd829b9a08ad),multi(2,[73c5da0a/84'/0'/3']xpub6CatWdiZiodmdHurRokjbycCrxddTDJgTsyEAaQfKjkWbwUi79LAWG5gHjMCQB7BeJc47MkubXuZdf45JZHK1qcr1GZ5EwREUDVDLVdPkEC/<0;1>/*,[b8688df1/84'/0'/0']xpub6DNfJehqF1LUs9kwaqDu12Ajpz9psYVtbGhTykQo1CYdkkqV2vAyR4DiWXSTTDujWHzVy1AtV6ENGKWgwbLWqa4wXMZR4ZmdpRjQBG5EgTV/<0;1>/*,[b8688df1/84'/0'/1']xpub6DNfJehqF1LUsoh1a4XLG3yuFY1oCRJd6Bkba97xvW6SwvY71Nc6LvkqfDLWuyCivJ4eMDxgFtiTxnazgp8rYVzQSKR3L8EGRs9asBarqpe/<0;1>/*))),or_i(and_v(v:older(65535),multi(2,[b8688df1/84'/0'/2']xpub6DNfJehqF1LUwZ1DrFcQN365CDMPLSsrqpSwaz4eRrtYz9qWdsVH9JqsHpQ6yNGdG4VXGKbLgcxKkXtwTB5B4iCuHcmjomqR8z6NNxNpU51/<0;1>/*,[b8688df1/84'/0'/3']xpub6DNfJehqF1LUxb8gRSstbR9LcAxWgwD4V678z5FZ7BLftWzwQCt3yZb5eW4U8AVJwbKcZ2iegjjvKv2xggJpMBRkk3CE7bz7g4uMV7Qp3TU/<0;1>/*)),and_v(v:older(4255898),multi(1,[28645006/84'/0'/0']xpub6DBbzvudcQg2nS4tHSkrm7FGkXL6arWxEYoZ1g1GWbaNdWRobcJmxH8KQdjuzTZJtwDveibGd83eGzdGCR6NjSqpU8p2Xx4wWQK7iGBycAm/<0;1>/*,[28645006/84'/0'/1']xpub6DBbzvudcQg2sPDRWpqfEn5VzPiwrd1zNes4aHmmNUog9Jmc2fc2JSfM2E39YZy2iakmpWRpa3rXhzGNmd4GKiJEZxmTCftQpwBGd9ihVks/<0;1>/*,[28645006/84'/0'/2']xpub6DBbzvudcQg2uk9BrxsuxCWjsYsbfPYkPahQfmTVABfJ4j8TRxUnRd5eGgEXfgPJ63xcuTV9uny7pQBFbu3XKCn9rNxcNGRaDT9BD1gkBGZ/<0;1>/*))))))#4ld0crxa
```

The watch-only card set:

```
$ mnemonic bundle --descriptor-file policy.desc --network mainnet
# ms1 (omitted — descriptor watch-only mode)

# mk1[0] (cosigner 0 xpub + origin)
mk1qptfcrzqqs945upmkpd8qwastfcrhvz6wqamqkns8wc95upmkpd8qwastfcrhvz6wqamqkns8wc95upmkpeutks2qvzgs7hd8pnnmcv56c4u
mk1qptfcrzpkg08auetmd998g9tyxuae9vxn38f9gtpr98q8s808l6szjkxjt6r83rk2jg0cqns0f30mtxzd65mvwcur9usv8qhcmu69gql0zrc
mk1qptfcrzz74hwqxqdp083jehp5tdrfa0n5zdfhw6425sqe8pgsyxftjtf4

# mk1[1] (cosigner 1 xpub + origin)
mk1qptfczzqqs945upmkpd8qwastfcrhvz6wqamqkns8wc95upmkpd8qwastfcrhvz6wqamqkns8wc95upmkpeutks2lcpag7x7w00uvz59ukc0
mk1qptfczzpszqgqzyqszqgqzypszqgqzqy3zepulhn90du6setz7wcggw0mnt0jarage0ptr0vxf3c9895ghkha0gk58rcwwcdclrvxvvprnta
mk1qptfczzzdqp82uxnprxcuejaentsxshgr9vudfrx5cc7npaegptf64nsud0nnnt334xdu0p83dn95g54tz

# mk1[2] (cosigner 2 xpub + origin)
mk1qptfcpzqqs945upmkpd8qwastfcrhvz6wqamqkns8wc95upmkpd8qwastfcrhvz6wqamqkns8wc95upmkpeutks2lcpag07c7z8k5qayd8z5
mk1qptfcpzpszqgqzyqszqgqzyzszqgqzqy3zepulhn90d76enfqwwtug4zdpn8r94nrhgnwycc6l2errqlch68l94cdsr0vxvngvzwq0fhe86j
mk1qptfcpzzqupqc34flc5lnhz7dzkjmzwfhtn2tf9tp8385pwpa2rh2p9a6flzxmhl43hhx3cmgcpj625l8l

# mk1[3] (cosigner 3 xpub + origin)
mk1qptfcqzqqs945upmkpd8qwastfcrhvz6wqamqkns8wc95upmkpd8qwastfcrhvz6wqamqkns8wc95upmkpeutks2lcpagqk6f7csue6uzgna
mk1qptfcqzpszqgqzyqszqgqzyrszqgqzqy3zepulhn90d6awkdsml0jm7xjylhyyp7esaklwkwkasq24gnq3578mne3sjr64gdauj68uhy768d
mk1qptfcqzzzqp0dmvslcy68sh0lw93nwwxth488eqjvn67gqjpx7fyf84xj7zjfc29xjtagpghsxz5xlu0rx

# mk1[4] (cosigner 4 xpub + origin)
mk1qptfc8zqqs945upmkpd8qwastfcrhvz6wqamqkns8wc95upmkpd8qwastfcrhvz6wqamqkns8wc95upmkzux3r03qvzgsnwq0e2f7mmwlnnx
mk1qptfc8zpkg0w55t7uh3mhpgjm2ctzuevmwdffap2f5zl9lkdtfh372tg8y9k3d35qm006q30450k4vmqvakpywv7pkdajnnrd36ymdaqksnv
mk1qptfc8zzm36y9wa86gpqpq5yyppy22y3ww23fxkwgjcskxqsga8sgz6tl

# mk1[5] (cosigner 5 xpub + origin)
mk1qptfcxzqqs945upmkpd8qwastfcrhvz6wqamqkns8wc95upmkpd8qwastfcrhvz6wqamqkns8wc95upmkzux3r03lcpagnlnxhkxfprlea74
mk1qptfcxzpszqgqzyqszqgqzypszqgqzqy3zepa6j30mjj26l5afq22n2ct2h3zwrjpsefyfqr5zrzs70jdmtqvx7wwt0z6a3fmfdmqr0khu23
mk1qptfcxzzdqp6ttmcly57ac6vels0u3jj86dd5pe7m355rttw8r3a2s45tszcmrx9xa5qyec4g3crut8d54

# mk1[6] (cosigner 6 xpub + origin)
mk1qptfc9zqqs945upmkpd8qwastfcrhvz6wqamqkns8wc95upmkpd8qwastfcrhvz6wqamqkns8wc95upmkzux3r03lcpagz84k67v3r27gvyw
mk1qptfc9zpszqgqzyqszqgqzyzszqgqzqy3zepa6j30mjemst4u6j4atq6uhd0m3m5l9lvu9cde6pcswdrm73ejss4lts9qugnufy3yg6r2y0f
mk1qptfc9zzvgpqdye4j7rexgzc7vsch5dxs9e587e8u22q38ujtrt54lm26dq30kfqqvm7wyvxmtzy38suff

# mk1[7] (cosigner 7 xpub + origin)
mk1qptfcyzqqs945upmkpd8qwastfcrhvz6wqamqkns8wc95upmkpd8qwastfcrhvz6wqamqkns8wc95upmkzux3r03lcpagd0hpxp2e6dx8r48
mk1qptfcyzpszqgqzyqszqgqzyrszqgqzqy3zepa6j30mjstc3wnact9lauqxytd7f9xhqmtmlma4zhsvtx5p6e3s3rggp2ce7mxam45j76f5qd
mk1qptfcyzz3qpzlzuwww4vzvcksequdfl7d8u34vrr7t6kyjwlagkep2ywk6g2q5h3z3tvy9m06y2yd6kfgs

# mk1[8] (cosigner 8 xpub + origin)
mk1qptfctzqqs945upmkpd8qwastfcrhvz6wqamqkns8wc95upmkpd8qwastfcrhvz6wqamqkns8wc95upmkq5xg5qxqvzgs9n3l96x69pda2ys
mk1qptfctzpkg0dqcwzpsyrk9a808d5f0gennj3x9vjds63rqgzndx78e533rulqyzd58cmvq4nudnky9jhqsh5zpemaskg7pdgxcftukrgv6n5
mk1qptfctzzt4c4w6rtnjmnht889ltvhne3ktx5encnny6spl8gsqf53tn98

# mk1[9] (cosigner 9 xpub + origin)
mk1qptfc2zqqs945upmkpd8qwastfcrhvz6wqamqkns8wc95upmkpd8qwastfcrhvz6wqamqkns8wc95upmkq5xg5qxlcpag9zzktxfdleumyfr
mk1qptfc2zpszqgqzyqszqgqzypszqgqzqy3zepa5rpcgx0jlag0vc85ztty3las62hfw73f3j8zx6t3lmtep8skfujv0xfu3hslfcdrrrp8me0
mk1qptfc2zzlcpn80c5zmmvvxqknngp3d279fuywf7zfw5kqe8wv93v9fvn0m39u65adlnvgwullstspxm44n

# mk1[10] (cosigner 10 xpub + origin)
mk1qptfcfzqqs945upmkpd8qwastfcrhvz6wqamqkns8wc95upmkpd8qwastfcrhvz6wqamqkns8wc95upmkq5xg5qxlcpag56yxxwr4asa24nc
mk1qptfcfzpszqgqzyqszqgqzyzszqgqzqy3zepa5rpcgxwd86v5gunengsxd2x3nhwsl8dfjs79jwhy7qnx8r24vls4gl05j6uxcc7dl2rjmec
mk1qptfcfzz2ypsarcmrl7tgutcg8uf4m7gfcd8lfv33knprwthl06am5gdw9aqnfx4jz8l2kjr0p0ur43e8j

# md1 (multisig wallet policy)
md1ffumxts9z3m6jzqaafpr802gfgaafp9nh4yypm6jzxw75sj3m6jzt802ggrh4yyvaafp9genj3xrjhltjxq
md1ffumxtsgqxpx2v6mqq85yszv6a4pxuusyh2unu8xqz8naa2r2akwukvgm42gws7xkx9k7ctqmfdfe9crr4q
md1ffumxtss5mngy26xzzqyn9xddhpk7cspxdw6snwwgzt4wf7rnqpre774p4wm8wtxyd64yxex7jxulxvqvkc
md1ffumxtsapudvvtdas2de5z9drq3rg4jnxhqqqrll7xppvaxdwqqs8sngvq9zdqeccpeutk5lpf37mlhphsr
md1ffumxt3zpgtnchdq5feutks2xu79mg9yhp5gmu2ms6yd794cdzxlz7ux3r03s2ry2qrf9qs784qr5aw8s98
md1ffumxt3wg5qx52ry2qrpdkssz22ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0zxwq65s8c69ev9z
md1ffumxt34j5jr7qyur6vt76esnw4xmrk8qe0yr02mhqrqxshncevms69k357he6px5mrn2zz033uqkqk8lql
md1ffumxt3u4308vyy88ae4hew375vhs43hkrycuznj6ytmt7h5t2r3u8dqp82uxnprxcuejudws6tu9ntlfdx
md1ffumxtj8xdwq6zaqv4n34yv6nrr6v8h9q9d82kwr347wwd9mtxdypee03z5f5xvuvkkvwu4eplqpedw7ye5
md1ffumxtjvfhzvvd04v33s0utarlj6uxcphkqupqc34flc5lnhz7dzkjmzwfhtn2tf9tp83xfskaweep093tj
md1ffumxtjks9c84gwagyhhf8ugmw8t46ekr0a7t0c6gn7uss8mxrkma6e6mkqp24zvzxnclwk74rja82tmurl
md1ffumxtjeuccfpazqp0dmvslcy68sh0lw93nwwxth488eqjvn67gqjpx7fyf84xj7zjfc2wr604yuru2eq34
md1ffumxtnqams5fd4v93wvkdhx557s4y6p0jlmx45mcl995rjzmgkc6qdhhaqgh668m2kdsxy5ru7p2fk5qgs
md1ffumxtnfmvzgueurvmm8w8gs4m5lfqyqyzsssyy3fgj9ee29zj26l5afq22n2ct2h3zwrj46trnchtrjptz
md1ffumxtnsxr9y3yqwsgv2re7fhdvpsmeeeduttgqwj6778e98hwxnx0urlyv537ntdqw0kuspxl3wyyry2c2
md1ffumxtne55rttw8r3a2s45tszcmrrfmst4u6j4atq6uhd0m3m5l9lvu9cde6pcswdrm73cph9wc0utcf6l5
md1ffumxt5x2zzhawq5rzqgrfxdvhs7fjqk8nyx9arf5pwdplkflzjsyflyjc6a9076kngytumwtunf5hft8gc
md1ffumxt5wtstc3wnact9lauqxytd7f9xhqmtmlma4zhsvtx5p6e3s3rggp2ezqz979cuua2mfvauyk5ut07p
md1ffumxt5nqnx95xg8r20lnflydtqclj743ynhl29kg23r4kjzs99qyrk9a808d5f0gennjslsrzrnnr484e6
md1ffumxt5uc4jfkr2yvpq2d5mclxjxy0nuqsfkslrdszk03kwcsk2uzz7sg880kzer6aw9tk9ypxcwzcyw6fy
md1ffumxt4zrtnjmnht889ltvhne3ktx5e8uhl2rmxpaqj6ey0lvxj46th52vv3c3kju0767gdjm2nnax3x5pw
md1ffumxt4f8skfujv0xfalsrxwl3g9hkccvpd8xsrz64u2ncgunuyjafvpjwuctzc2jexlhznktnnzzu3sml7
md1ffumxt430x4tnf7n9z8y7v6ypn235vam58em2v583vn4e8sye3c64t8u928ma9zqcw3udszulgfsxu8xeyl
md1ffumxt4lluk3chss0cnthusns607jerrdxzxuh07l4mhgs6ut6pxjq3gs642g78tg45

# === Wallet bundle: descriptor, mainnet ===
# Threshold: 3 of 11
# Cosigners:
#   @0: (no ms1; watch-only),mk1:5a703 (73c5da0a @ m/84'/0'/0')
#   @1: (no ms1; watch-only),mk1:5a702 (73c5da0a @ m/84'/0'/1')
#   @2: (no ms1; watch-only),mk1:5a701 (73c5da0a @ m/84'/0'/2')
#   @3: (no ms1; watch-only),mk1:5a700 (73c5da0a @ m/84'/0'/3')
#   @4: (no ms1; watch-only),mk1:5a707 (b8688df1 @ m/84'/0'/0')
#   @5: (no ms1; watch-only),mk1:5a706 (b8688df1 @ m/84'/0'/1')
#   @6: (no ms1; watch-only),mk1:5a705 (b8688df1 @ m/84'/0'/2')
#   @7: (no ms1; watch-only),mk1:5a704 (b8688df1 @ m/84'/0'/3')
#   @8: (no ms1; watch-only),mk1:5a70b (28645006 @ m/84'/0'/0')
#   @9: (no ms1; watch-only),mk1:5a70a (28645006 @ m/84'/0'/1')
#   @10: (no ms1; watch-only),mk1:5a709 (28645006 @ m/84'/0'/2')
# Template: descriptor
# md1: 5a70
# Recovery: any 3 of 11 signing keys + md1 (template card).
note: stdout is watch-only — public keys only, cannot spend
```

## 5.4 Restore round-trip -- the card set reconstructs the same first address

This 11-key, 4-branch policy is about as complex as a BIP-388 wallet gets, but
the md1 cards carry it faithfully: take the md1 chunks and `restore` to read back
the descriptor and its first address. The reconstructed first address is
**identical** to the canonical descriptor's (section 5.2) -- proof the card set
round-trips this whole policy without loss, every `after`/`older`/`sha256` lock
and `multi(...)` threshold across all four `or_i` branches preserved.

Appendix B carries the depth->=2 *taproot* twin of this idea. It used to be the
one shape the shipped `mnemonic` refused to restore; since v0.97.0 it is
supported too, so both round-trips below run on the real shipped binary.

```
$ mnemonic bundle --descriptor-file policy.desc --network mainnet --json | jq -r ".md1[]" > policy.md1
note: stdout is watch-only — public keys only, cannot spend
```

```
$ mnemonic restore --network mainnet $(sed "s/^/--md1 /" policy.md1)
miniscript policy restore (11 cosigners)
CONFIRM: verify each cosigner fingerprint against your records before importing.
  descriptor: wsh(or_i(and_v(v:after(1000000),and_v(v:sha256(a84dce40975727c398023cfbd50d5db3b9662375521d0f1ac62dbd829b9a08ad),multi(3,[73c5da0a/84'/0'/0']xpub661MyMwAqRbcFHMVYpCiBTXd2Caj7vZhNFHJSgE59Aue2yYkXSrz5q9GaQ4rRjJVhHZTsCiHWSzgMS5beaaTHWVmhpGC7SMdqMXHRXZi8as/<0;1>/*,[73c5da0a/84'/0'/1']xpub661MyMwAqRbcGaxoYcLaxHHXZqEgSRQmN2P5ung8MJ8MNE535mLuhq7zjnrMKyA5eX6ehicVbU1FFPU39LGXbY8PmLPLQxVRQmPFa3Q7spa/<0;1>/*,[73c5da0a/84'/0'/2']xpub661MyMwAqRbcGuXAHBK3oquZS1HJiz2fVZ2idNcK4GLGTXJyGZkPK7fviN6euv5GzY18JD3WBG3SoLat23TLAVjhQMxDVMAqymQNhg3RFT8/<0;1>/*))),or_i(and_v(v:after(1893456000),and_v(v:sha256(a84dce40975727c398023cfbd50d5db3b9662375521d0f1ac62dbd829b9a08ad),multi(2,[73c5da0a/84'/0'/3']xpub661MyMwAqRbcGHLCZcLjg25oG8wSyqSE5XNM9uMks6vrpH4pRDC8UmAynovThuKraidMeEKJ2FcqBw1eF76aeu1vrGtLJXUiJXr4r9N1TZQ/<0;1>/*,[b8688df1/84'/0'/0']xpub661MyMwAqRbcGowNgeNcLS8CgL2vnZybpJqtkbCmSQMdq2qzcDWqq3CXXg7x5BqvcNCSaNUw6nisoN7JFK2j3HfxV57nNm2RLKo2UzHgbs6/<0;1>/*,[b8688df1/84'/0'/1']xpub661MyMwAqRbcEv3U8uuxavsQA8LNNYwcNge8rT7SaMS5S8KiEwxoP72TQ8ARYjczPTtVQz6CxcaBTEE3XchmYvSiHcVbC9h17CmyfG7sVq9/<0;1>/*))),or_i(and_v(v:older(65535),multi(2,[b8688df1/84'/0'/2']xpub661MyMwAqRbcG7Xht9EwgNucA47Rmgg8Bn5bNmFdJMkotHQDXirpogQHkVNRcwAy6KwGnUYMUNBFCNaRq4WnsqWW2VNUDdW6ymHXfVpk4c3/<0;1>/*,[b8688df1/84'/0'/3']xpub661MyMwAqRbcEbqBvNkLuDtudGA2PHAbtWUuHKe3CKjZCaLjxLGSG8SJpwBCnXsj8xPGXaV9ZWL3j9ktbed8y1aeNVK95HrkgHfHGBXM5Eh/<0;1>/*)),and_v(v:older(4255898),multi(1,[28645006/84'/0'/0']xpub661MyMwAqRbcEdBofBaGbgnse74WRuyEbXRSmzq8jzthzutDnXTV2yNQPzgs3ubwuNp7yrSHnECoA5xHgnoEDH4HSGWqLtYdi6nWVZCfXPk/<0;1>/*,[28645006/84'/0'/1']xpub661MyMwAqRbcH2WNMbtz4pZ8wDtpxndYo6E4r5o8pXedve17srma1LCEjM8WcpVk67xsc36KpBNtYUdqo5dpcFMzRfzSZSa4C5DRty4eDNF/<0;1>/*,[28645006/84'/0'/2']xpub661MyMwAqRbcGqcAAnB9mhvQsdUx2fKasUoXT2gMpt2tFz94wRfAkhuLhZUJkjQ5pgnd9Ny9EwrgcHbAASVnQShCbfhnGsKAk2k6yGoWXAv/<0;1>/*))))))#jgulue7j
  first recv: bc1q4g7564xxd9hj68hqwu5e558cqafhsklerkr0asfzqp6puq74veesrp6qss
  cosigner @0: 73c5da0a [84'/0'/0']  from md1 (not independently verified)
  cosigner @1: 73c5da0a [84'/0'/1']  from md1 (not independently verified)
  cosigner @2: 73c5da0a [84'/0'/2']  from md1 (not independently verified)
  cosigner @3: 73c5da0a [84'/0'/3']  from md1 (not independently verified)
  cosigner @4: b8688df1 [84'/0'/0']  from md1 (not independently verified)
  cosigner @5: b8688df1 [84'/0'/1']  from md1 (not independently verified)
  cosigner @6: b8688df1 [84'/0'/2']  from md1 (not independently verified)
  cosigner @7: b8688df1 [84'/0'/3']  from md1 (not independently verified)
  cosigner @8: 28645006 [84'/0'/0']  from md1 (not independently verified)
  cosigner @9: 28645006 [84'/0'/1']  from md1 (not independently verified)
  cosigner @10: 28645006 [84'/0'/2']  from md1 (not independently verified)
UNVERIFIED: no --from/--cosigner cross-check supplied; verify each cosigner fingerprint above against your records before importing
note: stdout is watch-only — public keys only, cannot spend
```

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

```
$ cat taproot-4leaf.desc
tr([73c5da0a/84'/0'/4']xpub6CatWdiZiodmeXswr13Gd5aNtNqr2UHCBEsCoL3eEFVaM7n8kY5kS4daaP83gWQncmzL3Wzt79mEiLix6XZs6XQmGcQNeQ4HcjfVTn9TuXE/<0;1>/*,{{and_v(v:after(1000000),and_v(v:sha256(a84dce40975727c398023cfbd50d5db3b9662375521d0f1ac62dbd829b9a08ad),multi_a(3,[73c5da0a/84'/0'/0']xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V/<0;1>/*,[73c5da0a/84'/0'/1']xpub6CatWdiZiodmYVtWLtEQsAg1H9ooS1bmsJUBwQ83FE1Fyk386FWcyicJgEZv3quZSJKA5dh5Lo2PbubMGxCfZtRthV6ST2qquL9w3HSzcUn/<0;1>/*,[73c5da0a/84'/0'/2']xpub6CatWdiZiodmbNGqcQxxjGN165QxTU4PwNNi9WrijYgYf7VxcmuFxosRw3foczLgRDbjDjJbqZhPCTfkcaWmL9BuSw98ybKKJtcHgWeryy6/<0;1>/*))),and_v(v:after(1893456000),and_v(v:sha256(a84dce40975727c398023cfbd50d5db3b9662375521d0f1ac62dbd829b9a08ad),multi_a(2,[73c5da0a/84'/0'/3']xpub6CatWdiZiodmdHurRokjbycCrxddTDJgTsyEAaQfKjkWbwUi79LAWG5gHjMCQB7BeJc47MkubXuZdf45JZHK1qcr1GZ5EwREUDVDLVdPkEC/<0;1>/*,[b8688df1/84'/0'/0']xpub6DNfJehqF1LUs9kwaqDu12Ajpz9psYVtbGhTykQo1CYdkkqV2vAyR4DiWXSTTDujWHzVy1AtV6ENGKWgwbLWqa4wXMZR4ZmdpRjQBG5EgTV/<0;1>/*,[b8688df1/84'/0'/1']xpub6DNfJehqF1LUsoh1a4XLG3yuFY1oCRJd6Bkba97xvW6SwvY71Nc6LvkqfDLWuyCivJ4eMDxgFtiTxnazgp8rYVzQSKR3L8EGRs9asBarqpe/<0;1>/*)))},{and_v(v:older(65535),multi_a(2,[b8688df1/84'/0'/2']xpub6DNfJehqF1LUwZ1DrFcQN365CDMPLSsrqpSwaz4eRrtYz9qWdsVH9JqsHpQ6yNGdG4VXGKbLgcxKkXtwTB5B4iCuHcmjomqR8z6NNxNpU51/<0;1>/*,[b8688df1/84'/0'/3']xpub6DNfJehqF1LUxb8gRSstbR9LcAxWgwD4V678z5FZ7BLftWzwQCt3yZb5eW4U8AVJwbKcZ2iegjjvKv2xggJpMBRkk3CE7bz7g4uMV7Qp3TU/<0;1>/*)),and_v(v:older(4255898),multi_a(1,[28645006/84'/0'/0']xpub6DBbzvudcQg2nS4tHSkrm7FGkXL6arWxEYoZ1g1GWbaNdWRobcJmxH8KQdjuzTZJtwDveibGd83eGzdGCR6NjSqpU8p2Xx4wWQK7iGBycAm/<0;1>/*,[28645006/84'/0'/1']xpub6DBbzvudcQg2sPDRWpqfEn5VzPiwrd1zNes4aHmmNUog9Jmc2fc2JSfM2E39YZy2iakmpWRpa3rXhzGNmd4GKiJEZxmTCftQpwBGd9ihVks/<0;1>/*,[28645006/84'/0'/2']xpub6DBbzvudcQg2uk9BrxsuxCWjsYsbfPYkPahQfmTVABfJ4j8TRxUnRd5eGgEXfgPJ63xcuTV9uny7pQBFbu3XKCn9rNxcNGRaDT9BD1gkBGZ/<0;1>/*))}})
```

Asking the toolkit to export it:

```
$ mnemonic export-wallet --descriptor "$(cat taproot-4leaf.desc)" --format descriptor --network mainnet
tr([73c5da0a/84'/0'/4']xpub6CatWdiZiodmeXswr13Gd5aNtNqr2UHCBEsCoL3eEFVaM7n8kY5kS4daaP83gWQncmzL3Wzt79mEiLix6XZs6XQmGcQNeQ4HcjfVTn9TuXE/<0;1>/*,{{and_v(v:after(1000000),and_v(v:sha256(a84dce40975727c398023cfbd50d5db3b9662375521d0f1ac62dbd829b9a08ad),multi_a(3,[73c5da0a/84'/0'/0']xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V/<0;1>/*,[73c5da0a/84'/0'/1']xpub6CatWdiZiodmYVtWLtEQsAg1H9ooS1bmsJUBwQ83FE1Fyk386FWcyicJgEZv3quZSJKA5dh5Lo2PbubMGxCfZtRthV6ST2qquL9w3HSzcUn/<0;1>/*,[73c5da0a/84'/0'/2']xpub6CatWdiZiodmbNGqcQxxjGN165QxTU4PwNNi9WrijYgYf7VxcmuFxosRw3foczLgRDbjDjJbqZhPCTfkcaWmL9BuSw98ybKKJtcHgWeryy6/<0;1>/*))),and_v(v:after(1893456000),and_v(v:sha256(a84dce40975727c398023cfbd50d5db3b9662375521d0f1ac62dbd829b9a08ad),multi_a(2,[73c5da0a/84'/0'/3']xpub6CatWdiZiodmdHurRokjbycCrxddTDJgTsyEAaQfKjkWbwUi79LAWG5gHjMCQB7BeJc47MkubXuZdf45JZHK1qcr1GZ5EwREUDVDLVdPkEC/<0;1>/*,[b8688df1/84'/0'/0']xpub6DNfJehqF1LUs9kwaqDu12Ajpz9psYVtbGhTykQo1CYdkkqV2vAyR4DiWXSTTDujWHzVy1AtV6ENGKWgwbLWqa4wXMZR4ZmdpRjQBG5EgTV/<0;1>/*,[b8688df1/84'/0'/1']xpub6DNfJehqF1LUsoh1a4XLG3yuFY1oCRJd6Bkba97xvW6SwvY71Nc6LvkqfDLWuyCivJ4eMDxgFtiTxnazgp8rYVzQSKR3L8EGRs9asBarqpe/<0;1>/*)))},{and_v(v:older(65535),multi_a(2,[b8688df1/84'/0'/2']xpub6DNfJehqF1LUwZ1DrFcQN365CDMPLSsrqpSwaz4eRrtYz9qWdsVH9JqsHpQ6yNGdG4VXGKbLgcxKkXtwTB5B4iCuHcmjomqR8z6NNxNpU51/<0;1>/*,[b8688df1/84'/0'/3']xpub6DNfJehqF1LUxb8gRSstbR9LcAxWgwD4V678z5FZ7BLftWzwQCt3yZb5eW4U8AVJwbKcZ2iegjjvKv2xggJpMBRkk3CE7bz7g4uMV7Qp3TU/<0;1>/*)),and_v(v:older(4255898),multi_a(1,[28645006/84'/0'/0']xpub6DBbzvudcQg2nS4tHSkrm7FGkXL6arWxEYoZ1g1GWbaNdWRobcJmxH8KQdjuzTZJtwDveibGd83eGzdGCR6NjSqpU8p2Xx4wWQK7iGBycAm/<0;1>/*,[28645006/84'/0'/1']xpub6DBbzvudcQg2sPDRWpqfEn5VzPiwrd1zNes4aHmmNUog9Jmc2fc2JSfM2E39YZy2iakmpWRpa3rXhzGNmd4GKiJEZxmTCftQpwBGd9ihVks/<0;1>/*,[28645006/84'/0'/2']xpub6DBbzvudcQg2uk9BrxsuxCWjsYsbfPYkPahQfmTVABfJ4j8TRxUnRd5eGgEXfgPJ63xcuTV9uny7pQBFbu3XKCn9rNxcNGRaDT9BD1gkBGZ/<0;1>/*))}})#trqmzhua
note: stdout is watch-only — public keys only, cannot spend
```

So we use a **depth-1** tree (2 leaves) and pack two tiers per leaf with `or_i`:
Leaf A = tier 1 or tier 2 (the absolute-timelock + secret-word tiers); Leaf B =
tier 3 or tier 4 (the relative-timelock tiers). (A rust-miniscript release
> 13.1.0 containing #953 reopens deep trees -- tracked in FOLLOWUP
`taproot-coverage-cycle-on-miniscript-gt-13-1-0`.)

## 6.2 Build + validate

The hand-written depth-1 `tr(...)` descriptor file:

```
$ cat taproot.desc
tr([73c5da0a/84'/0'/4']xpub6CatWdiZiodmeXswr13Gd5aNtNqr2UHCBEsCoL3eEFVaM7n8kY5kS4daaP83gWQncmzL3Wzt79mEiLix6XZs6XQmGcQNeQ4HcjfVTn9TuXE/<0;1>/*,{or_i(and_v(v:after(1000000),and_v(v:sha256(a84dce40975727c398023cfbd50d5db3b9662375521d0f1ac62dbd829b9a08ad),multi_a(3,[73c5da0a/84'/0'/0']xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V/<0;1>/*,[73c5da0a/84'/0'/1']xpub6CatWdiZiodmYVtWLtEQsAg1H9ooS1bmsJUBwQ83FE1Fyk386FWcyicJgEZv3quZSJKA5dh5Lo2PbubMGxCfZtRthV6ST2qquL9w3HSzcUn/<0;1>/*,[73c5da0a/84'/0'/2']xpub6CatWdiZiodmbNGqcQxxjGN165QxTU4PwNNi9WrijYgYf7VxcmuFxosRw3foczLgRDbjDjJbqZhPCTfkcaWmL9BuSw98ybKKJtcHgWeryy6/<0;1>/*))),and_v(v:after(1893456000),and_v(v:sha256(a84dce40975727c398023cfbd50d5db3b9662375521d0f1ac62dbd829b9a08ad),multi_a(2,[73c5da0a/84'/0'/3']xpub6CatWdiZiodmdHurRokjbycCrxddTDJgTsyEAaQfKjkWbwUi79LAWG5gHjMCQB7BeJc47MkubXuZdf45JZHK1qcr1GZ5EwREUDVDLVdPkEC/<0;1>/*,[b8688df1/84'/0'/0']xpub6DNfJehqF1LUs9kwaqDu12Ajpz9psYVtbGhTykQo1CYdkkqV2vAyR4DiWXSTTDujWHzVy1AtV6ENGKWgwbLWqa4wXMZR4ZmdpRjQBG5EgTV/<0;1>/*,[b8688df1/84'/0'/1']xpub6DNfJehqF1LUsoh1a4XLG3yuFY1oCRJd6Bkba97xvW6SwvY71Nc6LvkqfDLWuyCivJ4eMDxgFtiTxnazgp8rYVzQSKR3L8EGRs9asBarqpe/<0;1>/*)))),or_i(and_v(v:older(65535),multi_a(2,[b8688df1/84'/0'/2']xpub6DNfJehqF1LUwZ1DrFcQN365CDMPLSsrqpSwaz4eRrtYz9qWdsVH9JqsHpQ6yNGdG4VXGKbLgcxKkXtwTB5B4iCuHcmjomqR8z6NNxNpU51/<0;1>/*,[b8688df1/84'/0'/3']xpub6DNfJehqF1LUxb8gRSstbR9LcAxWgwD4V678z5FZ7BLftWzwQCt3yZb5eW4U8AVJwbKcZ2iegjjvKv2xggJpMBRkk3CE7bz7g4uMV7Qp3TU/<0;1>/*)),and_v(v:older(4255898),multi_a(1,[28645006/84'/0'/0']xpub6DBbzvudcQg2nS4tHSkrm7FGkXL6arWxEYoZ1g1GWbaNdWRobcJmxH8KQdjuzTZJtwDveibGd83eGzdGCR6NjSqpU8p2Xx4wWQK7iGBycAm/<0;1>/*,[28645006/84'/0'/1']xpub6DBbzvudcQg2sPDRWpqfEn5VzPiwrd1zNes4aHmmNUog9Jmc2fc2JSfM2E39YZy2iakmpWRpa3rXhzGNmd4GKiJEZxmTCftQpwBGd9ihVks/<0;1>/*,[28645006/84'/0'/2']xpub6DBbzvudcQg2uk9BrxsuxCWjsYsbfPYkPahQfmTVABfJ4j8TRxUnRd5eGgEXfgPJ63xcuTV9uny7pQBFbu3XKCn9rNxcNGRaDT9BD1gkBGZ/<0;1>/*)))})
```

Validate and canonicalise it. The full canonical descriptor, every xpub in full:

```
$ mnemonic export-wallet --descriptor "$(cat taproot.desc)" --format descriptor --network mainnet
tr([73c5da0a/84'/0'/4']xpub6CatWdiZiodmeXswr13Gd5aNtNqr2UHCBEsCoL3eEFVaM7n8kY5kS4daaP83gWQncmzL3Wzt79mEiLix6XZs6XQmGcQNeQ4HcjfVTn9TuXE/<0;1>/*,{or_i(and_v(v:after(1000000),and_v(v:sha256(a84dce40975727c398023cfbd50d5db3b9662375521d0f1ac62dbd829b9a08ad),multi_a(3,[73c5da0a/84'/0'/0']xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V/<0;1>/*,[73c5da0a/84'/0'/1']xpub6CatWdiZiodmYVtWLtEQsAg1H9ooS1bmsJUBwQ83FE1Fyk386FWcyicJgEZv3quZSJKA5dh5Lo2PbubMGxCfZtRthV6ST2qquL9w3HSzcUn/<0;1>/*,[73c5da0a/84'/0'/2']xpub6CatWdiZiodmbNGqcQxxjGN165QxTU4PwNNi9WrijYgYf7VxcmuFxosRw3foczLgRDbjDjJbqZhPCTfkcaWmL9BuSw98ybKKJtcHgWeryy6/<0;1>/*))),and_v(v:after(1893456000),and_v(v:sha256(a84dce40975727c398023cfbd50d5db3b9662375521d0f1ac62dbd829b9a08ad),multi_a(2,[73c5da0a/84'/0'/3']xpub6CatWdiZiodmdHurRokjbycCrxddTDJgTsyEAaQfKjkWbwUi79LAWG5gHjMCQB7BeJc47MkubXuZdf45JZHK1qcr1GZ5EwREUDVDLVdPkEC/<0;1>/*,[b8688df1/84'/0'/0']xpub6DNfJehqF1LUs9kwaqDu12Ajpz9psYVtbGhTykQo1CYdkkqV2vAyR4DiWXSTTDujWHzVy1AtV6ENGKWgwbLWqa4wXMZR4ZmdpRjQBG5EgTV/<0;1>/*,[b8688df1/84'/0'/1']xpub6DNfJehqF1LUsoh1a4XLG3yuFY1oCRJd6Bkba97xvW6SwvY71Nc6LvkqfDLWuyCivJ4eMDxgFtiTxnazgp8rYVzQSKR3L8EGRs9asBarqpe/<0;1>/*)))),or_i(and_v(v:older(65535),multi_a(2,[b8688df1/84'/0'/2']xpub6DNfJehqF1LUwZ1DrFcQN365CDMPLSsrqpSwaz4eRrtYz9qWdsVH9JqsHpQ6yNGdG4VXGKbLgcxKkXtwTB5B4iCuHcmjomqR8z6NNxNpU51/<0;1>/*,[b8688df1/84'/0'/3']xpub6DNfJehqF1LUxb8gRSstbR9LcAxWgwD4V678z5FZ7BLftWzwQCt3yZb5eW4U8AVJwbKcZ2iegjjvKv2xggJpMBRkk3CE7bz7g4uMV7Qp3TU/<0;1>/*)),and_v(v:older(4255898),multi_a(1,[28645006/84'/0'/0']xpub6DBbzvudcQg2nS4tHSkrm7FGkXL6arWxEYoZ1g1GWbaNdWRobcJmxH8KQdjuzTZJtwDveibGd83eGzdGCR6NjSqpU8p2Xx4wWQK7iGBycAm/<0;1>/*,[28645006/84'/0'/1']xpub6DBbzvudcQg2sPDRWpqfEn5VzPiwrd1zNes4aHmmNUog9Jmc2fc2JSfM2E39YZy2iakmpWRpa3rXhzGNmd4GKiJEZxmTCftQpwBGd9ihVks/<0;1>/*,[28645006/84'/0'/2']xpub6DBbzvudcQg2uk9BrxsuxCWjsYsbfPYkPahQfmTVABfJ4j8TRxUnRd5eGgEXfgPJ63xcuTV9uny7pQBFbu3XKCn9rNxcNGRaDT9BD1gkBGZ/<0;1>/*)))})#snerswx7
note: stdout is watch-only — public keys only, cannot spend
```

`Kint` (`[73c5da0a/84'/0'/4']`) is the key-path; the two `or_i(...)` blocks are
the two script leaves; `after(...)` are the absolute (height/time) locks and
`older(...)` the relative (blocks/time) locks -- the same four kinds as section 5.

## 6.3 Engrave + first address

Every key is distinct, so it engraves (watch-only). Take the md1 chunks and
restore to read the first address (this round-trip also proves the **real
internal key** at the trunk reconstructs -- the non-NUMS internal-key feature,
shipped in v0.55.3). The
descriptor file (shown again):

```
$ cat taproot.desc
tr([73c5da0a/84'/0'/4']xpub6CatWdiZiodmeXswr13Gd5aNtNqr2UHCBEsCoL3eEFVaM7n8kY5kS4daaP83gWQncmzL3Wzt79mEiLix6XZs6XQmGcQNeQ4HcjfVTn9TuXE/<0;1>/*,{or_i(and_v(v:after(1000000),and_v(v:sha256(a84dce40975727c398023cfbd50d5db3b9662375521d0f1ac62dbd829b9a08ad),multi_a(3,[73c5da0a/84'/0'/0']xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V/<0;1>/*,[73c5da0a/84'/0'/1']xpub6CatWdiZiodmYVtWLtEQsAg1H9ooS1bmsJUBwQ83FE1Fyk386FWcyicJgEZv3quZSJKA5dh5Lo2PbubMGxCfZtRthV6ST2qquL9w3HSzcUn/<0;1>/*,[73c5da0a/84'/0'/2']xpub6CatWdiZiodmbNGqcQxxjGN165QxTU4PwNNi9WrijYgYf7VxcmuFxosRw3foczLgRDbjDjJbqZhPCTfkcaWmL9BuSw98ybKKJtcHgWeryy6/<0;1>/*))),and_v(v:after(1893456000),and_v(v:sha256(a84dce40975727c398023cfbd50d5db3b9662375521d0f1ac62dbd829b9a08ad),multi_a(2,[73c5da0a/84'/0'/3']xpub6CatWdiZiodmdHurRokjbycCrxddTDJgTsyEAaQfKjkWbwUi79LAWG5gHjMCQB7BeJc47MkubXuZdf45JZHK1qcr1GZ5EwREUDVDLVdPkEC/<0;1>/*,[b8688df1/84'/0'/0']xpub6DNfJehqF1LUs9kwaqDu12Ajpz9psYVtbGhTykQo1CYdkkqV2vAyR4DiWXSTTDujWHzVy1AtV6ENGKWgwbLWqa4wXMZR4ZmdpRjQBG5EgTV/<0;1>/*,[b8688df1/84'/0'/1']xpub6DNfJehqF1LUsoh1a4XLG3yuFY1oCRJd6Bkba97xvW6SwvY71Nc6LvkqfDLWuyCivJ4eMDxgFtiTxnazgp8rYVzQSKR3L8EGRs9asBarqpe/<0;1>/*)))),or_i(and_v(v:older(65535),multi_a(2,[b8688df1/84'/0'/2']xpub6DNfJehqF1LUwZ1DrFcQN365CDMPLSsrqpSwaz4eRrtYz9qWdsVH9JqsHpQ6yNGdG4VXGKbLgcxKkXtwTB5B4iCuHcmjomqR8z6NNxNpU51/<0;1>/*,[b8688df1/84'/0'/3']xpub6DNfJehqF1LUxb8gRSstbR9LcAxWgwD4V678z5FZ7BLftWzwQCt3yZb5eW4U8AVJwbKcZ2iegjjvKv2xggJpMBRkk3CE7bz7g4uMV7Qp3TU/<0;1>/*)),and_v(v:older(4255898),multi_a(1,[28645006/84'/0'/0']xpub6DBbzvudcQg2nS4tHSkrm7FGkXL6arWxEYoZ1g1GWbaNdWRobcJmxH8KQdjuzTZJtwDveibGd83eGzdGCR6NjSqpU8p2Xx4wWQK7iGBycAm/<0;1>/*,[28645006/84'/0'/1']xpub6DBbzvudcQg2sPDRWpqfEn5VzPiwrd1zNes4aHmmNUog9Jmc2fc2JSfM2E39YZy2iakmpWRpa3rXhzGNmd4GKiJEZxmTCftQpwBGd9ihVks/<0;1>/*,[28645006/84'/0'/2']xpub6DBbzvudcQg2uk9BrxsuxCWjsYsbfPYkPahQfmTVABfJ4j8TRxUnRd5eGgEXfgPJ63xcuTV9uny7pQBFbu3XKCn9rNxcNGRaDT9BD1gkBGZ/<0;1>/*)))})
```

```
$ mnemonic bundle --descriptor-file taproot.desc --network mainnet --json | jq -r ".md1[]" > taproot.md1
note: stdout is watch-only — public keys only, cannot spend
```

```
$ cat taproot.md1
md1f40gpvq9zem6jzwrh4yypm6jzxw75sj3m6jzt802ggrh4yyvaafp9rh4yykw75ss802ggemcn9wrf4m7m5yt9
md1f40gpvq2jz2sqrqgy2efntvqq7sjqfntk5ymnjqjatj0sucqg70h4gdtkemje3rw4fp6rc6cnnxqhq4hnx7rv
md1f40gpvqsckmmq5mngy26gzzzg6v6mwrda3qzv6a4pxuusyh2unu8xqz8naa2r2akwukvgm4g7w7kdyafr05mf
md1f40gpvq6gws7xkx9k7c9xu6pzkjqgj9vefntsqqplllyqshsnxhqqgrcf5gqzn2cvasqu79cwukltfwjcnckn
md1f40gpvprg9pw0za5z3883w6pgmnchdq53eutks2twrg3hckhp5gmutms6yd7x9cdzxlry5xgklamn3a93x43x
md1f40gpvpg5qx52ry2qrt9pj9qpskufqq44hccjz40ltkn8tdswlxkk6gs50wsfd6vu4jev3egqy8f0nxndxa6x
md1f40gpvp56glt4vs3t2qswfth67hsept4g4hf5939sedkqhey4nnc0chj6t2x82xgqygmpkvg7cuw0m7akf723
md1f40gpvp622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8q7nzlkkvymsvlrcugrdkju72
md1f40gpvz9fkca3cxteqm6kacqcp59u7xtxux3d5d847wsf4xev6setz7wcggw0mnt0jaragegqfesknkmcecqj
md1f40gpvz0ptr0vxf3c9895ghkha0gk58rcw6qzw4cdxzxd3en9mnxhqdpwsx2ec6jxdf33axqmwg8tzlv563mv
md1f40gpvznmjszkn4t8pc6l88xna4nxjquuhc32y6rxwxttx8w3xuf33474jxxpl30507ttsmq84l460gcw0g4u
md1f40gpvzcx7crsyrzx48lzn7wute526tvfexawdfdy4vy7y7s9c84gwagyhhf8ugmwft46ekqd54ka5c8c48mx
md1f40gpvrr0a7t0c6gn7uss8mxrkma6e6mkqp24zvzxnclw0xxzg0gsqtmwmy87px3u9mlm3vgd8s0y6hvrrvud
md1f40gpvrvmn3jaafe7gyny7hjqysfhjfzfaf5hs5jwzh3mhpgjm2ctzuevmwdffap2f5zl9lsqqkg20rz8nj9q
md1f40gpvrkdtfh372tg8y9k3d35qm006q30450k4vmqvakpywv7pkdanhr5g2a605szqzpgggqszv9xr4h0actx
md1f40gpvr6zg55fzuu4z339d06w5s99f4v94tc38peqcv5jysp6pp3g08exa4sxr0889h3ddqqnqhdl2vm2eucf
md1f40gpvyp6ttmcly57ac6vels0u3jj86dd5pe7m355rttw8r3a2s45tszcmrremst4u6j4atqccq4mtqrtm0ny
md1f40gpvyg6uhd0m3m5l9lvu9cde6pcswdrm73ejss4lts9qcszq6fnt9u8jvs93uep30g6dqg0k3hj4xq9v7xn
md1f40gpvynng0aj0c55pz0eykxhftlk456pzlvcqh3za8msktlmcqvgkmuj2dwpkhhlhm290qceu48zwwu535kp
md1f40gpvyckdgr4nrpzxssz4jyqytut3ee64sfnz6ryr348le5ljx4sv0e02cjfml4zmy9g36c6spmukzwf5w5f
md1f40gpv9rfpgzjjzpmz7nhnk6yh5veeegnzkfxcdg3sypfkn0ru6gc370szpx6rudkq2e7xecrr560mc7uy987
md1f40gpv9tzzetsgt6pqua7cty0t4c4w6rtnjmnht889ltvhne3ktx5etuhl2rmxpaqj6ey0lg6d4mp5l9hjrne
md1f40gpv95xj46th52vv3c3kju0767gfu9j0ynrej00uqenhu2pdakxrqtfe5qck40z57z8ylqssej5h2v929gq
md1f40gpv9eyh2tqvnhxzckz5kfhacj7d2lxnax2ywfue5grx4rgemhg0nk5eg0ze8tj0qfnr3syntujt2jgx5pl
md1f40gpvx92k0c250h62ypsarcmrl7tgutcg8uf4m7gfcd8lfv33knprwthl06am5gdw9aqnfqzkhq0nhyzweea
```

```
$ mnemonic restore --network mainnet $(sed "s/^/--md1 /" taproot.md1)
miniscript policy restore (12 cosigners)
CONFIRM: verify each cosigner fingerprint against your records before importing.
  descriptor: tr([73c5da0a/84'/0'/4']xpub661MyMwAqRbcEyUKSqsBgaz1Lob8pCa1rM1SJ8CEzGCYyP9LisxZ2m1goDqj137XvHdY2nNkctqiE1ixaAFqYHf91CFpFpKicVb7TzvrGsE/<0;1>/*,{or_i(and_v(v:after(1000000),and_v(v:sha256(a84dce40975727c398023cfbd50d5db3b9662375521d0f1ac62dbd829b9a08ad),multi_a(3,[73c5da0a/84'/0'/0']xpub661MyMwAqRbcFHMVYpCiBTXd2Caj7vZhNFHJSgE59Aue2yYkXSrz5q9GaQ4rRjJVhHZTsCiHWSzgMS5beaaTHWVmhpGC7SMdqMXHRXZi8as/<0;1>/*,[73c5da0a/84'/0'/1']xpub661MyMwAqRbcGaxoYcLaxHHXZqEgSRQmN2P5ung8MJ8MNE535mLuhq7zjnrMKyA5eX6ehicVbU1FFPU39LGXbY8PmLPLQxVRQmPFa3Q7spa/<0;1>/*,[73c5da0a/84'/0'/2']xpub661MyMwAqRbcGuXAHBK3oquZS1HJiz2fVZ2idNcK4GLGTXJyGZkPK7fviN6euv5GzY18JD3WBG3SoLat23TLAVjhQMxDVMAqymQNhg3RFT8/<0;1>/*))),and_v(v:after(1893456000),and_v(v:sha256(a84dce40975727c398023cfbd50d5db3b9662375521d0f1ac62dbd829b9a08ad),multi_a(2,[73c5da0a/84'/0'/3']xpub661MyMwAqRbcGHLCZcLjg25oG8wSyqSE5XNM9uMks6vrpH4pRDC8UmAynovThuKraidMeEKJ2FcqBw1eF76aeu1vrGtLJXUiJXr4r9N1TZQ/<0;1>/*,[b8688df1/84'/0'/0']xpub661MyMwAqRbcGowNgeNcLS8CgL2vnZybpJqtkbCmSQMdq2qzcDWqq3CXXg7x5BqvcNCSaNUw6nisoN7JFK2j3HfxV57nNm2RLKo2UzHgbs6/<0;1>/*,[b8688df1/84'/0'/1']xpub661MyMwAqRbcEv3U8uuxavsQA8LNNYwcNge8rT7SaMS5S8KiEwxoP72TQ8ARYjczPTtVQz6CxcaBTEE3XchmYvSiHcVbC9h17CmyfG7sVq9/<0;1>/*)))),or_i(and_v(v:older(65535),multi_a(2,[b8688df1/84'/0'/2']xpub661MyMwAqRbcG7Xht9EwgNucA47Rmgg8Bn5bNmFdJMkotHQDXirpogQHkVNRcwAy6KwGnUYMUNBFCNaRq4WnsqWW2VNUDdW6ymHXfVpk4c3/<0;1>/*,[b8688df1/84'/0'/3']xpub661MyMwAqRbcEbqBvNkLuDtudGA2PHAbtWUuHKe3CKjZCaLjxLGSG8SJpwBCnXsj8xPGXaV9ZWL3j9ktbed8y1aeNVK95HrkgHfHGBXM5Eh/<0;1>/*)),and_v(v:older(4255898),multi_a(1,[28645006/84'/0'/0']xpub661MyMwAqRbcEdBofBaGbgnse74WRuyEbXRSmzq8jzthzutDnXTV2yNQPzgs3ubwuNp7yrSHnECoA5xHgnoEDH4HSGWqLtYdi6nWVZCfXPk/<0;1>/*,[28645006/84'/0'/1']xpub661MyMwAqRbcH2WNMbtz4pZ8wDtpxndYo6E4r5o8pXedve17srma1LCEjM8WcpVk67xsc36KpBNtYUdqo5dpcFMzRfzSZSa4C5DRty4eDNF/<0;1>/*,[28645006/84'/0'/2']xpub661MyMwAqRbcGqcAAnB9mhvQsdUx2fKasUoXT2gMpt2tFz94wRfAkhuLhZUJkjQ5pgnd9Ny9EwrgcHbAASVnQShCbfhnGsKAk2k6yGoWXAv/<0;1>/*)))})#7cy3x3q9
  first recv: bc1p9stcwz5597fmkxae9343k8edzkcvdczf9qp65r6p447pg0et82yqst3d2c
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

(Restore re-serialises each key as a depth-0 `xpub661My...` -- a different
descriptor string, identical addresses; that is how the md1 card stores keys.)

## 6.4 Export for wallets (Nunchuk / Core / Sparrow)

`descriptor` and `bitcoin-core` both work for taproot. The descriptor file
(shown again):

```
$ cat taproot.desc
tr([73c5da0a/84'/0'/4']xpub6CatWdiZiodmeXswr13Gd5aNtNqr2UHCBEsCoL3eEFVaM7n8kY5kS4daaP83gWQncmzL3Wzt79mEiLix6XZs6XQmGcQNeQ4HcjfVTn9TuXE/<0;1>/*,{or_i(and_v(v:after(1000000),and_v(v:sha256(a84dce40975727c398023cfbd50d5db3b9662375521d0f1ac62dbd829b9a08ad),multi_a(3,[73c5da0a/84'/0'/0']xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V/<0;1>/*,[73c5da0a/84'/0'/1']xpub6CatWdiZiodmYVtWLtEQsAg1H9ooS1bmsJUBwQ83FE1Fyk386FWcyicJgEZv3quZSJKA5dh5Lo2PbubMGxCfZtRthV6ST2qquL9w3HSzcUn/<0;1>/*,[73c5da0a/84'/0'/2']xpub6CatWdiZiodmbNGqcQxxjGN165QxTU4PwNNi9WrijYgYf7VxcmuFxosRw3foczLgRDbjDjJbqZhPCTfkcaWmL9BuSw98ybKKJtcHgWeryy6/<0;1>/*))),and_v(v:after(1893456000),and_v(v:sha256(a84dce40975727c398023cfbd50d5db3b9662375521d0f1ac62dbd829b9a08ad),multi_a(2,[73c5da0a/84'/0'/3']xpub6CatWdiZiodmdHurRokjbycCrxddTDJgTsyEAaQfKjkWbwUi79LAWG5gHjMCQB7BeJc47MkubXuZdf45JZHK1qcr1GZ5EwREUDVDLVdPkEC/<0;1>/*,[b8688df1/84'/0'/0']xpub6DNfJehqF1LUs9kwaqDu12Ajpz9psYVtbGhTykQo1CYdkkqV2vAyR4DiWXSTTDujWHzVy1AtV6ENGKWgwbLWqa4wXMZR4ZmdpRjQBG5EgTV/<0;1>/*,[b8688df1/84'/0'/1']xpub6DNfJehqF1LUsoh1a4XLG3yuFY1oCRJd6Bkba97xvW6SwvY71Nc6LvkqfDLWuyCivJ4eMDxgFtiTxnazgp8rYVzQSKR3L8EGRs9asBarqpe/<0;1>/*)))),or_i(and_v(v:older(65535),multi_a(2,[b8688df1/84'/0'/2']xpub6DNfJehqF1LUwZ1DrFcQN365CDMPLSsrqpSwaz4eRrtYz9qWdsVH9JqsHpQ6yNGdG4VXGKbLgcxKkXtwTB5B4iCuHcmjomqR8z6NNxNpU51/<0;1>/*,[b8688df1/84'/0'/3']xpub6DNfJehqF1LUxb8gRSstbR9LcAxWgwD4V678z5FZ7BLftWzwQCt3yZb5eW4U8AVJwbKcZ2iegjjvKv2xggJpMBRkk3CE7bz7g4uMV7Qp3TU/<0;1>/*)),and_v(v:older(4255898),multi_a(1,[28645006/84'/0'/0']xpub6DBbzvudcQg2nS4tHSkrm7FGkXL6arWxEYoZ1g1GWbaNdWRobcJmxH8KQdjuzTZJtwDveibGd83eGzdGCR6NjSqpU8p2Xx4wWQK7iGBycAm/<0;1>/*,[28645006/84'/0'/1']xpub6DBbzvudcQg2sPDRWpqfEn5VzPiwrd1zNes4aHmmNUog9Jmc2fc2JSfM2E39YZy2iakmpWRpa3rXhzGNmd4GKiJEZxmTCftQpwBGd9ihVks/<0;1>/*,[28645006/84'/0'/2']xpub6DBbzvudcQg2uk9BrxsuxCWjsYsbfPYkPahQfmTVABfJ4j8TRxUnRd5eGgEXfgPJ63xcuTV9uny7pQBFbu3XKCn9rNxcNGRaDT9BD1gkBGZ/<0;1>/*)))})
```

Bitcoin Core `importdescriptors` payload:

```
$ mnemonic export-wallet --descriptor "$(cat taproot.desc)" --format bitcoin-core --network mainnet
[
  {
    "active": true,
    "desc": "tr([73c5da0a/84'/0'/4']xpub6CatWdiZiodmeXswr13Gd5aNtNqr2UHCBEsCoL3eEFVaM7n8kY5kS4daaP83gWQncmzL3Wzt79mEiLix6XZs6XQmGcQNeQ4HcjfVTn9TuXE/0/*,{or_i(and_v(v:after(1000000),and_v(v:sha256(a84dce40975727c398023cfbd50d5db3b9662375521d0f1ac62dbd829b9a08ad),multi_a(3,[73c5da0a/84'/0'/0']xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V/0/*,[73c5da0a/84'/0'/1']xpub6CatWdiZiodmYVtWLtEQsAg1H9ooS1bmsJUBwQ83FE1Fyk386FWcyicJgEZv3quZSJKA5dh5Lo2PbubMGxCfZtRthV6ST2qquL9w3HSzcUn/0/*,[73c5da0a/84'/0'/2']xpub6CatWdiZiodmbNGqcQxxjGN165QxTU4PwNNi9WrijYgYf7VxcmuFxosRw3foczLgRDbjDjJbqZhPCTfkcaWmL9BuSw98ybKKJtcHgWeryy6/0/*))),and_v(v:after(1893456000),and_v(v:sha256(a84dce40975727c398023cfbd50d5db3b9662375521d0f1ac62dbd829b9a08ad),multi_a(2,[73c5da0a/84'/0'/3']xpub6CatWdiZiodmdHurRokjbycCrxddTDJgTsyEAaQfKjkWbwUi79LAWG5gHjMCQB7BeJc47MkubXuZdf45JZHK1qcr1GZ5EwREUDVDLVdPkEC/0/*,[b8688df1/84'/0'/0']xpub6DNfJehqF1LUs9kwaqDu12Ajpz9psYVtbGhTykQo1CYdkkqV2vAyR4DiWXSTTDujWHzVy1AtV6ENGKWgwbLWqa4wXMZR4ZmdpRjQBG5EgTV/0/*,[b8688df1/84'/0'/1']xpub6DNfJehqF1LUsoh1a4XLG3yuFY1oCRJd6Bkba97xvW6SwvY71Nc6LvkqfDLWuyCivJ4eMDxgFtiTxnazgp8rYVzQSKR3L8EGRs9asBarqpe/0/*)))),or_i(and_v(v:older(65535),multi_a(2,[b8688df1/84'/0'/2']xpub6DNfJehqF1LUwZ1DrFcQN365CDMPLSsrqpSwaz4eRrtYz9qWdsVH9JqsHpQ6yNGdG4VXGKbLgcxKkXtwTB5B4iCuHcmjomqR8z6NNxNpU51/0/*,[b8688df1/84'/0'/3']xpub6DNfJehqF1LUxb8gRSstbR9LcAxWgwD4V678z5FZ7BLftWzwQCt3yZb5eW4U8AVJwbKcZ2iegjjvKv2xggJpMBRkk3CE7bz7g4uMV7Qp3TU/0/*)),and_v(v:older(4255898),multi_a(1,[28645006/84'/0'/0']xpub6DBbzvudcQg2nS4tHSkrm7FGkXL6arWxEYoZ1g1GWbaNdWRobcJmxH8KQdjuzTZJtwDveibGd83eGzdGCR6NjSqpU8p2Xx4wWQK7iGBycAm/0/*,[28645006/84'/0'/1']xpub6DBbzvudcQg2sPDRWpqfEn5VzPiwrd1zNes4aHmmNUog9Jmc2fc2JSfM2E39YZy2iakmpWRpa3rXhzGNmd4GKiJEZxmTCftQpwBGd9ihVks/0/*,[28645006/84'/0'/2']xpub6DBbzvudcQg2uk9BrxsuxCWjsYsbfPYkPahQfmTVABfJ4j8TRxUnRd5eGgEXfgPJ63xcuTV9uny7pQBFbu3XKCn9rNxcNGRaDT9BD1gkBGZ/0/*)))})#q3s6u2uk",
    "internal": false,
    "range": [
      0,
      999
    ],
    "timestamp": 0
  },
  {
    "active": true,
    "desc": "tr([73c5da0a/84'/0'/4']xpub6CatWdiZiodmeXswr13Gd5aNtNqr2UHCBEsCoL3eEFVaM7n8kY5kS4daaP83gWQncmzL3Wzt79mEiLix6XZs6XQmGcQNeQ4HcjfVTn9TuXE/1/*,{or_i(and_v(v:after(1000000),and_v(v:sha256(a84dce40975727c398023cfbd50d5db3b9662375521d0f1ac62dbd829b9a08ad),multi_a(3,[73c5da0a/84'/0'/0']xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V/1/*,[73c5da0a/84'/0'/1']xpub6CatWdiZiodmYVtWLtEQsAg1H9ooS1bmsJUBwQ83FE1Fyk386FWcyicJgEZv3quZSJKA5dh5Lo2PbubMGxCfZtRthV6ST2qquL9w3HSzcUn/1/*,[73c5da0a/84'/0'/2']xpub6CatWdiZiodmbNGqcQxxjGN165QxTU4PwNNi9WrijYgYf7VxcmuFxosRw3foczLgRDbjDjJbqZhPCTfkcaWmL9BuSw98ybKKJtcHgWeryy6/1/*))),and_v(v:after(1893456000),and_v(v:sha256(a84dce40975727c398023cfbd50d5db3b9662375521d0f1ac62dbd829b9a08ad),multi_a(2,[73c5da0a/84'/0'/3']xpub6CatWdiZiodmdHurRokjbycCrxddTDJgTsyEAaQfKjkWbwUi79LAWG5gHjMCQB7BeJc47MkubXuZdf45JZHK1qcr1GZ5EwREUDVDLVdPkEC/1/*,[b8688df1/84'/0'/0']xpub6DNfJehqF1LUs9kwaqDu12Ajpz9psYVtbGhTykQo1CYdkkqV2vAyR4DiWXSTTDujWHzVy1AtV6ENGKWgwbLWqa4wXMZR4ZmdpRjQBG5EgTV/1/*,[b8688df1/84'/0'/1']xpub6DNfJehqF1LUsoh1a4XLG3yuFY1oCRJd6Bkba97xvW6SwvY71Nc6LvkqfDLWuyCivJ4eMDxgFtiTxnazgp8rYVzQSKR3L8EGRs9asBarqpe/1/*)))),or_i(and_v(v:older(65535),multi_a(2,[b8688df1/84'/0'/2']xpub6DNfJehqF1LUwZ1DrFcQN365CDMPLSsrqpSwaz4eRrtYz9qWdsVH9JqsHpQ6yNGdG4VXGKbLgcxKkXtwTB5B4iCuHcmjomqR8z6NNxNpU51/1/*,[b8688df1/84'/0'/3']xpub6DNfJehqF1LUxb8gRSstbR9LcAxWgwD4V678z5FZ7BLftWzwQCt3yZb5eW4U8AVJwbKcZ2iegjjvKv2xggJpMBRkk3CE7bz7g4uMV7Qp3TU/1/*)),and_v(v:older(4255898),multi_a(1,[28645006/84'/0'/0']xpub6DBbzvudcQg2nS4tHSkrm7FGkXL6arWxEYoZ1g1GWbaNdWRobcJmxH8KQdjuzTZJtwDveibGd83eGzdGCR6NjSqpU8p2Xx4wWQK7iGBycAm/1/*,[28645006/84'/0'/1']xpub6DBbzvudcQg2sPDRWpqfEn5VzPiwrd1zNes4aHmmNUog9Jmc2fc2JSfM2E39YZy2iakmpWRpa3rXhzGNmd4GKiJEZxmTCftQpwBGd9ihVks/1/*,[28645006/84'/0'/2']xpub6DBbzvudcQg2uk9BrxsuxCWjsYsbfPYkPahQfmTVABfJ4j8TRxUnRd5eGgEXfgPJ63xcuTV9uny7pQBFbu3XKCn9rNxcNGRaDT9BD1gkBGZ/1/*)))})#zvdvstsr",
    "internal": true,
    "range": [
      0,
      999
    ],
    "timestamp": 0
  }
]
note: stdout is watch-only — public keys only, cannot spend
```

But **BSMS / BIP-129 is not available for taproot** (BIP-129 predates BIP-386) --
the toolkit says so and points you elsewhere:

```
$ mnemonic export-wallet --descriptor "$(cat taproot.desc)" --format bsms --network mainnet
error: --format bsms does not support taproot (P2trMulti); BIP-129 §1 prerequisites do not yet include BIP-386. Real emit support is tracked at FOLLOWUP `bsms-taproot-emit` and depends on a BIP-129 spec update. Use --format bitcoin-core (Core-importable) or --format sparrow (Sparrow JSON, taproot-capable) for taproot watch-only setup.
```

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

```
$ python3 -c "import hashlib; X=hashlib.sha256(b'please').digest(); print('preimage X =', X.hex()); print('hash Hp    =', hashlib.new('ripemd160', X).hexdigest())"
preimage X = 56ccc4dcfc96534b06fc0c08a301be24f13b491484d5d984953cc0dba9bbb89a
hash Hp    = 06d05e2f02fb90ddf98d8cd95d806ba12b27aff4
```

`mnemonic compare-cost` reports per-condition witness vbytes (key-agnostic --
abstract labels A,B,... are auto-dummy-keyed). **Folded** -- the new tier joins
Leaf B, so spending it reveals all of tiers 3+4+5:

```
$ mnemonic compare-cost --miniscript "or_i(and_v(v:older(65535),multi(2,A,B)),or_i(and_v(v:older(4255898),multi(1,C,D,E)),and_v(v:pk(F),ripemd160(06d05e2f02fb90ddf98d8cd95d806ba12b27aff4))))"
Input: or_i(and_v(v:older(65535),multi(2,A,B)),or_i(and_v(v:older(4255898),multi(1,C,D,E)),and_v(v:pk(F),ripemd160(06d05e2f02fb90ddf98d8cd95d806ba12b27aff4))))
Wrapper comparison: wsh(M)  vs  tr(NUMS, {M})
Feerate: 1.0 sat/vB

Condition             | wsh vB | tr vB |  Δ vB | wsh sats | tr sats | Δ sats
----------------------+--------+-------+-------+----------+---------+-------
A + B + older(blocks) |     79 |   147 |   +68 |       79 |     147 |    +68
C + older(512s)       |     61 |   132 |   +71 |       61 |     132 |    +71
D + older(512s)       |     61 |   132 |   +71 |       61 |     132 |    +71
E + older(512s)       |     61 |   132 |   +71 |       61 |     132 |    +71
F + preimage(h0)      |     69 |   139 |   +70 |       69 |     139 |    +70

note: per-condition vbytes are rounded individually; absolute numbers may differ by ±1 from real-tx accounting, Δ values are correct
note: input contains hash-preimage fragments; preimage-known rows are enumerated assuming the user can supply each preimage (cost only — no preimage knowledge is implied)
```

**Dedicated** -- the new tier is its own leaf, revealing only itself:

```
$ mnemonic compare-cost --miniscript "and_v(v:pk(F),ripemd160(06d05e2f02fb90ddf98d8cd95d806ba12b27aff4))"
Input: and_v(v:pk(F),ripemd160(06d05e2f02fb90ddf98d8cd95d806ba12b27aff4))
Wrapper comparison: wsh(M)  vs  tr(NUMS, {M})
Feerate: 1.0 sat/vB

Condition        | wsh vB | tr vB |  Δ vB | wsh sats | tr sats | Δ sats
-----------------+--------+-------+-------+----------+---------+-------
F + preimage(h0) |     68 |    90 |   +22 |       68 |      90 |    +22

note: per-condition vbytes are rounded individually; absolute numbers may differ by ±1 from real-tx accounting, Δ values are correct
note: input contains hash-preimage fragments; preimage-known rows are enumerated assuming the user can supply each preimage (cost only — no preimage knowledge is implied)
```

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

```
$ cat taproot-multi.desc
tr(50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0,sortedmulti_a(2,[73c5da0a/87'/0'/0']xpub6DBjiYnc4ewKti13Q1L35bqdodw5z3VGJnf516B3icHrEGEUcCuCG5GVQDZtH8Xmsyt3Fs9YDNwLaqjUbbRidwXZ6sxufZcr4VqqzrXvicM/<0;1>/*,[b8688df1/87'/0'/0']xpub6CbhrPzY2z7NcCGCGjLAJLq8iRyjUfwmdXQs66MxTVUReKqb9DpLnVJ5D1qpatZjUuPGTyxf5TYU1vA34YFE9FHB4TvfYmokYLVsyEFZFt9/<0;1>/*,[28645006/87'/0'/0']xpub6DB7HNqw6CZojxN85NuFTPWZhi2FagSnexPS1rv3nYQhngkmdHgb7iebYvTFmFKKDA3ozf5yezDsCH6cXAw3WZijviSZtZC2hjHn2uazz4z/<0;1>/*))
```

Validate and canonicalise it (the NUMS hex is the BIP-341 unspendable H-point):

```
$ mnemonic export-wallet --descriptor "$(cat taproot-multi.desc)" --format descriptor --network mainnet
tr(50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0,sortedmulti_a(2,[73c5da0a/87'/0'/0']xpub6DBjiYnc4ewKti13Q1L35bqdodw5z3VGJnf516B3icHrEGEUcCuCG5GVQDZtH8Xmsyt3Fs9YDNwLaqjUbbRidwXZ6sxufZcr4VqqzrXvicM/<0;1>/*,[b8688df1/87'/0'/0']xpub6CbhrPzY2z7NcCGCGjLAJLq8iRyjUfwmdXQs66MxTVUReKqb9DpLnVJ5D1qpatZjUuPGTyxf5TYU1vA34YFE9FHB4TvfYmokYLVsyEFZFt9/<0;1>/*,[28645006/87'/0'/0']xpub6DB7HNqw6CZojxN85NuFTPWZhi2FagSnexPS1rv3nYQhngkmdHgb7iebYvTFmFKKDA3ozf5yezDsCH6cXAw3WZijviSZtZC2hjHn2uazz4z/<0;1>/*))#8nz0lwja
note: stdout is watch-only — public keys only, cannot spend
```

Engrave the watch-only card set and read the first address back from the md1
chunks alone:

```
$ mnemonic bundle --descriptor-file taproot-multi.desc --network mainnet --json | jq -r ".md1[]" > taproot-multi.md1
note: stdout is watch-only — public keys only, cannot spend
```

```
$ mnemonic restore --network mainnet $(sed "s/^/--md1 /" taproot-multi.md1)
2-of-3 multisig restore
CONFIRM: verify each cosigner fingerprint against your records before importing.
  descriptor: tr(50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0,sortedmulti_a(2,[73c5da0a/87'/0'/0']xpub661MyMwAqRbcFrooZ2966EcDmVX5MoFXZhuJqXTudvJzwBTBfPQSc5JzX52fvS18oqSdEJXJ4kTGRJ76wPWDUSNJsY5JsgVBQoD6KrbdCLL/<0;1>/*,[b8688df1/87'/0'/0']xpub661MyMwAqRbcEnFgxHRLx7i1fnjcBPgc71qy8mVkbGXYukNGMK2XFRbAaCLYEJDUufNoBxTNa68i5MYhqmrEkfhjzgHCUEcvJBhXS5bk4RW/<0;1>/*,[28645006/87'/0'/0']xpub661MyMwAqRbcEdy4jr5EtEhQBctfscE6a99DGLr2cW4HnnmBsXDoe3odGzRiw3hcRM5wfKcQmb7s5FjdGrR6SrExXmeopaoY9Lk7tQusDjN/<0;1>/*))#k0lsap8u
  first recv: bc1p550zvnachy40z6hh8llka93mkm0c3635samp264ck6rfd0dcdc8s00n8c8
  cosigner @0: 73c5da0a [87'/0'/0']  from md1 (not independently verified)
  cosigner @1: b8688df1 [87'/0'/0']  from md1 (not independently verified)
  cosigner @2: 28645006 [87'/0'/0']  from md1 (not independently verified)
UNVERIFIED: no --from/--cosigner cross-check supplied; verify each cosigner fingerprint above against your records before importing
note: stdout is watch-only — public keys only, cannot spend
```

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

\newpage

# Appendix A -- the public test seeds used

| Slot | BIP-39 test phrase | Fingerprint (BIP-87) |
|---|---|---|
| @0 | `abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about` | `73c5da0a` |
| @1 | `legal winner thank year wave sausage worth useful legal winner thank yellow` | `b8688df1` |
| @2 | `letter advice cage absurd amount doctor acoustic avoid letter advice cage above` | `28645006` |

These are world-known BIP-39 vectors with no funds. Sections 5-6 derive their
keys from these same three seeds at distinct `m/84'/0'/N'` accounts. Generated
with `mnemonic` v0.97.0 on Linux. See the in-repo manual (`docs/manual/`) for
the authoritative per-flag reference.

\newpage

# Appendix B -- depth->=2 taproot reconstruction

This appendix used to be titled EXPERIMENTAL and told you to build a
proof-of-concept binary from a never-merged branch. **It is a supported feature
as of v0.97.0** and everything below is live-captured from the shipped
`mnemonic`, like the rest of this guide.

What changed: a depth->=2 taptree always ENGRAVED faithfully -- only reading it
back was gated, because the pinned rust-miniscript mis-formatted nested
taptrees on Display, printing `{{a,b},c}` as `{{a,b,c}}`, which its own parser
then rejected. Upstream PR #953 fixes it. Waiting for a crates.io release
carrying that fix was never a plan with a date: 13.1.0 was cut from a
maintenance line and is *newer* than the merge, so it does not contain it. The
toolkit pins the merge commit directly.

One caveat survives, and it is about YOUR build rather than this one: recovery
needs a `mnemonic` >= 0.97.0. An older binary still cannot reconstruct a
depth->=2 taptree, and a backup outlives the software that made it.

Recall the depth-2 four-leaf descriptor from section 6.1 (one tier per leaf):

```
$ cat taproot-4leaf.desc
tr([73c5da0a/84'/0'/4']xpub6CatWdiZiodmeXswr13Gd5aNtNqr2UHCBEsCoL3eEFVaM7n8kY5kS4daaP83gWQncmzL3Wzt79mEiLix6XZs6XQmGcQNeQ4HcjfVTn9TuXE/<0;1>/*,{{and_v(v:after(1000000),and_v(v:sha256(a84dce40975727c398023cfbd50d5db3b9662375521d0f1ac62dbd829b9a08ad),multi_a(3,[73c5da0a/84'/0'/0']xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V/<0;1>/*,[73c5da0a/84'/0'/1']xpub6CatWdiZiodmYVtWLtEQsAg1H9ooS1bmsJUBwQ83FE1Fyk386FWcyicJgEZv3quZSJKA5dh5Lo2PbubMGxCfZtRthV6ST2qquL9w3HSzcUn/<0;1>/*,[73c5da0a/84'/0'/2']xpub6CatWdiZiodmbNGqcQxxjGN165QxTU4PwNNi9WrijYgYf7VxcmuFxosRw3foczLgRDbjDjJbqZhPCTfkcaWmL9BuSw98ybKKJtcHgWeryy6/<0;1>/*))),and_v(v:after(1893456000),and_v(v:sha256(a84dce40975727c398023cfbd50d5db3b9662375521d0f1ac62dbd829b9a08ad),multi_a(2,[73c5da0a/84'/0'/3']xpub6CatWdiZiodmdHurRokjbycCrxddTDJgTsyEAaQfKjkWbwUi79LAWG5gHjMCQB7BeJc47MkubXuZdf45JZHK1qcr1GZ5EwREUDVDLVdPkEC/<0;1>/*,[b8688df1/84'/0'/0']xpub6DNfJehqF1LUs9kwaqDu12Ajpz9psYVtbGhTykQo1CYdkkqV2vAyR4DiWXSTTDujWHzVy1AtV6ENGKWgwbLWqa4wXMZR4ZmdpRjQBG5EgTV/<0;1>/*,[b8688df1/84'/0'/1']xpub6DNfJehqF1LUsoh1a4XLG3yuFY1oCRJd6Bkba97xvW6SwvY71Nc6LvkqfDLWuyCivJ4eMDxgFtiTxnazgp8rYVzQSKR3L8EGRs9asBarqpe/<0;1>/*)))},{and_v(v:older(65535),multi_a(2,[b8688df1/84'/0'/2']xpub6DNfJehqF1LUwZ1DrFcQN365CDMPLSsrqpSwaz4eRrtYz9qWdsVH9JqsHpQ6yNGdG4VXGKbLgcxKkXtwTB5B4iCuHcmjomqR8z6NNxNpU51/<0;1>/*,[b8688df1/84'/0'/3']xpub6DNfJehqF1LUxb8gRSstbR9LcAxWgwD4V678z5FZ7BLftWzwQCt3yZb5eW4U8AVJwbKcZ2iegjjvKv2xggJpMBRkk3CE7bz7g4uMV7Qp3TU/<0;1>/*)),and_v(v:older(4255898),multi_a(1,[28645006/84'/0'/0']xpub6DBbzvudcQg2nS4tHSkrm7FGkXL6arWxEYoZ1g1GWbaNdWRobcJmxH8KQdjuzTZJtwDveibGd83eGzdGCR6NjSqpU8p2Xx4wWQK7iGBycAm/<0;1>/*,[28645006/84'/0'/1']xpub6DBbzvudcQg2sPDRWpqfEn5VzPiwrd1zNes4aHmmNUog9Jmc2fc2JSfM2E39YZy2iakmpWRpa3rXhzGNmd4GKiJEZxmTCftQpwBGd9ihVks/<0;1>/*,[28645006/84'/0'/2']xpub6DBbzvudcQg2uk9BrxsuxCWjsYsbfPYkPahQfmTVABfJ4j8TRxUnRd5eGgEXfgPJ63xcuTV9uny7pQBFbu3XKCn9rNxcNGRaDT9BD1gkBGZ/<0;1>/*))}})
```

The engraved card is a faithful backup even on master -- `bundle` accepts it;
only *reading it back* is gated. Make the card and pull its md1 chunks with the
shipped binary:

```
$ mnemonic bundle --descriptor-file taproot-4leaf.desc --network mainnet --json | jq -r ".md1[]" > depth2.md1
note: stdout is watch-only — public keys only, cannot spend
```

```
$ cat depth2.md1
md1f8eucvq9zem6jzwrh4yypm6jzxw75sj3m6jzt802ggrh4yyvaafp9rh4yykw75ss802ggemcsk65ef96dmuhe
md1f8eucvq2jz2sqrqgy29fntvqq7sjqfntk5ymnjqjatj0sucqg70h4gdtkemje3rw4fp6rc6cqzhan8ac8kjyw
md1f8eucvqsckmmq5mngy26gzzzg6v6mwrda3qzv6a4pxuusyh2unu8xqz8naa2r2akwukvgm4gaa2pacrd4qv84
md1f8eucvq6gws7xkx9k7c9xu6pzkjqgj9v9fntsqqplllyqshsnxhqqgrcf5gqzn2cvasqu79capl4rdda7hr0z
md1f8eucvprg9pw0za5z3883w6pgmnchdq53eutks2twrg3hckhp5gmutms6yd7x9cdzxlry5xg4vfvrdrp8fdd6
md1f8eucvpg5qx52ry2qrt9pj9qpskufqq44hccjz40ltkn8tdswlxkk6gs50wsfd6vu4jev3egrhn7l0chmf9x6
md1f8eucvp56glt4vs3t2qswfth67hsept4g4hf5939sedkqhey4nnc0chj6t2x82xgqygmpkvgatgel8qeqxxkd
md1f8eucvp622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8q7nzlkkvyms0vh0v5afqayzk
md1f8eucvz9fkca3cxteqm6kacqcp59u7xtxux3d5d847wsf4xev6setz7wcggw0mnt0jaragegr6d8x0glwkquw
md1f8eucvz0ptr0vxf3c9895ghkha0gk58rcw6qzw4cdxzxd3en9mnxhqdpwsx2ec6jxdf33axqcausm7pgz4f8s
md1f8eucvznmjszkn4t8pc6l88xna4nxjquuhc32y6rxwxttx8w3xuf33474jxxpl30507ttsmqyxtz2nkucqsfq
md1f8eucvzcx7crsyrzx48lzn7wute526tvfexawdfdy4vy7y7s9c84gwagyhhf8ugmwft46ekqw8ppdgxrw6l86
md1f8eucvrr0a7t0c6gn7uss8mxrkma6e6mkqp24zvzxnclw0xxzg0gsqtmwmy87px3u9mlm3vgw5yc5xfg4v5q3
md1f8eucvrvmn3jaafe7gyny7hjqysfhjfzfaf5hs5jwzh3mhpgjm2ctzuevmwdffap2f5zl9lsrnzl6nax3u2eu
md1f8eucvrkdtfh372tg8y9k3d35qm006q30450k4vmqvakpywv7pkdanhr5g2a605szqzpgggqn3cjkltnejqh6
md1f8eucvr6zg55fzuu4z339d06w5s99f4v94tc38peqcv5jysp6pp3g08exa4sxr0889h3ddqqsnr60kjlukyy4
md1f8eucvyp6ttmcly57ac6vels0u3jj86dd5pe7m355rttw8r3a2s45tszcmrremst4u6j4atqmt5zth78a5h0c
md1f8eucvyg6uhd0m3m5l9lvu9cde6pcswdrm73ejss4lts9qcszq6fnt9u8jvs93uep30g6dqgv99qzfcynrx60
md1f8eucvynng0aj0c55pz0eykxhftlk456pzlvcqh3za8msktlmcqvgkmuj2dwpkhhlhm290qc60psjjscz7v2a
md1f8eucvyckdgr4nrpzxssz4jyqytut3ee64sfnz6ryr348le5ljx4sv0e02cjfml4zmy9g36cer4vv2u2lmkg4
md1f8eucv9rfpgzjjzpmz7nhnk6yh5veeegnzkfxcdg3sypfkn0ru6gc370szpx6rudkq2e7xecqsqdl8x62tamz
md1f8eucv9tzzetsgt6pqua7cty0t4c4w6rtnjmnht889ltvhne3ktx5etuhl2rmxpaqj6ey0lge7pv3gpppam09
md1f8eucv95xj46th52vv3c3kju0767gfu9j0ynrej00uqenhu2pdakxrqtfe5qck40z57z8ylqnrd9yt5gn9a5u
md1f8eucv9eyh2tqvnhxzckz5kfhacj7d2lxnax2ywfue5grx4rgemhg0nk5eg0ze8tj0qfnr3s8qltzh5k7fvar
md1f8eucvx92k0c250h62ypsarcmrl7tgutcg8uf4m7gfcd8lfv33knprwthl06am5gdw9aqnfqp9rhl0fq5pp9p
```

And the shipped `mnemonic` reconstructs it -- depth-2 taptree and all:

```
$ mnemonic restore --network mainnet $(sed "s/^/--md1 /" depth2.md1)
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
-- four leaves, one tier each. The capture above is LIVE and CI-gated now; it was
a static paste from an experimental build until v0.97.0, which is why this
appendix once carried a warning that it was not reproducible.
