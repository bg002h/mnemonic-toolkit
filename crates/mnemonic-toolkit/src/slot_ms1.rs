//! `--slot @N.ms1=` decode + language-resolution helper (SPEC §2/§3).
//!
//! Single shared decode + wire-language policy site for the three hand-rolled
//! binding loops (template `resolve_slots`, `bundle_run_unified_descriptor`,
//! `verify_bundle` descriptor loop). Factored out per
//! `feedback_fix_the_class_hunt_for_second_instance` to avoid 4-way drift.

use crate::error::ToolkitError;
use crate::language::{wire_code_to_bip39, CliLanguage};
use zeroize::Zeroizing;

/// Decode + language-resolution result for an `ms1` slot value. Consumed by
/// the binding-loop Ms1 arms (template `resolve_slots`,
/// `bundle_run_unified_descriptor`, `verify_bundle` descriptor loop).
pub struct Ms1SlotResolution {
    pub entropy: Zeroizing<Vec<u8>>,
    /// Language to DERIVE the seed with (entropy→phrase→PBKDF2 seed).
    pub derive_language: bip39::Language,
    /// Language to stamp on the EMITTED card (drives entr-vs-mnem at synth);
    /// `None` ⇒ entr card (English), `Some(wire)` ⇒ mnem card. Feeds
    /// `ResolvedSlot.language`.
    pub emit_language: Option<bip39::Language>,
}

