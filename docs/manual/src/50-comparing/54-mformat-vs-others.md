# m-format constellation vs SLIP-39 vs naked BIP-39 vs Shamir

Bitcoin self-custody has a small handful of mature seed-backup
standards. None is universally best; each has a use case for which
it was designed. This chapter compares the four most common: naked
BIP-39, the m-format constellation, SLIP-39 / Shamir-style splits, and
codex32 alone (without a card-bundle wrapper).

The intent here is **scope** rather than *ranking*. A reader's right
choice depends on threat model, recovery skills, and the wallet's
target lifetime.

## Side-by-side

| | Naked BIP-39 | m-format constellation | SLIP-39 / Shamir | codex32 alone |
|---|---|---|---|---|
| Per-character checksum | no | yes (BCH) | partial | yes (BCH) |
| Engraving-friendly alphabet | no (English wordlist) | yes (32-char) | partial | yes (32-char) |
| K-of-N share splitting | no | v0.2 (planned) | yes | yes |
| Wallet-policy binding | no | yes (md1) | no | no |
| Multisig coordination | external | yes (multi-source) | no | no |
| Library availability | wide (every wallet) | narrow (toolkit) | medium (Trezor, others) | medium (rust-codex32) |
| Hardware wallet first-class | yes | no (yet) | yes (Trezor) | no |
| Ceremony complexity | low | medium-high | medium | low |
| BIP standardisation | BIP-39 | none yet (BIP-draft) | SLIP-39 (Trezor) | BIP-93 |

## When each fits

**Naked BIP-39** — the path of least resistance for new users, with
a hardware-wallet ecosystem that already speaks it. The cost is
brittleness: no per-character checksum, no policy binding, and steel
engravings of 12-24 words have failed in the field due to
mis-stamping.

**The m-format constellation** — the natural fit when (a) the wallet is
multisig or non-default-template, (b) the backup must be
self-describing (template + xpubs travel with the seed), and (c)
the user values BCH error correction over hardware-wallet support.
The cost is operational complexity: three plates, a ceremony, and a
toolkit dependency.

**SLIP-39 / Shamir-style splits** — the natural fit when the goal
is *threshold secret-sharing* of a single seed across many parties
or locations, *and* the wallet is single-sig. SLIP-39 specifies the
share-encoding scheme; many hardware wallets implement it. The cost
is that SLIP-39 doesn't carry policy or xpub binding, so multisig
recovery still needs an out-of-band descriptor.

**codex32 alone** — the natural fit when the only need is "BIP-39
entropy with BCH error correction, no multisig, no descriptor." This
is essentially the m-format constellation's ms1 card without the bundle.
Useful for ms1-only pipelines (paper wallets, hardware-wallet
firmware) where the toolkit is overkill.

## Composability

The four standards are not mutually exclusive. Useful combinations
in production:

- **m-format constellation + hardware wallet.** The hardware wallet handles
  daily signing from BIP-39; the m-format bundle backs up the seed
  *and* the policy binding for disaster recovery.
- **SLIP-39 + the m-format mk1+md1 cards.** SLIP-39 splits the
  secret across N locations; the mk1 + md1 cards provide the
  watch-only side without a full toolkit dependency.
- **Naked BIP-39 + paper-walleted codex32.** The codex32 ms1 card
  serves as the verification mirror for a BIP-39 phrase engraved
  elsewhere; mismatches between the two surface as decode errors.

## Choosing for the long term

For a wallet meant to outlive the operator (estate planning, family
trusts), favour standards that are widely re-implemented and
unlikely to disappear:

| Likelihood of long-term software support | Standard |
|---|---|
| highest | BIP-39 (universal) |
| high | SLIP-39 (Trezor + others) |
| high | BIP-93 codex32 (BIP-tracked) |
| medium | m-format constellation (one toolkit, BIP-draft underway) |

A hybrid approach — e.g., naked BIP-39 phrase plus an m-format
bundle as the "self-describing" supplement — minimises the bet on
any one tool's continued maintenance.
