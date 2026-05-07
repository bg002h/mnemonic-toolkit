# Welcome to the m-format star

Self-custodying Bitcoin starts with one secret — your seed phrase —
and an inevitable question: *how do I keep that intact for decades?*
The m-format star is a family of four sibling formats that answer
the question by splitting the backup into independently-checksummed,
steel-engravable cards.

## The four siblings

```mermaid
flowchart LR
  toolkit["mnemonic-toolkit<br/>(integration layer)"]:::layer
  ms1["ms1<br/>(secret card)"]:::secret
  mk1["mk1<br/>(key card)"]:::key
  md1["md1<br/>(descriptor card)"]:::desc

  toolkit -- "synthesises" --> ms1
  toolkit -- "synthesises" --> mk1
  toolkit -- "synthesises" --> md1
  ms1 -. "BIP-39 entropy" .-> phrase[(seed phrase)]
  mk1 -. "xpub + origin" .-> wallet[(watch-only wallet)]
  md1 -. "wallet policy" .-> wallet

  classDef layer fill:#fef3c7,stroke:#92400e,stroke-width:2px
  classDef secret fill:#fee2e2,stroke:#991b1b,stroke-width:2px
  classDef key fill:#dbeafe,stroke:#1e40af,stroke-width:2px
  classDef desc fill:#dcfce7,stroke:#166534,stroke-width:2px
```

| Card | Carries | What it answers |
|---|---|---|
| **ms1\index{ms1}** | BIP-39 entropy | "What were the seed words?" |
| **mk1\index{mk1}** | xpub + origin (master fingerprint + BIP path) | "What public key did the wallet use, and at which derivation?" |
| **md1\index{md1}** | wallet policy (template + bound xpub) | "What was the wallet's *spending rule* — single-sig? 2-of-3 multisig? taproot?" |

A fourth piece — the **`mnemonic-toolkit`\index{mnemonic-toolkit}**
CLI — is the *integration layer* that synthesises a coherent bundle
from your inputs and verifies the cross-bindings. It does not engrave
its own card; it produces the three card strings (ms1, mk1, md1) plus
the visual engraving cards.

## Why three cards instead of one

A single seed phrase, engraved on steel, recovers a wallet *if and
only if* you also remember the wallet's spending rule. For a single-
sig BIP-84 wallet, that's implicit: most wallet apps will guess. For
a multisig wallet — a 2-of-3 with two cosigners' xpubs and a
taproot internal key — the seed alone is insufficient. You need:

- the **secret** (ms1) to sign,
- the **public keys** (mk1 cards from each cosigner) to construct the
  multisig script,
- the **policy** (md1) to know *what kind* of multisig.

Engraving all three on independent steel cards means each card can be
lost, water-damaged, or partially destroyed without compromising the
whole. Each carries its own BCH error-correction\index{BCH} checksum,
so partial damage is locatable, not catastrophic.

:::primer
**Cross-binding, in one sentence.** Each mk1 card carries a 4-byte
`policy_id_stub`\index{policy\_id\_stub} — `SHA-256(canonical wallet-policy
preimage)[0..4]` — that is also computable from each md1 card. If you
mix cards from different wallets, the stubs disagree and
`mnemonic verify-bundle` fails fast.
:::

## How to use this manual

Most readers want one of three things:

1. **"Show me what to do, fast."** Read [Part II — Quick start]
   (#your-first-bundle). 30 minutes, ends with a working bundle on
   paper.
2. **"I have this *specific* situation: 2-of-3 multisig with three
   cosigners on three machines."** Read [Part III —
   Guided workflows](#multi-source-2-of-3-multisig). Eight end-to-end
   recipes covering single-sig, multisig, taproot, watch-only,
   recovery, migration, wallet export, and BIP-85 children.
3. **"I want to know how this compares to SLIP-39 / Shamir / naked
   BIP-39."** Read [Part V — Comparing & contrasting]
   (#m-format-star-vs-slip-39-vs-naked-bip-39-vs-shamir).

The CLI reference (Part IV) and the appendices (Part VI) are
consulted, not read top-to-bottom.

## Status

This manual is **v0.1**, tracking `mnemonic-toolkit` `main` (initial
sync targets toolkit `v0.8.0`). Each new toolkit tag triggers a fresh
PDF build attached to the corresponding GitHub release. The
[release-history appendix](#appendix-h-release-history) summarises
what each toolkit version added or broke; the toolkit's own
`CHANGELOG.md` is the authoritative source.
