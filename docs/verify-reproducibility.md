# Verifying reproducibility of the `mnemonic` musl release binaries

> **Scope (task #23, P1 + P2 + P4).** This document covers the
> **x86_64-unknown-linux-musl** (P1, §1–8) and **aarch64-unknown-linux-musl**
> (P2, §9) `mnemonic` binaries, plus the **continuous drift gate** (P4, §8.1).
> The three codec CLIs `md` / `ms` / `mk` (P3) get their own sections as that
> phase lands. gnu, the GUI, and multi-builder signed attestation are out of
> scope this cycle (catalog-only FOLLOWUPs).

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

## 8.1 The continuous drift gate (P4) — scheduled re-proof + the remap-off negative

Reproducibility is not a one-time property: a runner-image bump, a pinned-container
change, a `cross` image drift, or a toolchain change could silently re-introduce a
build-path leak or a non-deterministic tarball. Two mechanisms keep the recipe
honest **between** releases:

- **Scheduled re-proof — `.github/workflows/repro-drift.yml`.** A weekly (Monday
  06:00 UTC) + on-demand (`workflow_dispatch`) workflow re-runs the whole
  reproducibility gate (`reproducible-musl-build.yml`) against the toolkit's
  current `HEAD`, with **`run_aarch64: true`** so it exercises **both** the x86_64
  **and** the aarch64 gates. This matters because the *release* path
  (`man-pages.yml`) runs `run_aarch64: false` (the ~30-60min QEMU aarch64 build
  would delay every release); without the scheduled gate, an **aarch64-only**
  reproducibility regression could ship unnoticed between releases. The drift gate
  closes that gap — environment / toolchain / base-image drift surfaces as a
  recurring red check rather than at the next release. (Each codec CLI — `md` /
  `ms` / `mk` — can add its own scheduled caller of the same reusable workflow
  later; a catalog FOLLOWUP.)

- **The remap-off negative — `ci/repro/remap-off-negative.sh`.** Byte-identity
  *with* the remap (the positive gate) is necessary but not sufficient: if
  `--remap-path-prefix` ever silently became a no-op (the path leak it guards no
  longer present, or the flag dropped), the positive gate would *still* be green.
  The negative probe runs the same two-distinct-path build with the remap
  **disabled** (empty `CARGO_BUILD_RUSTFLAGS`, no `-ffile-prefix-map`) and asserts
  the protection is **load-bearing**:
  - **x86_64 (`cargo`):** the two no-remap binaries MUST **differ** (each leg's own
    absolute path leaks into `.rodata`). If they are identical even without the
    remap, the remap is hollow → the gate REDs.
  - **aarch64 (`cross`):** `cross` collapses both legs to its fixed `/project`
    mount, so an A/B-difference is structurally unsatisfiable (the same reason A/B
    is weak aarch64 evidence). Instead the no-remap build MUST **leak** `/project`
    (or `$CARGO_HOME`) host-path residue that the remap would strip. Zero residue
    → the remap is hollow → the gate REDs.

  This step runs inside both gate jobs (`repro-x86_64-musl` and
  `repro-aarch64-musl`), so a regression that makes the remap unnecessary fails CI
  loudly — the positive gate's green is only trusted because the negative proves
  the remap is still doing real work (R0-r3-I1). To reproduce it yourself, inside
  the container at the two distinct paths:

  ```sh
  ci/repro/remap-off-negative.sh /build-a/src /build-b/src   # x86_64: no-remap binaries DIFFER
  BUILDER=cross ci/repro/remap-off-negative.sh /build-a/src /build-b/src   # aarch64: no-remap build leaks /project
  ```

### Why the drift gate does NOT explicitly assert `== published SHA256SUMS.<arch>`

The two-distinct-path positive gate (§8) proves the rebuild is byte-identical to
*itself*; it does **not** download the released `SHA256SUMS.<arch>` and assert the
rebuilt `.tar.gz` SHA-256 equals the *published* hash. That explicit comparison is
**intentionally deferred** from the drift gate, for two reasons:

- **A HEAD-scheduled gate has no release peer to compare against.** The scheduled
  re-proof runs against the toolkit's current `HEAD` (`github.sha`), which has no
  published release artifact. A literal `== published` assertion would false-RED on
  every schedule (and on every non-release commit), because there is nothing to
  compare the rebuild to.
- **The `== published` property is already satisfied *structurally* by the release
  re-home.** The P1/P3 release path does not upload a *separately* built artifact and
  hope it matches — it **publishes the exact canonical output of the same
  double-build gate**. Publish and gate therefore share an identical *(commit SHA,
  container digest, build recipe)* tuple, and the recipe is reproducible **by
  construction** ⇒ the published artifact **is** the gate-verified binary. The
  "rebuilt hash == published hash" closed loop holds not because a check asserts it,
  but because the same byte-for-byte recipe produced both.

An *explicit* per-release closed-loop check — a `release:`-published-triggered job
that downloads the just-uploaded `SHA256SUMS.<arch>` and asserts equality against a
**fresh, distinct-path** rebuild — is a worthwhile future addition (it would catch a
hypothetical upload-path corruption that the structural argument assumes away). It is
tracked as catalog FOLLOWUP `repro-explicit-published-hash-gate`.

## 9. aarch64-unknown-linux-musl (P2) — built via `cross` under QEMU

The aarch64 binary has no native runner, so it is built with
[`cross`](https://github.com/cross-rs/cross) under QEMU user-mode emulation.
`cross` ships its **own bundled aarch64-musl C toolchain** inside a runtime
container image — a **different** `musl-gcc` than the x86_64 leg's apt
`musl-tools`. That toolchain's determinism is therefore **re-validated
independently** (the x86_64 cc result does not transfer); CI's
`repro-aarch64-musl` gate proves it.

### 9.1 The pinned cross toolchain — `Cross.toml`, by digest

The cross runtime image is pinned **by sha256** in the committed `Cross.toml`
(not a floating `cross`-version tag) — this is the real aarch64 toolchain pin:

```toml
[target.aarch64-unknown-linux-musl]
image = "ghcr.io/cross-rs/aarch64-unknown-linux-musl@sha256:702154f52b2d8091671aa2c84d5582d849f949977228c735ff8462f93cc0e1e4"
```

(Resolved at adoption time, 2026-06-24, via the GHCR registry manifest API:
`ghcr.io/cross-rs/aarch64-unknown-linux-musl` tag `0.2.5` == `latest` — the
version `cargo install --locked cross` resolves to; the linux/amd64 sub-manifest
of that index is
`sha256:53a761857a806b4f73b209a15bf71eacc38a82d5a02e05b166300c4794d7ad83`.)

`cross` does **not** forward host env into its container automatically, so the
determinism-bearing vars are listed under `Cross.toml [build.env] passthrough`:

```toml
[build.env]
passthrough = [
  "SOURCE_DATE_EPOCH",
  "CFLAGS",
  "CFLAGS_aarch64_unknown_linux_musl",
  "LC_ALL",
  "TZ",
  "CARGO_BUILD_RUSTFLAGS",
]
```

Without this list the cc/rustflags mitigations would silently never reach the
aarch64 compiler.

### 9.2 The remap from-side is `/project`, not the host path

`cross` bind-mounts the project to its **fixed internal path `/project`** inside
the container (cross v0.2.5 `src/docker/local.rs`: `-v <host_root>:/project`),
sets `CARGO_HOME=/cargo`, and `CARGO_TARGET_DIR=/target`. So the in-container
compiler sees the source at `/project`, and the remap from-side is
**`/project=/build`** (plus `/cargo=/cargo`) — **not** the host checkout path.

A corollary: because both legs of a two-path build collapse to the same
`/project` inside the container, **A/B-equality is weak evidence for aarch64**.
The load-bearing aarch64 proof is the **direct** residue + `.comment`
passthrough-effectiveness assertions (`ci/repro/cc-validate.sh`): the libsecp
`.o` `.comment` must carry the expected pinned cross-toolchain producer string
(`GCC:`), and the `.o` + binary must contain **zero** `/project`/host-path or
`__DATE__` residue.

### 9.3 The exact build command (release leg)

Run from the repo root (where `Cross.toml` + `vendor/` are committed):

```sh
SOURCE_DATE_EPOCH=$(git show -s --format=%ct <tagged-commit-SHA>) \
LC_ALL=C TZ=UTC \
CARGO_BUILD_RUSTFLAGS="--remap-path-prefix=/project=/build --remap-path-prefix=/cargo=/cargo" \
CFLAGS="-ffile-prefix-map=/project=/build -ffile-prefix-map=/cargo=/cargo" \
CFLAGS_aarch64_unknown_linux_musl="-ffile-prefix-map=/project=/build -ffile-prefix-map=/cargo=/cargo" \
cross build --locked --offline --release \
  --target aarch64-unknown-linux-musl -p mnemonic-toolkit --bin mnemonic \
  --config 'source.crates-io.replace-with="vendored-sources"' \
  --config 'source."git+https://github.com/rust-bitcoin/rust-miniscript?rev=95fdd1c5773bd918c574d2225787973f63e16a66".git="https://github.com/rust-bitcoin/rust-miniscript"' \
  --config 'source."git+https://github.com/rust-bitcoin/rust-miniscript?rev=95fdd1c5773bd918c574d2225787973f63e16a66".rev="95fdd1c5773bd918c574d2225787973f63e16a66"' \
  --config 'source."git+https://github.com/rust-bitcoin/rust-miniscript?rev=95fdd1c5773bd918c574d2225787973f63e16a66".replace-with="vendored-sources"' \
  --config 'source.vendored-sources.directory="vendor"'

tar --sort=name --owner=0 --group=0 --numeric-owner --mtime="@$SOURCE_DATE_EPOCH" \
  -cf - -C target/aarch64-unknown-linux-musl/release mnemonic \
  | gzip -n -9 > mnemonic-<VER>-aarch64-linux-musl.tar.gz
```

The same three-block `[source]` activation, the same `--locked --offline`, the
same gzip-pinned tar as the x86_64 leg. `cross` forwards the `--config` flags to
the inner cargo; the committed `vendor/` (at `/project/vendor`) makes the build
fully offline.

### 9.4 Provenance tuple

Each release attaches, for aarch64:

- `mnemonic-<VER>-aarch64-linux-musl.tar.gz` — the static musl binary tarball.
- `SHA256SUMS.aarch64` — its SHA-256.
- `PROVENANCE.aarch64.txt`:

  ```
  artifact:          mnemonic-<VER>-aarch64-linux-musl.tar.gz
  sha256:            <hash>
  source_commit:     <full 40-char SHA>
  source_date_epoch: <epoch>
  cross_image:       ghcr.io/cross-rs/aarch64-unknown-linux-musl@sha256:<CROSS-DIGEST>
  ```

The aarch64 provenance tuple cites the **`cross` image digest** (the toolchain
pin) rather than the x86_64 leg's built repro-musl container digest — aarch64 is
built by `cross` in its own digest-pinned image, not in the repro-musl container.

### 9.5 Self-test (what the `repro-aarch64-musl` CI gate runs)

```sh
# Host: register binfmt (qemu) + install cross, then materialize two paths.
docker run --privileged --rm tonistiigi/binfmt --install arm64   # or docker/setup-qemu-action
cargo install --locked cross
BUILDER=cross CROSS_COMMENT_EXPECT='GCC:' \
  SOURCE_DATE_EPOCH=$(git show -s --format=%ct HEAD) \
  ci/repro/double-build.sh /build-a/src /build-b/src   # A/B (weak for aarch64) + libsecp .o
BUILDER=cross CROSS_COMMENT_EXPECT='GCC:' \
  SOURCE_DATE_EPOCH=$(git show -s --format=%ct HEAD) \
  ci/repro/cc-validate.sh /build-a/src                 # PRIMARY: residue + .comment passthrough
ci/repro/gzip-residue.sh mnemonic-<VER>-aarch64-linux-musl.tar.gz 03
qemu-aarch64 target/aarch64-unknown-linux-musl/release/mnemonic --version
```

The aarch64 QEMU build is **slow (~30-60min)** — expected for a gate.
