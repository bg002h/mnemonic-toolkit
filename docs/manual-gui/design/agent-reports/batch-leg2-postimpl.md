# batch `tutorial_surfaced_fixes_batch` — Leg 2 (toolkit BOOK) whole-diff post-implementation review

- **Reviewer:** opus architect — mandatory, non-deferrable post-implementation whole-diff review (adversarial, independent). Every gate re-run by me with REAL tool calls; PDFs built; figures opened; not trusted from the per-phase reports.
- **Under review:** `mnemonic-toolkit` branch `feat/tutorial-surfaced-fixes-book`, tip `59d0c788`, off `origin/master`. Diff = **102 files** (28 `.gui` + 32 gallery PNG + 26 tutorial PNG + pin + inventory + 5 reference `.md` + 6 tutorial `.md` + 2 FOLLOWUPS + cspell).
- **Authority:** `design/IMPLEMENTATION_PLAN_tutorial_surfaced_fixes_batch.md` §P2.1/§P2.2/§P2.3; the Leg-1 ship (`mnemonic-gui-v0.57.0`, `93954493`, `design/agent-reports/batch-leg1-postimpl.md` GREEN); the gui_example Leg-2 precedent (`manual-gui-v1.2.0`).
- **Environment:** fresh depth-1 clone of `mnemonic-gui-v0.57.0` (HEAD `93954493` == tag, verified); `env RUSTUP_TOOLCHAIN=stable MANUAL_GUI_UPSTREAM_ROOT=<clone>`; `gui-render` built `--no-default-features` from the clone; pandoc 3.6 + xelatex (TeX Live 2026) + makeindex; markdownlint-cli2 / cspell 8.19.4 / lychee 0.24.2 all present. `make html` run BEFORE `make lint`.

---

## VERDICT: RED — 1 Important blocks the tag. 0 Critical / **1 Important** / 2 Minor.

All machine gates are GREEN and the corpus is byte-faithful and secret-clean; the tutorial teaches the fixed flow; the OS-snapshot reconciliation is accurate. **But one un-gated prose defect blocks `manual-gui-v1.3.0`:** the restore form's on-load materialised `--template` default is stated as **`bip84`** in **5 sites**, when the actual GUI default is **`bip44`** — and the reference `(none)` section **contradicts itself** (says `bip44` in its intro, `bip84` in its multisig bullet). This is exactly the class of drift the whole-diff review is the designated oracle for (prose is un-gated; §P2.2). Fix is a mechanical `bip84 → bip44` at 5 sites + re-lint (NO figure re-drive, NO gate change), then re-enter the review loop for a scoped GREEN before tag.

---

## 1. Regen byte-fidelity — GREEN

| Surface | Result |
|---|---|
| 61 gallery PNG (`figures/gui/` vs clone `tests/snapshots/forms/`) | **0 mismatch / 0 missing** (byte-identical) |
| 50 tutorial PNG (`figures/tutorial/` vs clone `tests/snapshots/tutorial/`) | **0 mismatch / 0 missing** |
| 98 tutorial transcripts (`transcripts/tutorial/` vs clone) | **0 mismatch / 0 not-found** — the ruling-7 zero-delta invariant holds; git shows 0 tutorial-transcript delta |
| 28 `.gui` via `make verify-examples-gui` (regen vs pinned `gui-render`) | **61/61 renders byte-identical, no secret leak** |
| `.gui` line-level delta | every changed row is exactly `<masked> → <masked> [reveal]` (45 marker additions, **0 removals**); marker is strictly-ASCII per ruling 4 |
| `mnemonic-restore.gui` | carries **BOTH** deltas — `--template […,tr-sortedmulti-a,(none)]` at `:4` AND `--passphrase … <masked> [reveal]` at `:14` |
| `expected_gui_schema_inventory.json` | adds only the restore `--template` trailing `""` sentinel (line 1961) |

**Pin coherence:** `pinned-upstream.toml` `tag = mnemonic-gui-v0.57.0`; the four `*-tag-implied` fields (`mnemonic-toolkit-v0.75.0` / `descriptor-mnemonic-md-cli-v0.11.0` / `ms-cli-v0.13.0` / `mk-cli-v0.11.0`) VERIFIED byte-for-byte against the clone's own `pinned-upstream.toml` + `Cargo.toml` — pin-neutral for all four CLIs, as §P2.1 requires.

