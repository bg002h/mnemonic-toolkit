# mnemonic-toolkit

Top-level integration crate for the **m-format star** of Bitcoin self-custody backup formats: takes a BIP-39 seed phrase (or watch-only xpub) and emits a complete steel-engravable bundle of three sibling cards.

| Card | Format | What's on it |
|---|---|---|
| **ms1** | [`ms-codec`](https://github.com/bg002h/mnemonic-secret) | BIP-39 entropy (recovers the seed) |
| **mk1** | [`mk-codec`](https://github.com/bg002h/mnemonic-key) | xpub + origin (master fingerprint + BIP path) |
| **md1** | [`md-codec`](https://github.com/bg002h/descriptor-mnemonic) | wallet policy (template + bound xpub) |

Status: **v0.1 in design** — single-sig BIP-44/49/84/86 templates, single account 0, mainnet/testnet/signet/regtest. Multisig, multi-account, and other templates are planned for v0.2+.

The three cards engrave together as a coherent backup. Each card is independently BCH-checksummed by its sibling codec; the toolkit cross-binds them via the 4-byte `policy_id_stub` (`SHA-256(canonical wallet-policy preimage)[0..4]`) carried on each mk1 card and computable from each md1 card.

## Documentation

- [`design/SPEC_mnemonic_toolkit_v0_1.md`](design/SPEC_mnemonic_toolkit_v0_1.md) — v0.1 specification.

## License

CC0 1.0 Universal. See [LICENSE](LICENSE).
