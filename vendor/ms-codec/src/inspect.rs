//! Structural inspection of an ms1 string for debugging / future ms-cli.

use crate::codex32::Codex32String;
use crate::consts::MNEM_PREFIX;
use crate::envelope;
use crate::error::Result;
use crate::tag::Tag;
use std::fmt;
use zeroize::Zeroizing;

/// Payload kind as decoded by `inspect()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectKind {
    /// `entr` — raw BIP-39 entropy (0x00 prefix byte, v0.1).
    Entr,
    /// `mnem` — BIP-39 mnemonic entropy with language tag (0x02 prefix byte, v0.2).
    Mnem,
    /// Any other prefix byte — future or invalid.
    Unknown,
}

impl InspectKind {
    /// Kebab-case name for text/JSON output.
    pub fn as_str(self) -> &'static str {
        match self {
            InspectKind::Entr => "entr",
            InspectKind::Mnem => "mnem",
            InspectKind::Unknown => "unknown",
        }
    }
}

/// Structural dump of a parsed ms1 string. `#[non_exhaustive]` per SPEC §10
/// — v0.2+ may add fields (share-index detail, threshold-layer hints,
/// derivation metadata).
///
/// `Debug` is **hand-rolled** (not derived) to redact `payload_bytes`
/// (RULE Z-DEBUG, cycle-15 Lane M): `Zeroizing<Vec<u8>>`'s own derived `Debug`
/// is non-redacting (forwards to `Vec`), so a derived `Debug` here would leak
/// the raw entropy bytes. The hand-roll surfaces every *structural* field
/// verbatim and renders the secret bytes as a length-only `[REDACTED; N]`
/// placeholder. See the `impl fmt::Debug` below.
#[derive(Clone)]
#[non_exhaustive]
pub struct InspectReport {
    /// Expected "ms" in v0.1.
    pub hrp: String,
    /// Expected 0 in v0.1.
    pub threshold: u8,
    /// The parsed type tag (id field).
    pub tag: Tag,
    /// Expected 's' in v0.1.
    pub share_index: char,
    /// 0x00 in v0.1 (reserved); becomes type discriminator in v0.2+.
    pub prefix_byte: u8,
    /// Payload bytes after the prefix byte. Wrapped in `Zeroizing` so the
    /// decoded secret entropy is scrubbed on drop (cycle-15 Lane M). The
    /// hand-rolled `Debug` (below) redacts it; `Deref<Target=Vec<u8>>` keeps
    /// read-only consumers (`.len()`, `hex::encode(&field)`) unchanged.
    pub payload_bytes: Zeroizing<Vec<u8>>,
    /// BCH verification result. True if the upstream codex32 parser accepted.
    pub checksum_valid: bool,
    /// Payload kind derived from the prefix byte.
    pub kind: InspectKind,
    /// For `kind == Mnem`: the language byte (index into `MNEM_LANGUAGE_NAMES`).
    /// `None` for all other kinds.
    pub language: Option<u8>,
}

impl fmt::Debug for InspectReport {
    /// Hand-rolled redacting `Debug` (RULE Z-DEBUG): surfaces every structural
    /// field verbatim and renders the secret `payload_bytes` as a length-only
    /// `[REDACTED; N]` placeholder so the raw entropy can never reach a debug
    /// dump. Mirrors the no-echo precedent on `crate::error::Error` (`error.rs`).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InspectReport")
            .field("hrp", &self.hrp)
            .field("threshold", &self.threshold)
            .field("tag", &self.tag)
            .field("share_index", &self.share_index)
            .field("prefix_byte", &self.prefix_byte)
            .field(
                "payload_bytes",
                &format_args!("[REDACTED; {} bytes]", self.payload_bytes.len()),
            )
            .field("checksum_valid", &self.checksum_valid)
            .field("kind", &self.kind)
            .field("language", &self.language)
            .finish()
    }
}

