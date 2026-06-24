//! Top-level bytecode decoder: canonical `Vec<u8>` → `KeyCard`.
//!
//! Reverses [`crate::bytecode::encode::encode_bytecode`]; applies the
//! validity rules of `design/SPEC_mk_v0_1.md` §4.

use bitcoin::bip32::Fingerprint;

use crate::bytecode::header::BytecodeHeader;
use crate::bytecode::path::decode_path;
use crate::bytecode::xpub_compact::{decode_xpub_compact, reconstruct_xpub};
use crate::consts::{ORIGIN_FINGERPRINT_BYTES, POLICY_ID_STUB_BYTES};
use crate::error::{Error, Result};
use crate::key_card::KeyCard;

/// Decode canonical bytecode (pre-chunking) into a `KeyCard`.
///
/// Surfaces every SPEC §4 bytecode-layer validity rule via a unique
/// `Error` variant.
pub fn decode_bytecode(bytes: &[u8]) -> Result<KeyCard> {
    let mut cursor: &[u8] = bytes;

    let header_byte = read_u8(&mut cursor)?;
    let header = BytecodeHeader::parse(header_byte)?;

    let stub_count = read_u8(&mut cursor)?;
    if stub_count == 0 {
        return Err(Error::InvalidPolicyIdStubCount);
    }
    let mut policy_id_stubs: Vec<[u8; 4]> = Vec::with_capacity(stub_count as usize);
    for _ in 0..stub_count {
        let stub: [u8; POLICY_ID_STUB_BYTES] = read_array(&mut cursor)?;
        policy_id_stubs.push(stub);
    }

    let origin_fingerprint = if header.fingerprint_flag {
        let fp_bytes: [u8; ORIGIN_FINGERPRINT_BYTES] = read_array(&mut cursor)?;
        Some(Fingerprint::from(fp_bytes))
    } else {
        None
    };

    let origin_path = decode_path(&mut cursor)?;
    let compact = decode_xpub_compact(&mut cursor)?;
    let xpub = reconstruct_xpub(&compact, &origin_path)?;

    if !cursor.is_empty() {
        return Err(Error::TrailingBytes);
    }

    Ok(KeyCard {
        policy_id_stubs,
        origin_fingerprint,
        origin_path,
        xpub,
    })
}

fn read_u8(cursor: &mut &[u8]) -> Result<u8> {
    if cursor.is_empty() {
        return Err(Error::UnexpectedEnd);
    }
    let b = cursor[0];
    *cursor = &cursor[1..];
    Ok(b)
}

