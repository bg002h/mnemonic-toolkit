# Phase P2.4 batch 3 (Track M — 20-install) — R1 opus architect-reviewer

**Date:** 2026-05-15
**Branch:** `manual-gui-v1` (mnemonic-toolkit)
**Scope:** R1 verification of the 5 R0 folds — C-1 (MSRV 1.77→1.85), C-2 (Path C asset names + de-hedge), C-3 (4 FOLLOWUP ID corrections), I-1 (AirPlay overclaim soften), n-1 (resolved-FOLLOWUP framing). Plus `MSRV` added to cspell wordlist.

**Verdict:** **LOCK 0C / 0I / 0N / 0n.** Batch 3 promoted; executor proceeds to commit + check in with user before batch 4.

---

## Fold verification

| Fold | Status | Evidence |
|---|---|---|
| C-1 MSRV | PASS | `1.85` ×3 in chapters; `1.77` ×0; dead `rust-toolchain.toml` ref gone; `Cargo.toml:5` confirms `rust-version = "1.85"`; no `rust-toolchain*` file exists in mnemonic-gui repo |
| C-2 asset names | PASS | Canonical `${ARCH}-${OS}` order in all 3 chapters: `x86_64-linux` + `aarch64-linux` (21:73-74), `aarch64-macos` + `x86_64-macos` (22:57-58), `x86_64-windows` (23:60); "when published" ×0; aarch64-windows correctly not claimed (not in `.github/workflows/build.yml` matrix) |
| C-3 FOLLOWUP IDs | PASS | All 4 IDs resolve in `mnemonic-gui/FOLLOWUPS.md` (152, 157, 162, 180); fictional IDs `gui-macos-notarisation` / `gui-macos-code-signing` / `gui-windows-code-signing` ×0 in chapters |
| I-1 AirPlay | PASS | `22-macos.md:94-97` mentions `ScreenCaptureKit`, acknowledges per-window-vs-whole-display + `setSharingType` API limit; no absolute claim |
| n-1 resolved framing | PASS | `21-linux.md:96-101` uses "resolved entry `gui-glow-wayland-loop-broken` (v0.1.1 renderer swap)" framing |

## Hygiene

- `.cspell.json` — `MSRV` added. All other words from R0 (`EXCLUDEFROMCAPTURE`, `MSVC`, `notarisation`, `screencopy`, `USERPROFILE`, `xattr`) retained.
- Build artifacts fresh: HTML grep returns 5×`1.85`, 0×`1.77`, 1×`x86_64-linux`, 2×`gui-code-signing`.
- Cross-batch anchors `#how-the-gui-relates-to-the-four-clis` (chapter 12) and `#secret-handling` (chapter 14) still present and used as in-doc links from the install chapters.
- Lint phases 1-3 PASS / 4-5 at P1 baseline 459/59 (batch 3 adds no schema-driven content) / 6-7 WARN-skip.
- PDF render: 30 pages (up from 20 after batch 2).

## Final verdict

**LOCK 0C / 0I / 0N / 0n.** All 5 R0 folds are byte-correct. Schema cross-checks confirm the MSRV, asset names, and 4 FOLLOWUP IDs match `mnemonic-gui` source-of-truth. AirPlay claim properly hedged. n-1 resolved-framing applied. Batch 3 ready to commit; executor checks in with user before batch 4 (30-tour) per plan §3.5.
