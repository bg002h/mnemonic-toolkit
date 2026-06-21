# IMPLEMENTATION PLAN — cycle-6 — timelock decaying-multisig decay-ordering

Phased TDD execution plan for the R0-GREEN brainstorm spec
(`design/BRAINSTORM_cycle6_timelock_decay.md`, **0C/0I** at spec-R0 round 2 —
`design/agent-reports/cycle6-spec-r0-round{1,2}-review.md`). DESIGN ONLY — feeds the
mandatory plan-doc **R0 loop to 0C/0I BEFORE any code**. Toolkit-only, MINOR; no
registry publish; **no clap/`--json`/dropdown change → no GUI schema_mirror, no manual
leg, no codec change.** The bitcoind differential oracle is N/A (never consumes
build-descriptor output).

## Source-of-truth SHA
`/scratch/code/shibboleth/mnemonic-toolkit`, `origin/master` **`3fa2925b`**, toolkit
**0.63.0 → 0.64.0** (MINOR). (0.63.0 was cycle-5; 0.64.0 supersedes the paused
own-account cycle's stale 0.63.0 plan — don't touch that branch.)

**Execution model:** single implementer in a toolkit worktree off `origin/master`,
strict TDD (RED before GREEN), **FULL `cargo test -p mnemonic-toolkit`** at each phase
gate + `cargo clippy --all-targets -D warnings`. Re-grep every line (cite `3fa2925b`).
**NEVER `cargo fmt`** (mlock.rs fmt-exempt). The funds bug: a decaying multisig where a
recovery/last-resort tier unlocks BEFORE the primary → premature spend.

---

## Phase 1 — the `older_unit_value` helper + static abs-floor constants

**Why a new helper (spec protocol-correction):** `timelock_advisory::older_consensus_masked`
returns `None` for CLEAN (unmasked) operands → it can NOT classify a clean operand's
unit. Add `timelock_advisory::older_unit_value(n: u32) -> (TimelockUnit, u16)`:
unit = `if n & 0x0040_0000 != 0 { Seconds512 } else { Blocks }`; value = `(n & 0xFFFF) as u16`.
(Bit-31 disable / the >16-bit-value cases are already refused upstream by the existing
`older_consensus_masked` masked-operand gate — confirm; the new predicate runs on the
clean operands that gate lets through.)

**Static abs-floor constants** (D-decay-abs, BIP-65 height/time split at the literal
`500_000_000` — **plan-R0 M5: `cost/enumerate.rs:74` is a DOC COMMENT, not a named const;
hardcode the `500_000_000` literal inline** as the spec §4.2 code and `gate.rs:1041`
already do): `ABS_HEIGHT_PAST_FLOOR` (~`900_000`) and `ABS_TIME_PAST_FLOOR`
(~`1_750_000_000`, 2025-06-15) — conservative PAST floors so the check is **monotone-safe**
(only ever false-NEGATIVE on a borderline-recent locktime, never false-POSITIVE on a
legitimately-future one). Values + rationale per spec §4.

**RED tests (write first):** `older_unit_value` unit tests (`145 → (Blocks,145)`;
`4194305 → (Seconds512,1)`; `4000000 → (Blocks, …)`). **GREEN:** add the helper + consts.
**Gate:** package test + clippy.

---

## Phase 2 — the two decay predicates in `validate_params` (`descriptor_builder/archetype.rs:306`)

Both predicates extend the existing `validate_params` decay block (currently the raw
`recovery_older <= older` compare at `:306-317`) and push `Diagnostic{kind: Param}`
(exit 2 — **NO new `ToolkitError` variant, NO new `DiagnosticKind`**; reuse the existing
`param_diag` path).

