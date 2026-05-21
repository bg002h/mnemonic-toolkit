# BRAINSTORM — `mnemonic-toolkit-v0.31.0` (electrum-encrypted)

**Date:** 2026-05-21 (immediately post-`mnemonic-toolkit-v0.30.0` ship).
**Source SHA at brainstorm time:** master HEAD post-Cycle-5 (`0555008`).
**Sync state:** local master ≡ origin/master.
**Predecessor brainstorm:** `design/BRAINSTORM_v0_28_plus_residual_followups.md` §"Cycle 6 — `mnemonic-toolkit-v0.29.2` (electrum-encrypted)".
**Kickoff:** `design/CYCLE_6_KICKOFF.md`.
**P0 recon dossier:** `design/cycle-6-p0-recon.md`.

## Scope-correction findings (from P0 recon)

### Finding 1 — Scheme is sha256d + AES-CBC, NOT PBKDF2

Per P0 §A1: the parent FOLLOWUP body's "PBKDF2 + AES-CBC" claim is wrong. Verified against `electrum/crypto.py::_pw_decode_raw`:

- **Key derivation:** `key = sha256d(password)` (double SHA-256, no iteration count, no salt).
- **Encryption:** AES-256-CBC with random 16-byte IV + PKCS7 padding.
- **Wire format:** `base64(iv (16 bytes) || aes_cbc_encrypted_with_pkcs7)`.

**Cycle 6 implements sha256d + AES-CBC.** No PBKDF2 needed (despite `pbkdf2` already being a direct dep for SLIP-39).

### Finding 2 — Format A vs Format B scope split

Per P0 §A2: Electrum has TWO encryption surfaces.

- **Format A — field-level encryption** (per-field base64 ciphertext WITHIN a plaintext JSON wallet, with `"use_encryption": true` at the wallet root). The current refusal at `wallet_import/electrum.rs:305-313` targets this surface. **CYCLE 6 IN-SCOPE.**
- **Format B — whole-file storage encryption** (entire wallet file body = single base64 blob with version-byte prefix + 4-byte MAC suffix). NOT JSON-parseable as-is; current parser fails differently. **CYCLE 6 OUT-OF-SCOPE.** Filed as new FOLLOWUP `wallet-import-electrum-encrypted-storage-format-b` at Cycle 6 close.

## Decisions locked

1. **Scheme:** sha256d(password) + AES-256-CBC + PKCS7 + base64 (Format A only). Hardcode v1 (`PW_HASH_VERSION_LATEST=1` per Electrum source); reject v2+ with "unsupported password hash version" error if ever encountered.
2. **CLI surface:** 3-form password input on `mnemonic import-wallet`:
   - `--decrypt-password <VAL>` — inline; `secret_in_argv_warning`; `flag_is_secret`.
   - `--decrypt-password-file <PATH>` — file read; `warn_if_world_readable`.
   - `--decrypt-password-stdin` — boolean; reads from process stdin.
   - Mutex group: at most ONE per invocation.
3. **Parser integration:** orchestrator pre-decrypts sensitive fields in the JSON BEFORE calling `ElectrumParser::parse(blob, stderr)`. Preserves the parser-trait surface (no signature change). Decrypt-in-place: ciphertext field values replaced with plaintext in the JSON `obj`.
4. **Fields to decrypt:** all base64-ciphertext-shaped string values that match Electrum's sensitive-field set. Conservative: decrypt the top-level fields the parser actually reads (`seed`, `xprv`, `keystore.seed`, `keystore.xprv`, etc. — surface enumerated at Phase 3 plan-doc).
5. **No password supplied + use_encryption=true:** existing refusal fires with updated stderr template pointing at `--decrypt-password*`.
6. **Password supplied + use_encryption=false:** stderr advisory ("wallet is not encrypted; --decrypt-password* ignored") + proceed with normal parse.
7. **SemVer:** MINOR `v0.30.0 → v0.31.0` per parent brainstorm pre-locked rule: "password-on-argv is MINOR per architect I3 policy IF passed inline". User picked 3-form (inline included). Paired GUI `v0.15.0 → v0.16.0`.
8. **Library-local error pattern:** `electrum_crypto.rs` defines `ElectrumDecryptError` with hand-rolled `impl Display` (per `seed_xor.rs:31-67` precedent). CLI boundary maps to `ToolkitError::BadInput`.

