# Installing the toolkit

The m-format constellation ships as four crates across four sibling repositories,
each with a standalone CLI binary (`mnemonic`, `md`, `ms`, `mk`).
This chapter installs the four binaries.

:::primer
**The four sibling repos** are `bg002h/mnemonic-toolkit` (CLI:
`mnemonic`), `bg002h/descriptor-mnemonic` (CLI: `md`),
`bg002h/mnemonic-secret` (CLI: `ms`), and `bg002h/mnemonic-key`
(library `mk-codec` plus CLI `mk`, since v0.2). Each tags its releases
and attaches prebuilt binaries to them. The versions this manual
documents are the ones pinned in the toolkit's installer,
`scripts/install.sh` (`install.sh --list` prints them).
:::

**Do not install these CLIs from crates.io.** The copies there are
several releases older than this manual (for example `md` 0.13 against
the 0.20.3 documented here) and lack flags it describes.

## Path A — the installer (prebuilt binaries)

This is the recommended path. It needs no Rust toolchain:

```sh
sh -c "$(curl -fsSL https://raw.githubusercontent.com/bg002h/mnemonic-toolkit/master/scripts/install.sh)" -- --no-gui
```

For each CLI the installer downloads the binary for your platform from
its pinned GitHub release and checks it against the `SHA256SUMS` file
published with that release. It refuses a download whose digest does
not match, or that no published checksum covers, and it runs each
binary once before installing it, refusing one that does not report
the pinned version. The releases are not signed, so the check proves
the file is the one the release published, not who built it. Binaries
land in `~/.cargo/bin/` (`--root DIR` or `$CARGO_INSTALL_ROOT` puts them
in `DIR/bin/` instead; cargo's `install.root` config setting is not read);
the installer warns if that directory is not on your `PATH`.
`--root` covers the binaries only: man pages go to
`${XDG_DATA_HOME:-~/.local/share}/man/man1` unless you pass
`--man-dir DIR`, or `--no-man` to skip them.
`--dry-run` shows every URL first; `--help` lists the options.

If a release has no binary for your platform (FreeBSD, for example),
the installer says so and builds that component from the same pinned
tag with `cargo`, which then needs a Rust toolchain (below).
`--from-source` does that for every component. The same happens on
Linux where a binary needs a newer C library than yours:

| Binary | Needs |
|---|---|
| `mnemonic-gui`, Linux x86_64 | glibc ≥ 2.39 |
| `mnemonic-gui`, Linux aarch64 | glibc ≥ 2.18 |
| `md`, Linux x86_64 | glibc ≥ 2.34 |
| `mnemonic`, `ms`, `mk`, and `md` on aarch64 | any Linux (static) |

So on Ubuntu 22.04 (glibc 2.35) or Debian 12 (2.36) the GUI is built
from source, and on Ubuntu 20.04 (2.31) `md` is too. On a musl system
(Alpine, Void musl) the GUI is always built from source, because the
static musl build cannot open a window.

### Building the pinned tags from source

With a Rust toolchain you can build the same versions yourself. The
four CLIs build on `rustc` ≥ 1.85 (the toolkit MSRV); the optional
`mnemonic-gui` overlay (Path D) needs `rustc` ≥ 1.88. Install the
toolchain via `rustup` if you do not have it:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Then run the installer with `--from-source` added: it builds each
pinned tag with `cargo install --locked --git <repo> --tag <pin>`.
To see those four commands, with their tags, without running them:

```sh
sh -c "$(curl -fsSL https://raw.githubusercontent.com/bg002h/mnemonic-toolkit/master/scripts/install.sh)" -- --no-gui --from-source --dry-run
```

If you run such a command by hand, always pass `--tag`: without it
cargo builds the repository's default branch, which is not a release.

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
distribution image; use the release binaries (Path A), which for the
toolkit's Linux builds are reproducible from source (see
`docs/verify-reproducibility.md` in the toolkit repository).

## Path D — graphical interface (`mnemonic-gui`)

If you prefer a graphical front-end over the four CLIs,
`bg002h/mnemonic-gui` provides one. It is a cross-platform desktop
application that drives the same four binaries underneath: every
form maps 1:1 onto a CLI subcommand, the assembled command line is
visible before you run it, and the GUI never substitutes its own
implementation for the CLI behaviour. The CLIs remain the
ground truth.

The installer in Path A installs it too: run it without `--no-gui`,
or with `--only mnemonic-gui` to add only the GUI. It downloads the
pinned `mnemonic-gui` release binary for your platform and checks it
the same way. The four CLIs must still be on your `PATH` — the GUI
invokes them as subprocesses. The binaries are currently unsigned on
macOS and Windows; on first launch you will need to right-click → Open
(macOS) or click "More info → Run anyway" past SmartScreen (Windows).

The GUI's source lives at `bg002h/mnemonic-gui`; a dedicated
standalone paper covering the GUI in depth is planned separately.
This manual continues with the CLI surface.

Building the GUI from source requires `rustc` ≥ 1.88 (its
dependencies' MSRV); the four CLIs build on `rustc` ≥ 1.85. When the
installer (`scripts/install.sh`) has to build the GUI from source, it
skips it with a warning on an older toolchain and still installs the
four CLIs — upgrade `rustc` and re-run to add the GUI.

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
