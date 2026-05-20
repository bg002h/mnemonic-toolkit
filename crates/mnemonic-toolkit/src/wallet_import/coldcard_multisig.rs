//! Coldcard multisig text-file parser (`--format coldcard-multisig`).
//!
//! Per `design/SPEC_wallet_import_v0_28_0.md` §11.4. This format is a
//! line-oriented TEXT shape (NOT JSON) produced by Coldcard firmware when
//! the user exports a multisig wallet (`Settings → Multisig Wallets →
//! Export`). The shape is also accepted byte-identically by Blockstream
//! Jade via its `register_multisig` RPC `multisig_file` reply field —
//! `wallet_import/jade.rs` (Phase P5B) delegates here for the inner text.
//!
//! ## Sniff signature
//!
//! Top-of-blob lines (UTF-8; CRLF normalized to LF) contain ALL of:
//! - `Name:` line-prefix
//! - `Policy:` line-prefix
//! - `Format:` line-prefix
//!
//! Any leading `# …` comment lines and blank lines are tolerated. The
//! `XFP:` header line is OPTIONAL (firmware-variance). Sniff scans the
//! first ~20 lines of the blob (well above the maximum header-block size)
//! for these markers.
//!
//! ## On-disk shape
//!
//! Two firmware-variance shapes are accepted at parse time:
//!
//! 1. **Shared-derivation shape** (matches `wallet_export/coldcard.rs:254
//!    emit_coldcard_multisig_text` output — the toolkit's own emit form):
//!    ```text
//!    Name: <wallet-name>
//!    Policy: <K> of <N>
//!    Derivation: m/...
//!    Format: P2WSH | P2SH-P2WSH | P2SH
//!    <XFP>: <xpub>
//!    <XFP>: <xpub>
//!    ...
//!    ```
//!
//! 2. **Per-cosigner shape** (older Coldcard firmware + several third-party
//!    coordinators emit this form):
//!    ```text
//!    Name: <wallet-name>
//!    Policy: <K>-of-<N>
//!    Format: P2WSH
//!    Derivation: m/...
//!    <xpub>
//!    Derivation: m/...
//!    <xpub>
//!    ...
//!    ```
//!
//! Also accepted: an optional leading `XFP: <hex>` line carrying the master
//! fingerprint (Coldcard variant). When present, it OVERRIDES the
//! computed-from-xpub fingerprint per SPEC §11.4.1 5-row truth table.
//!
//! ## Provenance
//!
//! `ImportProvenance::ColdcardMultisig(ColdcardMultisigSourceMetadata)`.
//! `xfp_was_blob_supplied` / `xfp_header_disagreed` flags are populated per
//! the SPEC §11.4.1 truth table; the WARNING stderr message is emitted
//! during `parse` (not `sniff`).

use super::{ParsedImport, WalletFormatParser};
use crate::error::ToolkitError;
use std::io::Write;

pub(crate) struct ColdcardMultisigParser;

/// SPEC §11.4 — line-oriented Coldcard multisig text format provenance.
///
/// Carries the parsed header fields + the xfp-policy telemetry flags
/// per SPEC §11.4.1 (5-row truth table). `xfp_was_blob_supplied` is `true`
/// when the blob carried an `XFP:` header line; `xfp_header_disagreed` is
/// `true` only when both the header AND a computed fingerprint were
/// available AND they did NOT byte-match (the WARNING-fire row of the
/// truth table).
///
/// All fields populated by P4B's parse; at P4A the struct is declared but
/// only constructed in unit tests (the `parse()` method returns
/// `Err(ImportWalletParse(... lands in P4B))` until P4B's body lands).
/// `#[allow(dead_code)]` on each field for the P4A → P4B interim.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct ColdcardMultisigSourceMetadata {
    pub(crate) name: String,
    pub(crate) policy: PolicyKOfN,
    pub(crate) script_format: ColdcardMsFormat,
    /// SPEC §11.4.1 telemetry: blob carried an `XFP:` header line.
    pub(crate) xfp_was_blob_supplied: bool,
    /// SPEC §11.4.1 telemetry: header present AND computed available AND
    /// the two disagreed (WARNING surfaced via stderr). `false` for the
    /// silent-match row and for rows where `xfp_was_blob_supplied=false`.
    pub(crate) xfp_header_disagreed: bool,
    /// Future-proof: parser-encountered field names that were NOT consumed
    /// into typed metadata fields. Currently empty (header schema is
    /// closed); reserved for forward-compat with firmware extensions.
    pub(crate) dropped_fields: Vec<String>,
}

