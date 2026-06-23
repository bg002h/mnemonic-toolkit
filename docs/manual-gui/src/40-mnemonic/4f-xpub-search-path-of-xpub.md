# `mnemonic xpub-search path-of-xpub` {#mnemonic-xpub-search-path-of-xpub}

Reverse-search a BIP-32 derivation graph: given a seed (BIP-39 phrase
or `ms1` card) and a target xpub (or `mk1` card carrying an xpub),
find the BIP-32 path under the seed that produces it. The default
candidate set is the standard derivation templates (BIP-44 / 49 / 84 /
86 single-sig + BIP-48 multisig at `script_type ∈ {1', 2', 3'}`) ×
account range; `--add-path` extends it. First match wins. The GUI
exposes this as one form under the **mnemonic** tab's subcommand
selector.
\index{mnemonic xpub-search path-of-xpub}

This mode takes secret seed material. The GUI renders the secret
inputs as masked `SecretLineEdit` widgets, and the run-confirm modal
redacts secret-bearing argv tokens as a fixed `••••` sentinel (see
[§14 Defense 2](#secret-handling)). Read-only search: **no private
keys reach stdout.**

:::danger
The worked example in this chapter uses the canonical all-`abandon`
BIP-39 test vector. **Never engrave or fund** a wallet derived from
this phrase — chain watchers have swept it continuously since 2017.
:::

## Outline {#mnemonic-xpub-search-path-of-xpub-outline}

- [`--phrase`](#mnemonic-xpub-search-path-of-xpub-phrase) — master BIP-39 phrase (inline)
- [`--phrase-stdin`](#mnemonic-xpub-search-path-of-xpub-phrase-stdin) — read the master phrase from stdin
- [`--ms1`](#mnemonic-xpub-search-path-of-xpub-ms1) — `ms1` card carrying BIP-39 entropy (inline)
- [`--ms1-stdin`](#mnemonic-xpub-search-path-of-xpub-ms1-stdin) — read the `ms1` card from stdin
- [`--passphrase`](#mnemonic-xpub-search-path-of-xpub-passphrase) — optional BIP-39 passphrase (inline)
- [`--passphrase-stdin`](#mnemonic-xpub-search-path-of-xpub-passphrase-stdin) — read the passphrase from stdin
- [`--target-xpub`](#mnemonic-xpub-search-path-of-xpub-target-xpub) — target xpub or `mk1` card carrying an xpub
- [`--language`](#mnemonic-xpub-search-path-of-xpub-language) — BIP-39 wordlist (default `english`)
- [`--network`](#mnemonic-xpub-search-path-of-xpub-network) — network selector (default `mainnet`)
- [`--min-account`](#mnemonic-xpub-search-path-of-xpub-min-account) — lower bound of account iteration (default `0`)
- [`--number-of-accounts`](#mnemonic-xpub-search-path-of-xpub-number-of-accounts) — window size (default `20`)
- [`--max-account`](#mnemonic-xpub-search-path-of-xpub-max-account) — optional upper bound
- [`--add-path`](#mnemonic-xpub-search-path-of-xpub-add-path) — additional path template (repeatable)
- [`--json`](#mnemonic-xpub-search-path-of-xpub-json) — emit JSON envelope instead of text

## `--phrase` {#mnemonic-xpub-search-path-of-xpub-phrase}

The master BIP-39 phrase, supplied inline. Part of the seed-intake
mutex — exactly one of `--phrase` / `--phrase-stdin` / `--ms1` /
`--ms1-stdin` (or a positional `ms1`) is required. Inline use emits
the argv-leakage advisory; prefer `--phrase-stdin`. The GUI renders
this as a masked `SecretLineEdit`.

## `--phrase-stdin` {#mnemonic-xpub-search-path-of-xpub-phrase-stdin}

Boolean flag. Read the master BIP-39 phrase from stdin. XOR with the
other seed-intake rows; single-stdin-per-invocation.

## `--ms1` {#mnemonic-xpub-search-path-of-xpub-ms1}

An `ms1` card carrying BIP-39 entropy, supplied inline. Inline use
emits the argv-leakage advisory; prefer `--ms1-stdin`. Auto-fire BCH
repair applies only to the `--ms1` decode-failure path. The GUI
renders this as a masked `SecretLineEdit`.

## `--ms1-stdin` {#mnemonic-xpub-search-path-of-xpub-ms1-stdin}

Boolean flag. Read the `ms1` card from stdin (single chunk). XOR with
the other seed-intake rows; single-stdin-per-invocation.

## `--passphrase` {#mnemonic-xpub-search-path-of-xpub-passphrase}

The optional BIP-39 passphrase, supplied inline. Inline use emits the
argv-leakage advisory; prefer `--passphrase-stdin`. The GUI renders
this as a masked `SecretLineEdit`.

## `--passphrase-stdin` {#mnemonic-xpub-search-path-of-xpub-passphrase-stdin}

Boolean flag. Read the passphrase from stdin (NULL-byte-preserving; a
single trailing newline is stripped). XOR with `--passphrase`;
single-stdin-per-invocation.

## `--target-xpub` {#mnemonic-xpub-search-path-of-xpub-target-xpub}

The target xpub (any SLIP-0132 prefix:
`xpub`/`tpub`/`ypub`/`Ypub`/`zpub`/`Zpub`/`upub`/`Upub`/`vpub`/`Vpub`)
OR an `mk1...` bech32 card carrying an xpub. Required. The GUI renders
this as a Text widget.

## `--language` {#mnemonic-xpub-search-path-of-xpub-language}

The BIP-39 wordlist used to interpret `--phrase`. Optional; defaults
to `english`. Same 10 allowed values as
[`mnemonic bundle --language`](#mnemonic-bundle-language). The GUI
renders this as a Dropdown widget.

### Outline {#mnemonic-xpub-search-path-of-xpub-language-outline}

- [`english`](#mnemonic-xpub-search-path-of-xpub-language-english)
- [`simplifiedchinese`](#mnemonic-xpub-search-path-of-xpub-language-simplifiedchinese)
- [`traditionalchinese`](#mnemonic-xpub-search-path-of-xpub-language-traditionalchinese)
- [`czech`](#mnemonic-xpub-search-path-of-xpub-language-czech)
- [`french`](#mnemonic-xpub-search-path-of-xpub-language-french)
- [`italian`](#mnemonic-xpub-search-path-of-xpub-language-italian)
- [`japanese`](#mnemonic-xpub-search-path-of-xpub-language-japanese)
- [`korean`](#mnemonic-xpub-search-path-of-xpub-language-korean)
- [`portuguese`](#mnemonic-xpub-search-path-of-xpub-language-portuguese)
- [`spanish`](#mnemonic-xpub-search-path-of-xpub-language-spanish)

### `english` {#mnemonic-xpub-search-path-of-xpub-language-english}

The BIP-39 English wordlist (2048 entries). Default.

### `simplifiedchinese` {#mnemonic-xpub-search-path-of-xpub-language-simplifiedchinese}

BIP-39 Simplified Chinese wordlist. Cross-tab divergence with
`ms encode --language chinese-simplified` is documented at
[`ms encode --language`](#ms-encode-language).

### `traditionalchinese` {#mnemonic-xpub-search-path-of-xpub-language-traditionalchinese}

BIP-39 Traditional Chinese wordlist.

### `czech` {#mnemonic-xpub-search-path-of-xpub-language-czech}

BIP-39 Czech wordlist.

### `french` {#mnemonic-xpub-search-path-of-xpub-language-french}

BIP-39 French wordlist.

### `italian` {#mnemonic-xpub-search-path-of-xpub-language-italian}

BIP-39 Italian wordlist.

### `japanese` {#mnemonic-xpub-search-path-of-xpub-language-japanese}

BIP-39 Japanese wordlist.

### `korean` {#mnemonic-xpub-search-path-of-xpub-language-korean}

BIP-39 Korean wordlist.

### `portuguese` {#mnemonic-xpub-search-path-of-xpub-language-portuguese}

BIP-39 Portuguese wordlist.

### `spanish` {#mnemonic-xpub-search-path-of-xpub-language-spanish}

BIP-39 Spanish wordlist.

## `--network` {#mnemonic-xpub-search-path-of-xpub-network}

The Bitcoin network selector for the searched derivations. Optional;
defaults to `mainnet`. The GUI renders this as a Dropdown widget.

### Outline {#mnemonic-xpub-search-path-of-xpub-network-outline}

- [`mainnet`](#mnemonic-xpub-search-path-of-xpub-network-mainnet)
- [`testnet`](#mnemonic-xpub-search-path-of-xpub-network-testnet)
- [`signet`](#mnemonic-xpub-search-path-of-xpub-network-signet)
- [`regtest`](#mnemonic-xpub-search-path-of-xpub-network-regtest)

### `mainnet` {#mnemonic-xpub-search-path-of-xpub-network-mainnet}

Production Bitcoin mainnet. BIP-44 coin-type 0.

### `testnet` {#mnemonic-xpub-search-path-of-xpub-network-testnet}

The legacy public test network. Coin-type 1. Funds are valueless.

### `signet` {#mnemonic-xpub-search-path-of-xpub-network-signet}

The signature-secured test network. Coin-type 1. Funds are valueless.

### `regtest` {#mnemonic-xpub-search-path-of-xpub-network-regtest}

A locally-controlled regression-test network. Coin-type 1.

## `--min-account` {#mnemonic-xpub-search-path-of-xpub-min-account}

The lower bound of account-index iteration, inclusive. Default `0`.
The GUI renders this as a Number widget.

## `--number-of-accounts` {#mnemonic-xpub-search-path-of-xpub-number-of-accounts}

The window size of the account-index iteration, starting at
`--min-account`. Default `20`. The GUI renders this as a Number
widget.

## `--max-account` {#mnemonic-xpub-search-path-of-xpub-max-account}

Optional upper bound on the account index. Effective end:
`max(min_account + number_of_accounts, max_account + 1)`. The GUI
renders this as a Number widget.

## `--add-path` {#mnemonic-xpub-search-path-of-xpub-add-path}

An additional derivation-path template, repeatable. The literal token
`account'` (or `account`) is substituted with each iterated account
index; templates without an `account` token are searched once at the
literal path. The GUI renders this as a repeating Text input.

## `--json` {#mnemonic-xpub-search-path-of-xpub-json}

Boolean flag. Emit a versioned JSON envelope (schema `v1`,
`mode: "path-of-xpub"`) instead of the text report. The match shape
carries `path`, `template`, `account`, `target_xpub_canonical`,
`target_xpub_variant`, and `searched_count`; the no-match shape drops
`path`/`template`/`account`. `target_xpub_variant` serializes as
`null` when the target was already canonical `xpub`/`tpub`. The GUI
renders this as a checkbox.

## Worked example — find the path that produces a target xpub

1. Switch to the **mnemonic** tab; pick **xpub-search:
   path-of-xpub** in the subcommand selector.
2. Set the seed-intake to **`--phrase-stdin`** (the seed flows via
   stdin, never argv).
3. Paste the target xpub (or `mk1` card) into `--target-xpub`.
4. Leave the range knobs at their defaults.
5. Click **Run** and confirm the modal (the pasted phrase shows as
   `••••`).

Stdout (text form, match):

```text
match: m/84'/0'/0'  (template=bip84, account=0)
target-xpub: xpub6... (normalized from zpub; variant=zpub)
searched: 7 templates × 20 accounts = 140 paths
```

A no-match result exits 4; bad input (BIP-39/xpub parse failure)
exits 1; a clap arg-parse error exits 64.

## Exit codes

| Code | Meaning |
|---|---|
| 0 | Match found |
| 1 | Bad input (BIP-39 parse failure, xpub parse failure, seed-intake error) |
| 4 | No match in searched set |
| 5 | Auto-fire BCH short-circuit on `--ms1` decode failure (TTY-gated) |
| 64 | Clap arg-parse error |
