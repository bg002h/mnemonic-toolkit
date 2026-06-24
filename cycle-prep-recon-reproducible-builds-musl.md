# cycle-prep recon — 2026-06-24 — reproducible-builds-musl

**toolkit origin/master:** `43298598` (`ci(mnemonic-toolkit): register binfmt (qemu) for aarch64 cross test subprocess exec`) | **branch:** `master` | **sync:** in-sync with origin/master (0 ahead / 0 behind) | **untracked:** unrelated cycle-prep recon docs + design drafts for sibling cycles (bsd/musl-target/taproot/cycleX/cycleY) — none touch reproducible-builds machinery.

> **Framing — DEEP new-feature recon.** `reproducible-builds-musl` (task #23) is a NEW feature with **no pre-existing toolkit FOLLOWUP slug** (the only canonical `reproducible-builds` slug lives in `descriptor-mnemonic/design/FOLLOWUPS.md` L513-527, phase-1 RESOLVED md-codec-v0.9.1 / phase-2 OPEN @ v1.0). "Citations" below are therefore the **VERIFIED current build/determinism/external facts** from three recon legs (build-state, empirical determinism-measurement, external-facts), not decayed slug line-numbers.

---

## Per-item verification

### 1. Current build / release pipeline — CONFIRMED (the surface a fix plugs into)

**CONFIRMED.** Toolkit musl binaries are built by job `musl-binaries` in `.github/workflows/man-pages.yml`, firing on `mnemonic-toolkit-v*` tags:
- **x86_64-musl** (L120-124): native `cargo build --release --target x86_64-unknown-linux-musl -p mnemonic-toolkit --bin mnemonic`, env `CC_x86_64_unknown_linux_musl=musl-gcc`, `musl-tools` apt-installed.
- **aarch64-musl** (L125-127): `cross build --release --target aarch64-unknown-linux-musl ...` (Docker container w/ bundled musl C toolchain).
- Toolchain pinned `dtolnay/rust-toolchain@1.85.0` (L113), matching `rust-toolchain.toml` channel `1.85.0`.
- **Checksum step** (L128-145): each matrix job emits a **PER-ARCH** `SHA256SUMS.<arch>` (`sha256sum <tarball> > SHA256SUMS.<arch>`), uploaded `gh release upload --clobber` (per-arch to dodge a cross-job clobber race).

**CONFIRMED — invocation is bare.** No `--locked`, no `RUSTFLAGS`, no `[profile.release]` overrides, no path remap on ANY of the 5 repos' release builds (the only `--locked` usages are `cargo install --locked cross` tool installs + GUI's MSRV `cargo check`). Cargo.lock IS committed in all 5. Workflow comments explicitly state the binaries are "not bit-for-bit reproducible."

**CONFIRMED — sibling parity.** This is a **5-near-identical-copy** surface (toolkit `man-pages.yml`, md `man-pages.yml`, ms `man-release.yml`, mk `musl-binaries.yml`, GUI `build.yml`), NOT a reusable workflow → any repro edit is a 5-site change with no parity gate. GUI is the outlier: `build.yml:113` uses floating `dtolnay/rust-toolchain@stable` and GUI has **no `rust-toolchain.toml`**; GUI also emits ONE combined `SHA256SUMS` in a separate `release` job (`build.yml:171-207`), not per-arch.

### 2. Determinism enemies + the EMPIRICAL double-build verdict (how reproducible are we TODAY) — BLOCKER

**BLOCKER — TODAY THE BINARY IS NON-REPRODUCIBLE ACROSS BUILD PATHS (empirically measured).** Two release builds of `mnemonic` from byte-identical source (HEAD `4329859`, pinned 1.85.0) at two different absolute paths (worktrees `wt-build-A` vs `wt-build-B`) are NOT byte-identical:
- SHA256 A=`2dcb759a...` vs B=`a2996b83...`; `cmp` first differs at **byte 41**; sizes 17202688 vs 17202992 (**304-byte delta**); GNU build-ids differ (A=`5023a729` / B=`11563710`).

