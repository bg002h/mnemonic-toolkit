## R0 ARCHITECT REVIEW (round 3) — `reproducible-builds-musl` brainstorm spec

**Verdict: GREEN-pending — 0 Critical / 2 Important.** Not yet GREEN; fold the 2 Important findings and re-dispatch (reviewer-loop continues after every fold).

### Recon-basis + live-source verification (all PASS)
I re-grepped every load-bearing live fact the spec cites against current `origin/master` (toolkit @ `43298598`, confirmed HEAD):
- `tar czf` sites at `man-pages.yml:50` and `:133`, `sha256sum > SHA256SUMS.<arch>` at `:135` — **confirmed verbatim**. The published asset is a `.tar.GZ` (R0-r2-I1 premise holds).
- miniscript git fork `rev = "95fdd1c5773bd918c574d2225787973f63e16a66"` at `Cargo.toml:29` and `Cargo.lock:700` — **confirmed LIVE git dep** (R0-r2-I3b premise holds).
- `cc 1.2.61` in toolkit `Cargo.lock:300` — **confirmed**.
- Toolkit has **no** `.cargo/config.toml`, **no** `Cross.toml`, **no** `build.rs` in the crate; `rust-toolchain.toml` pins `1.85.0`; release job runs bare `cargo build`/`cross build` with no `--locked`/RUSTFLAGS and `cargo install --locked cross` (CLI, not image) — **all confirmed**, matching the recon and the spec's "must-pin" gap analysis.
- descriptor-mnemonic slug `reproducible-builds` (L513–527) — status **already corrected** at the prose level ("⚠ phase 1 PARTIAL — status corrected 2026-06-24"), explicitly states `--remap-path-prefix` OMITTED, cross-cites the toolkit cycle-prep; the shipped `.cargo/config.toml` carries ONLY `codegen-units=1` + `strip="symbols"` — **R0-I1 fold is accurate**; do NOT re-edit the status.

The two prior rounds' folds (C1 flag-form, C2 config-channel; r2-I1 gzip, r2-I2 epoch-on-retag, r2-I3 supply-chain) are each correctly reasoned and live-verified. The spec is materially sound.

### (a) Recipe completeness/correctness — SOUND
Top-level `--remap-path-prefix` (not `-C`) is the correct form for the 1.85.0 pin (trim-paths is nightly-gated → build-fails, per recon §5); `--locked`, `LC_ALL=C`/`TZ=UTC`/`umask 022`/`SOURCE_DATE_EPOCH`, fixed `/build/src`+`/cargo` with ENV-channel remap, and the gzip-pinned tar are all present and consistent with the recon's PROVEN byte-identical result. The F4 channel reasoning (config.toml is verbatim → cannot carry `$PWD` → remap must live in CI env at the fixed literal path) is correct and the honest "local installs NOT reproducible-by-default" caveat is the right call.

### (b) secp256k1-sys / cc-under-musl residual — PROPERLY DE-RISKED (this was the un-measured unknown)
The §5 per-arch double-build/diffoscope is correctly positioned as the FIRST gate before any "musl reproducible" claim, with the right drill-downs: isolate the libsecp `.o`, the SOURCE_DATE_EPOCH load-bearing probe (build with epoch unset → assert `.o` DIFFERS → proves it is honored), the `__DATE__`/`__TIME__` residue grep, and — crucially — the recognition that `cross` A/B-equality is WEAKER evidence (shared internal paths) so aarch64 needs direct residue/`.comment` assertions. The aarch64 `cross` **runtime-image-by-digest** pin (not `cargo install --locked cross`) and the `Cross.toml [build.env] passthrough` list (without which the cc mitigations silently never reach the container) are both correct and were real gaps caught in rounds 1–2.

### (c) Hermetic container — SOUND
Digest-pin (not floating tag), carries the musl C toolchain, and the R0-r2-I3a resolution (distribute the BUILT layered image BY DIGEST as source-of-truth rather than rebuild `Dockerfile.repro` from a mirror-of-the-day apt, with snapshot.debian.org + `musl-tools=<exact-version>` as the Dockerfile-path fallback) correctly closes the apt-layer non-determinism.

### (d) POC-first phasing + NO-BUMP — SOUND
P1 (x86_64) → P2 (aarch64) → P3 (3 codecs) → P4 (CI gate) is each independently shippable; NO-BUMP is correctly justified (CI/build-infra + docs; no CLI-surface/clap-flag/wire-shape change → does not trip the manual mirror or GUI `schema_mirror`; `changelog-check` fires on the tag and this is no-tag).

### Outstanding (fold before GREEN)
1. **[Important] P4 gate must build at a DIFFERENT path**, else a same-container same-path rebuild proves only container determinism and the remap can silently regress (re-leaking `$HOME`) between releases — the exact stale-slug false-assurance the spec warns against.
2. **[Important] Resolve the vendoring fork in the spec body.** Only the committed-`vendor/` + committed `[source]`-replacement option actually removes the live github.com trust root (R0-r2-I3b's stated goal); a CI-time vendor merely moves the fetch from compile-time to vendor-time. Also admit the `[source]`-replacement stanza as the one verbatim-safe committed-config exception to F4.

The 4 Minor findings (rustflags-override lint, gzip OS-byte, tag→commit derivation in the verify doc, F6 concentration-of-trust) are polish — fold opportunistically.

**Re-dispatch after folding the 2 Important findings.**