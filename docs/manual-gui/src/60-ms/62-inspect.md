# `ms inspect` {#ms-inspect}

Inspect an `ms1` string's structural fields and decoder verdict.
Lenient parser: returns a structured report even when the string
would fail one or more v0.1 decode rules, listing the failing
rules in ascending SPEC §4 order. Use this for cross-implementation
diagnostics or to understand exactly which validation rule a
candidate `ms1` fails.

> **GUI form:** see [GUI Forms › ms › inspect](#gui-form-ms-inspect).

## Outline {#ms-inspect-outline}

- [`--json`](#ms-inspect-json) — emit a single JSON object on stdout
- [`--in`](#ms-inspect-in) — read the `ms1` string from a file

## `--json` {#ms-inspect-json}

Boolean. Emit JSON output instead of the labeled-block text verdict
and fields. Default off.

## `--in` {#ms-inspect-in}

Path widget. Read the `ms1` string from FILE instead of the
positional. It is the private channel that frees stdin: a path on
argv is not secret, so a run that uses it needs no
`--allow-argv-secret` and shows no run-confirm modal. The GUI lets only
one input source through: the first filled source wins and the others
grey out, because the CLI refuses `--in` alongside them.

## Positional `ms1`

A single `ms1` string to inspect. Optional at the clap level; when
omitted or set to literal `-`, the binary reads the string from
stdin. The GUI renders this as a text field at the bottom of the
form.

## Worked example

1. **ms** tab; pick **Inspect (verdict + fields)** from the
   subcommand selector.
2. Paste the canonical `ms1` into the `ms1` positional field:

   ```text
   ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f
   ```

3. Leave `--json` unchecked.
4. Click **Run**. The run-confirm modal fires, because the `ms1`
   positional is secret-bearing; its argv shows `--allow-argv-secret`
   right after the subcommand, which the GUI adds on Run (see
   [Secret handling](#secret-argv-opt-in)). Confirm to proceed.

The output panel renders the verdict line and structured fields
on stdout:

```{.text include="62-ms-inspect.out"}
OK: would decode v0.1

hrp: ms
threshold: 0
tag: entr
share_index: s
prefix_byte: 0x00
payload_bytes: 00000000000000000000000000000000
checksum_valid: true
kind: entr
```

For an `ms1` that fails one or more validation rules, the verdict
line is `FAIL: would NOT decode v0.1` followed by one `reason:`
line per failed SPEC §4 rule, then the structured fields.

## Refusals

| Trigger | Refusal |
|---|---|
| Positional `ms1` is not a parseable BIP-93 string (includes the empty-stdin case, which decodes as `InvalidLength(0)`) | exit 1 with the `friendly_codex32`-rendered text, e.g. `error: string length 0 not a valid codex32 length` for empty input |
