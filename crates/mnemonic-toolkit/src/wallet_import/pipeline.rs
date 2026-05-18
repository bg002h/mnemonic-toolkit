//! Concrete-keys → `@N`-placeholder adapter (inverse of
//! `wallet_export::pipeline::descriptor_to_bip388_wallet_policy`).
//!
//! Per SPEC §4.2 step 5: lex `[fp/path]xpub` occurrences out of a third-party
//! descriptor body, assign sequential `@N` placeholders preserving
//! declaration order, and produce `(ParsedKey, ParsedFingerprint)` pairs that
//! feed `parse_descriptor::parse_descriptor`.
//!
//! Per SPEC §4.3: ordering is the literal first-occurrence ordering in the
//! descriptor body. `sortedmulti(N, @0, @1, ..., @M)`'s lexicographic sort
//! at render time is orthogonal to this placeholder-binding step — the input
//! order is preserved at `@N` substitution; the render-time sort is a
//! `Display`-impl operation in miniscript that does not touch the
//! TLV-level ordering.

use crate::error::ToolkitError;
use crate::parse_descriptor::{ParsedFingerprint, ParsedKey};
use crate::slip0132::normalize_xpub_prefix;
use bitcoin::bip32::Xpub;
use regex::Regex;
use std::str::FromStr;
use std::sync::OnceLock;

/// SPEC §4.2 step 5 regex: `[fp/path]xpub`. Accepts SLIP-132 prefix
/// variants (`xpub|tpub|ypub|Ypub|zpub|Zpub|upub|Upub|vpub|Vpub`) — the xpub
/// string is canonicalized via `slip0132::normalize_xpub_prefix` before
/// payload extraction. The `path` capture is anchored by `/` + decimal digits
/// optionally followed by a hardened `'` mark.
///
/// Note: the literal regex below uses `[xtyzuvYZUV]` for the first prefix
/// char to match the 10 accepted SLIP-132 prefixes plus xpub/tpub. The
/// downstream `Xpub::from_str` accepts the neutralized form returned by
/// `normalize_xpub_prefix`; SLIP-132 mainnet variants neutralize to `xpub`,
/// testnet variants neutralize to `tpub`.
fn key_regex() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"\[([0-9a-fA-F]{8})((?:/\d+'?)+)\]([xtyzuvYZUV]pub[A-HJ-NP-Za-km-z1-9]+)")
            .expect("key_regex is a fixed string literal")
    })
}

/// Convert a descriptor body bearing concrete `[fp/path]xpub` keys into the
/// placeholder form `[fp/path]@N` + accompanying `(ParsedKey,
/// ParsedFingerprint)` pairs for `parse_descriptor::parse_descriptor`.
///
/// The replacement preserves the `[fp/path]` origin annotation so that the
/// downstream `lex_placeholders` + `resolve_placeholders` pipeline can
/// consume the `@N` syntax with origin-path metadata intact. The trailing
/// multipath / range suffix (e.g., `/<0;1>/*`) is preserved by virtue of
/// being outside the regex match.
pub(crate) fn concrete_keys_to_placeholders(
    descriptor: &str,
) -> Result<(String, Vec<ParsedKey>, Vec<ParsedFingerprint>), ToolkitError> {
    let re = key_regex();
    let mut keys: Vec<ParsedKey> = Vec::new();
    let mut fingerprints: Vec<ParsedFingerprint> = Vec::new();
    let mut placeholder_form = String::with_capacity(descriptor.len());
    let mut last_end = 0usize;
    let mut idx: u8 = 0;

    for cap in re.captures_iter(descriptor) {
        let m = cap.get(0).expect("group 0 is always present");
        placeholder_form.push_str(&descriptor[last_end..m.start()]);

        let fp_hex = cap.get(1).expect("group 1 captured").as_str();
        let path = cap.get(2).expect("group 2 captured").as_str();
        let xpub_str = cap.get(3).expect("group 3 captured").as_str();

        // SLIP-132 → neutral (xpub|tpub) canonicalization; rejects non-78-byte
        // base58check payloads and unknown version prefixes.
        let (neutral_xpub_str, _variant) = normalize_xpub_prefix(xpub_str)?;
        let xpub = Xpub::from_str(&neutral_xpub_str).map_err(|e| {
            ToolkitError::ImportWalletParse(format!(
                "import-wallet: bsms: parse error: xpub decode failed for key @{idx}: {e}"
            ))
        })?;

        let fp_bytes = parse_fp_hex(fp_hex).map_err(|e| {
            ToolkitError::ImportWalletParse(format!(
                "import-wallet: bsms: parse error: fingerprint decode failed for key @{idx}: {e}"
            ))
        })?;

        keys.push(ParsedKey {
            i: idx,
            payload: crate::synthesize::xpub_to_65(&xpub),
        });
        fingerprints.push(ParsedFingerprint {
            i: idx,
            fp: fp_bytes,
        });

        // Substitute the `[fp/path]xpub` literal with `@N[fp/path]`. The
        // `lex_placeholders` regex (parse_descriptor.rs:69) expects the
        // annotation to FOLLOW `@N` (capture group order: `@N[fp/path]
        // /<multipath>/*`), not precede it.
        placeholder_form.push('@');
        placeholder_form.push_str(&idx.to_string());
        placeholder_form.push('[');
        placeholder_form.push_str(fp_hex);
        placeholder_form.push_str(path);
        placeholder_form.push(']');

        last_end = m.end();
        idx = idx.checked_add(1).ok_or_else(|| {
            ToolkitError::ImportWalletParse(
                "import-wallet: bsms: parse error: more than 256 keys (placeholder @N overflow)"
                    .to_string(),
            )
        })?;
    }
    placeholder_form.push_str(&descriptor[last_end..]);

    if keys.is_empty() {
        return Err(ToolkitError::ImportWalletParse(
            "import-wallet: bsms: parse error: no [fp/path]xpub keys found in descriptor"
                .to_string(),
        ));
    }
    Ok((placeholder_form, keys, fingerprints))
}

