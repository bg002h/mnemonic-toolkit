//! Five friendly mappers: bip39, bitcoin, ms_codec, mk_codec, md_codec.
//!
//! Realizes SPEC §6.4.0 routing principle + §6.4.1-§6.4.5 per-source
//! tables. All `#[non_exhaustive]` enums (bip39::Error,
//! bitcoin::bip32::Error, ms_codec::Error, mk_codec::Error) have a
//! wildcard `_` arm; md_codec::Error is closed and matched exhaustively.

use crate::error::BitcoinErrorKind;

pub fn friendly_bip39(e: &bip39::Error) -> String {
    match e {
        bip39::Error::BadEntropyBitCount(n) => format!(
            "BIP-39 entropy bit count {} invalid (must be 128, 160, 192, 224, or 256)",
            n,
        ),
        bip39::Error::BadWordCount(n) => format!(
            "BIP-39 word count {} invalid (must be 12, 15, 18, 21, or 24)",
            n,
        ),
        bip39::Error::UnknownWord(idx) => format!(
            "unknown BIP-39 word at position {} (not in selected wordlist; did you pick the right --language?)",
            idx,
        ),
        bip39::Error::InvalidChecksum => {
            "BIP-39 checksum failure (last word does not match the entropy)".to_string()
        }
        bip39::Error::AmbiguousLanguages(_) => {
            "BIP-39 phrase parses under multiple wordlists; specify --language explicitly"
                .to_string()
        }
    }
}

pub fn friendly_bitcoin(e: &BitcoinErrorKind) -> String {
    match e {
        BitcoinErrorKind::Bip32(b) => format!("BIP-32 error: {}", b),
        BitcoinErrorKind::XpubParse(s) => format!("--xpub parse error: {}", s),
        BitcoinErrorKind::FingerprintParse(s) => format!("--master-fingerprint parse error: {}", s),
    }
}

pub fn friendly_ms_codec(e: &ms_codec::Error) -> String {
    // Reuse ms-cli's mapping shape: most variants delegate to codex32_friendly,
    // structured variants get explicit messages. v0.1 toolkit is read-only on
    // ms-codec (it only encodes successfully or rejects with specific structural
    // errors during decode of toolkit-emitted strings — the encode path is
    // unreachable for variant errors since toolkit always supplies valid input).
    match e {
        ms_codec::Error::Codex32(c) => format!("ms1 codex32: {:?}", c),
        ms_codec::Error::WrongHrp { got } => {
            format!("ms1 wrong HRP: got {:?}, expected \"ms\"", got)
        }
        ms_codec::Error::ThresholdNotZero { got } => format!(
            "ms1 threshold not 0 (got '{}'); v0.1 single-string only",
            *got as char,
        ),
        ms_codec::Error::ShareIndexNotSecret { got } => {
            format!("ms1 share-index not 's' (got '{}')", got)
        }
        ms_codec::Error::TagInvalidAlphabet { got } => {
            format!("ms1 tag bytes not in codex32 alphabet: {:?}", got)
        }
        ms_codec::Error::UnknownTag { got } => format!(
            "ms1 unknown tag {:?}",
            std::str::from_utf8(got).unwrap_or("<non-utf8>"),
        ),
        ms_codec::Error::ReservedPrefixViolation { got } => {
            format!("ms1 reserved-prefix byte was 0x{:02x}, expected 0x00", got,)
        }
        ms_codec::Error::UnexpectedStringLength { got, .. } => format!(
            "ms1 string length {} not in v0.1 set [50, 56, 62, 69, 75]",
            got,
        ),
        ms_codec::Error::PayloadLengthMismatch { got, tag, .. } => format!(
            "ms1 tag {:?} payload length {} not in expected set [16, 20, 24, 28, 32]",
            std::str::from_utf8(tag).unwrap_or("<non-utf8>"),
            got,
        ),
        // v0.2 K-of-N share variants (SPEC_ms_v0_2_kofn §4 R0-m3). A consume
        // path (inspect/convert/decode) handed ONE share of a K-of-N set must
        // point the user at `mnemonic ms-shares combine`, NOT fall through to
        // the "unhandled" wildcard.
        ms_codec::Error::IsShareNotSingleString { threshold, index } => format!(
            "ms1 this is ONE of a K-of-N share set (threshold '{}', index '{}'); \
             use `mnemonic ms-shares combine` to recombine {} shares",
            threshold, index, threshold,
        ),
        ms_codec::Error::SecretShareSuppliedToCombine => {
            "ms1 the secret share (index 's') must not be combined; supply only the \
             distributed shares (the secret is the recovery target)"
                .to_string()
        }
        ms_codec::Error::InvalidThreshold(k) => format!(
            "ms-shares split: --threshold {} invalid; K-of-N shares require K in 2..=9",
            k,
        ),
        ms_codec::Error::InvalidShareCount { k, n } => format!(
            "ms-shares split: --shares {} invalid for --threshold {}; require K <= N <= 31",
            n, k,
        ),
        // ReservedTagNotEmittedInV01 routes via From in error.rs to FutureFormat; never reached here.
        _ => format!("unhandled ms_codec::Error variant: {:?}", e),
    }
}

