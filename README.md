# mnemonic-toolkit

> **⚠ DISCLAIMER — UNTESTED ALPHA SOFTWARE.** **This software has not yet been independently tested or audited. Do not use the m-format constellation to back up significant sums of money at this time — doing so is tantamount to asking to be rekt.** Use only with disposable amounts, on testnet, or for evaluation. Codecs, CLIs, BCH math, and cross-card invariants have all been authored and reviewed only by the original developer. Assume bugs until external review happens.

Top-level integration crate for the **m-format constellation** of Bitcoin self-custody backup formats: takes a BIP-39 seed phrase (or watch-only xpub, or multi-source set of seeds) and emits a complete steel-engravable bundle of three sibling cards.

| Card | Format | What's on it |
|---|---|---|
| **ms1** | [`ms-codec`](https://github.com/bg002h/mnemonic-secret) | BIP-39 entropy (recovers the seed) |
| **mk1** | [`mk-codec`](https://github.com/bg002h/mnemonic-key) | xpub + origin (master fingerprint + BIP path) |
| **md1** | [`md-codec`](https://github.com/bg002h/descriptor-mnemonic) | wallet policy (template + bound xpub) |

<!-- toolkit-version: 0.53.2 -->
Status: the `mnemonic` CLI (see [CHANGELOG.md](CHANGELOG.md) for the current release; subcommands grouped under [Subcommands](#subcommands)) spans seed/key/descriptor handling across the m-format constellation: 3-card bundle synthesis + round-trip verification; single-sig (BIP-44/49/84/86) + multisig + BIP-388 descriptors + multi-leaf taproot + multi-source full multisig; guided descriptor construction (`build-descriptor`: a validated policy-tree → wsh descriptor engine with 5 archetype presets and a reviewed `--allow` sanity opt-out); cross-format wallet import/export (Bitcoin Core, BIP-388, BSMS/BIP-129, Coldcard, Sparrow, Specter, Electrum); watch-only restore documents (single-sig from a seed + passphrase, fingerprint-gated; multisig from the shared md1 card alone, incl. taproot NUMS); seed/key conversion (BIP-39 / BIP-32 / WIF / ms1 / mk1 / BIP-38 / Casascius mini-key / Electrum native seed); batch watch-only address listing; backup splitting (Coldcard seed-XOR, SLIP-39, BIP-93 codex32 K-of-N shares via `ms-shares`, SeedQR); BIP-85 child derivation; BIP-352 silent-payment receiver addresses; nostr key wrapping; legacy + BIP-322 message verification; address decoding; and BCH error-correction / inspection. Mainnet / testnet / signet / regtest. Secret-input hygiene throughout (zeroize + `mlock` + argv-leak advisories + `*-stdin` / `@env:` channels).

For the authoritative, always-current CLI reference see the **[end-user manual](docs/manual/)** (single source of truth, lint-gated against the live `--help` surface); for the full release history see **[CHANGELOG.md](CHANGELOG.md)**.

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
siblings), use the installer's `--only` flag (it carries the
current version pin, so it never goes stale):

```sh
sh -c "$(curl -fsSL https://raw.githubusercontent.com/bg002h/mnemonic-toolkit/master/scripts/install.sh)" -- --only mnemonic
```

## Subcommands

The `mnemonic` subcommands, grouped below. Run any with `--help`, or see the
**[CLI reference chapter](docs/manual/src/40-cli-reference/41-mnemonic.md)** for
the authoritative, per-flag documentation.

- **Bundle** — `bundle` (synthesize the 3-card ms1+mk1+md1 backup, or watch-only 2-card from xpub), `verify-bundle` (re-derive + parity-check across cards).
- **Convert / derive** — `convert` (seed/key conversions across the typed graph: phrase / entropy / xpub / xprv / wif / fingerprint / path / ms1 / mk1 / bip38 / minikey / electrum-phrase / address), `addresses` (list a wallet's receive/change addresses, batch watch-only), `derive-child` (BIP-85 child entropy/keys).
- **Wallet import / export** — `import-wallet` (third-party blob → m-format bundle: Bitcoin Core, BSMS/BIP-129, Coldcard, Sparrow, Specter, Electrum), `export-wallet` (watch-only artifacts: Bitcoin Core importdescriptors, BIP-388 wallet_policy, BSMS), `restore` (watch-only restore documents — single-sig from a seed + passphrase, fingerprint-gated; multisig from the shared md1 card alone, incl. taproot NUMS), `decode-address` (address → network / script type / witness version / scriptPubKey).
- **Backup splitting** — `seed-xor` (Coldcard BIP-39 XOR split/combine), `slip39` (SLIP-39 K-of-N Shamir), `ms-shares` (BIP-93 codex32 ms1 K-of-N share split/combine), `seedqr` (SeedSigner SeedQR encode/decode).
- **Keys & messages** — `nostr` (wrap an nsec/npub as BTC addresses/descriptors/WIF), `silent-payment` (BIP-352 receiver address), `verify-message` (legacy signmessage + BIP-322), `final-word` (BIP-39 checksum-completion words).
- **Decrypt / repair / inspect** — `electrum-decrypt` (Electrum field-encrypted secret), `repair` (BCH error-correct ms1/mk1/md1), `inspect` (describe a card), `compare-cost` (wsh-vs-tr spending-condition cost), `xpub-search` (locate an xpub/descriptor/address/passphrase under a seed).
- **Descriptor construction** — `build-descriptor` (versioned JSON policy-tree or archetype preset → funds-safety-gated wsh descriptor + BIP-388 + cost preview; `--allow` reviewed sanity opt-out — cost preview unavailable on an overridden emit).
- **Introspection** — `gui-schema` (emit the GUI-overlay flag-surface schema; no user-facing semantics).

The three cards engrave together as a coherent backup. Each card is independently BCH-checksummed by its sibling codec; the toolkit cross-binds them via the 4-byte `policy_id_stub` (`SHA-256(canonical wallet-policy preimage)[0..4]`) carried on each mk1 card and computable from each md1 card.

## Documentation

- **[`docs/manual/`](docs/manual/)** — the end-user manual: the single source of truth for the m-format constellation CLI surface (`mnemonic` / `md` / `ms` / `mk`), lint-gated against the live `--help` output. Tagged builds attach a PDF to the GitHub release.
- [`CHANGELOG.md`](CHANGELOG.md) — full release history.
- [`design/`](design/) — SPECs, implementation plans, per-cycle architect reviews, and [`design/FOLLOWUPS.md`](design/FOLLOWUPS.md) (deferred-work tracker).

## License

MIT License. See [LICENSE](LICENSE).
