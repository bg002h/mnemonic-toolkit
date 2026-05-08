# Plan — End-user manual for the m-format star

**Plan-mode artifact.** Draft 1 (pre-architect-review).

| Field | Value |
|---|---|
| Date drafted | 2026-05-07 |
| Status | Drafting → architect review → ExitPlanMode |
| Target repo | `mnemonic-toolkit` |
| Manual root | `mnemonic-toolkit/docs/manual/` |
| Target audience | Two-track: Bitcoin power user (main flow) + onboarded newcomer (optional primers/appendix) |
| First release | Manual v0.1, tracking toolkit `main` (initial sync to v0.8.0) |
| Length budget | ~60–100 PDF pages (reference style) |

---

## Section 1 — Brainstorm

### 1.1 Context (why this exists)

The Shibboleth m-format star — `mnemonic-toolkit` (CLI `mnemonic`), `descriptor-mnemonic` (`md-codec` + `md-cli`), `mnemonic-key` (`mk-codec`, library-only), `mnemonic-secret` (`ms-codec` + `ms-cli`) — has shipped through `toolkit v0.8.0` (2026-05-07) with substantial CLI surface area (~103 flags across ~18 subcommands across 3 binaries) and an emerging "three-card steel-engravable bundle" UX. Currently the only end-user prose lives in four terse READMEs and four `Keep a Changelog`-format CHANGELOGs. There is no guided onboarding, no comparative reference, no glossary, no troubleshooting matrix, and nothing publishable as a PDF. The four-format star has reached a polish threshold where end-user adoption is gated on documentation, not code.

### 1.2 Goal

A single end-user manual, dual-rendered to **markdown** and **PDF (with mermaid graphics)**, that:

1. Onboards new users into the m-format mental model
2. Walks them through guided workflows (single-sig, multisig, watch-only, recovery, migration, wallet export, BIP-85 children)
3. Compares/contrasts overlapping features so users pick the right tool
4. Acts as a CLI reference for `mnemonic`, `md`, and `ms` (and as a Rust API guide for `mk-codec`)
5. Has TOC + glossary + index in both output formats

### 1.3 Decisions locked from clarifying-question rounds

