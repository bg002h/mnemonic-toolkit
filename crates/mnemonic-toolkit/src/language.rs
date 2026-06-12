//! `--language` clap enum + From<bip39::Language>.
//!
//! Realizes SPEC §1 (10 BIP-39 wordlists supported) + SPEC §5.2 stderr
//! language-defaulting warning. Mirrors ms-cli `language.rs`.

use crate::error::ToolkitError;
use clap::ValueEnum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Default)]
#[clap(rename_all = "lower")]
pub enum CliLanguage {
    #[default]
    English,
    SimplifiedChinese,
    TraditionalChinese,
    Czech,
    French,
    Italian,
    Japanese,
    Korean,
    Portuguese,
    Spanish,
}

impl CliLanguage {
    /// Human-readable name for stderr warnings.
    pub fn human_name(&self) -> &'static str {
        match self {
            CliLanguage::English => "english",
            CliLanguage::SimplifiedChinese => "simplified-chinese",
            CliLanguage::TraditionalChinese => "traditional-chinese",
            CliLanguage::Czech => "czech",
            CliLanguage::French => "french",
            CliLanguage::Italian => "italian",
            CliLanguage::Japanese => "japanese",
            CliLanguage::Korean => "korean",
            CliLanguage::Portuguese => "portuguese",
            CliLanguage::Spanish => "spanish",
        }
    }
}

/// Map a `CliLanguage` variant to the wire language byte used in
/// `ms_codec::Payload::Mnem { language, .. }`.
///
/// Wire order (`ms_codec::MNEM_LANGUAGE_NAMES`):
/// 0=english, 1=japanese, 2=korean, 3=spanish, 4=chinese-simplified,
/// 5=chinese-traditional, 6=french, 7=italian, 8=czech, 9=portuguese.
///
/// The toolkit `CliLanguage` declaration order is DIFFERENT (English,
/// SimplifiedChinese, TraditionalChinese, Czech, French, Italian, Japanese,
/// Korean, Portuguese, Spanish), so `as u8` is WRONG — use this explicit table.
// Used by Step 5 (emit sites); Step 4 (derive sites) uses wire_code_to_bip39.
#[allow(dead_code)]
pub fn cli_language_to_wire_code(l: CliLanguage) -> u8 {
    match l {
        CliLanguage::English => 0,
        CliLanguage::Japanese => 1,
        CliLanguage::Korean => 2,
        CliLanguage::Spanish => 3,
        CliLanguage::SimplifiedChinese => 4,
        CliLanguage::TraditionalChinese => 5,
        CliLanguage::French => 6,
        CliLanguage::Italian => 7,
        CliLanguage::Czech => 8,
        CliLanguage::Portuguese => 9,
    }
}

/// Map a wire language byte back to the corresponding `CliLanguage` variant.
/// Returns `None` for codes ≥ 10.
///
/// Used by `cmd/ms_shares.rs::run_combine` to key the I1 `--to entropy`
/// language-loss advisory off the recovered mnem payload's wire language.
pub fn wire_code_to_cli(c: u8) -> Option<CliLanguage> {
    match c {
        0 => Some(CliLanguage::English),
        1 => Some(CliLanguage::Japanese),
        2 => Some(CliLanguage::Korean),
        3 => Some(CliLanguage::Spanish),
        4 => Some(CliLanguage::SimplifiedChinese),
        5 => Some(CliLanguage::TraditionalChinese),
        6 => Some(CliLanguage::French),
        7 => Some(CliLanguage::Italian),
        8 => Some(CliLanguage::Czech),
        9 => Some(CliLanguage::Portuguese),
        _ => None,
    }
}

/// Map a wire language byte to a `bip39::Language`.
///
/// Returns `ToolkitError::BadInput` for codes ≥ 10. `ms_codec::discriminate`
/// already validates `language < 10`, so this path is defensive — never
/// use `unwrap` on a raw byte.
pub fn wire_code_to_bip39(c: u8) -> Result<bip39::Language, ToolkitError> {
    match c {
        0 => Ok(bip39::Language::English),
        1 => Ok(bip39::Language::Japanese),
        2 => Ok(bip39::Language::Korean),
        3 => Ok(bip39::Language::Spanish),
        4 => Ok(bip39::Language::SimplifiedChinese),
        5 => Ok(bip39::Language::TraditionalChinese),
        6 => Ok(bip39::Language::French),
        7 => Ok(bip39::Language::Italian),
        8 => Ok(bip39::Language::Czech),
        9 => Ok(bip39::Language::Portuguese),
        _ => Err(ToolkitError::BadInput(format!(
            "unknown mnem wordlist-language wire code: {c}"
        ))),
    }
}

