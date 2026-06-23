# `ms derive` {#ms-derive}

\index{ms derive}Derive the **master fingerprint** (the cheapest
"did I recover the right seed?" check) and, with
[`--template`](#ms-derive-template), an **account xpub** for
watch-only setup (ms-cli v0.5+). This is **read-only public
derivation** — no master seed, root xprv, or private keys reach
stdout, and `ms` never signs. For secret-bearing outputs use the
toolkit's `mnemonic convert`.

The entropy source is one of: a positional `ms1` string, raw
[`--hex`](#ms-derive-hex) entropy, or a BIP-39
[`--phrase`](#ms-derive-phrase). The BIP-39 seed is `PBKDF2` over
the *language-specific* mnemonic string, so the fingerprint / xpub
depend on [`--language`](#ms-derive-language); when omitted,
`english` is used and annotated `DEFAULT` on stdout and stderr
(exactly like [`ms decode`](#ms-decode)).

## Outline {#ms-derive-outline}

- [`--hex`](#ms-derive-hex) — raw entropy hex (alternative to the `ms1`, secret-bearing)
- [`--phrase`](#ms-derive-phrase) — BIP-39 mnemonic (alternative to the `ms1`, secret-bearing)
- [`--template`](#ms-derive-template) — account-path template (`bip44`|`bip49`|`bip84`|`bip86`); emits an account xpub
- [`--account`](#ms-derive-account) — account index for `--template` (default `0`)
- [`--network`](#ms-derive-network) — `mainnet` (default) or `testnet`; coin-type + xpub/tpub serialization
- [`--passphrase`](#ms-derive-passphrase) — BIP-39 passphrase (secret-bearing)
- [`--passphrase-stdin`](#ms-derive-passphrase-stdin) — read the BIP-39 passphrase from stdin
- [`--language`](#ms-derive-language) — BIP-39 wordlist (load-bearing; default `english`)
- [`--json`](#ms-derive-json) — emit a single JSON object on stdout instead of text

## Positional `ms1`

A single `ms1` string to derive from. **Secret-bearing** —
schema-`secret: true` on the positional. Optional at the clap
level; when omitted or set to a literal `-`, the binary reads the
string from stdin. Alternative to [`--hex`](#ms-derive-hex) /
[`--phrase`](#ms-derive-phrase). Any non-empty value triggers the
run-confirm modal.

## `--hex` {#ms-derive-hex}

Raw entropy as a hex string, an alternative to the positional
`ms1`. **Secret-bearing** — schema-`secret: true`. The GUI renders
this as a `SecretLineEdit` widget; a non-empty value fires the
run-confirm modal.

## `--phrase` {#ms-derive-phrase}

A BIP-39 mnemonic phrase, an alternative to the positional `ms1`.
**Secret-bearing** — schema-`secret: true`. The GUI renders this
as a `SecretLineEdit` widget; a non-empty value fires the
run-confirm modal. Parsed under [`--language`](#ms-derive-language).

## `--template` {#ms-derive-template}

Account-path template. Dropdown widget. When supplied, `ms derive`
additionally emits an **account xpub** at
`m/<purpose>'/<coin>'/<account>'`; without it, only the master
fingerprint is emitted. The four templates select the BIP purpose:
`bip44` (legacy P2PKH), `bip49` (P2SH-P2WPKH), `bip84` (native
P2WPKH), `bip86` (P2TR). The coin index follows
[`--network`](#ms-derive-network); the account index follows
[`--account`](#ms-derive-account).

## `--account` {#ms-derive-account}

Account index for [`--template`](#ms-derive-template). Text widget
(`u32`); default `0`. Selects the hardened account level in
`m/<purpose>'/<coin>'/<account>'`. Has no effect without
`--template`.

## `--network` {#ms-derive-network}

Network selector. Dropdown widget; `mainnet` (default) or
`testnet`. Sets the coin-type in the derivation path and the xpub
(`xpub`) vs tpub (`tpub`) serialization of the account key.

## `--passphrase` {#ms-derive-passphrase}

The optional BIP-39 passphrase (the "25th word"). **Secret-
bearing** — schema-`secret: true`. The GUI renders this as a
`SecretLineEdit` widget; a non-empty value fires the run-confirm
modal. Conflicts with
[`--passphrase-stdin`](#ms-derive-passphrase-stdin). The passphrase
changes the derived seed, so the fingerprint and xpub differ from
the no-passphrase derivation.

## `--passphrase-stdin` {#ms-derive-passphrase-stdin}

Boolean. Read the BIP-39 passphrase from stdin instead of the
[`--passphrase`](#ms-derive-passphrase) argument (keeps the
passphrase off the argv). Conflicts with `--passphrase`. Default
off.

## `--language` {#ms-derive-language}

BIP-39 wordlist used to interpret [`--phrase`](#ms-derive-phrase).
**Load-bearing** — the seed is `PBKDF2` over the language-specific
phrase, so the wrong wordlist yields a different fingerprint /
xpub. Optional; defaults to `english` (annotated `DEFAULT`).
Dropdown widget; 10 valid values, hyphenated Chinese tokens (see
[§61 cross-tab divergence](#ms-per-tab-reference)).

### Outline {#ms-derive-language-outline}

- [`english`](#ms-derive-language-english)
- [`japanese`](#ms-derive-language-japanese)
- [`korean`](#ms-derive-language-korean)
- [`spanish`](#ms-derive-language-spanish)
- [`chinese-simplified`](#ms-derive-language-chinese-simplified)
- [`chinese-traditional`](#ms-derive-language-chinese-traditional)
- [`french`](#ms-derive-language-french)
- [`italian`](#ms-derive-language-italian)
- [`czech`](#ms-derive-language-czech)
- [`portuguese`](#ms-derive-language-portuguese)

### `english` {#ms-derive-language-english}

See [`ms encode --language english`](#ms-encode-language-english).
Default; annotated `DEFAULT` when omitted.

### `japanese` {#ms-derive-language-japanese}

See [`ms encode --language japanese`](#ms-encode-language-japanese).

### `korean` {#ms-derive-language-korean}

See [`ms encode --language korean`](#ms-encode-language-korean).

### `spanish` {#ms-derive-language-spanish}

See [`ms encode --language spanish`](#ms-encode-language-spanish).

### `chinese-simplified` {#ms-derive-language-chinese-simplified}

See [`ms encode --language
chinese-simplified`](#ms-encode-language-chinese-simplified).

### `chinese-traditional` {#ms-derive-language-chinese-traditional}

See [`ms encode --language
chinese-traditional`](#ms-encode-language-chinese-traditional).

### `french` {#ms-derive-language-french}

See [`ms encode --language french`](#ms-encode-language-french).

### `italian` {#ms-derive-language-italian}

See [`ms encode --language italian`](#ms-encode-language-italian).

### `czech` {#ms-derive-language-czech}

See [`ms encode --language czech`](#ms-encode-language-czech).

### `portuguese` {#ms-derive-language-portuguese}

See [`ms encode --language
portuguese`](#ms-encode-language-portuguese).

## `--json` {#ms-derive-json}

Boolean. Emit a single JSON object on stdout instead of the
labeled-text form. Default off. Fields: `schema_version`,
`master_fingerprint`, `network`, `account_path?`, `account_xpub?`,
`language`, `language_defaulted` (the `account_*` fields are
present only with [`--template`](#ms-derive-template)).

## Worked example — master fingerprint + bip84 account xpub

:::danger
Examples use the canonical all-`abandon` test vector — a
**public** seed swept since 2017. Never engrave or fund any wallet
derived from it.
:::

1. **ms** tab; pick **Derive (master fingerprint + account
   xpub)**.
2. Paste the canonical `ms1` into the `ms1` positional field:

   ```text
   ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f
   ```

3. [`--template`](#ms-derive-template): `bip84`.
4. Leave [`--account`](#ms-derive-account) at `0`,
   [`--network`](#ms-derive-network) at `mainnet`,
   [`--language`](#ms-derive-language) at default `english`.
5. Click **Run**. The run-confirm modal fires (secret-bearing
   `ms1`); confirm to proceed.

The output panel emits the public derivation result on stdout:

```{.text include="69-ms-derive-bip84.out"}
master_fingerprint:  73c5da0a
account_path:        m/84'/0'/0'
account_xpub:        xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V
language:            english (DEFAULT)
```

## Refusals

| Trigger | Refusal |
|---|---|
| No entropy source (`ms1` / `--hex` / `--phrase`) | clap refusal: required input not provided |
| More than one entropy source | clap-group refusal: mutually-exclusive |
| `--passphrase` and `--passphrase-stdin` together | clap conflict refusal |
| `--account` without `--template` | the account index is ignored (master-fingerprint-only output) |
| `--phrase` with invalid BIP-39 checksum | exit 1 with `error: <bip39 error>` |
| `--language <token>` not in the 10-value set | clap value-parser refusal |
