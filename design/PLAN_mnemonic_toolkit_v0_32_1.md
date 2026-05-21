# mnemonic-toolkit-v0.32.1 Implementation Plan (Cycle 15 — bsms-encryption-round1-decrypt-then-verify)

> **For agentic workers:** REQUIRED SUB-SKILL: `superpowers:subagent-driven-development` or direct execution.

**Goal:** Ship `mnemonic-toolkit-v0.32.1` (SemVer-PATCH; behavior-expansion). Closes `bsms-encryption-round1-decrypt-then-verify` FOLLOWUP. Extend the `--bsms-round1` Round-1 verify flow to accept ENCRYPTED Round-1 KEY records (hex `MAC || ciphertext`), decrypting them with the shared `--bsms-encryption-token` before the existing BIP-322 signature verify. Closes the TV-3 decrypt-then-refuse boundary documented at `cli_import_wallet_bsms_encrypted.rs::tv3_decrypt_emits_notice_advisory`.

**Architecture:** Today `verify_bsms_round1_files` reads each `--bsms-round1` file as plaintext (5-line `BSMS 1.0\n…`). v0.32.1 adds an encrypted-record branch: if a `--bsms-round1` file's trimmed contents are all-hex AND do NOT start with the `BSMS 1.0` header, treat it as an encrypted Round-1 record — decrypt via the same `bsms_crypto` primitives the Round-2 path uses (`derive_encryption_key` → `derive_hmac_key` → split MAC(32)||ciphertext → IV=first16 → `decrypt` → `compute_mac` → byte-compare) → the resulting 5-line plaintext flows into the existing `parse_round1` + `verify_round1_signature`. The shared `--bsms-encryption-token` is read ONCE (hoisted above `verify_bsms_round1_files`) and passed to BOTH the Round-1 verify path and the existing Round-2 descriptor-decrypt block (de-duplicating the token read; prerequisite for the per-Signer-token generalization in the next cycle).

**Tech Stack:** Rust; reuses `bsms_crypto` (v0.31.0) + `hex` (existing); zero new deps; zero new clap flags; zero new `ToolkitError` variants (reuses `BsmsMacMismatch`, `BadInput`, `ImportWalletParse`); zero lib.rs changes.

**P0 STRICT-GATE recon (verified at master HEAD `1924e19`):**
- `cmd/import_wallet.rs:204-205` — `--bsms-encryption-token: Option<PathBuf>` (shared; unchanged this cycle).
- `cmd/import_wallet.rs:191-192` — `--bsms-round1: Vec<PathBuf>` (repeatable; already).
- `cmd/import_wallet.rs:252-256` — `verify_bsms_round1_files(&args.bsms_round1, strict, stderr)` call site.
- `cmd/import_wallet.rs:277-284` — stdin-contention guard (`--blob=- AND --token=-`).
- `cmd/import_wallet.rs:841-899` — Round-2 descriptor decrypt block (the crypto recipe to mirror); reads token at L843 via `read_bsms_token`.
- `cmd/import_wallet.rs:1897` — `read_bsms_token`.
- `cmd/import_wallet.rs:1952-2022` — `verify_bsms_round1_files` (add token param + encrypted branch).
- `wallet_import/bsms_round1.rs:26` — `BSMS_HEADER = "BSMS 1.0"`; `:84` — `parse_round1`.
- `bsms_crypto.rs:160` — `pub fn decrypt(...)`; `derive_encryption_key` / `derive_hmac_key` / `compute_mac` all pub (used at L874-885).
- TV-3 fixtures: `bsms-encrypted-standard-tv3.dat` (589 bytes hex) + `bsms-encrypted-standard-tv3-token.hex` (`a54044308ceac9b7`, 8-byte STANDARD). Decrypt → 5-line Round-1 plaintext (verified: currently decrypts + MAC-verifies, then BsmsParser refuses "got 5 lines").

**SemVer rationale (v0.32.0 → v0.32.1 PATCH):** behavior-expansion only — `--bsms-round1` now additionally accepts encrypted records. No new flag NAME (reuses `--bsms-round1` + `--bsms-encryption-token`). No GUI lockstep (schema_mirror flag-name set unchanged).

## File structure

