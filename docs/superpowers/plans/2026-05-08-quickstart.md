# m-format Quick Start v0.1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build and ship a 25-40pp newcomer-aimed Quick Start guide as a parallel artifact at `docs/quickstart/` in `bg002h/mnemonic-toolkit`, sharing the manual's toolchain and worked-example transcripts via symlinks, on its own `quickstart-v*` tag schedule.

**Architecture:** Parallel layout to `docs/manual/`. `.cspell.json` is local with `extends` to manual's; other configs (markdownlint, puppeteer, Dockerfile, lua filters, verify-examples.sh, transcripts/) symlink to manual. Pandoc PDF render drops `--template` (uses pandoc default; no `\printindex`). CI workflow `.github/workflows/quickstart.yml` builds host-installed (no Docker), uploads PDF on `quickstart-v*` tags.

**Tech Stack:** Pandoc + xelatex + mermaid-filter + lychee + cspell + markdownlint-cli2. Same versions as manual. cspell `extends` resolves relative to config-file location.

**Spec:** `docs/superpowers/specs/2026-05-08-quickstart-design.md` (with reviewer reports `-review-1.md` and `-review-2.md`).

---

## Phase 0 — Scaffolding

### Task 0.1: Create directory tree

**Files:**

- Create: `docs/quickstart/{src/00-frontmatter.md is later, build/, agent-reports/, tests/, pandoc/, .github/workflows/}` directory structure (dirs only)

- [ ] **Step 1:** From `mnemonic-toolkit` repo root, on a fresh branch off master:

```bash
git checkout master
git pull origin master
git checkout -b quickstart/v0_1
mkdir -p docs/quickstart/{src/{10-foundations,20-singlesig,30-multisig,40-watch-only,50-next-steps},build,agent-reports,tests,pandoc/{filters,templates}}
touch docs/quickstart/agent-reports/.gitkeep
```

- [ ] **Step 2:** Verify tree:

```bash
find docs/quickstart -type d | LC_ALL=C sort
```

Expected output (12 directories):

```text
docs/quickstart
docs/quickstart/agent-reports
docs/quickstart/build
docs/quickstart/pandoc
docs/quickstart/pandoc/filters
docs/quickstart/pandoc/templates
docs/quickstart/src
docs/quickstart/src/10-foundations
docs/quickstart/src/20-singlesig
docs/quickstart/src/30-multisig
docs/quickstart/src/40-watch-only
docs/quickstart/src/50-next-steps
docs/quickstart/tests
```

### Task 0.2: Create symlinks

**Files:** symlinks back to `docs/manual/`

- [ ] **Step 1:** Create the 7 required symlinks:

```bash
cd docs/quickstart
ln -s ../manual/transcripts transcripts
ln -s ../manual/.markdownlint-cli2.jsonc .markdownlint-cli2.jsonc
ln -s ../manual/.puppeteer.json .puppeteer.json
ln -s ../manual/Dockerfile.build Dockerfile.build
ln -s ../../manual/pandoc/filters pandoc/filters
ln -s ../../manual/tests/verify-examples.sh tests/verify-examples.sh
cd ../..
```

- [ ] **Step 2:** Verify each symlink resolves to an existing file/dir:

```bash
ls -L docs/quickstart/transcripts/22-first-bundle.cmd
ls -L docs/quickstart/.markdownlint-cli2.jsonc
ls -L docs/quickstart/.puppeteer.json
ls -L docs/quickstart/Dockerfile.build
ls -L docs/quickstart/pandoc/filters/strip-latex-from-md.lua
ls -L docs/quickstart/pandoc/filters/primer-box.lua
ls -L docs/quickstart/tests/verify-examples.sh
```

All seven `ls -L` commands must succeed (exit 0).

### Task 0.3: Create local `.cspell.json` with `extends`

**Files:**

- Create: `docs/quickstart/.cspell.json`

- [ ] **Step 1:** Write the file:

```json
{
  "version": "0.2",
  "language": "en,en-US",
  "extends": "../manual/.cspell.json",
  "ignorePaths": [
    "build/**",
    "tests/fixtures/**",
    "agent-reports/**",
    "transcripts/**"
  ],
  "ignoreRegExpList": [
    "(ms1|mk1|md1)[a-z0-9]+",
    "0x[a-fA-F0-9]+",
    "https?://[^\\s)]+",
    "`[^`]+`"
  ],
  "words": []
}
```

- [ ] **Step 2:** Verify cspell can parse the file:

```bash
cspell --config docs/quickstart/.cspell.json --no-progress --no-summary --version
```

Expected: cspell version string, exit 0.

- [ ] **Step 3:** **Phase-0 verify-item from spec §4 rationale:** confirm `extends` resolves correctly. Create a one-line test markdown that uses a manual-only word (`mdframed`):

```bash
echo "The mdframed example." > /tmp/cspell-test.md
cspell --config docs/quickstart/.cspell.json --no-progress --no-summary /tmp/cspell-test.md
```

Expected: `Issues found: 0 in 0 files.` (proving `mdframed` was inherited from manual's word list via `extends`).

If `Issues found: 1 in 1 files.` (i.e., `mdframed` flagged unknown) → `extends` is not resolving. Fall back to absolute path: change `"extends": "../manual/.cspell.json"` to `"extends": "/scratch/code/shibboleth/mnemonic-toolkit/docs/manual/.cspell.json"` and re-test. If still broken, file an issue and try `cspell.config.yaml` at repo root.

### Task 0.4: Create local trimmed `tests/lint.sh`

**Files:**

- Create: `docs/quickstart/tests/lint.sh`

- [ ] **Step 1:** Write the lint script (~50 lines; markdownlint + cspell + lychee only):

```bash
#!/usr/bin/env bash
# Trimmed lint for docs/quickstart/. Skips manual-only checks:
# glossary-coverage (no glossary), flag-coverage (no CLI ref part),
# index-bidirectional (no \index{} markers).
set -euo pipefail

