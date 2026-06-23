# `mk derive` {#mk-derive}

Derive a child xpub at a relative path from a card's
xpub.\index{mk derive} **Read-only** — an xpub can derive only
**unhardened** children (it carries no private key); any hardened
component in the requested path is rejected. The emitted
`child_xpub` is composable: pipe it back into
[`mk encode`](#mk-encode) to mint a child `mk1`, or into the
`md`/`mnemonic` descriptor surfaces. No signing, no spend
authority.

Exactly one of [`--path`](#mk-derive-path) or
[`--index`](#mk-derive-index) is required.

## Outline {#mk-derive-outline}

- [`--path`](#mk-derive-path) — relative unhardened derivation path (e.g. `m/0/5`)
- [`--index`](#mk-derive-index) — single external-chain index (sugar for `--path m/0/<N>`)
- [`--json`](#mk-derive-json) — emit structured JSON instead of text

## `--path` {#mk-derive-path}

Text widget. Relative derivation path, **unhardened only** — e.g.
`m/0/5`. A path containing a hardened marker (`'` or `h`) is
refused because a public key cannot derive hardened children. XOR
with [`--index`](#mk-derive-index): supply exactly one.

## `--index` {#mk-derive-index}

Text widget (non-negative integer). Convenience sugar for a single
external-chain child: `--index N` is exactly `--path m/0/<N>`. XOR
with [`--path`](#mk-derive-path).

## `--json` {#mk-derive-json}

Boolean. Emit a structured JSON object
(`{ "schema_version": 1, … "child_xpub": … }`) instead of the
text form. Default off.

## Worked example

:::danger
The card below derives from the canonical all-`abandon` test seed.
The derived child xpub is public test material — **never fund any
address derived from it.**
:::

1. **mk** tab; pick **Derive (xpub → child xpub)**.
2. Paste both canonical `mk1` strings into the `mk1-strings` field.
3. `--path`: `m/0/5`.
4. **Run** (no run-confirm modal — `mk derive` operates on public
   material).

The output panel renders the child xpub. Copy it into
[`mk encode`](#mk-encode) (with a fresh origin path) to mint a
child card, or into descriptor tooling.

## Refusals

| Trigger | Refusal |
|---|---|
| No positional `mk1-strings` provided AND stdin not used | clap-level `required` error |
| Neither `--path` nor `--index` supplied | exit ≠ 0 — exactly one is required |
| Both `--path` and `--index` supplied | exit 64 — mutually exclusive |
| `--path` contains a hardened component (`'`/`h`) | exit ≠ 0 — xpubs cannot derive hardened children |
| `--path` not a parseable relative derivation path | exit 64 with the matching parse error |
| Any positional that does not parse as `mk1` | exit 2 with the matching `mk-codec` error |
