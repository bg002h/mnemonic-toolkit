# cycle-prep recon — 2026-06-05 — gui-schema-restore-required-unless-md1-projection + gui-readme-install-pin-coherence-guard

**Origin/master SHA at recon time:** toolkit `0bd98c2` (v0.46.1 just shipped) · GUI `f6caa20` (v0.26.0)
**Local branch:** toolkit `master`, GUI `master`
**Sync state:** both `up-to-date (0 ahead / 0 behind)`
**Untracked:** recon/survey scratch + `.claude/` (none load-bearing).

Slug(s) verified: `gui-schema-restore-required-unless-md1-projection`, `gui-readme-install-pin-coherence-guard`. **Both citations essentially ACCURATE (one DRIFTED-by-2 line); the real finding is a structural one: slug 1 is a CROSS-REPO TWO-RELEASE arc (toolkit projects → GUI consumes), slug 2 is GUI-only and can ride the GUI half.**

---

## Per-slug verification

### `gui-schema-restore-required-unless-md1-projection`
- **WHAT:** Toolkit `gui-schema`'s `conditional_rules` projection has no `restore` arm → emits `conditional_rules: []` for restore, so the GUI's at-least-one rule (`--from` Required unless `--md1`) is GUI-authored/UNGATED. Add a `restore` arm so the rule is drift-gated by the GUI's `gui_schema_conditional_drift`.
- **Citations:**
  - `crates/mnemonic-toolkit/src/cmd/gui_schema.rs:336-345` (`build_subcommand_conditional_rules`) — **ACCURATE.** `fn` opens `:336`; the `match name` allowlist (`:337-345`) has arms `compare-cost`/`bundle`/`verify-bundle`/`export-wallet`/`convert`/`derive-child` then `_ => Vec::new()`. restore falls to `_` → `[]`. (FOLLOWUP listed all 6 arms correctly.)
  - `crates/mnemonic-toolkit/src/cmd/restore.rs:60` (`--from required_unless_present="md1"`) — **DRIFTED-by-2.** Now `:58` `#[arg(long, required_unless_present = "md1")]` + `:59` `pub from: Option<String>` (v0.45/0.46 shifted it down 2). Semantics unchanged.
  - **GUI hand-authored rule (`mnemonic-gui/src/form/conditional.rs::restore`)** — **ACCURATE + ALREADY MATCHES the target projection.** Body: `if !state.has_value("--md1") { vis.push(("--from", Visibility::Required)); }`. The toolkit projection that mirrors this is exactly `when: Not(FlagPresent "--md1") → effect: {--from, Required}`.
  - **Projection precedent EXISTS:** `gui_schema.rs:385 bundle_conditional_rules` already emits the identical shape for `--template` (`when: Not(AnyOf{--descriptor, --descriptor-file}) → {--template, Required}`, `VisibilityProjection::Required`). restore's is the simpler single-flag form `Not(FlagPresent "--md1")`. The toolkit test precedent is `tests/cli_gui_schema_conditional_rules.rs:167 bundle_template_required_unless_uses_not_any_of_predicate`.
  - **NOT schema_mirror-gated:** `schema_mirror` is flag-NAME parity only; `conditional_rules` is gated by the GUI's `gui_schema_conditional_drift` (a different test) — confirmed (no toolkit `*mirror*` test references `conditional`).
  - **NO schema-version bump:** `tests/cli_gui_schema_conditional_rules.rs:54` pins `version == 5`; bumps are tied to STRUCTURAL changes (new Flag fields/Visibility variants/predicate kinds). Populating restore's existing `conditional_rules` array with existing `Not`/`FlagPresent`/`Required` grammar is NOT structural → version stays **v5**, no bump, no break of `every_subcommand_has_conditional_rules_array` (`:71`).
