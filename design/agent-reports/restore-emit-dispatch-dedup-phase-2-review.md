# Phase 2 (GREEN) Code Review — `restore-emit-dispatch-dedup`

**Reviewer:** opus `feature-dev:code-reviewer` (mandatory per-phase review). **Date:** 2026-06-05.
**Branch:** `restore-emit-dispatch-dedup`. **Verdict:** **0 Critical / 0 Important** (+ 1 Minor). **GREEN — Phase 2 may proceed to Phase 3 (release).**

> Persisted verbatim per CLAUDE.md. Reviewer had no shell (code-reviewer agent), so the faithful-lift premise was cross-checked against `CliExportFormat`'s `#[value(name=...)]` declarations + existing gating tests + the GREEN 877+ suite, rather than a byte-diff of 33db764 (which R0 rounds 1+2 already performed line-by-line). M1 (stale line-cite) folded.

---

I reviewed the `emit_payload` helper extraction and its 4 call sites against SPEC `design/SPEC_restore_emit_dispatch_dedup.md`, the two R0 reviews, the `EmitInputs` struct, the `CliExportFormat` value-enum, and the Phase-1 test cell. Direct reads of the live helper (`crates/mnemonic-toolkit/src/cmd/export_wallet.rs:74-137`), all 4 call sites, the restore.rs import block, and the new test cell.

Note on method: there is no shell in this environment and git blobs aren't readable via the file tools, so the pre-refactor source could not be byte-diffed directly. The faithful-lift claim instead rests on direct inspection of the live helper cross-checked against `CliExportFormat`'s own `#[value(name=...)]` declarations (a source-internal invariant, not a pre-refactor dependency), corroborated by the two existing tests that already gate the unified strings and by the GREEN 877+ suite.

### VERDICT: 0 Critical / 0 Important (+ 1 Minor)

**GREEN — Phase 2 may proceed to Phase 3 (release).**

---

### Critical
None.

### Important
None.

### Minor

**M1 — Stale line-cite in the multisig restore doc-comment** (`crates/mnemonic-toolkit/src/cmd/restore.rs:627`). The comment reads `Mirrors export-wallet's multisig EmitInputs (export_wallet.rs:483-496)`. The 63-line `emit_payload` insertion (`export_wallet.rs:74-137`) shifted the canonical `run` `EmitInputs` literal down — it now lives at `export_wallet.rs:560-577`. The `:483-496` cite is stale. This block was edited this cycle (it references the new `emit_payload` two lines below), so it is in-scope for the cycle. Non-blocking; matches the repo's known line-cite-decay class (`feedback_followups_md_line_numbers_presumed_stale`). Fix: update to `export_wallet.rs:560-577` or drop the line number.

---

### What verified clean

1. **Helper is a faithful lift — verified by direct inspection of the live arms** (`export_wallet.rs:82-136`). Both matches have all 11 arms; each `collect_missing`/`emit` arm dispatches to the emitter whose name matches its `CliExportFormat` variant, and every one of the 11 `"format-name"` string literals matches that variant's `#[value(name=...)]` declaration (`export_wallet.rs:22-46`) — a source-internal check independent of any pre-refactor blob. The `ColdcardMultisig` 6-variant `CliTemplate` sub-match (`:114-127`) is complete (WshMulti / WshSortedMulti / ShWshMulti / ShWshSortedMulti / TrMultiA / TrSortedMultiA → `ColdcardEmitter::emit`; `_` → `BadInput` with the unified string at `:124`). That unified string is already gated by two pre-existing tests (`tests/cli_export_wallet_coldcard.rs:536`, `tests/cli_export_wallet_from_import_json.rs:916`), so an emitter swap or string drift on that arm would already be RED. Corroborated by the R0 round-1 review's arm-by-arm canonical documentation and the GREEN suite (every format is output-tested, closing the one swap case the message-cells alone can't).

2. **`&inputs` vs `inputs` correct; no by-value/clone** (`export_wallet.rs:74-77`). Signature takes `inputs: &EmitInputs`; the body passes `inputs` straight through to `Emitter::collect_missing(inputs)` / `::emit(inputs)` (no `&inputs`, no `.clone()`). All 4 callers pass `&inputs` (owned local → ref). A by-value param or double-ref would be a compile error; the GREEN build closes this.

3. **All 4 `EmitInputs` constructions are per-site and match the intended invariants.** `run` (`:560-577`) `master_xpub_at_0` from slot 0; `run_from_import_json` (`:751-775`) `master_xpub_at_0: None`, `taproot_internal_key: None`, `threshold_user_supplied: threshold.is_some()`; single-sig restore (`restore.rs:593-611`) `threshold: None, threshold_user_supplied: false, master_xpub_at_0: row.slot.master_xpub`; multisig restore (`restore.rs:647-664`) `threshold: Some(k), threshold_user_supplied: true`. `EmitInputs` has 16 fields (`wallet_export/mod.rs:466-518`) with no `Default`/spread, so every site populates all 16 by name — a dropped field would be a compile error. Per-site values match the task's stated invariants. The only edit at each site is `match {...}` → `emit_payload(...)`; the `EmitInputs` literal sits above the edited region.

4. **Single-sig behavior delta is exactly the intended one.** The old `"requires a multisig wallet; restore is single-sig — use --format coldcard"` string is GONE from all of `src/` (the only remaining mentions are a test comment, the SPEC, and design docs). Single-sig restore now tail-returns `emit_payload(&inputs, format)` (`restore.rs:622`); `bad()` is still defined (`:146`) and heavily used elsewhere (no orphan). No other single-sig arm changed semantics — specter/jade/bitcoin-core/etc. route through the identical shared dispatch.

5. **No dead code / `#[allow]` masking.** All 11 `*Emitter` + `WalletFormatEmitter` imports were physically pruned from `restore.rs` (the import block at `:38-40` now carries only `self, build_descriptor_string, BsmsForm, CheckedDescriptor, EmitInputs, TimestampArg`, all still used — `build_descriptor_string` at `:377`/`:829`). The only `#[allow(` in either file is the pre-existing `clippy::too_many_arguments` on the multisig fn (`restore.rs:635`); no `#[allow(unused)]`/`dead_code]` masks a now-unused import.

6. **Test cell is a faithful, non-vacuous guard** (`tests/cli_restore.rs:598-635`). Uses the real `TREZOR_12` BIP-39 vector (so it reaches the emit arm, not a parse error); asserts exit `1`, `stderr.contains("requires a multisig --template")`, and empty stdout. The substring is present only in the NEW unified message and absent from the OLD — a genuine discriminator (matching `cli_export_wallet_coldcard.rs:536`). Strengthener: the `exit == 1` assertion also runtime-confirms `ColdcardEmitter::collect_missing` is empty for this input — otherwise the refusal would be `ExportWalletMissingFields` (exit 2), not `BadInput` (exit 1). So the `collect_missing`→emit ordering and the divergent arm's reachability are runtime-verified, not assumed.

7. **No weakened guarantee.** Both `run` and `run_from_import_json` still wrap the helper's output in `writeln!(stdout, "{emitted}").map_err(ToolkitError::Io)?` (`export_wallet.rs:586`, `:785`) — broken-pipe/I/O propagation is intact, not bypassed. The `collect_missing`-first short-circuit ordering is preserved inside the helper (`:96-101` before the `emit` match). No clap surface change → SPEC §6 lockstep = NONE holds (no GUI `schema_mirror`, manual, or sibling change); no new error variant.
