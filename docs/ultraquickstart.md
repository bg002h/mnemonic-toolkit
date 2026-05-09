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
cargo install --locked --git https://github.com/bg002h/mnemonic-toolkit
mnemonic --version
```

## Bundle

Generate fresh entropy on an air-gapped machine for a real wallet. This
recipe uses the **public BIP-39 test phrase** so the output is reproducible
— never engrave or fund a wallet derived from it.

```sh
mnemonic bundle \
  --network mainnet \
  --template bip84 \
  --slot @0.phrase="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
```

Output: three card strings (`ms1…`, `mk1…`, `md1…`) plus a chunked-form
engraving guide. Each card has its own BCH error-correcting checksum.

## Verify (before stamping)

```sh
mnemonic verify-bundle \
  --network mainnet --template bip84 \
  --slot @0.phrase="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about" \
  --ms1 <ms1-string> \
  --mk1 <mk1-line-1> --mk1 <mk1-line-2> \
  --md1 <md1-line-1> --md1 <md1-line-2> --md1 <md1-line-3>
```

`result: ok` → safe to stamp. A `*_decode: error at position N` line names
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
mnemonic convert --from ms1=<ms1-string> --to phrase
mnemonic convert --from mk1=<mk1-string> --to xpub --to fingerprint --to path
md decode <md1-line-1> <md1-line-2> <md1-line-3>
```

`mnemonic convert` recovers the seed phrase from `ms1`, the xpub + origin
from `mk1`. `md decode` takes the three `md1` strings positionally and
returns the wallet policy.

If you have installed `mk-cli` separately, `mk decode <mk1-string>` is
the minimal-surface alternative for the public-card recovery step.

For multisig (`--template wsh-sortedmulti --threshold 2 --slot @0.phrase=… --slot @1.phrase=… --slot @2.phrase=…`)
or watch-only (`--slot @0.xpub=…`), see the QuickStart's Parts III + IV.
