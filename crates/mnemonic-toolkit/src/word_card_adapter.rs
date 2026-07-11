//! Canonical-payload adapter between the m-format sibling codecs (`mk1` / `md1`)
//! and the codec-agnostic Word-Card engine (`wc-codec`).
//!
//! Realizes `design/IMPLEMENTATION_PLAN_word_card_encoding.md` §2 (canonical-
//! payload adapter, toolkit-owned) + §6.2 (toolkit CLI surface) for **P6**.
//!
//! `wc-codec` is deliberately codec-AGNOSTIC: it operates on a
//! `(SourceKind, payload, payload_bits)` triple and knows nothing about `mk1` /
//! `md1` structure (plan §2). This module is the thin boundary that:
//!
//! 1. decodes an `mk1` / `md1` string into the sibling codec's in-memory type
//!    (reusing the SAME public decode entry points the rest of the toolkit uses
//!    — `mk_codec::decode` / `md_codec::reassemble`, see `cmd::inspect`),
//! 2. lifts it to the codec's **canonical pre-chunking payload bytes** via the
//!    P0 accessors (`KeyCard::canonical_payload_bytes` /
//!    `Descriptor::canonical_payload_bytes`),
//! 3. and inverts: from a recovered `(SourceKind, bytes, bits)` it rebuilds the
//!    `KeyCard` / `Descriptor` and re-emits the `m*1` string (and the
//!    xpub / descriptor text).
//!
//! # The mk1 vs md1 bit-length asymmetry (load-bearing)
//!
//! - **mk1** canonical bytes are **byte-aligned**: `payload_bits = 8 * len`.
//! - **md1** canonical bytes are **bit-precise**: the descriptor packer returns
//!   `(bytes, total_bits)` where `total_bits` is generally NOT a multiple of 8
//!   (the final byte carries up to 7 trailing zero-pad bits). `total_bits` is
//!   **load-bearing** — it MUST be carried verbatim into `wc-codec` and back
//!   into `Descriptor::from_canonical_payload_bytes`, never `bytes.len() * 8`.
//!
//! # mk1 re-encode is NOT string-deterministic (plan §7 P6 KAT note)
//!
//! `mk_codec::encode` draws a FRESH CSPRNG `chunk_set_id` for multi-chunk cards,
//! so a re-emitted `mk1` string is NEVER byte-identical to the original — the
//! recovered **`KeyCard` / xpub / canonical-payload** is. Round-trip assertions
//! therefore compare the recovered payload, not the literal string. `md1` IS
//! string-deterministic, so its literal string MAY additionally be asserted.

use crate::error::ToolkitError;
use wc_codec::SourceKind;

/// The recovered, decoded card after a Word-Card `decode` (the inverse adapter
/// output). Carries the rebuilt sibling-codec type plus its re-emitted `m*1`
/// string and the human-readable xpub / descriptor text.
#[derive(Debug, Clone)]
pub enum RecoveredCard {
    /// An `mk1` xpub card. `mk1` is the re-emitted (NON-deterministic
    /// `chunk_set_id`) string set; `xpub` is the stable identity.
    Mk1 {
        /// The rebuilt key card.
        card: mk_codec::KeyCard,
        /// The re-emitted `mk1` chunk(s) (fresh `chunk_set_id` — see module doc).
        mk1: Vec<String>,
        /// The stable xpub string (the deterministic identity).
        xpub: String,
    },
    /// An `md1` descriptor card. `md1` is string-deterministic (`md_codec::split`
    /// derives the `chunk_set_id` from a hash of the descriptor, not a CSPRNG).
    Md1 {
        /// The rebuilt descriptor.
        descriptor: md_codec::Descriptor,
        /// The re-emitted (deterministic) `md1` chunk(s). A small descriptor is
        /// a single chunk; a large one spans several.
        md1: Vec<String>,
    },
}

