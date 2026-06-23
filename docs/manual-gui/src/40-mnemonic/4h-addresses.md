# `mnemonic addresses` {#mnemonic-addresses}

List a wallet's receive/change addresses in batch. The watch-only
complement to `export-wallet --range` and the multi-address sibling of
`convert --to address`. Read-only public derivation — **no private
keys reach stdout, and `mnemonic` never signs.** The GUI exposes this
as one form under the **mnemonic** tab's subcommand selector; output
is the address rows on stdout (text or JSON).
\index{mnemonic addresses}

`--from` accepts an account `xpub=` (derived directly) or a seed
source (`phrase=` / `entropy=` / `seedqr=` / `electrum-phrase=`). For a
BIP-39 seed source, `--address-type` selects the BIP-44/49/84/86
account path; for an `xpub=` source the xpub *is* the account key, so
`--account` / `--passphrase` do not apply.

:::danger
The worked example in this chapter uses a public account-level xpub —
no secret material. When you supply a `phrase=` / `entropy=` /
`seedqr=` seed source instead, the GUI renders the secret value as a
masked `SecretLineEdit` and the run-confirm modal redacts it to a
fixed `••••` sentinel (see [§14 Defense 2](#secret-handling)). Use the
canonical all-`abandon` test vector for practice; **never fund** a
wallet derived from it.
:::

## Outline {#mnemonic-addresses-outline}

- [`--from`](#mnemonic-addresses-from) — source: `xpub=` or a seed source (required)
- [`--address-type`](#mnemonic-addresses-address-type) — script type / account path (required)
- [`--account`](#mnemonic-addresses-account) — account index for BIP-39 seed sources (default `0`)
- [`--count`](#mnemonic-addresses-count) — addresses per chain from index 0 (default `10`)
- [`--range`](#mnemonic-addresses-range) — inclusive index range `A..=B` (conflicts with `--count`)
- [`--chain`](#mnemonic-addresses-chain) — which chain(s) to list (`receive`/`change`/`both`)
- [`--network`](#mnemonic-addresses-network) — network selector
- [`--passphrase`](#mnemonic-addresses-passphrase) — BIP-39 passphrase (seed sources)
- [`--passphrase-stdin`](#mnemonic-addresses-passphrase-stdin) — read the passphrase from stdin
- [`--language`](#mnemonic-addresses-language) — BIP-39 wordlist (default `english`)
- [`--json`](#mnemonic-addresses-json) — emit JSON envelope instead of text rows

## `--from` {#mnemonic-addresses-from}

The source: `xpub=<v>` | `phrase=<v>` | `entropy=<hex>` |
`seedqr=<digits>` | `electrum-phrase=<v>`. Secret values support
`@env:VAR` and `-` (stdin). Required. For an `xpub=` source the xpub
*is* the account key (so `--account` / `--passphrase` do not apply;
supplying them is an error). `electrum-phrase=` (v0.47.0+) derives
Electrum's own native-seed addresses (not BIP-39/BIP-44); the script
type is fixed by the Electrum seed version. The GUI renders this as a
Text widget; for a secret source the value sub-field flips to a masked
`SecretLineEdit`.

## `--address-type` {#mnemonic-addresses-address-type}

The script type — `p2pkh` | `p2sh-p2wpkh` | `p2wpkh` | `p2tr`.
Required. For BIP-39 seed sources it selects the account path
(`p2pkh`→44', `p2sh-p2wpkh`→49', `p2wpkh`→84', `p2tr`→86') and the
render type; for `electrum-phrase=` it must match the seed version
(`p2pkh` standard / `p2wpkh` segwit — a mismatch is refused). The GUI
renders this as a Text widget (free-form, not a fixed dropdown enum at
the schema level).

## `--account` {#mnemonic-addresses-account}

The account index for BIP-39 seed sources. Default `0`. Not
applicable to `xpub=` or `electrum-phrase=` sources (supplying a
non-zero value is refused). The GUI renders this as a Number widget.

## `--count` {#mnemonic-addresses-count}

The number of addresses per chain, from index 0. Default `10`.
Conflicts with `--range`. Indices are bounded by the BIP-32
normal-index ceiling (`< 2^31`); an out-of-range request is rejected
(never a panic). The GUI renders this as a Number widget.

## `--range` {#mnemonic-addresses-range}

An inclusive index range `A..=B`. Conflicts with `--count`. The GUI
renders this as a Text widget (the `A,B` pair string).

## `--chain` {#mnemonic-addresses-chain}

Which chain(s) to list — `receive` / `change` / `both`. Default
`receive`. With `both`, rows are grouped by a `receive (m/0/i):` /
`change (m/1/i):` header. The GUI renders this as a Dropdown widget.

## `--network` {#mnemonic-addresses-network}

The Bitcoin network selector. Defaults to the xpub's version bytes
(xpub source) or `mainnet` (seed source); it must agree with an
xpub's network kind. The GUI renders this as a Dropdown widget.

### Outline {#mnemonic-addresses-network-outline}

- [`mainnet`](#mnemonic-addresses-network-mainnet)
- [`testnet`](#mnemonic-addresses-network-testnet)
- [`signet`](#mnemonic-addresses-network-signet)
- [`regtest`](#mnemonic-addresses-network-regtest)

### `mainnet` {#mnemonic-addresses-network-mainnet}

Production Bitcoin mainnet. BIP-44 coin-type 0.

### `testnet` {#mnemonic-addresses-network-testnet}

The legacy public test network. Coin-type 1. Funds are valueless.

### `signet` {#mnemonic-addresses-network-signet}

The signature-secured test network. Coin-type 1. Funds are valueless.

### `regtest` {#mnemonic-addresses-network-regtest}

A locally-controlled regression-test network. Coin-type 1.

## `--passphrase` {#mnemonic-addresses-passphrase}

The BIP-39 passphrase for seed sources; `@env:VAR` is supported.
Inline use emits the argv-leakage advisory; prefer
`--passphrase-stdin`. The GUI renders this as a masked
`SecretLineEdit`.

## `--passphrase-stdin` {#mnemonic-addresses-passphrase-stdin}

Boolean flag. Read the BIP-39 passphrase from stdin. Conflicts with
`--passphrase`. The GUI renders this as a checkbox.

## `--language` {#mnemonic-addresses-language}

The BIP-39 wordlist for `phrase=` / `seedqr=` sources. Default
`english`; ignored for `electrum-phrase=` (the Electrum seed is
stretched from the raw phrase string, not decoded via a wordlist).
Same 10 allowed values as
[`mnemonic bundle --language`](#mnemonic-bundle-language). The GUI
renders this as a Dropdown widget.

### Outline {#mnemonic-addresses-language-outline}

- [`english`](#mnemonic-addresses-language-english)
- [`simplifiedchinese`](#mnemonic-addresses-language-simplifiedchinese)
- [`traditionalchinese`](#mnemonic-addresses-language-traditionalchinese)
- [`czech`](#mnemonic-addresses-language-czech)
- [`french`](#mnemonic-addresses-language-french)
- [`italian`](#mnemonic-addresses-language-italian)
- [`japanese`](#mnemonic-addresses-language-japanese)
- [`korean`](#mnemonic-addresses-language-korean)
- [`portuguese`](#mnemonic-addresses-language-portuguese)
- [`spanish`](#mnemonic-addresses-language-spanish)

### `english` {#mnemonic-addresses-language-english}

The BIP-39 English wordlist (2048 entries). Default.

### `simplifiedchinese` {#mnemonic-addresses-language-simplifiedchinese}

BIP-39 Simplified Chinese wordlist. Cross-tab divergence with
`ms encode --language chinese-simplified` is documented at
[`ms encode --language`](#ms-encode-language).

### `traditionalchinese` {#mnemonic-addresses-language-traditionalchinese}

BIP-39 Traditional Chinese wordlist.

### `czech` {#mnemonic-addresses-language-czech}

BIP-39 Czech wordlist.

### `french` {#mnemonic-addresses-language-french}

BIP-39 French wordlist.

### `italian` {#mnemonic-addresses-language-italian}

BIP-39 Italian wordlist.

### `japanese` {#mnemonic-addresses-language-japanese}

BIP-39 Japanese wordlist.

### `korean` {#mnemonic-addresses-language-korean}

BIP-39 Korean wordlist.

### `portuguese` {#mnemonic-addresses-language-portuguese}

BIP-39 Portuguese wordlist.

### `spanish` {#mnemonic-addresses-language-spanish}

BIP-39 Spanish wordlist.

## `--json` {#mnemonic-addresses-json}

Boolean flag. Emit a JSON envelope (`schema_version: "1"`, with
`source`, `address_type`, `network`, optional `account`, and an
`addresses[]` array of `{chain, index, address}`) instead of the text
rows (`account` is present only for seed sources). The GUI renders
this as a checkbox.

## Worked example — list 3 receive addresses from an account xpub

1. Switch to the **mnemonic** tab; pick **addresses** in the
   subcommand selector.
2. Set `--from` to `xpub=<your account-level xpub>`.
3. Set `--address-type` to `p2wpkh` and `--count` to `3`.
4. Click **Run** (no run-confirm modal — an xpub is public).

Stdout (text mode prints two-space-indented `<index>  <address>`
rows). Because the addresses are derived keys, the non-English
wordlist advisory does not fire here (the language is already baked
into the derivation).
