# `mk address` {#mk-address}

Render the receive/change addresses controlled by a card's
xpub.\index{mk address} **Read-only public derivation** — the
`mk1` carries only an extended *public* key, so no private key,
no signing, and no spend authority is ever involved. This is the
public-watch counterpart of the toolkit's address tooling, scoped
to a single `mk1` card's key.

The address type is inferred from the origin-path purpose **at
canonical single-sig account depth** (`m/44'`→`p2pkh`,
`m/49'`→`p2sh-p2wpkh`, `m/84'`→`p2wpkh`, `m/86'`→`p2tr`) and is
overridable with [`--address-type`](#mk-address-address-type). A
card whose origin is *not* at a recognized account depth requires
the explicit flag (and the runtime prints a stderr advisory that
addresses are derived relative to the card's xpub).

Multisig-cosigner cards (`m/48'` / `m/87'` origins) are
**refused** — a single-key address would not match the multisig
wallet and could mislead an operator into funding an unspendable
script. Use descriptor tooling (the `md` tab / `mnemonic
addresses`) for multisig.

> **GUI form:** see [GUI Forms › mk › address](#gui-form-mk-address).

## Outline {#mk-address-outline}

- [`--address-type`](#mk-address-address-type) — script type override (`p2pkh`|`p2sh-p2wpkh`|`p2wpkh`|`p2tr`)
- [`--count`](#mk-address-count) — addresses per chain from index 0 (default 10; XOR `--range`)
- [`--range`](#mk-address-range) — inclusive index range `A,B` (XOR `--count`)
- [`--chain`](#mk-address-chain) — which chain(s): `receive`|`change`|`both`
- [`--network`](#mk-address-network) — network override (`mainnet`|`testnet`|`signet`|`regtest`)
- [`--json`](#mk-address-json) — emit structured JSON instead of text rows

## `--address-type` {#mk-address-address-type}

Dropdown — `p2pkh` · `p2sh-p2wpkh` · `p2wpkh` · `p2tr`. Optional.
Overrides the account-depth purpose heuristic. **Required** when
the card's origin path is not at a recognized single-sig account
depth (otherwise the runtime cannot pick a script type and refuses
with an advisory). The GUI presents this as a labelled dropdown;
leaving it unset lets the heuristic choose.

## `--count` {#mk-address-count}

Text widget (positive integer). Number of addresses to render per
selected chain, starting at index 0. Default 10. Mutually
exclusive with [`--range`](#mk-address-range) — supplying both is
refused.

## `--range` {#mk-address-range}

Text widget, `A,B` form. Inclusive index range `A..=B` per chain.
Mutually exclusive with [`--count`](#mk-address-count). Use this to
render a window away from index 0 (e.g. `100,109`).

## `--chain` {#mk-address-chain}

Dropdown — `receive` · `change` · `both`. Which BIP-32 external
(`receive`, chain `0`) and/or internal (`change`, chain `1`)
addresses to render. Default `receive`. Under `both`, the text
output groups rows by chain (`receive` first, then `change`).

## `--network` {#mk-address-network}

Dropdown — `mainnet` · `testnet` · `signet` · `regtest`. Optional
network override. Defaults to the network implied by the xpub's
version bytes; when supplied it MUST agree with the xpub's network
kind, otherwise the runtime refuses. See the canonical
[`network` reference](#mnemonic-bundle-network-outline) for the
shared four-value enum.

## `--json` {#mk-address-json}

Boolean. Emit a structured JSON object (per-chain arrays of
`{index, address}`) instead of the human text rows. Default off.

## Worked example

:::danger
The xpub below derives from the canonical all-`abandon` test seed.
**Never fund any address it produces.** It exists only to make the
worked example reproducible.
:::

1. **mk** tab; pick **Address (xpub → receive/change addresses)**.
2. Paste both canonical `mk1` strings (one per row) into the
   `mk1-strings` field.
3. `--count`: `3`.
4. Leave `--chain` at `receive`.
5. **Run** (no run-confirm modal — `mk address` operates on
   public material).

The output panel renders three receive addresses, one per row.
Switch `--chain` to `both` to additionally render the change
chain (rows grouped `receive` then `change`).

## Refusals

| Trigger | Refusal |
|---|---|
| No positional `mk1-strings` provided AND stdin not used | clap-level `required` error |
| Origin path not at a recognized account depth AND `--address-type` omitted | exit ≠ 0 with an advisory that the script type is undetermined |
| `m/48'` / `m/87'` multisig-cosigner origin | exit ≠ 0 — single-key addresses refused (use descriptor tooling) |
| Both `--count` and `--range` supplied | exit 64 — mutually exclusive |
| `--network` disagrees with the xpub's network kind | exit ≠ 0 — network mismatch |
| Any positional that does not parse as `mk1` | exit 2 with the matching `mk-codec` error |