for arg in "$@"; do
  case "$arg" in
    SRC_DIR=*)   SRC_DIR="${arg#*=}" ;;
    TESTS_DIR=*) TESTS_DIR="${arg#*=}" ;;
  esac
done

: "${SRC_DIR:?SRC_DIR is required}"
: "${TESTS_DIR:?TESTS_DIR is required}"

QUICKSTART_DIR="$(dirname "$TESTS_DIR")"
fail=0

step() { printf '\n[lint] === %s ===\n' "$1"; }
warn() { printf '[lint] WARN: %s\n' "$1" >&2; }
err()  { printf '[lint] FAIL: %s\n' "$1" >&2; fail=1; }

step "1/3 markdownlint"
if command -v markdownlint-cli2 >/dev/null; then
  ( cd "$QUICKSTART_DIR" && markdownlint-cli2 "src/**/*.md" "!build/**" "!tests/fixtures/**" ) || err "markdownlint reported issues"
else
  warn "markdownlint-cli2 not on PATH; skipping"
fi

step "2/3 cspell"
if command -v cspell >/dev/null; then
  ( cd "$QUICKSTART_DIR" && cspell --no-progress --no-summary "src/**/*.md" ) || err "cspell reported issues"
else
  warn "cspell not on PATH; skipping"
fi

step "3/3 lychee"
if command -v lychee >/dev/null; then
  lychee --offline --no-progress "$SRC_DIR" || err "lychee reported issues"
else
  warn "lychee not on PATH; skipping"
fi

if [ "$fail" -ne 0 ]; then
  printf '\n[lint] FAILED\n' >&2
  exit 1
fi
printf '\n[lint] OK\n'
```

- [ ] **Step 2:** Make it executable:

```bash
chmod +x docs/quickstart/tests/lint.sh
```

### Task 0.5: Create `pandoc/preamble.tex` (quickstart-tuned, no makeindex)

**Files:**

- Create: `docs/quickstart/pandoc/preamble.tex`

- [ ] **Step 1:** Read `docs/manual/pandoc/preamble.tex` to use as baseline:

```bash
cat docs/manual/pandoc/preamble.tex
```

- [ ] **Step 2:** Write `docs/quickstart/pandoc/preamble.tex` mirroring the manual's preamble but **without** `makeidx` setup. Contents to include:
    - `\usepackage{mdframed}` (primer-box render)
    - `primerbox` and `dangerbox` mdframed environments
    - `\usepackage{svg}` (mermaid SVG embedding)
    - `\usepackage{fvextra}` and the `\DefineVerbatimEnvironment{Highlighting}{Verbatim}{breaklines,breakanywhere,fontsize=\footnotesize}` block (for code-block wrapping)
    - **Do not** include `\usepackage{makeidx}` or `\makeindex` (no index in QuickStart)

### Task 0.6: Create `pandoc/metadata.yaml`

**Files:**

- Create: `docs/quickstart/pandoc/metadata.yaml`

- [ ] **Step 1:** Write the file:

```yaml
title: "m-format Quick Start"
subtitle: "Engrave your first 3-card backup in 90 minutes"
author: "bg002h"
date: ""
documentclass: book
classoption:
  - oneside
mainfont: "DejaVu Serif"
sansfont: "DejaVu Sans"
monofont: "DejaVu Sans Mono"
fontsize: 11pt
geometry:
  - margin=1in
colorlinks: true
linkcolor: NavyBlue
urlcolor: NavyBlue
toccolor: black
header-includes:
  - \usepackage{fvextra}
  - \DefineVerbatimEnvironment{Highlighting}{Verbatim}{breaklines,breakanywhere,fontsize=\footnotesize}
