# SPEC — manual anchor-dangler cleanup + fragment-gate enablement

**Cycle:** Cycle C of the post-v0.37.8 cluster (`manual-anchor-dangler-backlog-cleanup`).
**Tier:** `v0.37+-docs-hygiene`.
**SemVer:** test/docs/CI-only — no toolkit version bump.
**Source SHA at write time:** `origin/master` = `9f72d2f` (post-Cycle-B ship).

## 1 — Problem

The end-user manual at `docs/manual/` contains 128 unique intra-doc fragment-dangler references (`[text](#slug)` whose target heading doesn't exist). FOLLOWUP `manual-anchor-dangler-backlog-cleanup` recorded this and proposed enabling lychee's `--include-fragments` in CI once cleaned.

This cycle (a) discovers that ~15 of the 128 are **architectural artifacts of the build pipeline**, not authoring errors; (b) fixes the architectural and mechanical classes; (c) baseline-snapshots the residual ~97 to enable the gate going forward without rewriting every chapter.

## 2 — Brainstorm finding (cycle-prep): pandoc-GFM strips `{#id}` anchors

The current `make md` target emits `build/m-format-manual.md` via `pandoc --from markdown --to gfm`. **Pandoc's GFM emitter discards explicit `{#id}` heading anchors**: `## Foo {#my-id}` becomes `## Foo` in the GFM output, with the only anchor-target being pandoc's auto-derived slug from the heading text (`foo`).

Verified empirically:
```
$ echo '## Test heading {#my-test-id}' | pandoc --from markdown --to html
<h2 id="my-test-id">Test heading</h2>
$ echo '## Test heading {#my-test-id}' | pandoc --from markdown --to gfm
## Test heading
```

The src/ corpus defines 20 explicit `{#id}` anchors. Of these, **15 appear as dangling slugs in lychee's GFM output** because the emitter stripped them. Examples (verified by `comm -12 src-anchors danglers`):
- `{#jade-multisig}` — defined at `src/45-foreign-formats.md:607`
- `{#coldcard-multisig}`, `{#coldcard-singlesig}`, `{#electrum-wallet-file}`
- `{#bsms-round-2}`, `{#foreign-formats-not-supported}`
- `{#mnemonic-import-wallet-seed-overlay}`, `{#mnemonic-repair-max-indel}`
- (10 more — see `/tmp/lost-to-gfm.txt` in scratch — all valid src/ anchors that the GFM emit discards)

The fix is **architectural**: switch the fragment-gate's target from GFM markdown to HTML (which preserves `id="..."` natively).

## 3 — Dangler classification (re-counted post-finding)

`lychee --offline --include-fragments build/m-format-manual.md` reports **174 errors / 128 unique slugs / 734 total / 603 unique anchor references / 539 OK / 21 excluded** (canonical count locked here per R0 I2 fold; FOLLOWUPS.md:92 `174` is correct; the spec's earlier draft of `169` was a stale re-run number). Classified:

| Class | Unique slugs | Refs | Fix |
|---|---|---|---|
| A. Architectural (src `{#id}` + pandoc-GFM auto-slug mismatches both lost to GFM) | ~24 | ~30 | Piece 1: switch to HTML target |
| B. Authoring (literal-space-in-link-target → pandoc URL-encodes `%20`) | 2 | 8 | Piece 2: mechanical sed in src/ |
| D. Slug-guess / heading-rename residual | ~102 | ~137 | Piece 3: baseline-snapshot AFTER Pieces 1+2 land |
| **Total** | **128** | **174 (8 + ~30 + ~137 with some overlap accounted)** | |

**R0 C3 fold:** the prior draft's "Class C: 9 missing `worked-example-*` targets, drop links" was wrong on two counts: (a) `src/10-foundations/12-how-to-read.md` (corrected from mis-cited `src/12-how-to-read.md`) contains NO `worked-example-*` references, so there are no link wrappers to drop; (b) 34 real `### Worked example` headings exist across `src/40-cli-reference/*.md`, and the build-output danglers are pandoc-auto-generated TOC links pointing at auto-slugged forms that lychee's gfm slug rule disagrees with. This is the SAME architectural issue as Class A and dissolves under the Piece-1 HTML-target fix. Class C is folded into Class A; no companion FOLLOWUP needed.

**R0 I1 fold:** the Class A and Class D counts are interdependent; the ~102 Class D residual estimate pre-commits to a number that Piece 1 likely shrinks substantially (lychee on HTML reads `id="..."` directly, dissolving ALL pandoc-vs-lychee slug-rule disagreement, not just the 24 enumerated). The actual baseline-snapshot count for Piece 3 is captured AFTER Pieces 1+2 land in the working tree, against `build/m-format-manual.html` — see Piece 3 §4 below for the order-explicit recipe.

## 4 — Cycle structure (3 pieces, all in one ship)

### Piece 1 — switch fragment-gate to HTML emitter (architectural)

**New Makefile target `html`** alongside `md`:
- `docs/manual/Makefile`: add `html: $(BUILD)/m-format-manual.html` recipe using `pandoc --from markdown --to html --standalone --toc --toc-depth=3 $(MD_FILTER_ARGS) $(PANDOC_METADATA) --output $@ $(MD_SRC)`. Explicitly uses `$(MD_FILTER_ARGS)` (strip-latex + primer-box) NOT `$(PDF_FILTER_ARGS)` — the lychee-only consumer wants `\index{...}` LaTeX commands stripped, not preserved (R0 I3 fold). Mirrors `md` recipe exactly modulo `--to html`.
- New target `anchor-check` depends on `html`; invokes `bash $(TESTS_DIR)/anchor-check.sh BUILD_HTML=$(BUILD)/m-format-manual.html BASELINE=$(TESTS_DIR)/anchor-dangler-baseline.txt`.
- `audit` umbrella target depends on `lint`, `verify-examples`, AND `anchor-check`.

**CI workflow `.github/workflows/manual.yml`:** the existing "Audit manual" step (running `make audit`) automatically picks up `anchor-check` via the umbrella dependency — no workflow YAML change required.

**No effect on quickstart.yml** (R0 M2 fold): `quickstart.yml:75` runs `make lint` only (not `audit`), so it doesn't touch `anchor-check`. Stub-binary lint flow unaffected.

**Verification of fix:** running `lychee --offline --include-fragments build/m-format-manual.html` MUST recover the ~24 architectural anchors (Class A → 0 errors). Re-classify residual after this step — Class D count likely shrinks below ~102 because all pandoc-vs-lychee slug-rule disagreement dissolves under HTML.

### Piece 2 — mechanical authoring fixes

Only one class (B). R0 C3 fold deleted the prior "Class C" piece; those danglers fold into the Piece-1 architectural fix.

**B (literal-space-in-link-target → pandoc URL-encodes to `%20`):** the 2 unique build-output slugs `#welcome-to-the-m-format%20constellation` and `#m-format%20constellation-vs-slip-39-vs-naked-bip-39-vs-shamir` map to src/ link targets that contain LITERAL SPACES (NOT `%20` — that's what pandoc emits, not what the author wrote). R0 C1 + C2 folds enumerated the actual src/ hosts:

| src/ host | refs | slug fragment (literal-space form) | replacement |
|---|---|---|---|
| `src/60-appendices/69-index-table.md` | 6 (lines 15, 26, 28, 29, 30, 32) | `(#welcome-to-the-m-format constellation)` | `(#welcome-to-the-m-format-constellation)` |
| `src/10-foundations/11-welcome.md` | 1 (line 94) | `(#m-format constellation-vs-slip-39-vs-naked-bip-39-vs-shamir)` | `(#m-format-vs-slip-39-vs-naked-bip-39-vs-shamir)` |
| `src/50-comparing/51-format-decision.md` | 1 (line 59) | `(#m-format constellation-vs-slip-39-vs-naked-bip-39-vs-shamir)` | `(#m-format-vs-slip-39-vs-naked-bip-39-vs-shamir)` |

Note slug 2's actual heading (`src/50-comparing/54-mformat-vs-others.md:1 # m-format vs SLIP-39 vs naked BIP-39 vs Shamir`) drops "constellation" — author's slug guess was wrong on TWO counts (literal space AND "constellation" interpolation). Fix recipe:

```
sed -i 's|(#welcome-to-the-m-format constellation)|(#welcome-to-the-m-format-constellation)|g' \
  docs/manual/src/60-appendices/69-index-table.md

sed -i 's|(#m-format constellation-vs-slip-39-vs-naked-bip-39-vs-shamir)|(#m-format-vs-slip-39-vs-naked-bip-39-vs-shamir)|g' \
  docs/manual/src/10-foundations/11-welcome.md \
  docs/manual/src/50-comparing/51-format-decision.md
```

Verify: post-sed, `grep -rn '(#welcome-to-the-m-format constellation)' src/` and `grep -rn 'constellation-vs-slip' src/` both return zero hits.

### Piece 3 — baseline-snapshot residual

**Order-explicit (R0 I1 fold):** Pieces 1 + 2 land in the working tree FIRST. Only after `make html` produces `build/m-format-manual.html` AND the Piece-2 sed has run, the baseline is captured against the HTML output:

```
lychee --offline --include-fragments --no-progress build/m-format-manual.html 2>&1 \
  | grep '^\[ERROR\]' | grep -oE '#[^ ]+' | sed 's/^#//' \
  | sort -u > docs/manual/tests/anchor-dangler-baseline.txt
```

The residual after Pieces 1 + 2 is expected to be **much smaller** than the pre-cycle 128 unique slugs (Piece 1 dissolves Class A + a substantial fraction of Class D's slug-rule-mismatch artifacts; Piece 2 removes all 8 known Class B refs across 2 unique slugs). The exact post-piece-1+2 count is locked at Piece 3 capture time and pinned in the resolution narrative.

**Baseline-snapshot mechanism:**
- `docs/manual/tests/anchor-dangler-baseline.txt`: sorted one-slug-per-line file capturing every dangling slug AT THE TIME OF SNAPSHOT (post-Piece-1+2).
- `docs/manual/tests/anchor-check.sh` (new): runs lychee on `build/m-format-manual.html`; extracts error slugs; sorts; diffs against the baseline.
  - **NEW slug not in baseline** → exit 1 with `::error::anchor-check: <slug> (not in baseline; new authoring error). Re-run \`make html\` then \`lychee --include-fragments build/m-format-manual.html\` to reproduce; fix the offending link in src/.`
  - **OLD slug missing from current run** → exit 1 with `::error::anchor-check: baseline shrunk — slug '<slug>' no longer dangles; ratchet \`docs/manual/tests/anchor-dangler-baseline.txt\` to remove it.` This is R0 I4 fold option (a-hardened): make the ratchet ENFORCED, not voluntary. The baseline-shrunk case becomes a CI-blocking error that prompts the author to commit the smaller baseline in the same PR that fixed the dangler. This prevents the lagging-indicator gap that `feedback_fix_the_class_hunt_for_second_instance.md` and `feedback_schema_mirror_gui_lockstep_cumulative_gap.md` warn against.
  - Slugs unchanged from baseline → exit 0 silently.
- `Makefile`: `anchor-check` invokes the shell script.

**Script hardening (R0 M1 fold):** `anchor-check.sh` opens with `set -euo pipefail`; parses lychee output via the same `grep '^\[ERROR\]' | grep -oE '#[^ ]+' | sed 's/^#//' | sort -u` pipeline used for baseline capture (single-source-of-truth for the slug-extraction rule). No JSON parsing; matches the `tests/lint.sh` shell-script convention used by the existing harness.

Forward direction: every future PR that touches a heading or adds a link runs `make audit` → `anchor-check`; new slugs are blocked; baseline-shrinks force a same-PR ratchet of the baseline file.

### Piece 4 — manual-prose update (no, none required)

The user-visible manual prose doesn't reference the gate. The gate is CI-only. No chapter updates beyond the mechanical sed of Piece 2.

## 5 — Scope explicit non-goals

- **Not** authoring `worked-example-*` link targets (R0 C3 fold deleted this prior non-goal as misdirected; the worked-example references fold into Class A and dissolve under Piece 1).
- **Not** hand-fixing the residual slug-guess errors — baseline-snapshotted.
- **Not** removing existing lychee tooling — the offline-link-check stays; only the fragment-check gains the HTML-target + baseline.
- **Not** changing the published PDF output — the PDF flow uses LaTeX (preserves `{#id}` natively); this cycle only affects the CI gate.
- **Not** auto-fixing slug-guess errors via heuristic — the heuristic risks introducing wrong fixes; explicit author decision required per case.

## 6 — Files

**Added (4):**
- `docs/manual/tests/anchor-check.sh` — the baseline-snapshot gate.
- `docs/manual/tests/anchor-dangler-baseline.txt` — post-Piece-1+2 snapshot (sorted dangling slugs).
- `design/SPEC_manual_anchor_dangler_cleanup.md` — this spec.
- `design/agent-reports/manual-anchor-dangler-R0-review.md` — R0 verbatim.

**Modified (5):**
- `docs/manual/Makefile` — add `html` target + `anchor-check` target + extend `audit` dependency.
- `docs/manual/src/60-appendices/69-index-table.md` — 6 mechanical `(#welcome-to-the-m-format constellation)` → `(#welcome-to-the-m-format-constellation)` sed edits.
- `docs/manual/src/10-foundations/11-welcome.md` — 1 mechanical `(#m-format constellation-vs-slip-39-…)` → `(#m-format-vs-slip-39-…)` sed edit.
- `docs/manual/src/50-comparing/51-format-decision.md` — 1 mechanical sed edit (same as 11-welcome.md).
- `design/FOLLOWUPS.md` — flip `manual-anchor-dangler-backlog-cleanup` to `resolved`. (R0 C3 fold deleted the prior plan to add `manual-worked-example-anchor-targets-author` — those references dissolve under Piece 1, no follow-on FOLLOWUP needed.)

**End-of-cycle cleanup (R0 M3 fold):** `cycle-prep-recon-sparrow-name-+-manual-yml-pin-gate-+-anchor-backlog.md` (root) is a transient working note from this 3-cycle cluster; deleted at end of Cycle C ship.

## 7 — Test plan

1. `make html` cleanly emits `build/m-format-manual.html` with `<h2 id="my-id">...` for all src/ `{#id}` anchors.
2. `lychee --include-fragments build/m-format-manual.html` post-Piece-1 → reports fewer errors than the GFM baseline (recovers the ~24 architectural anchors: 15 explicit `{#id}` losses + ~9 worked-example-* TOC slug-rule mismatches). R1 M-b fold.
3. After Piece 2: literal-space references resolve (post-sed, `grep -rn '(#welcome-to-the-m-format constellation)' docs/manual/src/` and `grep -rn 'constellation-vs-slip' docs/manual/src/` both return zero hits). R1 M-a fold.
4. `tests/anchor-check.sh` against the working tree exits 0 silently (matches baseline).
5. Synthetic-drift: introduce a new dangler link in any src/ file → `anchor-check.sh` exits 1 with the `::error::` annotation naming the new slug.
6. Synthetic-recovery: fix an existing baseline dangler (rename or link-update) **without** ratcheting the baseline → `anchor-check.sh` exits 1 with the `::error::anchor-check: baseline shrunk — slug '<slug>' no longer dangles; ratchet docs/manual/tests/anchor-dangler-baseline.txt to remove it` annotation. The same PR MUST commit the ratcheted baseline to clear the gate (the I4-hardened enforcement at §4 Piece 3 line 106). R1 I-1 fold (was previously `exit 0 + ::warning::` — pre-I4-hardening wording).
7. `make audit` runs all three (lint + verify-examples + anchor-check) cleanly.

## 8 — SemVer disposition

CI-only PATCH; no version bump; no CHANGELOG entry; no GUI lockstep; no manual mirror; no sibling-codec companion. Matches the `sibling-pin-check.yml` precedent just landed.

## 9 — Reviewer-loop disposition

This spec dispatched to opus architect for R0 BEFORE implementation per CLAUDE.md mandatory pre-impl R0 gate. R0 must converge to 0C/0I before any Makefile / shell-script / source-edits land.
