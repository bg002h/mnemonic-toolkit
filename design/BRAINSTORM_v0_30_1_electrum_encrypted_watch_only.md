# BRAINSTORM — `mnemonic-toolkit-v0.30.1` (electrum-encrypted watch-only passthrough)

**Date:** 2026-05-21 (Cycle 6b R0 fold rev — Path A).
**Source SHA at brainstorm time:** master HEAD `d890de4` (post-Cycle-6a artifacts + brainstorm R0 review).
**Sync state:** local master ≡ origin/master.
**v1 predecessor (Path B, ARCHIVED):** `design/BRAINSTORM_v0_31_0_electrum_encrypted_v1_path_b.md`. The v1 brainstorm assumed Cycle 6 needed to decrypt `seed`/`xprv` fields; v1 was invalidated by opus R0 (`design/agent-reports/v0_31_0-brainstorm-r0-review.md` §C1) which observed the Electrum parser never reads those fields.
**R0 review:** `design/agent-reports/v0_31_0-brainstorm-r0-review.md` (RED — re-brainstorm; Path A recommended).
**Kickoff (still load-bearing for 6b execution discipline):** `design/CYCLE_6_KICKOFF.md`.

## R0 fold: Path A pivot

Opus R0 caught a foundational design error in v1:

> The Electrum parser reads `keystore.xpub` / `keystore.derivation` / `keystore.root_fingerprint` / `keystore.label` (singlesig at `wallet_import/electrum.rs:494,504,514,531`) and `xN/.xpub` / etc. (multisig at `:778-816`). All of these are PLAINTEXT even in encrypted Electrum wallets. The encrypted fields (`seed` / `xprv` / `passphrase` / `keypairs`) live in `keystore.*` but are NEVER read by the parser. The parser-needed field set ∩ encrypted-field set = ∅.

This invalidates v1's premise. The encrypted-wallet refusal at `electrum.rs:305-313` is **over-restrictive in principle** — the wallet's watch-only public-key material is already accessible WITHOUT a password.

**Path A locked (user direction, Cycle 6b R0 fold):**

- Drop `--decrypt-password*` flag family entirely.
- Downgrade the L305-313 refusal: parse plaintext xpub/derivation/fingerprint/label and IGNORE the encrypted `seed` / `xprv` / `passphrase` / `keypairs` fields. Emit stderr advisory describing the watch-only-passthrough semantic.
- Ship as **PATCH `v0.30.0 → v0.30.1`** (no new CLI surface; behavior expansion only). NO GUI lockstep (no schema-mirror delta).
- 6a-shipped `electrum_crypto.rs` becomes an **internal/unused-by-CLI** library module. Filed forward as FOLLOWUP `electrum-crypto-seed-extraction-subcommand` for a future cycle's seed-extraction CLI surface.

## Cycle slug rename

- **Original 6a slug:** `wallet-import-electrum-encrypted` (parent FOLLOWUP, open).
- **Cycle 6b slug:** `wallet-import-electrum-encrypted-watch-only-passthrough` (Path A surface). Resolves the parent FOLLOWUP by reinterpreting the user-need: importing an encrypted Electrum wallet for watch-only purposes does NOT need decryption.

## Decisions locked

