# Recovering from the plates

Recovery is the inverse of [Producing your first bundle](#producing-your-first-bundle):
read the cards off the engraved plates and reconstruct what you
need to spend or watch the wallet. Three commands, one per card.

## What you have, what you want

In hand: three engraved plates with the card strings from
[Producing your first bundle](#producing-your-first-bundle).
Wanted:

- the seed phrase (to sign spends),
- the xpub plus origin (to construct addresses),
- the wallet policy (to know *what kind* of wallet to recover into).

## Step 1 — recover the phrase from ms1

> **Reminder.** This Quick Start uses the public BIP-39 test phrase
> throughout — including in the output below. See [Generating entropy
> safely](22-generate-entropy.md) for why you must never use it for a
> real wallet.

```sh
mnemonic convert \
  --from ms1=ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f \
  --to phrase
```

Output:

```text
phrase: abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about
warning: secret material on stdout — consider redirecting (e.g., '> file.txt' or '| age -e ...')
```

The phrase imports directly into any BIP-39 wallet. The `warning`
on stderr is a reminder that the phrase is on your terminal and
should not stay there — redirect to a file or pipe through `age -e`
if you need to persist it.

## Step 2 — recover xpub, fingerprint, and path from mk1

The two mk1 strings are passed as a single space-separated value:

```sh
mnemonic convert \
  --from mk1="mk1qprsqhpqqsq3cqtsleeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8qp3yxg3dpe854wq4 mk1qprsqhpp0f30mtxzd65mvwcur9usdatwuqvq6z70r9nwrgk6xn6l8gy6nwa2n977sw6zh34rma0nh" \
  --to xpub --to fingerprint --to path
```

Output:

```text
xpub: xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V
fingerprint: 73c5da0a
path: 84'/0'/0'
```

The toolkit re-assembles the two strings, verifies the BCH checksum
on each, and emits the three derived values. The `--to` flag
repeats once per output you want — drop `--to fingerprint` and
`--to path` if you only need the xpub.

## Step 3 — recover the wallet policy from md1

The three md1 strings are passed *positionally* to `md decode`:

```sh
md decode \
  md1zsxdspqqqpm6jzzqqvqz6qu79mg9p2sgfff6p2eph8wftp5uf6gqnlgzqqqnymv0 \
  md1zsxdspq259s3jnsrcrhnlagpftrf9apnc3m9fy8uqfc85cha4nqnh5k67ey2hzyc \
  md1zsxdspqjd65mvwcur9usdatwuqvq6z70r9nwrgk6xn6l8gy6nvqhuuyvzgaejah6
```

Output:

```text
wpkh(@0/<0;1>/*)
```

`wpkh(@0/<0;1>/*)` is the BIP-388 wallet policy template for a
BIP-84 single-sig wallet: one key (`@0`), native segwit (`wpkh`),
with `<0;1>` covering the external-and-change descriptor pair.

## Putting it together

You now have everything a watch-only wallet needs (xpub, path,
fingerprint, template) and everything a signer needs (the
phrase). Re-running [`mnemonic verify-bundle`](#verifying-the-bundle)
with the recovered values is a useful final sanity check — it
exercises the same `policy_id_stub` cross-binding as the original
verification and confirms the three plates still belong together.

If verification passes, import the phrase into your signing wallet
and the xpub-plus-policy into your watch-only wallet of choice.
[Part IV — watch-only](#part-iv-watch-only) covers the import
shapes for Bitcoin Core, Sparrow, and Specter.

For multisig (a 2-of-3 wallet built from three independent
cosigners), continue with [Part III — multisig](#part-iii-multisig).
