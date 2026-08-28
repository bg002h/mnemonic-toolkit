# SPEC — `vendor/` freshness CI guard

**Status:** SHIPPED. Originally a one-check design (R0'd, merged as
`1dcf185e`); rewritten to a **four-check** design by F-354
(`926f6d2a` fix, `2c4510c0` gate rewrite, merged at `1f333ad2`). This revision
(F-381) brings the spec up to the shipped four-check gate — the prior version
still described the one-check design and mentioned none of the other three —
and adds an **executed** re-grounding rehearsal (§5); previously that recovery
procedure existed only as a comment in the script it governs.
**Tier:** CI hardening, but guards a **funds-relevant** artifact: the tree it
checks is what the reproducible **release** binary compiles from, and F-354
was a real, shipped defect in that binary (see §1.1).
**Source SHA (original R0):** master @ `45be1ec1` (post v0.74.0 re-cut).
**Source SHA (this revision):** worktree @ `1f333ad2`, script measured at 247
lines (`ci/repro/vendor-freshness.sh`).

## 1. Motivation (the bugs this prevents)

### 1.1 v0.74.0 — the original bug (why check (1) exists)
The Word-Card cycle bumped the codec deps (`md-codec 0.39.1`, `mk-codec 0.4.1`)
— updating `Cargo.lock` — but the committed **`vendor/`** tree was not
re-vendored, so it still held `md-codec 0.39.0` / `mk-codec 0.4.0`. The release
repro build runs `cargo build --locked --offline` against `vendor/` (3-block
source-replacement), which then **could not resolve** the bumped deps:

```
error: failed to select a version for the requirement `md-codec = "^0.39.1"` (locked to 0.39.1)
location searched: directory source `…/vendor` (which is replacing registry `crates-io`)
```

→ no musl binary published. **It surfaced only at the release tag**
(`man-pages.yml` is tag-triggered) — a *lagging* indicator. This spec's
original scope was exactly that gap: a **leading**, PR-time gate for the
vendor-tree leg.

### 1.2 F-354 — the bug that check (1) alone COULD NOT see
`vendor/miniscript` sat at rev `95fdd1c5` while `Cargo.toml`/`Cargo.lock`
pinned `ff4732e5` (rust-miniscript PR #953, "descriptor: fix Taproot tree
descriptor formatting"), **undetected for two months** (pre-existing since
`33c215a5`, 2026-06-24). Every networked build fetched the correct rev from
git and was fine; only the `--offline` vendored path — what the reproducible
**release** binary is built from — compiled the old formatter. A valid
depth ≥ 2 taproot backup became unrestorable by the release binary
(`left_heavy_3leaf_tr_restores_depth2` failed against the old tree, passed
against the fixed one; the discriminating cell — a right-spine tree restored
correctly on *both* pins, exactly as its own test comment predicted, so this
was not a blanket taproot break) while the freshness gate reported green the
entire time.

**Why check (1) was blind, and it is a structural blindness, not a tuning
gap.** Check (1) — `cargo metadata` resolution — validates `Cargo.lock`
against `vendor/` by **name, version and source-id only**. In the F-354 tree,
all three matched: same crate name, same declared `version = "13.0.0"`, same
git source-id string. Resolution had nothing to object to. **Check (1) never
reads the vendored bytes at all.**

**Why a checksum-only fix would ALSO have missed it — measured, not assumed.**
The mis-vendored tree was internally **self-consistent**: `cargo vendor`
writes each crate's `.cargo-checksum.json` from the same tree it just wrote,
so the checksums it emits always match the files on disk regardless of which
rev was vendored. On the defective tree this reported **168 crates, 7479
files, 0 checksum mismatches**. Integrity-vs-disk catches corruption and
hand-edits; it cannot catch "vendored correctly, from the **wrong** source." A
git-source provenance anchor — comparing the vendored tree against upstream,
not against its own manifest — is the only check that closes this, which is
why checks (3)/(4) exist and why (2) alone would not have sufficed.

