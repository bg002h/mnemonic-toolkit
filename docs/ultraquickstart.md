% m-format Constellation UltraQuickStart

# DISCLAIMER — UNTESTED ALPHA SOFTWARE {.unnumbered}

**This software has not yet been independently tested, audited, or
proven in production. Do not use the m-format constellation to back
up significant sums of money at this time. Doing so is tantamount to
asking to be rekt.** Use only with disposable amounts (a few sats),
on testnet, or for evaluation. The codecs, CLIs, BCH error-correcting
math, BIP-388 wallet-policy emission, and cross-card binding
invariants have all been authored and reviewed only by the original
developer; assume bugs until external review happens.

\newpage

# m-format Constellation UltraQuickStart {.unnumbered}

A two-page 3-card steel-backup recipe for a single-sig Bitcoin wallet. For
the gentle ~42-page newcomer walkthrough see
[`m-format-quickstart.pdf`](m-format-quickstart.pdf); for the comprehensive
121-page reference see [`m-format-manual.pdf`](m-format-manual.pdf).

## Install

```sh
sh -c "$(curl -fsSL https://raw.githubusercontent.com/bg002h/mnemonic-toolkit/master/scripts/install.sh)" -- --no-gui
mnemonic --version
```

The installer fetches the pinned, checksummed releases of `mnemonic`, `md`,
`ms` and `mk` (the recover step below uses `md`). To build from source
instead, see the manual's install chapter: a bare `cargo install --git`
without `--tag` builds an unreleased branch.

## Bundle

Generate fresh entropy on an air-gapped machine for a real wallet. This
recipe uses the **public BIP-39 test phrase** so the output is reproducible
— never engrave or fund a wallet derived from it.

```sh
printf '%s' 'abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about' \
  | mnemonic bundle \
      --network mainnet \
      --template bip84 \
      --slot @0.phrase=-
```

`--slot @0.phrase=-` reads the phrase from stdin. `mnemonic` refuses a seed
phrase, an `ms1` card or a passphrase typed on the command line (exit 2),
because other processes can read a command line.

Output: three card strings (`ms1…`, `mk1…`, `md1…`) plus a chunked-form
engraving guide. Each card has its own BCH error-correcting checksum.

## Verify (before stamping)

```sh
export MS1='<ms1-string>'
printf '%s' 'abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about' \
  | mnemonic verify-bundle \
  --network mainnet --template bip84 \
  --slot @0.phrase=- \
  --ms1 @env:MS1 \
  --mk1 <mk1-line-1> --mk1 <mk1-line-2> \
  --md1 <md1-line-1> --md1 <md1-line-2> --md1 <md1-line-3>
```

The phrase comes on stdin and the `ms1` card from the `MS1` environment
variable (`@env:MS1`). `result: ok` → safe to stamp. A `*_decode: error at position N` line names
the single character whose checksum disagrees — re-stamp character `N` of
the failing card.

## Stamp + separate

Stamp all three card strings on rated steel, re-decode the engraved set with
`verify-bundle` against the steel-read strings, then geographically separate
the three plates:

- **Red** (`ms1`) — primary safe.
- **Blue** (`mk1`) — bank safe-deposit box.
- **Green** (`md1`) — off-site (trusted person, attorney, vault).

The split-recovery property: an attacker who recovers only one plate cannot
derive the wallet alone.

## Recover

Read the cards off steel and reverse:

```sh
printf '%s' '<ms1-string>' | mnemonic convert --from ms1=- --to phrase
mnemonic convert --from "mk1=<mk1-line-1> <mk1-line-2>" --to xpub --to fingerprint --to path
md decode <md1-line-1> <md1-line-2> <md1-line-3>
```

`mnemonic convert` recovers the seed phrase from `ms1` (read from stdin:
`--from ms1=-`), the xpub + origin from `mk1` (both strings in one quoted
value). `md decode` takes the three `md1` strings positionally and
returns the wallet policy.

If you have installed `mk-cli` separately, `mk decode <mk1-line-1> <mk1-line-2>` is
the minimal-surface alternative for the public-card recovery step.

For multisig (`--template wsh-sortedmulti --threshold 2 --slot @0.phrase=@env:SEED_0 --slot @1.phrase=@env:SEED_1 --slot @2.phrase=@env:SEED_2`)
or watch-only (`--slot @0.xpub=…`), see the QuickStart's Parts III + IV.
