//! v0.28.0 format auto-detect (SPEC §6) — N-parser consult-all-then-count.
//!
//! Dispatcher consulted by `cmd::import_wallet::run` when the user does NOT
//! supply `--format`. Returns one of the following outcomes per SPEC §6.2
//! (variants listed in alphabetical order — `Ambiguous` + `NoMatch` are
//! aggregate outcomes, NOT parsers):
//!
//! - `Ambiguous`        — ≥2 parsers' sniff claim the blob (per SPEC §6.2
//!                        all-parsers-consulted rule; caller emits
//!                        `ImportWalletAmbiguousFormat` exit 1 with the
//!                        "blob matches multiple format heuristics" template).
//! - `BitcoinCore`      — only `BitcoinCoreParser::sniff` matches.
//! - `Bsms`             — only `BsmsParser::sniff` matches.
//! - `Coldcard`         — only `ColdcardParser::sniff` matches (P3A).
//! - `ColdcardMultisig` — only `ColdcardMultisigParser::sniff` matches (P4A).
//! - `Electrum`         — only `ElectrumParser::sniff` matches (P6A).
//! - `Jade`             — only `JadeParser::sniff` matches (P5A).
//! - `NoMatch`          — no parser claims the blob (caller emits
//!                        `ImportWalletAmbiguousFormat` exit 1 with the
//!                        "could not detect format" template).
//! - `Sparrow`          — only `SparrowParser::sniff` matches (P1A).
//! - `Specter`          — only `SpecterParser::sniff` matches (P2A).
//!
//! Per SPEC §6.1 (sniff heuristics at v0.28.0 cutover; the wired bools are
//! enumerated below — the remaining placeholder `false` slots flip on as
//! their per-parser P{N}A sub-phase lands):
//! - BSMS: blob (post CRLF→LF normalize) starts with `BSMS 1.0\n`.
//! - Bitcoin Core: valid JSON whose top-level object has a non-empty
//!   `descriptors: [{desc: String}]` array AND NO vendor-marker keys
//!   (`chain`, `policy`, `version`, etc.) that would indicate Specter /
//!   Sparrow / similar. Conservative-only per SPEC §6.1.2 lock.
//! - Coldcard multisig: SPEC §11.4 4-line text-shape header (P4A wired).
//! - Sparrow: SPEC §11.1 positive-marker on `policyType` + `scriptType` +
//!   `defaultPolicy.miniscript.script` + non-empty `keystores` (P1A wired).
//! - Specter: SPEC §11.2 positive-marker on `label` + `blockheight` (integer)
//!   + `descriptor` + `devices` (P2A wired).
//!
//! Per SPEC §6.2 (consult-all-then-count dispatch, N-parser generalization
//! of v0.26.0's 2-parser 2×2 truth table):
//! - 0 matches → `NoMatch` → caller emits `ImportWalletAmbiguousFormat`
//!   exit 1 with "could not detect format" template.
//! - 1 match  → that parser's variant.
//! - ≥2 matches → `Ambiguous` → caller emits `ImportWalletAmbiguousFormat`
//!   exit 1 with "blob matches multiple format heuristics" template.

use super::bitcoin_core::BitcoinCoreParser;
use super::bsms::BsmsParser;
use super::coldcard::ColdcardParser;
use super::coldcard_multisig::ColdcardMultisigParser;
use super::electrum::ElectrumParser;
use super::jade::JadeParser;
use super::sparrow::SparrowParser;
use super::specter::SpecterParser;
use super::WalletFormatParser;

/// SPEC §6 — sniff verdict. Names mirror SPEC §2.1 `--format` values where
/// possible. Variants ordered alphabetically; the per-parser variants
/// `Coldcard / ColdcardMultisig / Electrum / Jade / Sparrow / Specter` are
/// placeholder until their corresponding per-parser P{N}A sub-phase wires
/// the parser into `sniff_format`. `Ambiguous` and `NoMatch` are aggregate
/// outcomes (not parser-backed).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SniffOutcome {
    Ambiguous,
    BitcoinCore,
    Bsms,
    Coldcard,
    ColdcardMultisig,
    Electrum,
    Jade,
    NoMatch,
    Sparrow,
    Specter,
}

