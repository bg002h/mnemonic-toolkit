# About this manual

The m-format constellation is a family of four sibling Bitcoin self-custody backup formats — **md1**, **mk1**, **ms1**, and the `mnemonic-toolkit` integration layer — that engrave together as a coherent steel-engravable bundle. (See §I.1 for the definitional treatment with its index marker.)

This **technical manual** is the companion to the end-user manual (`docs/manual/`). Where the end-user manual walks newcomers through workflows and the CLI surface, the technical manual answers the deeper questions:

- What is on the wire for an md1 / mk1 / ms1 card, bit for bit?
- How does the BCH error-correcting math work across the (forked) md1↔mk1 plumbing and the BIP-93-direct ms1 layer?
- How does the toolkit compose a coherent three-card bundle, and how do the anti-collision invariants compose?
- How does address derivation flow from a wallet-policy template, through miniscript, to a Bitcoin address?
- What is the public Rust API surface of each crate?

> **MIT License.** This manual is freely redistributable under MIT terms. See the `LICENSE` file in the toolkit repository.

The manual is built from sources in `mnemonic-toolkit/docs/technical-manual/src/` via pandoc + xelatex. Each release cut (`tech-manual-vX.Y.Z`) attaches a fresh `m-format-technical-manual.pdf` to the corresponding GitHub release.

## Who is this manual for?

Two audiences sharing one volume:

- **Implementers and auditors.** Engineers re-implementing one of the four codecs in another language, or auditing the wire format, BCH math, canonicality rules, or cross-card binding invariants. Parts I–IV are written at the depth needed to reproduce or independently verify everything observable on the wire.
- **Rust integrators.** Engineers consuming `md-codec` / `mk-codec` / `ms-codec` / `mnemonic-toolkit` as library dependencies. Part V documents the public API surface — types, functions, feature flags, error taxonomies, and integration patterns — at the depth needed to integrate the constellation into a wallet, hardware signer, or coordinator.

A reader from either audience may skim the other half; the parts are interlinked through the back-matter glossary and index. The book is one volume, not two.

## What the technical manual is *not*

- **Not a BIP.** Per-format BIPs (md1 BIP draft at `bg002h/descriptor-mnemonic/bip/`, the future mk1 and ms1 BIP drafts) are authoritative for protocol claims. This manual cites them; it does not replace them.
- **Not a tutorial.** Workflow walkthroughs (creating a bundle, recovering from cards, exporting watch-only) live in the end-user manual.
- **Not a design-rationale archive.** Per-version SPECs in each repo's `design/` are the authoritative "why we did it this way at version X." This manual references them; it does not duplicate them.

## Navigating the manual

- **Part I — Foundations** orients the reader to the four-format star, BCH/codex32 math, and the bit/byte notation used throughout. Read this first regardless of audience.
- **Part II — Wire formats** documents md1, mk1, and ms1 at bit-level depth. Implementer-shaped; Rust integrators may skim.
- **Part III — Address derivation** covers the descriptor → miniscript → address pipeline including the v0.32 shape-coverage extension.
- **Part IV — Bundle formation** documents the three-card bundle envelope, anti-collision invariants, and the K-of-N share future.
- **Part V — Rust API reference** is one chapter per crate; integration-shaped.
- **Back matter** is the glossary, index, release history, BIP cross-reference table, troubleshooting matrix, and bibliography.

For a quick orientation to the four-format star, see §I.2.
