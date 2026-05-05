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
        mode: &'static str,
        flag: &'static str,
        message: String,
    },
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

impl ToolkitError {
    /// SPEC §6.1 exit-code mapping.
    pub fn exit_code(&self) -> u8 {
        match self {
            ToolkitError::BadInput(_)
            | ToolkitError::Bip39(_)
            | ToolkitError::Bitcoin(_)
            | ToolkitError::MsCodec(_)
            | ToolkitError::MkCodec(_)
            | ToolkitError::MdCodec(_) => 1,
            ToolkitError::ModeViolation { .. } | ToolkitError::NetworkMismatch { .. } => 2,
            ToolkitError::FutureFormat { .. } => 3,
            ToolkitError::BundleMismatch { .. } => 4,
        }
    }

    /// Stable discriminant for JSON `kind` field (SPEC §5.5).
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
            ToolkitError::ModeViolation { message, .. } => message.clone(),
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
                message: "x".into(),
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