| # | Decision | Locked value |
|---|---|---|
| D1 | CLI scope | All four formats: `mnemonic` + `md` + `ms` + `mk-codec` library chapter |
| D2 | Audience | Two-track: Bitcoin power user main flow + optional `:::primer` boxes & newcomer appendices |
| D3 | PDF toolchain | Pandoc + LaTeX + mermaid-filter |
| D4 | Graphics | Mermaid only (fenced ` ```mermaid ` blocks) |
| D5 | Manual location | `mnemonic-toolkit/docs/manual/` |
| D6 | Versioning | Living document tracking `main`; per-tag PDF builds attached to toolkit GitHub releases |
| D7 | Index mechanism | LaTeX `\index{}` markers inline in markdown (PDF page-number index via `makeidx`) + curated md-side `Term → §section` table |
| D8 | Length / depth | First release: reference style ~60–100 PDF pages. Quick-start (~25–40pp) and comprehensive book (150+pp) deferred — design accommodates both as future siblings |

### 1.4 Alternatives considered and rejected

- **Typst / AsciiDoctor as source of truth** — rejected because both create a second source format; markdown is the lingua franca of the four repos' READMEs and follow-up trackers. Pandoc keeps markdown authoritative.
- **mdBook** — rejected because PDF and index support via plugins is fragile.
- **Each repo holds its own chapter, toolkit hosts umbrella** — rejected because cross-repo authoring fragments the editing flow and complicates the per-tag PDF build pipeline. Toolkit repo is single source.
- **Auto-generated index from headings + glossary** — rejected as the sole index mechanism because it cannot mark concepts that don't appear in headings. Kept as a CI sanity check, not as the primary index.
- **Photographs of engraved cards / Excalidraw figures** — deferred (future polish). v0.1 of the manual is mermaid-only to keep the asset pipeline narrow.
- **Onboarding newcomers as the *main* flow** — rejected. Newcomer prose in the main flow (instead of in `:::primer` boxes / appendices) bloats every chapter and bores power users. The two-track convention solves this.

### 1.5 Non-goals (v0.1 of manual)

- **Not a Bitcoin tutorial.** Newcomer primers are concept *signposts*, not deep teaching material. Deep BIP-39 / BIP-32 / descriptor education stays linked-out to BIP texts and well-known references.
- **Not a SPEC.** Per-version specs in `mnemonic-toolkit/design/SPEC_*.md` are developer-facing and untouched.
- **Not a marketing page.** The README stays the marketing surface; the manual is post-purchase.
- **No HTML site (yet).** GitHub auto-renders markdown; standalone HTML is a future enhancement.
- **No translations** in v0.1. English only.
- **No troubleshooting bot / search index.** Only the printed/in-PDF index.
- **No coverage of retired / unreleased features** (e.g. retired `mc-codex32` extraction, unshipped BIP-85 RSA). Living document follows what's in `main`.

### 1.6 Risks and mitigations

| Risk | Mitigation |
|---|---|
| Doc drift as toolkit evolves | CI gate on toolkit repo: any PR adding/removing a CLI flag must touch the manual or apply a `manual-followup` label tracked in `docs/manual/FOLLOWUPS.md` |
| Worked-example seed mistaken for a real seed | Every example seed is the BIP-39 canonical test vector (`abandon abandon abandon … about`). Each appearance carries a fenced **DANGER** admonition: "Public test vector. Funds sent to addresses derived from this seed will be stolen instantly." |
| Pandoc/LaTeX setup brittle for contributors | `docs/manual/Makefile` + a Dockerfile (`docs/manual/Dockerfile.build`) pinning pandoc + texlive + mermaid-filter versions; `make pdf-docker` works without local install |
| mermaid-filter dependency unavailable on a contributor's box | `make pdf` falls back to pre-rendered SVG snapshots in `docs/manual/figures/cache/` checked into git when `MERMAID_FILTER=skip` |
| LaTeX `\index{}` markers leak into markdown viewer | Custom pandoc filter strips `\index{...}` from the markdown render path; PDF render path passes through. Verified by snapshot test. |
| Manual scope creep (becoming the comprehensive book early) | Hard ceiling: 100 PDF pages for v0.1. Architect review at end of Phase 5 (workflows complete) gates whether to scope-cut or accept overflow. |

---

## Section 2 — Spec

### 2.1 Output deliverables

1. **`docs/manual/build/m-format-manual.md`** — single-file concatenated markdown (TOC + body + glossary + index table). Renders cleanly on GitHub. Generated by `make md`.
2. **`docs/manual/build/m-format-manual.pdf`** — typeset PDF with mermaid graphics, page-numbered index, marginalia-style primer boxes. Generated by `make pdf`.
3. Both artifacts attached to each toolkit GitHub release (`make release-attach VERSION=v0.X.Y`).

### 2.2 Source-tree layout

```
mnemonic-toolkit/docs/manual/
├── README.md                       # how to build the manual
├── Makefile                        # `make md`, `make pdf`, `make pdf-docker`, `make lint`, `make clean`
├── Dockerfile.build                # pinned pandoc + texlive + mermaid-filter
├── pandoc/
│   ├── preamble.tex                # LaTeX preamble (makeidx, mdframed for primer boxes, fonts)
│   ├── metadata.yaml               # title, author, date, version, description
│   ├── filters/
│   │   ├── strip-latex-from-md.lua # strip raw-tex blocks for the markdown render path only
│   │   └── primer-box.lua          # render :::primer divs as mdframed in PDF, blockquote in md
│   └── templates/
│       └── manual.latex            # pandoc LaTeX template (extended default)
├── src/                            # AUTHORED CONTENT — single source for both md and PDF
│   ├── 00-frontmatter.md           # title page, copyright (CC0), about-this-manual, audience guide
│   ├── 10-foundations/             # Part I — Foundations
│   │   ├── 11-welcome.md           # what is the m-format star (mermaid: 4-card overview)
│   │   ├── 12-how-to-read.md       # two-track navigation, primer-box convention
│   │   └── 13-concept-signposts.md # 1-paragraph each on BIP-39 / BIP-32 / descriptors / multisig (signposts only; deep dives in appendices)
│   ├── 20-quickstart/              # Part II — Quick start
│   │   ├── 21-install.md           # cargo install / from-source / Docker
│   │   ├── 22-first-bundle.md      # `mnemonic bundle` BIP-84 single-sig walkthrough
│   │   ├── 23-verify.md            # `mnemonic verify-bundle` walkthrough
│   │   └── 24-recover.md           # minimal recovery walkthrough
│   ├── 30-workflows/               # Part III — Guided workflows (THE meat)
│   │   ├── 31-singlesig-steel.md   # single-sig steel-engraved backup (mermaid: card flow)
│   │   ├── 32-multisig-2of3.md     # multi-source 2-of-3 (`--slot @N.phrase=…`) (mermaid: 3-cosigner flow)
│   │   ├── 33-taproot-multi.md     # tap multisig (`tr-multi-a` / `tr-sortedmulti-a`) with NUMS internal key
│   │   ├── 34-watch-only.md        # xpub-only 2-card bundle
│   │   ├── 35-recovery-paths.md    # damaged card scenarios (which card lost → which tools recover)
│   │   ├── 36-migration.md         # BIP-39-only → m-format star migration
│   │   ├── 37-wallet-export.md     # Bitcoin Core / BIP-388 / Sparrow / Specter export
│   │   └── 38-bip85-children.md    # `mnemonic derive-child` for child entropy / passwords / DICE
│   ├── 40-cli-reference/           # Part IV — CLI reference
│   │   ├── 41-mnemonic.md          # `mnemonic` umbrella + 5 subcommands (~68 flags)
│   │   ├── 42-md.md                # `md` 8 subcommands (~22 flags)
│   │   ├── 43-ms.md                # `ms` 5 subcommands (~13 flags)
│   │   └── 44-mk-codec-rust.md     # `mk-codec` Rust API chapter (~11 public surface items)
│   ├── 50-comparing/               # Part V — Compare / contrast (THE other emphasis)
│   │   ├── 51-format-decision.md   # ms1 vs mk1 vs md1 vs toolkit (when to use which) — decision table
│   │   ├── 52-toolkit-vs-ms-cli.md # ms-cli alone vs `mnemonic` for secret cards
│   │   ├── 53-toolkit-vs-md-cli.md # md-cli alone vs `mnemonic` for descriptor cards
│   │   ├── 54-mformat-vs-others.md # m-format star vs SLIP-39 vs naked BIP-39 vs Shamir (carefully framed; no ranking)
│   │   ├── 55-singlesig-vs-multi.md # decision tree
│   │   ├── 56-bip39-vs-bip38-pass.md # the composite-edge subtlety (v0.8 BREAKING context)
│   │   └── 57-coredesc-vs-bip388.md # Bitcoin Core importdescriptors vs BIP-388 wallet_policy export
│   ├── 60-appendices/              # Part VI — Reference / newcomer deep-dives
│   │   ├── 61-glossary.md          # definitions; ordered alphabetically
│   │   ├── 62-bip39-primer.md      # BIP-39 entropy deep-dive (newcomer)
│   │   ├── 63-bip32-primer.md      # BIP-32 derivation deep-dive
│   │   ├── 64-descriptors-primer.md # descriptors + BIP-388 deep-dive
│   │   ├── 65-bch-codex-primer.md  # codex32 / BCH / m-codec error-correction sketch
│   │   ├── 66-test-seeds.md        # the canonical test vector + DANGER admonition
│   │   ├── 67-troubleshooting.md   # symptom → diagnosis → fix matrix
│   │   ├── 68-release-history.md   # auto-extracted CHANGELOG digest across all 4 repos
│   │   └── 69-index-table.md       # markdown-side curated `Term → §section` table
│   └── 99-build-banner.md          # appended to PDF only; "Built from $GIT_SHA on $DATE for toolkit $VERSION"
├── figures/                        # mermaid renders (cached SVGs); checked into git as fallback
│   └── cache/                      # populated by `make figures-cache`; consumed when MERMAID_FILTER=skip
├── agent-reports/                  # per-phase architect/reviewer reports (per repo convention)
│   └── .gitkeep
├── FOLLOWUPS.md                    # deferred work tracker (manual-local; mirrors repo convention)
└── tests/
    ├── cli-subcommands.list        # canonical `<binary> <subcommand>` enumeration consumed by lint.sh
    ├── fixtures/
    │   └── filter-smoke.md         # Phase 0 smoke fixture: 1 :::primer block + 1 \index{} marker
    ├── golden/
    │   ├── m-format-manual.md      # snapshot of `make md` output, regenerated on intentional change
    │   └── m-format-manual.layout  # PDF page-count + heading-page-number snapshot (not the bytes)
    └── lint.sh                     # single linter entry point: markdownlint + cspell + lychee + flag-coverage + glossary-coverage + index-bidirectional check
```

### 2.3 Two-track convention

Newcomer-targeted material in main-flow chapters appears inside a fenced div:

````markdown
:::primer
**Background — BIP-32 derivation.** A short paragraph (≤80 words) explaining
just enough of BIP-32 to follow this chapter. Power users skip these.
For a deep dive see [Appendix B](#appendix-b-bip-32-primer).
:::
````

- **PDF render** (via `pandoc/filters/primer-box.lua`): `mdframed` boxed sidebar with grey background, italic header, hairline border.
- **Markdown render**: blockquote with bold "Background — …" prefix.

### 2.4 Toolchain & build

- **`make md`** — concatenates `src/**/*.md` in lexicographic order **excluding** `99-build-banner.md`, runs `pandoc/filters/strip-latex-from-md.lua` to drop `\index{}` markers, runs primer-box filter (md mode), writes `build/m-format-manual.md`.
- **`make pdf`** — concatenates `src/**/*.md` in lexicographic order **including** `99-build-banner.md` (last; rendered as a plain page on the PDF inside back cover). Runs `primer-box.lua` in PDF mode (emits `mdframed`); does **not** run `strip-latex-from-md.lua`. Emits LaTeX → `xelatex` → `makeindex` → `xelatex` → `xelatex` (three passes; standard book-with-index recipe). Mermaid blocks rendered to SVG by `mermaid-filter` and embedded.
- **`make pdf-docker`** — same as `make pdf`, but inside `docs/manual/Dockerfile.build` (pinned pandoc + texlive-xetex + mermaid-filter). Default for CI and reproducibility.
- **`make figures-cache`** — runs the build once with mermaid-filter active and copies emitted SVGs into `figures/cache/` keyed by SHA-256 of the mermaid source block. Committed for `MERMAID_FILTER=skip` mode.
- **`make lint`** — runs `tests/lint.sh`, which is the single linter entry point. It calls in sequence: `markdownlint-cli2`, `cspell`, `lychee --offline`, `flag-coverage.sh` (subcommand-aware — see below), `glossary-coverage.sh`, and the index bidirectional-consistency check. (No separate top-level scripts; all logic lives behind `tests/lint.sh` for a single invocation surface.)
- **`flag-coverage.sh`** is invoked once per `<binary, subcommand>` pair, not just per binary: it iterates over the enumerated subcommand list — `mnemonic` ∈ {`bundle`, `verify-bundle`, `convert`, `export-wallet`, `derive-child`}; `md` ∈ {`encode`, `decode`, `inspect`, `address`, `bytecode`, `compile`, `vectors`, `verify`}; `ms` ∈ {`encode`, `decode`, `inspect`, `verify`, `vectors`} — invoking `${BIN} ${SUBCMD} --help` for each, parsing flags from each, and asserting each flag appears in the corresponding chapter section in `40-cli-reference/`. The subcommand list is committed at `tests/cli-subcommands.list` so additions in any sibling repo trigger a CI failure until the manual is updated.
- **`make clean`** — removes `build/` and the LaTeX byproducts (`*.aux`, `*.idx`, `*.ind`, `*.log`, `*.toc`).

### 2.5 Index mechanism

Authors place inline LaTeX directives in markdown source:

```markdown
The m-format star\index{m-format star} comprises three sibling cards…
```

- The `strip-latex-from-md.lua` filter erases `\index{...}` from the markdown output.
- The PDF pipeline keeps them; LaTeX `makeidx` builds a page-numbered alphabetical index emitted as the last appendix before `99-build-banner.md`.
- The markdown side ships `69-index-table.md` — a curated `Term → §section.subsection` table maintained by hand. Lint check: every term `\index{TERM}`'d in the source must appear in `69-index-table.md` (and vice versa), so the two indexes never diverge.

### 2.6 Glossary mechanism

`60-appendices/61-glossary.md` is a flat alphabetical list of definitions. Authoring rule: any acronym or m-format-specific term used anywhere in the manual must have a glossary entry. The glossary-coverage check is one of the steps inside `tests/lint.sh` (no separate top-level script): it greps capitalised acronyms and m-format-specific tokens (`policy_id_stub`, `slot`, `card`, `bundle`, `m-format star`, etc.) against the glossary.

### 2.7 Worked-example policy

- All worked examples use the canonical BIP-39 test vector `abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about`.
- Every chapter that introduces or first uses an example seed opens with a **DANGER** admonition box: *"This is a public test vector. Any wallet derived from this seed has been swept. Never engrave or fund this seed."*
- Existing pinned vectors in `mnemonic-toolkit/crates/mnemonic-toolkit/tests/vectors/v0_1/` and `tests/vectors/v0_2/` are the authoritative source for expected outputs. The build pipeline does *not* re-run the CLI to populate examples; instead, `make verify-examples` runs each chapter's examples against the locally-built `mnemonic`/`md`/`ms` binaries and compares against committed transcripts in `docs/manual/transcripts/`. Drift is a CI failure.

### 2.8 Living-document release model

- Manual master version follows `manual-vMAJOR.MINOR.PATCH`, independent of toolkit semver.
- Each toolkit tag triggers (in toolkit-repo CI): `cd docs/manual && make pdf-docker` and uploads `m-format-manual-toolkit-${TOOLKIT_VERSION}.pdf` as a release asset.
- `99-build-banner.md` is regenerated by `make pdf` to embed the resolved `git rev-parse HEAD`, ISO date, and the matching toolkit tag.
- A breaking content change to the manual (chapter restructuring, change of conventions) bumps `manual-MAJOR`; new chapters bump `manual-MINOR`; copyedits/typo fixes bump `manual-PATCH`.

### 2.9 Acceptance criteria (v0.1)

| # | Check | Verification |
|---|---|---|
| A1 | Manual builds clean to markdown | `make md` exits 0; `build/m-format-manual.md` non-empty; `markdownlint-cli2` passes |
| A2 | Manual builds clean to PDF | `make pdf-docker` exits 0; PDF page count is 60–100 |
| A3 | TOC present in both formats | First page of PDF; first heading section of markdown |
| A4 | Glossary present in both formats | `61-glossary.md` rendered in both; lint check passes |
| A5 | Index present in both formats | Last appendix in PDF (page-numbered); `69-index-table.md` in markdown; bidirectional consistency check passes |
| A6 | Mermaid graphics render in PDF | At least 4 mermaid blocks across the workflow chapters; PDF embeds them as SVGs |
| A7 | Two-track convention works | At least 6 `:::primer` boxes across main-flow chapters; renders as boxed sidebar in PDF and as styled blockquote in markdown |
| A8 | Every CLI flag is covered | `flag-coverage.sh` passes (every flag from `mnemonic`, `md`, `ms` `--help` is mentioned in Part IV) |
| A9 | All worked examples reproduce | `make verify-examples` exits 0 against locally-built binaries |
| A10a | CI build pipeline works | The `.github/workflows/manual.yml` workflow runs `make lint` + `make pdf-docker` on every push to `docs/manual/` and exits 0 |
| A10b | CI release-asset upload works | A pre-merge dry-run tag `manual-v0.1.0-rc1` triggers the release-asset upload step; the resulting PDF artifact passes A1–A9 (verified once before final tag; not part of the per-PR loop) |

---

## Section 3 — Plan

Work proceeds in nine phases. Each phase ends with `feature-dev:code-reviewer` (or `feature-dev:code-architect` for structure-affecting phases) iteration to 0 critical / 0 important findings, with the report persisted to `docs/manual/agent-reports/phase-N-review.md`. Lower-tier findings flow into `docs/manual/FOLLOWUPS.md`.

### Phase 0 — Toolchain skeleton

Create `Makefile`, `Dockerfile.build`, `pandoc/preamble.tex`, `pandoc/metadata.yaml`, both Lua filters, the LaTeX template, `tests/lint.sh`, and stub `src/` files (one heading per file, no body). `make pdf-docker` produces a placeholder PDF with TOC.

**Makefile binary-source convention (mandatory for Phase 0):** the Makefile defines three overrideable variables — `MNEMONIC_BIN`, `MD_BIN`, `MS_BIN` — used by every recipe that invokes a CLI (`verify-examples`, `flag-coverage`, transcript regeneration). Defaults:

```makefile
MNEMONIC_BIN ?= cargo run --quiet -p mnemonic-toolkit --bin mnemonic --
MD_BIN       ?= cargo run --quiet --manifest-path ../../../descriptor-mnemonic/Cargo.toml -p md-cli --bin md --
MS_BIN       ?= cargo run --quiet --manifest-path ../../../mnemonic-secret/Cargo.toml -p ms-cli --bin ms --
```

CI overrides these to point at pre-built binaries (faster, no per-test recompile).

**Filter-pipeline contract (mandatory for Phase 0):** `pandoc/filters/strip-latex-from-md.lua` is invoked **only** on the `make md` path (so raw `\index{}` and `\begin{mdframed}` markers are stripped from the markdown render). It is **never** invoked on `make pdf` / `make pdf-docker` (so `primer-box.lua` may safely emit raw LaTeX into the PDF stream). Phase 0 verification must include a smoke test fixture (`tests/fixtures/filter-smoke.md`) containing one `:::primer` block and one `\index{}` marker, asserting:
- `make md` output contains a styled blockquote (no `\index`, no `\begin{mdframed}`)
- `make pdf-docker` output PDF embeds an `mdframed` block and registers the index entry on the index page (verify by `pdftotext` snapshot)

**Verification:** `make pdf-docker` exits 0; output PDF is ≥1 page with a TOC. `make lint` exits 0. Filter smoke test passes both ways. Architect review on the build pipeline before proceeding.

### Phase 1 — Frontmatter + glossary scaffold + index plumbing

Author `00-frontmatter.md` (title page, audience guide). Populate `61-glossary.md` with the terms already known (~40 entries: m-format star, ms1, mk1, md1, card, bundle, slot, policy_id_stub, codex32, BCH, BIP-388, etc.) — definitions can be 1-line stubs, refined later. Wire up `\index{}` handling in `strip-latex-from-md.lua`.

**Real-marker discipline (mandatory):** Insert at least one real `\index{m-format star}` marker into `00-frontmatter.md` and a matching entry in `69-index-table.md`. Then **deliberately remove** the `69-index-table.md` entry, run `make lint`, and confirm the bidirectional consistency check fails (proving the linter is not vacuously passing on empty inputs). Re-add the entry; lint must then pass. Capture the deliberate-failure transcript in `docs/manual/agent-reports/phase-1-lint-trapdoor.md`.

**Verification:** A4 + A5 + glossary lint pass; the trapdoor test demonstrates the linter is not vacuously passing. Architect review.

### Phase 2 — Foundations chapters (Part I)

Write `11-welcome.md` (with the 4-card overview mermaid diagram), `12-how-to-read.md` (with two `:::primer` examples), `13-concept-signposts.md`. Confirm primer-box filter renders correctly in both formats. Build the first newcomer-appendix link round-trips.

**Verification:** A7 partial (≥2 primer boxes). Reviewer round.

### Phase 3 — Quick-start (Part II)

Write `21-install.md`, `22-first-bundle.md`, `23-verify.md`, `24-recover.md`. Establish the worked-example transcript convention: each chapter has `transcripts/22-first-bundle.txt` regenerable via `make verify-examples`. Lock the BIP-39 canonical test vector + DANGER admonition pattern.

**Verification:** A9 partial (the four quick-start examples reproduce). Reviewer round.

### Phase 4 — Guided workflows (Part III, the headline emphasis)

Write the eight workflow chapters in order: `31`–`38`. Each chapter includes ≥1 mermaid diagram. Multi-cosigner workflow (`32-multisig-2of3.md`) and recovery (`35-recovery-paths.md`) get extra polish since these are the highest-value differentiators. After this phase, the manual is functionally usable even if the CLI reference is incomplete.

**Verification:** A6 (≥4 mermaid blocks across workflows), A9 (all workflow examples reproduce). Architect review on the workflow set as a whole — this is the highest-content phase.

### Phase 5 — CLI reference (Part IV)

Generate `--help` snapshots into `transcripts/cli-help/` and turn them into the four CLI-reference chapters (`41-mnemonic.md` / `42-md.md` / `43-ms.md` / `44-mk-codec-rust.md`). Each subcommand chapter follows a fixed template: synopsis → flag table → 1–3 worked examples → cross-reference to relevant workflow chapter.

**Verification:** A8 (`flag-coverage.sh` passes). Reviewer round.

### Phase 6 — Compare/contrast (Part V, the other headline emphasis)

Write `51`–`57`. Each chapter is short (2–4 pages) and table-heavy. Decision-tree chapters (`51`, `55`) deserve mermaid flowcharts. The `54-mformat-vs-others.md` chapter must be carefully neutral — no ranking, focus on *when each fits*.

**Density watch (per architect review N1):** chapters `54-mformat-vs-others.md` and `57-coredesc-vs-bip388.md` have the highest spillover risk for the 2–4 page target — they require enough protocol detail to drift past 4 pages. Architect reviewer must explicitly evaluate each chapter's page count after first draft and recommend either a scope cut (move detail to an appendix) or a budget exception (signed off explicitly).

**Verification:** Reviewer round with explicit instruction to flag any partisan framing in `54` and to evaluate the page counts of `54` and `57` against the watch criterion.

### Phase 7 — Reference appendices (Part VI)

Newcomer deep-dive primers (`62`–`65`), `66-test-seeds.md`, troubleshooting matrix (`67`), release-history digest (`68` — **hand-authored for v0.1** by reading each repo's CHANGELOG.md and writing a short per-repo prose summary; auto-extraction is filed as a v0.2 follow-up in `docs/manual/FOLLOWUPS.md`), curated md-side index (`69-index-table.md`). Refine glossary stubs from Phase 1 into full definitions.

**Verification:** A5 full (bidirectional index check), glossary lint full, A1+A2 with full content. Reviewer round.

### Phase 8 — Final polish + CI integration

Copyedit pass across all chapters (architect review focused on consistency: voice, tense, table headings, code-fence languages, cross-reference style). PDF page-count audit: hit the 60–100pp target; if over, scope-cut workflow chapters into appendix-pointers; if under, expand a few examples. Wire `docs/manual/` into the toolkit-repo CI (`.github/workflows/manual.yml`): on any push touching `docs/manual/`, run `make lint` + `make pdf-docker`; on tag matching `manual-v*`, additionally upload the PDF as a release asset. Open the umbrella PR.

**Verification:** A1–A9 + A10a all pass on CI for the PR. A10b is verified once via a pre-merge `manual-v0.1.0-rc1` tag push to confirm the release-asset upload step; if the PDF passes A1–A9, the rc tag is deleted and the PR is merged. The first real `manual-v0.1.0` tag is pushed only post-merge. Final architect review on the integrated manual + the CI workflow file. PR opened.

### Phase 9 — Cross-repo follow-up filing

Per the mirror-invariant protocol documented in `descriptor-mnemonic/CLAUDE.md` ("primary entry in originating repo + companion in affected repo, with `Companion:` lines and lockstep resolution discipline"):

1. **Primary entry** — in `mnemonic-toolkit/design/FOLLOWUPS.md`, file a `cross-repo` tier entry titled `manual-cli-surface-mirror` recording: (a) the manual now mirrors `md-cli`, `ms-cli`, and `mk-codec` public surface in `docs/manual/40-cli-reference/`; (b) any flag/API addition in those siblings requires a companion update here; (c) lists the three companion entries.
2. **Companion entries** — in each of the three sibling repos' `design/FOLLOWUPS.md` (`descriptor-mnemonic/`, `mnemonic-secret/`, `mnemonic-key/`), file a `cross-repo` tier companion entry titled `manual-cli-surface-mirror` with a `Companion:` line citing the mnemonic-toolkit primary entry. Each companion notes that changes to its repo's CLI/API surface must mirror to the manual before release.
3. **CLAUDE.md updates** — add a `## Manual coverage` section to each of the three sibling repos' `CLAUDE.md` (and to `mnemonic-toolkit/CLAUDE.md` if absent) noting the manual location and the mirror-invariant entry id.

**Verification:** One primary entry + three companion entries (each with paired `Companion:` lines), four `CLAUDE.md` edits. Tracked as the closing entry in `mnemonic-toolkit/docs/manual/FOLLOWUPS.md`.

### Critical files / paths to reuse (do not duplicate)

| Asset | Path | Use |
|---|---|---|
| Toolkit overview prose | `mnemonic-toolkit/README.md` | Source for Phase 2 (welcome) — paraphrase, not copy |
| Per-version specs | `mnemonic-toolkit/design/SPEC_*.md` | Reference *only*; never quote (developer-facing) |
| Pinned test vectors | `mnemonic-toolkit/crates/mnemonic-toolkit/tests/vectors/v0_{1,2}/` | Authoritative outputs for Phase 3–5 examples |
| BIP-39 canonical vector | `mnemonic-toolkit/crates/mnemonic-toolkit/tests/bip39_trezor_vectors.json` | Source for the canonical test seed used in every example |
| Sibling READMEs | `descriptor-mnemonic/README.md`, `mnemonic-key/README.md`, `mnemonic-secret/README.md` | Source for Phase 6 (compare/contrast) and Phase 5 (CLI ref) opening prose |
| All four CHANGELOGs | `*/CHANGELOG.md` | Source for `68-release-history.md` (Phase 7) |
| md-codec corpus | `descriptor-mnemonic/design/CORPUS.md` + `crates/md-codec/tests/vectors/` (per-case sibling files keyed by case name: `<case>.bytes.hex`, `<case>.descriptor.json`, `<case>.phrase.txt`, `<case>.template`; no JSON aggregate) | Worked examples for `42-md.md` and `33-taproot-multi.md` |

### Reviewer dispatch protocol (per phase)

Per `feedback_iterative_review_every_phase`:
- Each phase ends with one or more agent rounds — `feature-dev:code-architect` for phases 0, 4, 8 (structural); `feature-dev:code-reviewer` for the rest.
- Rounds iterate until 0 critical and 0 important findings.
- Per-implementation-phase reports persist to `docs/manual/agent-reports/phase-N-review-{1,2,…}.md`.
- Brainstorm/spec/plan/final reviews stay in transcript only.
- Low/nit findings flow to `docs/manual/FOLLOWUPS.md`.

### Out-of-scope deferrals (filed in `FOLLOWUPS.md` once created)

- **Quick-start (~25–40pp) variant** — slim subset of the reference manual, post-v0.1.
- **Comprehensive book (150+pp)** — adds deep BIP background, full troubleshooting matrix, design history. Post-v0.1.
- **HTML output** (mdBook or pandoc HTML+CSS).
- **Translations** (es, ja, zh) — translation memory file format TBD.
- **Photographs / Excalidraw figures** for engraved-card illustrations.
- **Search index for the PDF** (xindy with multi-key entries instead of plain `makeidx`).
- **Companion website** at e.g. `manual.mnemonic-toolkit.org`.

---

## Verification of the plan itself

Before `ExitPlanMode`:

1. Plan file passes one or more `feature-dev:code-architect` review rounds with 0 critical / 0 important findings (per `feedback_architect_review_meta_plans`).
2. All open user-facing decisions answered (D1–D8 above) — confirmed via the two AskUserQuestion rounds.
3. No retired/abandoned features referenced (`mc-codex32` extraction is correctly absent).
4. Plan-mode file is the only edit (per plan-mode constraint).
