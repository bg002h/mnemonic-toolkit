# `ms` — per-tab reference

The `ms` tab covers the BIP-39-entropy CLI (`ms-cli`), nine
subcommands that operate on `ms1` cards (the secret card of the
m-format constellation bundle). The `ms1` encodes the raw BIP-39
entropy bytes in a BIP-93 / codex32 envelope — the seed card that
recovers the wallet on its own.

The `ms` tab's pinned upstream version at v1.1 of this manual is
`ms-cli v0.8.0` (per `docs/manual-gui/pinned-upstream.toml`).
Pinned-banner format `Pinned: ms 0.8.0`.

## Subcommand index

The nine subcommands group into five families:

- **Encode + decode.** Round-trip from BIP-39 entropy to `ms1` and
  back.
  - [`ms encode`](#ms-encode)\index{ms encode} — emit an `ms1`
    from a BIP-39 mnemonic or hex entropy.
  - [`ms decode`](#ms-decode)\index{ms decode} — recover the
    BIP-39 mnemonic + entropy bytes from an `ms1`.
- **Inspect + verify.** Read structural fields, check validity,
  optionally round-trip a phrase against an `ms1`.
  - [`ms inspect`](#ms-inspect)\index{ms inspect} — verdict +
    structured fields (HRP / threshold / tag / payload bytes /
    checksum status).
  - [`ms verify`](#ms-verify)\index{ms verify} — exit-code-only
    validity (and optional `--phrase` round-trip).
- **Derive + repair.** Public derivation + BCH error correction.
  - [`ms derive`](#ms-derive)\index{ms derive} — read-only master
    fingerprint and (with `--template`) an account xpub. No
    private keys reach stdout.
  - [`ms repair`](#ms-repair)\index{ms repair} — BCH error-correct
    a single corrupted `ms1` (up to four substitutions).
- **Shamir split + combine.** BIP-93 codex32 K-of-N share
  splitting (ms-cli v0.7.0+).
  - [`ms split`](#ms-split)\index{ms split} — split a secret into
    N codex32 shares, any K of which recombine.
  - [`ms combine`](#ms-combine)\index{ms combine} — recombine ≥K
    shares back into the original secret.
- **Maintainer tools.**
  - [`ms vectors`](#ms-vectors)\index{ms vectors} — print the
    SHA-pinned v0.1 test-vector corpus as JSON (typically used by
    ms-cli developers, not end users).

## Form shape

All nine subcommands follow the same form scaffolding described
in [chapter 31](#first-launch-walkthrough): top-of-form
`Pinned: ms 0.8.0` label + subcommand selector ComboBox +
per-subcommand `?` help-icon; per-flag widgets; an action bar
with **Copy command**, **Run** buttons; an always-on `Preview:`
line. None of the ms-tab subcommands accept slot input
(`allows_slots: false` for all 9).

Most of the nine subcommands consume secret-bearing input and
fire the run-confirm modal on any non-empty secret value (per
`mnemonic-gui/src/secrets.rs:should_confirm_run`):

- [`ms encode --phrase`](#ms-encode-phrase) /
  [`ms encode --hex`](#ms-encode-hex) — the mutually exclusive
  seed-input pair.
- [`ms verify --phrase`](#ms-verify-phrase) — the round-trip
  phrase.
- [`ms derive`](#ms-derive) — the `ms1` positional,
  [`--hex`](#ms-derive-hex), [`--phrase`](#ms-derive-phrase), and
  [`--passphrase`](#ms-derive-passphrase).
- [`ms repair --ms1`](#ms-repair-ms1) — the corrupted seed card
  (a deliberate GUI-side `secret: true` override).
- [`ms split --phrase`](#ms-split-phrase) /
  [`ms split --hex`](#ms-split-hex) — the secret being split.
- [`ms combine`](#ms-combine) — the positional shares are
  secret-equivalent (any K recover the secret).

The threat-model warning in [§14 Defense 2](#secret-handling)
about the modal-redaction history and the recommended cold-node
operational mitigation applies to all of these.

[`ms inspect`](#ms-inspect), [`ms decode`](#ms-decode), and
[`ms vectors`](#ms-vectors) do not accept any `secret: true`
schema flag and do not fire the modal.

## Cross-tab language-token divergence

The `ms` CLI uses **hyphenated** Chinese-wordlist tokens
(`chinese-simplified`, `chinese-traditional`) where the
`mnemonic` tab uses fused tokens (`simplifiedchinese`,
`traditionalchinese`). The two are not interchangeable on the
command line — passing the wrong form to either binary yields an
argv-rejection error. The GUI's per-subcommand dropdown emits the
correct token for the active tab; this divergence only matters
when authoring or pasting argv manually. Per-variant cross-refs
in this chapter link to the `mnemonic` tab's wordlist entries by
language, not by token form.

## Worked-example data convention

Examples in this chapter use the canonical all-`abandon` BIP-39
12-word test vector (16 bytes of zero entropy):

```text
abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about
```

The canonical `ms1` for this phrase under the English wordlist is:

```text
ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f
```

This phrase is **public** and any wallet derived from it has been
swept by chain watchers since 2017 — its use in the manual is for
round-trip demonstration only. Do not engrave or fund any wallet
derived from it.
