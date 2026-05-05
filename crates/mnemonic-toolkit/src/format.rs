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
/// Reserved: mk1 currently uses `chunk_5char` directly; mk-specific helper retained
/// for the eventual mk-codec chunked-form swap.
#[allow(dead_code)]
pub fn chunk_mk1(s: &str) -> String {
    chunk_5char(s)
}

/// Render an `md1` string in md-codec's `render_codex32_grouped(s, 5)` form.
pub fn chunk_md1(s: &str) -> String {
    md_codec::encode::render_codex32_grouped(s, 5)
}

/// Discriminated union for `BundleJson.mk1` (SPEC §5.3 v0.2 + Q9 closure).
///
/// - `Single`: flat `Vec<String>` for single-sig invocations (matches v0.1 shape).
/// - `Multi`: nested `Vec<Vec<String>>` for multisig (outer = per-cosigner).
///
/// `#[serde(untagged)]` makes the JSON output a bare array (or array-of-arrays)
/// — no `Single`/`Multi` discriminator wrapper. Consumers branch on
/// `BundleJson.multisig` (None → flat, Some → nested) before deserializing.
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum MkField {
    Single(Vec<String>),
    /// Reserved for Phase C multisig synthesis — emitted when
    /// `BundleJson.multisig` is `Some(_)`.
    #[allow(dead_code)]
    Multi(Vec<Vec<String>>),
}

/// Per-cosigner descriptor entry for `MultisigInfo.cosigners` (SPEC §5.3 v0.2).
#[derive(Debug, Serialize)]
pub struct CosignerEntry {
    pub index: usize,
    /// `None` when `--privacy-preserving` (mk1 omits origin_fingerprint).
    pub master_fingerprint: Option<String>,
    pub origin_path: String,
    pub xpub: String,
}

/// Multisig metadata block emitted into `BundleJson.multisig` (SPEC §5.3 v0.2).
#[derive(Debug, Serialize)]
pub struct MultisigInfo {
    pub template: &'static str,
    pub threshold: u8,
    pub cosigner_count: usize,
    /// `"bip48"` | `"bip87"`.
    pub path_family: &'static str,
    pub cosigners: Vec<CosignerEntry>,
}

/// Bundle JSON output schema (SPEC §5.3). Field order is part of the schema.
/// v0.2: schema_version "2"; ownership of mk1 moved from borrowed slice to
/// owned `MkField` to support the discriminated-union shape.
#[derive(Debug, Serialize)]
pub struct BundleJson {
    pub schema_version: &'static str,
    pub mode: &'static str, // "full" | "watch-only"
    pub network: &'static str,
    pub template: &'static str,
    pub account: u32,
    pub origin_path: String,
    pub master_fingerprint: String,
    pub ms1: Option<String>, // null in watch-only
    pub mk1: MkField,
    pub md1: Vec<String>,
    pub engraving_card: Option<String>,
    pub multisig: Option<MultisigInfo>,
    pub privacy_preserving: bool,
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

/// Compose the engraving-card stderr text (SPEC §5.2). Pinned byte-exact for
/// single-sig modes; multisig modes are scaffolded in Phase B with TODOs for
/// Phase C byte-exact pinning per SPEC §5.2 multisig stanzas.
pub fn engraving_card(
    network: &str,
    template: &str,
    origin_path: &str,
    master_fingerprint: &str,
    account: u32,
    mode: EngravingMode<'_>,
) -> String {
    let mut s = String::new();
    s.push_str(&format!("network: {}\n", network));
    s.push_str(&format!("template: {}\n", template));
    s.push_str(&format!("account: {}\n", account));
    match &mode {
        EngravingMode::FullMultisig { multisig_info, .. }
        | EngravingMode::WatchOnlyMultisig { multisig_info, .. } => {
            // TODO Phase C: pin byte-exact text per SPEC §5.2 multisig stanzas
            // (`threshold: K of N`, `cosigner_count: N`, `multisig_path_family`,
            // origin-paths block, SELF-MULTISIG WARNING, HARDWARE WALLET CAVEAT).
            s.push_str(&format!(
                "threshold: {} of {}\n",
                multisig_info.threshold, multisig_info.cosigner_count,
            ));
            s.push_str(&format!(
                "cosigner_count: {}\n",
                multisig_info.cosigner_count
            ));
            s.push_str(&format!(
                "multisig_path_family: {}\n",
                multisig_info.path_family
            ));
        }
        _ => {
            s.push_str(&format!("origin path: {}\n", origin_path));
            s.push_str(&format!("master fingerprint: {}\n", master_fingerprint));
        }
    }
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
        EngravingMode::FullMultisig { language, .. } => {
            // TODO Phase C: pin byte-exact text per SPEC §5.2 full-multisig stanza.
            s.push_str(&format!("language: {} (BIP-39 checksum valid)\n", language));
        }
        EngravingMode::WatchOnlyMultisig { .. } => {
            // TODO Phase C: pin byte-exact text per SPEC §5.2 watch-only-multisig stanza.
            s.push_str(
                "mode: watch-only multisig (xpub-supplied per cosigner; no entropy known to toolkit)\n",
            );
            s.push_str(
                "ms1 card omitted; recover entropy from each cosigner's individual seed backup.\n",
            );
        }
    }
    s.push_str("engrave each card on its own plate. record this card alongside.\n");
    s
}

