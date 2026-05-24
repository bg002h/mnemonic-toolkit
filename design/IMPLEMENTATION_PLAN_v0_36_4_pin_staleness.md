# v0.36.4 Implementation Plan — pin-staleness (manual.yml siblings + install.sh GUI pin)

> Per-phase reviewer-loop until 0C/0I. NO parallel code-gen. This is a config/CI/installer PATCH — no toolkit logic, no test logic.

**Goal:** Close `manual-yml-and-install-sh-sibling-gui-pin-staleness` — bump the stale non-`mnemonic` install pins (the default installer hands users an 11-version-stale GUI). Ship as a standalone PATCH v0.36.4 (user decision); the prose-exec gate + the `--from-import-json` CLI fix become follow-on cycles.

**Architecture:** pure pin/version edits across `manual.yml`, `scripts/install.sh`, `Cargo.toml`/`Cargo.lock`, BOTH READMEs (marker lockstep), CHANGELOG. No code, no new tests.

**Source SHA:** recon-verified vs `origin/master` @ `296dca2` (`cycle-prep-recon-prose-exec-gate+pin-staleness.md`).

---

## Context — what the prose-gate recon surfaced (decisions taken)
Test-running the chapter-45 recipes revealed **5 of 6 round-trip recipes are impossible as written** (template-requiring formats — sparrow/coldcard/jade/electrum — cannot round-trip via `--from-import-json`; it carries a descriptor, and `--from-import-json` `conflicts_with` `--template`; the CLI says so explicitly). **User decisions:** (1) fix direction = **CLI fix** (make `--from-import-json --format <template-format>` auto-derive the template from the envelope script_type) — a MINOR feature, its own cycle + R0; (2) ship the pin-staleness NOW as a standalone v0.36.4. So THIS cycle is ONLY the pin bumps; two FOLLOWUPs capture the rest.

## SemVer + lockstep
- **PATCH → v0.36.4** (config/CI/installer only; no CLI surface change → NO GUI schema_mirror lockstep).
- **R0 I1 — CI validation this cycle (corrected):** a `manual.yml`/`quickstart.yml`-only change does NOT fire those workflows (their `paths:` filters are `docs/manual/**` / `docs/quickstart/**`, NOT the workflow files themselves). So the bumped sibling pins validate LAZILY (next docs change / `manual-v*` tag) — NOT on this push. This cycle's actual gates: **`rust`** (the README-marker guard + version + suite) + **`install-pin-check`** (on the `mnemonic-toolkit-v0.36.4` tag, greps install.sh:32). All bumped target tags are verified-existing, so no `cargo install` breakage when the workflows DO next fire. (Do NOT add the workflow files to their own `paths` — keep this cycle config-only.)
- **README-marker lockstep (the trap):** the v0.36.3 guard `tests/readme_version_current.rs` asserts BOTH READMEs carry `<!-- toolkit-version: X -->` == `CARGO_PKG_VERSION`. Bumping `Cargo.toml`→0.36.4 ⇒ BOTH README markers MUST bump to 0.36.4 or the `rust` CI job fails. (This is the guard working as designed.)

## The exact edit set (recon-verified targets)
- `.github/workflows/manual.yml:77` `mk-cli-v0.4.1` → `mk-cli-v0.4.2`
- `.github/workflows/manual.yml:84` `descriptor-mnemonic-md-cli-v0.6.0` → `descriptor-mnemonic-md-cli-v0.6.1`
- `.github/workflows/manual.yml:88` `ms-cli-v0.4.0` → `ms-cli-v0.4.1`
- `scripts/install.sh:44` `mnemonic-gui-v0.10.0` → `mnemonic-gui-v0.21.1` (the high-impact one; latest GUI tag confirmed)
- `.github/workflows/quickstart.yml:71` `mk-cli-v0.2.0` → `mk-cli-v0.4.2` (R0 M1: an even-staler 3rd site; same defect class — close the whole class)
- **(R0 M2) do NOT touch `manual-gui.yml`** — it derives the GUI tag from `docs/manual-gui/pinned-upstream.toml` (`mnemonic-gui-v0.3.0`), intentionally version-locked to the GUI-manual authoring snapshot (re-pinned by a GUI-manual cycle, not this one).
- `scripts/install.sh:32` `mnemonic-toolkit-v0.36.3` → `mnemonic-toolkit-v0.36.4` (self-pin, install-pin-check)
- `crates/mnemonic-toolkit/Cargo.toml` `0.36.3` → `0.36.4`; `Cargo.lock` regen
- `README.md` + `crates/mnemonic-toolkit/README.md` markers `0.36.3` → `0.36.4`
- `CHANGELOG.md` `[0.36.4]` PATCH entry
- (install.sh siblings `:35/38/41` are already CURRENT — md-v0.6.1/ms-v0.4.1/mk-v0.4.2 — do NOT touch.)

