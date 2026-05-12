# `mnemonic` reference

The integration-layer CLI for the m-format constellation. Five subcommands:
[`bundle`](#mnemonic-bundle), [`verify-bundle`](#mnemonic-verify-bundle),
[`convert`](#mnemonic-convert), [`export-wallet`](#mnemonic-export-wallet),
and [`derive-child`](#mnemonic-derive-child). Run any with `--help`
for the latest flag set; this chapter mirrors v0.8.0.

---

## `mnemonic bundle`

Synthesise a 3-card engraving bundle from a phrase, entropy, or
xpub. Inputs are slotted via `--slot @N.<subkey>=<value>`, repeating.

### Synopsis

```sh
mnemonic bundle --network <NETWORK> [OPTIONS]
```

### Flags

| Flag | Purpose |
|---|---|
| `--network <NETWORK>` | mainnet / testnet / signet / regtest |
| `--template <TEMPLATE>` | bip44 / bip49 / bip84 / bip86 / wsh-multi / wsh-sortedmulti / sh-wsh-multi / sh-wsh-sortedmulti / tr-multi-a / tr-sortedmulti-a |
| `--descriptor <DESCRIPTOR>` | user-supplied BIP-388 descriptor; mutually exclusive with `--template` and `--descriptor-file` |
| `--descriptor-file <DESCRIPTOR_FILE>` | descriptor read from a single-line UTF-8 file; mutually exclusive with `--descriptor` |
| `--language <LANGUAGE>` | BIP-39 wordlist for the input phrase |
| `--passphrase <PASSPHRASE>` | BIP-39 mnemonic-extension passphrase |
| `--account <ACCOUNT>` | BIP-32 account index (default 0) |
| `--json` | emit JSON output |
| `--no-engraving-card` | suppress the stderr engraving-card layout |
| `--multisig-path-family <FAMILY>` | bip48 or bip87 (default bip87) |
| `--privacy-preserving` | suppress the master fingerprint from mk1 + engraving card |
| `--self-check` | re-parse and verify the emitted bundle round-trips |
| `--threshold <THRESHOLD>` | multisig K of N (1 ≤ K ≤ N ≤ 16) |
| `--slot <SLOT>` | repeating; `@N.<subkey>=<value>` (subkey: `phrase`, `entropy`, `xpub`, `fingerprint`, `path`, `wif`, `xprv`) |
| `--help` | print help |

### Worked example

See [Your first bundle](#your-first-bundle) for a single-sig
walkthrough; [Multi-source 2-of-3 multisig](#multi-source-2-of-3-multisig)
for multisig.

---

## `mnemonic verify-bundle`

Re-derive expected card content from a seed (or from a partial set
of cards) and report per-card pass/fail plus the overall verdict.

### Synopsis

```sh
mnemonic verify-bundle --network <NETWORK> [OPTIONS] [--ms1 ...] [--mk1 ...] [--md1 ...]
```

### Flags

| Flag | Purpose |
|---|---|
| `--network <NETWORK>` | mainnet / testnet / signet / regtest |
| `--template <TEMPLATE>` | as for `bundle` |
| `--descriptor <DESCRIPTOR>` | user-supplied BIP-388 descriptor |
| `--descriptor-file <DESCRIPTOR_FILE>` | descriptor read from file |
| `--threshold <THRESHOLD>` | multisig threshold |
| `--multisig-path-family <FAMILY>` | bip48 or bip87 |
| `--privacy-preserving` | match a privacy-preserving mk1 emission |
| `--language <LANGUAGE>` | BIP-39 wordlist |
| `--passphrase <PASSPHRASE>` | BIP-39 mnemonic passphrase |
| `--account <ACCOUNT>` | BIP-32 account index |
| `--slot <SLOT>` | repeating slot input |
| `--bundle-json <PATH>` | read the bundle from a JSON file emitted by `bundle --json` |
| `--ms1 <STRING>` | repeating; one ms1 card |
| `--mk1 <STRING>` | repeating; one mk1 card |
| `--md1 <STRING>` | repeating; one md1 card |
| `--json` | JSON output |
| `--help` | print help |

### Worked example

See [Verifying a bundle](#verifying-a-bundle).

---

## `mnemonic convert`

Single-format conversion across the 13-node typed graph: `phrase`,
`entropy`, `xpub`, `xprv`, `wif`, `fingerprint`, `path`, `ms1`, `mk1`,
`bip38`, `minikey`, `electrum-phrase`, `address`.

### Synopsis

```sh
mnemonic convert --from <NODE>=<value> --to <NODE> [--to <NODE>]... [OPTIONS]
```

### Flags

| Flag | Purpose |
|---|---|
| `--from <FROM>` | source node (`phrase=…`, `entropy=…`, `xpub=…`, `xprv=…`, `wif=…`, `ms1=…`, `mk1=…`, `bip38=…`, `minikey=…`, `electrum-phrase=…`); `=-` reads from stdin |
| `--to <TO>` | target node; repeating for multi-output |
| `--network <NETWORK>` | mainnet / testnet / signet / regtest |
| `--template <TEMPLATE>` | as for `bundle` |
| `--path <PATH>` | derivation path override |
| `--account <ACCOUNT>` | account index (default 0) |
| `--language <LANGUAGE>` | BIP-39 wordlist |
| `--passphrase <PASSPHRASE>` | BIP-39 passphrase |
| `--passphrase-stdin` | read `--passphrase` from stdin (raw, NULL-byte preserving); BIP-38 V3 use case |
| `--bip38-passphrase <BIP38_PASSPHRASE>` | distinct BIP-38 Scrypt passphrase channel (v0.8 BREAKING — separate from `--passphrase`) |
| `--electrum-version <ELECTRUM_VERSION>` | Electrum seed-version selector for `(Entropy, ElectrumPhrase)` |
| `--electrum-language <ELECTRUM_LANGUAGE>` | Electrum-specific wordlist (English + 4 non-English) |
| `--fingerprint <FINGERPRINT>` | master fingerprint (input on certain edges) |
| `--xpub-prefix <XPUB_PREFIX>` | SLIP-0132 prefix selector for emitted xpubs (e.g. zpub, ypub) |
| `--script-type <SCRIPT_TYPE>` | `p2wpkh` / `p2sh-p2wpkh` / `p2tr` for `(Xpub, Address)` derivation |
| `--json` | JSON output |
| `--help` | print help |

### Worked example

See [Minimal recovery walkthrough](#minimal-recovery-walkthrough)
and [Migrating from BIP-39-only to the m-format constellation](#migrating-from-bip-39-only-to-the-m-format constellation).

---

## `mnemonic export-wallet`

Emit watch-only wallet artifacts for Bitcoin Core, BIP-388, Coldcard,
Blockstream Jade, Sparrow, or Specter.

### Synopsis

```sh
mnemonic export-wallet [OPTIONS]
```

### Flags

| Flag | Purpose |
|---|---|
| `--template <TEMPLATE>` | as for `bundle` |
| `--descriptor <DESCRIPTOR>` | user-supplied BIP-388 descriptor |
| `--threshold <THRESHOLD>` | multisig threshold |
| `--multisig-path-family <FAMILY>` | bip48 or bip87 |
| `--network <NETWORK>` | default mainnet |
| `--language <LANGUAGE>` | ignored (watch-only); accepted for slot-parser symmetry |
| `--account <ACCOUNT>` | account index (default 0) |
| `--slot <SLOT>` | repeating `@N.<subkey>=<value>`; subkeys: `phrase`, `entropy`, `xpub`, `master_xpub`, `fingerprint`, `path`, `wif`, `xprv` (secret-bearing subkeys refused by `export-wallet`'s watch-only validator) |
| `--format <FORMAT>` | `bitcoin-core` (default) / `bip388` / `coldcard` / `jade` / `sparrow` / `specter` |
| `--output <OUTPUT>` | output path (`-` = stdout, default) |
| `--range <RANGE>` | Bitcoin Core `range` field; comma-separated; default `0,999` |
| `--timestamp <TIMESTAMP>` | Bitcoin Core `timestamp` field; `now` (default) or unix seconds |
| `--bitcoin-core-version <BITCOIN_CORE_VERSION>` | 24 or 25 (default 25) |
| `--wallet-name <WALLET_NAME>` | wallet name/label for formats that publish one (Coldcard generic JSON, Sparrow, Specter, Electrum); default `<template-human-name>-<account>` |
| `--taproot-internal-key <TAPROOT_INTERNAL_KEY>` | `nums` or `@N` for `tr-multi-a` / `tr-sortedmulti-a` |
| `--help` | print help |

### Worked example

See [Exporting to Bitcoin Core / BIP-388 / Sparrow / Specter](#exporting-to-bitcoin-core-bip-388-sparrow-specter).

---

## `mnemonic derive-child`

BIP-85 deterministic child entropy. Six in-scope applications:
`bip39`, `hd-seed`, `xprv`, `hex`, `password-base64`, `password-base85`,
plus `dice` (BIP-85 v1.3.0).

### Synopsis

```sh
mnemonic derive-child --from <FROM> --application <APP> --length <LEN> --index <INDEX> [OPTIONS]
```

### Flags

| Flag | Purpose |
|---|---|
| `--from <FROM>` | `xprv=<value>` or `phrase=<bip39>` (with `--passphrase` + `--language`); `=-` to read from stdin |
| `--application <APPLICATION>` | `bip39` / `hd-seed` / `xprv` / `hex` / `password-base64` / `password-base85` / `dice` |
| `--length <LENGTH>` | application-specific size; pass `0` for `hd-seed` and `xprv` |
| `--index <INDEX>` | hardened child index (`0..2^31`) |
| `--network <NETWORK>` | for `hd-seed` / `xprv` apps; defaults to mainnet |
| `--language <LANGUAGE>` | BIP-39 wordlist + BIP-85 language code for `--application bip39` |
| `--passphrase <PASSPHRASE>` | BIP-39 passphrase, only for `--from phrase=…` |
| `--dice-sides <DICE_SIDES>` | required for `--application dice`; range `2..=2^32-1` |
| `--help` | print help |

### Worked example

See [Deterministic child secrets via BIP-85](#deterministic-child-secrets-via-bip-85).
