# Install the toolkit

You need three command-line binaries: `mnemonic`, `md`, and `ms`.
This Quick Start uses all three. The recommended path until the
crates land on crates.io is `cargo install --git`.

## Pre-requisites

A recent **Rust toolchain** — the three CLIs build on `rustc` ≥ 1.85.
(The optional GUI in the last section needs `rustc` ≥ 1.88.) If you
don't have one, the easiest install is:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Verify with `cargo --version` and `rustc --version`.

## Install the three binaries

```sh
cargo install --locked --git https://github.com/bg002h/mnemonic-toolkit.git mnemonic-toolkit
cargo install --locked --git https://github.com/bg002h/descriptor-mnemonic.git md-cli
cargo install --locked --git https://github.com/bg002h/mnemonic-secret.git ms-cli
```

Each command compiles from source and writes the binary into
`~/.cargo/bin/`. Make sure that directory is on your `PATH`
(rustup adds it automatically on most platforms).

## Smoke check

```sh
mnemonic --version
md --version
ms --version
```

You should see a version line from each. `mnemonic --version`
must report `0.8.0` or later — earlier versions used a different
flag set than the rest of this guide.

The reference manual covers Docker and from-source build paths;
this Quick Start sticks to the cargo path.

## If you prefer a GUI

A cross-platform graphical front-end, `mnemonic-gui`, drives the
same three CLIs underneath and exposes every form 1:1 to a CLI
subcommand. Pre-built v0.2.0 binaries for Linux (x86_64, aarch64),
macOS (x86_64, aarch64), and Windows (x86_64) are attached to the
GitHub release:
<https://github.com/bg002h/mnemonic-gui/releases/tag/mnemonic-gui-v0.2.0>.
You will still need the three CLIs installed (the GUI invokes them
as subprocesses), so finish this chapter either way. The
end-user manual's install chapter has the full details.

Building the GUI from source requires `rustc` ≥ 1.88 (a newer MSRV
than the CLIs' ≥ 1.85). The constellation installer auto-skips the
GUI with a warning on an older toolchain.

Onward: generate the entropy you'll feed into your first bundle.
