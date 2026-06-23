# `mnemonic xpub-search passphrase-of-xpub` {#mnemonic-xpub-search-passphrase-of-xpub}

Reverse-search a BIP-32 derivation graph: given a seed (BIP-39 phrase
or `ms1` card) **plus a specific passphrase** (or a candidate list)
and a target xpub, verify that the passphrase produces that xpub under
the seed at one of the standard derivation templates (BIP-44 / 49 / 84
/ 86 single-sig + BIP-48 multisig at `script_type ∈ {1', 2', 3'}`) ×
account range. The semantic difference from `path-of-xpub` is that
this mode answers **"does THIS passphrase produce the xpub?"** rather
than **"what path produced this xpub?"**. The GUI exposes this as one
form under the **mnemonic** tab's subcommand selector.
\index{mnemonic xpub-search passphrase-of-xpub}

This mode takes secret seed + passphrase material. The GUI renders the
secret inputs as masked `SecretLineEdit` widgets, and the run-confirm
modal redacts secret-bearing argv tokens as a fixed `••••` sentinel —
the literal secret is never drawn on screen (see
[§14 Defense 2](#secret-handling)). Read-only verification: **no
private keys reach stdout.**

:::danger
The worked example in this chapter uses the canonical all-`abandon`
BIP-39 test vector. **Never engrave or fund** a wallet derived from
this phrase — chain watchers have swept it continuously since 2017.
:::

## Outline {#mnemonic-xpub-search-passphrase-of-xpub-outline}

- [`--phrase`](#mnemonic-xpub-search-passphrase-of-xpub-phrase) — master BIP-39 phrase (inline)
- [`--phrase-stdin`](#mnemonic-xpub-search-passphrase-of-xpub-phrase-stdin) — read the master phrase from stdin
- [`--ms1`](#mnemonic-xpub-search-passphrase-of-xpub-ms1) — `ms1` card carrying BIP-39 entropy (inline)
- [`--ms1-stdin`](#mnemonic-xpub-search-passphrase-of-xpub-ms1-stdin) — read the `ms1` card from stdin
- [`--passphrase`](#mnemonic-xpub-search-passphrase-of-xpub-passphrase) — the candidate passphrase (inline)
- [`--passphrase-stdin`](#mnemonic-xpub-search-passphrase-of-xpub-passphrase-stdin) — read the passphrase from stdin
- [`--passphrase-candidates-file`](#mnemonic-xpub-search-passphrase-of-xpub-passphrase-candidates-file) — scan a file of candidate passphrases (one per line)
- [`--target-xpub`](#mnemonic-xpub-search-passphrase-of-xpub-target-xpub) — target xpub or `mk1` card carrying an xpub
- [`--language`](#mnemonic-xpub-search-passphrase-of-xpub-language) — BIP-39 wordlist (default `english`)
- [`--network`](#mnemonic-xpub-search-passphrase-of-xpub-network) — network selector (default `mainnet`)
- [`--min-account`](#mnemonic-xpub-search-passphrase-of-xpub-min-account) — lower bound of account iteration (default `0`)
- [`--number-of-accounts`](#mnemonic-xpub-search-passphrase-of-xpub-number-of-accounts) — window size (default `20`)
- [`--max-account`](#mnemonic-xpub-search-passphrase-of-xpub-max-account) — optional upper bound
- [`--add-path`](#mnemonic-xpub-search-passphrase-of-xpub-add-path) — additional path template (repeatable)
- [`--json`](#mnemonic-xpub-search-passphrase-of-xpub-json) — emit JSON envelope instead of text

## `--phrase` {#mnemonic-xpub-search-passphrase-of-xpub-phrase}

The master BIP-39 phrase, supplied inline. Part of the seed-intake
mutex — exactly one of `--phrase` / `--phrase-stdin` / `--ms1` /
`--ms1-stdin` (or a positional `ms1`) is required. Inline use emits
the argv-leakage advisory; prefer `--phrase-stdin`. The GUI renders
this as a masked `SecretLineEdit`.

## `--phrase-stdin` {#mnemonic-xpub-search-passphrase-of-xpub-phrase-stdin}

Boolean flag. Read the master BIP-39 phrase from stdin. XOR with the
other seed-intake rows; single-stdin-per-invocation.

## `--ms1` {#mnemonic-xpub-search-passphrase-of-xpub-ms1}

An `ms1` card carrying BIP-39 entropy, supplied inline. Inline use
emits the argv-leakage advisory; prefer `--ms1-stdin`. Auto-fire BCH
repair applies only to the `--ms1` decode-failure path. The GUI
renders this as a masked `SecretLineEdit`.

## `--ms1-stdin` {#mnemonic-xpub-search-passphrase-of-xpub-ms1-stdin}

Boolean flag. Read the `ms1` card from stdin (single chunk). XOR with
the other seed-intake rows; single-stdin-per-invocation.

## `--passphrase` {#mnemonic-xpub-search-passphrase-of-xpub-passphrase}

The candidate BIP-39 passphrase to verify, supplied inline. One of
the **mandatory** passphrase-source group — exactly one of
`--passphrase` / `--passphrase-stdin` / `--passphrase-candidates-file`
must be supplied (omitting all three is a clap arg-parse error,
exit 64). Inline use emits the argv-leakage advisory; prefer
`--passphrase-stdin`. The GUI renders this as a masked
`SecretLineEdit`.

## `--passphrase-stdin` {#mnemonic-xpub-search-passphrase-of-xpub-passphrase-stdin}

Boolean flag. Read the candidate passphrase from stdin
(NULL-byte-preserving; a single trailing newline is stripped). One of
the mandatory passphrase-source group. Single-stdin-per-invocation.

## `--passphrase-candidates-file` {#mnemonic-xpub-search-passphrase-of-xpub-passphrase-candidates-file}

Path to a text file of candidate passphrases — **one per line**, no
argv exposure (v0.46.0). The command derives the master seed per
candidate and stops at the first that produces `--target-xpub`,
reporting the matching **file line number** to stdout (the matching
passphrase appears only under `--json`). Blank lines are skipped;
each non-blank line is a literal candidate (only the trailing
newline/CR is stripped). No match ⇒ exit 4 with the count tried. This
is bounded **verification of a list you supply**, not keyspace
generation. The candidate file is sensitive (holds secret candidates)
but is classified as a path (non-secret) flag. The GUI renders this
as a Path widget.

## `--target-xpub` {#mnemonic-xpub-search-passphrase-of-xpub-target-xpub}

The target xpub (any SLIP-0132 prefix:
`xpub`/`tpub`/`ypub`/`Ypub`/`zpub`/`Zpub`/`upub`/`Upub`/`vpub`/`Vpub`)
OR an `mk1...` bech32 card carrying an xpub. Required. The GUI renders
this as a Text widget.

## `--language` {#mnemonic-xpub-search-passphrase-of-xpub-language}

The BIP-39 wordlist used to interpret `--phrase`. Optional; defaults
to `english`. Same 10 allowed values as
[`mnemonic bundle --language`](#mnemonic-bundle-language). The GUI
renders this as a Dropdown widget.

### Outline {#mnemonic-xpub-search-passphrase-of-xpub-language-outline}

- [`english`](#mnemonic-xpub-search-passphrase-of-xpub-language-english)
- [`simplifiedchinese`](#mnemonic-xpub-search-passphrase-of-xpub-language-simplifiedchinese)
- [`traditionalchinese`](#mnemonic-xpub-search-passphrase-of-xpub-language-traditionalchinese)
- [`czech`](#mnemonic-xpub-search-passphrase-of-xpub-language-czech)
- [`french`](#mnemonic-xpub-search-passphrase-of-xpub-language-french)
- [`italian`](#mnemonic-xpub-search-passphrase-of-xpub-language-italian)
- [`japanese`](#mnemonic-xpub-search-passphrase-of-xpub-language-japanese)
- [`korean`](#mnemonic-xpub-search-passphrase-of-xpub-language-korean)
- [`portuguese`](#mnemonic-xpub-search-passphrase-of-xpub-language-portuguese)
- [`spanish`](#mnemonic-xpub-search-passphrase-of-xpub-language-spanish)

### `english` {#mnemonic-xpub-search-passphrase-of-xpub-language-english}

The BIP-39 English wordlist (2048 entries). Default.

### `simplifiedchinese` {#mnemonic-xpub-search-passphrase-of-xpub-language-simplifiedchinese}

BIP-39 Simplified Chinese wordlist. Cross-tab divergence with
`ms encode --language chinese-simplified` is documented at
[`ms encode --language`](#ms-encode-language).

### `traditionalchinese` {#mnemonic-xpub-search-passphrase-of-xpub-language-traditionalchinese}

BIP-39 Traditional Chinese wordlist.

### `czech` {#mnemonic-xpub-search-passphrase-of-xpub-language-czech}

BIP-39 Czech wordlist.

### `french` {#mnemonic-xpub-search-passphrase-of-xpub-language-french}

BIP-39 French wordlist.

### `italian` {#mnemonic-xpub-search-passphrase-of-xpub-language-italian}

BIP-39 Italian wordlist.

### `japanese` {#mnemonic-xpub-search-passphrase-of-xpub-language-japanese}

BIP-39 Japanese wordlist.

### `korean` {#mnemonic-xpub-search-passphrase-of-xpub-language-korean}

BIP-39 Korean wordlist.

### `portuguese` {#mnemonic-xpub-search-passphrase-of-xpub-language-portuguese}

BIP-39 Portuguese wordlist.

### `spanish` {#mnemonic-xpub-search-passphrase-of-xpub-language-spanish}

BIP-39 Spanish wordlist.

## `--network` {#mnemonic-xpub-search-passphrase-of-xpub-network}

The Bitcoin network selector for the searched derivations. Optional;
defaults to `mainnet`. The GUI renders this as a Dropdown widget.

### Outline {#mnemonic-xpub-search-passphrase-of-xpub-network-outline}

- [`mainnet`](#mnemonic-xpub-search-passphrase-of-xpub-network-mainnet)
- [`testnet`](#mnemonic-xpub-search-passphrase-of-xpub-network-testnet)
- [`signet`](#mnemonic-xpub-search-passphrase-of-xpub-network-signet)
- [`regtest`](#mnemonic-xpub-search-passphrase-of-xpub-network-regtest)

### `mainnet` {#mnemonic-xpub-search-passphrase-of-xpub-network-mainnet}

Production Bitcoin mainnet. BIP-44 coin-type 0.

### `testnet` {#mnemonic-xpub-search-passphrase-of-xpub-network-testnet}

The legacy public test network. Coin-type 1. Funds are valueless.

### `signet` {#mnemonic-xpub-search-passphrase-of-xpub-network-signet}

The signature-secured test network. Coin-type 1. Funds are valueless.

### `regtest` {#mnemonic-xpub-search-passphrase-of-xpub-network-regtest}

A locally-controlled regression-test network. Coin-type 1.

## `--min-account` {#mnemonic-xpub-search-passphrase-of-xpub-min-account}

The lower bound of account-index iteration, inclusive. Default `0`.
The GUI renders this as a Number widget.

## `--number-of-accounts` {#mnemonic-xpub-search-passphrase-of-xpub-number-of-accounts}

The window size of the account-index iteration, starting at
`--min-account`. Default `20`. The GUI renders this as a Number
widget.

## `--max-account` {#mnemonic-xpub-search-passphrase-of-xpub-max-account}

Optional upper bound on the account index. Effective end:
`max(min_account + number_of_accounts, max_account + 1)`. The GUI
renders this as a Number widget.

## `--add-path` {#mnemonic-xpub-search-passphrase-of-xpub-add-path}

An additional derivation-path template, repeatable. The literal token
`account'` (or `account`) is substituted with each iterated account
index; templates without an `account` token are searched once. The
GUI renders this as a repeating Text input.

## `--json` {#mnemonic-xpub-search-passphrase-of-xpub-json}

Boolean flag. Emit a versioned JSON envelope (schema `v1`,
`mode: "passphrase-of-xpub"`) instead of the text report. The GUI
renders this as a checkbox.

## Stderr advisory (always emitted)

Every invocation emits this advisory on stderr **before** the search
starts (it does not gate on match / no-match):

```{.text include="4e-xpub-search-passphrase-advisory.err" lines="1-1"}
note: passphrase verification searches the standard BIP-44/49/84/86 + BIP-48 templates × account range; if the wallet uses a non-standard path, supply --add-path or use `xpub-search path-of-xpub` to find the path first.
```

A "no match" result does **not** prove the passphrase is wrong — only
that no standard path under the (seed, passphrase) pair produces the
target. Extend the candidate set via `--add-path`, or solve the
path-lookup separately via [`path-of-xpub`](#mnemonic-xpub-search-path-of-xpub).

## Worked example — verify a passphrase produces the target xpub

1. Switch to the **mnemonic** tab; pick **xpub-search:
   passphrase-of-xpub** in the subcommand selector.
2. Set the seed-intake to **`--phrase-stdin`**; set the passphrase
   source to **`--passphrase-stdin`** — wait, only one stdin per
   invocation: use `--phrase-stdin` for the seed and type the
   passphrase inline into `--passphrase` (it renders masked, and the
   modal redacts it to `••••`).
3. Paste the target xpub (or `mk1` card) into `--target-xpub`.
4. Click **Run** and confirm the modal.

Stdout (text form, match) — captured against the pinned `mnemonic`
binary using the canonical all-`abandon` vector with an empty
passphrase:

```{.text include="4e-xpub-search-passphrase-advisory.out" lines="1-3"}
match: m/84'/0'/0'  (template=bip84, account=0)
target-xpub: xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V
searched: 140 candidate paths
```

If the `--target-xpub` was supplied under a non-`xpub` SLIP-0132
prefix (e.g. `zpub`), the `target-xpub:` line additionally notes the
normalization (`… (normalized from zpub; variant=zpub)`).

With `--passphrase-candidates-file`, the match line instead reports
the matching file line (the passphrase itself only under `--json`):

```text
match: candidate on line 2 derives the target xpub at m/84'/0'/0' (template=bip84, account=0)
```

## Exit codes

| Code | Meaning |
|---|---|
| 0 | Match found |
| 1 | Bad input (BIP-39 parse failure, xpub parse failure, seed-intake error) |
| 4 | No match in searched set |
| 5 | Auto-fire BCH short-circuit on `--ms1` decode failure (TTY-gated) |
| 64 | Clap arg-parse error (including no passphrase-source supplied) |
