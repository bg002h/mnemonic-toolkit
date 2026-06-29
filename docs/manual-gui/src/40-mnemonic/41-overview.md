# `mnemonic` — per-tab reference

The `mnemonic` tab is the largest and most-used surface of the GUI:
**32 subcommands** spanning bundle emission and verification, format
conversion, watch-only wallet export and import, descriptor building
and restore, xpub-search, address derivation, BIP-85 child derivation,
BIP-39 last-word completion, Coldcard-compatible seed-XOR splitting,
SLIP-0039 and codex32 (BIP-93) K-of-N share-splitting, SeedQR
encode/decode, Nostr and silent-payment derivation, message
verification, address decoding, cost comparison, steel-engravable
Word-Card encoding, man-page generation, and card
inspection / repair / Electrum decryption. The GUI exposes each
subcommand as its own form on the same tab; the subcommand selector at
the top of the form switches between them.

## Subcommand index

The 32 subcommands group naturally into nine families. Each entry
deep-links to the subcommand's chapter; read any chapter end-to-end
for its form-shape diagram, per-flag reference, and worked example.

- **Bundle emission and verification.** The headline three-card
  workflow.
  - [`mnemonic bundle`](#mnemonic-bundle)\index{mnemonic bundle} —
    emit one `ms1` + one or more `mk1` + one `md1` from a master seed
    plus optional cosigner xpubs.
  - [`mnemonic verify-bundle`](#mnemonic-verify-bundle)\index{mnemonic verify-bundle}
    — round-trip a card set: re-emit from `ms1` (+ optional cosigner
    inputs) and assert a byte-identical match.
  - [`mnemonic restore`](#mnemonic-restore) —
    reconstruct a wallet (and, for keyless templates, complete it from
    your own seed) from an `md1` descriptor card.
- **Format conversion and wallet export/import.** The non-bundle
  output and intake paths.
  - [`mnemonic convert`](#mnemonic-convert)\index{mnemonic convert} —
    convert one secret-or-public input to one or more public outputs.
  - [`mnemonic export-wallet`](#mnemonic-export-wallet)\index{mnemonic export-wallet}
    — emit a watch-only wallet descriptor for Sparrow / Specter / etc.
  - [`mnemonic import-wallet`](#mnemonic-import-wallet)
    — ingest a third-party BSMS or Bitcoin Core wallet blob.
  - [`mnemonic build-descriptor`](#mnemonic-build-descriptor)
    — assemble a BIP-388 descriptor from an archetype + key material.
- **Hierarchical derivation.** The BIP-85 child-secret family.
  - [`mnemonic derive-child`](#mnemonic-derive-child)\index{mnemonic derive-child}
    — derive a child secret (phrase, WIF, hex entropy, xprv) from a
    parent `ms1` or `phrase` per BIP-85.
- **xpub-search and address derivation.** Recover an unknown account /
  passphrase / path, or derive addresses.
  - [`mnemonic xpub-search-account-of-descriptor`](#mnemonic-xpub-search-account-of-descriptor)
    — find the account index that reproduces a known descriptor.
  - [`mnemonic xpub-search-passphrase-of-xpub`](#mnemonic-xpub-search-passphrase-of-xpub)
    — find the BIP-39 passphrase that reproduces a known xpub.
  - [`mnemonic xpub-search-path-of-xpub`](#mnemonic-xpub-search-path-of-xpub)
    — find the derivation path that reproduces a known xpub.
  - [`mnemonic xpub-search-address-of-xpub`](#mnemonic-xpub-search-address-of-xpub)
    — find the chain/index that produces a known address from an xpub.
  - [`mnemonic addresses`](#mnemonic-addresses)
    — derive a range of receive/change addresses from a seed or xpub.
- **Phrase repair and completion.**
  - [`mnemonic final-word`](#mnemonic-final-word)\index{mnemonic final-word}
    — given an N-1 word BIP-39 partial, emit the candidate Nth words
    that yield a valid checksum.
- **Share splitting (Coldcard / Trezor / codex32 families).**
  - [`mnemonic seed-xor-split`](#mnemonic-seed-xor-split)\index{mnemonic seed-xor-split}
    — Coldcard-compatible all-or-nothing XOR splitter.
  - [`mnemonic seed-xor-combine`](#mnemonic-seed-xor-combine)\index{mnemonic seed-xor-combine}
    — combine N seed-XOR shares back into the master phrase.
  - [`mnemonic slip39-split`](#mnemonic-slip39-split)\index{mnemonic slip39-split}
    — Trezor-compatible SLIP-0039 K-of-N threshold splitter.
  - [`mnemonic slip39-combine`](#mnemonic-slip39-combine)\index{mnemonic slip39-combine}
    — combine K SLIP-39 shares back into the master secret.
  - [`mnemonic ms-shares-split`](#mnemonic-ms-shares-split)\index{mnemonic ms-shares-split}
    — codex32 (BIP-93) K-of-N splitter emitting `ms1`-format shares.
  - [`mnemonic ms-shares-combine`](#mnemonic-ms-shares-combine)\index{mnemonic ms-shares-combine}
    — recombine ≥K codex32 shares into the recovered secret.
- **Alternate-key and address utilities.**
  - [`mnemonic nostr`](#mnemonic-nostr) — derive
    Nostr `nsec` / `npub` keys from a seed.
  - [`mnemonic silent-payment`](#mnemonic-silent-payment)
    — derive a BIP-352 silent-payment `sp1…` receiver address.
  - [`mnemonic verify-message`](#mnemonic-verify-message)
    — verify a legacy `signmessage` or BIP-322 signature.
  - [`mnemonic decode-address`](#mnemonic-decode-address)
    — decode an address to network / type / scriptPubKey.
  - [`mnemonic compare-cost`](#mnemonic-compare-cost)
    — compare wsh-vs-tr spending cost per spending condition.
- **Card inspection, repair, and SeedQR / Electrum.**
  - [`mnemonic inspect`](#mnemonic-inspect)\index{mnemonic inspect} —
    describe the contents of an `ms1` / `mk1` / `md1` card.
  - [`mnemonic repair`](#mnemonic-repair)\index{mnemonic repair} — BCH
    error-correct a corrupted `ms1` / `mk1` / `md1` card.
  - [`mnemonic seedqr-encode`](#mnemonic-seedqr-encode)\index{mnemonic seedqr-encode}
    — encode a BIP-39 phrase as a SeedQR numeric payload.
  - [`mnemonic seedqr-decode`](#mnemonic-seedqr-decode)\index{mnemonic seedqr-decode}
    — decode a SeedQR payload back to its BIP-39 phrase.
  - [`mnemonic electrum-decrypt`](#mnemonic-electrum-decrypt)\index{mnemonic electrum-decrypt}
    — decrypt an Electrum field-encrypted secret to its plaintext.
- **Engraving and documentation tools.**
  - [`mnemonic word-card`](#mnemonic-word-card) — re-encode a public
    `mk1` / `md1` card as an engravable BIP-39 Word Card (or decode one
    back).
  - [`mnemonic gen-man`](#mnemonic-gen-man) — emit `roff` man pages for
    the whole `mnemonic` CLI tree.

The nine families above describe what each subcommand *does*, not what
its form looks like in the GUI. Each subcommand is documented on its own
with form-shape diagrams, per-flag reference, and at least one worked
example.

## Form shape — what every subcommand has in common

All 32 subcommands render through the same form scaffolding described in
chapter 31: a top-of-form pinned-version label with a subcommand
selector ComboBox and per-subcommand `?` help-icon; per-flag widgets
(text fields, dropdowns, checkboxes, number spinners, path fields,
repeating-row editors, slot editors); a `Slot rows:` section if the
subcommand accepts the `--slot` repeating flag (currently `bundle`,
`verify-bundle`, `export-wallet`, `restore`, `build-descriptor`, and
`import-wallet`); an action bar with **Copy command (POSIX)**, **Copy
command (Windows)**, and **Run** buttons; an always-on `Preview:` line.

**Most** of the 30 subcommands consume secret-bearing inputs at
realistic form fillings — a master `ms1`, a `--passphrase`, a `--share`,
a `--from` whose chosen node is in the secret class (`phrase`,
`entropy`, `xprv`, `wif`, `ms1`, `bip38`, `electrum-phrase`,
`seedqr`), or a `--decrypt-password`. Under the `should_confirm_run`
predicate at `mnemonic-gui/src/secrets.rs` (any `secret: true` flag
value, any secret-class slot subkey, any secret-class NodeValueComposite
node), each fires the run-confirm modal under at least one valid form
input. The watch-only / public-only subcommands (`export-wallet` with
xpub-only slots, `decode-address`, `verify-message`, the
`xpub-search-*` family on public inputs) are the ones most likely to
fire **Run** without the modal in practice. The threat-model coverage in
[§14 Defense 2](#secret-handling) — the `••••` modal redaction and the
recommended cold-node operational mitigation — applies to **every**
secret-bearing invocation under this tab.

## Worked-example seed convention

Every chapter under this tab uses a canonical public test vector — the
12-word all-`abandon` phrase
`abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about`
or the 24-word `abandon × 23 + art` master where a longer seed is
required. These phrases are **public** and any wallet derived from them
has been swept by chain watchers since 2017 — their use in the manual is
for round-trip demonstration only. Each chapter that uses a test phrase
opens with a `:::danger` admonition restating this; do not engrave or
fund any wallet derived from them.

## Where to read next

If you are following the cycle from chapter 30 (the tour), pick a
subcommand from the index above and read its chapter end-to-end. If you
arrived via a `?` help-icon click in the GUI, you are already at the
right anchor; scroll up for the chapter outline, down for the per-flag
detail. Cross-references between chapters are explicit (`see [`mnemonic
bundle`](#mnemonic-bundle)`) and deep-link into the per-flag anchors
where relevant.
