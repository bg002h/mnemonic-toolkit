## Constellation Documentation Inventory — RECONCILED-LIVE @ toolkit `c630c933` (v0.72.0)

> R0-FOLD corrections vs the prior inventory are marked **[CORRECTED]** / **[R0-FOLD]**. All counts re-grepped live at `c630c933`.

All four user-facing books are **pandoc + xelatex** (NOT mdbook — 0 `book.toml`, 0 `SUMMARY.md`). Source = `docs/<book>/src/NN-section/NM-chapter.md` concatenated in `LC_ALL=C sort` order. Fonts uniform: DejaVu Serif/Sans/Mono @ 11pt. Mermaid figures pre-rendered to `figures/cache/` (CI runs `MERMAID_FILTER=skip`).

### B. CLI-OUTPUT BLOCK CENSUS — re-grepped live (the fidelity surface)

Output-shaped = ` ```text ` + ` ```json ` fence openers only. `sh`/`pwsh` are command-INPUT and NOT counted. Golden-backed = a sibling `.cmd`/`.out` transcript replays the binary (but never gates the prose paste — see C).

| Book | total fenced | output-shaped (text+json) **[live]** | golden `.cmd`/`.out` pairs (own) | output blocks transcript-backed | **to convert (re-derived)** |
|---|---:|---:|---:|---:|---:|
| manual | 349 | **115** (text 79 / json 36) | **20** (recursive incl. 2 subdirs `foreign-formats/`, `cross-format-recipes/`) + 14 `.err` | 20 (binary-gated; 0 prose-gated) | **~95** (115 − 20 golden − excerpt/illustration residual) |
| quickstart | 29 | **10** (text 10 / json 0) | **0** (`tests/verify-examples.sh` symlinks→manual's runner; covers NONE of its OWN prose) | 0 | **10** |
| technical-manual | 95 | **51** (text 37 / json 14) | **15** (11 CLI + 4 cargo-example `[[example]]` of crate `md-codec-examples`; NOT CI-run today) | 0 prose-gated | **~36** (+ unknown count of DRIFTED goldens to repair — see P1a) |
| manual-gui | 70 | **38** (text 36 / json 2) | **0** (`transcripts/` = `.gitkeep` only — confirmed live) | 0 | candidate ~27 after excluding ~9 ASCII-art mockups + ~2 truncated → backable re-derive in P4 from base 38 |
| **TOTAL** | **543** | **~214** | **35** | **35** (manual+tech binary side only) | **~150-155** |

**Blocks carrying binary-derived material (m-string/address/xpub):** manual 19 files, **quickstart 7 files [R0-FOLD: grep-verified — `md1`/`mk1`/`ms1` m-strings in 33-stamp-and-recover, 32-bundle, 25-stamp, 26-recover, 24-verify, 42-multisig-watch-only, 23-bundle]**, technical-manual 11, manual-gui 23 files (incl. mk1 fixtures + a bespoke `deadbeef`/`c0ffee00` fixture at ~7 sites). Canonical `abandon…about` seed (fp `73c5da0a`, NEVER-FUND) makes card content deterministic.

### C. The architectural gap (why "golden-backed" ≠ "gated")

`verify-examples` replays the binary and diffs against the `.out` ON DISK — it never reads `.md` prose. No book compares a prose fence to its golden `.out` (0 `lint.sh` refs to `.out`; 0 pandoc include filters). **Proven live drift:** the leading `warning: secret material on argv …` line appears in **4 `.out` + 1 `.err`** transcripts (`41-inheritance.out`, `24-recover.out`, `22-first-bundle.out`, `23-verify.out`, `cross-format-recipes/recipe-2-bitcoin-core-to-bundle.err`) and as documented warning-message LITERALS in `41-mnemonic.md` CLI-reference prose — but `22-first-bundle.md`'s prose transcript fence OMITS the warning that its `.out` line 1 carries (0 hits in that file's body). This is the confirmed canonical EXCERPT case. Makes the M1 "convert fence→whole-include" decision a PER-BLOCK content call, not a mechanical swap.

### C′. ERROR-DETECTION SCOPE — what the regen actually surfaces **[R0-FOLD: Important #2 — NEW]**

