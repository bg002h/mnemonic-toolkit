# Install the toolkit

You need three command-line binaries: `mnemonic`, `md`, and `ms`.
This Quick Start uses all three.

## Install the three binaries

The toolkit's installer downloads each CLI's prebuilt binary from its
pinned GitHub release, checks it against the `SHA256SUMS` file
published with that release, and refuses it on a mismatch. It needs no
Rust toolchain:

```sh
sh -c "$(curl -fsSL https://raw.githubusercontent.com/bg002h/mnemonic-toolkit/master/scripts/install.sh)" -- --only mnemonic,md,ms
```

The binaries land in `~/.cargo/bin/`. Make sure that directory is on
your `PATH`. Do not use `cargo install` from crates.io: the copies
there are several releases older than this guide.

To build the same versions from source instead, add `--from-source`
to that command. It then needs a Rust toolchain: the three CLIs build
on `rustc` ≥ 1.85, and `rustup` is the easiest install:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Smoke check

```sh
mnemonic --version
md --version
ms --version
```

You should see a version line from each. `mnemonic --version`
must report `0.8.0` or later — earlier versions used a different
flag set than the rest of this guide.

The reference manual's install chapter covers the other install
paths.

## If you prefer a GUI

A cross-platform graphical front-end, `mnemonic-gui`, drives the
same three CLIs underneath and exposes every form 1:1 to a CLI
subcommand. The same installer adds it: run it again with
`--only mnemonic-gui`. You will still need the three CLIs installed
(the GUI invokes them as subprocesses), so finish this chapter either
way. The end-user manual's install chapter has the full details.

Onward: generate the entropy you'll feed into your first bundle.