### Source files modified (toolkit)
- `crates/mnemonic-toolkit/src/cmd/import_wallet.rs`:
  - **Reorder the stdin-contention guard ABOVE L252 (R0 I1 fold).** `verify_bsms_round1_files` runs at L252 — BEFORE the L260 blob match, the L268 standalone early-return, the L277 stdin guard, and the L287 blob read. So the token read must hoist to before L252. Move the stdin-contention check above L252, rewritten to use `args.blob.as_ref()` + `args.bsms_encryption_token.as_ref()` directly (no dependency on the L260 `blob_path` binding). In standalone mode `args.blob` is None → guard doesn't fire (same outcome). This guarantees the dual-stdin refusal fires BEFORE the token consumes stdin.
  - **Hoist the token read** to immediately after that guard (before L252): read the `--bsms-encryption-token` (if present) ONCE, producing an `Option<(token_hex: String, token_raw: Vec<u8>)>`. Validate token width (8/16 bytes) at this single site (moved from L849).
  - **`verify_bsms_round1_files` signature**: add a `token: Option<&BsmsToken>` param (where `BsmsToken` is a small local struct holding `hex: String` + `raw: Vec<u8>`, OR pass `Option<(&str, &[u8])>`). Inside, per record: read file text; if `is_encrypted_round1(&text)` → decrypt-then-MAC-verify via a new local `decrypt_round1_record(text, token)` helper → plaintext string → `parse_round1`; else → `parse_round1(&text)` directly (existing).
  - **`decrypt_round1_record` helper**: hex-decode the wire → split MAC(32)||ciphertext → derive keys → IV → `bsms_crypto::decrypt` → `compute_mac` → byte-compare (return `BsmsMacMismatch` on failure) → return the UTF-8 plaintext. Mirrors the L862-890 Round-2 recipe.
  - **`is_encrypted_round1` helper**: `let t = text.trim(); !t.starts_with("BSMS 1.0") && !t.is_empty() && t.bytes().all(|b| b.is_ascii_hexdigit())`.
  - **Encrypted-record-without-token error**: if `is_encrypted_round1` AND `token.is_none()` → `BadInput("--bsms-round1: record N looks encrypted (hex MAC||ciphertext) but no --bsms-encryption-token was supplied")`.
  - **Round-2 block (L841+)**: consume the hoisted token instead of re-reading via `read_bsms_token`. Preserve the existing decrypt + MAC behavior byte-for-byte.
  - **stderr NOTICE**: on encrypted-Round-1 decrypt success, emit `notice: import-wallet: --bsms-round1: BIP-129 encrypted Round-1 record N decrypted (token width K hex chars; MAC verified)` (mirrors the Round-2 NOTICE).

### Test files modified (toolkit)
- `crates/mnemonic-toolkit/tests/cli_import_wallet_bsms_encrypted.rs`:
  - **Convert `tv3_decrypt_emits_notice_advisory`** → `tv3_round1_decrypt_then_verify` — feed `bsms-encrypted-standard-tv3.dat` via `--bsms-round1` + the token; assert exit 0 + the Round-1 verify NOTICE/envelope shows the decrypted record verified (BIP-322 pass) instead of the Round-2 parse refusal. (Keep a thin variant asserting the OLD `--blob` Round-2 path still refuses TV-3 with "got 5" if that boundary is still useful, OR drop it.)
  - `round1_encrypted_without_token_refused` — `--bsms-round1 <encrypted.dat>` WITHOUT `--token` → exit 1 + the no-token error.
  - `round1_encrypted_wrong_token_mac_mismatch` — encrypted Round-1 + wrong token → `BsmsMacMismatch` (exit 2).
  - `round1_plaintext_still_verifies` — a plaintext 5-line Round-1 fixture via `--bsms-round1` (no token) still verifies (regression: the encrypted-detection must not mis-classify plaintext).
  - `round1_json_envelope_encrypted_record` — `--json` envelope's `bsms_round1_verifications` carries the decrypted record's verified status.
  - **`round1_encrypted_decrypt_ok_but_sig_fail` (R0 M2 fold)** — an encrypted Round-1 record that decrypts + MAC-verifies OK but whose plaintext carries a flipped/invalid BIP-322 signature: assert lenient mode emits the verify-failed NOTICE + `Failed` status (exit 0), and `--bsms-verify-strict` makes it fatal (`BsmsSignatureMismatch`). Exercises decrypt-then-verify × strict-flag interaction on a DECRYPTED record. (Build the fixture by decrypting TV-3, flipping one base64-SIG char, re-encrypting with the token — OR hand-craft.)
- Possibly add `crates/mnemonic-toolkit/src/cmd/import_wallet.rs` in-file unit tests for `is_encrypted_round1` (hex-vs-BSMS-header discrimination: plaintext "BSMS 1.0…" → false; all-hex → true; empty → false; mixed → false).

