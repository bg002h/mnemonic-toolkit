//! THE v0.2-MIGRATION SEAM. This is the only module that contacts `rust-codex32`.
//!
//! Why isolated: SPEC §2.2 + §10. When K-of-N share encoding ships in v0.2, only
//! this module changes — `discriminate()` adds prefix-byte dispatch, `package()`
//! gains the `Threshold` parameter. The rest of the crate is untouched.
//!
//! Why wire-position re-parse: `rust-codex32 v0.1.0`'s `Parts` struct (verified
//! at `src/lib.rs:383-392` of the upstream crate) has non-`pub` fields; only
//! `Parts::data() -> Vec<u8>` is publicly accessible. We cannot read
//! `parts.hrp` / `parts.threshold` / `parts.id` / `parts.share_index` from
//! outside the upstream crate. The re-parse below replays what
//! `rust-codex32`'s own `parts_inner` does internally (it's a fast O(n) string
//! parse on a string already proven valid by `Codex32String::from_string`).
//! Re-parse cost is negligible — the upstream `Parts<'s>` is `Copy`.
//!
//! Wire positions (relative to the `1` separator at index `sep`):
//!
//! ```text
//! threshold:   sep + 1                  (1 char; v0.1 = '0')
//! id:          sep + 2 .. sep + 6       (4 chars; type tag in v0.1)
//! share-index: sep + 6                  (1 char; v0.1 = 's')
//! payload:     sep + 7 .. s.len() - 13  (variable; -13 strips short cksum)
//! checksum:    s.len() - 13 .. s.len()  (13 chars; short only in v0.1)
//! ```
//!
//! For v0.1 we never see long-checksum strings (rejected by SPEC §4 rule 9
//! before this module is reached); `CHECKSUM_LEN_SHORT = 13` is hard-coded.

use crate::codex32::{Codex32String, Fe};
use crate::consts::{
    CHECKSUM_LEN_SHORT, HRP, MNEM_PREFIX, RESERVED_PREFIX, SEPARATOR, SHARE_INDEX_V01,
    THRESHOLD_V01,
};
use crate::error::{Error, Result};
use crate::payload::Payload;
use crate::tag::Tag;
use zeroize::Zeroizing;

/// Wire-position offsets relative to the separator index.
const THRESHOLD_OFFSET: usize = 1;
const ID_START_OFFSET: usize = 2;
const ID_END_OFFSET: usize = 6;
const SHARE_INDEX_OFFSET: usize = 6;
const PAYLOAD_START_OFFSET: usize = 7;

/// Wire fields extracted from a BIP-93-validated ms1 string.
#[derive(Debug, Clone, Copy)]
pub(crate) struct WireFields<'s> {
    pub hrp: &'s str,
    pub threshold_byte: u8,
    pub id_bytes: [u8; 4],
    pub share_index_byte: u8,
}

/// Re-parse a string already validated by `Codex32String::from_string` to
/// extract wire-position fields. Caller MUST pass only strings that successfully
/// round-tripped through `rust-codex32` parsing.
///
/// Returns `Err(Error::UnexpectedStringLength)` if the string is too short to
/// contain the fixed wire prefix (defensive only; unreachable for inputs that
/// passed BIP-93 parsing).
pub(crate) fn extract_wire_fields(s: &str) -> Result<WireFields<'_>> {
    let sep = s.rfind(SEPARATOR).ok_or_else(|| Error::WrongHrp {
        // Cap to 4 chars at construction (ms-codec-error-display-echoes-input,
        // 0.4.4): a separator-less secret string would otherwise ride whole in
        // `got`. char-counted (multibyte-safe); 4 < the 8-char leak window.
        got: s.chars().take(4).collect::<String>(),
    })?;
    // The fixed wire prefix after the separator is 7 chars (threshold + 4-char
    // id + share-index) + 13-char short checksum = 20. Any v0.1-shaped string
    // therefore needs at least sep + 20 bytes.
    if s.len() < sep + PAYLOAD_START_OFFSET + CHECKSUM_LEN_SHORT {
        return Err(Error::UnexpectedStringLength {
            got: s.len(),
            allowed: crate::consts::VALID_STR_LENGTHS,
        });
    }
    let bytes = s.as_bytes();
    let id_slice = &bytes[sep + ID_START_OFFSET..sep + ID_END_OFFSET];
    Ok(WireFields {
        hrp: &s[..sep],
        threshold_byte: bytes[sep + THRESHOLD_OFFSET],
        id_bytes: [id_slice[0], id_slice[1], id_slice[2], id_slice[3]],
        share_index_byte: bytes[sep + SHARE_INDEX_OFFSET],
    })
}

