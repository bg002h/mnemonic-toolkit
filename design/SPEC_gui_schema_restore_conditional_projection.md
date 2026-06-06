# SPEC — toolkit `gui-schema`: project restore's `--from required_unless_present="md1"` conditional rule

**Status:** R0 gate (pre-implementation). MUST converge to 0 Critical / 0 Important before any code.
**Resolves (toolkit half):** FOLLOWUP `gui-schema-restore-required-unless-md1-projection` (the GUI consumption half is a follow-on GUI cycle).
**Source SHA:** branch `gui-schema-restore-conditional-projection` off master `0bd98c2` (toolkit v0.46.1).
**SemVer:** PATCH — additive `gui-schema` JSON projection (new entry in restore's existing `conditional_rules` array). No clap flag/value/subcommand change, no schema-version bump. v0.46.1 → **v0.46.2**.

---

## 1. Summary

Toolkit `restore` carries `#[arg(long, required_unless_present = "md1")]` on `--from` (`cmd/restore.rs:58`), but `gui-schema`'s `conditional_rules` projection has no `restore` arm (`gui_schema.rs:336` `build_subcommand_conditional_rules` — allowlist for `compare-cost`/`bundle`/`verify-bundle`/`export-wallet`/`convert`/`derive-child`, else `_ => Vec::new()`). So restore emits `conditional_rules: []`, and the GUI's at-least-one rule (`--from` Required unless `--md1`) is **hand-authored and ungated** (GUI `conditional::restore`, modeled in mnemonic-gui v0.25.0, same posture as repair/inspect).

This cycle adds the `restore` arm so the rule is **projected** — letting the GUI's `gui_schema_conditional_drift` enforce GUI↔toolkit parity once the GUI bumps its pin (the downstream GUI cycle). **Toolkit-repo-only.**

## 2. The change — `cmd/gui_schema.rs`

Add a `restore_conditional_rules()` builder mirroring `bundle_conditional_rules`'s `--template` Required-unless precedent (`gui_schema.rs:385-413`), but with the **single-flag** negative predicate `Not(FlagPresent "--md1")` (vs bundle's `Not(AnyOf{…})`):

```rust
fn restore_conditional_rules() -> Vec<ConditionalRule> {
    vec![ConditionalRule {
        rationale: "--from is required unless --md1 is supplied: single-sig \
                    restore needs a wallet-export source, while \
                    multisig-cosigner restore (--md1) supplies the policy and \
                    needs no --from."
            .to_string(),
        spec_ref: "cmd/restore.rs clap-derive required_unless_present = \"md1\"".to_string(),
        when: Predicate::Not {
            predicate: Box::new(Predicate::FlagPresent {
                flag: "--md1".to_string(),
            }),
        },
        effect: Effect {
            flag: "--from".to_string(),
            visibility: VisibilityProjection::Required,
        },
    }]
}
```

and register it in `build_subcommand_conditional_rules` (`:337-345`). **(R0 M1)** the real last arm before `_` is `"compare-cost"` (`:343`), NOT `"derive-child"` — append `"restore"` after it:

```rust
        "derive-child" => derive_child_conditional_rules(),
        "compare-cost" => compare_cost_conditional_rules(),
        "restore" => restore_conditional_rules(),   // NEW
        _ => Vec::new(),
```

Placement: alongside the other builders (e.g. after `restore.rs` analog `derive_child_conditional_rules` def). Match-arm order is free; insert the `"restore" =>` arm anywhere before `_ => Vec::new()`. No other emitter change.

## 3. Why this shape matches the GUI (the consumption contract)

The GUI's hand-authored rule (`mnemonic-gui/src/form/conditional.rs::restore`) is:
```rust
if !state.has_value("--md1") { vis.push(("--from", Visibility::Required)); }
```
i.e. `--from` Required ⟺ `--md1` absent — **exactly** `Not(FlagPresent "--md1") → {--from, Required}`. So the projection emitted here will match the GUI fn under `gui_schema_conditional_drift`'s synthesize-and-compare (the `Not` predicate is already exercised by bundle's drift-gated `--template` rule, so the synthesizer handles it). No GUI logic change is needed downstream — only a pin bump + a `SUBCOMMAND_FLOORS` entry (tracked for the GUI cycle).

