//! Descriptor wrapper-stripping. SPEC §2.
//!
//! Supports `wsh(M)` and `sh(wsh(M))`; refuses `tr(...)` and other
//! wrapper kinds. Single-leaf-tr support is tracked by FOLLOWUP
//! `compare-cost-single-leaf-tr-input` (filed in v0.27.0 cycle close;
//! deferred because the inner miniscript M is Tap-context with x-only
//! keys, requiring a tap→segv0 key+context conversion path not yet
//! built).

use std::str::FromStr;

use miniscript::descriptor::{DefiniteDescriptorKey, DescriptorPublicKey, ShInner};
use miniscript::{Descriptor, Miniscript, Segwitv0, Tap};

use super::translate::Translated;
use super::CompareCostError;

/// Strip the wrapper from a user-supplied descriptor string; return a
/// `Translated` ready for the enumerator (plus any advisory note). Concrete
/// hex pubkeys are preserved; no abstract labels are detected (those only
/// appear in `--miniscript` input).
pub fn translate_descriptor(input: &str) -> Result<(Translated, Option<String>), CompareCostError> {
    let desc = Descriptor::<DescriptorPublicKey>::from_str(input)
        .map_err(|e| CompareCostError::Parse(format!("descriptor parse: {e}")))?;

    // Materialize wildcards to a concrete derivation index (cost is identical
    // across child indices for a given descriptor shape; index 0 is fine).
    let definite: Descriptor<DefiniteDescriptorKey> = if desc.has_wildcard() {
        desc.derive_at_index(0)
            .map_err(|e| CompareCostError::Parse(format!("derivation: {e}")))?
    } else {
        // No wildcards: TryFrom converts cleanly without "index" indirection.
        Descriptor::<DefiniteDescriptorKey>::try_from(desc)
            .map_err(|e| CompareCostError::Parse(format!("definite-key conversion: {e}")))?
    };

    match definite {
        Descriptor::Wsh(wsh) => {
            let m_segv0 = wsh.into_inner();
            translated_from_segv0(input, m_segv0).map(|t| (t, None))
        }
        Descriptor::Sh(sh) => match sh.into_inner() {
            ShInner::Wsh(wsh) => {
                let m_segv0 = wsh.into_inner();
                translated_from_segv0(input, m_segv0).map(|t| (t, None))
            }
            ShInner::Wpkh(_) | ShInner::Ms(_) => Err(
                CompareCostError::UnsupportedWrapper("sh(non-wsh)".to_string()),
            ),
        },
        Descriptor::Tr(_) => Err(CompareCostError::UnsupportedWrapper(
            "tr-input deferred — see FOLLOWUP `compare-cost-single-leaf-tr-input`; supply --miniscript for now".to_string(),
        )),
        Descriptor::Bare(_) | Descriptor::Pkh(_) | Descriptor::Wpkh(_) => Err(
            CompareCostError::UnsupportedWrapper("pkh / wpkh / bare not a miniscript-wrapping question".to_string()),
        ),
    }
}

/// Build a Translated from an already-parsed Segwitv0 miniscript by
/// re-serializing the keys for Tap context (compressed-secp 33B → x-only 32B
/// via the standard "drop the 02/03 parity byte" projection).
fn translated_from_segv0(
    _original_input: &str,
    m_segv0: Miniscript<DefiniteDescriptorKey, Segwitv0>,
) -> Result<Translated, CompareCostError> {
    let segv0_str = m_segv0.to_string();
    let tap_str = segv0_string_to_tap_string(&segv0_str);
    let m_tap: Miniscript<DefiniteDescriptorKey, Tap> = Miniscript::from_str(&tap_str)
        .map_err(|e| CompareCostError::ContextIncompat {
            valid_in: "Segwitv0",
            invalid_in: "Tap",
            detail: format!("{e}"),
        })?;

    // SPEC §5: `extracted_miniscript` is the inner M (post-wrapper-strip),
    // not the original full descriptor string. Use the segv0-context
    // canonical serialization.
    Ok(Translated {
        extracted: segv0_str,
        segv0: m_segv0,
        tap: m_tap,
        concrete_keys: true,
        labels: Vec::new(),
        label_pubkeys: Vec::new(),
    })
}

/// Convert a Segwitv0-context miniscript string (compressed-secp 66-char hex
/// pubkeys, `multi`/`sortedmulti`) into a Tap-context miniscript string
/// (x-only 64-char hex, `multi_a`/`sortedmulti_a`).
///
/// Compressed pubkey → x-only: drop the leading 2-hex-char parity byte
/// (`02ab…cd` → `ab…cd`). This is the standard BIP-340 projection (x-only is
/// just the x-coordinate of the secp point).
fn segv0_string_to_tap_string(segv0: &str) -> String {
    let mut out = String::with_capacity(segv0.len());
    let bytes = segv0.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // Detect a 66-char compressed-pubkey hex run starting with 02/03.
        if i + 66 <= bytes.len() {
            let candidate = &bytes[i..i + 66];
            let starts_with_parity = matches!(candidate[0..2], [b'0', b'2'] | [b'0', b'3']);
            let all_hex = candidate.iter().all(|b| b.is_ascii_hexdigit());
            let prev_is_ident = i > 0 && (bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'_');
            let next_is_ident = i + 66 < bytes.len()
                && (bytes[i + 66].is_ascii_alphanumeric() || bytes[i + 66] == b'_');
            if starts_with_parity && all_hex && !prev_is_ident && !next_is_ident {
                // Append the x-only form (drop first 2 hex chars).
                out.push_str(std::str::from_utf8(&candidate[2..]).unwrap());
                i += 66;
                continue;
            }
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    // Now rewrite multi/sortedmulti → multi_a/sortedmulti_a.
    let out = replace_fragment(&out, "sortedmulti(", "sortedmulti_a(");
    replace_fragment(&out, "multi(", "multi_a(")
}

fn replace_fragment(haystack: &str, find: &str, repl: &str) -> String {
    let mut result = String::with_capacity(haystack.len());
    let bytes = haystack.as_bytes();
    let find_b = find.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if i + find_b.len() <= bytes.len() && &bytes[i..i + find_b.len()] == find_b {
            let before_ok = i == 0
                || !(bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'_');
            if before_ok {
                result.push_str(repl);
                i += find_b.len();
                continue;
            }
        }
        result.push(bytes[i] as char);
        i += 1;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drops_parity_byte_on_compressed_pubkey() {
        let segv0 = "pk(0288facd06da4a3b8b6c7e0a8f6c8c6e0a8f6c8c6e0a8f6c8c6e0a8f6c8c6e0a8f)";
        let tap = segv0_string_to_tap_string(segv0);
        assert_eq!(
            tap,
            "pk(88facd06da4a3b8b6c7e0a8f6c8c6e0a8f6c8c6e0a8f6c8c6e0a8f6c8c6e0a8f)"
        );
    }

    #[test]
    fn rewrites_multi_to_multi_a() {
        let input = "multi(2,02aa00aa00aa00aa00aa00aa00aa00aa00aa00aa00aa00aa00aa00aa00aa00aaaa,03bb00bb00bb00bb00bb00bb00bb00bb00bb00bb00bb00bb00bb00bb00bb00bbbb)";
        let out = segv0_string_to_tap_string(input);
        assert!(out.starts_with("multi_a("), "expected multi_a prefix: {out}");
    }
}