Honest framing: net-new captures (~150 blocks across manual/quickstart/gui/tech) are produced by CAPTURING current binary output and committing it as the new golden `.out`. For a block with no prior golden, the first capture PINS whatever the binary currently emits — it CANNOT retroactively detect a pre-existing wrong-output bug; it only freezes the current (possibly-wrong) output as the baseline. Recon §C confirms 0 prose fences are gated today → there is no oracle to validate the ~150 against at capture time. Genuine bug-SURFACING is concentrated in exactly two narrower places:
- **(a) P1a's repair of the 15 never-CI-replayed tech-manual goldens** — committed `.out` may already diverge from the current binary → REAL drift detection.
- **(b) ALL blocks going forward under M2** — any future behavior regression trips the gate.

P1a is therefore the one DISCOVERY phase with real bug-surfacing potential and must not be under-resourced relative to the bulk-conversion phases.

### F. PDFs — tech-manual release contradiction (confirmed live)

`technical-manual.yml` has NO tag trigger and NO `gh release upload` — `m-format-technical-manual.pdf` is NOT CI-attached. **CONTRADICTION:** toolkit `CHANGELOG.md:7` asserts the tech-manual PDF "ships as a GitHub release asset" — false per the live workflow. PE must resolve (add CI release path OR correct CHANGELOG.md:7).

### H. Stale-baseline & TWO-TIER pin reality

TWO intentional pin tiers, NOT one:
- **current-release tier** (manual / quickstart / technical-manual): `manual.yml` pins `descriptor-mnemonic-md-cli-v0.9.2` (`--features cli-compiler`) / `ms-cli-v0.11.0` / `mk-cli-v0.10.2` via `cargo install --git … --tag …` (verified `.github/workflows/manual.yml:79/86/90`).
- **gui-pinned tier** (manual-gui ONLY): frozen to the pinned GUI tag's `pinned-upstream.toml`. The R0-GREEN SPEC pins the v0.49.0-era set: `mnemonic-toolkit-v0.70.0` / `md-cli-v0.7.0` / `ms-cli-v0.8.0` / `mk-cli-v0.9.0` (`SPEC_cycleC_manualgui_v1_1_P0.md:30-33`).

**Cross-book `.out` divergence for the SAME `.cmd` is EXPECTED and correct.** Each book's transcripts live in its OWN dir, captured against its OWN tier.

