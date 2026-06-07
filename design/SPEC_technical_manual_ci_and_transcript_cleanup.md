# SPEC — technical-manual CI workflow + api-harvest transcript cleanup

**Cycle:** technical-manual Cycle C+D (the two deferred residuals from the
symbol-pin cycle, `project_technical_manual_symbol_pin_lint_shipped`).
**Date:** 2026-06-07.
**Source SHA:** `origin/master` == local `HEAD` == `6c9e629`.
**Disposition:** docs + CI only — **no version bump, no tag** (binary
byte-identical; mirrors `a83dc75` / `dd7c228` / the symbol-pin cycle `6c9e629`).
**Resolves FOLLOWUPs:** `technical-manual-transcript-lineref-staleness` (Item 1),
`technical-manual-ci-workflow-source-checkout` (Item 2).

---

## 0. Context

The symbol-pin cycle (`6c9e629`) replaced all ~826 `file.rs:N` line-number
citations in the 14 rendered `docs/technical-manual/src/` chapters with
drift-resistant `` `file.rs::symbol` `` anchors and added a **BLOCKING**
`make lint` gate (`tests/symbol-ref-check.py`, step 7/7). It deferred two
residuals:

1. The 4 tracked `transcripts/api-harvest-*.md` files still carry ~385 stale
   `file.rs:N` line-refs (out of the `src/`-only scope).
2. The technical manual has **no CI workflow** — `make lint` (incl. the new
   gate) runs only when a human invokes it.

This cycle tackles both, in one SPEC, behind one mandatory R0.

---

## Item 1 — Delete the 4 `api-harvest-*.md` transcripts (Cycle C)

### What

`git rm` these four files:

- `docs/technical-manual/transcripts/api-harvest-md-codec.md` (622 lines)
- `docs/technical-manual/transcripts/api-harvest-mk-codec.md` (376 lines)
- `docs/technical-manual/transcripts/api-harvest-ms-codec.md` (184 lines)
- `docs/technical-manual/transcripts/api-harvest-mnemonic-toolkit.md` (473 lines)

(1655 lines, ~385 `.rs:N` refs.)

### Why delete (not symbol-pin) — FOLLOWUP option (b)

- **Unrendered authoring scaffolding.** The Makefile globs `src/` only for
  every rendered + linted artifact (`MD_SRC := find $(SRC_DIR) ...`,
  Makefile:74). The transcripts are **never** rendered into the PDF, never
  `{{#include}}`'d (verified: no `{{#include}}` and no `api-harvest` reference
  anywhere in `src/`), and never linted (markdownlint/cspell/lychee all scan
  `$SRC_DIR` only).
- **Not consumed by the example harness.** `verify-examples.sh` replays the
  `.cmd`/`.out` transcript pairs, not the `.md` harvests (verified: no
  `api-harvest` reference in `tests/` or the Makefile). The `.cmd`/`.out`
  pairs are **kept** — only the 4 `.md` harvest snapshots are deleted.
- **Superseded.** The rendered, now-symbol-pinned, lint-gated Part V chapters
  (`50-rust-api/*`) are the authoritative API surface. The harvests were a
  pre-Part-V working note.
- **Version-stale + provably wrong to migrate.** Pinned to old crate versions
  (e.g. `Version | 0.8.0`); a mechanical migration is provably wrong (the
  prior cycle-prep found `api-harvest-mnemonic-toolkit.md:466` "`format.rs:114`"
  now resolves to `MultisigInfo`, not `BundleJson`).
- **Precedent.** Same class as the deleted cli-help goldens
  (`project_cli_help_golden_cleanup_shipped`): unrendered, superseded,
  version-stale scaffolding → delete rather than gate.

### Link-integrity proof (not asserted — measured)

`lychee --offline` scans `$SRC_DIR`. On a faithful clone with the transcripts
deleted it reports **547 OK / 0 Errors / 38 excluded** (§3 proof matrix). No
`src/` file links into the deleted transcripts; deletion creates no dangling
link.

---

## Item 2 — Add `.github/workflows/technical-manual.yml` (Cycle D)

### Design decision: option (A) toolkit-only, lint-only (NOT 4-repo checkout)

The FOLLOWUP's stated "wrinkle" is that `symbol-ref-check` resolves
`file.rs::symbol` against the **sibling codec source trees**, so a "complete"
CI would check out all four repos at coherent SHAs. We **deliberately reject**
that (option C) and choose **option A — single-repo, lint-only** because:

