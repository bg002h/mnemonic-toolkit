# Cycle-6 brainstorm spec — R0 review, round 1

- **Spec under review:** `design/BRAINSTORM_cycle6_timelock_decay.md`
- **Cluster:** timelock decaying-multisig decay-ordering funds-loss (D-decay-rel cross-unit BIP-68 `older()` blindness; D-decay-abs past `after(T)` immediately-spendable).
- **origin/master SHA:** `3fa2925b02a0d29846e823a1bfd4cec44256f70b` (toolkit 0.63.0, post-cycle-5/S-NET)
- **Date:** 2026-06-21
- **Reviewer:** opus software architect (independent; spec claims treated as hypotheses, re-verified against live source)

All citations in §0 of the spec were independently re-grepped against `3fa2925b`. Every architectural citation is ACCURATE (see "Citation audit" below). The protocol corrections (BIP-68 bit-22 = `0x0040_0000`; `older_consensus_masked` returns `None` for clean operands; BIP-65 `500_000_000` height/time split) are CONFIRMED against live source. The funds-safety thesis is sound and the two-predicate decomposition is correct and complete for the fixed 3-tier decaying shape.

**However**, the spec carries one CRITICAL self-contradiction (its own recommended positive-control / canon fixture value is refused by its own predicate) and two IMPORTANT coupled-fixture omissions (two existing tests that will go RED under the fix and are NOT in the spec's "THREE coupled sites" budget). These must be folded before R0 can be GREEN.

---

## Citation audit (independently re-verified @ `3fa2925b`)

| Spec claim | Live evidence | Verdict |
|---|---|---|
| `archetype.rs:305-317` raw `recovery_older <= older` compare, no unit norm | `:305` `if def.id == "decaying-multisig"`, `:307` `if recovery_older <= older`, `:311` format | ACCURATE |
| `validate_params` @244; `ArchetypeParams.older/.recovery_older/.after` @24/25/26 `Option<u32>` | confirmed | ACCURATE |
| `lower_decaying_multisig` @395-419 tier3→tier2→root andor nesting | confirmed; tier3 `and_v(v:pk(F), After(T))`, tier2 `andor(multi(recov), Older(N2), tier3)`, root `andor(multi(prim), Older(N1), tier2)` | ACCURATE |
| `param_diag` @326 builds `Diagnostic{kind: Param, flag: Some(..)}` | confirmed @326-333; **note it sets `flag: Some(flag)`** (provenance preserved) | ACCURATE (+ see Minor M3) |
| `gate.rs:306-324` After arm range-only (`n==0` @307-311; `n>0x7FFF_FFFF` @312-323); no future-ness | confirmed | ACCURATE |
| gate Older arm refuses ONLY masked operands (clean `0x40_0001` passes) | `:270` `if let Some(consequence) = older_consensus_masked(*n)` — clean ⇒ `None` ⇒ no diag | ACCURATE — confirms cross-unit reachability |
| `DiagnosticKind::Param` exists @90/123 | `:90` enum, `:123` `as_str => "param"` | ACCURATE |
| `older_consensus_masked` `None` for clean operands; inline unit `n & 0x0040_0000` @52 | `timelock_advisory.rs:48` predicate; tests @249-254 prove `[1,2016,52560,65535]` + `[0x40_0001,0x40_FFFF]` ⇒ `None`; @52 unit classify | ACCURATE — **`older_unit_value` helper is justified** |
| BIP-65 `500_000_000` split @ `cost/enumerate.rs:72-74` | confirmed (`<500_000_000`=height, `≥`=MTP-time); split used @242/248 | ACCURATE |
| validate→gate dispatch: `validate_params` @278 FIRST (`return Ok(2)`), gate @287 SECOND | confirmed; spec-mode (`--spec`) @321+ does NOT call `validate_params` | ACCURATE (+ see C1/I1/I2 implications) |
| existing decay neg test `:546-577` (same-unit `2000/2000`, exit 2) | confirmed; asserts stderr contains `--recovery-older` + `--older` | ACCURATE (+ see I1) |
| canon golden `:56-84` (`--older 1000 / --recovery-older 2000 / --after 500000`) | confirmed | ACCURATE |
| `:683` mutation `("decaying-multisig","--after","500001")` asserts `.success()` | confirmed @683; harness mutates canon `--after`, asserts exit 0 + descriptor differs | ACCURATE |
| SPEC §3.1 item 3 @110 unit-blind rule; §3.3 `Param`-not-`ToolkitError` note | confirmed @110, @119 | ACCURATE |
| FOLLOWUP `archetype-older-blocks-flag-accepts-time-units` @219-225 | confirmed (now @ that block); boundary-test caveat genuine | ACCURATE |
| Cargo.toml version 0.63.0 (MINOR → 0.64.0) | confirmed `:3 version = "0.63.0"` | ACCURATE |
| bitcoind oracle never consumes build-descriptor | `tests/bitcoind_differential.rs` — 0 `build-descriptor`/`decaying` refs | ACCURATE (R6 N/A) |
| `--spec` escape hatch is a real flag | `build_descriptor.rs:30` `pub spec`, `:58` `conflicts_with = "spec"` | ACCURATE |

---

## Critical

### C1 — The spec's own recommended canon/control value `--after 900000000` is REFUSED by its own D-decay-abs predicate (self-contradictory fixture)

**Where:** §4.2 prose ("recommend `--after 900000000`, a forward-dated MTP-time Unix timestamp"), R7 ("recommend MTP-time `--after 900000000`"), §7 abs-future-time control row ("`--after 900000000` (MTP-time, `≥ 500_000_000`, future) → still builds"), and the §7 fixture-regeneration note ("base `--after 900000000`, mutation `--after 900000001`").

**Evidence:** The spec defines `ABS_TIME_PAST_FLOOR = 1_750_000_000` (≈ 2025-06-15). Trace `--after 900000000` through the spec's own predicate:
- `900000000 < 500_000_000`? **No** ⇒ classified as **MTP-time** (correct classification).
- MTP-time past check: `after < ABS_TIME_PAST_FLOOR` ⇒ `900000000 < 1_750_000_000` ⇒ **TRUE ⇒ REFUSED as past.**

`900000000` as a Unix timestamp is **1998-07-09**, deeply in the past — NOT the future. The number is large enough to clear the `500_000_000` height/time threshold (so it *classifies* as time), but it is far below the time floor, so the predicate refuses it. Consequently:
- the canon positive golden (R7) would NOT build → `preset_descriptor_goldens` / `preset_bip388_goldens` / `emit_spec_value_equals_fixture_and_round_trips` go RED;
- the §7 abs-future-time **positive control** ("still builds") would FAIL (it would exit 2);
- the `:683`-replacement mutation `900000001` would also be refused, defeating its "mutate a numeric param, golden differs, still exits 0" purpose.

**Why Critical (not Minor):** this is the load-bearing positive-control value the TDD pass depends on. If the implementer follows the spec literally, the GREEN path is unreachable and the cycle stalls; worse, an implementer "fixing" the RED by relaxing the floor would erode the funds-safety guarantee. The error is that the spec conflated "a big number that classifies as time" with "a future timestamp." (Verified arithmetically: floors `{ABS_HEIGHT_PAST_FLOOR=900_000, ABS_TIME_PAST_FLOOR=1_750_000_000}` ⇒ `500000`→refused, `500001`→refused, `900000000`→**refused**, `900000001`→**refused**, `4000000`→builds, `4000001`→builds, `2_000_000_000`→builds, `1_900_000_000`→builds.)

**Required fix:** Choose canon/control values that are genuinely future under the chosen floors. Either
(a) a **block height** future value `--after 4000000` (height, `4_000_000 > 900_000` ⇒ builds; ~58 yr ahead of mainnet tip — comfortably future and stable), with mutation `4000001`; OR
(b) a **genuinely-future MTP-time** `--after 2000000000` (= 2033-05-18, `> 1_750_000_000` ⇒ builds), with mutation `2000000001`.
Pick ONE and use it consistently across R7, §4.2, §7 control rows, the §7 fixture-regen note, and the abs-future-time control. The height option (a) `4000000` keeps the canon in the same height *axis* as today's `500000` (minimal golden churn, no axis flip) and is the recommended choice; the spec's own §7 already lists `--after 4000000` as a valid "plausibly-future height" abs control — promote that to the canon. Delete every `900000000` recommendation.

---

## Important

### I1 — Coupled-fixture omission #1: the in-crate unit test `validate_params_decay_ordering` (`archetype.rs:743`) will go RED — it asserts `diags.len() == 1` over a fixture carrying past `after`

**Where:** `crates/mnemonic-toolkit/src/descriptor_builder/archetype.rs:743-754` (`validate_params_decay_ordering`) consumes `fixture_params("decaying-multisig")` (`archetype.rs:534-543`), which sets **`after: Some(500000)`** (`:542` — a past height) plus `older: Some(1000)`, and mutates `recovery_older` to `[1000, 999]`, asserting `assert_eq!(diags.len(), 1)`.

**Evidence:** Under the D-decay-abs predicate (which lands in the SAME `validate_params` decay arm), the fixture's `after = 500000` is a past height ⇒ predicate-2 pushes a SECOND diagnostic. With `recovery_older ∈ {1000, 999}` (predicate-1 also fires: same-unit `v2 <= v1`), `validate_params` now returns **`diags.len() == 2`**, breaking `assert_eq!(diags.len(), 1)`. This test goes RED and is NOT in the spec's §7 "THREE coupled sites" budget (which lists only the JSON fixture, the `.descriptor`/`.bip388` goldens, and the `:683` CLI mutation). The spec's claim that the existing `2000/2000` case "stays GREEN by construction" is true for the *CLI* test `:546` (whose assertions only check stderr substring, and which is masked by the new abs diag firing the same exit 2) but is silent on this *unit-layer* fixture, which has a hard `len() == 1` count assertion.

**Required fix:** The spec must (a) enumerate `fixture_params` `archetype.rs:542` `after: Some(500000)` as a coupled site, and (b) prescribe the update — change the fixture's `after` to the chosen future value (per the C1 resolution), which keeps `diags.len() == 1` (only the rel diagnostic fires). Confirm in §7's coupled-site list. (This is the SAME root cause as C1 — a stale `500000` baseline — but a DISTINCT site the spec missed.)

### I2 — Coupled-fixture omission #2: the CLI test at `cli_build_descriptor.rs:~1000-1031` (`repeated_keys` localization) will go RED — the past `after` now short-circuits before the gate ever runs

**Where:** `crates/mnemonic-toolkit/tests/cli_build_descriptor.rs:1000-1031` (the "Decaying intra-andor[2] cross-tier dup" block inside `gate_diagnostics_carry_flag_provenance_in_preset_mode`, line ~800). It invokes `--archetype decaying-multisig` with a correctly-ordered same-unit `--older 1000 / --recovery-older 2000`, a duplicate `--final-key K3 == --recovery-key K3`, and **`--after 500000`** (`:1020`), asserting `.code(2)` AND `diagnostics[0].kind == "repeated_keys"` at `node_path == "root.andor[2]"`.

**Evidence:** `validate_params` runs FIRST (`build_descriptor.rs:278`) and `return Ok(2)` on any diag BEFORE the gate (`:287`). Under D-decay-abs, `--after 500000` (past) makes `validate_params` push a `param` diagnostic, so the function returns exit 2 and **the gate never runs** — the `repeated_keys` diagnostic is never produced. The test still gets `.code(2)`, but `diagnostics[0].kind` is now `"param"` (not `"repeated_keys"`) and `node_path` is `"params"` (not `"root.andor[2]"`). The assertions `assert_eq!(d["kind"], "repeated_keys")` and `assert_eq!(d["node_path"], "root.andor[2]")` BREAK. This site is NOT in the spec's coupled-site enumeration. (Note: this is the FOURTH `--after 500000` CLI occurrence; the spec's §7 narrative implies three.)

