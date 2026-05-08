# Design — m-format Quick Start guide v0.1

| Field | Value |
|---|---|
| Date | 2026-05-08 |
| Status | Brainstorm complete; awaiting writing-plans handoff |
| Target repo | `mnemonic-toolkit` |
| Artifact root | `docs/quickstart/` |
| Audience | True Bitcoin / m-format newcomer |
| Length budget | 25-40 PDF pages (target ~34) |
| Independence | Own `quickstart-v*` tag schedule; not coupled to `manual-v*` |

## 1. Context

The reference manual `manual-v0.1.0` shipped 2026-05-08 (129pp, full surface coverage). It targets the Bitcoin power-user with `:::primer` boxes for newcomer asides. Per the v0.1 plan's `§3.5` deferrals, a separate Quick-Start variant was deferred for newcomer-aimed onboarding.

This spec covers that variant: a parallel artifact at `docs/quickstart/` with its own version tag schedule, sharing the manual's toolchain (pandoc + xelatex + mermaid-filter) and worked-example transcripts, but re-authoring all prose in a newcomer voice.

## 2. Locked decisions (D1-D5)

| # | Decision | Locked value |
|---|---|---|
| D1 | Audience | True Bitcoin / m-format newcomer; ~25-40pp |
| D2 | Workflow scope | Single-sig + 2-of-3 multisig + watch-only (single-sig WO + multisig WO) |
| D3 | Source-of-truth strategy | Independent prose authoring; shared assets via symlinks (transcripts, lint configs); mermaid blocks copy-pasted (allows newcomer-tuned captions/colors) |
| D4 | Location + versioning | `docs/quickstart/` parallel to `docs/manual/`; own `quickstart-v*` tag schedule |
| D5 | Worked-example seed convention | Same canonical BIP-39 test seed + DANGER pattern as the manual; reuses the 5 existing `docs/manual/transcripts/*.{cmd,out}` files via symlink |

## 3. Non-goals (v0.1)

- Not a CLI reference. Forward-pointed to manual Part IV.
- Not a comparison with SLIP-39 / codex32 alone / naked BIP-39. Forward-pointed to manual Part V.
- No taproot multisig, BIP-39-only migration, BIP-85 child secrets, wallet-export beyond mention. All forward-pointed to manual Part III.
- No HTML output. Markdown + PDF only.
- No translations. English only.
- No `\index{}` markers (newcomers read top-to-bottom, not via index).
- No formal glossary. Inline definitions where needed.

## 4. Source-tree layout

```text
mnemonic-toolkit/docs/quickstart/
├── README.md                       # how to build (with symlink note for non-Linux contributors)
├── Makefile                        # `make md / pdf / pdf-docker / lint / verify-examples / clean`
├── pandoc/
│   ├── preamble.tex                # quickstart-tuned LaTeX preamble (lighter than manual's; no makeindex)
│   ├── metadata.yaml               # title="m-format Quick Start", own author/date
│   └── filters/                    # SYMLINK → ../../manual/pandoc/filters/
├── src/
│   ├── 00-frontmatter.md
│   ├── 10-foundations/
│   │   ├── 11-what-is-this.md
│   │   ├── 12-bitcoin-in-30-seconds.md
│   │   └── 13-the-three-cards.md
│   ├── 20-singlesig/
│   │   ├── 21-install.md
│   │   ├── 22-generate-entropy.md
│   │   ├── 23-bundle.md
│   │   ├── 24-verify.md
│   │   ├── 25-stamp.md
│   │   └── 26-recover.md
│   ├── 30-multisig/
│   │   ├── 31-why-multisig.md
│   │   ├── 32-bundle.md
│   │   └── 33-stamp-and-recover.md
│   ├── 40-watch-only/
│   │   ├── 41-singlesig-watch-only.md
│   │   └── 42-multisig-watch-only.md
│   ├── 50-next-steps/
│   │   ├── 51-where-to-go.md
│   │   └── 52-troubleshooting.md
│   └── 99-build-banner.md
├── transcripts/                    # SYMLINK → ../manual/transcripts/
├── tests/
│   ├── lint.sh                     # local trimmed copy (markdownlint + cspell + lychee only)
│   └── verify-examples.sh          # SYMLINK → ../../manual/tests/verify-examples.sh
├── .cspell.json                    # SYMLINK → ../manual/.cspell.json
├── .markdownlint-cli2.jsonc        # SYMLINK → ../manual/.markdownlint-cli2.jsonc
├── .puppeteer.json                 # SYMLINK → ../manual/.puppeteer.json
├── Dockerfile.build                # SYMLINK → ../manual/Dockerfile.build
├── agent-reports/
│   └── .gitkeep
└── FOLLOWUPS.md
```

