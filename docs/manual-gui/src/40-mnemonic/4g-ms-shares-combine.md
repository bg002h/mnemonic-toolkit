# `mnemonic ms-shares-combine` {#mnemonic-ms-shares-combine}

Recombine ≥K codex32 (BIP-93) shares back into the recovered secret —
the inverse of [`mnemonic ms-shares-split`](#mnemonic-ms-shares-split).
Any K-of-N subset of the split's shares reconstructs; the codex32
threshold/index metadata travels in each share, so the form needs only
the shares themselves plus an output-shape choice. The GUI exposes the
toolkit's `ms-shares combine` sub-subcommand as a flat **ms-shares
Combine** form on the `mnemonic` tab. A recovered single-string `ms1`
(`--to ms1`) composes with the rest of the toolkit — feed it to
[`mnemonic bundle`](#mnemonic-bundle) `--slot @0.ms1=<recovered-ms1>`.

:::danger
The worked example recombines the shares of the canonical zero-entropy
master. **Never engrave or fund** any recovered wallet from a
demonstration share set. Each `--share` input, and the recovered secret
on stdout, is master key material. The run-confirm modal redacts the
secret-bearing argv tokens as a fixed `••••` sentinel (see [§14 Defense
2](#secret-handling)). Combining shares re-materialises the single point
of failure — do it on an airgapped machine and re-disperse afterward.
:::

> **GUI form:** see [GUI Forms › mnemonic › ms-shares-combine](#gui-form-mnemonic-ms-shares-combine).

## Outline {#mnemonic-ms-shares-combine-outline}

- [`--group-size`](#mnemonic-ms-shares-combine-group-size) — display grouping width for a `--to ms1` output (cosmetic; default 5)
- [`--separator`](#mnemonic-ms-shares-combine-separator) — display-grouping separator keyword (cosmetic)
- [`--share`](#mnemonic-ms-shares-combine-share) — a codex32 share string; repeating, supply at least K (required)
- [`--to`](#mnemonic-ms-shares-combine-to) — output shape (`phrase` default, `entropy`, or `ms1`)
- [`--language`](#mnemonic-ms-shares-combine-language) — BIP-39 wordlist for `--to phrase` on a plain `entr` payload (default `english`)
- [`--json`](#mnemonic-ms-shares-combine-json) — emit a JSON object instead of the plain secret line

## `--group-size` {#mnemonic-ms-shares-combine-group-size}

Number widget (0..65535; default 5). Display grouping for a `--to ms1`
recovery: break the recovered `ms1` string into groups of N characters.
Cosmetic only — intake strips separators. `0` emits an unbroken single
line. No effect on `--to phrase` / `--to entropy` output.

## `--separator` {#mnemonic-ms-shares-combine-separator}

Dropdown. The display-grouping separator keyword for a `--to ms1`
recovery (default `space`). Three allowed values. Cosmetic and
non-load-bearing. The `?` help-icon deep-links here.

### Outline {#mnemonic-ms-shares-combine-separator-outline}

- [`space`](#mnemonic-ms-shares-combine-separator-space)
- [`hyphen`](#mnemonic-ms-shares-combine-separator-hyphen)
- [`comma`](#mnemonic-ms-shares-combine-separator-comma)

### `space` {#mnemonic-ms-shares-combine-separator-space}

ASCII space between groups (default).

### `hyphen` {#mnemonic-ms-shares-combine-separator-hyphen}

ASCII hyphen (`-`) between groups.

### `comma` {#mnemonic-ms-shares-combine-separator-comma}

ASCII comma (`,`) between groups.

## `--share` {#mnemonic-ms-shares-combine-share}

The repeating share input. Required; supply at least K shares. The GUI
renders this as a repeating row of masked `SecretLineEdit` fields with
a **+ Add share** button — one share string per row. At most ONE share
may be `-` (read that share from stdin). Each inline value emits a
per-occurrence argv-leakage advisory; the value editors are masked
because the share SET is secret-equivalent.

`combine` rejects: fewer than K shares (a codex32 "threshold not passed"
refusal), a repeated share index, a mixed identifier / threshold /
length set, or the secret-carrying share at index `s` (which would
short-circuit interpolation and bypass validation).

## `--to` {#mnemonic-ms-shares-combine-to}

Dropdown. The output shape (default `phrase`). Three allowed values.
The `?` help-icon deep-links here.

### Outline {#mnemonic-ms-shares-combine-to-outline}

- [`phrase`](#mnemonic-ms-shares-combine-to-phrase)
- [`entropy`](#mnemonic-ms-shares-combine-to-entropy)
- [`ms1`](#mnemonic-ms-shares-combine-to-ms1)

### `phrase` {#mnemonic-ms-shares-combine-to-phrase}

Emit the recovered secret as a BIP-39 mnemonic (default). For a `mnem`
share-set the wordlist language is recovered from the wire; for a plain
`entr` share-set the language comes from [`--language`](#mnemonic-ms-shares-combine-language).

### `entropy` {#mnemonic-ms-shares-combine-to-entropy}

Emit the recovered secret as raw entropy hex. `--language` is ignored.

### `ms1` {#mnemonic-ms-shares-combine-to-ms1}

Re-encode the recovered secret as a single-string `ms1` card. This is
the form to feed back into [`mnemonic bundle`](#mnemonic-bundle)
`--slot @0.ms1=…`. The `--group-size` / `--separator` cosmetic grouping
applies to this output shape.

## `--language` {#mnemonic-ms-shares-combine-language}

Dropdown. The BIP-39 wordlist for `--to phrase` when the recovered
secret is a plain `entr` payload with no wire language (default
`english`). Ignored for `mnem` payloads (which carry their own language)
and for `--to entropy` / `--to ms1`. Same 10 values as the other BIP-39
surfaces; see [`mnemonic bundle --language`](#mnemonic-bundle-language)
for the per-wordlist detail. The `?` help-icon deep-links here.

### Outline {#mnemonic-ms-shares-combine-language-outline}

- [`english`](#mnemonic-ms-shares-combine-language-english)
- [`simplifiedchinese`](#mnemonic-ms-shares-combine-language-simplifiedchinese)
- [`traditionalchinese`](#mnemonic-ms-shares-combine-language-traditionalchinese)
- [`czech`](#mnemonic-ms-shares-combine-language-czech)
- [`french`](#mnemonic-ms-shares-combine-language-french)
- [`italian`](#mnemonic-ms-shares-combine-language-italian)
- [`japanese`](#mnemonic-ms-shares-combine-language-japanese)
- [`korean`](#mnemonic-ms-shares-combine-language-korean)
- [`portuguese`](#mnemonic-ms-shares-combine-language-portuguese)
- [`spanish`](#mnemonic-ms-shares-combine-language-spanish)

### `english` {#mnemonic-ms-shares-combine-language-english}

The BIP-39 English wordlist (2048 entries). Default for a plain `entr`
recovery under `--to phrase`.

### `simplifiedchinese` {#mnemonic-ms-shares-combine-language-simplifiedchinese}

BIP-39 Simplified Chinese wordlist.

### `traditionalchinese` {#mnemonic-ms-shares-combine-language-traditionalchinese}

BIP-39 Traditional Chinese wordlist.

### `czech` {#mnemonic-ms-shares-combine-language-czech}

BIP-39 Czech wordlist.

### `french` {#mnemonic-ms-shares-combine-language-french}

BIP-39 French wordlist.

### `italian` {#mnemonic-ms-shares-combine-language-italian}

BIP-39 Italian wordlist.

### `japanese` {#mnemonic-ms-shares-combine-language-japanese}

BIP-39 Japanese wordlist.

### `korean` {#mnemonic-ms-shares-combine-language-korean}

BIP-39 Korean wordlist.

### `portuguese` {#mnemonic-ms-shares-combine-language-portuguese}

BIP-39 Portuguese wordlist.

### `spanish` {#mnemonic-ms-shares-combine-language-spanish}

BIP-39 Spanish wordlist.

## `--json` {#mnemonic-ms-shares-combine-json}

Boolean flag. When set, emits a JSON object on stdout instead of the
plain recovered-secret line.

## Worked example — recombine 2 of 3 shares

1. Switch to the **mnemonic** tab; pick **ms-shares Combine** in the
   subcommand selector.
2. Add two `--share` rows and paste two of the three shares emitted by
   the [`ms-shares-split`](#mnemonic-ms-shares-split) worked example into
   the masked value editors.
3. Leave `--to` at its default `phrase` and `--language` at `english`.
4. The `Preview:` line resembles:

   ```text
   mnemonic ms-shares combine --share "••••" --share "••••" --to phrase --language english
   ```

5. Click **Run**; redact-confirm in the modal.

The output panel renders the original `abandon × 23 + art` 24-word
phrase on stdout. Switch `--to entropy` for the 64-hex-char form, or
`--to ms1` for a single recovered `ms1` to feed back into `bundle`. A
`warning: stdout carries private key material (can spend) …` advisory
fires on stderr.

\index{mnemonic ms-shares-combine}
