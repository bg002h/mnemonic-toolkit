# Appendix C — BIP-32 derivation primer

BIP-32 is how one secret seed becomes an unlimited tree of public-
private keypairs. It is the foundation of every "HD" (hierarchical
deterministic) wallet.

For the formal spec see
[BIP-32](https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki).

## The basics

A BIP-32 *master node* is a 32-byte private key plus a 32-byte
"chain code" (extra entropy that prevents an attacker who learns one
child key from inferring siblings). From any node, two operations
produce a child node:

- **Normal (non-hardened) derivation:** `child_i = HMAC-SHA-512(parent_chain_code, parent_pubkey || i)`
- **Hardened derivation:** `child_i' = HMAC-SHA-512(parent_chain_code, parent_privkey || (i + 2^31))`

Hardened derivation needs the parent's *private* key; normal
derivation can be done with only the parent's *public* key — which is
why xpubs can derive child addresses without exposing privkeys.

## Path notation

A BIP-32 derivation path is a slash-separated sequence:

```text
m / 84' / 0' / 0' / 0 / 5
```

- `m` = master.
- `'` (or `h`, or `H`) = hardened (add `2^31` to the index).
- Subsequent unprimed indices = non-hardened.

The standard wallet path `m/84'/0'/0'/0/5` reads:
"the 6th external receive address (index 5) of account 0 of Bitcoin
(coin 0) with BIP-84 (purpose 84')".

## Standard purpose paths

| Purpose | Path | Address shape |
|---|---|---|
| BIP-44 | `m/44'/0'/0'` | `1...` (legacy P2PKH) |
| BIP-49 | `m/49'/0'/0'` | `3...` (P2SH-wrapped P2WPKH) |
| BIP-84 | `m/84'/0'/0'` | `bc1q...` (native SegWit P2WPKH) |
| BIP-86 | `m/86'/0'/0'` | `bc1p...` (single-key taproot) |
| BIP-87 | `m/87'/0'/0'` | multisig (script-type-agnostic) |
| BIP-48 | `m/48'/0'/0'/2'` | multisig (script-type indexed via 4th level) |

Coin index `0` = Bitcoin mainnet; `1` = testnet/signet/regtest.

The mk1 card carries the *origin* — the master fingerprint plus the
hardened path used. Recovery software needs both to know which
sub-tree of the master node a given xpub came from.

## Multipath descriptors

Modern wallets use multipath notation `<0;1>/*` to mean "the same
xpub, with chain 0 for receives and chain 1 for change, then any
non-hardened child along the wildcard". This is what's actually
emitted on md1 cards — a wallet policy with multipath.

```text
wpkh([fp/84'/0'/0']xpub.../<0;1>/*)
```

reads "for each chain (0 = receive, 1 = change), all non-hardened
children form valid receive/change addresses."

## Why hardening matters for recovery

If an attacker compromises *one* xpub down a non-hardened path, they
can derive all sibling xpubs at the same level (and below, if those
levels are also non-hardened). Hardening at the *purpose / coin /
account* boundary prevents this; child-level non-hardening is
acceptable because the children only carry public keys.

The m-format star follows this convention: hardened path up to the
account level, then non-hardened wildcards for receive/change.
