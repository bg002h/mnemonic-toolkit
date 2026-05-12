# tech-manual-v0.4 final review — r2

Date: 2026-05-11
Reviewer: feature-dev:code-reviewer (final r2)

## Summary

0C / 0I / 0L / 0N.

## r1 fix verification

### C-1 at §V.4.8 (`docs/technical-manual/src/50-rust-api/54-mnemonic-toolkit-api.md` line 431)

CONFIRMED. The bullet now states `ToolkitError` is `#[non_exhaustive]`, explicitly identifies `md_codec::Error` as the exception ("NOT `#[non_exhaustive]`"), lists correct derives (`Debug, Error, PartialEq, Eq`), cross-references §V.1.3.9, and cites the toolkit's exhaustive `md_codec_exit_code` match at `error.rs:174`.

### C-1 at glossary (`docs/technical-manual/src/60-back-matter/61-glossary.md` line 221)

CONFIRMED. The `non_exhaustive` entry now lists `mk_codec::Error`, `ms_codec::Error`, `ToolkitError`, `KeyCard`, `Payload`, `InspectReport`, and sibling structs — `md_codec::Error` is absent. The corrective parenthetical is present.

## Source-code check

`grep -n non_exhaustive crates/md-codec/src/error.rs` — no matches. The attribute is not present on `md_codec::Error` at HEAD.

## Manual-wide grep sweep

Pattern `md_codec.*non_exhaustive|non_exhaustive.*md_codec` across `docs/technical-manual/src/` returns exactly two hits — both the corrected sites carrying exception language. No residual false attribution.

## Cycle-exit checks (inherited; no regression)

- `make lint` 6/6 green.
- `make verify-examples` 15/15.
- PDF: 242pp, 842,175 bytes, SHA256 `ffaa29b94e21a32aa583345965d2366b75d93895d1eac457ae99335417f580cf`, byte-identical across two clean `SOURCE_DATE_EPOCH=1746921600` builds.

## Verdict

- [x] 0 C / 0 I / 0 L / 0 N — tag-ready
- [ ] Findings present — iterate r3
