# `mk` — per-tab reference

The `mk` tab covers the BIP-32-xpub CLI (`mk-cli`), eight
subcommands that operate on `mk1` cards (the public-key card of
the m-format constellation bundle). The `mk1` encodes an xpub
plus origin metadata (master fingerprint, derivation path) plus
one or more `policy_id_stub` bytes that bind the card to a
matching `md1` wallet-policy template.

The `mk` tab's pinned upstream version at v1.1 of this manual is
`mk-cli v0.9.0` (per `docs/manual-gui/pinned-upstream.toml`).
Pinned-banner format `Pinned: mk 0.9.0`.

## Subcommand index

The eight subcommands group into four families:

- **Encode + decode.** Round-trip from xpub + origin metadata to
  `mk1` and back.
  - [`mk encode`](#mk-encode)\index{mk encode} — emit one or
    more `mk1` strings from an xpub + origin metadata + at least
    one policy-id stub.
  - [`mk decode`](#mk-decode)\index{mk decode} — reassemble +
    decode one or more `mk1` strings to xpub + origin metadata.
- **Inspect + verify + repair.** Read structural fields with
  richer commentary; check decode validity and optionally
  content-match against expected fields; BCH error-correct a
  corroded or mis-copied card.
  - [`mk inspect`](#mk-inspect)\index{mk inspect} — decode plus
    structural commentary (per-component path breakdown, per-chunk
    BCH variant, xpub-derived fingerprint).
  - [`mk verify`](#mk-verify)\index{mk verify} — BCH-check the
    cards and optionally content-match the decoded fields against
    user-supplied expected values.
  - [`mk repair`](#mk-repair)\index{mk repair} — BCH error-correct
    up to four substitution errors per chunk, with a per-position
    fix report.
- **Read-only derivation.** Public-watch surfaces over the card's
  xpub — no private keys, no signing.
  - [`mk address`](#mk-address)\index{mk address} — render the
    receive/change addresses controlled by the card's xpub.
  - [`mk derive`](#mk-derive)\index{mk derive} — derive an
    unhardened child xpub at a relative path (composable back into
    `mk encode`).
- **Maintainer tools.**
  - [`mk vectors`](#mk-vectors)\index{mk vectors} — print the
    SHA-pinned v0.1 test-vector corpus as JSON (typically used by
    mk-cli developers, not end users).

## Form shape

All eight subcommands follow the same form scaffolding described
in [chapter 31](#first-launch-walkthrough): top-of-form
`Pinned: mk 0.9.0` label + subcommand selector ComboBox +
per-subcommand `?` help-icon; per-flag widgets; an action bar
with **Copy command**, **Run** buttons; an always-on `Preview:`
line. None of the mk-tab subcommands accept slot input
(`allows_slots: false` for all 8).

The `mk` subcommands operate on **public** material throughout.
None of the schema flags is `secret: true`. The run-confirm modal
does not fire for any mk-tab invocation, regardless of input. An
xpub plus its derivation metadata is sensitive (it allows
chain-watch and address-derivation across the whole sub-tree)
but it is not "secret-class" under the threat model of
[§14 Defense 2](#secret-handling) — the modal is reserved for
material that recovers a wallet on its own.

## Worked-example data convention

The mk-tab worked examples reuse the `V2_bip84_mainnet_1_stub_with_fp`
fixture from the `mk-codec` v0.1 SHA-pinned vector corpus
(reproducible at any time via
[`mk vectors`](#mk-vectors)). The fixture's inputs are:

- **xpub:** `xpub6BmeGmRo4LosAcU21HDaGcvtaQ7GrqQcY48nBkE22qM6KVwQUjRJ1BGzk84SFVHgLcd61Vcnhr8petHexjjn5WbQ9PriVrRhphw4oCp2z6a`
- **origin_fingerprint:** `deadbeef`
- **origin_path:** `m/84'/0'/0'`
- **policy_id_stub:** `c0ffee00`

The fixture's canonical mk1 strings (chunk_set_id `144470`) are:

```text
mk1qpydzkpqqsqupllwqr02m0h0qvzg3vs7zqsrqq4g4z52329g4z52329g4z52329g4z52329g4z52329g4z52329g4qpy6m8lr3sdrxkguwax
mk1qpydzkppfdkdzdssxt9fh54wh8vsp2jdghv74kq2e9prxaxy2xnj2ng8vm68nf54c0vrdlfrgjzpd
```

This data is **public** — no real wallet ever held these keys.
Use it for round-trip demonstration only.

Note that `mk encode` generates a new `chunk_set_id` per
invocation (the 4-byte cross-chunk binding header), so a fresh
encode of the same inputs will emit mk1 strings with different
prefix bytes than the fixture's pinned ones. The decode +
inspect + verify chapters use the fixture's exact strings since
those subcommands are pure functions of their inputs.
