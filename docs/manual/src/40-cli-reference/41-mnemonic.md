# `mnemonic` reference

The integration-layer CLI for the m-format constellation. Nine subcommands:
[`bundle`](#mnemonic-bundle), [`verify-bundle`](#mnemonic-verify-bundle),
[`convert`](#mnemonic-convert), [`export-wallet`](#mnemonic-export-wallet),
[`derive-child`](#mnemonic-derive-child), [`final-word`](#mnemonic-final-word),
[`seed-xor`](#mnemonic-seed-xor), [`slip39`](#mnemonic-slip39), and
[`gui-schema`](#mnemonic-gui-schema) (introspection only, no user-facing
semantics). Run any with `--help` for the latest flag set; this chapter
mirrors v0.13.0.

---

## `mnemonic bundle`

Synthesise a 3-card engraving bundle from a phrase, entropy, or
xpub. Inputs are slotted via `--slot @N.<subkey>=<value>`, repeating.

### Synopsis

```sh
mnemonic bundle --network <NETWORK> [OPTIONS]
```

### Flags

| Flag | Purpose |
|---|---|
| `--network <NETWORK>` | mainnet / testnet / signet / regtest |
| `--template <TEMPLATE>` | bip44 / bip49 / bip84 / bip86 / wsh-multi / wsh-sortedmulti / sh-wsh-multi / sh-wsh-sortedmulti / tr-multi-a / tr-sortedmulti-a |
| `--descriptor <DESCRIPTOR>` | user-supplied BIP-388 descriptor; mutually exclusive with `--template` and `--descriptor-file` |
| `--descriptor-file <DESCRIPTOR_FILE>` | descriptor read from a single-line UTF-8 file; mutually exclusive with `--descriptor` |
| `--language <LANGUAGE>` | BIP-39 wordlist for the input phrase |
| `--passphrase <PASSPHRASE>` | BIP-39 mnemonic-extension passphrase |
| `--passphrase-stdin` | read `--passphrase` from stdin (raw, NULL-byte preserving); single stdin per invocation |
| `--account <ACCOUNT>` | BIP-32 account index (default 0) |
| `--json` | emit JSON output |
| `--no-engraving-card` | suppress the stderr engraving-card layout |
| `--multisig-path-family <FAMILY>` | bip48 or bip87 (default bip87) |
| `--privacy-preserving` | suppress the master fingerprint from mk1 + engraving card |
| `--self-check` | re-parse and verify the emitted bundle round-trips |
| `--threshold <THRESHOLD>` | multisig K of N (1 ≤ K ≤ N ≤ 16) |
| `--slot <SLOT>` | repeating; `@N.<subkey>=<value>` (subkey: `phrase`, `entropy`, `xpub`, `fingerprint`, `path`, `wif`, `xprv`); for secret-bearing subkeys `=-` reads from stdin |
| `--help` | print help |

### Worked example

See [Your first bundle](#your-first-bundle) for a single-sig
walkthrough; [Multi-source 2-of-3 multisig](#multi-source-2-of-3-multisig)
for multisig.

---

## `mnemonic verify-bundle`

Re-derive expected card content from a seed (or from a partial set
of cards) and report per-card pass/fail plus the overall verdict.

### Synopsis

```sh
mnemonic verify-bundle --network <NETWORK> [OPTIONS] [--ms1 ...] [--mk1 ...] [--md1 ...]
```

### Flags

| Flag | Purpose |
|---|---|
| `--network <NETWORK>` | mainnet / testnet / signet / regtest |
| `--template <TEMPLATE>` | as for `bundle` |
| `--descriptor <DESCRIPTOR>` | user-supplied BIP-388 descriptor |
| `--descriptor-file <DESCRIPTOR_FILE>` | descriptor read from file |
| `--threshold <THRESHOLD>` | multisig threshold |
| `--multisig-path-family <FAMILY>` | bip48 or bip87 |
| `--privacy-preserving` | match a privacy-preserving mk1 emission |
| `--language <LANGUAGE>` | BIP-39 wordlist |
| `--passphrase <PASSPHRASE>` | BIP-39 mnemonic passphrase |
| `--passphrase-stdin` | read `--passphrase` from stdin (raw, NULL-byte preserving); single stdin per invocation |
| `--account <ACCOUNT>` | BIP-32 account index |
| `--slot <SLOT>` | repeating slot input; for secret-bearing subkeys `=-` reads from stdin |
| `--bundle-json <PATH>` | read the bundle from a JSON file emitted by `bundle --json` |
| `--ms1 <STRING>` | repeating; one ms1 card |
| `--mk1 <STRING>` | repeating; one mk1 card |
| `--md1 <STRING>` | repeating; one md1 card |
| `--json` | JSON output |
| `--help` | print help |

### Worked example

See [Verifying a bundle](#verifying-a-bundle).

---

## `mnemonic convert`

Single-format conversion across the 13-node typed graph: `phrase`,
`entropy`, `xpub`, `xprv`, `wif`, `fingerprint`, `path`, `ms1`, `mk1`,
`bip38`, `minikey`, `electrum-phrase`, `address`.

### Synopsis

```sh
mnemonic convert --from <NODE>=<value> --to <NODE> [--to <NODE>]... [OPTIONS]
```

### Flags

| Flag | Purpose |
|---|---|
| `--from <FROM>` | source node (`phrase=…`, `entropy=…`, `xpub=…`, `xprv=…`, `wif=…`, `ms1=…`, `mk1=…`, `bip38=…`, `minikey=…`, `electrum-phrase=…`); `=-` reads from stdin |
| `--to <TO>` | target node; repeating for multi-output |
| `--network <NETWORK>` | mainnet / testnet / signet / regtest |
| `--template <TEMPLATE>` | as for `bundle` |
| `--path <PATH>` | derivation path override |
| `--account <ACCOUNT>` | account index (default 0) |
| `--language <LANGUAGE>` | BIP-39 wordlist |
| `--passphrase <PASSPHRASE>` | BIP-39 passphrase |
| `--passphrase-stdin` | read `--passphrase` from stdin (raw, NULL-byte preserving); BIP-38 V3 use case |
| `--bip38-passphrase <BIP38_PASSPHRASE>` | distinct BIP-38 Scrypt passphrase channel (v0.8 BREAKING — separate from `--passphrase`) |
| `--bip38-passphrase-stdin` | read `--bip38-passphrase` from stdin (raw, NULL-byte preserving); closes the BIP-38 V3 spec NULL-byte passphrase argv gap |
| `--electrum-version <ELECTRUM_VERSION>` | Electrum seed-version selector for `(Entropy, ElectrumPhrase)` |
| `--electrum-language <ELECTRUM_LANGUAGE>` | Electrum-specific wordlist (English + 4 non-English) |
| `--fingerprint <FINGERPRINT>` | master fingerprint (input on certain edges) |
| `--xpub-prefix <XPUB_PREFIX>` | SLIP-0132 prefix selector for emitted xpubs (e.g. zpub, ypub) |
| `--script-type <SCRIPT_TYPE>` | `p2wpkh` / `p2sh-p2wpkh` / `p2tr` for `(Xpub, Address)` derivation |
| `--json` | JSON output |
| `--help` | print help |

### Worked example

See [Minimal recovery walkthrough](#minimal-recovery-walkthrough)
and [Migrating from BIP-39-only to the m-format constellation](#migrating-from-bip-39-only-to-the-m-format constellation).

---

## `mnemonic export-wallet`

Emit watch-only wallet artifacts for Bitcoin Core, BIP-388, Coldcard,
Blockstream Jade, Sparrow, or Specter.

### Synopsis

```sh
mnemonic export-wallet [OPTIONS]
```

### Flags

| Flag | Purpose |
|---|---|
| `--template <TEMPLATE>` | as for `bundle` |
| `--descriptor <DESCRIPTOR>` | user-supplied BIP-388 descriptor |
| `--threshold <THRESHOLD>` | multisig threshold |
| `--multisig-path-family <FAMILY>` | bip48 or bip87 |
| `--network <NETWORK>` | default mainnet |
| `--language <LANGUAGE>` | ignored (watch-only); accepted for slot-parser symmetry |
| `--account <ACCOUNT>` | account index (default 0) |
| `--slot <SLOT>` | repeating `@N.<subkey>=<value>`; subkeys: `phrase`, `entropy`, `xpub`, `master_xpub`, `fingerprint`, `path`, `wif`, `xprv` (secret-bearing subkeys refused by `export-wallet`'s watch-only validator) |
| `--format <FORMAT>` | `bitcoin-core` (default) / `bip388` / `coldcard` / `jade` / `sparrow` / `specter` / `electrum` / `green` |
| `--output <OUTPUT>` | output path (`-` = stdout, default) |
| `--range <RANGE>` | Bitcoin Core `range` field; comma-separated; default `0,999` |
| `--timestamp <TIMESTAMP>` | Bitcoin Core `timestamp` field; `now` (default) or unix seconds |
| `--bitcoin-core-version <BITCOIN_CORE_VERSION>` | 24 or 25 (default 25) |
| `--wallet-name <WALLET_NAME>` | wallet name/label for formats that publish one (Coldcard generic JSON, Sparrow, Specter, Electrum); default `<template-human-name>-<account>` |
| `--taproot-internal-key <TAPROOT_INTERNAL_KEY>` | `nums` or `@N` for `tr-multi-a` / `tr-sortedmulti-a` |
| `--help` | print help |

### Notes

- **`--wallet-name` length cap.** The Coldcard multisig text (`--format coldcard` with a `wsh-*` / `sh-wsh-*` template) and the byte-identical Jade multisig text (`--format jade`) cap the `Name:` line at 20 Unicode scalar values per the Coldcard reference format. Longer names are truncated to the first 20 characters (not bytes — non-ASCII names are handled at codepoint granularity, so `🤐🤐🤐…` truncates cleanly without splitting a multi-byte sequence).
- **`@N.master_xpub=` parse vs emit.** The `master_xpub` slot subkey parses successfully under any `--format`, but `--format coldcard` with a singlesig template (`bip44` / `bip49` / `bip84`) currently refuses when the subkey is supplied because the resolution pipeline does not yet plumb the master xpub through to the Coldcard generic-JSON top-level `xpub` field (tracked by `design/FOLLOWUPS.md` entry `coldcard-master-xpub-plumbing-pending`, scheduled for v0.8.2). Re-invoke without the `master_xpub` slot to emit the JSON with the top-level `xpub` field omitted (which is what Coldcard accepts in the absence of a depth-0 xpub). Other formats silently ignore the subkey per the per-format ignored-input contract.
- **`--threshold` is REQUIRED for `--format sparrow` multisig.** Bitcoin Core / BIP-388 / Coldcard / Jade auto-default `K = N` (cosigner count) when `--threshold` is omitted, but Sparrow refuses with a missing-info error: Sparrow publishes the threshold in `defaultPolicy.miniscript.script` as `multi(K, ...)` / `sortedmulti(K, ...)`, and silently defaulting `K = N` would emit a wallet that looks like K=N was intentional rather than a missing-input default. Supply `--threshold <K>` explicitly when `--format sparrow` and the template is multisig.
- **`--wallet-name` is REQUIRED for `--format specter`.** Specter Desktop's UX requires an explicit wallet label; emitting a Specter wallet without one produces a wallet that displays as an empty string in the Specter UI (a UX regression vs. the user's likely intent). Other formats fall back to `<template-human-name>-<account>` when `--wallet-name` is omitted; Specter refuses via the SPEC §4 missing-info channel.

### Worked example

See [Exporting to Bitcoin Core / BIP-388 / Sparrow / Specter](#exporting-to-bitcoin-core-bip-388-sparrow-specter).

---

## `mnemonic derive-child`

BIP-85 deterministic child entropy. Six in-scope applications:
`bip39`, `hd-seed`, `xprv`, `hex`, `password-base64`, `password-base85`,
plus `dice` (BIP-85 v1.3.0).

### Synopsis

```sh
mnemonic derive-child --from <FROM> --application <APP> --length <LEN> --index <INDEX> [OPTIONS]
```

### Flags

| Flag | Purpose |
|---|---|
| `--from <FROM>` | `xprv=<value>` or `phrase=<bip39>` (with `--passphrase` + `--language`); `=-` to read from stdin |
| `--application <APPLICATION>` | `bip39` / `hd-seed` / `xprv` / `hex` / `password-base64` / `password-base85` / `dice` |
| `--length <LENGTH>` | application-specific size; pass `0` for `hd-seed` and `xprv` |
| `--index <INDEX>` | hardened child index (`0..2^31`) |
| `--network <NETWORK>` | for `hd-seed` / `xprv` apps; defaults to mainnet |
| `--language <LANGUAGE>` | BIP-39 wordlist + BIP-85 language code for `--application bip39` |
| `--passphrase <PASSPHRASE>` | BIP-39 passphrase, only for `--from phrase=…` |
| `--passphrase-stdin` | read `--passphrase` from stdin (raw, NULL-byte preserving); single stdin per invocation |
| `--dice-sides <DICE_SIDES>` | required for `--application dice`; range `2..=2^32-1` |
| `--help` | print help |

### Worked example

See [Deterministic child secrets via BIP-85](#deterministic-child-secrets-via-bip-85).

---

## `mnemonic final-word`

Given an N-1 word BIP-39 partial phrase, emit the lexicographically
sorted set of wordlist entries that, when appended as the Nth word,
yield a phrase with a valid BIP-39 checksum. Output set size is a
function of N alone: 128 for N=12, 64 for N=15, 32 for N=18, 16 for
N=21, 8 for N=24.

Use cases include paper-backup recovery (a smudged last word), manual
seed generation (compute the only-valid checksum-fixing word for a
hand-rolled partial), and phrase-typo verification (look up whether
your last word appears in the candidate set for the first N-1 you've
written down).

### Synopsis

```sh
mnemonic final-word --from <phrase=<value-or-->> [--language <LANGUAGE>] [--json-out <PATH>]
```

### Flags

| Flag | Purpose |
|---|---|
| `--from <phrase=<value-or-->>` | partial phrase as `phrase=<N-1 words>` (inline) or `phrase=-` to read from stdin; inline form emits a `/proc/$PID/cmdline` argv-leakage advisory on stderr |
| `--language <LANGUAGE>` | BIP-39 wordlist; one of `english` / `simplifiedchinese` / `traditionalchinese` / `czech` / `french` / `italian` / `japanese` / `korean` / `portuguese` / `spanish` (default `english`) |
| `--json-out <PATH>` | side-effect: write a versioned JSON envelope to `<PATH>` in addition to the plain candidate list on stdout; on Unix a world-readable result raises a permission-mode advisory |
| `--help` | print help |

### Worked example

```sh
echo "abandon abandon abandon abandon abandon abandon abandon abandon \
abandon abandon abandon abandon abandon abandon abandon abandon \
abandon abandon abandon abandon abandon abandon abandon" |
  mnemonic final-word --from phrase=- --language english
```

Stdout: 8 sorted candidate words, one per line — including `art` (the
canonical zero-entropy 24-word vector). For N=12 partial input
(`abandon × 11`), the output is 128 lines including `about` (the
canonical Trezor zero-entropy 12-word vector).

### JSON output

```json
{
  "schema_version": "1",
  "language": "english",
  "partial_word_count": 11,
  "target_word_count": 12,
  "candidate_count": 128,
  "candidates": ["abandon", "ability", "above", "..."]
}
```

Field order is part of the schema (SHA-pinned in
`tests/cli_final_word_json.rs`). `candidates` is lexicographically
sorted; `candidate_count == candidates.len()`. The plain stdout output
is emitted in parallel (the JSON file is a side-effect, not a
stdout-replacement).

### Refusals

| Trigger | Refusal |
|---|---|
| Partial word count not in `{11, 14, 17, 20, 23}` | `final-word: got K words; expected one of [11, 14, 17, 20, 23] ...` |
| Empty partial (0 words after `split_whitespace`) | `final-word: empty partial phrase; need 11/14/17/20/23 words ...` |
| Unknown word at position I | `final-word: unknown BIP-39 word at position I (not in selected wordlist; did you pick the right --language?)` |
| `--from` variant other than `phrase=` | `final-word --from only accepts phrase=<value> or phrase=-` |

### Advisories

| Trigger | Stderr advisory |
|---|---|
| Inline `--from phrase=<value>` | `warning: secret material on argv (--from phrase=) — pipe via --from phrase=- to avoid /proc/$PID/cmdline exposure` |
| Stdout is a TTY AND candidate set non-empty | `warning: candidate list is secret material — pairing the partial phrase with any candidate yields a valid seed phrase; do not paste this output into untrusted tools` |
| `--json-out PATH` with world-readable file (Unix umask 022 default) | `warning: --json-out <PATH> inherits umask (file may be world-readable, mode 644); consider --json-out /dev/stdout or chmod 0600 the path before invoking` |

---

## `mnemonic seed-xor`

Coldcard-compatible BIP-39 ↔ BIP-39 all-or-nothing XOR-based seed splitter.
Two sub-subcommands: `split` (master phrase → N BIP-39 shares) and `combine`
(N shares → master phrase). NOT a threshold scheme — ALL N shares are
required to reconstruct (for K-of-N use SLIP-39, planned for v0.13.0).

**Coldcard interop:** native at 12/18/24-word sizes (per Coldcard
`shared/xor_seed.py` accepting entropy lengths 16/24/32 bytes). 15/21-word
sizes are toolkit-only extensions; Coldcard hardware cannot round-trip
those two sizes.

**Security caveat:** Seed XOR has no authentication tag. Substitution of
a wrong-but-valid-BIP-39 share is mathematically undetectable — the
recovered phrase will validate but derive the wrong wallet. Verify the
recovered wallet's expected derived address before trusting.

### Synopsis

```sh
mnemonic seed-xor split   --from <phrase=<value-or-->> --shares <N> [OPTIONS]
mnemonic seed-xor combine --share <phrase=<value-or-->> ... --shares <N> [OPTIONS]
```

### `seed-xor split` flags

| Flag | Purpose |
|---|---|
| `--from <phrase=<value-or-->>` | master phrase as `phrase=<value>` (inline) or `phrase=-` to read from stdin; inline form emits a `/proc/$PID/cmdline` argv-leakage advisory on stderr |
| `--shares <N>` | number of shares to emit; must be >= 2 |
| `--language <LANGUAGE>` | BIP-39 wordlist: `english` (default) / `simplifiedchinese` / `traditionalchinese` / `czech` / `french` / `italian` / `japanese` / `korean` / `portuguese` / `spanish` |
| `--deterministic-from-master` | use Coldcard's SHA256d-deterministic share generation instead of OS CSPRNG; required for byte-equal Coldcard hardware interop |
| `--json-out <PATH>` | side-effect: write versioned JSON envelope to PATH (does NOT replace stdout) |
| `--help` | print help |

### `seed-xor combine` flags

| Flag | Purpose |
|---|---|
| `--share <phrase=<value-or-->>` | share phrase; repeating; at most ONE may be `phrase=-` (single stdin per invocation) |
| `--shares <N>` | asserted share count; MUST equal the number of `--share` flags (hard refusal on mismatch — catches cardinality omissions, NOT substitution) |
| `--language <LANGUAGE>` | BIP-39 wordlist of inputs + output (default `english`) |
| `--json-out <PATH>` | side-effect: write versioned JSON envelope |
| `--help` | print help |

### Worked example

```sh
# Split a 24-word seed into 3 shares (deterministic, Coldcard-interop)
echo "abandon abandon abandon abandon abandon abandon abandon abandon \
abandon abandon abandon abandon abandon abandon abandon abandon \
abandon abandon abandon abandon abandon abandon abandon art" |
  mnemonic seed-xor split --from phrase=- --shares 3 --deterministic-from-master
```

Stdout: 3 lines, each a 24-word BIP-39 phrase. Reverse via:

```sh
mnemonic seed-xor combine \
  --share "phrase=<share-1>" \
  --share "phrase=<share-2>" \
  --share "phrase=<share-3>" \
  --shares 3
```

Stdout: the original 24-word phrase recovered.

### JSON output

`--json-out <PATH>` writes a versioned envelope. Schema `v1`. `split`
shape:

```json
{
  "schema_version": "1",
  "operation": "split",
  "language": "english",
  "word_count": 12,
  "share_count": 3,
  "deterministic": false,
  "shares": ["phrase-1 ...", "phrase-2 ...", "phrase-3 ..."]
}
```

`combine` shape:

```json
{
  "schema_version": "1",
  "operation": "combine",
  "language": "english",
  "word_count": 12,
  "share_count": 3,
  "phrase": "reconstructed phrase ..."
}
```

Field order is part of the schema (SHA-pinned in
`tests/cli_seed_xor_json.rs`).

### Refusals

| Trigger | Refusal |
|---|---|
| `split --from` phrase word-count not in {12,15,18,21,24} | `seed-xor split: phrase must be 12/15/18/21/24 words; got K` |
| `split --shares` < 2 | `seed-xor split: --shares must be >= 2; got N` |
| `combine --share` count mismatch vs `--shares` | `seed-xor combine: --shares N requires exactly N --share arguments; got K --share values for --shares N` |
| `combine` mixed-length shares | `seed-xor combine: all shares must be the same word count; got mix of {K1, K2, ...}` |
| `combine` share at position I has BIP-39 checksum failure | `seed-xor combine: share at position I has invalid BIP-39 checksum (...)` |
| `combine` unknown word in share at position I | `seed-xor combine: share at position I: unknown BIP-39 word at index J ...` |
| `--from` or `--share` variant other than `phrase=` | `seed-xor only accepts phrase=<value> or phrase=-` |
| Two or more `--share phrase=-` (multi-stdin) | `seed-xor combine: at most one --share value may be \`-\` (single stdin per invocation)` |

### Advisories

| Trigger | Stderr advisory |
|---|---|
| Inline `--from phrase=<v>` OR inline `--share phrase=<v>` | `warning: secret material on argv (--from phrase= OR --share phrase=) — pipe via phrase=- to avoid /proc/$PID/cmdline exposure` (per-occurrence) |
| `split` AND stdout is a TTY | `warning: Seed XOR shares on stdout — each of the N=<n> lines is independently a complete BIP-39 phrase; ALL N shares are required to reconstruct the master; distribute them to N separate locations; do not paste this output into a single untrusted tool. Substitution of a wrong-but-valid-BIP-39 share is undetectable by Seed XOR — verify the recovered wallet's derived address before trusting it.` |
| `combine` AND stdout is a TTY | `warning: combined phrase is secret material — Seed XOR has no authentication tag; verify the recovered wallet's expected derived address before trusting; if a share was substituted with a wrong-but-valid one, the result will validate but derive the wrong wallet` |
| `split --deterministic-from-master` with 15/21-word input | `warning: --deterministic-from-master with 15-word input is toolkit-only — Coldcard's xor_seed.py natively supports 12/18/24 only; resulting shares will NOT round-trip a Coldcard device. For Coldcard interop, use 12/18/24-word input.` |
| `--json-out <PATH>` with world-readable file (Unix) | `warning: --json-out <PATH> inherits umask (file may be world-readable, mode 644); consider --json-out /dev/stdout or chmod 0600 the path before invoking` |

---

## `mnemonic slip39`

SLIP-39 K-of-N share-splitter. Two sub-subcommands: `split` (master
secret → groups × members of SLIP-39 mnemonic shares) and `combine`
(≥K shares → master secret). Unlike `seed-xor` this IS a threshold
scheme — any K-of-N subset reconstructs.

*Canonical chapter content (worked example, JSON envelope schema,
refusal taxonomy, advisory matrix, Trezor interop recipe) lands at
v0.13.0 P3.* This stub mentions every flag the
`mnemonic slip39 split --help` and `mnemonic slip39 combine --help`
output emits so the manual `flag-coverage` lint passes; see
`mnemonic slip39 --help` for the live synopsis.

### `slip39 split` flags

| Flag | Purpose |
|---|---|
| `--from <phrase=…\|entropy=…>` | master secret as `phrase=<value-or->` or `entropy=<hex-or->`; `=-` reads from stdin |
| `--passphrase <P>` | SLIP-39 passphrase (NOT the BIP-39 mnemonic-extension passphrase) |
| `--passphrase-stdin` | read `--passphrase` from stdin (single stdin per invocation) |
| `--group-threshold <G>` | groups required to reconstruct (1 ≤ G ≤ group count) |
| `--group <N,T>` | repeating group spec (`<member_count>,<member_threshold>`); position in argv = SLIP-39 `group_idx` |
| `--iteration-exponent <E>` | PBKDF2 cost; iterations = 10000 · 2^E (range 0..=15, default 0); E ≥ 5 emits a perf advisory |
| `--language <LANGUAGE>` | BIP-39 wordlist of input phrase; ignored for `entropy=` inputs |
| `--json-out <PATH>` | side-effect: write versioned JSON envelope to `<PATH>` (in addition to plain-stdout shares) |
| `--help` | print help |

### `slip39 combine` flags

| Flag | Purpose |
|---|---|
| `--share <slip39-mnemonic-or->` | repeating share input; at most ONE may be `-` (stdin) |
| `--passphrase <P>` | SLIP-39 passphrase used at split time |
| `--passphrase-stdin` | read `--passphrase` from stdin (incompatible with any `--share -`) |
| `--to <entropy\|phrase>` | output shape (default `entropy`); `phrase` emits a BIP-39 mnemonic per `--language` |
| `--language <LANGUAGE>` | BIP-39 wordlist for `--to phrase`; ignored for `--to entropy` |
| `--json-out <PATH>` | side-effect: write versioned JSON envelope to `<PATH>` (in addition to plain-stdout secret) |
| `--help` | print help |

---

## `mnemonic gui-schema`

Emit the SPEC §7 machine-readable schema of every existing
subcommand's flag surface as JSON to stdout. Companion to the
`mnemonic-gui` v0.2 schema-mirror contract — the GUI consumes this
output to render forms and refuses to launch on `version != 1`.

The schema is generated by walking the clap-derive `Command` tree
via `clap::CommandFactory`; the `gui-schema` subcommand itself is
filtered out (self-reference suppression).

### Synopsis

```sh
mnemonic gui-schema
```

### Flags

| Flag | Purpose |
|---|---|
| `--help` | print help |

### Output shape

```json
{
  "version": 1,
  "cli": "mnemonic",
  "subcommands": [
    {
      "name": "bundle",
      "flags":       [ {"name": "--network", "required": true, "kind": "dropdown", "choices": ["mainnet","testnet","signet","regtest"]} ],
      "positionals": []
    }
  ]
}
```

`kind` is one of `text` / `boolean` / `number` / `dropdown` / `path`.
`choices` is non-null only when `kind == "dropdown"`. Complex
GUI-side variants (NodeValueComposite, TaggedOrIndexed, Range,
Timestamp) intentionally collapse to `"text"` upstream and are
re-parsed client-side per the SPEC §7 lossy-mapping contract.
