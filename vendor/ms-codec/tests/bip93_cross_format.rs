//! BIP-93 cross-format conformance pin.
//!
//! Verifies ms-codec is a proper sub-format of upstream codex32 at the byte
//! level for the entr length bucket: take BIP-93 §Test Vector 93.4 (256-bit
//! `leet`), extract its raw 32-byte payload via upstream `rust-codex32`,
//! re-encode those bytes as ms-codec entr, and confirm round-trip + that the
//! resulting ms1 string is itself parseable by upstream `rust-codex32`.
//!
//! Catches drift in upstream bit-packing across `rust-codex32` patch versions
//! (we exact-pin at `=0.1.0`, so drift is gated to manual bumps; this test
//! makes such a bump fail loudly if encoding semantics shift).
//!
//! BIP-93 spec: <https://github.com/bitcoin/bips/blob/master/bip-0093.mediawiki>

use ms_codec::codex32::Codex32String;
use ms_codec::{decode, encode, Payload, Tag};

/// BIP-93 §Test Vectors, vector 4: `leet` 256-bit single-share secret.
const BIP93_VECTOR_4: &str =
    "ms10leetsllhdmn9m42vcsamx24zrxgs3qrl7ahwvhw4fnzrhve25gvezzyqqtum9pgv99ycma";

/// Full 32-byte payload of BIP-93 §93.4 as extracted by rust-codex32 (32 bytes
/// = 1-byte ms-codec-style "prefix" 0xff + 31 bytes after it; ms-codec ignores
/// that interpretation here and treats the full 32 bytes as opaque entropy).
const BIP93_VECTOR_4_PAYLOAD_HEX: &str =
    "ffeeddccbbaa99887766554433221100ffeeddccbbaa99887766554433221100";

#[test]
fn bip93_vector_4_payload_extracts_via_upstream() {
    let c = Codex32String::from_string(BIP93_VECTOR_4.to_string())
        .expect("BIP-93 §93.4 must parse via rust-codex32");
    let data = c.parts().data();
    let expected = hex_decode(BIP93_VECTOR_4_PAYLOAD_HEX);
    assert_eq!(
        data, expected,
        "BIP-93 §93.4 payload bytes drifted; rust-codex32 bit-packing changed?"
    );
}

#[test]
fn bip93_vector_4_payload_round_trips_as_ms_codec_entr() {
    // Extract the 32-byte payload from BIP-93 §93.4 via upstream codex32.
    let c = Codex32String::from_string(BIP93_VECTOR_4.to_string()).unwrap();
    let payload_bytes: Vec<u8> = c.parts().data();
    assert_eq!(payload_bytes.len(), 32, "expected 32-B entr-bucket payload");

    // Re-encode those bytes via ms-codec entr (which prepends a 0x00 prefix
    // byte, so the resulting ms1 string differs from the upstream `leet` form).
    let s = encode(Tag::ENTR, &Payload::Entr(payload_bytes.clone()))
        .expect("re-encode of BIP-93 §93.4 payload as entr must succeed");

    // Round-trip: ms-codec decode recovers the same 32 bytes.
    let (tag, recovered) = decode(&s).expect("ms-codec must decode its own output");
    assert_eq!(tag, Tag::ENTR);
    let Payload::Entr(recovered_bytes) = recovered else {
        panic!("expected Payload::Entr after decode");
    };
    assert_eq!(recovered_bytes, payload_bytes);

    // Cross-format conformance: the ms-codec-emitted string is a valid BIP-93
    // codex32 string per the upstream parser (sub-format invariant).
    let _c2 = Codex32String::from_string(s.clone())
        .expect("ms-codec output must be parseable by upstream rust-codex32");
}

fn hex_decode(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
