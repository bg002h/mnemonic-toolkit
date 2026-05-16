# IMPLEMENTATION_PLAN — GUI conditional-applicability lockstep cycle

**Cycle tag plan:** `mnemonic-toolkit-v0.16.0` + `mnemonic-gui-v0.5.0` (lockstep, both minor bumps)

**Scope class:** cross-repo feature, mechanism + comprehensive rule coverage (Option C per the brainstorming exchange — full gui-schema JSON projection of CLI §6.6/§6.9 mutex/conditional rules, drift-gated).

**2026-05-16 tag-plan revision (recorded post-P0-discovery, pre-P1):** The original plan targeted `mnemonic-gui-v0.4.0` as the lockstep GUI tag, but v0.4.0/v0.4.1/v0.4.2 had already shipped on 2026-05-16 for unrelated Windows-build + toolkit-dep patches. Architect-review (opus, `feature-dev:code-reviewer`) recommended a **two-step bump**: cut `mnemonic-gui v0.4.3` (toolkit v0.14.2 → v0.15.0 catchup, scope-isolated) BEFORE this cycle's P1 advances. This cycle then resumes against a v0.15.0-clean GUI baseline and ships `v0.5.0` consuming toolkit `v0.16.0`. Rationale: toolkit v0.15.0 was a "wire-format clean break" (per commit `5d92768` release notes); folding it into this cycle's reviewer-loop would couple two independent risk surfaces (§5.1 reproduction failure would be unable to disambiguate v0.15.0 wire-format regression from v0.16.0 conditional-applicability bug). See §4 **Prerequisite** gate added below.

---

## 1. Context

### 1.1 Motivating bug

GUI bundle form, default state (template = `bip84`, single-sig), emits:

```
mnemonic bundle --network mainnet --template bip84 --language english --account 0 \
  --multisig-path-family bip48 --self-check --threshold 1 \
  --slot '@0.phrase=beef beef beef beef beef beef beef beef beef beef beef beef'
```

CLI rejects with SPEC §6.6-pinned byte-exact error `--threshold is meaningful only with a multisig --template; single-sig templates ignore threshold.` (`crates/mnemonic-toolkit/src/cmd/bundle.rs:120, 207-220`).

Why `--threshold 1` materializes despite CLI declaring `pub threshold: Option<u8>` (`crates/mnemonic-toolkit/src/cmd/bundle.rs:91`, no default): the GUI's `default_flag_value_for` (`mnemonic-gui/src/form/widget.rs:101-126`) seeds `FlagValue::Number(*min)` from `FlagKind::Number { min: 1, max: 16 }` (`mnemonic-gui/src/schema/mnemonic.rs:227-232`) at first widget render. Once seeded, the Number variant has no "unset" sentinel (`schema/mod.rs:263-268` — `Number` is always-present in `flag_value_is_present`), so the value persists and emits regardless of whether the user ever interacted with the widget. The GUI default-state seed at `main.rs:197-211` is ALSO seeding `--multisig-path-family = bip87` before any user interaction.

### 1.2 Root cause — three stacked defects

1. **GUI default form-state pre-seeds a multisig-only flag for a single-sig template** — `mnemonic-gui/src/main.rs:197-211` hardcodes `--multisig-path-family = bip87` regardless of selected template.
2. **`assemble_argv` does not honor the existing Visibility infrastructure** — `mnemonic-gui/src/form/invocation.rs:42-113` emits every flag whose value is present, ignoring whether the form rendered it Hidden/Disabled. Comparison: Text/Dropdown/Path arms gate on `!v.is_empty()`; `Number`/`Range`/`Timestamp`/`TaggedOrIndexed` have no empty sentinel and emit unconditionally.
3. **No CLI-rule data interchange between toolkit and GUI** — the existing `mnemonic gui-schema` JSON describes flags in isolation (name, kind, choices). It does NOT project the CLI's §6.6/§6.9 mutex/conditional rule manifest. The GUI's hand-coded `conditional.rs` is the only source of truth for which rules the GUI honors, and it has drifted behind the CLI rule surface.

### 1.3 What's already in place (do NOT rebuild)

Per Phase 1 source-grounded exploration:

| Component | File:line | Status |
|---|---|---|
| `Visibility` enum (Visible/Hidden/Required/Disabled) | `mnemonic-gui/src/schema/mod.rs:111-122` | ✅ present |
| `SubcommandSchema.conditional: Option<fn(&FormState) -> FlagVisibility>` | `mnemonic-gui/src/schema/mod.rs:47` | ✅ wired per subcommand |
| `FormState::has_value` / `dropdown_value` / `composite_node` / `has_positional` accessors | `mnemonic-gui/src/schema/mod.rs:195-256` | ✅ all predicates we need exist |
| Per-frame visibility computation + render-layer Hide/Disable | `mnemonic-gui/src/main.rs:386-424` | ✅ wired |
| Mutex-pair conditional fns (`--passphrase`↔`--passphrase-stdin`, `--descriptor`↔`--descriptor-file`, `--bundle-json` XOR card triplet, value-inspect on `--context`, composite-node-inspect on `--from`) | `mnemonic-gui/src/form/conditional.rs:21-365` | ✅ encoded for 14 of the 41 CLI rules |
| `tests/conditional_visibility.rs` — per-cell test for every active constraint | (per Phase 1 GUI agent report) | ✅ established test pattern |
| `mnemonic gui-schema` CLI emitter | `crates/mnemonic-toolkit/src/cmd/gui_schema.rs` | ✅ present, emits `version`/`cli`/`subcommands[]/flags[]`, version-pinned |
| `tests/schema_mirror.rs` — flag-name set parity drift gate | `mnemonic-gui/tests/schema_mirror.rs:51-195` | ✅ pattern template for the new drift gate |

