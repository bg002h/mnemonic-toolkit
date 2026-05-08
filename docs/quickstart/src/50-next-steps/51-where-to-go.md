# Where to go from here

This Quick Start covered single-sig, multisig, and watch-only by
worked example. The reference manual goes deeper on every topic.
Topic-keyed pointers below.

## Going deeper on workflows

The manual's *Workflows* part walks each end-to-end ceremony in
full, including operational variants this Quick Start skipped:

- [Single-sig steel-engraving ceremony](../../../manual/src/30-workflows/31-singlesig-steel.md)
  — production-quality stamping, geographic separation,
  re-stamp-on-failure discipline.
- [Multisig 2-of-3 walkthrough](../../../manual/src/30-workflows/32-multisig-2of3.md)
  — canonical air-gapped multisig including PSBT routing and
  signing ceremony.
- [Taproot multisig](../../../manual/src/30-workflows/33-taproot-multi.md)
  — `tr-multi-a` / `tr-sortedmulti-a` and the `--taproot-internal-key`
  decision.
- [Watch-only xpub bundle](../../../manual/src/30-workflows/34-watch-only.md)
  — full watch-only chapter, including the privacy-preserving variant.
- [Recovery paths by damaged-card scenario](../../../manual/src/30-workflows/35-recovery-paths.md)
  — exhaustive damage matrix, including partial-card-damage handling.
- [Migration from BIP-39 / Shamir / SeedQR](../../../manual/src/30-workflows/36-migration.md)
  — moving an existing wallet onto m-format steel.
- [Exporting to Bitcoin Core / BIP-388 / Sparrow / Specter](../../../manual/src/30-workflows/37-wallet-export.md)
  — the four wallet-export shapes and their import flows.
- [BIP-85 child-seed derivation](../../../manual/src/30-workflows/38-bip85-children.md)
  — generating child seeds (BIP-39, WIF, xprv, DICE) from a master.

## CLI reference

For the per-flag, per-subcommand canonical reference:

- [mnemonic CLI reference](../../../manual/src/40-cli-reference/41-mnemonic.md)
  — every `mnemonic` subcommand, every flag, every output mode.
- [md CLI reference](../../../manual/src/40-cli-reference/42-md.md)
  — `md` (descriptor card) subcommands.
- [ms CLI reference](../../../manual/src/40-cli-reference/43-ms.md)
  — `ms` (secret card) subcommands.
- [mk-codec Rust API](../../../manual/src/40-cli-reference/44-mk-codec-rust.md)
  — for embedding the mk1 codec into another tool.

## Comparing m-format with other backup standards

When choosing between m-format and an existing BIP / SLIP / vendor
format:

- [Picking a backup format](../../../manual/src/50-comparing/51-format-decision.md)
  — the decision tree.
- [mnemonic-toolkit vs. ms-cli](../../../manual/src/50-comparing/52-toolkit-vs-ms-cli.md)
  — when to use which.
- [mnemonic-toolkit vs. md-cli](../../../manual/src/50-comparing/53-toolkit-vs-md-cli.md)
  — when to use which.
- [m-format vs. SeedQR / Shamir / SeedSigner](../../../manual/src/50-comparing/54-mformat-vs-others.md)
  — head-to-head capability matrix.
- [Single-sig vs. multisig](../../../manual/src/50-comparing/55-singlesig-vs-multi.md)
  — operational tradeoffs.
- [BIP-39 vs. BIP-38 passphrase models](../../../manual/src/50-comparing/56-bip39-vs-bip38-pass.md)
  — when each adds security and when it adds risk.
- [Bitcoin Core descriptors vs. BIP-388 wallet policies](../../../manual/src/50-comparing/57-coredesc-vs-bip388.md)
  — interchange-format choice.

## BIP primers

If a referenced BIP is unfamiliar, the manual's primers cover the
four most-cited:

- [BIP-39 primer](../../../manual/src/60-appendices/62-bip39-primer.md)
  — entropy → phrase → seed.
- [BIP-32 primer](../../../manual/src/60-appendices/63-bip32-primer.md)
  — hierarchical deterministic derivation, xpubs and xprvs.
- [Descriptors primer](../../../manual/src/60-appendices/64-descriptors-primer.md)
  — Bitcoin Core descriptor language.
- [BCH / codex32 primer](../../../manual/src/60-appendices/65-bch-codex-primer.md)
  — the BCH error-correction code under m-format and codex32.

## Troubleshooting full matrix

The next chapter covers the five most common newcomer issues. For
the long form — bundle synthesis, verify-bundle, convert / recovery,
engraving, wallet-export, and BIP-85 failure modes —
[Appendix G — Troubleshooting matrix](../../../manual/src/60-appendices/67-troubleshooting.md)
is the full reference.

Onward: the five most common newcomer issues, each with a fix.