/// SPEC §11.4 — K-of-N policy as parsed from the `Policy:` header line.
/// Both `K of N` (space form, the toolkit's own emit) and `K-of-N` (dash
/// form, third-party variant) accepted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) struct PolicyKOfN {
    pub(crate) k: u8,
    pub(crate) n: u8,
}

/// SPEC §11.4 — `Format:` header script-type discriminator. Maps to the
/// descriptor synthesis wrapper (`wsh(...)` vs `sh(wsh(...))` vs `sh(...)`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) enum ColdcardMsFormat {
    P2wsh,
    P2shP2wsh,
    P2sh,
}

impl WalletFormatParser for ColdcardMultisigParser {
    fn sniff(blob: &[u8]) -> bool {
        // SPEC §11.4 sniff: must be valid UTF-8 (Coldcard text export is
        // ASCII-only in practice; UTF-8 superset accepted for tolerance);
        // must contain `Name:` + `Policy:` + `Format:` line-prefixes
        // within the first ~20 lines of the blob (the header block is
        // ~5 lines + optional XFP/Derivation; 20 is far above any
        // plausible header-block size).
        let text = match std::str::from_utf8(blob) {
            Ok(s) => s,
            Err(_) => return false,
        };
        let normalized = text.replace("\r\n", "\n");
        let header_lines: Vec<&str> = normalized.lines().take(20).collect();
        let has_name = header_lines.iter().any(|l| line_key(l) == Some("Name"));
        let has_policy = header_lines.iter().any(|l| line_key(l) == Some("Policy"));
        let has_format = header_lines.iter().any(|l| line_key(l) == Some("Format"));
        has_name && has_policy && has_format
    }

    fn parse(_blob: &[u8], _stderr: &mut dyn Write) -> Result<Vec<ParsedImport>, ToolkitError> {
        // P4A skeleton — full parse implementation lands in P4B.
        Err(ToolkitError::ImportWalletParse(
            "import-wallet: coldcard-multisig: parse not yet implemented (Phase P4B)".to_string(),
        ))
    }
}

