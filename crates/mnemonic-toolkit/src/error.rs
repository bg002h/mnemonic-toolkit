//! ToolkitError + exit_code() + per-source From impls.
//!
//! Realizes SPEC §6.1 (exit-code table), §6.2 (ToolkitError enum),
//! §6.3 (exit-code mapping), §6.4.0 (routing principle).

use serde_json::json;

#[derive(Debug)]
#[non_exhaustive]
pub enum ToolkitError {
    BadInput(String),
    Bip39(bip39::Error),
    Bitcoin(BitcoinErrorKind),
    MsCodec(ms_codec::Error),
    MkCodec(mk_codec::Error),
    MdCodec(md_codec::Error),
    ModeViolation {
        // mode/flag are read by `details()` for SPEC §5.5 JSON output (wired in v0.1+ JSON path).
        #[allow(dead_code)]
        mode: &'static str,
        #[allow(dead_code)]
        flag: &'static str,
        message: &'static str,
    },
    /// SPEC §6.1 exit-4 variant. Constructed by integration tests in Phase 5; reserved
    /// for runtime emission once verify-bundle's optional-mismatch reporter wires up.
    #[allow(dead_code)]
    BundleMismatch {
        card: &'static str,
        message: String,
    },
    NetworkMismatch {
        xpub_network: &'static str,
        expected: &'static str,
    },
    FutureFormat {
        source: &'static str,
        detail: String,
    },
}

#[derive(Debug)]
pub enum BitcoinErrorKind {
    Bip32(bitcoin::bip32::Error),
    XpubParse(String),
    FingerprintParse(String),
}

/// SPEC §6.4.3 routing (delegates to ms-cli's §6.1.1 dispatch table).
/// `ReservedTagNotEmittedInV01` is intercepted by `From` to `FutureFormat` (exit 3).
fn ms_codec_exit_code(e: &ms_codec::Error) -> u8 {
    match e {
        ms_codec::Error::Codex32(_)
        | ms_codec::Error::UnexpectedStringLength { .. }
        | ms_codec::Error::PayloadLengthMismatch { .. } => 1,
        ms_codec::Error::WrongHrp { .. }
        | ms_codec::Error::ThresholdNotZero { .. }
        | ms_codec::Error::ShareIndexNotSecret { .. }
        | ms_codec::Error::TagInvalidAlphabet { .. }
        | ms_codec::Error::UnknownTag { .. }
        | ms_codec::Error::ReservedPrefixViolation { .. } => 2,
        // ReservedTagNotEmittedInV01 is intercepted by From → FutureFormat.
        _ => 1,
    }
}

/// SPEC §6.4.4 routing. `UnsupportedVersion` is intercepted by `From` to `FutureFormat`.
fn mk_codec_exit_code(e: &mk_codec::Error) -> u8 {
    match e {
        mk_codec::Error::InvalidStringLength(_)
        | mk_codec::Error::InvalidChar { .. }
        | mk_codec::Error::BchUncorrectable(_) => 1,
        mk_codec::Error::InvalidHrp(_)
        | mk_codec::Error::MixedCase
        | mk_codec::Error::UnsupportedCardType(_)
        | mk_codec::Error::MalformedPayloadPadding
        | mk_codec::Error::ChunkSetIdMismatch
        | mk_codec::Error::ChunkedHeaderMalformed(_)
        | mk_codec::Error::MixedHeaderTypes
        | mk_codec::Error::CrossChunkHashMismatch
        | mk_codec::Error::ReservedBitsSet
        | mk_codec::Error::InvalidPolicyIdStubCount
        | mk_codec::Error::InvalidPathIndicator(_)
        | mk_codec::Error::PathTooDeep(_)
        | mk_codec::Error::InvalidPathComponent(_)
        | mk_codec::Error::InvalidXpubVersion(_)
        | mk_codec::Error::InvalidXpubPublicKey(_)
        | mk_codec::Error::UnexpectedEnd
        | mk_codec::Error::TrailingBytes
        | mk_codec::Error::CardPayloadTooLarge { .. } => 2,
        // UnsupportedVersion is intercepted by From → FutureFormat.
        _ => 1,
    }
}

