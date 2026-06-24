//! One negative test per SPEC §4 decoder rule. Each test hand-constructs an
//! ms1 string that violates exactly one rule and asserts the corresponding
//! Error variant.

use ms_codec::codex32::{Codex32String, Fe};
use ms_codec::{decode, Error};

const VALID_PREFIX: u8 = 0x00;
const ENTROPY_16: &[u8] = &[0xAAu8; 16];

fn build_with(
    hrp: &str,
    threshold: usize,
    id: &str,
    share: Fe,
    prefix: u8,
    payload: &[u8],
) -> String {
    let mut data = vec![prefix];
    data.extend_from_slice(payload);
    Codex32String::from_seed(hrp, threshold, id, share, &data)
        .unwrap()
        .to_string()
}

#[test]
fn rule_1_invalid_checksum_rejected() {
    // Take a valid string and flip the last char to break BCH.
    let s = build_with("ms", 0, "entr", Fe::S, VALID_PREFIX, ENTROPY_16);
    let mut bytes = s.into_bytes();
    let last = bytes.len() - 1;
    bytes[last] = if bytes[last] == b'q' { b'p' } else { b'q' };
    let bad = String::from_utf8(bytes).unwrap();
    assert!(matches!(decode(&bad), Err(Error::Codex32(_))));
}

#[test]
fn rule_2_wrong_hrp_rejected() {
    // Build with HRP "mq" instead of "ms". HRP byte length is the same (2);
    // total string length is identical to the "ms" case (50). Length check
    // passes, upstream parse passes, our envelope::discriminate fires
    // WrongHrp deterministically. (SPEC §4 numbers the rules but doesn't
    // mandate check-order; rule 9 happens before rule 1 in our impl as a
    // defensive optimization, not as a SPEC requirement.)
    let s = build_with("mq", 0, "entr", Fe::S, VALID_PREFIX, ENTROPY_16);
    assert_eq!(s.len(), 50, "sanity: HRP swap doesn't change string length");
    assert!(matches!(decode(&s), Err(Error::WrongHrp { .. })));
}

#[test]
fn rule_3_threshold_2_routes_to_is_share() {
    // v0.2 (SPEC_ms_v0_2_kofn §1): the v0.1 ThresholdNotZero hard-reject for
    // threshold∈2..9 is RELAXED into a route — a threshold=2 string is one share
    // of a K-of-N set, so decode surfaces IsShareNotSingleString (directing the
    // user to `ms combine`), NOT ThresholdNotZero.
    //
    // Threshold = 2 with share_index = Fe::A produces a valid-length string
    // (9 fixed + 28 payload + 13 cksum = 50, in VALID_STR_LENGTHS). Length check
    // passes; upstream from_string accepts threshold=2 + share=A; our envelope
    // discriminate routes it deterministically.
    let s = build_with("ms", 2, "entr", Fe::A, VALID_PREFIX, ENTROPY_16);
    assert_eq!(
        s.len(),
        50,
        "sanity: 16-B + 0x00 prefix in threshold-2 form is 50 chars"
    );
    match decode(&s) {
        Err(Error::IsShareNotSingleString { threshold, index }) => {
            assert_eq!(threshold, '2');
            assert_eq!(index, 'a');
        }
        other => panic!("expected IsShareNotSingleString, got {other:?}"),
    }
}

#[test]
fn rule_4_share_index_not_secret_rejected() {
    // For threshold=0 with share_index != Fe::S, BIP-93 itself rejects at
    // upstream parse (rust-codex32 v0.1.0 lib.rs:202-204:
    // `if ret.threshold == 0 && ret.share_index != Fe::S { return InvalidShareIndex(...) }`).
    // Build a valid-length, valid-checksum string with share=Fe::C and confirm
    // our decoder surfaces Error::Codex32 wrapping the upstream error.
    let s = build_with("ms", 0, "entr", Fe::C, VALID_PREFIX, ENTROPY_16);
    assert_eq!(
        s.len(),
        50,
        "sanity: valid v0.1 length so the rule 9 length-check passes"
    );
    assert!(matches!(decode(&s), Err(Error::Codex32(_))));
}