fn parse_fp_hex(s: &str) -> Result<[u8; 4], String> {
    if s.len() != 8 {
        return Err(format!("fingerprint must be 8 hex chars; got {}", s.len()));
    }
    let mut out = [0u8; 4];
    for i in 0..4 {
        out[i] =
            u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).map_err(|e| format!("hex parse: {e}"))?;
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_keys_preserve_declaration_order() {
        // Synthetic testnet inputs (lifted from the user's flagship BSMS blob).
        // Replacement uses literal `[fp/path]@N` form for downstream lex.
        let desc = "wsh(thresh(2,pkh([704c7836/48'/1'/3'/2']tpubDEgS9fUEpucKatmvKAv21v8nViHxR6rsV7ohMWK4YjsWd4EWT3w8YzMgMEvNrDfsUANbid74WRFpr3Gym8UHBSLnqg6b1Lzvibw87cLSctC/<0;1>/*),s:pk([97139860/48'/1'/2'/2']tpubDFiXyf7zmBhQrSHoAQB6SmMpF3rfSihAxQGMdQUtZfE8HWHkWLLNLTiYpMzvHnFiTmuUSYieHUYv4tFguzmiHeDrYV8TtWGCWt5qpqox4w3/<0;1>/*),sln:older(32768)))";
        let (placeholder, keys, fps) = concrete_keys_to_placeholders(desc).unwrap();
        assert_eq!(keys.len(), 2);
        assert_eq!(fps.len(), 2);
        assert_eq!(keys[0].i, 0);
        assert_eq!(keys[1].i, 1);
        // Declaration order: @0 was the pkh slot, @1 was the s:pk slot.
        assert_eq!(fps[0].fp, [0x70, 0x4c, 0x78, 0x36]);
        assert_eq!(fps[1].fp, [0x97, 0x13, 0x98, 0x60]);
        // Origin annotation preserved (`@N[fp/path]` form matches
        // `lex_placeholders` regex at parse_descriptor.rs:69).
        assert!(placeholder.contains("@0[704c7836/48'/1'/3'/2']/<0;1>/*"));
        assert!(placeholder.contains("@1[97139860/48'/1'/2'/2']/<0;1>/*"));
    }

    #[test]
    fn no_keys_errors() {
        let desc = "wsh(thresh(2,older(144),older(288)))";
        let err = concrete_keys_to_placeholders(desc).unwrap_err();
        assert!(matches!(err, ToolkitError::ImportWalletParse(_)));
    }
}