/// Lowercase the OWNED wire copy of a BIP-93-validated codex32 string.
///
/// Soundness — canonicalization, NOT case-laundering: `Codex32String` has
/// already enforced consistent case + a valid checksum (codex32 rejects
/// within-one-string MIXED case as `InvalidCase` before any value reaches
/// here), so lowercasing the owned copy is canonical-form normalization of
/// the BIP-173 uppercase (QR-alphanumeric) form. Used by the production
/// extraction sites (`discriminate`, `inspect`, `combine_shares`) so the
/// wire-field compares (`hrp != "ms"`, `share_index != b's'`, …) see the
/// canonical lowercase form.
///
/// The returned bare `String` carries secret material unwrapped — an explicit
/// PARITY decision with the existing `c.to_string()` copies along the parse
/// pipeline: wrapping only this site in `Zeroizing` while the pipeline holds
/// unwrapped copies would be theater. The repo-wide Zeroizing posture is
/// tracked separately.
pub(crate) fn wire_string(c: &Codex32String) -> String {
    c.to_string().to_ascii_lowercase()
}

/// Decode-side v0.2-migration seam. Given a BIP-93-validated codex32 string,
/// extract `(Tag, Payload)` via prefix-byte dispatch. Enforces wire-format
/// invariants: HRP="ms", threshold='0', share-index='s'.
/// Tag/payload-length validation against the tag table happens in `decode.rs`.
///
/// Prefix-byte dispatch:
/// - `0x00` (`RESERVED_PREFIX`) → `Payload::Entr(rest)`
/// - `0x02` (`MNEM_PREFIX`)     → `Payload::Mnem { language: rest[0], entropy: rest[1..] }`
///   (`.validate()` is called to reject unknown language codes immediately)
/// - any other prefix            → `Err(Error::ReservedPrefixViolation)`
pub(crate) fn discriminate(c: &Codex32String) -> Result<(Tag, Payload)> {
    let s = wire_string(c);
    let fields = extract_wire_fields(&s)?;

    // Wire-invariant checks (SPEC §4 rules 2, 3, 4).
    if fields.hrp != HRP {
        return Err(Error::WrongHrp {
            // Cap to 4 chars at construction (ms-codec-error-display-echoes-input,
            // 0.4.4); char-counted (multibyte-safe). 4 < the 8-char leak window.
            got: fields.hrp.chars().take(4).collect::<String>(),
        });
    }
    // Threshold-field dispatch (SPEC_ms_v0_2_kofn §1): '0' → v0.1 single-string
    // (proceed); '2'..'9' → one share of a K-of-N set (route to `ms combine`,
    // do NOT treat its garbage prefix byte as a payload kind); anything else →
    // the v0.1 ThresholdNotZero reject. The share-index check stays on the '0'
    // path only (a share's index is a non-`s` distributed index by design).
    match fields.threshold_byte {
        THRESHOLD_V01 => {
            if fields.share_index_byte != SHARE_INDEX_V01 {
                return Err(Error::ShareIndexNotSecret {
                    got: fields.share_index_byte as char,
                });
            }
        }
        b'2'..=b'9' => {
            return Err(Error::IsShareNotSingleString {
                threshold: fields.threshold_byte as char,
                index: fields.share_index_byte as char,
            });
        }
        other => {
            return Err(Error::ThresholdNotZero { got: other });
        }
    }

    // Tag construction (SPEC §4 rule 5; rule 6/7 happen later in decode.rs).
    let tag_bytes = fields.id_bytes;
    let tag_str = std::str::from_utf8(&tag_bytes)
        .map_err(|_| Error::TagInvalidAlphabet { got: tag_bytes })?;
    let tag = Tag::try_new(tag_str)?;

    // Payload extraction via the upstream Parts::data(). For any string that
    // passed `extract_wire_fields` (s.len >= sep + 7 + 13 = at least 22 chars)
    // and `Codex32String::from_string` (s.len >= 48 for short codex32), the
    // payload is at least 26 codex32 symbols ≈ 16 raw bytes, so it cannot be
    // empty. No defensive `is_empty` arm needed.
    //
    // SPEC v0.9.0 §1 item 2 — wrap the OWNED payload buffer in `Zeroizing`
    // so it scrubs on function exit. Caller is responsible for wrapping the
    // returned Payload bytes — see `payload.rs` doc-comment.
    let payload_with_prefix: Zeroizing<Vec<u8>> = Zeroizing::new(c.parts().data());

    // Prefix-byte dispatch (v0.2 type discriminator) — the header-gate-free
    // tail, shared with `combine_shares` (which has NO threshold/share-index
    // header to gate — the recovered secret-at-S carries a random id + threshold
    // k, so it must NOT route through the gate above).
    let payload = dispatch_payload(&payload_with_prefix)?;

    Ok((tag, payload))
}

