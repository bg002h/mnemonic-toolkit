# mnemonic-toolkit

Top-level integration CLI for the **m-format star** of Bitcoin self-custody backup formats: takes a BIP-39 seed phrase (or watch-only xpub, or multi-source set of seeds) and emits a complete steel-engravable bundle of three sibling cards.

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
git clone --branch mnemonic-toolkit-v0.4.1 https://github.com/bg002h/mnemonic-toolkit
cd mnemonic-toolkit
cargo build --release --bin mnemonic
./target/release/mnemonic --help
```

## Quickstart

```bash
# Full mode (single-sig): phrase → 3-card bundle.
mnemonic bundle --phrase "abandon abandon ... art" --network mainnet --template bip84

# Watch-only mode (single-sig): xpub + master fingerprint → 2-card bundle (mk1 + md1; ms1 is empty-string sentinel).
mnemonic bundle --xpub xpub6... --master-fingerprint 5436d724 --network mainnet --template bip84

# Multisig 2-of-3 (watch-only with distinct cosigners — the production shape).
mnemonic bundle --network mainnet --template wsh-sortedmulti --threshold 2 \
    --cosigner xpub6A...:fingerprint1:m/87h/0h/0h \
    --cosigner xpub6B...:fingerprint2:m/87h/0h/0h \
    --cosigner xpub6C...:fingerprint3:m/87h/0h/0h

# Multisig 2-of-3 (full mode, multi-source — N distinct seeds via --slot, v0.4+).
mnemonic bundle --network mainnet --template wsh-sortedmulti --threshold 2 \
    --slot "@0.phrase=abandon abandon ... art" \
    --slot "@1.phrase=legal winner thank ... vote" \
    --slot "@2.phrase=letter advice cage ... above"

# Hybrid 2-of-3 (own seed @0 + watch-only cosigners @1, @2 via --slot).
mnemonic bundle --network mainnet --template wsh-sortedmulti --threshold 2 \
    --slot "@0.phrase=abandon abandon ... art" \
    --slot "@1.xpub=xpub6B..." --slot "@1.fingerprint=cafef00d" --slot "@1.path=87'/0'/0'" \
    --slot "@2.xpub=xpub6C..." --slot "@2.fingerprint=cafe1234" --slot "@2.path=87'/0'/0'"

# Privacy-preserving: omit master fingerprints from mk1 cards.
mnemonic bundle --privacy-preserving \
    --network mainnet --template wsh-sortedmulti --threshold 2 \
    --cosigner xpub6A...:fingerprint1:m/87h/0h/0h \
    --cosigner xpub6B...:fingerprint2:m/87h/0h/0h \
    --cosigner xpub6C...:fingerprint3:m/87h/0h/0h

# User-supplied BIP-388 descriptor (full or watch-only auto-detected).
mnemonic bundle --descriptor "wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))" \
    --network mainnet --phrase "abandon abandon ... art" \
    --cosigner xpub6B...:fingerprint2:m/87h/0h/0h \
    --cosigner xpub6C...:fingerprint3:m/87h/0h/0h

# --self-check: synthesize-then-verify before engraving.
mnemonic bundle --phrase "abandon abandon ... art" --network mainnet --template bip84 --self-check

# Round-trip verification: confirm the engraved bundle decodes against the original phrase.
mnemonic verify-bundle --phrase "abandon abandon ... art" \
    --network mainnet --template bip84 \
    --ms1 ms1... --mk1 mk1q... --md1 md1zs... --md1 md1zs... --md1 md1zs...
```

`--json` is available on both subcommands for tooling. Schema version `"4"` is the v0.4 wire envelope; `ms1` is a length-N array (empty-string sentinel for watch-only slots).

## Templates and networks

- **Single-sig templates:** `bip44` (pkh), `bip49` (sh-wpkh), `bip84` (wpkh), `bip86` (tr).
- **Multisig templates:** `wsh-multi`, `wsh-sortedmulti`, `sh-wsh-multi`, `sh-wsh-sortedmulti`, `tr-multi-a`, `tr-sortedmulti-a`. Threshold `1 ≤ K ≤ N ≤ 16`.
- **User-supplied descriptors:** any BIP-388 descriptor string via `--descriptor` / `--descriptor-file`. Multi-leaf taproot (`tr(K, {leaf1, leaf2, ...})`) supported in v0.4+.
- **Networks:** `mainnet`, `testnet`, `signet`, `regtest`.
- **Account:** `--account <u32>` (default `0`).
- **Multisig path family:** `--multisig-path-family {bip48,bip87}` (default `bip87`).

## Engraving caveats

- **BIP-388 distinct-key conformance** (v0.4): the toolkit hard-rejects any bundle whose slots resolve to identical `(xpub, derivation_path)` tuples. This catches the v0.2 self-multisig pattern (single seed used as N cosigners) at both `bundle` (exit 2) and `verify-bundle` (exit 4). Use `--cosigner` triples for watch-only multisig or `--slot @N.phrase=` per cosigner for multi-source full multisig.
- `ms1` v0.1 does NOT carry the BIP-39 wordlist language on the wire. Users with non-English wallets MUST record the wordlist language alongside the engraved card. The toolkit's `bundle` subcommand prints a default-card with that metadata to stderr (suppress with `--no-engraving-card`).
- `mk1` is single-string; the 20-bit `chunk_set_id` is derived deterministically from the policy_id_stub for byte-reproducible output. K-of-N share encoding is planned for the mk-codec v0.2 cycle.
- `md1` emits wallet-policy mode descriptors only.

## Documentation

- [SPEC v0.4](https://github.com/bg002h/mnemonic-toolkit/blob/master/design/SPEC_mnemonic_toolkit_v0_4.md) — current cycle delta (BIP-388 + `--slot` + multi-leaf taproot + schema-4).
- [SPEC v0.3](https://github.com/bg002h/mnemonic-toolkit/blob/master/design/SPEC_mnemonic_toolkit_v0_3.md) — descriptor-mode foundation.
- [SPEC v0.2](https://github.com/bg002h/mnemonic-toolkit/blob/master/design/SPEC_mnemonic_toolkit_v0_2.md) — multisig foundation.
- [SPEC v0.1](https://github.com/bg002h/mnemonic-toolkit/blob/master/design/SPEC_mnemonic_toolkit_v0_1.md) — single-sig foundation.
- [CHANGELOG](https://github.com/bg002h/mnemonic-toolkit/blob/master/CHANGELOG.md) — release notes.
- Sibling pointers: [`md-codec`](https://github.com/bg002h/descriptor-mnemonic), [`mk-codec`](https://github.com/bg002h/mnemonic-key), [`ms-codec`](https://github.com/bg002h/mnemonic-secret).

## License

CC0 1.0 Universal.