**LOCAL BASELINE — stale/aliased, confirmed live [R0-FOLD: Minor #6]:** `which md` → **SHELL ALIAS `md → mkdir -p`** (NOT a binary; `md --version` printing 0.9.2 is a coincidental-looking artifact of the alias-resolution path, do not trust it). `mnemonic` = 0.70.0 (crate 0.72.0); `ms` = 0.10.0; `mk` = 0.10.1 — NONE at the current-release pins (toolkit-0.72.0 / ms-cli-v0.11.0 / mk-cli-v0.10.2). A faithful baseline REQUIRES `cargo install --git … --tag <pin>` for md/ms/mk + `cargo build --bin mnemonic`, then passing ABSOLUTE bin paths to verify-examples (do NOT rely on `$PATH` `md`, which is a `mkdir` alias). `Pinned: mnemonic 0.13.0` literals live at 4 manual-gui sites (verified: `31-first-launch.md:21` mockup, `33-help-icons-and-deep-links.md:37` mockup, `31-first-launch.md:40` prose, `41-overview.md:62` prose) plus **12 broader `0.13.0`/`v0.13` references** (`grep -rln` count) in prose/tables — version-literal maintenance is Axis-2 hand-authoring, output-fidelity-ungated.

### I. Explicit EXCLUSION allow-list (non-output / non-generable)

- `docs/manual/src/99-build-banner.md` — `Built from commit … on …` is PROSE (0 fenced blocks), `_This page is included in the PDF render only._`, Makefile injects fresh `GIT_SHA`/`BUILD_DATE`. Intentional build-provenance stamp.
- ~9 manual-gui ASCII-art GUI mockups (`+---+`/`☑`/`▾`/`?`, width/font-sensitive).
- ~2 truncated/ellipsized manual-gui blocks (need a masking/line-range filter first).
- `docs/codex32/2023-03-07--color.pdf` (vendored MIT, pinned SHA-256).

### J. Runner inventory (4 forks — confirms M2 under-scope) — all live-verified

| Runner | recursive (`find`) | tmpdir per-cmd | triple (`.err`) | MK_BIN | symlink? |
|---|---|---|---|---|---|
| `docs/manual/tests/verify-examples.sh` (CANONICAL) | yes | yes | yes | yes | real file |
| `docs/quickstart/tests/verify-examples.sh` | — | — | — | — | **symlink→`../../manual/tests/verify-examples.sh`** (confirmed) |
| `docs/technical-manual/tests/verify-examples.sh` | **NO** (`"$TRANSCRIPTS"/*.cmd`) | **NO** | no | yes | real file |
| `docs/manual-gui/tests/verify-examples.sh` | **NO** | **NO** | no | **NO — MK-BLIND** (case block lines 18-25 has NO `MK_BIN=*` arm; `grep MK_BIN` → 0 hits, confirmed) | real file |

**Makefile verify-examples targets — ALL MK-blind except via the symlinked runner [R0-FOLD: Important #1 + Minor #3]:**
- `docs/quickstart/Makefile:198-203` `verify-examples:` → passes `MNEMONIC_BIN/MD_BIN/MS_BIN/TRANSCRIPTS` — **NO MK_BIN** (confirmed live).
- `docs/manual-gui/Makefile:230-235` `verify-examples:` → passes `MNEMONIC_BIN`(231)/`MD_BIN`(232)/`MS_BIN`(233 — actual line is `MS_BIN="$(MS_BIN)"` at **234** per the 230-235 span)/`TRANSCRIPTS=`(235) — **NO MK_BIN**. The MK_BIN line must be **INSERTED after the `MS_BIN` line, before `TRANSCRIPTS=`** (re-grep live line numbers at write time — they shift on insert; the prior "Makefile:234" single-line citation was imprecise).

**CI bin-stub reality (verified live) — every non-manual book runs `make lint` (NOT `verify-examples`) with stub bins → ZERO output-error-detection today:**
- `quickstart.yml:77` → `make lint MNEMONIC_BIN=true MD_BIN=true MS_BIN=true MK_BIN=mk` (lint, not verify; 3 of 4 stubbed).
- `technical-manual.yml:83-86` → `make lint MNEMONIC_BIN=true MD_BIN=true MS_BIN=true MK_BIN=true` (all 4 stubbed).
- `manual-gui.yml:106` → `make lint MNEMONIC_BIN=true MD_BIN=true MS_BIN=true MK_BIN=true` (all 4 stubbed).

### K. Filter-arg ordering — TWO INDEPENDENT CHAINS, not one [R0-FOLD: Minor #4]

`docs/manual/Makefile:80-81` (verified live):
- `MD_FILTER_ARGS := strip-latex-from-md.lua → primer-box.lua` (html/gfm path; strip-latex is MD-ONLY).
- `PDF_FILTER_ARGS := primer-box.lua → wrap-long-code.lua` (xelatex path; wrap-long-code is PDF-ONLY).

There is NO single chain containing all three filters. The M1 include filter must be **PREPENDED to BOTH** variables independently: before `strip-latex-from-md.lua` in `MD_FILTER_ARGS` AND before `wrap-long-code.lua` in `PDF_FILTER_ARGS`. `primer-box.lua` runs in both — confirm the include filter does not interfere with its sentinel detection. `filter-smoke` target exists at `docs/manual/Makefile:285`.

### L. manual-gui lint.sh phase numbering [R0-FOLD: Minor #5 — verified]

7 existing phases, hardcoded `step "N/7"` fractions at lint.sh: 56 (1/7 markdownlint), 64 (2/7 cspell), 76 (3/7 lychee), 84 (4/7 gui-schema-coverage), 99 (5/7 outline-coverage), 113 (6/7 glossary-coverage), 130 (7/7 index bidirectional). The new verify-examples phase is correctly the **8th**; landing it requires renumbering ALL 7 `step "N/7"` literals → `N/8` (mechanical, easy-to-miss; grep `step "`).