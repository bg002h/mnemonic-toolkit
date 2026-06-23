# Your first bundle

This chapter produces a complete 3-card bundle for a single-sig
BIP-84 mainnet wallet. End-to-end takes about three minutes.

:::danger
Every command in this manual uses the canonical BIP-39 test vector
`abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about`.
This phrase is **public** — chain watchers have swept any wallet
ever derived from it. **Never engrave or fund a wallet built from
the canonical test seed.** Generate fresh entropy for real wallets
(see [Test seeds and example data](#appendix-f-test-seeds-and-example-data)).
:::

## Prerequisites

You should have run [Path A or Path B](#installing-the-toolkit)
of the previous chapter and verified `mnemonic --version` reports
`0.8.0` or later.

## The command

```sh
mnemonic bundle \
  --network mainnet \
  --template bip84 \
  --slot @0.phrase="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
```

Three flags carry the work:

- **`--network mainnet`** — the wallet is for Bitcoin mainnet. Other
  values: `testnet`, `signet`, `regtest`.
- **`--template bip84`** — single-sig BIP-84 (native SegWit, P2WPKH,
  derivation `m/84'/0'/0'`). For multisig and taproot variants, see
  the [Workflows part](#multi-source-2-of-3-multisig).
- **`--slot @0.phrase=…`** — cosigner index `@0` (only one for single-
  sig), `phrase` subkey (the BIP-39 mnemonic). For multisig wallets,
  you'd add `--slot @1.phrase=…`, `--slot @2.phrase=…`, etc.

## Output

By default each of the three card strings is printed **once**, broken
into space-separated groups of five characters — the engraving-friendly
form, also easy to read aloud during verification. Pass `--group-size 0`
for the contiguous single-line form (handy for copy-paste into a wallet),
or `--separator hyphen` to group with dashes instead. Intake (`restore`,
`verify-bundle`, `convert`, `repair`) accepts any of these forms — the
display separators are non-load-bearing.

```{.text include="22-first-bundle.out" lines="2-21"}
PLACEHOLDER — generated from transcripts/22-first-bundle.out lines 2-21 at build
```

## Reading the output

Three card sections, each with a header comment and its chunked card
string(s), followed by a trailing `# === Wallet bundle:` block
summarising the bundle's metadata.

For this single-sig wallet:

| Card | Canonical (contiguous) form | Use |
|---|---|---|
| **ms1** | `ms10entrsqq…34v7f` | Engrave on the *secret* card. Recovers the seed. |
| **mk1** | `mk1qprsqhp…854wq4` (line 1) and `mk1qprsqhp…ma0nh` (line 2) | Engrave on the *key* card. Two strings because the xpub is too long to fit one BCH-checksummed group at the toolkit's chunking density. |
| **md1** | `md1fgdxlpq…` (three lines) | Engrave on the *descriptor* card. Three strings encode the wallet policy and bind it to the xpub. |

The *Canonical (contiguous)* column shows each card with grouping
stripped (the `--group-size 0` form) — what intake normalises to and
what you'd copy-paste into another wallet.

The last block is **not** part of the engraving — it's a bundle
*summary* showing the version stamps (`1c017`), the master fingerprint,
the origin path, the template, and the md1 version. Use it to
visually compare two bundles.

The trailing `warning: stdout carries private key material` is real and worth
heeding for production: redirect the output to a file or pipe to
`age -e` for encryption-at-rest before saving.

## What about the engraving cards themselves?

The toolkit also emits an *engraving-card* layout to stderr by
default; pass `--no-engraving-card` to suppress it. The card is plain
text — suitable for verification before committing to physical
engraving. See
[Single-sig steel-engraved backup workflow](#single-sig-steel-engraved-backup)
for the full ceremony, including which strings go on which physical
plate, plus visual checksum verification.

## Verifying the bundle

Run [the verify-bundle walkthrough](#verifying-a-bundle) next — it
re-derives the expected card content from the seed and confirms each
card decodes and matches.
