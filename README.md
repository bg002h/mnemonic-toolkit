# mnemonic-toolkit

> **⚠ DISCLAIMER — UNTESTED ALPHA SOFTWARE.** **This software has not yet been independently tested or audited. Do not use the m-format constellation to back up significant sums of money at this time — doing so is tantamount to asking to be rekt.** Use only with disposable amounts, on testnet, or for evaluation. Codecs, CLIs, BCH math, and cross-card invariants have all been authored and reviewed only by the original developer. Assume bugs until external review happens.

Top-level integration crate for the **m-format constellation** of Bitcoin self-custody backup formats: takes a BIP-39 seed phrase (or watch-only xpub, or multi-source set of seeds) and emits a complete steel-engravable bundle of three sibling cards.

| Card | Format | What's on it |
|---|---|---|
| **ms1** | [`ms-codec`](https://github.com/bg002h/mnemonic-secret) | BIP-39 entropy (recovers the seed) |
| **mk1** | [`mk-codec`](https://github.com/bg002h/mnemonic-key) | xpub + origin (master fingerprint + BIP path) |
| **md1** | [`md-codec`](https://github.com/bg002h/descriptor-mnemonic) | wallet policy (template + bound xpub) |

Status: **v0.8.0 shipped** (2026-05-07; **breaking change** to BIP-38 composite-edge passphrase semantics — see [CHANGELOG](CHANGELOG.md) migration sentence). Single-sig BIP-44/49/84/86 + multisig (`wsh-multi`, `wsh-sortedmulti`, `sh-wsh-multi`, `sh-wsh-sortedmulti`, `tr-multi-a`, `tr-sortedmulti-a`) + user-supplied BIP-388 descriptors + multi-leaf taproot + multi-source full multisig (`--slot @N.phrase=...` per cosigner). BIP-388 distinct-key conformance enforced symmetrically across `bundle` and `verify-bundle`. v0.7 added BIP-38 / Casascius mini-key / Electrum native seed / address derivation to `mnemonic convert`, plus the `mnemonic export-wallet` and `mnemonic derive-child` subcommands. v0.8 layers in distinct BIP-38 vs BIP-39 passphrase channels, raw-stdin passphrase input (BIP-38 V3 NULL-byte), 4 non-English Electrum wordlists, taproot-multisig export via `--taproot-internal-key <nums|@N>`, descriptor → BIP-388 wallet_policy interop, BIP-85 phrase-master input + language codes + testnet emission + DICE app. Mainnet / testnet / signet / regtest.

## Install

Install all 5 m-format constellation components (4 CLIs + the
`mnemonic-gui` overlay) with the in-repo installer:

```sh
sh -c "$(curl -fsSL https://raw.githubusercontent.com/bg002h/mnemonic-toolkit/master/scripts/install.sh)"
```

If you already have the repo cloned, run `scripts/install.sh` directly.
`scripts/install.sh --help` lists per-component flags (`--only`,
`--exclude`, `--no-gui`, `--dry-run`, `--list`, `--force`). The script
installs each component via `cargo install --locked --git --tag` into
`$CARGO_INSTALL_ROOT` (default: `~/.cargo/bin`); no `sudo`, no system
files touched. Requires `cargo` + `git` + a C toolchain.

To install just this toolkit's `mnemonic` binary (no constellation
siblings):

```sh
cargo install --locked --git https://github.com/bg002h/mnemonic-toolkit --tag mnemonic-toolkit-v0.13.0 mnemonic-toolkit
```

## Subcommands

- **`mnemonic bundle`** — synthesize a 3-card bundle (ms1 + mk1 + md1) from BIP-39 phrase / entropy / multi-source seed input, or a watch-only 2-card bundle from xpub.
- **`mnemonic verify-bundle`** — re-derive and check parity across the 3 cards; reports per-card pass/fail and cross-binding integrity.
- **`mnemonic convert`** — single-format conversions across the 13-node typed graph (`phrase`, `entropy`, `xpub`, `xprv`, `wif`, `fingerprint`, `path`, `ms1`, `mk1`, `bip38`, `minikey`, `electrum-phrase`, `address`). v0.8 adds `--bip38-passphrase` (distinct from `--passphrase`; BREAKING on composite arms), `--passphrase-stdin` (raw-stdin passphrase preserving NULL bytes), and `--electrum-language` (English + 4 non-English Electrum wordlists).
- **`mnemonic export-wallet`** *(v0.7)* — emit watch-only wallet artifacts in Bitcoin Core `importdescriptors` JSON (default) or BIP-388 `wallet_policy` JSON. Sparrow / Specter formats stubbed. v0.8 adds `--taproot-internal-key <nums|@N>` (unblocks `tr-multi-a` / `tr-sortedmulti-a`) and `--descriptor + --format bip388` interop.
- **`mnemonic derive-child`** *(v0.7)* — BIP-85 deterministic child entropy. 7 applications in scope: `bip39` (v0.8: 9 BIP-85-coded languages), `hd-seed` (v0.8: testnet), `xprv` (v0.8: testnet), `hex`, `password-base64`, `password-base85`, `dice` (v0.8 §"DICE", BIP-85 v1.3.0). v0.8 also adds `--from phrase=...` (with `--passphrase` for BIP-39 mnemonic extension) and stdin via `--from <node>=-`. RSA + RSA-GPG deferred to v0.9 pending RUSTSEC-2023-0071 patch.

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

MIT License. See [LICENSE](LICENSE).