- **Action for brainstorm spec:** **Toolkit half (Cycle A):** add `restore_conditional_rules() -> Vec<ConditionalRule>` (one rule, mirror `bundle`'s Required-unless but `Not(FlagPresent "--md1")`) + `"restore" => restore_conditional_rules()` to `build_subcommand_conditional_rules` (`gui_schema.rs:336`); add a toolkit test mirroring `bundle_template_required_unless_uses_not_any_of_predicate` (assert restore rule `when.kind=="not"`, inner `flag_present`/`--md1`, effect `--from`/`required`). ~20 LOC + 1 test. **GUI half (Cycle B, after the toolkit pin bump):** no GUI *logic* change (the `conditional::restore` fn already matches); add `("restore", 1)` to `SUBCOMMAND_FLOORS` (`gui_schema_conditional_drift.rs:300`) so the now-gated rule can't silently vanish. Cite toolkit SHA `0bd98c2`, GUI SHA `f6caa20`.

### `gui-readme-install-pin-coherence-guard`
- **WHAT:** mnemonic-gui `README.md` has no guard asserting its install-command `--tag` pins match `pinned-upstream.toml` (unlike toolkit's `readme_version_current.rs`); they drifted 3 versions before v0.25.0 backfilled them. Add a pure-logic `tests/readme_pin_coherence.rs`.
- **Citations:**
  - `mnemonic-gui/README.md` install block — **ACCURATE + currently COHERENT.** `:42` self-tag `mnemonic-gui-v0.26.0` (== `Cargo.toml` version 0.26.0 ✓); `:50` `mnemonic-toolkit-v0.46.0` (== `pinned-upstream.toml [mnemonic]:22` ✓); `:51` `descriptor-mnemonic-md-cli-v0.6.2`, `:52` `ms-cli-v0.7.0`, `:53` `mk-cli-v0.7.0`.
  - `mnemonic-gui/pinned-upstream.toml` — **ACCURATE: carries ALL FOUR tags** (`[mnemonic]:22`, `[md]:39`, `[ms]:46`, `[mk]:53`) → a full source-of-truth for the 4 sibling install lines; the GUI self-tag's source-of-truth is `Cargo.toml` `version`.
  - **No existing readme guard** — **ACCURATE.** `tests/` has only `pin_coherence.rs` (Cargo↔pinned-upstream `[mnemonic]` tag), `non_canonical_descriptor_account_pin.rs`, `secret_taxonomy_pin.rs`. None asserts README↔pinned-upstream.
  - **Existing precedent to mirror:** GUI `tests/pin_coherence.rs:24` (`cargo_toolkit_pin_matches_pinned_upstream_mnemonic_tag`) + toolkit `tests/readme_version_current.rs`.
- **Action for brainstorm spec:** **GUI half (Cycle B):** add `mnemonic-gui/tests/readme_pin_coherence.rs` (pure-logic, no binary): parse the 4 README `cargo install … --tag <X>` lines, assert each == the matching `pinned-upstream.toml` `[mnemonic|md|ms|mk].tag`, and assert the README self-tag (`mnemonic-gui-vX`) == `Cargo.toml` version. ~40 LOC. Cite GUI SHA `f6caa20`.

---

## Cross-cutting observations
1. **STRUCTURAL: slug 1 is a two-repo, two-RELEASE dependency chain — NOT a single cycle.** The toolkit must project the rule and SHIP (so `mnemonic gui-schema` emits it) BEFORE the GUI can bump its pin and let `gui_schema_conditional_drift` consume it. There is a hard ordering: **toolkit Cycle A → GUI Cycle B.** This mirrors the established `gui-*-pending-pin-bump` pattern (a toolkit gui-schema change can't ship in a GUI cycle).
2. **Slug 2 is GUI-only and independent** — it can ship in the GUI Cycle B alongside slug 1's GUI half (single GUI release), or standalone. No toolkit dependency.
3. **Citations are clean** (one DRIFTED-by-2, no structural errors) — both FOLLOWUPs are accurate. The GUI's `conditional::restore` already matches the target shape, so slug 1's GUI half is near-zero-logic (just FLOORS + pin bump).
4. **No schema_mirror / manual / sibling-codec lockstep** for either slug. Slug 1 toolkit change touches `gui-schema` JSON wire-shape only (conditional_rules), gated by the GUI drift test, not schema_mirror. Slug 2 is a test-only add.
5. **Incidental:** the GUI pins toolkit **v0.46.0** (its Cargo + pinned-upstream), one PATCH behind the just-shipped **v0.46.1**. Cycle B's pin bump should target the toolkit Cycle-A release (which will be ≥ v0.46.2), naturally catching up past v0.46.1 too.

---

## Recommended brainstorm-session scope

**Two sequential cycles (hard ordering A→B):**

**Cycle A — toolkit `gui-schema` restore conditional_rules projection.** Add `restore_conditional_rules()` + the `build_subcommand_conditional_rules` arm + a toolkit predicate-shape test. **SemVer: PATCH** (additive `gui-schema` JSON projection; no clap flag/value/subcommand change, no schema-version bump). Toolkit **v0.46.1 → v0.46.2**. Size: ~20 LOC + 1 test. Locksteps: **NONE** toolkit-side (no schema_mirror; the GUI consumption is the downstream Cycle B). Ship tag `mnemonic-toolkit-v0.46.2`.

**Cycle B — GUI consume + README pin guard.** (1) Bump GUI toolkit pin → v0.46.2 (Cargo + pinned-upstream + lock, `pin_coherence`); the existing `conditional::restore` fn now drift-gated by `gui_schema_conditional_drift` — add `("restore", 1)` to `SUBCOMMAND_FLOORS`. (2) Add `tests/readme_pin_coherence.rs` (slug 2). (3) README install pins → v0.46.2 / mnemonic-gui-v0.27.0. **SemVer: MINOR** (consumes a new toolkit projection + new guard; mirrors the v0.25.0/v0.26.0 catch-up precedent). GUI **v0.26.0 → v0.27.0**. Ship tag `mnemonic-gui-v0.27.0`. Then flip BOTH toolkit FOLLOWUPs → resolved.

**Inter-slug dependency:** Cycle A is the prerequisite for slug 1's GUI half; slug 2 is independent but bundles into Cycle B for one GUI release. Recommend executing **A first (small, unambiguous), then B**. Each cycle gets its own mandatory R0 gate.