### 1.4 What's missing (the v1 deliverable)

| Gap | Phase | LOC est. |
|---|---|---|
| CLI rule manifest formalized in SPEC + machine-readable in gui-schema JSON | P0 + P1 | SPEC ~80 + toolkit ~150 + tests ~200 |
| GUI `conditional.rs` entries for the ~15 template/descriptor-mode rules currently unencoded | P2 | ~80 + ~300 tests |
| `assemble_argv` visibility gate (Hidden flags suppressed from emission) | P3 | ~30 + ~100 tests |
| Drift gate: GUI test consumes toolkit gui-schema JSON `conditional_rules`, asserts byte-exact parity with `conditional.rs` outputs across all rule cells | P4 | ~200 |
| Default form-state cleanup: remove `--multisig-path-family bip87` seed at `main.rs:203` (it only belongs in a multisig-defaults template) | P5 | ~5 |

**Out of v1 scope (FOLLOWUP class):**
- Slot-count-dependent runtime rules (`pre_check_threshold` T-in-range, `pre_check_template_n` N=1-vs-multisig) — these depend on a dynamic slot count not knowable until the form is filled, and surface naturally at Run time. Their conditional shape (form-prefilled threshold-vs-N comparator) is a follow-on improvement; v1 ships argv-level submission and lets the CLI emit the typed error.
- BIP-388 distinct-key check (cross-slot xpub/path equality) — same posture: surfaces at Run time.
- `Number`-widget "unset" sentinel (no value at all). Out of v1 because the visibility-gate makes it unnecessary: a Disabled-by-template `--threshold` widget won't emit even if it still carries a value internally.

---

## 2. SPEC patches (toolkit)

### 2.1 §6.6 mode-violation ladder — add `gui_projection` column

`design/SPEC_mnemonic_toolkit_v0_5.md` §6.6 (lines 189-214). Each existing row gains a column documenting how the rule projects into the GUI schema. Concrete patch shape per row:

```
| Row | Flag(s) | Predicate | Error const | gui_projection |
|-----|---------|-----------|-------------|----------------|
| 6   | --threshold | template ∈ single-sig | THRESHOLD_WITHOUT_MULTISIG | (--threshold, Disabled, dropdown_value_in("--template", SINGLE_SIG_TEMPLATES)) |
| 7   | --multisig-path-family | template ∈ single-sig | PATH_FAMILY_WITHOUT_MULTISIG | (--multisig-path-family, Disabled, dropdown_value_in("--template", SINGLE_SIG_TEMPLATES)) |
| 9   | --descriptor + --template | both present | DESCRIPTOR_AND_TEMPLATE | mutex_pair("--descriptor", "--template", Disabled) |
| 9.5 | --descriptor + --descriptor-file | both present | DESCRIPTOR_AND_DESCRIPTOR_FILE | mutex_pair("--descriptor", "--descriptor-file", Disabled) |
| 10  | --descriptor + --threshold | both present | DESCRIPTOR_WITH_THRESHOLD | (--threshold, Disabled, flag_present("--descriptor")) |
| 11  | --descriptor + --multisig-path-family | both present | DESCRIPTOR_WITH_PATH_FAMILY | (--multisig-path-family, Disabled, flag_present("--descriptor")) |
| 12  | --descriptor + --account != 0 | both present | DESCRIPTOR_WITH_NONZERO_ACCOUNT | **DEFERRED v1** (R1 I3 fold — Effect vocabulary for "value-equal-to-zero coerced" not in §2.2; defaulted-to-0 makes this a rare misuse where the CLI error suffices) |
```

(Plus equivalent rows for `verify-bundle`, `export-wallet`, `slip39`, `derive-child` based on the 41-rule manifest from Phase 1. R1 S1 fold: row IDs above use SPEC §6.6 numbering verbatim — see `design/SPEC_mnemonic_toolkit_v0_5.md:199-213`.)

### 2.2 §7.X NEW — Conditional-applicability projection in gui-schema JSON

New subsection (insert after current §7 GUI-schema documentation). Normative text:

