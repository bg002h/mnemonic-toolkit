# `md` — per-tab reference

The `md` tab covers the descriptor-mnemonic CLI (`md-cli`), nine
subcommands that operate on `md1` cards (the descriptor card of
the m-format constellation bundle). The `md1` encodes a BIP-388
wallet-policy template plus the bound public-key references that
the bundle's `mk1` cards carry.

The `md` tab's pinned upstream version at v1.1 of this manual is
`md-cli v0.7.0` (per `docs/manual-gui/pinned-upstream.toml`).
Pinned-banner format `Pinned: md 0.7.0`.

## Subcommand index

The subcommands group into five families:

- **Decode + inspect.** Read what an `md1` carries.
  - [`md inspect`](#md-inspect)\index{md inspect} — decode + pretty-print.
  - [`md decode`](#md-decode)\index{md decode} — decode an `md1`
    string into the canonical wallet-policy template.
  - [`md bytecode`](#md-bytecode)\index{md bytecode} — low-level
    payload-bit dump for debugging.
- **Encode + verify.** Round-trip from template to `md1` and
  back.
  - [`md encode`](#md-encode)\index{md encode} — emit an `md1`
    from a BIP-388 template (or compile from a sub-Miniscript-Policy
    expression).
  - [`md verify`](#md-verify)\index{md verify} — assert one or
    more `md1` strings re-encode to a given template.
- **Compile.** Translate higher-level policy to template.
  - [`md compile`](#md-compile)\index{md compile} — translate a
    sub-Miniscript-Policy expression into a BIP-388 template.
- **Derive.** Use an `md1` to produce wallet artifacts.
  - [`md address`](#md-address)\index{md address} — derive Bitcoin
    addresses from an `md1` (or from a template + cosigner xpubs).
- **Maintainer tools.**
  - [`md vectors`](#md-vectors)\index{md vectors} — regenerate
    the test-vector corpus (typically used by md-cli developers,
    not end users).

## Form shape

All eight subcommands follow the same form scaffolding described
in [chapter 31](#first-launch-walkthrough): top-of-form `Pinned:
md 0.5.0` label + subcommand selector ComboBox + per-subcommand
`?` help-icon; per-flag widgets; an action bar with **Copy
command**, **Run** buttons; an always-on `Preview:` line. None of
the md-tab subcommands accept slot input (`allows_slots: false`
for all 8).

The `md` subcommands operate on **public** material throughout.
None of the schema flags is `secret: true`. The run-confirm modal
does not fire for any md-tab invocation, regardless of input.

## Worked-example data convention

Examples in this chapter reuse the canonical `md1` strings derived
from the all-`abandon` BIP-39 vector at the BIP-84 m/84'/0'/0'
path (consistent with the bundle/verify-bundle worked examples
in chapter 42 / 43). The three canonical `md1` strings are:

```text
md1fgdxlpqpqpm6jzzqqvqpdqw0za5zs4gyy55aq4vsmnhy4s6wyaypu34c7raqu8np
md1fgdxlpqf2zcgefcpupmel75q5435j7seugaj5jr7qyur6vt76es5cdeyrq7zdy0d
md1fgdxlpq3xa2dk8vwpj7gx74hwqxqdp083jehp5tdrfa0n5zdfkqcdlrvnh5r62jn
```

These are public material — `md1` cards encode wallet-policy
templates plus the `policy_id_stub` cross-binding metadata; they
do not carry secret keys.
