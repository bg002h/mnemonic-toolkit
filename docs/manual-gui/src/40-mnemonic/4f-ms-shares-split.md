# `mnemonic ms-shares-split` {#mnemonic-ms-shares-split}

Split an `ms1` secret into N codex32 (BIP-93) K-of-N shares using
codex32's native `threshold(k)`+`index` Shamir mechanism over `GF(32)`.
Any K-of-N subset of shares reconstructs the secret; fewer than K shares
reveal nothing. Each share is itself an `ms1`-format codex32 string in
the same human-typeable alphabet as a single-string `ms1` card. The GUI
exposes the toolkit's `ms-shares split` sub-subcommand as a flat
**ms-shares Split** form on the `mnemonic` tab. Recover with
[`mnemonic ms-shares-combine`](#mnemonic-ms-shares-combine).

The `mnem`-vs-`entr` payload kind survives the split: a non-English
`--language` phrase splits as a `mnem` share-set (the BIP-39 wordlist
language travels on the wire), while an English phrase or raw entropy
splits as a plain `entr` share-set.

:::danger
The worked example uses the canonical zero-entropy 24-word master
(`abandon × 23 + art`). **Never engrave or fund** a wallet derived from
it. The whole N-share SET is secret-equivalent to the master seed, and
the `--from` input is master key material. The run-confirm modal redacts
the secret-bearing argv token as a fixed `••••` sentinel (see [§14
Defense 2](#secret-handling)). Engrave each share on its own backup
medium; storing K shares together re-creates a single point of failure.
:::

> **GUI form:** see [GUI Forms › mnemonic › ms-shares-split](#gui-form-mnemonic-ms-shares-split).

## Outline {#mnemonic-ms-shares-split-outline}

- [`--group-size`](#mnemonic-ms-shares-split-group-size) — display grouping width for the emitted shares (cosmetic; default 5)
- [`--separator`](#mnemonic-ms-shares-split-separator) — display-grouping separator keyword (cosmetic)
- [`--from`](#mnemonic-ms-shares-split-from) — the secret to split (`phrase=` or `entropy=`; required)
- [`--threshold`](#mnemonic-ms-shares-split-threshold) — threshold K, minimum shares to recombine (2..=9; required)
- [`--shares`](#mnemonic-ms-shares-split-shares) — total shares N to emit (K ≤ N ≤ 31; required)
- [`--language`](#mnemonic-ms-shares-split-language) — BIP-39 wordlist of the input phrase (default `english`)
- [`--json`](#mnemonic-ms-shares-split-json) — emit a JSON object (`{"shares": […]}`) instead of one-share-per-line text

## `--group-size` {#mnemonic-ms-shares-split-group-size}

Number widget (0..65535; default 5). Display grouping: break each
emitted share string into groups of N characters for readability. This
is cosmetic only — share intake strips separators, so any grouping
re-ingests cleanly. `0` emits an unbroken single line. The `--json`
forensic strings always stay unbroken regardless of this flag.

## `--separator` {#mnemonic-ms-shares-split-separator}

Dropdown. The display-grouping separator keyword inserted between
`--group-size` groups (default `space`). Three allowed values. Cosmetic
and non-load-bearing — the separator is stripped on re-ingest. The `?`
help-icon deep-links here.

### Outline {#mnemonic-ms-shares-split-separator-outline}

- [`space`](#mnemonic-ms-shares-split-separator-space)
- [`hyphen`](#mnemonic-ms-shares-split-separator-hyphen)
- [`comma`](#mnemonic-ms-shares-split-separator-comma)

### `space` {#mnemonic-ms-shares-split-separator-space}

ASCII space between groups (default). The most legible for hand
transcription onto a backup medium.

### `hyphen` {#mnemonic-ms-shares-split-separator-hyphen}

ASCII hyphen (`-`) between groups. Useful when the share is embedded in
a context where spaces are collapsed or trimmed.

### `comma` {#mnemonic-ms-shares-split-separator-comma}

ASCII comma (`,`) between groups.

## `--from` {#mnemonic-ms-shares-split-from}

The secret to split. Required. The GUI renders this as a
NodeValueComposite field: a node-type selector (`phrase` or `entropy`)
plus a value editor. Grammar `phrase=<value-or->` or
`entropy=<hex-or->`; `=-` routes the value through stdin. The value
editor renders as a masked `SecretLineEdit`; any non-empty inline value
triggers the run-confirm modal.

### Outline {#mnemonic-ms-shares-split-from-outline}

- [`phrase`](#mnemonic-ms-shares-split-from-phrase)
- [`entropy`](#mnemonic-ms-shares-split-from-entropy)

### `phrase` {#mnemonic-ms-shares-split-from-phrase}

A BIP-39 mnemonic (12 / 15 / 18 / 21 / 24 words = 16 / 20 / 24 / 28 /
32 bytes of entropy). Combine with [`--language`](#mnemonic-ms-shares-split-language)
for a non-English wordlist; a non-English language produces a `mnem`
share-set that carries the wordlist on the wire.

### `entropy` {#mnemonic-ms-shares-split-from-entropy}

Raw BIP-39 entropy as hex (16 / 20 / 24 / 28 / 32 bytes). Splits as a
plain `entr` share-set; `--language` is ignored for `entropy=` input.

## `--threshold` {#mnemonic-ms-shares-split-threshold}

Number widget. The threshold K — the minimum number of shares that
recombine. Required. Allowed range 2..=9 (the codex32 threshold field
is a single ASCII digit; `0` is the unshared single-string sentinel and
`1` is invalid). K outside 2..=9 is a usage error (exit 64).

## `--shares` {#mnemonic-ms-shares-split-shares}

Number widget. The total number of shares N to emit. Required. Allowed
range K ≤ N ≤ 31 (there are exactly 31 valid non-`s` codex32 share
indices). N outside K..=31 is a usage error (exit 64).

## `--language` {#mnemonic-ms-shares-split-language}

Dropdown. The BIP-39 wordlist of the input phrase (default `english`).
Ignored for `entropy=` input. A non-English language produces a `mnem`
share-set so the wordlist survives the split. Same 10 values as the
other BIP-39 surfaces; see [`mnemonic bundle --language`](#mnemonic-bundle-language)
for the per-wordlist detail. The `?` help-icon deep-links here.

### Outline {#mnemonic-ms-shares-split-language-outline}

- [`english`](#mnemonic-ms-shares-split-language-english)
- [`simplifiedchinese`](#mnemonic-ms-shares-split-language-simplifiedchinese)
- [`traditionalchinese`](#mnemonic-ms-shares-split-language-traditionalchinese)
- [`czech`](#mnemonic-ms-shares-split-language-czech)
- [`french`](#mnemonic-ms-shares-split-language-french)
- [`italian`](#mnemonic-ms-shares-split-language-italian)
- [`japanese`](#mnemonic-ms-shares-split-language-japanese)
- [`korean`](#mnemonic-ms-shares-split-language-korean)
- [`portuguese`](#mnemonic-ms-shares-split-language-portuguese)
- [`spanish`](#mnemonic-ms-shares-split-language-spanish)

### `english` {#mnemonic-ms-shares-split-language-english}

The BIP-39 English wordlist (2048 entries). Default; splits as a plain
`entr` share-set.

### `simplifiedchinese` {#mnemonic-ms-shares-split-language-simplifiedchinese}

BIP-39 Simplified Chinese wordlist. Splits as a `mnem` share-set.

### `traditionalchinese` {#mnemonic-ms-shares-split-language-traditionalchinese}

BIP-39 Traditional Chinese wordlist. Splits as a `mnem` share-set.

### `czech` {#mnemonic-ms-shares-split-language-czech}

BIP-39 Czech wordlist. Splits as a `mnem` share-set.

### `french` {#mnemonic-ms-shares-split-language-french}

BIP-39 French wordlist. Splits as a `mnem` share-set.

### `italian` {#mnemonic-ms-shares-split-language-italian}

BIP-39 Italian wordlist. Splits as a `mnem` share-set.

### `japanese` {#mnemonic-ms-shares-split-language-japanese}

BIP-39 Japanese wordlist. Splits as a `mnem` share-set.

### `korean` {#mnemonic-ms-shares-split-language-korean}

BIP-39 Korean wordlist. Splits as a `mnem` share-set.

### `portuguese` {#mnemonic-ms-shares-split-language-portuguese}

BIP-39 Portuguese wordlist. Splits as a `mnem` share-set.

### `spanish` {#mnemonic-ms-shares-split-language-spanish}

BIP-39 Spanish wordlist. Splits as a `mnem` share-set.

## `--json` {#mnemonic-ms-shares-split-json}

Boolean flag. When set, emits a JSON object on stdout
(`{"shares": [...]}`) instead of the one-share-per-line text form. The
share strings in the JSON are always unbroken (the `--group-size` /
`--separator` cosmetic grouping applies to the text form only).

## Worked example — 2-of-3 split from the canonical master

1. Switch to the **mnemonic** tab; pick **ms-shares Split** in the
   subcommand selector.
2. In the `--from` field, set the node selector to `phrase` and paste
   the canonical 24-word master into the masked value editor:

   ```text
   abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art
   ```

3. Set `--threshold` to `2` and `--shares` to `3`. Leave `--language`,
   `--group-size`, and `--separator` at their defaults.
4. The `Preview:` line resembles:

   ```text
   mnemonic ms-shares split --from "phrase=abandon … art" --threshold 2 --shares 3
   ```

5. Click **Run**; redact-confirm in the modal.

The output panel renders three `ms1`-format codex32 shares on stdout,
one per line, each carrying the threshold digit `2`, a shared random
identifier, and a distinct non-`s` index. Because `split` is
CSPRNG-driven the exact share strings differ on every run. Recover with
any 2 via [`mnemonic ms-shares-combine`](#mnemonic-ms-shares-combine).

A `warning: stdout carries private key material (can spend) …` advisory
fires on stderr — the whole share SET is secret-equivalent.

\index{mnemonic ms-shares-split}