## 4. Not gated / no version bump (verified at recon)
- **NOT `schema_mirror`:** that gate is flag-NAME parity only; `conditional_rules` is a separate wire-shape gated by the GUI's `gui_schema_conditional_drift`. No toolkit schema_mirror change.
- **NO schema-version bump:** `tests/cli_gui_schema_conditional_rules.rs:54` pins `version == 5`; version bumps track STRUCTURAL changes (new Flag fields / Visibility variants / predicate kinds). Populating restore's existing `conditional_rules` array with existing `Not`/`FlagPresent`/`Required` grammar is not structural → stays **v5**. `every_subcommand_has_conditional_rules_array` (`:71`) still holds (restore's array goes `[] → [1 rule]`, still an array).
- **(R0 I1) BUT the dispatcher arm-count guard DOES move.** `tests/cli_gui_schema_conditional_rules.rs:532` `dispatcher_arm_count_matches_pinned_constant` pins `const EXPECTED_ARM_COUNT: usize = 6` and counts `"<sub>" => <fn>_conditional_rules(),` arms by regex. The new `"restore" =>` arm takes the count `6 → 7`, so **Phase 2 MUST bump `EXPECTED_ARM_COUNT` `6 → 7` at `:533` in the same commit** (this guard exists precisely to force a conscious bump on a new dispatcher arm). This is NOT a "green-stays-green" change.
- **No new error variant, no manual mirror, no sibling-codec change.**

## 5. Tests
- **Phase-1 RED:** add a toolkit cell to `tests/cli_gui_schema_conditional_rules.rs` mirroring `bundle_template_required_unless_uses_not_any_of_predicate` (`:167`): run `gui-schema`, pull restore's `conditional_rules`, assert **exactly 1 rule** with `when.kind=="not"`, inner `when.predicate.kind=="flag_present"` + `flag=="--md1"`, and `effect.flag=="--from"` + `effect.visibility=="required"`. RED against the current binary (restore emits `[]` → `conditional_rules` empty → assertion fails), GREEN after §2.
- **Mostly green-stays-green, with ONE required guard bump (R0 I1):** `every_subcommand_has_conditional_rules_array`, `schema_version_pinned_at_current_cycle` (still v5), and all existing per-subcommand rule cells are unaffected. The ONE test that MUST be updated is `dispatcher_arm_count_matches_pinned_constant` (`:532`) — bump `EXPECTED_ARM_COUNT` `6 → 7` (`:533`) in the Phase-2 commit (a new dispatcher arm by design trips this guard).
- Full workspace `cargo test --no-fail-fast` + clippy `--all-targets` GREEN.

## 6. Lockstep / scope
- **Toolkit-repo-only this cycle.** The GUI consumption (pin bump → drift-gate enforcement + `SUBCOMMAND_FLOORS += ("restore", 1)`) is the **downstream GUI cycle** (FOLLOWUP stays open until the GUI tag exists; this cycle's CHANGELOG notes the toolkit half shipped). No GUI change in THIS PR (the GUI can't consume a projection from an unreleased toolkit).
- The FOLLOWUP `gui-schema-restore-required-unless-md1-projection` is **cross-repo**: flip to `resolved` only after BOTH the toolkit projection (this cycle) AND the GUI consumption ship. This cycle annotates it "toolkit half shipped v0.46.2; GUI consumption pending pin bump".

## 7. Phased plan
- **Phase 1 (RED):** the restore-projection predicate-shape cell (asserts 1 rule, `not`/`flag_present`/`--md1` → `--from`/`required`). Verify RED-for-the-right-reason (current binary: restore `conditional_rules` empty).
- **Phase 2 (GREEN):** §2 `restore_conditional_rules()` + the `build_subcommand_conditional_rules` arm **+ bump `EXPECTED_ARM_COUNT` `6 → 7` at `cli_gui_schema_conditional_rules.rs:533` (R0 I1, same commit)**. Workspace test + clippy GREEN. Per-phase opus review → persist.
- **Phase 3 (release):** CHANGELOG `[0.46.2]`; version v0.46.1 → **v0.46.2** (Cargo.toml/lock + 2 READMEs + install.sh self-pin); annotate FOLLOWUP (toolkit half shipped, GUI half pending). Per-phase review.
- **Phase 4 (ship):** clean tree → ff-merge → tag `mnemonic-toolkit-v0.46.2` → push → watch CI (rust, install/sibling-pin-check).

## 8. Risk
Very low. One additive projection entry mirroring a tested precedent (`bundle`'s Required-unless), no clap surface change, no schema-version bump, no new variant. R0 must confirm: (i) the `Not(FlagPresent)` predicate serializes to the `{"kind":"not","predicate":{"kind":"flag_present","flag":"--md1"}}` JSON the GUI's `parse_gui_schema_conditional_rules` + synthesizer expect; (ii) no existing toolkit test pins restore's `conditional_rules` as empty (would need updating, not just the new cell); (iii) the GUI `conditional::restore` shape genuinely matches (so the future drift gate is satisfiable, not a trap for the GUI cycle).
