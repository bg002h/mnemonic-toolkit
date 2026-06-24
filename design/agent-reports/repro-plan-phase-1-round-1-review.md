# P1 Post-Implementation Adversarial Review — reproducible-builds-musl

**Cycle:** reproducible-builds-musl (task #23), Phase 1 (x86_64-musl POC + release re-home).
**Branch:** `feat/repro-p1-x86-poc` off `master` 33c215a5 (the P0 commit).
**Round:** 1 (post-implementation adversarial execution review over the whole P1 diff).
**Verdict:** **GREEN — 0 Critical, 0 Important.** 5 Minors (cosmetic / documentation / optional hardening).

Reviewer scope: all 6 changed/new files (the 3 `ci/repro/*.sh` gate scripts, `reproducible-musl-build.yml`, `man-pages.yml`, `Dockerfile.repro`) + `docs/verify-reproducibility.md`. The config logic was empirically proven on the dev box (3-block `--config` resolves `--offline`; 2-block + drop-`vendored-sources.directory` both fail). Docker/musl-gcc/cross absent locally → container build + byte-identical + cc proofs are CI-only (expected; not flagged).

---

## Critical
None.

## Important
None.

## Minor

**M1 — `cc-validate.sh` epoch-(a) probe is a near-guaranteed "benign no-op" that softens the gate's headline claim.** `cmp -s` of the pinned-epoch `.o` vs the `DIFF_EPOCH` `.o`: because the recipe sets `-ffile-prefix-map` AND libsecp likely embeds no `__DATE__`, the two `.o` will almost certainly be identical → `epoch_loadbearing=0` every run → the gate takes the documented "benign" branch (no-delta + no-residue = PASS). Logically sound (won't false-GREEN a non-reproducible build — (b) and (c) still gate), but the §5 framing implied (a) would *demonstrate* the epoch is honored; as written it usually degrades to "epoch is irrelevant here." Fix: document this is the expected outcome (the binary has no live timestamp). **FOLDED** — added an explicit note to `cc-validate.sh` (a) that the benign branch is the *expected* outcome for a no-live-timestamp binary, and that (b)+(c) are the load-bearing assertions.

**M2 — `double-build.sh` prepends `${CARGO_BUILD_RUSTFLAGS:-}` from the job env, but the gate job deliberately does NOT set it** → expands empty (trailing space; harmless — cargo tokenizes on whitespace). Only the header comment ("inherited from the job env") slightly oversells an intentionally-unused path. Cosmetic. **FOLDED** — comment clarified to "if the caller set one (the gate does not)."

**M3 — `readelf`/`od`/`grep -a` assumed present in the container.** The rust:1.85.0 (buildpack-deps/Debian) image ships binutils+coreutils → present. No action; dependency is real and met.

**M4 — git "dubious ownership" footgun at the epoch-resolution step (`cd src && git show`).** `actions/checkout@v4` configures `safe.directory`; low probability, but if it bites it is a hard RED at epoch resolution. Cheap insurance: `git -C src` or `git config --global --add safe.directory`. **FOLDED** — switched to `git config --global --add safe.directory "$PWD/src"` before the `git show`, and the re-home leg uses `git config --global --add safe.directory "$PWD"` before its `git show`/`git rev-parse`.

**M5 — `double-build.sh` SHA256SUMS uses a single-space separator** vs standard two-space `sha256sum` output. Cosmetic — P4 compares the HEX field; the gate's SHA256SUMS is not the published one. **FOLDED** — emit standard `sha256sum` two-space output directly (drop the `sed` rewrite).

---

## Plan-fidelity verdict (all ✓)
- Deliverable #8 negative isolation (keep 3 git blocks + crates-io.replace-with, drop ONLY `vendored-sources.directory`) — correct; empirically REDs with "could not find a configured source vendored-sources."
- Three-block `--config` copied exact across gate + re-home + verify doc.
- Two-distinct-path shape load-bearing (`/build-a/src`+`/build-b/src`→`/build`; same-path guard present).
- gzip sites: `:50` man tarball (hygiene-only, noted) + `:133`→re-home binary tarball (provenance).
- Re-home: per-step `docker run --network=none` x86_64-only; aarch64 cross leg untouched on host; host `apt-get musl-tools` + `CC_x86_64_unknown_linux_musl` removed; network boundary correct; SOURCE_DATE_EPOCH off COMMIT SHA.
- BUILT-DIGEST: build→push→`imagetools inspect {{.Manifest.Digest}}`→job/workflow output→`needs.repro.outputs.image`; `packages: write` on the `repro` caller with `secrets: inherit`.
- Provenance tuple published (`PROVENANCE.<arch>.txt`).

**Ship-ready for P1's CI-based gate.** All 5 Minors folded (M3 no-op).

---

## Round 2 — post-fold convergence re-review (per CLAUDE.md "reviewer-loop continues after every fold")

Scoped re-review of the 5 fold hunks (M1/M2/M4×2/M5). **GREEN — 0 Critical / 0 Important / 0 Minor.** All folds clean, no drift:
- M4a (`reproducible-musl-build.yml`) — `safe.directory "$PWD/src"` + `git -C src show` correctly target the `path: src` checkout.
- M4b (`man-pages.yml`) — `safe.directory "$PWD"` + host-side git ops preserve the `--network=none` build+package boundary.
- M5 (`double-build.sh`) — subshell `cd` cwd-restored; standard two-space `sha256sum -c`-compatible output.
- M2 (`double-build.sh` comment) — matches `build_leg` logic (inherited RUSTFLAGS appended after per-leg remap).
- M1 (`cc-validate.sh` comment/echo) — disambiguation logic unchanged.

Re-confirmed: `bash -n` passes all 3 scripts; `actionlint` exit 0 both workflows; the three-block `--config` strings byte-unchanged. **Reviewer-loop CONVERGED.**
