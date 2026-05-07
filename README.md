# mnemonic-toolkit

Top-level integration crate for the **m-format star** of Bitcoin self-custody backup formats: takes a BIP-39 seed phrase (or watch-only xpub, or multi-source set of seeds) and emits a complete steel-engravable bundle of three sibling cards.

| Card | Format | What's on it |
|---|---|---|
| **ms1** | [`ms-codec`](https://github.com/bg002h/mnemonic-secret) | BIP-39 entropy (recovers the seed) |
| **mk1** | [`mk-codec`](https://github.com/bg002h/mnemonic-key) | xpub + origin (master fingerprint + BIP path) |
| **md1** | [`md-codec`](https://github.com/bg002h/descriptor-mnemonic) | wallet policy (template + bound xpub) |

Status: **v0.7.0 shipped.** Single-sig BIP-44/49/84/86 + multisig (`wsh-multi`, `wsh-sortedmulti`, `sh-wsh-multi`, `sh-wsh-sortedmulti`, `tr-multi-a`, `tr-sortedmulti-a`) + user-supplied BIP-388 descriptors + multi-leaf taproot + multi-source full multisig (`--slot @N.phrase=...` per cosigner). BIP-388 distinct-key conformance enforced symmetrically across `bundle` and `verify-bundle`. v0.7 adds BIP-38 / Casascius mini-key / Electrum native seed / address derivation to `mnemonic convert`, plus new top-level subcommands `mnemonic export-wallet` (Bitcoin Core importdescriptors + BIP-388 wallet_policy) and `mnemonic derive-child` (BIP-85, 6 applications). Mainnet / testnet / signet / regtest.

## Subcommands

- **`mnemonic bundle`** — synthesize a 3-card bundle (ms1 + mk1 + md1) from BIP-39 phrase / entropy / multi-source seed input, or a watch-only 2-card bundle from xpub.
- **`mnemonic verify-bundle`** — re-derive and check parity across the 3 cards; reports per-card pass/fail and cross-binding integrity.
- **`mnemonic convert`** — single-format conversions across the 13-node typed graph (`phrase`, `entropy`, `xpub`, `xprv`, `wif`, `fingerprint`, `path`, `ms1`, `mk1`, `bip38`, `minikey`, `electrum-phrase`, `address`).
- **`mnemonic export-wallet`** *(v0.7)* — emit watch-only wallet artifacts in Bitcoin Core `importdescriptors` JSON (default) or BIP-388 `wallet_policy` JSON. Sparrow / Specter formats are stubbed for v0.8.
- **`mnemonic derive-child`** *(v0.7)* — BIP-85 deterministic child entropy from a master xpriv. 6 applications in scope: `bip39`, `hd-seed`, `xprv`, `hex`, `password-base64`, `password-base85`. RSA / RSA-GPG / DICE deferred to v0.8.

The three cards engrave together as a coherent backup. Each card is independently BCH-checksummed by its sibling codec; the toolkit cross-binds them via the 4-byte `policy_id_stub` (`SHA-256(canonical wallet-policy preimage)[0..4]`) carried on each mk1 card and computable from each md1 card.

## Documentation

- [`design/SPEC_convert_v0_6.md`](design/SPEC_convert_v0_6.md) — `mnemonic convert` SPEC (current; with v0.7 amendments §10.a / §11 / §12 / §13 / §14 for address / SPEC-pin / BIP-38 / Casascius / Electrum).
- [`design/SPEC_export_wallet_v0_7.md`](design/SPEC_export_wallet_v0_7.md) — `mnemonic export-wallet` SPEC (v0.7).
- [`design/SPEC_derive_child_v0_7.md`](design/SPEC_derive_child_v0_7.md) — `mnemonic derive-child` SPEC (v0.7, BIP-85).
- [`design/SPEC_mnemonic_toolkit_v0_5.md`](design/SPEC_mnemonic_toolkit_v0_5.md) — current bundle/verify-bundle SPEC (v0.5 cycle delta — typed-DerivationPath BIP-388 reversal, four-case ms1 short-circuit, mk1 cosigner-mapping diagnostic, legacy CLI flag deletion).
- [`design/SPEC_mnemonic_toolkit_v0_4.md`](design/SPEC_mnemonic_toolkit_v0_4.md) — predecessor (BIP-388 + `--slot` + multi-leaf taproot + schema-4).
- [`design/SPEC_mnemonic_toolkit_v0_3.md`](design/SPEC_mnemonic_toolkit_v0_3.md) — predecessor (descriptor-mode foundation).
- [`design/SPEC_mnemonic_toolkit_v0_2.md`](design/SPEC_mnemonic_toolkit_v0_2.md) — multisig foundation.
- [`design/SPEC_mnemonic_toolkit_v0_1.md`](design/SPEC_mnemonic_toolkit_v0_1.md) — single-sig foundation.
- [`CHANGELOG.md`](CHANGELOG.md) — release notes.
- [`design/FOLLOWUPS.md`](design/FOLLOWUPS.md) — deferred-work tracker.

## License

CC0 1.0 Universal. See [LICENSE](LICENSE).
