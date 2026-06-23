# `ms split` {#ms-split}

\index{ms split}Split a secret (a BIP-39 mnemonic or hex entropy)
into **N BIP-93 codex32 K-of-N shares** (ms-cli v0.7.0+). Any K of
the N shares recombine to the original secret via
[`ms combine`](#ms-combine), using codex32's native
`threshold(k)` + `index` Shamir mechanism over `GF(32)`.
Bounds: **2 ≤ K ≤ N ≤ 31**.

The whole N-share **set is secret-equivalent** — recovering any K
of them reconstructs the master secret — so the GUI treats each
emitted share as private key material and the run-confirm modal
fires on the secret-bearing input. A non-English
[`--phrase`](#ms-split-phrase) splits as a `mnem` share-set so the
BIP-39 wordlist language survives the split; an English phrase or
[`--hex`](#ms-split-hex) entropy splits as a plain `entr`
share-set. The toolkit front-end is `mnemonic ms-shares split`.

The encoder consumes exactly one of two mutually exclusive seed
inputs — [`--phrase`](#ms-split-phrase) XOR
[`--hex`](#ms-split-hex) — and the two required count flags
[`--threshold`](#ms-split-threshold) and
[`--shares`](#ms-split-shares).

## Outline {#ms-split-outline}

- [`--group-size`](#ms-split-group-size) — display-grouping chunk width for each emitted share (default `5`; `0` = unbroken)
- [`--separator`](#ms-split-separator) — display-grouping separator keyword (`space`|`hyphen`|`comma`; default `space`)
- [`--phrase`](#ms-split-phrase) — BIP-39 mnemonic to split (XOR with `--hex`, secret-bearing)
- [`--hex`](#ms-split-hex) — raw hex entropy to split (XOR with `--phrase`, secret-bearing)
- [`--threshold`](#ms-split-threshold) — threshold K, minimum shares to recombine (required; `2..=9`)
- [`--shares`](#ms-split-shares) — total shares N to produce (required; `K..=31`)
- [`--language`](#ms-split-language) — BIP-39 wordlist for `--phrase` (default `english`; ignored under `--hex`)
- [`--json`](#ms-split-json) — emit a single JSON object on stdout instead of multi-line text

## `--group-size` {#ms-split-group-size}

Display-grouping chunk width applied to **each** emitted share.
Number widget; range `0..=65535`, default `5`. The splitter breaks
each share string into groups of N characters separated by the
[`--separator`](#ms-split-separator) keyword; `0` emits each share
as a single unbroken line.

**Cosmetic — non-load-bearing.** Share intake strips separators,
so a grouped share and an unbroken share re-ingest identically
under [`ms combine`](#ms-combine). `--json` output always carries
unbroken shares regardless of this flag.

## `--separator` {#ms-split-separator}

Display-grouping separator keyword used between the
[`--group-size`](#ms-split-group-size) chunks of each share.
Dropdown widget; 3 values, default `space`. **Cosmetic —
non-load-bearing** (intake strips it).

### Outline {#ms-split-separator-outline}

- [`space`](#ms-split-separator-space)
- [`hyphen`](#ms-split-separator-hyphen)
- [`comma`](#ms-split-separator-comma)

### `space` {#ms-split-separator-space}

ASCII-space (`U+0020`) between chunks — the default.

### `hyphen` {#ms-split-separator-hyphen}

ASCII hyphen-minus (`-`) between chunks.

### `comma` {#ms-split-separator-comma}

ASCII comma (`,`) between chunks.

## `--phrase` {#ms-split-phrase}

The BIP-39 mnemonic phrase to split. **Secret-bearing** —
schema-`secret: true`. Mutually exclusive with
[`--hex`](#ms-split-hex). The GUI renders this as a
`SecretLineEdit` widget (masked text field); a non-empty value
triggers the run-confirm modal at click-Run time. A literal `-`
value reads the phrase from stdin.

A non-English phrase (selected via [`--language`](#ms-split-language))
produces a `mnem` share-set carrying the wordlist tag, so
[`ms combine --to phrase`](#ms-combine-to-phrase) recovers the
phrase in the original language without the recombiner needing to
restate it.

## `--hex` {#ms-split-hex}

Raw entropy as a hex string. **Secret-bearing** —
schema-`secret: true`. Mutually exclusive with
[`--phrase`](#ms-split-phrase). Accepts 32 / 40 / 48 / 56 / 64 hex
characters (= 16 / 20 / 24 / 28 / 32 bytes). The GUI renders this
as a `SecretLineEdit` widget. When `--hex` is the chosen mode,
[`--language`](#ms-split-language) is ignored and the split is a
plain `entr` share-set.

## `--threshold` {#ms-split-threshold}

Threshold **K** — the minimum number of shares required to
recombine the secret. **Required.** Number widget; range
`2..=9`. Any K of the N shares are sufficient to recover the
secret via [`ms combine`](#ms-combine); fewer than K reveal
nothing.

## `--shares` {#ms-split-shares}

Total number of shares **N** to produce. **Required.** Number
widget; range `2..=31`, and must satisfy `K ≤ N` (the
[`--threshold`](#ms-split-threshold) bound). Each share is
engraved on its own backup medium.

## `--language` {#ms-split-language}

BIP-39 wordlist used to interpret [`--phrase`](#ms-split-phrase).
Optional; defaults to `english`. Dropdown widget; 10 valid values,
hyphenated Chinese tokens (see [§61 cross-tab
divergence](#ms-per-tab-reference)). A non-English wordlist
produces a `mnem` share-set (the language travels on the wire);
ignored when [`--hex`](#ms-split-hex) is the chosen mode.

### Outline {#ms-split-language-outline}

- [`english`](#ms-split-language-english)
- [`japanese`](#ms-split-language-japanese)
- [`korean`](#ms-split-language-korean)
- [`spanish`](#ms-split-language-spanish)
- [`chinese-simplified`](#ms-split-language-chinese-simplified)
- [`chinese-traditional`](#ms-split-language-chinese-traditional)
- [`french`](#ms-split-language-french)
- [`italian`](#ms-split-language-italian)
- [`czech`](#ms-split-language-czech)
- [`portuguese`](#ms-split-language-portuguese)

### `english` {#ms-split-language-english}

See [`ms encode --language english`](#ms-encode-language-english).
Default; produces an `entr` share-set.

### `japanese` {#ms-split-language-japanese}

See [`ms encode --language japanese`](#ms-encode-language-japanese).

### `korean` {#ms-split-language-korean}

See [`ms encode --language korean`](#ms-encode-language-korean).

### `spanish` {#ms-split-language-spanish}

See [`ms encode --language spanish`](#ms-encode-language-spanish).

### `chinese-simplified` {#ms-split-language-chinese-simplified}

See [`ms encode --language
chinese-simplified`](#ms-encode-language-chinese-simplified).

### `chinese-traditional` {#ms-split-language-chinese-traditional}

See [`ms encode --language
chinese-traditional`](#ms-encode-language-chinese-traditional).

### `french` {#ms-split-language-french}

See [`ms encode --language french`](#ms-encode-language-french).

### `italian` {#ms-split-language-italian}

See [`ms encode --language italian`](#ms-encode-language-italian).

### `czech` {#ms-split-language-czech}

See [`ms encode --language czech`](#ms-encode-language-czech).

### `portuguese` {#ms-split-language-portuguese}

See [`ms encode --language
portuguese`](#ms-encode-language-portuguese).

## `--json` {#ms-split-json}

Boolean. Emit a single JSON object on stdout
(`{ shares, k, n, id, kind, language? }`) instead of the multi-line
text form (one share per line). Default off. The `language` field
is present only for a `mnem` share-set. JSON shares are always
unbroken regardless of [`--group-size`](#ms-split-group-size).

## Worked example — split entropy 2-of-3

:::danger
Examples use the canonical all-`abandon` 16-byte zero-entropy test
vector — a **public** seed swept since 2017. Never engrave or fund
any wallet derived from it.
:::

1. **ms** tab; pick **Split (BIP-93 codex32 K-of-N share
   splitter)**.
2. `--hex` (masked): enter `00000000000000000000000000000000`.
   The conditional engine disables `--phrase` and ignores
   `--language`.
3. `--threshold`: `2`. `--shares`: `3`.
4. Click **Run**. The run-confirm modal fires (secret-bearing
   input); confirm to proceed.

The output panel emits 3 ms1-format codex32 shares (one per line),
each carrying threshold digit `2`, a shared random 4-character
identifier, and a distinct non-`s` index. Recombine any 2 with
[`ms combine`](#ms-combine).

## Refusals

| Trigger | Refusal |
|---|---|
| Neither `--phrase` nor `--hex` supplied | clap-group refusal: required-input not provided |
| Both `--phrase` and `--hex` supplied | clap-group refusal: mutually-exclusive |
| `--threshold` or `--shares` omitted | clap refusal: required argument not provided |
| `K < 2` or `K > 9` | value-parser refusal (range `2..=9`) |
| `N < K` or `N > 31` | exit 1 with a `K <= N <= 31` bounds refusal |
| `--hex` not 16/20/24/28/32 bytes | ms-codec entropy-length refusal |
| `--phrase` with invalid BIP-39 checksum | exit 1 with `error: <bip39 error>` |
