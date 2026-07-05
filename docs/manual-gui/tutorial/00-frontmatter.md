# The `mnemonic-gui` Tutorial — Worked Journeys

_This front matter is a scaffold; the editorial pass fills the prose._

This book walks five end-to-end **journeys** through the `mnemonic-gui`
desktop application — from a single-signature card set to a four-tier
degrading vault and a Taproot twin — using nothing but the GUI's own
forms, Run button, and output pane. Every screenshot and every captured
transcript in this book is machine-generated from the pinned
`mnemonic-gui` release and byte-checked in CI; nothing here is
hand-pasted.

It is the companion to the reference manual (the per-tab, per-subcommand
`m-format` GUI manual): where the reference manual documents every flag,
this book shows a reader driving whole workflows form by form.

> **This is a scaffold build.** Chapter and step narratives are
> placeholders that the editorial pass completes. The figures and
> transcripts embedded below are the final, gated corpus.

## Conventions

_To be written in the editorial pass — the no-titlebar caveat, the
viewport-faithful scrolling contract, the output-pane top-slice, the
demo-seed baseline, and the version-skew callout._

## The demo seed — never fund it

Every journey uses **public** `BIP-39` test vectors. Any wallet derived
from them has been swept by chain watchers. **Never engrave them. Never
fund them.** They exist only so this book (and its CI corpus) is fully
reproducible.