/// Decode an `ms1` slot value → entropy + derive/emit languages.
///
/// `flag_language` is `None` iff `--language` was absent (so a Some/None
/// distinction is possible — `--language` has no clap default that would
/// collapse the absent case to English).
///
/// Applies the wire-wins-refuse-on-conflict policy (SPEC §3): an entr ms1
/// has no intrinsic language and derives with `flag_language` (English
/// default), emitting an entr card; a mnem ms1 carries a wire language that
/// drives derivation and is preserved on the emitted card, and a disagreeing
/// `--language` is a HARD REFUSE (`SlotInputViolation{kind:"language-conflict"}`
/// → exit 2).
pub fn resolve_ms1_slot(
    value: &str,
    flag_language: Option<CliLanguage>,
    slot_index: u8,
) -> Result<Ms1SlotResolution, ToolkitError> {
    // mstring display-grouping (SPEC §3.2): strip separators so a grouped or
    // unbroken ms1 slot value both re-ingest (ms1 is single-string — full strip).
    let value = crate::display_grouping::strip_display_separators(value);
    let (_tag, payload) = ms_codec::decode(&value).map_err(ToolkitError::from)?;
    match payload {
        // No intrinsic language — derive with the flag (English default),
        // emit an entr card. Byte-identical to `@N.entropy=<hex>` (SPEC §3).
        ms_codec::Payload::Entr(bytes) => Ok(Ms1SlotResolution {
            entropy: Zeroizing::new(bytes),
            derive_language: flag_language.unwrap_or_default().into(),
            emit_language: None,
        }),
        // Wire language wins; a disagreeing `--language` is a hard refuse.
        ms_codec::Payload::Mnem {
            language: wire,
            entropy,
        } => {
            let wire_lang = wire_code_to_bip39(wire)?;
            if let Some(flag) = flag_language {
                let flag_lang: bip39::Language = flag.into();
                if flag_lang != wire_lang {
                    return Err(ToolkitError::SlotInputViolation {
                        kind: "language-conflict",
                        message: format!(
                            "slot @{slot_index}.ms1= carries wordlist language {wire_lang:?} \
                             but --language {flag_lang:?} was supplied; omit --language or \
                             set it to {wire_lang:?}"
                        ),
                    });
                }
            }
            Ok(Ms1SlotResolution {
                entropy: Zeroizing::new(entropy),
                derive_language: wire_lang,
                emit_language: Some(wire_lang),
            })
        }
        // ms-codec `Payload` is `#[non_exhaustive]` (SPEC §0); a threshold≠0
        // K-of-N share + reserved tags are rejected by `decode` itself, so on
        // the Ok path we only ever see Entr/Mnem — but the arm is mandatory.
        _ => Err(ToolkitError::BadInput(
            "ms1 slot decoded to an unknown payload kind".into(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::language::CliLanguage;

    /// 16 non-zero bytes for the fixtures.
    const E16: &[u8] = &[0x01u8; 16];
    /// Wire code: Japanese = 1.
    const WIRE_JAPANESE: u8 = 1;

    fn entr_ms1(entropy: &[u8]) -> String {
        ms_codec::encode(
            ms_codec::Tag::ENTR,
            &ms_codec::Payload::Entr(entropy.to_vec()),
        )
        .expect("ms_codec::encode entr")
    }

    fn mnem_ms1(entropy: &[u8], wire_lang: u8) -> String {
        ms_codec::encode(
            ms_codec::Tag::ENTR,
            &ms_codec::Payload::Mnem {
                language: wire_lang,
                entropy: entropy.to_vec(),
            },
        )
        .expect("ms_codec::encode mnem")
    }

    #[test]
    fn entr_ms1_no_flag_is_english_entropy_emit_none() {
        let v = entr_ms1(E16);
        let r = resolve_ms1_slot(&v, None, 0).expect("entr resolves");
        assert_eq!(&r.entropy[..], E16);
        assert_eq!(r.derive_language, bip39::Language::English);
        assert_eq!(r.emit_language, None);
    }

    #[test]
    fn mnem_japanese_no_flag_uses_wire_language_emit_some() {
        let v = mnem_ms1(E16, WIRE_JAPANESE);
        let r = resolve_ms1_slot(&v, None, 0).expect("mnem resolves");
        assert_eq!(&r.entropy[..], E16);
        assert_eq!(r.derive_language, bip39::Language::Japanese);
        assert_eq!(r.emit_language, Some(bip39::Language::Japanese));
    }

    #[test]
    fn mnem_flag_matching_wire_is_ok() {
        let v = mnem_ms1(E16, WIRE_JAPANESE);
        let r = resolve_ms1_slot(&v, Some(CliLanguage::Japanese), 0).expect("matching flag ok");
        assert_eq!(r.derive_language, bip39::Language::Japanese);
        assert_eq!(r.emit_language, Some(bip39::Language::Japanese));
    }

    #[test]
    fn mnem_flag_conflicting_wire_is_language_conflict() {
        let v = mnem_ms1(E16, WIRE_JAPANESE);
        // `Ms1SlotResolution` carries secret entropy and intentionally has no
        // `Debug`; match on the Err branch directly rather than `expect_err`.
        match resolve_ms1_slot(&v, Some(CliLanguage::English), 0) {
            Err(ToolkitError::SlotInputViolation { kind, .. }) => {
                assert_eq!(kind, "language-conflict");
            }
            Err(other) => panic!("expected SlotInputViolation language-conflict, got {other:?}"),
            Ok(_) => panic!("expected language-conflict, got Ok"),
        }
    }

    #[test]
    fn k_of_n_share_is_rejected() {
        // A 2-of-3 share decodes to `IsShareNotSingleString` (a decode-time
        // error), mapped to a ToolkitError via `From<ms_codec::Error>`.
        let shares = ms_codec::encode_shares(
            ms_codec::Tag::ENTR,
            ms_codec::Threshold::new(2).unwrap(),
            3,
            &ms_codec::Payload::Entr(E16.to_vec()),
        )
        .expect("encode_shares");
        let one_share = &shares[0];
        // Mapped through `ToolkitError::from(ms_codec::Error)` → MsCodec.
        match resolve_ms1_slot(one_share, None, 0) {
            Err(ToolkitError::MsCodec(_)) => {}
            Err(other) => panic!("expected MsCodec share-rejection error, got {other:?}"),
            Ok(_) => panic!("expected a share-rejection error, got Ok"),
        }
    }
}
