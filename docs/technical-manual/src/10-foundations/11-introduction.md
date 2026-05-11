# Introduction

The m-format constellation\index{m-format constellation} (or "m-format star") answers a specific question: *how do you durably back up a Bitcoin self-custody wallet on engravable media, in a form that survives decades of physical decay and human transcription error?*

A 24-word BIP-39 phrase, engraved on steel, restores a wallet's *keys*. It does not, on its own, restore a wallet's *spending rule* — the descriptor template, derivation paths, and (for multisig wallets) cosigner xpubs. For a single-sig BIP-84 wallet, the spending rule is conventional enough that recovery software can guess it. For arbitrary miniscript policies (taproot multisig, inheritance schemes, decaying conditions), the seed alone is insufficient.

The m-format constellation splits the backup across three independently-checksummed cards, plus an integration toolkit that composes them into a coherent bundle:

- **ms1** carries BIP-39 entropy (or a BIP-32 master seed). HRP `ms`. Crate `ms-codec`.
- **mk1** carries an xpub plus its BIP-32 origin (master fingerprint + derivation path). HRP `mk`. Crate `mk-codec`.
- **md1**\index{md1} carries a BIP-388\index{BIP-388} wallet-policy template (and, for self-custody, one bound xpub per cosigner slot). HRP `md`. Crate `md-codec`.
- The **`mnemonic-toolkit`**\index{mnemonic-toolkit} CLI synthesises the three cards from end-user inputs and verifies cross-card bindings on recovery. It does not engrave its own card.

Each card is independently BCH-checksummed: a damaged ms1 card decodes without needing the mk1 or md1 cards; the toolkit's cross-card invariants (the `policy_id_stub` carried on mk1 and computable from md1) verify *coherence* across the three cards once they are decoded.

## Who this manual is for

This manual targets two audiences in one volume:

1. **Implementers and auditors.** Engineers re-implementing one of the four codecs in another language, or auditing the wire format, BCH math, canonicality rules, or cross-card binding invariants. **Parts I–IV** are written at the depth needed to reproduce or independently verify everything observable on the wire.

2. **Rust integrators.** Engineers consuming `md-codec` / `mk-codec` / `ms-codec` / `mnemonic-toolkit` as library dependencies in their own wallet, hardware signer, or coordinator codebase. **Part V** documents the public API surface — types, functions, feature flags, error taxonomies, and integration patterns.

The two halves cross-reference each other through the back-matter glossary (§61) and index (§62). A wire-format chapter in Part II naming a Rust type by its on-wire shape (e.g., `Body::MultiKeys`) links forward to the corresponding API chapter in Part V; a Part V chapter documenting an error variant (e.g., `Error::NUMSSentinelConflict`) links back to the canonicality rule in Part II that surfaces it.

## How to navigate

Most readers should read Part I in order — it establishes the BCH math (§I.3) and the wire-format notation conventions (§I.4) that the rest of the manual relies on without re-explaining.

After Part I, the parts can be read in any order. Cross-references between chapters use the form **§II.1.3** (= Part II, chapter 1, section 3); back-matter sections use **§61–§66**.

## Relationship to other documents

| Document | What it is | Where to find it |
|---|---|---|
| md1 BIP draft | Formal protocol specification for the descriptor card | `bg002h/descriptor-mnemonic/bip/bip-mnemonic-descriptor.mediawiki` |
| Per-version SPECs | "Why we did it this way at version X" for each repo | `bg002h/<repo>/design/SPEC_*.md` |
| End-user manual | Workflow walkthroughs, CLI quick reference | `bg002h/mnemonic-toolkit/docs/manual/` |
| **This manual** | Bit-level wire format + Rust API reference across all four repos | `bg002h/mnemonic-toolkit/docs/technical-manual/` |
| Crate `cargo doc` | Auto-generated Rust API docs | `docs.rs/md-codec`, etc. |

The BIPs and per-version SPECs are *authoritative* for protocol claims. This manual cites them; it does not replace them.

## Version coverage

This manual tracks `mnemonic-toolkit` `main` and the sibling repos' latest releases:

| Repo | Version covered at this manual cut |
|---|---|
| `mnemonic-toolkit` | v0.8.0 + post-v0.8 main |
| `descriptor-mnemonic` (md-codec / md-cli) | md-codec v0.32.0, md-cli v0.4.3 |
| `mnemonic-key` (mk-codec / mk-cli) | mk-codec v0.2.2, mk-cli v0.3.0 |
| `mnemonic-secret` (ms-codec / ms-cli) | ms-codec v0.1.1, ms-cli v0.1.1 |

The release-history table in §63 records the version covered by each tech-manual cut.

## Status

**Pre-Draft, AI + reference implementation, awaiting human review.** Every wire-format claim, BCH-math claim, canonicality rule, and cross-card invariant in this manual may be wrong. Cross-implementation work — re-implementing one of the codecs in another language and reproducing the corpus vectors — is the most valuable bug-finding activity at this stage and is *especially* welcome.

If you find a wrong claim, file an issue against the relevant repository.
