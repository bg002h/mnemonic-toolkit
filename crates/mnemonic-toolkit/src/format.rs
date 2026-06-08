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
/// The bundle mk1 text-card emit routes through this helper, so the eventual
/// mk-codec chunked-form swap is a single edit to the body below.
pub fn chunk_mk1(s: &str) -> String {
    chunk_5char(s)
}

/// Render an `md1` string in md-codec's `render_codex32_grouped(s, 5)` form.
pub fn chunk_md1(s: &str) -> String {
    md_codec::encode::render_codex32_grouped(s, 5)
}

/// SPEC §5.8 (v0.4) ms1 field type. Schema 4 layout: dense `Vec<String>` of
/// length-N, with empty-string sentinels (`""`) marking watch-only slots.
///
/// - `["ms1abc..."]`               — single-sig full (N=1, secret-bearing)
/// - `["", "", ""]`                — pure watch-only multisig (N=3)
/// - `["ms1...", "ms1...", "..."]` — multi-source full multisig (N=3)
/// - `["ms1...", "", ""]`          — hybrid (N=3, slot 0 secret, others watch-only)
///
/// Verify-bundle skips ms1 checks for empty-string elements. The dense layout
/// preserves slot-index correspondence `ms1[i] ↔ mk1[i] ↔ slot @i`.
///
/// Wired into `BundleJson.ms1` and `Bundle.ms1` in v0.4.1 (Phase H).
pub type MsField = Vec<String>;

/// Discriminated union for `BundleJson.mk1` (SPEC §5.3 v0.2 + Q9 closure).
///
/// - `Single`: flat `Vec<String>` for single-sig invocations (matches v0.1 shape).
/// - `Multi`: nested `Vec<Vec<String>>` for multisig (outer = per-cosigner).
///
/// `#[serde(untagged)]` makes the JSON output a bare array (or array-of-arrays)
/// — no `Single`/`Multi` discriminator wrapper. Consumers branch on
/// `BundleJson.multisig` (None → flat, Some → nested) before deserializing.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum MkField {
    Single(Vec<String>),
    /// Per-cosigner mk1 chunks for multisig synthesis (Phase C).
    Multi(Vec<Vec<String>>),
}

impl MkField {
    /// Read out the single-sig payload. Panics if `Multi`.
    #[allow(dead_code)]
    pub fn as_single(&self) -> Option<&Vec<String>> {
        match self {
            MkField::Single(v) => Some(v),
            MkField::Multi(_) => None,
        }
    }

    /// Read out the multisig per-cosigner payload. Panics if `Single`.
    #[allow(dead_code)]
    pub fn as_multi(&self) -> Option<&Vec<Vec<String>>> {
        match self {
            MkField::Multi(v) => Some(v),
            MkField::Single(_) => None,
        }
    }
}