**Required fix:** Enumerate `cli_build_descriptor.rs:1020` as a coupled site and prescribe changing its `--after` to the chosen future value so `validate_params` passes and the gate's `repeated_keys` path is exercised as intended. (Crisp inventory of ALL `--after 500000`/`500001` sites in the test file: `:82` canon, `:571` existing decay-neg, `:683` mutation, `:1020` repeated-keys — the spec covers `:82`/`:683` and implicitly `:571`, but misses `:1020`. Plus the in-crate `archetype.rs:542` per I1.)

---

## Minor

### M3 — `param_diag` sets `flag: Some(..)`, so the decay diagnostics DO carry provenance — spec's R3/§4 prose is slightly understated (cosmetic)

`param_diag` (`archetype.rs:326-333`) sets `flag: Some(flag.to_string())`, so the new decay diagnostics will render with a `(from --recovery-older)` / `(from --after)` suffix in human mode and a `"flag"` key in `--json`. This is correct and desirable, but the spec's framing ("the deliverable is the new message + flag-naming") slightly undersells it — the flag IS structured, not just named in the message. No change required to the design; consider a one-line note so the implementer's `--json`-shape expectation is pinned (the existing producer-diagnostic `--json` shape already carries `flag`, e.g. `producer_diagnostics_carry_flag_and_human_suffix` @1035 asserts `d["flag"] == "--older"`). Informational.

