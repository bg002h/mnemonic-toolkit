# `ms encode` {#ms-encode}

Encode a BIP-39 mnemonic phrase (or raw hex entropy) as an `ms1`
string suitable for engraving. The encoder consumes exactly one
of two mutually exclusive inputs — a BIP-39 phrase or hex
entropy — and emits the multi-line `ms1` (single string + blank
line + 5-char chunked form) on stdout plus a human-readable
engraving card on stderr (suppressible via
[`--no-engraving-card`](#ms-encode-no-engraving-card)).

Two-mode input: [`--phrase`](#ms-encode-phrase) XOR
[`--hex`](#ms-encode-hex). Upstream enforces this with clap's
`ArgGroup::new("input").required(true)` on those two args (per
`crates/ms-cli/src/cmd/encode.rs:26`). The
conditional-visibility engine at
`mnemonic-gui/src/form/conditional::ms_encode` mirrors the rule:
when neither is set, both are marked `Required`; when one is
set, the other is `Disabled` (and `--language` is `Hidden` when
`--hex` is the chosen mode, because the binary ignores it on the
hex path).

> **GUI form:** see [GUI Forms › ms › encode](#gui-form-ms-encode).

**Exactly-one input (not a conjunction).** The `(required)` markers on `--phrase` and `--hex` in the GUI form linked above are conditional-sourced: the form marks both required only until you fill *one*. Provide a phrase **or** raw hex (the two are mutually exclusive), not both.

## Outline {#ms-encode-outline}

- [`--group-size`](#ms-encode-group-size) — display-grouping chunk width for the emitted `ms1` (default `5`; `0` = unbroken)
- [`--separator`](#ms-encode-separator) — display-grouping separator keyword (`space`|`hyphen`|`comma`; default `space`)
- [`--phrase`](#ms-encode-phrase) — BIP-39 mnemonic input (XOR with `--hex`, secret-bearing)
- [`--hex`](#ms-encode-hex) — raw hex entropy input (XOR with `--phrase`, secret-bearing)
- [`--language`](#ms-encode-language) — BIP-39 wordlist for `--phrase` (default `english`; Hidden when `--hex` is set)
- [`--no-engraving-card`](#ms-encode-no-engraving-card) — suppress the stderr engraving card
- [`--json`](#ms-encode-json) — emit a single JSON object on stdout instead of multi-line text

## `--group-size` {#ms-encode-group-size}

Display-grouping chunk width. Number widget; range `0..=65535`,
default `5`. The encoder breaks the emitted `ms1` string into
groups of N characters separated by the
[`--separator`](#ms-encode-separator) keyword; `0` emits the
`ms1` as a single unbroken line.

**Cosmetic — non-load-bearing.** ms1 intake strips separators, so
a grouped card and an unbroken card re-ingest to the identical
secret. The default `5`-char grouping matches the canonical
chunked form printed below the single-string line. `--json` output
always carries the unbroken `ms1` regardless of this flag.

## `--separator` {#ms-encode-separator}

Display-grouping separator keyword used between the
[`--group-size`](#ms-encode-group-size) chunks. Dropdown widget;
3 values, default `space`. **Cosmetic — non-load-bearing** (intake
strips it, so any separator re-ingests).

### Outline {#ms-encode-separator-outline}

- [`space`](#ms-encode-separator-space)
- [`hyphen`](#ms-encode-separator-hyphen)
- [`comma`](#ms-encode-separator-comma)

### `space` {#ms-encode-separator-space}

ASCII-space (`U+0020`) between chunks — the default, matching the
canonical chunked-card form.

### `hyphen` {#ms-encode-separator-hyphen}

ASCII hyphen-minus (`-`) between chunks.

### `comma` {#ms-encode-separator-comma}

ASCII comma (`,`) between chunks.

## `--phrase` {#ms-encode-phrase}

The BIP-39 mnemonic phrase to encode. **Secret-bearing** —
schema-`secret: true`. Mutually exclusive with
[`--hex`](#ms-encode-hex). The GUI renders this as a
`SecretLineEdit` widget (masked text field). When non-empty, the
run-confirm modal fires at click-Run time. A literal `-` value
reads the phrase from stdin (rarely useful from the GUI; intended
for `mnemonic verify-bundle` and other piped workflows).

The conditional-visibility engine marks this flag as `Disabled`
when `--hex` has a value, and as `Required` when neither input
mode has been chosen.

## `--hex` {#ms-encode-hex}

Raw entropy as a hex string. **Secret-bearing** —
schema-`secret: true`. Mutually exclusive with
[`--phrase`](#ms-encode-phrase). Accepts lengths
32 / 40 / 48 / 56 / 64 hex characters (= 16 / 20 / 24 / 28 / 32
bytes, corresponding to 12 / 15 / 18 / 21 / 24-word BIP-39
phrases). The GUI renders this as a `SecretLineEdit` widget.

When `--hex` is the chosen mode, the conditional-visibility
engine `Hides` [`--language`](#ms-encode-language) because the
binary ignores `--language` on the hex path (the wordlist only
matters for phrase parsing, and hex skips that step).

## `--language` {#ms-encode-language}

BIP-39 wordlist used to interpret [`--phrase`](#ms-encode-phrase).
Optional; defaults to `english`. Dropdown widget; 10 valid values.
Conditionally `Hidden` when [`--hex`](#ms-encode-hex) is the
chosen mode (the binary ignores it on the hex path; the engine
hides it rather than disabling to keep the form compact).

The 10 values mirror the upstream `bip39::Language` enum but use
**hyphenated** Chinese tokens (`chinese-simplified`,
`chinese-traditional`), divergent from the fused tokens used by
the `mnemonic` tab. See [§61 cross-tab language-token
divergence](#ms-per-tab-reference) for the rationale.

### Outline {#ms-encode-language-outline}

- [`english`](#ms-encode-language-english)
- [`japanese`](#ms-encode-language-japanese)
- [`korean`](#ms-encode-language-korean)
- [`spanish`](#ms-encode-language-spanish)
- [`chinese-simplified`](#ms-encode-language-chinese-simplified)
- [`chinese-traditional`](#ms-encode-language-chinese-traditional)
- [`french`](#ms-encode-language-french)
- [`italian`](#ms-encode-language-italian)
- [`czech`](#ms-encode-language-czech)
- [`portuguese`](#ms-encode-language-portuguese)

### `english` {#ms-encode-language-english}

The BIP-39 English wordlist (2048 entries). Default when
`--language` is omitted.

### `japanese` {#ms-encode-language-japanese}

BIP-39 Japanese wordlist. ASCII-space separators accepted; the
canonical ideographic-space (U+3000) separator is normalised at
parse time.

### `korean` {#ms-encode-language-korean}

BIP-39 Korean wordlist.

### `spanish` {#ms-encode-language-spanish}

BIP-39 Spanish wordlist.

### `chinese-simplified` {#ms-encode-language-chinese-simplified}

BIP-39 Simplified Chinese wordlist (UTF-8). Cross-tab divergence
with [`mnemonic bundle --language
simplifiedchinese`](#mnemonic-bundle-language-simplifiedchinese):
the `ms` CLI uses the hyphenated token; the `mnemonic` CLI uses
the fused token. Both target the same upstream wordlist.

### `chinese-traditional` {#ms-encode-language-chinese-traditional}

BIP-39 Traditional Chinese wordlist (UTF-8). Hyphenated token;
see the cross-tab note on
[`chinese-simplified`](#ms-encode-language-chinese-simplified).

### `french` {#ms-encode-language-french}

BIP-39 French wordlist.

### `italian` {#ms-encode-language-italian}

BIP-39 Italian wordlist.

### `czech` {#ms-encode-language-czech}

BIP-39 Czech wordlist.

### `portuguese` {#ms-encode-language-portuguese}

BIP-39 Portuguese wordlist.

## `--no-engraving-card` {#ms-encode-no-engraving-card}

Boolean. Suppresses the human-readable stderr engraving card
(word count + language + passphrase reminder). Default off.

Use case: piping the stdout `ms1` into other tooling without
also capturing the stderr panel — the engraving card is intended
for human eyes during the engraving step, not for downstream
parsers.

## `--json` {#ms-encode-json}

Boolean. Emit a single JSON object on stdout instead of the
multi-line text form (canonical `ms1` + blank line + chunked
form). Default off.

The JSON object carries fields `schema_version`, `ms1`,
`language`, `word_count`, and `entropy_hex`. The `language` field
is **omitted entirely** when [`--hex`](#ms-encode-hex) is the
chosen input mode (the binary has no wordlist to record; the
field is gated by `serde(skip_serializing_if = "Option::is_none")`
per `crates/ms-cli/src/format.rs:40` and the
`encode_json_omits_language_for_hex_input` unit test pins this).

## Worked example — phrase → ms1

1. **ms** tab; pick **Encode (phrase/hex → ms1)**.
2. `--phrase` (masked): paste

   ```text
   abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about
   ```

3. Leave `--language` at default `english`.
4. Click **Run**. The run-confirm modal fires; confirm to proceed.

The output panel renders the canonical `ms1` in its default
5-character display grouping on stdout (the unbroken form
`ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f` is recovered
by dropping the separators, or by re-encoding with `--group-size 0`):

```{.text include="63-ms-encode-phrase.out"}
ms10e ntrsq qqqqq qqqqq qqqqq qqqqq qqqqq qqcj9 sxraq 34v7f
```

The stderr engraving card adds:

```{.text include="63-ms-encode-phrase.err" lines="1-3"}
word count: 12
language: english (BIP-39 checksum valid)
passphrase: not stored in ms1 (record separately if used)
```

## Worked example — hex → ms1

1. **ms** tab; pick **Encode (phrase/hex → ms1)**.
2. `--hex` (masked): enter `00000000000000000000000000000000`
   (16 bytes of zero = the canonical 12-word all-`abandon`
   entropy).
3. Note that the conditional-visibility engine `Hides`
   `--language` because `--hex` is set.
4. Optionally check `--no-engraving-card` to suppress the stderr
   panel.
5. **Run**. The modal fires; confirm to proceed.

Output (without engraving card; same canonical `ms1` as the phrase
path, in the default 5-character display grouping):

```{.text include="63-ms-encode-hex.out"}
ms10e ntrsq qqqqq qqqqq qqqqq qqqqq qqqqq qqcj9 sxraq 34v7f
```

## Refusals

| Trigger | Refusal |
|---|---|
| Neither `--phrase` nor `--hex` supplied | clap-group refusal: `error: the following required arguments were not provided: <--phrase <PHRASE>\|--hex <HEX>>` |
| Both `--phrase` and `--hex` supplied | clap-group refusal: `error: the argument '--phrase <PHRASE>' cannot be used with '--hex <HEX>'` |
| `--hex` with empty string | exit 1 with `error: expected hex of length 32/40/48/56/64 chars (got empty input)` |
| `--hex` with odd-length value | exit 1 with `error: expected even-length hex (one byte = 2 chars); got <N> chars` |
| `--hex` with non-hex character | exit 1 with `error: invalid character '<c>' at position <i>` |
| `--hex` value of valid hex but not 16 / 20 / 24 / 28 / 32 bytes | ms-codec entropy-length refusal per `ms encode` upstream |
| `--phrase` with invalid BIP-39 checksum | exit 1 with `error: <bip39 error>` per `ms-cli` `friendly_bip39` |
| `--language <token>` not a member of the 10-value set | clap-level value-parser refusal |
