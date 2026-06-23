# Recovering from the plates

Recovery is the inverse of [Producing your first bundle](23-bundle.md):
read the cards off the engraved plates and reconstruct what you
need to spend or watch the wallet. Three commands, one per card.

## What you have, what you want

In hand: three engraved plates with the card strings from
[Producing your first bundle](23-bundle.md).
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
warning: stdout carries private key material (can spend) — redirect or encrypt (e.g. '> file.txt' or '| age -e ...')
```

The phrase imports directly into any BIP-39 wallet. The `warning`
on stderr is a reminder that the phrase is on your terminal and
should not stay there — redirect to a file or pipe through `age -e`
if you need to persist it.

## Step 2 — recover xpub, fingerprint, and path from mk1

The two mk1 strings are passed positionally to `mk decode`:

```sh
mk decode \
  mk1qprsqhpqqsq3cqtsleeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8qp3yxg3dpe854wq4 \
  mk1qprsqhpp0f30mtxzd65mvwcur9usdatwuqvq6z70r9nwrgk6xn6l8gy6nwa2n977sw6zh34rma0nh
```

Output:

```text
xpub:                xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V
origin_fingerprint:  73c5da0a
origin_path:         m/84'/0'/0'
policy_id_stubs:     deadbeef
chunks:              2 (regular)
```

`mk decode` reassembles the two strings, verifies the BCH checksum
on each, and emits the decoded card fields. `mk-cli` is the
minimal-surface tool for mk1 recovery: it has no secret-material
dependencies, so it is the cleanest binary to install on an
air-gapped recovery machine.

> If you have already installed `mnemonic-toolkit`, the equivalent
> `mnemonic convert --from mk1="<string-1> <string-2>" --to xpub --to fingerprint --to path`
> works too — pick whichever matches your recovery host's tooling.

## Step 3 — recover the wallet policy from md1

The three md1 strings are passed *positionally* to `md decode`:

```sh
md decode \
  md1fgdxlpqpqpm6jzzqqvqpdqw0za5zs4gyy55aq4vsmnhy4s6wyaypu34c7raqu8np \
  md1fgdxlpqf2zcgefcpupmel75q5435j7seugaj5jr7qyur6vt76es5cdeyrq7zdy0d \
  md1fgdxlpq3xa2dk8vwpj7gx74hwqxqdp083jehp5tdrfa0n5zdfkqcdlrvnh5r62jn
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
phrase). Re-running [`mnemonic verify-bundle`](24-verify.md)
with the recovered values is a useful final sanity check — it
exercises the same `policy_id_stub` cross-binding as the original
verification and confirms the three plates still belong together.

If verification passes, import the phrase into your signing wallet
and the xpub-plus-policy into your watch-only wallet of choice.
[Part IV — watch-only](../40-watch-only/41-singlesig-watch-only.md)
covers the import shapes for Bitcoin Core, Sparrow, and Specter.

For multisig (a 2-of-3 wallet built from three independent
cosigners), continue with [Part III — multisig](../30-multisig/31-why-multisig.md).
