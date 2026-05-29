# R0 review — SPEC_manual_anchor_dangler_cleanup.md (verbatim)

**Reviewer:** opus architect (R0, pre-implementation)
**Spec:** `design/SPEC_manual_anchor_dangler_cleanup.md`
**Cycle:** Cycle C (post-v0.37.8 cluster, FOLLOWUP `manual-anchor-dangler-backlog-cleanup`)
**Source SHA:** spec claims `origin/master = 9f72d2f` (post-Cycle-B); verified against working tree `master`.
**Sources cited:** [lychee anchor recipes](https://lychee.cli.rs/recipes/anchors/), [lychee fragment-anchor issue](https://github.com/lycheeverse/lychee/issues/1790)

## VERDICT: RED (3 Critical / 4 Important / 3 Minor)

The architectural insight (pandoc-GFM strips `{#id}` anchors, pandoc-HTML preserves them) is correct and well-targeted — Piece 1 is sound and dissolves both Class A AND a substantial fraction of Class D in one swing (lychee on HTML reads `id="..."` directly, side-stepping all pandoc-vs-lychee slug-rule disagreement). However, the spec's source-file claims for Piece 2 and Piece 3 are riddled with citation drift: the fix-recipes target patterns that DO NOT EXIST in src/, the wrapper-drop target file does not contain the references it claims, and the "9 unauthored worked-example-* targets" classification is factually wrong (37 `### Worked example` headings exist; the "danglers" are pandoc-auto-generated TOC links to auto-slugged headings, dissolved by the same Piece-1 architectural fix). These must be folded BEFORE implementation begins.

## Critical findings

### C1 — Class B sed-fix recipe targets a pattern that does not exist in src/

**Where:** `SPEC_manual_anchor_dangler_cleanup.md:65-66` (Piece 2, Class B)

**Claim:** "Fix: `sed -i 's|#welcome-to-the-m-format%20constellation|#welcome-to-the-m-format-constellation|g'` across src/."

**Verified state (grep evidence):**
- `grep -rn '%20' docs/manual/src/` → no matches
- `grep -rn 'welcome-to-the-m-format' docs/manual/src/` → 6 hits in `src/60-appendices/69-index-table.md`, all with LITERAL SPACE inside `(#…)` link targets

src/ contains literal spaces inside `(#…)` link targets; pandoc URL-encodes them to `%20` when emitting GFM (visible in `build/m-format-manual.md:10494,10505,10507,10508,10509,10511`). The spec's sed is a no-op against src/. Correct sed targets the literal space:

```
sed -i 's|(#welcome-to-the-m-format constellation)|(#welcome-to-the-m-format-constellation)|g' \
  docs/manual/src/60-appendices/69-index-table.md
```

**Fold:** rewrite Piece 2 Class B fix-recipe to target the actual src/ pattern (literal space, not `%20`); also enumerate the actual host file (`src/60-appendices/69-index-table.md`, 6 refs).

### C2 — Class B 2nd unique slug is mis-targeted (wrong heading slug)

**Where:** `SPEC_manual_anchor_dangler_cleanup.md:66` (Piece 2, Class B, slug 2)

**Claim:** the slug `#m-format%20constellation-vs-slip-39-vs-naked-bip-39-vs-shamir` should be sed-replaced with `m-format-constellation-vs-slip-39-vs-naked-bip-39-vs-shamir`.

**Verified state:** the actual heading at `src/50-comparing/54-mformat-vs-others.md:1` is `# m-format vs SLIP-39 vs naked BIP-39 vs Shamir`. Pandoc auto-slug: `m-format-vs-slip-39-vs-naked-bip-39-vs-shamir` — NOT `m-format-constellation-vs-...`. The two link references that need fixing:
- `src/50-comparing/51-format-decision.md:59`
- `src/10-foundations/11-welcome.md:94`

Both author-side guesses interpolate "constellation" that the heading doesn't contain.

**Fold:** correct the slug-2 fix recipe (drop "constellation"); enumerate the actual host files.

### C3 — Class C "9 `worked-example-*` links in `src/12-how-to-read.md`" is a fabricated citation

**Where:** `SPEC_manual_anchor_dangler_cleanup.md:43, 68, 106` (Piece 2, Class C)

**Verified state:**
1. `src/12-how-to-read.md` does NOT exist. Actual path is `src/10-foundations/12-how-to-read.md`.
2. The actual file contains NO `worked-example-*` references, NO "Skim path" list, NO bullets to drop.
3. `#worked-example` link targets exist only in `build/m-format-manual.md` (lines 223, 228, 236, …) where they are **pandoc-auto-generated TOC entries** pointing at auto-slugged `### Worked example` headings. There are **37** such headings across `src/40-cli-reference/*.md`.

The targets are NOT unauthored — they exist as real headings. The danglers occur because pandoc-GFM doesn't emit explicit anchor markers AND lychee's gfm slug rule disagrees subtly with pandoc's TOC slug rule (disambiguation of duplicates).

**Therefore:** Class C is not a "drop 9 dead links" problem — it's a CONSEQUENCE of the same Piece-1 architectural issue, fully dissolved by the HTML-target fix.

**Fold:** delete Class C from Piece 2; re-classify worked-example-* as Class A (architectural artifact); delete the proposed companion FOLLOWUP; expect Piece 1 to dissolve these. Update Class A count from 15 to ~24.

## Important findings

### I1 — Spec's ~102 residual Class D is over-stated; Piece 1 likely shrinks it substantially

The §3 parenthetical acknowledges this but Piece 3's baseline-snapshot file is being committed in the same ship as Piece 1 — meaning Piece 3's content depends on Piece 1's output. This is fine if executed in dependency order (build HTML first, capture baseline second), but the spec doesn't make the order explicit.

**Fold:** make explicit in §4 Piece 3 that the baseline file is captured AFTER Pieces 1 + 2 land. Acknowledge residual count may be much lower than ~102.

### I2 — 174-vs-169 number discrepancy between FOLLOWUP and spec

`FOLLOWUPS.md:92` ("174 errors") vs spec ("169 error references"). The 603 unique anchor references field matches across both, but error counts differ by 5. Per `feedback_grep_verify_during_fold_not_just_during_write` and `feedback_negative_claims_grep_the_term_itself`, drift in load-bearing empirical numbers suggests stale lift.

**Fold:** add a "(re-verified against current `build/m-format-manual.md`)" annotation; lock the canonical number.

### I3 — `--lua-filter` invocation for `html` target is partially-spec'd

```
$(PANDOC) ... --to html --standalone --toc --toc-depth=3 --lua-filter ... --metadata-file=... ...
```

The `--lua-filter ...` placeholder needs to enumerate concretely: should the new html recipe use `MD_FILTER_ARGS` (strip-latex + primer-box) or `PDF_FILTER_ARGS`? For lychee-only consumer, MD_FILTER_ARGS is correct (mirrors `md` recipe).

**Fold:** spell out the filter args explicitly: `$(MD_FILTER_ARGS) $(PANDOC_METADATA) --output $@ $(MD_SRC)`.

### I4 — Piece 3 baseline-shrink-warning has no enforcement mechanism

> "**OLD slug missing from current run** → exit 0 with `::warning::sibling-dangler: baseline shrunk`"

CI annotations don't block PRs unless explicitly required by a status check. The baseline becomes voluntary. Options:
- (a) Second CI job asserts no `::warning::sibling-dangler:` lines → exit 1.
- (b) Emit `git diff`-style suggested-baseline-file and assert match.
- (c) Accept the lagging-indicator gap explicitly.

**Fold:** decide on one and document. Option (c) acceptable if documented as acknowledged limitation.

## Minor findings (sub-threshold)

### M1 — `tests/anchor-check.sh` script-language choice unspecified
Pin `set -euo pipefail`, lychee output-parsing strategy, sort/diff invocation. Match pattern from `tests/lint.sh`.

### M2 — No mention of how `anchor-check` plays with `make audit` running under `quickstart.yml`
`quickstart.yml:75` runs `make lint` with stub binaries; `manual.yml:102` runs `make audit`. Verify quickstart workflow doesn't break.

### M3 — §6 "Files" section omits `cycle-prep-recon-*.md` deletion at end of cycle
Minor.

## Required folds (apply before R1)

1. **C1:** rewrite Piece 2 Class B fix-recipe — target literal-space (not `%20`); enumerate host file.
2. **C2:** correct slug-2 — drop "constellation"; enumerate host files.
3. **C3:** delete Class C as a separate piece; re-classify worked-example-* as Class A; delete companion FOLLOWUP.
4. **I1:** make Piece 3 baseline-capture order-explicit (after Pieces 1+2 land).
5. **I2:** reconcile 174-vs-169; lock the canonical number.
6. **I3:** spell out `$(MD_FILTER_ARGS)` in Piece 1.
7. **I4:** pick a baseline-ratchet enforcement strategy.

## Verifications confirmed (positive signals)

- ✅ Piece 1 architectural insight correct (pandoc-GFM strips `{#id}`; HTML preserves them).
- ✅ `make audit` extension is one-line trivial.
- ✅ `.github/workflows/manual.yml` auto-picks up new audit deps.
- ✅ PDF flow uses src/ directly via `--to latex`; non-disruptive.
- ✅ Only consumer of GFM intermediate beyond `make md` is informational mention at `src/10-foundations/12-how-to-read.md:56`.
- ✅ SemVer-PATCH no-bump matches `sibling-pin-check.yml` precedent.
- ✅ Synthetic-drift + synthetic-recovery tests well-shaped.
- ✅ One-way-ratchet baseline-shrink design correct (modulo I4 about enforcement).
- ✅ No `noqa`-style escape hatch correct.

## Reviewer-loop expectation

This R0 finds 3C/4I/3M → RED. Per CLAUDE.md mandatory-pre-impl-R0 gate, **NO code lands until R1 GREEN**. After folding C1-C3 + I1-I4, re-dispatch R1. Expected convergence: 1-2 more rounds.
