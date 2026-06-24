//! Property-based round-trip tests: encode → decode → assert equal payload + tag,
//! across all 5 entr byte lengths.

use ms_codec::{decode, encode, Payload, Tag};
use proptest::prelude::*;

fn entropy_strategy(len: usize) -> impl Strategy<Value = Vec<u8>> {
    proptest::collection::vec(any::<u8>(), len..=len)
}

proptest! {
    #[test]
    fn round_trip_entr_16(entropy in entropy_strategy(16)) {
        let p = Payload::Entr(entropy.clone());
        let s = encode(Tag::ENTR, &p).unwrap();
        let (tag, recovered) = decode(&s).unwrap();
        prop_assert_eq!(tag, Tag::ENTR);
        prop_assert_eq!(recovered, Payload::Entr(entropy));
    }

    #[test]
    fn round_trip_entr_20(entropy in entropy_strategy(20)) {
        let p = Payload::Entr(entropy.clone());
        let s = encode(Tag::ENTR, &p).unwrap();
        let (tag, recovered) = decode(&s).unwrap();
        prop_assert_eq!(tag, Tag::ENTR);
        prop_assert_eq!(recovered, Payload::Entr(entropy));
    }

    #[test]
    fn round_trip_entr_24(entropy in entropy_strategy(24)) {
        let p = Payload::Entr(entropy.clone());
        let s = encode(Tag::ENTR, &p).unwrap();
        let (tag, recovered) = decode(&s).unwrap();
        prop_assert_eq!(tag, Tag::ENTR);
        prop_assert_eq!(recovered, Payload::Entr(entropy));
    }

    #[test]
    fn round_trip_entr_28(entropy in entropy_strategy(28)) {
        let p = Payload::Entr(entropy.clone());
        let s = encode(Tag::ENTR, &p).unwrap();
        let (tag, recovered) = decode(&s).unwrap();
        prop_assert_eq!(tag, Tag::ENTR);
        prop_assert_eq!(recovered, Payload::Entr(entropy));
    }

    #[test]
    fn round_trip_entr_32(entropy in entropy_strategy(32)) {
        let p = Payload::Entr(entropy.clone());
        let s = encode(Tag::ENTR, &p).unwrap();
        let (tag, recovered) = decode(&s).unwrap();
        prop_assert_eq!(tag, Tag::ENTR);
        prop_assert_eq!(recovered, Payload::Entr(entropy));
    }
}