## 2. Prose accuracy

### (a) Restore `(none)` reference (`4d-restore.md`, `#mnemonic-restore-template-`) — SEMANTICS GREEN, but see I-1
The single-sig vs md1 split is **correct and correctly distinguished from export-wallet's**: single-sig `(none)` = the honest "emit all four `bip44/49/84/86`" (the flag's omit-to-emit-all default); md1 `(none)` = **drops the single-sig template the CLI refuses** (a `--md1` single-sig `--template` is a `ModeViolation` → **exit 2**, ruling 7 / `restore.rs`). The `--format` cross-ref fold (`:89-95`) and the export-wallet staleness fix (`45-export-wallet.md:52-56`: "export-wallet + restore carry the sentinel; bundle / verify-bundle / convert stay at 10") are accurate. **Defect: the "materialises `bip84` on load" claim — see I-1.**

### (b) Reveal hygiene (`14-secret-handling.md`, `#secret-reveal-toggle`) — ACCURATE (GREEN)
Hold-to-reveal primary; pointer **tap does not latch**; bounded latch for keyboard/AT; **no timeout**; single-revealed-field invariant; auto-hide ×4 (Run / blur / window-focus-loss / tab-or-subcommand switch); the one-frame focus-loss race is honestly disclosed ("treat a revealed field as on-screen until you have looked away and back"); display-only — run-confirm modal, `argv:` echo, copy-command, paste-warn, never-persist, exit-sweep all stay masked regardless; not a `FormState`/schema field. Matches the shipped GUI behavior (Leg-1 §2a) and the opened figures (Preview stays `--slot ••••` while the field is revealed).

### (c) OS-snapshot reconciliation (`84-secrets-and-os.md`) — ACCURATE, does NOT understate (GREEN)
The stale "always masked → a window snapshot never captures a secret" claim is correctly falsified. New prose: a masked field's value is not in the frame, **"unless the field is revealed via the 👁 toggle, in which case the plaintext is drawn to the frame and can be captured like any other visible widget (re-mask before you look away)."** This is accurate and appropriately conservative — it explicitly acknowledges a revealed field IS snapshot-capturable, advises manual re-mask (stronger than relying on the focus-loss auto-hide), and does not wrongly reassure. The screenreader addition ("a revealed field advertises its plaintext through AccessKit deliberately") is also correct (reveal → `.password(false)` → AccessKit exposes the buffer). Rendered clean on PDF p422.

## 3. Tutorial rewrite = the FIXED flow — GREEN (modulo I-1)
- **All 6 restore steps** now teach the clean `(none)` md1 restore: `tut-j2-08` (30-j2:266-274), `tut-j3-13` (40-j3:168-171), `tut-j4-17` (50-j4:123-128), `tut-j4-nums` (50-j4:315), `tut-j5-23` (60-j5:49-53), `tut-j5-24` (60-j5:85-86). **Zero residual workaround framing** — grep for `for consistency` / `route.?around` / `workaround` / `inert when restoring` across `tutorial/` returns NOTHING (the old "set it to the wallet's own family for consistency" is gone).
- **4 reveal steps** note the now-visible public demo phrase: `tut-j1-01` (20-j1:18-26), `tut-j2-02` (30-j2:34-51), `tut-j2-03` (30-j2:74-79), `tut-j2-07` (30-j2:222-231, with the single-revealed-field teaching image explicit).
- **Ch-0** carries the reveal caveat (new "Revealed demo phrases in this book" section + the masked-fields bullet update).
- **Prose ↔ figure spot-checks:** opened `tut-j1-01-bundle-single-sig-form.png` (slot shows S0 `…abandon…about` revealed, Preview `--slot ••••` masked — matches 20-j1 prose), `tut-j2-07-bundle-all-seeds-form.png` (slots @0/@1 masked, @2 revealed S2 `…letter advice cage above`, Preview all `••••` — matches the single-revealed-field prose), `tut-j2-08-restore-form.png` (Template = `(none)`, public md1 chains — matches 30-j2). PDF p10 confirms the same in-book.

