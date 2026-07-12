//! Five friendly mappers: bip39, bitcoin, ms_codec, mk_codec, md_codec.
//!
//! Realizes SPEC §6.4.0 routing principle + §6.4.1-§6.4.5 per-source
//! tables. Only the `ms_codec::Error` and `mk_codec::Error` mappers carry a
//! wildcard `_` arm (both wrap `#[non_exhaustive]` enums). The other three are
//! exhaustive: `md_codec::Error` and `bip39::Error` are matched arm-by-arm
//! (both closed enums), and `friendly_bitcoin` matches the toolkit-local closed
//! `BitcoinErrorKind` — the `#[non_exhaustive]` `bitcoin::bip32::Error` is only
//! Display-forwarded inside the `Bip32(b)` arm, so that mapper has no wildcard.

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
        // The bread-and-butter `combine` share errors (I2, P3-R0): a too-small
        // share set, a duplicate share index, or a heterogeneous share set.
        // Render prose mirroring ms-cli's `codex32_friendly.rs`, NOT the Debug
        // dump (`ThresholdNotPassed { .. }`, `RepeatedIndex(Fe(0))`). The
        // generic `Codex32(_)` arm below stays as the fallback for the
        // non-share codex32 errors (parse/checksum/length/case/etc).
        ms_codec::Error::Codex32(ms_codec::codex32::Error::ThresholdNotPassed {
            threshold,
            n_shares,
        }) => format!(
            "ms1 not enough shares: have {}, need {}",
            n_shares, threshold
        ),
        ms_codec::Error::Codex32(ms_codec::codex32::Error::RepeatedIndex(fe)) => format!(
            "ms1 share index '{}' repeated (each share in a set must have a distinct index)",
            fe.to_char(),
        ),
        ms_codec::Error::Codex32(ms_codec::codex32::Error::MismatchedLength(a, b)) => format!(
            "ms1 share length mismatch: {} vs {} (all shares of one secret must share length)",
            a, b,
        ),
        ms_codec::Error::Codex32(ms_codec::codex32::Error::MismatchedHrp(a, b)) => {
            format!("ms1 HRP mismatch among shares: {:?} vs {:?}", a, b)
        }
        ms_codec::Error::Codex32(ms_codec::codex32::Error::MismatchedThreshold(a, b)) => {
            format!("ms1 threshold mismatch among shares: {} vs {}", a, b)
        }
        ms_codec::Error::Codex32(ms_codec::codex32::Error::MismatchedId(a, b)) => {
            format!("ms1 id mismatch among shares: {:?} vs {:?}", a, b)
        }
        // Leak-hardening (v0.53.4): InvalidChecksum embeds the FULL input
        // `string` — Debug-printing it via the catch-all below would echo the
        // near-secret on stderr. Withhold it FULLY (not a head-truncation like
        // UnknownHrp's: ms1 chars 9+ are payload, so any head-echo leaks
        // payload); the checksum kind + length stay (lets the user spot a
        // wrong-length card). FOLLOWUP `friendly-ms1-invalidchecksum-echoes-full-input`.
        ms_codec::Error::Codex32(ms_codec::codex32::Error::InvalidChecksum {
            checksum,
            string,
        }) => {
            format!(
                "ms1 codex32: invalid {checksum} checksum ({} chars; input withheld)",
                string.chars().count()
            )
        }
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
        // cycle-4 M6 (ms-codec 0.5.0): the supplied shares share the same
        // hrp/id/threshold/length header but are NOT all from one split — they
        // do not lie on a single Shamir polynomial. Combining them would have
        // silently recovered a WRONG secret pre-0.5.0; ms-codec now rejects.
        ms_codec::Error::InconsistentShareSet => {
            "ms1 inconsistent share set: one or more shares are not from the same split \
             (they share an id but do not lie on one polynomial). Combining them would \
             recover the WRONG secret — supply only shares from a single split"
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
            format!(
                "md1 wire-version mismatch: got {} (route via FutureFormat)",
                got
            )
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
        // md-codec 0.37.0 D5(a) decode canonical-form rejects.
        E::BaselineUseSiteOverride { idx } => format!(
            "md1 use-site override keyed on baseline @{} (the @0 baseline cannot be overridden)",
            idx,
        ),
        E::RedundantUseSiteOverride { idx } => format!(
            "md1 redundant use-site override for @{} (equals the baseline use-site path)",
            idx,
        ),
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
        // cycle-4 (md-codec 0.38.0) codex32 regular-code length caps.
        // H6 encode-side: a single md1 string is capped at 80 data symbols
        // (93-symbol codeword); the remedy is chunked encoding.
        E::PayloadTooLongForSingleString { data_symbols, max } => format!(
            "md1 payload is {} data symbols; a single md1 string caps at {} \
             (use chunked encoding / --force-chunked)",
            data_symbols, max,
        ),
        // M4 correcting-decode: an over-93-symbol chunk handed to the BCH
        // corrector is out of the regular code's domain (degree aliasing) and
        // is rejected before correction.
        E::ChunkSymbolCountOutOfRange {
            chunk_index,
            symbols,
            max,
        } => format!(
            "md1 chunk {} has {} symbols; the codex32 regular code caps a string at {}",
            chunk_index, symbols, max,
        ),
        // I1 non-correcting decode: an over-93-symbol single string is
        // structurally out-of-domain even when its checksum verifies.
        E::StringSymbolCountOutOfRange { symbols, max } => format!(
            "md1 string has {} symbols; the codex32 regular code caps a string at {}",
            symbols, max,
        ),
        // md-codec 0.41.0 F-A8: the shared TLV parser now rejects a non-zero
        // trailing-pad tail (≤7 bits) — the reference encoder always zero-pads
        // to the symbol boundary, so a non-zero pad is non-canonical wire.
        E::MalformedPayloadPadding { bits } => format!(
            "md1 non-zero trailing padding: the final {} pad bit(s) must be zero",
            bits,
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

    // ── v0.53.4: InvalidChecksum withholds the embedded full input ──────────

    #[test]
    fn ms_codec_invalid_checksum_withholds_input_string() {
        let secret = "ms10entrspqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
        let m = friendly_ms_codec(&ms_codec::Error::Codex32(
            ms_codec::codex32::Error::InvalidChecksum {
                checksum: "short",
                string: secret.to_string(),
            },
        ));
        // The redaction pin: the embedded input must NOT appear in the message.
        assert!(
            !m.contains(secret),
            "InvalidChecksum input string leaked: {m}"
        );
        assert!(
            !m.contains(&secret[9..30]),
            "a payload slice of the input leaked: {m}"
        );
        assert!(!m.contains("InvalidChecksum"), "variant name leaked: {m}");
        assert!(
            m.contains("invalid short checksum") && m.contains("withheld"),
            "expected the checksum-kind + withheld prose: {m}"
        );
        // length stays (actionable, non-secret).
        assert!(
            m.contains(&format!("{} chars", secret.chars().count())),
            "expected the char count: {m}"
        );
    }

    // ── I2 (P3-R0): codex32 SHARE errors render prose, not a Debug dump ──────

    #[test]
    fn ms_codec_threshold_not_passed_renders_prose() {
        let m = friendly_ms_codec(&ms_codec::Error::Codex32(
            ms_codec::codex32::Error::ThresholdNotPassed {
                threshold: 2,
                n_shares: 1,
            },
        ));
        assert!(m.contains("not enough shares"), "got: {m}");
        assert!(m.contains("have 1") && m.contains("need 2"), "got: {m}");
        // No Debug dump: no struct braces, no variant name.
        assert!(!m.contains('{'), "Debug-dumped braces leaked: {m}");
        assert!(
            !m.contains("ThresholdNotPassed"),
            "variant name leaked: {m}"
        );
    }

    #[test]
    fn ms_codec_repeated_index_renders_prose() {
        let m = friendly_ms_codec(&ms_codec::Error::Codex32(
            ms_codec::codex32::Error::RepeatedIndex(ms_codec::codex32::Fe::Q),
        ));
        assert!(m.contains("repeated"), "got: {m}");
        // No `RepeatedIndex(Fe(0))` opaque dump.
        assert!(!m.contains("Fe("), "Fe(..) leaked: {m}");
        assert!(!m.contains("RepeatedIndex"), "variant name leaked: {m}");
    }

    #[test]
    fn ms_codec_mismatched_set_errors_render_prose() {
        for (e, needle) in [
            (
                ms_codec::Error::Codex32(ms_codec::codex32::Error::MismatchedLength(50, 56)),
                "length mismatch",
            ),
            (
                ms_codec::Error::Codex32(ms_codec::codex32::Error::MismatchedThreshold(2, 3)),
                "threshold mismatch",
            ),
            (
                ms_codec::Error::Codex32(ms_codec::codex32::Error::MismatchedId(
                    "abcd".into(),
                    "efgh".into(),
                )),
                "id mismatch",
            ),
            (
                ms_codec::Error::Codex32(ms_codec::codex32::Error::MismatchedHrp(
                    "ms".into(),
                    "mk".into(),
                )),
                "HRP mismatch",
            ),
        ] {
            let m = friendly_ms_codec(&e);
            assert!(m.contains(needle), "expected {needle:?} in: {m}");
            assert!(!m.contains("Mismatched"), "variant name leaked: {m}");
        }
    }

    #[test]
    fn ms_codec_non_share_codex32_falls_through_to_generic() {
        // A non-share codex32 error (e.g. a bad char) still hits the generic
        // fallback arm — the new share arms must not swallow it.
        let m = friendly_ms_codec(&ms_codec::Error::Codex32(
            ms_codec::codex32::Error::InvalidChar('!'),
        ));
        assert!(m.starts_with("ms1 codex32:"), "got: {m}");
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

    // ── Table-driven per-mapper coverage (`friendly-mapper-unit-test-gaps`) ──
    //
    // Each table row is `(error, needle, variant_name)`. For every row we assert
    // the rendered string (a) carries the codec tag + the distinctive `needle`,
    // and (b) does NOT leak the raw Debug variant name (`variant_name`) — the
    // friendly message must be prose, not a `{:?}` dump. The variant name is
    // pinned per-row (not via a generic PascalCase scan) on purpose: some
    // friendly messages legitimately embed a CamelCase token — e.g.
    // `WireVersionMismatch` renders "(route via FutureFormat)" — which a generic
    // scan would false-trip on.
    //
    // The `!contains("unhandled")` guard is load-bearing ONLY for the two
    // wildcard mappers — `friendly_ms_codec` (`:`-arm) and `friendly_mk_codec`
    // — where a future `#[non_exhaustive]` variant silently falls to the bare
    // `_ => "unhandled … {:?}"` arm; that fallthrough is the real regression
    // these tests catch (and the wildcard itself stays untested-by-construction,
    // since it can only fire on a not-yet-existing variant). For the three
    // closed mappers (`friendly_md_codec`, `friendly_bip39`, `friendly_bitcoin`)
    // there is no `_` arm: a new variant breaks compilation, not a test, so the
    // `!contains("unhandled")` check is vacuous there and the substantive
    // assertions are the needle + no-Debug-leak (message-quality) checks.

    #[test]
    fn md_codec_all_arms_render_prose() {
        use md_codec::error::ContextKind;
        use md_codec::Error as E;
        use md_codec::Tag;
        let rows: [(E, &str, &str); 44] = [
            (
                E::BitStreamTruncated {
                    requested: 8,
                    available: 3,
                },
                "bitstream truncated",
                "BitStreamTruncated",
            ),
            (
                E::MalformedHeader {
                    detail: "bad".into(),
                },
                "malformed header",
                "MalformedHeader",
            ),
            (
                E::WireVersionMismatch { got: 9 },
                "wire-version mismatch",
                "WireVersionMismatch",
            ),
            (
                E::PathDepthExceeded { got: 20, max: 15 },
                "path depth",
                "PathDepthExceeded",
            ),
            (
                E::KeyCountOutOfRange { n: 40 },
                "key count",
                "KeyCountOutOfRange",
            ),
            (
                E::DivergentPathCountMismatch { n: 2, got: 3 },
                "divergent path count",
                "DivergentPathCountMismatch",
            ),
            (
                E::AltCountOutOfRange { got: 10 },
                "alt-count",
                "AltCountOutOfRange",
            ),
            (
                E::TagOutOfRange { primary: 0x3f },
                "tag value",
                "TagOutOfRange",
            ),
            (
                E::ThresholdOutOfRange { k: 40 },
                "threshold",
                "ThresholdOutOfRange",
            ),
            (
                E::ChildCountOutOfRange { count: 40 },
                "child count",
                "ChildCountOutOfRange",
            ),
            (
                E::KGreaterThanN { k: 3, n: 2 },
                "exceeds child count",
                "KGreaterThanN",
            ),
            (
                E::TlvOrderingViolation {
                    prev: 0x10,
                    current: 0x05,
                },
                "TLV ordering",
                "TlvOrderingViolation",
            ),
            (
                E::PlaceholderIndexOutOfRange { idx: 5, n: 3 },
                "placeholder index",
                "PlaceholderIndexOutOfRange",
            ),
            (
                E::OverrideOrderViolation {
                    prev: 2,
                    current: 1,
                },
                "override ordering",
                "OverrideOrderViolation",
            ),
            (
                E::EmptyTlvEntry { tag: 0x07 },
                "empty TLV entry",
                "EmptyTlvEntry",
            ),
            (
                E::TlvLengthExceedsRemaining {
                    length: 100,
                    remaining: 8,
                },
                "TLV length",
                "TlvLengthExceedsRemaining",
            ),
            (
                E::PlaceholderNotReferenced { idx: 1, n: 3 },
                "not referenced",
                "PlaceholderNotReferenced",
            ),
            (
                E::PlaceholderFirstOccurrenceOutOfOrder {
                    expected_first: 0,
                    got_first: 1,
                },
                "first-occurrence",
                "PlaceholderFirstOccurrenceOutOfOrder",
            ),
            (
                E::MultipathAltCountMismatch {
                    expected: 2,
                    got: 3,
                },
                "multipath alt-count mismatch",
                "MultipathAltCountMismatch",
            ),
            (
                E::ForbiddenTapTreeLeaf { tag: 0x09 },
                "forbidden tap-script-tree leaf",
                "ForbiddenTapTreeLeaf",
            ),
            (
                E::ChunkCountOutOfRange { count: 200 },
                "chunk count",
                "ChunkCountOutOfRange",
            ),
            (
                E::ChunkIndexOutOfRange { index: 5, count: 3 },
                "chunk index",
                "ChunkIndexOutOfRange",
            ),
            (
                E::ChunkSetIdOutOfRange { id: 0x1fffff },
                "chunk-set-id",
                "ChunkSetIdOutOfRange",
            ),
            (
                E::ChunkHeaderChunkedFlagMissing,
                "chunked-flag missing",
                "ChunkHeaderChunkedFlagMissing",
            ),
            (
                E::ChunkCountExceedsMax { needed: 70 },
                "exceeds max 64",
                "ChunkCountExceedsMax",
            ),
            (
                E::Codex32DecodeError("oops".into()),
                "codex32 decode",
                "Codex32DecodeError",
            ),
            (
                E::Codex32EncodeError("oops".into()),
                "codex32 encode",
                "Codex32EncodeError",
            ),
            (E::ChunkSetEmpty, "chunk set empty", "ChunkSetEmpty"),
            (E::ChunkSetInconsistent, "disagree", "ChunkSetInconsistent"),
            (
                E::ChunkSetIncomplete {
                    got: 1,
                    expected: 2,
                },
                "chunk set incomplete",
                "ChunkSetIncomplete",
            ),
            (
                E::ChunkIndexGap {
                    expected: 1,
                    got: 2,
                },
                "chunk index gap",
                "ChunkIndexGap",
            ),
            (
                E::ChunkSetIdMismatch {
                    expected: 0x10,
                    derived: 0x20,
                },
                "chunk-set-id mismatch",
                "ChunkSetIdMismatch",
            ),
            (
                E::VarintOverflow { value: 999 },
                "varint overflow",
                "VarintOverflow",
            ),
            (
                E::MissingExplicitOrigin { idx: 2 },
                "missing explicit origin",
                "MissingExplicitOrigin",
            ),
            (
                E::InvalidPresenceByte {
                    reserved_bits: 0x04,
                },
                "presence byte",
                "InvalidPresenceByte",
            ),
            (
                E::InvalidXpubBytes { idx: 1 },
                "invalid xpub bytes",
                "InvalidXpubBytes",
            ),
            (
                E::MissingPubkey { idx: 1 },
                "missing pubkey",
                "MissingPubkey",
            ),
            (
                E::ChainIndexOutOfRange {
                    chain: 5,
                    alt_count: 2,
                },
                "chain index",
                "ChainIndexOutOfRange",
            ),
            (
                E::HardenedPublicDerivation,
                "hardened public-key derivation",
                "HardenedPublicDerivation",
            ),
            (
                E::AddressDerivationFailed {
                    detail: "boom".into(),
                },
                "address derivation failed",
                "AddressDerivationFailed",
            ),
            (
                E::NUMSSentinelConflict,
                "NUMS sentinel conflict",
                "NUMSSentinelConflict",
            ),
            (
                E::OperatorContextViolation {
                    tag: Tag::Multi,
                    context: ContextKind::MultiBody,
                },
                "not allowed in context",
                "OperatorContextViolation",
            ),
            (
                E::DecodeRecursionDepthExceeded { depth: 9, max: 8 },
                "recursion depth",
                "DecodeRecursionDepthExceeded",
            ),
            (
                E::TooManyErrors {
                    chunk_index: 0,
                    bound: 8,
                },
                "uncorrectable",
                "TooManyErrors",
            ),
        ];
        for (e, needle, variant) in rows {
            let m = friendly_md_codec(&e);
            assert!(m.contains("md1"), "missing md1 tag for {variant}: {m}");
            assert!(m.contains(needle), "expected {needle:?} for {variant}: {m}");
            assert!(!m.contains(variant), "variant name {variant} leaked: {m}");
            // No `!contains("unhandled")` here: this is a CLOSED mapper (no `_`
            // arm), so that guard is vacuous and is deliberately confined to the
            // two wildcard mappers (SPEC §Item-2 M5 — do not cargo-cult it).
        }
    }

    #[test]
    fn mk_codec_remaining_arms_render_prose() {
        use mk_codec::Error as E;
        // PathTooDeep + XpubOriginPathMismatch are covered above; the rest here.
        let rows: [(E, &str, &str); 19] = [
            (E::InvalidHrp("xx".into()), "wrong HRP", "InvalidHrp"),
            (E::MixedCase, "mixed case", "MixedCase"),
            (
                E::InvalidStringLength(94),
                "data-part length",
                "InvalidStringLength",
            ),
            (
                E::InvalidChar {
                    ch: '!',
                    position: 3,
                },
                "invalid character",
                "InvalidChar",
            ),
            (
                E::BchUncorrectable("3 errors".into()),
                "BCH uncorrectable",
                "BchUncorrectable",
            ),
            (
                E::UnsupportedCardType(0x05),
                "unsupported card type",
                "UnsupportedCardType",
            ),
            (
                E::MalformedPayloadPadding,
                "malformed payload padding",
                "MalformedPayloadPadding",
            ),
            (
                E::ChunkSetIdMismatch,
                "chunk_set_id mismatch",
                "ChunkSetIdMismatch",
            ),
            (
                E::ChunkedHeaderMalformed("bad".into()),
                "chunked-header malformed",
                "ChunkedHeaderMalformed",
            ),
            (
                E::MixedHeaderTypes,
                "mixed string-layer header types",
                "MixedHeaderTypes",
            ),
            (
                E::CrossChunkHashMismatch,
                "cross-chunk integrity hash mismatch",
                "CrossChunkHashMismatch",
            ),
            (E::ReservedBitsSet, "reserved bits set", "ReservedBitsSet"),
            (
                E::InvalidPolicyIdStubCount,
                "policy_id_stub_count",
                "InvalidPolicyIdStubCount",
            ),
            (
                E::InvalidPathIndicator(0x16),
                "invalid path indicator",
                "InvalidPathIndicator",
            ),
            (
                E::InvalidPathComponent("bad".into()),
                "invalid path component",
                "InvalidPathComponent",
            ),
            (
                E::InvalidXpubVersion(0xdead_beef),
                "invalid xpub version",
                "InvalidXpubVersion",
            ),
            (
                E::InvalidXpubPublicKey("bad point".into()),
                "invalid xpub public key",
                "InvalidXpubPublicKey",
            ),
            (E::UnexpectedEnd, "unexpected end", "UnexpectedEnd"),
            (E::TrailingBytes, "trailing bytes", "TrailingBytes"),
        ];
        for (e, needle, variant) in rows {
            let m = friendly_mk_codec(&e);
            assert!(m.contains("mk1"), "missing mk1 tag for {variant}: {m}");
            assert!(m.contains(needle), "expected {needle:?} for {variant}: {m}");
            assert!(!m.contains(variant), "variant name {variant} leaked: {m}");
            // Load-bearing here: friendly_mk_codec has a bare `_` wildcard.
            assert!(!m.contains("unhandled"), "for {variant}: {m}");
        }
        // CardPayloadTooLarge: struct fields, tested separately so the table
        // row tuple stays simple.
        let m = friendly_mk_codec(&E::CardPayloadTooLarge {
            bytecode_len: 2000,
            max_supported: 1692,
        });
        assert!(m.contains("mk1"), "got: {m}");
        assert!(m.contains("card payload too large"), "got: {m}");
        assert!(
            !m.contains("CardPayloadTooLarge"),
            "variant name leaked: {m}"
        );
        assert!(!m.contains("unhandled"), "got: {m}");
    }

    #[test]
    fn ms_codec_structural_arms_render_prose() {
        use ms_codec::Error as E;
        // The Codex32 share arms + WrongHrp + IsShareNotSingleString +
        // SecretShareSuppliedToCombine + InvalidThreshold/ShareCount are covered
        // above; these are the remaining structural decode arms.
        let rows: [(E, &str, &str); 7] = [
            (
                E::ThresholdNotZero { got: b'2' },
                "threshold not 0",
                "ThresholdNotZero",
            ),
            (
                E::ShareIndexNotSecret { got: 'a' },
                "share-index not 's'",
                "ShareIndexNotSecret",
            ),
            (
                E::TagInvalidAlphabet {
                    got: [b'!', b'!', b'!', b'!'],
                },
                "codex32 alphabet",
                "TagInvalidAlphabet",
            ),
            (
                E::UnknownTag {
                    got: [b'x', b'y', b'z', b'w'],
                },
                "unknown tag",
                "UnknownTag",
            ),
            (
                E::ReservedPrefixViolation { got: 0x01 },
                "reserved-prefix byte",
                "ReservedPrefixViolation",
            ),
            (
                E::UnexpectedStringLength {
                    got: 51,
                    allowed: &[50, 56, 62, 69, 75],
                },
                "string length",
                "UnexpectedStringLength",
            ),
            (
                E::PayloadLengthMismatch {
                    tag: [b'm', b's', b'e', b'c'],
                    expected: &[16, 20, 24, 28, 32],
                    got: 17,
                },
                "payload length",
                "PayloadLengthMismatch",
            ),
        ];
        for (e, needle, variant) in rows {
            let m = friendly_ms_codec(&e);
            assert!(m.contains("ms1"), "missing ms1 tag for {variant}: {m}");
            assert!(m.contains(needle), "expected {needle:?} for {variant}: {m}");
            assert!(!m.contains(variant), "variant name {variant} leaked: {m}");
            // Load-bearing here: friendly_ms_codec has a bare `_` wildcard.
            assert!(!m.contains("unhandled"), "for {variant}: {m}");
        }
    }

    #[test]
    fn bip39_constructible_arms_render_prose() {
        // AmbiguousLanguages is intentionally excluded: its payload
        // `[bool; MAX_NB_LANGUAGES]` is a tuple-struct field with no public
        // constructor, so it is not buildable from the test crate.
        let rows: [(bip39::Error, &str, &str); 3] = [
            (
                bip39::Error::BadEntropyBitCount(100),
                "entropy bit count",
                "BadEntropyBitCount",
            ),
            (bip39::Error::BadWordCount(13), "word count", "BadWordCount"),
            (
                bip39::Error::InvalidChecksum,
                "checksum failure",
                "InvalidChecksum",
            ),
        ];
        for (e, needle, variant) in rows {
            let m = friendly_bip39(&e);
            assert!(
                m.contains("BIP-39"),
                "missing BIP-39 tag for {variant}: {m}"
            );
            assert!(m.contains(needle), "expected {needle:?} for {variant}: {m}");
            assert!(!m.contains(variant), "variant name {variant} leaked: {m}");
        }
    }

    #[test]
    fn bitcoin_all_arms_render_prose() {
        use crate::error::BitcoinErrorKind as B;
        // friendly_bitcoin has no uniform codec tag across arms: only the Bip32
        // arm carries "BIP-32"; XpubParse/FingerprintParse carry the flag name.
        // The tag is folded into the per-row needle accordingly.
        let rows: [(B, &str, &str); 3] = [
            (
                // A UNIT bip32 variant — avoids wrapper variants with payloads.
                B::Bip32(bitcoin::bip32::Error::CannotDeriveFromHardenedKey),
                "BIP-32",
                "Bip32",
            ),
            (B::XpubParse("bad base58".into()), "--xpub", "XpubParse"),
            (
                B::FingerprintParse("bad hex".into()),
                "--master-fingerprint",
                "FingerprintParse",
            ),
        ];
        for (e, needle, variant) in rows {
            let m = friendly_bitcoin(&e);
            assert!(m.contains(needle), "expected {needle:?} for {variant}: {m}");
            assert!(!m.contains(variant), "variant name {variant} leaked: {m}");
        }
    }
}