pub fn friendly_mk_codec(e: &mk_codec::Error) -> String {
    use mk_codec::Error as E;
    match e {
        E::InvalidHrp(s) => format!("mk1 wrong HRP: got {:?}, expected \"mk\"", s),
        E::MixedCase => "mk1 mixed case in input string".to_string(),
        E::InvalidStringLength(n) => format!(
            "mk1 data-part length {} not valid (regular code: 14-93; long code: 95-108; the gap at 94 is reserved-invalid)",
            n,
        ),
        E::InvalidChar { ch, position } => format!(
            "mk1 invalid character '{}' at position {} (not in bech32 alphabet)",
            ch, position,
        ),
        E::BchUncorrectable(s) => format!(
            "mk1 BCH uncorrectable: {} (engraving error or transcription typo)",
            s,
        ),
        E::UnsupportedCardType(b) => format!("mk1 unsupported card type: 0x{:02x}", b),
        E::MalformedPayloadPadding => "mk1 malformed payload padding".to_string(),
        E::ChunkSetIdMismatch => "mk1 chunk_set_id mismatch across chunks".to_string(),
        E::ChunkedHeaderMalformed(s) => format!("mk1 chunked-header malformed: {}", s),
        E::MixedHeaderTypes => "mk1 mixed string-layer header types".to_string(),
        E::CrossChunkHashMismatch => "mk1 cross-chunk integrity hash mismatch".to_string(),
        E::ReservedBitsSet => "mk1 reserved bits set in bytecode header".to_string(),
        E::InvalidPolicyIdStubCount => "mk1 policy_id_stub_count must be ≥ 1".to_string(),
        E::InvalidPathIndicator(b) => format!("mk1 invalid path indicator byte: 0x{:02x}", b),
        E::PathTooDeep(n) => format!("mk1 path too deep: {} components (max 10)", n),
        E::InvalidPathComponent(s) => format!("mk1 invalid path component: {}", s),
        E::InvalidXpubVersion(v) => format!("mk1 invalid xpub version: 0x{:08x}", v),
        E::InvalidXpubPublicKey(s) => format!("mk1 invalid xpub public key: {}", s),
        E::UnexpectedEnd => "mk1 unexpected end of bytecode".to_string(),
        E::TrailingBytes => "mk1 trailing bytes after xpub".to_string(),
        E::CardPayloadTooLarge {
            bytecode_len,
            max_supported,
        } => format!(
            "mk1 card payload too large: bytecode_len {} > max_supported {}",
            bytecode_len, max_supported,
        ),
        E::XpubOriginPathMismatch {
            xpub_depth,
            path_depth,
            ..
        } => format!(
            "mk1 xpub/origin-path depth mismatch: xpub depth {} vs origin_path depth {} (toolkit bug — the mk1 card's path must round-trip its xpub)",
            xpub_depth, path_depth,
        ),
        // UnsupportedVersion routes via From → FutureFormat; never reached here.
        _ => format!("unhandled mk_codec::Error variant: {:?}", e),
    }
}

