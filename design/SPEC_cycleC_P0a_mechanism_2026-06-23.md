# SPEC — Cycle C P0a MECHANISM (M1 include filter + M2 unified runner + CI gate + sample proof)

**Source SHA:** toolkit `737ff5ffb582745df607a3db2c9cc6051fadece8` (HEAD, v0.72.0). All paths/line-numbers below re-grepped LIVE at this SHA — re-grep again at implementation time (they decay every merge).
**SemVer:** NO-BUMP (docs + CI infra only; no crate surface touched).
**Scope:** M1 (include filter) + M2 (runner unification) + the CI gate + an end-to-end SAMPLE PROOF. The bulk manual conversion (~95 blocks) is P0b — explicitly OUT of this spec. tech-manual/quickstart/manual-gui conversions are P1–P4 — OUT.

**R0-FOLD provenance:** four R0 findings folded (1 Important §5-sample-misclassification + 3 Minor: filter-smoke-leg, golden-count, bare-md-CI). Each fold was re-verified live against this SHA before incorporation — see the inline **[R0-FOLD]** markers.

---

## 0. Problem statement (the paste-drift hole — proven live)

`verify-examples` replays the binary and diffs the committed `.out` ON DISK; it NEVER reads `.md` prose. So a fenced CLI-output block in a chapter is a hand-pasted copy with zero sync to its golden. **Proven live at this SHA:** `docs/manual/src/20-quickstart/22-first-bundle.md` fences (lines 51-72) reproduce `transcripts/22-first-bundle.out` *lines 2-21* but DROP `.out` line 1 — the `warning: secret material on argv (--slot @0.phrase=) …` line that the current binary emits (verified: replaying the `.cmd` against `/scratch/code/shibboleth/.docbins/current/bin/mnemonic` 0.72.0 produces output byte-identical to the committed `.out`; the prose fence omits its line 1, keeps line 21's spend-warning). This is the canonical EXCERPT case.

**The excerpt pattern is PERVASIVE, not isolated [R0-FOLD: Important].** Re-grepping all three quickstart goldens that the sample-proof draws on confirms NONE is pasted whole into prose:
- `22-first-bundle.md:51-72` == `22-first-bundle.out` **lines 2-21** (drops `.out` line 1 argv-warning; keeps line 21 spend-warning).
- `23-verify.md:43-54` == `23-verify.out` **lines 2-11** (drops `.out` line 1 argv-warning).
- `24-recover.md:65-69` == `24-recover-mk1.out` **lines 2-4** (drops `.out` line 1 SLIP-0132 note AND line 5 watch-only note).
- `24-recover.md:93-95` == `24-recover-md1.out` **line 1 only** (drops `.out` line 2 keyless-template note).

Consequence for the sample proof: the prior draft named `24-recover-mk1`/`24-recover-md1` as the WHOLE-INCLUDE sample, but at HEAD both are EXCERPTS — converting either to a bare whole-include (`include="…"` with no `lines=`) would SILENTLY ADD the `note:` lines the chapter author deliberately elided, smuggling a content change in under a "mechanism proof." See §5 for the corrected sample set (re-classify both `24-recover` blocks as excerpts; add a purpose-built whole-include fixture so the whole-include code path is genuinely validated).

**Two distinct mechanisms (do NOT conflate):**
- **M1** makes a fence's rendered body COME FROM the golden `.out` by construction → prose==.out is structural, not hand-maintained.
- **M2 + the CI gate** make `.out`==binary by replaying the real pinned binary in CI for every book.
- Composed: **prose == .out (M1) ∧ .out == binary (M2 gate) ⟹ prose == binary**, transitively, with zero hand-sync.

---

## 1. M1 — build-time include filter `include-transcript.lua`

### 1.1 File + wiring

- **New file:** `docs/manual/pandoc/filters/include-transcript.lua`.
- **Fence convention (whole-file):** ` ```{.text include="22-first-bundle.out"} ` — class (`text`/`json`) preserved; `include=` names a path RELATIVE to the transcripts root. Body between the fences is a human-readable PLACEHOLDER (e.g. `PLACEHOLDER — generated from transcripts/22-first-bundle.out at build`); the filter REPLACES it wholesale.
- **Fence convention (excerpt):** add `lines="2-21"` (1-based inclusive; open-ended `lines="2-"` = "line 2 to EOF"). The filter selects that contiguous range from the `.out`. (Proven: `22-first-bundle`'s prose fence == `.out` lines 2-21 exactly.)
- **Path resolution:** the filter reads env var `TRANSCRIPTS_DIR` (absolute), set by the Makefile from `$(TRANSCRIPTS)`. Rationale mirrors `mermaid-cache-filter.lua`'s `MERMAID_CACHE_DIR` pattern (Makefile lines 86-90) — pandoc may run from any cwd, so a relative value would mis-resolve.
- **Fail-closed:** missing `TRANSCRIPTS_DIR`, missing include target, malformed `lines=`, or `lines=` out of range → `io.stderr:write(...)` + `os.exit(1)` → **build FAILS** (never a silent empty fence). Proven: a missing target exits 1 with `[include-transcript] FATAL: include target missing: <abs-path> (fence include="…")`.
- **Trailing-newline handling:** whole-file include strips exactly one trailing `\n` so the fence body has no spurious blank last line. Excerpt joins selected lines with `\n` (no trailing). `.out` files have NO trailing-newline-after-last-line in some cases (`22-first-bundle.out` ends `...age -e ...').` with one `\n`) — the split-on-`\n` logic must not emit a phantom final empty line.
- **Output:** return `pandoc.CodeBlock(body, pandoc.Attr(el.identifier, el.classes, attrs_without_include_and_lines))` — DROP the `include=`/`lines=` attributes so downstream filters/writers see a clean code block.

### 1.2 Filter-arg wiring (TWO INDEPENDENT CHAINS — Inventory §K, verified live)

`docs/manual/Makefile:80-81`:
- `MD_FILTER_ARGS := strip-latex-from-md.lua → primer-box.lua` (md + html paths)
- `PDF_FILTER_ARGS := primer-box.lua → wrap-long-code.lua` (pdf path)

PREPEND `--lua-filter $(FILTERS_DIR)/include-transcript.lua` to **BOTH** variables (first position, before `strip-latex-from-md.lua` in MD and before `primer-box.lua` in PDF) so the include resolves BEFORE any other transform sees the block. The two are independent assignments, not one chain — wire both. Export `TRANSCRIPTS_DIR := $(TRANSCRIPTS)` in the Makefile (already an absolute var, line 39).

Proven-live ordering safety: include-transcript THEN wrap-long-code on the LATEX writer correctly chunks an included contiguous `xpub6…` (>100 char) run with embedded newlines (the `24-recover-mk1.out` xpub split after 64 chars). `primer-box.lua` operates on `Div`, not `CodeBlock` — zero interaction with include fences (confirmed by reading both filters). The implementer MUST still confirm no `primer-box` sentinel collision in the `filter-smoke` render.

**filter-smoke proves include→wrap COMPOSITION — but the current PDF leg loads neither filter [R0-FOLD: Minor].** The live `docs/manual/tests/filter-smoke.sh` PDF render path (lines 75-81) loads ONLY `--lua-filter primer-box.lua` — it does NOT load `wrap-long-code.lua`, does NOT load `include-transcript.lua`, and does NOT set or pass `TRANSCRIPTS_DIR`. To make the smoke ACTUALLY exercise include→wrap composition, the PDF leg must be extended with ALL of: (1) `--lua-filter $FILTERS/include-transcript.lua` (PREPENDED, first), (2) `--lua-filter $FILTERS/wrap-long-code.lua` (after primer-box, matching the real PDF chain), and (3) a `TRANSCRIPTS_DIR` source — either exported into the smoke's env from the Makefile `filter-smoke` target, or passed as a `TRANSCRIPTS_DIR=` arg the script parses (mirror the existing `MANUAL_DIR=*`/`PANDOC=*` arg-parse loop, lines 20-27). See §5 step 4 for the fixture requirement that makes the wrap assertion non-vacuous.

### 1.3 Why a bespoke Lua filter (not pandoc's `include-code-files`)

`pandoc --list-extensions` shows no native include; the third-party `include-code-files` is whole-file only (no line-range, no fail-closed). Custom-attribute round-trip is proven: `el.attributes["include"]`/`["lines"]` survive into html, latex, AND gfm writers (tested all three). One ~50-line filter covers whole-file + excerpt + fail-closed across the whole pipeline.

### 1.4 Excerpt mechanism decision — `lines=` line-range is the PRIMARY, not the fallback

The PLAN (M1, FOLD Minor #7) defaulted to a "prose-is-substring-of-golden subset gate" (b) because line-range (a) was thought to need bespoke Lua. Since we are ALREADY writing bespoke Lua, **mechanism (a) `lines="N-M"` is free and is the PRIMARY excerpt mechanism** — and it is SUPERIOR to (b): (a) keeps prose==.out structural for excerpts too (the subset still comes from the file by construction), whereas (b) only gates that prose is *a substring*, leaving the hand-paste alive. The canonical `22-first-bundle` excerpt is a clean contiguous range (`lines="2-21"`) → (a) applies directly. Reserve (b) (substring gate) ONLY for the rare non-contiguous-excerpt fence if one is found during P0b triage; P0a ships (a). This RESOLVES FORK-equivalent ambiguity in favor of the stronger guarantee.

### 1.5 Per-block triage rule (carried into P0b, stated here as the mechanism contract)

Every output fence is classified BEFORE it counts as "converted":
- **whole-include** → ` ```{.text include="<stem>.out"} ` (no `lines=`). REQUIRES that the current prose fence reproduce the FULL `.out` body — if it drops a leading/trailing `note:`/`warning:` line, it is an EXCERPT, not a whole-include (the `24-recover` blocks are the proven trap — see §0/§5).
- **excerpt (contiguous)** → add `lines="N-M"`.
- **excerpt (non-contiguous / annotated)** → NOT convertible by line-range; either restructure into multiple contiguous includes OR keep on the explicit excluded allow-list (Inventory §I) with rationale. Never count an unconverted excerpt as "done" (guards the L4 false-completion pattern).

**Mechanical pre-check before naming any block "whole-include":** `diff <(sed -n '<fence-body-range>p' chapter.md) transcripts/<stem>.out` must be EMPTY. A non-empty diff that is purely leading/trailing-line drops ⟹ it is an excerpt; derive the `lines=` range from the surviving contiguous span. This is the guard that the prior §5 misclassification of `24-recover-mk1` would have failed.

---

## 2. M2 — unify the 4 verify-examples runners on the canonical

Runner inventory (Inventory §J, re-verified live):

| Runner | recursive | tmpdir | triple `.err` | MK_BIN | kind |
|---|---|---|---|---|---|
| `docs/manual/tests/verify-examples.sh` (CANONICAL) | yes | yes | yes | yes | real file |
| `docs/quickstart/tests/verify-examples.sh` | — | — | — | — | **already symlink → `../../manual/tests/verify-examples.sh`** |
| `docs/technical-manual/tests/verify-examples.sh` | NO (`for cmd_file in "$TRANSCRIPTS"/*.cmd`) | NO | no | yes (line 25/51) | real file |
| `docs/manual-gui/tests/verify-examples.sh` | NO | NO | no | **NO — MK-blind** | real file |

**M2.1 quickstart** — runner is ALREADY a symlink to canonical → NO runner change. But the quickstart **Makefile** `verify-examples:` target (lines 198-203) passes `MNEMONIC_BIN/MD_BIN/MS_BIN/TRANSCRIPTS` and **NO MK_BIN** → INSERT `MK_BIN="$(MK_BIN)" \` after the `MS_BIN` line (currently line 202), before `TRANSCRIPTS=`. Add a `MK_BIN ?=` default to the quickstart Makefile var block (it has none today — verified grep shows MNEMONIC/MS but the `verify-examples` target also passes MD_BIN at line 201, so add MK to both the `?=` defaults near line 42 AND the target).

**M2.2 technical-manual** — `rm` the real-file runner; replace with a symlink `docs/technical-manual/tests/verify-examples.sh → ../../manual/tests/verify-examples.sh` (quickstart's pattern). Its Makefile target already passes MK_BIN (it had the arm). **MIGRATION HAZARD — flag for P1a, NOT P0a:** the 4 cargo-example `.cmd` (`md-/mk-/ms-codec-api-roundtrip.cmd`, `mnemonic-toolkit-api-roundtrip.cmd`) contain a **relative** `cargo run --quiet --manifest-path examples/Cargo.toml --example …`. The OLD tech-manual runner runs `.cmd` in the invocation cwd (`docs/technical-manual/`), so `examples/Cargo.toml` resolves. The CANONICAL runner runs each `.cmd` in a fresh `mktemp -d` cwd → these 4 relative paths BREAK. P1a MUST rewrite those 4 `.cmd` to an ABSOLUTE manifest-path (or a `$`-substituted var the canonical runner expands) BEFORE flipping the runner symlink. This is the "verify no .cmd relies on old pair-only / no-tmpdir semantics" task — record it as the gating P1a precondition so the runner flip does not RED the build. (P0a does not touch tech-manual goldens; it only documents the contract.)

**M2.3 manual-gui** — `rm` the MK-blind real-file runner; replace with a symlink to canonical. INSERT `MK_BIN="$(MK_BIN)" \` into `docs/manual-gui/Makefile` `verify-examples:` target **after line 234 (`MS_BIN="$(MS_BIN)" \`), before line 235 (`TRANSCRIPTS=`)** — re-grep live, the line shifts on insert. The Makefile already has a `MK_BIN ?=` default (line 45). manual-gui transcripts are P4 (dir is `.gitkeep`-only today) — P0a only readies the runner+Makefile so P4 lands transcripts against a correct runner. (P4 also renumbers the 7 `step "N/7"`→`N/8` lint labels — out of P0a.)

**Canonical runner semantics the unification standardizes** (so all books inherit): recursive `find … -name '*.cmd' -not -path '*/cli-help/*'`, per-cmd `mktemp -d` cwd, triple-`.err` auto-detect (pair-mode `2>&1` merge when no `.err` sibling), all four `$*_BIN` + `$FIXTURES_DIR` substituted, mandatory-bin `:?` guards (no silent `/bin/true`).

---

## 3. CI GATE — make verify-examples actually RUN for every book

Today (verified live) the 3 non-manual workflows run `make lint` with STUB bins → ZERO output-error-detection:
- `quickstart.yml:77` → `make lint … MK_BIN=mk` (3 stubbed).
- `technical-manual.yml:82-86` → `make lint … =true` (all 4 stubbed).
- `manual-gui.yml:106` → `make lint … =true` (all 4 stubbed).

**P0a's CI change is MANUAL-ONLY** — `manual.yml` already does the right thing: it `cargo install`s the current-tier pins (`md-cli-v0.9.2 --features cli-compiler` / `ms-cli-v0.11.0` / `mk-cli-v0.10.2`, lines 79/86/90), builds `mnemonic` (debug), and runs `make audit` (= lint + verify-examples + anchor-check) with real bins. So with M1 wired, manual's prose is gated transitively the moment P0a lands. **The only `manual.yml` edit P0a needs: confirm `TRANSCRIPTS_DIR` reaches pandoc** — it flows via the Makefile export (§1.2), and `make pdf`/`make html` in `manual.yml` inherit it. Add a one-line assertion to the build step only if the export proves not to propagate (it does, via `export` in the Makefile).

**`manual.yml`'s bare `MD_BIN=md MS_BIN=ms MK_BIN=mk` is CORRECT — do NOT "fix" it [R0-FOLD: Minor].** The live `manual.yml:107-109` passes `MD_BIN=md`, `MS_BIN=ms`, `MK_BIN=mk` (bare names) to `make audit`. In CI these resolve via `$PATH` to the `cargo install`ed `~/.cargo/bin/{md,ms,mk}` from the install steps above — there is NO `md→mkdir` shell alias in CI. This is correct as-is. The absolute-path / "never bare md" rule (§4(d)) is a LOCAL dev-box constraint and MUST NOT be applied to rewrite `manual.yml`'s bare-name args (doing so would be a no-op churn at best and could break if the workspace-relative path were wrong). See §4(d) for the scoping.

**Per-book CI changes for quickstart / tech-manual / manual-gui are P2 / P1a / P4** (each adds `cargo install` current-tier pins + `MNEMONIC_BIN/MD_BIN/MS_BIN/MK_BIN` + switches `make lint`→`make verify-examples` or `make audit`). P0a STATES this contract and the per-book remediation shape but does not implement them (they ride their conversion phases). Per (b) above, those workflows MAY pass bare `md`/`ms`/`mk` IF they `cargo install` the pins into `$PATH` first; the absolute-path requirement is a LOCAL-only constraint.

---

## 4. DETERMINISM (hard rules, inherited by every conversion)

(a) **Fixed PUBLIC vectors ONLY** — canonical `abandon…about` seed (fp `73c5da0a`, NEVER-FUND) + `ms10entrsq…` vector. A real-seed capture would persist a secret-bearing `.out` → **HARD RULE: no secret-bearing `.out` is ever committed.** The argv/spend warnings ARE expected output and are captured (they are advisories, not secrets).
(b) `--network mainnet` + fixed `--template`; **per-TIER pins**: current-release tier (manual/quickstart/tech) = `md-cli-v0.9.2`/`ms-cli-v0.11.0`/`mk-cli-v0.10.2`/`mnemonic`@HEAD; gui-pinned tier (manual-gui) = the v0.49.0-era set. Cross-book `.out` divergence for the SAME `.cmd` is EXPECTED — each book pins its own tier; NO shared cross-book `.out`.
(c) No `.cmd` reads `$HOME/$RANDOM/date/$$`; the argv advisory is a literal (not interpolated) → byte-stable.
(d) **ABSOLUTE bin paths are a LOCAL dev-box rule, NOT an "everywhere" rule [R0-FOLD: Minor].** On the dev box, `md` is a `mkdir -p` shell ALIAS (confirmed `type md` → `aliased to mkdir -p`), so LOCAL invocations MUST use the absolute current-tier binaries `/scratch/code/shibboleth/.docbins/current/bin/{mnemonic,md,ms,mk}` (verified 0.72.0 / 0.9.2 / 0.11.0 / 0.10.2) — NEVER bare `md` locally. In **CI** there is no such alias: bare `md`/`ms`/`mk` resolve to the `cargo install`ed `~/.cargo/bin` binaries and are CORRECT (this is exactly what `manual.yml:107-109` does — see §3). So: absolute LOCALLY (alias hazard); bare-or-absolute in CI (both resolve to the installed pins). Do NOT rewrite `manual.yml`'s bare args under a misread "absolute everywhere" rule.
(e) `--version`-echo sed normalization is ~0 surface today; apply only if a generated `--version` lands in a transcript.

---

## 5. SAMPLE PROOF — mandatory, BEFORE any bulk conversion (TDD)

Prove BOTH M1 code paths (whole-include AND excerpt) end-to-end, render both pipelines, and run the deliberate-drift + fail-closed tests. This is the P0a acceptance gate. **[R0-FOLD: Important — the prior draft's whole-include sample was misclassified.]** At HEAD, NONE of the 20 manual goldens is pasted WHOLE into prose — all are excerpts (the leading argv/SLIP-0132 `note:`/`warning:` lines are dropped; see §0). So the whole-include leg CANNOT be proven with an existing chapter block without silently adding elided lines. The corrected sample set therefore uses:

**Sample block A — PROVEN EXCERPT (existing chapter, real drift fixed):**
`22-first-bundle.md:51-72` → ` ```{.text include="22-first-bundle.out" lines="2-21"} ` — proven byte-for-byte: prose lines 52-71 == `.out` lines 2-21 (the argv warning on `.out` line 1 is the legitimately-excluded excerpt intent, now machine-pinned by the documented `lines=` range; the spend warning on `.out` line 21 is retained). This conversion FIXES the proven paste-drift by making the excerpt structural.

**Sample block B — PROVEN EXCERPT (the misclassification, corrected):**
`24-recover.md:65-69` → ` ```{.text include="24-recover-mk1.out" lines="2-4"} ` — proven: prose lines 66-68 == `.out` lines 2-4 (`xpub:`/`fingerprint:`/`path:`); `.out` line 1 (SLIP-0132 note) and line 5 (watch-only note) are deliberately elided by the chapter author and stay elided under `lines="2-4"`. (The md1 fence at `24-recover.md:93-95` is the analogous `lines="1-1"` excerpt of `24-recover-md1.out` — convert it too if a second excerpt datapoint is wanted, but block A+B already cover the excerpt leg.) **DO NOT convert either `24-recover` block as a bare whole-include** — that would add the `note:` lines the author elided.

**Sample block C — PROVEN WHOLE-INCLUDE (purpose-built fixture so the whole-include code path is genuinely validated):**
Because no existing chapter pastes a full `.out` body, P0a CREATES a minimal whole-include datapoint that exercises the no-`lines=` path AND the §5-step-4 PDF wrap:
- **Fixture golden:** `docs/manual/tests/fixtures/include-whole-sample.out` — a small, fully-pasted body containing (i) a contiguous `xpub6…` run >64 chars (so the wrap-long-code assertion in step 4 is NON-vacuous — reuse the `24-recover-mk1.out` xpub line) and (ii) 2-3 short lines, with NO trailing-newline-after-last-line ambiguity (exercises the §1.1 trailing-`\n` strip).
- **Fixture chapter fence:** add the whole-include block to the SAME `tests/fixtures/filter-smoke.md` fixture the smoke renders (NOT a shipped chapter — keeps the sample proof self-contained and avoids touching a published manual page for an infra proof): ` ```{.text include="include-whole-sample.out"} ` (no `lines=`). Point its `TRANSCRIPTS_DIR` at `tests/fixtures/` for the smoke render.

This split — A/B prove the EXCERPT path against real chapters (and fix real drift); C proves the WHOLE-INCLUDE + PDF-wrap path against a purpose-built fixture — means BOTH M1 code paths are validated in the mandatory pre-bulk proof, and zero shipped-chapter content silently changes.

**Proof steps (all proven feasible in this recon):**
1. Capture/refresh the A/B `.out` from `/scratch/code/shibboleth/.docbins/current/bin` (already byte-identical to committed — no recapture needed, but the spec mandates the implementer re-run to confirm); author the C fixture `.out` by hand (it is a fixture, not a binary replay — it carries the >64-char xpub line lifted from `24-recover-mk1.out`).
2. Wire `include-transcript.lua` into both `MD_FILTER_ARGS` and `PDF_FILTER_ARGS` + export `TRANSCRIPTS_DIR` (§1.2).
3. **`make html` — assert the EXCERPT blocks (A: `lines="2-21"`, B: `lines="2-4"`) render exactly that slice, and the WHOLE-INCLUDE block (C, no `lines=`) renders the full fixture body.** Name in the acceptance: **one PROVEN whole-include block = C (`include="include-whole-sample.out"`, no `lines=`); one PROVEN excerpt block = A (`include="22-first-bundle.out" lines="2-21"`).** PROVEN in recon (html output of the excerpt fences rendered correctly; whole-include path proven via `24-recover-mk1.out` full-body render).
4. **`make pdf` / `filter-smoke` — assert the WHOLE-INCLUDE C fixture's long contiguous `xpub6…` is chunked by `wrap-long-code` in the LATEX/PDF path.** This REQUIRES extending `filter-smoke.sh`'s PDF leg (lines 75-81) with: (i) `--lua-filter $FILTERS/include-transcript.lua` (prepended), (ii) `--lua-filter $FILTERS/wrap-long-code.lua` (the current leg loads NEITHER), and (iii) a `TRANSCRIPTS_DIR` pointed at `tests/fixtures/` (env-exported from the `filter-smoke` Makefile target at `Makefile:285`, or a parsed `TRANSCRIPTS_DIR=` arg mirroring the existing arg-loop) — see §1.2 [R0-FOLD]. The C fixture's >64-char xpub line makes the wrap assertion NON-vacuous. PROVEN in recon (an included xpub split after 64 chars in the LaTeX writer).
5. **DELIBERATE-DRIFT (RED) test:** mutate sample A's `.out` (e.g. `73c5da0a`→`DEADBEEF`) and run `make verify-examples` — assert exit≠0 with a STDOUT-drift diff. PROVEN in recon (the canonical runner went RED with `17c17 # fingerprint: DEADBEEF`). Revert the mutation; assert GREEN. This double-duty proves the runner gates `.out`==binary AND (via M1) prose==`.out`.
6. **MISSING-FILE (FAIL-CLOSED) test:** point a fence at a nonexistent include and assert `make html` exits 1 with the FATAL diagnostic. PROVEN in recon.

**Acceptance for P0a = all 6 steps GREEN on the 3 sample blocks (A/B excerpt + C whole-include) + `filter-smoke` extended (include filter + wrap-long-code + TRANSCRIPTS_DIR + the >64-char C fixture) + the 20 existing manual goldens still pass `make verify-examples` [R0-FOLD: Minor — it is 20, not 35].** P0a exercises ONLY the MANUAL runner; `make verify-examples` in `docs/manual` replays exactly **20** `.cmd` transcripts (verified live: `[verify-examples] OK (20 transcripts pass)`). The other 15 goldens are the technical-manual's; their runner flip + drift-repair is P1a, and they are NOT re-run in P0a — so P0a's backward-compat surface is the 20 manual goldens, not 35. (The runner change is backward-compatible for those 20 — manual is ALREADY canonical, so this is a no-op-on-behavior wiring of the include filter, not a runner swap, for the manual book.) Then P0a STOPS — P0b does the ~95-block bulk.

---

## 6. Files this spec creates / edits

- **NEW** `docs/manual/pandoc/filters/include-transcript.lua` (the M1 filter; ~50 lines; prototype proven in recon).
- **NEW** `docs/manual/tests/fixtures/include-whole-sample.out` (the sample-C whole-include fixture; carries a >64-char `xpub6…` line + 2-3 short lines).
- **EDIT** `docs/manual/Makefile`: prepend the filter to `MD_FILTER_ARGS` + `PDF_FILTER_ARGS` (lines 80-81); `export TRANSCRIPTS_DIR := $(TRANSCRIPTS)`; add `include-transcript.lua` to the `md`/`html`/`tex` target prerequisites (lines 116/139/172); export/pass a `TRANSCRIPTS_DIR` (pointed at `tests/fixtures/`) into the `filter-smoke` target (line 285) so the smoke's include resolves.
- **EDIT** `docs/manual/tests/filter-smoke.sh`: add `--lua-filter include-transcript.lua` (prepended) + `--lua-filter wrap-long-code.lua` to the PDF leg (lines 75-81), parse/consume a `TRANSCRIPTS_DIR`, add the sample-C whole-include fence to `tests/fixtures/filter-smoke.md`, and assert the included >64-char xpub wraps in the LaTeX output.
- **EDIT** `docs/manual/tests/fixtures/filter-smoke.md`: add the sample-C whole-include fence.
- **EDIT** 2 sample chapter `.md`: `22-first-bundle.md` (sample A excerpt `lines="2-21"`) + `24-recover.md` (sample B excerpt `lines="2-4"`; optionally the md1 `lines="1-1"` fence too). NOTE: these are EXCERPT conversions only — neither chapter gains a whole-include (the whole-include leg is proven by the C fixture, not a shipped chapter).
- **RUNNER UNIFICATION (P0a readies, conversions land in P1a/P2/P4):** `rm`+symlink `docs/technical-manual/tests/verify-examples.sh` and `docs/manual-gui/tests/verify-examples.sh`; insert `MK_BIN` into `docs/quickstart/Makefile` (~line 202) and `docs/manual-gui/Makefile` (after line 234). **P0a MAY land the manual-gui + quickstart Makefile MK_BIN inserts + the runner symlinks (low-risk, no transcripts yet); MUST DEFER the tech-manual runner flip to P1a** (the relative-manifest-path `.cmd` rewrite must precede it).
- **NO CI workflow edits in P0a** beyond confirming `manual.yml` propagates `TRANSCRIPTS_DIR` (it does, via Makefile export). Specifically: do NOT rewrite `manual.yml`'s bare `MD_BIN=md`/`MS_BIN=ms`/`MK_BIN=mk` args — they are correct in CI (§3/§4(d) [R0-FOLD]).

---

## 7. Open items handed to later phases (NOT P0a)

- P1a: rewrite the 4 cargo-example `.cmd` relative manifest-paths to absolute before flipping the tech-manual runner symlink; triage the 15 never-CI-replayed goldens (doc-stale vs binary-regressed) BEFORE repair.
- P0b: per-block whole/excerpt triage + convert ~95 manual blocks (apply the §1.5 mechanical pre-check — a leading/trailing `note:`/`warning:` drop ⟹ excerpt, not whole-include; the `24-recover` blocks are the cautionary datapoint).
- P2/P4: per-book CI `cargo install` + real-bin `make verify-examples`/`audit` switch (bare `md`/`ms`/`mk` OK once the pins are installed into `$PATH` — the absolute-path rule is LOCAL-only per §4(d)); manual-gui `step "N/7"`→`N/8` renumber.
- PE: tech-manual `CHANGELOG.md:7` PDF-release contradiction.
