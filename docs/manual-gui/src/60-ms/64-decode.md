# `ms decode` {#ms-decode}

Decode an `ms1` string back to its BIP-39 mnemonic and raw entropy
bytes. The inverse of [`ms encode`](#ms-encode). Wordlist is
disambiguated via [`--language`](#ms-decode-language); when
omitted, the binary defaults to English and emits an explicit
`DEFAULT` annotation on both stderr and the stdout `language:`
line so users notice when the default may not match their wallet.

## Outline {#ms-decode-outline}

- [`--language`](#ms-decode-language) — BIP-39 wordlist for the recovered phrase (default `english`, with explicit-default annotation)
- [`--json`](#ms-decode-json) — emit a single JSON object on stdout

## `--language` {#ms-decode-language}

BIP-39 wordlist used to render the recovered phrase. Optional;
when omitted, the binary defaults to English **and emits a
"DEFAULT" annotation** per SPEC §6.3 on both the stderr
diagnostic line and the stdout `language:` line. This is the
hazard-surfacing default: if your wallet was created with a
non-English wordlist and you decode without `--language`, the
output mnemonic will be the wrong words for the same entropy
bytes — the binary nudges you to verify.

Dropdown widget; 10 valid values, hyphenated Chinese tokens (see
[§61 cross-tab divergence](#ms-per-tab-reference)).

### Outline {#ms-decode-language-outline}

- [`english`](#ms-decode-language-english)
- [`japanese`](#ms-decode-language-japanese)
- [`korean`](#ms-decode-language-korean)
- [`spanish`](#ms-decode-language-spanish)
- [`chinese-simplified`](#ms-decode-language-chinese-simplified)
- [`chinese-traditional`](#ms-decode-language-chinese-traditional)
- [`french`](#ms-decode-language-french)
- [`italian`](#ms-decode-language-italian)
- [`czech`](#ms-decode-language-czech)
- [`portuguese`](#ms-decode-language-portuguese)

### `english` {#ms-decode-language-english}

See [`ms encode --language english`](#ms-encode-language-english).
Default when `--language` is omitted; output carries an explicit
`DEFAULT` annotation in that case.

### `japanese` {#ms-decode-language-japanese}

See [`ms encode --language japanese`](#ms-encode-language-japanese).

### `korean` {#ms-decode-language-korean}

See [`ms encode --language korean`](#ms-encode-language-korean).

### `spanish` {#ms-decode-language-spanish}

See [`ms encode --language spanish`](#ms-encode-language-spanish).

### `chinese-simplified` {#ms-decode-language-chinese-simplified}

See [`ms encode --language
chinese-simplified`](#ms-encode-language-chinese-simplified).

### `chinese-traditional` {#ms-decode-language-chinese-traditional}

See [`ms encode --language
chinese-traditional`](#ms-encode-language-chinese-traditional).

### `french` {#ms-decode-language-french}

See [`ms encode --language french`](#ms-encode-language-french).

### `italian` {#ms-decode-language-italian}

See [`ms encode --language italian`](#ms-encode-language-italian).

### `czech` {#ms-decode-language-czech}

See [`ms encode --language czech`](#ms-encode-language-czech).

### `portuguese` {#ms-decode-language-portuguese}

See [`ms encode --language
portuguese`](#ms-encode-language-portuguese).

## `--json` {#ms-decode-json}

Boolean. Emit a single JSON object on stdout (fields:
`schema_version`, `entropy_hex`, `phrase`, `language`,
`word_count`, `language_defaulted`) instead of the labeled-block
text form. Default off.

The `language_defaulted` field is `true` iff the user omitted
`--language` — useful for programmatic detection of the
hazard-surfacing case.

## Positional `ms1`

A single `ms1` string to decode. Optional at the clap level; when
omitted or set to literal `-`, the binary reads the string from
stdin. The GUI renders this as a text field at the bottom of the
form.

## Worked example

1. **ms** tab; pick **Decode (ms1 → phrase)**.
2. Paste the canonical `ms1` into the `ms1` positional field:

   ```text
   ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f
   ```

3. Leave `--language` unset.
4. Click **Run** (no run-confirm modal — `ms decode` has no
   secret-bearing flag, only public input).

The output panel renders the decoded entropy, phrase, and the
language line (with the explicit-default annotation since
`--language` was omitted):

```{.text include="64-ms-decode.out"}
entropy: 00000000000000000000000000000000
phrase: abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about
language: english (12 words, default — verify against your records)
```

Stderr adds the explicit-default diagnostic:

```{.text include="64-ms-decode.err" lines="1-1"}
note: --language defaulted to 'english'; if your wallet was created with a different wordlist, decode with --language <lang>.
```

## Refusals

| Trigger | Refusal |
|---|---|
| Positional `ms1` is not a parseable BIP-93 string | exit 1 with `error: <codex32 parse error>` |
| `ms1` parses but its tag is `entr`-adjacent unknown (not `entr`, not reserved-not-emitted) | exit 2 with `error: unknown tag …` |
| `ms1` tag is `reserved-not-emitted-in-v0.1` | **exit 3, silent in text mode** (stdout silent; stderr suppressed by the `FutureFormat`-text-mode carve-out at `ms-cli/src/main.rs:147-155`). Under `--json`, the standard error envelope on stdout carries `kind: "FutureFormat"` |
| `ms1` has wrong HRP / threshold / share-index / prefix byte | exit 2 with the matching `FormatViolation` message |
| `--language <token>` not a member of the 10-value set | clap-level value-parser refusal |
