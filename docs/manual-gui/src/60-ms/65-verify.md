# `ms verify` {#ms-verify}

Verify that an `ms1` string is structurally valid (and optionally
round-trips against a supplied BIP-39 phrase). Exit-code-oriented:
**exit 0** = valid v0.1 entr, **exit 2** = format violation,
**exit 3** = valid-but-future format (`reserved-tag-not-emitted-in-v0.1`),
**exit 4** = `--phrase` round-trip mismatch, **exit 1** = user-input
error. Use this for cross-implementation conformance checks and
end-to-end engraving verification.

## Outline {#ms-verify-outline}

- [`--phrase`](#ms-verify-phrase) — original BIP-39 phrase to round-trip-check (secret-bearing; exit 4 on mismatch)
- [`--language`](#ms-verify-language) — BIP-39 wordlist for `--phrase` (default `english`)
- [`--json`](#ms-verify-json) — emit success JSON on stdout

## `--phrase` {#ms-verify-phrase}

The original BIP-39 phrase. **Secret-bearing** —
schema-`secret: true`. When supplied, the binary decodes the
`ms1` to entropy, parses the supplied phrase against the same
wordlist, derives a phrase from the entropy via
`Mnemonic::from_entropy_in`, and compares the two phrase strings.
Mismatch returns **exit 4** with `error: phrase mismatch (decoded
does not match --phrase)`.

The GUI renders this as a `SecretLineEdit` widget (masked text
field). Any non-empty value triggers the run-confirm modal at
click-Run time. A literal `-` value reads the phrase from stdin
(rarely useful from the GUI). Note that supplying both `ms1` and
`--phrase` as `-` simultaneously is refused (exit 1 with
`error: cannot read both ms1 and --phrase from stdin`).

## `--language` {#ms-verify-language}

BIP-39 wordlist used to parse [`--phrase`](#ms-verify-phrase).
Optional; defaults to `english`. Dropdown widget; 10 valid values,
hyphenated Chinese tokens (see [§61 cross-tab
divergence](#ms-per-tab-reference)). Unlike
[`ms decode --language`](#ms-decode-language), there is no
explicit-default annotation here — `--language` only matters when
`--phrase` is also supplied (the round-trip path), and the
mismatch-exit-4 path already surfaces wordlist mistakes.

### Outline {#ms-verify-language-outline}

- [`english`](#ms-verify-language-english)
- [`japanese`](#ms-verify-language-japanese)
- [`korean`](#ms-verify-language-korean)
- [`spanish`](#ms-verify-language-spanish)
- [`chinese-simplified`](#ms-verify-language-chinese-simplified)
- [`chinese-traditional`](#ms-verify-language-chinese-traditional)
- [`french`](#ms-verify-language-french)
- [`italian`](#ms-verify-language-italian)
- [`czech`](#ms-verify-language-czech)
- [`portuguese`](#ms-verify-language-portuguese)

### `english` {#ms-verify-language-english}

See [`ms encode --language english`](#ms-encode-language-english).

### `japanese` {#ms-verify-language-japanese}

See [`ms encode --language japanese`](#ms-encode-language-japanese).

### `korean` {#ms-verify-language-korean}

See [`ms encode --language korean`](#ms-encode-language-korean).

### `spanish` {#ms-verify-language-spanish}

See [`ms encode --language spanish`](#ms-encode-language-spanish).

### `chinese-simplified` {#ms-verify-language-chinese-simplified}

See [`ms encode --language
chinese-simplified`](#ms-encode-language-chinese-simplified).

### `chinese-traditional` {#ms-verify-language-chinese-traditional}

See [`ms encode --language
chinese-traditional`](#ms-encode-language-chinese-traditional).

### `french` {#ms-verify-language-french}

See [`ms encode --language french`](#ms-encode-language-french).

### `italian` {#ms-verify-language-italian}

See [`ms encode --language italian`](#ms-encode-language-italian).

### `czech` {#ms-verify-language-czech}

See [`ms encode --language czech`](#ms-encode-language-czech).

### `portuguese` {#ms-verify-language-portuguese}

See [`ms encode --language
portuguese`](#ms-encode-language-portuguese).

## `--json` {#ms-verify-json}

Boolean. Emit a single success JSON object on stdout (fields:
`schema_version`, `status`, `message`) instead of the labeled-text
OK line. Default off.

The `status` field is `valid` for the no-phrase path and
`round-trip-ok` for the `--phrase`-supplied path. Failure paths
emit the standard `ms-cli` error envelope on stdout under `--json`
(not the success object).

## Positional `ms1`

A single `ms1` string to verify. Optional at the clap level; when
omitted or set to literal `-`, the binary reads the string from
stdin.

## Worked example — bare verify

1. **ms** tab; pick **Verify (phrase ↔ ms1 round-trip)**.
2. Paste the canonical `ms1` into the `ms1` positional field:

   ```text
   ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f
   ```

3. Leave `--phrase` empty.
4. Click **Run** (no run-confirm modal — `--phrase` is empty so
   the form has no secret-bearing value).

The output panel emits the simple OK line on stdout and exit 0:

```{.text include="65-ms-verify-bare.out"}
OK: valid v0.1 entr (12 words, 50 chars)
```

## Worked example — round-trip with phrase

1. **ms** tab; pick **Verify (phrase ↔ ms1 round-trip)**.
2. Paste the canonical `ms1` into the `ms1` positional field.
3. `--phrase` (masked): paste

   ```text
   abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about
   ```

4. Leave `--language` at default `english`.
5. **Run**. The run-confirm modal fires (because `--phrase` is
   non-empty); confirm to proceed.

The output panel emits the round-trip OK line on stdout and exit 0:

```{.text include="65-ms-verify-roundtrip.out"}
OK: round-trip valid (12 words, language=english)
```

If the supplied phrase does not match the entropy in the `ms1`
(or if the wordlist disagrees), the binary exits 4 with stderr
`error: phrase mismatch (decoded does not match --phrase)`.

## Refusals

| Trigger | Refusal |
|---|---|
| `ms1` and `--phrase` both `-` | exit 1 with `error: cannot read both ms1 and --phrase from stdin` |
| Positional `ms1` is not a parseable BIP-93 string | exit 1 with `error: <codex32 parse error>` |
| `ms1` has wrong HRP / threshold / share-index / prefix byte | exit 2 with the matching `FormatViolation` message |
| `ms1` tag reserved-not-emitted in v0.1 | **exit 3, success-shaped stdout** `OK: valid future format (v0.2+, tag <tag>)` (per `verify.rs:127`); stderr silent in text mode (suppressed by the `FutureFormat` carve-out at `ms-cli/src/main.rs:147-155`). Under `--json`, the binary emits the standard error envelope on stdout instead |
| `--phrase` supplied and does not round-trip-match the decoded entropy | **exit 4** with `error: phrase mismatch (decoded does not match --phrase)` |
| `--phrase` does not parse under the chosen `--language` | exit 1 with `error: <bip39 error>` per `friendly_bip39` |
| `--language <token>` not a member of the 10-value set | clap-level value-parser refusal |
