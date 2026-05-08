# Troubleshooting

Five most common newcomer issues. For the full failure-mode matrix,
see the manual's
[Appendix G — Troubleshooting matrix](../../../manual/src/60-appendices/67-troubleshooting.md).

## 1. "I forgot `--threshold`"

```text
error: a value is required for '--threshold <THRESHOLD>'
```

Multisig templates (`wsh-multi`, `wsh-sortedmulti`, `sh-wsh-multi`,
`sh-wsh-sortedmulti`, `tr-multi-a`, `tr-sortedmulti-a`) require an
explicit threshold `K`. Single-sig templates (`bip44`, `bip49`,
`bip84`, `bip86`) do not.

**Fix:** add `--threshold K` (with `1 ≤ K ≤ N ≤ 16`). For the 2-of-3
walkthrough in [chapter 32](../30-multisig/32-bundle.md), `K = 2`.

## 2. "verify-bundle says ms1_decode error at position N"

```text
ms1_decode: error at position 27
```

A character at position 27 of the ms1 string is wrong — typically a
mis-stamped or mis-typed character on the engraved plate. The BCH
checksum localised the failure to that one position; the m-format
alphabet excludes visually-similar characters (`0` vs `o`, `1` vs
`l`/`i`), so most errors trace to a stamping artefact rather than
genuine confusion.

**Fix:** look at character N on the original digital bundle (or the
re-derived ms1 from `mnemonic convert --from phrase=… --to ms1`),
compare to the steel plate, and re-stamp that one character. Re-run
`verify-bundle` after the correction.

The same pattern applies to `mk1_decode` and `md1_decode` failures —
read the position N off the diagnostic, find that character on the
plate, re-stamp.

## 3. "Bitcoin Core won't import"

```text
error: Descriptor not in canonical form
```

Bitcoin Core 25+ accepts the toolkit's default `--format bitcoin-core`
output directly; older Bitcoin Core versions need the v24 shape,
and BIP-388 wallet-policy importers (Sparrow, Specter, Coldcard,
Ledger) need a different format entirely.

**Fix:** re-run `mnemonic export-wallet` with the format matched to
the receiving software:

- Bitcoin Core 25+ → `--format bitcoin-core` (default).
- Bitcoin Core 24 → `--format bitcoin-core --bitcoin-core-version 24`.
- Sparrow / Specter / Coldcard / Ledger → `--format bip388`.

Note: `--format sparrow` and `--format specter` are reserved values
that currently return a deferral stub — use `--format bip388` for
those wallets and import via their BIP-388-aware path. The manual's
[Exporting to Bitcoin Core / BIP-388 / Sparrow / Specter](../../../manual/src/30-workflows/37-wallet-export.md)
chapter covers the import-shape matrix.

## 4. "Wrong xpub for my wallet"

`mnemonic bundle` produced a perfectly-valid bundle, but the
addresses don't match the wallet you intended to back up. The
toolkit derived a different xpub than your existing wallet uses.

**Cause:** mismatched `--template`, `--account`, or `--network`. The
xpub depends on all three; changing any one produces a different
xpub. Common pitfalls:

- BIP-44 vs. BIP-49 vs. BIP-84 vs. BIP-86 — each emits a different
  xpub at a different derivation path.
- `--account 0` (default) vs. `--account 1` — non-zero accounts
  derive at `m/<purpose>'/<coin>'/N'` for the selected `N`.
- `mainnet` vs. `testnet` — different network identifiers, different
  xpub prefixes.

**Fix:** before stamping, run `mnemonic convert --from phrase=… --to xpub`
with the same `--template`, `--account`, and `--network` as your
bundle invocation, and compare to your wallet's xpub. If they
match, the bundle is correct; if not, adjust the flags to match the
wallet you're backing up.

## 5. "I'm on Windows and the symlinks broke"

The Quick Start and the reference manual share files via git
symlinks (e.g., the worked-example transcripts). Windows clones
default to leaving symlinks as plain text files containing the path
target, which breaks builds and reads.

**Fix:** enable symlink support in git:

```sh
git config --global core.symlinks true
git checkout -- .  # re-materialise as symlinks
```

Symlink creation on Windows requires either Developer Mode enabled
or running git as administrator. The Microsoft documentation on
"Enable Developer Mode" walks through the setting; `core.symlinks=true`
on its own is necessary but not sufficient.

## When in doubt

1. **Re-read the chapter.** Most failures trace to a small flag-set
   discrepancy between the example and the actual command.
2. **Run `--help` directly.** The cli-help snapshots in the manual
   match toolkit v0.8.0; later versions may have added flags.
3. **Run `verify-bundle` on the engraved cards.** It is the single
   most useful diagnostic — the BCH error-position diagnostic
   names the failing card and the character offset.

For the full failure-mode matrix — bundle synthesis, verify-bundle,
convert / recovery, engraving, wallet-export, and BIP-85 — see
[Appendix G — Troubleshooting matrix](../../../manual/src/60-appendices/67-troubleshooting.md).