## 4. Book secret hygiene — GREEN
- No `[xt]prv…` / `[xt]pub`-as-secret literals anywhere in `manual-gui` `.md`/`.gui`/`.json`.
- **S1/S2 phrases** (`legal …`, `letter …`, `advice cage above`) appear **nowhere** in prose or committed transcripts — only as rasterized pixels inside the 4 allowlisted reveal `-form` figures (and the `secret_values_are_allowlisted` + `[S0,S1,S2]` allowlist enforce that GUI-side).
- The all-`abandon` **S0** vector appears in reference-manual CLI examples + `.cmd`/`.out` transcripts — the long-standing, documented **public** test-vector convention, NOT part of this cycle's diff, explicitly labelled public. Not a leak.
- Reference gallery figures show the 👁 beside **masked** value fields (public gallery, no live key material). Reveal figures show only allowlisted public phrases with the masked Preview.

## 5. PDF correctness — GREEN (1 cosmetic Minor)
- **Reference** `m-format-gui-manual.pdf`: 446 pages, builds clean, embed census **61**. Opened p176 (restore `(none)` section — renders cleanly), p353 (`[reveal]` ASCII marker docs — renders perfectly), p422 (OS-snapshot reconciliation — clean). New sections render without LaTeX breakage.
- **gui-example** `gui_example.pdf`: force-rebuilt clean, 97 pages, embed census **50** (byte-identical content to the P2.2 build). Opened p10 (J1-01 reveal figure: S0 plaintext in the slot + 👁 icon, Preview `--slot ••••` masked, panes legible, no placeholder/TODO), p22 (export `(none)` BSMS step, public xpubs only). Restore pages show Template `(none)`.
- **M-1 (cosmetic):** the `👁` emoji in *prose* renders as a missing-glyph box (tofu) under xelatex/DejaVu-Serif in BOTH PDFs (e.g. p353 heading "with a deliberate □ reveal", p422 "revealed via the □ toggle", gui-example p10 "the row's □ reveal toggle"). The load-bearing ASCII `[reveal]` marker and the real GUI eye glyph in the figures render correctly; text stays legible. HTML renders the emoji fine. Non-blocking; fix = emoji-font fallback in `preamble.tex` or reword to "eye".

## 6. Gates — GREEN (all counts as specified)
`make html` then `make -C docs/manual-gui lint` = **12/12 OK**:
1 markdownlint 0 err (99 files) · 2 cspell 0 issues · 3 lychee 2059 OK / 0 err · 4 **gui-schema-coverage 984** (61 subs — restore-template anchors 12→13, the `+1` `(none)` anchor, git-verified) · 5 outline 129 · 6 glossary OK · 7 index OK · 8 gui-form-xref 61 · 9 **verify-figures-gui 61/61 byte-identical** · 10 **verify-tutorial-figures 50 byte-identical** · 11 **verify-tutorial-transcripts 98 byte-identical (zero-delta)** · 12 **tutorial-xref 50 figures + 98 transcripts**. Plus the separate **verify-examples-gui 61/61** (.gui regen). Pin bump coherent + pin-neutral.

## 7. The `93-dropdown-reference.md` no-op — ACCEPTABLE (Minor M-2)
Untouched (git shows 0 delta). The `### TEMPLATES (10 values)` appendix still lists `bundle / verify-bundle / convert / export-wallet` as consumers and omits restore. **Ruling: acceptable deliberate no-op**, consistent with the F1 precedent and pre-blessed by plan §P2.2 ("optional parity note only, un-gated"): the appendix documents the *shared* `TEMPLATES` const, which genuinely has 10 values; the per-flag `(none)` sentinels of export-wallet and restore (their 11-value `EXPORT_WALLET_TEMPLATES` / `RESTORE_TEMPLATES`) are documented in each subcommand's own chapter. **Minor caveat:** now that `45-export-wallet.md` explicitly says "export-wallet + restore carry the sentinel; bundle / verify-bundle / convert stay at 10", a reader cross-referencing the appendix (which lists export-wallet under "(10 values)" and omits restore) sees a mild inconsistency. A one-line parity note ("export-wallet and restore append an 11th `(none)` sentinel — see their chapters") would close it. Non-blocking.

