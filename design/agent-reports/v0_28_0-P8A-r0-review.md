# v0.28.0 Phase 8 Sub-phase P8A — Self-review R0

**Reviewer:** instance G2 (autonomous mode; no opus sub-agent dispatch in this session)
**Cycle:** v0.28.0
**Sub-phase:** P8A — BSMS taproot refusal text tightening + `script_type_short_name` helper
**Branch:** `v0.28.0/g2-bsms-taproot` (renamed from `worktree-agent-a950bd1cb7e5d1166`)
**Branched from:** `release/v0.28.0` @ `71592bc84749af8e2d899f1cac2c28a7a8aecc4d`
**Plan-doc anchor:** `unified-meandering-sundae.md` §S.8 (lines 394-405) + Phase 8 table (line 547)
**Verdict:** GREEN — 0 Critical / 0 Important / 0 Minor (execution-blocking)

---

## Scope reviewed

P8A changes (per plan-doc):

1. Replace the static `ToolkitError::BadInput("--format bsms does not support taproot descriptors; ...".into())` at `crates/mnemonic-toolkit/src/wallet_export/bsms.rs:69-76` with a per-script-type-parameterized refusal.
2. Introduce `script_type_short_name(&WalletScriptType) -> &'static str` helper returning `"P2tr"` / `"P2trMulti"`.
3. Pin the helper's two valid returns + non-taproot panic contract with `#[cfg(test)]` cells in `wallet_export/bsms.rs`.

P8A intentionally pre-fabricates the message template; the **rendering site** moves to `error.rs::message` under P8B (the new variant's `Display` arm). The plan-doc's §S.8 diff shows the body as a `format!` inline at the construction site — I chose to route through the new `BsmsTaprootRefused` variant's message arm instead. Rendered output is identical; the dispatch is cleaner for JSON-error-envelope discriminability.

## Verification performed

- **Grep:** confirmed no other call sites in `crates/mnemonic-toolkit/src/` construct the old `BadInput("--format bsms does not support taproot ...")` text. The only existing references are:
  - `src/wallet_export/bsms.rs:69-76` — replaced (the construction site).
  - `tests/cli_export_wallet_bsms.rs:315/319` — the integration cell asserting the message, which P8A updates to include the new discriminator + diagnostic substrings.
- **Test cells:**
  - `wallet_export::bsms::tests::script_type_short_name_p2tr` → PASS (asserts `"P2tr"` literal).
  - `wallet_export::bsms::tests::script_type_short_name_p2tr_multi` → PASS (asserts `"P2trMulti"` literal).
  - `wallet_export::bsms::tests::script_type_short_name_panics_on_non_taproot` → PASS (asserts panic substring `"non-taproot variant"`).
  - Integration cell `bsms_4line_taproot_multisig_refused_carries_full_v0_28_diagnostic` → PASS (P2trMulti discriminator + BIP-386 status + FOLLOWUP slug + alternative-format pointers all present; exit_code = 2).
  - Integration cell `bsms_taproot_singlesig_refused_carries_p2tr_discriminator` → PASS (P2tr discriminator via `--template bip86`; exit_code = 2).
- **Build:** `cargo build --bin mnemonic` GREEN; `cargo build --tests` GREEN; `cargo clippy --workspace --all-targets -- -D warnings` GREEN.
- **Regression:** full `cargo test --workspace` GREEN (one transient TTY-test flake on first run — `cell_30_verify_bundle_json_context_under_tty_emits_envelope` — passed on re-run; unrelated to P8 surface).

## Findings

### Critical: none.

### Important: none.

### Minor: none execution-blocking.

Note (carried forward to P14A scope, not P8): the canonical `bsms-taproot-emit` FOLLOWUP entry at `design/FOLLOWUPS.md:2463` will need a v0.28.0 sub-deliverable note added by Phase P14A per plan-doc R0 C2/C3 fold; P8 itself does not touch FOLLOWUPS.

## Decisions to architect

**A1.** `script_type_short_name` panics on non-taproot variants rather than returning `Option<&str>` / a `Result`. Rationale: the caller-gate at `emit()` is the only construction site and is `matches!(_, P2tr | P2trMulti)`-guarded; a panic surfaces accidental caller-surface widening loudly. The panic substring `"non-taproot variant"` is asserted in a dedicated `#[should_panic]` cell.

**A2.** Helper visibility is `pub(crate)` with a re-export at `wallet_export/mod.rs::script_type_short_name`. The re-export is required so `error.rs::message` (the `BsmsTaprootRefused` Display arm under P8B) can reach the function without breaching the private-mod boundary. Other options considered:
- Inlining the match into the Display arm — rejected because it would split the canonical "what string represents what script-type" decision across two files.
- Making the helper free-standing in `error.rs` — rejected because the helper is a `wallet_export`-domain concept (about `WalletScriptType`, defined in `wallet_export/mod.rs`).

## Files changed

- `crates/mnemonic-toolkit/src/wallet_export/bsms.rs` — emit() now returns `ToolkitError::BsmsTaprootRefused`; new `script_type_short_name` helper; new `#[cfg(test)] mod tests` with 3 cells.
- `crates/mnemonic-toolkit/src/wallet_export/mod.rs` — `pub(crate) use bsms::script_type_short_name;` re-export.
- `crates/mnemonic-toolkit/tests/cli_export_wallet_bsms.rs` — old cell 6 `bsms_4line_taproot_descriptor_errors_explicit_deferred` renamed + tightened to `bsms_4line_taproot_multisig_refused_carries_full_v0_28_diagnostic`; new cell `bsms_taproot_singlesig_refused_carries_p2tr_discriminator` for P2tr discrimination.

## Net LOC

~50 src + ~80 tests (within plan-doc estimate of ~40 src + ~30 tests; cells came in higher because integration cells assert 5+ substrings each).