---

## Phase 1 — pin + version edits

- [ ] **Step 1** — apply all edits above (manual.yml ×3; install.sh:44 GUI + :32 self-pin; Cargo.toml; both README markers).
- [ ] **Step 2** — `cargo update -p mnemonic-toolkit` (Cargo.lock → 0.36.4).
- [ ] **Step 3** — `cargo test --test readme_version_current` → GREEN (both markers = 0.36.4); `cargo build --bin mnemonic` → `mnemonic 0.36.4`.
- [ ] **Step 4 — CHANGELOG** `[0.36.4]` entry (config-only PATCH; names the GUI-pin fix + the two follow-on FOLLOWUPs).
- [ ] **Step 5** — commit.

## Phase 2 — FOLLOWUPs + ship

- [ ] **Step 1 — FOLLOWUPS:** mark `manual-yml-and-install-sh-sibling-gui-pin-staleness` `resolved (v0.36.4)` — broaden its closing note to cover `quickstart.yml:71` too (R0 M1). Use the CANONICAL slug in the CHANGELOG (R0 M4: not the v0.36.3 shorthand). File **`export-wallet-from-import-json-template-format-reemit`** (the CLI fix: `--from-import-json --format <sparrow|coldcard|jade|electrum>` should auto-derive the `--template` from the envelope's `script_type` so template-requiring formats round-trip; today it errors "descriptor passthrough is not supported"; `--from-import-json` `conflicts_with` `--template` so the user can't supply it; MINOR feature; `script_type_from_descriptor` (`wallet_export/mod.rs:211`) makes it feasible; own R0; **R0 M3: the next R0 must budget for the multisig inverse-ambiguity** — `WalletScriptType→CliTemplate` is 1:1 for singlesig but ambiguous for multisig (P2wshMulti ← WshMulti|WshSortedMulti; P2trMulti ← TrMultiA|TrSortedMultiA, `script_type_from_template:191-203`)). Update **`manual-prose-command-execution-gate`**: note the 5 chapter-45 round-trip recipes (sparrow/coldcard/jade/electrum) are BLOCKED on the CLI fix; the gate can initially cover the WORKING recipes (specter + descriptor-passthrough re-emits to bitcoin-core/bip388/bsms) and expand once the CLI fix lands.
- [ ] **Step 2 — end-of-cycle opus review** → persist `design/agent-reports/v0_36_4-end-of-cycle-review.md` → fold → GREEN.
- [ ] **Step 3 — ship:** merge→master (ff), tag `mnemonic-toolkit-v0.36.4`, push, GH release; verify rust + manual + install-pin-check CI. No GUI cycle.

---

## Self-review / R0 dispositions folded (all RESOLVED)
- **I1:** corrected the CI-validation claim (manual.yml/quickstart.yml don't fire on workflow-file-only edits; pins validate lazily; this cycle's gates = rust + install-pin-check).
- **M1:** added `quickstart.yml:71` to the edit set (3rd stale site); FOLLOWUP broadened.
- **M2:** manual-gui.yml explicitly NOT touched (intentionally version-locked).
- **M3:** multisig inverse-ambiguity note added to the CLI-fix FOLLOWUP.
- **M4:** canonical slug in the v0.36.4 CHANGELOG.
- Confirmed: README-marker lockstep handled (both → 0.36.4); pin targets all live + latest (gui v0.21.1, mk v0.4.2, ms v0.4.1, md v0.6.1) + verified-existing tags (no cargo-install break); PATCH/no-GUI-lockstep correct; tagging needed for install-pin-check + README-guard lockstep.
