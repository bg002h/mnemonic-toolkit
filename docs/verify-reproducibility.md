# Verifying reproducibility of the `mnemonic` musl release binaries

> **Scope (task #23, P1).** This document covers the **x86_64-unknown-linux-musl**
> `mnemonic` binary. The aarch64-musl leg (P2) and the three codec CLIs
> `md` / `ms` / `mk` (P3) get their own sections as those phases land. gnu, the
> GUI, and multi-builder signed attestation are out of scope this cycle
> (catalog-only FOLLOWUPs).

## 1. What reproducibility buys you — provenance, not just integrity

The published `SHA256SUMS.<arch>` next to each release tarball lets you confirm a
download matches **what the maintainer uploaded** (integrity). It does NOT, on
its own, tell you the maintainer built it **from the claimed source**. A
*reproducible* build closes that gap: if you rebuild from the exact source at the
exact tag in the exact pinned environment and get the **bit-for-bit identical**
tarball, the published hash becomes a **provenance** statement — "provably built
from this source@commit" — verifiable by anyone, with no trust in the
maintainer's machine (cf. Bitcoin Core Guix, Tor).

## 2. Source — keyed off the COMMIT SHA (not the tag name)

```sh
git clone https://github.com/bg002h/mnemonic-toolkit
cd mnemonic-toolkit
git checkout <mnemonic-toolkit-vX.Y.Z>      # the release tag
git rev-parse HEAD                            # MUST equal the published source_commit
```

**Invariant.** The published hash is valid **only** for the tuple
`{ source COMMIT SHA + SOURCE_DATE_EPOCH + container digest }`. A tag is mutable;
if it is moved / re-cut / force-pushed, the tagged-commit timestamp changes, the
epoch changes, the binary changes, and the previously-published hash is
**invalidated and re-published**. Always verify against the **commit SHA**
published in `PROVENANCE.<arch>.txt`, not the tag name.

`SOURCE_DATE_EPOCH` is derived identically by the maintainer's CI and by you:

```sh
SOURCE_DATE_EPOCH=$(git show -s --format=%ct <tagged-commit-SHA>)
```

`%ct` is the **committer date of the exact tagged commit** — NOT the tag name,
NOT an annotated tag's own `%(taggerdate)`.

## 3. The pinned environment — `docker pull` the container BY DIGEST

The build runs inside a **digest-pinned** container. The **built, layered image
is the source of truth** and is published by digest to GHCR — you `docker pull`
it, you do **not** rebuild `Dockerfile.repro` from apt (the transitive
`musl-tools` apt deps are NOT pinned by the base image digest and would resolve
to whatever the Debian mirror serves that day).

```sh
# From PROVENANCE.<arch>.txt:
CONTAINER_IMAGE=ghcr.io/bg002h/repro-musl@sha256:<BUILT-DIGEST>
docker pull "$CONTAINER_IMAGE"      # no auth needed — the package is PUBLIC
```

> **Maintainer one-time setup — the `repro-musl` GHCR package MUST be Public.**
> GHCR container packages are **private by default**, and an external rebuilder
> pulling by digest does so **without a token** — so the package has to be
> public for this provenance model to work. CI (`reproducible-musl-build.yml` →
> `build-container`) attempts to self-promote it to public via `gh api … -X PATCH
> …/packages/container/repro-musl/visibility -f visibility=public`, but the
> default `GITHUB_TOKEN` often lacks the scope to flip visibility (it is
> `|| true`, never hard-failing the build). If the self-promotion does not take,
> an admin must set it **once** by hand: GitHub → (user/org) → **Packages** →
> `repro-musl` → **Package settings** → Danger Zone → **Change visibility** →
> **Public**. After that one-time flip the package stays public.

- **Base image** (recorded in `Dockerfile.repro`): the official
  `rust:1.85.0` Debian image pinned by index digest
  `sha256:0ff31c9ffa641a62e48d543fb00b4960955ea375f40776f40f585b89e654cc5e`
  (linux/amd64 sub-manifest
  `sha256:16a7f242108de02f10fe4a392991679bafa7694e59f5b40a54d5af1be9b40d03`).
- **Built image** (`<BUILT-DIGEST>`): resolved + pushed by CI
  (`reproducible-musl-build.yml` → `build-container`) and recorded in each
  release's `PROVENANCE.<arch>.txt`.
- **Fallback only — building the container from `Dockerfile.repro`:** pin apt via
  `snapshot.debian.org` at a fixed timestamp + `apt-get install
  musl-tools=<exact-version>` (the recipe is in `Dockerfile.repro`). The canonical
  channel is `docker pull <built-digest>`, not a from-source rebuild.

### The fixed in-container layout

The container fixes `WORKDIR /build/src` and `CARGO_HOME=/cargo`. These fixed
literals are what make the `--remap-path-prefix` from-side a known constant
(`/build/src=/build`, `/cargo=/cargo`) — see §4.

## 4. The exact build command + full env

Run **inside the pinned container**, at the fixed `/build/src`, with
`--network=none` (the build is fully offline against the committed `vendor/`
tree):

```sh
docker run --rm --network=none \
  -v "$PWD":/build/src -w /build/src \
  -e CARGO_HOME=/cargo \
  -e SOURCE_DATE_EPOCH="$SOURCE_DATE_EPOCH" \
  -e LC_ALL=C -e TZ=UTC \
  -e CARGO_BUILD_RUSTFLAGS="--remap-path-prefix=/build/src=/build --remap-path-prefix=/cargo=/cargo" \
  -e CFLAGS="-ffile-prefix-map=/build/src=/build -ffile-prefix-map=/cargo=/cargo" \
  -e CFLAGS_x86_64_unknown_linux_musl="-ffile-prefix-map=/build/src=/build -ffile-prefix-map=/cargo=/cargo" \
  "$CONTAINER_IMAGE" \
  bash -euxo pipefail -c '
    umask 022
    cargo build --locked --offline --release \
      --target x86_64-unknown-linux-musl -p mnemonic-toolkit --bin mnemonic \
      --config '"'"'source.crates-io.replace-with="vendored-sources"'"'"' \
      --config '"'"'source."git+https://github.com/rust-bitcoin/rust-miniscript?rev=95fdd1c5773bd918c574d2225787973f63e16a66".git="https://github.com/rust-bitcoin/rust-miniscript"'"'"' \
      --config '"'"'source."git+https://github.com/rust-bitcoin/rust-miniscript?rev=95fdd1c5773bd918c574d2225787973f63e16a66".rev="95fdd1c5773bd918c574d2225787973f63e16a66"'"'"' \
      --config '"'"'source."git+https://github.com/rust-bitcoin/rust-miniscript?rev=95fdd1c5773bd918c574d2225787973f63e16a66".replace-with="vendored-sources"'"'"' \
      --config '"'"'source.vendored-sources.directory="vendor"'"'"'
    tar --sort=name --owner=0 --group=0 --numeric-owner --mtime="@$SOURCE_DATE_EPOCH" \
      -cf - -C target/x86_64-unknown-linux-musl/release mnemonic \
      | gzip -n -9 > mnemonic-<VER>-x86_64-linux-musl.tar.gz
  '
```

Notes on each load-bearing flag:

- **`--remap-path-prefix` is the *top-level* flag** (NOT `-Cremap-path-prefix`,
  which errors `unknown codegen option` on 1.85.0). It is delivered via the
  `CARGO_BUILD_RUSTFLAGS` **env** at the fixed `/build/src` — NOT a committed
  `.cargo/config.toml` value (a committed config value is passed to rustc
  verbatim with no `$PWD` expansion → it would no-op and give false assurance).
  This remap is the single biggest lever; it removes the absolute build-path
  leak in `.rodata` (a `file!()`/panic-`Location` literal) that makes
  default builds non-reproducible — and closes the `$HOME` privacy leak.
- **`CFLAGS` / `CFLAGS_<triple>` `-ffile-prefix-map`** strips absolute paths from
  the libsecp256k1 objects compiled by `cc-rs` under `musl-gcc`.
- **`SOURCE_DATE_EPOCH`** neutralizes `cc`'s `__DATE__` / `__TIME__`.
- **The three-block job-scoped `[source]` activation is mandatory.** The
  `vendor/` directory is committed but **inert** — there is **no committed
  `.cargo/config.toml [source]` block** (a committed root `[source]` block is
  repo-global via cargo's directory-ancestry config discovery and would bleed
  into every other cargo job). The redirect is activated **job-scoped** on the
  build command via `cargo --config` (stable since 1.63). **All three
  `cargo vendor`-emitted blocks are required** — dropping the
  `source."git+…rev=95fdd1c5…"` git-fork block makes `cargo build --offline`
  **FAIL** (`can't checkout … offline mode`) or reach the live GitHub host,
  because the miniscript `[patch.crates-io]` git dep has its own
  `git+…?rev=…` source key not served by `source.crates-io`. An external
  rebuilder MUST pass the same `--config` overrides (or use an isolated
  `$CARGO_HOME/config.toml` carrying the verbatim `cargo vendor` output).
- **`--locked --offline`** + the committed `vendor/` tree mean the compile
  touches **no live external git host** at build OR vendor time. The miniscript
  fork (`rust-miniscript?rev=95fdd1c5773bd918c574d2225787973f63e16a66`, a
  `[patch.crates-io]` entry at `Cargo.toml:28-29`) is a **named supply-chain
  trust root** materialized into committed `vendor/` — a force-push or deletion
  of that rev upstream cannot break the build.

This is the EXACT command the maintainer's CI runs (re-homed into the same
container — see `man-pages.yml` `musl-binaries` x86_64 leg).

