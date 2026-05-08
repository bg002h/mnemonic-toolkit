# Producing your first bundle

This chapter produces a complete 3-card bundle for a single-sig
BIP-84 mainnet wallet. End-to-end takes about three minutes.

## The command

> **Reminder.** The phrase below is the public BIP-39 test vector —
> never engrave or fund a wallet built from it. See [Generating
> entropy safely](22-generate-entropy.md) for the full warning.

```sh
mnemonic bundle \
  --network mainnet \
  --template bip84 \
  --slot @0.phrase="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
```

Three flags carry the work:

- **`--network mainnet`** — the wallet is for Bitcoin mainnet.
  Other values: `testnet`, `signet`, `regtest`.
- **`--template bip84`** — the wallet's spending rule. `bip84` is
  single-sig native SegWit (P2WPKH) at derivation `m/84'/0'/0'`.
  Other single-sig templates: `bip44` (legacy), `bip49`
  (nested-segwit), `bip86` (taproot).
- **`--slot @0.phrase=…`** — the seed-bearing input for cosigner
  index `@0`. Single-sig has only one cosigner, so only `@0`. The
  multisig walkthrough in Part III uses `@0`, `@1`, `@2`.

For a real bundle, replace the canonical phrase with the one you
generated in [Generating entropy safely](22-generate-entropy.md).

## Output

Three card strings, each shown twice — once contiguous (handy for
copy-paste) and once chunked in 5-character groups (handy for
engraving):

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

Each card section opens with a header comment, then the contiguous
form, then the chunked form. The trailing block beginning
`# === Wallet bundle:` is a metadata summary, not part of the
engraving.

| Card | Contiguous form | What it is |
|---|---|---|
| **ms1** | `ms10entrsqq…34v7f` (1 string) | Goes on the *secret* card. Carries the BIP-39 entropy. |
| **mk1** | `mk1qprsqhp…854wq4` and `mk1qprsqhp…ma0nh` (2 strings) | Goes on the *key* card. The xpub + origin is too long for one BCH-checksummed group, so it splits across two strings. |
| **md1** | `md1zsxdspq…` (3 strings) | Goes on the *descriptor* card. Encodes the wallet policy and binds it to the xpub. |

The metadata block at the end shows the version stamps (`1c017`),
master fingerprint, origin path, template, and md1 version — useful
for visually comparing two bundles, never engraved.

The trailing `warning: secret material on stdout` is real. For a
real bundle, redirect to a file (`> bundle.txt`) and either delete
the file after engraving or pipe through `age -e` to encrypt at
rest. The secret here is your seed phrase and the ms1 card derived
from it.

## What about the engraving cards themselves?

The toolkit also emits an *engraving-card* layout to stderr — a
visual aid showing where each chunk goes on the physical plate.
The cards covered in Part II ([Stamping the steel plates](25-stamp.md))
walk through using the layout when transferring strings to steel.
Pass `--no-engraving-card` to suppress the stderr emission if you
don't want it.

Onward: verify the bundle round-trips before engraving.
