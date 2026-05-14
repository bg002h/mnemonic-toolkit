//! v0.13.0 P1b — library tests for SLIP-39 Feistel encryption pipeline.
//!
//! Per SPEC §2.1 + §4 G1 (vectors-driven verification deferred to P1c).
//! This file covers:
//!   - encrypt → decrypt round-trip across 5 SLIP-39 secret sizes
//!   - empty-passphrase + non-empty-passphrase paths
//!   - iteration_exponent variations (0, 1, 5; 5 is the advisory
//!     threshold)
//!   - identifier-mismatch produces different EMS
//!   - constants pin
//!
//! Byte-pinned anchor vectors against python-shamir-mnemonic are
//! deferred to P1c (vectors.json includes 15 positive vectors that
//! exercise the full pipeline end-to-end).

use mnemonic_toolkit::slip39::feistel;

// ============================================================================
// Constants pin
// ============================================================================

#[test]
fn base_iteration_count_is_10000() {
    assert_eq!(feistel::BASE_ITERATION_COUNT, 10000);
}

#[test]
fn round_count_is_four() {
    assert_eq!(feistel::ROUND_COUNT, 4);
}

#[test]
fn customization_string_is_shamir() {
    assert_eq!(feistel::CUSTOMIZATION_STRING, b"shamir");
}

// ============================================================================
// Round-trip tests across SLIP-39's 5 entropy sizes
// ============================================================================

fn round_trip(secret_len_bytes: usize, iteration_exponent: u8) {
    let master = vec![0xA5u8; secret_len_bytes];
    let passphrase = b"";
    let identifier = 0x1234u16;
    let ems = feistel::encrypt(&master, passphrase, iteration_exponent, identifier, false);
    let recovered = feistel::decrypt(&ems, passphrase, iteration_exponent, identifier, false);
    assert_eq!(
        recovered.as_slice(),
        master.as_slice(),
        "round-trip mismatch at secret_len={secret_len_bytes}, exp={iteration_exponent}",
    );
}

#[test]
fn round_trip_16_bytes() {
    round_trip(16, 0);
}

#[test]
fn round_trip_20_bytes() {
    round_trip(20, 0);
}

#[test]
fn round_trip_24_bytes() {
    round_trip(24, 0);
}

#[test]
fn round_trip_28_bytes() {
    round_trip(28, 0);
}

#[test]
fn round_trip_32_bytes() {
    round_trip(32, 0);
}

// ============================================================================
// Passphrase paths
// ============================================================================

#[test]
fn empty_passphrase_round_trip() {
    let master = [0u8; 16];
    let ems = feistel::encrypt(&master, b"", 0, 0xABCD, false);
    let recovered = feistel::decrypt(&ems, b"", 0, 0xABCD, false);
    assert_eq!(recovered.as_slice(), &master);
}

#[test]
fn non_empty_passphrase_round_trip() {
    let master = [0xFFu8; 32];
    let pass = b"correct horse battery staple";
    let ems = feistel::encrypt(&master, pass, 0, 0x4567, false);
    let recovered = feistel::decrypt(&ems, pass, 0, 0x4567, false);
    assert_eq!(recovered.as_slice(), &master);
}

#[test]
fn wrong_passphrase_yields_different_secret() {
    // Without a digest verification step (which lives in the share-
    // combine layer, NOT the Feistel layer), wrong-passphrase decrypt
    // returns DIFFERENT garbage bytes — NOT an error. This is the
    // SLIP-39 design: the Feistel layer is purely cryptographic; the
    // digest check at the share layer detects the mismatch.
    let master = [0x42u8; 16];
    let ems = feistel::encrypt(&master, b"correct", 0, 0x1111, false);
    let recovered_wrong = feistel::decrypt(&ems, b"wrong", 0, 0x1111, false);
    assert_ne!(
        recovered_wrong.as_slice(),
        &master,
        "wrong passphrase must yield different bytes (no digest check at this layer)",
    );
}

// ============================================================================
// iteration_exponent variations
// ============================================================================

#[test]
fn iteration_exponent_zero_round_trip() {
    round_trip(16, 0);
}

#[test]
fn iteration_exponent_one_round_trip() {
    round_trip(16, 1);
}

#[test]
fn iteration_exponent_changes_ems() {
    // Same master + passphrase + identifier but different
    // iteration_exponent → different EMS (PBKDF2 iters differ).
    let master = [0x33u8; 16];
    let ems0 = feistel::encrypt(&master, b"", 0, 0xAABB, false);
    let ems1 = feistel::encrypt(&master, b"", 1, 0xAABB, false);
    assert_ne!(ems0.as_slice(), ems1.as_slice());
}

// ============================================================================
// Identifier sensitivity
// ============================================================================