/// Map a `bip39::Language` to the wire language byte used in
/// `ms_codec::Payload::Mnem { language, .. }`.
///
/// This is the inverse of `wire_code_to_bip39`; keyed on variant, not index.
/// Used by emit sites (Step 5) where the source language is already a
/// `bip39::Language` (e.g. from `slot.language`).
pub fn bip39_to_wire_code(l: bip39::Language) -> u8 {
    match l {
        bip39::Language::English => 0,
        bip39::Language::Japanese => 1,
        bip39::Language::Korean => 2,
        bip39::Language::Spanish => 3,
        bip39::Language::SimplifiedChinese => 4,
        bip39::Language::TraditionalChinese => 5,
        bip39::Language::French => 6,
        bip39::Language::Italian => 7,
        bip39::Language::Czech => 8,
        bip39::Language::Portuguese => 9,
    }
}

/// Resolve the `bip39::Language` for a single decoded ms1 card's derivation.
///
/// For a `mnem` card the wire language wins — the card itself declares the
/// BIP-39 wordlist used to encode the entropy. For an `entr` card (legacy
/// language-agnostic) the CLI `--language` / English default is used.
///
/// Call this with EACH card's payload (not a run-level single value) inside
/// per-cosigner loops.
pub fn payload_bip39_language(
    payload: &ms_codec::Payload,
    cli: CliLanguage,
) -> Result<bip39::Language, ToolkitError> {
    match payload {
        ms_codec::Payload::Mnem { language, .. } => wire_code_to_bip39(*language),
        _ => Ok(cli.into()),
    }
}

impl From<CliLanguage> for bip39::Language {
    fn from(l: CliLanguage) -> bip39::Language {
        match l {
            CliLanguage::English => bip39::Language::English,
            CliLanguage::SimplifiedChinese => bip39::Language::SimplifiedChinese,
            CliLanguage::TraditionalChinese => bip39::Language::TraditionalChinese,
            CliLanguage::Czech => bip39::Language::Czech,
            CliLanguage::French => bip39::Language::French,
            CliLanguage::Italian => bip39::Language::Italian,
            CliLanguage::Japanese => bip39::Language::Japanese,
            CliLanguage::Korean => bip39::Language::Korean,
            CliLanguage::Portuguese => bip39::Language::Portuguese,
            CliLanguage::Spanish => bip39::Language::Spanish,
        }
    }
}

