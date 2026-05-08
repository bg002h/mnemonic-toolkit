# Appendix D — Descriptors and BIP-388 primer

A *descriptor* is a Bitcoin Core data type that describes a wallet's
spending rule completely: the script type, the keys involved, and
how to derive specific addresses. *BIP-388* extends descriptors into
a wallet-policy format suitable for hardware wallets and multisig
coordinators.

For the specs see [BIP-380 through BIP-389](https://github.com/bitcoin/bips/).

## Descriptor anatomy

A typical descriptor wraps a key (or set of keys) inside one or more
*script-context wrappers* that determine how the script is executed:

```text
wpkh(  [73c5da0a/84'/0'/0']xpub6Cat.../<0;1>/*  )
```

The pieces:

- `wpkh(...)` — Witness Public Key Hash, the BIP-84 native-segwit
  outer wrapper.
- `[73c5da0a/84'/0'/0']` — *origin*: master fingerprint + the path
  used to derive the inner xpub.
- `xpub6Cat...` — the BIP-32 extended public key.
- `/<0;1>/*` — *multipath*: chain 0 for receives, chain 1 for
  change; `*` is the wildcard non-hardened address index.

## Common outer wrappers

| Wrapper | Address shape | Multisig? |
|---|---|---|
| `pkh(...)` | `1...` legacy P2PKH | no |
| `sh(...)` | `3...` P2SH | yes (legacy multisig) |
| `wpkh(...)` | `bc1q...` native segwit | no |
| `wsh(...)` | `bc1q...` (longer) native segwit script | yes |
| `sh(wsh(...))` | `3...` nested segwit | yes |
| `tr(...)` | `bc1p...` taproot | yes (via tapscript) |

Inside multisig wrappers, `multi(K, key1, key2, …)` and
`sortedmulti(K, …)` (BIP-67-sorted) construct the K-of-N policy.
Taproot uses `multi_a` / `sortedmulti_a` instead.

## What BIP-388 adds

A flat descriptor string is opaque: a hardware wallet can't show
the user "this is a 2-of-3 sortedmulti" without showing them the
full string. BIP-388 separates the *template* from the *bound keys*:

```json
{
  "name": "wsh-sortedmulti-2-of-3",
  "description_template": "wsh(sortedmulti(2,@0/**,@1/**,@2/**))",
  "keys_info": [
    "[fp0/87h/0h/0h]xpub...",
    "[fp1/87h/0h/0h]xpub...",
    "[fp2/87h/0h/0h]xpub..."
  ]
}
```

The template uses placeholders `@0`, `@1`, `@2`; the `keys_info`
array binds each placeholder to a concrete xpub at decode time.

The hardware-wallet UI shows the template (verifiable as "2-of-3
sortedmulti") without exposing the per-cosigner xpub strings to a
casual viewer.

## Why this matters for the m-format constellation

The md1 card carries a BIP-388 wallet-policy template plus a *single*
bound xpub for the slot it represents. Each cosigner's mk1 card
carries that cosigner's xpub independently. Reconstructing the
flat descriptor requires the md1 + each cosigner's mk1 — the
cross-binding the toolkit checks via `policy_id_stub`.

This separation is what enables multi-cosigner air-gapped synthesis:
each cosigner produces their own xpub on their own machine; the
coordinator builds the wallet-policy template; recovery composes
both sides.

## Multipath / `<0;1>/*` semantics

The `<0;1>/*` notation is BIP-389 (multipath descriptors): one
descriptor expresses *both* receive and change paths. Wallet
software derives addresses by substituting each value in the
multipath set into the position; for `<0;1>/*` that means two
distinct address sequences, one for chain 0 (external/receive)
and one for chain 1 (internal/change).

The toolkit emits multipath descriptors by default — both chains in
one md1 card, no need to engrave receive and change as separate
descriptors.
