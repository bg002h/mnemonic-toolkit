//! 4-round Feistel encryption + PBKDF2-derived round keys.
//!
//! Per SLIP-0039 §"Encryption" / §"Decryption". Wraps a master secret
//! S of length n bytes (n even, n >= 16, n <= 32) into an encrypted
//! master secret (EMS) of the same length.
//!
//! Algorithm (matches python-shamir-mnemonic reference impl):
//!
//! ```text
//! L, R = S[:n/2], S[n/2:]
//! for i in 0..4:
//!     (L, R) = (R, L XOR F(i, R))
//! EMS = R || L          // note swapped order at the end
//!
//! F(i, R) = PBKDF2-HMAC-SHA-256(
//!     password = bytes([i]) || passphrase,
//!     salt     = salt_prefix || R,
//!     iters    = (10000 << iteration_exponent) / 4,
//!     dkLen    = n/2,
//! )
//!
//! salt_prefix = b""                                 if extendable
//!             = b"shamir" || identifier_be(2 bytes) otherwise
//! ```
//!
//! Per SLIP-0039 §"Encryption of the master secret": "If ext = 1, then
//! salt_prefix is an empty string." The `extendable` parameter routes
//! both paths.
//!
//! Total iteration count across 4 rounds = `10000 * 2^iteration_exponent`.
//!
//! Decryption uses the same Feistel structure with rounds run in
//! reverse (3, 2, 1, 0) — the same structure decrypts because Feistel
//! is its own inverse with reversed key schedule.
//!
//! Single-buffer round-key reuse: a SINGLE `Zeroizing<Vec<u8>>` of
//! length `master_secret.len() / 2` is refilled across the 4 rounds —
//! one heap allocation per encryption/decryption pass, not four.
//!
//! Internal module. Public `slip39_split` / `slip39_combine` at P1c
//! validate inputs before reaching this layer; this module panics on
//! invariant violations (caller's responsibility).

use hmac::Hmac;
use pbkdf2::pbkdf2;
use sha2::Sha256;

/// Base PBKDF2 iteration count (before applying the exponent).
pub const BASE_ITERATION_COUNT: u32 = 10000;

/// Feistel round count.
pub const ROUND_COUNT: usize = 4;

/// SLIP-39 customization string prefix for the Feistel round-key salt.
pub const CUSTOMIZATION_STRING: &[u8] = b"shamir";

/// Encrypt master_secret via 4-round Feistel.
///
/// `extendable=true` switches to the SLIP-0039 ext=1 path, which uses an
/// empty salt_prefix instead of `b"shamir" || identifier_be`.
pub fn encrypt(
    master_secret: &[u8],
    passphrase: &[u8],
    iteration_exponent: u8,
    identifier: u16,
    extendable: bool,
) -> zeroize::Zeroizing<Vec<u8>> {
    feistel_run(
        master_secret,
        passphrase,
        iteration_exponent,
        identifier,
        extendable,
        false,
    )
}

/// Decrypt encrypted_master_secret via 4-round Feistel run in reverse.
///
/// `extendable` must match the value passed at encrypt time; mismatched
/// axes produce garbage bytes (no error at this layer — the share-combine
/// digest check at the higher layer catches the mismatch).
pub fn decrypt(
    encrypted_master_secret: &[u8],
    passphrase: &[u8],
    iteration_exponent: u8,
    identifier: u16,
    extendable: bool,
) -> zeroize::Zeroizing<Vec<u8>> {
    feistel_run(
        encrypted_master_secret,
        passphrase,
        iteration_exponent,
        identifier,
        extendable,
        true,
    )
}

/// Common Feistel driver. `reverse=false` encrypts; `reverse=true` decrypts.
fn feistel_run(
    input: &[u8],
    passphrase: &[u8],
    iteration_exponent: u8,
    identifier: u16,
    extendable: bool,
    reverse: bool,
) -> zeroize::Zeroizing<Vec<u8>> {
    let n = input.len();
    assert!(n >= 16 && n % 2 == 0 && n <= 32, "feistel: invalid secret length {n}");
    assert!(iteration_exponent <= 15, "feistel: iteration_exponent out of range");

    let half = n / 2;
    let iters_per_round = (BASE_ITERATION_COUNT << iteration_exponent) / (ROUND_COUNT as u32);
    assert!(iters_per_round > 0, "feistel: iters_per_round computed as 0");

    let mut l = zeroize::Zeroizing::new(input[..half].to_vec());
    let mut r = zeroize::Zeroizing::new(input[half..].to_vec());

    // Single round-key buffer reused across rounds.
    let mut round_key = zeroize::Zeroizing::new(vec![0u8; half]);

    let salt_prefix = build_salt_prefix(identifier, extendable);

    let round_order: Vec<u8> = if reverse {
        (0..ROUND_COUNT as u8).rev().collect()
    } else {
        (0..ROUND_COUNT as u8).collect()
    };

    for &i in &round_order {
        // Round function F(i, R) → round_key.
        round_function_into(
            i,
            passphrase,
            iters_per_round,
            &salt_prefix,
            &r,
            &mut round_key,
        );
        // (L, R) = (R, L XOR round_key)
        for j in 0..half {
            l[j] ^= round_key[j];
        }
        std::mem::swap(&mut l, &mut r);
    }

    // Per SLIP-0039 + python-shamir-mnemonic reference: the final
    // output concatenates `r || l` (the halves swapped one extra time
    // at the output boundary). This is the symmetric structure that
    // makes decryption simply run the same loop with the round-key
    // schedule reversed.
    let mut out = zeroize::Zeroizing::new(Vec::with_capacity(n));
    out.extend_from_slice(&r);
    out.extend_from_slice(&l);
    out
}

/// Build the per-encryption salt prefix.
///
/// Per SLIP-0039 §"Encryption of the master secret":
///   - `extendable == true`  ⇒ empty (the identifier is excluded from
///     the salt so the share set is "extendable" — additional shares can
///     be re-derived later from the master without re-encrypting).
///   - `extendable == false` ⇒ `b"shamir" || identifier_be(2 bytes)`.
fn build_salt_prefix(identifier: u16, extendable: bool) -> Vec<u8> {
    if extendable {
        Vec::new()
    } else {
        let mut salt = Vec::with_capacity(CUSTOMIZATION_STRING.len() + 2);
        salt.extend_from_slice(CUSTOMIZATION_STRING);
        salt.extend_from_slice(&identifier.to_be_bytes());
        salt
    }
}

/// Per-round F function: fills `out` with
/// `PBKDF2-HMAC-SHA-256(password = [i] || passphrase, salt = salt_prefix || r, iters, dkLen = out.len())`.
fn round_function_into(
    round_idx: u8,
    passphrase: &[u8],
    iters: u32,
    salt_prefix: &[u8],
    r: &[u8],
    out: &mut [u8],
) {
    // password = [round_idx] || passphrase
    let mut password = Vec::with_capacity(1 + passphrase.len());
    password.push(round_idx);
    password.extend_from_slice(passphrase);

    // salt = salt_prefix || r
    let mut salt = Vec::with_capacity(salt_prefix.len() + r.len());
    salt.extend_from_slice(salt_prefix);
    salt.extend_from_slice(r);

    pbkdf2::<Hmac<Sha256>>(&password, &salt, iters, out)
        .expect("pbkdf2 fill must succeed (dkLen + iters in supported range)");

    // Scrub locals.
    zeroize::Zeroize::zeroize(&mut password);
    zeroize::Zeroize::zeroize(&mut salt);
}
