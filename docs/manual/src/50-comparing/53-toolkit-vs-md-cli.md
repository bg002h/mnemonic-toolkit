# `mnemonic` vs `md-cli` for descriptor cards

For md1 (the descriptor card), both `mnemonic` and `md` cover the
core needs but with different specialities.

## Use `md` (the standalone CLI) when

- You want a *direct* descriptor → md1 conversion (`md encode`) or
  md1 → descriptor conversion (`md decode`).
- You want to compile a Miniscript Policy expression into a BIP-388
  template (`md compile`).
- You need to derive concrete addresses from a wallet-policy
  descriptor (`md address` — receive/change selectors, count + index
  windowing).
- You want to inspect the raw payload bits of an md1 string for
  debugging (`md bytecode`).
- You want to regenerate the project's pinned test vectors
  (`md vectors --out` — maintainer use).
- You're verifying md1 strings against a known template
  (`md verify`).
- You're integrating md1 into a non-toolkit pipeline.

## Use `mnemonic` (the toolkit) when

- You need md1 alongside ms1 and mk1 in a bundle.
- You want the cross-binding `policy_id_stub` to verify md1 against
  the seed-and-key side automatically.
- You're building a watch-only artifact for Bitcoin Core / BIP-388 /
  Sparrow / Specter (`mnemonic export-wallet` consumes md1's role
  internally).
- You're tracking a multi-source multisig flow where md1 is one of
  many cards.

## Side-by-side

| Capability | `md` standalone | `mnemonic` toolkit |
|---|---|---|
| Encode descriptor → md1 | yes (`md encode`) | yes (via `bundle`) |
| Decode md1 → descriptor | yes (`md decode`) | yes (via `bundle` round-trip) |
| Compile Policy → template | yes (`md compile`) | no (only template names) |
| Derive addresses | yes (`md address`) | no (use `md address` separately) |
| Inspect raw bytes | yes (`md bytecode`, `md inspect`) | no |
| Verify against template | yes (`md verify`) | yes (via `verify-bundle`) |
| Bundle synthesis with ms1 + mk1 | no | yes |
| Cross-binding `policy_id_stub` | no | yes |
| Wallet export | no | yes |

## Practical takeaway

`md` is the right tool for descriptor-side work — Policy
compilation, address derivation, raw-byte inspection. `mnemonic`
takes md1 as part of a bundle but doesn't replicate the
descriptor-side power tools that `md` provides directly. Many
workflows use both: `md compile` to land a template, then
`mnemonic bundle --descriptor=…` to ship it across all three
cards.