### M4 — The gate's `MixedTimelock`/`HeightTimelockCombination` guard does NOT overlap the cross-unit decay case — worth one sentence so the implementer doesn't assume redundancy

The gate's mixed-timelock guard (`gate.rs:407`, `has_mixed_timelocks()`) fires only when height+time locks mix in a SINGLE satisfaction branch (NCA `and_v`; see `mixed_timelock_localizes_to_nearest_common_ancestor` @1109). In the decaying tree, tier1's `older(N1)` and tier2's `older(N2)` live in DISTINCT `andor` satisfaction paths (`andor(A,B,C) = or(and(A,B),C)`), so a height-tier1 + time-tier2 cross-unit pair does NOT trip the gate guard. The new predicate-1 is therefore genuinely additive, not redundant with the existing mixed-timelock guard. Recommend the spec add this one-line non-overlap note (it strengthens the "this check is needed" thesis and forestalls an implementer wondering why the gate doesn't already catch it). Informational.

### M5 — Message-quality nit on masked-operand precedence (already safe; documentation only)

When a genuinely-masked operand (bit-31 set, e.g. `0x80_00_0FFF`) reaches `validate_params`, `older_unit_value` reads its low-16/bit-22 and the decay check may fire (or pass), and because `validate_params` precedes the gate, the user may see the DECAY message rather than the more-specific masked-`older()` gate message. This is FAIL-CLOSED in every case (verified: either the decay check refuses, or the decay check passes and the gate independently refuses the masked operand — no masked operand can build), so it is not a funds-safety issue. The spec's §4.1 belt-and-suspenders note is correct. Optional: one line acknowledging the message-precedence (decay-before-mask) so the whole-diff reviewer isn't surprised. Informational.