## 8. Ship-readiness (P2.3) — correctly OUTSTANDING; nothing tag-only wrongly in the leg
- `docs/manual-gui/CHANGELOG.md` — **NO `[1.3.0]` entry** ( `[Unreleased]` empty, top is `[1.2.0]` ) → correctly owed at P2.3.
- FOLLOWUPs — `gui-secret-reveal-toggle` filed in **both** `design/FOLLOWUPS.md` and `docs/manual-gui/FOLLOWUPS.md` with cross-citing `Companion:` lines, status **"open — flips RESOLVED in the manual-gui-v1.3.0 shipping commit (P2.3)"** → correctly NOT pre-flipped.
- `manual-gui-v1.3.0` tag + release-attach of the rebuilt `gui_example.pdf` (release job inherits the attach from v1.2.0) → owed at P2.3; `build/` is gitignored (PDFs correctly not committed).
- No version-site/vendor/ToolkitError ritual applies (zero `crates/` `src/` changes). Working tree clean; no stray `.new.png`.

## 9. Findings by severity

- **Critical:** none.
- **Important — I-1: the restore form's materialised `--template` default is stated as `bip84` (5 sites); the truth is `bip44`; the reference `(none)` section self-contradicts.**
  - Ground truth `bip44`: GUI `src/schema/mnemonic.rs` comment "the virgin form-loop still materializes `opts[0]` (`bip44`)" + `RESTORE_TEMPLATES[0]=="bip44"`; `.gui` render `--template … -> bip44`; and the manual's own `4d-restore.md:185` ("materialises to `bip44`") and `:256` ("still materialises to `bip44` on load").
  - Wrong `bip84` sites: `src/40-mnemonic/4d-restore.md:269` ("but the GUI form materialises `bip84` on load" — **directly contradicts `:256`** ~13 lines above); `tutorial/30-j2-multisig.md:271`; `tutorial/40-j3-degrading-vault.md:170`; `tutorial/50-j4-taproot-twin.md:126`; `tutorial/60-j5-watch-only.md:51` (all "the form's default `bip84`").
  - NOT errors (leave as-is): the illustrative command `mnemonic restore --md1 … --template bip84 exits 2` (`4d-restore.md:268-269`, first `bip84`) — bip84 in `--md1` mode genuinely exits 2; and the single-sig walkthrough `4d-restore.md:581` ("Set `--template` to `bip84`") — a valid single-sig choice in that `--from`-seed example.
  - Why Important: a reader opening a fresh restore form sees `bip44`, the manual says `bip84`; it is a **self-contradiction inside one shipped reference section**; repeated 5×; and this cycle's entire premise is fidelity-to-the-real-GUI. Un-gated (all 12 phases GREEN despite it) → the whole-diff review is the only oracle.
  - Remediation: `bip84 → bip44` at the 5 sites above. NO figure re-drive (figures show `(none)`), NO gate/anchor change (`#mnemonic-restore-template-` unchanged). The pedagogical point (any single-sig template refused in `--md1` → exit 2) survives verbatim. Re-run `make lint` + re-read, then a scoped GREEN convergence review before the tag.
- **Minor:**
  - **M-1** — `👁` emoji renders as missing-glyph tofu in both PDFs' prose (cosmetic; ASCII `[reveal]` marker + GUI eye figures render correctly; HTML fine). Fix = emoji-font fallback in `preamble.tex` or reword. Non-blocking.
  - **M-2** — `93-dropdown-reference.md` "TEMPLATES (10 values)" appendix lists export-wallet (omits restore) though both now use 11-value variants (pre-existing F1 inconsistency, plan-blessed no-op). A one-line parity note would resolve. Non-blocking.

## Bottom line
Byte-fidelity, gates (12/12 + verify-examples-gui 61/61), pin coherence, secret-hygiene, the OS-snapshot reconciliation, the reveal hygiene model, and the fixed-flow tutorial (clean `(none)` md1 restore, zero workaround framing; visible public reveal phrases) are all correct and PDF-verified. **One Important prose defect — a 5× `bip84`-should-be-`bip44` default-value error, including a self-contradiction within the reference `(none)` section — blocks the `manual-gui-v1.3.0` tag** under the house zero-Important gate. It is a mechanical, re-drive-free, gate-neutral fix. Land `bip84 → bip44` (5 sites), re-lint, re-enter the loop for a scoped GREEN, then proceed to P2.3 (CHANGELOG `[1.3.0]` + FOLLOWUP flips + tag + release-attach verify). **RED pending I-1.**

_Repo left clean: clone + all `build/` artifacts (gitignored) removed; no tracked-file modifications; no stray `.new.png`._

---

## CONVERGENCE ADDENDUM — scoped re-review of the fold — `599fe8d2` (atop `59d0c788`)

