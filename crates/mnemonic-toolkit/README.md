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
git clone --branch mnemonic-toolkit-v0.2.0 https://github.com/bg002h/mnemonic-toolkit
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

# Multisig 2-of-3 (full mode, self-multisig — same seed used as all 3 cosigners).
# Emits a SELF-MULTISIG WARNING because all N xpubs are byte-identical.
mnemonic bundle --phrase "abandon abandon ... art" \
    --network mainnet --template wsh-sortedmulti \
    --threshold 2 --cosigner-count 3

# Multisig 2-of-3 (watch-only, distinct cosigners — production shape).
mnemonic bundle --network mainnet --template wsh-sortedmulti --threshold 2 \
    --cosigner xpub6A...:fingerprint1:m/87h/0h/0h \
    --cosigner xpub6B...:fingerprint2:m/87h/0h/0h \
    --cosigner xpub6C...:fingerprint3:m/87h/0h/0h

# Privacy-preserving multisig: omit master fingerprints from mk1 cards.
mnemonic bundle --phrase "abandon abandon ... art" \
    --network mainnet --template wsh-sortedmulti \
    --threshold 2 --cosigner-count 3 --privacy-preserving

# --self-check: synthesize-then-verify before engraving (catches synthesis drift).
mnemonic bundle --phrase "abandon abandon ... art" --network mainnet --template bip84 --self-check

# Non-zero account.
mnemonic bundle --phrase "abandon abandon ... art" --network mainnet --template bip84 --account 5

# Round-trip verification: confirm the engraved bundle decodes against the original phrase.
mnemonic verify-bundle --phrase "abandon abandon ... art" \
    --network mainnet --template bip84 \
    --ms1 ms1... --mk1 mk1q... --mk1 mk1q... --md1 md1zs... --md1 md1zs... --md1 md1zs...
```

`--json` is available on both subcommands for tooling.

## Templates and networks

- **Single-sig templates:** `bip44` (pkh), `bip49` (sh-wpkh), `bip84` (wpkh), `bip86` (tr).
- **Multisig templates (v0.2):** `wsh-multi`, `wsh-sortedmulti`, `sh-wsh-multi`, `sh-wsh-sortedmulti`, `tr-multi-a`, `tr-sortedmulti-a`. Threshold `1 ≤ K ≤ N ≤ 16`.
- **Networks:** `mainnet`, `testnet`, `signet`, `regtest`.
- **Account:** `--account <u32>` (default `0`).
- **Multisig path family:** `--multisig-path-family {bip48,bip87}` (default `bip87`).

## Engraving caveats

- `ms1` v0.1 does NOT carry the BIP-39 wordlist language on the wire. Users with non-English wallets MUST record the wordlist language alongside the engraved card. The toolkit's `bundle` subcommand prints a default-card with that metadata to stderr (suppress with `--no-engraving-card`).
- `mk1` v0.2 is still single-string at the threshold-1 level (K-of-N share encoding planned for v0.3+). The 20-bit `chunk_set_id` is derived deterministically from the policy_id_stub for byte-reproducible output.
- `md1` emits wallet-policy mode descriptors only.
- Full-mode multisig (`--cosigner-count > 1`) emits a non-suppressible SELF-MULTISIG WARNING to stderr because all N cosigner xpubs are byte-identical (single seed). Production multisig uses watch-only mode with distinct cosigners.

## Documentation

- [SPEC v0.2](https://github.com/bg002h/mnemonic-toolkit/blob/master/design/SPEC_mnemonic_toolkit_v0_2.md) — full CLI surface specification.
- [SPEC v0.1](https://github.com/bg002h/mnemonic-toolkit/blob/master/design/SPEC_mnemonic_toolkit_v0_1.md) — predecessor.
- Sibling pointers: [`md-codec`](https://github.com/bg002h/descriptor-mnemonic), [`mk-codec`](https://github.com/bg002h/mnemonic-key), [`ms-codec`](https://github.com/bg002h/mnemonic-secret).

## License

CC0 1.0 Universal.