/// Returns a stderr advisory iff `lang` is a non-English BIP-39 wordlist (the
/// language is load-bearing for the seed but is NOT carried by `form`). `form`
/// names the language-dropping output ("an ms1 card", "raw entropy",
/// "SLIP-39 shares"). English → None (English self-recovers as the universal
/// default). v0.37.11 — path A of the `mnem` footgun; see
/// `design/SPEC_non_english_seed_advisory.md`.
pub(crate) fn non_english_seed_advisory(lang: CliLanguage, form: &str) -> Option<String> {
    if lang == CliLanguage::English {
        return None;
    }
    let name = lang.human_name();
    Some(format!(
        "warning: encoding a {name} BIP-39 seed as {form} — it carries only the \
         entropy, not the wordlist language. Record \"{name}\" alongside the backup: \
         recovering the entropy with English-defaulted software derives a DIFFERENT \
         seed and a DIFFERENT wallet."
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Step 5 tests ─────────────────────────────────────────────────────────

    /// bip39_to_wire_code(wire_code_to_bip39(c)) == c for all 10 codes.
    #[test]
    fn bip39_to_wire_code_round_trip_all_10() {
        for c in 0u8..10 {
            let lang = wire_code_to_bip39(c).unwrap();
            let back = bip39_to_wire_code(lang);
            assert_eq!(
                back, c,
                "code {c}: bip39_to_wire_code(wire_code_to_bip39({c})) = {back}"
            );
        }
    }

    // ── Step 3 tests ─────────────────────────────────────────────────────────

    /// (a) for all 10 codes: wire_code_to_bip39(c) == bip39::Language::from(wire_code_to_cli(c).unwrap())
    #[test]
    fn wire_code_round_trip_bip39_identity() {
        for c in 0u8..10 {
            let via_bip39 = wire_code_to_bip39(c).unwrap();
            let via_cli: bip39::Language = wire_code_to_cli(c).unwrap().into();
            assert_eq!(
                via_bip39, via_cli,
                "code {c}: wire_code_to_bip39 and via CliLanguage diverge"
            );
        }
    }

    /// (b) cli_language_to_wire_code ∘ wire_code_to_cli round-trips all 10
    #[test]
    fn cli_wire_code_round_trip_all_10() {
        for c in 0u8..10 {
            let cli = wire_code_to_cli(c).unwrap();
            let back = cli_language_to_wire_code(cli);
            assert_eq!(
                back, c,
                "code {c}: round-trip failed via CliLanguage::{cli:?}"
            );
        }
    }

    /// (c) pin MNEM_LANGUAGE_NAMES[c] to expected literal for all 10 codes
    #[test]
    fn mnem_language_names_label_pin() {
        let expected = [
            "english",
            "japanese",
            "korean",
            "spanish",
            "chinese-simplified",
            "chinese-traditional",
            "french",
            "italian",
            "czech",
            "portuguese",
        ];
        for (c, &name) in expected.iter().enumerate() {
            assert_eq!(
                ms_codec::consts::MNEM_LANGUAGE_NAMES[c],
                name,
                "MNEM_LANGUAGE_NAMES[{c}] drifted"
            );
        }
    }

    /// (d) anchors: wire_code_to_bip39(1)==Japanese, (4)==SimplifiedChinese, (5)==TraditionalChinese
    #[test]
    fn wire_code_anchor_chinese_japanese() {
        assert_eq!(
            wire_code_to_bip39(1).unwrap(),
            bip39::Language::Japanese,
            "wire code 1 must be Japanese"
        );
        assert_eq!(
            wire_code_to_bip39(4).unwrap(),
            bip39::Language::SimplifiedChinese,
            "wire code 4 must be SimplifiedChinese"
        );
        assert_eq!(
            wire_code_to_bip39(5).unwrap(),
            bip39::Language::TraditionalChinese,
            "wire code 5 must be TraditionalChinese"
        );
    }

    #[test]
    fn wire_code_to_bip39_rejects_code_10_and_above() {
        assert!(wire_code_to_bip39(10).is_err());
        assert!(wire_code_to_bip39(255).is_err());
    }

    // ── advisory tests ────────────────────────────────────────────────────────

    #[test]
    fn advisory_none_for_english() {
        assert_eq!(
            non_english_seed_advisory(CliLanguage::English, "an ms1 card"),
            None
        );
    }

    #[test]
    fn advisory_some_for_french_with_form() {
        let m = non_english_seed_advisory(CliLanguage::French, "an ms1 card").unwrap();
        assert!(m.contains("french"), "{m}");
        assert!(m.contains("an ms1 card"), "{m}");
        assert!(m.contains("DIFFERENT"), "{m}");
    }

    #[test]
    fn advisory_uses_kebab_name() {
        let m = non_english_seed_advisory(CliLanguage::SimplifiedChinese, "raw entropy").unwrap();
        assert!(m.contains("simplified-chinese"), "{m}");
    }

    #[test]
    fn default_is_english() {
        assert_eq!(CliLanguage::default(), CliLanguage::English);
    }

    #[test]
    fn human_name_lowercase_kebab() {
        assert_eq!(CliLanguage::English.human_name(), "english");
        assert_eq!(
            CliLanguage::SimplifiedChinese.human_name(),
            "simplified-chinese"
        );
    }

    #[test]
    fn maps_to_bip39_language() {
        let _l: bip39::Language = CliLanguage::English.into();
        let _l: bip39::Language = CliLanguage::Japanese.into();
    }
}
