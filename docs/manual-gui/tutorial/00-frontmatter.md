# Introduction

This book walks five end-to-end **journeys** through the `mnemonic-gui`
desktop application — from a single-signature card set to a four-tier
degrading vault, a Taproot twin, and a watch-only export — using
nothing but the GUI's own forms, **Run** button, and output panel. No
shell, no piping, no hand-typed command lines: you select a
subcommand, fill a few fields, click **Run**, and read the panel.

It is the companion to the reference manual (the per-tab,
per-subcommand `m-format` GUI manual). Where the reference manual
documents every flag in isolation, this book shows a reader driving
whole wallet workflows, form by form. It also mirrors the command-line
`Examples.pdf` tutorial: the same five wallets, the same fingerprints
and descriptors, taught as a point-and-click walkthrough instead of a
shell session.

Every screenshot and every captured transcript in this book is
machine-generated from the pinned `mnemonic-gui` release, driving the
real application window against the pinned `mnemonic 0.75.0`
command-line tier, and byte-checked in CI. Nothing here is hand-drawn
or hand-pasted; if the application changed, these pages would fail to
build.

The Orientation chapter explains how to read the book — the window
layout, the two-shots-per-step convention, the reproducibility
guarantee, and the secret-hygiene rules. The five journeys follow.

## The demo seed — never fund it

Every journey uses **public** `BIP-39` test vectors — the world-known
`abandon abandon … about` phrase and two of its published siblings.
Their fingerprints (`73c5da0a`, `b8688df1`, `28645006`), xpubs, and
descriptors are the same values printed throughout `Examples.pdf`. Any
wallet derived from them has been swept by chain watchers.

> **Never engrave these phrases. Never fund them. Never type a real
> seed into a networked machine.** They exist here only so the book —
> and its CI corpus — is fully reproducible. When you run these
> journeys for real, you substitute your own seed; the GUI masks it in
> every field, preview, modal, and output line, exactly as the
> Orientation chapter describes.