/// The canonical-payload triple handed to `wc-codec::encode` (plan §2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalPayload {
    /// Which sibling codec produced the payload.
    pub kind: SourceKind,
    /// The canonical pre-chunking payload bytes (zero-padded to a byte boundary
    /// for `md1`; exactly the bytecode for `mk1`).
    pub bytes: Vec<u8>,
    /// The EXACT payload bit length. `8 * bytes.len()` for `mk1`; the bit-precise
    /// `total_bits` for `md1` (load-bearing — see module doc).
    pub payload_bits: usize,
}

/// Lift an `mk1` chunk set to its canonical Word-Card payload.
///
/// Reuses `mk_codec::decode` (the SAME entry point `cmd::inspect::decode_card`
/// uses) → `KeyCard::canonical_payload_bytes`. `mk1` bytecode is byte-aligned, so
/// `payload_bits = 8 * len` and `kind = SourceKind::Mk1Xpub`.
pub fn mk1_to_canonical(chunks: &[&str]) -> Result<CanonicalPayload, ToolkitError> {
    let card = mk_codec::decode(chunks).map_err(ToolkitError::from)?;
    let bytes = card.canonical_payload_bytes().map_err(ToolkitError::from)?;
    let payload_bits = bytes.len() * 8;
    Ok(CanonicalPayload {
        kind: SourceKind::Mk1Xpub,
        bytes,
        payload_bits,
    })
}

/// Lift an `md1` chunk set to its canonical Word-Card payload.
///
/// Reuses `md_codec::reassemble` (the SAME entry point `cmd::inspect::decode_card`
/// uses) → `Descriptor::canonical_payload_bytes`, which returns the bit-precise
/// `(bytes, total_bits)`. `payload_bits` carries the EXACT `total_bits`
/// (load-bearing — see module doc) and `kind = SourceKind::Md1Descriptor`.
pub fn md1_to_canonical(chunks: &[&str]) -> Result<CanonicalPayload, ToolkitError> {
    let desc = md_codec::reassemble(chunks).map_err(ToolkitError::from)?;
    let (bytes, total_bits) = desc.canonical_payload_bytes().map_err(ToolkitError::from)?;
    Ok(CanonicalPayload {
        kind: SourceKind::Md1Descriptor,
        bytes,
        payload_bits: total_bits,
    })
}

/// Auto-route one `m*1` card (one or MORE chunk strings) to its canonical
/// payload by HRP prefix. A multi-chunk `mk1` / `md1` card is supplied as all of
/// its chunk strings together. `mk1…` → [`mk1_to_canonical`]; `md1…` →
/// [`md1_to_canonical`]. Any other prefix (or a mixed-HRP set) is refused with a
/// clear [`ToolkitError::UnknownHrp`] (the Word-Card encoder only consumes the
/// two PUBLIC-material codecs; `ms1` entropy is a SECRET and is intentionally NOT
/// word-card-able).
pub fn chunks_to_canonical(chunks: &[&str]) -> Result<CanonicalPayload, ToolkitError> {
    let first = chunks.first().map(|s| s.trim()).unwrap_or("");
    let lc = first.to_ascii_lowercase();
    if lc.starts_with("mk1") {
        mk1_to_canonical(chunks)
    } else if lc.starts_with("md1") {
        md1_to_canonical(chunks)
    } else {
        Err(ToolkitError::UnknownHrp {
            got: first.to_string(),
            expected_one_of: vec!["mk1", "md1"],
        })
    }
}

/// Convenience: route a single `m*1` string (one chunk OR a whitespace-separated
/// chunk list of one card) to its canonical payload. Splits on whitespace into
/// chunk tokens, then delegates to [`chunks_to_canonical`].
pub fn string_to_canonical(s: &str) -> Result<CanonicalPayload, ToolkitError> {
    let tokens: Vec<&str> = s.split_whitespace().collect();
    if tokens.is_empty() {
        return Err(ToolkitError::UnknownHrp {
            got: String::new(),
            expected_one_of: vec!["mk1", "md1"],
        });
    }
    chunks_to_canonical(&tokens)
}

