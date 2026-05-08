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

Three card strings, each shown twice — once contiguous (handy for
copy-paste into a wallet) and once chunked (handy for engraving):

```text
# ms1 (entropy, BCH-checksummed)
ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f

ms10e ntrsq qqqqq qqqqq qqqqq qqqqq qqqqq qqcj9 sxraq 34v7f

# mk1 (xpub + origin)
mk1qprsqhpqqsq3cqtsleeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8qp3yxg3dpe854wq4
mk1qprsqhpp0f30mtxzd65mvwcur9usdatwuqvq6z70r9nwrgk6xn6l8gy6nwa2n977sw6zh34rma0nh

mk1qp rsqhp qqsq3 cqtsl eeutk s2qvz g3vs7 0mejh k622w s2kgd
emj2c d8zwj 2skzx 2wq0q w70l4 q99vd yh5x0 z8v4y slsp8 qp3yx
g3dpe 854wq 4
mk1qp rsqhp p0f30 mtxzd 65mvw cur9u sdatw uqvq6 z70r9 nwrgk
6xn6l 8gy6n wa2n9 77sw6 zh34r ma0nh

# md1 (wallet policy)
md1zsxdspqqqpm6jzzqqvqz6qu79mg9p2sgfff6p2eph8wftp5uf6gqnlgzqqqnymv0
md1zsxdspq259s3jnsrcrhnlagpftrf9apnc3m9fy8uqfc85cha4nqnh5k67ey2hzyc
md1zsxdspqjd65mvwcur9usdatwuqvq6z70r9nwrgk6xn6l8gy6nvqhuuyvzgaejah6

md1zs-xdspq-qqpm6-jzzqq-vqz6q-u79mg-9p2sg-fff6p-2eph8-wftp5-uf6gq-nlgzq-qqnym-v0
md1zs-xdspq-259s3-jnsrc-rhnla-gpftr-f9apn-c3m9f-y8uqf-c85ch-a4nqn-h5k67-ey2hz-yc
md1zs-xdspq-jd65m-vwcur-9usda-twuqv-q6z70-r9nwr-gk6xn-6l8gy-6nvqh-uuyvz-gaeja-h6

# === Wallet bundle: bip84, mainnet ===
# ms1: 1c017
# mk1: 1c017
# fingerprint: 73c5da0a
# origin path: 84'/0'/0'
# Template: bip84
# md1: 1c01
warning: secret material on stdout — consider redirecting (e.g., '> file.txt' or '| age -e ...')
```

## Reading the output

Three card sections, each with a header comment, contiguous string,
chunked string, and a trailing `# === Wallet bundle:` block summarising
the bundle's metadata.

For this single-sig wallet:

| Card | Contiguous form | Use |
|---|---|---|
| **ms1** | `ms10entrsqq…34v7f` | Engrave on the *secret* card. Recovers the seed. |
| **mk1** | `mk1qprsqhp…854wq4` (line 1) and `mk1qprsqhp…ma0nh` (line 2) | Engrave on the *key* card. Two strings because the xpub is too long to fit one BCH-checksummed group at the toolkit's chunking density. |
| **md1** | `md1zsxdspq…` (three lines) | Engrave on the *descriptor* card. Three strings encode the wallet policy and bind it to the xpub. |

The last block is **not** part of the engraving — it's a bundle
*summary* showing the version stamps (`1c017`), the master fingerprint,
the origin path, the template, and the md1 version. Use it to
visually compare two bundles.

The trailing `warning: secret material on stdout` is real and worth
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