#[test]
fn identifier_mismatch_yields_different_ems() {
    let master = [0u8; 16];
    let pass = b"";
    let ems_a = feistel::encrypt(&master, pass, 0, 0x1234, false);
    let ems_b = feistel::encrypt(&master, pass, 0, 0x5678, false);
    assert_ne!(
        ems_a.as_slice(),
        ems_b.as_slice(),
        "different identifiers yield different EMS",
    );
}

#[test]
fn identifier_zero_round_trip() {
    let master = [0u8; 16];
    let ems = feistel::encrypt(&master, b"", 0, 0, false);
    let recovered = feistel::decrypt(&ems, b"", 0, 0, false);
    assert_eq!(recovered.as_slice(), &master);
}

// ============================================================================
// EMS length matches master length
// ============================================================================

#[test]
fn ems_length_equals_master_length() {
    for &len in &[16usize, 20, 24, 28, 32] {
        let master = vec![0u8; len];
        let ems = feistel::encrypt(&master, b"", 0, 0, false);
        assert_eq!(ems.len(), len, "EMS length must equal master length for size {len}");
    }
}

// ============================================================================
// Encryption is deterministic given the same inputs
// ============================================================================

#[test]
fn encrypt_is_deterministic() {
    let master = [0x77u8; 24];
    let pass = b"test phrase";
    let id = 0x0F0F;
    let ems1 = feistel::encrypt(&master, pass, 0, id, false);
    let ems2 = feistel::encrypt(&master, pass, 0, id, false);
    assert_eq!(ems1.as_slice(), ems2.as_slice(), "encrypt must be deterministic");
}

// ============================================================================
// Zeroize discipline (compile-time check)
// ============================================================================

#[test]
fn returns_zeroizing_vec() {
    let master = [0u8; 16];
    let ems: zeroize::Zeroizing<Vec<u8>> = feistel::encrypt(&master, b"", 0, 0, false);
    let recovered: zeroize::Zeroizing<Vec<u8>> = feistel::decrypt(&ems, b"", 0, 0, false);
    // Type-binding check; if return shape changes, this won't compile.
    let _ = (ems, recovered);
}

// ============================================================================
// Extendable axis (SLIP-0039 §"Encryption of the master secret"):
//   "If ext = 1, then salt_prefix is an empty string. If ext = 0, then
//    salt_prefix = 'shamir' || id."
// extendable=true paths bypass the `shamir`-prefix + identifier salt, so
// the resulting EMS diverges from the ext=false EMS even with identical
// (master, passphrase, iter_exp, identifier).
// ============================================================================

#[test]
fn extendable_round_trip_16_bytes() {
    let master = [0xA5u8; 16];
    let pass = b"correct horse battery staple";
    let ems = feistel::encrypt(&master, pass, 0, 0x1234, true);
    let recovered = feistel::decrypt(&ems, pass, 0, 0x1234, true);
    assert_eq!(
        recovered.as_slice(),
        &master,
        "ext=true 16-byte round-trip must recover the master",
    );
}

#[test]
fn extendable_round_trip_32_bytes_non_empty_passphrase() {
    let master = [0xFFu8; 32];
    let ems = feistel::encrypt(&master, b"pw", 1, 0xCAFE, true);
    let recovered = feistel::decrypt(&ems, b"pw", 1, 0xCAFE, true);
    assert_eq!(
        recovered.as_slice(),
        &master,
        "ext=true 32-byte round-trip must recover the master",
    );
}

#[test]
fn extendable_axis_non_interop_with_non_extendable() {
    // Cross-axis: encrypt at ext=false, decrypt at ext=true. The salts
    // differ across all 4 rounds, so decryption produces garbage — NOT
    // the master. This is structural: SLIP-0039 designs the ext bit so
    // the two axes are mutually unrecoverable even with matching
    // (identifier, iter_exp).
    let master = [0x42u8; 16];
    let pass = b"";
    let ems = feistel::encrypt(&master, pass, 0, 0xBEEF, false);
    let recovered = feistel::decrypt(&ems, pass, 0, 0xBEEF, true);
    assert_ne!(
        recovered.as_slice(),
        &master,
        "ext=false encrypt must NOT round-trip through an ext=true decrypt",
    );
}

#[test]
fn extendable_axis_changes_ems_bytes() {
    // Same inputs other than the ext bit ⇒ different EMS, because the
    // salt_prefix flips between b"shamir" || id_be and the empty string.
    let master = [0x11u8; 16];
    let ems_false = feistel::encrypt(&master, b"", 0, 0x1234, false);
    let ems_true = feistel::encrypt(&master, b"", 0, 0x1234, true);
    assert_ne!(
        ems_false.as_slice(),
        ems_true.as_slice(),
        "flipping the extendable bit must change the EMS bytes",
    );
}
