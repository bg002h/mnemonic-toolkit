# mnemonic-toolkit

> **⚠ DISCLAIMER — UNTESTED ALPHA SOFTWARE.** **This software has not yet been independently tested or audited. Do not use the m-format constellation to back up significant sums of money at this time — doing so is tantamount to asking to be rekt.** Use only with disposable amounts, on testnet, or for evaluation. Assume bugs until external review happens.

Top-level integration CLI for the **m-format constellation** of Bitcoin self-custody backup formats: takes a BIP-39 seed phrase (or watch-only xpub, or multi-source set of seeds) and emits a complete steel-engravable bundle of three sibling cards.

Installs as binary `mnemonic`.

<!-- toolkit-version: 0.55.3 -->
The `mnemonic` subcommands span 3-card bundle synthesis + verification, seed/key conversion (BIP-39 / BIP-32 / WIF / ms1 / mk1 / BIP-38 / Casascius / Electrum), batch watch-only address listing, cross-format wallet import/export (Bitcoin Core, BIP-388, BSMS/BIP-129, Coldcard, Sparrow, Specter, Electrum), guided descriptor construction (build-descriptor: policy-tree/archetype presets → gated wsh descriptors), watch-only restore documents (single-sig from a seed + passphrase, fingerprint-gated; multisig from the shared md1 card alone, incl. taproot NUMS), backup splitting (seed-XOR, SLIP-39, BIP-93 codex32 K-of-N shares via ms-shares, SeedQR), BIP-85 derivation, BIP-352 silent-payment addresses, nostr key wrapping, legacy + BIP-322 message verification, address decoding, and BCH repair / inspection. Mainnet / testnet / signet / regtest. See **[CHANGELOG.md](https://github.com/bg002h/mnemonic-toolkit/blob/master/CHANGELOG.md)** for the release history.

| Card | Format | What's on it |
|---|---|---|
| **ms1** | [`ms-codec`](https://github.com/bg002h/mnemonic-secret) | BIP-39 entropy (recovers the seed) |
| **mk1** | [`mk-codec`](https://github.com/bg002h/mnemonic-key) | xpub + origin (master fingerprint + BIP path) |
| **md1** | [`md-codec`](https://github.com/bg002h/descriptor-mnemonic) | wallet policy (template + bound xpub) |

The three cards engrave together as a coherent backup. Each card is independently BCH-checksummed by its sibling codec; the toolkit cross-binds them via the 4-byte `policy_id_stub` (`SHA-256(canonical wallet-policy preimage)[0..4]`) carried on each mk1 card and computable from each md1 card.

## Installation

`cargo install mnemonic-toolkit` is gated on the three sibling codecs reaching crates.io; until then install from the GitHub tag via the in-repo installer (it carries the current version pin):

```bash
sh -c "$(curl -fsSL https://raw.githubusercontent.com/bg002h/mnemonic-toolkit/master/scripts/install.sh)" -- --only mnemonic
```

Or pin a specific release tag directly: `cargo install --locked --git https://github.com/bg002h/mnemonic-toolkit --tag mnemonic-toolkit-vX.Y.Z mnemonic-toolkit` (use the latest tag from the [releases page](https://github.com/bg002h/mnemonic-toolkit/releases)).

## Subcommands & reference

The `mnemonic` subcommands — `bundle` / `verify-bundle`, `convert` / `addresses` / `derive-child`, `import-wallet` / `export-wallet` / `restore` / `decode-address`, `seed-xor` / `slip39` / `ms-shares` / `seedqr`, `nostr` / `silent-payment` / `verify-message` / `final-word`, `electrum-decrypt` / `repair` / `inspect` / `compare-cost` / `xpub-search`, `build-descriptor`, and `gui-schema`. Run any with `--help`.

The **[end-user manual](https://github.com/bg002h/mnemonic-toolkit/tree/master/docs/manual)** is the authoritative, always-current CLI reference (lint-gated against the live `--help` surface), with worked examples and round-trip recipes for every subcommand and foreign-wallet format. The repo-root [`README`](https://github.com/bg002h/mnemonic-toolkit/blob/master/README.md) has the grouped subcommand inventory.

<details><summary>Quickstart (bundle + verify-bundle)</summary>

```bash
# Full mode (single-sig): phrase → 3-card bundle.
mnemonic bundle --phrase "abandon abandon ... art" --network mainnet --template bip84

# Watch-only (single-sig): xpub + master fingerprint → 2-card bundle (mk1 + md1).
mnemonic bundle --xpub xpub6... --master-fingerprint 5436d724 --network mainnet --template bip84

# Multisig 2-of-3 (watch-only with distinct cosigners — the production shape).
mnemonic bundle --network mainnet --template wsh-sortedmulti --threshold 2 \
    --cosigner xpub6A...:fingerprint1:m/87h/0h/0h \
    --cosigner xpub6B...:fingerprint2:m/87h/0h/0h \
    --cosigner xpub6C...:fingerprint3:m/87h/0h/0h

# Round-trip verification: confirm the engraved bundle decodes against the original phrase.
mnemonic verify-bundle --phrase "abandon abandon ... art" \
    --network mainnet --template bip84 \
    --ms1 ms1... --mk1 mk1q... --md1 md1zs... --md1 md1zs... --md1 md1zs...
```

`--json` is available on `bundle` / `verify-bundle` for tooling.
</details>

## Templates and networks

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

`--json` is available on `bundle` / `verify-bundle` for tooling. Schema version `"4"` is the v0.4 wire envelope; `ms1` is a length-N array (empty-string sentinel for watch-only slots).

### `mnemonic convert` (v0.6+) — single-format conversions

```bash
# BIP-39 phrase ↔ entropy ↔ xpub.
mnemonic convert --from "phrase=abandon abandon ... art" --to entropy
mnemonic convert --from "entropy=00000000000000000000000000000000" --to phrase
mnemonic convert --from "phrase=abandon abandon ... art" --to xpub --template bip84 --network mainnet

# BIP-38 encrypted WIF (v0.7).
mnemonic convert --from "wif=L4..." --to bip38 --passphrase "TestingOneTwoThree"
mnemonic convert --from "bip38=6PYN..." --to wif --passphrase "TestingOneTwoThree"

# v0.8 BREAKING — composite (phrase|entropy, bip38) splits the passphrase:
#   --passphrase  drives BIP-39 PBKDF2 (mnemonic extension);
#   --bip38-passphrase  drives BIP-38 Scrypt independently.
# v0.7 dual-purpose semantics: `--passphrase X` reused for both legs.
# v0.8: `--passphrase X --bip38-passphrase X` to preserve v0.7 behavior.
mnemonic convert --from "phrase=abandon abandon ... art" --to bip38 \
    --path "m/84'/0'/0'/0/0" --passphrase "extension" --bip38-passphrase "scrypt-key"

# Raw-stdin passphrase (preserves NULL bytes; BIP-38 V3 spec vector).
printf '\xcf\x93\x00\xf0\x90\x90\x80\xf0\x9f\x92\xa9' | \
    mnemonic convert --from "bip38=6PRW..." --to wif --passphrase-stdin

# Electrum native seed (v0.7; v0.8 adds --electrum-language for non-English).
mnemonic convert --from "electrum-phrase=cram swing cover prefer ... able" --to entropy
mnemonic convert --from "electrum-phrase=almíbar tibio superar ... odisea" --to entropy \
    --electrum-language spanish

# Casascius mini-key (v0.7).
mnemonic convert --from "minikey=SzavMBLoXU6kDrqtUVmffv" --to wif
```

### `mnemonic export-wallet` (v0.7) — watch-only wallet artifacts

```bash
# Bitcoin Core importdescriptors JSON (default).
mnemonic export-wallet --template bip84 --network mainnet \
    --slot "@0.xpub=xpub6..." --slot "@0.fingerprint=73c5da0a" --slot "@0.path=m/84'/0'/0'"

# BIP-388 wallet_policy JSON (template-mode).
mnemonic export-wallet --template wsh-sortedmulti --threshold 2 --format bip388 --network mainnet \
    --slot "@0.xpub=xpub6A..." --slot "@0.fingerprint=fp1" --slot "@0.path=m/48'/0'/0'/2'" \
    --slot "@1.xpub=xpub6B..." --slot "@1.fingerprint=fp2" --slot "@1.path=m/48'/0'/0'/2'"

# v0.8: descriptor → BIP-388 wallet_policy interop (multipath form required).
mnemonic export-wallet --format bip388 --network mainnet \
    --descriptor "wsh(sortedmulti(2,[fp1/48'/0'/0'/2']xpub6A.../<0;1>/*,[fp2/48'/0'/0'/2']xpub6B.../<0;1>/*))"

# v0.8: taproot multisig — choose internal key (NUMS or cosigner @N).
mnemonic export-wallet --template tr-multi-a --threshold 2 --network mainnet \
    --taproot-internal-key nums \
    --slot "@0.xpub=xpub6A..." --slot "@0.fingerprint=fp1" --slot "@0.path=m/48'/0'/0'/2'" \
    --slot "@1.xpub=xpub6B..." --slot "@1.fingerprint=fp2" --slot "@1.path=m/48'/0'/0'/2'"
```

### `mnemonic derive-child` (v0.7) — BIP-85 deterministic children

```bash
# BIP-85 BIP-39 children.
mnemonic derive-child --from "xprv=xprv9s21..." \
    --application bip39 --length 12 --index 0

# v0.8: phrase as master + non-English wordlist.
mnemonic derive-child --from "phrase=girl mad pet ... nose" \
    --application bip39 --length 12 --index 0 --language japanese

# v0.8: testnet emission (hd-seed / xprv).
mnemonic derive-child --from "xprv=xprv9s21..." \
    --application hd-seed --length 0 --index 0 --network testnet

# v0.8: BIP-85 DICE (deterministic dice rolls per BIP-85 v1.3.0 §"DICE").
mnemonic derive-child --from "xprv=xprv9s21..." \
    --application dice --length 10 --index 0 --dice-sides 6
# → 1,0,0,2,0,1,5,5,2,4
```

## Templates and networks

- **Single-sig templates:** `bip44` (pkh), `bip49` (sh-wpkh), `bip84` (wpkh), `bip86` (tr).
- **Multisig templates:** `wsh-multi`, `wsh-sortedmulti`, `sh-wsh-multi`, `sh-wsh-sortedmulti`, `tr-multi-a`, `tr-sortedmulti-a`. Threshold `1 ≤ K ≤ N ≤ 16`. Taproot multisig under `mnemonic export-wallet` requires `--taproot-internal-key <nums|@N>` (v0.8).
- **User-supplied descriptors:** any BIP-388 descriptor string via `--descriptor` / `--descriptor-file`. Multi-leaf taproot (`tr(K, {leaf1, leaf2, ...})`) supported in v0.4+. v0.8 lifts the `--descriptor + --format bip388` refusal in `export-wallet` (multipath `<0;1>/*` form required).
- **Networks:** `mainnet`, `testnet`, `signet`, `regtest`.
- **Account:** `--account <u32>` (default `0`).
- **Multisig path family:** `--multisig-path-family {bip48,bip87}` (default `bip87`).

## Engraving caveats

- **BIP-388 distinct-key conformance** (v0.4): the toolkit hard-rejects any bundle whose slots resolve to identical `(xpub, derivation_path)` tuples. This catches the v0.2 self-multisig pattern (single seed used as N cosigners) at both `bundle` (exit 2) and `verify-bundle` (exit 4). Use `--cosigner` triples for watch-only multisig or `--slot @N.phrase=` per cosigner for multi-source full multisig.
- `ms1` v0.1 does NOT carry the BIP-39 wordlist language on the wire. Users with non-English wallets MUST record the wordlist language alongside the engraved card. The toolkit's `bundle` subcommand prints a default-card with that metadata to stderr (suppress with `--no-engraving-card`).
- `mk1` is single-string; the 20-bit `chunk_set_id` is derived deterministically from the policy_id_stub for byte-reproducible output. K-of-N share encoding is planned for the mk-codec v0.2 cycle.
- `md1` emits wallet-policy mode descriptors only.

## Documentation

- **[end-user manual](https://github.com/bg002h/mnemonic-toolkit/tree/master/docs/manual)** — authoritative, always-current CLI reference (lint-gated against the live `--help`), with per-subcommand chapters + worked examples.
- [CHANGELOG](https://github.com/bg002h/mnemonic-toolkit/blob/master/CHANGELOG.md) — full release history.
- [`design/`](https://github.com/bg002h/mnemonic-toolkit/tree/master/design) — SPECs, implementation plans, per-cycle architect reviews, and [FOLLOWUPS](https://github.com/bg002h/mnemonic-toolkit/blob/master/design/FOLLOWUPS.md).
- Sibling pointers: [`md-codec`](https://github.com/bg002h/descriptor-mnemonic), [`mk-codec`](https://github.com/bg002h/mnemonic-key), [`ms-codec`](https://github.com/bg002h/mnemonic-secret).

## License

MIT License.