pub enum EngravingMode<'a> {
    FullNoPassphrase {
        language: &'a str,
    },
    FullWithPassphrase {
        language: &'a str,
    },
    WatchOnly,
    /// Phase B scaffolding — Phase C will pin byte-exact text per SPEC §5.2.
    #[allow(dead_code)]
    FullMultisig {
        language: &'a str,
        multisig_info: &'a MultisigInfo,
        account: u32,
        paths_shared: bool,
    },
    /// Phase B scaffolding — Phase C will pin byte-exact text per SPEC §5.2.
    #[allow(dead_code)]
    WatchOnlyMultisig {
        multisig_info: &'a MultisigInfo,
        account: u32,
        paths_shared: bool,
    },
}

/// Extract a chunk_set_id from an mk1 chunked-header string per SPEC §2.2.1
/// step 1. Returns `None` for SingleString-headered strings or decode failures.
///
/// Used by verify-bundle multisig grouping: cosigners' mk1 chunks are grouped
/// by `chunk_set_id` to recover per-cosigner card sets from a flat input list.
#[allow(dead_code)]
pub fn chunk_set_id_extract(s: &str) -> Option<u32> {
    use mk_codec::string_layer::{decode_string, StringLayerHeader};
    let decoded = decode_string(s).ok()?;
    let (header, _consumed) = StringLayerHeader::from_5bit_symbols(decoded.data()).ok()?;
    match header {
        StringLayerHeader::Chunked { chunk_set_id, .. } => Some(chunk_set_id),
        StringLayerHeader::SingleString { .. } => None,
        // StringLayerHeader is #[non_exhaustive]; future variants → None.
        _ => None,
    }
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
            0,
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
            0,
            EngravingMode::FullWithPassphrase {
                language: "english",
            },
        );
        assert!(card.contains(
            "passphrase: USED — not engraved on any card; record separately and never lose it.\n"
        ));
    }

    /// Phase B.3 unit test (resolves I-1 from PLAN r1 review): MkField::Single
    /// serializes byte-identically to v0.1's flat `Vec<String>` shape via
    /// #[serde(untagged)] — no Single discriminator wrapper in the JSON output.
    #[test]
    fn mk_field_single_serde_byte_identical_to_v0_1() {
        let mk = MkField::Single(vec!["mk1qfoo".to_string()]);
        let json = serde_json::to_string(&mk).unwrap();
        assert_eq!(json, "[\"mk1qfoo\"]");
    }

    #[test]
    fn mk_field_multi_serializes_as_nested_array() {
        let mk = MkField::Multi(vec![
            vec!["mk1qa".to_string()],
            vec!["mk1qb".to_string(), "mk1qc".to_string()],
        ]);
        let json = serde_json::to_string(&mk).unwrap();
        assert_eq!(json, "[[\"mk1qa\"],[\"mk1qb\",\"mk1qc\"]]");
    }

    #[test]
    fn chunk_set_id_extract_returns_none_for_garbage() {
        assert_eq!(chunk_set_id_extract("not-an-mk1-string"), None);
        assert_eq!(chunk_set_id_extract(""), None);
    }

    #[test]
    fn engraving_card_watch_only_omits_ms1() {
        let card = engraving_card(
            "mainnet",
            "bip84",
            "m/84'/0'/0'",
            "deadbeef",
            0,
            EngravingMode::WatchOnly,
        );
        assert!(card.contains("mode: watch-only"));
        assert!(card.contains("ms1 card omitted"));
        assert!(!card.contains("language:"));
        assert!(!card.contains("passphrase:"));
    }
}
