# BRAINSTORM — A/B/C FOLLOWUP release plan (post-manual-v0.2.0)

**Date:** 2026-05-20
**Source SHA at brainstorm time:** `8a60cdcb0e9a573204a5710512c31dfd779c312a` (master ≡ origin/master)
**Cycle-prep recon dossier:** `cycle-prep-recon.md` (this branch root)
**Source FOLLOWUPs (filed in manual-v0.2.0 cycle):** `design/FOLLOWUPS.md:2657-2710`

This brainstorm decomposes the 5 FOLLOWUPs filed during the manual-v0.2.0 content-audit cycle into 4 release cycles (5 tags total). It does NOT implement; the next phase invokes the `writing-plans` skill per cycle to produce a `PLAN_*.md` per release.

## Decisions locked (with the user, in this brainstorm session)

- **Cycle decomposition:** 4 cycles, 5 tags. A1 ships as a paired toolkit+GUI tag (2 tags, 1 cycle); A2, B, and C each ship as 1 cycle with 1 tag.
- **Sequencing:** dependency-respecting parallel batch.
  - Wave 1 (parallel-safe): A2 + B.
  - Wave 2 (separate cadence): A1 + C.
- **Within-wave ordering:**
  - Wave 1: B first, then A2.
  - Wave 2: A1 first, then C.
- **C scope partitioning:** single large cycle `manual-v0.3.0` (no sub-partition).

## FOLLOWUP-to-cycle mapping

| FOLLOWUP slug | Group | Cycle | Tag(s) |
|---|---|---|---|
| `emitinputs-canonical-descriptor-checksum-invariant-enforcement` | A2 | Cycle 1 | `mnemonic-toolkit-v0.28.3` |
| `manual-md-bin-real-binary-promote` | B-md | Cycle 2 | `manual-v0.2.1` |
| `manual-ms-bin-real-binary-promote` | B-ms | Cycle 2 | `manual-v0.2.1` |
| `export-wallet-coldcard-multisig-alias` | A1 | Cycle 3 | `mnemonic-toolkit-v0.28.4` + `mnemonic-gui-v0.X` |
| `manual-chapters-22-23-24-post-v0.15.0-wire-format-refresh` | C | Cycle 4 | `manual-v0.3.0` |

## Cycle inventory

| # | Cycle | Tags | FOLLOWUPs closed | Estimated effort | Reviewer |
|---|---|---|---|---|---|
| 1 | `mnemonic-toolkit-v0.28.3` (A2) | 1 | 1 | ~30-50 LOC src + 2-5 test cells; ~1 hour | sonnet |
| 2 | `manual-v0.2.1` (B) | 1 | 2 | ~15-25 LOC manual.yml + opportunistic install.sh sibling-pin reconcile; ~30 min | sonnet |
| 3 | `mnemonic-toolkit-v0.28.4` + `mnemonic-gui-v0.X` (A1) | 2 (paired) | 1 | ~20-40 LOC toolkit + 10-20 LOC GUI schema + 10 LOC manual prose; ~2-3 hours | opus |
| 4 | `manual-v0.3.0` (C) | 1 | 1 | ~200-500 LOC manual diffs + 4 transcript recaptures + 6 chapter prose audits; **3-5 days** (per architect I3 fold) | opus |

## Sequencing waves

```
Wave 1 (parallel-safe):
  ├── Cycle 2 (B)  ─── manual-v0.2.1 ──── ships first
  └── Cycle 1 (A2) ─── v0.28.3 ─────────── ships second

Wave 2 (separate cadence):
  ├── Cycle 3 (A1) ─── v0.28.4 + GUI ──── ships first
  └── Cycle 4 (C)  ─── manual-v0.3.0 ──── ships second (depends on Wave 1 #2 for CI binaries)
```

