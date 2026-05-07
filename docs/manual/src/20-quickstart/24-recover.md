# Minimal recovery walkthrough

Recovery is the inverse of [your first bundle](#your-first-bundle):
given engraved cards, produce a wallet usable for signing. This
chapter walks the simplest case — single-sig, all three cards in
hand. The
[recovery-paths workflow](#recovery-paths-by-damaged-card-scenario)
covers damaged or partial bundles.

## Goal

Recover the seed phrase and a watch-only descriptor from the cards
in your hand:

- ms1: `ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f`
- mk1: two strings (xpub + origin)
- md1: three strings (wallet policy + bound xpub)

The seed phrase signs spends. The descriptor (md1, decoded) tells a
watch-only wallet — Bitcoin Core, Sparrow, Specter — how to construct
addresses.

## Step 1 — recover the seed phrase from ms1

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

The phrase is the canonical BIP-39 12-word mnemonic. Your wallet of
choice imports BIP-39 phrases directly.

:::primer
**Why ms1 instead of just engraving the phrase?** A 12-word phrase
on steel is fragile — a single mis-stamped letter can turn a valid
word into a different valid word, silently corrupting the seed.
The ms1 encoding wraps the entropy in a BCH error-correction
checksum so a small number of bit errors in the engraving are
*detected and located*. The phrase you'd write from memory is
recoverable; ms1 is recoverable even with a few stamping mistakes.
:::

## Step 2 — confirm the public-key side via mk1

If you only need to *watch* the wallet (no signing), the seed isn't
required. Decode the mk1 card to recover the xpub and origin:

```sh
md decode \
  --mk1 mk1qprsqhpqqsq3cqtsleeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8qp3yxg3dpe854wq4 \
  --mk1 mk1qprsqhpp0f30mtxzd65mvwcur9usdatwuqvq6z70r9nwrgk6xn6l8gy6nwa2n977sw6zh34rma0nh
```

(The `md` CLI handles mk1 decoding too; future `mk` CLI is on the
v0.2 roadmap.)

## Step 3 — re-derive the descriptor from md1

The md1 card carries the wallet policy. Decoding it tells your
watch-only wallet what kind of multisig (or single-sig) script to
expect:

```sh
md decode \
  --md1 md1zsxdspqqqpm6jzzqqvqz6qu79mg9p2sgfff6p2eph8wftp5uf6gqnlgzqqqnymv0 \
  --md1 md1zsxdspq259s3jnsrcrhnlagpftrf9apnc3m9fy8uqfc85cha4nqnh5k67ey2hzyc \
  --md1 md1zsxdspqjd65mvwcur9usdatwuqvq6z70r9nwrgk6xn6l8gy6nvqhuuyvzgaejah6
```

For this BIP-84 single-sig bundle, the decoded descriptor is the
standard `wpkh(xpub.../84'/0'/0'/0/*)` shape.

## Step 4 — verify before trusting

Before importing the seed into a hot wallet, run
[the verify-bundle walkthrough](#verifying-a-bundle) once more —
this time using the values you just decoded — to confirm the cards
agree with each other across the cross-binding (`policy_id_stub`).

If `result: ok`, the bundle is intact. Import the seed into your
signing wallet of choice; import the descriptor into your watch-only
wallet of choice. The
[wallet-export workflow](#exporting-to-bitcoin-core-bip-388-sparrow-specter)
covers Bitcoin Core / BIP-388 / Sparrow / Specter export shapes in
detail.

---

That's the minimal recovery — three cards in, one phrase and one
descriptor out. From here:

- For ceremony details (which physical plate carries which string,
  visual-checksum verification before importing), read
  [Single-sig steel-engraved backup workflow](#single-sig-steel-engraved-backup).
- For damaged-card scenarios (lost ms1, partially-stamped mk1,
  illegible md1), read
  [Recovery paths by damaged-card scenario](#recovery-paths-by-damaged-card-scenario).
