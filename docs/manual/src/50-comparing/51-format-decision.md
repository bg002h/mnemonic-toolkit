# Format-by-format decision table

When does each format earn its place? This chapter lays out the four
m-format constellation surfaces side by side so a reader can pick *which*
format a given task targets — and then jump to the right tool.

## The four surfaces

| Format | Carries | Reach (network) | Standalone CLI | Library |
|---|---|---|---|---|
| `ms1` | BIP-39 entropy | mainnet / testnet / signet / regtest | `ms` | `ms-codec` (Rust) |
| `mk1` | xpub + origin (fingerprint + path) | mainnet / testnet / signet / regtest | `mk` | `mk-codec` (Rust) |
| `md1` | wallet policy (template + bound xpub) | mainnet / testnet / signet / regtest | `md` | `md-codec` (Rust) |
| `mnemonic-toolkit` | integration over the three card formats | all four; multi-source slot inputs | `mnemonic` | `mnemonic-toolkit` (Rust) |

## Pick by task

| Task | Use |
|---|---|
| Engrave a single-sig BIP-84 wallet | `mnemonic bundle --template bip84 --slot @0.phrase=…` |
| Engrave a 2-of-3 multisig wallet | `mnemonic bundle --template wsh-sortedmulti --threshold 2 --slot @0.phrase=… --slot @1.phrase=… --slot @2.phrase=…` |
| Engrave a taproot multisig | `mnemonic bundle --template tr-sortedmulti-a --taproot-internal-key nums …` |
| Re-derive a phrase from an ms1 card | `mnemonic convert --from ms1=- --to phrase` (or `ms decode --in <FILE>`) |
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
[m-format constellation vs other backup standards](#m-format-vs-slip-39-vs-naked-bip-39-vs-shamir),
[single-sig vs multisig](#single-sig-vs-multisig-decision-tree),
[BIP-39 vs BIP-38 passphrases](#bip-39-vs-bip-38-passphrases),
[Bitcoin Core importdescriptors vs BIP-388](#bitcoin-core-importdescriptors-vs-bip-388-wallet-policy).

## Which tool owns which verb

Each per-format tool (`ms`, `mk`, `md`, `mt`) centres on `encode` and `decode` for its own format, plus the verbs only that format has. A verb that several formats implement with largely the same code belongs to the toolkit (`mnemonic`), so that one engine serves every format. `repair` moves first. `inspect`, `verify`, `address` and `derive` are candidates to move later, each only where its code is largely the same across formats; a part that is specific to one format stays with its tool. `mr`, the Refugium tool, keeps its own verbs; only its `repair` moves.

| Tool | Keeps | Moving to `mnemonic` (planned) | Candidates to move later |
|---|---|---|---|
| `ms` | `encode`, `decode`, `hashlock`, `split`, `combine` | `repair` | `inspect`, `verify`, `derive` |
| `mk` | `encode`, `decode` | `repair` | `inspect`, `verify`, `address`, `derive` |
| `md` | `encode`, `decode`, `bytecode`, `compile`, `compose`, `shape-key`, `descriptor`, `decompose` | `repair` | `inspect`, `verify`, `address` |
| `mt` | `encode`, `decode` | none: `mt` has no `repair` | `inspect`, `verify` |
| `mr` | `create`, `upgrade`, `verify`, `decode`, `member`, `address`, `current`, `units`, `export`, `root`, `child`, `lost-or-stolen`, `procedure` | `repair` | none |

Each tool also keeps its own `vectors`, `gui-schema` and `gen-man` where it has them: they describe that tool, not a format.

Today `mnemonic repair` repairs `ms1`, `mk1` and `md1` strings, except an `ms1` hashlock preimage plate (`ms10hash…`), which it does not read yet: repair that plate with `ms repair`. It does not accept `mr1` or `mt1` yet: repair an `mr1` string with `mr repair`, and `mt` corrects substituted characters in `mt1` strings while it reads them. `mnemonic repair` is planned to accept `mr1` and `mt1`, and `ms repair`, `mk repair`, `md repair` and `mr repair` are planned for removal.

Every other `mnemonic` verb (bundles, conversion, wallet import and export, BIP-85 and more; the `mnemonic` reference chapter lists them) is the toolkit's own. `mnemonic ms-shares` splits and combines the same codex32 shares as `ms split` and `ms combine`, through the same `ms-codec` engine; no move is planned for either.

`me` (mnemonic-engrave) is not a per-format tool: it prepares wallet backups and signed transactions for a SeedHammer II, owns no card format, and none of its verbs moves.