```

(Note: no `\usepackage{makeidx}` or `\makeindex` in `header-includes` — the no-index decision per spec I-1.)

### Task 0.7: Create `Makefile` (clone of manual's, with the documented diffs)

**Files:**

- Create: `docs/quickstart/Makefile`

- [ ] **Step 1:** Read the manual's Makefile as baseline:

```bash
cat docs/manual/Makefile
```

- [ ] **Step 2:** Write `docs/quickstart/Makefile` with these diffs from the manual's:
    - `MANUAL_DIR` → `QUICKSTART_DIR`
    - `SRC_DIR := $(QUICKSTART_DIR)/src` (etc.)
    - Output filename: `m-format-quickstart` (not `m-format-manual`)
    - `DOCKER_IMAGE ?= mnemonic-quickstart-build:latest` (per spec C-2)
    - **Drop `--template=$(TEMPLATES_DIR)/manual.latex`** from the `pdf` recipe's pandoc invocation (per spec I-1)
    - `release-attach VERSION=quickstart-v0.1.0` recipe identical shape

The PDF render command in the new Makefile should look like:

```makefile
$(BUILD_DIR)/m-format-quickstart.tex: $(PDF_SRC) $(FILTERS_DIR)/primer-box.lua $(PANDOC_DIR)/metadata.yaml $(PANDOC_DIR)/preamble.tex | $(BUILD_DIR)
	$(PANDOC) \
		--from markdown \
		--to latex \
		--standalone \
		--toc \
		--toc-depth=3 \
		--top-level-division=chapter \
		-H $(PANDOC_DIR)/preamble.tex \
		$(PDF_FILTER_ARGS) \
		$(PANDOC_METADATA) \
		--output $@ \
		$(PDF_SRC)
```

(Note: no `--template` flag.)

- [ ] **Step 3:** Verify Makefile syntax:

```bash
make -C docs/quickstart help 2>&1 | head -10
```

Expected: usage / target list output, exit 0.

### Task 0.8: Create 16 chapter stubs + frontmatter + build banner

**Files:**

- Create: 18 stub `.md` files under `docs/quickstart/src/`

- [ ] **Step 1:** Create each stub with a single H1 + one-line `(Phase N — to be authored.)` body. Mapping:

| File | H1 |
|---|---|
| `src/00-frontmatter.md` | `# About this Quick Start` |
| `src/10-foundations/11-what-is-this.md` | `# What you're building` |
| `src/10-foundations/12-bitcoin-in-30-seconds.md` | `# Bitcoin in 30 seconds` |
| `src/10-foundations/13-the-three-cards.md` | `# The three cards: ms1, mk1, md1` |
| `src/20-singlesig/21-install.md` | `# Install the toolkit` |
| `src/20-singlesig/22-generate-entropy.md` | `# Generating entropy safely` |
| `src/20-singlesig/23-bundle.md` | `# Producing your first bundle` |
| `src/20-singlesig/24-verify.md` | `# Verifying the bundle` |
| `src/20-singlesig/25-stamp.md` | `# Stamping the steel plates` |
| `src/20-singlesig/26-recover.md` | `# Recovering from the plates` |
| `src/30-multisig/31-why-multisig.md` | `# Why multisig` |
| `src/30-multisig/32-bundle.md` | `# Producing a 2-of-3 bundle` |
| `src/30-multisig/33-stamp-and-recover.md` | `# Stamping and recovering a 2-of-3 wallet` |
| `src/40-watch-only/41-singlesig-watch-only.md` | `# Watch-only single-sig` |
| `src/40-watch-only/42-multisig-watch-only.md` | `# Watch-only multisig (air-gapped)` |
| `src/50-next-steps/51-where-to-go.md` | `# Where to go from here` |
| `src/50-next-steps/52-troubleshooting.md` | `# Troubleshooting` |
| `src/99-build-banner.md` | `# Build banner` |

Each file body: `(Phase N — to be authored.)\n` (one line plus trailing newline).

### Task 0.9: Create README.md, FOLLOWUPS.md

**Files:**

- Create: `docs/quickstart/README.md`
- Create: `docs/quickstart/FOLLOWUPS.md`

- [ ] **Step 1:** `README.md` content:

```markdown
# m-format Quick Start guide

Newcomer-aimed onboarding for the m-format star. Sibling artifact to the reference manual at `docs/manual/`.

## Build

```sh
cd docs/quickstart
make pdf      # PDF (needs pandoc + xelatex + mermaid-filter on host)
make md       # concatenated GFM markdown
make lint     # markdownlint + cspell + lychee
make verify-examples MNEMONIC_BIN=… MD_BIN=… MS_BIN=…   # transcript drift check
```

## Contributor notes

Several configs are symlinked back to the reference manual (`../manual/`). The QuickStart's `.cspell.json` is a local file using cspell's `extends` key for its own word-list extensions without mutating the manual's config.

The symlinks require `git config core.symlinks true` (default on Linux/Mac; off on some Windows installs). If you're on Windows, set this flag before checking out the repo.
```

