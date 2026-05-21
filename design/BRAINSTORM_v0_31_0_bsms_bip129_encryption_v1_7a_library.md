# BRAINSTORM — Cycle 7a (BIP-129 encryption library)

**Date:** 2026-05-21 (post-Cycle-6b ship).
**Source SHA at brainstorm time:** master HEAD `b03cac3`.
**Sync state:** local master ≡ origin/master.
**P0 recon dossier:** `design/cycle-7-p0-recon.md` (primary-source verified vs `bitcoin/bips`).
**Kickoff:** `design/CYCLE_7_KICKOFF.md`.
**FOLLOWUP slug:** `bsms-bip129-encryption-envelope` (`design/FOLLOWUPS.md:2546`).
**Predecessor recon:** `design/agent-reports/v0_27_0-phase-2-bip129-recon.md` (cross-validated TV-3 + TV-4 values).

## Cycle scope (this brainstorm covers ONLY 7a)

Cycle 7 splits 7a (library) + 7b (CLI + parser integration + ship). 7a is library-only: BIP-129 encryption primitives + unit tests, no CLI surface, no parser touch, no version bump. 7b handles all user-visible surfaces.

## R0 lesson from Cycle 6

Cycle 6a's brainstorm shipped without opus R0 review. Cycle 6b R0 then caught a foundational design error that invalidated the entire `--decrypt-password*` design (Path A pivot). Cycle 7a brainstorm WILL dispatch opus R0 BEFORE Phase 1 library implementation. Cycle 7b WILL dispatch opus R0 on the plan-doc BEFORE Phase 2 implementation.

## Decisions locked

