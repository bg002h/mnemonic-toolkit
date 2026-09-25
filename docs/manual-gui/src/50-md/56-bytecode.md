# `md bytecode` {#md-bytecode}

Low-level inspector: dump the raw payload bits of one or more
`md1` strings. Intended for md-cli debugging and for
cross-implementation conformance testing; not typically needed
by end users (see [`md inspect`](#md-inspect) for the
human-friendly view).

> **GUI form:** see [GUI Forms › md › bytecode](#gui-form-md-bytecode).

## Outline {#md-bytecode-outline}

- [`--json`](#md-bytecode-json) — emit JSON output
- [`--in`](#md-bytecode-in) — read the `md1` strings from a file, one per line

## `--json` {#md-bytecode-json}

Boolean. Emit JSON output. Default off.

## `--in` {#md-bytecode-in}

Path widget. Read the `md1` strings from FILE, one per line, instead of
the positional (P3 §6b, the file channel added in md 0.20). The
GUI treats `--in` as an alternative to the positional, which is no
longer marked required: fill one or the other.

## Positional `strings`

One or more `md1` strings whose payload bits to dump. Required,
repeating.

## Worked example

1. **md** tab; pick **Bytecode (raw payload bits)**.
2. Paste the canonical first md1 into `strings`.
3. **Run**.

The output panel renders the raw decoded bytes (hex) plus the
per-field bit decomposition (template tag, placeholder count,
multipath shape, policy_id_stub bytes). Use this for
cross-implementation byte-level diffing.

## Refusals

| Trigger | Refusal |
|---|---|
| No positional `strings` provided | clap-level `required` error |
| Any positional that does not parse as `md1` | md1-decode error per `md-cli` |
