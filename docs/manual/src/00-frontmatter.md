# About this manual

The m-format constellation\index{m-format constellation} is a family of four sibling
Bitcoin self-custody backup formats that engrave together as a coherent
steel-engravable bundle. This manual is the end-user companion to the
four formats and to the `mnemonic-toolkit` integration tool.

> **MIT License.** This manual is freely redistributable under MIT terms.
> See the `LICENSE` file in the source repository.

This is **manual v0.1**, tracking `mnemonic-toolkit` `main` (initial
sync targets toolkit `v0.8.0`). The manual is a *living document*: each
toolkit tag triggers a fresh PDF build attached to the corresponding
GitHub release.

## Who is this manual for?

The manual is **two-track**:

- **Bitcoin power users** — readers familiar with BIP-39 mnemonic
  phrases, BIP-32 derivation paths, descriptors, and multisig — read
  the main flow of every chapter and skip the `:::primer` boxes.
- **Newcomers** — readers who know what a Bitcoin wallet is but have
  not engraved a steel backup, do not yet know what BIP-388 is, or
  have never set up multisig — read the `:::primer` boxes for the
  background a chapter assumes, and consult Part VI's deep-dive
  appendices for full primers.

If you do not yet know what BIP-39, BIP-32, descriptors, or multisig
*are*, start with [Appendix B (BIP-39)](#appendix-b-bip-39-entropy-primer),
[Appendix C (BIP-32)](#appendix-c-bip-32-derivation-primer), and
[Appendix D (descriptors / BIP-388)](#appendix-d-descriptors-and-bip-388-primer).
Then come back to the front and read forward.

## How to read this manual

The chapters are divided into six parts:

| Part | Chapters | Read order |
|---|---|---|
| I — Foundations | Welcome, navigation, concept signposts | Read first. |
| II — Quick start | Install, first bundle, verify, recover | Read next; gives a working bundle in under 30 minutes. |
| III — Guided workflows | 8 end-to-end recipes (single-sig, multisig, taproot multi, watch-only, recovery, migration, wallet export, BIP-85 children) | Skim the table of contents and read the workflows that match what you're building. |
| IV — CLI reference | `mnemonic`, `md`, `ms`, `mk-codec` | Reference; consult per command. |
| V — Comparing & contrasting | 7 decision-oriented chapters | Read to choose between overlapping features. |
| VI — Appendices | Glossary + 4 newcomer primers + test seeds + troubleshooting + release history + index | Mix of reference and primer material. |

Every example in this manual uses the canonical BIP-39 test vector
`abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about`.
This phrase is **public** and any wallet derived from it has been
swept by chain watchers. **Never engrave it. Never fund it.** Each
example chapter opens with a `:::danger` admonition restating this.

## Versioning and releases

The manual is a *living document*. Each `mnemonic-toolkit` GitHub
release attaches a freshly-built PDF asset
(`m-format-manual-toolkit-${TOOLKIT_VERSION}.pdf`). A breaking content
change to the manual itself bumps `manual-MAJOR`; new chapters bump
`manual-MINOR`; copyedits and typo fixes bump `manual-PATCH`. Independent
of toolkit semver.
