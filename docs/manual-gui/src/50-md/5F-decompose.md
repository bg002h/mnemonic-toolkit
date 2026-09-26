# `md decompose` {#md-decompose}

\index{md decompose}Take **one concrete output descriptor** — a
wallet's export, with real xpubs — and split it into the pieces the
m-format cards carry: the BIP-388 template, one key record per slot in
BIP-380 origin notation, and the per-slot `--fingerprint` flags. It is
the inverse of [`md descriptor`](#md-descriptor), and the usual first
step when you already have a wallet and want its cards.

Two input modes, exactly one of which you fill: the `descriptors`
positional **or** [`--in`](#md-decompose-in). The conditional-visibility
engine (`form::conditional::md_decompose`) marks both **Required**
while neither has a value and greys `--in` once the positional has one.
The positional takes exactly **one** descriptor: md refuses two, and
explains that a receive/change pair must first be combined into one
`<0;1>` multipath descriptor.

The inputs are public, so the run-confirm modal does not fire — with
one exception: a descriptor holding a **private** key. See the
positional below.

> **GUI form:** see [GUI Forms › md › decompose](#gui-form-md-decompose).

## Outline {#md-decompose-outline}

- [`--in`](#md-decompose-in) — read the descriptor from a file (XOR with the positional)
- [`--emit`](#md-decompose-emit) — which artifact to print (default `all`)
- [`--network`](#md-decompose-network) — network the descriptor's xpubs must belong to (default `mainnet`)

## `--in` {#md-decompose-in}

Path widget. Read the descriptor from FILE instead of the positional.
Blank lines and `#` comments are skipped.

## `--emit` {#md-decompose-emit}

Dropdown; default `all`. Which artifact to print on stdout.

### Outline {#md-decompose-emit-outline}

- [`all`](#md-decompose-emit-all)
- [`template`](#md-decompose-emit-template)
- [`keys`](#md-decompose-emit-keys)
- [`fingerprints`](#md-decompose-emit-fingerprints)
- [`descriptor`](#md-decompose-emit-descriptor)
- [`commands`](#md-decompose-emit-commands)

### `all` {#md-decompose-emit-all}

The template, the key records and the fingerprint flags, each under a
`#` comment naming the command it feeds (see the worked example).

### `template` {#md-decompose-emit-template}

Only the BIP-388 template, with the origins inline
(`wpkh(@0/84'/0'/0'/<0;1>/*)`) — ready for
[`md encode`](#md-encode) or
[`md descriptor --template`](#md-descriptor-template).

### `keys` {#md-decompose-emit-keys}

Only the key records, one per line in BIP-380 origin notation
(`[73c5da0a/84'/0'/0']xpub…`), the input `mk encode --keys` reads.

### `fingerprints` {#md-decompose-emit-fingerprints}

Only the per-slot `--fingerprint @i=HEX` flags.

### `descriptor` {#md-decompose-emit-descriptor}

The descriptor itself, as md normalizes it.

### `commands` {#md-decompose-emit-commands}

Two ready-to-run shell recipes: route 1, the **keyed** card (one `md1`
carrying template and keys, via `md encode … --key …`); and route 2,
the **split** set (a keyless policy card via `md encode … --out
policy.md1`, plus one `mk1` card per key via
`mk encode --keys keys.txt --from-md1-set policy.md1`).

## `--network` {#md-decompose-network}

Dropdown; default `mainnet`. The network the descriptor's extended
keys must belong to.

### Outline {#md-decompose-network-outline}

- [`mainnet`](#md-decompose-network-mainnet)
- [`testnet`](#md-decompose-network-testnet)
- [`signet`](#md-decompose-network-signet)
- [`regtest`](#md-decompose-network-regtest)

### `mainnet` {#md-decompose-network-mainnet}

See [`mnemonic bundle --network mainnet`](#mnemonic-bundle-network-mainnet).

### `testnet` {#md-decompose-network-testnet}

See [`mnemonic bundle --network testnet`](#mnemonic-bundle-network-testnet).

### `signet` {#md-decompose-network-signet}

See [`mnemonic bundle --network signet`](#mnemonic-bundle-network-signet).

### `regtest` {#md-decompose-network-regtest}

See [`mnemonic bundle --network regtest`](#mnemonic-bundle-network-regtest).

## Positional `descriptors`

The concrete descriptor — exactly one, with or without a `#checksum`,
multipath (`<0;1>`) or fixed-path. The GUI ends its options with `--`
before it, so a descriptor can never be read as a flag.

**Public keys only.** md refuses a descriptor holding a private key
(`md: decompose: this is not a descriptor md can parse …`, exit 1), but
by then it would have been on screen. So when any key-shaped part of
the value reads as a private extended key (`xprv`, `tprv`, …), the GUI
masks the field, never saves it with the session (no positional is
saved), shows it as `••••` in the Preview and the confirm dialog, and
asks for confirmation. The **Copy command** buttons then read *"—
reveals secret"*: md takes the descriptor only on the command line, so
a copied command would carry the key.

## Worked example — split a single-sig descriptor

1. **md** tab; pick **Decompose (descriptor -> template, keys, fingerprints)**.
2. Paste into the positional:
   `wpkh([73c5da0a/84'/0'/0']xpub6CatWdiZi…VMrjPC7PW6V/<0;1>/*)`.
3. Leave `--emit` at `all`; **Run**.

```text
exit: 0
stdout:
# template — `md encode <TEMPLATE>` / `md descriptor --template`
wpkh(@0/84'/0'/0'/<0;1>/*)
# keys — BIP-380 origin notation, one record per line (`mk encode --keys`)
[73c5da0a/84'/0'/0']xpub6CatWdiZi…VMrjPC7PW6V
# fingerprints — per-slot flags for `md encode` / `md descriptor`
--fingerprint @0=73c5da0a
```

(The xpub is abbreviated here; md prints it whole.) Set `--emit` to
`commands` for the two shell recipes instead.

## Refusals

| Trigger | Refusal |
|---|---|
| Neither the positional nor `--in` | clap, exit 2 (the GUI marks both Required first) |
| Both | clap `cannot be used with`, exit 2 (the GUI greys `--in`) |
| Two descriptors (a receive/change pair) | `md: decompose: decompose takes ONE descriptor and 2 were supplied. …`, exit 1 — combine them into one `<0;1>` descriptor first |
| A BIP-388 template (`@0`, `@1`, …) instead of a descriptor | md, exit 1: a template goes to `md encode` |
| A private key in the descriptor | md, exit 1; the GUI masks it first |
