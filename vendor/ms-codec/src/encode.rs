//! Public encoder. v0.1 entr-only; future kinds in v0.2+ via the envelope seam.

use crate::consts::RESERVED_NOT_EMITTED_V01;
use crate::envelope;
use crate::error::{Error, Result};
use crate::payload::Payload;
use crate::tag::Tag;

/// Encode a `(Tag, Payload)` as a v0.1 ms1 string.
///
/// Per SPEC §3.5 + §3.5.1:
/// - Encoder validates `Payload` length first (rejects out-of-set entr lengths).
/// - Encoder rejects reserved-not-emitted tags symmetrically with the decoder
///   (SPEC §4 rule 7), preventing a v0.1 ms-codec from emitting a string that
///   v0.1 ms-codec itself cannot decode.
pub fn encode(tag: Tag, payload: &Payload) -> Result<String> {
    // §3.5.1: encoder symmetry on reserved-not-emitted tags.
    if RESERVED_NOT_EMITTED_V01.contains(tag.as_bytes()) {
        return Err(Error::ReservedTagNotEmittedInV01 {
            got: *tag.as_bytes(),
        });
    }
    // §3.5: payload length validation.
    payload.validate()?;
    // Hand off to envelope.
    let c = envelope::package(tag, payload)?;
    Ok(c.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::VALID_STR_LENGTHS;

    #[test]
    fn encode_entr_all_lengths_succeed() {
        for (i, len) in [16usize, 20, 24, 28, 32].iter().enumerate() {
            let p = Payload::Entr(vec![0xAAu8; *len]);
            let s = encode(Tag::ENTR, &p).unwrap();
            assert_eq!(s.len(), VALID_STR_LENGTHS[i]);
            assert!(s.starts_with("ms10entrs"), "got {}", s);
        }
    }

    #[test]
    fn encode_rejects_seed_tag() {
        let p = Payload::Entr(vec![0u8; 16]);
        let seed_tag = Tag::try_new("seed").unwrap();
        assert!(matches!(
            encode(seed_tag, &p),
            Err(Error::ReservedTagNotEmittedInV01 { .. })
        ));
    }

    #[test]
    fn encode_rejects_xprv_tag() {
        let p = Payload::Entr(vec![0u8; 16]);
        let xprv_tag = Tag::try_new("xprv").unwrap();
        assert!(matches!(
            encode(xprv_tag, &p),
            Err(Error::ReservedTagNotEmittedInV01 { .. })
        ));
    }

    #[test]
    fn encode_rejects_off_by_one_entr_length() {
        let p = Payload::Entr(vec![0u8; 17]);
        assert!(matches!(
            encode(Tag::ENTR, &p),
            Err(Error::PayloadLengthMismatch { .. })
        ));
    }
}