/// Header-gate-free prefix→`Payload` dispatch: read `data[0]` and split the
/// remaining bytes into the typed `Payload`, then `validate()`. This is the
/// TAIL of `discriminate` factored out so `combine_shares` can reuse it WITHOUT
/// the threshold/share-index header gate (the recovered secret-at-S has a random
/// id + threshold `k`, so it must never re-enter that gate).
///
/// - `0x00` (`RESERVED_PREFIX`) → `Payload::Entr(rest)`
/// - `0x02` (`MNEM_PREFIX`)     → `Payload::Mnem { language: rest[0], entropy: rest[1..] }`
/// - any other prefix          → `Err(Error::ReservedPrefixViolation)`
///
/// `validate()` rejects unknown language codes / bad payload lengths. NOTE:
/// `data` is `Codex32String::parts().data()` (`Parts::data()`), NOT `c.data()`.
pub(crate) fn dispatch_payload(data: &[u8]) -> Result<Payload> {
    let payload = match data[0] {
        RESERVED_PREFIX => {
            // 0x00 → Entr: strip prefix, rest is raw entropy bytes.
            let p = Payload::Entr(data[1..].to_vec());
            // Validate length immediately; rejects non-standard entropy lengths.
            // Parity with the Mnem arm below + this fn's doc contract. WITHOUT
            // this, a valid-checksum but non-standard-length Entr share set
            // recovered via `combine_shares` flowed unvalidated to the CLI's
            // `from_entropy_in`, which panicked (audit I9, exit 101).
            p.validate()?;
            p
        }
        MNEM_PREFIX => {
            // 0x02 → Mnem: rest[0]=language, rest[1..]=entropy.
            // layout: [0x02][lang][entropy...].
            let language = data[1];
            let entropy = data[2..].to_vec();
            let p = Payload::Mnem { language, entropy };
            // Validate language code immediately; rejects unknown codes.
            p.validate()?;
            p
        }
        other => {
            return Err(Error::ReservedPrefixViolation { got: other });
        }
    };
    Ok(payload)
}