/// Per-cosigner descriptor entry for `MultisigInfo.cosigners` (SPEC §5.3 v0.2).
#[derive(Debug, Clone, Serialize)]
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
/// owned `MkField` to support the discriminated-union shape. `origin_path`
/// (single-sig OR shared-path multisig) and `origin_paths` (divergent-path
/// multisig) are mutually exclusive per SPEC §5.3. `master_fingerprint` is
/// `null` for multisig OR `--privacy-preserving`.
#[derive(Debug, Serialize)]
pub struct BundleJson {
    pub schema_version: &'static str,
    pub mode: &'static str, // "full" | "watch-only"
    pub network: &'static str,
    /// `None` in descriptor mode (v0.3 §5.6); always `Some` in template mode.
    pub template: Option<&'static str>,
    /// User-supplied descriptor verbatim; `Some` in descriptor mode, `None`
    /// in template mode (v0.3 §5.6).
    pub descriptor: Option<String>,
    pub account: u32,
    /// Single-sig OR shared-path multisig. `None` for divergent-path multisig.
    pub origin_path: Option<String>,
    /// Divergent-path multisig. `None` otherwise.
    pub origin_paths: Option<Vec<String>>,
    /// `None` for multisig OR `--privacy-preserving`.
    pub master_fingerprint: Option<String>,
    /// SPEC §5.8 schema-4 ms1 field. Length-N invariant; `""` empty-string
    /// sentinel marks watch-only slots; non-empty marks secret-bearing slots.
    /// Single-sig watch-only: `[""]`; pure watch-only multisig N=3: `["", "", ""]`;
    /// multi-source full multisig N=3: `["ms1...", "ms1...", "ms1..."]`.
    pub ms1: MsField,
    pub mk1: MkField,
    pub md1: Vec<String>,
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

/// SPEC §5.7 verify-bundle check entry. v0.4.3 Phase P.0: shape corrected
/// from `result: &'static str` (v0.4.1 J.1 long-standing drift) to
/// `passed: bool` per SPEC §5.7. Skipped checks use
/// `passed: true + decode_error: Some("skipped: <reason>")` per SPEC's
/// hybrid-mode treatment.
///
/// Forensic fields (gained at v0.4.1 Phase J.1) are populated on
/// `passed: false` checks. `#[serde(skip_serializing_if = "Option::is_none")]`
/// keeps the JSON envelope clean — forensic fields are omitted entirely
/// for passing checks.
#[derive(Debug, Clone, Serialize)]
pub struct VerifyCheck {
    pub name: String,
    pub passed: bool,
    pub detail: String,
    /// Expected encoded string (for string-mismatch checks); omitted otherwise.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected: Option<String>,
    /// Actual encoded string (for string-mismatch checks); omitted otherwise.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual: Option<String>,
    /// First UTF-8 byte position where expected and actual differ.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff_byte_offset: Option<usize>,
    /// Decode-error message text for decode-failure checks.
    /// Also used for skipped checks: `Some("skipped: <reason>")`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decode_error: Option<String>,
}

impl Default for VerifyCheck {
    fn default() -> Self {
        Self {
            name: String::new(),
            passed: true,
            detail: String::new(),
            expected: None,
            actual: None,
            diff_byte_offset: None,
            decode_error: None,
        }
    }
}

impl VerifyCheck {
    /// First UTF-8 byte position where `a` and `b` differ; `min(len_a, len_b)`
    /// for length-mismatch (one is a prefix of the other).
    pub fn diff_offset(a: &str, b: &str) -> usize {
        a.bytes().zip(b.bytes()).take_while(|(x, y)| x == y).count()
    }
}

/// Returns true for taproot-multisig templates that need the HARDWARE WALLET CAVEAT.
fn is_tr_multisig(template: &str) -> bool {
    matches!(template, "tr-multi-a" | "tr-sortedmulti-a")
}

// ============================================================================
// v0.4.1 Phase I — unified engraving card (SPEC §5.5).
// ============================================================================

/// SPEC §5.5 unified engraving card input — single shape carrying header +
/// per-slot blocks + template-or-descriptor + md1 reference.
///
/// Sole engraving-card surface in v0.5+ (legacy per-mode emission removed
/// in v0.4.2; the dead `BundleJson.engraving_card: Option<String>` field
/// removed in v0.5.0 Phase A.3 — engraving cards are stderr-only).
#[derive(Debug, Clone)]
pub struct BundleInputForCard {
    pub network: &'static str,
    pub template_or_descriptor: TemplateOrDescriptor,
    pub threshold: Option<u8>,
    pub n: u8,
    pub language: Option<&'static str>,
    pub passphrase_used: bool,
    #[allow(dead_code)]
    pub privacy_preserving: bool,
    pub per_slot: Vec<SlotCardBlock>,
    pub md1_chunk_set_id: String,
}

#[derive(Debug, Clone)]
pub enum TemplateOrDescriptor {
    Template(&'static str),
    #[allow(dead_code)]
    Descriptor(String),
}

#[derive(Debug, Clone)]
pub struct SlotCardBlock {
    pub index: u8,
    /// 4-hex chunk_set_id derived from policy_id_stub (None for watch-only).
    pub ms1_card_id: Option<String>,
    /// 4-hex chunk_set_id for mk1.
    pub mk1_card_id: String,
    /// Master fingerprint (None under privacy_preserving).
    pub fingerprint: Option<String>,
    /// Origin derivation path (None if absent / wif slot).
    pub origin_path: Option<String>,
}

/// SPEC §5.5 unified card layout: header / threshold / cosigners block /
/// template-or-descriptor / md1 reference / recovery hint.
///
/// Truncation policy (SPEC §5.5): descriptor strings > 80 chars render as
/// `<first 60 chars>... [md1: <chunk-set-id>] (<descriptor_len> chars total)`.
pub fn engraving_card_unified(input: &BundleInputForCard) -> String {
    const DESCRIPTOR_MAX_INLINE: usize = 80;
    const DESCRIPTOR_TRUNC_PREFIX: usize = 60;

    let mut s = String::new();

    // 1. Header line.
    let summary = match &input.template_or_descriptor {
        TemplateOrDescriptor::Template(t) => (*t).to_string(),
        TemplateOrDescriptor::Descriptor(d) => {
            if d.len() <= DESCRIPTOR_MAX_INLINE {
                d.clone()
            } else {
                format!("descriptor[{}..]", &d[..DESCRIPTOR_TRUNC_PREFIX.min(d.len())])
            }
        }
    };
    s.push_str(&format!(
        "# === Wallet bundle: {}, {} ===\n",
        summary, input.network
    ));

    // 2. Threshold line (multisig only).
    if let Some(t) = input.threshold {
        if input.n > 1 {
            s.push_str(&format!("# Threshold: {} of {}\n", t, input.n));
        }
    }

    // 3. Cosigners block.
    if input.n > 1 {
        s.push_str("# Cosigners:\n");
        for blk in &input.per_slot {
            let ms1_part = match &blk.ms1_card_id {
                Some(id) => format!("ms1:{},", id),
                None => "(no ms1; watch-only),".to_string(),
            };
            let mk1_part = format!("mk1:{}", blk.mk1_card_id);
            let fp_part = match &blk.fingerprint {
                Some(fp) => fp.clone(),
                None => "anon".to_string(),
            };
            let path_part = blk
                .origin_path
                .clone()
                .unwrap_or_else(|| "(no path)".to_string());
            s.push_str(&format!(
                "#   @{}: {}{} ({} @ {})\n",
                blk.index, ms1_part, mk1_part, fp_part, path_part
            ));
        }
    } else if let Some(blk) = input.per_slot.first() {
        // Single-sig: emit one slot block without the "Cosigners:" header.
        let ms1_part = match &blk.ms1_card_id {
            Some(id) => format!("# ms1: {}\n", id),
            None => "# ms1: (omitted; watch-only)\n".to_string(),
        };
        s.push_str(&ms1_part);
        s.push_str(&format!("# mk1: {}\n", blk.mk1_card_id));
        if let Some(fp) = &blk.fingerprint {
            s.push_str(&format!("# fingerprint: {}\n", fp));
        }
        if let Some(p) = &blk.origin_path {
            s.push_str(&format!("# origin path: {}\n", p));
        }
    }

    // 4. Template OR descriptor line.
    match &input.template_or_descriptor {
        TemplateOrDescriptor::Template(t) => {
            s.push_str(&format!("# Template: {}\n", t));
        }
        TemplateOrDescriptor::Descriptor(d) => {
            if d.len() <= DESCRIPTOR_MAX_INLINE {
                s.push_str(&format!("# Descriptor: {}\n", d));
            } else {
                s.push_str(&format!(
                    "# Descriptor: {}... [md1: {}] ({} chars total)\n",
                    &d[..DESCRIPTOR_TRUNC_PREFIX.min(d.len())],
                    input.md1_chunk_set_id,
                    d.len()
                ));
            }
        }
    }

    // 5. md1 reference line.
    s.push_str(&format!("# md1: {}\n", input.md1_chunk_set_id));

    // 6. Recovery hint line.
    if let Some(t) = input.threshold {
        if input.n > 1 {
            s.push_str(&format!(
                "# Recovery: any {} of {} signing keys + md1 (template card).\n",
                t, input.n
            ));
        }
    }

    // 7. Language / passphrase footer.
    if let Some(l) = input.language {
        s.push_str(&format!("# Language: {}\n", l));
    }
    if input.passphrase_used {
        s.push_str("# Passphrase: USED — not engraved on any card; record separately.\n");
    }

    // 8. Hardware wallet caveat for tr-multisig templates (SPEC §5.5).
    if let TemplateOrDescriptor::Template(t) = &input.template_or_descriptor {
        if is_tr_multisig(t) {
            s.push_str(
                "# HARDWARE WALLET CAVEAT: taproot multisig (multi_a / sortedmulti_a) signing-side\n#   support is nascent as of v0.4; verify your signing device supports it before engraving.\n",
            );
        }
    }

    s
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

    // ---- v0.4.1 Phase I — engraving_card_unified shape tests (SPEC §5.5) ----

    fn slot_block(idx: u8, secret: bool, fp: Option<&str>, path: Option<&str>) -> SlotCardBlock {
        SlotCardBlock {
            index: idx,
            ms1_card_id: if secret { Some("abcde".into()) } else { None },
            mk1_card_id: "abcde".into(),
            fingerprint: fp.map(String::from),
            origin_path: path.map(String::from),
        }
    }

    #[test]
    fn unified_card_single_sig_full_includes_template_and_md1() {
        let input = BundleInputForCard {
            network: "mainnet",
            template_or_descriptor: TemplateOrDescriptor::Template("bip84"),
            threshold: None,
            n: 1,
            language: Some("english"),
            passphrase_used: false,
            privacy_preserving: false,
            per_slot: vec![slot_block(0, true, Some("5436d724"), Some("m/84'/0'/0'"))],
            md1_chunk_set_id: "1234".into(),
        };
        let card = engraving_card_unified(&input);
        assert!(card.contains("# === Wallet bundle: bip84, mainnet ===\n"));
        assert!(card.contains("# ms1: abcde\n"));
        assert!(card.contains("# mk1: abcde\n"));
        assert!(card.contains("# fingerprint: 5436d724\n"));
        assert!(card.contains("# origin path: m/84'/0'/0'\n"));
        assert!(card.contains("# Template: bip84\n"));
        assert!(card.contains("# md1: 1234\n"));
        assert!(card.contains("# Language: english\n"));
        assert!(!card.contains("# Threshold:"));
        assert!(!card.contains("# Cosigners:"));
    }

    #[test]
    fn unified_card_single_sig_watch_only_omits_ms1_id_and_passphrase_footer() {
        let input = BundleInputForCard {
            network: "mainnet",
            template_or_descriptor: TemplateOrDescriptor::Template("bip84"),
            threshold: None,
            n: 1,
            language: None,
            passphrase_used: false,
            privacy_preserving: false,
            per_slot: vec![slot_block(0, false, Some("5436d724"), Some("m/84'/0'/0'"))],
            md1_chunk_set_id: "1234".into(),
        };
        let card = engraving_card_unified(&input);
        assert!(card.contains("# ms1: (omitted; watch-only)\n"));
        assert!(!card.contains("Passphrase:"));
        assert!(!card.contains("Language:"));
    }

    #[test]
    fn unified_card_multisig_2_of_3_includes_threshold_cosigners_recovery() {
        let input = BundleInputForCard {
            network: "mainnet",
            template_or_descriptor: TemplateOrDescriptor::Template("wsh-sortedmulti"),
            threshold: Some(2),
            n: 3,
            language: Some("english"),
            passphrase_used: false,
            privacy_preserving: false,
            per_slot: vec![
                slot_block(0, true, Some("aaaaaaaa"), Some("m/48'/0'/0'/2'")),
                slot_block(1, false, Some("bbbbbbbb"), Some("m/48'/0'/0'/2'")),
                slot_block(2, false, Some("cccccccc"), Some("m/48'/0'/0'/2'")),
            ],
            md1_chunk_set_id: "9999".into(),
        };
        let card = engraving_card_unified(&input);
        assert!(card.contains("# Threshold: 2 of 3\n"));
        assert!(card.contains("# Cosigners:\n"));
        assert!(card.contains("#   @0: ms1:abcde,mk1:abcde (aaaaaaaa @ m/48'/0'/0'/2')\n"));
        assert!(card.contains("#   @1: (no ms1; watch-only),mk1:abcde (bbbbbbbb @ m/48'/0'/0'/2')\n"));
        assert!(card.contains("#   @2: (no ms1; watch-only),mk1:abcde (cccccccc @ m/48'/0'/0'/2')\n"));
        assert!(card.contains("# Recovery: any 2 of 3 signing keys + md1 (template card).\n"));
    }

    #[test]
    fn unified_card_privacy_preserving_anonymizes_fingerprints() {
        let input = BundleInputForCard {
            network: "mainnet",
            template_or_descriptor: TemplateOrDescriptor::Template("wsh-sortedmulti"),
            threshold: Some(2),
            n: 2,
            language: None,
            passphrase_used: false,
            privacy_preserving: true,
            per_slot: vec![
                slot_block(0, true, None, Some("m/48'/0'/0'/2'")),
                slot_block(1, false, None, Some("m/48'/0'/0'/2'")),
            ],
            md1_chunk_set_id: "abcd".into(),
        };
        let card = engraving_card_unified(&input);
        assert!(card.contains("anon @ m/48'/0'/0'/2'"));
        assert!(!card.contains("5436d724"));
    }

    #[test]
    fn unified_card_descriptor_truncation_at_80_chars() {
        let long_d = "wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*,@3/<0;1>/*,@4/<0;1>/*,@5/<0;1>/*))";
        assert!(long_d.len() > 80);
        let input = BundleInputForCard {
            network: "mainnet",
            template_or_descriptor: TemplateOrDescriptor::Descriptor(long_d.into()),
            threshold: Some(2),
            n: 6,
            language: None,
            passphrase_used: false,
            privacy_preserving: false,
            per_slot: (0..6)
                .map(|i| slot_block(i, false, Some("ffffffff"), Some("m/48'/0'/0'/2'")))
                .collect(),
            md1_chunk_set_id: "1234".into(),
        };
        let card = engraving_card_unified(&input);
        // Long descriptor renders truncated with chars-total annotation.
        assert!(card.contains(&format!("({} chars total)", long_d.len())));
        assert!(card.contains("[md1: 1234]"));
    }

    // ---- v0.4.1 Phase J — VerifyCheck forensic fields ----

    #[test]
    fn verify_check_diff_offset_finds_first_byte_divergence() {
        assert_eq!(VerifyCheck::diff_offset("abcdef", "abcdef"), 6);
        assert_eq!(VerifyCheck::diff_offset("abcdef", "abcXef"), 3);
        assert_eq!(VerifyCheck::diff_offset("abc", "abcdef"), 3);
        assert_eq!(VerifyCheck::diff_offset("", "abc"), 0);
    }

    #[test]
    fn verify_check_default_is_ok_with_no_forensics() {
        let vc = VerifyCheck::default();
        assert!(vc.passed);
        assert!(vc.expected.is_none());
        assert!(vc.actual.is_none());
        assert!(vc.diff_byte_offset.is_none());
        assert!(vc.decode_error.is_none());
    }

    #[test]
    fn verify_check_serde_skip_omits_none_forensics_in_ok_path() {
        let vc = VerifyCheck {
            name: "ms1_entropy_match".into(),
            passed: true,
            detail: "ms1 byte-identical".into(),
            ..Default::default()
        };
        let json = serde_json::to_string(&vc).unwrap();
        assert!(!json.contains("expected"));
        assert!(!json.contains("actual"));
        assert!(!json.contains("diff_byte_offset"));
        assert!(!json.contains("decode_error"));
    }

    #[test]
    fn verify_check_serde_includes_populated_forensics_on_fail() {
        let vc = VerifyCheck {
            name: "ms1_entropy_match".into(),
            passed: false,
            detail: "expected ms1 bytes differ from supplied".into(),
            expected: Some("ms1abcde".into()),
            actual: Some("ms1abcXX".into()),
            diff_byte_offset: Some(6),
            decode_error: None,
        };
        let json = serde_json::to_string(&vc).unwrap();
        assert!(json.contains("\"expected\":\"ms1abcde\""));
        assert!(json.contains("\"actual\":\"ms1abcXX\""));
        assert!(json.contains("\"diff_byte_offset\":6"));
        assert!(!json.contains("decode_error"));
    }

    #[test]
    fn unified_card_tap_multisig_includes_hardware_caveat() {
        let input = BundleInputForCard {
            network: "mainnet",
            template_or_descriptor: TemplateOrDescriptor::Template("tr-sortedmulti-a"),
            threshold: Some(2),
            n: 3,
            language: None,
            passphrase_used: false,
            privacy_preserving: false,
            per_slot: vec![
                slot_block(0, true, Some("aaaaaaaa"), Some("m/86'/0'/0'")),
                slot_block(1, false, Some("bbbbbbbb"), Some("m/86'/0'/0'")),
                slot_block(2, false, Some("cccccccc"), Some("m/86'/0'/0'")),
            ],
            md1_chunk_set_id: "1234".into(),
        };
        let card = engraving_card_unified(&input);
        assert!(card.contains("HARDWARE WALLET CAVEAT"));
    }
}