/// SPEC §6.4.5 routing. md_codec::Error is NOT `#[non_exhaustive]`; match is exhaustive.
/// `UnsupportedVersion` is intercepted by `From` to `FutureFormat` (exit 3).
fn md_codec_exit_code(e: &md_codec::Error) -> u8 {
    match e {
        md_codec::Error::Codex32DecodeError(_) | md_codec::Error::Codex32EncodeError(_) => 1,
        md_codec::Error::BitStreamTruncated { .. }
        | md_codec::Error::ReservedHeaderBitSet
        | md_codec::Error::PathDepthExceeded { .. }
        | md_codec::Error::KeyCountOutOfRange { .. }
        | md_codec::Error::DivergentPathCountMismatch { .. }
        | md_codec::Error::AltCountOutOfRange { .. }
        | md_codec::Error::UnknownPrimaryTag(_)
        | md_codec::Error::UnknownExtensionTag(_)
        | md_codec::Error::ThresholdOutOfRange { .. }
        | md_codec::Error::ChildCountOutOfRange { .. }
        | md_codec::Error::KGreaterThanN { .. }
        | md_codec::Error::TlvOrderingViolation { .. }
        | md_codec::Error::PlaceholderIndexOutOfRange { .. }
        | md_codec::Error::OverrideOrderViolation { .. }
        | md_codec::Error::EmptyTlvEntry { .. }
        | md_codec::Error::TlvLengthExceedsRemaining { .. }
        | md_codec::Error::PlaceholderNotReferenced { .. }
        | md_codec::Error::PlaceholderFirstOccurrenceOutOfOrder { .. }
        | md_codec::Error::MultipathAltCountMismatch { .. }
        | md_codec::Error::ForbiddenTapTreeLeaf { .. }
        | md_codec::Error::ChunkCountOutOfRange { .. }
        | md_codec::Error::ChunkIndexOutOfRange { .. }
        | md_codec::Error::ChunkSetIdOutOfRange { .. }
        | md_codec::Error::ChunkHeaderChunkedFlagMissing
        | md_codec::Error::ChunkCountExceedsMax { .. }
        | md_codec::Error::ChunkSetEmpty
        | md_codec::Error::ChunkSetInconsistent
        | md_codec::Error::ChunkSetIncomplete { .. }
        | md_codec::Error::ChunkIndexGap { .. }
        | md_codec::Error::ChunkSetIdMismatch { .. }
        | md_codec::Error::VarintOverflow { .. }
        | md_codec::Error::MissingExplicitOrigin { .. }
        | md_codec::Error::InvalidPresenceByte { .. }
        | md_codec::Error::InvalidXpubBytes { .. }
        | md_codec::Error::MissingPubkey { .. }
        | md_codec::Error::ChainIndexOutOfRange { .. }
        | md_codec::Error::HardenedPublicDerivation
        | md_codec::Error::UnsupportedDerivationShape => 2,
        // UnsupportedVersion is intercepted by From → FutureFormat.
        md_codec::Error::UnsupportedVersion { .. } => 3,
    }
}

impl ToolkitError {
    /// SPEC §6.1 exit-code mapping; sibling-codec wrappers dispatch to per-variant
    /// helpers per SPEC §6.4.3 / §6.4.4 / §6.4.5 routing tables.
    pub fn exit_code(&self) -> u8 {
        match self {
            ToolkitError::BadInput(_) | ToolkitError::Bip39(_) | ToolkitError::Bitcoin(_) => 1,
            ToolkitError::MsCodec(e) => ms_codec_exit_code(e),
            ToolkitError::MkCodec(e) => mk_codec_exit_code(e),
            ToolkitError::MdCodec(e) => md_codec_exit_code(e),
            ToolkitError::ModeViolation { .. } | ToolkitError::NetworkMismatch { .. } => 2,
            ToolkitError::FutureFormat { .. } => 3,
            ToolkitError::BundleMismatch { .. } => 4,
        }
    }

