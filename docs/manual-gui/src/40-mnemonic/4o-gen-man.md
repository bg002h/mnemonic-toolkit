# `mnemonic gen-man` {#mnemonic-gen-man}

Emit `roff` man pages for the whole `mnemonic` CLI tree into a directory.
The pages are generated directly from the compiled clap `Command` tree
(`clap_mangen`), so they are **binary-faithful by construction** — the man
page cannot drift from the binary's actual flag surface. One page is
written per (nested) subcommand, named hyphen-joined parent→child:
`mnemonic.1` (root), `mnemonic-bundle.1`,
`mnemonic-seed-xor-split.1`, and so on. The project's `install.sh`
invokes this after `cargo install` to drop the pages into the user
manpath (no sudo, no system files).

The GUI exposes `gen-man` as a one-field **Gen Man** form on the
`mnemonic` tab. It carries no secret material; the run-confirm modal does
not fire.

> **GUI form:** see [GUI Forms › mnemonic › gen-man](#gui-form-mnemonic-gen-man).

## `--out` {#mnemonic-gen-man-out}

Path field, **required**. The directory to write the `*.1` man pages
into; created if absent (`mkdir -p` semantics). The GUI renders it as a
Path widget with a directory picker. After a run, point your pager at the
directory — `man -M <dir> mnemonic` — or copy the pages into a directory
already on your `MANPATH`.

\index{mnemonic gen-man}
