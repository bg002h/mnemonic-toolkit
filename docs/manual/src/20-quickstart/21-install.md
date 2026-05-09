# Installing the toolkit

The m-format constellation ships as four crates across four sibling repositories,
each with a standalone CLI binary (`mnemonic`, `md`, `ms`, `mk`).
This chapter installs the four binaries.

:::primer
**The four sibling repos** are `bg002h/mnemonic-toolkit` (CLI:
`mnemonic`), `bg002h/descriptor-mnemonic` (CLI: `md`),
`bg002h/mnemonic-secret` (CLI: `ms`), and `bg002h/mnemonic-key`
(library `mk-codec` plus CLI `mk`, since v0.2). They are published
to crates.io as soon as their dependencies land there; until then,
install from source via `cargo install --git`. Future versions will
pin to crates.io directly.
:::

## Pre-requisites

You need a recent **Rust toolchain** — the `rust-toolchain.toml` in
each repository pins `1.77+`. Install via `rustup` if you do not have
it:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

(Or use your distribution's package manager. Verify with
`cargo --version` and `rustc --version`.)

## Path A — install from source via cargo

This is the recommended path until crates.io publication completes:

```sh
cargo install --locked --git https://github.com/bg002h/mnemonic-toolkit.git mnemonic-toolkit
cargo install --locked --git https://github.com/bg002h/descriptor-mnemonic.git md-cli
cargo install --locked --git https://github.com/bg002h/mnemonic-secret.git ms-cli
cargo install --locked --git https://github.com/bg002h/mnemonic-key.git --tag mk-cli-v0.2.0 --bin mk
```

Each command compiles the source and writes the binary into
`~/.cargo/bin/`. Make sure that directory is on your `PATH` (most
rustup installations add it automatically).

Verify the binaries:

```sh
mnemonic --version
md --version
ms --version
mk --version
```

## Path B — clone and build (for contributors)

If you intend to read the code, modify it, or run the test suites:

```sh
git clone https://github.com/bg002h/mnemonic-toolkit.git
git clone https://github.com/bg002h/descriptor-mnemonic.git
git clone https://github.com/bg002h/mnemonic-key.git
git clone https://github.com/bg002h/mnemonic-secret.git

cd mnemonic-toolkit && cargo build --release --bin mnemonic
cd ../descriptor-mnemonic && cargo build --release --bin md
cd ../mnemonic-secret && cargo build --release --bin ms
cd ../mnemonic-key && cargo build --release --bin mk
```

The release binaries land in each repo's `target/release/`. Either
copy them onto your `PATH` (`cp target/release/mnemonic ~/.local/bin/`)
or invoke them directly.

The four repos coordinate via cross-repo `FOLLOWUPS.md` mirrors and
shared release-cycle conventions; clone them as siblings rather than
nested. The `CLAUDE.md` files in each repo are non-binding guidance
for AI-assisted contributions.

## Path C — Docker (CI / reproducible builds)

For CI and reproducible installations, the toolkit ships a build
image at `docs/manual/Dockerfile.build` (used by `make pdf-docker`
for the manual). For the *binaries themselves* there is no
distribution image yet — see the [follow-up](#manual-coverage)
on cargo-publishing the workspace; that's the prerequisite for
official binary releases.

## Verifying your install

A trivial smoke check that all four CLIs respond:

```sh
mnemonic --help | head -5
md --help | head -5
ms --help | head -5
mk --help | head -5
```

You should see a short usage banner from each. Now read on to
[Your first bundle](#your-first-bundle) — a single-sig BIP-84
walkthrough that produces three real card strings on your terminal.