> **IMPORTANT — what the double-build actually measured.** The measurement used the **HOST `x86_64-unknown-linux-gnu` target, NOT musl** — the musl C toolchain (`musl-gcc`/`musl-tools`/arch `musl`) is **absent on the recon box** and is REQUIRED by secp256k1-sys's `cc` build (the rustup musl *std* target is installed, but the C cross-compiler is not). So this measures **Rust-level determinism + path leaks**, which transfer to musl, but **musl adds its own un-measured variables** (static musl-libc objects, `cc`-compiled libsecp under musl). A musl double-build MUST be re-run on a musl-equipped box before declaring musl reproducible.

**CONFIRMED — root cause is SINGLE-SOURCE: absolute build-path embedding in `.rodata`.** Via Rust's `file!()`/`#[track_caller]` panic-`Location` string literals: per build, 4 project-source-root occurrences carrying the diverging `wt-build-A/B` substring (the actual inter-build divergence), + 178 `.cargo/registry`, 63 `.cargo/git` (miniscript git pin), 372 `/rustc/<hash>` sysroot paths. The byte-41 differ is `e_shoff` shifting because `.strtab` size differs (symbol-name mangling encodes the crate path). Build-id difference is a **downstream symptom**, not an independent cause (fixing the path leak fixes it transitively — both remapped builds → build-id `5f2627d6`).

