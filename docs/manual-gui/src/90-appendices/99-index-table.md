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
| `md repair` | [`md repair`](#md-repair) |
| `md vectors` | [`md vectors`](#md-vectors) |
| `md verify` | [`md verify`](#md-verify) |
| `m-format constellation` | [Glossary](#glossary) |
| `mk1` | [Glossary](#glossary) |
| `mk address` | [`mk address`](#mk-address) |
| `mk decode` | [`mk decode`](#mk-decode) |
| `mk derive` | [`mk derive`](#mk-derive) |
| `mk encode` | [`mk encode`](#mk-encode) |
| `mk inspect` | [`mk inspect`](#mk-inspect) |
| `mk repair` | [`mk repair`](#mk-repair) |
| `mk vectors` | [`mk vectors`](#mk-vectors) |
| `mk verify` | [`mk verify`](#mk-verify) |
| `mnemonic addresses` | [`mnemonic addresses`](#mnemonic-addresses) |
| `mnemonic build-descriptor` | [`mnemonic build-descriptor`](#mnemonic-build-descriptor) |
| `mnemonic bundle` | [`mnemonic bundle`](#mnemonic-bundle) |
| `mnemonic compare-cost` | [`mnemonic compare-cost`](#mnemonic-compare-cost) |
| `mnemonic convert` | [`mnemonic convert`](#mnemonic-convert) |
| `mnemonic decode-address` | [`mnemonic decode-address`](#mnemonic-decode-address) |
| `mnemonic derive-child` | [`mnemonic derive-child`](#mnemonic-derive-child) |
| `mnemonic electrum-decrypt` | [`mnemonic electrum-decrypt`](#mnemonic-electrum-decrypt) |
| `mnemonic export-wallet` | [`mnemonic export-wallet`](#mnemonic-export-wallet) |
| `mnemonic final-word` | [`mnemonic final-word`](#mnemonic-final-word) |
| `mnemonic inspect` | [`mnemonic inspect`](#mnemonic-inspect) |
| `mnemonic ms-shares-combine` | [`mnemonic ms-shares-combine`](#mnemonic-ms-shares-combine) |
| `mnemonic ms-shares-split` | [`mnemonic ms-shares-split`](#mnemonic-ms-shares-split) |
| `mnemonic nostr` | [`mnemonic nostr`](#mnemonic-nostr) |
| `mnemonic repair` | [`mnemonic repair`](#mnemonic-repair) |
| `mnemonic-gui` | [Glossary](#glossary) |
| `mnemonic restore` | [`mnemonic restore`](#mnemonic-restore) |
| `mnemonic seed-xor-combine` | [`mnemonic seed-xor-combine`](#mnemonic-seed-xor-combine) |
| `mnemonic seed-xor-split` | [`mnemonic seed-xor-split`](#mnemonic-seed-xor-split) |
| `mnemonic seedqr-decode` | [`mnemonic seedqr-decode`](#mnemonic-seedqr-decode) |
| `mnemonic seedqr-encode` | [`mnemonic seedqr-encode`](#mnemonic-seedqr-encode) |
| `mnemonic silent-payment` | [`mnemonic silent-payment`](#mnemonic-silent-payment) |
| `mnemonic slip39-combine` | [`mnemonic slip39-combine`](#mnemonic-slip39-combine) |
| `mnemonic slip39-split` | [`mnemonic slip39-split`](#mnemonic-slip39-split) |
| `mnemonic verify-bundle` | [`mnemonic verify-bundle`](#mnemonic-verify-bundle) |
| `mnemonic verify-message` | [`mnemonic verify-message`](#mnemonic-verify-message) |
| `mnemonic xpub-search account-of-descriptor` | [`mnemonic xpub-search account-of-descriptor`](#mnemonic-xpub-search-account-of-descriptor) |
| `mnemonic xpub-search address-of-xpub` | [`mnemonic xpub-search address-of-xpub`](#mnemonic-xpub-search-address-of-xpub) |
| `mnemonic xpub-search passphrase-of-xpub` | [`mnemonic xpub-search passphrase-of-xpub`](#mnemonic-xpub-search-passphrase-of-xpub) |
| `mnemonic xpub-search path-of-xpub` | [`mnemonic xpub-search path-of-xpub`](#mnemonic-xpub-search-path-of-xpub) |
| `ms1` | [Glossary](#glossary) |
| `ms combine` | [`ms combine`](#ms-combine) |
| `ms decode` | [`ms decode`](#ms-decode) |
| `ms derive` | [`ms derive`](#ms-derive) |
| `ms encode` | [`ms encode`](#ms-encode) |
| `ms inspect` | [`ms inspect`](#ms-inspect) |
| `ms repair` | [`ms repair`](#ms-repair) |
| `ms split` | [`ms split`](#ms-split) |
| `ms vectors` | [`ms vectors`](#ms-vectors) |
| `ms verify` | [`ms verify`](#ms-verify) |
| `passphrase` | [Glossary](#glossary) |
| `run-confirm modal` | [Glossary](#glossary) |
| `screenreader` | [Glossary](#glossary) |
| `screenshot` | [Glossary](#glossary) |
| `SecretLineEdit` | [Glossary](#glossary) |
| `Wayland` | [Glossary](#glossary) |
