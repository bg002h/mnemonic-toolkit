# `mnemonic xpub-search address-of-xpub` {#mnemonic-xpub-search-address-of-xpub}

Reverse-search a BIP-32 derivation graph by address: given a parent
xpub (or `mk1` card carrying an xpub) plus one or more target
addresses, scan child receive (`chain=0`) and change (`chain=1`)
addresses across the gap-limit window and report which targets matched
at which `(chain, index)`. Takes **no seed material** — auto-fire BCH
repair does not apply, and there is no argv-leakage surface beyond the
(non-secret) xpub itself. The GUI exposes this as one form under the
**mnemonic** tab's subcommand selector.
\index{mnemonic xpub-search address-of-xpub}

The script-type used to render each child address comes from the
xpub's SLIP-0132 prefix where unambiguous (`ypub`/`upub` →
P2SH-P2WPKH; `zpub`/`vpub` → P2WPKH); for neutral `xpub`/`tpub` (or to
override), supply `--address-type`. Multisig SLIP-0132 prefixes
(`Ypub`/`Zpub`/`Upub`/`Vpub`) are refused — use
[`account-of-descriptor`](#mnemonic-xpub-search-account-of-descriptor)
instead. Read-only search: **no private keys are involved.**

> **GUI form:** see [GUI Forms › mnemonic › xpub-search-address-of-xpub](#gui-form-mnemonic-xpub-search-address-of-xpub).

## Outline {#mnemonic-xpub-search-address-of-xpub-outline}

- [`--xpub`](#mnemonic-xpub-search-address-of-xpub-xpub) — parent xpub or `mk1` card (single-sig prefix)
- [`--xpub-stdin`](#mnemonic-xpub-search-address-of-xpub-xpub-stdin) — read the parent xpub from stdin
- [`--target-address`](#mnemonic-xpub-search-address-of-xpub-target-address) — target address (repeatable; at least one required)
- [`--gap-limit`](#mnemonic-xpub-search-address-of-xpub-gap-limit) — per-chain scan window (default `20`)
- [`--external-only`](#mnemonic-xpub-search-address-of-xpub-external-only) — scan the receive chain only (skip change)
- [`--address-type`](#mnemonic-xpub-search-address-of-xpub-address-type) — explicit child-address script type
- [`--network`](#mnemonic-xpub-search-address-of-xpub-network) — network selector (default inferred from the xpub)
- [`--json`](#mnemonic-xpub-search-address-of-xpub-json) — emit JSON envelope instead of text

## `--xpub` {#mnemonic-xpub-search-address-of-xpub-xpub}

The parent xpub (any SLIP-0132 single-sig prefix:
`xpub`/`tpub`/`ypub`/`upub`/`zpub`/`vpub`) OR an `mk1...` bech32 card
carrying an xpub. Mutex with `--xpub-stdin`; supplying neither is
refused with `supply --xpub <VALUE> or --xpub-stdin`. Multisig
prefixes (`Ypub`/`Zpub`/`Upub`/`Vpub`) are refused. The xpub is public
material — not a secret. The GUI renders this as a Text widget.

## `--xpub-stdin` {#mnemonic-xpub-search-address-of-xpub-xpub-stdin}

Boolean flag. Read the parent xpub from stdin (single line, trailing
newline stripped). Mutex with `--xpub`. The GUI renders this as a
checkbox.

## `--target-address` {#mnemonic-xpub-search-address-of-xpub-target-address}

A target address to search for, repeatable; at least one is required.
The GUI renders this as a repeating Text input (one row per
occurrence; the argv assembler emits one `--target-address` token per
row). The per-target results appear in the output in user-supplied
order.

## `--gap-limit` {#mnemonic-xpub-search-address-of-xpub-gap-limit}

The per-chain scan window — child indices `0..N`. Default `20`. The
GUI renders this as a Number widget.

## `--external-only` {#mnemonic-xpub-search-address-of-xpub-external-only}

Boolean flag. Restrict the scan to the external (receive) chain; skip
the change chain. The default scans both. When set, `scanned_internal`
is `0` for no-match JSON entries. The GUI renders this as a checkbox.

## `--address-type` {#mnemonic-xpub-search-address-of-xpub-address-type}

The explicit script-type for child-address rendering. Required for
neutral `xpub`/`tpub` (which carry no SLIP-0132 single-sig signal);
otherwise it overrides the prefix-inferred type. Four allowed values.
The GUI renders this as a Dropdown widget.

### Outline {#mnemonic-xpub-search-address-of-xpub-address-type-outline}

- [`p2pkh`](#mnemonic-xpub-search-address-of-xpub-address-type-p2pkh)
- [`p2sh-p2wpkh`](#mnemonic-xpub-search-address-of-xpub-address-type-p2sh-p2wpkh)
- [`p2wpkh`](#mnemonic-xpub-search-address-of-xpub-address-type-p2wpkh)
- [`p2tr`](#mnemonic-xpub-search-address-of-xpub-address-type-p2tr)

### `p2pkh` {#mnemonic-xpub-search-address-of-xpub-address-type-p2pkh}

Legacy pay-to-pubkey-hash. Address prefix `1` on mainnet.

### `p2sh-p2wpkh` {#mnemonic-xpub-search-address-of-xpub-address-type-p2sh-p2wpkh}

Nested SegWit (P2SH-wrapped P2WPKH). Address prefix `3` on mainnet.

### `p2wpkh` {#mnemonic-xpub-search-address-of-xpub-address-type-p2wpkh}

Native SegWit P2WPKH. Address prefix `bc1q` on mainnet.

### `p2tr` {#mnemonic-xpub-search-address-of-xpub-address-type-p2tr}

Taproot P2TR (key-path). Address prefix `bc1p` on mainnet.

## `--network` {#mnemonic-xpub-search-address-of-xpub-network}

The Bitcoin network selector. Optional; default inferred from the
xpub version byte. `--network signet` / `--network regtest` overrides
the test/signet/regtest ambiguity the version byte collapses. The GUI
renders this as a Dropdown widget.

### Outline {#mnemonic-xpub-search-address-of-xpub-network-outline}

- [`mainnet`](#mnemonic-xpub-search-address-of-xpub-network-mainnet)
- [`testnet`](#mnemonic-xpub-search-address-of-xpub-network-testnet)
- [`signet`](#mnemonic-xpub-search-address-of-xpub-network-signet)
- [`regtest`](#mnemonic-xpub-search-address-of-xpub-network-regtest)

### `mainnet` {#mnemonic-xpub-search-address-of-xpub-network-mainnet}

Production Bitcoin mainnet.

### `testnet` {#mnemonic-xpub-search-address-of-xpub-network-testnet}

The legacy public test network. Funds are valueless.

### `signet` {#mnemonic-xpub-search-address-of-xpub-network-signet}

The signature-secured test network. Funds are valueless.

### `regtest` {#mnemonic-xpub-search-address-of-xpub-network-regtest}

A locally-controlled regression-test network.

## `--json` {#mnemonic-xpub-search-address-of-xpub-json}

Boolean flag. Emit a versioned JSON envelope (schema `v1`,
`mode: "address-of-xpub"`) instead of the text report. The `results`
array carries one entry per `--target-address` in user-supplied order;
mixed match / no-match payloads are supported. Each match entry
carries `target`, `result`, `chain`, `index`, `script_type`; the
envelope carries `xpub_canonical`, `xpub_variant`, `gap_limit`. The
GUI renders this as a checkbox.

## Worked example — confirm a child address by index

1. Switch to the **mnemonic** tab; pick **xpub-search:
   address-of-xpub** in the subcommand selector.
2. Paste an account-level zpub into `--xpub`.
3. Add the candidate child address to the `--target-address` rows.
4. Click **Run**.

Stdout (text form, match):

```text
match: bc1q... → 0/5  (script_type=p2wpkh, chain=external, index=5)
targets: 1; matched: 1; unmatched: 0
```

The summary line reports total / matched / unmatched counts after the
per-target lines. An unmatched target exits 4; bad input (xpub or
address parse failure, multisig prefix, missing `--address-type` for a
neutral xpub) exits 1.

## Exit codes

| Code | Meaning |
|---|---|
| 0 | All targets matched |
| 1 | Bad input (xpub parse failure, address parse failure, multisig SLIP-0132 prefix, missing `--address-type` for neutral xpub) |
| 4 | At least one target unmatched |
| 64 | Clap arg-parse error |

This mode takes no secret material; auto-fire BCH repair (exit 5)
does not apply.
