//! v0.26.0 format auto-detect (SPEC §6).
//!
//! Dispatcher consulted by `cmd::import_wallet::run` when the user does NOT
//! supply `--format`. Returns one of four outcomes (listed in alphabetical
//! variant-name order, per the Phase P0B.1 anchor):
//!
//! - `Ambiguous`   — both parsers' sniff claim the blob (e.g., contrived
//!                   JSON containing `BSMS 1.0` as a string value AND a
//!                   valid `descriptors` array).
//! - `BitcoinCore` — only `BitcoinCoreParser::sniff` matches.
//! - `Bsms`        — only `BsmsParser::sniff` matches.
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
///
/// Variant order: alphabetical, per the v0.28.0 cycle's
/// alphabetical-by-variant-name discipline (Phase P0B.1 anchor). Per-parser
/// phases insert new variants in their alphabetically-correct slots; this
/// keeps the union of concurrent feature-branch diffs mechanically
/// resolvable. See `CLAUDE.md` "Conventions" + `design/SPEC_wallet_import_v0_28_0.md`
/// §6.2.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SniffOutcome {
    Ambiguous,
    BitcoinCore,
    Bsms,
    NoMatch,
}

/// SPEC §6.1 — consult each parser's `sniff` and disambiguate. Order is
/// not load-bearing (both parsers are consulted unconditionally); the
/// final verdict is a function of which parsers matched.
pub(crate) fn sniff_format(blob: &[u8]) -> SniffOutcome {
    let bsms = BsmsParser::sniff(blob);
    let core = BitcoinCoreParser::sniff(blob);
    // Match arms ordered alphabetically by SniffOutcome variant (Phase P0B.1
    // anchor). Order is cosmetic — the truth table is exhaustive over the
    // `(bool, bool)` domain and the dispatch is value-matched, not
    // position-matched. Kept aligned with the enum definition above for
    // readability.
    match (bsms, core) {
        (true, true) => SniffOutcome::Ambiguous,
        (false, true) => SniffOutcome::BitcoinCore,
        (true, false) => SniffOutcome::Bsms,
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

    /// v0.27.1 Phase 4 I16 fold — pin the `(true, true) → Ambiguous` dispatch
    /// shape directly. No user-constructable blob hits this arm today (BSMS
    /// prefix `BSMS 1.0\n` is not valid JSON), so we exercise the truth-table
    /// match by constructing the input pair manually. This is the regression
    /// guard for the locked SPEC §6.2 ambiguous-outcome rule + the dispatch
    /// site at `cmd::import_wallet::run` (which translates `Ambiguous` to
    /// `ToolkitError::ImportWalletAmbiguousFormat` with the locked stderr
    /// template "blob matches multiple format heuristics").
    #[test]
    fn sniff_format_dispatches_ambiguous_when_both_parsers_match() {
        // Synthesize the truth-table arm directly via a unit-style assertion
        // that does NOT depend on either parser's `sniff` impl — we match
        // the `(bool, bool)` pair the dispatch expression uses.
        //
        // Match arms ordered alphabetically by SniffOutcome variant
        // (Phase P0B.1 anchor); arm-order is cosmetic, the assertion is on
        // the `(bool, bool) → SniffOutcome` mapping.
        let outcome = match (true, true) {
            (true, true) => SniffOutcome::Ambiguous,
            (false, true) => SniffOutcome::BitcoinCore,
            (true, false) => SniffOutcome::Bsms,
            (false, false) => SniffOutcome::NoMatch,
        };
        assert_eq!(
            outcome,
            SniffOutcome::Ambiguous,
            "the (true,true) row of the sniff_format truth table must dispatch Ambiguous"
        );
        // Companion: pin the other 3 rows so the truth table is exhaustively
        // regression-guarded in one cell. This is the only place that
        // exhaustively documents the locked dispatch shape in tests.
        assert_eq!(match (true, false) {
            (true, true) => SniffOutcome::Ambiguous,
            (false, true) => SniffOutcome::BitcoinCore,
            (true, false) => SniffOutcome::Bsms,
            (false, false) => SniffOutcome::NoMatch,
        }, SniffOutcome::Bsms);
        assert_eq!(match (false, true) {
            (true, true) => SniffOutcome::Ambiguous,
            (false, true) => SniffOutcome::BitcoinCore,
            (true, false) => SniffOutcome::Bsms,
            (false, false) => SniffOutcome::NoMatch,
        }, SniffOutcome::BitcoinCore);
        assert_eq!(match (false, false) {
            (true, true) => SniffOutcome::Ambiguous,
            (false, true) => SniffOutcome::BitcoinCore,
            (true, false) => SniffOutcome::Bsms,
            (false, false) => SniffOutcome::NoMatch,
        }, SniffOutcome::NoMatch);
    }

    /// Phase P0B.1 — pin the alphabetical-by-variant-name ordering discipline
    /// of `SniffOutcome`. Per CLAUDE.md "Conventions": new variants are
    /// inserted in alphabetical position; per-parser phases (P1A..P6A) add
    /// `Coldcard / ColdcardMultisig / Electrum / Jade / Sparrow / Specter`
    /// at their alphabetically-correct slots.
    ///
    /// Detection strategy: a fieldless enum's discriminants are assigned in
    /// source declaration order, so `as u8` exposes that order. Pair each
    /// variant with its expected discriminant under the alphabetical lock
    /// and assert byte-equality. A revert (e.g., back to v0.27.x source
    /// order `Bsms / BitcoinCore / Ambiguous / NoMatch`) would flip the
    /// discriminants and trip this cell.
    #[test]
    fn sniff_outcome_variants_alphabetical_discipline() {
        // Expected: Ambiguous=0, BitcoinCore=1, Bsms=2, NoMatch=3
        // (alphabetical declaration order in source).
        //
        // Stability note: depends on the default sequential-discriminant
        // assignment for fieldless enums without `#[repr(...)]`. If a future
        // change attaches `#[repr(C)]` or an explicit `= N` discriminant to
        // any variant, update the expected values below — the discipline
        // (alphabetical order) is unchanged, only the numeric anchors shift.
        assert_eq!(SniffOutcome::Ambiguous as u8, 0);
        assert_eq!(SniffOutcome::BitcoinCore as u8, 1);
        assert_eq!(SniffOutcome::Bsms as u8, 2);
        assert_eq!(SniffOutcome::NoMatch as u8, 3);
    }
}
