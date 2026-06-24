//! Integration tests for `mk-codec` end-to-end encode/decode.
//!
//! These tests live as a separate integration test target so they
//! exercise only the public `mk_codec::{encode, decode, ...}` surface,
//! the way an external consumer would. Unit-level coverage of the
//! string-layer pipeline (including all sad-path Error variants) lives
//! in `crate::string_layer::pipeline::tests`.

use std::str::FromStr;

use bitcoin::NetworkKind;
use bitcoin::bip32::{ChainCode, ChildNumber, DerivationPath, Fingerprint, Xpub};
use bitcoin::secp256k1::{PublicKey, Secp256k1, SecretKey};
use mk_codec::{KeyCard, decode, encode, encode_with_chunk_set_id};

fn synthetic_xpub(path: &DerivationPath) -> Xpub {
    let secp = Secp256k1::new();
    let sk = SecretKey::from_slice(&[0x42u8; 32]).expect("valid secret key");
    let pk = PublicKey::from_secret_key(&secp, &sk);
    let components: Vec<ChildNumber> = path.into_iter().copied().collect();
    let depth = components.len() as u8;
    let child_number = components
        .last()
        .copied()
        .unwrap_or(ChildNumber::Normal { index: 0 });
    Xpub {
        network: NetworkKind::Main,
        depth,
        parent_fingerprint: Fingerprint::from([0x10, 0x20, 0x30, 0x40]),
        child_number,
        public_key: pk,
        chain_code: ChainCode::from([0xCCu8; 32]),
    }
}

#[test]
fn round_trip_single_xpub_one_policy_id_stub() {
    // BIP 48 multisig at index 2' (segwit-v0) — the recommended path
    // for the typical mk1 use case (foreign xpub for multisig recovery).
    let path = DerivationPath::from_str("48'/0'/0'/2'").unwrap();
    let card = KeyCard::new(
        vec![[0x11, 0x22, 0x33, 0x44]],
        Some(Fingerprint::from([0xAA, 0xBB, 0xCC, 0xDD])),
        path.clone(),
        synthetic_xpub(&path),
    );

    let strings = encode(&card).expect("encode succeeds");
    assert!(
        !strings.is_empty(),
        "encode must produce at least one mk1 string"
    );
    for s in &strings {
        assert!(s.starts_with("mk1"), "string did not start with mk1: {s}");
    }

    let parts: Vec<&str> = strings.iter().map(|s| s.as_str()).collect();
    let recovered = decode(&parts).expect("decode succeeds");
    assert_eq!(recovered, card);
}

#[test]
fn deterministic_round_trip_with_explicit_chunk_set_id() {
    let path = DerivationPath::from_str("84'/0'/0'").unwrap();
    let card = KeyCard::new(
        vec![[0xAA, 0xBB, 0xCC, 0xDD]],
        Some(Fingerprint::from([0x12, 0x34, 0x56, 0x78])),
        path.clone(),
        synthetic_xpub(&path),
    );

    let s1 = encode_with_chunk_set_id(&card, 0xABCDE).expect("encode");
    let s2 = encode_with_chunk_set_id(&card, 0xABCDE).expect("encode");
    assert_eq!(s1, s2, "explicit chunk_set_id must be byte-deterministic");

    let parts: Vec<&str> = s1.iter().map(|s| s.as_str()).collect();
    let recovered = decode(&parts).expect("decode succeeds");
    assert_eq!(recovered, card);
}

#[test]
fn round_trip_fingerprint_omitted() {
    // Privacy-preserving mode: bytecode-header bit 2 unset, no
    // origin_fingerprint on the wire (closure Q-8).
    let path = DerivationPath::from_str("44'/0'/0'").unwrap();
    let card = KeyCard::new(
        vec![[0x55, 0x66, 0x77, 0x88]],
        None,
        path.clone(),
        synthetic_xpub(&path),
    );

    let strings = encode(&card).expect("encode");
    let parts: Vec<&str> = strings.iter().map(|s| s.as_str()).collect();
    let recovered = decode(&parts).expect("decode");
    assert_eq!(recovered, card);
    assert!(
        recovered.origin_fingerprint.is_none(),
        "decoded card must round-trip the missing fingerprint"
    );
}
