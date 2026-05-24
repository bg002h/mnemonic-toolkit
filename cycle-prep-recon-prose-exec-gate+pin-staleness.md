# cycle-prep recon — 2026-05-24 — manual-prose-command-execution-gate + manual-yml-and-install-sh-sibling-gui-pin-staleness

**Origin/master SHA at recon time:** `296dca2`
**Local branch:** `master`
**Sync state:** `up-to-date (0 ahead / 0 behind)`
**Untracked:** `.claude/`

Slug(s) verified: `manual-prose-command-execution-gate`, `manual-yml-and-install-sh-sibling-gui-pin-staleness`. **Both CLEAN — every citation ACCURATE against current source** (both were filed at the v0.36.3 ship one cycle ago, so no decay). They differ sharply in character: the pin-staleness is a trivial config bump; the prose-exec gate is a meaty test-infra build with a real design challenge (placeholder recipes).

---

## Per-slug verification

### `manual-prose-command-execution-gate`
- **WHAT:** the manual lint validates flag NAMES but never EXECUTES documented recipes; build a stage/test that runs them against the pinned binary.
- **Citations:**
  - `docs/manual/tests/lint.sh` "6 stages" — **ACCURATE.** Header (`:3-9`) + `step "N/6 …"` markers: 1 markdownlint, 2 cspell, 3 lychee (`:55`), 4 flag-coverage (`:63`), 5 glossary, 6 index-bidirectional. None executes documented recipes.
  - "NEVER EXECUTES the documented commands" — **ACCURATE.** The only `eval` is stage-4 `flags=$(eval $cmd …)` (`:84`) — and `$cmd` is `$bin $sub --help` (flag extraction), NOT a prose recipe block. No code-block execution anywhere.
  - lychee `--include-fragments` absent — **ACCURATE.** `:57` `lychee --offline --no-progress "$SRC_DIR"` (no `--include-fragments` → intra-doc `#anchor` fragments unchecked).
  - `design/AUDIT_FINDINGS_manual_v0_28_0_content.md` (v0.28.1 breakage record) — **ACCURATE** (present; documents all 6 chapter-45 recipes failing at the `export-wallet` step).
  - "all 6 chapter-45 recipes" — **ACCURATE.** `45-foreign-formats.md` has exactly 6 `### Round-trip example` blocks.
  - `feedback_architect_must_run_prose_commands` (memory) — records the manual discipline; ACCURATE as cited.
- **Action for brainstorm spec:** the design challenge (NOT in the FOLLOWUP body) is that **most manual code blocks use `...` ellipsis placeholders** (`abandon abandon ... art`, `xpub6...`, `--cosigner xpub6A...:fingerprint1:…`) that are NOT directly executable. The gate must either (a) run a CURATED subset of fully-runnable recipes (with real fixtures), (b) maintain a parallel runnable-recipe corpus, or (c) make the documented recipes themselves runnable (replace placeholders with real test vectors). Cite SHA `296dca2`. This is the load-bearing R0 question.

### `manual-yml-and-install-sh-sibling-gui-pin-staleness`
- **WHAT:** non-`mnemonic` install pins (manual.yml siblings + install.sh GUI) lag, ungated by install-pin-check.
- **Citations:**
  - `manual.yml:77` `mk-cli-v0.4.1` — **ACCURATE** (`cargo install … --tag mk-cli-v0.4.1`).
  - `manual.yml:84` `descriptor-mnemonic-md-cli-v0.6.0` — **ACCURATE.**
  - `manual.yml:88` `ms-cli-v0.4.0` — **ACCURATE.**
  - `install.sh:32` `mnemonic-toolkit-v0.36.3` — **ACCURATE** (current; the install-pin-check-gated self-pin).
  - `install.sh:35` `descriptor-mnemonic-md-cli-v0.6.1`, `:38` `ms-cli-v0.4.1`, `:41` `mk-cli-v0.4.2` — **ACCURATE** (current sibling tags, ahead of manual.yml).
  - `install.sh:44` `mnemonic-gui-v0.10.0` — **ACCURATE + STALE.** Live GUI = `mnemonic-gui-v0.21.1` (verified: latest GUI tag + GUI `Cargo.toml version = "0.21.1"`) → **11 versions stale** (0.10→0.21). The default all-5 `install.sh` installs a 10+-version-stale GUI.
