# Phase 1 review — descriptor-builder presets — round 2
**Verdict: GREEN** (0 Critical / 0 Important)

## Round-1 fold verification

**I1 — RESOLVED.** `preset_negative_discrimination_mutated_param_breaks_golden` (`crates/mnemonic-toolkit/tests/cli_build_descriptor.rs:463-488`) now loops a 5-tuple table — one mutated numeric param per archetype, exactly SPEC §7's clause ("mutate one param (a threshold or timelock) **per archetype**", SPEC:174) and exactly the r1 prescription. Each mutation verified on three axes:

| Archetype | Mutation | ≠ fixture value | Gate-valid | Read by lower fn (discriminates hardcode) |
|---|---|---|---|---|
| simple-timelocked-inheritance | `--older` 65535→65534 | ✓ (preset_args :43) | ✓ (older bound 1..2³¹, gate.rs:175) | ✓ `params.older` at archetype.rs:412 |
| decaying-multisig | `--after` 500000→500001 | ✓ (:56) | ✓ (after ≥1, gate.rs:181; still height-domain) | ✓ `params.after` at archetype.rs:351 |
| kofn-recovery | `--threshold` 2→3 (3 keys) | ✓ (:66) | ✓ k=n=3 — probed: exit 0, `multi(3,…)` | ✓ `params.threshold` at archetype.rs:394 |
| tiered-recovery | `--older` 4032→4033 | ✓ (:77) | ✓ | ✓ `params.older` at archetype.rs:436 |
| hashlock-gated | `--older` 144→145 | ✓ (:90) | ✓ | ✓ `params.older` at archetype.rs:384 |

The discrimination argument holds for every row: a lower fn hardcoding the fixture value renders the mutated run byte-identical to the golden → `assert_ne` at :486 fails; a gate-invalid mutation fails the `.success()` at :484. Both arms are live (suite green proves all 5 mutations pass the gate today). The position-based splice (:474-478) is sound: each mutated flag occurs exactly once in its `preset_args` (exact-string `==`, so `--older` cannot match `--recovery-older`; decaying deliberately uses the unique `--after`), and every flag is value-followed so `argv[i+1]` is in-bounds. Failure messages carry the archetype name — good diagnosability.

**M1 — RESOLVED** as prescribed: §9 Phase 1 now reads "the 9 value-param flags (the 10th `requires`-carrying flag, `--emit-spec`, is Phase 2 — P1-r1 M1 errata)" (`design/SPEC_descriptor_builder_presets.md:188`). Cross-checked that §1's "All 10 param flags carry clap-level `requires`" sentence (SPEC:42) correctly stands unedited — its "10" counts `--emit-spec` (SPEC:36 gives it `requires = "archetype"`), so the only internal contradiction was the §9 one, now gone.

**M6 — RESOLVED.** The dead `augment_args` line and its misleading comment are gone from drift test (b); `ClapArgs` dropped from the import (`build_descriptor.rs:362`, test body :384-405 now realizes the surface solely via `Probe::command()`). Clippy with `-D warnings` confirms the import trim left nothing unused.

**M7 — RESOLVED.** `descriptor_builder/mod.rs:1-11` rewritten: the phantom "main.rs `#![allow(dead_code)]`" reference is gone; the new text accurately describes Release A (ir/schema/gate/clap surface, v0.50.0 — matches Cargo.toml:3) and Release B's `archetype` addition with the presets-SPEC pointer.

**M2-M5 — carry-forward ACKNOWLEDGED.** The SPEC fold-log entry (SPEC_descriptor_builder_presets.md:219) records all four accurately against the r1 text: M2 `keys[i]` prefix-semantics resolver case, M3 decaying intra-`andor[2]` `flag: None` cell, M4 clap-rejects-scalar-repeats note in the test file, M5 success-path `--json`/`--network` cells. Phase 2's reviewer has what it needs.

**No fold-drift.** The fold commit touches exactly 5 files; the only src changes are the M6/M7 deletions-plus-doc, no behavior change; fixtures untouched (still last modified `3085330`); the persisted r1 review file matches the round-1 deliverable verbatim.

## Critical

None.

## Important

None.

## Minor

**M8 (observation, no action required) — one-param-per-archetype is the SPEC's mandate and r1's prescription, both now met; the remaining numeric params (decaying's `--older`/`--recovery-older`/both thresholds, kofn's `--older`, tiered's thresholds) stay individually non-vacuity-unpinned.** SPEC §7 asks for exactly one per archetype, so this is conforming, not a gap. If Phase 2 ever touches the table, extending rows is one tuple each — note only, not a carry-forward obligation.

## Empirical probes run

1. `cargo test -p mnemonic-toolkit --test cli_build_descriptor` → **22 passed, 0 failed** (count unchanged — the loop replaced the single cell in place; `preset_negative_discrimination_mutated_param_breaks_golden` listed ok).
2. `cargo test -p mnemonic-toolkit --bin mnemonic` → **937 passed, 2 ignored** — identical to round 1; the fold added no bin-crate cells and broke none.
3. `cargo clippy -p mnemonic-toolkit --all-targets` → clean; then forced a genuine re-lint with `-- -D warnings` (fingerprint invalidation: "Checking mnemonic-toolkit v0.50.0", 8.68s) → **finished, zero diagnostics**. The M6 import trim is clippy-clean.
4. Manual kofn mutation probe (`--threshold 3` with 3 keys, `--format descriptor`) → exit 0, output begins `wsh(or_d(multi(3,…` — gate-valid AND visibly different from the `multi(2,…)` golden, confirming the most semantically interesting row (k=n boundary) discriminates rather than refuses.
5. Per-row hardcode-hypothesis check (static, against current source): all 5 mutated flags map to `req(params.<x>, …)` reads inside the matching lower fn (archetype.rs:351, :384, :394, :412, :436) — each row kills the corresponding hardcode mutant.
6. `git show d0967e6 --stat` → 5 files, test/doc/SPEC only; no `archetype.rs`, no fixtures, no gate/ir/schema source touched — consistent with r1's closing note that no source changes were required.

Phase 1 is at 0C/0I. Gate satisfied — proceed to Phase 2 (`--emit-spec`, provenance `flag` field, schema archetypes section, manual) with the four recorded carry-forwards.