## 2. Goal

A **lightweight, PR-time (+ `main`/`master` push)** CI gate that REDs iff
either:
1. the committed `vendor/` tree cannot satisfy the current `Cargo.lock` under
   the reproducible build's `--offline --locked` source-replacement config
   (the v0.74.0 failure class), **or**
2. the vendored tree's **content** is not what it claims to be — corrupted,
   hand-edited, or (for the one git-fork dependency) vendored from a
   revision other than the one `Cargo.lock` pins (the F-354 failure class,
   which (1) structurally cannot detect) —

so both a forgotten re-vendor and a wrong-revision re-vendor fail on the
**PR**, not at the next release tag, and not silently for two months.

## 3. Design

### 3.1 The four checks
`ci/repro/vendor-freshness.sh` runs all four, in ascending cost; **any one RED
fails the gate**. Measured on the current tree: (1) is a `cargo metadata`
call; (2)–(4) are a single Python pass that inspects every vendored file,
measured at ~0.2s for 169 crates / 7490 files — nothing is sampled.

**(1) RESOLUTION** *(the original, R0-round-1 design — unchanged)*

```sh
cargo metadata --format-version 1 --locked --offline "${SRC_CONFIG[@]}" >/dev/null
```
reusing the reproducible build's 3-block source-replacement config (same form
as `ci/repro/double-build.sh`). `MINISCRIPT_REV` is derived from `Cargo.lock`
(the authoritative, comment-free `rev=<40-hex>` source line), never from
`Cargo.toml`'s comment prose; an empty match hard-errors (fail-closed).
- **Asserts:** `vendor/` contains, for every `Cargo.lock` entry, a crate of
  the right **name and version** — i.e. the offline build can resolve at all.
- **Cannot see:** whether the vendored *content* is what the pinned source
  actually contains. Resolution is silent about bytes. This is exactly the
  F-354 gap (§1.2).
- **Catches:** the v0.74.0 class (missing/wrong-version crate) directly, in
  <1s, with no `--target`, no compile — R0-round-1 proved a
  `cfg(windows)`-only dependency still REDs on a gnu Linux host, so there is
  no target-conditioned false negative. See
  `design/agent-reports/vendor-freshness-guard-r0-round-1.md`.

**(2) INTEGRITY** — every vendored file's sha256 vs the digest recorded for it
in its crate's own `.cargo-checksum.json`.
- **Asserts:** the tree on disk matches what `cargo vendor` wrote at
  vendoring time — catches corruption and hand-edits after the fact.
- **Cannot see:** whether what `cargo vendor` wrote came from the pinned
  rev in the first place. A tree vendored wholesale from the wrong revision
  is, by construction, self-consistent with its own manifest — this is the
  measured F-354 counter-example in §1.2 (168 crates / 7479 files / 0
  mismatches on the defective tree). Integrity is a **tamper** check, not a
  **provenance** check.
- **Catches:** post-vendor corruption/edits; nothing about source correctness.

**(3) REGISTRY PROVENANCE** — for every crates.io-sourced crate, the
`package` digest in its `.cargo-checksum.json` (the sha256 of the published
`.crate` tarball) must equal the `checksum` `Cargo.lock` pins for that
`(name, version)`; separately, the **set** of vendored crates lacking such an
anchor must be exactly the one this gate is grounded for (currently: exactly
`{miniscript}`).
- **Asserts:** every registry crate's vendored content is provably the
  tarball crates.io published for that exact version, **and** no new
  unanchored (git/path) source has appeared without the gate knowing about it.
