# `mnemonic nostr` {#mnemonic-nostr}

Wrap an existing nostr key (`npub`/`nsec`, NIP-19 bech32 or 64-hex) as
Bitcoin addresses, descriptors, and (for `nsec`) a WIF. Taproot
(`p2tr`) is the default and the native x-only mapping for nostr keys —
the x-only pubkey is used directly as the taproot internal key,
yielding a key-path-only P2TR output. Non-taproot script types use the
BIP-340 even-y `02‖x` compressed form of the x-only pubkey. The GUI
exposes this as one form under the **mnemonic** tab's subcommand
selector.
\index{mnemonic nostr}

For `nsec` inputs, the secret is **normalized to even-y** (BIP-340):
if `d·G` has odd y, the toolkit uses `n−d` so the emitted WIF controls
the emitted address; a `notice:` is printed on stderr when the
normalization negates the key.

:::danger
The worked examples use public NIP-19 test keys (no funds). When you
supply an `nsec` via `--secret`, the GUI renders it as a masked
`SecretLineEdit` and the run-confirm modal redacts secret-bearing
argv tokens to a fixed `••••` sentinel (see
[§14 Defense 2](#secret-handling)). WIF output is private-key material
— the toolkit emits the `warning: stdout carries private key material`
advisory. **Never engrave or fund** a wallet from a published-key
example.
:::

## Outline {#mnemonic-nostr-outline}

- [`--all-script-types`](#mnemonic-nostr-all-script-types) — emit all four script types
- [`--import`](#mnemonic-nostr-import) — append a Bitcoin Core `importdescriptors` recipe
- [`--json`](#mnemonic-nostr-json) — emit JSON instead of the human-readable block
- [`--network`](#mnemonic-nostr-network) — network selector (default `mainnet`)
- [`--pubkey`](#mnemonic-nostr-pubkey) — public key (`npub1…` or 64-hex x-only); watch-only
- [`--script-type`](#mnemonic-nostr-script-type) — address/descriptor script type (default `p2tr`)
- [`--secret`](#mnemonic-nostr-secret) — secret key (`nsec1…` or 64-hex scalar)
- [`--secret-file`](#mnemonic-nostr-secret-file) — read the secret from a file
- [`--secret-stdin`](#mnemonic-nostr-secret-stdin) — read the secret from stdin
- [`--timestamp`](#mnemonic-nostr-timestamp) — Bitcoin Core rescan anchor for `--import`

## `--all-script-types` {#mnemonic-nostr-all-script-types}

Boolean flag. Emit a descriptor + address for all four script types
(`p2tr`, `p2wpkh`, `p2sh-p2wpkh`, `p2pkh`). Mutually exclusive with a
single `--script-type` choice. The GUI renders this as a checkbox.

## `--import` {#mnemonic-nostr-import}

Append a ready-to-paste Bitcoin Core `importdescriptors` recipe for
the derived address(es). `readonly` = watch-only (the pubkey
descriptor; `active: false`, `internal: false`). `spending` / `both`
are reserved for a future cycle (rejected with a "deferred" message).
The GUI renders this as a Text widget.

## `--json` {#mnemonic-nostr-json}

Boolean flag. Emit a single JSON object on stdout instead of the
human-readable block. For `nsec` inputs the object additionally
carries `"wif"` at the top level and each non-taproot `outputs` entry
includes `"electrum"`. The GUI renders this as a checkbox.

## `--network` {#mnemonic-nostr-network}

The Bitcoin network selector — affects the address HRP and WIF version
byte. Default `mainnet`. The GUI renders this as a Dropdown widget.

### Outline {#mnemonic-nostr-network-outline}

- [`mainnet`](#mnemonic-nostr-network-mainnet)
- [`testnet`](#mnemonic-nostr-network-testnet)
- [`signet`](#mnemonic-nostr-network-signet)
- [`regtest`](#mnemonic-nostr-network-regtest)

### `mainnet` {#mnemonic-nostr-network-mainnet}

Production Bitcoin mainnet. Address HRP `bc1`; WIF version `0x80`.

### `testnet` {#mnemonic-nostr-network-testnet}

The legacy public test network. Address HRP `tb1`. Funds are
valueless.

### `signet` {#mnemonic-nostr-network-signet}

The signature-secured test network. Address HRP `tb1`. Funds are
valueless.

### `regtest` {#mnemonic-nostr-network-regtest}

A locally-controlled regression-test network. Address HRP `bcrt1`.

## `--pubkey` {#mnemonic-nostr-pubkey}

The public key: `npub1…` (NIP-19 bech32) or 64-hex x-only. Emits
watch-only outputs (no WIF). Exactly one of `--pubkey` / `--secret` /
`--secret-file` / `--secret-stdin` is required (clap arg-group;
missing/multiple → exit 64). The pubkey is public material. The GUI
renders this as a Text widget.

## `--script-type` {#mnemonic-nostr-script-type}

The address/descriptor script type: `p2pkh` / `p2wpkh` /
`p2sh-p2wpkh` / `p2tr`. Defaults to `p2tr` when neither this nor
`--all-script-types` is given. The GUI renders this as a Text widget
(free-form, not a fixed dropdown enum at the schema level).

## `--secret` {#mnemonic-nostr-secret}

The secret key: `nsec1…` (NIP-19 bech32) or 64-hex scalar. Adds a
WIF + `electrum:` line (non-taproot script types only). Inline use emits
the argv-leakage advisory; prefer `--secret-stdin` / `--secret-file`.
One of the required key-source group. The GUI renders this as a masked
`SecretLineEdit`.

## `--secret-file` {#mnemonic-nostr-secret-file}

Read the secret key from a file (avoids argv exposure). One of the
required key-source group. The GUI renders this as a Path widget.

## `--secret-stdin` {#mnemonic-nostr-secret-stdin}

Boolean flag. Read the secret key from stdin (avoids argv exposure).
One of the required key-source group. The GUI renders this as a
checkbox.

## `--timestamp` {#mnemonic-nostr-timestamp}

The Bitcoin Core rescan anchor for `--import`: `now` or unix seconds.
Default `0` (rescan from genesis to discover an existing key's funds).
The GUI renders this as a Text widget.

## Worked example — `npub` (watch-only, default `p2tr`)

1. Switch to the **mnemonic** tab; pick **nostr** in the subcommand
   selector.
2. Paste an `npub1…` key into `--pubkey`.
3. Leave `--script-type` empty (defaults to `p2tr`).
4. Click **Run** (no run-confirm modal — an `npub` is public).

The output block renders the x-only key, script type, descriptor, and
address. With `--all-script-types`, four output rows are emitted;
`p2tr` uses the bare x-only key while the others use the BIP-340
even-y `02‖x` compressed form. Taproot emits no `electrum:` line —
Electrum has no taproot private-key import path.
