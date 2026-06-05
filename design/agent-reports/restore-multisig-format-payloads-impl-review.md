# Per-phase implementation review — mnemonic-toolkit v0.45.0 (multisig restore --format payloads)

**Reviewer:** opus `feature-dev:code-reviewer` (gate before tag).
**Date:** 2026-06-05. **Branch:** `restore-multisig-format-payloads` vs `master` (Phase 1 RED `…`, Phase 2 `1f337bf`, Phase 3 `277ec5b`).
**Verdict:** **0 Critical / 0 Important — GREEN** (source-and-test basis). 2 no-action Minors.

> Persisted verbatim per CLAUDE.md. **Reviewer environment had no shell** (could not run the binary); the 9-emit/2-refuse behavioral split it flagged as "verified by inheritance, not independent run" is **independently discharged** by the orchestrator's Phase-2 `cargo test` GREEN run — `tests/cli_restore_multisig_format.rs` exercises all 9 emit-format threshold tokens + specter(exit2)/green(exit1) refusals against a real bundled md1, all passing — plus the pre-SPEC empirical `export-wallet --template wsh-sortedmulti` table. GREEN ⇒ no fold, no re-dispatch.

---

**Scope reviewed:** `run_multisig` + `build_multisig_import_payload` (`restore.rs`), the dispatch in `export_wallet.rs`, the refusal emitters (`specter.rs`, `green.rs`, `sparrow.rs`), `EmitInputs` (`wallet_export/mod.rs`), exit-code mapping (`error.rs`), `tests/cli_restore_multisig_format.rs`, CHANGELOG/FOLLOWUPS/version/install.sh, the manual section.

**Environment disclosure:** could not execute `target/release/mnemonic` (no Bash tool). Every claim verified by source + test inspection. The two behavioral asks (9-emit/2-refuse split; runtime token check) discharged structurally (inheritance from export-wallet's verbatim emitters + arm-for-arm dispatch parity) — and empirically by the orchestrator's test run (see header).

### Critical
None.

### Important
None.

### Minor (no action required)
- **`build_multisig_import_payload`'s coldcard-multisig arm omits the redundant inline `use crate::template::CliTemplate;`** that `export_wallet.rs:538` carries. NOT a divergence — `CliTemplate` is already imported at `restore.rs:37`, so the match is functionally byte-identical (it compiled + tested GREEN). Noted only so the future 3-way de-dup (`restore-emit-dispatch-3way-dedup`) consolidator doesn't treat it as a behavioral delta. Confidence benign: 95.
- **Watch-only test guard (`tests/…:167-168`, `!contains("xprv"/"tprv")`) is narrower than the structural guarantee** — would miss a WIF (`L`/`K`/`5`) leak. Acceptable: the real guarantee is structural (every slot built `entropy: None, master_xpub: None`, so no emitter ever receives private material). Test is a backstop, not the guarantee.

### What verified clean (basis)

1. **`EmitInputs` literal (16 fields)** — ACCURATE vs struct `wallet_export/mod.rs:466-518`: `threshold: Some(k)`, `threshold_user_supplied: true`, `resolved_slots: slots`, `template: Some(template)`, `taproot_internal_key: None`, `master_xpub_at_0: slots.first().and_then(|s| s.master_xpub)`, synth `wallet_name="<template>-<account>"`, `range:(0,999)`, `timestamp: Now`, `bitcoin_core_version:25`, `bsms_form: default()` — all match export-wallet's defaults (`export_wallet.rs:483-500`). The `collect_missing`→refuse→`emit` dispatch (restore) is arm-for-arm identical to `export_wallet.rs:506-560`, incl. the 6-variant coldcard-multisig `CliTemplate` match.
2. **run_multisig weave** — ACCURATE. `import_payload` computed AFTER the step-7 mismatch hard-gate (a non-`--allow-mismatch` MISMATCH `return`s exit-4 `RestoreMismatch` before `build_multisig_import_payload` is called). stdout routing: `--json` adds `envelope["import_payload"]`; `--format && !--json` → payload alone; else text doc. Verification doc → stderr under `--format && !--json`. `--output FILE` writes the payload (not the doc).
3. **No regression** — single-sig `--format` (in `run`, not `run_multisig`) untouched (`threshold:None`); non-`--format` multisig text/JSON path preserved.
4. **Tests catch regressions** — threshold tokens non-vacuous + per-format-pinned; 3-fp cell asserts real md1 fps; `FOREIGN="zoo…wrong"` is a checksum-valid BIP-39 vector so it reaches the cross-check → exit 4 (not a parse error) + asserts no payload; `--output` cell asserts stderr `cosigner @0` + file `sortedmulti(2,`.
5. **9-emit/2-refuse** — ACCURATE by inheritance (restore reuses export-wallet emitters verbatim; only delta is `threshold_user_supplied: true`, which is what `sparrow.rs:43` keys off → sparrow emits). specter→exit2 (`specter.rs:34` `MissingField::WalletName` when `!wallet_name_is_non_default`, restore sets false → `ExportWalletMissingFields`→2). green→exit1 (`green.rs:36` `BadInput` when `script_type.is_multisig()`; `script_type_from_template(WshSortedMulti)=P2wshMulti`→1).
6. **CHANGELOG/FOLLOWUPS/version/manual** — Cargo.toml/lock=0.45.0, both README markers=0.45.0, install.sh pin=v0.45.0; FOLLOWUP resolved + dedup filed; manual `--format` paragraph accurate + Scope line no longer lists `--format` single-sig-only. No overclaim.
7. **Watch-only-out** — structurally stronger than the test: slots built `entropy: None, master_xpub: None`; emitters receive only public xpubs → no private key can reach any payload by construction.
8. **GUI/sibling lockstep** — no clap flag/value-enum change (`--format` pre-existing; `EXPORT_FORMATS` lists all 11) → no `schema_mirror` change; toolkit-only.

### VERDICT
**0 Critical / 0 Important — GREEN.** Faithful mirror of single-sig restore's weave + export-wallet's dispatch; load-bearing `threshold_user_supplied: true`, post-hard-gate ordering, watch-only-by-construction, and all release-prep sites correct. The 11-format empirical split (reviewer couldn't run) is covered by the passing Phase-2 test file.
