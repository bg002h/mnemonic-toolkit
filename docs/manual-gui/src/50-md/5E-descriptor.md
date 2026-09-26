# `md descriptor` {#md-descriptor}

\index{md descriptor}Render the **concrete BIP-380 descriptor** of a
wallet — the string a wallet coordinator imports — from what you hold:
an `md1` card, a template plus keys, or a keyless policy card plus its
`mk1` key cards. It is the descriptor-producing sibling of
[`md address`](#md-address), and shares its input modes and seating
flags.

Three input modes (`form::conditional::md_descriptor`):

- **A — `md1` phrases.** Paste the card into the `[PHRASES]`
  positional. [`--template`](#md-descriptor-template),
  [`--key`](#md-descriptor-key) and
  [`--fingerprint`](#md-descriptor-fingerprint) grey out.
- **B — template + keys.** Fill [`--template`](#md-descriptor-template);
  [`--key`](#md-descriptor-key) becomes **Required** until it has at
  least one row (md: `--key @i=<XPUB> required when --template is
  supplied`, exit 2), and the key-card flags grey out.
- **C — key cards.** Fill [`--from-mk1`](#md-descriptor-from-mk1) or
  [`--from-mk1-file`](#md-descriptor-from-mk1-file) together with the
  **keyless** policy card's phrases (the positional turns Required).
  Only here are [`--seat`](#md-descriptor-seat) and
  [`--emit`](#md-descriptor-emit) enabled.

While nothing is filled, `--template` and the positional are both
marked Required. [`--chain`](#md-descriptor-chain) and
[`--change`](#md-descriptor-change) exclude each other: filling one
greys the other.

The inputs are public, so the run-confirm modal does not fire — with
one exception: a `--key` row holding a **private** key (see
[`--key`](#md-descriptor-key)).

> **GUI form:** see [GUI Forms › md › descriptor](#gui-form-md-descriptor).

## Outline {#md-descriptor-outline}

- [`--template`](#md-descriptor-template) — BIP-388 template (mode B; XOR with the phrases)
- [`--key`](#md-descriptor-key) — concrete xpub for placeholder `@i` (repeating; mode B)
- [`--fingerprint`](#md-descriptor-fingerprint) — master fingerprint for `@i` (repeating; mode B)
- [`--path`](#md-descriptor-path) — shared origin path for slots whose template gave none
- [`--from-mk1`](#md-descriptor-from-mk1) — `mk1` key cards to seat into a keyless policy card (repeating; mode C)
- [`--from-mk1-file`](#md-descriptor-from-mk1-file) — read `mk1` key cards from a file (mode C)
- [`--seat`](#md-descriptor-seat) — assert which card seats a slot (repeating; mode C)
- [`--network`](#md-descriptor-network) — network for xpub validation (default `mainnet`)
- [`--chain`](#md-descriptor-chain) — collapse the multipath group to one chain (XOR with `--change`)
- [`--change`](#md-descriptor-change) — sugar for `--chain 1`
- [`--emit`](#md-descriptor-emit) — `md1`: print the keyed card instead of the descriptor (mode C only)
- [`--out`](#md-descriptor-out) — write the keyed `md1` to a file (only with `--emit md1`)
- [`--group-size`](#md-descriptor-group-size) — engraving-card grouping (only with `--emit md1`)
- [`--separator`](#md-descriptor-separator) — engraving-card separator (only with `--emit md1`)
- [`--json`](#md-descriptor-json) — emit JSON output
- [`--verify-against`](#md-descriptor-verify-against) — spend-equality check against another card
- [`--experimental`](#md-descriptor-experimental) — accept a template with a signature-free spend path

## `--template` {#md-descriptor-template}

Text widget. A BIP-388 template such as `wpkh(@0/<0;1>/*)` (for
example the output of [`md compose`](#md-compose) or
[`md decompose --emit template`](#md-decompose-emit-template)).
Requires at least one [`--key`](#md-descriptor-key). Greyed when the
positional has a value.

## `--key` {#md-descriptor-key}

Repeating Text widget, `@i=XPUB` or the origin-notated
`@i=[fingerprint/path]XPUB`. Enabled only in mode B, and Required
there until one row is filled.

**Public keys only.** A row holding a private extended key (`xprv`,
`tprv`, …) is masked in the field, never saved with the session, shown
as `••••` in the Preview and the confirm dialog, and makes the run ask
for confirmation. md refuses it anyway, but the GUI does not wait for
that to hide it. The **Copy command** buttons then read *"— reveals
secret"*: md takes the key only on the command line, so a copied
command would carry it. (Measured: with `@0=[73c5da0a/84'/0'/0']xprv…`
the dialog opens with *"This invocation passes secret-bearing arguments
to md:"* and `--key ••••`.)

## `--fingerprint` {#md-descriptor-fingerprint}

Repeating Text widget, `@i=HEX` (eight hex digits). Mode B only.

## `--path` {#md-descriptor-path}

Text widget. A shared origin path applied **per slot** to whichever
`@i` the template gave no inline origin; an inline origin always
wins. Same rule as [`md address --path`](#md-address-path).

## `--from-mk1` {#md-descriptor-from-mk1}

Repeating Text widget. An `mk1` key-card string, supplied together
with the keyless policy card's phrases: each card is seated in the
slot whose declared origin it satisfies. Greyed in mode B. The seating
rules are those of [`md address --from-mk1`](#md-address-from-mk1).

## `--from-mk1-file` {#md-descriptor-from-mk1-file}

Path widget. Read `mk1` key-card strings from FILE, one per line
(blank lines and `#` comments skipped). Combines with
[`--from-mk1`](#md-descriptor-from-mk1).

## `--seat` {#md-descriptor-seat}

Repeating Text widget, `@i=<chunk-set-id>[#k]`. Asserts the seating of
one slot; greyed until a key card is given. Same rule as
[`md address --seat`](#md-address-seat).

## `--network` {#md-descriptor-network}

Dropdown; default `mainnet`. The network the xpubs are validated
against.

### Outline {#md-descriptor-network-outline}

- [`mainnet`](#md-descriptor-network-mainnet)
- [`testnet`](#md-descriptor-network-testnet)
- [`signet`](#md-descriptor-network-signet)
- [`regtest`](#md-descriptor-network-regtest)

### `mainnet` {#md-descriptor-network-mainnet}

See [`mnemonic bundle --network mainnet`](#mnemonic-bundle-network-mainnet).

### `testnet` {#md-descriptor-network-testnet}

See [`mnemonic bundle --network testnet`](#mnemonic-bundle-network-testnet).

### `signet` {#md-descriptor-network-signet}

See [`mnemonic bundle --network signet`](#mnemonic-bundle-network-signet).

### `regtest` {#md-descriptor-network-regtest}

See [`mnemonic bundle --network regtest`](#mnemonic-bundle-network-regtest).

## `--chain` {#md-descriptor-chain}

Number widget, `0` or `1`. Collapse the `<0;1>` multipath group to one
chain (`0` = receive, `1` = change). Leave it unset for the multipath
form. Greyed while [`--change`](#md-descriptor-change) is set.

## `--change` {#md-descriptor-change}

Boolean. Sugar for `--chain 1`; greyed while `--chain` has a value.

## `--emit` {#md-descriptor-emit}

Dropdown. Enabled only in mode C; greyed otherwise, which also keeps a
stale `md1` choice off the command line (md refuses `--emit md1`
outside mode C, exit 2, and points to `md encode <TEMPLATE> --key …`).

### Outline {#md-descriptor-emit-outline}

- [`(none)`](#md-descriptor-emit-)
- [`md1`](#md-descriptor-emit-md1)

### `(none)` {#md-descriptor-emit-}

The default: no flag; stdout is the concrete descriptor.

### `md1` {#md-descriptor-emit-md1}

Put the **keyed** `md1` card on stdout instead of the descriptor,
minted from the seating result, with its engraving card on stderr.
Enables [`--out`](#md-descriptor-out),
[`--group-size`](#md-descriptor-group-size) and
[`--separator`](#md-descriptor-separator).

## `--out` {#md-descriptor-out}

Path widget. Write the keyed `md1` to FILE (created `0600`) instead of
stdout. Only with `--emit md1`.

## `--group-size` {#md-descriptor-group-size}

Number widget, `0..=255`, default `5` (`0` = unbroken). Groups the
engraving card on stderr. Only with `--emit md1`.

## `--separator` {#md-descriptor-separator}

Dropdown with one value, `space` (the default). Only with
`--emit md1`.

### `space` {#md-descriptor-separator-space}

ASCII space between groups.

## `--json` {#md-descriptor-json}

Boolean. Emit JSON output.

## `--verify-against` {#md-descriptor-verify-against}

Text widget. A **spend-equality** comparison target: an `md1` string,
or a FILE holding one or more. Exit `0` = spend-equal, `5` = not
spend-equal. Works with any input mode.

## `--experimental` {#md-descriptor-experimental}

Boolean. Accept a template with a spend path that requires no
signature, mirroring [`md encode --experimental`](#md-encode-experimental).

## Positional `[PHRASES]`

One or more `md1` phrases (mode A), or, with `--from-mk1`, the keyless
policy card's phrases (mode C). Mutually exclusive with `--template`.

## Worked example — template + key

1. **md** tab; pick **Descriptor (md1 / template / key cards -> descriptor)**.
2. `--template`: `wpkh(@0/<0;1>/*)`.
3. Add a `--key` row:
   `@0=[73c5da0a/84'/0'/0']xpub6CatWdiZi…VMrjPC7PW6V`.
4. **Run**.

```text
exit: 0
stdout:
wpkh([73c5da0a/84'/0'/0']xpub6BemYiVNp19ZzsXfZEitq3w1XJmqRCJcSbxPVjjkZEbmdd445fc9CJ9qiRzFf5MLa3KsF6tb8rWoVUfQW489vNiYJUAYvfiaDBP3Tq1adUC/<0;1>/*)#j8a2anzg
stderr:
note: coordinators for this descriptor, multipath form (verified versions only; newer: unmeasured):
  …
note: stdout is watch-only — public keys only, cannot spend
```

The xpub text differs from the one you pasted, but the key is the
same: md writes the xpub with a zero parent fingerprint, because a card
cannot carry one (the key and chain code are unchanged). Tick
`--change` and the same run prints the change-chain descriptor,
`…/1/*)#6m00nkeg`.

## Refusals

| Trigger | Refusal |
|---|---|
| `--template` without a `--key` | `md: --key @i=<XPUB> required when --template is supplied`, exit 2 (the GUI marks `--key` Required) |
| `--emit md1` outside mode C | md, exit 2 (the GUI greys `--emit`) |
| `--chain` and `--change` together | clap, exit 2 (the GUI greys the other) |
| A private key in `--key` | md refuses it (exit 1); the GUI masks it first |
