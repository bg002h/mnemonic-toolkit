# QuickStart FOLLOWUPS

QuickStart-local deferred-work tracker. Closes lockstep with QuickStart release cadence.

## Open

### `lint-sh-dead-args` — drop unused env-var passes (or add no-op arms)

**Tier:** quickstart-local
**Filed:** 2026-05-08 (Phase 0 r2 code-quality review, S-1)
**Status:** open

`docs/quickstart/Makefile`'s `lint` target accepts `MNEMONIC_BIN` / `MD_BIN` / `MS_BIN` env vars (inherited from `docs/manual/Makefile`'s call shape), and CI passes them through. QuickStart's `tests/lint.sh` only handles `markdownlint` / `cspell` / `lychee` and silently ignores the binary-presence flags. Either drop the env-var passes from the Makefile + CI step, or add no-op `case` arms in `lint.sh` for `glossary` / `flag-coverage` / `index-bidirectional` so a copy-paste from the manual's CI doesn't fail-open. ~5 lines.

### `quickstart-makefile-all-src-dead-variable` — cosmetic

**Tier:** quickstart-local
**Filed:** 2026-05-08 (Phase 0 r2 code-quality review, S-2)
**Status:** open

`Makefile` defines `ALL_SRC := $(wildcard src/*.md)` but never uses it. Inherited from `docs/manual/Makefile`. Cosmetic; not a regression. Resolve by either dropping the variable or wiring it into a real target (e.g., a future `make check-headers`).

### `quickstart-metadata-fontsize-redundancy` — cosmetic

**Tier:** quickstart-local
**Filed:** 2026-05-08 (Phase 0 r2 code-quality review, S-3)
**Status:** open

`pandoc/metadata.yaml` carries `fontsize` redundancy inherited from `docs/manual/pandoc/metadata.yaml`. Cosmetic; not a regression.

## Closed

### `mermaid-block-silently-dropped-from-pdf` — Resolved by manual-v0.1.9 / quickstart-v0.1.6 (figures-cache cycle)

**Tier:** cross-cutting (manual + quickstart)
**Filed:** 2026-05-08 (Phase 1 implementer concern, observed in `m-format-quickstart.pdf` chapter 11 + verified in shipped `manual-v0.1.0` chapter 11)
**Resolved:** 2026-05-09 (figures-cache cycle, bonus closure)

Original report: `mermaid-filter` exits clean but the rendered TeX has no `\includegraphics` reference for chapter 11's 4-card overview. The same dropout was present in `manual-v0.1.0` chapter 11.

**Bonus closure root-cause finding:** mid-cycle Spike B (figures-cache implementation, 2026-05-09) revealed the dropout was **not chapter-11-specific** — `qpdf --qdf` on `m-format-manual.pdf` (manual-v0.1.8 build, MERMAID_FILTER=on) showed `/Subtype /Form` count = 0 across the entire PDF, meaning **all 9 manual mermaid blocks were silently dropped**, not just chapter 11. The bug was in `mermaid-filter`'s pandoc-AST emission path — its synthesised `\includegraphics` references were apparently malformed in a way that xelatex silently skipped them (no error, no diagram). The cache-mode pipeline (introduced in this cycle) bypasses `mermaid-filter` entirely: the new pandoc Lua filter `docs/manual/pandoc/filters/mermaid-cache-filter.lua` emits its own `\includegraphics{cache/<sha>.pdf}` directly, and `qpdf --qdf` on the cache-mode build shows `/Subtype /Form` count = 9 — all diagrams successfully embedded. The published `manual-v0.1.9` PDF (released from this cycle) is the on-mode build, which still has the bug; once the `ci-chromium-drop` FOLLOWUP retires the on-mode leg in a future cycle, all releases will use the working cache-mode build.

## Other closed entries

### `pandoc-highlighting-macros-leaked-to-pdf` — toolchain (cross-cuts manual + quickstart) — RESOLVED

**Tier:** cross-cutting (manual + quickstart)
**Filed:** 2026-05-08 (Phase 1 implementer concern, observed in `m-format-quickstart.pdf` chapter 12 + verified in shipped `manual-v0.1.0` chapter 63)
**Resolved:** 2026-05-08 — user-reported, fixed in same session

Root cause: both `docs/manual/pandoc/preamble.tex` and `docs/quickstart/pandoc/preamble.tex` redefined the `Highlighting` Verbatim environment via `\DefineVerbatimEnvironment{Highlighting}{Verbatim}{breaklines, breakanywhere, fontsize=\footnotesize}` *without* `commandchars=\\\{\}`. Pandoc's syntax-highlighting macros (`\ExtensionTok`, `\NormalTok`, `\AttributeTok`, `\DataTypeTok`, `\OperatorTok`, etc.) require `commandchars=\\\{\}` on the Verbatim environment to be expanded; without it they render as literal text inside the PDF.

Fix: added `commandchars=\\\{\}` as the first option in both `\DefineVerbatimEnvironment{Highlighting}{Verbatim}{...}` blocks. Verified post-fix: `pdftotext` on freshly built PDFs returns no `*Tok` raw-text leaks; rendered code blocks now show clean syntax-highlighted output. Manual PDF page count dropped 129→121 pp; QuickStart 44→42 pp (highlighting-macros leak had bloated both).