/// Assemble the on-wire payload bytes for a `Payload`: the `[prefix]||payload`
/// layout shared by `package()` (v0.1 single-string emit) and `encode_shares()`
/// (v0.2 K-of-N share-set emit). Wire layout by kind:
/// - `Payload::Entr(e)`                    → `[0x00][e...]`
/// - `Payload::Mnem { language, entropy }` → `[0x02][language][entropy...]`
///
/// The returned buffer is `Zeroizing` so it scrubs on drop (secret material).
/// `Payload` is a closed 2-variant enum within this crate (`#[non_exhaustive]`
/// only affects downstream crates), so the match is exhaustive.
pub(crate) fn payload_wire_bytes(p: &Payload) -> Zeroizing<Vec<u8>> {
    match p {
        Payload::Entr(e) => {
            // [0x00 reserved-prefix] || entropy — BYTE-IDENTICAL to v0.1.
            let mut v = Zeroizing::new(Vec::with_capacity(1 + e.len()));
            v.push(RESERVED_PREFIX);
            v.extend_from_slice(e);
            v
        }
        Payload::Mnem { language, entropy } => {
            // [0x02 mnem-prefix] || [language] || entropy
            let mut v = Zeroizing::new(Vec::with_capacity(2 + entropy.len()));
            v.push(MNEM_PREFIX);
            v.push(*language);
            v.extend_from_slice(entropy);
            v
        }
    }
}

/// Encode-side v0.2-migration seam. Given `(tag, payload)`, build a
/// BIP-93-validated codex32 string. Wire layout by kind:
/// - `Payload::Entr(e)`                → `[0x00][e...]` (byte-identical to v0.1)
/// - `Payload::Mnem { language, entropy }` → `[0x02][language][entropy...]`
///
/// Fixed wire-field values: threshold=0, share-index='s'.
///
/// SPEC v0.9.0 §1 item 2 — the OWNED encode buffer is wrapped in `Zeroizing`
/// so it scrubs on function exit (tracked at `rust-codex32-zeroize-upstream`).
pub(crate) fn package(tag: Tag, payload: &Payload) -> Result<Codex32String> {
    let data: Zeroizing<Vec<u8>> = payload_wire_bytes(payload);

    // Delegate to rust-codex32. Always uses threshold=0, share=Fe::S.
    // `?` leverages the From<crate::codex32::Error> for Error impl in error.rs.
    Ok(Codex32String::from_seed(
        HRP,
        0,
        tag.as_str(),
        Fe::S,
        &data[..],
    )?)
}

#[cfg(test)]
mod tests_extract {
    use super::*;

    #[test]
    fn bip93_test_vector_1_extracts_correctly() {
        // From rust-codex32 src/lib.rs bip_vector_1 test (BIP-93 vector 1):
        // hrp="ms", threshold=0, id="test", share_index='s', payload=26 'x' chars.
        let s = "ms10testsxxxxxxxxxxxxxxxxxxxxxxxxxx4nzvca9cmczlw";
        let fields = extract_wire_fields(s).unwrap();
        assert_eq!(fields.hrp, "ms");
        assert_eq!(fields.threshold_byte, b'0');
        assert_eq!(&fields.id_bytes, b"test");
        assert_eq!(fields.share_index_byte, b's');
    }

    #[test]
    fn rejects_too_short_string() {
        // "ms1" alone is below the minimum.
        assert!(matches!(
            extract_wire_fields("ms1"),
            Err(Error::UnexpectedStringLength { .. })
        ));
    }
}

#[cfg(test)]
mod tests_discriminate {
    use super::*;

    fn build_v01_entr(entropy: &[u8]) -> Codex32String {
        let mut data = vec![RESERVED_PREFIX];
        data.extend_from_slice(entropy);
        Codex32String::from_seed(HRP, 0, "entr", Fe::S, &data).unwrap()
    }

    #[test]
    fn v01_entr_16_round_trips_through_discriminate() {
        let entropy = vec![0xAAu8; 16];
        let c = build_v01_entr(&entropy);
        let (tag, recovered) = discriminate(&c).unwrap();
        assert_eq!(tag, Tag::ENTR);
        assert_eq!(recovered, Payload::Entr(entropy));
    }

