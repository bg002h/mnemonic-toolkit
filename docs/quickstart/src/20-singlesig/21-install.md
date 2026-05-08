# Install the toolkit

You need three command-line binaries: `mnemonic`, `md`, and `ms`.
This Quick Start uses all three. The recommended path until the
crates land on crates.io is `cargo install --git`.

## Pre-requisites

A recent **Rust toolchain** (1.77 or newer). If you don't have one,
the easiest install is:

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

Onward: generate the entropy you'll feed into your first bundle.
