//! v0.26.0 format auto-detect (SPEC §6).
//!
//! Dispatcher consulted by `cmd::import_wallet::run` when the user does NOT
//! supply `--format`. Returns one of four outcomes:
//!
//! - `Bsms`        — only `BsmsParser::sniff` matches.
//! - `BitcoinCore` — only `BitcoinCoreParser::sniff` matches.
//! - `Ambiguous`   — both parsers' sniff claim the blob (e.g., contrived
//!                   JSON containing `BSMS 1.0` as a string value AND a
//!                   valid `descriptors` array).
//! - `NoMatch`     — neither parser's sniff claims the blob.
//!
//! Per SPEC §6.1:
//! - BSMS: blob (post CRLF→LF normalize) starts with `BSMS 1.0\n`.
//! - Bitcoin Core: valid JSON whose top-level object has a non-empty
//!   `descriptors: [{desc: String}]` array AND NO vendor-marker keys
//!   (`chain`, `policy`, `version`, etc.) that would indicate Specter /
//!   Sparrow / similar. Conservative-only per SPEC §6.1.2 lock.
//!
//! Per SPEC §6.2:
//! - 0 matches → `NoMatch` → caller emits `ImportWalletAmbiguousFormat`
//!   exit 1 with "could not detect format" template.
//! - 2 matches → `Ambiguous` → caller emits `ImportWalletAmbiguousFormat`
//!   exit 1 with "blob matches multiple format heuristics" template.

use super::bitcoin_core::BitcoinCoreParser;
use super::bsms::BsmsParser;
use super::WalletFormatParser;

/// SPEC §6 — sniff verdict. Names mirror SPEC §2.1 `--format` values where
/// possible (`Bsms` / `BitcoinCore`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SniffOutcome {
    Bsms,
    BitcoinCore,
    Ambiguous,
    NoMatch,
}

/// SPEC §6.1 — consult each parser's `sniff` and disambiguate. Order is
/// not load-bearing (both parsers are consulted unconditionally); the
/// final verdict is a function of which parsers matched.
pub(crate) fn sniff_format(blob: &[u8]) -> SniffOutcome {
    let bsms = BsmsParser::sniff(blob);
    let core = BitcoinCoreParser::sniff(blob);
    match (bsms, core) {
        (true, false) => SniffOutcome::Bsms,
        (false, true) => SniffOutcome::BitcoinCore,
        (true, true) => SniffOutcome::Ambiguous,
        (false, false) => SniffOutcome::NoMatch,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sniff_bsms_2line_lf() {
        let blob = b"BSMS 1.0\nwpkh([deadbeef/84'/0'/0']xpub...)\n";
        assert_eq!(sniff_format(blob), SniffOutcome::Bsms);
    }

    #[test]
    fn sniff_bsms_2line_crlf() {
        let blob = b"BSMS 1.0\r\nwpkh([deadbeef/84'/0'/0']xpub...)\r\n";
        assert_eq!(sniff_format(blob), SniffOutcome::Bsms);
    }

    #[test]
    fn sniff_core_object_descriptors() {
        let blob = br#"{"wallet_name":"a","descriptors":[{"desc":"wpkh(xpub...)#abcdefgh"}]}"#;
        assert_eq!(sniff_format(blob), SniffOutcome::BitcoinCore);
    }

    #[test]
    fn sniff_core_vendor_marker_rejected() {
        // SPEC §6.1.2: presence of vendor-marker key (chain) → NoMatch even
        // though `descriptors` array is well-formed.
        let blob = br#"{"chain":"main","descriptors":[{"desc":"wpkh(xpub...)#abcdefgh"}]}"#;
        assert_eq!(sniff_format(blob), SniffOutcome::NoMatch);
    }

    #[test]
    fn sniff_no_match_random_text() {
        let blob = b"some random text\n";
        assert_eq!(sniff_format(blob), SniffOutcome::NoMatch);
    }

    #[test]
    fn sniff_no_match_empty() {
        let blob = b"";
        assert_eq!(sniff_format(blob), SniffOutcome::NoMatch);
    }

    #[test]
    fn sniff_ambiguous_bsms_header_inside_json_value() {
        // Contrived: a blob that starts with `BSMS 1.0\n` (so BSMS sniff
        // matches) AND happens to parse as JSON with a `descriptors` key.
        // In practice, this is hard to construct because `BSMS 1.0\n…` is
        // not valid JSON. The Ambiguous arm is reserved per SPEC §6.2 for
        // any future heuristic that could co-fire; for now we exercise
        // the dispatch shape via a forced-positive case below.
        //
        // The intended user-facing ambiguous trigger is a JSON blob
        // containing BOTH `BSMS 1.0` as a literal string AND a valid
        // `descriptors` array. SPEC §6.1.1's BSMS sniff is a strict prefix
        // match, so this JSON-with-BSMS-substring case is in fact NoMatch
        // (BSMS sniff returns false because the blob starts with `{`).
        // We document this in the test rather than fabricate a contrived
        // double-match.
        let blob = br#"{"descriptors":[{"desc":"wpkh(xpub...)#abcdefgh"}],"note":"BSMS 1.0"}"#;
        // BSMS sniff: false (no `BSMS 1.0\n` prefix). Core sniff: true.
        // Verdict: BitcoinCore (not Ambiguous).
        assert_eq!(sniff_format(blob), SniffOutcome::BitcoinCore);
    }

    #[test]
    fn sniff_core_bare_array_not_matched() {
        // SPEC §6.1.2 is currently object-only at sniff time. A bare-array
        // top-level JSON (the export emitter's shape) is recognized at
        // parse time but the sniff is conservative — sniff returns false
        // → user must pass `--format bitcoin-core` explicitly. Document
        // this here so the behavior is pinned.
        let blob = br#"[{"desc":"wpkh(xpub...)#abcdefgh"}]"#;
        assert_eq!(sniff_format(blob), SniffOutcome::NoMatch);
    }

    #[test]
    fn sniff_core_empty_descriptors_array_not_matched() {
        let blob = br#"{"descriptors":[]}"#;
        assert_eq!(sniff_format(blob), SniffOutcome::NoMatch);
    }

    #[test]
    fn sniff_core_desc_missing_not_matched() {
        let blob = br#"{"descriptors":[{"foo":"bar"}]}"#;
        assert_eq!(sniff_format(blob), SniffOutcome::NoMatch);
    }
}
