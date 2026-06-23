# `ms inspect` {#ms-inspect}

Inspect an `ms1` string's structural fields and decoder verdict.
Lenient parser: returns a structured report even when the string
would fail one or more v0.1 decode rules, listing the failing
rules in ascending SPEC §4 order. Use this for cross-implementation
diagnostics or to understand exactly which validation rule a
candidate `ms1` fails.

## `--json` {#ms-inspect-json}

Boolean. Emit JSON output instead of the labeled-block text verdict
and fields. Default off.

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
4. Click **Run** (no run-confirm modal — `ms inspect` has no
   secret-bearing flag).

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