#[test]
fn rule_5_tag_invalid_alphabet_unreachable_via_decode() {
    // Tag bytes outside the codex32 alphabet would be rejected at upstream parse
    // (rust-codex32 validates every char in the data part is in the alphabet).
    // Our rule 5 path is therefore defensive-only. No-op test documents this.
}

#[test]
fn rule_6_unknown_tag_rejected() {
    // Build with id="wxyz" — codex32-alphabet-valid (w/x/y/z all in
    // qpzry9x8gf2tvdw0s3jn54khce6mua7l) but NOT in RESERVED_TAG_TABLE.
    // Note: 'b', 'i', 'o', '1' are excluded from the codex32 alphabet
    // for OCR safety, so "abcd" / "iron" / "boat" would fail at upstream
    // parse (Codex32 variant) before reaching our rule 6.
    let s = build_with("ms", 0, "wxyz", Fe::S, VALID_PREFIX, ENTROPY_16);
    assert!(matches!(decode(&s), Err(Error::UnknownTag { .. })));
}

#[test]
fn rule_7_reserved_not_emitted_tags_rejected() {
    // "mnem" as a TAG is not in RESERVED_NOT_EMITTED_V01 any more (v0.2 removed
    // it from the not-emitted set since the Mnem payload now uses the "entr" tag
    // with a 0x02 prefix byte). "mnem" as a tag now falls through to UnknownTag.
    for reserved in ["seed", "xprv", "prvk"] {
        let s = build_with("ms", 0, reserved, Fe::S, VALID_PREFIX, ENTROPY_16);
        let err = decode(&s).unwrap_err();
        assert!(
            matches!(err, Error::ReservedTagNotEmittedInV01 { got: _ }),
            "tag {:?}: expected ReservedTagNotEmittedInV01, got {:?}",
            reserved,
            err
        );
    }
    // "mnem" as a tag (distinct from the mnem payload kind which uses entr tag +
    // 0x02 prefix) is an UnknownTag — it is no longer reserved-not-emitted.
    let s_mnem_tag = build_with("ms", 0, "mnem", Fe::S, VALID_PREFIX, ENTROPY_16);
    assert!(
        matches!(decode(&s_mnem_tag), Err(Error::UnknownTag { .. })),
        "mnem as a tag should be UnknownTag in v0.2, got {:?}",
        decode(&s_mnem_tag)
    );
}

#[test]
fn rule_8_reserved_prefix_violation_rejected() {
    // Build with prefix byte = 0x01 instead of 0x00.
    let s = build_with("ms", 0, "entr", Fe::S, 0x01, ENTROPY_16);
    assert!(matches!(
        decode(&s),
        Err(Error::ReservedPrefixViolation { got: 0x01 })
    ));
}

#[test]
fn rule_9_unexpected_string_length_rejected() {
    // 52 chars: outside both the entr set [50,56,62,69,75] and the mnem set
    // [51,58,64,70,77] — guaranteed to be rejected at the union length gate.
    // (51 is now a valid mnem length in v0.2, so we use 52 instead.)
    let s = "ms10entrsxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx";
    assert_eq!(s.len(), 52);
    assert!(matches!(
        decode(s),
        Err(Error::UnexpectedStringLength { got: 52, .. })
    ));
}

#[test]
fn rule_10_payload_length_mismatch_unreachable_via_decode() {
    // Rule 10 (Payload::validate post-extraction) cannot be reached for valid
    // inputs because rule 9 (string length) fires first. The two rules are
    // length-set-equivalent: VALID_STR_LENGTHS bijects with VALID_ENTR_LENGTHS
    // via the 22-fixed-char prefix (locked by the consts.rs bijection test).
    // Defensive-only path. No-op test documents this.
}
