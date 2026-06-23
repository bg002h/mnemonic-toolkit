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

:::primer
The `@N` notation is the toolkit's way of indexing a slot in a
wallet template. For single-sig there is only `@0`. For 2-of-3
multisig there are `@0`, `@1`, `@2` — one per cosigner. Inputs
attach to slots via `--slot @N.<subkey>=<value>` where `<subkey>`
can be `phrase` (BIP-39 seed phrase), `xpub` (watch-only xpub),
`entropy` (raw bytes), `wif` (single-key import), and a few more.
The full subkey grammar lives in `mnemonic bundle --help`.
:::

For a real bundle, replace the canonical phrase with the one you
generated in [Generating entropy safely](22-generate-entropy.md).

## Output

Each card's strings are printed in 5-character groups (handy for
engraving). For a real seed, pipe the phrase in via
`--slot @0.phrase=-` rather than on the command line — the toolkit
also prints a leading `warning:` about argv exposure, omitted below
for brevity:

```text
# ms1 (entropy, BCH-checksummed)
ms10e ntrsq qqqqq qqqqq qqqqq qqqqq qqqqq qqcj9 sxraq 34v7f

# mk1 (xpub + origin)
mk1qp rsqhp qqsq3 cqtsl eeutk s2qvz g3vs7 0mejh k622w s2kgd emj2c d8zwj 2skzx 2wq0q w70l4 q99vd yh5x0 z8v4y slsp8 qp3yx g3dpe 854wq 4
mk1qp rsqhp p0f30 mtxzd 65mvw cur9u sdatw uqvq6 z70r9 nwrgk 6xn6l 8gy6n wa2n9 77sw6 zh34r ma0nh

# md1 (wallet policy)
md1fg dxlpq pqpm6 jzzqq vqpdq w0za5 zs4gy y55aq 4vsmn hy4s6 wyayp u34c7 raqu8 np
md1fg dxlpq f2zcg efcpu pmel7 5q543 5j7se ugaj5 jr7qy ur6vt 76es5 cdeyr q7zdy 0d
md1fg dxlpq 3xa2d k8vwp j7gx7 4hwqx qdp08 3jehp 5tdrf a0n5z dfkqc dlrvn h5r62 jn

# === Wallet bundle: bip84, mainnet ===
# ms1: 1c017
# mk1: 1c017
# fingerprint: 73c5da0a
# origin path: m/84'/0'/0'
# Template: bip84
# md1: 1c01
warning: stdout carries private key material (can spend) — redirect or encrypt (e.g. '> file.txt' or '| age -e ...')
```

## Reading the output

Each card section opens with a header comment, then its string(s)
chunked in 5-character groups. The trailing block beginning
`# === Wallet bundle:` is a metadata summary, not part of the
engraving.

| Card | Contiguous form | What it is |
|---|---|---|
| **ms1** | `ms10entrsqq…34v7f` (1 string) | Goes on the *secret* card. Carries the BIP-39 entropy. |
| **mk1** | `mk1qprsqhp…854wq4` and `mk1qprsqhp…ma0nh` (2 strings) | Goes on the *key* card. The xpub + origin is too long for one BCH-checksummed group, so it splits across two strings. |
| **md1** | `md1fgdxlpq…` (3 strings) | Goes on the *descriptor* card. Encodes the wallet policy and binds it to the xpub. |

The metadata block at the end shows the version stamps (`1c017`),
master fingerprint, origin path, template, and md1 version — useful
for visually comparing two bundles, never engraved.

The trailing `warning: stdout carries private key material` is real. For a
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