## 5. Expected per-artifact SHA-256 + the provenance tuple

Each release attaches, per arch:

- `mnemonic-<VER>-x86_64-linux-musl.tar.gz` — the static musl binary tarball.
- `SHA256SUMS.x86_64` — its SHA-256.
- `PROVENANCE.x86_64.txt` — the tuple:

  ```
  artifact:          mnemonic-<VER>-x86_64-linux-musl.tar.gz
  sha256:            <hash>
  source_commit:     <full 40-char SHA>
  source_date_epoch: <epoch>
  container_image:   ghcr.io/bg002h/repro-musl@sha256:<BUILT-DIGEST>
  ```

## 6. Compare

```sh
sha256sum -c SHA256SUMS.x86_64        # OK ⇒ your rebuild matches the published artifact
```

On a mismatch, `diffoscope` the two tarballs. **Ignore** `target/.fingerprint`
and `.rustc_info.json` — they are non-reproducible cache artifacts, not part of
the shipped binary or tarball. Also check the gzip header: the mtime field
(offset 4–7) must be **zero** (`gzip -l` shows no stored timestamp) and the OS
byte (offset 9) must equal the pinned `03` (Unix) — a non-`-n` or divergent-gzip
build would ship a different tarball hash even with a byte-identical inner
binary. (CI asserts this via `ci/repro/gzip-residue.sh`.)