**Symlink rationale.** Linux/Mac default; Windows requires `core.symlinks=true`. Documented in `README.md`. Local + CI both run on Linux, so this is operational not blocking. Updates to lint configs in the manual propagate to QuickStart automatically.

**Why mermaid is *not* shared.** Mermaid blocks for the QuickStart's newcomer audience may want different colours, simpler labels, or trimmed nodes. Copy-pasting respects that. Drift cost is bounded — both copies are visible in `git grep '^```mermaid'`.

## 5. Chapter structure (~34 PDF pages)

| File | Pages | Content |
|---|---:|---|
| `00-frontmatter.md` | 1 | title, "what you'll have at the end", prerequisites |
| **Part I — Foundations** | **6** | |
| `11-what-is-this.md` | 2 | the m-format star in 2 pages; mermaid: 4-card overview |
| `12-bitcoin-in-30-seconds.md` | 2 | seed phrase + xpub + descriptor inline primer |
| `13-the-three-cards.md` | 2 | ms1 / mk1 / md1 mapped onto BIP concepts; what each card *answers* |
| **Part II — Single-sig walkthrough** | **12** | |
| `21-install.md` | 1 | `cargo install --git`; smoke check |
| `22-generate-entropy.md` | 2 | DANGER box; how to make a real seed; "this guide uses canonical test seed" framing |
| `23-bundle.md` | 3 | `mnemonic bundle` walkthrough, output explained line-by-line |
| `24-verify.md` | 2 | `mnemonic verify-bundle` walkthrough |
| `25-stamp.md` | 2 | physical ceremony; colour-keyed plates; re-decode-after-stamp; mermaid: ceremony flowchart |
| `26-recover.md` | 2 | ms1 → phrase + watch-only side; forward-pointers |
| **Part III — 2-of-3 multisig** | **7** | |
| `31-why-multisig.md` | 2 | when single-sig isn't enough; air-gapped vs coordinator framing |
| `32-bundle.md` | 3 | `--template wsh-sortedmulti --threshold 2` walkthrough; mermaid: 3-cosigner flow |
| `33-stamp-and-recover.md` | 2 | which plates each cosigner holds; recovery quick-table |
| **Part IV — Watch-only** | **4** | |
| `41-singlesig-watch-only.md` | 2 | derive 2-card mk1+md1 bundle from phrase; importing into Sparrow / Bitcoin Core |
| `42-multisig-watch-only.md` | 2 | the air-gapped multisig coordinator flow (xpubs only); mermaid: air-gapped synthesis |
| **Part V — Next steps** | **4** | |
| `51-where-to-go.md` | 2 | pointers to reference manual chapters by topic |
| `52-troubleshooting.md` | 2 | condensed symptom → fix table (subset of manual Appendix G) |
| **Total** | **~34** | 16 chapters + frontmatter + build banner |

**Voice differences from manual:**

- Inline primers (3-sentence asides), not appendix forward-pointers.
- Single linear path: each chapter forward-points to the *next* one.
- BIP terms introduced *as needed*, not in a separate primer chapter.
- Power-user-only material (privacy-preserving, account-index, multipath family override) explicitly forward-pointed to the manual's relevant chapter.
- DANGER box once per chapter that uses the canonical seed; subsequent re-uses cross-reference.

## 6. Build pipeline

### 6.1 Makefile

Cloned from `docs/manual/Makefile`. Differences:

- `MANUAL_DIR` → `QUICKSTART_DIR`
- `SRC_DIR = $(QUICKSTART_DIR)/src`, `BUILD_DIR = $(QUICKSTART_DIR)/build`
- Output `m-format-quickstart.{md,pdf}` (not `m-format-manual`)
- `MD_SRC = $(shell find $(SRC_DIR) -type f -name '*.md' ! -name '99-build-banner.md' | LC_ALL=C sort)` (same exclusion pattern)
- `PANDOC_METADATA = --metadata-file=$(QUICKSTART_DIR)/pandoc/metadata.yaml`
- `MNEMONIC_BIN / MD_BIN / MS_BIN` defaults reuse manual's pattern (cargo run via symlinked workspace paths)
- `release-attach VERSION=quickstart-v0.1.0` recipe identical shape

`make pdf-docker` consumes the symlinked Dockerfile.build; reuses the same `mnemonic-manual-build:latest` image tag.

### 6.2 Lint trimming

`docs/quickstart/tests/lint.sh` is a *local trimmed copy* (not symlink), running 3 of the manual's 6 checks:

| Check | Source |
|---|---|
| markdownlint-cli2 | symlinked `.markdownlint-cli2.jsonc` |
| cspell | symlinked `.cspell.json` |
| lychee `--offline` | host-installed (CI installs in workflow) |

