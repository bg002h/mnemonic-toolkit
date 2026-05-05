//! Output formatting: multi-section stdout, engraving-card stderr,
//! JSON envelopes for bundle and verify-bundle.
//!
//! Realizes SPEC §5.1, §5.2, §5.3, §5.4.

use serde::Serialize;

/// Render an `ms1` string in 5-char-grouped chunked form (10 groups/line max).
/// Mirrors ms-cli `format::chunked_form`.
pub fn chunk_5char(s: &str) -> String {
    let mut out = String::new();
    let chars: Vec<char> = s.chars().collect();
    let mut groups: Vec<String> = Vec::new();
    for chunk in chars.chunks(5) {
        groups.push(chunk.iter().collect::<String>());
    }
    for (i, g) in groups.iter().enumerate() {
        if i > 0 && i % 10 == 0 {
            out.push('\n');
        } else if i > 0 {
            out.push(' ');
        }
        out.push_str(g);
    }
    out
}

/// Render an `mk1` string in mk-codec's chunked form. v0.1: defer to mk-codec
/// internal chunked-form when available; fallback to chunk_5char for v0.1.
pub fn chunk_mk1(s: &str) -> String {
    chunk_5char(s)
}

/// Render an `md1` string in md-codec's `render_codex32_grouped(s, 5)` form.
pub fn chunk_md1(s: &str) -> String {
    md_codec::encode::render_codex32_grouped(s, 5)
}

/// Bundle JSON output schema (SPEC §5.3). Field order is part of the schema.
#[derive(Debug, Serialize)]
pub struct BundleJson<'a> {
    pub schema_version: &'static str,
    pub mode: &'static str, // "full" | "watch-only"
    pub network: &'static str,
    pub template: &'static str,
    pub account: u32,
    pub origin_path: String,
    pub master_fingerprint: String,
    pub ms1: Option<&'a str>, // null in watch-only
    pub mk1: &'a [String],
    pub md1: &'a [String],
    pub engraving_card: Option<String>,
}

/// Verify-bundle JSON output schema (SPEC §5.4). Field order is part of the schema.
#[derive(Debug, Serialize)]
pub struct VerifyBundleJson {
    pub schema_version: &'static str,
    pub result: &'static str, // "ok" | "mismatch"
    pub checks: Vec<VerifyCheck>,
}

#[derive(Debug, Serialize)]
pub struct VerifyCheck {
    pub name: &'static str,
    pub result: &'static str, // "ok" | "fail" | "skipped"
    pub detail: String,
}

/// Compose the engraving-card stderr text (SPEC §5.2). Pinned byte-exact.
pub fn engraving_card(
    network: &str,
    template: &str,
    origin_path: &str,
    master_fingerprint: &str,
    mode: EngravingMode<'_>,
) -> String {
    let mut s = String::new();
    s.push_str(&format!("network: {}\n", network));
    s.push_str(&format!("template: {}\n", template));
    s.push_str("account: 0\n");
    s.push_str(&format!("origin path: {}\n", origin_path));
    s.push_str(&format!("master fingerprint: {}\n", master_fingerprint));
    match mode {
        EngravingMode::FullNoPassphrase { language } => {
            s.push_str(&format!("language: {} (BIP-39 checksum valid)\n", language));
            s.push_str("passphrase: not used\n");
        }
        EngravingMode::FullWithPassphrase { language } => {
            s.push_str(&format!("language: {} (BIP-39 checksum valid)\n", language));
            s.push_str("passphrase: USED — not engraved on any card; record separately and never lose it.\n");
        }
        EngravingMode::WatchOnly => {
            s.push_str("mode: watch-only (xpub-supplied; no entropy known to toolkit)\n");
            s.push_str(
                "ms1 card omitted; recover entropy from the original wallet's other backup.\n",
            );
        }
    }
    s.push_str("engrave each card on its own plate. record this card alongside.\n");
    s
}

pub enum EngravingMode<'a> {
    FullNoPassphrase { language: &'a str },
    FullWithPassphrase { language: &'a str },
    WatchOnly,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_5char_groups() {
        let s = "abcdefghij";
        assert_eq!(chunk_5char(s), "abcde fghij");
    }

    #[test]
    fn chunk_5char_remainder() {
        let s = "abcdefg";
        assert_eq!(chunk_5char(s), "abcde fg");
    }

    #[test]
    fn chunk_5char_wraps_at_10_groups() {
        let s: String = "x".repeat(55); // 11 groups of 5
        let out = chunk_5char(&s);
        assert!(out.contains('\n'));
        let first_line = out.lines().next().unwrap();
        let group_count = first_line.split(' ').count();
        assert_eq!(group_count, 10);
    }

    #[test]
    fn engraving_card_full_no_passphrase_byte_exact() {
        let card = engraving_card(
            "mainnet",
            "bip84",
            "m/84'/0'/0'",
            "deadbeef",
            EngravingMode::FullNoPassphrase {
                language: "english",
            },
        );
        let expected = "\
network: mainnet
template: bip84
account: 0
origin path: m/84'/0'/0'
master fingerprint: deadbeef
language: english (BIP-39 checksum valid)
passphrase: not used
engrave each card on its own plate. record this card alongside.
";
        assert_eq!(card, expected);
    }

    #[test]
    #[allow(non_snake_case)]
    fn engraving_card_with_passphrase_uses_uppercase_USED() {
        let card = engraving_card(
            "mainnet",
            "bip84",
            "m/84'/0'/0'",
            "deadbeef",
            EngravingMode::FullWithPassphrase {
                language: "english",
            },
        );
        assert!(card.contains(
            "passphrase: USED — not engraved on any card; record separately and never lose it.\n"
        ));
    }

    #[test]
    fn engraving_card_watch_only_omits_ms1() {
        let card = engraving_card(
            "mainnet",
            "bip84",
            "m/84'/0'/0'",
            "deadbeef",
            EngravingMode::WatchOnly,
        );
        assert!(card.contains("mode: watch-only"));
        assert!(card.contains("ms1 card omitted"));
        assert!(!card.contains("language:"));
        assert!(!card.contains("passphrase:"));
    }
}