## Cycle 6 architectural shape

### Cycle 6a (THIS session — ships)

- `design/CYCLE_6_KICKOFF.md`
- `design/cycle-6-p0-recon.md`
- This brainstorm doc + opus R0 review (deferred — brainstorm is small enough that the plan-doc review will catch scope issues).
- Plan-doc with deferred Phase 2-7 (opus R0 review applies at 6b execution-start).
- **Phase 1:** `crates/mnemonic-toolkit/src/electrum_crypto.rs` with `decrypt_field()` + `encrypt_field()` (symmetric helper for fixture generation) + library-local `ElectrumDecryptError` + ~20 unit cells.
- Cargo.toml: add `aes = "0.8"`, `cbc = "0.1"`, `base64 = "0.22"` as direct deps.
- `lib.rs` doc-comment: append `electrum_crypto` to the lib-local-error sibling list.

### Cycle 6b (NEXT session)

- Phase 2: CLI plumbing in `cmd/import_wallet.rs` (3 new flags + secret hygiene).
- Phase 3: parser integration in `wallet_import/electrum.rs` (pre-decrypt orchestration).
- Phase 4: `tests/cli_import_wallet_electrum_encrypted.rs` (~30 cells).
- Phase 5: manual chapter-41 + chapter-45 updates.
- Phase 6: cycle close (version bump + CHANGELOG + tag + push + install-pin-check + GH Release).
- Phase 7: GUI lockstep + opus end-of-cycle review + FOLLOWUP closure.

## Architecture (6a focus)

### `crates/mnemonic-toolkit/src/electrum_crypto.rs` (NEW)

```rust
//! Electrum field-level encryption decrypt + encrypt primitives.
//! Implements Format A (`pw_decode_bytes` / `pw_encode_bytes`) per
//! `electrum/crypto.py` v1 password-hash version.
//!
//! Scheme: sha256d(password) + AES-256-CBC + PKCS7 + base64.

use aes::Aes256;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use cbc::{Decryptor, Encryptor};
use cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

type Aes256CbcDec = Decryptor<Aes256>;
type Aes256CbcEnc = Encryptor<Aes256>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ElectrumDecryptError {
    Base64DecodeFailure(String),       // input is not valid base64
    CiphertextTooShort { got: usize }, // < 16 bytes (no room for IV)
    AesDecryptFailure,                 // PKCS7 unpadding refused (wrong key or corrupt ciphertext)
    Utf8DecodeFailure,                 // decrypted bytes are not valid UTF-8
}

impl std::fmt::Display for ElectrumDecryptError { /* ... */ }
impl std::error::Error for ElectrumDecryptError {}

/// Derive the AES key from a password via Electrum's sha256d scheme.
pub fn derive_key(password: &[u8]) -> Zeroizing<[u8; 32]> { /* ... */ }

/// Decrypt a base64-encoded Electrum-encrypted field.
/// Wire format: base64(iv (16 bytes) || aes-cbc(plaintext + PKCS7)).
pub fn decrypt_field(b64_ciphertext: &str, password: &[u8]) -> Result<Zeroizing<String>, ElectrumDecryptError> { /* ... */ }

/// Encrypt a UTF-8 string into Electrum-encrypted field format. Symmetric inverse of decrypt_field.
/// IV is supplied by caller for determinism; production code should pass cryptographically random bytes.
pub fn encrypt_field(plaintext: &str, password: &[u8], iv: &[u8; 16]) -> String { /* ... */ }
```

### `lib.rs` doc-comment extension

Append after the seedqr bullet (post-Cycle-5):

```rust
//! - `electrum_crypto` — Electrum field-level encryption primitives (v0.31.0).
//!   Defines a small `ElectrumDecryptError` so the library surface does
//!   not pull in the binary-private `ToolkitError`. The CLI handler in
//!   `src/cmd/import_wallet.rs` (Cycle 6b Phase 3) converts via a boundary
//!   mapper to `ToolkitError::BadInput` at orchestrator pre-decrypt time.
```

## Test plan (6a unit cells)

**Target: ~20 unit cells** in `electrum_crypto.rs`:

