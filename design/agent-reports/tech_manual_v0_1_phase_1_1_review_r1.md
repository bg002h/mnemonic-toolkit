# tech-manual v0.1 Phase 1.1 review r1 — Foundations chapters (commit b617976)

Reviewed: 00-frontmatter.md, 00-disclaimer.md, 11-introduction.md, 12-the-m-format-star.md, 13-codex32-and-bch.md, 14-conventions-and-notation.md, 62-index-table.md, .cspell.json. Cross-checked against `bg002h/descriptor-mnemonic/crates/md-codec/src/bch.rs`, `crates/md-cli/src/cmd/encode.rs`, and `tests/vectors/wpkh_basic.phrase.txt`.

## Critical: 0

## Important: 2 (both folded inline)

### I1 — md1 has no long code; the chapter's "Regular/Long code" framing was false

**File:** `13-codex32-and-bch.md`. Source proof: `bg002h/descriptor-mnemonic/crates/md-codec/src/bch.rs:4` ("long code dropped along with v0.x") + `crates/md-cli/src/cmd/encode.rs:90-94` (`--force-long-code` is a no-op flag).

The chapter claimed md1 ships with both a regular and long code, described long-vs-regular dispatch via a header length indicator, and described `--force-long-code` as forcing the long code. None of this is true at v0.30+.

**Fix applied:** removed the two-code framing for md1; the error-detection table now lists md1/mk1 regular-only alongside ms1's BIP-93 regular + long. Replaced "Long vs regular code dispatch" section with a "Note on the retired long code" section explaining v0.12's retirement and the `--force-long-code` no-op flag.

### I2 — The polynomial generator constants do NOT differ between md1, mk1, and codex32

**Files:** `13-codex32-and-bch.md` + `12-the-m-format-star.md`. Source proof: `bch.rs:7-15` (`GEN_REGULAR` — the BCH(93,80,8) polynomial — is the same family as codex32) + `bch.rs:17` (`MD_REGULAR_CONST` — the per-format target residue) + `bch.rs:43-51` (`hrp_expand` — standard BIP-173 expansion prepended to polymod input).

The chapter claimed each format mixes its HRP into the polynomial generator constants, and that md1 and mk1 use *different polynomial coefficients* than codex32. Both claims are wrong. The actual fork: same generator polynomial, distinct target residue, standard BIP-173 HRP expansion prepended to the polymod input.

**Fix applied:** rewrote the "md1 and mk1: forked BCH plumbing" section to accurately describe the polymod path (generator unchanged; target residue = `MD_REGULAR_CONST = top-65-bits-of SHA-256("shibbolethnums")`; HRP via BIP-173 expansion). Rewrote the "HRP mixing" subsection to walk through the three-step verify path. Updated the §II.2-companion paragraph in `12-the-m-format-star.md`.

## Low: 2 (both folded inline)

### L1 — BIP-93 long code's consecutive-erasure count is 15, not 13

**File:** `13-codex32-and-bch.md` error-detection table. **Fix applied:** new table separates md1/mk1 (regular only) from ms1 regular and ms1 long; the ms1-long column carries 15.

### L2 — Duplicate `\index{m-format constellation}` markers in 00-frontmatter.md + 11-introduction.md

**Fix applied:** removed the marker from `00-frontmatter.md` (the frontmatter chapter now points readers to §I.1 for the definitional treatment); kept the marker in `11-introduction.md` per "first definitional use" convention.

## Nit: 3 (all folded inline)

### N1 — "retired in 2026-05-03" → "retired on 2026-05-03"

**Fix applied** in 13-codex32-and-bch.md (two places).

### N2 — "An mismatched dispatch" → "A mismatched dispatch"

**Fix applied** — moot after I1 deleted the section.

### N3 — m-format-star.md HRP-mixing inaccuracy (companion to I2)

**Fix applied** as part of I2.

## Positive findings retained

- Worked decode example phrase (`md1yqpqqxqq8xtwhw4xwn4qh`) matches `wpkh_basic.phrase.txt` exactly.
- BIP-93 regular-code guarantees (4 random / 8 known / 13 consecutive) accurate.
- `policy_id_stub` description correct.
- mc-codex32 retirement date (2026-05-03) consistent with CLAUDE.md.
- Mermaid figure in §I.2: forked-vs-direct boundary correctly drawn.
- §I.4 notation conventions well-defined and self-consistent.
- §I.1 cross-references (§II.1, §IV.2, §V.1) match SPEC §4.2 layout.
- Version-coverage table in §I.1 matches HEAD tags.

## Post-fix verification

- `make lint` 6/6 green.
- `make pdf` builds (48pp; +1pp from the rewrites).
- `SOURCE_DATE_EPOCH=1746921600 make pdf` byte-identical across runs.

## Disposition

Both Important findings reflected an *inaccurate read* of the BCH plumbing — substantive technical errors that would have misled cross-implementers. Both fixed in place; the chapter now accurately describes the v0.30 reference implementation. All Lows and Nits folded inline. 0C/0I/0L/0N at Phase 1.1 close.

**Operational note:** the Phase 1.1 chapter on BCH (13-codex32-and-bch.md) is the kind of content where reviewer rounds against the actual source code are essential, not optional. Future Part II / Part V chapters making similar wire-format / API-surface claims must cite specific source files line-by-line during drafting, not at review.