> The `mnemonic gui-schema` JSON document gains a per-subcommand `conditional_rules: [ConditionalRule]` array. Each `ConditionalRule` is a triple `(when: Predicate, effect: Effect, rationale: String, spec_ref: String)` that projects one §6.6/§6.9 rule into the GUI's per-frame visibility computation.
>
> **Predicate AST** (tagged JSON union):
> - `{"kind": "flag_present", "flag": "--name"}` — name is a Text/Dropdown/Path/Composite flag and its `has_value` is true
> - `{"kind": "dropdown_value_in", "flag": "--name", "values": ["a", "b"]}` — name's Dropdown value ∈ set
> - `{"kind": "composite_node_is", "flag": "--name", "node": "x"}` — name's Composite node token equals x
> - `{"kind": "positional_present", "index": N}` — positional[N] is non-empty
> - `{"kind": "all_of", "predicates": [P1, P2]}` — conjunction
> - `{"kind": "any_of", "predicates": [P1, P2]}` — disjunction
> - `{"kind": "not", "predicate": P}` — negation
>
> **Effect**: `{"flag": "--name", "visibility": "hidden|disabled|required"}` — the target flag's visibility override when the predicate holds. (Visibility::Visible is the implicit default; never appears as an Effect.)
>
> **Semantics**: when a form's FormState satisfies a rule's predicate, the rule's effect overrides the target flag's visibility for that frame. Multiple rules may target the same flag; effects compose **first-rule-wins** per the existing engine at `mnemonic-gui/src/main.rs:391-394` (`vis.iter().find(...)`). The JSON projection MUST emit rules in priority-descending order per target flag — the first rule whose predicate matches a given form state wins. (R1 C1 fold: the prior plan-draft claim of "later entries win" was inverted; verified against `main.rs:391` which uses `Iterator::find`, returning the first match.)
>
> **Drift invariant**: for every rule in the JSON's `conditional_rules`, the corresponding hand-coded `conditional` fn in `mnemonic-gui/src/form/conditional.rs` MUST return the declared visibility when given an exemplar FormState satisfying the predicate. The drift gate test `mnemonic-gui/tests/gui_schema_conditional_drift.rs` enforces this byte-exactly.

### 2.3 Toolkit FOLLOWUPS

`design/FOLLOWUPS.md` new entry `gui-schema-conditional-rules-v1`:
- **Status:** in-progress (this cycle)
- **Surfaces:** projects SPEC §6.6/§6.9 mutex/conditional rules into `gui-schema` JSON; drift-gated against mnemonic-gui hand-coded conditionals
- **Companion:** mnemonic-gui FOLLOWUP `gui-conditional-applicability-drift-fix` (created in this cycle)
- **Deferred:** slot-count-dependent runtime rules, BIP-388 distinct-key check (see §1.4)

### 2.4 mnemonic-gui SPEC

No mnemonic-gui SPEC file exists yet; the `pinned-upstream.toml` + design docs are the canonical surfaces. Add a CHANGELOG entry citing toolkit SPEC §6.6/§7.X and the new conditional surfaces.

---

## 3. Rule manifest — v1 coverage status

(Source: Phase 1 explore agent report. Status reflects code in master at exploration time.)

| # | Subcommand | Rule | Status |
|---|---|---|---|
| 1 | bundle | --descriptor ↔ --descriptor-file mutex | ✅ encoded |
| 2 | bundle | --descriptor ↔ --template mutex | ⚠️ NEW (not in current `bundle()` — only `--template` Required-unless is encoded) |
| 3 | bundle | --threshold disabled when template ∈ single-sig | **NEW** |
| 4 | bundle | --multisig-path-family disabled when template ∈ single-sig | **NEW** |
| 5 | bundle | --threshold disabled when --descriptor present | **NEW** |
| 6 | bundle | --multisig-path-family disabled when --descriptor present | **NEW** |
| 7 | bundle | --account != 0 disallowed when --descriptor present | **DEFERRED v1** (R1 I3 fold — see §2.1 row 12; defaulted-to-0 means CLI rejection only fires on active user misuse, and the Effect vocabulary for "value-equal-to-zero coerced" is not in §2.2; defer to FOLLOWUP `gui-schema-numeric-flag-value-pin-effect`) |
| 8 | bundle | --passphrase ↔ --passphrase-stdin mutex | ✅ encoded |
| 9 | bundle | --template Required-unless-descriptor | ✅ encoded |
| 10 | verify-bundle | --threshold disabled when template ∈ single-sig | **NEW** |
| 11 | verify-bundle | --multisig-path-family disabled when template ∈ single-sig | **NEW** |
| 12 | verify-bundle | --descriptor ↔ --template mutex | ⚠️ partial (only --template Required-unless) |
| 13 | verify-bundle | --bundle-json XOR (--ms1, --mk1, --md1) | ✅ encoded |
| 14 | verify-bundle | --passphrase ↔ --passphrase-stdin mutex | ✅ encoded |
| 15 | export-wallet | --template ↔ --descriptor mutex + Required-one-of | ✅ encoded |
| 16 | export-wallet | --taproot-internal-key disabled when template ∉ tr-multi-a/tr-sortedmulti-a | **NEW** |
| 17 | export-wallet | --taproot-internal-key required when template ∈ tr-multi-a/tr-sortedmulti-a | **NEW** |
| 18 | export-wallet | --threshold disabled when template ∈ single-sig | **NEW** |
| 19 | export-wallet | --multisig-path-family disabled when template ∈ single-sig | **NEW** |
| 20 | convert | --passphrase ↔ --passphrase-stdin mutex | ✅ encoded |
| 21 | convert | --bip38-passphrase ↔ --bip38-passphrase-stdin mutex | ✅ encoded |
| 22 | convert | --xpub-prefix non-default requires --network | **NEW** (Required-marker on --network) |
| 23 | derive-child | --passphrase ↔ --passphrase-stdin mutex | ✅ encoded |
| 24 | derive-child | --dice-sides required when --application dice | **NEW** |
| 25 | slip39-split | --passphrase ↔ --passphrase-stdin mutex | ✅ encoded |
| 26 | slip39-split | --language hidden when --from node == entropy | ✅ encoded |
| 27 | slip39-combine | --passphrase ↔ --passphrase-stdin mutex | ✅ encoded |
| 28 | slip39-combine | --language hidden when --to == entropy | ✅ encoded |
| 29 | md encode | template positional ↔ --from-policy XOR + value-inspect | ✅ encoded |
| 30 | md compile | --unspendable-key disabled when --context == segwitv0 | ✅ encoded |
| 31 | md address | phrases positional ↔ --template XOR | ✅ encoded |
| 32 | ms encode | --phrase ↔ --hex XOR + --language Hidden on hex | ✅ encoded |
| 33 | mk encode | --origin-fingerprint ↔ --privacy-preserving mutex | ✅ encoded |
| 34 | seed-xor split | (TBD — explore agent did not cover) | ⚠️ confirm in P2 |
| 35 | seed-xor combine | (TBD — explore agent did not cover) | ⚠️ confirm in P2 |
| 36-41 | runtime rules (T-in-range, N-vs-template, BIP-388 distinct-key, convert edge-class refusals, etc.) | — | **DEFERRED** (§1.4) |

