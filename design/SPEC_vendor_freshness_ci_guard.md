# SPEC — `vendor/` freshness CI guard

**Status:** draft → R0. **Tier:** CI hardening (no runtime/funds-safety surface).
**Source SHA:** master @ `45be1ec1` (post v0.74.0 re-cut).

## 1. Motivation (the bug this prevents)

v0.74.0's reproducible-musl release **failed**: the Word-Card cycle bumped the
codec deps (`md-codec 0.39.1`, `mk-codec 0.4.1`) — updating `Cargo.lock` — but the
committed **`vendor/`** tree was not re-vendored, so it still held `md-codec 0.39.0`
/ `mk-codec 0.4.0`. The release repro build runs `cargo build --locked --offline`
against `vendor/` (3-block source-replacement), which then **could not resolve**
the bumped deps:

```
error: failed to select a version for the requirement `md-codec = "^0.39.1"` (locked to 0.39.1)
location searched: directory source `…/vendor` (which is replacing registry `crates-io`)
```

→ no musl binary published. **It surfaced only at the release tag** (`man-pages.yml`
is tag-triggered) — a *lagging* indicator. Companion failure (`install-pin-check`,
same cycle) is already tag-gated; this spec covers the vendor-tree leg, which has
**no PR-time gate at all**. Same root class as the `gui-schema-mirror` lagging-gate
lesson: the leading discipline must run on the PR, not at ship.

## 2. Goal

A **lightweight, PR-time (+ `main`/`master` push)** CI gate that REDs **iff** the
committed `vendor/` tree cannot satisfy the current `Cargo.lock` under the
reproducible build's `--offline --locked` source-replacement config — i.e. the
exact v0.74.0 failure — so a forgotten re-vendor fails on the **PR**, not at the
next release tag.

## 3. Design

### 3.1 The check  *(R0 round-1: command settled to `cargo metadata` — M1)*
Reuse the reproducible build's source-replacement config (the same 3-block
`--config source."…".replace-with="vendored-sources"` form as
`ci/repro/double-build.sh`, lines ~106–118). Run a **full-workspace, all-target
resolution** (no compile):

```sh
cargo metadata --format-version 1 --locked --offline "${SRC_CONFIG[@]}" >/dev/null
```

- STALE `vendor/` (a `Cargo.lock` entry missing / wrong-version in `vendor/`) ⇒ cargo
  fails at **resolution** ("failed to select a version … directory source vendor")
  ⇒ gate REDs.
- FRESH `vendor/` ⇒ resolution succeeds ⇒ gate GREENs.
- `--offline` ⇒ no network reached; `--locked` ⇒ `Cargo.lock` authoritative.
- **No `--target`, no compile, no musl/Docker/QEMU.** With
  `source.crates-io.replace-with="vendored-sources"` active, cargo's **resolution
  phase validates EVERY `Cargo.lock` entry against `vendor/` regardless of target
  cfg** — R0 round-1 **empirically proved** this (a deleted/`wrong-version`
  `cfg(windows)`-only dep REDs on a gnu Linux host; the exact v0.74.0 md-codec
  staleness REDs at `cargo metadata` in <1s). So there is **no musl-only false
  negative**, and `--target musl` (which would force a toolchain) buys nothing.
  See `design/agent-reports/vendor-freshness-guard-r0-round-1.md`.
- *(Trade-off, not chosen: a host `cargo check --locked --offline <SRC_CONFIG>`
  additionally catches `.cargo-checksum.json` tampering of a right-version crate,
  at +~15s cold. That failure mode is far rarer than a forgotten re-vendor and is
  still caught by the release repro build's checksum verification; the leading PR
  gate optimizes for the real bug class at <1s.)*

`MINISCRIPT_REV` is **derived from `Cargo.lock`** (M2) — the authoritative,
comment-free `source = "git+https://github.com/rust-bitcoin/rust-miniscript?rev=<REV>#…"`
line — NOT from `Cargo.toml` (whose `[patch]` comment prose also contains the rev,
a grep-fragility risk, and which would not disambiguate a future 2nd patched-git
dep). Empty match ⇒ the guard hard-errors (fail-closed: a missing fork pin means
the config would silently drop the git-fork block and mis-resolve).

### 3.2 Why NOT `cargo vendor --locked && git diff --exit-status vendor/`
That catches more (extra/removed vendor files), but: (a) needs **network**;
(b) risks **false positives** from cargo-version / checksum-format differences in
the regenerated tree; (c) flags **harmless** drift (an extra unused vendored crate
does not break the offline build). The offline-resolution check tests **exactly**
the build-breaking property with **zero false positives**. (Documented as the
rejected alternative.)

### 3.3 Files
- `ci/repro/vendor-freshness.sh` — derives `MINISCRIPT_REV` from `Cargo.lock`
  (fail-closed on empty), builds `SRC_CONFIG` (same construction as
  `double-build.sh`), runs the `cargo metadata` resolution, emits a clear
  `::error::` on failure:
  *"vendor/ is out of sync with Cargo.lock — run `cargo vendor vendor/` and commit
  the result (see docs/verify-reproducibility.md)."*
- `.github/workflows/vendor-freshness.yml` — `pull_request` + `push` to
  `[main, master]`; **path-filtered** to `Cargo.lock`, `Cargo.toml`,
  `crates/**/Cargo.toml`, `vendor/**`, `ci/repro/vendor-freshness.sh`,
  `.github/workflows/vendor-freshness.yml` (runs only when deps/vendor could have
  changed). Toolchain pinned to match the repro leg.

### 3.4 Non-goals
- Does **NOT** re-prove bit-for-bit reproducibility (that is `repro-drift.yml` /
  the release `repro` gate, Docker-based). This guard ensures only the
  **necessary precondition** v0.74.0 missed: `vendor/` can satisfy the offline
  build.
- Does not run the musl / Docker / aarch64 path (kept lightweight for PRs).

## 4. Test plan (must pass before commit)
1. **FRESH** (current master): `vendor-freshness.sh` exits 0 (`cargo metadata`
   resolves against `vendor/`).
2. **STALE** (simulate on an isolated copy: restore `vendor/md-codec` to 0.39.0, or
   delete a vendored crate dir): the script exits non-zero with the resolution
   error + the clear `::error::` message. (Restore `vendor/` byte-clean after.)
3. **Rev derivation**: the Cargo.lock grep yields `95fdd1c5…`; an empty result
   hard-errors (fail-closed).
4. **Path filter**: a `Cargo.lock`-only change triggers the workflow; an unrelated
   change does not.

## 5. Companion
Closes the leading-gate gap exposed by the v0.74.0 release-CI post-mortem
(`fix(release): re-vendor … @ 45be1ec1`). FOLLOWUP slug:
`vendor-freshness-pr-gate`.

**Codec-repo exposure (R0 N3).** `md-codec` / `mk-codec` / `ms-codec` each commit
their own `vendor/` tree (125 / 118 / 135 files) consuming the same recipe and have
the **same latent bug with no PR-time guard**. They use the **two-block** form
(no miniscript fork ⇒ `MINISCRIPT_REV=""`), so `vendor-freshness.sh` ports verbatim.
Out of scope for this cycle but a real shared exposure — file catalog FOLLOWUP
`vendor-freshness-pr-gate` with cross-cited companions in each codec's
`design/FOLLOWUPS.md` (per CLAUDE.md cross-repo follow-up convention).
