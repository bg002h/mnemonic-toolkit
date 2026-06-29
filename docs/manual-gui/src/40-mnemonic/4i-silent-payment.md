# `mnemonic silent-payment` {#mnemonic-silent-payment}

Derive a [BIP-352](https://github.com/bitcoin/bips/blob/master/bip-0352.mediawiki)
**Silent Payments** *receiver* static address from a seed-bearing
secret. A silent payment address (`sp1…` mainnet, `tsp1…`
testnet/signet/regtest) is published once; senders derive a unique
on-chain output for each payment with no on-chain link and no
sender↔receiver interaction. The GUI exposes this as one form under
the **mnemonic** tab's subcommand selector.
\index{mnemonic silent-payment}

The scan key is derived at `m/352'/<coin>'/<account>'/1'/0` and the
spend key at `m/352'/<coin>'/<account>'/0'/0`; the base (unlabeled)
address encodes the compressed pubkeys `B_scan ‖ B_spend`. This
derives the **receiver** address only — sender output construction and
chain scanning are out of scope.

:::danger
The worked example uses the canonical all-`abandon` BIP-39 test
vector. **Never engrave or fund** a wallet derived from this phrase.
The GUI renders the secret input as a masked `SecretLineEdit`; the
run-confirm modal redacts secret-bearing argv tokens to a fixed
`••••` sentinel (see [§14 Defense 2](#secret-handling)). The command
emits the **spend private key** (`b_spend`, the COLD key with full
spending authority) behind the `warning: stdout carries private key
material` advisory — never paste `b_spend` into a scanning service.
:::

> **GUI form:** see [GUI Forms › mnemonic › silent-payment](#gui-form-mnemonic-silent-payment).

## Outline {#mnemonic-silent-payment-outline}

- [`--account`](#mnemonic-silent-payment-account) — BIP-32 account index (default `0`)
- [`--change-address`](#mnemonic-silent-payment-change-address) — also emit the m=0 change address (own-detection only)
- [`--json`](#mnemonic-silent-payment-json) — emit JSON envelope instead of text
- [`--label`](#mnemonic-silent-payment-label) — emit a labeled address for label m (repeatable; m≥1)
- [`--network`](#mnemonic-silent-payment-network) — network selector (default `mainnet`)
- [`--passphrase`](#mnemonic-silent-payment-passphrase) — BIP-39 passphrase (inline)
- [`--passphrase-stdin`](#mnemonic-silent-payment-passphrase-stdin) — read the passphrase from stdin
- [`--secret`](#mnemonic-silent-payment-secret) — seed-bearing secret (inline)
- [`--secret-file`](#mnemonic-silent-payment-secret-file) — read the secret from a file
- [`--secret-stdin`](#mnemonic-silent-payment-secret-stdin) — read the secret from stdin

## `--account` {#mnemonic-silent-payment-account}

The BIP-32 account index `m/352'/coin'/<account>'/…`. Default `0`. The
GUI renders this as a Number widget.

## `--change-address` {#mnemonic-silent-payment-change-address}

Boolean flag. Also emit the BIP-352 **m=0 change address** — for the
wallet's OWN change detection ONLY; **never hand it out as a receiving
address** (additive; the base address is still emitted). The JSON
envelope carries a `change_address_warning` when set. The GUI renders
this as a checkbox.

## `--json` {#mnemonic-silent-payment-json}

Boolean flag. Emit a JSON envelope instead of the human-readable
block. The GUI renders this as a checkbox.

## `--label` {#mnemonic-silent-payment-label}

Emit a labeled address for label m (repeatable); **m≥1**. A labeled
address encodes `B_scan ‖ B_m` where
`B_m = B_spend + hash_BIP0352/Label(b_scan ‖ m)·G`. `--label 0` is
refused — m=0 is the reserved BIP-352 change label and must never be
published (use `--change-address` for own-change detection). The GUI
renders this as a repeating Number input (one row per occurrence).

## `--network` {#mnemonic-silent-payment-network}

The Bitcoin network selector. Default `mainnet` → `sp` address +
coin-type 0; `testnet`/`signet`/`regtest` → `tsp` address +
coin-type 1. The GUI renders this as a Dropdown widget.

### Outline {#mnemonic-silent-payment-network-outline}

- [`mainnet`](#mnemonic-silent-payment-network-mainnet)
- [`testnet`](#mnemonic-silent-payment-network-testnet)
- [`signet`](#mnemonic-silent-payment-network-signet)
- [`regtest`](#mnemonic-silent-payment-network-regtest)

### `mainnet` {#mnemonic-silent-payment-network-mainnet}

Production Bitcoin mainnet. Coin-type 0; address HRP `sp`.

### `testnet` {#mnemonic-silent-payment-network-testnet}

The legacy public test network. Coin-type 1; address HRP `tsp`. Funds
are valueless.

### `signet` {#mnemonic-silent-payment-network-signet}

The signature-secured test network. Coin-type 1; address HRP `tsp`.
Funds are valueless.

### `regtest` {#mnemonic-silent-payment-network-regtest}

A locally-controlled regression-test network. Coin-type 1; address
HRP `tsp`.

## `--passphrase` {#mnemonic-silent-payment-passphrase}

The BIP-39 mnemonic-extension passphrase ("25th word"). Applies to
phrase / `ms1` / entropy-hex inputs; **ignored (with a warning) for an
xprv input** (the xprv is already the master). Inline use emits the
argv-leakage advisory; prefer `--passphrase-stdin`. The GUI renders
this as a masked `SecretLineEdit`.

## `--passphrase-stdin` {#mnemonic-silent-payment-passphrase-stdin}

Boolean flag. Read the BIP-39 passphrase from stdin
(whitespace-preserving — significant PBKDF2 salt). Mutually exclusive
with `--passphrase`, and with `--secret-stdin` (one stdin per
invocation). The GUI renders this as a checkbox.

## `--secret` {#mnemonic-silent-payment-secret}

The seed-bearing secret: BIP-39 phrase / `ms1` / entropy-hex / master
xprv, supplied inline. A single private key (WIF/minikey) is refused —
it cannot derive `m/352'`. Inline use emits the argv-leakage advisory;
prefer `--secret-file` / `--secret-stdin`. Exactly one of `--secret` /
`--secret-file` / `--secret-stdin` is required (clap arg-group). The
GUI renders this as a masked `SecretLineEdit`.

## `--secret-file` {#mnemonic-silent-payment-secret-file}

Read the seed-bearing secret from a file (avoids argv exposure). One
of the required secret-source group. The GUI renders this as a Path
widget.

## `--secret-stdin` {#mnemonic-silent-payment-secret-stdin}

Boolean flag. Read the seed-bearing secret from stdin. One of the
required secret-source group. Mutually exclusive with
`--passphrase-stdin` (one stdin per invocation). The GUI renders this
as a checkbox.

## Worked example — derive a receiver silent-payment address

1. Switch to the **mnemonic** tab; pick **silent-payment** in the
   subcommand selector.
2. Set the secret source to **`--secret-stdin`** (the seed flows via
   stdin, never argv).
3. Leave `--account` at `0` and `--network` at `mainnet`.
4. Click **Run** and confirm the modal.

The address and the scan/spend **public** keys are publishable — hand
the base (`sp1…`) address to senders. The command also emits the scan
private key (`b_scan`, the online/hot key a watch-server uses) and the
spend private key (`b_spend`, the COLD key) behind the
private-key-material advisory; the secret is `mlock`-pinned +
zeroized.
