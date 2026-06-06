# Phase 2 (GREEN) Review — timestamp-default-zero (v0.47.3)

> Persisted verbatim from the opus `feature-dev:code-reviewer` agent
> (`agentId: aefef0bcf0aefa1cb`). Bash was unavailable in the review env; the
> reviewer verified statically and relied on the operator's gate attestation
> (full suite 0 failed, clippy 0, audit GREEN — all operator-run before review).
> The single Minor was folded after this review (see operator note at bottom).

---

## Review: Phase 2 GREEN — `--timestamp` default `0` everywhere (v0.47.3)

Reviewing commits `61cbb4e` (Phase 2 GREEN) and `0624380` (Phase 1 RED) on branch `timestamp-default-zero-v0.47.3`.

### Verified Clean

**1. Source flip correctness — all 3 sites correct.**
- `crates/mnemonic-toolkit/src/cmd/export_wallet.rs:212` — `default_value = "0"` with updated doc-comment. ✓
- `crates/mnemonic-toolkit/src/cmd/restore.rs:611` — `timestamp: TimestampArg::Unix(0)` in `build_import_payload`. ✓
- `crates/mnemonic-toolkit/src/cmd/restore.rs:667` — `timestamp: TimestampArg::Unix(0)` in `build_multisig_import_payload`. ✓

Both restore sites feed through `crate::cmd::export_wallet::emit_payload(...)` which routes to `bitcoin_core.rs`'s `format_bitcoin_core_importdescriptors`. `TimestampArg::Unix(0)` renders as `json!(0)` — the integer `0`, valid for Bitcoin Core. `parse_timestamp("0")` → `Unix(0)` confirms round-trip correctness.

**2. Inventory exhaustive — no other `now`-default timestamp emitter.**
- `nostr.rs:108` already had `default_value = "0"` pre-cycle. ✓
- `bitcoin_core.rs:118` internal unit test uses `TimestampArg::Unix(0)` — unchanged. ✓
- `bundle`, `synthesize`, `bip85`: no timestamp path (confirmed by grep across all `src/cmd/*.rs`). ✓
- No other `default_value.*now` pattern exists in source. ✓

**3. Test cells discriminating and correct.**
- `cli_export_wallet.rs:127` — existing cell_1 assertion flipped to `as_u64().unwrap() == 0`. The `as_u64()` call returns `None` for the old string `"now"` (discriminating). ✓
- `cli_gui_schema_v5_extensions.rs:124` — `ts["default_value"] == "0"` (string `"0"`, not number 0); `is_string()` stays GREEN (clap `default_value` source string is preserved verbatim, so `"0"` is still a string). Test renamed `…zero_as_string`. ✓ These are the exactly-2 flips R0 predicted.
- `export_wallet_default_timestamp_is_zero_not_now` — tests all entries in the array via `as_u64()` discriminator. ✓
- `export_wallet_explicit_timestamp_now_stays_now` — guard that `--timestamp now` still emits `"now"` (string). ✓
- `restore_format_bitcoin_core_default_timestamp_is_zero` — tests the single-sig restore path. ✓
- `restore_md1_format_bitcoin_core_default_timestamp_is_zero` — tests the multisig restore path. ✓

**4. No spurious `"now"` assertions missed.**
Full grep of all test files for `"now"` found:
- `cli_export_wallet.rs` — the `"now"` references are the guard cell (correct).
- `cli_nostr.rs:193` — explicit `--timestamp now` path, correctly stays `"now"`. ✓
- `cli_import_wallet_bitcoin_core.rs:55,460` — import-input fixtures (the `"timestamp": "now"` inline fixture blob fed to `import-wallet`). Import acceptance of `"now"` unchanged and correct. ✓
- `cli_auto_repair.rs:358` — `"now"` is a `corrected_positions` object key, unrelated to timestamp. ✓
- `cli_export_wallet_from_import_json.rs:61` — `is_string() || is_number()` type-agnostic assertion stays GREEN. ✓

**5. Fixture integrity.** `tests/fixtures/wallet_import/core-bip{44,84,86}-mainnet.json` — all three still carry `"timestamp": "now"`. Import *inputs*; `import-wallet` correctly continues accepting the historical string form. Not touched. ✓

**6. Golden regen integrity.** Both `recipe-1-bsms-to-bitcoin-core.out` and `recipe-5-specter-to-bitcoin-core.out` show `"timestamp": 0` (integer, no quotes) in both receive and change entries — no extra drift. `mnemonic-export-wallet.txt` (cli-help golden, not CI-gated): lines 66-68 now show the updated doc-comment and `[default: 0]`. Targeted edit internally consistent. ✓

**7. Docs truthfulness.**
- `41-mnemonic.md:707` — row updated to "`0` (default; rescan from genesis …), `now`, or unix seconds". ✓
- `41-mnemonic.md:2301` — nostr row correctly untouched. ✓
- `37-wallet-export.md:329-335` — Timestamp bullet leads with "`--timestamp 0` (rescan from genesis)" as the default, explains `now`, includes the note "(The example above passes `now` explicitly to show the flag.)" ✓
- `restore` section of `41-mnemonic.md` does NOT imply a `now` anchor for `restore --format bitcoin-core`, so SPEC §5b's conditional note requirement is not triggered. ✓

**8. GUI cross-repo deferred correctly.** No toolkit-side code change for the D33 latent bug (SPEC §5e). FOLLOWUP `gui-timestamp-default-value-drift-v0.47.3` is a Phase 3 action. Correctly deferred. ✓

**9. Commit scope (static).** `61cbb4e` affected files match the expected Phase 2 scope. No stray edits.

Note: Bash unavailable; `cargo test`/`clippy` not run by the reviewer. Commit message records "full suite 0 failed; clippy 0; make audit GREEN (verify-examples 20/20)" — implementer attestation, corroborated by static analysis.

### Issues Found

**Minor (confidence 45) — `37-wallet-export.md:28-36`: lead-in says "default output matches" but code block uses `--timestamp now`.** "default output matches" followed by `--timestamp now` (now explicitly non-default) could mislead a skimming reader. Within the explicit SPEC allowance (the parenthetical at :335 clarifies). Cosmetic; does not block. Fix (optional): reword the lead-in or drop `--timestamp now` from the code block.

### VERDICT: 0 Critical / 0 Important (+ 1 Minor)

**Phase 2 is cleared for Phase 3 (release).**

All three source flips are correct and reach the bitcoin-core emitter. The discriminating test cells are genuine (`as_u64()` vs the old `"now"` string). The R0-predicted blast radius of exactly 2 assertion flips is confirmed. Import fixtures carrying `"now"` are untouched. The GUI cross-repo dependency is correctly deferred to a Phase 3 FOLLOWUP. The single Minor (cosmetic lead-in) is within the explicit SPEC allowance and does not affect correctness.

---

## Operator note (Minor folded) — 2026-06-06

The Minor was folded: `37-wallet-export.md:27-28` lead-in reworded from "the
toolkit's default output matches:" to "export-wallet emits exactly that shape
(the example below passes `--timestamp now` explicitly; the default is `0` —
see the Timestamp note under Tips):". `make -C docs/manual lint` re-run GREEN
(markdownlint 0, cspell 0, lychee 0 errors). Phase 2 GREEN (0C/0I) — proceeding
to Phase 3 (release).
