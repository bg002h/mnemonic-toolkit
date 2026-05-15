# `md inspect` {#md-inspect}

Decode one or more `md1` strings and pretty-print their contents
as human-readable wallet-policy template + metadata. The default
inspection target for end users; for the low-level payload-bit
view see [`md bytecode`](#md-bytecode).

## `--json` {#md-inspect-json}

Boolean. Emit structured JSON instead of pretty-printed text.
Default off.

## Positional `md1-strings`

One or more `md1` strings, supplied as positional arguments.
Required (at least one). Repeating. The GUI renders this as a
text field that accepts whitespace-separated strings; pass
multiple by separating with spaces.

## Worked example

1. **md** tab; pick **Inspect (decode + pretty-print)** from the
   subcommand selector.
2. Paste the canonical 3 `md1` strings into the `md1-strings`
   positional field, separated by spaces.
3. Leave `--json` unchecked.
4. Click **Run** (no run-confirm modal — `md1` is public material).

The output panel renders the decoded template + metadata for
each `md1` on stdout: wallet-policy template string, descriptor
context (segwitv0 / tap), per-placeholder origin metadata, and
`policy_id_stub` cross-binding bytes.

## Refusals

| Trigger | Refusal |
|---|---|
| No positional `md1-strings` provided | clap-level `required` error |
| Any positional that does not parse as `md1` | md1-decode error per `md-cli` |
