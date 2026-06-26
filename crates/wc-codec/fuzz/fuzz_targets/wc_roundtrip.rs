//! Fuzz target: `wc-codec` encode → decode round-trip (P6).
//!
//! Oracle (the recoverability charter): for ANY `(SourceKind, payload,
//! payload_bits)` that `encode` accepts, a CLEAN `decode` of the resulting word
//! list MUST recover the EXACT canonical `(payload, payload_bits)` — never a
//! wrong payload, never a panic. Parity / integrity-bit width are drawn from the
//! fuzz input so the geometry varies. Any panic, or any payload mismatch on a
//! clean (uncorrupted) round-trip, is a real finding.
#![no_main]

use libfuzzer_sys::fuzz_target;
use wc_codec::{decode, encode, EncodeOpts, SourceKind};

fuzz_target!(|data: &[u8]| {
    if data.len() < 4 {
        return;
    }
    // Header bytes pick the geometry; the rest is the payload.
    let kind = if data[0] & 1 == 0 {
        SourceKind::Mk1Xpub
    } else {
        SourceKind::Md1Descriptor
    };
    // integrity_bits in [33, 64].
    let integrity_bits = 33 + (data[1] % 32);
    // parity_words in [0, 31].
    let parity_words = (data[2] % 32) as usize;

    let payload = &data[3..];
    // Cap the payload so the codeword stays under the GF(2^11) field cap (2047
    // distinct evaluation points). A few hundred bytes is plenty of variety.
    if payload.is_empty() || payload.len() > 240 {
        return;
    }

    // For mk1 the payload is byte-aligned; for md1 we may shave a few trailing
    // bits to exercise the bit-precise path.
    let max_bits = payload.len() * 8;
    let payload_bits = match kind {
        SourceKind::Mk1Xpub => max_bits,
        SourceKind::Md1Descriptor => {
            // Trim 0..7 trailing bits, but keep at least 1 bit.
            let trim = (data[0] >> 1) as usize % 8;
            max_bits.saturating_sub(trim).max(1)
        }
    };

    let opts = EncodeOpts {
        parity_words,
        integrity_bits,
        ..Default::default()
    };

    let words = match encode(kind, payload, payload_bits, &opts) {
        Ok(w) => w,
        Err(_) => return, // rejected geometry — fine, just not a round-trip case
    };

    // Clean decode MUST recover the EXACT canonical payload.
    let word_refs: Vec<&str> = words.iter().copied().collect();
    let decoded = decode(&word_refs).expect("clean round-trip must decode");
    assert_eq!(decoded.kind, kind, "kind must round-trip");
    assert_eq!(
        decoded.payload_bits, payload_bits,
        "payload_bits must round-trip"
    );

    // Compare against the CANONICAL payload form (trailing sub-byte bits zeroed),
    // which is exactly what decode recovers.
    let n_bytes = payload_bits.div_ceil(8);
    let mut canonical = vec![0u8; n_bytes];
    for i in 0..payload_bits {
        let bit = (payload[i / 8] >> (7 - (i % 8))) & 1;
        if bit != 0 {
            canonical[i / 8] |= bit << (7 - (i % 8));
        }
    }
    assert_eq!(decoded.payload, canonical, "payload must round-trip exactly");
});
