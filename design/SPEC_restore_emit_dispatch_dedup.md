# SPEC — consolidate the 4-way `WalletFormatEmitter` dispatch into one `emit_payload`

**Status:** R0 gate (pre-implementation). MUST converge to 0 Critical / 0 Important before any code.
**Resolves:** FOLLOWUP `restore-emit-dispatch-3way-dedup` (corrected: **4-way**, not 3).
**Source SHA:** branch `restore-emit-dispatch-dedup` off master `33db764`.
**SemVer:** PATCH — pure refactor; the only user-visible change is a reworded single-sig `restore --format coldcard-multisig` refusal (behavior-equivalent, exit 1 unchanged). v0.46.0 → **v0.46.1**.

---

## 1. Summary

The `collect_missing`-first → refuse → 11-arm `emit` `WalletFormatEmitter` dispatch (incl. the 6-variant coldcard-multisig `CliTemplate` branch) exists in **FOUR** byte-identical-modulo-one-arm copies (recon corrected the FOLLOWUP's "3"):

1. `export_wallet.rs::run` — `collect_missing` `:506`, `emit` `:527`.
2. `export_wallet.rs::run_from_import_json` (`:584`) — `collect_missing` `:760`, `emit` `:783` (**the FOLLOWUP missed this one**).
3. `restore.rs::build_import_payload` (single-sig, `:587`) — `collect_missing` `:624`, `emit` `:645`.
4. `restore.rs::build_multisig_import_payload` (`:673`) — `collect_missing` `:705`, `emit` `:728`.

The `collect_missing` half is byte-identical across all 4. The `emit` half is byte-identical for **3** (sites 1/2/4 use the 6-variant coldcard-multisig template match); **site 3 (single-sig restore) diverges** — its coldcard-multisig arm (`Err(bad(…))` opens `:649`, string literal `:650`) is `Err(bad("--format coldcard-multisig requires a multisig wallet; restore is single-sig — use --format coldcard"))` instead.

Consolidate into one `pub(crate) fn emit_payload(inputs: &EmitInputs, format: CliExportFormat) -> Result<String, ToolkitError>` consumed by all 4 sites. The `EmitInputs` construction is NOT shared (each site builds it from different inputs — that is legitimate per-site code; the dedup is the dispatch only).

## 2. The helper — `cmd/export_wallet.rs`

Home it in `cmd/export_wallet.rs` (where `CliExportFormat` + the `run` dispatch live), NOT `wallet_export` — keeps the dependency direction `cmd → wallet_export` clean (wallet_export must not import `cmd::export_wallet::CliExportFormat`). restore.rs (also `cmd`) calls `crate::cmd::export_wallet::emit_payload`.

```rust
/// Shared `WalletFormatEmitter` dispatch: collect_missing-first → emit.
/// Consolidates the formerly-4 byte-identical copies (FOLLOWUP
/// `restore-emit-dispatch-3way-dedup`). The caller builds `inputs`.
pub(crate) fn emit_payload(
    inputs: &EmitInputs,
    format: CliExportFormat,
) -> Result<String, ToolkitError> {
    // collect_missing-first (verbatim from export_wallet.rs:506-525).
    let (missing, format_name): (Vec<crate::wallet_export::MissingField>, &'static str) =
        match format { /* the 11 arms, byte-identical to :508-518 */ };
    if !missing.is_empty() {
        return Err(ToolkitError::ExportWalletMissingFields { format: format_name, missing });
    }
    // emit (verbatim from export_wallet.rs:527-561, incl. the coldcard-multisig
    // 6-variant CliTemplate match :531-553).
    match format { /* the 11 arms */ }
}
```

The body is lifted **verbatim** from `export_wallet.rs::run`'s two matches (the canonical copy). The trailing `?` that `run` applies to `emit`'s `Result` moves into the helper (helper returns `Result<String>`).

## 3. The 4 call-site replacements

Each site keeps its `EmitInputs` build, then replaces its two inline matches with one call:

- **`export_wallet.rs::run`** (`:506-561`): `let emitted = emit_payload(&inputs, args.format)?;` (the `let emitted: String = match … }?;` block collapses).
- **`export_wallet.rs::run_from_import_json`** (`:760-…`): same — `emit_payload(&inputs, <its format>)?`.
- **`restore.rs::build_import_payload`** (`:624-660`): `emit_payload(&inputs, format)` (the fn already returns `Result<String>`; drop the local matches). **This changes the single-sig `--format coldcard-multisig` refusal** — see §4.
- **`restore.rs::build_multisig_import_payload`** (`:705-755`): `emit_payload(&inputs, format)`.

Remove the now-unused per-site emitter imports only if they become unused (most stay — `EmitInputs` construction still references emitter types? No — the construction references `EmitInputs`/`BsmsForm`/`TimestampArg`, not the `*Emitter` types; the `*Emitter::emit`/`collect_missing` calls move into the helper. So restore.rs's `Bip388Emitter`/`BitcoinCoreEmitter`/… imports likely become unused → remove them; clippy/`cargo build` warnings will flag exactly which).

## 4. Single-sig restore coldcard-multisig — DECISION (a): accept the unified message

Routing single-sig `build_import_payload` through the shared helper means `restore --format coldcard-multisig` on a single-sig template (bip44/49/84/86) now hits the 6-variant match's `_ => Err(ToolkitError::BadInput("--format coldcard-multisig requires a multisig --template (wsh-sortedmulti, wsh-multi, sh-wsh-sortedmulti, sh-wsh-multi, tr-multi-a, tr-sortedmulti-a). For Coldcard singlesig export use --format coldcard with bip44/bip49/bip84."))` — instead of the old `"…requires a multisig wallet; restore is single-sig — use --format coldcard"`. **Exit code unchanged (BadInput → 1).** The reword is acceptable (still a clear refuse-with-pointer); **no test pins the old message** (grep-confirmed across `tests/` + `src/`). **(M1 wrinkle to note in CHANGELOG)** the unified message's *first* sentence steers toward multisig `--template` values (`wsh-sortedmulti`, …) that single-sig restore refuses upfront (`restore.rs:199-203`); its *second* sentence — "For Coldcard singlesig export use --format coldcard with bip44/bip49/bip84" — carries the actually-correct pointer for the restore caller. This is the conscious cost of Decision (a) over (b); surface it in the CHANGELOG line so a human sees the wrinkle. (Decision (b) — a single-sig pre-check to preserve the old message — is rejected as needless special-casing that defeats the dedup; R0 confirmed.)

## 5. Tests
- **Green-stays-green:** the existing `--format` cells in `tests/cli_export_wallet*`, `tests/cli_restore_multisig_format.rs` (9 emit / 2 refuse), and the single-sig restore `--format` cells already exercise the dispatch behavior end-to-end — a behavior-preserving refactor for sites 1/2/4. Run them.
- **Phase-1 RED cell (I1 — discriminator named):** add ONE cell that runs `restore --from phrase=<TREZOR_12> --template bip84 --format coldcard-multisig` and asserts **exit `1` AND `stderr.contains("requires a multisig --template")`**. **(M3)** the phrase MUST be a real BIP-39 vector — use the existing `TREZOR_12` constant (`tests/cli_restore.rs:17`, already driving the `--template bip84` cells at `:76`/`:483+`); an invalid phrase fails at row-building before `restore.rs:452` and the cell never reaches the emit arm (→ RED-for-the-wrong-reason + never GREEN). The `"requires a multisig --template"` substring is the RED discriminator — it is present in the NEW unified message and ABSENT from the OLD (which says "requires a multisig **wallet**; restore is single-sig"). **Exit is 1 both before and after** (`bad()`→BadInput→1 today, `_ => BadInput`→1 after); the message substring — NOT the exit code — is what makes the cell RED against the current binary. (Substring already proven as a working discriminator at `tests/cli_export_wallet_coldcard.rs:536`.) This makes the §4 reword intentional + locks it.
- **from-import-json:** confirm an `export-wallet --from-import-json … --format X` cell still passes (site 2 is the least-tested copy — if no cell exercises it, add a smoke cell so the refactor of `run_from_import_json` is covered).
- Full workspace `cargo test --no-fail-fast` + clippy GREEN (clippy will flag any now-unused emitter imports → remove them).

## 6. Lockstep / scope
- **NONE.** No clap flag/option/value-enum/subcommand change (the `--format` value set is identical) → **no GUI `schema_mirror`, no manual mirror, no sibling-codec change**. No new error variant (reuses `ExportWalletMissingFields` + `BadInput`).
- Manual: the single-sig `restore --format coldcard-multisig` refusal message isn't quoted in the manual (verify at R0); if it is, update + `make audit`.

## 7. Phased plan
- **Phase 1 (RED):** the single-sig coldcard-multisig message cell (asserts exit `1` AND `stderr.contains("requires a multisig --template")` — see §5). Verify RED-for-the-right-reason: the current binary emits the OLD "requires a multisig wallet; restore is single-sig" message, so the substring assertion fails (exit 1 unchanged — the substring, not the exit code, is the discriminator).
- **Phase 2 (GREEN):** §2 `emit_payload` helper + §3 four call-site replacements + remove now-unused imports. Workspace test + clippy GREEN. Per-phase opus review → persist.
- **Phase 3 (release):** CHANGELOG `[0.46.1]` (note the reworded single-sig refusal + the M1 wrinkle: the unified message's first clause points at multisig templates restore rejects, second clause carries the correct `--format coldcard` pointer); version v0.46.0 → **v0.46.1** (Cargo.toml/lock + 2 READMEs + install.sh self-pin); FOLLOWUP `restore-emit-dispatch-3way-dedup` → resolved (correcting "3-way" → "4-way" in the close). Per-phase review.
- **Phase 4 (ship):** clean tree → ff-merge → tag `mnemonic-toolkit-v0.46.1` → push → watch CI (rust, install/sibling-pin-check; manual fires only if a manual file changed).

## 8. Risk
Low. Sites 1/2/4 are a verbatim lift (behavior-identical, test-covered). The single-sig restore reword (§4) is the only behavior delta — contained to one refusal message, exit-code-preserving, ungated. R0 must confirm: (i) no test/manual pins the old single-sig coldcard-multisig message; (ii) `run_from_import_json`'s dispatch is genuinely byte-identical to `run`'s (so site 2 folds cleanly); (iii) the helper's home (`cmd/export_wallet.rs` vs `wallet_export`) doesn't create an import cycle.