    /// Stable discriminant for JSON `kind` field (SPEC §5.5).
    /// Reserved for the §5.5 JSON-error envelope path (covered by tests in v0.1).
    #[allow(dead_code)]
    pub fn kind(&self) -> &'static str {
        match self {
            ToolkitError::BadInput(_) => "BadInput",
            ToolkitError::Bip39(_) => "Bip39",
            ToolkitError::Bitcoin(_) => "Bitcoin",
            ToolkitError::MsCodec(_) => "MsCodec",
            ToolkitError::MkCodec(_) => "MkCodec",
            ToolkitError::MdCodec(_) => "MdCodec",
            ToolkitError::ModeViolation { .. } => "ModeViolation",
            ToolkitError::NetworkMismatch { .. } => "NetworkMismatch",
            ToolkitError::BundleMismatch { .. } => "BundleMismatch",
            ToolkitError::FutureFormat { .. } => "FutureFormat",
        }
    }

    /// Friendly human-readable message. Five sibling-source mappers live in
    /// `friendly.rs` (Phase 3 task 3.3) and are dispatched here.
    pub fn message(&self) -> String {
        match self {
            ToolkitError::BadInput(m) => m.clone(),
            ToolkitError::Bip39(e) => crate::friendly::friendly_bip39(e),
            ToolkitError::Bitcoin(e) => crate::friendly::friendly_bitcoin(e),
            ToolkitError::MsCodec(e) => crate::friendly::friendly_ms_codec(e),
            ToolkitError::MkCodec(e) => crate::friendly::friendly_mk_codec(e),
            ToolkitError::MdCodec(e) => crate::friendly::friendly_md_codec(e),
            ToolkitError::ModeViolation { message, .. } => (*message).to_owned(),
            ToolkitError::NetworkMismatch {
                xpub_network,
                expected,
            } => format!(
                "xpub network {} does not match --network {}",
                xpub_network, expected,
            ),
            ToolkitError::BundleMismatch { card, message } => {
                format!("bundle mismatch on {}: {}; v0.1 hardcodes account=0; if the engraved bundle was produced with a non-zero account, mismatch is expected — re-run with v0.2's --account flag once available",
                    card, message)
            }
            ToolkitError::FutureFormat { source, detail } => format!(
                "{} reserved-not-emitted: {}; deferred to v0.2+",
                source, detail,
            ),
        }
    }

    /// JSON `details` field (SPEC §5.5).
    /// Reserved for the §5.5 JSON-error envelope path.
    #[allow(dead_code)]
    pub fn details(&self) -> Option<serde_json::Value> {
        match self {
            ToolkitError::ModeViolation { mode, flag, .. } => Some(json!({
                "mode": mode,
                "flag": flag,
            })),
            ToolkitError::NetworkMismatch {
                xpub_network,
                expected,
            } => Some(json!({
                "xpub_network": xpub_network,
                "expected": expected,
            })),
            ToolkitError::BundleMismatch { card, .. } => Some(json!({ "card": card })),
            ToolkitError::FutureFormat { source, detail } => Some(json!({
                "source": source,
                "detail": detail,
            })),
            _ => None,
        }
    }
}

impl std::fmt::Display for ToolkitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error: {}", self.message())
    }
}

impl std::error::Error for ToolkitError {}

impl From<bip39::Error> for ToolkitError {
    fn from(e: bip39::Error) -> Self {
        ToolkitError::Bip39(e)
    }
}

impl From<bitcoin::bip32::Error> for ToolkitError {
    fn from(e: bitcoin::bip32::Error) -> Self {
        ToolkitError::Bitcoin(BitcoinErrorKind::Bip32(e))
    }
}

impl From<ms_codec::Error> for ToolkitError {
    fn from(e: ms_codec::Error) -> Self {
        match e {
            ms_codec::Error::ReservedTagNotEmittedInV01 { got } => ToolkitError::FutureFormat {
                source: "ms_codec",
                detail: format!(
                    "reserved tag {:?}",
                    std::str::from_utf8(&got).unwrap_or("<non-utf8>")
                ),
            },
            other => ToolkitError::MsCodec(other),
        }
    }
}

impl From<mk_codec::Error> for ToolkitError {
    fn from(e: mk_codec::Error) -> Self {
        match e {
            mk_codec::Error::UnsupportedVersion(v) => ToolkitError::FutureFormat {
                source: "mk_codec",
                detail: format!("unsupported version {}", v),
            },
            other => ToolkitError::MkCodec(other),
        }
    }
}

