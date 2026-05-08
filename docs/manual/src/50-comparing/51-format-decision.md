# Format-by-format decision table

When does each format earn its place? This chapter lays out the four
m-format constellation surfaces side by side so a reader can pick *which*
format a given task targets — and then jump to the right tool.

## The four surfaces

| Format | Carries | Reach (network) | Standalone CLI | Library |
|---|---|---|---|---|
| `ms1` | BIP-39 entropy | mainnet / testnet / signet / regtest | `ms` | `ms-codec` (Rust) |
| `mk1` | xpub + origin (fingerprint + path) | mainnet / testnet / signet / regtest | (none in v0.1) | `mk-codec` (Rust) |
| `md1` | wallet policy (template + bound xpub) | mainnet / testnet / signet / regtest | `md` | `md-codec` (Rust) |
| `mnemonic-toolkit` | integration over the three card formats | all four; multi-source slot inputs | `mnemonic` | `mnemonic-toolkit` (Rust) |

## Pick by task

| Task | Use |
|---|---|
| Engrave a single-sig BIP-84 wallet | `mnemonic bundle --template bip84 --slot @0.phrase=…` |
| Engrave a 2-of-3 multisig wallet | `mnemonic bundle --template wsh-sortedmulti --threshold 2 --slot @0.phrase=… --slot @1.phrase=… --slot @2.phrase=…` |
| Engrave a taproot multisig | `mnemonic bundle --template tr-sortedmulti-a --taproot-internal-key nums …` |
| Re-derive a phrase from an ms1 card | `mnemonic convert --from ms1=… --to phrase` (or `ms decode <STRING>`) |
| Re-derive xpub + path from an mk1 card | `mnemonic convert --from mk1=… --to xpub --to fingerprint --to path` |
| Decode a wallet policy from md1 | `md decode <STRINGS>` (positional) |
| Cross-check a 3-card bundle | `mnemonic verify-bundle …` |
| Watch-only export (Bitcoin Core / BIP-388 / Sparrow / Specter) | `mnemonic export-wallet …` |
| Derive a child secret (BIP-85) | `mnemonic derive-child …` |

## When you don't need the toolkit

Some workflows are simpler with the standalone CLIs:

- **Just an ms1 round-trip.** Use `ms encode` + `ms decode`. No
  toolkit dependency.
- **Just an md1 round-trip.** Use `md encode` + `md decode` or
  `md verify`. No toolkit dependency.
- **Library integration in Rust.** Use the codec crates directly;
  the toolkit is an integration-and-CLI layer atop them.

## When you need the toolkit

- **Multi-card bundles.** The toolkit synthesises and verifies
  ms1+mk1+md1 together; the standalone CLIs each see only their own
  format.
- **Cross-binding verification.** `policy_id_stub` cross-binding is
  computed by the toolkit, not by the standalone CLIs.
- **Multi-source multisig.** Each cosigner's seed → bundle synthesis
  is a toolkit feature; the standalone CLIs handle one card at a
  time.
- **Wallet export.** `mnemonic export-wallet` produces Bitcoin Core /
  BIP-388 / Sparrow / Specter artifacts; standalone codecs produce
  card strings only.
- **BIP-85.** `mnemonic derive-child` is toolkit-only.

The next chapters compare narrower pairs:
[`mnemonic` vs `ms`](#mnemonic-toolkit-vs-ms-cli),
[`mnemonic` vs `md`](#mnemonic-toolkit-vs-md-cli),
[m-format constellation vs other backup standards](#m-format constellation-vs-slip-39-vs-naked-bip-39-vs-shamir),
[single-sig vs multisig](#single-sig-vs-multisig-decision-tree),
[BIP-39 vs BIP-38 passphrases](#bip-39-vs-bip-38-passphrases),
[Bitcoin Core importdescriptors vs BIP-388](#bitcoin-core-importdescriptors-vs-bip-388-wallet-policy).
