# Bitcoin Core descriptors vs BIP-388 wallet_policy

Two interchange formats for the same conceptual object — a wallet's
spending rule — coexist in the Bitcoin tool ecosystem. Bitcoin Core
ships its own descriptor strings; BIP-388 standardises a JSON-shaped
"wallet policy" representation. The m-format md1 card carries a
BIP-388 wallet policy; `mnemonic export-wallet` emits either.

This is a density-watch chapter (≤4 pages); detail is intentionally
shallow.

## What each format is

| | Bitcoin Core descriptor | BIP-388 wallet_policy |
|---|---|---|
| Shape | flat string | JSON object |
| Inline keys | full xpubs in the string | placeholders `@0`, `@1`; xpubs in `keys_info` |
| Multipath | inline `<0;1>/*` | template-side `@N/**` (per-key path elided into the placeholder) |
| Master fingerprint | inline `[fp/path]` prefix | in `keys_info` strings |
| Origin paths | inline | in `keys_info` strings |
| Hardware-wallet display | full string | template-only (privacy) |
| Authoritative | Bitcoin Core source | BIP-388 spec |
| Formal name | "output descriptor" | "wallet policy" |

## What the toolkit emits

Both, via `mnemonic export-wallet --format <bitcoin-core | bip388>`:

- **`bitcoin-core` (default)** — JSON array of single-string
  descriptors, suitable for `bitcoin-cli importdescriptors`. Both
  receive (`<0;1>/0/*`) and change (`<0;1>/1/*`) descriptors are
  emitted.
- **`bip388`** — single JSON object with `description_template`
  (the template string) and `keys_info` (the bound xpubs). Targets
  hardware-wallet coordinators (Coldcard, Ledger, Foundation
  Passport) and third-party wallet imports that consume the
  BIP-388 shape directly. Bitcoin Core's `importdescriptors` RPC
  does **not** consume this format; use `--format bitcoin-core`
  for Bitcoin Core.

## Why two formats coexist

Hardware wallets care about privacy: they want to display the
*template* of a wallet (e.g., "this is a 2-of-3 sortedmulti")
without showing the user a 200-character descriptor. BIP-388
separates the template from its bound keys to make this UI
practical.

Bitcoin Core, by contrast, has historically preferred descriptors
as a *complete* representation: one string carries everything. The
flat string is easy to copy-paste and unambiguous; the trade-off is
no separable display.

Bitcoin Core (any version) accepts the descriptor shape via
`importdescriptors`. The BIP-388 wallet-policy shape targets
hardware wallets and third-party coordinators; Bitcoin Core's
`importdescriptors` RPC does not consume it directly (an open
issue in Bitcoin Core's tracker proposes adding rendering, but
nothing has shipped).

## Side-by-side example

A 2-of-3 sortedmulti, BIP-87 family, three known cosigners:

**Bitcoin Core descriptor (single-string form):**

```text
wsh(sortedmulti(2,
  [fp0/87h/0h/0h]xpub6Cosig0.../<0;1>/*,
  [fp1/87h/0h/0h]xpub6Cosig1.../<0;1>/*,
  [fp2/87h/0h/0h]xpub6Cosig2.../<0;1>/*
))
```

**BIP-388 wallet_policy:**

```json
{
  "name": "wsh-sortedmulti-2-of-3",
  "description": "",
  "description_template": "wsh(sortedmulti(2,@0/**,@1/**,@2/**))",
  "keys_info": [
    "[fp0/87h/0h/0h]xpub6Cosig0...",
    "[fp1/87h/0h/0h]xpub6Cosig1...",
    "[fp2/87h/0h/0h]xpub6Cosig2..."
  ]
}
```

Same wallet; different shapes. The toolkit converts between them via
`mnemonic export-wallet`.

## Choosing one format over the other

| Receiving software | Use |
|---|---|
| Bitcoin Core (any version, via `importdescriptors`) | `--format bitcoin-core` |
| Sparrow | `--format bip388` (sparrow-native export deferred) |
| Specter | `--format bip388` (specter-native export deferred) |
| Coldcard / SeedSigner / Foundation Passport | `--format bip388` |
| Custom tooling | whichever shape is cheapest to parse |

(`--format sparrow` and `--format specter` flags are accepted by
the binary but currently return a deferral stub; use
`--format bip388` for both. A future v0.8.x patch may light up
the stubs.)

## What the m-format md1 card carries

md1 stores a BIP-388 wallet-policy *template*: the policy string
with `@0`, `@1` placeholders, plus a single bound xpub for the
slot it represents. Multi-cosigner reconstruction picks up each
cosigner's mk1 and resolves the placeholders at decode time.

BIP-388's separation of template-from-keys was the structural
choice that made the m-format possible: each mk1 carries its
cosigner's xpub independently, and the shared md1 carries the
template. Reconstructing a flat Bitcoin Core descriptor requires
mk1 + md1 simultaneously — exactly the cross-binding the
`policy_id_stub` enforces.
