# mnemonic-toolkit

Top-level integration crate for the **m-format star** of Bitcoin self-custody backup formats: takes a BIP-39 seed phrase (or watch-only xpub, or multi-source set of seeds) and emits a complete steel-engravable bundle of three sibling cards.

| Card | Format | What's on it |
|---|---|---|
| **ms1** | [`ms-codec`](https://github.com/bg002h/mnemonic-secret) | BIP-39 entropy (recovers the seed) |
| **mk1** | [`mk-codec`](https://github.com/bg002h/mnemonic-key) | xpub + origin (master fingerprint + BIP path) |
| **md1** | [`md-codec`](https://github.com/bg002h/descriptor-mnemonic) | wallet policy (template + bound xpub) |

Status: **v0.4.1 shipped.** Single-sig BIP-44/49/84/86 + multisig (`wsh-multi`, `wsh-sortedmulti`, `sh-wsh-multi`, `sh-wsh-sortedmulti`, `tr-multi-a`, `tr-sortedmulti-a`) + user-supplied BIP-388 descriptors + multi-leaf taproot + multi-source full multisig (`--slot @N.phrase=...` per cosigner). BIP-388 distinct-key conformance enforced symmetrically across `bundle` and `verify-bundle`. Mainnet / testnet / signet / regtest.

The three cards engrave together as a coherent backup. Each card is independently BCH-checksummed by its sibling codec; the toolkit cross-binds them via the 4-byte `policy_id_stub` (`SHA-256(canonical wallet-policy preimage)[0..4]`) carried on each mk1 card and computable from each md1 card.

## Documentation

- [`design/SPEC_mnemonic_toolkit_v0_4.md`](design/SPEC_mnemonic_toolkit_v0_4.md) — current SPEC (v0.4 cycle delta).
- [`design/SPEC_mnemonic_toolkit_v0_3.md`](design/SPEC_mnemonic_toolkit_v0_3.md) — predecessor (descriptor-mode foundation).
- [`design/SPEC_mnemonic_toolkit_v0_2.md`](design/SPEC_mnemonic_toolkit_v0_2.md) — multisig foundation.
- [`design/SPEC_mnemonic_toolkit_v0_1.md`](design/SPEC_mnemonic_toolkit_v0_1.md) — single-sig foundation.
- [`CHANGELOG.md`](CHANGELOG.md) — release notes.
- [`design/FOLLOWUPS.md`](design/FOLLOWUPS.md) — deferred-work tracker.

## License

CC0 1.0 Universal. See [LICENSE](LICENSE).
