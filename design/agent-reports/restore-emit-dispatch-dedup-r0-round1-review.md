# R0 Architect Review (round 1) — `SPEC_restore_emit_dispatch_dedup.md`

**Reviewer:** opus `feature-dev:code-reviewer` (mandatory pre-implementation R0 gate). **Date:** 2026-06-05.
**Branch:** `restore-emit-dispatch-dedup` (off master `33db764`). **Verdict:** **0 Critical / 1 Important** (+ 2 Minor).

> Persisted verbatim per CLAUDE.md BEFORE the fold step. The Important (I1) is a SPEC-text fold (name the RED discriminator); architecture sound; verbatim-lift premise confirmed. Fold → re-dispatch per the after-every-fold loop.

---

**VERDICT: 0 Critical / 1 Important** (+ 2 Minor)

The verbatim-lift premise **holds**. Sites 1/2/4 differ only in their `EmitInputs` construction (legitimate per-site code that stays) and in comment/line-wrap cosmetics; nothing inside either match references a per-site local outside `inputs`/`format`. The one Important is a SPEC-text fold (name the RED discriminator), not an architecture defect — fold it and re-dispatch per the after-every-fold loop. Do not start coding until that fold lands and the architect re-runs to GREEN.

Source verified against working tree on branch `restore-emit-dispatch-dedup` (current line numbers below; the SPEC's were drifted by 1 on the divergent string — `:649` vs actual `:650`).

---

## Critical
None.

---

## Important

### I1 — Phase-1 RED cell: exit code is 1 BOTH before and after; the RED-ness rests ENTIRELY on the message substring. The SPEC must name the discriminator or the lone behavior change ships untested.

Evidence:
- `BadInput(_) => 1` (`error.rs:490`). The OLD single-sig arm is `Err(bad(...))` (`restore.rs:649-651`) and `bad()` constructs `ToolkitError::BadInput` (`restore.rs:148-150`). The NEW unified arm is `_ => Err(ToolkitError::BadInput(...))` (`export_wallet.rs:548-551`). **Both map to exit 1.**
- I confirmed the emit arm is actually *reached* (not pre-empted by `collect_missing`): `ColdcardEmitter::collect_missing` returns `Vec::new()` unconditionally (`coldcard.rs:24-39`), so the `if !missing.is_empty()` short-circuit (`restore.rs:638`) never fires for `ColdcardMultisig`. The divergent arm is live code today, and Decision (a) is a genuine behavior change — not a no-op. Good.
- Consequence: an exit-code assertion on the new cell is GREEN→GREEN (no signal). The cell can only be RED-for-the-right-reason by asserting on the message. The OLD message is `"--format coldcard-multisig requires a multisig wallet; restore is single-sig — use --format coldcard"` (`restore.rs:650`); the NEW message is `"--format coldcard-multisig requires a multisig --template (wsh-sortedmulti, …)…"` (`export_wallet.rs:549`). The substring `"requires a multisig --template"` is present in the new and ABSENT from the old (the old says "requires a multisig **wallet**"). That exact substring is already proven as a working discriminator at `cli_export_wallet_coldcard.rs:536`.

Why blocking (Important, not Critical): the SPEC §5/§7 says "pin the chosen message" and "RED-for-the-right-reason" but never names the substring, and §6/§8 repeatedly frame the change as "exit 1 unchanged" — which actively invites an exit-only assertion that would never go RED. Without the explicit discriminator the SPEC permits a cell that passes vacuously, leaving the only behavior delta in the cycle unguarded.

Fix (one-line SPEC fold): in §5 and §7, state that the Phase-1 cell runs `restore --from phrase=… --template bip84 --format coldcard-multisig`, asserts exit `1` AND `stderr.contains("requires a multisig --template")`, and add the note: *"exit is 1 both before and after; the message substring — not the exit code — is what makes this cell RED against the current binary (which emits 'requires a multisig wallet; restore is single-sig')."* Then re-dispatch the architect.

---

## Minor

### M1 — The unified message's first clause is a dead-end pointer in the restore context.
Single-sig restore already refuses multisig `--template` upfront (`restore.rs:199-203`, "restore is single-sig only; --template ∈ {bip44,bip49,bip84,bip86}"). The new unified message's first sentence steers the user toward `wsh-sortedmulti, wsh-multi, …` — templates restore rejects. The second sentence ("For Coldcard singlesig export use --format coldcard with bip44/bip49/bip84") salvages it with the correct pointer. This is the conscious cost of Decision (a) over (b), which the SPEC rejected with reasoning (§4). Surface it in the CHANGELOG note so a human sees the wrinkle; not blocking.

### M2 — The SPEC's `:649` citation for the divergent string is off-by-one.
Actual `Err(bad(…))` open is `restore.rs:649`; the string literal is `:650`. Cosmetic; the SPEC's other line numbers are close enough to navigate. Refresh when folding I1 (this is the [[feedback_followups_md_line_numbers_presumed_stale]] / off-by-N class — worth correcting at write time).

---

## What verified clean (audit trail)

**Item 1 — exactly 4 dispatch sites, no 5th.** Grep of `collect_missing` across all of `src/` returns dispatch copies only in `export_wallet.rs` (`run` collect at `:506-519` / emit at `:527-561`; `run_from_import_json` collect at `:760-775` / emit at `:783-817`) and `restore.rs` (`build_import_payload` collect at `:624-637` / emit at `:645-659`; `build_multisig_import_payload` collect at `:705-718` / emit at `:728-755`). All other `collect_missing` hits are the per-emitter trait impls (`wallet_export/*.rs`) and the trait decl (`mod.rs:398`). No 5th copy.

**Item 2 — `collect_missing` half byte-identical across all 4.** The 11 arms at `export_wallet.rs:508-518`, `:765-774`, `restore.rs:626-636`, `:707-717` are identical token-for-token (same emitter, same `"format-name"` literal, same `(…, "…")` tuple). The only difference is cosmetic line-wrapping of the `BitcoinCore` arm at `export_wallet.rs:762-764` (3 lines vs 1) — semantically identical.

**Item 3 — `emit` half byte-identical for sites 1/2/4 (the highest-risk item).** `run` (`:527-561`) and `run_from_import_json` (`:783-817`) are identical including the coldcard-multisig 6-variant `CliTemplate` sub-match (`:531-553` ≡ `:787-809`), the `use crate::template::CliTemplate;` line, the comment block, the emitter calls, and the trailing `}?;`. Site 4 (`restore.rs:728-755`) is identical in the coldcard-multisig sub-match (`:732-747`) and all other arms, differing only by (a) omitting the redundant `use crate::template::CliTemplate;` (already imported at `restore.rs:37`) and the comment block, and (b) being the function's tail `Result<String>` with NO trailing `?` (vs sites 1/2 which apply `?` then write `emitted`). The helper returning `Result<String, ToolkitError>` accommodates both: sites 1/2 call `emit_payload(&inputs, args.format)?`; sites 3/4 return `emit_payload(&inputs, format)` directly. No subtle variable-name / field / extra-branch divergence.

**Item 4 — single-sig restore (site 3) divergence is exactly one arm.** Only the `ColdcardMultisig` arm diverges: `restore.rs:649-651` is `Err(bad("…requires a multisig wallet; restore is single-sig — use --format coldcard"))` — a flat `Err`, not the 6-variant `CliTemplate` sub-match. All 10 other arms (`:646-648`, `:652-658`) are byte-identical to canonical. No other field/emitter divergence; the `EmitInputs` it feeds (`:595-613`) is single-sig-shaped (`threshold: None`, `threshold_user_supplied: false`) but that is legitimate per-site construction, not dispatch divergence.

**Item 5 — Decision (a) safety; no test/manual pins the old behavior; exit preserved.** Grep for `"restore is single-sig — use --format coldcard"` / `"requires a multisig wallet"` across `tests/` returns nothing (only the source definition at `restore.rs:650`). No `cli_restore*` test runs `restore --format coldcard-multisig` on a single-sig template: `cli_restore.rs` single-sig `--format` cells cover descriptor/bitcoin-core/bip388/specter/jade (`:481-1062`) — the coldcard-multisig single-sig arm is currently UNCOVERED (hence the RED cell is needed and possible). `cli_restore_multisig_format.rs` exercises coldcard-multisig only on a multisig md1 (`:79-89`, asserts exit 0 + "Policy: 2 of"), which hits site 4's `Some(WshSortedMulti)` arm — unchanged. The manual does not quote the old message (no matches in `docs/manual/`). Exit preserved: old `bad()`→BadInput→1 and new `_ => BadInput`→1 (`error.rs:490`); the `collect_missing` short-circuit cannot pre-empt it because `ColdcardEmitter::collect_missing` is `Vec::new()` (`coldcard.rs:39`).

**Item 6 — homing in `cmd/export_wallet.rs` is correct; no cycle.** `CliExportFormat` is defined in `cmd/export_wallet.rs:23`. `wallet_export` does NOT import it (grep empty), so homing the helper in `wallet_export` would force `wallet_export → cmd::export_wallet::CliExportFormat` — a dependency inversion the SPEC correctly avoids. `restore.rs` already imports `CliExportFormat` from `crate::cmd::export_wallet` (`restore.rs:31`), so `crate::cmd::export_wallet::emit_payload` is callable without a new cycle. The helper's home already has every needed symbol in scope (`WalletFormatEmitter` + all emitter types at `export_wallet.rs:11-18`).

**Item 7 — `EmitInputs` shape; signature sufficient (the highest-risk item).** `EmitInputs<'a>` is `pub(crate)` (`wallet_export/mod.rs:466`), threaded by `&EmitInputs` through the trait (`mod.rs:398-399`). Each of the 4 sites builds its own `EmitInputs` (`export_wallet.rs:483-500`, `:732-756`; `restore.rs:595-613`, `:684-701`) — legitimate per-site code that stays. Inside BOTH matches at all 4 sites, the ONLY referenced symbols are `inputs` (the `EmitInputs`), `format`/`args.format` (→ the `format` param), the `*Emitter` types, `CliTemplate` (sub-match), `ToolkitError`, and `"format-name"` string literals. No stderr handle, no network, no `args.*` beyond `args.format`, no other per-site local is referenced. The signature `fn emit_payload(inputs: &EmitInputs, format: CliExportFormat) -> Result<String, ToolkitError>` is sufficient and complete (lifetime elides correctly given `EmitInputs<'a>`).

**Item 8 — unused imports after refactor; remove-on-warning is sound.** In `restore.rs`, the 10 emitter types + `WalletFormatEmitter` (`:39-41`) are used ONLY inside the two matches (all hits at `:626-658`, `:707-754`) → all 11 become unused after the lift and clippy/`cargo build` will flag exactly them. `build_descriptor_string` (`:379`, `:915`), `EmitInputs`, `CheckedDescriptor`, `BsmsForm`, `TimestampArg`, `wallet_export::{self}`, `script_type_from_template` STAY used. In `export_wallet.rs` the emitter imports STAY used (the helper lives in that module). The SPEC's "remove on warning" plan is correct and complete.

**Item 9 — SemVer PATCH + no lockstep + no new variant.** Current version `0.46.0` (`crates/mnemonic-toolkit/Cargo.toml:3`) → `0.46.1` PATCH. No clap flag/value/subcommand change (the `--format` value set is untouched) → no GUI `schema_mirror`, no manual mirror, no sibling-codec change. No new error variant — reuses `ExportWalletMissingFields` (exit 2, `error.rs:513`) + `BadInput` (exit 1, `error.rs:490`).

**Item 10 — Phase-1 RED cell is well-formed (modulo I1).** RED against the current binary (old "requires a multisig wallet" message) and GREEN after, because the emit arm is reachable (`collect_missing` empty) and the messages differ on the `"requires a multisig --template"` substring. The phased plan (RED → GREEN helper+4 sites+import-prune → release → ship) is sound. The only gap is that the SPEC must name the substring discriminator and note the exit is 1 both ways — see I1.

---

**Gate status: NOT yet GREEN.** Fold I1 (name the RED discriminator + exit-1-both-ways note) and optionally M1/M2, then re-dispatch the architect for the next round per the after-every-fold loop. Architecture is sound; the verbatim-lift premise is confirmed; no Critical.