---

## Verdicts on the prompt's three load-bearing questions

**(a) Is cross-unit REFUSE correct (no legit mixed-unit decaying wallet over-rejected)? — YES, acceptable; the REFUSE-with-`--spec`-escape-hatch is the defensible fail-closed call.** A mixed-unit decaying wallet (e.g. primary `older(144 blocks)` + recovery `older(2 weeks of 512-s units)`) IS expressible and arguably reasonable, so REFUSE is a (small) availability narrowing. But: (i) cross-unit `older` durations cannot be totally ordered offline without baking in a ~10-min block-interval assumption the toolkit must not assume; (ii) the failure mode of NOT refusing is a SILENT funds-loss (recovery quorum spendable before primary), which is strictly worse than an availability regression; (iii) the `--spec` escape hatch is REAL and bypasses `validate_params` entirely (`build_descriptor.rs:321+` does not call it), so a user who genuinely wants mixed-unit tiers can author the node-tree directly — and the spec's refusal message points there. The same-unit ordering predicate (`v2 > v1` in the same unit, unit via `n & 0x0040_0000`, value via `n & 0xFFFF`) is correct, and bit-31-masked operands remain fail-closed (refused by decay OR independently by the gate). **The choice is sound.** (Recommend the refusal message keep its explicit `--spec` pointer — already present in the spec's draft text.)

**(b) Is the D-decay-abs static-floor genuinely monotone-safe? — YES in PRINCIPLE (the design is monotone-safe: only ever false-NEGATIVE, never false-POSITIVE on a legitimately-future locktime), and it DOES catch the reported past-spend bug (`after(500000)` height → refused). The height/time split is correctly handled on both axes (`<500_000_000` ⇒ height floor `900_000`; `≥` ⇒ time floor `1_750_000_000`). BUT the spec's CONCRETE recommended fixture value violates this in practice (C1): `900000000` classifies as time yet is below the time floor ⇒ refused — so the chosen "future" control is actually past.** The floor *values themselves* are conservative-correct: `900_000` is below any plausible future mainnet height yet above all genesis-era heights (mainnet tip ≈ 910k mid-2026, so `900_000` is already-mined and a forward-dated height like `4_000_000` is never refused); `1_750_000_000` (2025-06-15) is already elapsed. The design's monotone-safety claim holds; the bug is purely in the recommended TEST INPUT (C1), which must be corrected to a value that is future under the floors. Once C1 is folded, (b) is satisfied.

