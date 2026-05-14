//! 4-round Feistel encryption + PBKDF2-derived round keys.
//!
//! Per SLIP-0039 §"Encryption" / §"Decryption". Wraps a master secret
//! S of length n bytes (n even, n >= 16, n <= 32) into an encrypted
//! master secret (EMS) of the same length, using:
//!   - 4 Feistel rounds
//!   - Round-key derivation via PBKDF2-HMAC-SHA-256
//!   - Iteration count = `(10000 << iteration_exponent) / 4` per round
//!     = `10000 * 2^E` total across all 4 rounds
//!   - Per-round password = `[round_idx] || passphrase`
//!   - Per-round salt = `b"shamir" || identifier_be_bytes || R`
//!     (R is the right-half of the current Feistel state)
//!
//! Decryption runs the rounds in reverse (3, 2, 1, 0).
//!
//! Single-buffer round-key reuse: a SINGLE `Zeroizing<Vec<u8>>` of
//! length `master_secret.len() / 2` is refilled across the 4 rounds.
//! One `pin_pages_for` call per encryption pass, not four.
//!
//! Internal module. Public `slip39_split` / `slip39_combine` at P1c
//! validate inputs (length, iteration_exponent bounds, identifier range)
//! before reaching this layer; this module panics on invariant violations.

// Phase 1b RED stub: type signatures only. Bodies land in P1b GREEN.

/// Base PBKDF2 iteration count (before applying the exponent).
pub const BASE_ITERATION_COUNT: u32 = 10000;

/// Feistel round count.
pub const ROUND_COUNT: usize = 4;

/// SLIP-39 customization string prefix for the Feistel round-key salt.
pub const CUSTOMIZATION_STRING: &[u8] = b"shamir";

/// Encrypt master_secret via 4-round Feistel with PBKDF2-derived round
/// keys, parameterized by passphrase + iteration_exponent + identifier.
///
/// PANICS if `master_secret.len()` is odd or < 16. `iteration_exponent`
/// must be in `0..=15` (callers validate; this layer asserts only that
/// the resulting iteration count is non-zero).
pub fn encrypt(
    _master_secret: &[u8],
    _passphrase: &[u8],
    _iteration_exponent: u8,
    _identifier: u16,
) -> zeroize::Zeroizing<Vec<u8>> {
    todo!("P1b GREEN — implement 4-round Feistel encrypt")
}

/// Decrypt encrypted_master_secret via 4-round Feistel run in reverse.
/// Symmetric counterpart of `encrypt` — same parameters yield the
/// original master secret iff the passphrase matches.
pub fn decrypt(
    _encrypted_master_secret: &[u8],
    _passphrase: &[u8],
    _iteration_exponent: u8,
    _identifier: u16,
) -> zeroize::Zeroizing<Vec<u8>> {
    todo!("P1b GREEN — implement 4-round Feistel decrypt")
}
