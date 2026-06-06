# R0 Architect Review (round 1) — `SPEC_gui_schema_restore_conditional_projection.md`

**Reviewer:** opus `feature-dev:code-reviewer` (mandatory pre-implementation R0 gate). **Date:** 2026-06-05.
**Branch:** `gui-schema-restore-conditional-projection` (off master `0bd98c2`, toolkit v0.46.1). **Verdict:** **0 Critical / 1 Important** (+ 1 Minor).

> Persisted verbatim per CLAUDE.md BEFORE the fold step. I1 (missing arm-count-guard bump) is a real Phase-2 gate failure; fold → re-dispatch per the after-every-fold loop.

---

## VERDICT: 0 Critical / 1 Important / 1 Minor

NOT GREEN. One Important must be folded before implementation; re-dispatch the architect per the R0 loop after folding.

---

## Important

### I1 — SPEC omits the `dispatcher_arm_count_matches_pinned_constant` update; following the SPEC verbatim produces a RED suite at its own Phase-2 GREEN gate

**File:** `crates/mnemonic-toolkit/tests/cli_gui_schema_conditional_rules.rs:531-545` (the test), `:533` (the constant to bump).

`dispatcher_arm_count_matches_pinned_constant` pins `const EXPECTED_ARM_COUNT: usize = 6` and counts dispatcher arms via the regex `^\s+"[a-z-]+" => [a-z_]+_conditional_rules\(\),$`. The new arm the SPEC adds —

```rust
"restore" => restore_conditional_rules(),
```

— matches that regex exactly, taking the count `6 → 7`. The test then asserts `actual == 6` and fails.

This directly falsifies two SPEC claims:
- **§4:** "`every_subcommand_has_conditional_rules_array` (`:71`) still holds … restore's array goes `[] → [1 rule]`" — true, but the SPEC stops there and misses this *separate* arm-count guard.
- **§5 "Green-stays-green"** and **§7 Phase 2 "Workspace test + clippy GREEN":** both are false as written. The SPEC's "restore was the only `_`-arm subcommand being touched" reasoning is correct about the `_ => Vec::new()` fall-through but blind to the regex-based arm-count pin, which is the very test designed to fire on a new dispatcher arm (its docstring at `:525-530`: "Concurrent feature PRs that add a new subcommand must consciously bump this constant").

**Fix:** Phase 2 must bump `EXPECTED_ARM_COUNT` `6 → 7` at `cli_gui_schema_conditional_rules.rs:533` in the same commit that adds the dispatcher arm (co-located edit, same file as the new RED cell). Update §4/§5/§7 of the SPEC to (a) list this constant bump as a required Phase-2 edit and (b) drop the unqualified "green-stays-green / only the new cell needs adding" language. This is the test the gate is built to catch; it is load-bearing, not a nitpick.

---

## Minor

### M1 — Empirical jq check (item 3) could not be executed in this review environment

The kickoff asked me to run `cargo run -q -p mnemonic-toolkit -- gui-schema | jq` to confirm the wire shape empirically. No Bash tool was available in this session, so I discharged item 3 from source + an existing running-binary test instead (see "What verified clean"). The shape is fully pinned and an analogous live-binary assertion already exists, so this does not change the verdict — but the implementer should run the Phase-1 RED cell against the freshly-built binary to confirm RED-for-the-right-reason (restore `conditional_rules == []`) before writing §2, per `feedback_architect_must_run_prose_commands`.

---

## What verified clean

