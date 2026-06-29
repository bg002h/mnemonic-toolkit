# `mk gen-man` {#mk-gen-man}

Emit `roff` man pages for the whole `mk` CLI tree into a directory. The
pages are generated directly from the compiled clap `Command` tree
(`clap_mangen`), so they are binary-faithful by construction. One page is
written per (nested) subcommand, hyphen-joined: `mk.1` (root),
`mk-encode.1`, `mk-decode.1`, and so on. The project's `install.sh`
invokes this after `cargo install` to drop the pages into the user
manpath (no sudo).

The GUI exposes `gen-man` as a one-field **Gen Man** form on the `mk`
tab. It carries no secret material; the run-confirm modal does not fire.

## `--out` {#mk-gen-man-out}

Path field, **required**. The directory to write the `*.1` man pages
into; created if absent (`mkdir -p` semantics). The GUI renders it as a
Path widget with a directory picker. After a run, read a page with
`man -M <dir> mk`, or copy the pages into a directory on your `MANPATH`.

\index{mk gen-man}
