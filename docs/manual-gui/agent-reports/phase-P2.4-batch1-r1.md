# Phase P2.4 batch 1 (Track M — 00-frontmatter + 00-disclaimer) — R1 opus architect-reviewer

**Date:** 2026-05-15
**Branch:** `manual-gui-v1` (mnemonic-toolkit)
**Scope:** R1 verification of the 4 R0 folds — C-1 (`wrap-long-code.lua` format gate), I-1 (TaggedOrIndexed in help-icon-contract), n-1 (`rekt` dedup), n-2 (`redeploys` removal). No new content under review; only fold correctness + non-regression.

**Verdict:** **LOCK 0C / 0I / 0N / 0n.** Batch 1 promoted; executor proceeds to commit batch 1 and dispatch batch 2 (`10-foundations/`) reviewer.

The four R0 folds land cleanly. The C-1 format gate is symmetric across both `Code` and `CodeBlock` handlers, uses the byte-identical `primer-box.lua:28` idiom, the LaTeX path still receives `\brktt{...}` wrappers (19 sites in `build/m-format-gui-manual.tex`), and the HTML path now renders all ≥12-char tokens that R0 documented as stripped — including the canonical help-icon-contract URL at HTML line 351. The I-1 reword at `src/00-frontmatter.md:62-63` enumerates all four affordance classes byte-for-byte matching the `needs_help_icon` predicate. The cspell file is clean: single `rekt`, no `redeploys`, prose at `00-frontmatter.md:76` uses `re-deploys` (cspell-passes via tokenization).

---

## Verification trace

1. **C-1 fold correctness — format gate symmetric on both handlers.** `if not FORMAT:match("latex") then return nil end` is at line 82 (top of `Code` body, before `has_long_run` check) and line 96 (top of `CodeBlock` body, before the line-walking loop). Both guards are the very first statement after the function signature. File-header rationale at lines 14-19 + 53-69 cites `primer-box.lua:28` lockstep. PASS.

2. **C-1 fold lockstep with primer-box.lua canonical idiom.** `primer-box.lua:28` reads `if FORMAT:match("latex") then` and `wrap-long-code.lua:82,96` reads `if not FORMAT:match("latex") then return nil end`. The `FORMAT:match("latex")` token is byte-identical to the canonical idiom; the inversion is correct (primer-box has a two-branch `if/else`, wrap-long-code wants an early-return guard). PASS.

3. **C-1 empirical re-verification — canonical URL + ≥12-char tokens survive in HTML render.** `grep -c '#mnemonic-convert-from' build/m-format-gui-manual.html` → 1; appears at HTML line 351 inside the help-icon-contract `<h2 id="the-help-icon-contract">` section as `<code>https://bg002h.github.io/mnemonic-toolkit/manual-gui/#mnemonic-convert-from</code>`. Spot-checks: `<code>mnemonic-gui</code>` → 5, `mnemonic-toolkit` → 4, `mnemonic-gui v0.3.0` → 1, `manual-gui-v` → 2, `manual-gui-MAJOR` → 1, `m-format-gui-manual-${MANUAL_GUI_VERSION}.pdf` → 1. Build-banner timestamp present at HTML line 369 inside `<code>...</code>`. All R0-documented strip sites recovered. PASS.

4. **C-1 PDF non-regression — LaTeX path still emits `\brktt`.** `build/m-format-gui-manual.tex` is kept; `grep -c brktt` → 19 (4 preamble macro-defs; 15 content-site wraps including `\brktt{mnemonic-gui}` x5, `\brktt{mnemonic-toolkit}` x2, `\brktt{mnemonic-gui v0.3.0}`, `\brktt{manual-gui-v*}` x2, `\brktt{https://bg002h.github.io/mnemonic-toolkit/manual-gui/...}` x2, `\brktt{m-format-gui-manual-\$\{MANUAL\_GUI\_VERSION\}.pdf}`, `\brktt{manual-gui-MAJOR/MINOR/PATCH}`, build-banner timestamp). LaTeX `\seqsplit`-wrap pipeline intact. PASS.

5. **I-1 fold correctness — TaggedOrIndexed added.** `src/00-frontmatter.md:62-63` reads "Every Dropdown / NodeValueComposite / TaggedOrIndexed / repeating-field flag in the GUI renders with a `?` button next to its label." The four-class set matches `mnemonic-gui/src/form/widget.rs:27-32` (`FlagKind::Dropdown(_) | FlagKind::NodeValueComposite(_) | FlagKind::TaggedOrIndexed(_)) || flag.repeating`). Plan §1.6 line 238-240 + 244 byte-aligned. PASS.

6. **n-1 fold correctness — no duplicate `rekt`.** `grep -n rekt .cspell.json` → exactly one match at line 90. The R0-flagged appended duplicate is gone. PASS.

7. **n-2 fold correctness — no `redeploys` cspell entry; prose uses hyphenated `re-deploys`.** `grep -n redeploys .cspell.json` → 0 matches. `grep -n re-deploys src/00-frontmatter.md` → line 76. cspell tokenization splits `re-deploys` to `re` + `deploys`, both in the standard English dictionary; no new cspell warnings. PASS.

8. **Lint phase 1-3 PASS confirmation.** Per executor's quoted output (phases 1-3 PASS clean; phases 4-5 RED at 459/59 P1 baseline; phases 6-7 WARN-skip per `tests/lint.sh:118-127,131-155` because `90-appendices/` is absent). PASS.

9. **No content regressions — TaggedOrIndexed cspell wordlist coverage.** `.cspell.json:143` declares `"TaggedOrIndexed"`. The I-1 reword introduces the token to prose for the first time; the wordlist entry was pre-existing and unchanged. cspell-passes. PASS.

10. **PDF render not visibly degraded by C-1.** Executor reports a minor 3pt Overfull `\hbox` for `m-format-gui-manual-${MANUAL_GUI_VERSION}.pdf`. This is the expected `\brktt` + `\seqsplit` wrap behavior on the long literal placeholder string. Pre-existing pattern, not introduced by the format gate. The C-1 fold strictly *adds* a guard; the LaTeX-path AST is unchanged from R0. PASS.

---

## Final verdict

**LOCK 0C / 0I / 0N / 0n.**

All four R0 folds are correct and self-consistent. The C-1 format gate is symmetric across `Code` + `CodeBlock`, uses the project-canonical `primer-box.lua:28` idiom byte-for-byte, restores every ≥12-char token to the HTML render (including the canonical help-icon-contract URL), and leaves the LaTeX `\brktt`/`\seqsplit` path fully intact (19 wrap sites in the .tex). I-1 enumerates all four affordance classes and matches `needs_help_icon` byte-for-byte. The two cspell nits are cleanly applied with no new warnings.

Batch 1 promoted. Executor proceeds to commit batch 1 (staged paths: `pandoc/filters/wrap-long-code.lua`, `src/00-frontmatter.md`, `src/00-disclaimer.md`, `.cspell.json`) and dispatch batch 2 (`10-foundations/`) reviewer per plan §3.5.