- The codecs are **independently versioned** (toolkit pins them via crates.io:
  `md-codec=0.35`, `mk/ms-codec=0.4.0`). A 4-repo source checkout would couple
  the technical-manual CI to live codec `master` (or to a hand-maintained SHA
  matrix), reintroducing exactly the cross-repo drift coupling the constellation
  avoids elsewhere (advisor decision, pre-R0).
- The **regression-prevention prize is binary-independent.** What CI must catch
  is the *reintroduction of `file.rs:N` line-refs* (G1) — the entire point of
  the symbol-pin cycle. **G1 needs no source at all** and fires on all 14
  chapters. G2 (symbol existence) for **toolkit** refs is checkable in CI
  (toolkit source is the repo itself). G2 for **codec** refs is the only piece
  that needs siblings — and the gate skips those gracefully (after the Item-2a
  fix below).
- `make lint` is **binary-independent.** Steps 1–3 (markdownlint/cspell/lychee)
  scan source text; step 4 (api-surface-coverage) is **warning-only** (`|| warn`)
  and reads `lib.rs`/`format.rs` source (not the built binaries); steps 5–7 are
  grep/python. The `*_BIN` args are vestigial for lint (lint.sh:23 "unused at
  v0.1"). So **no `cargo build`, no `cargo install`, no chromium** is required.
- `figures-cache-verify` (a `make lint` prerequisite, Makefile:222) runs
  `render-mermaid-cache.py --verify`, which is **render-free** (it checks the
  checked-in SHA-keyed cache; it does not invoke `mmdc`/chromium).

**Accepted, documented gap (→ new FOLLOWUP):** codec-file G2 (symbol existence
in md/mk/ms-codec + clis) is **not** enforced in CI — those refs skip when
siblings are absent. It remains enforced by **local `make lint`** (siblings
present), which every contributor runs. The same skip extends to a
*non-authoritative-chapter* ref to any file unresolvable in the present
(toolkit) repo. This is acceptable: the parent FOLLOWUP's resolved wording is
"CI-gated **OR** make lint-gated", and the leading regression (line-ref
reintroduction) is fully CI-gated (a line-ref can only be reintroduced by
editing a chapter, which fires the docs trigger path).

**Post-R0 addendum (task-complete advisor checkpoint) — a second, distinct
CI-coverage gap.** Separately from the siblings-absent *skip-logic* gap above,
there is a *trigger-path* gap: the workflow fires on `docs/technical-manual/**`
(+ the cache tool + itself), NOT on `crates/**`. So a toolkit-source rename
that breaks a `file.rs::symbol` citation **without** a docs edit will not fire
this workflow; the broken citation ships uncaught until the next change under
the docs path (or a local `make lint`). Different mechanism — the gate logic is
fine, the workflow simply isn't triggered. Adding `crates/**` to the trigger is
deliberately declined (it would run the docs lint on every unrelated code PR;
`manual.yml` declines the symmetric coupling). Both gaps are filed together as
`technical-manual-g2-uncovered-in-bare-ci` (skip-logic + trigger-path).

### Item 2a — Fix the graceful-skip hole in `symbol-ref-check.py` (prerequisite)

**The bug.** `resolve()` returns `unresolved` (→ G2 **FAIL**) — not
`skip:<repo>` — for a **bare codec basename cited in a NON-authoritative
chapter** when siblings are absent. An authoritative chapter knows its repo and
emits `skip:<repo>` when that repo is absent; a catch-all chapter (10/14/30s/60s)
does not know which absent sibling a bare basename belongs to, so it falls
through to `unresolved` and **false-fails CI**.

Empirically (siblings-absent simulation, §3): **59 false-fails**, all in the
two non-authoritative chapters `30-address-derivation/*` and
`60-back-matter/61-glossary.md`, all citing **md-codec** files
(`to_miniscript.rs`, `address_derivation.rs` [a `tests/` file], `identity.rs`,
`canonical_origin.rs`, `canonicalize.rs`, `key_card.rs`, `origin_path.rs`,
`payload.rs`, `phrase.rs`, `bytecode/xpub_compact.rs`). Zero false-fails in any
authoritative chapter (those already skip correctly).

**The fix.** In `resolve()`, after the existing `hit` / `ambiguous` checks and
before the final `return (None, "unresolved")`, add:

```python
    # Non-authoritative catch-all chapter: a bare basename may belong to any
    # repo. With siblings absent (bare CI) we cannot disprove that it lives in
    # an absent sibling, so skip rather than false-fail. Local runs (all repos
    # present, ABSENT empty) still fail on a genuinely-missing symbol.
    if not auth and not qualified and ABSENT:
        return (None, "skip:absent-sibling")
    return (None, "unresolved")
```

with `ABSENT = [r for r in REPO_ROOTS if r not in PRESENT_REPOS]` hoisted to
module scope (the bottom-of-file `absent = ...` is replaced by a reference to
it).

**Why this is correct and preserves the prize:**

- Restricted to `not auth` → **authoritative chapters stay strict**: a renamed
  toolkit symbol in chapter 41/42/54 still **FAILs** in CI (the prize). Proven
  (§3 Test D).
- Restricted to `not qualified` → an explicit `crates/<codec>/...` path still
  routes through the qualified branch (which already returns `skip:<repo>` when
  absent).
- Gated on `ABSENT` non-empty → **local behavior is byte-identical**: with all
  repos present `ABSENT == []`, the branch never fires, and a genuinely-missing
  symbol still FAILs locally. Proven (§3 Test B: 725 checked / 0 skipped,
  identical to the pre-fix local GREEN).
- File-resolution skip only — **segment checks on present (toolkit) files still
  run**: a fake symbol on a resolvable toolkit file FAILs even in CI (a bad
  anchor only escapes CI if its *file* is unresolvable). Proven (§3 Test D).

**Warning reword (advisor note).** The bottom-of-file warning currently says
"`%d codec-chapter refs skipped`". Since catch-all-chapter refs now also skip,
reword to "`%d sibling-repo refs skipped (codec G2 not enforced in bare CI)`".

### Item 2b — The workflow YAML

`.github/workflows/technical-manual.yml`, mirroring `manual.yml`'s node/lychee
install pattern but **lint-only** (no pandoc/texlive, no cargo, no chromium):

```yaml
name: technical-manual

# Lint-only CI for the m-format TECHNICAL manual. Single-repo (toolkit) by
# design — `make lint` is binary-independent (markdownlint/cspell/lychee scan
# source text; api-surface-coverage is warning-only and reads lib.rs/format.rs;
# figures-cache-verify is render-free). symbol-ref-check enforces G1 (line-ref
# ban) on all chapters + G2 (symbol existence) for refs resolvable in the
# present (toolkit) repo; G2 for any ref whose file is unresolvable here —
# codec files in absent siblings, AND renamed toolkit files cited by bare
# basename from non-authoritative chapters — skips gracefully in bare CI
# (enforced by local `make lint`; see FOLLOWUP
# technical-manual-g2-uncovered-in-bare-ci). No tag trigger (the technical
# manual has no release asset; manual.yml owns the manual-v* PDF release).

on:
  push:
    branches: [main, master]
    paths:
      - 'docs/technical-manual/**'
      - 'docs/tools/render-mermaid-cache.py'
      - '.github/workflows/technical-manual.yml'
  pull_request:
    paths:
      - 'docs/technical-manual/**'
      - 'docs/tools/render-mermaid-cache.py'
      - '.github/workflows/technical-manual.yml'

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        # Default checkout (NO `path:`): GitHub nests the repo at
        # $GITHUB_WORKSPACE = .../work/<repo>/<repo>, so symbol-ref-check's
        # WS-derivation (4 dirs up from src/) lands on .../work/<repo>, and
        # REPO_ROOTS[toolkit] = WS/mnemonic-toolkit/crates/mnemonic-toolkit
        # resolves to the checkout. A `path:` override would break this.
        uses: actions/checkout@v4

      - name: Install make
        run: |
          sudo apt-get update
          sudo apt-get install -y --no-install-recommends make

      - name: Set up Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Install npm lint tools
        run: |
          npm install -g \
            markdownlint-cli2@^0.13 \
            cspell@^8

      - name: Install lychee
        run: |
          LARCH=x86_64-unknown-linux-gnu
          curl -fsSL "https://github.com/lycheeverse/lychee/releases/download/lychee-v0.24.2/lychee-${LARCH}.tar.gz" \
            | sudo tar -xz --strip-components=1 -C /usr/local/bin "lychee-${LARCH}/lychee"
          sudo chmod +x /usr/local/bin/lychee

      - name: Lint technical manual
        working-directory: docs/technical-manual
        # *_BIN=true: the `make lint` path never INVOKES a binary —
        # api-surface-coverage reads lib.rs/format.rs source directly and
        # ignores these vars, symbol-ref-check ignores them. `=true` is
        # belt-and-suspenders against the Makefile's default `cargo run ...`
        # so no accidental compile can be triggered. A non-zero `make lint`
        # fails the job.
        run: |
          make lint \
            MNEMONIC_BIN=true \
            MD_BIN=true \
            MS_BIN=true \
            MK_BIN=true
```

Notes:
- **No `manual-v*` (or any) tag trigger** — the technical manual ships no
  release asset; `manual.yml` owns the `manual-v*` PDF release. This workflow
  is push/PR-on-paths only.
- `python3`, `git`, `curl` are preinstalled on `ubuntu-latest`; `make` is
  installed defensively (mirrors `manual.yml`).
- A failing lint step → `make lint` exits 1 → the job fails (blocking on PRs).

---

## 3. Proof matrix (already measured at SPEC time, on `6c9e629`)

All runs on a faithful `git archive HEAD` clone at
`/tmp/ciclone/mnemonic-toolkit` (one repo dir under a workspace parent, no
siblings — mirrors GitHub's `work/<repo>/<repo>` nesting), with the Item-2a fix
applied and the Item-1 transcripts deleted, unless noted.

| # | Scenario | Expected | Result |
|---|----------|----------|--------|
| A | gate, siblings-absent, clean | exit 0 | ✅ 298 checked, 427 skipped, OK |
| B | gate, **siblings-present**, clean (no regression) | exit 0, 0 skipped | ✅ 725 checked / 0 skipped (byte-identical to pre-fix local GREEN) |
| C | gate, siblings-absent, planted `bundle.rs:999` | exit 1 (G1) | ✅ caught (`line-ref bundle.rs:9`) |
| D | gate, siblings-absent, planted fake symbol on a real toolkit file in **authoritative** ch42 | exit 1 (G2 prize) | ✅ caught (segment not found) |
| E | **full `make lint`** (all 7 steps), siblings-absent, transcripts deleted, fix applied, `*_BIN=true` | exit 0, all steps green | ✅ md-lint 0 / cspell 0 / **lychee 547 OK 0 Errors** / api-surface OK (3 warns) / glossary OK / index OK / symbol-ref OK (298/427) / figures-cache-verify pass (no chromium) |

Test E is the load-bearing CI fidelity check: the **real entrypoint** CI will
run, on the exact layout CI produces, proving simultaneously (a) all 7 steps
green with siblings absent, (b) the tool-install list (node + markdownlint-cli2
+ cspell + lychee + python3; **no** cargo/chromium) is complete, and (c) the
transcript deletion leaves no dangling lychee link.

---

## 4. Ship plan

1. Apply Item-2a fix to `docs/technical-manual/tests/symbol-ref-check.py`
   (+ warning reword).
2. `git rm` the 4 `api-harvest-*.md` transcripts (Item 1).
3. Add `.github/workflows/technical-manual.yml` (Item 2b).
4. Re-prove: `make -C docs/technical-manual lint` GREEN **locally** (siblings
   present — must stay byte-identical, 725/0) AND the `/tmp/ciclone`
   siblings-absent `make lint` GREEN (Test E).
5. `design/FOLLOWUPS.md`: flip both resolved; file the new
   `technical-manual-g2-uncovered-in-bare-ci` gap FOLLOWUP. Its body must
   inherit §2's *full* framing, not a codec-only narrowing (R0-r1 M1): with
   siblings absent, **any** bare-basename ref in a non-authoritative chapter
   whose file is unresolvable in the present (toolkit) repo is CI-skipped —
   that includes both (i) codec-file G2 (the common case) AND (ii) a *renamed
   toolkit file* cited by bare basename from a non-authoritative chapter. Both
   are caught only by local `make lint` (where `ABSENT == []` so the skip
   branch never fires), not by bare-CI. Record the lockstep cost (revisit if a
   multi-repo source checkout is ever warranted).
6. Stage paths explicitly (no `git add -A`). Commit (Co-Authored-By trailer,
   `git commit -F -`). Push to `master`. **No version bump, no tag.**
7. Record cycle in memory + MEMORY.md index.

### Out of scope (this cycle)

- Multi-repo (4-repo) source checkout for full codec-G2-in-CI (option C) —
  deferred to the new gap FOLLOWUP; rejected for now (couples to codec drift).
- Building/uploading any PDF asset (the technical manual ships none).
- Re-validating chapter prose claims or migrating any `src/` citation (done in
  the symbol-pin cycle).
- Touching the `.cmd`/`.out` example transcript pairs (kept; used by
  verify-examples).
