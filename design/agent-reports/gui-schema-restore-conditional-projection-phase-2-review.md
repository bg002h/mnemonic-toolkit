# Phase 2 (GREEN) Code Review — `gui-schema-restore-conditional-projection` (Cycle A)

**Reviewer:** opus `feature-dev:code-reviewer` (mandatory per-phase review). **Date:** 2026-06-05.
**Branch:** `gui-schema-restore-conditional-projection`. **Verdict:** **0 Critical / 0 Important** (+ 1 Minor). **GREEN — Phase 2 may proceed to Phase 3 (release).**

> Persisted verbatim per CLAUDE.md. Reviewer had no Bash; verified statically from both repos. The runtime checks it deferred were operator-run GREEN: `gui-schema|jq` matched the predicted shape exactly; full suite 0 failures; clippy `--all-targets` exit 0; `git --stat` = 2 files. M1 (rationale wording) folded.

---

## VERDICT: 0 Critical / 0 Important (+ 1 Minor)

**GREEN — Phase 2 may proceed to Phase 3 (release).**

Method note: no Bash was exposed to the reviewer; it verified every item statically from source in both repos (serde-derive behavior is mechanically determined, so the wire-shape conclusion is conclusive). The operator confirmed the four runtime commands GREEN (jq shape, suite, clippy, git --stat).

### Item 1 — emitted wire-shape (serde-derive analysis; operator-confirmed via jq)
Builder constructs one `ConditionalRule`; serde traces to:
```json
[{ "rationale": "...", "spec_ref": "cmd/restore.rs clap-derive required_unless_present = \"md1\"",
   "when": { "kind": "not", "predicate": { "kind": "flag_present", "flag": "--md1" } },
   "effect": { "flag": "--from", "visibility": "required" } }]
```
No deviation from SPEC §2. (Operator `gui-schema|jq` output matched byte-for-byte.)

### Item 2 — GUI consumption contract — MATCH (highest-value check)
`mnemonic-gui/src/form/conditional.rs:935-941` `restore()`: `if !state.has_value("--md1") { vis.push(("--from", Visibility::Required)); }` = `--from Required ⟺ --md1 absent` = exactly the projected `Not(FlagPresent --md1) → {--from, Required}`. The drift gate's `Not` synthesizer (`gui_schema_conditional_drift.rs:144-151`) returns the empty base → `restore(default)` pushes `(--from, Required)` → matches (same mechanism bundle's `Not(AnyOf{…})` already rides). GUI fn touches ONLY `--from` — no omitted flag. `restore` correctly ABSENT from `SUBCOMMAND_FLOORS` (downstream GUI cycle adds `("restore", 1)` per SPEC §6). One honest limitation (out of scope, identical to bundle): the `Not` synthesizer only tests the satisfied direction, so the gate is one-directional.

### Item 3 — no regression (static; operator-confirmed suite GREEN)
Version stays `5` (`build_schema` hardcodes `version: 5`; pin test unchanged — no structural grammar added). `every_subcommand_has_conditional_rules_array` holds (`[] → [1 rule]`). No test pins restore's `conditional_rules` empty (whole `tests/` grepped). Operator: `cargo test --no-fail-fast` → every `test result:` 0 failed.

### Item 4 — arm-count bump — CORRECT, no off-by-one
Dispatcher has exactly 7 quoted `_conditional_rules()` arms (bundle/verify-bundle/export-wallet/convert/derive-child/compare-cost/restore); `EXPECTED_ARM_COUNT == 7`; regex matches all 7 and not `_ => Vec::new()`.

### Item 5 — code quality
`restore_conditional_rules()` consistent with the six sibling builders (shape, doc-comment citing the FOLLOWUP, single-flag `Not(FlagPresent)` mirroring bundle). `--md1` correct (`restore.rs:66-67` `#[arg(long)] pub md1: Vec<String>`); `--from required_unless_present="md1"` at `restore.rs:58`. Operator: clippy `--all-targets` exit 0.

### Item 6 — scope
`restore.rs` unmodified (only cited). Changes scoped to `gui_schema.rs` (builder + arm) + `cli_gui_schema_conditional_rules.rs` (`EXPECTED_ARM_COUNT` + Phase-1 cell). Operator `git --stat`: 2 files.

---

## Minor (folded)

### M1 — rationale wording: `--from` is a SEED source, not a "wallet-export source".
`gui_schema.rs` (builder) called `--from` a *"wallet-export source"*, but `restore.rs:53` documents it as *"Seed source: ms1/phrase/entropy/seedqr"* and `restore.rs:192` refuses non-seed nodes ("not a seed source for restore"). Copied verbatim from the R0-GREEN SPEC (§2), so not a Phase-2 regression; affects no test (drift gate reads `when`/`effect` + a non-empty `rationale` check) and no wire assertion. **FOLDED:** rationale changed to "single-sig restore needs a seed source (ms1/phrase/entropy/seedqr), while multisig-cosigner restore (--md1) supplies the policy and makes --from optional" in BOTH `gui_schema.rs` and the SPEC §2. Re-ran the conditional_rules suite (19/19) + re-confirmed via jq. Non-blocking, folded without re-dispatch (one-line accuracy fix, no shape/test/count touched).

---

**0 Critical / 0 Important — GREEN, Phase 2 may proceed to Phase 3 (release).**
