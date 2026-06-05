# R0 Architect Review (round 1) — SPEC_restore_multisig_format_payloads.md

**Reviewer:** opus `feature-dev:code-reviewer` (mandatory pre-implementation R0 gate).
**Date:** 2026-06-05. **Branch:** `restore-multisig-format-payloads` (master `9bb80a0`).
**Verdict:** **0 Critical / 1 Important — RED.** (3 Minors.)

> Persisted verbatim per CLAUDE.md BEFORE folding. Fold note at end; re-dispatched after fold (R0-r2).

---

**Tooling note (limitation, surfaced per `feedback_architect_must_run_prose_commands`):** this environment exposes no shell/Bash execution, so I could **not run the v0.44.0 binary**. All verification below is **source-read** against current branch files. The SPEC's §2 EMIT-success *byte counts* (e.g. "coldcard 442b") and runtime exit behavior are therefore **source-confirmed-but-unrun**; the REFUSAL claims and exit codes are fully source-solid (predicates + `exit_code()` read directly). Phase-1 RED must re-capture the §2 table against the actual binary as its gate.

Note: the SPEC cites bare filenames (`restore.rs`, `export_wallet.rs`); the real paths are `crates/mnemonic-toolkit/src/cmd/restore.rs` and `crates/mnemonic-toolkit/src/cmd/export_wallet.rs`. The cited **line numbers all resolve correctly** against those files.

## Critical
None.

## Important

**I1 — §6 primary assertion "payload contains the threshold `2`" is vacuous as worded; it must name the exact per-format threshold token.** (`SPEC §6`, EMIT × multisig-fidelity bullet.)
This is the SPEC's *named primary correctness check* and the only assertion (besides `--format descriptor` exact-equality) that catches a wrong-`k` single-sig-ify in the **non-descriptor** formats. But "`2`" is not a load-bearing substring: the digit `2` appears in cosigner xpubs, in origin paths (`48'/0'/0'/2'`), in the BIP-48 script-type index, and in dates. An implementer reading §6 literally is free to write `payload.contains("2")`, which passes vacuously even on a K-1 / dropped-cosigner regression — exactly the failure mode this cell exists to catch, and exactly the class in the project's own `feedback_ci_snapshot_test_substring_vacuity` lesson. The threshold representation is genuinely format-specific and was confirmed in source:
- `descriptor` / `bitcoin-core` / `bip388` / `sparrow`: `sortedmulti(2,` or `multi(2,` inside the descriptor/policy string.
- `coldcard` / `coldcard-multisig`: the literal line `Policy: 2 of 3` (`coldcard.rs:355`).
- `electrum` / `jade`: yet another encoding.

**Fix:** §6 must pin the exact per-format threshold token each cell asserts (e.g. a table). The "all three cosigner fingerprints" half is already robust and catches the *drop-a-cosigner* case — keep it; the threshold-token half just needs a non-vacuous token so the *wrong-K, right-cosigner-count* case is actually covered. SPEC-tightening only (no code/design change); fold it and the gate flips GREEN.

## Minor

**M1 — `is_multisig()` mischaracterization of the coldcard-multisig branch.** The real branch at `export_wallet.rs:539-552` does **not** call `is_multisig()` — it `match`es `inputs.template` against the six multisig `CliTemplate` variants and emits, else `Err(BadInput)`. Behaviorally identical, cosmetic — but since the implementer is told to copy "byte-identical," describe it as a template-variant match, not an `is_multisig()` call.

**M2 — §2/§7 byte-parity-drop rationale overstates `--slot`'s limitation.** "`export-wallet --slot @N.xpub=` uses placeholder origin `00000000`" is true only for the *bare* `[Xpub]` slot form; `slot_input.rs:354-359` shows `[Xpub, Fingerprint]` and `[Xpub, Fingerprint, Path]` are accepted, so real fp+origin *can* be supplied. Conclusion still holds (the in-run `--format descriptor` exact-equality check is strictly stronger and clean) — just soften "NOT viable" to "not cleanly reproducible without hand-reconstructing every cosigner's fp+origin, and unnecessary given the in-run descriptor-equality check."

**M3 — field-count label.** §3 prose calls `EmitInputs` "16-field" — correct (recounted `mod.rs:472-517`: 16 fields); §3 literal initializes all 16 with matching types. No action.

## What verified clean

