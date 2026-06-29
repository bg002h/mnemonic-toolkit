# `md vectors` {#md-vectors}

Maintainer tool: regenerate the test-vector corpus consumed by
`md-cli`'s integration tests. End users do NOT need this
subcommand; it is included in the GUI's schema for completeness
and because the upstream binary surfaces it.

> **GUI form:** see [GUI Forms › md › vectors](#gui-form-md-vectors).

## `--out` {#md-vectors-out}

Output directory for the regenerated test-vector corpus. Path
widget. The directory must exist (or be creatable). The corpus
is a set of files describing canonical input/output pairs for
each `md` subcommand path.

## Worked example

This subcommand is not part of the typical end-user flow. The
maintainer use case is:

1. **md** tab; pick **Vectors (test-vector corpus)**.
2. `--out`: provide a path to an empty directory.
3. **Run**.

The output panel reports the number of vectors written and the
emitted file names.

## Refusals

| Trigger | Refusal |
|---|---|
| `--out` path is not writable | filesystem I/O error per `md-cli` |
