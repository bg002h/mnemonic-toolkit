# `mnemonic` — per-tab reference

The `mnemonic` tab is the largest and most-used surface of the GUI:
30 subcommands covering bundle emission, bundle verification, format
conversion, watch-only wallet export and import, descriptor building,
xpub-search, address derivation, BIP-85 child derivation, BIP-39
last-word completion, Coldcard-compatible seed-XOR splitting,
SLIP-0039 K-of-N share-splitting, SeedQR encode/decode, nostr and
silent-payment derivation, and message verification. The GUI exposes
each subcommand as its own form on the same tab; the subcommand
selector at the top of the form switches between them.

## Subcommand index

The subcommands group naturally into five families:

- **Bundle emission and verification.** The headline three-card
  bundle workflow: emit one bundle from a seed phrase, verify a
  round-trip from cards back to seed.
  - [`mnemonic bundle`](#mnemonic-bundle)\index{mnemonic bundle} —
    emit one `ms1` + one or more `mk1` + one `md1` from a master
    seed plus optional cosigner xpubs.
  - [`mnemonic verify-bundle`](#mnemonic-verify-bundle)\index{mnemonic verify-bundle}
    — round-trip a card set: from `ms1` (+ optional cosigner inputs)
    re-emit the bundle and assert byte-identical match against the
    inputs.
- **Format conversion and wallet export.** The two non-bundle
  output paths.
  - [`mnemonic convert`](#mnemonic-convert)\index{mnemonic convert}
    — convert one secret-or-public input to one or more public
    outputs (`phrase` → `ms1`, `xpub` → `mk1`, etc.; the matrix is
    in the `--from` / `--to` cross-product).
  - [`mnemonic export-wallet`](#mnemonic-export-wallet)\index{mnemonic export-wallet}
    — emit a watch-only wallet descriptor (BIP-388 wallet policy
    plus key sources) for import into Sparrow, Specter, etc.
- **Hierarchical derivation.** The BIP-85 child-secret family.
  - [`mnemonic derive-child`](#mnemonic-derive-child)\index{mnemonic derive-child}
    — derive a child secret (BIP-39 phrase, WIF, hex entropy, xprv)
    from a parent `ms1` or `phrase` per BIP-85.
- **Phrase repair and completion.**
  - [`mnemonic final-word`](#mnemonic-final-word)\index{mnemonic final-word}
    — given an N-1 word BIP-39 partial, emit the lexicographically
    sorted set of last words that yield a valid checksum.
- **Share splitting (Coldcard + Trezor families).**
  - [`mnemonic seed-xor-split`](#mnemonic-seed-xor-split)\index{mnemonic seed-xor-split}
    — Coldcard-compatible BIP-39 ↔ BIP-39 all-or-nothing XOR
    splitter (every share required to recover the master).
  - [`mnemonic seed-xor-combine`](#mnemonic-seed-xor-combine)\index{mnemonic seed-xor-combine}
    — combine N seed-XOR shares back into the master phrase.
  - [`mnemonic slip39-split`](#mnemonic-slip39-split)\index{mnemonic slip39-split}
    — Trezor-compatible SLIP-0039 K-of-N threshold splitter (any K
    of N shares recover the master).
  - [`mnemonic slip39-combine`](#mnemonic-slip39-combine)\index{mnemonic slip39-combine}
    — combine K SLIP-39 shares back into the master secret.

The five families above describe what each subcommand *does*, not
what its form looks like in the GUI. Each subcommand is documented
on its own with form-shape diagrams, per-flag reference, and at
least one worked example.

## Form shape — what every subcommand has in common

All 30 subcommands render through the same form scaffolding
described in chapter 31: a top-of-form `Pinned: mnemonic 0.13.0`
label with a subcommand selector ComboBox and per-subcommand `?`
help-icon; per-flag widgets (text fields, dropdowns, checkboxes,
slot editors); a `Slot rows:` section if the subcommand accepts
the `--slot` repeating flag (currently `bundle`, `verify-bundle`,
and `export-wallet`); an action bar with **Copy command (POSIX)**,
**Copy command (Windows)**, and **Run** buttons; an always-on
`Preview:` line.

**Most** of the ten subcommands consume secret-bearing inputs at
realistic form fillings — a master `ms1`, a `--passphrase`, a
`--share`, or a `--from` whose chosen node is in the secret class
(`phrase`, `entropy`, `xprv`, `wif`, `ms1`, `bip38`, `electrum-phrase`).
Under the `should_confirm_run` predicate at
`mnemonic-gui/src/secrets.rs:80-105` (any `secret: true` flag value,
any secret-class slot subkey, any secret-class NodeValueComposite
node), all ten can fire the modal under at least one valid form
input. `export-wallet` is the only one *intended* to be filled
exclusively with public material (xpub-only slots, no `--passphrase`)
and is therefore the most likely to fire **Run** without the modal
in practice. The threat-model warning in [§14 Defense
2](#secret-handling) about the v0.3.0 modal-redaction gap and the
recommended cold-node operational mitigation applies to **every**
secret-bearing invocation under this tab.

## Worked-example seed convention

Every chapter under this tab uses the canonical all-`abandon`
BIP-39 test vector
`abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about`
as the input phrase wherever a seed is required. This phrase is
**public** and any wallet derived from it has been swept by chain
watchers since 2017 — its use in the manual is for round-trip
demonstration only. Each chapter that uses the phrase opens with
a `:::danger` admonition restating this; do not engrave or fund
any wallet derived from it.

## Where to read next

If you are following the cycle from chapter 30 (the tour), pick a
subcommand from the index above and read its chapter end-to-end.
If you arrived via a `?` help-icon click in the GUI, you are
already at the right anchor; scroll up for the chapter outline,
down for the per-flag detail. Cross-references between chapters
are explicit (`see [`mnemonic bundle`](#mnemonic-bundle)`) and
deep-link into the per-flag anchors where relevant.
