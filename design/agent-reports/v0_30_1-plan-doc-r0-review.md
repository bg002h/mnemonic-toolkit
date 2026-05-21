# v0.30.1 plan-doc R0 review

**Reviewer:** opus
**Round:** R0
**Plan under review:** design/PLAN_mnemonic_toolkit_v0_30_1.md
**Brainstorm v2:** design/BRAINSTORM_v0_30_1_electrum_encrypted_watch_only.md
**Date:** 2026-05-21
**Source SHA:** faa037a (master HEAD)

## Critical (C)

None. Plan implements brainstorm v2 cleanly. Phase 2 Step 1/2 line-citations grep-verify: `electrum.rs:305-313` is exactly the `if use_encryption { return Err(...) }` block; `:47` is the module-doc Refusals line `Refusals (`2fa` / `imported` / `use_encryption: true`) per SPEC §11.6.1.`; `:258` is the parse-fn docstring step-3 line. Phase 6 line citation at L2572-2583 verifies (slug header at L2573). Brainstorm-locked stderr advisory text matches Phase 2 implementation, Phase 3 test substrings ("wallet is encrypted", "watch-only material only", "electrum --decrypt-wallet"), Phase 4 manual prose, and Phase 5 CHANGELOG byte-for-byte. SemVer-PATCH classification holds (no clap surface, no JSON envelope shape, no flag rename). Out-of-scope invariants honored (no `cmd/import_wallet.rs`, no `secrets.rs`, no `ToolkitError` variants, no new Cargo deps, no `electrum_crypto.rs` modification).

## Important (I)

- **I1 — Phase 2 Step 5 misses the integration-test refusal cell.** The lib test `parse_use_encryption_refuses_with_specific_message` at `crates/mnemonic-toolkit/src/wallet_import/electrum.rs:1335` is covered by Phase 2 Step 5's grep, but the integration cell `electrum_encrypted_fixture_refuses_with_decrypt_wallet_message` at `crates/mnemonic-toolkit/tests/cli_import_wallet_electrum.rs:154` (consuming the existing `electrum-encrypted-refused.json` fixture) is NOT — Step 5's grep is scoped to `src/wallet_import/electrum.rs` only. This cell asserts `.failure()` + "encrypted" + "decrypt-wallet" stderr substring and WILL break under Phase 2. Fix: expand Step 5 grep to also search `crates/mnemonic-toolkit/tests/cli_import_wallet_electrum.rs` (and update or delete that cell + decide existing-fixture disposition: keep it as a "minimal happy-path-with-advisory" cell or rename to `electrum-encrypted-watch-only.json`).

- **I2 — Phase 4 chapter-45 rewrite misses two of three encrypted-Electrum mentions.** Beyond the §"Encrypted wallets" subsection at L667-682, chapter-45 has stale "deferred" framing at L718-722 (§Deferrals bullet referencing the FOLLOWUP) AND at L791-793 (§"What's NOT supported" entry). Phase 4 Step 2 prescribes rewriting only one location. The other two will leave the manual self-contradictory (one section says watch-only-passthrough; the other two still say "refused / deferred"). Fix: grep `docs/manual/src/45-foreign-formats.md` for `wallet-import-electrum-encrypted` + `use_encryption` and rewrite all three sites in lockstep; the §"What's NOT supported" entry needs deletion-or-strikethrough (analogous to v0.30.0's `~~Jade SeedQR variant~~` pattern at L790).

- **I3 — Phase 3 fixture name collides with existing fixture.** `electrum-encrypted-refused.json` already exists at `crates/mnemonic-toolkit/tests/fixtures/wallet_import/`. Phase 3 Step 2.5 prescribes new fixtures with different names (`electrum-encrypted-singlesig-watch-only.json`, etc.) — no collision, but the existing `electrum-encrypted-refused.json` is now misnamed (it will pass under v0.30.1, not refuse). Fix: either delete the existing fixture and remove the integration cell at I1, or rename + repurpose it as one of the new watch-only fixtures (and update the test at cli_import_wallet_electrum.rs:154 to assert success + stderr advisory rather than failure). Plan should make this explicit.

- **I4 — Phase 4 Step 3 prescribes wrong markdown shape for chapter-41 stderr-templates list.** Plan says "add to stderr-template list" as a dash-bullet; the actual structure at `docs/manual/src/40-cli-reference/41-mnemonic.md:771-784` is a `| Class | Template |` markdown table. Implementer needs to add a `| NOTICE (exit 0) | \`notice: import-wallet: electrum: ...\` |` row. (Plan-doc's bullet syntax would visually break the table.) Also: the advisory does NOT use a `warning:`/`notice:` prefix in Phase 2's text; either add a `notice:` prefix to the advisory (cosmetically consistent with the table's other NOTICE rows) or pick a different table column structure — but pick deliberately.

## Minor (M)

- **M1 — Phase 3 fixture sample uses `seed_version: 41`.** Existing plaintext fixtures use `seed_version: 17`; sniff accepts 11..71+ so 41 works, but using 17 is more consistent and avoids drawing reviewer attention to whether `41` was a typo for the expected range. Cosmetic.

- **M2 — Phase 2 Step 2 advisory uses Rust string-continuation backslashes** which embed a literal newline-and-leading-whitespace pair into the stderr output if not careful. Rust's `\` line-continuation in string literals DOES eat the leading whitespace of the next line, so the output is a single-space-joined string — this works correctly, but the plan should explicitly note this for the implementer (and the Phase 4 manual code block already shows the line-wrapped form, which won't byte-match the actual single-line stderr output). The Phase 4 Step 5 "byte-match prose against actual command stderr" check needs to compare against a single-line normalization, not the wrapped manual prose.

- **M3 — CHANGELOG `2026-MM-DD` placeholder** noted self-reviewed but should bind the date explicitly to "ship day, post-tag-push" rather than plan-write day; otherwise risk of stale-date drift if Phase 5 is held for any reason.

## Verdict

YELLOW — fold then proceed.

The plan is architecturally sound and faithfully implements the brainstorm v2 Path A pivot, with all line citations grep-verified at HEAD. SemVer-PATCH classification + invariants (no clap surface, no GUI lockstep, no new deps/variants) all hold. Four Important findings — I1 (orphan integration-test cell at `cli_import_wallet_electrum.rs:154`), I2 (two additional stale manual-prose sites at `45-foreign-formats.md:718-722,791-793`), I3 (existing `electrum-encrypted-refused.json` fixture disposition), and I4 (chapter-41 stderr-templates table-row shape) — require fold but are mechanical. Recommend folding all four Importants into the plan-doc, persisting this review verbatim at `design/agent-reports/v0_30_1-plan-doc-r0-review.md`, and proceeding to Phase 2 dispatch.