- **Action for brainstorm spec:** bump `manual.yml:77/84/88` → `mk-cli-v0.4.2`/`descriptor-mnemonic-md-cli-v0.6.1`/`ms-cli-v0.4.1` (match install.sh); bump `install.sh:44` → `mnemonic-gui-v0.21.1`. Verify install.sh siblings are themselves current at impl (no newer mk/ms/md tags than v0.4.2/v0.4.1/v0.6.1). Cite SHA `296dca2`.

---

## Cross-cutting observations
1. **Both CLEAN, zero drift** — filed one cycle ago with live citations; the only "drift" is the intended target staleness (install.sh:44 GUI pin), which IS the finding.
2. **The two slugs are SEPARABLE + very different effort.** Pin-staleness = ~4-line config edit (manual.yml ×3 + install.sh ×1), no new logic, no test. Prose-exec gate = a recipe-extraction + execution harness (the meaty part) with the placeholder-recipe design problem.
3. **Higher-impact-first:** the install.sh:44 GUI pin (users get GUI v0.10.0 from the default installer) is the most user-visible defect of the two and is a trivial fix — worth doing early/first.
4. **No install-pin-check coverage** for either pin site (it gates only install.sh:32 `mnemonic`). A durable fix would add a check (out of scope; note for the prose-gate cycle which is already touching CI).
5. **The prose-exec gate could surface NEW content breakage** — running the 6 round-trip recipes against v0.36.3 may reveal a still-broken one (they were fixed reactively at v0.28.1+ but never gated since). If so, the cycle escalates to a content fix. R0 + impl must budget for that.

---

## Recommended brainstorm-session scope
- **ONE combined PATCH cycle (v0.36.4), PHASED — pin-staleness FIRST (quick, high-impact), prose-gate SECOND (meaty).** Both are manual/CI/installer hygiene; combining keeps one tag. SemVer **PATCH** (test/CI/installer config + a new test stage; NO CLI surface change → NO GUI schema_mirror lockstep; manual workflow fires on lint.sh/docs changes). Precedent: v0.28.5/v0.36.2/v0.36.3 docs/test PATCHes.
- **Phase 1 — pin-staleness (trivial):** bump `manual.yml:77/84/88` to match install.sh siblings; bump `install.sh:44` GUI → `mnemonic-gui-v0.21.1`. ~4 lines. No new test (verify CI still installs cleanly).
- **Phase 2+ — prose-exec gate (the cycle's substance):** R0 must settle:
  - (a) **Harness location:** a Rust integration test (`tests/manual_recipes.rs`, runs under the `rust` CI job, robust extraction) vs a new `lint.sh` stage-7 (bash, closer to the manual lint but fragile extraction). Lean Rust test (the project's gates are Rust tests; bash recipe-running is brittle).
  - (b) **The placeholder problem (LOAD-BEARING):** manual recipes use `...`/`xpub6...` placeholders → not directly executable. Options: curated runnable subset with real fixtures (a tagged code-fence convention, e.g. ```` ```bash {.runnable} ````), a parallel runnable-recipe corpus, or making recipes runnable. R0 to pick.
  - (c) **Scope of coverage:** the 6 chapter-45 round-trip recipes only (the historically-broken class), vs all CLI-reference worked examples. Start with the 6.
  - (d) **lychee `--include-fragments`** anchor-validation add (closes the dangling-anchor gap from v0.36.3 C2) — fold into this cycle or a sub-FOLLOWUP.
  - (e) **escalation budget:** if running the recipes reveals a broken one, fix the recipe/doc in the same cycle (per-phase reviewer-loop).
- **Sizing:** Phase 1 ~4 lines. Phase 2 medium-large (extraction + run + fixtures; the placeholder design is the work, not LOC). No sibling-codec companions. install.sh GUI pin is NOT install-pin-check-gated (so no toolkit-tag lockstep for it).
- **Inter-slug:** independent; no ordering dependency, but pin-staleness first banks the user-visible win before the meatier gate work.