pub fn friendly_md_codec(e: &md_codec::Error) -> String {
    // md_codec::Error is NOT #[non_exhaustive]; exhaustive match required.
    use md_codec::Error as E;
    match e {
        E::BitStreamTruncated {
            requested,
            available,
        } => format!(
            "md1 bitstream truncated: requested {} bits, {} available",
            requested, available,
        ),
        // v0.30 wire-format break removed ReservedHeaderBitSet; replaced
        // by the more general MalformedHeader { detail }.
        E::MalformedHeader { detail } => format!("md1 malformed header: {}", detail),
        // v0.30 renamed UnsupportedVersion -> WireVersionMismatch.
        // Routes via From → FutureFormat (exit 3); arm retained for
        // exhaustiveness with a defensive message in case the routing
        // is ever bypassed (e.g., direct construction in a test).
        E::WireVersionMismatch { got } => {
            format!("md1 wire-version mismatch: got {} (route via FutureFormat)", got)
        }
        E::PathDepthExceeded { got, max } => {
            format!("md1 path depth {} exceeds max {}", got, max)
        }
        E::KeyCountOutOfRange { n } => format!("md1 key count {} out of range (1..=32)", n),
        E::DivergentPathCountMismatch { n, got } => format!(
            "md1 divergent path count {} does not match key count {}",
            got, n,
        ),
        E::AltCountOutOfRange { got } => {
            format!("md1 multipath alt-count {} out of range (2..=9)", got)
        }
        // v0.30 collapsed UnknownPrimaryTag(u8) + UnknownExtensionTag(u8)
        // into a single TagOutOfRange { primary } variant.
        E::TagOutOfRange { primary } => {
            format!("md1 tag value 0x{:02x} out of range", primary)
        }
        E::ThresholdOutOfRange { k } => {
            format!("md1 threshold k={} out of range (1..=32)", k)
        }
        E::ChildCountOutOfRange { count } => {
            format!("md1 child count {} out of range (1..=32)", count)
        }
        E::KGreaterThanN { k, n } => {
            format!("md1 threshold k={} exceeds child count n={}", k, n)
        }
        E::TlvOrderingViolation { prev, current } => {
            format!("md1 TLV ordering: 0x{:02x} after 0x{:02x}", current, prev)
        }
        E::PlaceholderIndexOutOfRange { idx, n } => {
            format!("md1 placeholder index {} out of range (n={})", idx, n)
        }
        E::OverrideOrderViolation { prev, current } => {
            format!("md1 override ordering: @{} after @{}", current, prev)
        }
        E::EmptyTlvEntry { tag } => format!("md1 empty TLV entry tag 0x{:02x}", tag),
        E::TlvLengthExceedsRemaining { length, remaining } => {
            format!("md1 TLV length {} exceeds remaining {}", length, remaining)
        }
        E::PlaceholderNotReferenced { idx, n } => {
            format!("md1 placeholder @{} not referenced (n={})", idx, n)
        }
        E::PlaceholderFirstOccurrenceOutOfOrder {
            expected_first,
            got_first,
        } => format!(
            "md1 placeholder first-occurrence: expected @{}, got @{}",
            expected_first, got_first,
        ),
        E::MultipathAltCountMismatch { expected, got } => format!(
            "md1 multipath alt-count mismatch: expected {}, got {}",
            expected, got,
        ),
        E::ForbiddenTapTreeLeaf { tag } => {
            format!("md1 forbidden tap-script-tree leaf tag 0x{:02x}", tag)
        }
        E::ChunkCountOutOfRange { count } => {
            format!("md1 chunk count {} out of range (1..=64)", count)
        }
        E::ChunkIndexOutOfRange { index, count } => {
            format!("md1 chunk index {} ≥ count {}", index, count)
        }
        E::ChunkSetIdOutOfRange { id } => {
            format!("md1 chunk-set-id 0x{:x} exceeds 20-bit range", id)
        }
        E::ChunkHeaderChunkedFlagMissing => "md1 chunk header chunked-flag missing".to_string(),
        E::ChunkCountExceedsMax { needed } => {
            format!("md1 chunk count {} exceeds max 64", needed)
        }
        E::Codex32DecodeError(s) => format!("md1 codex32 decode: {}", s),
        E::Codex32EncodeError(s) => format!("md1 codex32 encode: {}", s),
        E::ChunkSetEmpty => "md1 chunk set empty".to_string(),
        E::ChunkSetInconsistent => "md1 chunks disagree on version/chunk-set-id/count".to_string(),
        E::ChunkSetIncomplete { got, expected } => {
            format!(
                "md1 chunk set incomplete: got {}, expected {}",
                got, expected
            )
        }
        E::ChunkIndexGap { expected, got } => {
            format!("md1 chunk index gap: expected {}, got {}", expected, got)
        }
        E::ChunkSetIdMismatch { expected, derived } => format!(
            "md1 chunk-set-id mismatch: expected 0x{:x}, derived 0x{:x}",
            expected, derived,
        ),
        E::VarintOverflow { value } => format!("md1 varint overflow: {}", value),
        E::MissingExplicitOrigin { idx } => format!("md1 missing explicit origin for @{}", idx),
        E::InvalidPresenceByte { reserved_bits } => format!(
            "md1 presence byte non-zero reserved bits 0x{:02x}",
            reserved_bits,
        ),
        E::InvalidXpubBytes { idx } => format!("md1 invalid xpub bytes for @{}", idx),
        E::MissingPubkey { idx } => format!(
            "md1 missing pubkey for @{} (wallet-policy mode requires all @N)",
            idx,
        ),
        E::ChainIndexOutOfRange { chain, alt_count } => format!(
            "md1 chain index {} out of range (alt_count={})",
            chain, alt_count,
        ),
        E::HardenedPublicDerivation => "md1 hardened public-key derivation forbidden".to_string(),
        // v0.32 replaced UnsupportedDerivationShape with the more general
        // AddressDerivationFailed { detail: String }.
        E::AddressDerivationFailed { detail } => {
            format!("md1 address derivation failed: {}", detail)
        }
        // v0.30 NUMS sentinel rule on Body::Tr — see SPEC §7 + §11.
        E::NUMSSentinelConflict => {
            "md1 NUMS sentinel conflict: is_nums=false with key_index out of range".to_string()
        }
        // v0.31 operator-context enforcement (e.g., a Multi-family tag
        // appearing as a top-level operator instead of inside a wrapper).
        E::OperatorContextViolation { tag, context } => {
            format!(
                "md1 operator {:?} not allowed in context {:?}",
                tag, context
            )
        }
        // v0.19 decode-side recursion-depth hardening.
        E::DecodeRecursionDepthExceeded { depth, max } => format!(
            "md1 decode recursion depth {} exceeds maximum {}",
            depth, max
        ),
        // v0.34.0 BCH-error-correction (Phase B.2): chunk uncorrectable.
        // Typically intercepted by the toolkit's repair-helper translation
        // table (Phase B.7) BEFORE this arm fires; retained for
        // exhaustiveness in case a direct codec call bypasses the helper.
        E::TooManyErrors { chunk_index, bound } => format!(
            "md1 chunk {} uncorrectable (exceeds singleton bound = {})",
            chunk_index, bound,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bip39_unknown_word_mentions_language() {
        let m = friendly_bip39(&bip39::Error::UnknownWord(5));
        assert!(m.contains("--language"));
    }

    #[test]
    fn ms_codec_wrong_hrp() {
        let m = friendly_ms_codec(&ms_codec::Error::WrongHrp { got: "mq".into() });
        assert!(m.contains("ms1"));
        assert!(m.contains("\"ms\""));
    }

    #[test]
    fn mk_codec_path_too_deep() {
        let m = friendly_mk_codec(&mk_codec::Error::PathTooDeep(11));
        assert!(m.contains("11"));
        assert!(m.contains("max 10"));
    }

    #[test]
    fn ms_codec_share_points_at_ms_shares_combine() {
        let m = friendly_ms_codec(&ms_codec::Error::IsShareNotSingleString {
            threshold: '2',
            index: 'a',
        });
        assert!(m.contains("ms-shares combine"), "got: {m}");
        assert!(!m.contains("unhandled"), "got: {m}");
    }

    #[test]
    fn ms_codec_secret_share_to_combine_is_explicit() {
        let m = friendly_ms_codec(&ms_codec::Error::SecretShareSuppliedToCombine);
        assert!(m.contains("secret share"), "got: {m}");
        assert!(!m.contains("unhandled"), "got: {m}");
    }

    #[test]
    fn ms_codec_invalid_threshold_is_explicit() {
        let m = friendly_ms_codec(&ms_codec::Error::InvalidThreshold(1));
        assert!(m.contains("2..=9"), "got: {m}");
        assert!(!m.contains("unhandled"), "got: {m}");
    }

    #[test]
    fn ms_codec_invalid_share_count_is_explicit() {
        let m = friendly_ms_codec(&ms_codec::Error::InvalidShareCount { k: 3, n: 2 });
        assert!(m.contains("N <= 31") || m.contains("K <= N"), "got: {m}");
        assert!(!m.contains("unhandled"), "got: {m}");
    }

    #[test]
    fn mk_codec_xpub_origin_path_mismatch() {
        use bitcoin::bip32::ChildNumber;
        let m = friendly_mk_codec(&mk_codec::Error::XpubOriginPathMismatch {
            xpub_depth: 3,
            path_depth: 4,
            xpub_child: ChildNumber::Hardened { index: 0 },
            path_child: None,
        });
        assert!(m.contains("depth mismatch"), "got: {m}");
        assert!(!m.contains("unhandled"), "got: {m}");
    }
}
