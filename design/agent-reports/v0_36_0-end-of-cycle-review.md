# v0.36.0 — End-of-cycle architect review (opus) — MANDATORY pre-tag gate

**Date:** 2026-05-23
**Cycle:** v0.36.0 `decode-address` + `verify-message` (legacy P2PKH + BIP-322) + convert/electrum lock-tests
**Branch:** `v0.36.0-verify-decode-address`
**Reviewer:** opus (feature-dev:code-reviewer), end-of-cycle (agentId a532ba0d9890ddf1e)
**Scope:** whole-cycle diff `origin/master..HEAD` (23 files, ~1481 ins; commits 53b3ebb..f9a7344) + live source.

---

## Critical
None.

## Important
None.

## Minor (all observations / already-filed — none block the tag)
- **`format_requested` JSON coupled to enum Debug** (`cmd/verify_message.rs:112`). Correct today; filed as FOLLOWUP `verify-message-format-requested-debug-string`. Conf 85.
- **electrum→address refusal wording imprecise** (`convert.rs:460` shared barrier; lock-test only asserts `contains("electrum-phrase")`). Deferred per R0 disposition (b); filed as `electrum-phrase-address-refusal-honest-wording`. Conf 80.
- **`cli-subcommands.list` still omits `electrum-decrypt` + `seedqr`** — PRE-EXISTING (predates this cycle), out of scope; both NEW v0.36.0 subcommands ARE listed. Not a regression. Conf 90.

## Verification summary (all 7 gate items VERIFIED)
1. **Code correctness:** auto-dispatch partition is the exact complement of bitcoin 0.32 P2PKH-only `is_signed_by_address`; C1 `catch_unwind` isolation correct (take/silence/restore hook around AssertUnwindSafe closure → unwind maps to clean `VerifyMessage` err); regression test crafts a curve-valid uncompressed pubkey reaching `wpubkey_hash().unwrap()`; thread-safety caveat documented + accurate. `decode_address` total (only `parse` fallible → `DecodeAddress`; `AddressType` non_exhaustive → "unknown" fallback). error.rs variants alphabetical, exit 1, kind strings, no details arm. main.rs dispatch correct (Ok(1) decoded-but-invalid → no stderr error).
2. **Version/release:** Cargo.toml 0.36.0; Cargo.lock mnemonic-toolkit 0.36.0; install.sh:32 self-pin `mnemonic-toolkit-v0.36.0` (matches install-pin-check tag compare); CHANGELOG [0.36.0] accurate; SemVer MINOR correct.
3. **Lockstep:** cli_gui_schema.rs lists both + count comment + :102 assert-message all 25; manual chapters (`41-mnemonic.md:2095`,`:2119`) document every flag; cli-subcommands.list:27-28 lists both; intro "Fourteen subcommands":3 matches; toolkit gui-schema complete (CommandFactory walk; positionals filtered by existing pattern).
4. **Dep hygiene:** `bip322 = "=0.0.10"` exact-pinned, crate name `bip322`; single bitcoin entry in Cargo.lock (0.32.8), no duplicate; transitive `snafu` benign.
5. **Secret taxonomy:** `flag_is_secret` matches none of the new flags (public-by-design); no taxonomy entry; argv-leak lint count (28) unaffected; no GUI secret-projection delta.
6. **Test integrity:** BIP-322 = genuine mediawiki vectors; legacy = self-generated RFC6979 (SecretKey only in #[cfg(test)]); C1 regression genuine; cli_decode_address (5) + cli_verify_message (8) cover exit-codes/ArgGroup/stdin-strip/json. convert+electrum lock-tests present/correct.
7. **Clean-tag blockers:** NONE. No debug/TODO/dead-code; 5 FOLLOWUPs well-filed; plan-doc present; safety-lint scope unaffected (test-only SecretKey excluded).

Reportedly: full suite 2348 pass / 0 fail; clippy clean; manual lint 6/6.

VERDICT: GREEN (0C/0I)

---

## Controller note
GREEN → gate satisfied; no Critical/Important. The three Minors are observations: two are already FOLLOWUP-filed (format-debug, electrum-wording), one is pre-existing out-of-scope (cli-subcommands.list electrum-decrypt/seedqr omission — could be swept in a future docs-hygiene cycle). No fold needed. Proceeding to tag/ship toolkit, then GUI lockstep (Phase 5).
