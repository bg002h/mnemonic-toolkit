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

#[cfg(test)]
mod tests {
    use super::*;

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