    #[test]
    fn v01_entr_32_round_trips_through_discriminate() {
        let entropy = vec![0x55u8; 32];
        let c = build_v01_entr(&entropy);
        let (tag, recovered) = discriminate(&c).unwrap();
        assert_eq!(tag, Tag::ENTR);
        assert_eq!(recovered, Payload::Entr(entropy));
    }

    #[test]
    fn discriminate_rejects_non_zero_prefix() {
        let mut data = vec![0x01u8];
        data.extend_from_slice(&[0xAAu8; 16]);
        let c = Codex32String::from_seed(HRP, 0, "entr", Fe::S, &data).unwrap();
        assert!(matches!(
            discriminate(&c),
            Err(Error::ReservedPrefixViolation { got: 0x01 })
        ));
    }

    #[test]
    fn discriminate_rejects_wrong_hrp() {
        let mut data = vec![RESERVED_PREFIX];
        data.extend_from_slice(&[0xAAu8; 16]);
        let c = Codex32String::from_seed("mq", 0, "entr", Fe::S, &data).unwrap();
        assert!(matches!(discriminate(&c), Err(Error::WrongHrp { .. })));
    }

    #[test]
    fn discriminate_mnem_prefix_returns_mnem_payload() {
        let entropy = vec![0xBBu8; 16];
        let mut data = vec![MNEM_PREFIX, 0x02u8]; // language=2 (Korean)
        data.extend_from_slice(&entropy);
        let c = Codex32String::from_seed(HRP, 0, "entr", Fe::S, &data).unwrap();
        let (tag, recovered) = discriminate(&c).unwrap();
        assert_eq!(tag, Tag::ENTR);
        assert_eq!(
            recovered,
            Payload::Mnem {
                language: 2,
                entropy
            }
        );
    }

    #[test]
    fn discriminate_routes_threshold_2_to_is_share() {
        // A threshold=2 / non-`s` index string is one share of a K-of-N set.
        // discriminate must route it to IsShareNotSingleString, NOT ThresholdNotZero.
        let data = vec![0xAAu8; 16];
        let c = Codex32String::from_seed(HRP, 2, "tst7", Fe::P, &data).unwrap();
        match discriminate(&c) {
            Err(Error::IsShareNotSingleString { threshold, index }) => {
                assert_eq!(threshold, '2');
                assert_eq!(index, 'p');
            }
            other => panic!("expected IsShareNotSingleString, got {other:?}"),
        }
    }

    #[test]
    fn threshold_1_is_unconstructible_so_never_reaches_discriminate() {
        // m2 (P1-R0): threshold '1' is NOT a valid codex32 share threshold
        // (BIP-93 admits 0 = unshared or 2..=9 = K-of-N; never 1). Two facts
        // pin this — proven empirically against the pinned codex32 0.1.0:
        //
        // (a) `from_seed(.., 1, ..)` is rejected at construction with
        //     `InvalidThresholdN(1)` — you cannot MINT a threshold-1 string.
        let mut data = vec![0x00u8];
        data.extend_from_slice(&[0xAAu8; 16]);
        match Codex32String::from_seed(HRP, 1, "tst7", Fe::A, &data) {
            Err(crate::codex32::Error::InvalidThresholdN(1)) => {}
            other => panic!("expected InvalidThresholdN(1) from from_seed, got {other:?}"),
        }

        // (b) Hand-flipping a valid threshold-2 string's threshold char to '1'
        //     breaks the BCH checksum (the threshold char is covered by the
        //     code), so `from_string` rejects it with `InvalidChecksum` — it
        //     never reaches `discriminate`'s threshold gate. There is therefore
        //     NO validly-checksummed threshold-'1' string for `discriminate` to
        //     route; the `other => ThresholdNotZero` arm is unreachable for '1'.
        let s2 = Codex32String::from_seed(HRP, 2, "tst7", Fe::A, &data)
            .unwrap()
            .to_string();
        let sep = s2.rfind('1').expect("codex32 separator '1' present");
        let mut bytes = s2.into_bytes();
        bytes[sep + 1] = b'1'; // threshold char '2' -> '1'
        let forged = String::from_utf8(bytes).unwrap();
        assert!(
            matches!(
                Codex32String::from_string(forged),
                Err(crate::codex32::Error::InvalidChecksum { .. })
            ),
            "a forged threshold-'1' char must fail the BCH checksum at from_string",
        );
    }
}

