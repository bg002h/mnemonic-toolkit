# Appendix A — Glossary

Definitions are intentionally terse for v0.1; expanded in Phase 7. For
deeper background, see the newcomer primers in
[Appendix B](#appendix-b-bip-39-entropy-primer)–[Appendix E](#appendix-e-codex32-bch-m-codec-error-correction).

## BCH

Bose–Chaudhuri–Hocquenghem error-correction code. The `mk-codec` and
`md-codec` use a Bitcoin-tuned BCH polynomial (forked from BIP-93
codex32) to detect and locate engraving errors on each card. The
`ms-codec` uses BIP-93 codex32 directly (via `rust-codex32`).

## BIP-32

Hierarchical Deterministic (HD) wallet derivation. Defines extended
keys (`xprv` / `xpub`) and child-key derivation paths like
`m/84'/0'/0'`. See [Appendix C](#appendix-c-bip-32-derivation-primer).

## BIP-38

Passphrase encryption of a single private key. Used by the toolkit's
`mnemonic convert` subcommand for the `bip38` and `minikey` node types.
v0.8 introduced a distinct `--bip38-passphrase` channel separate from
the BIP-39 passphrase.

## BIP-39

Mnemonic phrase encoding of wallet entropy as 12, 15, 18, 21, or 24
English (or other-language) words. The "seed phrase" stored on the
**ms1** card. See [Appendix B](#appendix-b-bip-39-entropy-primer).

## BIP-44 / BIP-49 / BIP-84 / BIP-86

Purpose-field constants for single-sig HD wallets:
`44'` = legacy P2PKH, `49'` = P2SH-wrapped P2WPKH, `84'` = native
SegWit P2WPKH, `86'` = single-key taproot.

## BIP-85

Deterministic child-entropy derivation. From a single master seed,
derive deterministic child secrets for other wallets, password
strings, raw entropy, or BIP-85 DICE entropy. Exposed via
`mnemonic derive-child`.

## BIP-93

The codex32 standard. A Bitcoin-tuned 32-character alphabet with a
BCH-style checksum and human-readable prefix. Adopted directly by
the **ms1** format; **md1** and **mk1** fork the BCH plumbing.

## BIP-388

Wallet-policy descriptor templates. A canonical JSON shape for
exchanging multisig descriptors between wallets. Exposed via
`mnemonic export-wallet --format bip388`.

## bundle

The 3-card aggregate (ms1 + mk1 + md1) emitted by `mnemonic bundle`.
Each card is independently BCH-checksummed by its sibling codec; the
toolkit cross-binds them via the `policy_id_stub`, which is carried
on each mk1 card and is computable from each md1 card. (The `mnemonic`
toolkit is the integration *layer* over the three card formats; it
emits no separate "toolkit card" of its own.)

## card

A single engravable string emitted by one of the three card codecs:
**ms1** (secret), **mk1** (key), or **md1** (descriptor). Each card
carries its own BCH checksum so partial damage is locatable.

## checksum

The BCH residue suffix on every card. `ms1` uses BIP-93 codex32; `md1`
and `mk1` use HRP-mixed BCH (forked from codex32, not upstream-shared).

## codex32

The BIP-93 checksummed alphabet used by `ms-codec`. See **BCH**.

## cosigner

A participant in a multisig wallet, contributing one xpub. The toolkit
takes one `--slot @N.<subkey>=<value>` per cosigner when synthesising a
multi-source bundle.

## derivation path

A BIP-32 path string like `m/84'/0'/0'/0/0`. Apostrophes mark hardened
indices.

## descriptor

A Bitcoin Core wallet description-language string like
`wsh(sortedmulti(2,xpub.../84h/0h/0h/0/*,xpub.../84h/0h/0h/0/*))#cksum`.
The **md1** card encodes a descriptor.

## DICE

A BIP-85 application emitting deterministic dice rolls (1–6) or
n-sided rolls. Useful for paper-friendly child entropy. Added in
toolkit v0.8.

## engraving card

A printable layout that includes one card string plus its visual
checksum row, formatted for steel engraving. The `--no-engraving-card`
flag suppresses generation.

## entropy

Raw byte-level wallet seed material. BIP-39 phrases encode entropy of
128, 160, 192, 224, or 256 bits.

## fingerprint

The first 4 bytes of HASH160 of a public master key. Identifies a
wallet origin in BIP-32 / BIP-388 derivations.

## HMAC-SHA-512

The keyed-hash primitive driving BIP-32 child-key derivation and
BIP-85 child-entropy derivation.

## m-format constellation

The four sibling formats — **ms1**, **mk1**, **md1**, and the
`mnemonic-toolkit` integration layer. The "star" is the visual mental
model: toolkit at the centre, three card formats radiating out.

## md1 / md-cli / md-codec

The descriptor card. Encodes a BIP-388-style wallet policy. CLI
binary `md`; library crate `md-codec` in repo
`bg002h/descriptor-mnemonic`.

## mk1 / mk-cli / mk-codec

The key card. Encodes an xpub plus its BIP-32 origin (master
fingerprint + derivation path). CLI binary `mk` (since v0.2);
library crate `mk-codec`. Repo `bg002h/mnemonic-key`.

## ms1 / ms-cli / ms-codec

The secret card. Encodes BIP-39 entropy (recovers the seed). CLI
binary `ms`; library crate `ms-codec`. Repo `bg002h/mnemonic-secret`.

## mnemonic

The integration CLI binary, shipped by the `mnemonic-toolkit` crate.
Seven subcommands: `bundle`, `verify-bundle`, `convert`,
`export-wallet`, `derive-child`, `final-word`, and `gui-schema`
(introspection-only). Multi-source seeds, xpubs, and related wallet
inputs flow in via the uniform `--slot @N.<subkey>=<value>` shape
(where `@N` is a cosigner index).

## mnemonic phrase

A BIP-39 seed phrase. The plain-text representation of wallet
entropy.

## multi

A descriptor key-list constructor: `multi(K, key1, key2, …)` — keys
remain in the original order.

## multisig

A wallet that requires *at least K of N* cosigners' signatures to
spend. Composed at the script layer, not the seed layer; each
cosigner has their own seed. Toolkit support: `wsh-multi`,
`wsh-sortedmulti`, `sh-wsh-multi`, `sh-wsh-sortedmulti`, `tr-multi-a`,
`tr-sortedmulti-a`.

## multi_a

The taproot variant of `multi`, defined in BIP-386 (`multi_a`
descriptor) and exchanged via BIP-388 (wallet policy).

## NUMS internal key

A Nothing Up My Sleeve elliptic-curve point used as the taproot
internal key when a multisig is engraved with no cooperative-spend
path. Selected via `mnemonic export-wallet --taproot-internal-key nums`.

## policy_id_stub

A 4-byte stub of `SHA-256(canonical wallet-policy preimage)`, carried
on each `mk1` card and computable from each `md1` card. Cross-binds
the cards in a bundle so cards from different wallets cannot be mixed.

## SLIP-0132

The convention for prefixing extended-key serialisations with format-
identifying bytes (`xpub` / `ypub` / `zpub` / `Ypub` / `Zpub` etc.).
Toolkit v0.6 added prefix-tolerant input + a `--xpub-prefix` output
flag.

## slot

The toolkit's `--slot @N.<subkey>=<value>` input shape, where `@N` is
a cosigner index in a multisig wallet and `<subkey>` is `phrase`,
`entropy`, `xpub`, etc.

## sortedmulti

A descriptor key-list constructor that lexicographically sorts keys
before script construction; signing is order-independent.

## sortedmulti_a

The taproot variant of `sortedmulti`.

## taproot

BIP-341 single-leaf or multi-leaf P2TR script type. Toolkit supports
single-key taproot (BIP-86) and multisig taproot via `tr-multi-a` /
`tr-sortedmulti-a` templates.

## threshold (K-of-N)

A multisig parameter: any K of the N cosigners can sign. Set via
`--threshold K` (any value 1..=N). Note: K-of-N *secret-share splitting*
(splitting the ms1 card itself into N shares) is a separate feature
planned for ms-codec v0.2.

## tr (taproot descriptor)

The descriptor outer wrapper for a taproot output: `tr(internal_key,
{leaf1, leaf2, …})`.

## verify-bundle

A `mnemonic` subcommand that re-derives expected card content from a
seed (or a partial set of cards) and checks parity across the three
cards.

## watch-only wallet

A wallet holding only public keys (`xpub`, descriptor) and no signing
material. Created via `mnemonic export-wallet`.

## wallet_policy

The BIP-388 JSON shape: `{ name, description, description_template,
keys_info }`. Emitted via
`mnemonic export-wallet --format bip388`.

## wsh

Witness Script Hash — the descriptor wrapper for native SegWit
multisig (`wsh(multi(...))`, `wsh(sortedmulti(...))`).

## xprv / xpub

BIP-32 extended private / public keys. The 78-byte serialisation
that propagates across BIP-32 derivations.
