# The three cards: ms1, mk1, md1

The m-format constellation maps the three Bitcoin concepts from the previous
chapter onto three cards, one per concept.

## One card per concept

- **`ms1` — the secret card.** Carries the random bits behind your
  seed phrase (the BIP-39 *entropy*, before the words). With the
  `ms1` card alone you can regenerate the same seed phrase your
  wallet was created from.
- **`mk1` — the key card.** Carries one xpub plus its *origin* — a
  4-byte master-fingerprint identifier and the BIP-32 path used to
  derive the xpub. (The fingerprint and path together pin the xpub
  to a specific position in the master tree.) With one or more
  `mk1` cards you can rebuild the public side of a wallet.
- **`md1` — the descriptor card.** Carries the wallet's spending
  rule as a BIP-388 wallet policy — a *template* (e.g.
  `wsh(sortedmulti(2,@0/**,@1/**,@2/**))`) plus *one* bound xpub
  for the slot this card represents. With the `md1` card you know
  what kind of wallet to recover into.

## What each card answers

| Card | Carries | What it answers |
|---|---|---|
| **`ms1`** | BIP-39 entropy | "What were the seed words?" |
| **`mk1`** | xpub + origin (master fingerprint + BIP path) | "What public key did the wallet use, and at which derivation?" |
| **`md1`** | wallet policy (template + one bound xpub) | "What was the wallet's *spending rule* — single-sig? 2-of-3 multisig?" |

For a **single-signature** wallet you produce one of each: one
`ms1`, one `mk1`, one `md1`. For a **2-of-3 multisig** the secret
holders each get their own `ms1` card on their own machine; each
cosigner contributes one `mk1` (their xpub); the coordinator
produces a single `md1` describing the 2-of-3 policy.

## Why three cards instead of one

A naked seed phrase recovers a single-sig wallet *if and only if*
the recovery software can guess the spending rule (BIP-84 native
segwit, in practice). For a multisig wallet — say a 2-of-3 with
three cosigners' xpubs and a custom path — the seed alone is
insufficient. You need the **secret** (`ms1`) to sign, the
**public keys** (one `mk1` per cosigner) to construct the multisig
script, and the **policy** (`md1`) to know *what kind* of multisig.

Splitting the backup across three independently-checksummed steel
cards gives two distinct kinds of resilience.

**Per-card BCH error correction.** Each card carries its own
codex32-pattern BCH checksum. Per BIP-93 §"Error Correction", the
guarantees per card are:

- **detection:** any error affecting up to 8 characters per card,
  guaranteed (random patterns beyond that miss with probability
  < 3 × 10⁻²⁰);
- **correction (substitutions):** up to 4 wrong characters at
  unknown positions can be reconstructed;
- **correction (erasures):** up to 8 characters at known positions
  (e.g., illegible smudges) can be reconstructed; or up to 13 in
  a single consecutive run (a scratch).

An erasure means *the position is preserved, the value is
unknown*. **Deletions and insertions** (length-changing damage)
are outside this model and are caught by length-check rather than
BCH math — count the characters on a plate before decoding.

**Cross-card recovery (asymmetric).** Public material (`mk1`,
`md1`) is re-derivable from a surviving `ms1` plus knowledge of
the wallet template. A lost or destroyed `ms1`, however, **cannot
be reconstructed** from the public cards alone — those degrade to
a watch-only wallet. For 2-of-3 multisig, the threshold adds
further redundancy: any one cosigner's `ms1` may be lost without
losing spending capability; two cannot. The reference manual's
recovery-paths chapter walks through every scenario; appendix E
of the manual cites the BIP-93 numbers in full.

The toolkit verifies that the three cards belong together via a
small fingerprint called the **policy ID stub**: a 4-byte hash of
the wallet policy that each `mk1` and `md1` card carries at encode
time, so mixing cards from different wallets is caught immediately.
If the stubs disagree, `mnemonic verify-bundle` fails fast.

Onward: install the toolkit and produce your first single-sig
bundle.
