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
| `BCH` | [Welcome to the m-format star](#welcome-to-the-m-format-star) |
| `BIP-32` | [Concept signposts](#concept-signposts) |
| `BIP-39` | [Concept signposts](#concept-signposts) |
| `codex32` | [Concept signposts](#concept-signposts) |
| `descriptor` | [Concept signposts](#concept-signposts) |
| `m-format star` | [About this manual](#about-this-manual) |
| `md1` | [Welcome to the m-format star](#welcome-to-the-m-format-star) |
| `mk1` | [Welcome to the m-format star](#welcome-to-the-m-format-star) |
| `mnemonic-toolkit` | [Welcome to the m-format star](#welcome-to-the-m-format-star) |
| `ms1` | [Welcome to the m-format star](#welcome-to-the-m-format-star) |
| `multisig` | [Concept signposts](#concept-signposts) |
| `policy_id_stub` | [Welcome to the m-format star](#welcome-to-the-m-format-star) |
| `slot` | [Concept signposts](#concept-signposts) |
