# `md address` {#md-address}

Derive Bitcoin addresses from an `md1` card (or from a BIP-388
template + cosigner xpubs). The end-of-pipeline subcommand for
the md tab: where `inspect` and `decode` show what an `md1`
encodes, `address` produces the actual on-chain artifacts the
wallet receives funds at.

Two-mode input: either the **`[PHRASES]` positional** (one or
more `md1` strings) OR **`--template` + `--key`/`--fingerprint`
bindings**. Mutually exclusive, mutually-required-one-of (the
conditional-visibility engine at
`mnemonic-gui/src/form/conditional::md_address` enforces the
constraint: positional set → `--template`/`--key`/`--fingerprint`
all Disabled; neither set → `--template` Required; positional
Required is widget-layer).

> **GUI form:** see [GUI Forms › md › address](#gui-form-md-address).

## Outline {#md-address-outline}

- [`--template`](#md-address-template) — BIP-388 template (XOR with `[PHRASES]` positional)
- [`--key`](#md-address-key) — concrete xpub for placeholder `@i` (repeating; requires `--template`)
- [`--fingerprint`](#md-address-fingerprint) — master fingerprint for `@i` (repeating; requires `--template`)
- [`--network`](#md-address-network) — network for xpub validation + address rendering (default `mainnet`)
- [`--chain`](#md-address-chain) — multipath alternative selector (0 = receive, 1 = change)
- [`--change`](#md-address-change) — sugar for `--chain 1`
- [`--index`](#md-address-index) — starting index along the wildcard (default 0)
- [`--count`](#md-address-count) — number of consecutive addresses to derive (default 1)
- [`--json`](#md-address-json) — emit JSON output

## `--template` {#md-address-template}

A BIP-388 wallet-policy template. Plain Text widget. Mutually
exclusive with the `[PHRASES]` positional. Requires at least one
`--key` to bind concrete xpubs to the template's `@i`
placeholders.

The conditional-visibility engine marks this flag as Disabled
when the `[PHRASES]` positional has a value, and as Required
when neither input mode has been chosen.

## `--key` {#md-address-key}

Concrete xpub for placeholder `@i`. Format `@<index>=<xpub>`.
Repeating. **Requires `--template`** (the engine disables this
flag when the positional path is chosen). For a 2-of-3 multisig
template, supply 3 `--key` rows.

## `--fingerprint` {#md-address-fingerprint}

Master-key fingerprint for placeholder `@i`. Format
`@<index>=<8-hex-chars>`. Repeating. **Requires `--template`**
(disabled when positional path chosen). Pairs with `--key` rows
to provide complete origin metadata.

## `--network` {#md-address-network}

Bitcoin network for xpub validation AND for address-prefix
rendering. Default `mainnet`. Same 4 values as
[`mnemonic bundle --network`](#mnemonic-bundle-network).

### Outline {#md-address-network-outline}

- [`mainnet`](#md-address-network-mainnet)
- [`testnet`](#md-address-network-testnet)
- [`signet`](#md-address-network-signet)
- [`regtest`](#md-address-network-regtest)

### `mainnet` {#md-address-network-mainnet}

See [`mnemonic bundle --network mainnet`](#mnemonic-bundle-network-mainnet).

### `testnet` {#md-address-network-testnet}

See [`mnemonic bundle --network testnet`](#mnemonic-bundle-network-testnet).

### `signet` {#md-address-network-signet}

See [`mnemonic bundle --network signet`](#mnemonic-bundle-network-signet).

### `regtest` {#md-address-network-regtest}

See [`mnemonic bundle --network regtest`](#mnemonic-bundle-network-regtest).

## `--chain` {#md-address-chain}

Multipath alternative selector. Number widget; range 0..65_535.
Default 0 (the receive chain for canonical `<0;1>/*` multipath
descriptors). Set `--chain 1` to derive from the change chain.

For non-canonical multipath shapes (e.g. `<0;1;2>/*`), the
selector picks among the listed alternatives by index.

## `--change` {#md-address-change}

Boolean. Sugar for `--chain 1`. The clap-level conflict between
`--chain` and `--change` is **not confirmed** from the upstream
help text per the schema's `mnemonic-gui/src/form/conditional.rs`
note; the conditional-visibility engine leaves the pair as
Visible pending md-cli source audit. Setting both has
implementation-defined behavior (typically the latter wins, per
clap's default override semantics).

## `--index` {#md-address-index}

Starting index along the wildcard (`*` placeholder in the
descriptor's multipath segment). Number widget; range
0..2_147_483_647. Default 0.

For a wallet-policy template like `wpkh(@0/<0;1>/*)`, setting
`--index 5` (with `--chain 0`) derives addresses starting at
the 6th receive address (index counts from 0).

## `--count` {#md-address-count}

Number of consecutive addresses to derive starting at `--index`.
Number widget; range 1..10_000. Default 1.

Use `--count 20` to derive a batch of 20 sequential addresses
for a gap-limit warm-up workflow.

## `--json` {#md-address-json}

Boolean. Emit JSON output instead of plain-text address-per-line.
Default off.

## Positional `[PHRASES]`

One or more `md1` strings. Repeating. **Optional** at the clap
level. Mutually exclusive with `--template`. The positional
form is the more common end-user invocation: paste the engraved
`md1` strings, get addresses.

## Worked example — derive 5 addresses from canonical bundle

1. **md** tab; pick **Address (derive from md1)**.
2. Paste the canonical 3 `md1` strings into the `phrases`
   positional, separated by spaces.
3. `--network`: leave at default (`mainnet`).
4. `--chain`: leave at default (0 = receive).
5. `--index`: leave at default (0).
6. `--count`: set to `5`.
7. Click **Run**. No run-confirm modal (no secret-class flag).

The output panel renders 5 receive-chain addresses on stdout,
one per line. For the canonical all-`abandon` BIP-84 bundle on
mainnet, the first address is the well-known
`bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu`.

## Worked example — derive from template + xpub (no md1)

1. **md** tab; pick **Address**.
2. Leave the `[PHRASES]` positional empty.
3. `--template`: paste `wpkh(@0/<0;1>/*)`.
4. Add `--key` row: `@0=xpub6CatWdiZi...VMrjPC7PW6V`.
5. Add `--fingerprint` row: `@0=73c5da0a`.
6. Set `--count` to `3`.
7. **Run**.

Output: 3 addresses on stdout, identical to the first 3 from the
md1-positional invocation above (the template + xpub form
re-derives the same descriptor that the md1 cards encode).

## Refusals

| Trigger | Refusal |
|---|---|
| Neither `[PHRASES]` positional nor `--template` set | runtime pre-check: `md address requires either [PHRASES] positional or --template` |
| Both `[PHRASES]` positional and `--template` set | clap-level `conflicts_with` (per `md-cli` `conflicts_with = "phrases"` on `--template`) |
| `--key` set without `--template` | clap-level requirement refusal |
| `--fingerprint` set without `--template` | clap-level requirement refusal |
| `--key @i=<value>` not parseable | md-cli format error |
| `--fingerprint @i=<value>` not 8 hex chars | md-cli format error |
| Network mismatch between `--network` and the supplied xpub prefix | md-cli mismatch error |
| `--chain` exceeds the descriptor's multipath alternative count | md-cli runtime error |