**v1 implementation cost:** ~14 NEW rules (rule 7 deferred per R1 I3) + 2 partials + 2 TBDs (seed-xor) ≈ ~17 conditional fn additions across `bundle()`, `verify_bundle()`, `export_wallet()`, `convert()`, `derive_child()`, plus the JSON projection on the toolkit side.

**Rule count reconciliation (R1 S1 fold):** the "41 total rules" figure from §1.4 conflates enforceable visibility rules (rows 1-33 above) with runtime/dynamic rules (rows 34-41). SPEC §6.6 documents rows 1-14 plus runtime rows (`pre_check_threshold` T-in-range, `pre_check_template_n` N-vs-template, BIP-388 distinct-key) at `design/SPEC_mnemonic_toolkit_v0_5.md:199-213`. The accurate framing: **~33 enforceable in v1's visibility model + ~8 runtime rules deferred**.

---

## 4. Phased implementation (TDD per phase)

**Prerequisite (added 2026-05-16):** `mnemonic-gui-v0.4.3` (toolkit v0.15.0 wire-format catchup) MUST be tagged + CI-green BEFORE P1 advances. P0 (toolkit SPEC + FOLLOWUPS) is independent of the GUI catchup and may commit on toolkit master in advance. Phases P1–P5 + the drift gate (P4) MUST run against the v0.15.0-clean GUI baseline so their reviewer-loop disambiguates v0.16.0 cycle changes from v0.15.0 wire-format drift. See revision note at the top of this file.

### P0 — SPEC patch (toolkit)

**Files**: `design/SPEC_mnemonic_toolkit_v0_5.md`, `design/FOLLOWUPS.md`

**Deliverables**:
- §6.6 table gains `gui_projection` column for all 14 enforced rows
- §7.X NEW subsection per §2.2 above (Predicate AST + Effect + drift invariant)
- FOLLOWUPS entries: in-progress `gui-schema-conditional-rules-v1`, deferred `gui-schema-runtime-conditional-projection`

**Tests**: SPEC-mirror grep (existing convention in `tests/spec_mirror_*.rs` if present — confirm in P0 setup; otherwise pin by inclusion in the gui_schema.rs serializer constants)

**No code change in this phase. Pure SPEC.**

### P1 — Toolkit `gui_schema.rs` JSON projection

**Files**: `crates/mnemonic-toolkit/src/cmd/gui_schema.rs`, `crates/mnemonic-toolkit/tests/cli_gui_schema_*.rs`, **and `mnemonic-gui/src/schema_check.rs`** (R1 I5 fold — see version-relax below)

**Deliverables**:
- New types: `ConditionalRule { rationale: String, spec_ref: String, when: Predicate, effect: Effect }`, `Predicate` (tagged enum per §2.2), `Effect { flag: String, visibility: VisibilityProjection }`, `VisibilityProjection` enum (Hidden/Disabled/Required — Visible omitted)
- `serde::Serialize` impls producing the JSON shape in §2.2
- Rules emitted in **priority-descending order per target flag** (R1 C1 fold; the first rule whose predicate matches wins per `main.rs:391` semantics)
- New `build_subcommand_conditional_rules(name) -> Vec<ConditionalRule>` keyed off subcommand, hand-encoding the ~17 v1 rules per §3 manifest
- Wire `conditional_rules` field into the existing `GuiSchemaSubcommand` serialization at the per-subcommand level
- `version: u32` bump from 1 → 2 (additive: v1 consumers parse v2 docs ignoring unknown fields by serde-default)

