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

### `pandoc-highlighting-macros-leaked-to-pdf` — toolchain (cross-cuts manual + quickstart)

**Tier:** cross-cutting (manual + quickstart)
**Filed:** 2026-05-08 (Phase 1 implementer concern, observed in `m-format-quickstart.pdf` chapter 12 + verified in shipped `manual-v0.1.0` chapter 63)
**Status:** open

Inside fenced `text` code blocks, pandoc's `\NormalTok{...}` and `\textless{}` highlighting macros render as raw text rather than expanding. Caused by `docs/manual/pandoc/preamble.tex:50` overriding the `Highlighting` Verbatim environment in a way that suppresses pandoc's `$highlighting-macros$` definitions. Same issue is present in the published manual (BIP-32 primer line 69 has identical input and renders identically broken — verified via `pdftotext` on `manual-v0.1.0`). Inherited; not a Phase 1 regression. Resolve during a toolchain pass touching `pandoc/preamble.tex`.

## Closed

(none yet)
