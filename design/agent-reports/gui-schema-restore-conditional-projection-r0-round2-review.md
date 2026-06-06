# R0 Architect Review (round 2) — `SPEC_gui_schema_restore_conditional_projection.md`

**Reviewer:** opus `feature-dev:code-reviewer` (mandatory pre-implementation R0 gate). **Date:** 2026-06-05.
**Branch:** `gui-schema-restore-conditional-projection`. **Verdict:** **0 Critical / 0 Important** (+ 1 Minor new, 1 carried). **GREEN — implementation may proceed.**

> Persisted verbatim per CLAUDE.md. I1 fold verified correct + complete; no new drift; no test other than the arm-count guard trips on the new arm. M1 (snippet adjacency) folded without re-dispatch (SPEC-only one-liner).

---

## VERDICT: 0 Critical / 0 Important (+ 1 Minor)

**GREEN — implementation may proceed.**

The Round-1 Important (I1) fold is correct and complete; it introduced no new drift; the core premise still holds under a fresh adversarial pass; and no test other than the one I1 names trips on the new dispatcher arm.

---

## I1 fold — verified correct + complete

The SPEC now unambiguously requires the `EXPECTED_ARM_COUNT 6 → 7` bump in Phase 2, in three mutually-consistent places (§4, §5, §7 Phase 2).

**6 → 7 is exactly right, confirmed against source:**
- The test pins `const EXPECTED_ARM_COUNT: usize = 6` at `:533`; regex `^\s+"[a-z-]+" => [a-z_]+_conditional_rules\(\),$` (`:536`).
- Current dispatcher arms `gui_schema.rs:338-343` — `bundle`, `verify-bundle`, `export-wallet`, `convert`, `derive-child`, `compare-cost` = **exactly 6**. (`_ => Vec::new()` at `:344` is unquoted → not matched; `fn` defs end in `{` → not matched.)
- `"restore" => restore_conditional_rules(),` matches the regex exactly. Adds exactly 1 → **6 → 7**.

The SPEC's `:533` citation matches current source byte-for-byte.

## No new drift from the fold

§2/§4/§5/§7 agree; no residual unqualified "green-stays-green." §4's "no schema-version bump (stays v5)" consistent with `schema_version_pinned_at_current_cycle` + the v5-extensions pin. §4/§5's "`every_subcommand_has_conditional_rules_array` still holds" correct (`:71-81` requires only `.is_array()`).

## Core premise re-confirmed (fresh adversarial pass)

- **Literal compiles:** `ConditionalRule` (`:96-102`), `Predicate::Not{ predicate: Box<Predicate> }` (`:137-139`), `Predicate::FlagPresent{ flag }` (`:113-115`), `Effect` (`:155-159`), `VisibilityProjection::Required` (`:183`).
- **Wire shape:** `Not` → `"not"`, inner field `"predicate"`; `FlagPresent` → `"flag_present"`; `Required → "required"`. Restore `when` = `{"kind":"not","predicate":{"kind":"flag_present","flag":"--md1"}}`, `effect` = `{"flag":"--from","visibility":"required"}`.
- **SemVer:** PATCH; no `schema_mirror` (flag-NAME gate only); no schema-version bump.
- **GUI consumption satisfiable:** `mnemonic-gui` `conditional::restore` = `Not(FlagPresent "--md1") → {--from, Required}` matches.

## Item 4 — nothing else trips (the Round-2 value-add)

The new restore rule passes every all-subcommand rule validator:
- `predicate_kinds_emitted_in_snake_case` (`:404-433`): `not` + `flag_present` ∈ `allowed_kinds`.
- `effect_visibilities_are_in_allowed_set` (`:449-500`): bare `"required"` ∈ `bare_allowed`.
- `every_rule_has_rationale_and_spec_ref` (`:505-521`): restore's rule supplies both.

**No aggregate/total/snapshot test exists.** Per-subcommand counts (`convert==4` `:333`, `compare-cost==2` `:347`, `bundle==11` v4_extensions `:139`) don't pin restore or a cross-subcommand total. No `insta`/`assert_snapshot`/`.snap` golden of the full conditional_rules JSON.

**`cli_gui_schema_v5_extensions.rs`: clean.** Its all-subcommand iterators (`secret_flag_enumeration` `:293`, `global_local_id_disjointness` `:331`) + `no_auto_repair_appears_…_in_every_subcommand` (`:166-200`, hard-coded 12-name list excluding restore) iterate/assert over FLAGS — restore adds no flags. Unaffected. `cli_gui_schema.rs` iterators (`:310`, `:343`) likewise flags-not-rules; `choices.len()==10/11` are dropdown counts (restore adds no dropdown values).

The only test that fires on the new arm is `dispatcher_arm_count_matches_pinned_constant` — exactly the one I1 names.

---

## Minor (non-blocking)

### M1 (Round 2) — §2 registration snippet omits the `compare-cost` arm (source-narrative drift)
SPEC §2 lines 44-48 show `"derive-child" =>` directly above `_ => Vec::new()`, but the real dispatcher has `"compare-cost" => compare_cost_conditional_rules(),` between them (`gui_schema.rs:343`). Match-arm order is free, so functionally irrelevant — but an implementer doing a literal context-anchored insert should append `"restore" =>` after `"compare-cost"` (`:343`), not after `"derive-child"`. Fix: update §2's snippet context to show `"compare-cost" =>` as the last arm before `_`.

### M2 (carried from Round-1 M1) — empirical jq not run in this environment
The restore-emits-`[]` claim is fully dischargeable from source (no `restore` arm → falls through to `_ => Vec::new()` → `[]`). **Implementer action:** run the Phase-1 RED cell against a freshly-built binary to confirm RED-for-the-right-reason before writing §2.

---

**0 Critical / 0 Important — GREEN, implementation may proceed.** Fold M1 (one-line §2 snippet fix) opportunistically; SPEC-only one-line correction does not require a re-dispatch round.
