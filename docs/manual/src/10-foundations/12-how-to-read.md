# How to read this manual

You don't need to read straight through. The structure is designed so
power users can skim, newcomers can fill in gaps from the appendices,
and reference users can jump to the index.

## The two-track design

Two kinds of reader, one document. Most chapters address Bitcoin
power users by default — the prose assumes you already know what a
BIP-39 phrase is, why a wallet has a derivation path, and what
"multisig" means at the script level. When a chapter would otherwise
need to detour into background, it does so inside a fenced
`:::primer` box like this:

:::primer
A short paragraph (≤80 words) explaining just enough background for
the chapter's main flow. Power users skip these. The deeper version
is in the appendix the box links to.
:::

If you find the primer boxes useful, you're a newcomer. Read them
all and supplement with the matching appendix when a topic clicks
incompletely. If you find them noise, ignore them and read the main
flow.

## A separate kind of box for warnings

When a chapter introduces or first uses a value that is *publicly
known* and therefore unsafe to engrave, it opens with a `:::danger`
admonition. Throughout this manual, every example uses the canonical
BIP-39 test vector
`abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about` —
which is **public** and has been swept by chain watchers. Each
example chapter restates that warning on its first use.

If you copy an example *literally*, you will derive addresses that
have already been emptied. **Never engrave the canonical test seed.
Never fund it.** Generate fresh entropy for your real wallets.

## Six parts, six purposes

| Part | Purpose | Read when |
|---|---|---|
| I — Foundations | Establish the four-card mental model and this manual's conventions. | First. |
| II — Quick start | Install the toolkit; produce and verify your first bundle. | After Part I, before anything else. |
| III — Guided workflows | Eight end-to-end recipes: single-sig, multisig, taproot, watch-only, recovery, migration, wallet export, BIP-85 children. | Pick the one that matches what you're building. |
| IV — CLI reference | Per-binary, per-subcommand flag reference for `mnemonic`, `md`, `ms`, plus a Rust-API tour of `mk-codec`. | Consult by command, not by reading. |
| V — Comparing & contrasting | Decision-oriented chapters: when to use which format, which descriptor variant, which passphrase channel. | When two features look similar and you need to choose. |
| VI — Appendices | Glossary; newcomer primers for BIP-39 / BIP-32 / descriptors / codex32; test-seed handling; troubleshooting; release history; index. | Reference. |

## Cross-references

Every chapter links forward and backward. Internal links point to the
target's section anchor; the markdown render resolves these inside
the concatenated `m-format-manual.md`, and the PDF render resolves
them as hyperlinked TOC entries.

The PDF includes a true page-numbered alphabetical index built from
inline `\index{}` markers throughout the source. The markdown render
strips those markers; instead, [Appendix I](#appendix-i-index-of-terms)
ships a curated `Term → §section` table that is held in lockstep with
the LaTeX index by the lint script.

:::primer
**If you write contributions back to this manual.** See
`docs/manual/AUTHORING.md` in the toolkit repository. It documents
the convention for `:::primer` / `:::danger` boxes, inline
`\index{}` placement and its mirror requirement on
[Appendix I](#appendix-i-index-of-terms), the canonical-test-seed
DANGER policy, glossary discipline, and the pre-commit lint
expectations.
:::
