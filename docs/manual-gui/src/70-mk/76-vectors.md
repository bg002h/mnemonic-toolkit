# `mk vectors` {#mk-vectors}

Maintainer tool: print the SHA-pinned v0.1 `mk` test-vector
corpus as JSON. The corpus is re-exported from
`mk_codec::test_vectors::V0_1_JSON` and `include_str!`-baked into
the binary at compile time. End users do NOT need this
subcommand; it is included in the GUI's schema for completeness
and because the upstream binary surfaces it.

> **GUI form:** see [GUI Forms › mk › vectors](#gui-form-mk-vectors).

## Outline {#mk-vectors-outline}

- [`--pretty`](#mk-vectors-pretty) — pretty-print the JSON output
- [`--out`](#mk-vectors-out) — write per-fixture files to a directory instead of stdout

## `--pretty` {#mk-vectors-pretty}

Boolean. Indent the JSON output for human readability. Default
off (compact JSON, byte-identical to the in-tree corpus file
modulo a trailing newline).

**Source-vs-help-text discrepancy:** the upstream help line
(`mk vectors --help`) reads "Ignored when `--out` is supplied",
and the GUI schema mirrors that description. Source actually
honors `--pretty` even when `--out` is set: per
`crates/mk-cli/src/cmd/vectors.rs:70-74`, each per-fixture
written file uses `serde_json::to_string_pretty` when `pretty`
is true. The manual mirrors source-truth; the help-text drift is
tracked as a v1.1 cycle FOLLOWUP (see
`design/FOLLOWUPS.md::mk-vectors-pretty-out-help-mismatch` —
filed at batch-8 closure).

## `--out` {#mk-vectors-out}

Optional output directory. **Path widget.** When set, the binary
writes one `<name>.json` file per fixture in the corpus's
`vectors` array (the `name` field of each fixture entry, e.g.
`V2_bip84_mainnet_1_stub_with_fp.json`) into the supplied
directory instead of emitting a single JSON blob to stdout. The
binary creates the directory if it does not exist
(`std::fs::create_dir_all`).

After writing, the binary emits a one-line summary to **stderr**:
`wrote <N> vector file(s) to <path>`. Stdout is empty in this
mode.

## Worked example — stdout, pretty-printed

1. **mk** tab; pick **Vectors (test-vector dump)**.
2. Check `--pretty`.
3. Leave `--out` empty.
4. **Run**.

The output panel renders the indented JSON corpus on stdout. The
top-level object has fields `family_token`, `schema`, and
`vectors`; the `vectors` array carries one entry per fixture
(40 fixtures at `mk-cli v0.3.1` / `mk-codec v0.3.0`).

## Worked example — per-fixture files

1. **mk** tab; **Vectors** subcommand.
2. `--out`: supply a directory path (e.g. `/tmp/mk-vectors-out`).
   The directory will be created if missing.
3. Optionally check `--pretty` — contrary to the help-text, the
   per-fixture files DO get indented when `--pretty` is set.
4. **Run**.

Stderr emits the one-line summary (`wrote 40 vector file(s) to
…`); stdout is empty. The output directory contains one
`<name>.json` file per fixture.

## Refusals

| Trigger | Refusal |
|---|---|
| Internal vector-corpus parse failure | exit 64 with `error: vector corpus parse: <serde error>` (defensive; the corpus is baked at compile time and the build's integration tests gate this) |
| `--out` path exists but is not a directory, or directory is not writable | exit 1 with `error: io error: …` per `std::io::Error` |
| Per-fixture serialization or write failure | exit 1 with `error: io error: …` or exit 64 with `error: vector serialize: …` |