- **Honest scope note, measured:** the digest *comparison* itself is
  redundant with cargo — tampering a `package` digest REDs in check (1)
  already (`checksum for 'bitcoin v0.32.8' changed between lock files`),
  because cargo validates it during resolution (mutation M5, §5 methodology
  reused from the gate's own mutation suite). What is **not** redundant, and
  the reason (3) stays, is the **unanchored-set assertion**: cargo is
  perfectly happy to resolve a new git or path dependency that no digest can
  vouch for, and that is the shape F-354 arrived in — a dependency the
  registry-anchor mechanism cannot reach at all.
- **Cannot see:** anything about a git/path source — by definition, those
  have no published tarball to compare against, which is exactly why (4)
  exists.
- **Catches:** a tampered registry-crate digest (redundantly, via (1)); an
  unexpected new unanchored source (uniquely — cargo alone does not flag
  this).

**(4) GIT-FORK PROVENANCE** — the one hole (3) cannot reach: a git source has
no published tarball, so both `package` and `Cargo.lock`'s `checksum` are
null for it. Anchored instead against a digest a human grounded by hand:

```sh
EXPECTED_GIT_FORK_REV="ff4732e5f75aa555682343cb180fa72ee3e8e9d5"
EXPECTED_GIT_FORK_MANIFEST_SHA256="30cc80f5ea57305f09790b661805b58cfdcd16aaaddd26c3769078eccd9a1277"
```
Two comparisons: `Cargo.lock`'s derived rev must equal
`EXPECTED_GIT_FORK_REV` (else: "the pin MOVED ... re-ground"), and
`sha256(vendor/miniscript/.cargo-checksum.json)` must equal
`EXPECTED_GIT_FORK_MANIFEST_SHA256` (else: "GIT-FORK PROVENANCE MISMATCH").
- **Asserts:** the vendored miniscript tree is **byte-for-byte** the tree a
  human verified against `github.com/rust-bitcoin/rust-miniscript` at the
  pinned rev, at the time of grounding, and has not changed since.
- **Cannot see, and this is the central limitation of the whole gate (§4):**
  whether the grounding itself was correct. It is **trust-on-first-use** —
  it proves the tree has not *changed* since a human checked it once; it
  cannot by itself prove that check was right. See §5 for the recovery
  procedure this implies, executed end to end.
- **Catches:** exactly the F-354 shape — a wrong-rev vendor that is
  self-consistent with its own manifest and resolves cleanly. This is the
  **only** check of the four that reaches this defect class (§1.2 confirmed
  this by direct measurement on the historical defective tree: (1) green,
  (2) green, (4) is where it RODE red).

### 3.2 Why NOT `cargo vendor --locked && git diff --exit-status vendor/`
That catches more (extra/removed vendor files), but: (a) needs **network**;
(b) risks **false positives** from cargo-version / checksum-format differences
in the regenerated tree; (c) flags **harmless** drift (an extra unused
vendored crate does not break the offline build). The offline checks above
test **exactly** the build-breaking and provenance-breaking properties, fully
offline, with (by the gate's own mutation suite, §5) zero observed false
positives. (Documented as the rejected alternative; unchanged by F-354.)

### 3.3 Files
- `ci/repro/vendor-freshness.sh` (247 lines) — derives `MINISCRIPT_REV` from
  `Cargo.lock` (fail-closed on empty), builds `SRC_CONFIG` (same construction
  as `double-build.sh`), runs check (1) via `cargo metadata`, then hands off
  to an embedded Python block (`python3 - <<'PY' ... PY`) that runs checks
  (2)–(4) in a single pass over `vendor/*/.cargo-checksum.json`, printing a
  `(n/4)` progress line per check and a consolidated `::error::` listing every
  content defect found on failure.
- `.github/workflows/vendor-freshness.yml` — `pull_request` + `push` to
  `[main, master]`; path-filtered to `Cargo.lock`, `Cargo.toml`,
  `crates/**/Cargo.toml`, `vendor/**`, `ci/repro/vendor-freshness.sh`,
  `.github/workflows/vendor-freshness.yml`. Unaffected by F-354 — the CI
  surface didn't need to change, only the script's internals.

### 3.4 Non-goals
- Does **NOT** re-prove bit-for-bit reproducibility (that stays with
  `repro-drift.yml`, scheduled, and the release `repro` gate, tag-triggered,
  Docker-based). This guard ensures only that `vendor/` can satisfy the
  offline build **and** is provably the content it claims to be — not that
  the resulting binary is bit-identical across builders.
- Does not run the musl / Docker / aarch64 path (kept lightweight for PRs).
- Does not verify that the **grounding itself** was correct — see §4.

## 4. Blind spots — read before trusting a green

Stated here, not only in the script, because a spec a reviewer reads to learn
what the gate guarantees is where a false sense of completeness would do the
most damage.

- **Trust-on-first-use (check 4).** The git-fork anchor proves the vendored
  tree has not *changed* since a human verified it against upstream once. It
  cannot, by construction, prove that verification was done correctly, or at
  all. A wrong grounding — pasted from the wrong rev, or never actually
  diffed against upstream — would pass silently forever, because nothing
  downstream re-derives it. The gate's only defense against a wrong grounding
  is the human step at grounding time; see §5 for what that step must
  actually do.
- **Checksums prove disk matches the manifest, not that the manifest matches
  the pin.** Checks (2) and (3)'s digest comparisons both terminate at
  `.cargo-checksum.json` / `Cargo.lock` — internally consistent artifacts
  that `cargo vendor` can write correctly from the *wrong* source (§1.2).
  For crates.io crates this gap is closed because the registry publishes an
  independent tarball digest cargo itself validates (check 1, restated in
  check 3). For the git fork there is no equivalent independent digest —
  which is precisely why check (4) exists, and precisely why it is the
  weakest of the four (trust-on-first-use, not live verification).
- **No bit-for-bit reproducibility proof.** All four checks are offline,
  no-compile content checks. None of them re-derives that the vendored source
  actually *produces* the release binary bit-for-bit; that property is
  proven elsewhere (`repro-drift.yml`, the release `repro` gate).
- **A brand-new unanchored source fails closed, not silently.** If a future
  dependency change introduces a *second* git or path source, check (3)'s
  set-comparison REDs on it by name rather than silently exempting it — but
  until someone grounds it the way miniscript is grounded, that dependency
  has **no** check-4-equivalent protection at all. The grounding work does
  not generalize automatically; each new unanchored source needs its own.

## 5. Re-grounding procedure — documented AND executed (F-381)

The procedure lived only as a comment in the script's own header
(`ci/repro/vendor-freshness.sh:73-76`). A procedure nobody has run is a
hypothesis, not a gate — this repo has shipped exactly that shape of invisible
defect before (a `journeys.sh` gate green for months against a mutated
binary; a plan whose acceptance walk was unsatisfiable). So it was rehearsed
end to end, on a scratch worktree, 2026-08-27, rather than merely re-described.

### 5.1 Setup — simulating a legitimate pin move
A first attempt bumped the pin to a commit that also touched
`rust-miniscript`'s `Cargo.toml` (an "Automated update to rustc nightly"
commit, `aea13ab`, 70 commits ahead of the pinned `ff4732e5` on
`rust-bitcoin/rust-miniscript`'s `master`). That was
**already caught by check (1)**, with a different failure signature than the
classic v0.74.0 one — `cargo` reported `the lock file ... needs to be updated
but --locked was passed to prevent this`, because `cargo update --precise`
(run against the real upstream source, to build a legitimate new
`Cargo.lock`) picked a resolution that the stale `vendor/miniscript` manifest
could not reproduce. This is a real, useful finding but not the scenario check
(4) exists for, so the rehearsal was redone against a commit chosen
specifically to reproduce the **F-354 shape**: `5dcd5fc`
(`5dcd5fcbf3b56c83e55864c9fc99386f49074cce`), 18 real commits ahead of
`ff4732e5` on `rust-bitcoin/rust-miniscript`'s `master`, confirmed by diff to
have a
**byte-identical `Cargo.toml`** to the pinned rev (same declared version, same
dependency set) while carrying substantive source changes (58 files, +2746/
−1640 lines, including a full-tree reformat and a new `src/validation.rs`).
Same-manifest / different-content is exactly what makes check (1) structurally
blind (§1.2, §3.1(1)) — the correct condition to rehearse recovery against.

### 5.2 The RED
```
$ sed -i 's/.../rev = "5dcd5fcbf3b56c83e55864c9fc99386f49074cce"/' Cargo.toml
$ cargo update -p miniscript --precise 5dcd5fcbf3b56c83e55864c9fc99386f49074cce
      Adding miniscript v13.0.0 (…?rev=5dcd5fcb…)
      Removing miniscript v13.0.0 (…?rev=ff4732e5…)
      note: pass `--verbose` to see 64 unchanged dependencies behind latest
$ bash ci/repro/vendor-freshness.sh
vendor-freshness: (1/4) OK — vendor/ satisfies Cargo.lock.
::error::vendor-freshness: 1 content defect(s) in vendor/:
  - the miniscript pin MOVED: Cargo.lock is at 5dcd5fcb…, but this gate is
      grounded at ff4732e5…. […] Re-vendor, verify the tree against upstream
      at the new rev, then update EXPECTED_GIT_FORK_REV /
      EXPECTED_GIT_FORK_MANIFEST_SHA256 in ci/repro/vendor-freshness.sh …
exit 1
```
Confirmed exactly as designed: **(1) stayed GREEN** (name/version/manifest all
still matched the stale vendor tree — the blind spot, demonstrated live, not
assumed) and **(4) alone** caught the moved pin.

### 5.3 The re-grounding, exactly as documented — no missing step
```
$ cargo vendor --locked vendor/
   [… full re-vendor, 169 crates …]
$ sha256sum vendor/miniscript/.cargo-checksum.json
9f5d6dccfcd02458c310489ce4e07259afcf06f3db72872501d11b69e9b08f86  vendor/miniscript/.cargo-checksum.json
```
Then, per the header's "verify against upstream before you do": a full
byte-for-byte comparison of every non-generated vendored file against a local
clone of `rust-miniscript` checked out at `5dcd5fc` (equivalent in method to
the GitHub-trees-API check the original grounding used) —
**102 of 102 files byte-identical, 0 mismatches, 0 missing** (`Cargo.toml` is
excluded, as it always is: `cargo vendor` rewrites it). The two constants were
then updated:
```
EXPECTED_GIT_FORK_REV="5dcd5fcbf3b56c83e55864c9fc99386f49074cce"
EXPECTED_GIT_FORK_MANIFEST_SHA256="9f5d6dccfcd02458c310489ce4e07259afcf06f3db72872501d11b69e9b08f86"
```
```
$ bash ci/repro/vendor-freshness.sh
vendor-freshness: (1/4) OK — vendor/ satisfies Cargo.lock.
vendor-freshness: (2/4) OK — 7495 files across 169 crates match their recorded sha256.
vendor-freshness: (3/4) OK — 168 crates anchored…; 1 git-fork source(s) exempt by grounding (miniscript).
vendor-freshness: (4/4) OK — vendor/miniscript matches the tree grounded against upstream 5dcd5fcb….
exit 0
```
**The documented procedure worked exactly as written, on the first attempt.**
No step was missing, no step was wrong; the header comment did not need a
correction. (The `cargo clean -p <crate>` recompilation-trap caveat noted
below did not bite here because these checks never compile anything — it
applies to a subsequent `cargo test`/`cargo build` step, not to this gate.)

### 5.4 Restore
```
$ git checkout -- Cargo.toml Cargo.lock ci/repro/vendor-freshness.sh vendor/
$ git clean -f -- vendor/miniscript/{AGENTS.md,CLAUDE.md,CONTRIBUTING.md,SECURITY.md,src/validation.rs}
$ git status --porcelain   # empty
$ git diff --stat 1f333ad2 # empty — tree is byte-identical to the committed state
$ bash ci/repro/vendor-freshness.sh
vendor-freshness: (4/4) OK — vendor/miniscript matches the tree grounded against upstream ff4732e5….
exit 0
```
`git checkout --` restores tracked-file *content* but does not remove files a
`cargo vendor` run newly created (the newer rev vendors a handful of files
`ff4732e5` doesn't have) — worth recording as a rehearsal-specific gotcha:
restoring a vendor tree after an experimental re-vendor needs an explicit
cleanup of the untracked additions, not just a checkout, or `git status` will
still show drift.

### 5.5 Recompilation-trap caveat (documented, not re-executed here)
`ci/repro/vendor-freshness.sh` never compiles anything, so the trap the F-354
fix commit measured (`cargo` not noticing a vendored source changed under a
fixed package-id, reporting a false `21/21` green from stale binaries) does
not apply to *this* gate. It applies to whatever compiles against the
re-vendored tree next — restated here because a maintainer following this
procedure will naturally follow it with a build: **run
`cargo clean -p <crate>` and verify a `Compiling <crate>` line appears** before
trusting any subsequent test result against a freshly re-vendored tree.

## 6. Test plan (must pass before commit)
1. **FRESH** (current master): `vendor-freshness.sh` exits 0, all four checks
   OK. *(Confirmed on the restored worktree tree: exit 0, 7490 files / 169
   crates, §5.4.)*
2. **STALE, resolution-visible** (simulate: restore `vendor/md-codec` to an
   older version, or delete a vendored crate dir): exits non-zero at check
   (1) with the resolution error + the clear `::error::` message.
3. **STALE, resolution-blind — the F-354 shape** (simulate: move the
   `[patch.crates-io]` miniscript rev to a commit with an unchanged manifest,
   leave `vendor/` untouched): check (1) stays green; check (4) REDs with the
   re-grounding message. **Executed, §5.2** — this is the scenario the
   original test plan did not name and could not have caught, because it
   predates the four-check design.
4. **Re-grounding recovery**: follow the header's documented procedure after
   a genuine pin move; confirm it returns the gate to exit 0 without a
   missing or incorrect step. **Executed, §5.3** — passed on the first
   attempt.
5. **Rev derivation**: the `Cargo.lock` grep yields the 40-hex rev; an empty
   result hard-errors (fail-closed).
6. **Path filter**: a `Cargo.lock`-only change triggers the workflow; an
   unrelated change does not.
7. **Mutation suite** (already run and merged with F-354, `2c4510c0`; not
   re-run here — cited for completeness): M1 tamper-a-byte → RED at (2)
   naming the file; M2 restore → clean; M3 restore the actual historical
   F-354 tree → RED at (4), (1)/(2) both silent; M4 restore fixed tree →
   clean; M5 tamper a registry `package` digest → RED at (1), redundantly
   (documents why (3)'s digest check is kept only for the unanchored-set
   assertion); M6 simulate a new unanchored source → RED, named; M7 simulate
   an ungrounded pin move → RED, with re-ground instructions (this is what
   §5 additionally *executed* rather than only simulating).

## 7. Companion
Closes the leading-gate gap exposed by the v0.74.0 release-CI post-mortem
(`fix(release): re-vendor … @ 45be1ec1`) and, since F-354, the wrong-revision
gap exposed by the two-month undetected `vendor/miniscript` drift. FOLLOWUP
slug: `vendor-freshness-pr-gate` (original scope); this revision closes
`F-381` (spec-vs-implementation drift + an unexecuted recovery procedure).

**Codec-repo exposure (R0 N3, still open).** `md-codec` / `mk-codec` /
`ms-codec` each commit their own `vendor/` tree consuming the same recipe and
have the **same latent exposure this spec's original one-check design did**:
a wrong-revision vendor of any git-sourced dependency they carry would be
just as invisible to a resolution-only check. They currently run the
**pre-F-354, one-check** form (no miniscript-equivalent fork, so this was
lower-priority), and porting the four-check design — including the
grounding step, if any of them ever gains a git-sourced dependency — remains
out of scope for this cycle. Tracked in each codec's own `design/FOLLOWUPS.md`
per the cross-repo follow-up convention.
