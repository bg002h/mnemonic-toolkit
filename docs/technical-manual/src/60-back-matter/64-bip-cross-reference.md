# BIP Cross-Reference

This table maps each Bitcoin Improvement Proposal cited in the manual to the sections that cite it. v0.1 seeds the table from Parts I + II citations; subsequent cuts add rows as new Parts cite additional BIPs.

For BIP texts, see [github.com/bitcoin/bips](https://github.com/bitcoin/bips/blob/master/README.mediawiki). Per-version SPECs in each sibling repo's `design/` folder are the authoritative "why we did it this way at version X" references; this table cross-references the BIPs they normatively cite.

| BIP | Title | Sections citing it |
|---|---|---|
| BIP-32 | Hierarchical Deterministic Wallets | §I.1, §I.2, §I.3, §I.4, §II.1, §II.2, §II.3, §III.1, §III.3, §IV.2, §V.1, §V.2, §V.4 |
| BIP-38 | Passphrase-protected private key | §V.4 |
| BIP-39 | Mnemonic code for generating deterministic keys | §I.1, §I.2, §II.3, §IV.1, §IV.2, §IV.3, §V.1, §V.3, §V.4 |
| BIP-44 | Multi-Account Hierarchy for Deterministic Wallets | §III.1, §III.2, §V.4 |
| BIP-45 | Structure for Deterministic P2SH Multisignature Wallets | §III.2 |
| BIP-48 | Multi-Script Hierarchy for Multi-Sig Wallets | §II.2, §III.1, §III.2, §III.3, §V.2, §V.4 |
| BIP-49 | Derivation scheme for P2SH-P2WPKH based accounts | §III.1, §III.2, §III.3 |
| BIP-84 | Derivation scheme for P2WPKH based accounts | §I.1, §I.2, §II.2, §III.1, §III.2, §III.3, §IV.1, §V.4 |
| BIP-85 | Deterministic Entropy From BIP-32 Keychains | §V.4 |
| BIP-86 | Key Derivation for Single Key P2TR Outputs | §III.1, §III.2, §V.4 |
| BIP-87 | Hierarchy for Deterministic Multisig Wallets | §V.4 |
| BIP-93 | codex32 — Checksummed SSSS-aware BIP-32 seeds | §I.2, §I.3, §I.4, §II.2, §II.3, §IV.3, §V.2, §V.3 |
| BIP-173 | Base32 address format (bech32) | §I.2, §I.3, §II.3, §V.2, §V.3 |
| BIP-340 | Schnorr Signatures for secp256k1 | Bibliography |
| BIP-341 | Taproot: SegWit version 1 spending rules | §II.1, §III.2, §V.1 |
| BIP-342 | Validation of Taproot Scripts (tapscript) | §II.1 |
| BIP-379 | Miniscript | Glossary (§"miniscript"), Bibliography |
| BIP-380 | Output Script Descriptors General Operation | §II.2, §III.1 |
| BIP-388 | Wallet Policies for Descriptor Wallets | §I.1, §I.2, §I.3, §I.4, §II.1, §III.1, §III.2, §III.3, §IV.1, §IV.2, §V.1, §V.4 |
| BIP-389 | Multipath Descriptor Key Expressions | §I.4, §II.1, §III.1, §IV.1 |

## Non-BIP cross-references

| Reference | Subject | Sections citing it |
|---|---|---|
| SLIP-0132 | Alternative BIP-32 extended-key version bytes (`zpub`/`ypub`/`Zpub` family) | §III.3, Glossary |
