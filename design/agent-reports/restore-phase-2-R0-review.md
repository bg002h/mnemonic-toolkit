# mnemonic restore — Phase 2 R0 Review (--format / --json / --output)

**Verdict (round 0): RED (0C / 1I)** → fold I1 + re-dispatch.

Commits `24efce3` (P2.1), `3777ee1` (P2.2). Watch-only-out + redaction invariants AIRTIGHT (hammered: every format, ±passphrase, across stdout/stderr/file/json → zero private-key/passphrase/seed leakage). One blocker: a spec-fidelity deviation in the `--format` dispatch.

## Critical
None. No path emits `account_xpriv`/xprv/tprv/yprv/zprv/WIF/passphrase/seed to any sink (stdout/stderr/json/file). Verify-gate correctly precedes payload/json emission (mismatch → exit 4, 0-byte stdout, error via `message()` not a json-success body).

## Important

**I1 — `--format` dispatch drops the `collect_missing` pre-check from the cited reuse region (`cmd/restore.rs:556-598` `build_import_payload`).** SPEC §3.5 + plan Task 2.1 cite the `WalletFormatEmitter` dispatch at `export_wallet.rs:507-561`, which is the **`collect_missing`-gated** dispatch (`:506-525` run `collect_missing` → short-circuit to `ExportWalletMissingFields` exit 64 before any `emit()`). `build_import_payload` mirrored only `emit()`. Consequences (all RUN):
- **`specter`**: `SpecterEmitter::collect_missing` (`specter.rs:31`) pushes `MissingField::WalletName` when `!wallet_name_is_non_default`. Restore hardcodes `wallet_name_is_non_default:false` + synth `wallet_name:"bip84-0"`, so export-wallet refuses (exit 64) but `restore --format specter` emits exit 0 a Specter JSON with placeholder `"label":"bip84-0"` — the silent-default the export rule rejects.
- **`jade` / `coldcard-multisig` (single-sig)**: both refuse, but `restore` returns **exit 1** (BadInput via the emitter `emit()` fallback) vs export-wallet's **exit 64** (ExportWalletMissingFields); the jade message even reads `"mnemonic export-wallet --format jade"` inside a `restore` invocation.
No leak / no wrong descriptor (green/electrum/coldcard/bitcoin-core/bip388/descriptor/sparrow/bsms all well-formed watch-only) → Important. **Fold (option a):** call `collect_missing` first + short-circuit to `ExportWalletMissingFields` (exit 64), mirroring `export_wallet.rs:506-525` exactly, so `restore --format` and `export-wallet --format` refuse identically.

## Minor
- **M1** `WalletRow.account_xpub` slightly redundant with `row.slot`'s xpub (still read by `--expect-xpub` `:375`); no action.
- **M2** jade refusal text says `mnemonic export-wallet` inside `restore` (shared `jade.rs:61` static) — auto-resolved by the I1 option-(a) fix.

## Verification ledger (RUN)
Diff scope = only `cmd/restore.rs` + `tests/cli_restore.rs` ✓. Redaction non-vacuous (ms1+real pp → fp `48c24d93`≠`73c5da0a`, `passphrase_applied:true`, no `correct-horse`/`ms10entr`/`abandon`/`xprv`/`tprv` in json) ✓. `--format` gate: all-4 default → ModeViolation exit 2, 0-byte stdout ✓. `--format descriptor` → `wpkh([73c5da0a/84'/0'/0']…/<0;1>/*)#hpg6d6w2` on stdout, verify-block on stderr ✓. bitcoin-core → 2-elem importdescriptors; bip388 → wallet-policy ✓. Mismatch+`--format`/`--json` (no allow) → exit 4, 0-byte stdout (no payload/no json-success) ✓. `--json` shape valid (master_fingerprint/passphrase_applied/network/verification/wallets[]; all-4→4 wallets; import_payload only w/ --format); status verified/unverified/overridden ✓. `--output` text+json → file, stdout 0 bytes; bad path → exit 1 ✓. `EmitInputs.script_type` = `wallet_export::script_type_from_template`→`WalletScriptType` (NOT convert::ScriptType) ✓. Lockstep: cli_gui_schema (4) + lint_argv_secret_flags (16) green (new flags non-secret) ✓. Full suite **0 FAILED**; clippy clean. cli_restore = 35 tests (20 P1 + 15 P2).

**Bottom line:** fold I1 (option a — mirror `collect_missing` short-circuit) + a test asserting `restore --format specter` refuses identically to export-wallet; re-dispatch R0.

---

## Round 1 — GREEN (0C / 0I)

Fold `475a5d1` (collect_missing short-circuit). **The round-0 finding's exit-code prose was WRONG** and the fold corrected it at runtime: `ExportWalletMissingFields::exit_code()` = **2** (not 64; the 64 override is clap-parse-only), and **jade was a false premise** (jade's `collect_missing` is empty by design; its single-sig refusal is an internal `emit()` `BadInput` exit 1, ALREADY at export-wallet parity). The real defect (specter emitting a placeholder-name wallet at exit 0) is fixed → now refuses at **exit 2**, byte-identical message to `export-wallet --format specter`, locked by a parity test.

- **Mirror exactness:** restore's added dispatch (`restore.rs:583-610`) is arm-for-arm identical to `export_wallet.rs:506-525` — same 11 `CliExportFormat` arms, same `ExportWalletMissingFields { format, missing }` variant (no new error), gating before `emit()`.
- **11-format emit-vs-refuse table (RUN, restore vs export-wallet, single-sig no-wallet-name):** IDENTICAL refusal set — emit(0): bitcoin-core/bip388/coldcard/sparrow/electrum/green/bsms/descriptor; refuse: coldcard-multisig/jade exit 1 (multisig-only), **specter exit 2** (missing wallet_name). No good format wrongly refuses.
- descriptor unregressed (exit 0, `…#hpg6d6w2`); watch-only-out holds (no new xprv/tprv path); 4 new tests pass; full suite 0 FAILED; clippy clean; diff = 2 files.
- **Minor (non-blocking, immutable history):** commit `475a5d1` subject says "exit 64" but actual is exit 2 — the body, in-code comment, error variant, and all tests correctly assert exit 2.

**Phase 2 GREEN — cleared for Phase 3.**