**Version-relaxation contract (R1 I5 fold)** — the GUI's existing consumer at `mnemonic-gui/src/schema_check.rs:105-110` hard-rejects `version != 1` and returns `None`. The fix must land in lockstep:
- Relax `parse_gui_schema_json` to accept `version >= 1` (current uses only `version` for the gate; flag-name extraction is unaffected by v2's additive fields)
- ADD a new fn `parse_gui_schema_conditional_rules(json_str, subcommand_name) -> Option<Vec<ConditionalRule>>` next to the existing `parse_gui_schema_json`. The new fn requires `version >= 2`; v1 docs return `None` (legacy compat — the conditional-rules consumer is the new drift gate test, which gates on v2 minimum)
- Confirm `schema_check.rs:67-90` `GuiSchemaSubcommand` struct is extended (or a parallel `GuiSchemaSubcommandV2` is added) with `conditional_rules: Vec<ConditionalRule> #[serde(default)]` so the existing test that consumes only flag names sees no behavior change

**Tests** (TDD-first):
- `cli_gui_schema_conditional_rules_bundle.rs` — snapshot test: `mnemonic gui-schema` emits expected bundle rules including (3), (4), (5), (6) from §3 (rule 7 deferred per R1 I3)
- `cli_gui_schema_conditional_rules_verify_bundle.rs` — same shape, rules (10), (11), (12)
- `cli_gui_schema_conditional_rules_export_wallet.rs` — rules (16), (17), (18), (19)
- `cli_gui_schema_predicate_serde.rs` — round-trip serde for each Predicate variant
- `cli_gui_schema_version_bump.rs` — version field == 2; backward-compat consumer test (parse-only-flag-names path still works on v2 docs)
- `cli_gui_schema_priority_order.rs` — for any flag with multiple rules, assert emission order matches first-wins priority intent (e.g., bundle's `--threshold` has rule 5 "descriptor mode" emitted BEFORE rule 3 "single-sig template" since descriptor-mode-then-template-pick is a more specific predicate)

**Existing reference**: `crates/mnemonic-toolkit/src/cmd/gui_schema.rs:125` (version pin); the existing per-flag serializer is the pattern template.

### P2 — GUI `conditional.rs` extensions

**Files**: `mnemonic-gui/src/form/conditional.rs`, `mnemonic-gui/tests/conditional_visibility.rs`

**Deliverables**:
- Add `SINGLE_SIG_TEMPLATES: &[&str] = &["bip44", "bip49", "bip84", "bip86"]` module-level constant (mirror toolkit-side `Template::is_multisig` predicate inversion). Source-of-truth lives in toolkit; GUI replicates the set + drift gate enforces parity.
- Extend `bundle(state)` with rules (2), (3), (4), (5), (6) from §3 (rule 7 deferred per R1 I3 → FOLLOWUP `gui-schema-numeric-flag-value-pin-effect`)
- Extend `verify_bundle(state)` with rules (10), (11), (12)
- Extend `export_wallet(state)` with rules (16), (17), (18), (19)
- Extend `convert(state)` with rule (22) — Required-marker on --network when --xpub-prefix non-default
- Extend `derive_child(state)` with rule (24) — Required-marker on --dice-sides when --application dice
- Confirm seed-xor coverage (rules 34, 35); add conditional fn entries if any constraints surface

**Tests** (TDD-first):
- One new cell per NEW rule in `conditional_visibility.rs`, asserting: given a synthesized FormState satisfying the rule's predicate, the conditional fn returns the expected Visibility for the target flag
- Negative cells: predicate-not-satisfied → flag Visible (or current baseline visibility)
- Compose cells: simultaneous template-single-sig AND --descriptor-present → both rules' effects compose deterministically (**first-rule-wins** per §2.2 / R1 C1 fold; `mnemonic-gui/src/main.rs:391-394` uses `Iterator::find`. The conditional fn's emission order in the `vis: Vec<(name, Visibility)>` IS the priority order, not Vec append order)

**Existing reference**: `mnemonic-gui/src/form/conditional.rs:21-365` — the pattern. Mirror the doc-comment style citing upstream `cmd/bundle.rs:NNN`.

### P3 — GUI `assemble_argv` visibility gate

**Files**: `mnemonic-gui/src/form/invocation.rs`, `mnemonic-gui/tests/argv_assembler.rs`, `mnemonic-gui/tests/argv_assembler_visibility.rs` (NEW)

**Deliverables**:
- `assemble_argv` computes visibility internally via `subcommand.conditional.map(|f| f(state)).unwrap_or_default()` (R1 S2 fold: pick the in-fn path explicitly; the render loop at `main.rs:386-389` continues to compute its own copy since `FlagVisibility` is a `Vec` and not `Copy`). Public signature unchanged.
- Before per-flag dispatch, look up the flag's effective visibility. **Both Hidden AND Disabled suppress emission**; Required does not affect emission (R1 I2 fold). Rationale:
  - **Hidden** = structurally non-applicable to current mode (e.g., `--threshold` when template ∈ single-sig). Widget is removed from the form; emission must follow.
  - **Disabled** = user-side mutex (they chose a conflicting sibling, e.g., set `--passphrase-stdin` so `--passphrase` is grayed). Widget remains visible-but-grayed; the value in state is retained so toggling the mutex restores it; but emission MUST be suppressed because the user explicitly opted into the alternative path. (Verified: prior plan-draft policy of "Disabled emits" would (i) fail the user's reported bug since a Number widget defaults to `min=1` and would still emit, AND (ii) silently break existing mutex pairs where the user's typed-then-mutex-disabled value would clap-reject under `conflicts_with`. Reviewer-confirmed via bundle.rs:200-205 `--account != 0` case: a user who typed `5` then enables `--descriptor` would still emit `--account 5` and trigger SPEC §6.6 row 12.)
  - **Required** = decorative validation hint; widget renders with `*` marker; no emission effect.
- The visibility check fires at the TOP of the per-flag iteration loop, BEFORE both the secret-flag branch and the `state.values` branch (R1 S3 fold; affects `invocation.rs:62-94`).
- Slot emission (`invocation.rs:54-61` when `flag.name == "--slot" && allows_slots`) is unaffected by visibility (slot values are not gated by §6.6/§6.9 rules in v1; the slot-mix and slot-contiguity checks are runtime/deferred per §1.4).

**Tests** (TDD-first):
- `argv_assembler_visibility.rs`: for each NEW Hidden- OR Disabled-effecting rule in §3, assert the target flag is absent from the assembled argv when the rule's predicate is satisfied
- Composition: simultaneous template-single-sig (Disabled threshold) AND `--descriptor` present (Disabled threshold via different rule) → first-wins per C1 fold; emission is suppressed under both predicates
- Required-decoration regression: a flag marked Required by its conditional fn DOES still emit (e.g., `--mk1` Required-unless-bundle-json in verify-bundle); existing argv-shape cells in `argv_assembler.rs` must remain green
- Mutex-pair argv-emission regression: typed-then-mutex-disabled value MUST NOT emit (e.g., user types `--passphrase=foo`, then sets `--passphrase-stdin`; assert argv contains `--passphrase-stdin` but NOT `--passphrase foo`). This is a LATENT-bug fix piggybacking on the visibility-gate rollout — call it out in P3 release notes.

**Design note** (resolved): visibility computation in-fn. Pseudocode:

```rust
pub fn assemble_argv(schema, subcommand, state) -> Vec<String> {
    let vis = subcommand.conditional.map(|f| f(state)).unwrap_or_default();
    let visibility_of = |name| vis.iter()
        .find(|(n, _)| *n == name)
        .map(|(_, v)| *v)
        .unwrap_or(Visibility::Visible);
    let suppresses = |v| matches!(v, Visibility::Hidden | Visibility::Disabled);
    for flag in subcommand.flags {
        if suppresses(visibility_of(flag.name)) { continue; }
        ...
    }
}
```

This keeps the public signature stable and mirrors the render-loop pattern at `main.rs:390-395` for first-wins resolution.

### P4 — Drift gate

**Files**: `mnemonic-gui/tests/gui_schema_conditional_drift.rs` (NEW)

**Deliverables**:
- Test shells out to `<MNEMONIC_BIN> gui-schema` (env var per `schema_mirror.rs:51-195` convention)
- Parses JSON via serde, extracts `conditional_rules` per subcommand
- For each rule:
  1. Synthesize an exemplar FormState from `when` predicate (helper `synthesize_form_state_for(predicate) -> FormState`)
  2. Invoke the corresponding `SubcommandSchema.conditional` fn from the hand-coded schema
  3. Assert: the returned `FlagVisibility` contains `(effect.flag, effect.visibility)` (or that Visibility::Visible is the result if predicate is NOT satisfied — round-trip both polarities)
- Loop over all subcommands the toolkit emits gui-schema for, all rules

**Tests within the test**:
- Per-rule cells via `rstest` parameterization or manual unrolling — match the existing per-cell pattern in `conditional_visibility.rs`
- Failure message must cite the rule's `rationale` + `spec_ref` so a future drift surfaces the exact rule that broke

**Existing reference**: `mnemonic-gui/tests/schema_mirror.rs` — the binary-resolution helper + JSON parser pattern. Reuse `parse_gui_schema_json` extended to deserialize `conditional_rules` (currently it only extracts flag names per `src/schema_check.rs:100-114`).

### P5 — Default form-state cleanup

**Files**: `mnemonic-gui/src/main.rs`, `mnemonic-gui/tests/widget_interaction.rs` (or equivalent)

**Deliverables**:
- Remove `("--multisig-path-family", FlagValue::Dropdown("bip87".into()))` from `main.rs:203` (the default form-state seed is for the *screenshot demo*; pre-seeding a multisig-only flag with default template = `bip84` is the original "user has no way to disable it" bug)
- Add a test cell verifying that the bundle form's default state, on startup, produces an argv that the CLI accepts under `--template bip84` (smoke-test against installed `mnemonic` binary, gated by `MNEMONIC_BIN` env var per existing test conventions)

**Tests** (TDD-first):
- `bundle_default_form_state_smoke_test`: invoke `mnemonic bundle --network mainnet --template bip84 --self-check --slot @0.phrase=<canonical-test-vector>` and assert exit 0 (only when `MNEMONIC_BIN` is set; otherwise skip)

---

## 5. Verification

### 5.1 Manual reproduction (the user's bug)

**Prerequisite:** the `mnemonic-gui v0.4.3` catchup (toolkit pin v0.14.2 → v0.15.0) must be on master before this reproduction runs; otherwise the reproduction's `cargo run --release` exercises v0.14.2 wire-format paths and the §5.1 exit-0 assertion is testing a v0.16.0 conditional-applicability fix against a v0.14.2 toolkit, not the intended v0.16.0-clean baseline. This reproduction tests **v0.16.0 conditional-applicability behavior only**; v0.15.0 wire-format paths are validated separately in the v0.4.3 catchup cycle.

```sh
# Before: GUI emits --threshold 1 --multisig-path-family bip48 on bip84 → CLI exit 2
# After:  GUI hides --threshold and --multisig-path-family widgets; argv excludes them.

cd /scratch/code/shibboleth/mnemonic-gui
cargo run --release
# In the launched GUI:
#   1. Mnemonic tab → bundle subcommand
#   2. Confirm: --template defaults to bip84
#   3. Confirm: --threshold widget grayed/hidden
#   4. Confirm: --multisig-path-family widget grayed/hidden
#   5. Fill --slot @0.phrase=<canonical-test-vector>
#   6. Click Run → CLI exit 0 with valid bundle output
```

### 5.2 Full test suite

```sh
# Toolkit
cd /scratch/code/shibboleth/mnemonic-toolkit
cargo test --release --workspace
# expect: all P1 new cells pass; pre-existing cells regression-free

# GUI
cd /scratch/code/shibboleth/mnemonic-gui
MNEMONIC_BIN=/scratch/code/shibboleth/mnemonic-toolkit/target/release/mnemonic \
  cargo test --release
# expect: P2/P3/P4/P5 new cells pass; pre-existing cells regression-free
```

### 5.3 Drift gate cross-validation

```sh
# Manually verify the drift gate catches a planted divergence.
cd /scratch/code/shibboleth/mnemonic-gui
# Temporarily comment out one new rule in src/form/conditional.rs (e.g., rule 3 from §3)
MNEMONIC_BIN=... cargo test --release gui_schema_conditional_drift
# expect: test FAILS, citing the unencoded rule's rationale
# Restore the rule.
```

### 5.4 CI green light

Both repos: `cargo test` + existing schema-mirror tests + `docs/manual/tests/lint.sh` (toolkit only) all green on master before tag.

**Precondition (added 2026-05-16):** `mnemonic-gui-v0.4.3` tag CI must be fully green BEFORE this cycle's `mnemonic-gui-v0.5.0` tag is cut. The v0.4.3 catchup serves as the scope-isolated v0.15.0 wire-format regression gate; this cycle's v0.5.0 builds on top of it.

---

## 6. Tag plan

### 6.1 toolkit `v0.16.0`

- Minor bump: additive `gui-schema` JSON v2 (backward-compatible flag-name extraction still works on v2)
- Release notes cite SPEC §6.6/§7.X + the conditional-applicability projection
- No CLI behavior change — only schema export surface

### 6.2 mnemonic-gui `v0.5.0`

- Minor bump: enforces new template/descriptor-mode conditional rules; adds drift gate
- Release notes cite the motivating bug + lockstep toolkit `v0.16.0`
- `Cargo.toml` + `pinned-upstream.toml`: bump `mnemonic-toolkit` pin from v0.15.0 (post-v0.4.3-catchup baseline) to v0.16.0
- **Predecessor:** `mnemonic-gui-v0.4.3` (toolkit v0.15.0 wire-format catchup; scope-isolated). v0.5.0 builds atop v0.4.3.

### 6.3 Cross-repo lockstep

Both tags ship in lockstep per CLAUDE.md cross-repo follow-up conventions:
- toolkit FOLLOWUP `gui-schema-conditional-rules-v1` closed at toolkit v0.16.0 tag commit
- mnemonic-gui FOLLOWUP `gui-conditional-applicability-drift-fix` closed at mnemonic-gui v0.5.0 tag commit
- Each entry's `Companion:` line cites the other

---

## 7. FOLLOWUPS filed during this cycle

1. **`gui-schema-runtime-conditional-projection`** (toolkit + gui, deferred) — extend the conditional projection to slot-count-dependent rules (T-in-range vs N, single-sig-N>1, BIP-388 distinct-key) once a form-side slot-count signal is plumbed through to the conditional engine.
2. **`gui-number-widget-unset-sentinel`** (gui, deferred) — Number/Range/Timestamp/TaggedOrIndexed widgets currently have no "no value" sentinel. v1 sidesteps this via the visibility gate (Hidden + Disabled flags don't emit regardless). A future cycle may add an explicit unset state for UX clarity (e.g., a "clear" affordance next to numeric widgets).
3. **`gui-default-form-state-template-aware-seed`** (gui, optional follow-on) — replace the static-default screenshot-mode seed at `main.rs:197-211` with a template-aware default (e.g., select multisig defaults when the user picks a multisig template). Out of v1 scope but a natural successor to P5.
4. **`gui-schema-numeric-flag-value-pin-effect`** (toolkit + gui, deferred — R1 I3 fold) — add a `pin_value: { flag, value: Number(0) }` Effect variant to §2.2 vocabulary so SPEC §6.6 row 12 ("--account != 0 when --descriptor present") can be projected. Currently deferred because the GUI default of 0 is the safe value; the CLI's byte-exact error is informative for active misuse.
5. **`gui-schema-template-groups-meta-field`** (toolkit + gui, deferred — R1 I4 fold) — emit a per-subcommand `meta.template_groups: { single_sig: [..], multisig: [..] }` block sourced from `Template::is_multisig()`, eliminating the `SINGLE_SIG_TEMPLATES` static const in `mnemonic-gui/src/form/conditional.rs`. Currently the drift gate suffices to catch divergence, but a future cleanup cycle can collapse the const.

---

## 8. Critical files index

| File | Repo | Phase | Role |
|---|---|---|---|
| `design/SPEC_mnemonic_toolkit_v0_5.md` | toolkit | P0 | SPEC §6.6/§7.X patch target |
| `design/FOLLOWUPS.md` | toolkit | P0 | FOLLOWUP entries |
| `crates/mnemonic-toolkit/src/cmd/gui_schema.rs` | toolkit | P1 | JSON projection emitter |
| `crates/mnemonic-toolkit/src/cmd/bundle.rs` (mode_text module, lines 118-130) | toolkit | P0/P1 (reference) | Byte-exact rule constants — DO NOT MODIFY semantics |
| `src/form/conditional.rs` | gui | P2 | Add ~17 new rule entries (rule 7 deferred per R1 I3) |
| `src/schema/mod.rs` (Visibility enum, line 111) | gui | P2 (reference) | Existing infrastructure |
| `src/form/invocation.rs` (assemble_argv, lines 42-185) | gui | P3 | Visibility-gate insertion |
| `src/schema_check.rs` (parse_gui_schema_json, lines 100-114; version gate at 105-110) | gui | **P1+P4** (R1 I5 fold) | Relax version gate to `>= 1` for flag-name path; ADD parallel `parse_gui_schema_conditional_rules` fn requiring `version >= 2` |
| `tests/conditional_visibility.rs` | gui | P2 | Test-cell pattern; add new cells |
| `tests/argv_assembler.rs` | gui | P3 (reference) | Test-cell pattern for argv assertions |
| `tests/schema_mirror.rs` (binary resolution, lines 51-195) | gui | P4 (reference) | Helper functions to reuse |
| `tests/gui_schema_conditional_drift.rs` | gui | P4 | NEW drift-gate test file |
| `src/main.rs` (default seed, lines 197-211) | gui | P5 | Remove `--multisig-path-family` seed |

---

## 9. Build sequence summary

```
P0 (SPEC + FOLLOWUPS; toolkit-only, commit on master) →
  PREREQ: mnemonic-gui v0.4.3 catchup (toolkit pin v0.14.2→v0.15.0; tag + CI green) →
    P1 (toolkit gui_schema.rs + tests; cargo test toolkit green) →
      [Toolkit can ship to a local tag-candidate at this point if desired]
    P2 (gui conditional.rs + tests; cargo test gui green WITHOUT P1's new fields wired) →
      P3 (gui assemble_argv gate + tests) →
        P5 (gui main.rs default-state cleanup + smoke test) →
          P4 (gui drift gate; pin against toolkit P1 output via MNEMONIC_BIN)
            → manual reproduction check (§5.1)
              → toolkit tag v0.16.0
                → mnemonic-gui pinned-upstream.toml bump + tag v0.5.0
                  → CI green on both tags
                    → release notes + FOLLOWUP closure
```

Estimated cycle wall-clock: 1-1.5 working days for P1–P4 (P1 + P2 are the bulk; P3/P4/P5 are mechanically straightforward). The v0.4.3 catchup prerequisite is an independent ~half-session before P1.

---

## 10. Per-phase reviewer-loop policy

Per memory `[feedback-opus-primary-review-agent]` and user-confirmed end-only opus cadence this cycle:

- Per-phase: source-grounded R0 self-check by the implementing agent (grep/cite, verify against §3 manifest)
- End-of-cycle: ONE opus reviewer-loop on the consolidated PR diff (both repos) per memory `[feedback-r0-must-read-source-off-by-n]` and `[feedback-architect-must-run-prose-commands]` (architect MUST run the §5.1 manual reproduction + §5.2/§5.3 test suites end-to-end, not just source-grep)

Reviewer-loop terminates at 0 critical / 0 important. Fold pattern: Critical → block-merge; Important → fold this cycle; Surface → defer to FOLLOWUPS unless mechanically trivial.