**Citations (item 1) — all accurate (recon's drift-by-2 confirmed):**
- `cmd/restore.rs:58` `#[arg(long, required_unless_present = "md1")]` + `:59` `pub from: Option<String>`. SPEC cites `:58` correctly (the recon's earlier "`:60`" was the stale FOLLOWUP snapshot).
- `cmd/gui_schema.rs:336` `fn build_subcommand_conditional_rules(name: &str)`; arms `:338-343` (`bundle`/`verify-bundle`/`export-wallet`/`convert`/`derive-child`/`compare-cost`), `_ => Vec::new()` at `:344`. No `restore` arm — restore falls through to `[]`. Confirmed.
- `bundle_conditional_rules` precedent at `:385-602`; the `--template` Required-unless rule at `:391-413` uses `Predicate::Not { predicate: Box::new(Predicate::AnyOf{…}) }` → `VisibilityProjection::Required`. Exactly the shape restore mirrors (single-flag `FlagPresent` instead of `AnyOf`).

**The literal compiles (item 2) — all types/fields match:**
- `ConditionalRule { rationale: String, spec_ref: String, when: Predicate, effect: Effect }` (`:96-102`).
- `Predicate::Not { predicate: Box<Predicate> }` — field named `predicate`, type `Box<Predicate>` (`:137-139`). `Box::new(...)` correct.
- `Predicate::FlagPresent { flag: String }` (`:113-115`). `"--md1".to_string()` correct.
- `Effect { flag: String, visibility: VisibilityProjection }` (`:155-159`).
- `VisibilityProjection::Required` exists (`:183`). `"--from".to_string()` correct.

**JSON serialization shape (item 3) — pinned by serde attrs + an existing live-binary test:**
- `Predicate` has `#[serde(tag = "kind", rename_all = "snake_case")]` (`:111`) → `FlagPresent` → `"flag_present"`, `Not` → `"not"`; `Not`'s inner field serializes as `"predicate"` (`:138`).
- `VisibilityProjection::Required => ser.serialize_str("required")` (custom `Serialize`, `:203`) → bare `"required"`.
- The exact wire shape is already asserted against the **running binary** for bundle's `--template` rule at `cli_gui_schema_conditional_rules.rs:178-179` (`when["kind"]=="not"`, `when["predicate"]["kind"]=="any_of"`) + `:174-175` (`effect["visibility"]=="required"`), and the generic `not → predicate` recursion at `:430`. Restore's rule serializes to `{"kind":"not","predicate":{"kind":"flag_present","flag":"--md1"}}` for `when` and `{"flag":"--from","visibility":"required"}` for `effect`.

**No existing test pins restore's `conditional_rules` as empty (item 4 — partially confirmed, see I1):**
- No `cli_restore.rs` test touches `conditional_rules` (all functional).
- No test asserts restore's `conditional_rules.is_empty()` / `== []` / a subcommand-count excluding restore.
- `every_subcommand_has_conditional_rules_array` (`:71-81`) only requires `.is_array()` — `[] → [1 rule]` stays an array. GREEN.
- `schema_version_pinned_at_current_cycle` (`:54-66`) asserts `version == 5`; no structural grammar change (reuses existing `Not`/`FlagPresent`/`Required`) → stays v5. GREEN.
- **EXCEPTION:** `dispatcher_arm_count_matches_pinned_constant` DOES break (I1).
- No aggregate rule-count / subcommand-count assertion in `cli_gui_schema.rs`, `_v3_`, `_v4_`, or `_v5_extensions.rs`.

**GUI consumption contract is satisfiable, not a downstream trap (item 5):**
- (a) `mnemonic-gui/src/form/conditional.rs:935-941` `restore()`: `if !state.has_value("--md1") { vis.push(("--from", Visibility::Required)); }` — exactly `Not(FlagPresent "--md1") → {--from, Required}`. Matches the projection.
- (b) `gui_schema_conditional_drift.rs::synthesize_satisfying` handles `Predicate::Not` (`:144-151`) by returning the empty `base` unchanged; the caller passes `FormState::default()` (`:259`), so `--md1` is absent → `restore()` pushes `("--from", Required)` → matches. This is the **same empty-base mechanism bundle's `Not(AnyOf{…})` already rides through the gate** — proven-working, not a new path.
- (c) The GUI `restore()` fn touches ONLY `--from` — no other flag the projection omits, so no extra-divergence. And `mnemonic-gui/src/schema/mnemonic.rs:3464` already wires `conditional: Some(crate::form::conditional::restore)`, so the drift gate's `handcoded.conditional` Some-check (`:236`) passes and restore is inserted into `per_subcommand_rules`; the downstream `SUBCOMMAND_FLOORS += ("restore", 1)` then evaluates `unwrap_or(0) >= 1` → holds. The recon's "GUI half = FLOORS + pin bump only" is correct.

**SemVer + lockstep (item 6):**
- PATCH (v0.46.1 → v0.46.2) correct: additive `gui-schema` JSON projection, no clap flag/value/subcommand change, no schema-version bump.
- NOT `schema_mirror`-gated: that gate is flag-NAME parity only; `conditional_rules` is a separate wire-shape gated by the GUI's `gui_schema_conditional_drift` (a different test, lives downstream). No toolkit `*mirror*` test references `conditional`.
- No new `ToolkitError` variant, no manual mirror, no sibling-codec change.
- No toolkit-side consumer of `gui-schema`'s `conditional_rules` (the only consumer is the GUI drift gate, downstream).

**Phasing + RED cell (item 7):** Phase-1 cell asserting exactly 1 rule with `when.kind=="not"` / inner `flag_present` `--md1` / `effect.flag=="--from"` / `effect.visibility=="required"` is well-formed and will be RED against the current binary (restore emits `[]`, so the `find`/index will fail) and GREEN after §2. (Sound, modulo running it against a freshly-built binary per M1.)

---

**Action:** fold I1 (add the `EXPECTED_ARM_COUNT 6→7` bump to Phase 2; correct the "green-stays-green / only the new cell" claims in §4/§5/§7) → persist this review → re-dispatch the architect.
