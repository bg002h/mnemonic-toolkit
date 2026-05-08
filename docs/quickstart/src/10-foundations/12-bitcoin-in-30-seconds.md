# Bitcoin in 30 seconds

Three concepts power every modern Bitcoin wallet: a *seed phrase*,
an *extended public key*, and a *wallet descriptor*. This chapter
introduces each in three or four sentences. The m-format star
encodes all three.

## Seed phrase

A seed phrase is a sequence of 12 to 24 English words that encodes
a random secret. The standard for this encoding is **BIP-39**:
roughly, you pick 128 or 256 random bits, compute a short checksum
of them, append it, and slice the combined bitstream into 11-bit
chunks that index into a fixed 2048-word list. Almost every Bitcoin wallet today understands
BIP-39 phrases. The seed phrase is the only thing you actually
need to keep secret — every key in your wallet derives from it.

## Extended public key (xpub)

From the seed, your wallet computes a *master key*, then derives a
tree of child keys via **BIP-32**. Each child has both a private
form (used to sign transactions) and a public form (used to receive
to addresses). An **xpub** is a node in that tree exported in its
public-only form: you can hand an xpub to wallet software, and it
can derive *every receive address* the wallet will ever use,
without ever seeing the secret. xpubs are how watch-only wallets
work; they are also what cosigners share with each other in
multisig setups.

## Wallet descriptor

A descriptor is a one-line text recipe describing a wallet's
spending rule completely. Example shape (the parts will be
explained later):

```text
wpkh([fingerprint/84h/0h/0h]xpub6Cat.../<0;1>/*)
```

That string says: "this is a single-key, native-segwit wallet
(`wpkh`); the key is the xpub shown; both receive addresses
(chain 0) and change addresses (chain 1) are valid." Multisig
descriptors look the same but with multiple keys and a threshold,
e.g. `wsh(sortedmulti(2, xpub_A, xpub_B, xpub_C))`. **BIP-388**
extends descriptors into a *wallet policy* format that hardware
wallets and the m-format `md1` card use: it splits the
*template* (the recipe shape) from the *bound keys* (which xpub
goes into which slot).

## BIP, what's a BIP?

A **BIP** — *Bitcoin Improvement Proposal* — is a numbered
specification document. Bitcoin's standards process publishes
each protocol or wallet convention as a BIP at
<https://github.com/bitcoin/bips>. When this guide says "BIP-39"
or "BIP-388" it means a specific document there. The m-format
star uses several BIPs as building blocks: BIP-39 for entropy,
BIP-32 for key derivation, BIP-388 for wallet policies, and
BIP-93 ("codex32") for the error-correcting alphabet engraved on
each card.

Onward: how those three concepts map onto the three m-format
cards.
