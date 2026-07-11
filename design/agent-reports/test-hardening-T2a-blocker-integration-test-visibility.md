# T2-a (#6) implementer STOP+report — integration-test visibility blocker + resolution

**Persisted per CLAUDE.md.** Sub-implementer (opus) stopped rather than making a forbidden production change; dispatcher (opus) verified ground truth and resolved. Toolkit @ `master` (working tree clean; scratch probe removed; `mlock.rs` untouched).

## The blocker (implementer finding — VERIFIED by dispatcher)
The R0-GREEN SPEC directed `crates/mnemonic-toolkit/tests/prop_repair_never_wrong.rs` — an **integration test** (separate crate, links only the library surface). But the modules the RED-proofs call are exposed **only under `#[cfg(fuzzing)]`**:

- `crates/mnemonic-toolkit/src/lib.rs:144-190` — `#[cfg(fuzzing)] pub mod repair;` / `#[cfg(fuzzing)] pub mod indel;` (+ every other module). In a normal `cargo test` build these are binary-private (declared as plain `mod repair; mod indel;` in `main.rs:19,25`). `error.rs`/`ToolkitError` likewise pub-under-fuzzing-only — the deliberate "binary-private by design" invariant (lib.rs:140-143).
- `recover_indel_card`, `is_indel_trigger`, `indel_exit_code` are additionally `pub(crate)`.

Implementer proved it with a throwaway probe: `error[E0432]: unresolved import mnemonic_toolkit::repair … note: the item is gated here #[cfg(fuzzing)] (lib.rs:171)`. There is no `[features]` section and no non-fuzzing re-export. So an integration test cannot call any RED-proof target. Every existing repair/indel test drives the compiled binary via `assert_cmd` (`cli_repair.rs`, `cli_mk1_repair_reverify.rs`, `cli_ms1_repair_demote.rs`, `cli_indel.rs`). **The SPEC's premise that `repair_card`/`recover_indel` are callable from a `tests/` file is false against master.** The R0 loop verified the tri-state *semantics* (`repair.rs:439-476`) but not the *visibility from an integration test* — that is the gap.

## Dispatcher ground-truth verification
- lib.rs:144-190 confirmed: all `pub mod` gated `#[cfg(fuzzing)]`; `main.rs:3-41` declares them as private `mod …`.
- **`indel.rs:302` and `repair.rs:1910` are already `#[cfg(test)]` in-crate unit modules** — the repo ALREADY unit-tests exactly these modules in-crate (e.g. `recover_indel_reports_ambiguous_on_multiple_distinct_recovered`, indel.rs:357-374, uses a mock `AcceptAll` oracle + a direct `recover_indel` call — CLI-unreachable, since an Ambiguous indel outcome is ~2⁻⁶⁵ with real vectors per `cmd/repair.rs:455-459`).
- proptest dev-dep present (`Cargo.toml:76`).

## Resolution — Option A′ (in-crate `#[cfg(test)]` unit module), supersedes the implementer's A/B
Implementer offered **A** (CLI-driven `assert_cmd` harness — loses the indel-ambiguity RED-proof, slow per-case process spawn) and **B** (`pub mod repair; pub mod indel;` — a published-API change forbidden by Acceptance §5 / NO-BUMP / binary-private invariant). Dispatcher chose a THIRD, strictly-superior option the implementer did not surface:

**A′: relocate the harness to an in-crate `#[cfg(test)]` unit module** (`src/prop_repair_never_wrong.rs`, declared `#[cfg(test)] mod prop_repair_never_wrong;` in `main.rs`). Rationale:
- Direct private access to `crate::repair::{repair_card, SetVerify, RepairError, CardKind}` + `crate::indel::recover_indel` (+ mock-oracle ambiguity path) — **every** SPEC RED-proof reachable, including the indel-ambiguity fold that CLI cannot reach.
- TEST-only, zero production/lib-surface change → **NO-BUMP preserved** (nothing added to the `#[cfg(fuzzing)]` lib block or shipped binary; `#[cfg(test)]` compiles only under `cargo test`).
- Consistent with repo convention (indel.rs:302 / repair.rs:1910 already do this).
- Direct calls, no per-case subprocess → fast CI (a CLI-driven proptest would spawn `mnemonic` per case).

Deviation from the R0-GREEN SPEC is **harness LOCATION only** (`tests/` integration → in-crate `#[cfg(test)]` unit) — all oracles, constructed/pinned F4 vectors, and named mutations are unchanged and coverage is strengthened (ambiguity RED-proof recovered). SPEC T2-a folded accordingly; scoped R0 re-bless dispatched before re-tasking the implementer. #7 (md) and #8 (mk) are library crates with a public decode surface → unaffected by this binary-crate-only blocker.
