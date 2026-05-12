# Exporting to Bitcoin Core / BIP-388 / vendor formats

The bundle's md1 card carries the wallet policy (template + bound
xpubs). To turn that into a watch-only wallet your monitoring
software can import, run `mnemonic export-wallet`. The subcommand
emits the same wallet in any of eight interchange formats (v0.8.1
expanded the original four with `coldcard`, `jade`, `sparrow`,
`specter`, `electrum`, `green`); pick the one your software wants.

## Format selection

```mermaid
flowchart LR
  A[Wallet policy<br/>md1 + mk1 set] --> B{Receiving<br/>software?}
  B -- Bitcoin Core 25+ --> C["--format bitcoin-core<br/>(importdescriptors JSON)"]
  B -- Generic BIP-388 --> D["--format bip388<br/>(wallet_policy JSON)"]
  B -- Coldcard --> E["--format coldcard<br/>(generic JSON or multisig text)"]
  B -- Blockstream Jade --> F["--format jade<br/>(multisig text)"]
  B -- Sparrow --> G["--format sparrow<br/>(wallet JSON)"]
  B -- Specter Desktop --> H["--format specter<br/>(import JSON)"]
  B -- Electrum --> I["--format electrum<br/>(wallet-db JSON)"]
  B -- Blockstream Green --> J["--format green<br/>(descriptor text)"]
```

## Bitcoin Core (default format)

Bitcoin Core's `importdescriptors` RPC accepts a JSON array; the
toolkit's default output matches:

```sh
mnemonic export-wallet \
  --template bip84 \
  --slot @0.xpub=<xpub> \
  --network mainnet \
  --range 0,999 \
  --timestamp now
```

Output is a JSON array with the descriptor, its `range`, the
`timestamp`, and the `internal` flag for receive vs. change. Pipe
into `bitcoin-cli`:

```sh
bitcoin-cli importdescriptors "$(mnemonic export-wallet --template bip84 --slot @0.xpub=<xpub>)"
```

For a multisig wallet, repeat `--slot @N.xpub` and add `--threshold K`.

The `--bitcoin-core-version` flag controls compatibility:
`--bitcoin-core-version 24` emits the older format; `25` (the
default) is current.

## BIP-388 wallet policy

Bitcoin Core 24+, Sparrow, Specter, Coldcard, and most modern
wallets accept BIP-388 wallet policies:

```sh
mnemonic export-wallet \
  --template wsh-sortedmulti \
  --threshold 2 \
  --slot @0.xpub=<xpub-0> \
  --slot @1.xpub=<xpub-1> \
  --slot @2.xpub=<xpub-2> \
  --format bip388
```

Output is the canonical BIP-388 JSON shape:

```json
{
  "name": "wsh-sortedmulti-2-of-3",
  "description": "",
  "description_template": "wsh(sortedmulti(2,@0/**,@1/**,@2/**))",
  "keys_info": [
    "[fp0/87h/0h/0h]xpub...",
    "[fp1/87h/0h/0h]xpub...",
    "[fp2/87h/0h/0h]xpub..."
  ]
}
```

(The default `--multisig-path-family bip87` produces `m/87'/0'/0'`
paths. For Coldcard / SeedSigner / older-Sparrow compatibility, add
`--multisig-path-family bip48` and the paths become `m/48'/0'/0'/2'`.)

## Sparrow + Specter (currently via BIP-388)

`--format sparrow` and `--format specter` are accepted by the
binary but currently return a deferral stub:

```text
error: --format <sparrow> is deferred to a future release; use
--format bitcoin-core or --format bip388 instead.
```

For now, export as BIP-388 and import via the receiving wallet's
BIP-388-aware path:

```sh
mnemonic export-wallet \
  --template wsh-sortedmulti \
  --threshold 2 \
  --slot @0.xpub=<xpub-0> \
  --slot @1.xpub=<xpub-1> \
  --slot @2.xpub=<xpub-2> \
  --format bip388 \
  --output wallet-policy.json
```

Sparrow consumes wallet-policy JSON via *File → Import → Wallet
Policy*. Specter accepts BIP-388 via the *Add Wallet → Import → Multisig*
flow. Native Sparrow / Specter shapes will land if a future toolkit
release lights up the format stubs.

## From a user-supplied descriptor

If you have a descriptor string that doesn't match a built-in
template:

```sh
mnemonic export-wallet \
  --descriptor 'tr(NUMS,sortedmulti_a(2,@0,@1,@2))' \
  --slot @0.xpub=<xpub-0> \
  --slot @1.xpub=<xpub-1> \
  --slot @2.xpub=<xpub-2> \
  --format bip388
```

The toolkit accepts any BIP-388-conformant descriptor and binds the
slotted xpubs into it.

## Taproot multisig export

Taproot multisig requires the `--taproot-internal-key` flag (mirrors
`bundle`):

```sh
mnemonic export-wallet \
  --template tr-sortedmulti-a \
  --threshold 2 \
  --taproot-internal-key nums \
  --slot @0.xpub=<xpub-0> \
  --slot @1.xpub=<xpub-1> \
  --slot @2.xpub=<xpub-2> \
  --format bip388
```