1. **Surface change:** `wallet_import/electrum.rs` L305-313 refusal becomes a stderr advisory: `"import-wallet: electrum: wallet is encrypted (use_encryption=true); importing watch-only material only (encrypted seed/xprv/passphrase/keypairs fields ignored). To extract the encrypted seed, use 'electrum --decrypt-wallet' out-of-band then re-import the plaintext wallet."`
2. **NO CLI flag additions.** `cmd/import_wallet.rs` unchanged. No `--decrypt-password*` family. No `secrets.rs` updates.
3. **Field set ignored when use_encryption=true:** `keystore.seed`, `keystore.xprv`, `keystore.passphrase`, `keystore.keypairs` (singlesig) + same paths under each `xN/` (multisig). These are passed through to the parser ONLY as plaintext echoes — i.e., the parser reads only the plaintext xpub/derivation/fingerprint/label paths as before. If `keystore.xpub` is somehow absent (which would be a malformed wallet — Electrum always emits xpub plaintext even when encrypted), the parser's existing "missing or non-string" error fires.
4. **Test surface:** new integration tests covering (a) singlesig encrypted-wallet imports watch-only successfully; (b) multisig encrypted-wallet imports watch-only successfully; (c) stderr advisory text byte-matches; (d) the parser's existing plaintext-keystore-xpub-required refusal still fires if `xpub` is absent.
5. **`electrum_crypto.rs` disposition:** keep the library module in-tree (no revert of 6a's Phase 1 work). Mark as `#[allow(dead_code)]` IF the compiler complains about unused code (currently the tests reference all public items, so compilation is clean). File new FOLLOWUP `electrum-crypto-seed-extraction-subcommand` describing future use case (e.g., new `mnemonic convert --from electrum-encrypted-wallet --to phrase` subcommand) that would consume the library.
6. **SemVer:** PATCH `v0.30.0 → v0.30.1`. No CLI surface change; behavior expansion is strictly more accepting (formerly refused inputs now succeed with stderr advisory).
7. **GUI lockstep:** NOT MANDATORY. `gui-schema` JSON output is unchanged (no new clap surface). The `schema_mirror` gate continues to pass against the v0.30.1 toolkit pin without GUI changes. GUI consumer of v0.30.1 inherits the looser behavior automatically (no opt-in required).
8. **FOLLOWUP body update:** at Cycle 6b close, update `design/FOLLOWUPS.md`'s `wallet-import-electrum-encrypted` entry: (a) status → resolved (watch-only-passthrough); (b) correct "PBKDF2 + AES-CBC" claim to "sha256d + AES-256-CBC" with citation to `design/cycle-6-p0-recon.md` §A1; (c) note that field-level decryption was determined to be unnecessary for watch-only import per opus R0 at Cycle 6b execute-start.

## Folded R0 findings

| R0 finding | Fold action |
|---|---|
| C1 — parser doesn't read encrypted fields | Path A locked. `--decrypt-password*` family dropped. Refusal downgraded to advisory. |
| C2 — field-set citation wrong + missing variants | Resolved by Path A: the field set is now "ignored fields" not "decrypted fields"; full set documented at decision item 3. |
| I1 — password-validation strategy unspecified | N/A under Path A (no password). |
| I2 — 3-form `--decrypt-password*` net-new | N/A under Path A (no password flags). |
| I3 — FOLLOWUP body update under-specified | Resolved via decision item 8 (lock all three fold elements: status, scheme citation, R0-correction note). |
| I4 — brainstorm needs revising post-R0 | This brainstorm IS the rev (v2 = `BRAINSTORM_v0_30_1_electrum_encrypted_watch_only.md`; v1 archived). |
| I5 — SemVer re-derivation | Resolved: PATCH `v0.30.1` (no CLI surface change). |
| M1 — derive_key rustdoc | N/A under Path A (electrum_crypto.rs unused-by-CLI; library is correct as-is). |
| M2 — wrong-password test wider-than-needed match | N/A under Path A (no wrong-password code path in CLI). Test stays defensive in-library. |
| M3 — `aes` transitive-via-bitcoin claim | Cosmetic recon framing; no impact under Path A (cbc + base64 already added; aes promoted to direct dep is a no-op as locked). |

## Filename rename (post-cycle artifact)

This brainstorm is named `BRAINSTORM_v0_30_1_electrum_encrypted_watch_only.md` (Path A target version). The original `BRAINSTORM_v0_31_0_electrum_encrypted.md` was renamed to `BRAINSTORM_v0_31_0_electrum_encrypted_v1_path_b.md` to archive the Path B framing. The version-number-in-filename mismatch (v0.31.0 vs v0.30.1) reflects the SemVer reclassification: Path B was MINOR (v0.31.0); Path A is PATCH (v0.30.1).

## Cycle 6b deliverables (Path A — THIS continuation)

1. ✓ R0 brainstorm review (this rev IS the R0 fold).
2. Plan-doc: `design/PLAN_mnemonic_toolkit_v0_30_1.md` (compact; 3-4 phases instead of v1's 6).
3. Phase 2: refactor `electrum.rs:305-313` refusal → advisory.
4. Phase 3: integration tests covering encrypted-watch-only happy paths.
5. Phase 4: manual chapter update (chapter-45 §"Encrypted Electrum wallets" rewrite + chapter-41 §`mnemonic import-wallet` exit-code/refusal-template addendum).
6. Phase 5: cycle close (Cargo.toml v0.30.0 → v0.30.1 + install.sh + CHANGELOG + tag + push + install-pin-check CI + GH Release).
7. Phase 6: FOLLOWUP closure (`wallet-import-electrum-encrypted` → resolved-watch-only-passthrough) + file new FOLLOWUP `electrum-crypto-seed-extraction-subcommand` + file new FOLLOWUP `wallet-import-electrum-encrypted-storage-format-b` (Format B whole-file encryption, still open).

## Cross-cutting

- **No `ToolkitError` variants added.** `ElectrumDecryptError` from 6a remains library-local + unused-by-CLI.
- **No `secrets.rs` updates.** No new secret-bearing flags.
- **install.sh self-pin:** v0.30.0 → v0.30.1 at Cycle 6b Phase 5.
- **Manual chapter-45 §"Encrypted Electrum wallets" rewrite:** drops the "deferred" framing; documents the watch-only-passthrough semantic + out-of-band `electrum --decrypt-wallet` workflow for users who want seed material.
- **Chapter-41 §`mnemonic import-wallet` addendum:** new sub-subsection or stderr-template-list addition documenting the watch-only-passthrough advisory.

## Memory entries consulted

- `project_v0_31_0_cycle_6a_shipped` — Cycle 6a context (electrum_crypto.rs shipped; brainstorm + plan-doc R0-pending).
- `project_v0_30_0_cycle_shipped` — Cycle 5 (most recent ship).
- `feedback_r0_must_read_source_off_by_n` — R0 reviewer reads source ground truth; Path A pivot is the canonical example.
- `feedback_architect_must_run_prose_commands` — manual chapter command-blocks must be run locally.
- `feedback_no_parallelism_for_code_generation` — subagent dispatch hygiene.

## Open questions for plan-doc R0

None at brainstorm-rev time. All architectural decisions locked under Path A. Plan-doc R0 should verify:
- The `keystore.xpub` field IS plaintext even when `use_encryption: true` (claim from R0 §C1; we trust Electrum's keystore.py per opus's read but Phase 3 implementer should verify against a real encrypted-wallet fixture if one is constructable).
- The `electrum.rs` parser doesn't have OTHER refusal sites that would still block encrypted wallets (the L305-313 refusal is the documented gate; verify no other `use_encryption` references exist in the file or in `sniff.rs`).
- The stderr advisory text format matches existing toolkit precedent (`secret_in_argv_warning`-style template).