1. **`derive_key`:**
   - Known-vector cross-check: `sha256d(b"test-password")` matches Python `hashlib.sha256(hashlib.sha256(b"test-password").digest()).digest()`. Lock the exact hex at impl time.
   - Empty password produces deterministic key (non-zero).
   - Long password handled (no truncation).

2. **`encrypt_field` + `decrypt_field` round-trips:**
   - 12-word phrase plaintext round-trips byte-for-byte.
   - Empty string plaintext round-trips.
   - Maximum-length plaintext round-trips (test the PKCS7 padding boundary at 16-byte multiples).
   - Multi-byte UTF-8 plaintext round-trips.

3. **`decrypt_field` refusal cells:**
   - Wrong-password refusal → `AesDecryptFailure` (PKCS7 strip refuses).
   - Base64-decode-fail refusal → `Base64DecodeFailure`.
   - Ciphertext shorter than IV (16 bytes) → `CiphertextTooShort`.
   - Ciphertext correct length but not multiple-of-16 after IV → `AesDecryptFailure`.
   - UTF-8-decode-fail after decrypt → `Utf8DecodeFailure` (use a ciphertext that decrypts to non-UTF-8 bytes).

4. **Cross-impl smoke (Python reference):**
   - One test cell uses a HARD-CODED ciphertext + password generated by `electrum/crypto.py::pw_encode_bytes` (against a known plaintext "test-mnemonic" + password "test-password" + IV from `os.urandom`). The Rust `decrypt_field` MUST produce byte-identical plaintext. Recipe for fixture generation documented in the test comments.

## R0 review scope (deferred — 6a small enough to ship after self-review)

Given Cycle 6a is materially small (1 new library module + cargo.toml deps + lib.rs doc-comment + ~20 unit cells), the brainstorm + plan-doc R0 reviewer-loop discipline is RELAXED for 6a only:

- 6a self-review covers placeholders, scope, contradictions.
- 6b execution-start (next session) dispatches opus R0 review on the FULL plan-doc (which includes 6a's locks + 6b's parser integration + CLI + manual + ship).
- The brainstorm itself ships with 6a; 6b only updates if scope drift surfaces.

This is a deliberate scope cut to ensure 6a completes in this session. If review surfaces issues, they fold inline before 6b dispatch.

## FOLLOWUP closure semantics

**At Cycle 6 ship (in 6b):**

- **Close:** `wallet-import-electrum-encrypted` (resolved by v0.31.0). FOLLOWUP body update: correct the "PBKDF2 + AES-CBC" claim to "sha256d + AES-256-CBC" with a "FOLLOWUP body had stale scheme citation; corrected at Cycle 6 P0 recon" note.

**New FOLLOWUPs filed at 6b close:**

1. `wallet-import-electrum-encrypted-storage-format-b` — whole-file storage encryption (per P0 §A2). Tier `v0.31+`.
2. `wallet-import-electrum-ecies-variants` — ECIES-encrypted seed variants in some hardware-wallet Electrum modes. Tier `v0.31+`.

## Cross-cutting

- **No `ToolkitError` variants added.** `ElectrumDecryptError` is library-local; CLI boundary maps to `BadInput`.
- **`secrets.rs::flag_is_secret`** add `"--decrypt-password"` (unconditionally secret). Three-form variants `-file` / `-stdin` NOT added — those are flag-level neutral, value-level secret.
- **install.sh self-pin:** v0.30.0 → v0.31.0 at Cycle 6b Phase 6.
- **Manual mirror:** any new clap surface in import-wallet must reflect in chapter-41's `## \`mnemonic import-wallet\`` section + chapter-45's "Encrypted wallet support" subsection rewrite.

## Memory entries consulted

- `project_v0_30_0_cycle_shipped` — Cycle 5 (most recent ship).
- `feedback_a0_recon_check_gui_schema_json` — schema-mirror discipline.
- `feedback_no_parallelism_for_code_generation` — subagent dispatch hygiene.
- `feedback_opus_primary_review_agent` — opus for substantive reviews.
- `feedback_architect_must_run_prose_commands` — manual chapter command-blocks must run locally.
- `feedback_r0_must_read_source_off_by_n` — R0 reviewer should grep against source; relevant for the Phase 3 parser-integration plan-doc R0 in 6b.

## Open questions for 6b resume

None at 6a brainstorm-write time. All architectural decisions locked. 6b's job is implementing Phases 2-7 against the locks above.