Dependency model (verified):
- A1 ⊥ A2 ⊥ C (file-disjoint; no overlap)
- C → B (C's chapter-42/43 audit needs B's real `md`/`ms` binaries in CI to actually validate prose-vs-binary drift; otherwise C's CI gate silently passes against placeholders — same masking-class bug as the pre-v0.2.0 quickstart-chapters drift)
- A1 ↔ chapter-45 prose: A1 closes the format-name asymmetry; chapter-45's "Format-name asymmetry note" added in P2c (commit `5d2c0a6`) becomes historical-context and is updated in Cycle 3 Phase 5

## Per-cycle phase decomposition

### Cycle 2 — `manual-v0.2.1` (B) — Wave 1 first

1. **Phase 0 — recon:** confirm latest sibling-CLI tags vs `scripts/install.sh:35` (md-cli), `:38` (ms-cli), `:42` (mk-cli); check crates.io availability for cargo-install steps.
2. **Phase 1 — manual.yml edits:**
   - Add `cargo install --git https://github.com/bg002h/descriptor-mnemonic --tag descriptor-mnemonic-md-cli-v0.6.0 md-cli` step (mirror existing mk-cli pattern at L72-77).
   - Add `cargo install --git https://github.com/bg002h/mnemonic-secret --tag ms-cli-v0.4.0 ms-cli` step.
   - Bump existing mk-cli install pin from `mk-cli-v0.2.0` → `mk-cli-v0.4.1` (incidental cross-pin reconcile; surfaced in cycle-prep recon §4).
   - Flip `MD_BIN=true` → `MD_BIN=md` and `MS_BIN=true` → `MS_BIN=ms` in the Audit manual step (`manual.yml:85-96`).
3. **Phase 2 — local validation:** run `make audit MNEMONIC_BIN=… MD_BIN=md MS_BIN=ms MK_BIN=mk` with `cargo install`-installed binaries. Expect new findings from the previously-vacuous flag-coverage warns for chapters 42/43 — these become FOLLOWUPs to be folded into Cycle 4, NOT this cycle.
4. **Phase 3 — actionlint + sonnet fold-verify.**
5. **Phase 4 — commit + tag `manual-v0.2.1` + push + GH Release.**
6. **Phase 5 — FOLLOWUPS.md Status flips** for both `manual-{md,ms}-bin-real-binary-promote` → `resolved <commit-sha>`.

### Cycle 1 — `mnemonic-toolkit-v0.28.3` (A2) — Wave 1 second

1. **Phase 0 — design lock:** newtype `CheckedDescriptor<'_>(&'_ str)` (per architect lean: newtype wins on surface area + compiler-enforced invariant vs constructor-assertion alternative).
2. **Phase 1 — TDD red:** 2-3 cells in `crates/mnemonic-toolkit/tests/` asserting that a stripped-body descriptor cannot construct `EmitInputs` (compile-error OR runtime-panic per newtype shape).
3. **Phase 2 — impl:**
   - Add `CheckedDescriptor<'_>(&'_ str)` newtype + constructor + `Deref` impl in `crates/mnemonic-toolkit/src/wallet_export/mod.rs`.
   - Change `EmitInputs.canonical_descriptor` field type at `mod.rs:345` from `&str` → `CheckedDescriptor<'_>`.
   - Wrap the value at 2 construction sites: `cmd/export_wallet.rs:437` (serves both `--template` AND `--descriptor` modes per architect I1 — same `run()` site) + `cmd/export_wallet.rs:608` (`run_from_import_json`).
4. **Phase 3 — bsms.rs invariant comment update:** `wallet_export/bsms.rs:86-90` comment changes from "by convention" to "by type" (the invariant is now compiler-enforced).
5. **Phase 4 — TDD green + clippy + full test suite:** expect 1996 → ~1999 cells.
6. **Phase 5 — sonnet reviewer fold-verify** (small refactor; sonnet appropriate).
7. **Phase 6 — commit + tag + push:**
   - Commit MUST include `scripts/install.sh:32` bump `mnemonic-toolkit-v0.28.2` → `mnemonic-toolkit-v0.28.3` (per architect I5 + install-pin-check.yml CI gate).
   - Commit MUST include Cargo.toml version bump + CHANGELOG.md entry (matches v0.28.2 P2a precedent at commit `615b10e`).
   - Tag `mnemonic-toolkit-v0.28.3` + push.
   - install-pin-check.yml CI gate confirms self-pin match.
   - Create GH Release manually (`gh release create`) — convention per `project_v0_28_1_patch_shipped` + Cycle in this session.
8. **Phase 7 — FOLLOWUPS Status flip** for `emitinputs-canonical-descriptor-checksum-invariant-enforcement` → `resolved <commit-sha>`.

### Cycle 3 — `mnemonic-toolkit-v0.28.4` + `mnemonic-gui-v0.X` (A1) — Wave 2 first

1. **Phase 0 — cross-repo recon:** verify `../mnemonic-gui` is on a stable branch + capture current `schema_mirror` baseline against the toolkit pin (will be `v0.28.3` post-Wave-1).
2. **Phase 1 — toolkit src:** add `CliExportFormat::ColdcardMultisig` variant at `cmd/export_wallet.rs:22-36` (per cycle-prep recon §A1 — the FOLLOWUP body cites `wallet_export/mod.rs` but actual location is `cmd/export_wallet.rs:22-36`; **STRUCTURAL fix per recon dossier** is to amend the FOLLOWUP citation in the same PR). Emit dispatch: multisig-template precheck (refusal pointer for singlesig templates routes through error message); same-body delegation to existing `Coldcard` dispatch path.
3. **Phase 2 — toolkit tests:** 2-3 cells in `crates/mnemonic-toolkit/tests/cli_export_wallet_coldcard.rs` exercising `--format coldcard-multisig --template wsh-sortedmulti` (happy path) + singlesig-template-refusal (e.g., `--template bip84` should error with pointer text).
4. **Phase 3 — gui-schema JSON regen:** rebuild toolkit binary; run `mnemonic gui-schema` to emit the new enum value; pin in `mnemonic-gui/src/schema/mnemonic.rs`.
5. **Phase 4 — GUI dropdown wiring:** add new value to relevant `--format` dropdown(s) in `mnemonic-gui/src/`.
6. **Phase 5 — chapter-45 prose touch-up:** the "Format-name asymmetry note" I added in P2c (manual-v0.2.0 cycle `5d2c0a6`) becomes historical-context. Rewrite to acknowledge the asymmetry was closed in v0.28.4. Anchor: `docs/manual/src/45-foreign-formats.md` (the note block immediately below the chapter-45 §Coldcard multisig Round-trip example).
7. **Phase 6 — opus reviewer dispatch** (cross-repo cycle warrants opus per `feedback_opus_primary_review_agent`).
8. **Phase 7 — paired commit + tags:**
   - **Toolkit:** commit (includes `install.sh:32` bump v0.28.3 → v0.28.4 per architect I5) + tag `mnemonic-toolkit-v0.28.4` + push.
   - **GUI:** bump `mnemonic-gui/pinned-upstream.toml` toolkit pin v0.28.3 → v0.28.4 + commit + tag `mnemonic-gui-v0.X` + push.
9. **Phase 8 — closure-verification step (per architect I4):** confirm GUI repo's CI run on the new GUI tag fires `schema_mirror`-gate GREEN against the bumped toolkit pin. This is the **lagging-indicator gate**; without it, the cycle isn't actually closed. The gate does NOT fire from the toolkit-tag CI alone (`install-pin-check.yml` only validates the toolkit self-pin). CLAUDE.md §"GUI schema-mirror coverage" (L23-34) is the canonical reference; v0.27.2 historical case study (8 accumulated missing flags) is the worked example.
10. **Phase 9 — FOLLOWUPS Status flip** for `export-wallet-coldcard-multisig-alias` → `resolved <toolkit-sha>` with note about paired GUI tag.

### Cycle 4 — `manual-v0.3.0` (C) — Wave 2 second

**Effort estimate: 3-5 days (per architect I3 fold — manual-v0.2.0 took multi-session work for 3 chapters; 9 chapters ≈ 3× throughput).**

1. **Phase 0 — multi-chapter recon + FOLLOWUP body amendment:**
   - Grep `'ms10entrsq\|mk1qprsqhp\|md1zsxdsp'` across all 9 chapters; pin every stale-card-string line.
   - **Amend `design/FOLLOWUPS.md:2706`** to "9 total = 3 quickstart (22/23/24) + 6 cross-reference (31/35/41/42/43/44)" per architect I2 fold. The current body says "9 OTHER" but parenthetically lists 6 — self-inconsistent.
   - Build a finding table similar to `design/AUDIT_FINDINGS_manual_v0_28_0_content.md`.
2. **Phase 1 — recapture 4 transcripts:** re-run against latest toolkit binary at cycle-start time (likely v0.28.4 post-Cycle-3 OR v0.28.3 if Cycle 3 still in flight). Files: `docs/manual/transcripts/{22-first-bundle,23-verify,24-recover,24-recover-md1}.{cmd,out}`. Apply same `$MNEMONIC_BIN` + `$FIXTURES_DIR` convention from the manual-v0.2.0 P3 wiring (commit `52f33f7`).
3. **Phase 2 — audit chapter prose:** for each of the 9 chapters, run the documented commands end-to-end (per `feedback-architect-must-run-prose-commands`). Compare actual output against documented claims. Record findings per chapter.
4. **Phase 3a — P1b architect classification** (new sub-phase per architect I3 fold; mirrors manual-v0.2.0's P1b R0+R1 discipline): doc-update vs toolkit-fix per finding; gray-area locks for any behavior-vs-doc drift. Opus dispatch.
5. **Phase 3b — apply prose updates** for doc-update findings; recompute md1/mk1/ms1 strings in prose to match recaptured transcripts; cascade fingerprint-case corrections per BIP-388 lowercase rule (the F10/F10b drift class).
6. **Phase 4 — remove SKIP_STEMS** from `docs/manual/tests/verify-examples.sh`. Single-line removal per stem (the 4-entry block).
7. **Phase 5 — local `make audit`:** confirm all 14 transcripts (10 active + 4 newly-readmitted) pass via `make audit MNEMONIC_BIN=… MD_BIN=md MS_BIN=ms MK_BIN=mk FIXTURES_DIR=…`.
8. **Phase 6 — opus end-of-cycle review** (mirror v0.2.0's P5.2 R0+R1 cadence: YELLOW → fold → R1 → GREEN).
9. **Phase 7 — commit + tag `manual-v0.3.0` + push + GH Release** with refreshed PDF (auto-built by manual.yml workflow on tag push).
10. **Phase 8 — FOLLOWUPS Status flip** for `manual-chapters-22-23-24-post-v0.15.0-wire-format-refresh` → `resolved <commit-sha>`.

## Cross-cutting concerns

### install-pin-check.yml self-pin discipline (Cycles 1 + 3)

Per architect I5 + v0.18.1 precedent (memory `project_v0_18_1_v0_7_2_b1_bugfix_closed`): every toolkit-tag commit MUST also bump `scripts/install.sh:32` to the new tag value. The `install-pin-check.yml` workflow gate (`.github/workflows/install-pin-check.yml`) fires on `mnemonic-toolkit-v*` tag push and validates that `install.sh`'s self-pin matches the tag. Pre-this-discipline drift was a real recurring class.

Applies to:
- Cycle 1 Phase 6 commit (v0.28.2 → v0.28.3)
- Cycle 3 Phase 7 toolkit commit (v0.28.3 → v0.28.4)
- NOT Cycle 2 (manual-only tag; install.sh has no manual-side pin)
- NOT Cycle 4 (same)

### GUI schema-mirror lagging-indicator (Cycle 3)

Per architect I4 + CLAUDE.md §"GUI schema-mirror coverage" (L23-34): the `schema_mirror` test on the GUI side is a **lagging indicator**, not a leading one. It fires only on GUI pin-bump CI run, not on toolkit-tag CI. The discipline therefore is:
1. Make the schema-mirror update + dropdown wiring on the GUI side in the same paired-PR cycle.
2. Push toolkit tag.
3. Push GUI tag (with toolkit pin-bump).
4. **Watch the GUI tag's CI run for `schema_mirror`-gate result** (GREEN required to declare cycle closed).

If this step (Cycle 3 Phase 8) is skipped, drift accumulates silently until the next GUI pin-bump catches the cumulative delta — v0.27.2 historical case study showed 8 accumulated missing flags.

### crates.io publish (Cycles 1 + 3)

Toolkit tags v0.28.3 + v0.28.4 are blocked from crates.io publish on miniscript `[patch.crates-io]` (same as v0.22.x+ tags per `project_v0_24_0_cycle_shipped`). Both ship via git+tag only; `install.sh` continues to handle the install-path via git ref.

### Branch convention

Master-direct commits for all 4 cycles (matches v0.28.2 + manual-v0.2.0 precedent shipped earlier this session). No release/* branch unless a future cycle warrants longer-running parallel work.

### Reviewer mode per cycle

- **Cycle 1 (A2):** sonnet — small refactor; mechanical newtype substitution.
- **Cycle 2 (B):** sonnet — small CI yml edit; actionlint as the structural gate.
- **Cycle 3 (A1):** opus — cross-repo coordination; ergonomic surface change with downstream GUI impact.
- **Cycle 4 (C):** opus — judgment-heavy multi-chapter audit; P1b classification needs opus per `feedback_opus_primary_review_agent`.

### FOLLOWUP closure pattern

Each cycle's final commit message references the closed FOLLOWUP slug(s) + commit-SHA pin. Status flips happen in the same commit (post-resolution). Matches manual-v0.2.0 cycle pattern at commits `52f33f7` (P3 closes `inheritance-example-transcript-coverage` + partial-closes `manual-yml-bind-real-mnemonic-bin`) + `fe32e9e` (P4+P5.1 successor entries).

## Architect review folds applied (this brainstorm session)

Opus end-of-cycle architect dispatched on the Section 1 + Section 2 draft returned **YELLOW** with 5 Important findings; all folded inline before this doc was written:

| # | Finding | Fold location |
|---|---|---|
| I1 | A2 has 2 EmitInputs construction sites, not 3 (recon double-counted L437 for `--template` + `--descriptor` modes) | Cycle 1 Phase 2 — explicit L437 + L608 citation |
| I2 | FOLLOWUP body L2706 self-inconsistent ("9 OTHER" but lists 6) | Cycle 4 Phase 0 — amend FOLLOWUP body to "9 total = 3 + 6" |
| I3 | Cycle 4 effort "1-2 days" understated; v0.2.0 throughput says 3-5 days for 9 chapters | Effort estimate revised to 3-5 days; added explicit Phase 3a P1b classification sub-phase |
| I4 | Cycle 3 schema_mirror is lagging-indicator; gate doesn't fire on toolkit-tag CI | Added Cycle 3 Phase 8 (closure-verification step on GUI CI) |
| I5 | install.sh:32 self-pin must bump in same commit as toolkit tag | Cycle 1 Phase 6 + Cycle 3 Phase 7 toolkit-commit phases call this out explicitly |

## Recon dossier reference

`cycle-prep-recon.md` (at repo root) carries the per-slug citation verification at HEAD `8a60cdc`. Key takeaways:
- 1 STRUCTURAL citation issue (A1 — `CliExportFormat` location); fixed in Cycle 3 Phase 1.
- 1 CLAIM-COUNTING ambiguity (C — "9 OTHER" wording); fixed in Cycle 4 Phase 0.
- Pre-existing cross-pin staleness: `manual.yml:77` mk-cli pin lags `install.sh:42`; addressed in Cycle 2 Phase 1 (incidental).
- No DRIFTED-by-N findings (all 5 FOLLOWUPs filed in current session; sync delta is 0).

## Next phase

Per the `brainstorming` skill flow, the next step is invoking the `writing-plans` skill (one invocation per cycle, OR one invocation for an umbrella plan-doc covering all 4 cycles). User direction needed on plan-doc shape:

- **Option (a):** 4 separate `PLAN_*.md` files in `design/`, one per cycle. Maximum modularity; matches per-cycle execution discipline.
- **Option (b):** 1 umbrella `PLAN_followups_abc.md` covering all 4 cycles. Single read; harder to per-cycle reviewer-loop.
- **Option (c):** Hybrid — 1 umbrella with per-cycle sections; each per-cycle section is then liftable into a fresh plan-doc when execution begins.

Default lean: **(a) — 4 separate plan-docs** — each cycle is a self-contained execution unit with its own R0 reviewer-loop; the umbrella view is captured by this brainstorm doc itself.

---

**Brainstorm session ends here.** This document is the SPEC + sequencing lock for the next 4 cycles. Each cycle's detailed implementation plan is produced by a separate `writing-plans` skill invocation, citing this doc + the cycle-prep recon dossier as authoritative inputs.
