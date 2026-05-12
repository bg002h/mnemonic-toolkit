# Session handoff — `export-wallet-v0.8` execution, begin at Phase 0

| Field | Value |
|---|---|
| Created | 2026-05-11 |
| Pause point | Plan approved; no Phase 0 work started yet |
| Resume target | Phase 0 (promote inline SPEC + IMPLEMENTATION_PLAN to `design/`, run reviewer loops, then module reorganization) |
| Predecessor cut | `mnemonic-toolkit-v0.7.0` shipped; v0.8 series in flight (taproot-internal-key + electrum-version-info-stderr already resolved). |
| Plan file | `/home/bcg/.claude/plans/we-need-to-make-recursive-pnueli.md` (approved post-R1, 0C/0I) |

## Read these first (in order)

1. **`/home/bcg/.claude/plans/we-need-to-make-recursive-pnueli.md`** — the approved plan. Parts A (SPEC), B (IMPLEMENTATION_PLAN), C (iterative-review log). Read end-to-end before touching code.
2. **`design/SPEC_export_wallet_v0_7.md`** — v0.7 baseline this cycle extends. The new SPEC (Part A) mirrors its §-numbered style; preserve that style when promoting.
3. **`design/IMPLEMENTATION_PLAN_v0_7.md`** — v0.7 phase-style precedent. The new IMPLEMENTATION_PLAN (Part B) mirrors its TDD + phase + reviewer-loop discipline.
4. **`CLAUDE.md`** — project-wide conventions (per-phase TDD before impl, per-phase opus reviewer-loop until 0C/0I, manual-mirror invariant gated by `tests/lint.sh flag-coverage`, stage paths explicitly).
5. **`design/FOLLOWUPS.md`** lines around 857 / 899 / 908 — entries this cycle touches.

## Approved scope (locked, do not re-litigate)

1. **Integration:** extend the existing `mnemonic export-wallet` subcommand (NOT a new subcommand).
2. **Coverage:** six new formats — `coldcard`, `jade`, `sparrow`, `specter`, `electrum`, `green` — alongside existing `bitcoin-core` and `bip388`.
3. **Missing-info UX:** byte-exact refusal that lists ALL missing fields in one message, deterministically ordered (global discriminant first, then per-slot). Exit 2. Never partial JSON.
4. **Phases (5):** (1) coldcard + jade together — Jade multisig text is byte-identical to Coldcard's; (2) sparrow; (3) specter; (4) electrum — SLIP-132 round-tripping is heaviest piece, deferred to last; (5) green — thin descriptor-text emitter.
5. **Version:** v0.8 series. Artifact names: `design/SPEC_export_wallet_v0_8.md`, `design/IMPLEMENTATION_PLAN_v0_8_export_wallet_expansion.md`.

## R1 resolutions already folded (do not re-discover)

- **Electrum seed_version** is **NOT** pinned to Coldcard's stale sample value of 17. Phase 4 step 0 is a **spike** against current Electrum (>= 4.5.x) to observe the value Electrum writes for watch-only wallets; `ELECTRUM_SEED_VERSION_PIN` is locked from the spike report.
- **WalletFormatEmitter trait returns `Result<String, ToolkitError>`** (not `Vec<u8>`). Phase 0 thin-wraps existing `format_bitcoin_core_importdescriptors` / `format_bip388_wallet_policy` (which return `serde_json::Value`) via `serde_json::to_string_pretty`. The `cmd::export_wallet::run` call-site loses its own `to_string_pretty` invocation.
- **`WalletScriptType`** is a NEW enum local to `crate::wallet_export` covering single + multisig. `crate::cmd::convert::ScriptType` stays untouched (single-sig-only, scoped to `(Xpub, Address)` edge).
- **Coldcard generic JSON does NOT support `bip86`** (not in upstream schema). `--template bip86 --format coldcard` REFUSES with byte-exact pointer text; new FOLLOWUPS `coldcard-bip86-generic-export-pending-firmware`.
- **Sparrow/Specter stub arms stay alive until their phase replaces them**. Phase 1 does NOT delete them; Phase 2 deletes Sparrow stub; Phase 3 deletes Specter stub. v0.7 refusal tests pass through every phase.
- **`--wallet-name` is REQUIRED for `--format specter`** (Specter UX requires a label). Phase 3 pins `specter_missing_wallet_name_refusal.stderr`.
- **No mk-codec `Companion:` cross-cite** — `crate::slip0132` doesn't cross the codec boundary.