**(c) Does the two-predicate decomposition fully cover the decay invariant? — YES, for the fixed 3-tier decaying-multisig shape.** Verified against the registry: ONLY `decaying-multisig` carries both `--recovery-older` AND `--after` (`archetype.rs:121-133`); every other archetype has at most a single `--older`. The shape is fixed at exactly 2 relative `older` tiers + 1 absolute `after` tier (`lower_decaying_multisig` @395-419), so there is no n-tier or alternate-combination gap. Predicate-1 (same-frame rel ordering, tier1↔tier2) + predicate-2 (abs tier3 future-ness) jointly cover the invariant; the cross-frame tier2-`older`↔tier3-`after` pair is genuinely un-orderable offline (different reference frames — relative delay vs absolute moment — without a per-UTXO confirmation assumption), and future-ness is correctly identified as the maximal sound, offline-decidable check. All three timelock params are `required: true`, so the `if let Some` guards never silently skip. The gate's `MixedTimelock` guard does not overlap (M4), confirming predicate-1 is additive. **The decomposition is complete and correctly ratified.**

---

## Other axes (confirmed)

- **No-new-variant / exit-2 path:** CONFIRMED. The decay diagnostics route through the existing `Diagnostic{kind: DiagnosticKind::Param}` (`param_diag` @326) → `emit_diagnostics` → `return Ok(2)` (`build_descriptor.rs:278-280`), fail-closed with a clear provenance-tagged message. Exit 2 is correct for a build-refusal (matches the existing producer/gate refusal convention). No new `enum ToolkitError` variant and no new `DiagnosticKind` variant — the CLAUDE.md alphabetical-`ToolkitError` rule does not apply (R3 correct).
- **SemVer / lockstep / oracle:** CONFIRMED. master = 0.63.0 ⇒ MINOR → 0.64.0 (tightening the producer's accepted input set is behavioural). No new clap flag (reuses `--older`/`--recovery-older`/`--after`), so zero `schema_mirror` flag-name delta, zero GUI/dropdown delta, zero manual flag-row delta, zero `--json` wire-shape change (the `{diagnostics:[…]}` shape is unchanged; only new message strings + new `Param` call-sites). md-codec / mk-codec / ms-codec NO-BUMP (toolkit-local producer logic). The `bitcoind_differential` oracle genuinely never consumes build-descriptor output (0 refs) ⇒ N/A (R6 correct; the producer/unit layer is the decisive gate).
- **`older_unit_value` helper:** JUSTIFIED. `older_consensus_masked` returns `None` for clean operands (proven by `timelock_advisory.rs` tests @249-254), so it cannot classify a clean operand's unit; a tiny `pub fn older_unit_value(n) -> (TimelockUnit, u16)` with inline `n & 0x0040_0000` / `n & 0xFFFF` is the right DRY placement next to the existing bit-math. (R4 correct.)
- **R5 (keep FOLLOWUP `archetype-older-blocks-flag-accepts-time-units` OPEN, narrow resolution):** CORRECT. The broad blocks-only `1..=65535` bound would refuse legitimate 512-s-unit timelocks the gate accepts and would break the deliberate `validate_params_does_not_duplicate_gate_rules` boundary (`archetype.rs:758`); the cross-unit decay refusal resolves the funds-loss facet precisely without touching that boundary. Re-scope, do not flip.

---

## Verdict

**R0 ROUND 1: 1C / 2I** — RED.

Fold C1 (replace every `--after 900000000` recommendation with a value genuinely future under the floors — recommend the height `--after 4000000` consistently across R7/§4.2/§7), I1 (enumerate + update `archetype.rs:542` `fixture_params` `after`; it breaks `validate_params_decay_ordering`'s `len()==1`), and I2 (enumerate + update `cli_build_descriptor.rs:1020`; the past `after` short-circuits before the gate and breaks the `repeated_keys` assertion). The Minors (M3-M5) are optional documentation polish. Re-dispatch after the fold (the reviewer-loop continues after every fold — folds can introduce drift, especially the cross-site fixture-value substitution which must be applied uniformly).
