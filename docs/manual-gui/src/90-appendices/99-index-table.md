# Index of terms {#index-of-terms}

The PDF render emits a true page-numbered alphabetical index
(built by `makeindex` from `\index{}` markers throughout the
source). The markdown render emits this curated `Term → §section`
table instead, since markdown viewers have no notion of page
numbers.

The two indexes are kept in lockstep by the bidirectional
consistency check in `tests/lint.sh` phase 7: every
`\index{TERM}` marker in `src/` must have a matching row here,
and vice versa. Adding a marker without adding the row (or vice
versa) fails the lint.

| Term | Section |
|---|---|
| `BIP-39` | [Glossary](#glossary) |
| `bundle` | [Glossary](#glossary) |
| `clipboard` | [Secrets, the OS, and operational hygiene](#secrets-and-os) |
| `conditional-visibility` | [Glossary](#glossary) |
| `cross-binding` | [Glossary](#glossary) |
| `md1` | [Glossary](#glossary) |
| `md address` | [`md address`](#md-address) |
| `md bytecode` | [`md bytecode`](#md-bytecode) |
| `md compile` | [`md compile`](#md-compile) |
| `md decode` | [`md decode`](#md-decode) |
| `md encode` | [`md encode`](#md-encode) |
| `md inspect` | [`md inspect`](#md-inspect) |
| `md vectors` | [`md vectors`](#md-vectors) |
| `md verify` | [`md verify`](#md-verify) |
| `m-format constellation` | [Glossary](#glossary) |
| `mk1` | [Glossary](#glossary) |
| `mk decode` | [`mk decode`](#mk-decode) |
| `mk encode` | [`mk encode`](#mk-encode) |
| `mk inspect` | [`mk inspect`](#mk-inspect) |
| `mk vectors` | [`mk vectors`](#mk-vectors) |
| `mk verify` | [`mk verify`](#mk-verify) |
| `mnemonic bundle` | [`mnemonic bundle`](#mnemonic-bundle) |
| `mnemonic convert` | [`mnemonic convert`](#mnemonic-convert) |
| `mnemonic derive-child` | [`mnemonic derive-child`](#mnemonic-derive-child) |
| `mnemonic export-wallet` | [`mnemonic export-wallet`](#mnemonic-export-wallet) |
| `mnemonic final-word` | [`mnemonic final-word`](#mnemonic-final-word) |
| `mnemonic-gui` | [Glossary](#glossary) |
| `mnemonic seed-xor-combine` | [`mnemonic seed-xor-combine`](#mnemonic-seed-xor-combine) |
| `mnemonic seed-xor-split` | [`mnemonic seed-xor-split`](#mnemonic-seed-xor-split) |
| `mnemonic slip39-combine` | [`mnemonic slip39-combine`](#mnemonic-slip39-combine) |
| `mnemonic slip39-split` | [`mnemonic slip39-split`](#mnemonic-slip39-split) |
| `mnemonic verify-bundle` | [`mnemonic verify-bundle`](#mnemonic-verify-bundle) |
| `ms1` | [Glossary](#glossary) |
| `ms decode` | [`ms decode`](#ms-decode) |
| `ms encode` | [`ms encode`](#ms-encode) |
| `ms inspect` | [`ms inspect`](#ms-inspect) |
| `ms vectors` | [`ms vectors`](#ms-vectors) |
| `ms verify` | [`ms verify`](#ms-verify) |
| `passphrase` | [Glossary](#glossary) |
| `run-confirm modal` | [Glossary](#glossary) |
| `screenreader` | [Glossary](#glossary) |
| `screenshot` | [Glossary](#glossary) |
| `SecretLineEdit` | [Glossary](#glossary) |
| `Wayland` | [Glossary](#glossary) |