### Fixtures (toolkit)
- Reuse `bsms-encrypted-standard-tv3.dat` + `-token.hex` (the encrypted Round-1 record).
- Add `bsms-round1-plaintext-tv1.txt` (a plaintext 5-line Round-1 for the no-misclassify regression) — OR reuse an inline-string fixture in the test.

### Documentation (toolkit)
- `docs/manual/src/45-foreign-formats.md` (BSMS section) OR `40-cli-reference/41-mnemonic.md` (`import-wallet --bsms-round1` flag): document that `--bsms-round1` accepts BOTH plaintext (5-line) AND encrypted (hex MAC||ciphertext, decrypted via `--bsms-encryption-token`) records.

### Release tooling
- `Cargo.toml:3` — `0.32.0` → `0.32.1`.
- `CHANGELOG.md` — `## [0.32.1]`.
- `scripts/install.sh:32` — pin → `v0.32.1`.
- `design/FOLLOWUPS.md` — close `bsms-encryption-round1-decrypt-then-verify`.

## Tasks

### Task 1: Phase 2 — orchestrator: hoist token + encrypted-Round-1 branch
- [ ] Add `is_encrypted_round1` + `decrypt_round1_record` helpers.
- [ ] Hoist the token read; add token param to `verify_bsms_round1_files`; wire the encrypted branch + no-token error + NOTICE.
- [ ] Refactor the Round-2 block (L841+) to consume the hoisted token.
- [ ] Build.
- [ ] Commit Phase 2.

### Task 2: Phase 3 — Integration + unit tests
- [ ] Convert/add the 5 integration cells + `is_encrypted_round1` unit cells.
- [ ] Build + run.
- [ ] Commit Phase 3.

### Task 3: Phase 4 — Manual
- [ ] Document the dual plaintext/encrypted `--bsms-round1` intake.
- [ ] Manual lint.
- [ ] Commit Phase 4.

### Task 4: Phase 5 — Cycle close
- [ ] Version bump + install.sh + CHANGELOG.
- [ ] Pre-tag audit (test + clippy + manual lint).
- [ ] Opus end-of-cycle review.
- [ ] Commit + tag mnemonic-toolkit-v0.32.1 + push + GH Release.
- [ ] install-pin-check CI green.
- [ ] Close FOLLOWUP + memory.

## Cross-phase invariants
- Opus R0 review on plan-doc BEFORE Phase 2.
- Opus end-of-cycle review BEFORE tag.
- No `cargo fmt --all`.
- No GUI lockstep (no new flag).
- install-pin-check CI gate.

## Risk register
- **Token read ordering / stdin** — hoisting the token read above `verify_bsms_round1_files` must preserve the `--blob=- AND --token=-` stdin-contention guard. `--bsms-round1` itself has no stdin support (deferred). Standalone Round-1 mode (no `--blob`) reads the token once for the Round-1 path; the early-return at L268 is AFTER `verify_bsms_round1_files` so the hoist must happen before L252.
- **Encrypted-detection false-positive** — a plaintext Round-1 record starts with `BSMS 1.0`; the all-hex check excludes it (the header has spaces + letters beyond [a-f]). A degenerate all-hex "plaintext" cannot be a valid 5-line BSMS record (needs newlines + the header). Safe. Locked by `round1_plaintext_still_verifies` + the `is_encrypted_round1` unit cells.
- **MAC-compare timing** — same single-attempt non-interactive CLI flow as Round-2; byte-compare is acceptable (process exits on first mismatch; no repeated-probe surface). Mirror the L886 comment.
- **TV-3 BIP-322 verify** — the decrypted TV-3 Round-1 record passes `verify_round1_signature`: ALREADY PROVEN by the existing `wallet_import/bsms_verify.rs:109::tv3_standard_encryption_xpub_signer1_verifies` test (R0 M1 — the "confirm-or-escalate" framing was over-cautious; this is a known-good signed vector).
- **Token width validation site move** — moving the 8/16-byte check from L849 to the hoisted read must keep the same error text + apply to both consumers.

## Self-review (pre-R0 dispatch)
- ✓ P0 recon + crypto-recipe site identified (L862-890 mirror).
- ✓ Token-read hoist designed (de-dup + prerequisite for #3 per-Signer).
- ✓ Encrypted-detection predicate analyzed for false-positive safety.
- ✓ SemVer PATCH (no new flag) + no GUI lockstep.
- ✓ Test surface: 5 integration + is_encrypted_round1 unit cells.
- ✓ TV-3 BIP-322-verify-must-pass flagged as a Phase 3 confirm-or-escalate.
