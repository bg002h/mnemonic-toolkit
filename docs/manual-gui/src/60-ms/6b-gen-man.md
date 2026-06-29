# `ms gen-man` {#ms-gen-man}

Emit `roff` man pages for the whole `ms` CLI tree into a directory. The
pages are generated directly from the compiled clap `Command` tree
(`clap_mangen`), so they are binary-faithful by construction. One page is
written per (nested) subcommand, hyphen-joined: `ms.1` (root),
`ms-encode.1`, `ms-decode.1`, and so on. The project's `install.sh`
invokes this after `cargo install` to drop the pages into the user
manpath (no sudo).

The GUI exposes `gen-man` as a one-field **Gen Man** form on the `ms`
tab. It carries no secret material; the run-confirm modal does not fire.

> **GUI form:** see [GUI Forms › ms › gen-man](#gui-form-ms-gen-man).

## `--out` {#ms-gen-man-out}

Path field, **required**. The directory to write the `*.1` man pages
into; created if absent (`mkdir -p` semantics). The GUI renders it as a
Path widget with a directory picker. After a run, read a page with
`man -M <dir> ms`, or copy the pages into a directory on your `MANPATH`.

\index{ms gen-man}