/// Invert: from a recovered `(SourceKind, bytes, payload_bits)` rebuild the
/// sibling-codec type and re-emit the `m*1` string(s) + text identity (plan §2).
///
/// - `Mk1Xpub` → `KeyCard::from_canonical_payload_bytes` → `mk_codec::encode`
///   (fresh `chunk_set_id`; the xpub is the stable identity).
/// - `Md1Descriptor` → `Descriptor::from_canonical_payload_bytes(bytes,
///   payload_bits)` → `md_codec::encode_md1_string` (deterministic).
pub fn canonical_to_recovered(
    kind: SourceKind,
    bytes: &[u8],
    payload_bits: usize,
) -> Result<RecoveredCard, ToolkitError> {
    match kind {
        SourceKind::Mk1Xpub => {
            let card = mk_codec::KeyCard::from_canonical_payload_bytes(bytes)
                .map_err(ToolkitError::from)?;
            let xpub = card.xpub.to_string();
            let mk1 = mk_codec::encode(&card).map_err(ToolkitError::from)?;
            Ok(RecoveredCard::Mk1 { card, mk1, xpub })
        }
        SourceKind::Md1Descriptor => {
            let descriptor =
                md_codec::Descriptor::from_canonical_payload_bytes(bytes, payload_bits)
                    .map_err(ToolkitError::from)?;
            // `split` handles single AND multi-chunk md1 deterministically (the
            // `chunk_set_id` is a hash of the descriptor — NOT a CSPRNG, unlike
            // mk1). `encode_md1_string` would refuse a descriptor whose payload
            // exceeds one codex32 string (the common multi-chunk md1 case).
            let md1 = md_codec::split(&descriptor).map_err(ToolkitError::from)?;
            Ok(RecoveredCard::Md1 { descriptor, md1 })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Real fixtures generated from the all-`abandon…about` BIP-39 seed at
    // `mnemonic bundle --network mainnet --template bip84` (deterministic seed,
    // fixed origin path m/84'/0'/0'). The mk1 is a 2-chunk card; the md1 a
    // 3-chunk card. These are PUBLIC material (xpub / descriptor), no secrets.
    const MK1: [&str; 2] = [
        "mk1qprsqhpqqsq3cqtsleeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8qp3yxg3dpe854wq4",
        "mk1qprsqhpp0f30mtxzd65mvwcur9usdatwuqvq6z70r9nwrgk6xn6l8gy6nwa2n977sw6zh34rma0nh",
    ];
    const MD1: [&str; 3] = [
        "md1fgdxlpqpqpm6jzzqqvqpdqw0za5zs4gyy55aq4vsmnhy4s6wyaypu34c7raqu8np",
        "md1fgdxlpqf2zcgefcpupmel75q5435j7seugaj5jr7qyur6vt76es5cdeyrq7zdy0d",
        "md1fgdxlpq3xa2dk8vwpj7gx74hwqxqdp083jehp5tdrfa0n5zdfkqcdlrvnh5r62jn",
    ];

    #[test]
    fn mk1_canonical_is_byte_aligned_mk1xpub() {
        let cp = mk1_to_canonical(&MK1).expect("mk1 → canonical");
        assert_eq!(cp.kind, SourceKind::Mk1Xpub);
        assert!(!cp.bytes.is_empty());
        // mk1 bytecode is byte-aligned.
        assert_eq!(cp.payload_bits, cp.bytes.len() * 8);
    }

    #[test]
    fn md1_canonical_carries_bit_precise_total_bits() {
        let cp = md1_to_canonical(&MD1).expect("md1 → canonical");
        assert_eq!(cp.kind, SourceKind::Md1Descriptor);
        assert!(!cp.bytes.is_empty());
        // md1 is bit-precise: payload_bits ≤ 8 * len, and (for this fixture) NOT
        // a multiple of 8 in general — assert the contract `payload_bits` is in
        // (8*(len-1), 8*len] (the final byte carries 1..8 real bits).
        assert!(cp.payload_bits <= cp.bytes.len() * 8);
        assert!(cp.payload_bits > (cp.bytes.len().saturating_sub(1)) * 8);
        // T3-c (test-hardening mutation-gap close, NOT a bugfix — see
        // `design/SPEC_test_hardening_T3_wire_goldens.md` T3-c): the two range
        // asserts above alone do NOT catch a regression from `total_bits` to
        // `bytes.len() * 8` at `md1_to_canonical`'s `payload_bits: total_bits`
        // (this fn's `:108`) — that regression is `<= cp.bytes.len() * 8`
        // (equality) and vacuously `> (len-1)*8`, so both survive undetected.
        // Pin the EXACT bit count as an independent-of-this-adapter oracle:
        // `md bytecode <MD1> --json` (descriptor-mnemonic `md-cli`,
        // `crates/md-cli/src/cmd/bytecode.rs:15,27`) reports `"payload_bits"`
        // by calling `encode_payload` directly (a separate CLI invocation path
        // from this adapter). Run once (2026-07-10) against the MD1 fixture
        // above: `md bytecode <MD1 chunks> --json` → `"payload_bits": 644`
        // (md-cli 0.11.3 binary / md-codec 0.41.0 source at
        // descriptor-mnemonic `b9662e5f`; cross-checked against this
        // workspace's own md-codec 0.40.0 dep, which computes the identical
        // 644 via `Descriptor::canonical_payload_bytes()` for this fixture).
        assert_eq!(cp.payload_bits, 644);
    }

    #[test]
    fn mk1_canonical_round_trips_to_same_xpub_not_literal_string() {
        // The load-bearing assertion (plan §7 P6): mk1 re-encode is NOT
        // string-deterministic (fresh chunk_set_id), so we assert on the
        // recovered KeyCard / xpub / canonical payload, NEVER the literal string.
        let cp = mk1_to_canonical(&MK1).expect("mk1 → canonical");
        let recovered =
            canonical_to_recovered(cp.kind, &cp.bytes, cp.payload_bits).expect("inverse");
        let RecoveredCard::Mk1 { card, xpub, mk1 } = recovered else {
            panic!("expected Mk1 recovery");
        };
        // The KeyCard's canonical payload round-trips byte-identically.
        assert_eq!(
            card.canonical_payload_bytes().expect("re-canonical"),
            cp.bytes
        );
        // The xpub is the stable identity.
        let orig = mk_codec::decode(&MK1).unwrap();
        assert_eq!(xpub, orig.xpub.to_string());
        // The re-emitted mk1 IS decode-able back to the same card.
        let mk1_refs: Vec<&str> = mk1.iter().map(String::as_str).collect();
        assert_eq!(mk_codec::decode(&mk1_refs).unwrap().xpub, orig.xpub);
    }

    #[test]
    fn md1_canonical_round_trips_descriptor_and_literal_string() {
        // md1 IS string-deterministic, so the literal re-emitted chunk set
        // equals a fresh deterministic `split` of the same descriptor.
        let cp = md1_to_canonical(&MD1).expect("md1 → canonical");
        let recovered =
            canonical_to_recovered(cp.kind, &cp.bytes, cp.payload_bits).expect("inverse");
        let RecoveredCard::Md1 { descriptor, md1 } = recovered else {
            panic!("expected Md1 recovery");
        };
        let orig = md_codec::reassemble(&MD1).unwrap();
        assert_eq!(descriptor, orig);
        // Literal string determinism: re-emit equals the canonical split; AND it
        // re-ingests to the same descriptor.
        assert_eq!(md1, md_codec::split(&orig).unwrap());
        let md1_refs: Vec<&str> = md1.iter().map(String::as_str).collect();
        assert_eq!(md_codec::reassemble(&md1_refs).unwrap(), orig);
    }

    #[test]
    fn chunks_to_canonical_routes_by_hrp() {
        // Multi-chunk cards: supply ALL chunks together.
        assert_eq!(chunks_to_canonical(&MK1).unwrap().kind, SourceKind::Mk1Xpub);
        assert_eq!(
            chunks_to_canonical(&MD1).unwrap().kind,
            SourceKind::Md1Descriptor
        );
    }

    #[test]
    fn string_to_canonical_splits_whitespace_chunk_list() {
        // A single --from value holding a whitespace-joined chunk list of one
        // multi-chunk card routes correctly.
        let joined_mk1 = MK1.join(" ");
        assert_eq!(
            string_to_canonical(&joined_mk1).unwrap().kind,
            SourceKind::Mk1Xpub
        );
        let joined_md1 = MD1.join(" ");
        assert_eq!(
            string_to_canonical(&joined_md1).unwrap().kind,
            SourceKind::Md1Descriptor
        );
    }

    #[test]
    fn string_to_canonical_refuses_unknown_hrp() {
        // ms1 (a SECRET) is intentionally NOT word-card-able.
        let err = string_to_canonical("ms10entrsqqqqqqqqq").unwrap_err();
        assert!(matches!(err, ToolkitError::UnknownHrp { .. }));
        // Exit-code routing: UnknownHrp is exit 2.
        assert_eq!(err.exit_code(), 2);
    }

    #[test]
    fn full_wc_round_trip_mk1_payload_equals_original() {
        // The end-to-end engine round-trip at the LIBRARY level: mk1 → canonical
        // → wc encode → wc decode → canonical → recovered. Assert on the
        // recovered PAYLOAD / xpub, not the literal mk1 string (plan §7 P6).
        let cp = mk1_to_canonical(&MK1).expect("mk1 → canonical");
        let opts = wc_codec::EncodeOpts {
            parity_words: 8,
            ..Default::default()
        };
        let words =
            wc_codec::encode(cp.kind, &cp.bytes, cp.payload_bits, &opts).expect("wc encode");
        let word_refs: Vec<&str> = words.to_vec();
        let decoded = wc_codec::decode(&word_refs).expect("wc decode");
        assert_eq!(decoded.kind, SourceKind::Mk1Xpub);
        assert_eq!(decoded.payload, cp.bytes);
        assert_eq!(decoded.payload_bits, cp.payload_bits);
        let recovered =
            canonical_to_recovered(decoded.kind, &decoded.payload, decoded.payload_bits)
                .expect("inverse");
        let RecoveredCard::Mk1 { xpub, .. } = recovered else {
            panic!("expected Mk1");
        };
        assert_eq!(xpub, mk_codec::decode(&MK1).unwrap().xpub.to_string());
    }

    #[test]
    fn full_wc_round_trip_md1_payload_and_descriptor() {
        let cp = md1_to_canonical(&MD1).expect("md1 → canonical");
        let opts = wc_codec::EncodeOpts {
            parity_words: 8,
            ..Default::default()
        };
        let words =
            wc_codec::encode(cp.kind, &cp.bytes, cp.payload_bits, &opts).expect("wc encode");
        let word_refs: Vec<&str> = words.to_vec();
        let decoded = wc_codec::decode(&word_refs).expect("wc decode");
        assert_eq!(decoded.kind, SourceKind::Md1Descriptor);
        assert_eq!(decoded.payload, cp.bytes);
        // The bit-precise total_bits round-trips.
        assert_eq!(decoded.payload_bits, cp.payload_bits);
        let recovered =
            canonical_to_recovered(decoded.kind, &decoded.payload, decoded.payload_bits)
                .expect("inverse");
        let RecoveredCard::Md1 { descriptor, md1 } = recovered else {
            panic!("expected Md1");
        };
        assert_eq!(descriptor, md_codec::reassemble(&MD1).unwrap());
        // md1 string determinism + re-ingest.
        assert_eq!(md1, md_codec::split(&descriptor).unwrap());
        let md1_refs: Vec<&str> = md1.iter().map(String::as_str).collect();
        assert_eq!(md_codec::reassemble(&md1_refs).unwrap(), descriptor);
    }
}
