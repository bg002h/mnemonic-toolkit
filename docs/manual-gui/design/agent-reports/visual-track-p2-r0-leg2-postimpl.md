# P2 R0 + LEG-2 POST-IMPL WHOLE-DIFF REVIEW — visual-screenshot track, manual-gui leg

*(Opus-tier architect review, persisted 2026-07-01. COMBINED per-phase P2 R0 + Leg-2 post-implementation whole-diff review — combination deliberate and disclosed: P2 is Leg 2's only phase (precedent: the Leg-1 combined review at `visual-track-p1-r0-leg1-postimpl.md`). Inputs: plan `IMPLEMENTATION_PLAN_gui_visual_screenshot_track.md` P2 (lines 28–39) + risk notes (line 46); spec `SPEC_gui_visual_screenshot_track.md` §6/§7/§8; the P0-R0 advisories A1–A5; the Leg-1 review §5 owed-at-ship + m2; live branch `feat/manual-gui-visual-figures` @ `5eaa0116` (5 commits `a3cd7f0c` → `5eaa0116`, off master `22c6d5bb`, NOT pushed); GUI companion `mnemonic-gui@7e9dcca`; upstream root = `/scratch/code/shibboleth/mnemonic-gui` @ `7e9dcca` — VERIFIED valid: its only delta from tag `mnemonic-gui-v0.54.0` is `FOLLOWUPS.md` (+27 lines, the companion), `tests/snapshots/forms/` untouched, working tree clean. Every gate was re-run locally; every load-bearing claim re-derived from ground truth, not trusted.)*

## VERDICT: **RED — 0 Critical / 1 Important** (4 Minor). The Important is a 10-line, in-remit CI fold (the HTML embed census into `manual-gui.yml`'s build job — exact step text in I1); everything else on the branch is GREEN, adversarially re-verified. **Fold I1 → scoped convergence re-check → PR.** No other change owed on the branch.

---

## 1. The byte chain end-to-end — GREEN (verified in full, not spot)

- **Full 61/61 both-direction byte-compare** (independent of the lint script): every `docs/manual-gui/figures/gui/*.png` byte-identical (`cmp`) to `git archive mnemonic-gui-v0.54.0:tests/snapshots/forms/` — 0 mismatches forward, 0 missing reverse. Spot-sha256 cross-checked on 4 stems via `git show mnemonic-gui-v0.54.0:tests/snapshots/forms/<f>.png`: `mnemonic-inspect` = `057513b6084ded9f…`, `ms-vectors` = `77b72cadf3609755…`, `mnemonic-verify-bundle` = `e0fe0c42be8fbf0d…`, `md-encode` = `a1d3cf1640ef8b28…` — identical both sides.
- **Chain to the fleet-proven spike corpus:** `git diff 79a9074 mnemonic-gui-v0.54.0 -- tests/snapshots/` = EMPTY (the tag's corpus is object-identical to the Leg-1-reviewed branch tip, whose 61 blobs the Leg-1 review §2 verified as 100%-renames of the spike corpus, itself byte-verified to the run-1 CI artifact by the P0-R0). Chain unbroken: runner artifact → spike → `79a9074` → tag → `figures/gui/`.
- **Census:** exactly 61 PNGs; per-tab 32/10/10/9 (mnemonic/md/ms/mk); `figures/gui/` contains exactly 61 entries total — no stray files of any type.
- **Step-0 provenance INDEPENDENTLY REPRODUCED** (not read off the commit message): `git ls-remote` → tag SHA `d67f5d687b0da6352e8a940c2cf170a48fe8aa0d`; `gh api …/commits/<sha>/check-runs` → `snapshots` conclusions: `success` ×2 (push + tag-push events, both green). Recorded in `a3cd7f0c` exactly as the plan's step 0 (m2/m8) demands.

## 2. Gate re-runs (all executed by this review, `RUSTUP_TOOLCHAIN=stable`, upstream root = the verified v0.54.0-equivalent clone)

| Gate | Result |
|---|---|
| `make lint` (9 phases) | **GREEN 9/9** — markdownlint 0 err (92 files); cspell 0; lychee `2003 Total / 1998 OK / 0 Errors` (the 61 image links ARE in scope — proven by negative control, §4); gui-schema-coverage **982 anchors / 61 subcommands** (the plan's expected ZERO delta, live-verified); outline 129 OK; gui-form-xref 61/61, 0 orphans; **verify-figures-gui `OK (61/61 … census clean both directions)`** |
| `make verify-examples-gui` | **GREEN 61/61** renders match the pinned `gui-render`, no secret leak — structural track untouched, exactly as spec §8 requires |
| `make html` + embed census | **GREEN** — pandoc 3.6 local; census 61 (full analysis §3) |
| `make pdf` | **GREEN** — 444 pages; **61 unique `figures/gui/*.png` in the LaTeX log AND 61 raster images in the final PDF (`pdfimages -list`)**; 0 `Too many unprocessed floats`; visual spot-open §5 |
| `make md` | **GREEN** (722 KB output) |
| `verify-figures-gui` bite cases | **RE-PROVEN LIVE** (§4) |

## 3. The embed-census substitute — **RULING: APPROVED, equivalent-or-stronger**

The plan's literal `grep -c 'img src="data:'` reads **0** under pandoc 3.6 (re-verified myself: 3.6 emits `<img role="img" … src="data:…">`). The implementer's substitute — `grep -c 'src="data:image/png'` == 61 **plus** `<img` count == 61 — was interrogated against both false-inflation and alternate-embed escape:

- **False inflation impossible in the current document:** total occurrences of `data:image/png` in ANY context = **61** — all inside `<img src=` attributes. `url(data:` in CSS = 0; `data:image/svg` = 0; `<figure>` = 61 = `<img>` = 61. The paired `<img>`-count census bounds the img population, so a hypothetical 62nd non-figure data-PNG img would trip the ==61 equality (exact-match, not ≥) — fail-closed in the inflation direction too.
- **Alternate-embed escape is fail-loud:** if a figure embedded via any non-`<img src>` mechanism (`<object>`, CSS), the src-census would read ≤60 → FAIL. Both directions closed.
- **Strictly better than the plan literal on版本 robustness:** `src="data:image/png` matches BOTH pandoc 3.1.3 (`<img src="data:…`, the CI apt version) and ≥3.2.1 (`<img role="img" src="data:…`) — the literal matches only <3.2.1. (`grep -c` counts lines, not occurrences; occurrence-count cross-check = 61 = line count, and any multi-img-per-line collapse would under-read → spurious FAIL, the safe direction.)
- The implementer's negative control (broken path → census 60) is consistent with pandoc's verified warn-and-exit-0 behavior.

## 4. `verify-figures-gui` soundness — GREEN; bite cases re-proven by this review

- **Script read in full** (`tests/lint.sh` phase 9, lines 193–249): stems via `find -maxdepth 1 -name '*.png'`; census both directions against `EXPECTED_GUI_RENDER_COUNT` (threaded from the Makefile's single site, `?= 61`, the same variable `verify-examples-gui` uses); `comm -23`/`-13` for orphan/gap with stems NAMED; `cmp -s` per shared stem; `err()` accumulates → `exit 1`. Guards fail-closed on unset/wrong `FIGURES_GUI` and on a pre-v0.54.0 upstream (missing `tests/snapshots/forms` → explicit FAIL naming the ≥v0.54.0 requirement — the pin-bump ordering trap is covered). Wiring: Makefile threads `FIGURES_GUI` + `EXPECTED_GUI_RENDER_COUNT` (mirroring the `TRANSCRIPTS_GUI` precedent per plan m6); banners renumbered 1/9…9/9 with no phase skipped; `manual-gui.yml` lint-job comment updated. CI needs no new step — the lint job's existing pinned depth-1 clone contains the corpus (spec §6 M5 ruling).
- **Bite case re-proven live by this review** (not trusted from the commit message): flipped the last byte of `figures/gui/ms-verify.png` + added `zz-orphan.png` → `make lint` → **3 FAILs** — `census: 62 PNGs, expected 61` + `orphan baseline: zz-orphan` + `byte drift … ms-verify` — exit 1. Restored; restored file re-`cmp`'d byte-identical to the tag; clean `make lint` back to 9/9 GREEN.
- **Leg-1 m2 closure CONFIRMED:** the orphan-baseline direction (a stray committed PNG upstream, invisible to the GUI job's rendered-`.new.png` census) is exactly the `only_snap`/`only_fig` + census arms proven above — the lagging net the Leg-1 review called for now exists and bites.
- **Lychee negative control (this review):** broke one image path → lychee `1 Error` naming the missing file → restored → 0 Errors. Proves the file-relative form is genuinely lychee-checked (the I5 constraint-map's load-bearing leg), and that lychee guards figure-path integrity in CI's lint job.

## 5. Template/preamble edits — scoped, side-effect-free, warning-profile-comparable

- **All four LaTeX changes live in `pandoc/preamble.tex` only** (`templates/manual.latex` untouched): `\graphicspath{{../src/75-gui-forms/}}` (the plan-I5-corrected prepend-as-written form — `build/` + `../src/75-gui-forms/` + `../../figures/gui/x.png` → the committed figure; verified by 61/61 log inclusions); Gin `\maxwidth`/`\maxheight`/`keepaspectratio` defaults (mirrors pandoc's default template; the custom template lacks the sizing block); the guarded `\pandocbounded` shim; `float` + `\floatplacement{figure}{H}`.
- **Mermaid non-interference VERIFIED at source:** `mermaid-cache-filter.lua:71-76` emits **raw LaTeX** `\includegraphics[width=\textwidth]{<absolute>.pdf}` — not a pandoc Image, not in a `{figure}` environment — so `\graphicspath` (absolute path), the Gin width default (explicit width given), and `[H]` (no float) all cannot touch it; exactly 1 mermaid block exists manual-wide. Code blocks/tables unaffected (`floatplacement{figure}` scopes to figure floats).
- **The shim is genuinely two-arm-guarded:** `\@ifundefined{pandocbounded}{…}{}` — pandoc <3.2.1 (CI 3.1.3) never emits the macro (definition inert); ≥3.2.1 emits it and the custom template doesn't define it (the shim supplies it; body verbatim from pandoc's default template; `\Gscale@div` available — graphicx loaded 30 lines earlier). The ≥3.2.1 arm is LIVE-PROVEN by this review's pandoc-3.6 PDF build; the 3.1.3 arm is structurally inert-by-construction and gets its live proof on the PR's CI run (noted in §8).
- **Warning-profile comparison vs a master (`22c6d5bb`) baseline built by this review in a clean worktree:** LaTeX Warnings 6 → 6 with an **IDENTICAL warning-type histogram** (diff of normalized warning lines = empty); the `There were undefined references` summary is **PRE-EXISTING on master** (1 → 1; specific `Reference '…' undefined` lines: 0 → 0 — a pre-existing quirk, not this leg's); Overfull hboxes 78 → 82 (+4, the long `xpub-search-*` index lines — cosmetic); `Too many unprocessed floats`: 0; pages 406 → 444.
- **PDF spot-open (plan gate):** page 381 = `mnemonic-verify-bundle` (the tallest form, 850×1253) — complete from `--network` to `Run`, no clipping, scaled to the text block, captioned `Figure 79.26`. Page 366 = `mnemonic inspect` (secret-bearing) — `--ms1` secret field EMPTY in the pixels, screenshot ABOVE the structural render (which shows `<masked>`), the designed pairing intact.

## 6. Ship-records accuracy + scope hygiene — GREEN

- **Embed placement mechanically verified:** all 61 `![…](../../figures/gui/<stem>.png)` embeds pair with their `{#gui-form-<stem>}` section anchors — **0 anchor↔stem mismatches** (scripted check, 32/10/10/9 per chapter); every image sits directly under its anchor, above the structural-render include, so no new anchors and `gui-form-xref`/`gui-schema-coverage` stay untouched-green (re-run, §2).
- **`pinned-upstream.toml`:** GUI pin → v0.54.0; the 4 implied CLI pins unchanged — and the claim "IDENTICAL to v0.53.0's" verified by diffing the two tags' own `pinned-upstream.toml`: **byte-identical files** (toolkit-v0.74.0 / md-cli-v0.11.0 / ms-cli-v0.13.0 / mk-cli-v0.11.0 == the manual's implied fields == `manual-gui.yml` Job-1b tags, untouched).
- **`750-overview.md` (plan m1):** dark theme / 2× / egui-content-only / frame = flags + positionals + Run / placeholder-lines-not-depicted — all present; the `[ slot editor: N rows ]` example is REAL transcript text (grep-verified in `transcripts/gui/mnemonic-bundle.gui` et al.).
- **CHANGELOG `[Unreleased]`:** factual throughout — **12-package dev-graph lock delta cited (P0-R0 A4 honored)**; census-both-directions, `--resource-path`/`\graphicspath`/Gin/`[H]`/shim, the pandoc-warns-exits-0 rationale, 982/61 zero delta, verify-examples-gui 61/61 — every claim re-verified above. (One clause becomes stronger after the I1 fold; optional touch-up noted there.)
- **FOLLOWUP `manual-gui-visual-screenshot-track`:** moved Open → Resolved 2026-07-01, both legs summarized accurately (incl. "now a required check on GUI master" — **live-verified: `gh api …/branches/master/protection` → required contexts `["snapshots"]`**), the human-UX non-goal restated per spec §7, companion cross-cited.
- **GUI companion `mnemonic-gui@7e9dcca`** (pushed to master): well-formed RESOLVED record-entry `gui-form-snapshot-corpus-manual-consumer` — declares the corpus a cross-repo API, documents the UPDATE_SNAPSHOTS/pin-bump lockstep consequence, the m2 downstream-tripwire location, and the step-0 provenance anchor; cross-cites the toolkit entry; trailers present. Both-ways convention satisfied.
- **Leg-1 §5 owed-at-ship items — ALL CONFIRMED DONE** before P2 step 0: PR #27 merged, protection rule live (re-verified above), release commit + tag `mnemonic-gui-v0.54.0` cut, tag-run `snapshots` green (re-verified §1). Nothing GUI-side remains.
- **Scope:** whole-leg diff = 73 files = 61 PNGs + exactly 12 expected files (workflow comment, CHANGELOG, FOLLOWUPS, Makefile, preamble.tex, pinned-upstream.toml, 5 gallery chapters, lint.sh); NOTHING outside `docs/manual-gui/` + `manual-gui.yml`; zero `transcripts/` or structural-render changes; no stray artifacts; all 5 commits carry both trailers; branch base = current master tip `22c6d5bb`.

## 7. The CI-census question (the implementer's own flag) — **RULING: FOLD NOW (Important I1)**

**Finding I1 (Important): the HTML embed census exists only as a local, one-time verification — the CI pipeline that BUILDS AND SHIPS the HTML has no embed gate.** `manual-gui.yml`'s `build` job (lines 291–302) runs `make pdf` + `make html` and verifies only `test -f`; the `release` job publishes THAT html to gh-pages (the GUI help-icon deep-link target, `url.rs:31-34`) and as a release asset. pandoc exits 0 on a missing image (empirically verified in-plan), so CI can go fully green while shipping an HTML whose 61 figures are broken relative links. Why this is Important, not a filed FOLLOWUP:

1. The plan's I5 fold is explicit: *"the HTML failure mode is SILENT — the embed census is the gate, not the build exit code"* (plan line 46) and the P2 gate line names *"the HTML embed census == 61"*. In this project a "gate" is CI-permanent by construction — every other P2 deliverable (byte-gate, xref, schema, lychee) is CI-wired. A census that ran once on a dev machine recreates the exact silent-failure window I5 targeted, permanently, for every future manual PR.
2. **The local census does not even certify the CI artifact.** Local = pandoc 3.6; CI = apt 3.1.3 — the very version divergence that zeroed the plan-literal grep. Nobody has verified 3.1.3's `--resource-path` + `--self-contained` embed path end-to-end; only an in-CI census can, and it does so automatically on this PR's first run.
3. In-remit: a gate-completeness fix, ~10 lines, in a file this branch already touches.

**The exact fold** — into the `build` job, after the `Build PDF + HTML` step (before `Verify build artifacts`, or merged into it):

```yaml
      - name: Embed census — 61 figure PNGs are data-URI-embedded in the HTML
        # pandoc exits 0 with a WARNING on a missing image, so a green
        # `make html` proves nothing about embeds (visual-track plan I5);
        # the positive census is the gate. `src="data:image/png` matches
        # both pandoc 3.1.3 (apt: <img src="data:...) and >= 3.2.1
        # (<img role="img" src="data:...).
        working-directory: docs/manual-gui
        run: |
          count=$(grep -o 'src="data:image/png' build/m-format-gui-manual.html | wc -l)
          echo "embed census: ${count} data-URI PNG figures (expect 61)"
          test "${count}" -eq 61
```

One step, build job only (the lint job's HTML is an internal input to phase 4; the build job's HTML is the shipped artifact). Optional same-commit touch-up: append "(CI-enforced in the build job)" to the CHANGELOG's census clause.

## 8. Findings

**Critical: none.**

**Important:**
- **I1 — fold the HTML embed census into `manual-gui.yml`'s build job** (§7; exact step given). Blocks the PR under the 0C/0I bar; trivially folded.

**Minor (none blocking):**
- **m1 — `verify-figures-gui` sees only `*.png`:** a stray non-PNG file in `figures/gui/` (notes.txt, a .jpg) is invisible to the census. The gate's charter is PNG byte-parity and review catches strays (this one verified 61-entries-exactly); note-only.
- **m2 — census-count default duplicated:** `lint.sh`'s `${EXPECTED_GUI_RENDER_COUNT:-61}` fallback duplicates the Makefile's `?= 61`. The Makefile always threads it, so the fallback fires only on direct script invocation; single-version-site discipline would drop the fallback or leave a comment. Note-only.
- **m3 — +4 Overfull hboxes** (long `xpub-search-*` index-column lines) — cosmetic; the pre-existing `There were undefined references` summary (present on master, 0 specific refs both sides) is NOT this leg's — pre-existing quirk, could be chased in a docs-hygiene pass someday.
- **m4 — the pandoc-3.1.3 shim/embed arm has no local proof** (local machine is 3.6-only): inert-by-construction for the shim, and the I1 census makes the PR's own CI run the live proof for the embed path. Nothing further owed.

## 9. What's owed at/after PR

1. **Fold I1** (the census step; optional CHANGELOG clause) → per the standing post-impl-fold rule, a **scoped convergence re-check** (the fold is one workflow step; the scoped check = step text matches §7 + `make lint`/`git diff` clean) — then PR.
2. **Commit this review report** to `design/agent-reports/` on the branch (or master) before/with the PR — the audit-trail rule.
3. **PR → `manual-gui.yml` green** — this is the branch's FIRST CI run: it live-proves the pandoc-3.1.3 arm (plain-`\includegraphics` PDF path + the 3.1.3 embed census via I1) and `verify-figures-gui` in CI's own pinned clone. Verify all 4 jobs (lint / verify-examples / verify-examples-gui / build) green before merge.
4. **Merge** per repo convention. No manual-gui tag is owed now (CHANGELOG rides `[Unreleased]`; the plan's Leg-2 sequence ends at merge).
5. **Nothing owed GUI-side or cross-repo** — companion pushed (`7e9dcca`), branch protection live (`["snapshots"]`), tag + tag-run verified (§1). The structural track's gates all re-verified untouched-green (spec §8 honored).

## 10. Independent verification log

| Claim | Ground truth checked | Result |
|---|---|---|
| Upstream root == tag | `git diff --stat mnemonic-gui-v0.54.0..HEAD` = FOLLOWUPS.md only; status clean | VALID |
| figures == tag corpus | full 61×`cmp` vs `git archive` of the tag + 4 spot-sha256 + reverse census | 61/61 IDENTICAL both directions |
| tag corpus == Leg-1 corpus | `git diff 79a9074 <tag> -- tests/snapshots/` | EMPTY |
| Step-0 (tag-run green) | `gh api …/d67f5d68…/check-runs` `snapshots` | `success` ×2 |
| Protection rule live | `gh api …/branches/master/protection` | contexts `["snapshots"]` |
| CLI pins identical | diff of the two tags' `pinned-upstream.toml` | BYTE-IDENTICAL |
| lint 9/9 / schema 982/61 / xref 61 | local `make lint` | GREEN (all banners) |
| verify-figures-gui bites | 1-byte flip + orphan → lint | 3 FAILs, stems named, exit 1; restored + re-cmp'd |
| lychee checks image links | broken-path negative control | 1 Error naming the file; restored → 0 |
| Embed census + inflation surface | occurrence-scans over built HTML (3.6) | 61 = imgs = figures = all-context data-PNGs; 0 CSS/svg URIs |
| Plan-literal grep dead on 3.6 | `grep -c 'img src="data:'` | 0 (role attr) — substitute APPROVED |
| verify-examples-gui | local run | 61/61, no secret leak |
| PDF embeds + no clipping | log (61 PNGs) + `pdfimages` (61) + pages 366/381 opened | GREEN; tallest complete; secret field empty |
| Warning profile vs master | clean-worktree baseline build @ `22c6d5bb` | identical histogram; undefined-refs PRE-EXISTING |
| Anchor↔stem pairing | scripted scan of 4 chapters | 61 paired, 0 mismatches |
| Scope / trailers | `git diff --name-only` (73 = 61+12) + `git log` | CLEAN; trailers ×5 |
| CI has no embed gate | `manual-gui.yml:291-302` read | CONFIRMED → I1 |

*Both repos left clean: figure perturbation + orphan removed and re-verified byte-identical to the tag; `753-ms.md` restored; master-baseline worktree removed; mnemonic-gui untouched (read-only). Evidence: branch `a3cd7f0c…5eaa0116`; tag `mnemonic-gui-v0.54.0` = `d67f5d68`; companion `7e9dcca`; local runs under pandoc 3.6 / TeXLive xelatex / lychee 0.24.x.*
