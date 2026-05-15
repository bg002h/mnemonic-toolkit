# Phase P1.3 (Track M — markdownlint + cspell + lychee baseline) — R1 opus architect-reviewer

**Date:** 2026-05-15
**Branch:** `manual-gui-v1` (mnemonic-toolkit)
**Scope:** Re-verify R0's I-1 + N-1 + N-2 folds.

**Verdict:** **LOCK 0C / 0I / 1N / 1n.**

## Folds verified

| R0 finding | Status |
|---|---|
| I-1: MD041 first-line-heading blocker | **LOCKed** — `"first-line-heading": false` at line 26; rationale comment at lines 18-25 accurately cites SPEC §1.4 (tab dirs `40-mnemonic/` etc.) + §2.3 (H2-first `## \`<tab> <subcommand>\` {#tab-subcommand}`); CLI-manual one-file-per-binary contrast noted. |
| N-1: stale inherited comment | folded (rewritten in place) |
| N-2: missing pre-seed cspell terms | partially folded — 7 of 9 candidates added (`ComboBox`, `Wayland`, `xdg`, `monospaced`, `RepaintCause`, `ViewportBuilder`, `clipboard`); 2 explicitly deferred (`wgpu_hal`, keyboard-modifier strings) with correct reasoning. `egui_kittest` deferral correct (cspell subtoken split — `egui` + `kittest` both present). |
| N-3: glossary + index warn-and-skip | INTENTIONALLY deferred to P2.4 — guards remain warn-and-continue. |
| n-1: unused MNEMONIC_BIN/MD_BIN/MS_BIN/MK_BIN argv | not folded — reserved-for-future per lint.sh header comment; correct. |
| n-2: cspell `--no-summary` | not folded — cosmetic; correct. |

## H2-fixture verification

```
echo "## H2 heading {#x}" > docs/manual-gui/src/test-md041.md
make -C docs/manual-gui lint 2>&1 | grep -A2 "1/7 markdownlint"
```

Produces:

```
[lint] === 1/7 markdownlint ===
Linting: 1 file(s)
Summary: 0 error(s)
```

Fixture deleted post-verification. MD041 fix confirmed: an H2-first file produces 0 errors.

## Remaining findings

### N-1 — Phase-prefix comment vs printed-step drift in `lint.sh` (cosmetic)

Pre-existing from P1.2 R0; the section header comments at lint.sh:112 and :129 both read `# 6.` while the corresponding `step` calls print `6/7` and `7/7`. (FOLDED post-R1: line 129 comment corrected to `# 7. index bidirectional`.)

### n-1 — JSONC comment line-length

`.markdownlint-cli2.jsonc:18-25` rationale block at ~73c — no action needed.

---

**Final verdict:** **LOCK 0C / 0I / 1N / 1n.** P1.3 ready to ship. The MD041 fold neutralizes the only P2.4 blocker R0 raised; the cspell pre-seed reduces false-positive thrash; lychee on empty src/ behaves correctly. Proceed to P1.4 (Track G widget_help_icon kittest RED).

**Post-R1 cosmetic fold:** lint.sh:129 comment-number-prefix corrected from `# 6.` to `# 7.` (R1's N-1 finding).
