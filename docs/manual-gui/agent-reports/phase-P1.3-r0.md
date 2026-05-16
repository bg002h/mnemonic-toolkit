# Phase P1.3 (Track M — markdownlint + cspell + lychee baseline) — R0 opus architect-reviewer

**Date:** 2026-05-15
**Branch:** `manual-gui-v1` (mnemonic-toolkit)
**Scope:** §3.1 P1.3 sub-phase — `docs/manual-gui/.markdownlint-cli2.jsonc` (NEW, derived from CLI manual), `docs/manual-gui/.cspell.json` (23 GUI-specific words appended), `docs/manual-gui/tests/lint.sh` (cspell `--no-must-find-files` flag added).

**Verdict:** **ITERATE 0C / 1I / 3N / 2n.**

The three baseline phases run pass-clean on an empty manual as required and the `--no-must-find-files` flag is correctly applied. However, MD041 (`first-line-heading: true`) — inherited verbatim from the CLI manual — is a latent block on Track M P2.4 GREEN: SPEC §2.3 mandates per-subcommand `.md` files start with `## ` (H2), not `# ` (H1); MD041 in per-file mode will fire on every such file. The CLI manual gets away with MD041=on because it uses one-file-per-binary; the GUI manual's per-subcommand-file structure (~160 files per §3.2 P2.4) deviates.

---

## Critical

None.

## Important

### I-1 — MD041 `first-line-heading: true` will fail on every per-subcommand markdown file in P2.4

**Where:** `docs/manual-gui/.markdownlint-cli2.jsonc` lines 18-21.

The config inherits the CLI manual's `first-line-heading: true` setting along with the comment "Each chapter file's first line IS h1, but markdownlint is sometimes confused by leading raw blocks." That rationale is **CLI-manual-true, GUI-manual-false**:

- CLI manual: `docs/manual/src/40-cli-reference/41-mnemonic.md:1` = `# \`mnemonic\` reference`. ONE file per binary; H1 at line 1; MD041 happy.
- GUI manual (planned per SPEC §2.3 line 563): each subcommand section opens with `## \`<tab> <subcommand>\` {#tab-subcommand}` — an H2, not an H1. §3.2 P2.4 specifies ~160 markdown files corresponding to a tab/subcommand/flag tree, and the plan's chapter structure (§1.4) shows per-tab directories `40-mnemonic/`, `50-md/`, etc. — meaning per-subcommand sub-files. Each such sub-file's first heading is `##`, tripping MD041.

**Resolution:** disable MD041 in the GUI manual config with a comment citing SPEC §1.4 + §2.3. This is the minimum-touch fix at P1.3 close.

## Nice-to-have

### N-1 — Inherited MD041 comment text mismatches GUI manual reality

Companion to I-1. The comment "Each chapter file's first line IS h1" was never true for the GUI manual; rewrite inline along with the rule flip.

### N-2 — Missing GUI-source spelling terms likely to surface in P2

P2 prose will introduce GUI source-surface terms not yet covered: `ComboBox`, `egui_kittest`, `wgpu_hal`, `xdg`, `Wayland`, `monospaced`, `RepaintCause`, `ViewportBuilder`, `clipboard`. Pre-seed obvious ones at P1.3 close so the first P2.4 batch doesn't bounce.

### N-3 — Glossary + index phases warn-and-skip is correct at P1; should harden in P2.4

P2.4 will create `91-glossary.md` and `99-index-table.md`, naturally turning warn-paths into err-paths. No action needed at P1.3.

## Nit

### n-1 — Unused MNEMONIC_BIN/MD_BIN/MS_BIN/MK_BIN argv (carry-over from P1.1)

Already documented in P1.1 R0. Reserved-for-future, no action.

### n-2 — cspell `--no-summary` flag would silence the one-line summary

Cosmetic only; current output is concise.

---

## Verification matrix

| # | Claim | Source-of-truth | Result |
|---|-------|-----------------|--------|
| A | `.markdownlint-cli2.jsonc` parses as valid JSONC | direct read | **PASS** |
| B | Same four rules as CLI manual (MD013 off, MD051 off, MD041 on, MD024 siblings_only) | diff against CLI manual config | **PASS** — byte-identical inside `"config":{...}` except for the leading "GUI manual" comment swap. |
| C | Same ignore globs `build/**` + `tests/fixtures/**` | direct read | **PASS** |
| D | `.cspell.json` parses as valid JSON | direct read | **PASS** |
| E | 5 SPEC §2.1 G3-named terms present | grep `egui\|FormState\|MnemonicGuiApp\|kittest\|NodeValueComposite` | **PASS** — all 5 present. |
| F | All appended GUI-specific words | count of additions | **PASS** — covers SPEC G3 named terms + extras. |
| G | Inherited 140-word block byte-identical to CLI manual `.cspell.json` | diff | **PASS** |
| H | `--no-must-find-files` is a real cspell flag, suppresses only zero-files exit | upstream README | **PASS** |
| I | cspell version pinned to `^8` in Dockerfile.build supports `--no-must-find-files` | Dockerfile.build | **PASS** |
| J | lychee `[WARN] No files found` on empty src/ exits 0 | empirical | **PASS** |
| K | All three baseline phases silent on empty manual | empirical | **PASS** |
| L | No tool silently skipped on PATH-missing in CI | Dockerfile.build pins all three | **PASS** |

## Empirical reproducibility

Documented empirical lint output reproduces by direct construction; phases 1-3 silent on empty manual; phase 3 emits `[WARN] No files found` (lychee informational, exit 0); phases 4-5 RED with P1.1 + P1.2-reproduced exit codes; phases 6-7 warn-and-skip; aggregate `fail=1` from phases 4-5.

## Parse-time / compile-time hazards

- JSONC parse of `.markdownlint-cli2.jsonc`: valid.
- Strict JSON parse of `.cspell.json`: valid.
- `lint.sh` shell-level quoting: every variable expansion uses `"$VAR"`; `set -euo pipefail` ensures any subshell-pipeline failure surfaces.

---

**Final verdict:** **ITERATE 0C / 1I / 3N / 2n.** LOCK criterion (0C/0I) not met. After folding I-1 (disable MD041 + rewrite comment) and N-2 (pre-seed obvious cspell terms), this phase should LOCK cleanly.
