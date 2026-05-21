# Cycle 6 kickoff — `mnemonic-toolkit-v0.31.0` (electrum-encrypted)

**Created:** 2026-05-21, at the Cycle 6a/6b split point. Read this file first when resuming Cycle 6b in a fresh session.

## Where we are

- **Cycle 5 SHIPPED 2026-05-21:** `mnemonic-toolkit-v0.30.0` (`56dd2b6`) + `mnemonic-gui-v0.15.0` (`5582e22`). SeedQR encode/decode subcommand. See `project_v0_30_0_cycle_shipped` memory.
- **Cycle 6a SHIPPED in same session:** Phase 0 recon + brainstorm + plan-doc + Phase 1 (crypto primitives + unit tests). Artifacts committed to master.
- **Cycle 6b remains:** Phase 2 (CLI plumbing) → Phase 3 (parser integration) → Phase 4 (CLI integration tests) → Phase 5 (manual chapter) → Phase 6 (cycle close) → Phase 7 (GUI lockstep + FOLLOWUP closure).

## Cycle 6 scope (locked at brainstorm)

- **Slug:** `wallet-import-electrum-encrypted` (parent FOLLOWUP, open).
- **Shape:** decrypt Electrum field-level encryption (`use_encryption: true` wallet JSON with base64 ciphertext fields) at parse time. Plaintext flows through the existing Electrum parser path.
- **Scheme:** `sha256d(password) + AES-256-CBC + PKCS7 + base64` (Electrum's Format A; verified against `electrum/crypto.py::_pw_decode_raw`). NO PBKDF2 (the FOLLOWUP body's "PBKDF2 + AES-CBC" claim was wrong — corrected at Cycle 6 P0 recon).
- **Out of scope (deferred to follow-on FOLLOWUP):** Format B (whole-file storage encryption with version-byte + 4-byte MAC). The current Electrum refusal at `wallet_import/electrum.rs:305-313` fires on JSON-parseable wallets with `use_encryption: true`; whole-file encrypted wallets are NOT JSON-parseable so they take a different code path (likely fail at JSON parse with a different error message).
- **CLI surface:** 3-form password input flags on `mnemonic import-wallet`:
  - `--decrypt-password <VAL>` (inline; emits `secret_in_argv_warning`; classified secret in `flag_is_secret`).
  - `--decrypt-password-file <PATH>` (reads from file; emits `warn_if_world_readable` if applicable).
  - `--decrypt-password-stdin` (boolean; reads from process stdin).
- **GUI lockstep:** MANDATORY. New schema-mirror entries; paired `mnemonic-gui-v0.16.0`.
- **SemVer:** MINOR (`v0.30.0` → `v0.31.0`). Per parent brainstorm pre-locked rule: "password-on-argv is MINOR per architect I3 policy IF passed inline". User picked 3-form (inline included).

## Cycle 6a deliverables (THIS session, complete)

1. `design/CYCLE_6_KICKOFF.md` (this file).
2. `design/cycle-6-p0-recon.md` — empirical Electrum-source recon.
3. `design/BRAINSTORM_v0_31_0_electrum_encrypted.md` — architectural locks + scope split.
4. `design/PLAN_mnemonic_toolkit_v0_31_0.md` — phase decomposition + bite-sized tasks.
5. `crates/mnemonic-toolkit/src/electrum_crypto.rs` — library: `decrypt_field` primitive + library-local `ElectrumDecryptError` enum + ~20 unit cells (covering: known-vector decrypt happy path; PKCS7 unpadding refusal; wrong-key refusal; base64-decode refusal; wrong-IV-length refusal; UTF-8-decode refusal; sha256d key-derivation determinism).
6. Cargo.toml: new deps `aes = "0.8"`, `cbc = "0.1"`, `base64 = "0.22"` (verify version pins against transitive resolves).
7. `lib.rs` doc-comment block: append `electrum_crypto` to the lib-local-error sibling list.

**NOT shipped in 6a:** version bump, install.sh self-pin, CHANGELOG entry, GUI changes, parser integration, CLI flags, manual chapter. All deferred to 6b.

## Cycle 6b resume prompt

After `/clear` (or in a fresh session), issue this prompt:

```
Read design/CYCLE_6_KICKOFF.md and proceed with Cycle 6b (electrum-encrypted parser integration + CLI + ship). Use the same discipline as Cycle 5 (P0 STRICT-GATE recon refresh → plan-doc R0+R1 opus review if changes since 6a → subagent-driven implementation → opus end-of-cycle review → split commits if mechanical + version-bump bundled → install-pin-check CI gate on tag push). Cycle 6a shipped electrum_crypto.rs + 20 unit cells; 6b ships the parser integration through to v0.31.0 tag + GUI v0.16.0 lockstep + FOLLOWUP closure.
```