For the cosigner-as-internal-key variant, use `--taproot-internal-key @N`.
See [Taproot multisig](#taproot-multisig) for the design choice.

## Tips

- **Range.** The `--range 0,999` default covers the first 1000
  addresses. Increase if you've used more (e.g., a heavily-used
  exchange wallet). Bitcoin Core re-scans the chain for the
  imported range; large ranges cost time, not safety.
- **Timestamp.** `--timestamp now` skips re-scan (assumes the wallet
  has no historical transactions before "now"). Use a unix-seconds
  value to re-scan from a specific epoch — e.g. `--timestamp 1700000000`
  for late 2023.
- **Output redirect.** Use `--output file.json` (or `> file.json`)
  to keep the JSON out of your shell history and ready for
  piped import.

## Coldcard multisig text (worked example)

Coldcard's multisig wallet-import format is a small text file with
exactly five line-types: `Name:`, `Policy:`, `Derivation:`,
`Format:`, and one `<XFP>: xpub...` line per cosigner. The same
shape is accepted byte-for-byte by Blockstream Jade
(`--format jade` delegates to Coldcard's emitter).

The 2-of-3 wsh-sortedmulti export below uses three Trezor-style test
xpubs at the BIP-48 wsh path `m/48'/0'/0'/2'`:

```sh
mnemonic export-wallet \
  --format coldcard \
  --template wsh-sortedmulti \
  --threshold 2 \
  --multisig-path-family bip48 \
  --network mainnet \
  --wallet-name "VaultColdStorage" \
  --slot @0.xpub=xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX \
  --slot @0.fingerprint=b8688df1 \
  --slot @0.path=m/48\'/0\'/0\'/2\' \
  --slot @1.xpub=xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6 \
  --slot @1.fingerprint=28645006 \
  --slot @1.path=m/48\'/0\'/0\'/2\' \
  --slot @2.xpub=xpub6Buxw9MmbkJr4iAw8SACNci2hQNuPCMwt9P7HkK62ZQAW9UcJaQ2bc6ARD892TToQQ9Rp6AHujHxBLXqAsvn5fRnLfnhKSRfz8qtaoyKUYx \
  --slot @2.fingerprint=5436d724 \
  --slot @2.path=m/48\'/0\'/0\'/2\' \
  --output coldcard-multisig.txt
```

Output (`coldcard-multisig.txt`):

```text
Name: VaultColdStorage
Policy: 2 of 3
Derivation: m/48'/0'/0'/2'
Format: P2WSH
5436D724: xpub6Buxw9MmbkJr4iAw8SACNci2hQNuPCMwt9P7HkK62ZQAW9UcJaQ2bc6ARD892TToQQ9Rp6AHujHxBLXqAsvn5fRnLfnhKSRfz8qtaoyKUYx
28645006: xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6
B8688DF1: xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX
```

Vendor-specific details encoded here:

- **Cosigner order.** For `wsh-sortedmulti` (and `sh-wsh-sortedmulti`)
  the cosigner lines are sorted lexicographically by xpub, matching
  Bitcoin Core's `sortedmulti` consensus rule. For the unsorted
  variants (`wsh-multi` / `sh-wsh-multi`) cosigners appear in
  slot-index order (`@0`, `@1`, `@2`).
- **XFP case.** Coldcard expects uppercase 8-hex master fingerprints
  on cosigner lines. The slot input `@N.fingerprint=` accepts
  either case; the emitter upcases.
- **xpub form.** BIP-32 base58 form (`xpub.../xprv...`); SLIP-132
  variants (`Zpub` / `Vpub` — capital, multisig) are NOT used on
  cosigner lines per the Coldcard format, even though the toolkit
  accepts them on the slot input (and normalizes to BIP-32 internally).
- **`Format:`.** `P2WSH` for `wsh-*` templates; `P2SH-P2WSH` for `sh-wsh-*`.
- **`Derivation:`.** A single line whose value is the shared origin
  path across cosigners (if all match); falls back to the Coldcard
  convention `m/0'/0'` if cosigners disagree.
- **`Name:` truncation.** Capped at 20 Unicode scalar values per the
  Coldcard reference format; non-ASCII names are truncated at
  codepoint granularity (not byte) so multi-byte sequences are not
  split.

The same text is byte-identical to Jade's
`register_multisig.multisig_file` — switch `--format coldcard` to
`--format jade` and the output is the same file. Both Coldcard and
Jade firmware import this file via SD-card or QR-stream.

Taproot multisig (`tr-multi-a` / `tr-sortedmulti-a`) is not yet
supported by either Coldcard or Jade firmware (tracked under
FOLLOWUPS `coldcard-tr-multi-a-pending-firmware` and
`jade-tr-multi-a-pending-firmware`). For taproot multisig setup,
use `--format bitcoin-core` (descriptor) or `--format sparrow`
(which supports taproot multisig via descriptor-passthrough).
