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

### `mermaid-block-silently-dropped-from-pdf` — toolchain (cross-cuts manual + quickstart)

**Tier:** cross-cutting (manual + quickstart)
**Filed:** 2026-05-08 (Phase 1 implementer concern, observed in `m-format-quickstart.pdf` chapter 11 + verified in shipped `manual-v0.1.0` chapter 11)
**Status:** open

mermaid-filter exits clean (zero-byte `mermaid-filter.err`) but the rendered TeX has no `\includegraphics` / `\includesvg` reference for the chapter 11 4-card overview. Source `.md` correctly opens with the ` ```mermaid ` fence (Q4 source-grep check passes). The same dropout is present in the published `manual-v0.1.0` PDF at chapter 11 — root cause is in shared toolchain (mermaid-filter + pandoc + svg integration), not the QuickStart source. Investigate during a future toolchain pass touching `docs/manual/pandoc/preamble.tex` or the mermaid-filter integration. Does NOT block QuickStart v0.1 since the issue is pre-existing.

## Closed

### `pandoc-highlighting-macros-leaked-to-pdf` — toolchain (cross-cuts manual + quickstart) — RESOLVED

**Tier:** cross-cutting (manual + quickstart)
**Filed:** 2026-05-08 (Phase 1 implementer concern, observed in `m-format-quickstart.pdf` chapter 12 + verified in shipped `manual-v0.1.0` chapter 63)
**Resolved:** 2026-05-08 — user-reported, fixed in same session

Root cause: both `docs/manual/pandoc/preamble.tex` and `docs/quickstart/pandoc/preamble.tex` redefined the `Highlighting` Verbatim environment via `\DefineVerbatimEnvironment{Highlighting}{Verbatim}{breaklines, breakanywhere, fontsize=\footnotesize}` *without* `commandchars=\\\{\}`. Pandoc's syntax-highlighting macros (`\ExtensionTok`, `\NormalTok`, `\AttributeTok`, `\DataTypeTok`, `\OperatorTok`, etc.) require `commandchars=\\\{\}` on the Verbatim environment to be expanded; without it they render as literal text inside the PDF.

Fix: added `commandchars=\\\{\}` as the first option in both `\DefineVerbatimEnvironment{Highlighting}{Verbatim}{...}` blocks. Verified post-fix: `pdftotext` on freshly built PDFs returns no `*Tok` raw-text leaks; rendered code blocks now show clean syntax-highlighted output. Manual PDF page count dropped 129→121 pp; QuickStart 44→42 pp (highlighting-macros leak had bloated both).
