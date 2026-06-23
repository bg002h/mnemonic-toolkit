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
warning: stdout carries private key material (can spend) — redirect or encrypt (e.g. '> file.txt' or '| age -e ...')
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
required. Decode the mk1 card via `mnemonic convert` to recover the
xpub, master fingerprint, and origin path:

```sh
mnemonic convert \
  --from mk1="mk1qprsqhpqqsq3cqtsleeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8qp3yxg3dpe854wq4 mk1qprsqhpp0f30mtxzd65mvwcur9usdatwuqvq6z70r9nwrgk6xn6l8gy6nwa2n977sw6zh34rma0nh" \
  --to xpub --to fingerprint --to path
```

Output:

```{.text include="24-recover-mk1.out" lines="2-4"}
PLACEHOLDER — generated from transcripts/24-recover-mk1.out lines 2-4 at build
```

The two mk1 strings are passed as a single space-separated value;
the toolkit re-assembles them and verifies the BCH checksum on each.
Alternatively, `mk decode <mk1-string-1> <mk1-string-2>` recovers
the same fields directly from the standalone `mk-cli` binary
without bundling the entire toolkit (useful on minimal-surface
recovery machines).

## Step 3 — re-derive the descriptor from md1

The md1 card carries the wallet policy. Decoding it tells your
watch-only wallet what kind of multisig (or single-sig) script to
expect. Pass the strings positionally to `md decode`:

```sh
md decode \
  md1fgdxlpqpqpm6jzzqqvqpdqw0za5zs4gyy55aq4vsmnhy4s6wyaypu34c7raqu8np \
  md1fgdxlpqf2zcgefcpupmel75q5435j7seugaj5jr7qyur6vt76es5cdeyrq7zdy0d \
  md1fgdxlpq3xa2dk8vwpj7gx74hwqxqdp083jehp5tdrfa0n5zdfkqcdlrvnh5r62jn
```

Output:

```{.text include="24-recover-md1.out" lines="1-1"}
PLACEHOLDER — generated from transcripts/24-recover-md1.out line 1 at build
```

For this BIP-84 single-sig bundle, the decoded wallet-policy template
is `wpkh(@0/<0;1>/*)` — single key, native segwit, with `<0;1>` for
external-and-change descriptor pairs (BIP-388 multipath).

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
