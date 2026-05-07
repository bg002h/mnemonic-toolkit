# Deterministic child secrets via BIP-85

A single seed phrase can mathematically derive an unlimited supply
of *new* deterministic secrets — fresh BIP-39 phrases, raw entropy,
HD-seed bytes, child xprvs, password strings, even DICE entropy
streams. This is BIP-85: take one master seed and grind it through
HMAC-SHA-512\index{HMAC-SHA-512} with an application code and an
index to produce a deterministic, *independent* child secret.

The use cases:

- **Child wallets** — derive an entire new BIP-39 phrase
  (12 / 18 / 24 words; 9 BIP-85 languages) for an air-gapped
  hardware wallet without storing a separate backup.
- **Passwords** — derive deterministic per-service passwords; lose
  the password manager but keep the master seed and you can
  re-derive every login.
- **Deterministic randomness** — derive entropy for tools that take
  hex or base64 input.
- **DICE** — derive deterministic *fair-dice* outcomes; useful for
  collaborative games or audit-replay.

:::primer
**Background — why BIP-85 is safe for the master seed.** BIP-85
applies HMAC-SHA-512 to the master node + a hardened path encoding
(application + length + index). The output bits are
cryptographically independent from the parent seed *and* from each
other; observing or even compromising a child secret reveals nothing
about the master or about siblings. The master seed is unchanged
and unstored alongside the child, so child compromise is contained.
:::

## Subcommand surface

```sh
mnemonic derive-child \
  --from <FROM> \
  --application <APPLICATION> \
  --length <LENGTH> \
  --index <INDEX>
```

The four required flags:

- **`--from`** — the master source. `--from xprv=<xprv>` (BIP-85
  canonical), `--from phrase="<bip39 phrase>"` (combined with
  `--passphrase` and `--language`), or `--from xprv=-` /
  `--from phrase=-` to read from stdin.
- **`--application`** — what kind of child secret to derive. Listed
  below.
- **`--length`** — application-specific size. Range varies; pass
  `--length 0` for `hd-seed` and `xprv` (length is irrelevant
  there).
- **`--index`** — hardened child index in `0..2^31`. Increment to
  derive sibling secrets.

## Applications

| `--application` | Output | `--length` |
|---|---|---|
| `bip39` | new BIP-39 phrase (9 BIP-85-coded languages) | `12`, `18`, `24` |
| `hd-seed` | 64-byte HD-seed bytes (raw) | `0` (sentinel) |
| `xprv` | child master xprv | `0` (sentinel) |
| `hex` | raw hex entropy | `16..=64` |
| `password-base64` | base64-shaped password | `20..=86` |
| `password-base85` | base85-shaped password | `10..=80` |
| `dice` | deterministic dice rolls | `1..=10000` |

The 6 in-scope BIP-85 applications map to BIP-85 codes 39', 2',
32', 128169', 707764', and 707785'. RSA and RSA-GPG (BIP-85 codes
828365' and 707785') are out-of-scope for v0.8 pending
RUSTSEC-2023-0071 patch and will land in v0.9.

## Worked examples

(Same canonical test seed as [Chapter 22](#your-first-bundle); see
the DANGER box there.)

### Derive a child BIP-39 phrase

```sh
mnemonic derive-child \
  --from phrase="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about" \
  --application bip39 \
  --length 12 \
  --index 0
```

Output: a fresh 12-word BIP-39 phrase, deterministically derived
from the master. Increment `--index` to derive sibling phrases.

### Derive a deterministic password

```sh
mnemonic derive-child \
  --from phrase="<your master phrase>" \
  --application password-base64 \
  --length 32 \
  --index 0
```

Output: a 32-character base64 password. Use `--index N` for the
N-th deterministic password.

### Derive a child xprv

```sh
mnemonic derive-child \
  --from phrase="<your master phrase>" \
  --application xprv \
  --length 0 \
  --index 0
```

Output: a child `xprv...` (or `tprv...` on testnet). Useful for
provisioning an air-gapped hardware wallet from a parent backup.

### Derive deterministic dice rolls

```sh
mnemonic derive-child \
  --from phrase="<your master phrase>" \
  --application dice \
  --length 100 \
  --index 0 \
  --dice-sides 6
```

Output: 100 deterministic d6 rolls. The `--dice-sides` flag is
required for `dice` and accepts values in `2..=2^32-1`.

## Cross-network derivation

For testnet-targeted children:

```sh
mnemonic derive-child \
  --from phrase="<phrase>" \
  --application xprv \
  --length 0 \
  --index 0 \
  --network testnet
```

Default is mainnet (matching BIP-85 §"Test Vectors"). Testnet
children emit `tprv...` for `xprv` and `c...` WIF for the
`hd-seed` application.

## When to use BIP-85 vs. multisig

- **BIP-85 child wallets** — operationally simpler, single backup.
  Compromise of *any* child reveals nothing about siblings, but
  compromise of the *master* compromises every child. Best suited
  for low-value or rotated-frequently child wallets.
- **Multisig** (covered in [chapters 32](#multi-source-2-of-3-multisig)
  and [33](#taproot-multisig)) — operationally complex, multiple
  cosigners, threshold-based. Best suited for high-value vaults
  where compromise of any single secret is recoverable.

The two compose: a multisig wallet whose cosigners' secrets are
themselves BIP-85 children of separate masters is a common
"deterministic-recovery + threshold" pattern.