#[cfg(test)]
mod tests_wire_bytes {
    use super::*;

    #[test]
    fn entr_wire_bytes_are_prefix_plus_entropy() {
        let p = Payload::Entr(vec![0xABu8; 16]);
        let mut expected = vec![0x00u8];
        expected.extend(std::iter::repeat_n(0xABu8, 16));
        assert_eq!(&payload_wire_bytes(&p)[..], &expected[..]);
    }

    #[test]
    fn mnem_wire_bytes_are_prefix_lang_entropy() {
        let p = Payload::Mnem {
            language: 1,
            entropy: vec![0xABu8; 16],
        };
        let mut expected = vec![0x02u8, 0x01u8];
        expected.extend(std::iter::repeat_n(0xABu8, 16));
        assert_eq!(&payload_wire_bytes(&p)[..], &expected[..]);
    }
}

#[cfg(test)]
mod tests_package {
    use super::*;

    #[test]
    fn package_entr_round_trips_through_discriminate() {
        for len in [16usize, 20, 24, 28, 32] {
            let entropy = vec![0xAAu8; len];
            let p = Payload::Entr(entropy.clone());
            let c = package(Tag::ENTR, &p).unwrap();
            let (tag, recovered) = discriminate(&c).unwrap();
            assert_eq!(tag, Tag::ENTR);
            assert_eq!(recovered, Payload::Entr(entropy));
        }
    }

    #[test]
    fn package_mnem_round_trips_through_discriminate() {
        for len in [16usize, 20, 24, 28, 32] {
            let entropy = vec![0xCCu8; len];
            let p = Payload::Mnem {
                language: 3,
                entropy: entropy.clone(),
            };
            let c = package(Tag::ENTR, &p).unwrap();
            let (tag, recovered) = discriminate(&c).unwrap();
            assert_eq!(tag, Tag::ENTR);
            assert_eq!(
                recovered,
                Payload::Mnem {
                    language: 3,
                    entropy
                }
            );
        }
    }

    #[test]
    fn package_produces_str_lengths_in_v01_set() {
        let expected_lengths = crate::consts::VALID_STR_LENGTHS;
        for (i, len) in [16usize, 20, 24, 28, 32].iter().enumerate() {
            let entropy = vec![0xAAu8; *len];
            let p = Payload::Entr(entropy);
            let c = package(Tag::ENTR, &p).unwrap();
            let s = c.to_string();
            assert_eq!(
                s.len(),
                expected_lengths[i],
                "length mismatch for {}-B entr entropy: got {}, expected {}",
                len,
                s.len(),
                expected_lengths[i]
            );
        }
    }

    #[test]
    fn package_mnem_produces_str_lengths_in_mnem_set() {
        let expected_lengths = crate::consts::VALID_MNEM_STR_LENGTHS;
        for (i, len) in [16usize, 20, 24, 28, 32].iter().enumerate() {
            let entropy = vec![0xAAu8; *len];
            let p = Payload::Mnem {
                language: 0,
                entropy,
            };
            let c = package(Tag::ENTR, &p).unwrap();
            let s = c.to_string();
            assert_eq!(
                s.len(),
                expected_lengths[i],
                "length mismatch for {}-B mnem entropy: got {}, expected {}",
                len,
                s.len(),
                expected_lengths[i]
            );
        }
    }
}
