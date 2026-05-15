# `md verify` {#md-verify}

Round-trip verification: assert that one or more `md1` strings
re-encode (with `--key` / `--fingerprint` placeholders bound) to
the supplied `--template`. Emits per-string pass/fail.

## Outline {#md-verify-outline}

- [`--template`](#md-verify-template) — BIP-388 template (required)
- [`--key`](#md-verify-key) — concrete xpub for placeholder `@i` (repeating)
- [`--fingerprint`](#md-verify-fingerprint) — master fingerprint for `@i` (repeating)
- [`--network`](#md-verify-network) — network for xpub validation (default `mainnet`)

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
