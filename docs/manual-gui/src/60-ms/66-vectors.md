# `ms vectors` {#ms-vectors}

Maintainer tool: print the SHA-pinned v0.1 `ms` test-vector corpus
as JSON on stdout. The corpus is `include_str!`-baked into the
binary at compile time from
`crates/ms-cli/vectors/v0.1.json`; parity against
`crates/ms-codec/tests/vectors/v0.1.json` is enforced by a
codec-side parity test. End users do NOT need this subcommand; it
is included in the GUI's schema for completeness and because the
upstream binary surfaces it.

> **GUI form:** see [GUI Forms › ms › vectors](#gui-form-ms-vectors).

## `--pretty` {#ms-vectors-pretty}

Boolean. Indent the JSON output for human readability. Default
off (compact JSON, byte-identical to the in-tree corpus file
modulo a trailing newline).

## Worked example

This subcommand is not part of the typical end-user flow. The
maintainer use case is:

1. **ms** tab; pick **Vectors (test-vector dump)**.
2. Optionally check `--pretty` for indented JSON.
3. **Run**. (No run-confirm modal — no secret-bearing flag.)

The output panel renders the test-vector corpus as JSON on
stdout. Pipe through `jq` for ad-hoc inspection, or diff against
the in-tree `vectors/v0.1.json` to detect drift.

## Refusals

| Trigger | Refusal |
|---|---|
| Internal vector-corpus parse failure | exit 1 with `error: vector corpus parse: <serde error>` (defensive; corpus is baked at compile time so this is unreachable in practice) |