**CONFIRMED — CLEAN on every other classic source.** No embedded timestamps (secp256k1-sys's `cc` build embeds none beyond the cargo-registry root); NO local `build.rs` in the toolkit crate (zero build scripts → no env-codegen/time/RNG); deterministic release defaults (no `[profile.release]` for the bin — only `fuzz/Cargo.toml` has one; cgu=16 is reproducible; debug=0 → no DWARF, no `.debug_*` sections). The fix surface is **narrow** — no timestamp/RNG/hashmap-order gremlins to chase on the Rust side.

**CONFIRMED — FIX PROVEN on the pinned 1.85.0, zero toolchain change.** `RUSTFLAGS="--remap-path-prefix=$PWD=/build --remap-path-prefix=$CARGO_HOME=/cargo --remap-path-prefix=$SYSROOT=/rust"` applied to both cross-path builds → **BYTE-IDENTICAL** (both SHA256 `aa5b5fd4...`), all `/home/bcg` + `wt-build-A` leaks eliminated (0 occurrences), identical build-id, binary still runs (`--version` → 0.73.3). `$SYSROOT` remap is a practical no-op (rustc already virtualizes to `/rustc/<HASH>`) but harmless.

**NEEDS-WORK — strip is an add-on, not a fix.** `strip` alone does NOT achieve repro (rodata panic-paths survive; stripped A vs B differ at byte 977). `remap+strip` is byte-identical AND ~1.8MB smaller (15.39MB vs 17.20MB; binary currently ships NOT-stripped, ~470KB symbols incl. mangled crate paths). If a fix adds `strip`, pair it with remap, never instead.

### 3. The must-pin input list — CONFIRMED (this IS the checklist)

**CONFIRMED (external-facts leg, cited to reproducible-builds.org/docs/rust + rustc #129080 + Cargo #5505 + Tor blog).** Complete pin set for bit-for-bit Rust repro: (1) exact toolchain version (rust-toolchain.toml/RUSTUP_TOOLCHAIN); (2) target triple; (3) Cargo.lock committed + **`--locked`**; (4) RUSTFLAGS incl. remap; (5) profile (trim-paths, debuginfo, strip, codegen-units, lto, panic); (6) fixed build `$PWD` + `$CARGO_HOME` remapped; (7) registry/cache location; (8) `LC_ALL=C`; (9) `TZ=UTC`; (10) umask; (11) C toolchain (cc/clang) version + CFLAGS + `SOURCE_DATE_EPOCH` for any cc-compiled dep. `codegen-units=1` removes parallel-codegen ordering nondeterminism (if a future profile relaxes it, also `-Zthreads=1`). Ignore `target/.fingerprint`, `.rustc_info.json` when diffing.

**Status vs toolkit today:** `--locked` = NEEDS-WORK (absent); `[profile.release]` cgu=1/strip = NEEDS-WORK (absent in toolkit/GUI/ms/mk — only descriptor-mnemonic has them via `.cargo/config.toml`); remap = BLOCKER (absent everywhere, incl. descriptor-mnemonic whose FOLLOWUP *claims* `--remap-path-prefix` but the shipped config OMITS it → **path-leak open in ALL 5 repos** despite the slug reading RESOLVED); `LC_ALL`/`TZ`/`umask`/`SOURCE_DATE_EPOCH` = NEEDS-WORK (none set in CI).

### 4. Hermetic-env options — CONFIRMED (decision input, no incumbent)

**CONFIRMED.** Three realistic approaches: (1) **digest-pinned `rust:<ver>` Docker** + fixed env (LC_ALL/TZ/SOURCE_DATE_EPOCH/build-path) — lowest effort, common cargo-dist/CI pattern, image MUST be pinned by sha256 not tag; (2) **Nix flake** — content-addressed pin of toolchain+C-toolchain+deps, strongest hermeticity for moderate cost; (3) **Guix** — what Bitcoin Core uses, highest assurance/complexity, overkill for 5 small crates. Tor historically used Gitian (legacy Ubuntu-VM wrapper). **Recommendation (external-facts leg): digest-pinned `rust:<ver>`-on-musl Docker** is the pragmatic sweet spot — matches the existing musl-static pipeline with minimal new infra; Nix = upgrade path; Guix = disproportionate. **NEEDS-WORK** — no hermetic env exists today (release builds are local + CI has no SOURCE_DATE_EPOCH/remap/build-id pinning/double-build gate; `install.sh` builds via `cargo install` so it WOULD inherit a `.cargo/config.toml` fix).

### 5. cargo trim-paths / SOURCE_DATE_EPOCH mechanism — CONFIRMED with a HARD CONSTRAINT

**CONFIRMED + BLOCKER-on-current-pin.** `trim-paths` is a Cargo PROFILE option (RFC 3127, stabilized ~Rust **1.79**, PR #13608 — one source said 1.80; the underlying rustc `-Cremap-path-scope` only *fully* stabilized 1.95 / Cargo #16536). It **defaults to `object` for the RELEASE profile** → modern toolchains trim binary paths for free; it wraps rustc's `--remap-path-scope=object` (alias for macro,coverage,debuginfo) + `--remap-path-prefix` for sysroot/registry/local — coarser & safer than raw textual `--remap-path-prefix` (last-match-wins, no separator awareness).

> **HARD CONSTRAINT (empirical):** the ergonomic `--config 'profile.release.trim-paths=true'` **FAILS on the pinned Cargo 1.85.0** — it trips the unstable gate ("Consider trying a newer version of Cargo ... nightly") and emits NO binary. So on the deliberate 1.85.0 pin the portable fix is **`-Cremap-path-prefix` via `.cargo/config.toml [build] rustflags`** (proven byte-identical), NOT `trim-paths`, unless the build toolchain is bumped. A naive "just add trim-paths to the profile" plan silently fails to build.

**CONFIRMED — SOURCE_DATE_EPOCH scope.** rustc/Cargo embed NO wall-clock timestamps into normal output → a bare Rust binary needs no timestamp control. SOURCE_DATE_EPOCH matters ONLY for (a) build-metadata crates (vergen/`built`/git-embed/rust-embed mtimes) and (b) the `cc` crate's C compilation (GCC honors it for `__DATE__`/`__TIME__`). Cargo passes it through to build.rs. **NEEDS-WORK** — AUDIT the dep tree for vergen/`built`/rust-embed (toolkit recon found none in the toolkit crate, but the bitcoin stack could pull one transitively); secp256k1-sys's `cc` build is the live SOURCE_DATE_EPOCH consumer.

---

## Cross-cutting observations

- **How far off are we today?** One mechanically-fixable lever away on the Rust side, plus an un-measured musl C-toolchain frontier. The empirical verdict: NON-reproducible across build paths today (two builders cannot confirm they produced the same `mnemonic`), root cause SINGLE-SOURCE (absolute build-path leak in `.rodata` panic-Location strings), and **the fix is PROVEN byte-identical on the current 1.85.0 pin** via `-Cremap-path-prefix`. Everything else (timestamps/RNG/codegen/DWARF) is already clean. The **single biggest lever = path-remap** (`$PWD` + `$CARGO_HOME`); it also closes a **privacy leak** (the maintainer's `$HOME`/exact build dir is currently baked into every shipped binary).

- **Load-bearing fact — trim-paths stabilization vs the pin.** `trim-paths=object` is the clean Cargo-native fix and is the release-profile default on ≥1.79 — but it is **nightly-gated on the pinned Cargo 1.85.0** (empirically build-fails). The 1.85.0 deliberate pin therefore forces the `-Cremap-path-prefix` form (or a toolchain bump). This is the design's central constraint.

- **Load-bearing fact — secp256k1-sys / `cc` is the residual showstopper.** The only C dep is vendored libsecp256k1 in `secp256k1-sys 0.10.1`, compiled by `cc-rs` (cc 1.2.61 toolkit / 1.2.62 GUI — version drift becomes a per-repo lockfile-pin concern once `--locked` lands). `cc` shells out to the C compiler, which can embed `__DATE__`/`__TIME__` + absolute OUT_DIR/CARGO_MANIFEST_DIR/`-I` paths. **x86_64-musl uses host `musl-gcc`; aarch64-musl uses cross's bundled musl toolchain → DIFFERENT, separately-versioned C compilers → libsecp object determinism MUST be validated per-arch**, and aarch64 additionally needs the cross container image **digest** pinned (`cargo install --locked cross` pins the CLI, not the runtime image). Mitigation: `SOURCE_DATE_EPOCH` + CFLAGS `-ffile-prefix-map` + pinned C compiler; validate with a two-different-paths diffoscope/double-build test.

- **Load-bearing fact — musl-as-an-asset.** Static-musl HELPS repro: a fully-static `*-unknown-linux-musl` binary has no host-glibc dependency, eliminating a major cross-machine variance source. No musl-specific linker nondeterminism in authoritative sources (the `.pdb`/OSO caveats are windows-msvc/macOS only). So the toolkit's existing static-musl choice (task #20) is repro-FRIENDLY; pin the musl triple + the linker (lld vs GNU-ld is part of toolchain identity).

- **Load-bearing fact — verification model.** reproducible-builds.org / Tor / Bitcoin Core converge on: publish (a) source@exact-tag, (b) pinned env/toolchain/build-script-or-container, (c) expected per-artifact SHA-256/512; an independent rebuilder byte-compares and fails loudly on drift. The toolkit already emits per-arch `SHA256SUMS.<arch>` (codec/toolkit) / one combined `SHA256SUMS` (GUI) at the package step — the **exact attach point** for a build-twice-and-diff CI gate + a published verify-reproducibility recipe. Multi-signer attestation is future-hardening, not a v1 blocker. **NB the recipe must be written twice** (per-arch codec/toolkit vs combined GUI).

- **Plan-of-record gap.** The canonical `reproducible-builds` slug is descriptor-mnemonic-scoped + phase-2-deferred-to-v1.0; **NO equivalent slug in toolkit/GUI/ms/mk**. Extending repro means authoring companion FOLLOWUP entries (cross-citing per CLAUDE.md) and replicating phase-1 flags to the 4 lacking repos. **Also: re-verify descriptor-mnemonic's `.cargo/config.toml` — its FOLLOWUP claims `--remap-path-prefix` but the shipped config omits it, so trusting "phase-1 RESOLVED" would leave path-leak open even there.**

---

## Recommended brainstorm-session scope

**POC-first plan (recommended ordering):**

1. **POC — x86_64-musl `mnemonic` reproducible in a pinned container.** On a musl-equipped box / digest-pinned `rust:1.85.0`-musl image: wire `-Cremap-path-prefix` (`$PWD`→`/build`, `$CARGO_HOME`→`/cargo`) via `.cargo/config.toml [build] rustflags` (covers `cargo build`/`cargo install`/CI uniformly) + `--locked` + `SOURCE_DATE_EPOCH=<release-commit %ct>` + `LC_ALL=C TZ=UTC` + CFLAGS `-ffile-prefix-map` for the cc/secp256k1-sys build. **Build twice in two different paths → diffoscope → prove byte-identical.** Emit + publish the expected SHA-256 + author a `verify-reproducibility` doc (source@tag + exact build cmd/env + expected hash).
2. **Extend to aarch64-musl** — re-run the double-build under `cross` with a **digest-pinned** container image; validate libsecp determinism independently (different C toolchain).
3. **Extend to the 3 codec CLIs** (`md`/`ms`/`mk`) — replicate the same `.cargo/config.toml` + CI env to their near-identical musl jobs; companion FOLLOWUP entries cross-cited per CLAUDE.md.
4. **(Decision-gated) GUI + gnu builds** — see forks.

> **Be explicit about what the double-build measured:** the proven byte-identical result is on the **host gnu target** (musl C toolchain absent on the recon box). The Rust-level path-leak + remap fix transfers to musl, but **step 1's first action is to RE-RUN the double-build under actual musl** before claiming musl repro — the static musl-libc + cc-under-musl objects are un-measured.

**Rough sizing:** POC (step 1) = small-to-moderate (additive `.cargo/config.toml` + a CI double-build-and-diff job + a docs page; the fix recipe is proven). Steps 2-3 = moderate (per-repo replication × 3, the 5-copy no-parity-gate risk). Step 4 (GUI) = larger (NEW `rust-toolchain.toml` + build-job pin, MSRV ≥1.88 reconciliation). **CI-time cost:** `codegen-units=1` adds measurable build time (~+30s on descriptor-mnemonic; larger on toolkit's patched-miniscript git fork + bigger graph) — acceptable for tag-only release jobs.

**SemVer:** **likely NO-BUMP** — CI/build-infra + a docs add, no CLI-surface change. **If any `mnemonic`/CLI flag is added** (none anticipated), the manual (`docs/manual/src/40-cli-reference/`) + GUI `schema_mirror` lockstep fire per CLAUDE.md — but a pure infra+SHA256SUMS+docs cycle does NOT trip the clap-flag-name mirror gates.

**KEY DECISION FORKS to settle in brainstorm:**
- **Hermetic env:** digest-pinned `rust:<ver>` **Docker** (recommended; matches existing musl pipeline) vs **Nix** flake (stronger, more setup) vs **Guix** (Bitcoin-Core-grade, overkill).
- **Scope:** CLIs-only (toolkit + 3 codecs) vs **GUI too** (GUI is the biggest hole — floating `@stable`, no `rust-toolchain.toml`, MSRV 1.88 floor, combined-SHA256SUMS shape; needs NEW files, not just a flag tweak).
- **Target scope:** musl-only vs **also make the existing gnu builds reproducible** (the empirical proof is *on* gnu; remap fixes both, low marginal cost).
- **Trim-paths vs remap:** stay on 1.85.0 + `-Cremap-path-prefix` (proven today) vs **bump the build toolchain** to one where `trim-paths=object` is non-nightly (cleaner, but breaks the deliberate pin).
- **Verification posture:** publish-the-recipe-only vs a **CI rebuild-and-compare gate** vs a **public multi-builder attest** flow (Tor-style signed `sha256sums.txt`).

**Lockstep:** companion FOLLOWUP entries in all touched repos (cross-citing per CLAUDE.md); the 5-copy musl-job surface has **no parity gate** → any shared change risks missing a repo (consider whether a brainstorm output is a reusable workflow or a parity lint). Re-verify descriptor-mnemonic's actual config (FOLLOWUP vs shipped drift). **Ordering:** POC x86_64-musl → aarch64-musl → 3 codecs → (decision) GUI / gnu; settle the hermetic-env + GUI-scope forks BEFORE the POC since they shape the container + file layout.

**R0 gate reminder:** brainstorm spec → mandatory opus R0 loop to GREEN (0C/0I) BEFORE any implementation; per CLAUDE.md no code before GREEN.