- [ ] **Step 2:** `FOLLOWUPS.md` content (mirror manual's tracker shape):

```markdown
# QuickStart FOLLOWUPS

Manual-local deferred-work tracker. Closes lockstep with QuickStart release cadence.

## Open

(none yet)

## Closed

(none yet)
```

### Task 0.10: Create `.github/workflows/quickstart.yml`

**Files:**

- Create: `.github/workflows/quickstart.yml`

- [ ] **Step 1:** Read the manual's CI workflow as baseline:

```bash
cat .github/workflows/manual.yml
```

- [ ] **Step 2:** Write `.github/workflows/quickstart.yml` with these diffs:
    - Top-of-file comment carried verbatim about `paths` filters not applying to tags
    - `paths: docs/quickstart/**` (push) instead of `docs/manual/**`
    - `paths: docs/quickstart/**, docs/manual/.markdownlint-cli2.jsonc, docs/manual/pandoc/filters/**` (pull_request) — the cross-paths set per spec I-2
    - `tags: quickstart-v*`
    - `working-directory: docs/quickstart` (host-build steps only)
    - Asset path `build/m-format-quickstart.pdf`
    - Same install steps + puppeteer-config write step + ensure-release-exists step

### Task 0.11: Smoke-test the scaffolding

- [ ] **Step 1:** From repo root, build the placeholder PDF:

```bash
cd docs/quickstart
make pdf
```

Expected: PDF emitted to `build/m-format-quickstart.pdf`. Will be a small (~3-5 page) document since stubs are 1 line each.

- [ ] **Step 2:** Run lint with placeholder binaries (verify-examples will be vacuous-pass since stubs have no shell commands):

```bash
make lint MNEMONIC_BIN=true MD_BIN=true MS_BIN=true
```

Expected: `[lint] OK` (3/3 checks pass).

- [ ] **Step 3:** Run verify-examples:

```bash
make verify-examples \
  MNEMONIC_BIN=/scratch/code/shibboleth/mnemonic-toolkit/target/release/mnemonic \
  MD_BIN=/scratch/code/shibboleth/descriptor-mnemonic/target/release/md \
  MS_BIN=/scratch/code/shibboleth/mnemonic-secret/target/release/ms
```

Expected: `OK (5 transcripts pass)` (consuming the symlinked transcripts/ dir).

- [ ] **Step 4:** Stage all and commit Phase 0:

```bash
cd ../..
git add docs/quickstart/ .github/workflows/quickstart.yml
git status --short
git commit -m "docs(quickstart): Phase 0 — scaffolding (parallel to docs/manual/)

Lays out docs/quickstart/ as a sibling artifact to docs/manual/.
Symlinks transcripts, markdownlint config, puppeteer config,
Dockerfile, pandoc filters, and verify-examples.sh back to the
manual. .cspell.json is a local file using cspell's extends key
(per spec C-1). Makefile drops --template (per spec I-1) and uses
mnemonic-quickstart-build:latest as DOCKER_IMAGE (per spec C-2).
CI workflow at .github/workflows/quickstart.yml clones manual.yml
with cross-paths watching docs/manual/.markdownlint-cli2.jsonc and
docs/manual/pandoc/filters/** (per spec I-2).

PDF builds clean with placeholder content (~3-5 pages). Lint
passes 3/3 (markdownlint, cspell, lychee). verify-examples passes
the 5 manual-shared transcripts via symlink.

16 chapter stubs + frontmatter + build banner ready for Phases 1-4.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

### Task 0.12: Phase 0 architect review

- [ ] **Step 1:** Dispatch `feature-dev:code-architect` (foreground) reviewing the scaffolding (file tree + symlinks + Makefile + CI workflow + Phase 0 commit). Prompt should ask the architect to:
  - Verify each symlink resolves (`ls -L` on each)
  - Verify the Makefile clone is correct relative to the spec's listed differences
  - Verify the CI workflow's `paths` filter shape
  - Verify the trimmed lint.sh matches spec §6.2 (3 checks; no glossary/flag-coverage/index)
  - Verify the cspell `extends` resolves a manual-only word

- [ ] **Step 2:** Persist the report to `docs/quickstart/agent-reports/phase-0-review-1.md`.

- [ ] **Step 3:** Apply any 0C/0I findings inline; commit with message `docs(quickstart): Phase 0 architect r1 fixes`. If findings introduce new criticals (manual cycle Phase 1 r2 lesson), iterate to round 2.

---

## Phase 1 — Part I foundations (3 chapters + frontmatter)

### Task 1.1: Author `00-frontmatter.md`

**Files:**

- Modify: `docs/quickstart/src/00-frontmatter.md`

- [ ] **Step 1:** Replace the stub with frontmatter content (~1pp):
    - Title: "About this Quick Start"
    - Audience: "you've heard of Bitcoin self-custody and want to engrave your first multi-card backup"
    - Prerequisites: "a Linux/Mac terminal; basic comfort with shell commands; no Bitcoin background needed"
    - "What you'll have at the end": single-sig steel-engraved bundle (Part II) + 2-of-3 multisig (Part III) + watch-only setups (Part IV)
    - Reading order: top-to-bottom, ~90 minutes
    - Pointer to the reference manual for deep dives

### Task 1.2: Author `11-what-is-this.md` (~2pp, includes mermaid)

**Files:**

- Modify: `docs/quickstart/src/10-foundations/11-what-is-this.md`

**Source materials to read first:**

- `docs/manual/src/10-foundations/11-welcome.md` (manual's equivalent — adapt the 4-card mermaid)
- `mnemonic-toolkit/README.md`

- [ ] **Step 1:** Author the chapter. Sections:
    - "The problem in one paragraph" — seed phrase fragility on steel
    - "The m-format answer in one paragraph" — three checksum-protected cards
    - mermaid block: 4-card overview (toolkit + 3 cards), copy-pasted from manual ch 11 with newcomer-tuned labels
    - "What this guide covers" — single-sig + multisig + watch-only forward-pointers
    - Forward-pointer to `12-bitcoin-in-30-seconds.md`

- [ ] **Step 2:** Confirm mermaid block opens with ` ```mermaid` (verifiable for Q4).

### Task 1.3: Author `12-bitcoin-in-30-seconds.md` (~2pp)

**Files:**

- Modify: `docs/quickstart/src/10-foundations/12-bitcoin-in-30-seconds.md`

**Source materials:**

- `docs/manual/src/60-appendices/62-bip39-primer.md`
- `docs/manual/src/60-appendices/63-bip32-primer.md`

- [ ] **Step 1:** Author. Sections (each ~3-4 sentences):
    - "Seed phrase" — what it does; the BIP-39 word list one-liner
    - "Extended public key (xpub)" — what it does; the "tree of keys" idea
    - "Wallet descriptor" — the spending rule; example shape `wpkh(xpub.../<0;1>/*)`
    - "BIP, what's a BIP?" — one-paragraph aside (BIPs are the Bitcoin spec system)
    - Forward-pointer to `13-the-three-cards.md`

### Task 1.4: Author `13-the-three-cards.md` (~2pp)

**Files:**

- Modify: `docs/quickstart/src/10-foundations/13-the-three-cards.md`

- [ ] **Step 1:** Author. Map the 3 BIP concepts (from ch 12) onto the 3 m-format cards:
    - ms1 = seed phrase entropy under BCH error correction
    - mk1 = xpub + origin (master fingerprint + path)
    - md1 = wallet descriptor as a BIP-388 wallet policy
    - "What each card *answers*" 3-row table (lifted from manual ch 11)
    - "Why three cards instead of one" — split-recovery property; cross-binding via `policy_id_stub` (one-sentence inline primer)
    - Forward-pointer to Part II install

### Task 1.5: Lint, commit, reviewer round

- [ ] **Step 1:** Run lint:

```bash
cd docs/quickstart
make lint MNEMONIC_BIN=true MD_BIN=true MS_BIN=true
```

Add new cspell entries to `.cspell.json`'s local `words` array as needed (NOT to the manual's). Re-run lint until OK.