Drops: glossary-coverage (no formal glossary), flag-coverage (no CLI ref part), index-bidirectional (no `\index{}` markers).

`make verify-examples` symlinks to manual's `tests/verify-examples.sh` and consumes the symlinked transcripts dir; gives the 5-transcript drift check.

### 6.3 CI workflow `.github/workflows/quickstart.yml`

Clone of `manual.yml`. Differences:

```yaml
on:
  push:
    branches: [main, master]
    paths:
      - 'docs/quickstart/**'
    tags:
      - 'quickstart-v*'
  pull_request:
    paths:
      - 'docs/quickstart/**'
```

Same install steps (apt pandoc + texlive + chromium-browser; npm tools; lychee tarball with `--strip-components=1`). Same `/etc/puppeteer-config.json` write step. Same `gh release create --generate-notes` if absent + `gh release upload --clobber`. Different `working-directory: docs/quickstart` and asset path (`build/m-format-quickstart.pdf`).

### 6.4 Tag schedule

- `quickstart-v0.1.0-rc1` — A10b smoke test (delete after upload verified).
- `quickstart-v0.1.0` — final tag.
- Independent of manual's `manual-v*` tags.

## 7. Acceptance criteria (Q1-Q9)

| # | Check | Verification |
|---|---|---|
| Q1 | `make md` clean | markdownlint + cspell pass |
| Q2 | `make pdf` clean, **25-40 pages** | tighter band than manual's 60-100; QuickStart fails review if it spills past 40 |
| Q3 | TOC in both renders | `--toc --toc-depth=3` in pandoc args |
| Q4 | ≥4 mermaid blocks | one each in Parts I-IV; Part V (next-steps + troubleshooting) doesn't fit one naturally |
| Q5 | ≥3 `:::primer` boxes for newcomer asides | lower bar than manual's ≥6 since the QuickStart's main flow is already newcomer-aimed |
| Q6 | DANGER box on every chapter that uses the canonical seed | manually audited |
| Q7 | `make verify-examples` reproduces the 5 shared transcripts | OK 5 transcripts pass |
| Q8 | CI on push runs lint + PDF | `.github/workflows/quickstart.yml` exits 0 on PRs touching `docs/quickstart/` |
| Q9 | `quickstart-v*` tag triggers release-asset upload | verified once via rc tag, same A10b pattern as the manual |

## 8. Phase plan (high-level)

| Phase | Scope | Reviewer | Convergence |
|---|---|---|---|
| 0 | Scaffolding | architect (structural) | 0C/0I |
| 1 | Part I foundations (3 chapters) | reviewer | 0C/0I |
| 2 | Part II single-sig (6 chapters) | reviewer | 0C/0I |
| 3 | Part III multisig (3 chapters) | reviewer | 0C/0I |
| 4 | Part IV watch-only (2 chapters) | reviewer | 0C/0I |
| 5 | Part V next steps (2 chapters) | reviewer | 0C/0I |
| 6 | Polish + CI smoke + PR + tag + release | architect (integrated) | Q1-Q9 |

Per-phase reports persist to `docs/quickstart/agent-reports/phase-N-review-{1,2}.md`. Single-shot convergence acceptable when r1 reaches 0C/0I (manual cycle precedent).

No Phase 9 cross-repo equivalent. Existing `manual-cli-surface-mirror` already covers the underlying CLI surface that the QuickStart consumes.

## 9. Operational guardrails (carry-overs from manual cycle)

- Stage paths explicitly (`feedback_avoid_git_add_all`).
- Verify HEAD content after each commit (`feedback_verify_committed_content_not_working_tree`).
- Terse prose (`feedback_terse_code`).
- Re-grep the entire repo when applying review fixes (Phase 6→8 incomplete-fix lesson from manual cycle).
- Read package source before guessing config-file names (mermaid-filter `.puppeteer.json` lesson).
- Prefer host-installed CI over Docker for Chromium-dependent steps.
- Default to using `--strip-components=1` for tarballs that include a top-level dir.

## 10. Out-of-scope deferrals (filed in `docs/quickstart/FOLLOWUPS.md`)

| Item | Rationale | Re-visit |
|---|---|---|
| HTML output | markdown + PDF cover the immediate need | quickstart-v0.2 if user demand surfaces |
| Translations | scoped out for v0.1 | quickstart-v0.2+ |
| Side-by-side comparison with manual | QuickStart's job is onboarding, not contrast | not planned |
| Auto-extracted "next-step" links in `51-where-to-go.md` | hand-curated for v0.1 | quickstart-v0.2 |
| Inline glossary / index plumbing | newcomers read top-to-bottom; no need for index machinery | not planned |