/// SPEC §6 — consult every parser's `sniff` and disambiguate via
/// consult-all-then-count (R3-C2 + R4-I1 fold; v0.28.0 generalization of
/// v0.26.0's 2-bool 2×2 truth-table dispatch).
///
/// Order in the `votes` array is alphabetical by SniffOutcome PARSER variant
/// (`Ambiguous`/`NoMatch` excluded — they are not parsers). Per-parser
/// P{N}A sub-phases flip ONE placeholder `false` bool to the corresponding
/// `XParser::sniff(blob)` call; the wiring slot is fixed by alphabetical
/// position so no merge conflict accumulates across parallel branches.
pub(crate) fn sniff_format(blob: &[u8]) -> SniffOutcome {
    let bitcoin_core = BitcoinCoreParser::sniff(blob);
    let bsms = BsmsParser::sniff(blob);
    let coldcard = ColdcardParser::sniff(blob);
    let coldcard_multisig = ColdcardMultisigParser::sniff(blob);
    let electrum = ElectrumParser::sniff(blob);
    let jade = JadeParser::sniff(blob);
    let sparrow = SparrowParser::sniff(blob);
    let specter = SpecterParser::sniff(blob);

    let votes: [(bool, SniffOutcome); 8] = [
        (bitcoin_core, SniffOutcome::BitcoinCore),
        (bsms, SniffOutcome::Bsms),
        (coldcard, SniffOutcome::Coldcard),
        (coldcard_multisig, SniffOutcome::ColdcardMultisig),
        (electrum, SniffOutcome::Electrum),
        (jade, SniffOutcome::Jade),
        (sparrow, SniffOutcome::Sparrow),
        (specter, SniffOutcome::Specter),
    ];

    let matched: Vec<SniffOutcome> = votes
        .iter()
        .filter(|(b, _)| *b)
        .map(|(_, v)| *v)
        .collect();
    match matched.len() {
        0 => SniffOutcome::NoMatch,
        1 => matched[0],
        _ => SniffOutcome::Ambiguous,
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

    /// v0.28.0 P0D R3-C2 + R4-C1 fold — pin the consult-all-then-count
    /// dispatch directly via synthetic 8-bool tuples (NOT exhaustive 2^8=256
    /// enumeration — equivalence-class coverage per SPEC §6.3.1).
    ///
    /// v0.27.1 Phase 4 I16 fold's exhaustive-2x2 documentation generalizes
    /// to equivalence-class coverage under N-parser consult-all-then-count:
    /// (a) 0-true → NoMatch, (b) exactly-1-true → that parser's variant (8
    /// representatives, one per parser-position), (c) ≥2-true → Ambiguous.
    /// The 2^8=256 raw truth-table rows collapse to these 3 equivalence
    /// classes by construction of the dispatch.
    ///
    /// This synthetic test does NOT depend on any parser's real `sniff`
    /// impl — it exercises the dispatch arithmetic directly via 8-tuples
    /// matching the `votes` array shape at `sniff_format`. The 8 variant
    /// positions correspond to the alphabetical parser order: BitcoinCore,
    /// Bsms, Coldcard, ColdcardMultisig, Electrum, Jade, Sparrow, Specter.
    #[test]
    fn sniff_format_dispatches_consult_all_then_count() {
        // Mirror the dispatch arithmetic in test-local form so we exercise
        // the consult-all-then-count semantic on synthetic inputs (the
        // function-under-test would force all-false for the 6 non-yet-wired
        // parsers; this local helper lets us cover the (b) and (c) classes
        // for those positions too, anticipating per-parser P{N}A wirings).
        fn dispatch(bools: [bool; 8]) -> SniffOutcome {
            let votes: [(bool, SniffOutcome); 8] = [
                (bools[0], SniffOutcome::BitcoinCore),
                (bools[1], SniffOutcome::Bsms),
                (bools[2], SniffOutcome::Coldcard),
                (bools[3], SniffOutcome::ColdcardMultisig),
                (bools[4], SniffOutcome::Electrum),
                (bools[5], SniffOutcome::Jade),
                (bools[6], SniffOutcome::Sparrow),
                (bools[7], SniffOutcome::Specter),
            ];
            let matched: Vec<SniffOutcome> = votes
                .iter()
                .filter(|(b, _)| *b)
                .map(|(_, v)| *v)
                .collect();
            match matched.len() {
                0 => SniffOutcome::NoMatch,
                1 => matched[0],
                _ => SniffOutcome::Ambiguous,
            }
        }

        // (a) 0-true equivalence class → NoMatch (single representative).
        assert_eq!(
            dispatch([false, false, false, false, false, false, false, false]),
            SniffOutcome::NoMatch,
            "0-true class must dispatch NoMatch (SPEC §6.2)"
        );

        // (b) exactly-1-true equivalence class → that parser's variant.
        // One representative per parser-position (8 total).
        let exactly_one_true_cases: [([bool; 8], SniffOutcome); 8] = [
            (
                [true, false, false, false, false, false, false, false],
                SniffOutcome::BitcoinCore,
            ),
            (
                [false, true, false, false, false, false, false, false],
                SniffOutcome::Bsms,
            ),
            (
                [false, false, true, false, false, false, false, false],
                SniffOutcome::Coldcard,
            ),
            (
                [false, false, false, true, false, false, false, false],
                SniffOutcome::ColdcardMultisig,
            ),
            (
                [false, false, false, false, true, false, false, false],
                SniffOutcome::Electrum,
            ),
            (
                [false, false, false, false, false, true, false, false],
                SniffOutcome::Jade,
            ),
            (
                [false, false, false, false, false, false, true, false],
                SniffOutcome::Sparrow,
            ),
            (
                [false, false, false, false, false, false, false, true],
                SniffOutcome::Specter,
            ),
        ];
        for (bools, expected) in exactly_one_true_cases {
            assert_eq!(
                dispatch(bools),
                expected,
                "exactly-1-true class at position with {expected:?} must dispatch that variant (SPEC §6.2)"
            );
        }

        // (c) ≥2-true equivalence class → Ambiguous. Sample 3 representatives
        // (2-true adjacent, 2-true non-adjacent, all-true) to cover the
        // class without enumerating all 2^8 - 9 = 247 members.
        assert_eq!(
            dispatch([true, true, false, false, false, false, false, false]),
            SniffOutcome::Ambiguous,
            "2-true (adjacent) must dispatch Ambiguous (SPEC §6.2)"
        );
        assert_eq!(
            dispatch([true, false, false, false, false, false, false, true]),
            SniffOutcome::Ambiguous,
            "2-true (non-adjacent endpoints) must dispatch Ambiguous (SPEC §6.2)"
        );
        assert_eq!(
            dispatch([true, true, true, true, true, true, true, true]),
            SniffOutcome::Ambiguous,
            "8-true (all-match) must dispatch Ambiguous (SPEC §6.2)"
        );
    }
}