fn read_array<const N: usize>(cursor: &mut &[u8]) -> Result<[u8; N]> {
    if cursor.len() < N {
        return Err(Error::UnexpectedEnd);
    }
    let mut buf = [0u8; N];
    buf.copy_from_slice(&cursor[..N]);
    *cursor = &cursor[N..];
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode::encode::encode_bytecode;
    use crate::bytecode::test_helpers::synthetic_xpub;
    use bitcoin::bip32::DerivationPath;
    use std::str::FromStr;

    fn fixture_card_1stub_with_fp() -> KeyCard {
        let path = DerivationPath::from_str("m/48'/0'/0'/2'").unwrap();
        KeyCard {
            policy_id_stubs: vec![[0xAA; 4]],
            origin_fingerprint: Some(Fingerprint::from([0xD3, 0x4D, 0xB3, 0x3F])),
            xpub: synthetic_xpub(&path),
            origin_path: path,
        }
    }

    fn fixture_card_3stubs_no_fp() -> KeyCard {
        let path = DerivationPath::from_str("m/48'/0'/0'/2'").unwrap();
        KeyCard {
            policy_id_stubs: vec![[0xAA; 4], [0xBB; 4], [0xCC; 4]],
            origin_fingerprint: None,
            xpub: synthetic_xpub(&path),
            origin_path: path,
        }
    }

    fn fixture_card_explicit_path() -> KeyCard {
        let path = DerivationPath::from_str("m/9999'/1234'/56'/7'").unwrap();
        KeyCard {
            policy_id_stubs: vec![[0x11, 0x22, 0x33, 0x44]],
            origin_fingerprint: Some(Fingerprint::from([0xAB, 0xCD, 0xEF, 0x01])),
            xpub: synthetic_xpub(&path),
            origin_path: path,
        }
    }

    #[test]
    fn round_trip_1stub_with_fp() {
        let card = fixture_card_1stub_with_fp();
        let wire = encode_bytecode(&card).unwrap();
        let decoded = decode_bytecode(&wire).unwrap();
        assert_eq!(decoded, card);
    }

    #[test]
    fn round_trip_3stubs_no_fp() {
        let card = fixture_card_3stubs_no_fp();
        let wire = encode_bytecode(&card).unwrap();
        let decoded = decode_bytecode(&wire).unwrap();
        assert_eq!(decoded, card);
    }

    #[test]
    fn round_trip_explicit_path() {
        let card = fixture_card_explicit_path();
        let wire = encode_bytecode(&card).unwrap();
        let decoded = decode_bytecode(&wire).unwrap();
        assert_eq!(decoded, card);
    }

    #[test]
    fn rejects_unsupported_version() {
        let card = fixture_card_1stub_with_fp();
        let mut wire = encode_bytecode(&card).unwrap();
        wire[0] = 0x10; // version=1
        assert!(matches!(
            decode_bytecode(&wire),
            Err(Error::UnsupportedVersion(1)),
        ));
    }

    #[test]
    fn rejects_reserved_bits_set() {
        let card = fixture_card_1stub_with_fp();
        let mut wire = encode_bytecode(&card).unwrap();
        wire[0] |= 0b0000_0010; // bit 1 = reserved
        assert!(matches!(
            decode_bytecode(&wire),
            Err(Error::ReservedBitsSet),
        ));
    }

    #[test]
    fn rejects_zero_stub_count() {
        let card = fixture_card_1stub_with_fp();
        let mut wire = encode_bytecode(&card).unwrap();
        wire[1] = 0; // stub_count = 0
        assert!(matches!(
            decode_bytecode(&wire),
            Err(Error::InvalidPolicyIdStubCount),
        ));
    }

    #[test]
    fn rejects_invalid_path_indicator() {
        let card = fixture_card_1stub_with_fp();
        let mut wire = encode_bytecode(&card).unwrap();
        // path indicator is at offset 1+1+4+4 = 10. Use 0x18, the
        // smallest reserved testnet-range indicator (0x18..=0xFD are
        // all reserved). 0x16 was the obvious choice in v0.1.x but
        // graduated to a defined indicator in v0.2.0 — see
        // `bytecode/path::round_trip_indicator_0x16_added_in_v0_2`.
        wire[10] = 0x18;
        assert!(matches!(
            decode_bytecode(&wire),
            Err(Error::InvalidPathIndicator(0x18)),
        ));
    }

    #[test]
    fn rejects_invalid_xpub_version() {
        let card = fixture_card_1stub_with_fp();
        let mut wire = encode_bytecode(&card).unwrap();
        // xpub_compact version is at offset 1+1+4+4+1 = 11
        wire[11] = 0xDE;
        wire[12] = 0xAD;
        wire[13] = 0xBE;
        wire[14] = 0xEF;
        assert!(matches!(
            decode_bytecode(&wire),
            Err(Error::InvalidXpubVersion(0xDEADBEEF)),
        ));
    }

    #[test]
    fn rejects_trailing_bytes() {
        let card = fixture_card_1stub_with_fp();
        let mut wire = encode_bytecode(&card).unwrap();
        wire.push(0xFF); // extra byte after xpub
        assert!(matches!(decode_bytecode(&wire), Err(Error::TrailingBytes),));
    }

    #[test]
    fn rejects_truncated_mid_stub() {
        let card = fixture_card_1stub_with_fp();
        let wire = encode_bytecode(&card).unwrap();
        let truncated = &wire[..4]; // header + count + 2/4 stub bytes
        assert!(matches!(
            decode_bytecode(truncated),
            Err(Error::UnexpectedEnd),
        ));
    }

    #[test]
    fn rejects_path_too_deep_at_top_level() {
        // Construct a card with a hand-crafted bytecode where the path
        // is an explicit-path with count = 11 (one over the cap).
        let card = fixture_card_1stub_with_fp();
        let wire = encode_bytecode(&card).unwrap();
        // path indicator at offset 1+1+4+4 = 10. Replace the std-table
        // indicator + xpub_compact tail with explicit-path +
        // 11 single-byte LEB128 components + xpub_compact.
        let header_and_pre_path = &wire[..10]; // header + count + stubs + fp
        let xpub_compact_tail = &wire[11..]; // skip the 1-byte std-table indicator
        let mut new_wire: Vec<u8> = header_and_pre_path.to_vec();
        new_wire.push(0xFE); // explicit-path indicator
        new_wire.push(11); // count = 11 (one over cap)
        for i in 0..11 {
            new_wire.push(i); // single-byte LEB128 component
        }
        new_wire.extend_from_slice(xpub_compact_tail);
        assert!(matches!(
            decode_bytecode(&new_wire),
            Err(Error::PathTooDeep(11)),
        ));
    }

    #[test]
    fn rejects_invalid_path_component_at_top_level() {
        // Construct a card with a hand-crafted bytecode where the
        // explicit-path LEB128 has a 6-byte continuation (overflow).
        let card = fixture_card_1stub_with_fp();
        let wire = encode_bytecode(&card).unwrap();
        let header_and_pre_path = &wire[..10];
        let xpub_compact_tail = &wire[11..];
        let mut new_wire: Vec<u8> = header_and_pre_path.to_vec();
        new_wire.push(0xFE); // explicit-path indicator
        new_wire.push(1); // count = 1
        // 6-byte LEB128 with all continuation bits set: triggers overflow check
        new_wire.extend_from_slice(&[0x80, 0x80, 0x80, 0x80, 0x80, 0x80]);
        new_wire.extend_from_slice(xpub_compact_tail);
        assert!(matches!(
            decode_bytecode(&new_wire),
            Err(Error::InvalidPathComponent(_)),
        ));
    }

    #[test]
    fn rejects_invalid_xpub_public_key() {
        // Perturb the public_key bytes (offset 40 within xpub_compact)
        // to a value that doesn't parse as a compressed secp256k1 point.
        let card = fixture_card_1stub_with_fp();
        let mut wire = encode_bytecode(&card).unwrap();
        // xpub_compact starts at offset 1+1+4+4+1 = 11; public_key
        // within xpub_compact starts at +40 (= 51).
        let pub_key_offset = 11 + 40;
        // 0x05 is not a valid compressed-point prefix (must be 0x02 or 0x03).
        wire[pub_key_offset] = 0x05;
        // Fill the rest with garbage that's almost certainly not on the curve.
        for i in 1..33 {
            wire[pub_key_offset + i] = 0xFF;
        }
        assert!(matches!(
            decode_bytecode(&wire),
            Err(Error::InvalidXpubPublicKey(_)),
        ));
    }
}
