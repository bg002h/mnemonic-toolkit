# Watch-only single-sig

A *watch-only* wallet sees the chain — incoming payments, balance,
address derivation — without holding the seed. The m-format star
supports this via a **2-card bundle**: mk1 + md1, no ms1. Without
the secret card the wallet cannot sign, but Bitcoin Core, Sparrow,
or Specter can derive addresses and watch the chain.

Use this when you want to monitor a wallet from an internet-connected
host while the seed stays on an air-gapped device — for incoming-payment
tracking, balance reporting, or coordinator workflows that present
PSBTs to remote signers.

## Two-step shape

The seed never crosses to the bundle-building host. Step 1 derives
the xpub on the seed-holding machine; step 2 builds the bundle from
the xpub alone on a separate (potentially internet-connected) host.

> **Reminder.** The phrase below is the public BIP-39 test vector
> used throughout this Quick Start — never use it for a real wallet.
> See [Generating entropy safely](../20-singlesig/22-generate-entropy.md).

### Step 1 — derive the xpub (on the air-gapped seed-holder)

```sh
mnemonic convert \
  --from phrase="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about" \
  --to xpub \
  --template bip84 \
  --network mainnet
```

Output:

```text
xpub: xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V
```

Hand-carry that xpub string to the bundle-building host (it is
public information — paper, USB stick, QR code, or just retyping all
work).

### Step 2 — build the 2-card bundle (on any host)

```sh
mnemonic bundle \
  --network mainnet \
  --template bip84 \
  --slot @0.xpub=xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V
```

Because the slot input is `xpub` rather than `phrase`, no entropy
exists for the toolkit to encode — the output is **2 cards**: one
mk1 (xpub + origin) and one md1 (wallet policy). No ms1, no secret
material on stdout, no `warning: secret material` line.

The same `--template`, `--network`, and `--account` choices from
[chapter 23](../20-singlesig/23-bundle.md) apply — the only
difference is the slot input shape (`@0.xpub=…` instead of
`@0.phrase=…`).

## What you get

| Card | Strings | Holds |
|---|---|---|
| **mk1** | 2 | xpub + origin (master fingerprint + path) |
| **md1** | 3 | wallet policy `wpkh(@0/<0;1>/*)` bound to the xpub |

Stamp the two plates using the same discipline from
[chapter 25](../20-singlesig/25-stamp.md). Recovery from mk1 + md1
alone (`mnemonic convert --from mk1=… --to xpub --to fingerprint --to path`
plus `md decode <md1 strings>`) yields everything a watch-only
client needs.

## Importing into a wallet

Once you have the watch-only bundle (or recovered xpub + policy
from the plates), turn it into the artifact your monitoring software
wants — Bitcoin Core `importdescriptors` JSON, BIP-388 wallet
policy, or a wallet-specific shape — using `mnemonic export-wallet`.
The reference manual's
[Exporting to Bitcoin Core / BIP-388 / Sparrow / Specter](../../../manual/src/30-workflows/37-wallet-export.md)
chapter walks through the four output formats and their import
flows.

Onward: the air-gapped multisig variant, where each cosigner
derives their own xpub locally and only public material crosses to
the coordinator.
