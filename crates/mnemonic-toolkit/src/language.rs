//! `--language` clap enum + From<bip39::Language>.
//!
//! Realizes SPEC §1 (10 BIP-39 wordlists supported) + SPEC §5.2 stderr
//! language-defaulting warning. Mirrors ms-cli `language.rs`.

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
        let m =
            non_english_seed_advisory(CliLanguage::SimplifiedChinese, "raw entropy").unwrap();
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
