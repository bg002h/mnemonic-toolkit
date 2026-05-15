# Appendix I — Index of terms

The PDF render emits a true page-numbered alphabetical index (built
by `makeindex` from `\index{}` markers throughout the source). The
markdown render emits this curated `Term → §section` table instead,
since markdown viewers have no notion of page numbers.

The two indexes are kept in lockstep by the bidirectional consistency
check in `tests/lint.sh`: every `\index{TERM}` marker in the source
must have a matching row here, and vice versa. Adding a marker
without adding the row (or vice versa) fails the lint.

| Term | Section |
|---|---|
| `BCH` | [Welcome to the m-format constellation](#welcome-to-the-m-format constellation) |
| `BCH error correction` | [Single-sig steel-engraved backup](#single-sig-steel-engraved-backup) |
| `BIP-32` | [Concept signposts](#concept-signposts) |
| `BIP-39` | [Concept signposts](#concept-signposts) |
| `codex32` | [Concept signposts](#concept-signposts) |
| `cross-binding` | [Single-sig steel-engraved backup](#single-sig-steel-engraved-backup) |
| `descriptor` | [Concept signposts](#concept-signposts) |
| `group threshold` | [mnemonic slip39](#mnemonic-slip39) |
| `HMAC-SHA-512` | [Deterministic child secrets via BIP-85](#deterministic-child-secrets-via-bip-85) |
| `K-of-N` | [mnemonic slip39](#mnemonic-slip39) |
| `m-format constellation` | [About this manual](#about-this-manual) |
| `md1` | [Welcome to the m-format constellation](#welcome-to-the-m-format constellation) |
| `member threshold` | [mnemonic slip39](#mnemonic-slip39) |
| `mk1` | [Welcome to the m-format constellation](#welcome-to-the-m-format constellation) |
| `mnemonic-toolkit` | [Welcome to the m-format constellation](#welcome-to-the-m-format constellation) |
| `ms1` | [Welcome to the m-format constellation](#welcome-to-the-m-format constellation) |
| `multisig` | [Concept signposts](#concept-signposts) |
| `policy_id_stub` | [Welcome to the m-format constellation](#welcome-to-the-m-format constellation) |
| `SLIP-39` | [mnemonic slip39](#mnemonic-slip39) |
| `SLIP-39 share` | [mnemonic slip39](#mnemonic-slip39) |
| `slot` | [Concept signposts](#concept-signposts) |
| `Trezor SLIP-0039 interop` | [mnemonic slip39](#mnemonic-slip39) |
