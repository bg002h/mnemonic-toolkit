## R0 (round 3) — `reproducible-builds-musl` IMPLEMENTATION PLAN-DOC

**Verdict: NOT GREEN — 0 Critical / 3 Important / 3 Minor.** The plan faithfully carries the locked brainstorm's load-bearing constraints and the two round-2 folds are correctly applied and live-verified. But three Important executability gaps remain, all clustered around the committed-`vendor/` + `.cargo/config.toml [source]` mechanism the design locked in and around the un-cited sibling-repo workflows.

### What I verified (live, against current source)
- **`man-pages.yml` citations are ACCURATE.** `musl-binaries` is a single job (line 94, `runs-on: ubuntu-latest` line 96); combined `strategy.matrix.include` covers x86_64 (`musl_native`, lines 105-107) + aarch64 (`cross`, lines 108-110); `build-native` 120-124, `build-cross` 125-127, host `apt-get install musl-tools` line 117, `CC_x86_64_unknown_linux_musl` line 123, `package`/`tar czf` line 133, man-`tar czf` line 50, `ensure-release`/`upload` 136-145. The R0-r2-I-matrix-scope and R0-r2-I-network-boundary folds are both correctly grounded.
- **Substrate facts confirmed:** no `.cargo/config.toml`, no `Cross.toml`, no `vendor/`, no `reproducible-musl-build.yml` exist today; miniscript `[patch.crates-io]` at `Cargo.toml:28-29`; single `git+` source in `Cargo.lock:700`; `secp256k1-sys 0.10.1` / `cc 1.2.61`; toolchain `1.85.0`; `vendor/` is NOT a workspace member (members = `crates/mnemonic-toolkit` only) so it won't be traversed by `fmt --all`/`clippy --all-targets`.
- **gzip gate-script byte offsets are technically correct:** `gzip -n -9` zeroes bytes 4-7 (mtime) and writes `0x03` at byte 9 (OS) — exactly the `gzip-residue.sh` assertions.
- **Faithful to the locked design:** ENV-channel remap (not committed config), full three-stanza vendor block, digest-pinned image, `SOURCE_DATE_EPOCH=$(git show -s --format=%ct <SHA>)`, gzip pin at both tar sites, `--locked --offline`, two-DISTINCT-path P1+P4 gate, P4 remap-off negative test, NO-BUMP/no-schema-mirror/no-manual-mirror all carried correctly.

### The blocking gaps (all Important)
1. **Repo-global scope of `.cargo/config.toml [source]` is unaddressed.** Source-replacement is repo-wide — it re-points EVERY cargo invocation (the existing `rust.yml` gnu/macos test matrix, the leading `cargo metadata --locked` guard, `clippy --all-targets`, `bitcoind-differential`, fuzz), none of which pass `--offline`. P0's `repro-substrate` gate proves only the musl `--offline` build resolves; it never re-confirms the pre-existing default CI still GREENs with the committed config. P0 is declared independently shippable but can RED existing CI on the same commit.
2. **`cargo vendor` platform-completeness unspecified.** A bare `cargo vendor vendor/` vendors host resolution; the committed (global) block forces the macos-latest leg + both musl arches + gnu host all through `vendor/`, so the tree must be platform-complete. The plan never names the vendor invocation's target/feature coverage.
3. **P3 sibling workflows assumed, not cited.** The per-step-`docker run`-vs-job-`container:` decision hinges on each sibling's matrix structure, but P3 names filenames/structure ('as the toolkit's does') without a grep-verified citation against the sibling repos — contra the project's citation-discipline.

### Minors
- `<registry>` placeholder undecided (P0 push target + public-pull posture).
- `gzip -l` does not expose the mtime field — `gzip-residue.sh` must use header-byte inspection only.
- Large `vendor/` tree not flagged for explicit staging under the no-`git add -A` rule + a completeness spot-check.

### Path to GREEN
Fold findings 1-3 (add the repo-global-scope statement + the pre-existing-CI-still-green P0 gate; pin the platform-complete vendor invocation; add a P3 sibling-recon-with-live-citations prelude), clear the three Minors, persist this review verbatim, then RE-DISPATCH per the reviewer-loop discipline before any implementation begins.