impl From<md_codec::Error> for ToolkitError {
    fn from(e: md_codec::Error) -> Self {
        match e {
            md_codec::Error::UnsupportedVersion { got } => ToolkitError::FutureFormat {
                source: "md_codec",
                detail: format!("unsupported version {}", got),
            },
            other => ToolkitError::MdCodec(other),
        }
    }
}

/// Convenience alias; reserved for in-crate use.
#[allow(dead_code)]
pub type Result<T> = std::result::Result<T, ToolkitError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_code_table_per_variant() {
        assert_eq!(ToolkitError::BadInput("x".into()).exit_code(), 1);
        assert_eq!(
            ToolkitError::ModeViolation {
                mode: "watch-only",
                flag: "--passphrase",
                message: "x",
            }
            .exit_code(),
            2,
        );
        assert_eq!(
            ToolkitError::NetworkMismatch {
                xpub_network: "main",
                expected: "test"
            }
            .exit_code(),
            2,
        );
        assert_eq!(
            ToolkitError::FutureFormat {
                source: "ms_codec",
                detail: "x".into()
            }
            .exit_code(),
            3,
        );
        assert_eq!(
            ToolkitError::BundleMismatch {
                card: "mk1",
                message: "x".into()
            }
            .exit_code(),
            4,
        );
    }

    #[test]
    fn ms_codec_inner_variant_routing() {
        // Exit-2 (format-violation) variants per SPEC §6.4.3.
        assert_eq!(
            ToolkitError::MsCodec(ms_codec::Error::WrongHrp { got: "mq".into() }).exit_code(),
            2,
        );
        assert_eq!(
            ToolkitError::MsCodec(ms_codec::Error::ReservedPrefixViolation { got: 0x01 })
                .exit_code(),
            2,
        );
        // Exit-1 (user-input) variants.
        assert_eq!(
            ToolkitError::MsCodec(ms_codec::Error::UnexpectedStringLength {
                got: 51,
                allowed: &[],
            })
            .exit_code(),
            1,
        );
    }

    #[test]
    fn mk_codec_inner_variant_routing() {
        // Exit-2 (format-violation) variants per SPEC §6.4.4.
        assert_eq!(
            ToolkitError::MkCodec(mk_codec::Error::InvalidHrp("foo".into())).exit_code(),
            2,
        );
        assert_eq!(
            ToolkitError::MkCodec(mk_codec::Error::ReservedBitsSet).exit_code(),
            2,
        );
        assert_eq!(
            ToolkitError::MkCodec(mk_codec::Error::MalformedPayloadPadding).exit_code(),
            2,
        );
        // Exit-1 (user-input) variants.
        assert_eq!(
            ToolkitError::MkCodec(mk_codec::Error::InvalidStringLength(50)).exit_code(),
            1,
        );
        assert_eq!(
            ToolkitError::MkCodec(mk_codec::Error::BchUncorrectable("foo".into())).exit_code(),
            1,
        );
    }

    #[test]
    fn md_codec_inner_variant_routing() {
        // Exit-2 (format-violation) variants per SPEC §6.4.5.
        assert_eq!(
            ToolkitError::MdCodec(md_codec::Error::ReservedHeaderBitSet).exit_code(),
            2,
        );
        assert_eq!(
            ToolkitError::MdCodec(md_codec::Error::ChunkSetEmpty).exit_code(),
            2,
        );
        assert_eq!(
            ToolkitError::MdCodec(md_codec::Error::HardenedPublicDerivation).exit_code(),
            2,
        );
        // Exit-1 (user-input) variants.
        assert_eq!(
            ToolkitError::MdCodec(md_codec::Error::Codex32DecodeError("foo".into())).exit_code(),
            1,
        );
        assert_eq!(
            ToolkitError::MdCodec(md_codec::Error::Codex32EncodeError("bar".into())).exit_code(),
            1,
        );
    }

    #[test]
    fn kind_strings_stable() {
        assert_eq!(ToolkitError::BadInput("x".into()).kind(), "BadInput");
        assert_eq!(
            ToolkitError::BundleMismatch {
                card: "ms1",
                message: "".into()
            }
            .kind(),
            "BundleMismatch",
        );
        assert_eq!(
            ToolkitError::FutureFormat {
                source: "ms_codec",
                detail: "".into()
            }
            .kind(),
            "FutureFormat",
        );
    }
}