- **Reviewer:** opus architect — scoped convergence check (the same reviewer who returned the RED above). Every claim re-run with REAL tool calls against a **fresh depth-1 clone** of `mnemonic-gui-v0.57.0` (HEAD `93954493` == tag, verified); `env RUSTUP_TOOLCHAIN=stable MANUAL_GUI_UPSTREAM_ROOT=<clone>`; `gui-render` rebuilt `--no-default-features` from the clone; `make html` **before** `make lint`; both PDFs force-rebuilt; the touched pages opened as rasters. pandoc 3.6 · cspell 8.19.4 · lychee 0.24.2.
- **Fold under review:** `599fe8d2 docs(manual-gui): fold leg-2 post-impl (I-1 bip84->bip44 restore default x5 + M-1 emoji-tofu strip + M-2 dropdown parity)`. `git diff 59d0c788..599fe8d2 --stat` = **11 `.md` files, +25 / -21, prose-only** — zero `.gui` / figure / transcript / inventory / pin / build-config churn (verified: every changed path ends `.md`).

### VERDICT: **GREEN — `manual-gui-v1.3.0` tag CLEARED.** 0 Critical / 0 Important / (1 non-blocking source-hygiene nit noted below). The one Important (I-1) and both Minors (M-1, M-2) are resolved with no regression; both PDFs build clean.

### I-1 RESOLVED (was blocking) — restore on-load `--template` default `bip84 → bip44`
- **Ground truth re-confirmed** from the fresh clone: `src/schema/mnemonic.rs` `RESTORE_TEMPLATES[0] == "bip44"`; the committed `transcripts/gui/mnemonic-restore.gui:4` renders `--template dropdown[bip44,…,(none)]  -> bip44`. The form materialises **`bip44`** on load — the fold's correction is right.
- **All 5 previously-wrong sites now read `bip44`:** `4d-restore.md:269` ("the GUI form materialises `bip44` on load"); `tutorial/30-j2:271`, `40-j3:170`, `50-j4:126`, `60-j5:51` (each "the form's default `bip44`"). Grep confirms **no restore-default `bip84` remains** in any of the four tutorial files.
- **Self-contradiction GONE:** `4d-restore.md` `:185` / `:256` / `:269` now all say `bip44` — internally consistent within the single `(none)` reference section.
- **Legitimate `bip84` PRESERVED (no over-correction):** dropdown-value section `:213` (`{#mnemonic-restore-template-bip84}`) + outline link `:195` + see-ref `:215`; the four-template lists `:179` / `:263`; the illustrative `mnemonic restore --md1 … --template bip84` **exits 2** command (`:268–269`, the first `bip84` on that line — genuinely exits 2 in `--md1` mode); and the single-sig `--from`-seed walkthrough `:581` ("Set `--template` to `bip84`"). All intact.

### M-1 RESOLVED — literal eye-emoji stripped, no tofu
- **Zero U+1F441** in all shipped book prose: `perl` codepoint count over `src/` + `tutorial/` = **0**; and **0** in both rendered PDFs' text (`pdftotext`). (The emoji legitimately survives only in `FOLLOWUPS.md` — the tracking entry describing the toggle — and in `design/agent-reports/`, neither of which is book content. The GUI eye still appears in the FIGURES, which is correct.)
- **Renders clean, no missing-glyph box:** opened the reveal-hygiene page (ref PDF p52, "6.5 The reveal toggle — deliberate, display-only exposure" heading + "a small **reveal button**" body — no tofu) and a tutorial reveal step (gui-example p7, "Revealed demo phrases" + the J1 "Masked fields" bullet). The tofu boxes the RED flagged at ref p353/p422 and gui-example p10 are gone.
- The sole `U+FFFD` in the ref PDF (p~419) is **pre-existing deliberate prose** in `83-form-and-output.md` (a troubleshooting cell literally showing a `from_utf8_lossy` `�`) — present at base `59d0c788`, in a file the fold never touched. Not tofu, not a regression.
- **NON-BLOCKING nit (noted, not gating):** the emoji strip at `tutorial/10-ch0-orientation.md:133` split the source `**👁\nreveal**` into a `**` left dangling at end-of-line with `reveal**` wrapping to `:134`. Under the pinned pandoc 3.6 this renders **correctly** as bold "reveal" (verified on the raster: "those steps hold the **reveal** toggle" — no literal asterisks, no visible double-space; whitespace collapses) and `markdownlint` passes 0. It is the lone dangling-opener `**` in the whole book. A future one-line tidy (join `**reveal**` onto a single line) is advisable for source hygiene / stricter-CommonMark robustness, but it produces **no visible defect** and does not gate the tag.