## 7. Scope honesty — local installs are NOT reproducible by default

Reproducibility is guaranteed **only** when building **at the fixed `/build/src`
inside the pinned container** with the env above. **`cargo install` /
`install.sh` at an arbitrary `$PWD` are NOT reproducible-by-default** — they
build at a path no static config canonicalizes, so the `.rodata` build-path leak
returns. (A future toolchain bump to Cargo-native `trim-paths`, which
self-canonicalizes, would restore local-install reproducibility; that is a
catalog-only FOLLOWUP — `trim-paths` is nightly-gated on the deliberate 1.85.0
pin.)

## 8. The two-distinct-path self-test (what CI proves)

CI does not just rebuild once — it builds at **two distinct** real paths
(`/build-a/src` and `/build-b/src`, both remapped to `/build`) and asserts the
two binaries + tarballs + libsecp `.o` are byte-identical. **The two-distinct-path
shape is load-bearing:** the `--remap-path-prefix` from-side only *varies* — and
the remap is only *proven effective* — when the two real paths differ. A
same-path A/B would match trivially without exercising the remap, letting it
silently regress and the binary resume leaking `$HOME`. To reproduce this
yourself:

```sh
# Inside the container, materialize the source at two distinct paths and run:
ci/repro/double-build.sh /build-a/src /build-b/src   # binary + tar + libsecp .o identical
ci/repro/cc-validate.sh  /build-a/src                # epoch load-bearing + zero __DATE__/path residue
ci/repro/gzip-residue.sh mnemonic-<VER>-x86_64-linux-musl.tar.gz 03
```

This is the SAME two-path shape the maintainer's CI drift gate (P4) runs against
the published hash release-over-release. A rebuilder confirming *provenance* (not
just *integrity*) should run the two-distinct-path build; a single-path rebuild
only confirms the published hash, not that the remap is doing its job.
