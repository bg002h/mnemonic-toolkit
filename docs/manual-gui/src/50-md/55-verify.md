# `md verify` {#md-verify}

Round-trip verification: assert that one or more `md1` strings
re-encode (with `--key` / `--fingerprint` placeholders bound) to
the supplied `--template`. Emits per-string pass/fail.

> **GUI form:** see [GUI Forms › md › verify](#gui-form-md-verify).

## Outline {#md-verify-outline}

- [`--template`](#md-verify-template) — BIP-388 template (required)
- [`--key`](#md-verify-key) — concrete xpub for placeholder `@i` (repeating)
- [`--fingerprint`](#md-verify-fingerprint) — master fingerprint for `@i` (repeating)
- [`--network`](#md-verify-network) — network for xpub validation (default `mainnet`)
- [`--in`](#md-verify-in) — read the `md1` strings from a file, one per line
- [`--path`](#md-verify-path) — override the inferred origin path with one shared path
- [`--experimental`](#md-verify-experimental) — accept a template with a signature-free spend path

## `--template` {#md-verify-template}

The BIP-388 wallet-policy template the `md1` strings are expected
to re-encode to. Required at the clap level. Plain Text widget.

## `--key` {#md-verify-key}

Concrete xpub for placeholder `@i` substitution. Format
`@<index>=<xpub>`. Repeating. The GUI renders this as a multi-row
text widget; one row per `@i=xpub` binding.

## `--fingerprint` {#md-verify-fingerprint}

Master-key fingerprint for placeholder `@i`. Format
`@<index>=<8-hex-chars>`. Repeating. Pairs with `--key` rows
to provide complete origin metadata for verification.

## `--network` {#md-verify-network}

Bitcoin network for xpub validation. Default `mainnet`. Same 4
values as [`mnemonic bundle --network`](#mnemonic-bundle-network).

### Outline {#md-verify-network-outline}

- [`mainnet`](#md-verify-network-mainnet)
- [`testnet`](#md-verify-network-testnet)
- [`signet`](#md-verify-network-signet)
- [`regtest`](#md-verify-network-regtest)

### `mainnet` {#md-verify-network-mainnet}

See [`mnemonic bundle --network mainnet`](#mnemonic-bundle-network-mainnet).

### `testnet` {#md-verify-network-testnet}

See [`mnemonic bundle --network testnet`](#mnemonic-bundle-network-testnet).

### `signet` {#md-verify-network-signet}

See [`mnemonic bundle --network signet`](#mnemonic-bundle-network-signet).

### `regtest` {#md-verify-network-regtest}

See [`mnemonic bundle --network regtest`](#mnemonic-bundle-network-regtest).

## `--in` {#md-verify-in}

Path widget. Read the `md1` strings from FILE, one per line, instead of
the positional (P3 §6b). The GUI treats it as an alternative to the
positional, which is no longer marked required.

## `--path` {#md-verify-path}

Text widget. Override the inferred origin path with a single shared
path (flattening Divergent mode to Shared). Accepts the named
(`bip44`/`48`/`49`/`84`/`86`), hex (`0xNN`) and literal (`m/…`) forms,
as [`md encode --path`](#md-encode-path) does. Without it, the
non-canonical wrappers this flag exists for are unreachable here: they
refuse with "non-canonical wrapper requires explicit origin for @N".

## `--experimental` {#md-verify-experimental}

Boolean. Accept a template with a spend path that requires no
signature, mirroring [`md encode --experimental`](#md-encode-experimental).
Without it, a card authored with `--experimental` cannot be verified at
all.

## Positional `strings`

One or more `md1` strings to verify. Required, repeating.

## Worked example

1. **md** tab; pick **Verify (md1 ↔ template)**.
2. `--template`: paste the expected template (e.g.
   `wpkh(@0/<0;1>/*)` for the canonical BIP-84 bundle).
3. Add one `--key` row: `@0=xpub6CatWdiZi...VMrjPC7PW6V`.
4. Optionally add `--fingerprint` row: `@0=73c5da0a`.
5. Paste the canonical 3 md1 strings into the `strings`
   positional.
6. **Run**.

The output panel reports per-string `pass`/`fail` with a final
`verdict: pass` or `verdict: fail` line.

## Refusals

| Trigger | Refusal |
|---|---|
| Missing `--template` | clap-level `required` error |
| Missing positional `strings` | clap-level `required` error |
| `--key` value not parseable as `@i=xpub` | md-cli format error |
| `--fingerprint` value not 8 hex chars | md-cli format error |
| Network mismatch between `--network` and the xpub prefix | md-cli mismatch error |