See plan file Part C for the full R1 finding-by-finding log.

## Phase 0 — concrete first steps (resume here)

Pre-code (artifact promotion + reviewer loops):

1. Copy plan Part A → **new file** `design/SPEC_export_wallet_v0_8.md`. Adapt header block to mirror `SPEC_export_wallet_v0_7.md`:

   ```
   # mnemonic-toolkit v0.8 SPEC — `export-wallet` multi-format expansion

   **Version:** 0.8.0 (extension)
   **Date:** 2026-05-XX (fill at promotion)
   **Status:** DRAFT (post-plan-R1; awaiting SPEC-level reviewer-loop)
   **Predecessors:** [SPEC_export_wallet_v0_7.md](SPEC_export_wallet_v0_7.md)
   ```

   Strip the plan's framing prose; keep §1–§14 verbatim.

2. Copy plan Part B → **new file** `design/IMPLEMENTATION_PLAN_v0_8_export_wallet_expansion.md`. Adapt header to mirror `IMPLEMENTATION_PLAN_v0_7.md`. Append plan Part C verbatim as the `## Iterative-review log` section.

3. Run **opus reviewer-loop** on `SPEC_export_wallet_v0_8.md`. Reviewer prompt model: same shape as the in-plan R1 review (find Critical / Important / Low / Nit; check against authoritative vendor format URLs; verify file:line refs against current source). Persist report to `design/agent-reports/v0_8-spec-r1.md`. Iterate to 0C/0I. Fold resolutions inline; record in the SPEC's own log section.

4. Run opus reviewer-loop on `IMPLEMENTATION_PLAN_v0_8_export_wallet_expansion.md`. Persist `design/agent-reports/v0_8-impl-plan-r1.md`. Iterate to 0C/0I.

5. **STOP and check with user** before any code changes. The user is the gate for moving from artifact-promotion to module-reorganization.

Code (module reorganization, after user approval to proceed):

6. Split `src/wallet_export.rs` → `src/wallet_export/` submodule tree per SPEC §12. Move `format_bitcoin_core_importdescriptors` → `bitcoin_core.rs`; `format_bip388_wallet_policy` → `bip388.rs`; descriptor pipeline → `pipeline.rs`; `format_stub_message` → `mod.rs`. Thin-wrap moved functions as `WalletFormatEmitter::emit -> Result<String, ToolkitError>` impls.
7. Update `cmd::export_wallet::run` call-site to the trait-dispatch pattern (per SPEC §12 dispatch snippet). Delete own `to_string_pretty` call.
8. Add `WalletScriptType` enum + `script_type_from_template` + `script_type_from_descriptor` to `wallet_export/mod.rs`.
9. Add `MissingField` enum + `build_missing_fields_refusal` + `ToolkitError::ExportWalletMissingFields { format, missing }` variant + `user_text()` arm in `src/error.rs`.
10. Zero new behavior tests in Phase 0. Existing v0.7 test suite stays GREEN.

Phase 0 exit gate:

```fish
cd /scratch/code/shibboleth/mnemonic-toolkit
cargo build --workspace
cargo test --workspace --no-fail-fast
cargo clippy --workspace --all-targets -- -D warnings
bash tests/lint.sh
```

All GREEN; manual mirror untouched (no new flags yet); zero new tests added.

## Subsequent phases (one-line each)