/// Extract the "key" portion of a line of the form `Key: value`. Returns
/// `Some("Key")` (trimmed of surrounding whitespace, case-preserved) when
/// the line matches `^<word>:<rest>`; returns `None` for blank lines,
/// comment lines (`# …`), or lines without a `:` separator.
///
/// Used by both sniff (header-presence check) and parse (line classification).
pub(super) fn line_key(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return None;
    }
    let colon = trimmed.find(':')?;
    let key = trimmed[..colon].trim();
    if key.is_empty() {
        return None;
    }
    Some(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// SPEC §11.4 sniff: shared-derivation shape (toolkit's own emit form).
    /// Headers in order: Name / Policy / Derivation / Format.
    #[test]
    fn sniff_true_on_shared_derivation_shape() {
        let blob = b"Name: testwallet\n\
Policy: 2 of 3\n\
Derivation: m/48'/0'/0'/2'\n\
Format: P2WSH\n\
\n\
B8688DF1: xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX\n";
        assert!(ColdcardMultisigParser::sniff(blob));
    }

    /// SPEC §11.4 sniff: per-cosigner-derivation shape (older firmware).
    #[test]
    fn sniff_true_on_per_cosigner_shape() {
        let blob = b"Name: testwallet\n\
Policy: 2-of-3\n\
Format: P2WSH\n\
Derivation: m/48'/0'/0'/2'\n\
xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX\n";
        assert!(ColdcardMultisigParser::sniff(blob));
    }

    /// SPEC §11.4 sniff: optional `XFP:` header line accepted (firmware-variance).
    #[test]
    fn sniff_true_with_xfp_header() {
        let blob = b"XFP: B8688DF1\n\
Name: testwallet\n\
Policy: 2 of 3\n\
Format: P2WSH\n\
Derivation: m/48'/0'/0'/2'\n";
        assert!(ColdcardMultisigParser::sniff(blob));
    }

    /// SPEC §11.4 sniff: CRLF blobs (Windows line endings) accepted.
    #[test]
    fn sniff_true_on_crlf() {
        let blob = b"Name: t\r\nPolicy: 2 of 3\r\nFormat: P2WSH\r\nDerivation: m/0\r\n";
        assert!(ColdcardMultisigParser::sniff(blob));
    }

    /// SPEC §11.4 sniff: missing `Format:` → false.
    #[test]
    fn sniff_false_on_missing_format() {
        let blob = b"Name: t\nPolicy: 2 of 3\nDerivation: m/0\n";
        assert!(!ColdcardMultisigParser::sniff(blob));
    }

    /// SPEC §11.4 sniff: missing `Name:` → false.
    #[test]
    fn sniff_false_on_missing_name() {
        let blob = b"Policy: 2 of 3\nFormat: P2WSH\nDerivation: m/0\n";
        assert!(!ColdcardMultisigParser::sniff(blob));
    }

    /// SPEC §11.4 sniff: missing `Policy:` → false.
    #[test]
    fn sniff_false_on_missing_policy() {
        let blob = b"Name: t\nFormat: P2WSH\nDerivation: m/0\n";
        assert!(!ColdcardMultisigParser::sniff(blob));
    }

    /// SPEC §11.4 sniff: BSMS blob → false (no Name/Policy/Format headers).
    /// Critical — BSMS parser owns this shape; ColdcardMultisig must not
    /// co-fire with BSMS.
    #[test]
    fn sniff_false_on_bsms_blob() {
        let blob = b"BSMS 1.0\nwsh(sortedmulti(2,...))#abcdefgh\n";
        assert!(!ColdcardMultisigParser::sniff(blob));
    }

    /// SPEC §11.4 sniff: Bitcoin Core JSON blob → false (JSON shape rejected
    /// at line-key extraction; `{` is not a key).
    #[test]
    fn sniff_false_on_bitcoin_core_json() {
        let blob = br#"{"wallet_name":"x","descriptors":[{"desc":"wpkh(xpub...)#abcdefgh"}]}"#;
        assert!(!ColdcardMultisigParser::sniff(blob));
    }

    /// SPEC §11.4 sniff: empty blob → false.
    #[test]
    fn sniff_false_on_empty_blob() {
        assert!(!ColdcardMultisigParser::sniff(b""));
    }

    /// SPEC §11.4 sniff: non-UTF-8 blob → false (Coldcard text export is
    /// ASCII; non-UTF-8 input cannot be a valid Coldcard multisig export).
    #[test]
    fn sniff_false_on_non_utf8() {
        let blob = &[0xFF, 0xFE, 0xFD, b'\n'];
        assert!(!ColdcardMultisigParser::sniff(blob));
    }

    /// SPEC §11.4 sniff: random text without Name/Policy/Format → false.
    #[test]
    fn sniff_false_on_random_text() {
        let blob = b"hello world\nlorem ipsum\n";
        assert!(!ColdcardMultisigParser::sniff(blob));
    }

    /// SPEC §11.4 sniff: comment lines + blank lines tolerated.
    #[test]
    fn sniff_true_with_leading_comments() {
        let blob = b"# exported from Coldcard\n\
\n\
Name: t\n\
Policy: 2 of 3\n\
Format: P2WSH\n\
Derivation: m/0\n";
        assert!(ColdcardMultisigParser::sniff(blob));
    }

    /// `line_key` helper: well-formed key:value line → Some(key).
    #[test]
    fn line_key_extracts_key_for_wellformed_line() {
        assert_eq!(line_key("Name: testwallet"), Some("Name"));
        assert_eq!(line_key("Policy: 2 of 3"), Some("Policy"));
        assert_eq!(line_key("  Format: P2WSH"), Some("Format"));
        assert_eq!(line_key("XFP: DEADBEEF"), Some("XFP"));
    }

    /// `line_key` helper: blank/comment/no-colon lines → None.
    #[test]
    fn line_key_rejects_blank_or_comment_or_keyless_lines() {
        assert_eq!(line_key(""), None);
        assert_eq!(line_key("   "), None);
        assert_eq!(line_key("# a comment"), None);
        assert_eq!(line_key("just a single line"), None);
        assert_eq!(line_key(":no key"), None);
    }

    /// `line_key` helper: per-cosigner xpub line (single base58 token) → None.
    /// The xpub itself contains no colon, so it routes to the "value" arm at
    /// parse time, not the header arm.
    #[test]
    fn line_key_rejects_bare_xpub_line() {
        let xpub = "xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX";
        assert_eq!(line_key(xpub), None);
    }
}