- **Refusal gate + data availability.** `restore.rs:735-741` is the `args.format.is_some()` multisig `ModeViolation` (exit 2). At the proposed weave point (after reconstruction `:823-824` + after the mismatch hard-gate `:977-987`), `template`/`slots`/`k`/`descriptor`/`network`/`args.account` are in scope; the mismatch gate (`RestoreMismatch` exit 4) returns before any payload — §6 mismatch-precedence structurally guaranteed. ACCURATE.
- **EmitInputs field set + types.** Struct `mod.rs:466-518`, 16 fields, all match (`threshold_user_supplied: bool`, `master_xpub_at_0: Option<Xpub>`, `taproot_internal_key: Option<TaprootInternalKey>`, `bsms_form: BsmsForm`, `timestamp: TimestampArg`, `range: (u32,u32)`, `bitcoin_core_version: u8`). `resolved_slots: &'a [ResolvedSlot]`; lifetimes fine (mirrors single-sig `from_ref` `:597` + borrowed `wallet_name` `:606`). ACCURATE.
- **`threshold_user_supplied: true`.** Only consumer repo-wide is `sparrow.rs:43` (`collect_missing` pushes `MissingField::Threshold` iff `t.is_multisig() && !threshold_user_supplied`); no `emit()` reads it. `true` un-refuses sparrow, breaks nothing else, semantically correct (k from md1 authoritative). ACCURATE / load-bearing.
- **Dispatch copy.** `collect_missing`→refuse→`emit` at `export_wallet.rs:506-561`; coldcard-multisig template-variant branch `:531-553`. All emitter type names + `collect_missing`/`emit`/`WalletFormatEmitter` + `CliTemplate` already imported into restore.rs (`:31,:37,:38-42`); single-sig `build_import_payload` (`:624-659`) already uses them. ACCURATE (modulo M1 wording).
- **Per-format outcomes (source-confirmed).** `collect_missing` empty for bitcoin-core/bip388/coldcard/bsms/descriptor/jade/electrum/green; specter pushes `WalletName` when `!wallet_name_is_non_default` (`specter.rs:34`) → refuses (`ExportWalletMissingFields` exit 2); green refuses in `emit()` via `script_type.is_multisig()` (`green.rs:36`) → `BadInput` exit 1. `script_type_from_template` maps every multisig template to a multisig `WalletScriptType` (`mod.rs:193-205`). Exit codes (`error.rs:483,506,521,528`): BadInput→1, ExportWalletMissingFields→2, ModeViolation→2, RestoreMismatch→4. `wallet_name` default `{human_name}-{account}` matches `export_wallet.rs:469-475` + single-sig `restore.rs:594`. ACCURATE (byte counts unrun).
- **`--format descriptor` exact-equality is sound.** `build_descriptor_string` → `Descriptor::to_string()` includes `#<csum>` (`pipeline.rs:28-30`); same string stored as `descriptor` (→ JSON `wallets[0].descriptor` `:823,1046`) and passed to `CheckedDescriptor::new`; `CheckedDescriptor::Display` forwards verbatim (`mod.rs:455-458`); `DescriptorEmitter::emit` returns `canonical_descriptor.to_string()` (`descriptor.rs:20`). So `--format descriptor` stdout == JSON descriptor byte-for-byte. Genuine byte-parity for that format.
- **Test strategy.** Containment catches silent single-sig-ify via the 3-fingerprint assertion (drop-a-cosigner) + descriptor-equality (exact). Residual wrong-K/right-count gap in non-descriptor formats closed only once I1 pins a non-vacuous token. Provenance argument for dropping cross-tool byte-parity is directionally right though overstated (M2); unnecessary to fix given the stronger in-run check.
- **Scope/SemVer/lockstep.** No new clap flag (restore already declares `--format: Option<CliExportFormat>` `:132-133` reusing the enum → `schema_mirror` untouched, GUI `EXPORT_FORMATS` needs no change). taproot refused upstream `:766`. SemVer MINOR defensible. Manual target `### Multisig-cosigner restore (--md1)` exists `41-mnemonic.md:900`; §7 invokes `make audit`. FOLLOWUP `restore-multisig-format-payloads` exists `FOLLOWUPS.md:70`; new `restore-emit-dispatch-3way-dedup` appropriately filed. §7 complete.
- **Watch-only-out.** All EMIT payloads public-only (slots carry `master_xpub: None`, `entropy: None` `:809-812`). §6 "NOT contains xprv/tprv" across stdout+stderr+json adequate; WIF-heuristic exclusion correctly avoided.

## VERDICT
**0 Critical / 1 Important — RED.** Single blocker I1 (vacuous `contains("2")`). SPEC-text tightening, no design change. Fold I1 (+ M1/M2 wording), re-persist, re-dispatch; gate flips GREEN. Phase-1 RED should re-capture the §2 EMIT table against the live binary (byte counts unrunnable here).

---

## Fold note (applied after persisting)
- **I1 — FOLDED:** §6 now pins the exact per-format threshold token per emit format (table), determined empirically from the v0.44.0 binary. The 3-fingerprint assertion is retained alongside.
- **M1 — FOLDED:** §2/§3 reworded — the coldcard-multisig arm is a six-variant `CliTemplate` match (`export_wallet.rs:539-552`), not an `is_multisig()` call.
- **M2 — FOLDED:** §2 byte-parity rationale softened to "not cleanly reproducible without hand-reconstructing each cosigner's fp+origin, and unnecessary given the in-run `--format descriptor` exact-equality check."
- **M3 — no action** (label correct).
- Re-dispatched R0 (R0-r2) per "re-dispatch after every fold round."
