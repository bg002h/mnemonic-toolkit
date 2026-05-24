# v0.36.4 — Plan-doc R0 architect review (opus) — MANDATORY pre-implementation gate

**Date:** 2026-05-24
**Cycle:** v0.36.4 pin-staleness PATCH (manual.yml siblings + install.sh GUI pin)
**Branch:** `v0.36.4-prose-gate-pins`
**Reviewer:** opus (feature-dev:code-reviewer), R0 (agentId a05a246ce80e2a158)
**Target:** `design/IMPLEMENTATION_PLAN_v0_36_4_pin_staleness.md`

---

## Critical
None.

## Important
**I1 — the plan's "manual workflow fires (manual.yml changed)" claim is FALSE; the bumped sibling pins get NO CI validation this cycle.** `manual.yml:8-19` triggers only on `push`/`pull_request` with `paths: ['docs/manual/**', …]` or `manual-v*` tags — the workflow's OWN path is not in the filter, so a manual.yml-only change does NOT run it. The bumped pins are exercised lazily (next docs change / `manual-v*` tag). NOT a breakage (all target tags verified to exist), but the plan's verification premise is wrong. FOLD: correct the claim — this cycle's CI = `rust` (README guard + version + suite) + `install-pin-check` (tag); the workflow pins validate lazily. (Option b — add `.github/workflows/manual.yml` to its own `paths` — is a separate real fix; reviewer recommends keeping config-only.)

## Minor
- **M1 — `quickstart.yml:71` has an EVEN-staler `mk-cli-v0.2.0`** (live mk = v0.4.2). Same defect class; not in the FOLLOWUP's stated scope. quickstart.yml also won't fire on this PR (paths `docs/quickstart/**`). FOLD: include the quickstart.yml bump in this cycle (close the whole class) + broaden the resolved-FOLLOWUP note.
- **M2 — `manual-gui.yml:44` clones GUI at `docs/manual-gui/pinned-upstream.toml`'s `mnemonic-gui-v0.3.0`** — intentionally version-locked to the GUI-manual authoring snapshot (a GUI-manual cycle re-pins it). Correctly untouched; note for completeness.
- **M3 — CLI-fix FOLLOWUP feasibility sound but has a multisig inverse-ambiguity wrinkle for ITS OWN R0.** `script_type_from_descriptor` (`wallet_export/mod.rs:211`) derives the script-type from the envelope descriptor (the `template==None` descriptor-mode path, :178-179), so deriving a template is feasible for singlesig (P2wpkh→bip84 etc., 1:1). But the inverse `WalletScriptType → CliTemplate` is AMBIGUOUS for multisig (P2wshMulti ← WshMulti OR WshSortedMulti; P2trMulti ← TrMultiA OR TrSortedMultiA, `script_type_from_template:191-203`). Add a one-line note to the FOLLOWUP so the next R0 budgets for it.
- **M4 — v0.36.3 CHANGELOG:16 used shorthand `manual-yml-sibling-cli-pin-staleness`** vs canonical `manual-yml-and-install-sh-sibling-gui-pin-staleness` (FOLLOWUPS.md:3170). Already-committed; use the canonical slug in the v0.36.4 CHANGELOG.

## Verification summary (confirmed correct)
- **Edit targets (all live + latest):** manual.yml:77/84/88 mk-cli-v0.4.1/md-cli-v0.6.0/ms-cli-v0.4.0 → v0.4.2/v0.6.1/v0.4.1 ✓; install.sh:44 mnemonic-gui-v0.10.0 → v0.21.1 ✓; install.sh:32 → v0.36.4 ✓; Cargo.toml:3 0.36.3→0.36.4 ✓; install.sh siblings :35/38/41 ALREADY current (untouched) ✓.
- **Tag existence (highest-value):** all bumped pins are REAL pushed tags (mk v0.4.2, md v0.6.1, ms v0.4.1, gui v0.21.1) — no `cargo install` CI break. `mnemonic-toolkit-v0.36.4` not yet existing (tagging clear); v0.36.3 = 296dca2 (matches recon).
- **README-marker lockstep:** both READMEs carry `<!-- toolkit-version: 0.36.3 -->` (only occurrences); `tests/readme_version_current.rs` asserts both == CARGO_PKG_VERSION → bumping Cargo→0.36.4 REQUIRES both markers→0.36.4 (else rust job fails). Status lines say "v0.36.x" (version-agnostic; only the marker literal bumps). Plan handles it.
- **install-pin-check:** greps install.sh mnemonic pin == tag; :32→v0.36.4 + tag v0.36.4 stays green; GUI :44 + manual.yml pins NOT gated by it.
- **SemVer/scope:** config-only → PATCH, no GUI lockstep; tagging consistent with v0.28.5/v0.36.2/v0.36.3 + needed for install-pin-check + README-guard lockstep.
- **CLI-fix FOLLOWUP feasibility:** sound (script_type_from_descriptor exists), modulo M3 multisig ambiguity.

VERDICT: RED (0C/1I)

---

## Fold disposition (controller) — R0 → R1
- **I1:** correct the plan's CI-validation claim — manual.yml-only change does NOT fire the manual workflow; pins validate lazily; this cycle's gates = rust + install-pin-check. Do NOT add manual.yml to its own paths (keep config-only).
- **M1:** add `quickstart.yml:71` `mk-cli-v0.2.0` → `mk-cli-v0.4.2` to the edit set; broaden the resolved-FOLLOWUP to cover quickstart.yml.
- **M2:** explicitly note manual-gui.yml is intentionally version-locked (do NOT touch).
- **M3:** add the multisig inverse-ambiguity note to the `export-wallet-from-import-json-template-format-reemit` FOLLOWUP.
- **M4:** use the canonical slug in the v0.36.4 CHANGELOG.
Re-dispatch R1.