- **Phase 1 — Coldcard + Jade.** RED fixtures first (`coldcard_generic_bip{44,49,84}_*.json`, `coldcard_multisig_2of3_wsh.txt`, byte-equal `jade_multisig_2of3_wsh.txt`, refusal fixtures). Then `coldcard.rs` + `jade.rs`. Wire `CliExportFormat::{Coldcard,Jade}`. Manual mirror + lint.sh flag-coverage. Reviewer loop → `design/agent-reports/v0_8-phase-1-coldcard-jade-r{N}.md`. FOLLOWUPS: add `coldcard-tr-multi-a-pending-firmware`, `coldcard-bip86-generic-export-pending-firmware`, `jade-tr-multi-a-pending-firmware`.
- **Phase 2 — Sparrow.** Fixtures, impl, delete v0.7 Sparrow stub arm, manual mirror, reviewer loop.
- **Phase 3 — Specter.** Fixtures (including `specter_missing_wallet_name_refusal.stderr`), impl, delete v0.7 Specter stub arm (stub-arm block fully gone after this phase), manual mirror, reviewer loop.
- **Phase 4 — Electrum.** Spike first (step 0 — observe Electrum's seed_version for watch-only). Then fixtures pinned to spike-observed shape (NOT Coldcard's stale samples). SLIP-132 round-trip test. Manual mirror, reviewer loop. FOLLOWUPS: `electrum-final-seed-version-drift`, `electrum-tr-multi-a-pending-libsecp-taproot`.
- **Phase 5 — Green.** Thin 3-line descriptor-text emitter. Multisig refuses. Manual mirror, reviewer loop. FOLLOWUPS: `green-native-multisig-pending-server-support`. Flip `wallet-export-industry-formats` to fully RESOLVED.
- **Phase 6 — Release roll-up.** Smoke tests, CHANGELOG `[0.8.X]` entry, manual workflow page for Coldcard multisig text, tag `mnemonic-toolkit-v0.8.X`.

## Critical files (cheat sheet)

To modify:
- `crates/mnemonic-toolkit/src/wallet_export.rs` → split into `crates/mnemonic-toolkit/src/wallet_export/` (Phase 0)
- `crates/mnemonic-toolkit/src/cmd/export_wallet.rs:148-155` (stub arm site; deleted incrementally Phase 2 + Phase 3)
- `crates/mnemonic-toolkit/src/error.rs` (new `ExportWalletMissingFields` variant + arm; delete `ExportWalletFormatStub` once Phase 3 closes)
- `crates/mnemonic-toolkit/src/slip0132.rs` (add `variant_for` helper if not present)
- `docs/manual/src/40-cli-reference/41-mnemonic.md` (mirror invariant — one update per phase)
- `design/FOLLOWUPS.md` (per-phase entries)

To create:
- `design/SPEC_export_wallet_v0_8.md`
- `design/IMPLEMENTATION_PLAN_v0_8_export_wallet_expansion.md`
- `crates/mnemonic-toolkit/src/wallet_export/{mod,pipeline,bitcoin_core,bip388,coldcard,jade,sparrow,specter,electrum,green}.rs`
- `tests/export_wallet/*` byte-exact fixtures (~13 files per plan §13)
- `tests/helpers/coldcard_parse.rs`
- `design/agent-reports/v0_8-{spec,impl-plan,phase-1-coldcard-jade,phase-2-sparrow,phase-3-specter,phase-4-electrum,phase-5-green}-r{N}.md`

## User directives carried into this cycle

- **Plan artifact must embed inline SPEC + IMPLEMENTATION_PLAN AND go through reviewer-loop before approval** — see memory `feedback-plan-artifact-mirror-project-convention`. The plan file already satisfies this; subsequent SPEC/IMPLEMENTATION_PLAN promotions each get their own reviewer-loop.
- **CLAUDE.md mirror invariant** — any flag add/remove updates `docs/manual/src/40-cli-reference/41-mnemonic.md` in the same PR; CI gate is `tests/lint.sh flag-coverage`.
- **Cross-repo follow-ups** — if any toolkit work surfaces an action item affecting a sibling codec, mirror entries in BOTH repos' `design/FOLLOWUPS.md` with cross-citing `Companion:` lines. None anticipated for this cycle.
- **Stage paths explicitly** — never `git add -A`.

## Open FOLLOWUPS this cycle will introduce

- `coldcard-tr-multi-a-pending-firmware` (Phase 1)
- `coldcard-bip86-generic-export-pending-firmware` (Phase 1)
- `jade-tr-multi-a-pending-firmware` (Phase 1)
- `electrum-final-seed-version-drift` (Phase 4)
- `electrum-tr-multi-a-pending-libsecp-taproot` (Phase 4)
- `green-native-multisig-pending-server-support` (Phase 5)

## Decision gate before resuming code work

Before Phase 0 step 6 (module reorganization), confirm with the user:
- Are `SPEC_export_wallet_v0_8.md` and `IMPLEMENTATION_PLAN_v0_8_export_wallet_expansion.md` reviewer-loop-converged (0C/0I)?
- Has the user reviewed and approved both promoted artifacts?
- Should we open a feature branch (e.g., `export-wallet-v0.8`) before any code change, or commit directly to master per recent project pattern?

When in doubt, ASK before editing code.
