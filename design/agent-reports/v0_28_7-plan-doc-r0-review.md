# v0.28.7 plan-doc R0 architect review — opus

**Reviewer:** opus
**Plan-doc:** `design/PLAN_mnemonic_toolkit_v0_28_7.md`
**Date:** 2026-05-20

## Verdict: YELLOW

The plan-doc is structurally sound on architecture (P0 locks correctly applied; Slug 1/2/4 source citations grep-clean) but carries one Critical issue (Task 3 fixture-path fabrication) and one Important issue (Task 1 Step 6 reasoning is technically wrong, though the resulting code is still defensible as defense-in-depth).

## Critical (must fold before proceeding)

### C1 — Task 3 Step 5: 11 of 17 fixture paths do not exist

Plan-doc invents fixture filenames that follow a non-existent naming convention. Grep against `crates/mnemonic-toolkit/tests/fixtures/wallet_import/` (64 files) yields:

- `coldcard-singlesig-bip84-mainnet.json` — EXISTS (cited 3x)
- `coldcard-multisig-mainnet.json` — **MISSING** (cited 3x; Coldcard multisig fixtures are `coldcard-ms-*.txt`, not `.json`)
- `electrum-singlesig-mainnet.json` — **MISSING** (actual: `electrum-standard-bip{49,84}-mainnet.json`)
- `jade-singlesig-mainnet.json` — **MISSING** (only `jade-multisig-*` and `jade-singlesig-refused.json` exist — Jade is a multisig-only target)
- `sparrow-singlesig-mainnet.json` — **MISSING** (actual: `sparrow-singlesig-p2wpkh.json`)
- `specter-singlesig-mainnet.json` — **MISSING** (actual: `specter-singlesig-p2wpkh.json`)

Per memory `feedback-r0-must-read-source-off-by-n` and `feedback-architect-must-run-prose-commands` — implementer following this verbatim will hit 11/17 cell failures at Task 3 Step 6 (`cargo test ... matrix`), with non-obvious "file not found" panic.

## Important (fold inline)

### I1 — Task 1 Step 6: regex "bug" being fixed is not actually a bug

Plan-doc L136 claims: "the existing regex `sortedmulti\(` DOES match `sortedmulti_a(` because `_` is a word boundary and `\(` is just literal parenthesis." This is **wrong**. The regex `(?:thresh|multi|sortedmulti)\((\d+)\s*,` requires literal `(` immediately after `sortedmulti` (or `multi` or `thresh`). For `sortedmulti_a(2,...)`, the char after `sortedmulti` is `_`, not `(` — the alternation does not match. Source comment at `tests/cli_import_wallet_bsms.rs:961-966` and assertion at L984-988 (`threshold=none`) confirm this is the current observed reality.

The defense-in-depth code (the `sortedmulti_a(` substring check returning `Err(BsmsTaprootImportRefused)`) is still defensible — it converts an `Ok(None)` no-match into an explicit refusal — but the plan-doc's narrative path to it is wrong and will confuse the implementer.

**Fold:** Rewrite Step 6 prose to: (a) acknowledge the regex correctly does NOT match `sortedmulti_a(` today (cite source-comment at `tests/cli_import_wallet_bsms.rs:961`); (b) frame the defense-in-depth as "convert silent `Ok(None)` to explicit `Err(BsmsTaprootImportRefused)` for `sortedmulti_a` / `multi_a` substrings, defending against any future code path that bypasses parse-entry refusal"; (c) drop the 4 false-start regex variants and lead with the 2-stage form as the single authoritative proposal.

### I2 — Task 4 Step 2: string-sniff weakness; prefer parse-side detection

`canonical_descriptor_body.starts_with("tr(")` misses whitespace prefixes. Cleaner: put the refusal AFTER `script_type_from_descriptor(&parsed_ms)` at L612 and check `matches!(script_type, WalletScriptType::P2tr | WalletScriptType::P2trMulti)`. This uses parse-side detection rather than string-sniff, and `script_type` is already in scope.

### I3 — Task 1 Steps 1-4: alphabetical insertion direction reversed

Plan-doc says "Insert it AFTER `BsmsTaprootRefused`" — this **violates** CLAUDE.md alphabetical-by-variant-name. `BsmsTaprootImportRefused` < `BsmsTaprootRefused` (`I` < `R`). Correct insertion is BEFORE `BsmsTaprootRefused`. Same correction applies to Steps 2/3/4 ordering (Display / exit_code / kind blocks).

## Minor (defer to implementer)

- **M1.** Task 1 Step 5: `blob_descriptor_text` placeholder; actual variable is `descriptor_body: &str` at `bsms.rs:114` (also L132, L148 per-branch). Plan-doc flags this as implementer-substitution.
- **M2.** `WalletScriptType` variant names confirmed at `wallet_export/mod.rs:163-166`: `P2shMulti | P2shP2wshMulti | P2wshMulti | P2trMulti`. Plan-doc proposal matches; **VERIFIED**.
- **M3.** Task 4 Step 1 cites "envelope-gate at `cmd/export_wallet.rs:650`" — verified.
- **M4.** `BsmsParser::parse` has three branch arms (2/4/6-line) sharing one descriptor_body afterwards — implementer should know.

## Cross-cutting observations

- Test-count math: 2008 + 20 = 2028 verified. ✓
- CHANGELOG accuracy: matches plan-doc body; no grep-faults.
- No GUI lockstep: correct.
- install.sh pin bump: correctly scoped at L32.
- Slug 3 NEW FOLLOWUP filing: discovered-gaps counts (Coldcard 2 + Sparrow 4 + Specter 3 + Electrum 1 = 10) match plan-doc Task 7 Step 4 text.
- P0 STRICT-GATE lock honor: all four locks (Slug 1 α, Slug 3 B, Slug 4 B-α) honored.

## Path to GREEN

Fold C1 (rewrite Task 3 fixture paths against actually-existing files) + I1 (rewrite Step 6 narrative) + I2 (use `script_type` post-parse instead of string-sniff for Slug 4) + I3 (flip insertion direction in Task 1 Steps 1-4). Re-dispatch reviewer for R1; expected GREEN after these 4 folds.
