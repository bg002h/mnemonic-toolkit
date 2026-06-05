# mnemonic restore — Phase 1 R0 Review (single-sig core)

**Verdict: GREEN (0 Critical / 0 Important).** Cleared for Phase 2. Reviewer (opus, full shell) ran the complete security + functional + regression gate at runtime.

Commits: `10ad466` (RestoreMismatch + scaffold + gui-schema 28→29), `f204514` (derivation→fp+descriptor+addr), `7aedc04` (verify-gate + advisory). 8 files; `cli_restore.rs` = 20 tests.

## Critical / Important
None.

## Minor (cosmetic; not folded — for the record)
1. **ms1 `--language` conflict message borrows "slot @0" framing** (`restore.rs:206` via `slot_ms1.rs`). Pre-acknowledged SPEC M-idx; still actionable (names the conflicting languages + fix). Acceptable.
2. **`template_label` fallthrough → `"multisig"`** (`restore.rs:417`) is unreachable (multisig `--template` rejected at `:136-142` before any row is built). Harmless; a `debug_assert!`/`unreachable!` would be marginally safer. Not required.

## Verification ledger (every item RUN)
**Gates:** `cargo build` clean; `cargo test -p mnemonic-toolkit --no-fail-fast` FAILED-count **0** (`cli_restore` 20, `lint_argv_secret_flags` 16, `cli_gui_schema` 4 all ok); `clippy --all-targets -D warnings` exit **0**. Diff = exactly 8 files; NO gui/version/Cargo.lock/docs.

**WATCH-ONLY-OUT (security):** derivation uses `Secp256k1::verification_only()` (`restore.rs:255`); `account_xpriv` never referenced (slot built `entropy:None`); runtime default+passphrase+testnet runs → `xprv`/`tprv`/`yprv…`/WIF/`private key` = **0 hits** (emits `tpub` on testnet). Non-vacuous: `DerivedAccount.account_xpriv: Xpriv` exists + `account_xpriv.to_string()→xprv…` (`convert.rs:1218`), so the leak tokens are correct; `restore_emits_no_private_key_material` exercises a live-xpriv path + greps both streams for both prefixes across two arg-sets.

**RestoreMismatch:** enum slot `:279` (RepairShortCircuit `:274` < RestoreMismatch < SilentPayment `:289`); `exit_code`→4 (`:528`); `kind`→PascalCase `"RestoreMismatch"` (`:589`, passes `kind_strings_stable` `:1203`); `message`→`restore:`-prefixed (`:766`); NO `details()` arm (`_=>None`). Exhaustive build OK.

**Mismatch policy (exit codes RUN):** expect-fingerprint match→0; mismatch→**4 + `wpkh(` count 0** (no descriptors) + `✗ MISMATCH`; mismatch+`--allow-mismatch`→0 + `wpkh(` 1 + `MISMATCH (overridden)`; no-reference→0 + `UNVERIFIED`; `--expect-xpub` w/o `--template`→**2** (ModeViolation).

**Input resolution:** non-seed `--from xpub=`→1 (BadInput); stdin-mutex→1; `@env:`+`--passphrase-stdin`→0 derives `b4e3f5ed`; ms1 Japanese `mnem`→`0ed2c5a4` (NOT `73c5da0a`), `--language english` conflict→**2** (SlotInputViolation); routed via `slot_ms1::resolve_ms1_slot`/`.derive_language`. mlock + argv-warn present.

**Derivation core:** multisig `--template wsh-sortedmulti`→1 (BadInput); `convert::script_type_from_template` pub(crate) (not over-exposed); `ResolvedSlot` all-7-fields watch-only; bip84 descriptor exact `wpkh([73c5da0a/84'/0'/0']xpub6CatW…/<0;1>/*)#hpg6d6w2`; all-4 default emits bip44/49/84/86 (same fp); first recv `bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu`; `--count 2`→2 addrs.

**Lockstep `lint_argv_secret_flags.rs`:** `restore --passphrase`/`--from` routes added — OBLIGATED (the `flag_axis_set_equals_gui_schema`/`from_axis_set_equals_gui_schema` set-equality closures would fail otherwise), a correct leading addition (not gold-plating); evidence anchors present (`passphrase-stdin`, `=-`, `from.value == "-"` `:122`); `flag_is_secret` unchanged (`--passphrase*` already secret). Gate now complete — no P3 residue.

**gui-schema 28→29:** `:74/:108` bumped, `"restore"` alpha slot; test green; runtime gui-schema emits `restore` 12 flags (`--passphrase*` secret, `--from` not).

**Test quality:** 20 meaningful (exact string asserts, not vacuous substrings); `b4e3f5ed` **re-derived** at runtime via independent `convert` path then asserted (`feedback_recapture_golden_only_when_current_correct`); Japanese-ms1 negative non-vacuous (asserts `0ed2c5a4` present AND `73c5da0a` absent).

**Bottom line:** GREEN 0C/0I. Cleared for Phase 2 (import formats + --json).
