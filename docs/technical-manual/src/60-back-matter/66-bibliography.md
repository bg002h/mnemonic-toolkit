# Bibliography

This bibliography seeds with the protocol and academic references cited in Parts I + II. Subsequent cuts add references as new Parts cite additional sources.

## Bitcoin Improvement Proposals

For up-to-date BIP texts, see [github.com/bitcoin/bips](https://github.com/bitcoin/bips/blob/master/README.mediawiki). Tagged BIP versions referenced by the manual are stable; the linked README enumerates current status.

- **BIP-32.** Pieter Wuille. *Hierarchical Deterministic Wallets.* [bip-0032.mediawiki](https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki). Cited in §I.1, §I.2, §I.3, §I.4, §II.1, §II.2, §II.3.
- **BIP-39.** Marek Palatinus, Pavol Rusnak, Aaron Voisine, Sean Bowe. *Mnemonic code for generating deterministic keys.* [bip-0039.mediawiki](https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki). Cited in §I.1, §I.2, §II.3.
- **BIP-93.** *codex32 — Checksummed SSSS-aware BIP-32 seeds.* [bip-0093.mediawiki](https://github.com/bitcoin/bips/blob/master/bip-0093.mediawiki). Cited in §I.2, §I.3, §I.4, §II.2, §II.3. (For the design history and the academic precursor see also the codex32 paper entry below.)
- **BIP-173.** Pieter Wuille, Greg Maxwell. *Base32 address format for native v0-16 witness outputs (bech32).* [bip-0173.mediawiki](https://github.com/bitcoin/bips/blob/master/bip-0173.mediawiki). Cited in §I.2, §I.3, §II.3.
- **BIP-340.** Pieter Wuille, Jonas Nick, Tim Ruffing. *Schnorr Signatures for secp256k1.* [bip-0340.mediawiki](https://github.com/bitcoin/bips/blob/master/bip-0340.mediawiki). Cited indirectly via BIP-341.
- **BIP-341.** Pieter Wuille, Jonas Nick, Anthony Towns. *Taproot: SegWit version 1 spending rules.* [bip-0341.mediawiki](https://github.com/bitcoin/bips/blob/master/bip-0341.mediawiki). Cited in §II.1.
- **BIP-342.** Pieter Wuille, Jonas Nick, Anthony Towns. *Validation of Taproot Scripts.* [bip-0342.mediawiki](https://github.com/bitcoin/bips/blob/master/bip-0342.mediawiki). Cited in §II.1.
- **BIP-379.** Pieter Wuille, Andrew Poelstra, Sanket Kanjalkar. *Miniscript.* [bip-0379.md](https://github.com/bitcoin/bips/blob/master/bip-0379.md). Cited indirectly (the type system underlying md1's bytecode AST).
- **BIP-380.** Pieter Wuille, Andrew Chow. *Output Script Descriptors General Operation.* [bip-0380.mediawiki](https://github.com/bitcoin/bips/blob/master/bip-0380.mediawiki). Cited in §II.2.
- **BIP-388.** Salvatore Ingala. *Wallet Policies for Descriptor Wallets.* [bip-0388.mediawiki](https://github.com/bitcoin/bips/blob/master/bip-0388.mediawiki). Cited in §I.1, §I.2, §I.3, §I.4, §II.1.
- **BIP-389.** Andrew Chow. *Multipath Descriptor Key Expressions.* [bip-0389.mediawiki](https://github.com/bitcoin/bips/blob/master/bip-0389.mediawiki). Cited in §I.4, §II.1.

## Academic and protocol references

- **codex32 paper.** Leon Olsson Curr (pseudonym "Pearlwort Sneed"), Andrew Poelstra. *codex32: A Better Way To Back Up Keys.* [codex32.org](https://codex32.org/). The original design analysis for the BCH-over-GF(32) scheme that BIP-93 normatively specifies. Cited in §I.3.
- **miniscript paper.** Andrew Poelstra, Pieter Wuille, Sanket Kanjalkar. *Miniscript: streamlined Bitcoin scripting.* [bitcoin.sipa.be/miniscript](https://bitcoin.sipa.be/miniscript/). The type-system foundations of BIP-379 (Miniscript) underlying md1's bytecode AST. Cited indirectly throughout §II.1.
- **FROST RFC 9591.** Chelsea Komlo, Tim Ruffing, et al. *Flexible Round-Optimized Schnorr Threshold Signatures (FROST).* [rfc-editor.org/rfc/rfc9591](https://www.rfc-editor.org/rfc/rfc9591). The threshold-signature protocol relevant to a potential future fifth m-format point. Cited in §I.2 "Future formats".

## Reference implementations cited normatively

- **`rust-codex32`** v0.1.0 (Andrew Poelstra, CC0). [docs.rs/rust-codex32](https://docs.rs/rust-codex32). ms1's BIP-93 codex32 dependency. §II.3 normative.
- **`rust-miniscript`** master (Andrew Poelstra et al.). [docs.rs/miniscript](https://docs.rs/miniscript). The reference miniscript implementation md-codec v0.32's `to_miniscript_descriptor` converter targets. Cited normatively by §II.1 and (in tech-manual-v0.2) by §III.1 / §III.2.

## Per-version SPECs (authoritative for "why we did it this way at version X")

The technical manual cites these but does not duplicate them. Read them when the rationale behind a wire-format decision matters.

- **`bg002h/descriptor-mnemonic/design/SPEC_v0_30_wire_format.md`** — md1 v0.30 wire-format authority.
- **`bg002h/descriptor-mnemonic/design/SPEC_v0_11_wire_format.md`** — md1's wire-layer-dictionary retirement record (§1.4).
- **`bg002h/descriptor-mnemonic/bip/bip-mnemonic-descriptor.mediawiki`** — md1 BIP draft (current; tracks `main`).
- **`bg002h/mnemonic-key/design/SPEC_mk_v0_1.md`** — mk1 v0.1 authority.
- **`bg002h/mnemonic-key/bip/bip-mnemonic-key.mediawiki`** — mk1 BIP draft (Pre-Draft).
- **`bg002h/mnemonic-secret/design/SPEC_ms_v0_1.md`** — ms1 v0.1 authority.