1. **New library module:** `crates/mnemonic-toolkit/src/bsms_crypto.rs`. Mirrors `electrum_crypto.rs` (Cycle 6a) and `seedqr.rs` (Cycle 5) precedent: pub library + library-local error + hand-rolled `impl Display`.
2. **Public surface (5 functions + 1 error enum):**
   - `derive_encryption_key(token_raw: &[u8]) -> Zeroizing<[u8; 32]>` — PBKDF2-SHA512(password=`"No SPOF"`, salt=`token_raw`, c=2048, dkLen=32).
   - `derive_hmac_key(encryption_key: &[u8; 32]) -> [u8; 32]` — SHA256(encryption_key). NOT zeroized (the HMAC_KEY is derived-and-used-immediately; the underlying ENCRYPTION_KEY is zeroized; HMAC_KEY's exposure is short-lived).
   - `compute_mac(hmac_key: &[u8; 32], token_hex: &str, data: &[u8]) -> [u8; 32]` — HMAC-SHA256 over `(token_hex.as_bytes() || data)`. Returns the 32-byte HMAC.
   - `decrypt(ciphertext: &[u8], encryption_key: &[u8; 32], iv: &[u8; 16]) -> Result<Zeroizing<Vec<u8>>, BsmsCryptoError>` — AES-256-CTR-Decrypt. Returns plaintext wrapped in Zeroizing.
   - `encrypt(plaintext: &[u8], encryption_key: &[u8; 32], iv: &[u8; 16]) -> Vec<u8>` — symmetric helper for fixture generation. Returns ciphertext (NOT zeroized; ciphertext is wire-format material).
3. **Library-local error enum `BsmsCryptoError`:**
   - `InvalidWireFormat { reason: String }` — covers all parse failures (bad hex, MAC too short, etc.). 7b's CLI maps these to `ToolkitError::BadInput`.
   - `MacMismatch` — MAC verification failure (signal wrong token OR tampered ciphertext). 7b's CLI maps to `ToolkitError::BadInput` (or a typed variant per FOLLOWUP body recommendation).
   - Hand-rolled `impl Display` + empty `impl Error` per `seed_xor.rs:31-67` precedent.
4. **NO MAC-verify helper at the library level.** 7b's CLI orchestrator does the MAC verify by calling `compute_mac` + comparing to received MAC byte-by-byte (constant-time). Rationale: keeping the library helpers single-purpose; the CLI is the natural place for "expected vs received" comparisons + the typed error for mismatch.
5. **TOKEN-mode handling NOT in library scope.** The library accepts raw bytes for the salt input + ASCII-hex for the HMAC input; it does NOT validate that the token length matches STANDARD (8 bytes) or EXTENDED (16 bytes) modes. 7b's CLI does mode-validation.
6. **NO Cargo.toml dep additions beyond `ctr = "0.9"`.** PBKDF2-SHA512 + HMAC-SHA256 + SHA256 + AES-256 all already available; the only new dep is the AES-CTR mode wrapper.

## Architecture (locked)

```rust
//! BIP-129 encryption-envelope crypto primitives (v0.31.0 / Cycle 7a).
//!
//! Implements PBKDF2-SHA512 key derivation + AES-256-CTR + HMAC-SHA256
//! MAC per BIP-129 §Encryption (verified vs `bitcoin/bips` 2026-05-21).
//!
//! Scope: pure crypto primitives. CLI integration (Cycle 7b) reads the
//! TOKEN, calls these primitives, verifies MAC, dispatches plaintext to
//! the existing `wallet_import/bsms.rs` parser.

use aes::Aes256;
use ctr::cipher::{KeyIvInit, StreamCipher};
use hmac::{Hmac, Mac};
use pbkdf2::pbkdf2;
use sha2::{Digest, Sha256, Sha512};
use zeroize::Zeroizing;

type Aes256Ctr = ctr::Ctr64BE<Aes256>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BsmsCryptoError {
    InvalidWireFormat { reason: String },
    MacMismatch,
}

impl std::fmt::Display for BsmsCryptoError { /* hand-rolled */ }
impl std::error::Error for BsmsCryptoError {}

pub fn derive_encryption_key(token_raw: &[u8]) -> Zeroizing<[u8; 32]> {
    let mut out = Zeroizing::new([0u8; 32]);
    pbkdf2::<Hmac<Sha512>>(b"No SPOF", token_raw, 2048, out.as_mut_slice())
        .expect("pbkdf2 fill must succeed (dkLen + iters in supported range)");
    out
}

pub fn derive_hmac_key(encryption_key: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(encryption_key);
    let mut out = [0u8; 32];
    out.copy_from_slice(&hasher.finalize());
    out
}

pub fn compute_mac(hmac_key: &[u8; 32], token_hex: &str, data: &[u8]) -> [u8; 32] {
    let mut mac = <Hmac<Sha256> as Mac>::new_from_slice(hmac_key).expect("HMAC accepts any key length");
    mac.update(token_hex.as_bytes());
    mac.update(data);
    let result = mac.finalize().into_bytes();
    let mut out = [0u8; 32];
    out.copy_from_slice(&result);
    out
}

pub fn decrypt(
    ciphertext: &[u8],
    encryption_key: &[u8; 32],
    iv: &[u8; 16],
) -> Result<Zeroizing<Vec<u8>>, BsmsCryptoError> {
    // ... AES-256-CTR-Decrypt via ctr::Ctr64BE<Aes256>
}

pub fn encrypt(plaintext: &[u8], encryption_key: &[u8; 32], iv: &[u8; 16]) -> Vec<u8> {
    // ... AES-256-CTR-Encrypt via ctr::Ctr64BE<Aes256> (symmetric inverse for fixtures)
}
```

### AES-CTR variant choice

BIP-129 line 150 cites RFC 3686 (AES-256-CTR). RFC 3686 specifies the counter format as `nonce (4 bytes) || iv (8 bytes) || counter (4 bytes)` totaling 16 bytes. But BIP-129 line 154 says `IV = First 16 bytes of MAC` — i.e., the full 16-byte IV is supplied as a single block.

The Rust `ctr` crate offers `Ctr32BE`, `Ctr64BE`, `Ctr128BE` variants based on counter-width. For "use the full 16-byte IV as-is and increment as a 128-bit big-endian counter" → `Ctr128BE<Aes256>`. For "treat first 8 bytes as nonce and increment last 8 as 64-bit BE counter" → `Ctr64BE<Aes256>`.

**Cycle 7a TV-3 unit cell will determine the correct variant empirically:**
- Compute ENCRYPTION_KEY for TV-3 (we have the expected value `7673ffd9...`).
- Decrypt TV-3's ciphertext with our chosen Ctr variant.
- If the decrypted text matches the BIP-129 TV-3 plaintext (the Round-1 Signer key record), variant is right.
- If not, switch variant + retry.

**Hypothesis:** `Ctr128BE<Aes256>` is the right variant for BIP-129 (full 16-byte block treated as 128-bit counter). Coinkite Python ref uses `pycryptodome` which interprets the IV as the initial counter block. Verify at impl time.

### Doc-comment fold for lib.rs

Append after the electrum_crypto bullet:

```rust
//! - `bsms_crypto` — BIP-129 encryption-envelope crypto primitives
//!   (v0.31.0 / Cycle 7a). Implements PBKDF2-SHA512 + AES-256-CTR + HMAC-
//!   SHA256 per BIP-129 §Encryption. Defines a library-local
//!   `BsmsCryptoError` per the same pattern. The CLI handler in
//!   `src/cmd/import_wallet.rs` (Cycle 7b Phase 3) will convert via a
//!   boundary mapper to `ToolkitError::BadInput` at orchestrator
//!   pre-decrypt time.
```

## Test plan (Cycle 7a unit cells)

**Target: 18-25 unit cells** covering:

1. **`derive_encryption_key`:**
   - TV-3 cross-validation: PBKDF2-SHA512(`"No SPOF"`, hex_decode(`a54044308ceac9b7`), 2048, 32) must equal `7673ffd9efd70336a5442eda0b31457f7b6cdf7b42fe17f274434df55efa9839`. Locked vs BIP-129 + v0.27.0 dossier.
   - TV-1/TV-2 NO_ENCRYPTION sanity: TOKEN=`0x00` → some deterministic key (less load-bearing but documents the NO_ENCRYPTION path).
   - TV-4 EXTENDED Signer 1 cross-validation.

2. **`derive_hmac_key`:**
   - TV-3: SHA256(`7673ffd9...`) must equal `3d4c422806ba8964c9ee45070cd675c024d96648a0ddb4001325818c84951de2`.

3. **`compute_mac`:**
   - TV-3 cross-validation: HMAC-SHA256(HMAC_KEY=`3d4c4228...`, ASCII-hex of TOKEN=`a54044308ceac9b7` || TV-3 plaintext) must equal `fbdbdb64e6a8231c342131d9f13dcd5a954b4c5021658fa5afcb3fc74dc82706`.

4. **`decrypt` (variant determination + TV-3 cross-validation):**
   - First cell: try `Ctr128BE<Aes256>` against TV-3 ciphertext + ENCRYPTION_KEY + IV → MUST produce the BIP-129 TV-3 plaintext. If fails, switch to `Ctr64BE<Aes256>` and retry.
   - Document the correct variant in `bsms_crypto.rs` once locked.

5. **`encrypt` (symmetric inverse):**
   - Round-trip: encrypt(plaintext, key, iv) → decrypt(ciphertext, key, iv) must yield byte-identical plaintext.
   - Cross-validate against TV-3 ciphertext (encrypt TV-3 plaintext with TV-3 KEY+IV; must produce TV-3 ciphertext byte-identical).

6. **Refusal classes:**
   - `decrypt` with ciphertext of length 0 → `InvalidWireFormat` or empty plaintext (locked at impl).
   - `encrypt` followed by `decrypt` with wrong key → garbage output (NOT an error in AES-CTR; the MAC verify at CLI level catches this).

### TV plaintext for TV-3

Per BIP-129 lines 301-315, TV-3 Signer-1 Round-1 record (plaintext, before encryption):

```
BSMS 1.0
a54044308ceac9b7
[b7868815/48'/0'/0'/2']xpub6FA5rfxJc94K1kNtxRby1hoHwi7YDyTWwx1KUR3FwskaF6HzCbZMz3zQwGnCqdiFeMTPV3YneTGS2YQPiuNYsSvtggWWMQpEJD4jXU7ZzEh
Signer 1 key
H8DYht5P6ko0bQqDV6MtUxpzBSK+aVHxbvMavA5byvLrOlCEGmO1WFR7k2wu42J6dxXD8vrmDQSnGq5MTMMbZ98=
```

(5 lines per BIP-129 Round-1 Signer; line 5 is base64-encoded ECDSA signature.) The ciphertext (TV-3 wire `MAC || ciphertext` hex string) needs extraction from `bip-0129.mediawiki` lines 301-315; the recon dossier has the MAC but not the full ciphertext. **Phase 1 implementer fetches the full TV-3 ciphertext hex from the bip-0129.mediawiki source at impl time** and locks it as a hardcoded test constant.

## Open questions for opus R0

1. **Ctr variant:** `Ctr128BE<Aes256>` (treat IV as 128-bit counter) vs `Ctr64BE<Aes256>` (8-byte nonce + 8-byte counter). BIP-129 line 150 cites RFC 3686 but doesn't specify counter width. The empirical TV-3 cell will lock this; opus R0 should confirm the framing is sound.
2. **`derive_hmac_key` return type:** `[u8; 32]` (not `Zeroizing<[u8; 32]>`) — is the lifetime-bound exposure of HMAC_KEY adequately addressed? The encryption_key INPUT is via `&[u8; 32]` (caller's responsibility to zeroize); the SHA256 output is short-lived. Document the threat model.
3. **`encrypt` symmetric helper:** is it appropriate to have an in-library encrypt helper (used only for fixture generation), or should it be `#[cfg(test)]`-gated? `electrum_crypto.rs` (Cycle 6a) has `encrypt_field` as `pub`; following that precedent.

## Out of scope (defer to 7b)

- CLI flag `--bsms-encryption-token <FILE|->`.
- `bsms.rs` parser integration (orchestrator pre-decrypt before existing 4-line/6-line parse).
- `secrets.rs::flag_is_secret` update.
- Cross-impl smoke against Coinkite Python ref (`bsms-bitcoin-secure-multisig-setup` repo).
- Manual chapter updates.
- Version bump + tag + GH Release.
- GUI lockstep + schema-mirror updates.
- FOLLOWUP closure (`bsms-bip129-encryption-envelope`).

## Memory entries consulted

- `project_v0_30_1_cycle_6b_shipped` — Cycle 6b R0 fold lesson (always opus-R0 before Phase 2).
- `project_v0_31_0_cycle_6a_shipped` — Cycle 6a library-ship pattern (split-cycle precedent).
- `feedback_r0_must_read_source_off_by_n` — R0 reviewer reads source ground truth.
- `feedback_opus_primary_review_agent` — opus for substantive reviews.
