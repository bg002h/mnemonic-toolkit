//! Secret M/N batch tk-1 (2026-11-25): dependency-level zeroize hygiene.
//!
//! * tk-bip39-mnemonic-not-zeroized: `bip39::Mnemonic` must be `ZeroizeOnDrop`
//!   (the `zeroize` feature of bip39 2.2.x).
//! * tk-sha3-shake256-no-zeroize: the `sha3` `zeroize` feature must be on, so
//!   the Keccak sponge state (`Sha3State`) zeroes on drop. Observed through
//!   Cargo.lock, since the state type is crate-private upstream. Limit: the
//!   feature scrubs the 200-byte sponge state; the reader's output block
//!   buffer is not scrubbed by sha3 0.10.

#[test]
fn bip39_mnemonic_is_zeroize_on_drop() {
    fn needs_zod<T: zeroize::ZeroizeOnDrop>() {}
    needs_zod::<bip39::Mnemonic>();
}

#[test]
fn sha3_has_zeroize_feature_enabled() {
    let lock = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../Cargo.lock"))
        .expect("read Cargo.lock");
    let mut in_sha3 = false;
    let mut found = false;
    for line in lock.lines() {
        if line.starts_with("[[package]]") {
            in_sha3 = false;
        } else if line == "name = \"sha3\"" {
            in_sha3 = true;
        } else if in_sha3 && line.trim() == "\"zeroize\"," {
            found = true;
        }
    }
    assert!(
        found,
        "sha3 does not depend on zeroize in Cargo.lock: enable the sha3 `zeroize` feature"
    );
}