- [ ] **Step 2:** Build PDF and confirm chapters land:

```bash
make pdf
pdfinfo build/m-format-quickstart.pdf | grep Pages
```

Expected: `Pages: 6-8` (foundation chapters added).

- [ ] **Step 3:** Stage paths explicitly and commit:

```bash
cd ../..
git add docs/quickstart/src/00-frontmatter.md docs/quickstart/src/10-foundations/ docs/quickstart/.cspell.json
git commit -m "docs(quickstart): Phase 1 — Part I foundations (3 chapters + frontmatter)

Authors:
- 00-frontmatter.md (about this Quick Start; audience + prerequisites)
- 11-what-is-this.md (4-card overview mermaid; what the guide covers)
- 12-bitcoin-in-30-seconds.md (seed phrase / xpub / descriptor inline primer)
- 13-the-three-cards.md (BIP concepts mapped onto ms1/mk1/md1)

cspell additions: <list>

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

- [ ] **Step 4:** Dispatch `feature-dev:code-reviewer` (foreground) on Part I. Prompt asks:
    - Newcomer voice — does each chapter assume zero Bitcoin background?
    - Inline primer correctness — BIP-39 / BIP-32 / descriptor claims technically right
    - 4-card mermaid — labels match the toolkit's actual surface
    - Forward-pointer chain (00 → 11 → 12 → 13 → install) intact

- [ ] **Step 5:** Persist report to `docs/quickstart/agent-reports/phase-1-review-1.md`. Apply 0C/0I findings; iterate if r1 surfaces new criticals.

---

## Phase 2 — Part II single-sig walkthrough (6 chapters)

### Task 2.1: Author `21-install.md` (~1pp)

**Files:**

- Modify: `docs/quickstart/src/20-singlesig/21-install.md`

**Source materials:**

- `docs/manual/src/20-quickstart/21-install.md`

- [ ] **Step 1:** Author. Lighter than the manual's chapter — focus on `cargo install --git` one-liner, smoke-check (`mnemonic --version`). Defer Docker / from-source paths to the manual.

### Task 2.2: Author `22-generate-entropy.md` (~2pp)

**Files:**

- Modify: `docs/quickstart/src/20-singlesig/22-generate-entropy.md`

- [ ] **Step 1:** Author. Sections:
    - "Why fresh entropy matters" — the brittleness of "I'll come up with words myself"
    - "How to generate entropy in production" — hardware wallet new-seed flow OR offline `bitcoinjs` browser; air-gapped device imperative
    - The DANGER box: re-authored in newcomer voice (per spec N-2). Explain *why* the canonical seed is public + swept, in one short paragraph
    - "For this guide, we're using the canonical test seed for reproducibility" — frame the canonical seed as a reading aid

- [ ] **Step 2:** Verify the DANGER box uses the `:::danger` fenced div syntax (matches the primer-box Lua filter).

### Task 2.3: Author `23-bundle.md` (~3pp)

**Files:**

- Modify: `docs/quickstart/src/20-singlesig/23-bundle.md`

**Source materials:**

- `docs/manual/src/20-quickstart/22-first-bundle.md`
- `docs/manual/transcripts/22-first-bundle.{cmd,out}` — the canonical worked example

- [ ] **Step 1:** Author. Sections:
    - "The command" — `mnemonic bundle --network mainnet --template bip84 --slot @0.phrase=...`
    - Explain each flag for a newcomer (don't assume `--template` is obvious)
    - "Output" — the actual transcript output, line-by-line
    - "Reading the output" — what each card group is; what the trailing summary is
    - Forward-pointer to verify

- [ ] **Step 2:** Confirm the bundle invocation matches `transcripts/22-first-bundle.cmd` so verify-examples still passes.

### Task 2.4: Author `24-verify.md` (~2pp)

**Files:**

- Modify: `docs/quickstart/src/20-singlesig/24-verify.md`

**Source materials:**

- `docs/manual/src/20-quickstart/23-verify.md`
- `docs/manual/transcripts/23-verify.{cmd,out}`

- [ ] **Step 1:** Author. Mirror the manual's structure but lighter on per-line-check explanations (the newcomer can read those off the output). Emphasis on "always verify before you stamp."

### Task 2.5: Author `25-stamp.md` (~2pp, includes mermaid)

**Files:**

- Modify: `docs/quickstart/src/20-singlesig/25-stamp.md`

**Source materials:**

- `docs/manual/src/30-workflows/31-singlesig-steel.md` (full ceremony)

- [ ] **Step 1:** Author. Condense the manual's 6-step ceremony into ~2 pages:
    - Mermaid: condensed ceremony flowchart (entropy → bundle → verify → stamp → re-decode → geographic separation)
    - Stamping discipline (one paragraph each: pick steel, use a magnifier, re-decode after each plate)
    - Where each plate goes: red/blue/green colour-key

### Task 2.6: Author `26-recover.md` (~2pp)

**Files:**

- Modify: `docs/quickstart/src/20-singlesig/26-recover.md`

**Source materials:**

- `docs/manual/src/20-quickstart/24-recover.md`
- `docs/manual/transcripts/24-recover.{cmd,out}` and `24-recover-mk1.{cmd,out}` and `24-recover-md1.{cmd,out}`

- [ ] **Step 1:** Author. Three steps:
    - Recover phrase from ms1 via `mnemonic convert --from ms1=… --to phrase`
    - Recover xpub/path/fingerprint from mk1 (this uses the v0.8 corrected `mnemonic convert --from mk1=…` path, not the broken `md decode --mk1`)
    - Decode descriptor from md1 via positional `md decode <STRINGS>`
    - Forward-pointer to multisig (Part III) and watch-only (Part IV)

### Task 2.7: Lint, commit, reviewer round

- [ ] **Step 1:** Run lint + verify-examples + PDF build (per Task 1.5 step 1-2). PDF should now be ~18-22 pages.

- [ ] **Step 2:** Stage and commit Part II + cspell additions.

- [ ] **Step 3:** Dispatch `feature-dev:code-reviewer` on Part II. Prompt:
    - CLI flags accurate against `cli-help/*.txt`
    - Newcomer voice maintained
    - Worked-example commands match the symlinked transcripts
    - Cross-references resolve

- [ ] **Step 4:** Persist report to `docs/quickstart/agent-reports/phase-2-review-1.md`. Apply 0C/0I findings; iterate if needed.

---

## Phase 3 — Part III multisig (3 chapters)

### Task 3.1: Author `31-why-multisig.md` (~2pp)

**Files:**

- Modify: `docs/quickstart/src/30-multisig/31-why-multisig.md`

- [ ] **Step 1:** Author. Sections:
    - "When single-sig isn't enough" — single seed = single point of compromise
    - "What 2-of-3 means" — two of three cosigners cooperate to spend
    - "Air-gapped vs coordinated" framing (forward-pointer to ch 42 for full air-gapped flow)
    - "Why 2-of-3 specifically" — 1-of-N defeats multisig's purpose, K=N loses recovery property
    - Forward-pointer to ch 32

### Task 3.2: Author `32-bundle.md` (~3pp, includes mermaid)

**Files:**

- Modify: `docs/quickstart/src/30-multisig/32-bundle.md`

**Source materials:**

- `docs/manual/src/30-workflows/32-multisig-2of3.md`

- [ ] **Step 1:** Author. Sections:
    - Mermaid: 3-cosigner flow (3 phrases → toolkit → ms1 + mk1 + md1 set)
    - The command: `mnemonic bundle --template wsh-sortedmulti --threshold 2 --slot @0.phrase=… --slot @1.phrase=… --slot @2.phrase=… --self-check`
    - Explain `--threshold`, `--slot @N.phrase=`, `wsh-sortedmulti` (newcomer terms; primer-box if needed)
    - Output: 7 cards = 3 × ms1 + 3 × mk1 + 1 × md1
    - DANGER box (re-authored newcomer voice; references chapter 22's box)

### Task 3.3: Author `33-stamp-and-recover.md` (~2pp)

**Files:**

- Modify: `docs/quickstart/src/30-multisig/33-stamp-and-recover.md`

- [ ] **Step 1:** Author. Sections:
    - Per-cosigner plate set: own ms1 + 3 mk1s + 1 md1
    - Recovery quick-table: what's still spendable / watch-only / bricked across damage scenarios (subset of manual ch 35's table)
    - Forward-pointer to Part IV

### Task 3.4: Lint, commit, reviewer round

- [ ] **Step 1-2:** lint + commit (per Task 1.5 / 2.7).

- [ ] **Step 3:** Dispatch reviewer; persist to `phase-3-review-1.md`; apply 0C/0I.

---

## Phase 4 — Parts IV + V (4 chapters)

### Task 4.1: Author `41-singlesig-watch-only.md` (~2pp)

**Files:**

- Modify: `docs/quickstart/src/40-watch-only/41-singlesig-watch-only.md`

**Source materials:**

- `docs/manual/src/30-workflows/34-watch-only.md`

- [ ] **Step 1:** Author. Sections:
    - "What a watch-only wallet is" (one paragraph; newcomer)
    - Two-step: derive xpub via `mnemonic convert --from phrase=… --to xpub --template bip84`, then `mnemonic bundle --slot @0.xpub=…` produces 2-card mk1+md1
    - Forward-pointer to wallet-export (manual ch 37) for Sparrow / Bitcoin Core import

### Task 4.2: Author `42-multisig-watch-only.md` (~2pp, includes mermaid)

**Files:**

- Modify: `docs/quickstart/src/40-watch-only/42-multisig-watch-only.md`

- [ ] **Step 1:** Author. Sections:
    - Mermaid: air-gapped synthesis (3 cosigner machines → coordinator with xpubs only → 4-card watch-only bundle, 3 mk1s + 1 md1)
    - Step 1 (per cosigner): derive xpub on own machine
    - Step 2 (coordinator): bundle from xpubs only — no seeds touched centrally
    - Step 3 (per cosigner separately): derive own ms1
    - Forward-pointer to manual ch 32 for the canonical full air-gapped multisig procedure

### Task 4.3: Author `51-where-to-go.md` (~2pp)

**Files:**

- Modify: `docs/quickstart/src/50-next-steps/51-where-to-go.md`

- [ ] **Step 1:** Author. Topic-keyed pointers to the reference manual chapters. Sections:
    - "Going deeper on workflows" — manual chs 31-38
    - "CLI reference" — manual chs 41-44
    - "Comparing m-format with other backup standards" — manual chs 51-57
    - "BIP primers" — manual chs 62-65
    - "Troubleshooting full matrix" — manual ch 67

### Task 4.4: Author `52-troubleshooting.md` (~2pp)

**Files:**

- Modify: `docs/quickstart/src/50-next-steps/52-troubleshooting.md`

**Source materials:**

- `docs/manual/src/60-appendices/67-troubleshooting.md` (subset)

- [ ] **Step 1:** Author. Five most common newcomer issues:
    - "I forgot `--threshold`" → fix
    - "verify-bundle says ms1_decode error at position N" → re-stamp that character
    - "Bitcoin Core won't import" → use --format bitcoin-core not bip388
    - "Wrong xpub for my wallet" → check `--template` / `--account`
    - "I'm on Windows and the symlinks broke" → set `core.symlinks=true`
    - Forward-pointer to manual ch 67 for the full matrix

### Task 4.5: Lint, commit, reviewer round

- [ ] **Step 1-3:** lint + commit + reviewer; persist to `phase-4-review-1.md`; apply 0C/0I.

---

## Phase 5 — Polish + CI smoke + tag + release

### Task 5.1: Final architect review

- [ ] **Step 1:** Build PDF; record page count:

```bash
cd docs/quickstart
make pdf
pdfinfo build/m-format-quickstart.pdf | grep Pages
```

Expected: 25-40 pages (Q2 acceptance criterion). If outside band, file in FOLLOWUPS or trim.

- [ ] **Step 2:** Run `make lint` + `make verify-examples` with real binaries; both must pass.

- [ ] **Step 3:** Dispatch `feature-dev:code-architect` on the integrated QuickStart. Prompt should walk through Q1-Q9 acceptance criteria. Persist report to `docs/quickstart/agent-reports/phase-5-review-1.md`.

- [ ] **Step 4:** Apply 0C/0I findings. Iterate if r1 surfaces new criticals.

### Task 5.2: Open umbrella PR

- [ ] **Step 1:** Push branch:

```bash
cd ../..
git push -u origin quickstart/v0_1
```

- [ ] **Step 2:** Open PR via `gh pr create --base master --head quickstart/v0_1 --title "docs: m-format Quick Start guide v0.1 (umbrella)" --body "<see template below>"`.

PR body template:

```markdown
## Summary

Lands `docs/quickstart/` as a parallel artifact to `docs/manual/`. Newcomer-aimed onboarding (~30pp) covering single-sig + 2-of-3 multisig + watch-only. Own `quickstart-v*` tag schedule.

## Test plan

- [x] `make pdf` produces 25-40 page PDF
- [x] `make lint` passes 3/3
- [x] `make verify-examples` reports OK 5 transcripts
- [ ] CI workflow runs cleanly on push
- [ ] `quickstart-v0.1.0-rc1` tag triggers release-asset upload (A10b smoke)
- [ ] After merge, `quickstart-v0.1.0` final tag triggers release upload

🤖 Generated with [Claude Code](https://claude.com/claude-code)
```

### Task 5.3: A10b smoke test via rc tag

- [ ] **Step 1:** Push rc tag:

```bash
git tag quickstart-v0.1.0-rc1 quickstart/v0_1
git push origin quickstart-v0.1.0-rc1
```

- [ ] **Step 2:** Watch CI run:

```bash
sleep 30
gh run list --workflow quickstart.yml --limit 2
gh run watch <RUN_ID> --exit-status
```

Expected: success. If failure, debug per manual cycle's Phase 8 lessons (Docker mount, lychee tarball, mermaid puppeteer config).

- [ ] **Step 3:** Verify PDF asset on rc release:

```bash
gh release view quickstart-v0.1.0-rc1 --json url,assets
```

Expected: `m-format-quickstart.pdf` asset uploaded.

- [ ] **Step 4:** Clean up rc:

```bash
gh release delete quickstart-v0.1.0-rc1 --yes
git tag -d quickstart-v0.1.0-rc1
git push origin :refs/tags/quickstart-v0.1.0-rc1
```

### Task 5.4: Merge PR

- [ ] **Step 1:** Confirm PR is mergeable + CI is green:

```bash
gh pr view <PR_NUMBER> --json mergeable,mergeStateStatus
gh pr checks <PR_NUMBER>
```

- [ ] **Step 2:** Merge:

```bash
gh pr merge <PR_NUMBER> --merge --delete-branch=false
```

### Task 5.5: Push final `quickstart-v0.1.0` tag

- [ ] **Step 1:** Fetch master + tag the merge commit:

```bash
git fetch origin master
git tag quickstart-v0.1.0 origin/master
git push origin quickstart-v0.1.0
```

- [ ] **Step 2:** Watch CI:

```bash
sleep 30
gh run list --workflow quickstart.yml --limit 2
gh run watch <RUN_ID> --exit-status
```

- [ ] **Step 3:** Verify final release + asset:

```bash
gh release view quickstart-v0.1.0 --json url,assets
```

Expected: `m-format-quickstart.pdf` asset present.

### Task 5.6: Update memory

- [ ] **Step 1:** Update `/home/bcg/.claude/projects/-scratch-code-shibboleth-descriptor-mnemonic/memory/MEMORY.md` and create `mnemonic_toolkit_quickstart_v0_1_state.md` documenting the cycle close, page count, release URL, and any lessons learned.

---

## Self-review

- [x] **Spec coverage** — every spec section has at least one task. D1-D5 covered. Q1-Q9 are gated in Phase 5 final review. §6.1 / §6.2 / §6.3 each have explicit Phase 0 tasks. §9 guardrails referenced in Phase 0 + Phase 5.
- [x] **No placeholders** — every task has either explicit code/content or specific source-material file paths to read first. The chapter-authoring tasks point at exact source files (`docs/manual/src/...`) for the engineer to ground voice and structure.
- [x] **Type / name consistency** — `DOCKER_IMAGE`, `MNEMONIC_BIN`, `MD_BIN`, `MS_BIN`, `quickstart-v0.1.0`, `quickstart-v0.1.0-rc1`, file paths all consistent across phases.
- [x] **Cspell `extends` verification** — Task 0.3 explicitly tests that `extends` resolves correctly with a manual-only word, with documented fallbacks if the assumption fails (per spec I-R1).
- [x] **A10b smoke test** — Task 5.3 mirrors the manual's smoke test pattern verbatim (rc tag → CI → verify asset → delete).