## Phase 2-7 detail (deferred to 6b)

### Phase 2 — CLI plumbing in `cmd/import_wallet.rs`

- Add `--decrypt-password <VAL>` + `--decrypt-password-file <PATH>` + `--decrypt-password-stdin` (boolean) clap-derive args to `ImportWalletArgs`.
- Mutex group: at most ONE of the three may be set per invocation.
- Resolve to a `Zeroizing<Vec<u8>>` password buffer at the entry of the import-wallet run() function.
- `secret_in_argv_warning` for inline form.
- `warn_if_world_readable` for `--decrypt-password-file`.
- `secrets.rs::flag_is_secret`: add `"--decrypt-password"` (unconditionally secret). `-file`/`-stdin` variants are NOT secret flag-level (their VALUES are; classification at the flag-name level is for the inline form only).

### Phase 3 — parser integration in `wallet_import/electrum.rs`

- Pre-parse hook: if JSON has `use_encryption: true` AND a decrypt-password is supplied (threaded through the orchestrator), decrypt sensitive fields via `electrum_crypto::decrypt_field(b64_ciphertext, &password)`.
- The decryption happens at the orchestrator level in `cmd/import_wallet.rs`, BEFORE calling `ElectrumParser::parse(blob, stderr)`. The orchestrator decrypts the JSON in-place (replaces ciphertext field values with plaintext) and then passes the de-encrypted JSON blob to the existing parser.
- If `use_encryption: true` AND no password: the existing refusal fires (with updated stderr template pointing at `--decrypt-password*`).
- If `use_encryption: false` AND password supplied: stderr advisory "wallet is not encrypted; --decrypt-password* ignored" (warn but don't refuse).

### Phase 4 — CLI integration tests

- ~30 cells in `tests/cli_import_wallet_electrum_encrypted.rs`.
- Test fixtures: generate via known Electrum vectors (use `electrum_crypto::encrypt_field` helper — symmetric inverse — to create fixtures from known plaintexts + passwords).
- Refusal classes: wrong password (decrypts to garbage / PKCS7 fails) → exit 1; missing password → existing refusal (updated message); both-inline-and-file → exit 64 (clap conflict).

### Phase 5 — Manual chapter

- New subsection under chapter-41 `## \`mnemonic import-wallet\`` for the new flags.
- Worked example: encrypt a known-plaintext wallet via Python `pw_encode_bytes`; verify `mnemonic import-wallet --format electrum <enc.json> --decrypt-password-stdin` decrypts to the expected import envelope.
- Chapter-45 §"Encrypted wallet support" subsection (rewrite the current "deferred" framing).

### Phase 6 — Cycle close

- `Cargo.toml:3` v0.30.0 → v0.31.0.
- `scripts/install.sh:32` toolkit pin bump.
- `CHANGELOG.md` v0.31.0 section.
- Audit clean + tag + push + GH Release.

### Phase 7 — GUI lockstep + FOLLOWUP closure

- GUI: pin bump v0.30.0 → v0.31.0; workspace version v0.15.0 → v0.16.0.
- New `SubcommandSchema` flag entries for the 3 new `--decrypt-password*` flags on the import-wallet schema. Verify schema_mirror with explicit `MNEMONIC_BIN=...`.
- GUI CHANGELOG + tag + push + GH Release.
- FOLLOWUP closure:
  - Close `wallet-import-electrum-encrypted` (resolved by v0.31.0).
  - File new `wallet-import-electrum-encrypted-storage-format-b` (Format B whole-file encryption, v0.31+).
  - File new `wallet-import-electrum-ecies-variants` (ECIES-encrypted wallets, v0.31+).

## Memory entries to consult on resume

- `project_v0_30_0_cycle_shipped` — Cycle 5 full context (most recent ship).
- `feedback_no_parallelism_for_code_generation` — subagent dispatch hygiene.
- `feedback_opus_primary_review_agent` — opus for substantive reviews; sonnet for trivial folds.
- `feedback_architect_must_run_prose_commands` — manual chapter command-blocks must be run locally.

## Repo state at session-end (post-Cycle-6a)

- mnemonic-toolkit master HEAD: TBD (post-Phase-1 commit).
- mnemonic-gui master HEAD: `5582e22` (Cycle 5 paired GUI commit, unchanged from Cycle 5 close).
- Both Cycle 5 tags pushed; CI green.
