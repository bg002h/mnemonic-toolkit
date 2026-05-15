# `mnemonic export-wallet` {#mnemonic-export-wallet}

Emit watch-only wallet artifacts from the bundle's public material
in a wallet-software-specific format. Eight target formats span
the major Bitcoin wallets — Bitcoin Core's `importdescriptors`
JSON, BIP-388 wallet policies for descriptor-aware tooling,
Coldcard generic JSON, Blockstream Jade, Sparrow, Specter, the
historical Electrum format, and Blockstream Green. Inputs are
slot-shaped (same `--slot @N.<subkey>=<value>` grammar as bundle
and verify-bundle); outputs are watch-only by design — secret-
bearing slot subkeys are refused by the watch-only validator.

The companion to bundle: where bundle EMITS the three engravable
cards, export-wallet TRANSLATES the bundle's public material into
whatever shape your spending wallet imports.

## Outline {#mnemonic-export-wallet-outline}

- [`--template`](#mnemonic-export-wallet-template) — pre-built descriptor template (mutually-required-one-of with `--descriptor`)
- [`--descriptor`](#mnemonic-export-wallet-descriptor) — user-supplied BIP-388 descriptor (XOR with `--template`)
- [`--threshold`](#mnemonic-export-wallet-threshold) — multisig threshold K (1 ≤ K ≤ N)
- [`--multisig-path-family`](#mnemonic-export-wallet-multisig-path-family) — `bip48` or `bip87` (default `bip87`)
- [`--network`](#mnemonic-export-wallet-network) — Bitcoin network (default `mainnet`)
- [`--language`](#mnemonic-export-wallet-language) — BIP-39 wordlist (ignored; kept for slot-parser symmetry)
- [`--account`](#mnemonic-export-wallet-account) — BIP-32 account index (default 0)
- [`--slot`](#mnemonic-export-wallet-slot) — repeating slot input (the public material)
- [`--format`](#mnemonic-export-wallet-format) — output format (default `bitcoin-core`)
- [`--output`](#mnemonic-export-wallet-output) — output path (`-` = stdout, default)
- [`--range`](#mnemonic-export-wallet-range) — Bitcoin Core `range` field (default `0,999`)
- [`--timestamp`](#mnemonic-export-wallet-timestamp) — Bitcoin Core `timestamp` field (default `now`)
- [`--bitcoin-core-version`](#mnemonic-export-wallet-bitcoin-core-version) — `24` or `25` (default `25`)
- [`--taproot-internal-key`](#mnemonic-export-wallet-taproot-internal-key) — `nums` or `@N` for tr-multi-a / tr-sortedmulti-a
- [`--wallet-name`](#mnemonic-export-wallet-wallet-name) — wallet label (required for `sparrow` / `specter` / `electrum` / `green`)

## `--template` {#mnemonic-export-wallet-template}

The descriptor template the bundle was emitted under. Mutually-
required-one-of with `--descriptor` (the conditional-visibility
engine marks both as `Required` when neither is set, and
`Disabled` for the other when one has a value). Same 10 values
as [`mnemonic bundle --template`](#mnemonic-bundle-template).

### Outline {#mnemonic-export-wallet-template-outline}

- [`bip44`](#mnemonic-export-wallet-template-bip44)
- [`bip49`](#mnemonic-export-wallet-template-bip49)
- [`bip84`](#mnemonic-export-wallet-template-bip84)
- [`bip86`](#mnemonic-export-wallet-template-bip86)
- [`wsh-multi`](#mnemonic-export-wallet-template-wsh-multi)
- [`wsh-sortedmulti`](#mnemonic-export-wallet-template-wsh-sortedmulti)
- [`sh-wsh-multi`](#mnemonic-export-wallet-template-sh-wsh-multi)
- [`sh-wsh-sortedmulti`](#mnemonic-export-wallet-template-sh-wsh-sortedmulti)
- [`tr-multi-a`](#mnemonic-export-wallet-template-tr-multi-a)
- [`tr-sortedmulti-a`](#mnemonic-export-wallet-template-tr-sortedmulti-a)

### `bip44` {#mnemonic-export-wallet-template-bip44}

See [`mnemonic bundle --template bip44`](#mnemonic-bundle-template-bip44).

### `bip49` {#mnemonic-export-wallet-template-bip49}

See [`mnemonic bundle --template bip49`](#mnemonic-bundle-template-bip49).

### `bip84` {#mnemonic-export-wallet-template-bip84}

See [`mnemonic bundle --template bip84`](#mnemonic-bundle-template-bip84).

### `bip86` {#mnemonic-export-wallet-template-bip86}

See [`mnemonic bundle --template bip86`](#mnemonic-bundle-template-bip86).

### `wsh-multi` {#mnemonic-export-wallet-template-wsh-multi}

See [`mnemonic bundle --template wsh-multi`](#mnemonic-bundle-template-wsh-multi).

### `wsh-sortedmulti` {#mnemonic-export-wallet-template-wsh-sortedmulti}

See [`mnemonic bundle --template wsh-sortedmulti`](#mnemonic-bundle-template-wsh-sortedmulti).

### `sh-wsh-multi` {#mnemonic-export-wallet-template-sh-wsh-multi}

See [`mnemonic bundle --template sh-wsh-multi`](#mnemonic-bundle-template-sh-wsh-multi).

### `sh-wsh-sortedmulti` {#mnemonic-export-wallet-template-sh-wsh-sortedmulti}

See [`mnemonic bundle --template sh-wsh-sortedmulti`](#mnemonic-bundle-template-sh-wsh-sortedmulti).

### `tr-multi-a` {#mnemonic-export-wallet-template-tr-multi-a}

See [`mnemonic bundle --template tr-multi-a`](#mnemonic-bundle-template-tr-multi-a).

### `tr-sortedmulti-a` {#mnemonic-export-wallet-template-tr-sortedmulti-a}

See [`mnemonic bundle --template tr-sortedmulti-a`](#mnemonic-bundle-template-tr-sortedmulti-a).

## `--descriptor` {#mnemonic-export-wallet-descriptor}

User-supplied BIP-388 descriptor. Mutually-required-one-of with
`--template`. The conditional-visibility engine (per
`mnemonic-gui/src/form/conditional::export_wallet`) implements
both the clap `conflicts_with` and the runtime pre-check
(`crates/mnemonic-toolkit/src/cmd/export_wallet.rs:215-219`):
when neither flag is set, both render with `Visibility::Required`;
when one has a value, the other is `Disabled`.

## `--threshold` {#mnemonic-export-wallet-threshold}

Multisig threshold K. Same range (1 to 16 inclusive) and same
single-sig refusal as [`mnemonic bundle --threshold`](#mnemonic-bundle-threshold).
**Required for `--format sparrow` with multisig templates** (per
the chapter's notes section in the CLI manual at
`docs/manual/src/40-cli-reference/41-mnemonic.md:172`):
Bitcoin Core / BIP-388 / Coldcard / Jade auto-default `K = N`,
but Sparrow refuses with a missing-info error.

## `--multisig-path-family` {#mnemonic-export-wallet-multisig-path-family}

`bip48` or `bip87`. Same semantics as
[`mnemonic bundle --multisig-path-family`](#mnemonic-bundle-multisig-path-family).
Default `bip87`.

### Outline {#mnemonic-export-wallet-multisig-path-family-outline}

- [`bip48`](#mnemonic-export-wallet-multisig-path-family-bip48)
- [`bip87`](#mnemonic-export-wallet-multisig-path-family-bip87)

### `bip48` {#mnemonic-export-wallet-multisig-path-family-bip48}

See [`mnemonic bundle --multisig-path-family bip48`](#mnemonic-bundle-multisig-path-family-bip48).

### `bip87` {#mnemonic-export-wallet-multisig-path-family-bip87}

See [`mnemonic bundle --multisig-path-family bip87`](#mnemonic-bundle-multisig-path-family-bip87).

## `--network` {#mnemonic-export-wallet-network}

Same 4 values + descriptions as
[`mnemonic bundle --network`](#mnemonic-bundle-network). Default
`mainnet`.

### Outline {#mnemonic-export-wallet-network-outline}

- [`mainnet`](#mnemonic-export-wallet-network-mainnet)
- [`testnet`](#mnemonic-export-wallet-network-testnet)
- [`signet`](#mnemonic-export-wallet-network-signet)
- [`regtest`](#mnemonic-export-wallet-network-regtest)

### `mainnet` {#mnemonic-export-wallet-network-mainnet}

See [`mnemonic bundle --network mainnet`](#mnemonic-bundle-network-mainnet).

### `testnet` {#mnemonic-export-wallet-network-testnet}

See [`mnemonic bundle --network testnet`](#mnemonic-bundle-network-testnet).

### `signet` {#mnemonic-export-wallet-network-signet}

See [`mnemonic bundle --network signet`](#mnemonic-bundle-network-signet).

### `regtest` {#mnemonic-export-wallet-network-regtest}

See [`mnemonic bundle --network regtest`](#mnemonic-bundle-network-regtest).

## `--language` {#mnemonic-export-wallet-language}

BIP-39 wordlist. **Ignored** by export-wallet (watch-only outputs
do not need a phrase wordlist) but kept in the schema for slot-
parser symmetry — the slot editor's `phrase=` subkey would still
need a language to validate against, even though such slot rows
are refused by the watch-only validator. Same 10 values as
[`mnemonic bundle --language`](#mnemonic-bundle-language).

### Outline {#mnemonic-export-wallet-language-outline}

- [`english`](#mnemonic-export-wallet-language-english)
- [`simplifiedchinese`](#mnemonic-export-wallet-language-simplifiedchinese)
- [`traditionalchinese`](#mnemonic-export-wallet-language-traditionalchinese)
- [`czech`](#mnemonic-export-wallet-language-czech)
- [`french`](#mnemonic-export-wallet-language-french)
- [`italian`](#mnemonic-export-wallet-language-italian)
- [`japanese`](#mnemonic-export-wallet-language-japanese)
- [`korean`](#mnemonic-export-wallet-language-korean)
- [`portuguese`](#mnemonic-export-wallet-language-portuguese)
- [`spanish`](#mnemonic-export-wallet-language-spanish)

### `english` {#mnemonic-export-wallet-language-english}

See [`mnemonic bundle --language english`](#mnemonic-bundle-language-english).

### `simplifiedchinese` {#mnemonic-export-wallet-language-simplifiedchinese}

See [`mnemonic bundle --language simplifiedchinese`](#mnemonic-bundle-language-simplifiedchinese).

### `traditionalchinese` {#mnemonic-export-wallet-language-traditionalchinese}

See [`mnemonic bundle --language traditionalchinese`](#mnemonic-bundle-language-traditionalchinese).

### `czech` {#mnemonic-export-wallet-language-czech}

See [`mnemonic bundle --language czech`](#mnemonic-bundle-language-czech).

### `french` {#mnemonic-export-wallet-language-french}

See [`mnemonic bundle --language french`](#mnemonic-bundle-language-french).

### `italian` {#mnemonic-export-wallet-language-italian}

See [`mnemonic bundle --language italian`](#mnemonic-bundle-language-italian).

### `japanese` {#mnemonic-export-wallet-language-japanese}

See [`mnemonic bundle --language japanese`](#mnemonic-bundle-language-japanese).

### `korean` {#mnemonic-export-wallet-language-korean}

See [`mnemonic bundle --language korean`](#mnemonic-bundle-language-korean).

### `portuguese` {#mnemonic-export-wallet-language-portuguese}

See [`mnemonic bundle --language portuguese`](#mnemonic-bundle-language-portuguese).

### `spanish` {#mnemonic-export-wallet-language-spanish}

See [`mnemonic bundle --language spanish`](#mnemonic-bundle-language-spanish).

## `--account` {#mnemonic-export-wallet-account}

BIP-32 account index. Default 0; range 0..2_147_483_647. Number
widget; no `?` help-icon.

## `--slot` {#mnemonic-export-wallet-slot}

The repeating slot-input flag. Same grammar as
[`mnemonic bundle --slot`](#mnemonic-bundle-slot). For
export-wallet, secret-bearing slot subkeys (`phrase`, `entropy`,
`wif`, `xprv`) are **refused** by the watch-only validator —
export-wallet only emits public artifacts, so passing a master
phrase would be inconsistent with the subcommand's purpose.

The slot editor renders identically; rows with secret subkeys
will be refused at run time with a watch-only-validator error.
The `master_xpub` subkey is the export-wallet-shaped optional
add-on documented at
[`mnemonic bundle --slot`](#mnemonic-bundle-slot).

## `--format` {#mnemonic-export-wallet-format}

The output format. Default `bitcoin-core`. Eight allowed values
covering the major spending-wallet ecosystems.

### Outline {#mnemonic-export-wallet-format-outline}

- [`bitcoin-core`](#mnemonic-export-wallet-format-bitcoin-core)
- [`bip388`](#mnemonic-export-wallet-format-bip388)
- [`coldcard`](#mnemonic-export-wallet-format-coldcard)
- [`jade`](#mnemonic-export-wallet-format-jade)
- [`sparrow`](#mnemonic-export-wallet-format-sparrow)
- [`specter`](#mnemonic-export-wallet-format-specter)
- [`electrum`](#mnemonic-export-wallet-format-electrum)
- [`green`](#mnemonic-export-wallet-format-green)

### `bitcoin-core` {#mnemonic-export-wallet-format-bitcoin-core}

Bitcoin Core `importdescriptors` JSON. Default. Includes the
`range`, `timestamp`, and `internal` fields per the
`importdescriptors` RPC schema. Pair with
`--bitcoin-core-version` to target Core 24 or 25 (the latter is
default; the difference is in the `active` / `next_index`
optional fields).

### `bip388` {#mnemonic-export-wallet-format-bip388}

Bare BIP-388 wallet-policy JSON without the Bitcoin-Core-specific
wrapper. Use for descriptor-aware tooling that consumes the
upstream BIP-388 standard format directly.

### `coldcard` {#mnemonic-export-wallet-format-coldcard}

Coldcard generic JSON (single-sig) or text (multisig). Multisig
text caps `Name:` at 20 Unicode scalar values per the Coldcard
reference format; longer `--wallet-name` values are truncated to
the first 20 codepoints (not bytes — non-ASCII names handled at
codepoint granularity per the chapter notes in
`docs/manual/src/40-cli-reference/41-mnemonic.md:170`).

### `jade` {#mnemonic-export-wallet-format-jade}

Blockstream Jade format (byte-identical to the Coldcard multisig
text for multisig templates; distinct JSON shape for single-sig).
Same `--wallet-name` 20-codepoint cap as Coldcard.

### `sparrow` {#mnemonic-export-wallet-format-sparrow}

Sparrow's wallet JSON. **`--threshold` is required for multisig
templates** under this format (Bitcoin Core / BIP-388 / Coldcard
/ Jade auto-default `K = N`; Sparrow refuses with a missing-info
error per `docs/manual/src/40-cli-reference/41-mnemonic.md:172`).
Sparrow publishes the threshold in
`defaultPolicy.miniscript.script` as `multi(K, ...)` /
`sortedmulti(K, ...)`.

### `specter` {#mnemonic-export-wallet-format-specter}

Specter Desktop format. **`--wallet-name` is required** —
Specter refuses an empty wallet label via the SPEC §4
missing-info channel (other formats fall back to
`<template-human-name>-<account>` when the name is omitted).

### `electrum` {#mnemonic-export-wallet-format-electrum}

Electrum's historical JSON wallet format. Use for importing into
Electrum 4.x watch-only wallets.

### `green` {#mnemonic-export-wallet-format-green}

Blockstream Green format. v0.13.0 ships this as a Phase-3 promoted
format (no longer a stub).

## `--output` {#mnemonic-export-wallet-output}

Output destination path. Default `-` (stdout). Pass a filesystem
path to write the wallet artifact to disk; this is also the
recommended pattern for non-text formats that benefit from
explicit `.json` filenames.

The GUI renders this as a Path widget. The schema sets
`stdio_sentinel: true` for this flag — the GUI accepts `-` as a
valid value to mean stdout.

## `--range` {#mnemonic-export-wallet-range}

Bitcoin Core `range` field as a comma-separated `start,end` pair.
Default `0,999`. Used only for `--format bitcoin-core` (other
formats ignore this flag).

The GUI renders this as a Range widget (a single text field
with parsing for the `start,end` shape). Refusal: `--range start
must be <= end` (per `cmd/export_wallet.rs::parse_range`).

## `--timestamp` {#mnemonic-export-wallet-timestamp}

Bitcoin Core `timestamp` field. Two valid forms: `now` (the
default; emits the literal string `"now"` in the JSON, which
Core interprets at import time as the current block timestamp)
or a non-negative integer Unix-seconds value. Used only for
`--format bitcoin-core`.

The GUI renders this as a Timestamp widget. Refusal: `--timestamp
unix seconds must be >= 0` (per `cmd/export_wallet.rs`).

## `--bitcoin-core-version` {#mnemonic-export-wallet-bitcoin-core-version}

Bitcoin Core target version. Number widget; range 24..25
inclusive; default 25. Different Core versions accept slightly
different `importdescriptors` field shapes; this flag picks
which to emit.

## `--taproot-internal-key` {#mnemonic-export-wallet-taproot-internal-key}

The Taproot internal-key designation for `tr-multi-a` /
`tr-sortedmulti-a` templates. Two valid forms: the literal
`nums` (a BIP-341 NUMS-point unspendable internal key) or
`@N` to designate cosigner N's xpub as the internal key.

The GUI renders this as a TaggedOrIndexed widget — a Dropdown
with the `nums` tag plus a Number spinner for the `@N` form.
The `?` help-icon deep-links here per
[§33 placement](#help-icons-and-deep-links-into-this-manual)
(Class-2 enumerated-flag affordance).

**Required for tr-multi-a and tr-sortedmulti-a** (per
`cmd/export_wallet.rs::BadInput`):
`--template <T> requires --taproot-internal-key (use 'nums' for
an unspendable BIP-341 NUMS point, or '@N' to designate cosigner
N as the key-path internal key)`. Refused for non-Taproot
templates: `--taproot-internal-key applies only to --template
tr-multi-a / tr-sortedmulti-a`.

### `nums` {#mnemonic-export-wallet-taproot-internal-key-nums}

The enumerated-tag form. Designates a BIP-341 NUMS-point
unspendable internal key as the Taproot internal key. The
`@N` form (cosigner-N designation) is handled by the same flag
but goes through the `Indexed` half of the `TaggedOrIndexed`
widget — the GUI's Number spinner accompanies the dropdown.

## `--wallet-name` {#mnemonic-export-wallet-wallet-name}

Wallet label. Optional for most formats (defaults to
`<template-human-name>-<account>`); **required** for
`--format sparrow`, `specter`, `electrum`, and `green`.
20-codepoint cap for `coldcard` + `jade` multisig text formats
(silently truncates longer names).

## Worked example — Bitcoin Core watch-only from canonical bundle

1. **mnemonic** tab; pick **Export Wallet (watch-only)**.
2. Set `--template` to `bip84`, `--network` to `mainnet`.
3. Clear `--multisig-path-family` (single-sig template; same
   FOLLOWUP as `bundle`).
4. In the slot editor, change `@0.xpub` to the canonical mainnet
   BIP-84 m/84'/0'/0' xpub:

   ```text
   xpub6CatWdiZiodmU...VMrjPC7PW6V
   ```

   (extract from `mk decode <canonical mk1>` if you don't have
   it on hand). The slot is `xpub` (public; not refused).
5. Leave `--format` at `bitcoin-core` and `--output` at `-`.
6. Click **Run**. No run-confirm modal (no secret-class flag
   value). The output panel renders the
   `importdescriptors` JSON on stdout:

   ```json
   [
     {
       "desc": "wpkh([73c5da0a/84h/0h/0h]xpub6CatWdiZi.../<0;1>/*)#checksum",
       "active": true,
       "internal": false,
       "range": [0, 999],
       "timestamp": "now",
       "next_index": 0
     }
   ]
   ```

   Pipe this into `bitcoin-cli importdescriptors` to register the
   watch-only wallet on a running node.

## Refusals

| Trigger | Refusal |
|---|---|
| neither `--template` nor `--descriptor` | `export-wallet requires either --template or --descriptor` (per `cmd/export_wallet.rs:215-219`) |
| `--template` AND `--descriptor` | clap-level `conflicts_with` |
| Taproot-multisig template without `--taproot-internal-key` | `--template <T> requires --taproot-internal-key (use 'nums' for an unspendable BIP-341 NUMS point, or '@N' to designate cosigner N as the key-path internal key)` |
| `--taproot-internal-key` with non-Taproot template | `--taproot-internal-key applies only to --template tr-multi-a / tr-sortedmulti-a` |
| `--range start > end` | `--range start <START> must be <= end <END>` |
| `--timestamp <negative integer>` | `--timestamp unix seconds must be >= 0; got <N>` |
| Slot row with secret-bearing subkey (`phrase` / `entropy` / `wif` / `xprv`) | watch-only-validator refusal: secret material rejected |
| `--format sparrow` with multisig template, no `--threshold` | Sparrow-specific missing-info refusal |
| `--format specter` without `--wallet-name` | Specter-specific missing-info refusal |

## Advisories

The watch-only nature of export-wallet means there are no
argv-leakage advisories — no flag carries secret material. The
secret-bearing-slot-rejection refusals above prevent secret input
from reaching the assembled argv.
