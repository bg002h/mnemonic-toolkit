## Opus ARCHITECT R0 — IMPLEMENTATION_PLAN_reproducible_builds_musl (round 1)

**Verdict: NOT GREEN — 0 Critical / 1 Important / 3 Minor.** Scope: PLAN executability + fidelity to the LOCKED brainstorm (design not re-litigated, per instruction).

### Fidelity to the locked design — PASS
Verified against current source @ `4329859899663a669624efd990d621bef61979a5` (matches the recon basis):
- Root `Cargo.toml:28-29` IS the miniscript `[patch.crates-io]` entry (rev `95fdd1c5…`) — plan's footer and §3.12 citation correct. (Resolves via `Cargo.lock:700`, verified.)
- `man-pages.yml:50` (`tar czf mnemonic-man.tar.gz`, hygiene-only), `:133` (per-arch binary tarball), `:135` (`sha256sum → SHA256SUMS.<arch>`) — all three line numbers exact.
- descriptor-mnemonic slug `reproducible-builds` IS already corrected at `a759c79` ("phase 1 PARTIAL — remap omitted"); its `.cargo/config.toml` carries exactly `[profile.release] codegen-units=1 + strip="symbols"` — P3's "KEEP those, do NOT re-edit status, flip RESOLVED in shipping commit" is faithful (R0-I1).
- Sibling release-workflow filenames (md `man-pages.yml`, ms `man-release.yml`, mk `musl-binaries.yml`) all verified present.
- No existing `Cross.toml`/`Dockerfile.repro`/`.cargo/`/`vendor/` (verified absent) — all NEW, as the plan states.

**Design-constraint carry-through is faithful**: remap via `CARGO_BUILD_RUSTFLAGS` ENV (two top-level `--remap-path-prefix` flags, never committed config, R0-C2/C1); the ONE committed-config exception = the full 3-stanza vendor `[source]` block; `SOURCE_DATE_EPOCH=$(git show -s --format=%ct <SHA>)` off the commit SHA; container-by-digest (BUILT not Dockerfile-rebuild); committed `vendor/` + `--locked --offline`; gzip `-n -9` at BOTH `:50`+`:133`; two-DISTINCT-paths shape load-bearing in P1/P2/P4; P4 remap-EXERCISING distinct-path gate + remap-off negative test; NO-BUMP + no manual/schema_mirror/changelog trip. Gate sufficiency for the cc-under-musl residual (epoch-unset-→-differs probe + `.comment`/`__DATE__`/host-path residue grep) is concrete and adequately de-risks the un-measured musl-cc class the recon flagged. F6 reusable-workflow + per-repo coupling sound; the md remap-add + slug-flip handled correctly.

### The one Important gap
The plan stands up the pinned container and runs the **gate** jobs inside it, but **never specifies the edit that re-homes the actual release-publishing build into that container.** The live `man-pages.yml musl-binaries` job runs on bare `ubuntu-latest` (no `container:` key — grep-verified) with un-pinned host `apt-get install musl-tools`. Unless P1/P3 explicitly move the release build into `@sha256:<BUILT-DIGEST>` at `/build/src`, the published SHA256 is produced by a different toolchain/path than the gate measures — making P4's "leg SHA256 == published SHA256SUMS" assertion either fail or vacuous. The brainstorm §4 mandates the container + fixed `/build/src` for the published-artifact build; the plan must carry that into a concrete release-job edit, not only the gate jobs.

### Minors
Three documentation/scoping gaps (vendor/ explicit-staging note vs the no-`git add -A` rule; the existing `cargo install --locked cross` CLI step disposition; fuzz sub-workspace intentionally-not-vendored note). None block on their own.

**Re-dispatch after folding the Important + the 3 Minors.**