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

(none yet)