### M-2 RESOLVED — dropdown-reference parity note + resolving link
- `93-dropdown-reference.md` now states export-wallet **and** restore `--template` use the **11-value variant** (the 10 plus the appended GUI-only `(none)` sentinel); `convert` correctly stays under the shared 10-value list.
- The new `[restore](#mnemonic-restore-template)` link **resolves** — anchor exists at `4d-restore.md:176` (`## `--template` {#mnemonic-restore-template}`); lychee **0 errors**. Total link count 2060 (was 2059 in the RED) — the `+1` is exactly this new internal link.

### NO REGRESSION (fold is prose-only)
- `make html` then `make lint` = **12/12 OK**: markdownlint 0 (99 files) · cspell 0 · lychee **2060 OK / 0 err** (5 excluded) · **gui-schema-coverage 984 UNCHANGED** (61 subs) · outline 129 · glossary OK · index OK · gui-form-xref 61 · **verify-figures-gui 61/61** · **verify-tutorial-figures 50** · **verify-tutorial-transcripts 98 (zero-delta)** · tutorial-xref 50 + 98. Plus the separate **verify-examples-gui 61/61** (`.gui` regen, no secret leak).
- **Both PDFs build clean:** reference `m-format-gui-manual.pdf` **446 pages**, embed census **61** (122 `pdfimages` objects = 61 × [image + alpha-smask], no placeholder); gui-example `gui_example.pdf` force-rebuilt **97 pages**, embed census **50** (100 objects = 50 × 2). No placeholder/TODO on any opened page.
- `git diff --stat 59d0c788..599fe8d2` = **only prose `.md`**; zero figure / `.gui` / transcript / inventory / pin / Makefile / pandoc-config churn.

### Leg-2 GREEN-on-substance findings undisturbed
- **Corpus** byte-faithful (all verify-* gates green; zero figure/transcript delta in the diff). **OS-snapshot reconciliation** intact — the `84-secrets-and-os.md` fold is **emoji-only** (three lines, `👁` removed, semantics unchanged); the load-bearing falsification "**unless the field is revealed via the toggle**, … the plaintext is drawn to the frame … re-mask before you look away" is preserved verbatim. **Tutorial fixed-flow** unchanged in substance — `bip84 → bip44` does not touch the clean `(none)` md1-restore teaching or the zero-workaround framing. **Secret hygiene** clean (verify-examples-gui no leak; reveal figures still show only allowlisted public phrases with masked Preview).

### Owed at P2.3 (correctly still OUTSTANDING at `599fe8d2`)
- `docs/manual-gui/CHANGELOG.md` — **no `[1.3.0]` entry yet** (`[Unreleased]` empty, top is `[1.2.0]`) → add at P2.3.
- FOLLOWUP `gui-secret-reveal-toggle` — status **"open … flips RESOLVED in the `manual-gui-v1.3.0` shipping commit (P2.3)"** in **both** `design/FOLLOWUPS.md` and `docs/manual-gui/FOLLOWUPS.md`, cross-citing `Companion:` lines present; correctly NOT pre-flipped.
- Tag `manual-gui-v1.3.0` + release-attach the rebuilt `gui_example.pdf` (release job inherits the attach from v1.2.0). `build/` gitignored (PDFs correctly not committed); working tree carries no tracked modifications and no stray `.new.png`.

### Bottom line
The fold cleanly discharges I-1 (5× `bip84 → bip44`, self-contradiction gone, legitimate `bip84` preserved), M-1 (emoji stripped, no tofu, renders verified), and M-2 (dropdown parity + resolving link), with **12/12 lint + verify-examples-gui 61/61 + both PDFs clean + prose-only diff** — no regression. **GREEN: `manual-gui-v1.3.0` is cleared to tag.** Proceed to P2.3 (CHANGELOG `[1.3.0]` + FOLLOWUP flips + tag + `gui_example.pdf` release-attach). The lone non-blocking nit (ch0:133 dangling `**`, renders correct) is a candidate for an incidental future source tidy.

_Convergence review left the repo clean: fresh clone + all `build/` artifacts (gitignored) removed; no tracked-file modifications introduced by this review._