/// Inspect an ms1 string. Less strict than `decode()`: returns a report even
/// for strings that would fail decoder validity rules (e.g., wrong threshold,
/// reserved-not-emitted tag, non-zero prefix byte) — caller can examine the
/// fields to diagnose what's wrong. Still requires a valid BIP-93 parse.
pub fn inspect(s: &str) -> Result<InspectReport> {
    // `?` leverages From<crate::codex32::Error> for Error.
    let c = Codex32String::from_string(s.to_string())?;
    // Canonical lowercase wire copy (BIP-173 uppercase QR form folds here;
    // codex32 already rejected mixed case). Lowercasing loses no diagnostic
    // information — codex32 enforces whole-string uniform case, and the
    // "surface the raw observation" intent is about non-table tag VALUES.
    let s_owned = envelope::wire_string(&c);
    let fields = envelope::extract_wire_fields(&s_owned)?;

    // For tag construction in inspect we accept whatever bytes were on the wire
    // (alphabet-valid or not) — surfacing the raw observation is the point.
    let tag = match std::str::from_utf8(&fields.id_bytes) {
        Ok(t) => Tag::try_new(t).unwrap_or_else(|_| Tag::from_raw_bytes(fields.id_bytes)),
        Err(_) => Tag::from_raw_bytes(fields.id_bytes),
    };

    let payload_with_prefix = c.parts().data();
    let (prefix_byte, payload_bytes) = if payload_with_prefix.is_empty() {
        (0u8, Vec::new())
    } else {
        (payload_with_prefix[0], payload_with_prefix[1..].to_vec())
    };

    // Classify the payload kind and extract the language byte for mnem payloads.
    let (kind, language) = match prefix_byte {
        0x00 => (InspectKind::Entr, None),
        MNEM_PREFIX => {
            // payload_bytes = [lang_byte, entropy...]; language is the first byte.
            let lang = payload_bytes.first().copied();
            (InspectKind::Mnem, lang)
        }
        _ => (InspectKind::Unknown, None),
    };

    Ok(InspectReport {
        hrp: fields.hrp.to_string(),
        threshold: fields.threshold_byte - b'0', // ASCII to digit
        tag,
        share_index: fields.share_index_byte as char,
        prefix_byte,
        payload_bytes: Zeroizing::new(payload_bytes),
        checksum_valid: true, // if from_string accepted, BCH was valid
        kind,
        language,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{encode, payload::Payload};

    #[test]
    fn inspect_v01_entr_returns_expected_fields() {
        let entropy = vec![0xAAu8; 16];
        let s = encode::encode(Tag::ENTR, &Payload::Entr(entropy.clone())).unwrap();
        let r = inspect(&s).unwrap();
        assert_eq!(r.hrp, "ms");
        assert_eq!(r.threshold, 0);
        assert_eq!(r.tag, Tag::ENTR);
        assert_eq!(r.share_index, 's');
        assert_eq!(r.prefix_byte, 0x00);
        // I-1 (cycle-15 Lane M): `payload_bytes` is now `Zeroizing<Vec<u8>>`,
        // which has no `PartialEq<Vec<u8>>` (and `Deref` doesn't bridge `==`),
        // so deref the field rather than deriving `PartialEq` on the secret-
        // bearing struct.
        assert_eq!(*r.payload_bytes, entropy);
        assert!(r.checksum_valid);
    }

    #[test]
    fn inspect_returns_report_for_decoder_rejects() {
        // A non-zero-prefix string: decode() rejects, inspect() returns the report.
        let mut data = vec![0x01u8];
        data.extend_from_slice(&[0xAAu8; 16]);
        let c = Codex32String::from_seed("ms", 0, "entr", crate::codex32::Fe::S, &data).unwrap();
        let r = inspect(&c.to_string()).unwrap();
        assert_eq!(r.prefix_byte, 0x01); // would fail decode rule 8, inspect surfaces it
    }
}
