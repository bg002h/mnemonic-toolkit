# mnemonic-toolkit

Top-level integration CLI for the **m-format star** of Bitcoin self-custody backup formats: takes a BIP-39 seed phrase (or watch-only xpub) and emits a complete steel-engravable bundle of three sibling cards.

Installs as binary `mnemonic`.

| Card | Format | What's on it |
|---|---|---|
| **ms1** | [`ms-codec`](https://github.com/bg002h/mnemonic-secret) | BIP-39 entropy (recovers the seed) |
| **mk1** | [`mk-codec`](https://github.com/bg002h/mnemonic-key) | xpub + origin (master fingerprint + BIP path) |
| **md1** | [`md-codec`](https://github.com/bg002h/descriptor-mnemonic) | wallet policy (template + bound xpub) |

The three cards engrave together as a coherent backup. Each card is independently BCH-checksummed by its sibling codec; the toolkit cross-binds them via the 4-byte `policy_id_stub` (`SHA-256(canonical wallet-policy preimage)[0..4]`) carried on each mk1 card and computable from each md1 card.

## Installation

`cargo install mnemonic-toolkit` is gated on the three sibling codecs reaching crates.io; until then build from the GitHub tag:

```bash
git clone --branch mnemonic-toolkit-v0.1.0 https://github.com/bg002h/mnemonic-toolkit
cd mnemonic-toolkit
cargo build --release --bin mnemonic
./target/release/mnemonic --help
```

## Quickstart

```bash
# Full mode: phrase → 3-card bundle.
mnemonic bundle --phrase "abandon abandon ... art" --network mainnet --template bip84

# Watch-only mode: xpub + master fingerprint → 2-card bundle (mk1 + md1; no ms1).
mnemonic bundle --xpub xpub6... --master-fingerprint 5436d724 --network mainnet --template bip84

# Round-trip verification: confirm the engraved bundle decodes against the original phrase.
mnemonic verify-bundle --phrase "abandon abandon ... art" \
    --network mainnet --template bip84 \
    --ms1 ms1... --mk1 mk1q... --mk1 mk1q... --md1 md1zs... --md1 md1zs... --md1 md1zs...
```

`--json` is available on both subcommands for tooling.

## Templates and networks

- **Templates:** `bip44` (pkh), `bip49` (sh-wpkh), `bip84` (wpkh), `bip86` (tr).
- **Networks:** `mainnet`, `testnet`, `signet`, `regtest`.
- Account is hardcoded `0` in v0.1; `--account` flag deferred to v0.2.

## Engraving caveats

- `ms1` v0.1 does NOT carry the BIP-39 wordlist language on the wire. Users with non-English wallets MUST record the wordlist language alongside the engraved card. The toolkit's `bundle` subcommand prints a default-card with that metadata to stderr (suppress with `--no-engraving-card`).
- `mk1` v0.1 is single-string only at the threshold-1 level (multi-share K-of-N is planned for v0.2). The 20-bit `chunk_set_id` is derived deterministically from the policy_id_stub for byte-reproducible output.
- `md1` v0.1 emits wallet-policy mode descriptors only.

## Documentation

- [SPEC v0.1](https://github.com/bg002h/mnemonic-toolkit/blob/master/design/SPEC_mnemonic_toolkit_v0_1.md) — full CLI surface specification.
- Sibling pointers: [`md-codec`](https://github.com/bg002h/descriptor-mnemonic), [`mk-codec`](https://github.com/bg002h/mnemonic-key), [`ms-codec`](https://github.com/bg002h/mnemonic-secret).

## License

CC0 1.0 Universal.
