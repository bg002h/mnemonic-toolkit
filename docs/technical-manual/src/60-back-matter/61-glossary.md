# Glossary

This is the technical-manual glossary, focused on **wire-format / API / cryptography** terms. End-user-facing concepts (BIP-39 / BIP-32 walkthroughs, multisig UX) live in the end-user manual's glossary at `docs/manual/src/60-appendices/61-glossary.md`.

Entries populate incrementally per cut; the tech-manual-v0.1 seed below tracks what Parts I + II introduce.

## BCH

Bose–Chaudhuri–Hocquenghem error-correction code. md1 and mk1 share a Bitcoin-tuned BCH polynomial *forked* from BIP-93 codex32 (HRP-mixed, per-format target residues). ms1 uses BIP-93 codex32 directly via `rust-codex32`.

## BIP-388

Wallet-policy descriptor templates. The canonical JSON shape (`name`, `description_template`, `keys_info`) exchanged between hardware wallets and coordinators. md1 encodes BIP-388 wallet policies.

## codex32

BIP-93 — a Bitcoin-tuned 32-character alphabet with a BCH-style checksum and human-readable prefix. Adopted directly by ms1; the BCH plumbing is forked (not shared as a crate) by md1 and mk1.

## is_nums

A 1-bit flag on `Body::Tr` (md1 wire format, v0.30+). When `1`, signals the BIP-341 NUMS H-point as the implicit Taproot internal key (with `key_index` field suppressed entirely on the wire). When `0`, references the placeholder at `key_index` (width `kiw = ⌈log₂(n)⌉`).

## m-format constellation

The four sibling formats — **md1**, **mk1**, **ms1**, and the `mnemonic-toolkit` integration layer. Visually a star with the toolkit at the centre.

## md1

The descriptor card. Encodes a BIP-388-style wallet policy. HRP `md`. Library crate `md-codec`; CLI binary `md`. Repo `bg002h/descriptor-mnemonic`.

## miniscript

A subset of Bitcoin Script with type-checking and analysis properties (BIP-379). Each md1 descriptor body is a miniscript expression beneath the outer wrapper (`wsh()` / `tr()` / etc.).

## mk1

The key card. Encodes an xpub plus its BIP-32 origin (master fingerprint + derivation path). HRP `mk`. Library crate `mk-codec`. Repo `bg002h/mnemonic-key`.

## ms1

The secret card. Encodes BIP-39 entropy (or a BIP-32 master seed). HRP `ms`. Library crate `ms-codec`; uses `rust-codex32` directly. Repo `bg002h/mnemonic-secret`.

## NUMS

Nothing-Up-My-Sleeve. The BIP-341 H-point with no known discrete log, used as the Taproot internal key when a wallet has no cooperative-spend path. In md1 v0.30+ the NUMS encoding is the `is_nums = 1` flag on `Body::Tr`.

## wire format

The bit-level serialisation of a backup card. md1's current wire format is v0.30 (a clean break from v0.x — see `bg002h/descriptor-mnemonic/design/SPEC_v0_30_wire_format.md`). mk1's wire format mirrors md1's BCH plumbing but has its own primary-tag space. ms1's wire format is BIP-93 codex32 directly.