**(a) D-decay-rel — cross-unit refuse + same-unit strict order.** Replace the raw
`recovery_older <= older` compare: extract `(u_p, v_p) = older_unit_value(older)` and
`(u_r, v_r) = older_unit_value(recovery_older)`. If `u_p != u_r` → push a Param diag
("cross-unit `--older`/`--recovery-older` not orderable offline; express both in the
same unit"). Else if `v_r <= v_p` → push the existing "recovery must be > primary" diag.
(Masked operands stay fail-closed via the existing upstream gate.)

**(b) D-decay-abs — static future-ness.** For `params.after` (Some): classify via the
`500_000_000` split; if height (`< 500_000_000`) and `after < ABS_HEIGHT_PAST_FLOOR` →
Param diag ("absolute `after(N)` is a past block height → last-resort key immediately
spendable"); if time (`>= 500_000_000`) and `after < ABS_TIME_PAST_FLOOR` → the
time-variant diag. Future values are silent (monotone-safe). **plan-R0 M1: use STRICT `<`
against the floors** (pinned to the GREEN spec; fail-closed either way, but match the spec).

**RED tests (write first) — the two GENUINE RED-first negatives** (exit-0-accepted today,
verified): `--older 145`(blocks) + `--recovery-older 4194305`(512-sec) → **now exit 2**
(cross-unit); `--after 500000`(past height) → **now exit 2** (abs). **plan-R0 M3:** the
same-unit mis-order (`--older 2000 --recovery-older 1000`) is ALREADY refused today (raw
`1000 <= 2000`) → keep it as a **regression guard** (must STAY exit 2), NOT a RED-first
test.
**Positive controls (MUST stay/build green):** same-unit correctly-ordered (`--older
1000 --recovery-older 2000`) + `--after 4000000`(future) → builds; the existing
same-unit `2000/2000`... (per spec) stays green.

**Gate:** package test + clippy. NOTE Phase 3's coupled-site migration must land
together or the canon-golden tests go RED (the canon uses a past `after(500000)`).

---

## Phase 3 — coupled-site migration (the canon golden cascade + CLI tests)

The new abs-future check fires on the canon's past `after(500000)` → migrate the canon
to the future `after(4000000)`. Per the spec §7 table — **9 MIGRATE sites** (re-grep each
against `3fa2925b`; the spec lists live lines):
1. `descriptor_builder/archetype.rs:33` — doc comment `after(500000)` → `after(4000000)`.
2. `descriptor_builder/archetype.rs:542` — `fixture_params` `after: Some(500000)` → `Some(4000000)` (keeps the in-crate `validate_params_decay_ordering` `:743` `diags.len()` count — a FUTURE after adds NO abs diagnostic).
3. `descriptor_builder/mod.rs:66` — canon descriptor STRING golden `after(500000)` → `after(4000000)`.
4. `tests/fixtures/descriptor_builder/decaying-multisig.json` — `"after": 500000` → `4000000`.
5. `tests/fixtures/descriptor_builder/decaying-multisig.descriptor` — `after(500000)` → `after(4000000)` **AND REGENERATE the BIP-380 checksum** (`#llvl05j9` → new; a stale checksum fails parse — use `md`/the in-tree checksum engine).
6. `tests/fixtures/descriptor_builder/decaying-multisig.bip388` — the template `after(500000)` → `after(4000000)`.
7. `cli_build_descriptor.rs:81-82` — canon CLI `--after 500000` → `4000000`.
8. `cli_build_descriptor.rs:570-571` — canon-cleanliness `--after` → `4000000`.
9. `cli_build_descriptor.rs:683` — the mutation `("decaying-multisig","--after","500001")` → `"4000001"` (so it still tests its INTENDED mutation, not the new abs reject).
10. `cli_build_descriptor.rs:1019-1020` — the `repeated_keys` test `--after 500000` → `4000000` (so `validate_params` does NOT short-circuit exit 2 before the gate's `repeated_keys` path; otherwise the `kind=="repeated_keys"`/`node_path` assertions never run).

**UNAFFECTED (leave; verified non-`validate_params`):** `descriptor_builder/ir.rs:390`
(pure `After(500000).render()` unit test); `cli_compare_cost.rs:1123` (`after(1500000000)`
raw `compare-cost --miniscript` policy string, no archetype path); `specter-descriptor-
with-checksum.json` `"blockheight": 500000` (wpkh single-sig wallet-import metadata, no
`after()`).

**Gate:** FULL package test green (the canon goldens now consistent at `4000000`) + clippy.

---

## Phase 4 — full suite + ship (0.64.0)

**Zero-regression proof:** FULL `cargo test -p mnemonic-toolkit` green — every migrated
golden consistent, the new RED tests green, the existing decay/same-unit tests green.

**Version sites (release ritual `project_toolkit_release_ritual_version_sites`):**
toolkit `0.63.0 → 0.64.0` (Cargo.toml + BOTH READMEs + `scripts/install.sh` self-pin +
`fuzz/Cargo.lock` + CHANGELOG). (plan-R0 M4: root `Cargo.lock` toolkit entry self-heals
on the Phase-4 build — no manual edit needed, but verify it lands at 0.64.0.) CHANGELOG: **MINOR — decaying-multisig decay-ordering
fail-closed validation (cycle-6).** `validate_params` now refuses a cross-unit
`--older`/`--recovery-older` pair (un-orderable offline) and a non-strict same-unit
order (D-decay-rel: recovery tier could unlock before primary), and refuses an absolute
`after(N)` that is a PAST height/time (D-decay-abs: last-resort key immediately
spendable) — both previously silently mis-built a wrong spending policy. New
`older_unit_value` helper + static BIP-65 past-floors; routed through the existing
`Diagnostic{Param}` (exit 2). No new flag/wire/variant. Canon decaying-multisig fixtures
migrated `after(500000)`→`after(4000000)`. Closes D-decay-rel + D-decay-abs; re-scopes
FOLLOWUP `archetype-older-blocks-flag-accepts-time-units` (funds facet resolved).

**FOLLOWUP / report:** tick **D-decay-rel** + **D-decay-abs** `[ ]`→`[x]` in
`design/agent-reports/constellation-bughunt-2026-06-20.md` with the fixing commit; flip
the re-scoped FOLLOWUP.

---

## Mandatory post-implementation gate
After Phase 3 GREEN, before ship: a **mandatory independent adversarial whole-diff
review** — primary targets: (1) the cross-unit refuse + same-unit strict-order predicate
is correct (no mis-order slips, no legit same-unit wallet over-rejected); (2) the
abs static-floor is monotone-safe (catches past-`after`, never refuses a legit future);
(3) the canon-golden cascade is consistent (descriptor checksum valid; no orphaned
`500000`); (4) the in-crate `diags.len()` + the `repeated_keys` CLI assertions still
exercise their intended paths. Persist to `design/agent-reports/`. Ship only after GREEN.

## Phase order
P1 (helper+consts) → P2 (predicates) + P3 (migration) land together (the canon migration
must accompany the predicate or goldens go RED) → P4 (ship). Plan-R0 must converge 0C/0I
before any P1 code. Work in a worktree off `origin/master`; design-trail + report ticks
via a master worktree; do NOT commit on the paused branch.
